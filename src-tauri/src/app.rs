use std::{sync::Arc, time::Duration};

use serde_json::{json, Value};
use tauri::{
    image::Image,
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu},
    tray::TrayIconBuilder,
    AppHandle, Manager, RunEvent, State, WebviewUrl, WebviewWindowBuilder,
};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt as _};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
use tauri_plugin_store::StoreExt;

use crate::{feature::FeatureManager, features};

type Mgr = Arc<FeatureManager>;

const TRAY_ON: &[u8] = include_bytes!("../icons/tray-on.png");
const TRAY_OFF: &[u8] = include_bytes!("../icons/tray-off.png");
const STORE: &str = "settings.json";

fn refresh_icon(app: &AppHandle, m: &FeatureManager) {
    if let Some(tray) = app.tray_by_id("main") {
        let bytes = if m.any_enabled() && !m.is_paused() { TRAY_ON } else { TRAY_OFF };
        if let Ok(img) = Image::from_bytes(bytes) {
            let _ = tray.set_icon(Some(img));
            let _ = tray.set_icon_as_template(true);
        }
        let _ = tray.set_title(if m.is_paused() { Some("⏸") } else { None });
    }
}

fn persist(app: &AppHandle, m: &FeatureManager) {
    if let Ok(s) = app.store(STORE) {
        s.set("enabled", json!(m.enabled_ids()));
        for f in m.features() {
            s.set(format!("config:{}", f.id()), f.config());
        }
        let _ = s.save();
    }
}

fn restore(app: &AppHandle, m: &FeatureManager) {
    let Ok(s) = app.store(STORE) else { return };
    for f in m.features() {
        if let Some(v) = s.get(format!("config:{}", f.id())) {
            let _ = f.set_config(v);
        }
    }
    if let Some(Value::Array(ids)) = s.get("enabled") {
        for id in ids.iter().filter_map(Value::as_str) {
            if let Err(e) = m.toggle(id) {
                tracing::warn!("restore {id}: {e}");
            }
        }
    }
}

fn toggle(app: &AppHandle, m: &FeatureManager, id: &str) {
    if let Err(e) = m.toggle(id) {
        tracing::warn!("toggle {id}: {e}");
    }
    persist(app, m);
    refresh_icon(app, m);
}

fn until_tomorrow() -> Duration {
    use chrono::{Duration as D, Local};
    let now = Local::now();
    let midnight = (now.date_naive() + D::days(1)).and_hms_opt(0, 0, 0).and_then(|t| t.and_local_timezone(Local).earliest());
    midnight.and_then(|t| (t - now).to_std().ok()).unwrap_or(Duration::from_secs(8 * 3600))
}

fn open_settings(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("settings") {
        let _ = w.show();
        let _ = w.set_focus();
        return;
    }
    let _ = WebviewWindowBuilder::new(app, "settings", WebviewUrl::App("index.html".into()))
        .title("Fidget Settings")
        .inner_size(420.0, 520.0)
        .build();
}

#[tauri::command]
fn get_state(m: State<Mgr>) -> Value {
    let features: Vec<Value> = m
        .features()
        .iter()
        .map(|f| json!({"id": f.id(), "name": f.name(), "enabled": m.is_enabled(f.id()), "fields": f.config_schema()}))
        .collect();
    json!({"paused": m.is_paused(), "features": features})
}

#[tauri::command]
fn set_enabled(app: AppHandle, m: State<Mgr>, id: String, enabled: bool) -> bool {
    if m.is_enabled(&id) != enabled {
        toggle(&app, &m, &id);
    }
    m.is_enabled(&id)
}

#[tauri::command]
fn set_config(app: AppHandle, m: State<Mgr>, id: String, config: Value) -> Result<(), String> {
    m.set_config(&id, config).map_err(|e| e.to_string())?;
    persist(&app, &m);
    Ok(())
}

pub fn run() {
    tracing_subscriber::fmt::init();
    let manager: Mgr = Arc::new(FeatureManager::new(features::register_all()));
    let shortcuts: Vec<(Shortcut, &'static str)> = manager
        .features()
        .iter()
        .filter_map(|f| Some((f.shortcut()?.parse().ok()?, f.id())))
        .collect();
    let (sm, sc) = (manager.clone(), shortcuts.clone());

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| open_settings(app)))
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, None))
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app, shortcut, ev| {
                    if ev.state() == ShortcutState::Pressed {
                        if let Some((_, id)) = sc.iter().find(|(s, _)| s == shortcut) {
                            toggle(app, &sm, id);
                        }
                    }
                })
                .build(),
        )
        .manage(manager.clone())
        .invoke_handler(tauri::generate_handler![get_state, set_enabled, set_config])
        .setup(move |app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            for (s, _) in &shortcuts {
                if let Err(e) = app.global_shortcut().register(*s) {
                    tracing::warn!("shortcut {s}: {e}");
                }
            }
            restore(app.handle(), &manager);

            // Menu is generated from the registered features.
            let menu = Menu::new(app)?;
            menu.append(&MenuItem::with_id(app, "title", "Fidget", false, None::<&str>)?)?;
            for f in manager.features() {
                menu.append(&PredefinedMenuItem::separator(app)?)?;
                let label = match f.shortcut() {
                    Some(s) => format!("{}   ({})", f.name(), s.replace("CmdOrControl", "⌘/Ctrl")),
                    None => f.name().to_string(),
                };
                menu.append(&CheckMenuItem::with_id(app, format!("toggle:{}", f.id()), label, true, manager.is_enabled(f.id()), None::<&str>)?)?;
                menu.append(&MenuItem::with_id(app, format!("status:{}", f.id()), f.status(), false, None::<&str>)?)?;
            }
            menu.append(&PredefinedMenuItem::separator(app)?)?;
            menu.append(&Submenu::with_items(
                app,
                "Pause All",
                true,
                &[
                    &MenuItem::with_id(app, "pause:900", "for 15 minutes", true, None::<&str>)?,
                    &MenuItem::with_id(app, "pause:3600", "for 1 hour", true, None::<&str>)?,
                    &MenuItem::with_id(app, "pause:tomorrow", "until tomorrow", true, None::<&str>)?,
                    &PredefinedMenuItem::separator(app)?,
                    &MenuItem::with_id(app, "resume", "Resume", false, None::<&str>)?,
                ],
            )?)?;
            menu.append(&MenuItem::with_id(app, "settings", "Settings…", true, Some("CmdOrControl+,"))?)?;
            menu.append(&CheckMenuItem::with_id(app, "autostart", "Launch at Login", true, app.autolaunch().is_enabled().unwrap_or(false), None::<&str>)?)?;
            menu.append(&PredefinedMenuItem::separator(app)?)?;
            menu.append(&MenuItem::with_id(app, "quit", "Quit Fidget", true, Some("CmdOrControl+Q"))?)?;

            let m = manager.clone();
            TrayIconBuilder::with_id("main")
                .icon(Image::from_bytes(TRAY_OFF)?)
                .icon_as_template(true)
                .menu(&menu)
                .on_menu_event(move |app, ev| {
                    let id = ev.id().as_ref();
                    match id {
                        "quit" => app.exit(0),
                        "settings" => open_settings(app),
                        "resume" => m.resume(),
                        "autostart" => {
                            let al = app.autolaunch();
                            let _ = if al.is_enabled().unwrap_or(false) { al.disable() } else { al.enable() };
                        }
                        "pause:tomorrow" => m.pause(until_tomorrow()),
                        _ => {
                            if let Some(fid) = id.strip_prefix("toggle:") {
                                toggle(app, &m, fid);
                            } else if let Some(secs) = id.strip_prefix("pause:").and_then(|s| s.parse().ok()) {
                                m.pause(Duration::from_secs(secs));
                            }
                        }
                    }
                    refresh_icon(app, &m);
                })
                .build(app)?;
            refresh_icon(app.handle(), &manager);

            // Keep check marks, status lines and pause state in sync.
            let (handle, m, menu) = (app.handle().clone(), manager.clone(), menu.clone());
            tauri::async_runtime::spawn(async move {
                loop {
                    m.tick();
                    for f in m.features() {
                        if let Some(i) = menu.get(&format!("toggle:{}", f.id())).and_then(|k| k.as_check_menuitem().cloned()) {
                            let _ = i.set_checked(m.is_enabled(f.id()));
                        }
                        if let Some(i) = menu.get(&format!("status:{}", f.id())).and_then(|k| k.as_menuitem().cloned()) {
                            let _ = i.set_text(f.status());
                        }
                    }
                    if let Some(i) = menu.get("autostart").and_then(|k| k.as_check_menuitem().cloned()) {
                        let _ = i.set_checked(handle.autolaunch().is_enabled().unwrap_or(false));
                    }
                    refresh_icon(&handle, &m);
                    tokio::time::sleep(Duration::from_secs(2)).await;
                }
            });
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("failed to build Fidget")
        .run(|_, ev| {
            // Closing the settings window must not quit the tray app.
            if let RunEvent::ExitRequested { api, code: None, .. } = ev {
                api.prevent_exit();
            }
        });
}
