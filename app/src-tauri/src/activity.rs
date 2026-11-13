//! Logdatei von Roblox mitlesen: Wo bin ich gerade, wie lange spiele ich?
//!
//! Brickstrap öffnet nur die Textdatei in `%localappdata%\Roblox\logs` zum Lesen
//! (mit Share-Read/Write, wie Bloxstrap). Kein Zugriff auf den Roblox-Prozess.
//! Die Zeilenformate sind nicht dokumentiert und können sich ändern. Unbekannte Zeilen
//! werden ignoriert.

use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::time::{Duration, Instant, SystemTime};

use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::discord::{self, RpcOverride};
use crate::paths::{logs_dir, roblox_running};
use crate::roblox_api::{game_brief, game_brief_by_place};
use crate::state::{now, read_json, write_json, AppState};

const HISTORY_FILE: &str = "history.json";
const MAX_SESSIONS: usize = 500;
const MIN_SESSION_SECS: u64 = 10;
const MAX_READ: u64 = 8 * 1024 * 1024;
const NEW_FILE_WINDOW: Duration = Duration::from_secs(15);

// ---------- Typen für die Oberfläche ----------

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Activity {
    /// "idle", "joining" oder "playing"
    pub state: String,
    pub place_id: Option<u64>,
    pub job_id: Option<String>,
    pub universe_id: Option<u64>,
    pub server_ip: Option<String>,
    /// Unix-Zeit in Sekunden, ab wann das Spiel läuft
    pub since: Option<u64>,
    pub game_name: Option<String>,
    pub icon_url: Option<String>,
    pub creator: Option<String>,
}

impl Default for Activity {
    fn default() -> Self {
        Self {
            state: "idle".into(),
            place_id: None,
            job_id: None,
            universe_id: None,
            server_ip: None,
            since: None,
            game_name: None,
            icon_url: None,
            creator: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub universe_id: u64,
    pub place_id: u64,
    pub name: String,
    pub start: u64,
    pub end: u64,
    pub secs: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameTime {
    pub universe_id: u64,
    pub place_id: u64,
    pub name: String,
    pub total_secs: u64,
    pub sessions: u64,
    pub last_played: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Playtime {
    pub games: Vec<GameTime>,
    pub recent: Vec<Session>,
}

// ---------- Zeilen verstehen ----------

#[derive(Clone, Debug, PartialEq)]
pub enum LogEvent {
    Joining { job_id: String, place_id: u64, ip: String },
    Universe(u64),
    Udmux(String),
    Joined,
    Disconnect,
    Leave,
    Rpc { command: String, data: Value },
}

struct Regexes {
    joining: Regex,
    universe: Regex,
    udmux: Regex,
}

fn regexes() -> Option<&'static Regexes> {
    static R: OnceLock<Option<Regexes>> = OnceLock::new();
    R.get_or_init(|| {
        Some(Regexes {
            joining: Regex::new(r"! Joining game '([0-9a-fA-F\-]{36})' place ([0-9]+) at ([0-9\.]+)").ok()?,
            universe: Regex::new(r"universeid:([0-9]+)").ok()?,
            udmux: Regex::new(r"UDMUX Address = ([0-9\.]+), Port = [0-9]+ \| RCC Server Address = ([0-9\.]+), Port = [0-9]+").ok()?,
        })
    })
    .as_ref()
}

/// Eine Logzeile in ein Ereignis übersetzen. Alles Unbekannte ergibt `None`.
pub fn parse_line(line: &str) -> Option<LogEvent> {
    let rx = regexes()?;
    if line.contains("[FLog::Output] ! Joining game") {
        let c = rx.joining.captures(line)?;
        return Some(LogEvent::Joining {
            job_id: c.get(1)?.as_str().to_ascii_lowercase(),
            place_id: c.get(2)?.as_str().parse().ok()?,
            ip: c.get(3)?.as_str().to_string(),
        });
    }
    if line.contains("[FLog::GameJoinLoadTime] Report game_join_loadtime:") {
        let c = rx.universe.captures(line)?;
        return Some(LogEvent::Universe(c.get(1)?.as_str().parse().ok()?));
    }
    if line.contains("[FLog::Network] UDMUX Address = ") {
        let c = rx.udmux.captures(line)?;
        // Die erste Adresse ist die echte Maschine, die zweite der Server hinter dem Proxy.
        return Some(LogEvent::Udmux(c.get(1)?.as_str().to_string()));
    }
    if line.contains("[FLog::Network] Replicator created: ") {
        return Some(LogEvent::Joined);
    }
    if line.contains("[FLog::Network] Time to disconnect replication data:") {
        return Some(LogEvent::Disconnect);
    }
    if line.contains("[FLog::SingleSurfaceApp] leaveUGCGameInternal") {
        return Some(LogEvent::Leave);
    }
    if line.contains("[FLog::CreatorOutput]") {
        let at = line.find("[BloxstrapRPC] ")?;
        let json = line.get(at + "[BloxstrapRPC] ".len()..)?.trim();
        if json.len() > 64 * 1024 {
            return None;
        }
        let v: Value = serde_json::from_str(json).ok()?;
        let command = v.get("command")?.as_str()?.to_string();
        let data = v.get("data").cloned().unwrap_or(Value::Null);
        return Some(LogEvent::Rpc { command, data });
    }
    None
}

// ---------- Zustandsmaschine ----------

struct Tracker {
    activity: Activity,
    rpc: RpcOverride,
    last_rpc: Option<Instant>,
    brief_requested: bool,
}

#[derive(Default)]
struct Outcome {
    changed: bool,
    finished: Option<Session>,
    fetch_brief: bool,
}

impl Tracker {
    fn new() -> Self {
        Self { activity: Activity::default(), rpc: RpcOverride::default(), last_rpc: None, brief_requested: false }
    }

    /// Laufende Sitzung beenden und den Zustand auf „idle“ setzen.
    fn finish(&mut self, now: u64) -> Option<Session> {
        let a = &self.activity;
        let session = match (a.state.as_str(), a.since) {
            ("playing", Some(start)) => {
                let secs = now.saturating_sub(start);
                (secs >= MIN_SESSION_SECS).then(|| Session {
                    universe_id: a.universe_id.unwrap_or(0),
                    place_id: a.place_id.unwrap_or(0),
                    name: a
                        .game_name
                        .clone()
                        .unwrap_or_else(|| a.place_id.map(|p| format!("Spiel {p}")).unwrap_or_else(|| "Unbekanntes Spiel".into())),
                    start,
                    end: now,
                    secs,
                })
            }
            _ => None,
        };
        self.activity = Activity::default();
        self.rpc = RpcOverride::default();
        self.brief_requested = false;
        session
    }

    fn handle(&mut self, ev: LogEvent, now: u64) -> Outcome {
        let mut out = Outcome::default();
        let active = self.activity.state != "idle";
        match ev {
            LogEvent::Joining { job_id, place_id, ip } => {
                // Teleport: die alte Sitzung zählt bis hierher.
                out.finished = self.finish(now);
                self.activity.state = "joining".into();
                self.activity.place_id = Some(place_id);
                self.activity.job_id = Some(job_id);
                self.activity.server_ip = Some(ip);
                out.changed = true;
            }
            LogEvent::Universe(u) if active => {
                self.activity.universe_id = Some(u);
                out.changed = true;
            }
            LogEvent::Udmux(ip) if active => {
                self.activity.server_ip = Some(ip);
                out.changed = true;
            }
            LogEvent::Joined if self.activity.state == "joining" => {
                self.activity.state = "playing".into();
                self.activity.since = Some(now);
                out.changed = true;
                if !self.brief_requested {
                    self.brief_requested = true;
                    out.fetch_brief = true;
                }
            }
            LogEvent::Disconnect | LogEvent::Leave if active => {
                out.finished = self.finish(now);
                out.changed = true;
            }
            LogEvent::Rpc { command, data } if active && command == "SetRichPresence" => {
                // Höchstens eine Nachricht pro Sekunde (wie Bloxstrap).
                let ok = self.last_rpc.map(|t| t.elapsed() >= Duration::from_secs(1)).unwrap_or(true);
                if ok {
                    self.last_rpc = Some(Instant::now());
                    self.rpc.merge(&data);
                    out.changed = true;
                }
            }
            _ => {}
        }
        out
    }
}

fn tracker() -> &'static Mutex<Tracker> {
    static T: OnceLock<Mutex<Tracker>> = OnceLock::new();
    T.get_or_init(|| Mutex::new(Tracker::new()))
}

fn lock() -> MutexGuard<'static, Tracker> {
    tracker().lock().unwrap_or_else(|p| p.into_inner())
}

#[tauri::command]
pub fn activity_get() -> Activity {
    lock().activity.clone()
}

/// Zustand an die Oberfläche und an Discord melden.
fn publish(app: &AppHandle, activity: &Activity, rpc: &RpcOverride) {
    let _ = app.emit("activity", activity);
    let rpc = if rpc.is_empty() { None } else { Some(rpc) };
    discord::update(app, activity, rpc);
}

fn spawn_brief(app: AppHandle, universe: Option<u64>, place: Option<u64>, job: Option<String>) {
    let _ = std::thread::Builder::new().name("brickstrap-brief".into()).spawn(move || {
        let state: State<'_, AppState> = app.state();
        let brief = tauri::async_runtime::block_on(async {
            match (universe, place) {
                (Some(u), _) => game_brief(&state, u).await,
                (None, Some(p)) => game_brief_by_place(&state, p).await,
                _ => None,
            }
        });
        let Some(b) = brief else { return };
        let (a, rpc) = {
            let mut t = lock();
            // Nur eintragen, wenn es noch dieselbe Sitzung ist.
            if t.activity.state != "playing" || t.activity.place_id != place || t.activity.job_id != job {
                return;
            }
            t.activity.game_name = Some(b.name);
            t.activity.icon_url = b.icon_url;
            t.activity.creator = Some(b.creator).filter(|c| !c.is_empty());
            if t.activity.universe_id.is_none() {
                t.activity.universe_id = Some(b.universe_id);
            }
            (t.activity.clone(), t.rpc.clone())
        };
        publish(&app, &a, &rpc);
    });
}

fn process_line(app: &AppHandle, line: &str) {
    let Some(ev) = parse_line(line) else { return };
    let (out, snapshot) = {
        let mut t = lock();
        let out = t.handle(ev, now());
        (out, (t.activity.clone(), t.rpc.clone()))
    };
    if let Some(s) = out.finished {
        save_session(&app.state::<AppState>(), s);
    }
    if out.changed {
        publish(app, &snapshot.0, &snapshot.1);
    }
    if out.fetch_brief {
        spawn_brief(app.clone(), snapshot.0.universe_id, snapshot.0.place_id, snapshot.0.job_id.clone());
    }
}

// ---------- Verlauf ----------

fn save_session(state: &AppState, s: Session) {
    let path = state.file(HISTORY_FILE);
    let mut all: Vec<Session> = read_json(&path).unwrap_or_default();
    all.push(s);
    if all.len() > MAX_SESSIONS {
        let extra = all.len() - MAX_SESSIONS;
        all.drain(..extra);
    }
    if let Err(e) = write_json(&path, &all) {
        eprintln!("Verlauf: {e}");
    }
}

fn summarize(all: Vec<Session>) -> Playtime {
    // Schlüssel: Universe-ID, ohne sie die Place-ID.
    let mut map: HashMap<(u64, u64), GameTime> = HashMap::new();
    for s in &all {
        let key = if s.universe_id != 0 { (s.universe_id, 0) } else { (0, s.place_id) };
        let g = map.entry(key).or_insert_with(|| GameTime {
            universe_id: s.universe_id,
            place_id: s.place_id,
            name: s.name.clone(),
            total_secs: 0,
            sessions: 0,
            last_played: 0,
        });
        g.total_secs += s.secs;
        g.sessions += 1;
        if s.end >= g.last_played {
            g.last_played = s.end;
            g.place_id = s.place_id;
            g.name = s.name.clone();
        }
    }
    let mut games: Vec<GameTime> = map.into_values().collect();
    games.sort_by(|a, b| b.total_secs.cmp(&a.total_secs).then(b.last_played.cmp(&a.last_played)));
    let mut recent = all;
    recent.sort_by_key(|s| std::cmp::Reverse(s.start));
    recent.truncate(30);
    Playtime { games, recent }
}

#[tauri::command]
pub fn playtime_get(state: State<'_, AppState>) -> Playtime {
    summarize(read_json(&state.file(HISTORY_FILE)).unwrap_or_default())
}

// ---------- Logdatei lesen ----------

fn open_shared(path: &Path) -> std::io::Result<File> {
    let mut o = std::fs::OpenOptions::new();
    o.read(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        // FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE
        o.share_mode(7);
    }
    o.open(path)
}

/// Neueste Datei mit „Player“ im Namen und ihre Erstellzeit.
fn newest_player_log(dir: &Path) -> Option<(PathBuf, SystemTime)> {
    let mut best: Option<(PathBuf, SystemTime)> = None;
    for e in std::fs::read_dir(dir).ok()?.flatten() {
        let name = e.file_name().to_string_lossy().into_owned();
        if !name.contains("Player") {
            continue;
        }
        let Ok(meta) = e.metadata() else { continue };
        if !meta.is_file() {
            continue;
        }
        let t = meta.created().or_else(|_| meta.modified()).unwrap_or(SystemTime::UNIX_EPOCH);
        if best.as_ref().map(|(_, bt)| t > *bt).unwrap_or(true) {
            best = Some((e.path(), t));
        }
    }
    best
}

struct Watcher {
    started: SystemTime,
    initialized: bool,
    current: Option<PathBuf>,
    pos: u64,
    pending: Vec<u8>,
    gone_ticks: u8,
    /// Lief Roblox beim letzten Durchlauf?
    was_running: bool,
}

impl Watcher {
    fn new() -> Self {
        Self { started: SystemTime::now(), initialized: false, current: None, pos: 0, pending: Vec::new(), gone_ticks: 0, was_running: false }
    }

    /// Neue ganze Zeilen seit dem letzten Aufruf.
    fn read_lines(&mut self) -> Vec<String> {
        let Some(path) = self.current.clone() else { return Vec::new() };
        let Ok(mut f) = open_shared(&path) else { return Vec::new() };
        let Ok(len) = f.metadata().map(|m| m.len()) else { return Vec::new() };
        if len < self.pos {
            // Datei wurde gekürzt oder ersetzt.
            self.pos = 0;
            self.pending.clear();
        }
        if len == self.pos || f.seek(SeekFrom::Start(self.pos)).is_err() {
            return Vec::new();
        }
        let mut buf = Vec::new();
        if (&mut f).take((len - self.pos).min(MAX_READ)).read_to_end(&mut buf).is_err() {
            return Vec::new();
        }
        self.pos += buf.len() as u64;
        self.pending.extend_from_slice(&buf);
        let Some(last_nl) = self.pending.iter().rposition(|&b| b == b'\n') else { return Vec::new() };
        let complete: Vec<u8> = self.pending.drain(..=last_nl).collect();
        String::from_utf8_lossy(&complete)
            .split('\n')
            .map(|l| l.trim_end_matches('\r').to_string())
            .filter(|l| !l.is_empty())
            .collect()
    }

    /// Ein Durchlauf. `false` heißt: kein Logs-Ordner, später neu versuchen.
    fn tick(&mut self, app: &AppHandle) -> bool {
        let Some(dir) = logs_dir().filter(|d| d.is_dir()) else { return false };
        let newest = newest_player_log(&dir);

        if !self.initialized {
            // Beim Start die aktuellste Datei nehmen und ab ihrem Ende lesen.
            self.initialized = true;
            if let Some((p, _)) = newest {
                self.pos = std::fs::metadata(&p).map(|m| m.len()).unwrap_or(0);
                self.current = Some(p);
            }
        } else if let Some((p, created)) = newest {
            let fresh = created >= self.started.checked_sub(NEW_FILE_WINDOW).unwrap_or(SystemTime::UNIX_EPOCH);
            if fresh && self.current.as_ref() != Some(&p) {
                // Neuer Roblox-Start: eine offene Sitzung der alten Datei ist vorbei.
                if lock().activity.state != "idle" {
                    process_event(app, LogEvent::Leave);
                }
                self.current = Some(p);
                self.pos = 0;
                self.pending.clear();
            }
        }

        for line in self.read_lines() {
            process_line(app, &line);
        }

        // Roblox wurde geschlossen: Brickstrap wieder zeigen (abschaltbar).
        let running = roblox_running();
        if self.was_running && !running && app.state::<AppState>().config().show_on_exit {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.unminimize();
                let _ = w.show();
                let _ = w.set_focus();
            }
        }
        self.was_running = running;

        // Roblox abgestürzt oder beendet, ohne dass eine Abschlusszeile kam.
        if lock().activity.state != "idle" && !roblox_running() {
            self.gone_ticks = self.gone_ticks.saturating_add(1);
            if self.gone_ticks >= 3 {
                process_event(app, LogEvent::Leave);
                self.gone_ticks = 0;
            }
        } else {
            self.gone_ticks = 0;
        }
        true
    }
}

fn process_event(app: &AppHandle, ev: LogEvent) {
    let (out, snapshot) = {
        let mut t = lock();
        let out = t.handle(ev, now());
        (out, (t.activity.clone(), t.rpc.clone()))
    };
    if let Some(s) = out.finished {
        save_session(&app.state::<AppState>(), s);
    }
    if out.changed {
        publish(app, &snapshot.0, &snapshot.1);
    }
}

/// Startet den Überwachungs-Thread.
pub fn start(app: AppHandle) {
    let _ = std::thread::Builder::new().name("brickstrap-activity".into()).spawn(move || {
        let mut w = Watcher::new();
        loop {
            let ok = w.tick(&app);
            std::thread::sleep(Duration::from_secs(if ok { 1 } else { 5 }));
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    const JOIN: &str = "2025-01-01T12:00:00.123Z,1.0,abc,6 [FLog::Output] ! Joining game '0a1b2c3d-0000-4444-8888-aabbccddeeff' place 920587237 at 128.116.1.2";

    #[test]
    fn zeilen() {
        assert_eq!(
            parse_line(JOIN),
            Some(LogEvent::Joining { job_id: "0a1b2c3d-0000-4444-8888-aabbccddeeff".into(), place_id: 920587237, ip: "128.116.1.2".into() })
        );
        assert_eq!(
            parse_line("x [FLog::GameJoinLoadTime] Report game_join_loadtime: placeid:1, universeid:383310974, userid:5, time:2"),
            Some(LogEvent::Universe(383310974))
        );
        assert_eq!(
            parse_line("x [FLog::Network] UDMUX Address = 1.2.3.4, Port = 5 | RCC Server Address = 9.8.7.6, Port = 7"),
            Some(LogEvent::Udmux("1.2.3.4".into()))
        );
        assert_eq!(parse_line("x [FLog::Network] Replicator created: serverId: 1.2.3.4|5"), Some(LogEvent::Joined));
        assert_eq!(parse_line("x [FLog::Network] Time to disconnect replication data: 0.1"), Some(LogEvent::Disconnect));
        assert_eq!(parse_line("x [FLog::SingleSurfaceApp] leaveUGCGameInternal"), Some(LogEvent::Leave));
        match parse_line("x [FLog::CreatorOutput] [BloxstrapRPC] {\"command\":\"SetRichPresence\",\"data\":{\"details\":\"Hi\"}}") {
            Some(LogEvent::Rpc { command, data }) => {
                assert_eq!(command, "SetRichPresence");
                assert_eq!(data["details"], "Hi");
            }
            other => panic!("{other:?}"),
        }
        assert_eq!(parse_line("irgendeine Zeile"), None);
        assert_eq!(parse_line("[FLog::Output] ! Joining game 'kaputt'"), None);
    }

    #[test]
    fn ablauf() {
        let mut t = Tracker::new();
        t.handle(parse_line(JOIN).expect("join"), 1000);
        assert_eq!(t.activity.state, "joining");
        t.handle(LogEvent::Universe(7), 1001);
        let o = t.handle(LogEvent::Joined, 1005);
        assert!(o.fetch_brief);
        assert_eq!(t.activity.state, "playing");
        t.activity.game_name = Some("Spiel".into());
        let o = t.handle(LogEvent::Disconnect, 1065);
        let s = o.finished.expect("sitzung");
        assert_eq!((s.universe_id, s.secs, s.name.as_str()), (7, 60, "Spiel"));
        assert_eq!(t.activity.state, "idle");

        // Kurze Sitzung zählt nicht.
        t.handle(parse_line(JOIN).expect("join"), 2000);
        t.handle(LogEvent::Joined, 2001);
        assert!(t.handle(LogEvent::Leave, 2005).finished.is_none());
    }

    #[test]
    fn auswertung() {
        let mk = |u, secs, end| Session { universe_id: u, place_id: u, name: format!("G{u}"), start: end - secs, end, secs };
        let p = summarize(vec![mk(1, 100, 1000), mk(2, 500, 2000), mk(1, 100, 3000)]);
        assert_eq!(p.games.len(), 2);
        assert_eq!(p.games[0].universe_id, 2);
        assert_eq!(p.games[1].sessions, 2);
        assert_eq!(p.recent[0].end, 3000);
    }
}
