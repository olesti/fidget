use std::{sync::Arc, time::Duration};

use tauri::{
    image::Image,
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    AppHandle,
};

use crate::{feature::FeatureManager, features};

const TRAY_ON: &[u8] = include_bytes!("../icons/tray-on.png");
const TRAY_OFF: &[u8] = include_bytes!("../icons/tray-off.png");

fn refresh_icon(app: &AppHandle, m: &FeatureManager) {
    if let Some(tray) = app.tray_by_id("main") {
        let bytes = if m.any_enabled() { TRAY_ON } else { TRAY_OFF };
        if let Ok(img) = Image::from_bytes(bytes) {
            let _ = tray.set_icon(Some(img));
            let _ = tray.set_icon_as_template(true);
        }
    }
}

pub fn run() {
    tracing_subscriber::fmt::init();
    let manager = Arc::new(FeatureManager::new(features::register_all()));

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|_, _, _| {}))
        .setup(move |app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            // Menu is generated from the registered features.
            let menu = Menu::new(app)?;
            menu.append(&MenuItem::with_id(app, "title", "Fidget", false, None::<&str>)?)?;
            for f in manager.features() {
                menu.append(&PredefinedMenuItem::separator(app)?)?;
                menu.append(&CheckMenuItem::with_id(
                    app,
                    format!("toggle:{}", f.id()),
                    f.name(),
                    true,
                    false,
                    None::<&str>,
                )?)?;
                menu.append(&MenuItem::with_id(app, format!("status:{}", f.id()), f.status(), false, None::<&str>)?)?;
            }
            menu.append(&PredefinedMenuItem::separator(app)?)?;
            menu.append(&MenuItem::with_id(app, "quit", "Quit Fidget", true, Some("CmdOrCtrl+Q"))?)?;

            let m = manager.clone();
            TrayIconBuilder::with_id("main")
                .icon(Image::from_bytes(TRAY_OFF)?)
                .icon_as_template(true)
                .menu(&menu)
                .on_menu_event(move |app, ev| {
                    let id = ev.id().as_ref();
                    if id == "quit" {
                        app.exit(0);
                    } else if let Some(fid) = id.strip_prefix("toggle:") {
                        if let Err(e) = m.toggle(fid) {
                            tracing::warn!("toggle {fid}: {e}");
                        }
                        refresh_icon(app, &m); // check mark is re-synced by the ticker below
                    }
                })
                .build(app)?;

            // Keep check marks and live status lines in sync.
            let (handle, m, menu) = (app.handle().clone(), manager.clone(), menu.clone());
            tauri::async_runtime::spawn(async move {
                loop {
                    for f in m.features() {
                        if let Some(i) = menu.get(&format!("toggle:{}", f.id())).and_then(|k| k.as_check_menuitem().cloned()) {
                            let _ = i.set_checked(m.is_enabled(f.id()));
                        }
                        if let Some(i) = menu.get(&format!("status:{}", f.id())).and_then(|k| k.as_menuitem().cloned()) {
                            let _ = i.set_text(f.status());
                        }
                    }
                    refresh_icon(&handle, &m);
                    tokio::time::sleep(Duration::from_secs(2)).await;
                }
            });
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("failed to build Fidget")
        .run(|_, _| {});
}
