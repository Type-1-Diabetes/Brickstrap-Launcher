//! Anmeldung mit dem Roblox-Konto.
//!
//! Brickstrap öffnet die echte Login-Seite von roblox.com in einem eigenen Fenster (privater Modus,
//! nichts bleibt im WebView gespeichert) und übernimmt danach das Cookie `.ROBLOSECURITY`.
//! Das Cookie ist ein Generalschlüssel zum Konto: Es liegt nur im Rust-Teil, verschlüsselt im
//! Windows-Anmeldeinformations-Manager, und geht nur an `*.roblox.com`. Nie an die Oberfläche.

use std::sync::Mutex;
use std::time::Duration;

use serde::Serialize;
use serde_json::Value;
use tauri::{AppHandle, Emitter, Manager, State, WebviewUrl, WebviewWindowBuilder};

use crate::state::AppState;

const DIENST: &str = "app.brickstrap.launcher";
const KONTO: &str = "roblosecurity";
const LOGIN_URL: &str = "https://www.roblox.com/login";

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthStatus {
    pub logged_in: bool,
    pub user_id: Option<u64>,
    pub name: Option<String>,
    pub display_name: Option<String>,
}

static LETZTER: Mutex<Option<AuthStatus>> = Mutex::new(None);

fn eintrag() -> Option<keyring::Entry> {
    keyring::Entry::new(DIENST, KONTO).ok()
}

/// Nutzer-ID der bestätigten Anmeldung, falls bekannt.
pub fn angemeldet_id() -> Option<u64> {
    LETZTER.lock().ok()?.as_ref()?.user_id
}

/// Gespeichertes Cookie (nur für Anfragen an roblox.com im Rust-Teil).
pub fn cookie() -> Option<String> {
    eintrag()?.get_password().ok().filter(|c| !c.is_empty())
}

async fn pruefen(state: &AppState, cookie: &str) -> Option<AuthStatus> {
    let r = state
        .http
        .get("https://users.roblox.com/v1/users/authenticated")
        .header("Cookie", format!(".ROBLOSECURITY={cookie}"))
        .send()
        .await
        .ok()?;
    if !r.status().is_success() {
        return None;
    }
    let v: Value = r.json().await.ok()?;
    Some(AuthStatus {
        logged_in: true,
        user_id: v.get("id").and_then(Value::as_u64),
        name: v.get("name").and_then(Value::as_str).map(String::from),
        display_name: v.get("displayName").and_then(Value::as_str).map(String::from),
    })
}

const ABGEMELDET: AuthStatus = AuthStatus { logged_in: false, user_id: None, name: None, display_name: None };

/// Ist jemand angemeldet? Prüft das Cookie einmal je Programmstart bei Roblox.
#[tauri::command]
pub async fn auth_status(state: State<'_, AppState>) -> Result<AuthStatus, String> {
    if let Some(s) = LETZTER.lock().ok().and_then(|g| g.clone()) {
        return Ok(s);
    }
    let Some(c) = cookie() else { return Ok(ABGEMELDET) };
    let status = match pruefen(&state, &c).await {
        Some(s) => s,
        None => {
            // Netz weg: nicht abmelden, nur nicht bestätigen. Abgelaufen: Eintrag löschen.
            if state.http.get("https://users.roblox.com").send().await.is_ok() {
                if let Some(e) = eintrag() {
                    let _ = e.delete_credential();
                }
                ABGEMELDET
            } else {
                AuthStatus { logged_in: true, user_id: None, name: None, display_name: None }
            }
        }
    };
    if let Ok(mut g) = LETZTER.lock() {
        *g = Some(status.clone());
    }
    Ok(status)
}

/// Login-Fenster öffnen. Sobald Roblox das Cookie setzt, wird es übernommen und das
/// Ereignis `auth` gesendet.
#[tauri::command]
pub async fn auth_login(app: AppHandle) -> Result<(), String> {
    if let Some(w) = app.get_webview_window("login") {
        let _ = w.set_focus();
        return Ok(());
    }
    let url = LOGIN_URL.parse().map_err(|e| format!("{e}"))?;
    let fenster = WebviewWindowBuilder::new(&app, "login", WebviewUrl::External(url))
        .title("Bei Roblox anmelden – Brickstrap")
        .inner_size(480.0, 720.0)
        .center()
        .incognito(true)
        .build()
        .map_err(|e| format!("Login-Fenster: {e}"))?;

    // Cookies nie auf dem Hauptthread lesen (unter Windows sonst Verklemmung).
    std::thread::spawn(move || {
        let ziel: tauri::Url = match "https://www.roblox.com".parse() {
            Ok(u) => u,
            Err(_) => return,
        };
        for _ in 0..900 {
            std::thread::sleep(Duration::from_millis(800));
            let Some(w) = app.get_webview_window("login") else { return };
            let gefunden = w
                .cookies_for_url(ziel.clone())
                .ok()
                .and_then(|cs| cs.into_iter().find(|c| c.name() == ".ROBLOSECURITY").map(|c| c.value().to_string()));
            if let Some(c) = gefunden.filter(|c| c.len() > 50) {
                let state = app.state::<AppState>();
                let status = tauri::async_runtime::block_on(pruefen(&state, &c));
                if let Some(s) = status {
                    if let Some(e) = eintrag() {
                        let _ = e.set_password(&c);
                    }
                    if let Ok(mut g) = LETZTER.lock() {
                        *g = Some(s.clone());
                    }
                    let _ = w.close();
                    let _ = app.emit("auth", &s);
                    if let Some(m) = app.get_webview_window("main") {
                        let _ = m.show();
                        let _ = m.set_focus();
                    }
                    return;
                }
            }
        }
        let _ = fenster.close();
    });
    Ok(())
}

/// Abmelden: Cookie aus dem Anmeldeinformations-Manager löschen.
#[tauri::command]
pub fn auth_logout(app: AppHandle) -> Result<(), String> {
    if let Some(e) = eintrag() {
        let _ = e.delete_credential();
    }
    if let Ok(mut g) = LETZTER.lock() {
        *g = Some(ABGEMELDET);
    }
    let _ = app.emit("auth", &ABGEMELDET);
    Ok(())
}
