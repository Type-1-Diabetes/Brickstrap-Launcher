//! Discord Rich Presence. Ein eigener Thread hält die IPC-Verbindung zum Discord-Programm.
//!
//! Ohne Discord oder ohne eigene App-ID (Einstellung `discord_app_id`) passiert nichts.
//! Verbinden scheitert still und wird höchstens alle 15 Sekunden neu versucht.

use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use discord_rich_presence::{activity as drp, DiscordIpc, DiscordIpcClient};
use serde_json::Value;
use tauri::{AppHandle, Manager};

use crate::activity::Activity;
use crate::state::{AppState, Config};

const RETRY: Duration = Duration::from_secs(15);
const TICK: Duration = Duration::from_secs(5);
const MAX_TEXT: usize = 128;
/// Name des Bilds unter „Rich Presence → Art Assets“ der Discord-Anwendung.
/// Fehlt es dort, zeigt Discord an dieser Stelle nichts.
const LOGO: &str = "brickstrap";

// ---------- BloxstrapRPC ----------

/// Felder, die ein Spiel per `SetRichPresence` setzt. Bilder aus der Nachricht ignorieren
/// wir bewusst: Ein Spielskript könnte sonst beliebige Adressen setzen, die Discord dann
/// abruft (Tracking über den Discord-Proxy). Brickstrap zeigt immer das Spielicon.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RpcOverride {
    pub details: Option<String>,
    pub state: Option<String>,
    /// Unix-Zeit in Sekunden
    pub time_start: Option<i64>,
    pub time_end: Option<i64>,
}

fn clip(s: &str) -> Option<String> {
    let t: String = s.trim().chars().take(MAX_TEXT).collect();
    // Discord verlangt mindestens 2 Zeichen.
    if t.chars().count() < 2 {
        None
    } else {
        Some(t)
    }
}

/// Sekunden oder Millisekunden (Einheit der Nachricht ist unbestätigt) in Sekunden.
fn to_secs(n: i64) -> i64 {
    if n > 100_000_000_000 {
        n / 1000
    } else {
        n
    }
}

impl RpcOverride {
    /// Übernimmt die Felder aus `data`. `"<reset>"` (oder `null`) setzt ein Feld zurück.
    pub fn merge(&mut self, data: &Value) {
        let text = |v: &Value| -> Option<String> {
            match v.as_str() {
                Some("<reset>") | None => None,
                Some(s) => clip(s),
            }
        };
        let time = |v: &Value| -> Option<i64> {
            match v.as_i64() {
                Some(n) if n > 0 => Some(to_secs(n)),
                _ => None,
            }
        };
        if let Some(v) = data.get("details") {
            self.details = text(v);
        }
        if let Some(v) = data.get("state") {
            self.state = text(v);
        }
        if let Some(v) = data.get("timeStart") {
            self.time_start = time(v);
        }
        if let Some(v) = data.get("timeEnd") {
            self.time_end = time(v);
        }
    }

    pub fn is_empty(&self) -> bool {
        *self == RpcOverride::default()
    }
}

// ---------- Nachrichten an den Thread ----------

/// Was der Nutzer gerade im Launcher ansieht (solange kein Spiel läuft).
#[derive(Clone, Debug, Default)]
pub struct Seite {
    pub details: Option<String>,
    pub state: Option<String>,
    pub place_id: Option<u64>,
}

enum Msg {
    Seite(Box<Seite>, Box<Config>),
    Update { activity: Box<Activity>, rpc: Option<RpcOverride>, cfg: Box<Config> },
    Config(Box<Config>),
}

static SENDER: OnceLock<Option<Mutex<Sender<Msg>>>> = OnceLock::new();

fn send(msg: Msg) {
    let sender = SENDER.get_or_init(|| {
        let (tx, rx) = mpsc::channel::<Msg>();
        std::thread::Builder::new()
            .name("brickstrap-discord".into())
            .spawn(move || run(rx))
            .ok()
            .map(|_| Mutex::new(tx))
    });
    if let Some(m) = sender {
        if let Ok(tx) = m.lock() {
            let _ = tx.send(msg);
        }
    }
}

/// Wird von `activity.rs` bei jeder Änderung aufgerufen.
pub fn update(app: &AppHandle, activity: &Activity, rpc: Option<&RpcOverride>) {
    let cfg = app.try_state::<AppState>().map(|s| s.config()).unwrap_or_default();
    send(Msg::Update { activity: Box::new(activity.clone()), rpc: rpc.cloned(), cfg: Box::new(cfg) });
}

/// Die Oberfläche meldet die offene Seite.
#[tauri::command]
pub fn discord_seite(state: tauri::State<'_, AppState>, details: Option<String>, state_text: Option<String>, place_id: Option<u64>) {
    let seite = Seite { details: details.as_deref().and_then(clip), state: state_text.as_deref().and_then(clip), place_id };
    send(Msg::Seite(Box::new(seite), Box::new(state.config())));
}

/// Einstellungen haben sich geändert: Status neu setzen oder löschen.
pub fn config_changed(state: &AppState) {
    // Läuft der Thread noch nicht, gibt es nichts anzupassen: das erste `update` liest die Einstellungen.
    if let Some(Some(m)) = SENDER.get() {
        if let Ok(tx) = m.lock() {
            let _ = tx.send(Msg::Config(Box::new(state.config())));
        }
    }
}

// ---------- Inhalt ----------

#[derive(Clone, Debug, PartialEq)]
struct Payload {
    details: Option<String>,
    state: Option<String>,
    large_image: Option<String>,
    large_text: Option<String>,
    /// Kleines Bild unten rechts: Brickstrap-Logo (Asset-Schlüssel der Discord-Anwendung)
    small: bool,
    start: Option<i64>,
    end: Option<i64>,
    buttons: Vec<(String, String)>,
}

fn is_job_id(s: &str) -> bool {
    s.len() == 36 && s.chars().all(|c| c.is_ascii_hexdigit() || c == '-')
}

/// Status, solange Brickstrap offen ist und kein Spiel läuft.
fn launcher_payload(s: &Seite, since: i64) -> Payload {
    let mut buttons = Vec::new();
    if let Some(place) = s.place_id {
        buttons.push(("Spielseite".into(), format!("https://www.roblox.com/games/{place}")));
    }
    Payload {
        details: s.details.clone().or_else(|| clip("Im Launcher")),
        state: s.state.clone(),
        large_image: Some(LOGO.into()),
        large_text: clip("Brickstrap"),
        small: false,
        start: Some(since),
        end: None,
        buttons,
    }
}

fn build_payload(cfg: &Config, a: &Activity, rpc: Option<&RpcOverride>) -> Option<Payload> {
    if a.state != "playing" {
        return None;
    }
    let name = a.game_name.as_deref().and_then(clip);
    let mut p = Payload {
        details: name.clone().or_else(|| clip("Spielt Roblox")),
        state: a.creator.as_deref().and_then(|c| clip(&format!("von {c}"))),
        large_image: a.icon_url.clone().filter(|u| u.starts_with("https://")),
        large_text: name,
        small: true,
        start: a.since.map(|s| s as i64),
        end: None,
        buttons: Vec::new(),
    };
    if let Some(r) = rpc {
        if r.details.is_some() {
            p.details = r.details.clone();
        }
        if r.state.is_some() {
            p.state = r.state.clone();
        }
        if r.time_start.is_some() {
            p.start = r.time_start;
        }
        p.end = r.time_end;
    }
    if let Some(place) = a.place_id {
        // Discord erlaubt in Knöpfen nur http(s), darum kein roblox://-Link.
        if let (true, Some(job)) = (cfg.discord_join_button, a.job_id.as_deref().filter(|j| is_job_id(j))) {
            // Format der Adresse unbestätigt: leitet Roblox auf den Server weiter?
            p.buttons.push((
                "Server beitreten".into(),
                format!("https://www.roblox.com/games/start?placeId={place}&gameInstanceId={job}"),
            ));
        }
        p.buttons.push(("Spielseite".into(), format!("https://www.roblox.com/games/{place}")));
    }
    p.buttons.truncate(2);
    Some(p)
}

fn show(client: &mut DiscordIpcClient, p: &Payload) -> Result<(), String> {
    let mut act = drp::Activity::new();
    if let Some(d) = &p.details {
        act = act.details(d);
    }
    if let Some(s) = &p.state {
        act = act.state(s);
    }
    if p.large_image.is_some() || p.large_text.is_some() || p.small {
        let mut assets = drp::Assets::new();
        if p.small {
            assets = assets.small_image(LOGO).small_text("Brickstrap");
        }
        if let Some(i) = &p.large_image {
            assets = assets.large_image(i);
        }
        if let Some(t) = &p.large_text {
            assets = assets.large_text(t);
        }
        act = act.assets(assets);
    }
    if p.start.is_some() || p.end.is_some() {
        let mut ts = drp::Timestamps::new();
        if let Some(s) = p.start {
            ts = ts.start(s);
        }
        if let Some(e) = p.end {
            ts = ts.end(e);
        }
        act = act.timestamps(ts);
    }
    if !p.buttons.is_empty() {
        act = act.buttons(p.buttons.iter().map(|(l, u)| drp::Button::new(l, u)).collect());
    }
    client.set_activity(act).map_err(|e| e.to_string())
}

// ---------- Thread ----------

struct Worker {
    client: Option<DiscordIpcClient>,
    connected_id: String,
    cfg: Config,
    activity: Activity,
    rpc: Option<RpcOverride>,
    last_try: Option<Instant>,
    /// Offene Seite im Launcher und seit wann Brickstrap läuft
    seite: Option<Seite>,
    since: i64,
    /// Zuletzt gesendeter Inhalt; `None` = nichts angezeigt
    shown: Option<Payload>,
}

impl Worker {
    fn close(&mut self) {
        if let Some(mut c) = self.client.take() {
            let _ = c.clear_activity();
            let _ = c.close();
        }
        self.shown = None;
    }

    fn connect(&mut self, app_id: &str) -> bool {
        if self.client.is_some() && self.connected_id == app_id {
            return true;
        }
        self.close();
        if self.last_try.map(|t| t.elapsed() < RETRY).unwrap_or(false) {
            return false;
        }
        self.last_try = Some(Instant::now());
        let Ok(mut c) = DiscordIpcClient::new(app_id) else { return false };
        if c.connect().is_err() {
            return false;
        }
        self.client = Some(c);
        self.connected_id = app_id.to_string();
        true
    }

    fn sync(&mut self) {
        let app_id = self.cfg.discord_id();
        if !self.cfg.discord || app_id.is_empty() {
            self.close();
            return;
        }
        let want = build_payload(&self.cfg, &self.activity, self.rpc.as_ref()).or_else(|| {
            // Während des Beitritts nichts zeigen, sonst die offene Seite des Launchers.
            if self.activity.state == "idle" {
                self.seite.as_ref().map(|s| launcher_payload(s, self.since))
            } else {
                None
            }
        });
        match want {
            None => {
                // Idle oder Beitritt: Status löschen, Verbindung bleibt für den nächsten Start.
                if self.shown.is_some() {
                    if let Some(c) = self.client.as_mut() {
                        if c.clear_activity().is_err() {
                            self.client = None;
                        }
                    }
                    self.shown = None;
                }
            }
            Some(p) => {
                if self.shown.as_ref() == Some(&p) && self.client.is_some() {
                    return;
                }
                if !self.connect(&app_id) {
                    return;
                }
                let ok = match self.client.as_mut() {
                    Some(c) => show(c, &p).is_ok(),
                    None => false,
                };
                if ok {
                    self.shown = Some(p);
                } else {
                    // Verbindung weg (Discord beendet?). Später neu versuchen.
                    self.client = None;
                    self.shown = None;
                    self.last_try = Some(Instant::now());
                }
            }
        }
    }
}

fn run(rx: mpsc::Receiver<Msg>) {
    let mut w = Worker {
        client: None,
        connected_id: String::new(),
        cfg: Config::default(),
        activity: Activity::default(),
        rpc: None,
        last_try: None,
        seite: None,
        since: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0),
        shown: None,
    };
    loop {
        match rx.recv_timeout(TICK) {
            Ok(Msg::Update { activity, rpc, cfg }) => {
                w.activity = *activity;
                w.rpc = rpc;
                w.cfg = *cfg;
            }
            Ok(Msg::Config(cfg)) => w.cfg = *cfg,
            Ok(Msg::Seite(seite, cfg)) => {
                w.seite = Some(*seite);
                w.cfg = *cfg;
            }
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => break,
        }
        w.sync();
    }
    w.close();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rpc_merge() {
        let mut r = RpcOverride::default();
        r.merge(&serde_json::json!({"details": "Runde 3", "state": "Level 7", "timeStart": 1700000000}));
        assert_eq!(r.details.as_deref(), Some("Runde 3"));
        assert_eq!(r.time_start, Some(1_700_000_000));
        r.merge(&serde_json::json!({"details": "<reset>", "timeStart": 1700000000000i64}));
        assert_eq!(r.details, None);
        assert_eq!(r.state.as_deref(), Some("Level 7"));
        assert_eq!(r.time_start, Some(1_700_000_000));
    }

    #[test]
    fn payload() {
        let cfg = Config { discord: true, discord_app_id: "1".into(), ..Config::default() };
        let mut a = Activity::default();
        assert!(build_payload(&cfg, &a, None).is_none());
        a.state = "playing".into();
        a.place_id = Some(5);
        a.job_id = Some("0123456789abcdef0123456789abcdef0123".into());
        a.game_name = Some("Testspiel".into());
        a.creator = Some("Ersteller".into());
        let p = build_payload(&cfg, &a, None);
        let p = p.expect("payload");
        assert_eq!(p.state.as_deref(), Some("von Ersteller"));
        assert_eq!(p.buttons.len(), 2);
    }
}
