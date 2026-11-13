//! Roblox starten (nur von außen: Deeplink oder Player-Programm) und App-Einstellungen.

use serde::Serialize;
use tauri::{AppHandle, Manager, State};

use crate::paths;
use crate::roblox_api::get_json;
use crate::state::{now, AppState, Config, Pinned};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RobloxStatus {
    pub installed: bool,
    pub running: bool,
    /// Name des neuesten Versionsordners, z. B. `version-1a2b3c`
    pub version: Option<String>,
}

#[tauri::command]
pub fn roblox_status() -> RobloxStatus {
    let dir = paths::player_version_dirs().into_iter().next();
    RobloxStatus {
        installed: dir.is_some() || paths::protocol_registered(),
        running: paths::roblox_running(),
        version: dir.and_then(|d| d.file_name().map(|n| n.to_string_lossy().into_owned())),
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    /// Neuester installierter Versionsordner
    pub installed: Option<String>,
    /// Neueste Version laut Roblox (`version-…`), `None` wenn der Abruf scheitert
    pub latest: Option<String>,
    pub update_available: bool,
}

const UPDATE_CACHE_SECS: u64 = 600;

/// Steht ein Roblox-Update an? Vergleicht die Versionsordner mit der Angabe von Roblox
/// (dokumentierter Endpunkt, ohne Login). Das Ergebnis gilt 10 Minuten.
/// Ohne Versionsordner (Store-Version) oder ohne Netz: kein Update gemeldet.
#[tauri::command]
pub async fn roblox_update(state: State<'_, AppState>) -> Result<UpdateInfo, String> {
    let dirs: Vec<String> = paths::player_version_dirs()
        .iter()
        .filter_map(|d| d.file_name().map(|n| n.to_string_lossy().into_owned()))
        .collect();
    let installed = dirs.first().cloned();
    if dirs.is_empty() {
        return Ok(UpdateInfo { installed, latest: None, update_available: false });
    }
    let cached = state
        .latest_version
        .lock()
        .ok()
        .and_then(|c| c.clone())
        .filter(|(t, _)| now().saturating_sub(*t) < UPDATE_CACHE_SECS)
        .map(|(_, v)| v);
    let latest = match cached {
        Some(v) => Some(v),
        None => {
            let v = get_json(&state, "https://clientsettingscdn.roblox.com/v2/client-version/WindowsPlayer")
                .await
                .ok()
                .and_then(|v| v.get("clientVersionUpload")?.as_str().map(String::from))
                .filter(|v| v.starts_with("version-"));
            if let (Some(v), Ok(mut c)) = (&v, state.latest_version.lock()) {
                *c = Some((now(), v.clone()));
            }
            v
        }
    };
    let update_available = latest.as_ref().is_some_and(|l| !dirs.iter().any(|d| d.eq_ignore_ascii_case(l)));
    Ok(UpdateInfo { installed, latest, update_available })
}

/// Deeplink für einen öffentlichen Server oder einen beliebigen Server.
pub fn deeplink(place_id: u64, job_id: Option<&str>) -> String {
    match job_id.filter(|j| is_job_id(j)) {
        Some(j) => format!("roblox://experiences/start?placeId={place_id}&gameInstanceId={j}"),
        None => format!("roblox://experiences/start?placeId={place_id}"),
    }
}

/// JobIds sind GUIDs. Alles andere wird nicht in den Link übernommen.
fn is_job_id(s: &str) -> bool {
    s.len() == 36 && s.chars().all(|c| c.is_ascii_hexdigit() || c == '-')
}

/// Roblox starten. Ohne `place_id` öffnet sich die Roblox-App, sonst das Spiel
/// (mit `job_id` ein bestimmter Server). Vorher werden die FastFlags geschrieben.
#[tauri::command]
pub fn launch(app: AppHandle, state: State<'_, AppState>, place_id: Option<u64>, job_id: Option<String>) -> Result<(), String> {
    let has_dirs = !paths::player_version_dirs().is_empty();
    let installed = has_dirs || paths::protocol_registered();
    if has_dirs {
        if let Err(e) = crate::fastflags::apply_before_launch(&state) {
            // Ein Fehler hier darf den Start nicht verhindern.
            eprintln!("FastFlags: {e}");
        }
    }
    match place_id {
        Some(p) => {
            let link = deeplink(p, job_id.as_deref());
            tauri_plugin_opener::open_url(link, None::<&str>).map_err(|e| format!("START|{e}"))?;
        }
        None => match paths::player_exe() {
            Some(exe) => {
                std::process::Command::new(exe)
                    .arg("--app")
                    .spawn()
                    .map_err(|e| format!("START|{e}"))?;
            }
            // Ohne Versionsordner (z. B. Store-Version) startet der Link die Roblox-App.
            None if installed => {
                tauri_plugin_opener::open_url("roblox://", None::<&str>).map_err(|e| format!("START|{e}"))?;
            }
            None => return Err("NICHT_INSTALLIERT|Roblox ist nicht installiert.".to_string()),
        },
    }
    // Ohne gefundene Roblox-Installation öffnet Windows evtl. nur „App auswählen“:
    // dann bleibt Brickstrap sichtbar.
    if installed && state.config().hide_on_launch {
        if let Some(w) = app.get_webview_window("main") {
            let _ = w.hide();
        }
    }
    Ok(())
}

/// Einladungslink, der auch ohne Brickstrap funktioniert (öffnet die Roblox-Webseite).
/// Mit `job_id` führt er auf genau diesen Server (Format wie im Discord-Knopf, unbestätigt).
#[tauri::command]
pub fn invite_link(place_id: u64, job_id: Option<String>) -> String {
    match job_id.as_deref().filter(|j| is_job_id(j)) {
        Some(j) => format!("https://www.roblox.com/games/start?placeId={place_id}&gameInstanceId={j}"),
        None => format!("https://www.roblox.com/games/{place_id}"),
    }
}

// ---------- App-Einstellungen und Anpinnen ----------

#[tauri::command]
pub fn config_get(state: State<'_, AppState>) -> Config {
    state.config()
}

/// Hat Brickstrap eine eigene Discord-Anwendung eingebaut?
#[tauri::command]
pub fn discord_builtin() -> bool {
    !crate::state::DISCORD_APP_ID.is_empty()
}

#[tauri::command]
pub fn config_set(state: State<'_, AppState>, config: Config) -> Result<Config, String> {
    // Angepinnte Spiele nur über pin_set ändern.
    let mut next = config;
    next.pinned = state.config().pinned;
    next.discord_app_id = next.discord_app_id.trim().chars().filter(char::is_ascii_digit).collect();
    state.save_config(next.clone())?;
    crate::discord::config_changed(&state);
    Ok(next)
}

#[tauri::command]
pub fn pin_set(state: State<'_, AppState>, universe_id: u64, place_id: u64, name: String, pinned: bool) -> Result<Vec<Pinned>, String> {
    let mut cfg = state.config();
    cfg.pinned.retain(|p| p.universe_id != universe_id);
    if pinned {
        cfg.pinned.insert(0, Pinned { universe_id, place_id, name, pinned_at: now(), cover: None });
    }
    state.save_config(cfg.clone())?;
    Ok(cfg.pinned)
}

/// Titelbild eines angepinnten Spiels setzen (`None` = Standardbild von Roblox).
/// Erlaubt sind Roblox-Bilder (https://…rbxcdn.com) und eigene Bilder als data:image/…
#[tauri::command]
pub fn pin_cover_set(state: State<'_, AppState>, universe_id: u64, cover: Option<String>) -> Result<Vec<Pinned>, String> {
    if let Some(c) = &cover {
        let roblox = c.starts_with("https://") && c.split('/').nth(2).is_some_and(|h| h.ends_with(".rbxcdn.com"));
        let eigen = c.starts_with("data:image/") && c.len() < 3_000_000;
        if !roblox && !eigen {
            return Err("BILD|Dieses Bild lässt sich nicht verwenden.".into());
        }
    }
    let mut cfg = state.config();
    let p = cfg.pinned.iter_mut().find(|p| p.universe_id == universe_id).ok_or("Spiel ist nicht angepinnt.")?;
    p.cover = cover;
    state.save_config(cfg.clone())?;
    Ok(cfg.pinned)
}

/// Reihenfolge der angepinnten Spiele ändern (Liste von Universe-IDs).
#[tauri::command]
pub fn pin_order(state: State<'_, AppState>, order: Vec<u64>) -> Result<Vec<Pinned>, String> {
    let mut cfg = state.config();
    cfg.pinned.sort_by_key(|p| order.iter().position(|id| *id == p.universe_id).unwrap_or(usize::MAX));
    state.save_config(cfg.clone())?;
    Ok(cfg.pinned)
}
