//! Brickstrap: startet den offiziellen Roblox-Client nur von außen.
//! Keine Injection, kein Speicherzugriff, keine Änderung an der Client-Datei.

mod activity;
mod basicsettings;
mod discord;
mod fastflags;
mod launch;
mod paths;
mod roblox_api;
mod state;

use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{Manager, WindowEvent};

fn show_main(app: &tauri::AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.unminimize();
        let _ = w.show();
        let _ = w.set_focus();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // Zweiter Start holt das vorhandene Fenster nach vorn.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| show_main(app)))
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            app.manage(state::AppState::new(data_dir));

            // Symbol im Infobereich
            let open = MenuItem::with_id(app, "open", "Brickstrap öffnen", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Beenden", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&open, &quit])?;
            TrayIconBuilder::with_id("main")
                .icon(app.default_window_icon().cloned().expect("Icon"))
                .tooltip("Brickstrap")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, e| match e.id.as_ref() {
                    "open" => show_main(app),
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, e| {
                    if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = e {
                        show_main(tray.app_handle());
                    }
                })
                .build(app)?;

            activity::start(app.handle().clone());
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let to_tray = window
                    .app_handle()
                    .try_state::<state::AppState>()
                    .map(|s| s.config().close_to_tray)
                    .unwrap_or(true);
                if to_tray {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            roblox_api::home_feed,
            roblox_api::search,
            roblox_api::game_details,
            roblox_api::universe_of_place,
            roblox_api::thumbnails,
            roblox_api::servers,
            launch::roblox_status,
            launch::launch,
            launch::invite_link,
            launch::config_get,
            launch::discord_builtin,
            launch::config_set,
            launch::pin_set,
            launch::pin_order,
            fastflags::flags_get,
            fastflags::flags_set,
            fastflags::flags_presets,
            basicsettings::basic_get,
            basicsettings::basic_set,
            activity::activity_get,
            activity::playtime_get,
        ])
        .run(tauri::generate_context!())
        .expect("Brickstrap konnte nicht starten");
}
