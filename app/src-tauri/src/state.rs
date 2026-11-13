//! Gemeinsamer Zustand der App: Datenordner, Einstellungen, HTTP-Client.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::{de::DeserializeOwned, Deserialize, Serialize};

/// Ein angepinntes Spiel.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pinned {
    pub universe_id: u64,
    pub place_id: u64,
    pub name: String,
    /// Unix-Zeit in Sekunden
    pub pinned_at: u64,
    /// Eigenes Titelbild: https-Adresse eines Roblox-Bilds oder eigenes Bild als data:-URL
    #[serde(default)]
    pub cover: Option<String>,
}

/// Einstellungen der App (nicht die von Roblox).
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Config {
    pub pinned: Vec<Pinned>,
    /// Discord-Status anzeigen
    pub discord: bool,
    /// Eigene Discord-Anwendung (öffentliche ID, kein Geheimnis). Leer = Discord aus.
    pub discord_app_id: String,
    /// Knopf „Server beitreten“ im Discord-Status zeigen
    pub discord_join_button: bool,
    /// Fenster schließen = in den Infobereich
    pub close_to_tray: bool,
    /// Nach dem Start von Roblox das Fenster verstecken
    pub hide_on_launch: bool,
    /// Serverliste ohne volle Server
    pub hide_full_servers: bool,
    /// Brickstrap öffnen, wenn Roblox beendet wird
    pub show_on_exit: bool,
}

/// Discord-Anwendung von Brickstrap (öffentliche ID, kein Geheimnis).
/// Leer, bis der Projektinhaber die App im Discord Developer Portal angelegt hat.
pub const DISCORD_APP_ID: &str = "";

impl Config {
    /// Eigene ID aus den Einstellungen, sonst die von Brickstrap.
    pub fn discord_id(&self) -> String {
        let own = self.discord_app_id.trim();
        if own.is_empty() { DISCORD_APP_ID.to_string() } else { own.to_string() }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            pinned: Vec::new(),
            discord: true,
            discord_app_id: String::new(),
            discord_join_button: true,
            close_to_tray: true,
            hide_on_launch: true,
            hide_full_servers: false,
            show_on_exit: true,
        }
    }
}

pub struct AppState {
    pub data_dir: PathBuf,
    pub config: Mutex<Config>,
    pub http: reqwest::Client,
    /// Zufällige ID je Programmstart, verlangt von Such- und Explore-API.
    pub session_id: String,
}

impl AppState {
    pub fn new(data_dir: PathBuf) -> Self {
        let _ = std::fs::create_dir_all(&data_dir);
        let config = read_json(&data_dir.join("config.json")).unwrap_or_default();
        let http = reqwest::Client::builder()
            .user_agent(concat!("Brickstrap/", env!("CARGO_PKG_VERSION")))
            .timeout(std::time::Duration::from_secs(15))
            .pool_idle_timeout(std::time::Duration::from_secs(90))
            .build()
            .expect("HTTP-Client");
        Self {
            data_dir,
            config: Mutex::new(config),
            http,
            session_id: uuid::Uuid::new_v4().to_string(),
        }
    }

    pub fn config(&self) -> Config {
        self.config.lock().map(|c| c.clone()).unwrap_or_default()
    }

    pub fn save_config(&self, cfg: Config) -> Result<(), String> {
        write_json(&self.data_dir.join("config.json"), &cfg)?;
        if let Ok(mut c) = self.config.lock() {
            *c = cfg;
        }
        Ok(())
    }

    pub fn file(&self, name: &str) -> PathBuf {
        self.data_dir.join(name)
    }
}

/// JSON-Datei lesen; fehlt sie oder ist sie kaputt: `None`.
pub fn read_json<T: DeserializeOwned>(path: &Path) -> Option<T> {
    let text = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&text).ok()
}

/// JSON-Datei atomar schreiben (erst temporäre Datei, dann umbenennen).
pub fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let text = serde_json::to_string_pretty(value).map_err(|e| e.to_string())?;
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, text).map_err(|e| format!("Schreiben fehlgeschlagen: {e}"))?;
    std::fs::rename(&tmp, path).map_err(|e| format!("Schreiben fehlgeschlagen: {e}"))
}

/// Unix-Zeit in Sekunden.
pub fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
