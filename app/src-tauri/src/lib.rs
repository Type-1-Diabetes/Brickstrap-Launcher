//! Brickstrap: startet den offiziellen Roblox-Client nur von außen.
//! Keine Injection, kein Speicherzugriff, keine Änderung an der Client-Datei.

mod account;
mod auth;
mod activity;
mod basicsettings;
mod discord;
mod fastflags;
mod friends;
mod install;
mod launch;
mod paths;
mod region;
mod roblox_api;
mod state;
mod update;

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
            // Aus „Downloads“ o. Ä. gestartet: an den festen Platz kopieren und von dort neu starten.
            if install::sicherstellen() {
                std::process::exit(0);
            }
            let data_dir = app.path().app_data_dir()?;
            region::init(&data_dir);
            app.manage(state::AppState::new(data_dir));
            update::start(app.handle().clone());

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

            // Rückfall: Die Oberfläche zeigt das Fenster nach dem ersten Zeichnen selbst.
            // Scheitert sie beim Laden, wäre das Fenster sonst für immer unsichtbar.
            let handle = app.handle().clone();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_secs(4));
                if let Some(w) = handle.get_webview_window("main") {
                    if !w.is_visible().unwrap_or(true) && !w.is_minimized().unwrap_or(false) {
                        let _ = w.show();
                    }
                }
            });
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
            vollbild,
            update::update_check,
            update::update_install,
            discord::discord_seite,
            roblox_api::home_feed,
            roblox_api::search,
            roblox_api::game_details,
            roblox_api::universe_of_place,
            roblox_api::thumbnails,
            roblox_api::servers,
            roblox_api::game_media,
            roblox_api::server_avatars,
            roblox_api::game_votes,
            roblox_api::game_extras,
            roblox_api::search_suggest,
            account::account_get,
            friends::friends_online,
            auth::auth_status,
            auth::auth_login,
            auth::auth_logout,
            launch::pin_cover_set,
            launch::roblox_status,
            launch::roblox_update,
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

/// F11: Vollbild des Hauptfensters an/aus. Gibt den neuen Zustand zurück.
#[tauri::command]
fn vollbild(window: tauri::WebviewWindow) -> Result<bool, String> {
    let an = !window.is_fullscreen().map_err(|e| format!("UNBEKANNT|{e}"))?;
    window.set_fullscreen(an).map_err(|e| format!("UNBEKANNT|{e}"))?;
    Ok(an)
}
