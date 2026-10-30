use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};

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
use tauri_plugin_updater::UpdaterExt;

use crate::{
    feature::{Feature, FeatureManager, Permission},
    features, i18n,
    i18n::t,
    platform,
};

type Mgr = Arc<FeatureManager>;
/// feature id -> (parsed shortcut, accelerator string). Missing = no shortcut.
type Shortcuts = Arc<Mutex<HashMap<&'static str, (Shortcut, String)>>>;

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

fn store_set(app: &AppHandle, key: &str, v: Value) {
    if let Ok(s) = app.store(STORE) {
        s.set(key, v);
        let _ = s.save();
    }
}

fn restore(app: &AppHandle, m: &FeatureManager, sc: &Shortcuts) {
    let Ok(s) = app.store(STORE) else { return };
    if let Some(Value::String(l)) = s.get("lang") {
        i18n::set_lang(&l);
    }
    for f in m.features() {
        if let Some(v) = s.get(format!("config:{}", f.id())) {
            let _ = f.set_config(v);
        }
        if let Some(Value::String(a)) = s.get(format!("shortcut:{}", f.id())) {
            match a.parse::<Shortcut>() {
                Ok(sh) if !a.is_empty() => sc.lock().unwrap().insert(f.id(), (sh, a)),
                _ => sc.lock().unwrap().remove(f.id()),
            };
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

/// `tab` pre-selects a settings tab (the UI reads `window.__tab`).
fn open_settings(app: &AppHandle, tab: &str) {
    if let Some(w) = app.get_webview_window("settings") {
        let _ = w.eval(format!("window.__tab='{tab}';window.dispatchEvent(new Event('fidget-tab'))"));
        let _ = w.show();
        let _ = w.set_focus();
        return;
    }
    let _ = WebviewWindowBuilder::new(app, "settings", WebviewUrl::App("index.html".into()))
        .initialization_script(format!("window.__tab='{tab}'"))
        .title("Fidget")
        .inner_size(520.0, 600.0)
        .build();
}

fn feature_label(f: &dyn Feature, sc: &Shortcuts) -> String {
    match sc.lock().unwrap().get(f.id()) {
        Some((_, a)) => format!("{}   ({})", f.name(), a.replace("CmdOrControl", "⌘/Ctrl")),
        None => f.name().to_string(),
    }
}

#[tauri::command]
fn get_state(app: AppHandle, m: State<Mgr>, sc: State<Shortcuts>) -> Value {
    let features: Vec<Value> = m
        .features()
        .iter()
        .map(|f| {
            let shortcut = sc.lock().unwrap().get(f.id()).map(|s| s.1.clone()).unwrap_or_default();
            json!({"id": f.id(), "name": f.name(), "enabled": m.is_enabled(f.id()), "fields": f.config_schema(), "shortcut": shortcut})
        })
        .collect();
    json!({
        "paused": m.is_paused(),
        "lang": i18n::lang(),
        "strings": i18n::dump(),
        "autostart": app.autolaunch().is_enabled().unwrap_or(false),
        "version": app.package_info().version.to_string(),
        "features": features,
    })
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

#[tauri::command]
fn set_lang(app: AppHandle, lang: String) {
    i18n::set_lang(&lang);
    store_set(&app, "lang", json!(i18n::lang()));
}

#[tauri::command]
fn set_autostart(app: AppHandle, enabled: bool) -> Result<(), String> {
    let al = app.autolaunch();
    (if enabled { al.enable() } else { al.disable() }).map_err(|e| e.to_string())
}

/// Empty `accel` clears the shortcut. On failure the previous shortcut stays registered.
#[tauri::command]
fn set_shortcut(app: AppHandle, m: State<Mgr>, sc: State<Shortcuts>, id: String, accel: String) -> Result<(), String> {
    let fid = m.features().iter().find(|f| f.id() == id).map(|f| f.id()).ok_or("unknown feature")?;
    let new = if accel.is_empty() { None } else { Some((accel.parse::<Shortcut>().map_err(|e| e.to_string())?, accel.clone())) };
    let gs = app.global_shortcut();
    let mut map = sc.lock().unwrap();
    let old = map.get(fid).map(|o| o.0);
    if let Some(o) = old {
        let _ = gs.unregister(o);
    }
    if let Some((s, _)) = &new {
        if let Err(e) = gs.register(*s) {
            if let Some(o) = old {
                let _ = gs.register(o);
            }
            return Err(e.to_string());
        }
    }
    match new {
        Some(n) => map.insert(fid, n),
        None => map.remove(fid),
    };
    drop(map);
    store_set(&app, &format!("shortcut:{fid}"), json!(accel));
    Ok(())
}

/// Returns the new version if an update is available.
#[tauri::command]
async fn check_update(app: AppHandle) -> Result<Option<String>, String> {
    let u = app.updater().map_err(|e| e.to_string())?.check().await.map_err(|e| e.to_string())?;
    Ok(u.map(|u| u.version))
}

#[tauri::command]
async fn install_update(app: AppHandle) -> Result<(), String> {
    let u = app.updater().map_err(|e| e.to_string())?.check().await.map_err(|e| e.to_string())?.ok_or("no update")?;
    u.download_and_install(|_, _| {}, || {}).await.map_err(|e| e.to_string())?;
    app.restart()
}

#[tauri::command]
fn get_permissions(m: State<Mgr>) -> Value {
    let mut all: Vec<Permission> = vec![];
    for p in m.features().iter().flat_map(|f| f.required_permissions()) {
        if !all.contains(&p) {
            all.push(p);
        }
    }
    all.into_iter().map(|p| json!({"id": p.id(), "granted": platform::permission_granted(p)})).collect()
}

#[tauri::command]
fn request_permission(id: String) {
    if id == Permission::Accessibility.id() {
        platform::request_permission(Permission::Accessibility);
    }
}

#[tauri::command]
fn open_logs(app: AppHandle) -> Result<(), String> {
    let dir = app.path().app_log_dir().map_err(|e| e.to_string())?;
    let _ = std::fs::create_dir_all(&dir);
    let opener = if cfg!(target_os = "macos") { "open" } else if cfg!(target_os = "windows") { "explorer" } else { "xdg-open" };
    std::process::Command::new(opener).arg(dir).spawn().map(|_| ()).map_err(|e| e.to_string())
}

pub fn run() {
    i18n::init();
    let manager: Mgr = Arc::new(FeatureManager::new(features::register_all()));
    let shortcuts: Shortcuts = Arc::new(Mutex::new(
        manager.features().iter().filter_map(|f| Some((f.id(), (f.shortcut()?.parse().ok()?, f.shortcut()?.to_string())))).collect(),
    ));
    let (hm, hs) = (manager.clone(), shortcuts.clone());

    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_single_instance::init(|app, _, _| open_settings(app, "features")))
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, None))
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app, shortcut, ev| {
                    if ev.state() == ShortcutState::Pressed {
                        let id = hs.lock().unwrap().iter().find(|(_, (s, _))| s == shortcut).map(|(id, _)| *id);
                        if let Some(id) = id {
                            toggle(app, &hm, id);
                        }
                    }
                })
                .build(),
        )
        .manage(manager.clone())
        .manage(shortcuts.clone())
        .invoke_handler(tauri::generate_handler![
            get_state,
            set_enabled,
            set_config,
            set_lang,
            set_autostart,
            set_shortcut,
            get_permissions,
            request_permission,
            open_logs,
            check_update,
            install_update
        ])
        .setup(move |app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            restore(app.handle(), &manager, &shortcuts);
            for (s, _) in shortcuts.lock().unwrap().values() {
                if let Err(e) = app.global_shortcut().register(*s) {
                    tracing::warn!("shortcut {s}: {e}");
                }
            }

            // Menu is generated from the registered features; `texts` re-localizes it on language change.
            let mut texts: Vec<(MenuItem<_>, &'static str)> = vec![];
            let mut item = |id: &str, key: &'static str, enabled: bool, accel: Option<&str>| -> tauri::Result<MenuItem<_>> {
                let i = MenuItem::with_id(app, id, t(key), enabled, accel)?;
                texts.push((i.clone(), key));
                Ok(i)
            };
            let resume = item("resume", "tray.resume", false, None)?;
            let pause_items = [
                item("pause:900", "tray.p15", true, None)?,
                item("pause:3600", "tray.p60", true, None)?,
                item("pause:tomorrow", "tray.ptomorrow", true, None)?,
            ];
            let settings = item("settings", "tray.settings", true, Some("CmdOrControl+,"))?;
            let update = item("update", "tray.update", true, None)?;
            let quit = item("quit", "tray.quit", true, Some("CmdOrControl+Q"))?;

            let menu = Menu::new(app)?;
            menu.append(&MenuItem::with_id(app, "title", "Fidget", false, None::<&str>)?)?;
            for f in manager.features() {
                menu.append(&PredefinedMenuItem::separator(app)?)?;
                menu.append(&CheckMenuItem::with_id(app, format!("toggle:{}", f.id()), feature_label(&**f, &shortcuts), true, manager.is_enabled(f.id()), None::<&str>)?)?;
                menu.append(&MenuItem::with_id(app, format!("status:{}", f.id()), f.status(), false, None::<&str>)?)?;
            }
            menu.append(&PredefinedMenuItem::separator(app)?)?;
            let pause = Submenu::with_items(
                app,
                t("tray.pause"),
                true,
                &[&pause_items[0], &pause_items[1], &pause_items[2], &PredefinedMenuItem::separator(app)?, &resume],
            )?;
            menu.append(&pause)?;
            menu.append(&settings)?;
            menu.append(&update)?;
            let autostart = CheckMenuItem::with_id(app, "autostart", t("tray.autostart"), true, app.autolaunch().is_enabled().unwrap_or(false), None::<&str>)?;
            menu.append(&autostart)?;
            menu.append(&PredefinedMenuItem::separator(app)?)?;
            menu.append(&quit)?;

            let m = manager.clone();
            TrayIconBuilder::with_id("main")
                .icon(Image::from_bytes(TRAY_OFF)?)
                .icon_as_template(true)
                .menu(&menu)
                .on_menu_event(move |app, ev| {
                    let id = ev.id().as_ref();
                    match id {
                        "quit" => app.exit(0),
                        "settings" => open_settings(app, "features"),
                        "update" => open_settings(app, "update"),
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

            // Keep check marks, labels, status lines and pause state in sync.
            let (handle, m, sc) = (app.handle().clone(), manager.clone(), shortcuts.clone());
            tauri::async_runtime::spawn(async move {
                loop {
                    m.tick();
                    for f in m.features() {
                        if let Some(i) = menu.get(&format!("toggle:{}", f.id())).and_then(|k| k.as_check_menuitem().cloned()) {
                            let _ = i.set_checked(m.is_enabled(f.id()));
                            let _ = i.set_text(feature_label(&**f, &sc));
                        }
                        if let Some(i) = menu.get(&format!("status:{}", f.id())).and_then(|k| k.as_menuitem().cloned()) {
                            let _ = i.set_text(f.status());
                        }
                    }
                    for (i, key) in &texts {
                        let _ = i.set_text(t(key));
                    }
                    let _ = pause.set_text(t("tray.pause"));
                    let _ = resume.set_enabled(m.is_paused());
                    let _ = autostart.set_text(t("tray.autostart"));
                    let _ = autostart.set_checked(handle.autolaunch().is_enabled().unwrap_or(false));
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
