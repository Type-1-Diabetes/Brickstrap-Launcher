//! Updates von Brickstrap selbst über GitHub-Releases (Repo öffentlich, kein Token).
//!
//! Ablauf: neuestes Release abfragen → exe und `.sha256` laden → Prüfsumme vergleichen →
//! laufende exe in `<name>.old` umbenennen (das erlaubt Windows) → neue an ihre Stelle →
//! neu starten. Die `.old` löscht der nächste Start.

use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Emitter, Manager, State};

use crate::state::AppState;

const LATEST: &str = "https://api.github.com/repos/Type-1-Diabetes/Brickstrap-Launcher/releases/latest";
/// Automatische Suche: beim Start und danach alle 3 Stunden.
const ABSTAND: Duration = Duration::from_secs(3 * 60 * 60);

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub version: String,
    pub current: String,
    /// Text des Releases (Markdown, ungekürzt)
    pub notes: String,
    pub size: u64,
    #[serde(skip)]
    exe_url: String,
    #[serde(skip)]
    sha_url: String,
}

static GEFUNDEN: Mutex<Option<(Instant, Option<UpdateInfo>)>> = Mutex::new(None);

/// "v0.0.12" → (0, 0, 12)
fn teile(v: &str) -> Option<(u64, u64, u64)> {
    let mut it = v.trim().trim_start_matches('v').split(['.', '-']).map(|p| p.parse::<u64>());
    Some((it.next()?.ok()?, it.next()?.ok()?, it.next()?.ok()?))
}

fn neuer(remote: &str, lokal: &str) -> bool {
    matches!((teile(remote), teile(lokal)), (Some(r), Some(l)) if r > l)
}

async fn abfragen(state: &AppState) -> Result<Option<UpdateInfo>, String> {
    let r = state
        .http
        .get(LATEST)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .map_err(|_| "NETZ|GitHub ist nicht erreichbar.".to_string())?;
    if r.status().as_u16() == 404 {
        return Ok(None); // noch kein Release
    }
    if !r.status().is_success() {
        return Err(format!("HTTP|{}", r.status().as_u16()));
    }
    let v: Value = r.json().await.map_err(|_| "FORMAT|Antwort von GitHub unlesbar.".to_string())?;
    let tag = v.get("tag_name").and_then(Value::as_str).unwrap_or("");
    let current = env!("CARGO_PKG_VERSION").to_string();
    if !neuer(tag, &current) {
        return Ok(None);
    }
    let assets = v.get("assets").and_then(Value::as_array).cloned().unwrap_or_default();
    let name = |a: &Value| a.get("name").and_then(Value::as_str).unwrap_or("").to_string();
    let url = |a: &Value| a.get("browser_download_url").and_then(Value::as_str).unwrap_or("").to_string();
    let Some(exe) = assets.iter().find(|a| name(a).ends_with(".exe")) else { return Ok(None) };
    let Some(sha) = assets.iter().find(|a| name(a) == format!("{}.sha256", name(exe))) else { return Ok(None) };
    Ok(Some(UpdateInfo {
        version: tag.trim_start_matches('v').to_string(),
        current,
        notes: v.get("body").and_then(Value::as_str).unwrap_or("").to_string(),
        size: exe.get("size").and_then(Value::as_u64).unwrap_or(0),
        exe_url: url(exe),
        sha_url: url(sha),
    }))
}

/// Gibt es eine neuere Version? `force` fragt sofort, sonst höchstens alle 10 Minuten.
#[tauri::command]
pub async fn update_check(state: State<'_, AppState>, force: Option<bool>) -> Result<Option<UpdateInfo>, String> {
    if !force.unwrap_or(false) {
        if let Some((t, info)) = GEFUNDEN.lock().ok().and_then(|g| g.clone()) {
            if t.elapsed() < Duration::from_secs(600) {
                return Ok(info);
            }
        }
    }
    let info = abfragen(&state).await?;
    if let Ok(mut g) = GEFUNDEN.lock() {
        *g = Some((Instant::now(), info.clone()));
    }
    Ok(info)
}

#[derive(Clone, Serialize)]
struct Fortschritt {
    done: u64,
    total: u64,
}

/// Lädt das Update, prüft es und startet die neue Version. Kehrt nur bei Fehlern zurück.
#[tauri::command]
pub async fn update_install(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let info = match GEFUNDEN.lock().ok().and_then(|g| g.clone()).and_then(|(_, i)| i) {
        Some(i) => i,
        None => abfragen(&state).await?.ok_or("UNBEKANNT|Kein Update gefunden.")?,
    };
    let exe = std::env::current_exe().map_err(|e| format!("DATEI|{e}"))?;
    let neu: PathBuf = exe.with_extension("new");
    let alt: PathBuf = exe.with_extension("old");

    // Prüfsumme zuerst (klein)
    let sha_text = state
        .http
        .get(&info.sha_url)
        .send()
        .await
        .map_err(|_| "NETZ|Download abgebrochen.".to_string())?
        .text()
        .await
        .map_err(|_| "NETZ|Download abgebrochen.".to_string())?;
    let erwartet = sha_text.split_whitespace().next().unwrap_or("").to_lowercase();
    if erwartet.len() != 64 {
        return Err("FORMAT|Prüfsumme des Updates fehlt.".into());
    }

    // exe in Stücken laden, Fortschritt melden
    let mut r = state
        .http
        .get(&info.exe_url)
        .timeout(Duration::from_secs(600))
        .send()
        .await
        .map_err(|_| "NETZ|Download abgebrochen.".to_string())?;
    if !r.status().is_success() {
        return Err(format!("HTTP|{}", r.status().as_u16()));
    }
    let total = r.content_length().unwrap_or(info.size);
    let mut daten: Vec<u8> = Vec::with_capacity(total as usize);
    let mut hasher = Sha256::new();
    let mut letzte = Instant::now();
    while let Some(stueck) = r.chunk().await.map_err(|_| "NETZ|Download abgebrochen.".to_string())? {
        hasher.update(&stueck);
        daten.extend_from_slice(&stueck);
        if letzte.elapsed() > Duration::from_millis(100) {
            letzte = Instant::now();
            let _ = app.emit("update-progress", Fortschritt { done: daten.len() as u64, total });
        }
    }
    let _ = app.emit("update-progress", Fortschritt { done: daten.len() as u64, total });
    let ist: String = hasher.finalize().iter().map(|b| format!("{b:02x}")).collect();
    if ist != erwartet {
        return Err("FORMAT|Prüfsumme passt nicht. Das Update wurde verworfen.".into());
    }

    // Tauschen
    std::fs::write(&neu, &daten).map_err(|e| format!("SCHREIBGESCHUETZT|{e}"))?;
    let _ = std::fs::remove_file(&alt);
    std::fs::rename(&exe, &alt).map_err(|e| {
        let _ = std::fs::remove_file(&neu);
        format!("SCHREIBGESCHUETZT|{e}")
    })?;
    if let Err(e) = std::fs::rename(&neu, &exe) {
        let _ = std::fs::rename(&alt, &exe); // zurück
        return Err(format!("SCHREIBGESCHUETZT|{e}"));
    }

    crate::install::version_merken(&exe, &info.version);
    neu_starten(&exe)?;
    app.exit(0);
    Ok(())
}

/// Startet die neue exe, nachdem diese Instanz beendet ist. Sofort gestartet würde sie
/// wegen der Einzelinstanz nur die alte nach vorn holen und sich schließen.
#[cfg(windows)]
pub(crate) fn neu_starten(exe: &std::path::Path) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    // `ping` wartet etwa 2 Sekunden, `start` löst die exe vom cmd-Fenster.
    std::process::Command::new("cmd")
        .raw_arg(format!("/C ping -n 3 127.0.0.1 >nul & start \"\" \"{}\"", exe.display()))
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("START|{e}"))
}

#[cfg(not(windows))]
pub(crate) fn neu_starten(exe: &std::path::Path) -> Result<(), String> {
    std::process::Command::new(exe).spawn().map(|_| ()).map_err(|e| format!("START|{e}"))
}

/// Beim Start: alte exe aufräumen, dann regelmäßig nach Updates suchen (abschaltbar).
pub fn start(app: AppHandle) {
    if let Ok(exe) = std::env::current_exe() {
        let alt = exe.with_extension("old");
        // Die alte Instanz kann noch kurz laufen: ein paar Versuche.
        std::thread::spawn(move || {
            for _ in 0..10 {
                if !alt.exists() || std::fs::remove_file(&alt).is_ok() {
                    break;
                }
                std::thread::sleep(Duration::from_secs(1));
            }
        });
    }
    tauri::async_runtime::spawn(async move {
        // Start nicht bremsen
        tokio::time::sleep(Duration::from_secs(5)).await;
        loop {
            let state = app.state::<AppState>();
            if state.config().update_check {
                if let Ok(Some(info)) = update_check(state, Some(true)).await {
                    let _ = app.emit("update", info);
                }
            }
            tokio::time::sleep(ABSTAND).await;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn versionen() {
        assert!(neuer("v0.0.10", "0.0.9"));
        assert!(!neuer("v0.0.7", "0.0.7"));
        assert!(!neuer("v0.0.6", "0.0.7"));
        assert!(neuer("v0.1.0", "0.0.99"));
    }
}
