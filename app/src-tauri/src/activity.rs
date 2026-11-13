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
use crate::paths::{all_logs_dirs, player_version_dirs, roblox_running};
use crate::roblox_api::{game_brief, game_brief_by_place};
use crate::state::{now, read_json, write_json, AppState};

const HISTORY_FILE: &str = "history.json";
const MAX_SESSIONS: usize = 2000;
const MONTH_SECS: u64 = 30 * 86400;
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
    /// Ungefährer Standort des Servers, z. B. „Frankfurt, DE“ (aus der IP, kann fehlen)
    pub region: Option<String>,
    /// Spieler und Plätze auf dem eigenen Server (aus der Serverliste, kann fehlen)
    pub server_playing: Option<u64>,
    pub server_max: Option<u64>,
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
            region: None,
            server_playing: None,
            server_max: None,
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
    /// Server der Sitzung, für „Wieder beitreten“ (fehlt bei alten Einträgen)
    #[serde(default)]
    pub job_id: Option<String>,
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
    /// Alle Sitzungen der letzten 30 Tage (für die Statistik nach Tag und Zeitraum)
    pub month: Vec<Session>,
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
                    job_id: a.job_id.clone(),
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
                self.activity.region = crate::region::lookup(&ip);
                self.activity.server_ip = Some(ip);
                out.changed = true;
            }
            LogEvent::Universe(u) if active => {
                self.activity.universe_id = Some(u);
                out.changed = true;
            }
            LogEvent::Udmux(ip) if active => {
                // Die UDMUX-Adresse ist die öffentliche; nur sie steht in der Regionstabelle.
                self.activity.region = crate::region::lookup(&ip).or(self.activity.region.take());
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

/// Zeitstempel am Zeilenanfang (`2025-01-01T12:00:00.123Z,…`, UTC) als Unix-Sekunden.
pub fn line_time(line: &str) -> Option<u64> {
    let b = line.as_bytes();
    if b.len() < 19 || b[4] != b'-' || b[7] != b'-' || b[10] != b'T' || b[13] != b':' || b[16] != b':' {
        return None;
    }
    let n = |from: usize, to: usize| line.get(from..to)?.parse::<i64>().ok();
    let (mut y, m, d) = (n(0, 4)?, n(5, 7)?, n(8, 10)?);
    let (h, mi, s) = (n(11, 13)?, n(14, 16)?, n(17, 19)?);
    if !(1..=12).contains(&m) || !(1..=31).contains(&d) || h > 23 || mi > 59 || s > 60 {
        return None;
    }
    // Tage seit 1970 (Kalenderrechnung nach Howard Hinnant).
    if m <= 2 {
        y -= 1;
    }
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146_097 + doe - 719_468;
    u64::try_from(days * 86_400 + h * 3600 + mi * 60 + s).ok()
}

/// Stand aus schon geschriebenen Zeilen nachbauen: Roblox lief schon, als Brickstrap startete.
/// Die Zeiten kommen aus den Zeilen selbst, damit „seit“ stimmt.
fn replay(lines: &[String], fallback: u64) -> Tracker {
    let mut t = Tracker::new();
    for line in lines {
        if let Some(ev) = parse_line(line) {
            // Beim Nachlesen gilt die Bremse für BloxstrapRPC nicht.
            t.last_rpc = None;
            t.handle(ev, line_time(line).unwrap_or(fallback));
        }
    }
    t.last_rpc = None;
    t
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

/// Fragt während des Spiels jede Minute die Spielerzahl des eigenen Servers ab (für Discord).
/// Endet, sobald die Sitzung vorbei ist. Höchstens drei Abrufe pro Minute.
fn spawn_server_poll(app: AppHandle, place: Option<u64>, job: Option<String>) {
    let (Some(place_id), Some(job_id)) = (place, job) else { return };
    let _ = std::thread::Builder::new().name("brickstrap-server".into()).spawn(move || loop {
        let state: State<'_, AppState> = app.state();
        let count = tauri::async_runtime::block_on(crate::roblox_api::server_count(&state, place_id, &job_id));
        let snapshot = {
            let mut t = lock();
            if t.activity.state != "playing" || t.activity.job_id.as_deref() != Some(job_id.as_str()) {
                return;
            }
            let (p, m) = count.map_or((None, None), |(p, m)| (Some(p), Some(m)));
            if t.activity.server_playing != p || t.activity.server_max != m {
                t.activity.server_playing = p;
                t.activity.server_max = m;
                Some((t.activity.clone(), t.rpc.clone()))
            } else {
                None
            }
        };
        if let Some((a, rpc)) = snapshot {
            publish(&app, &a, &rpc);
        }
        std::thread::sleep(Duration::from_secs(60));
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
        spawn_server_poll(app.clone(), snapshot.0.place_id, snapshot.0.job_id.clone());
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

fn summarize(all: Vec<Session>, now: u64) -> Playtime {
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
    let from = now.saturating_sub(MONTH_SECS);
    let month: Vec<Session> = recent.iter().filter(|s| s.end >= from).cloned().collect();
    recent.truncate(30);
    Playtime { games, recent, month }
}

#[tauri::command]
pub fn playtime_get(state: State<'_, AppState>) -> Playtime {
    summarize(read_json(&state.file(HISTORY_FILE)).unwrap_or_default(), now())
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

/// Neueste Datei mit „Player“ im Namen und ihre Erstellzeit, über alle Log-Ordner.
fn newest_player_log(dirs: &[PathBuf]) -> Option<(PathBuf, SystemTime)> {
    let mut best: Option<(PathBuf, SystemTime)> = None;
    for e in dirs.iter().filter_map(|d| std::fs::read_dir(d).ok()).flatten().flatten() {
        let name = e.file_name().to_string_lossy().into_owned();
        // Neben jedem Spiel-Log legt Roblox eine `…_Player_…_CrashHandler_last.log` an,
        // etwas später. Die enthält keine Spielzeilen und darf nicht gewinnen.
        if !name.contains("Player") || name.contains("CrashHandler") || !name.ends_with(".log") {
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
    /// Namen der Versionsordner beim letzten Durchlauf
    versions: Option<Vec<String>>,
}

impl Watcher {
    fn new() -> Self {
        Self { started: SystemTime::now(), initialized: false, current: None, pos: 0, pending: Vec::new(), gone_ticks: 0, was_running: false, versions: None }
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
        let dirs = all_logs_dirs();
        if dirs.is_empty() {
            return false;
        }
        let newest = newest_player_log(&dirs);

        if !self.initialized {
            // Beim Start die aktuellste Datei nehmen. Läuft Roblox schon, den Stand aus dem
            // Ende der Datei nachbauen (höchstens MAX_READ), sonst ab dem Ende weiterlesen.
            self.initialized = true;
            if let Some((p, _)) = newest {
                let len = std::fs::metadata(&p).map(|m| m.len()).unwrap_or(0);
                self.current = Some(p);
                if roblox_running() {
                    self.pos = len.saturating_sub(MAX_READ);
                    let lines = self.read_lines();
                    adopt(app, replay(&lines, now()));
                } else {
                    self.pos = len;
                }
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

        self.watch_versions(app);

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

        // Roblox abgestürzt oder beendet, ohne dass eine Abschlusszeile kam. Schreibt Roblox
        // noch in die Logdatei, läuft es (z. B. Store-Version mit anderem Prozessnamen).
        let log_aktiv = self
            .current
            .as_ref()
            .and_then(|p| std::fs::metadata(p).and_then(|m| m.modified()).ok())
            .and_then(|t| t.elapsed().ok())
            .is_some_and(|d| d < Duration::from_secs(30));
        if lock().activity.state != "idle" && !running && !log_aktiv {
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

impl Watcher {
    /// Roblox hat sich aktualisiert: Der neue Versionsordner hat noch keine
    /// `ClientAppSettings.json`. Sofort schreiben, damit die Flags möglichst schon bei
    /// diesem Start greifen (nur eine Datei, kein Zugriff auf den Prozess).
    fn watch_versions(&mut self, app: &AppHandle) {
        let mut names: Vec<String> = player_version_dirs()
            .iter()
            .filter_map(|d| d.file_name().map(|n| n.to_string_lossy().into_owned()))
            .collect();
        names.sort();
        let neu = self.versions.as_ref().is_some_and(|old| names.iter().any(|n| !old.contains(n)));
        if neu {
            if let Err(e) = crate::fastflags::apply_before_launch(&app.state::<AppState>()) {
                eprintln!("FastFlags nach Update: {e}");
            }
            let _ = app.emit("roblox-version", names.clone());
        }
        self.versions = Some(names);
    }
}

/// Nachgebauten Stand übernehmen und melden (nur wenn gerade gespielt oder beigetreten wird).
fn adopt(app: &AppHandle, found: Tracker) {
    if found.activity.state == "idle" {
        return;
    }
    let (a, rpc) = {
        let mut t = lock();
        *t = found;
        (t.activity.clone(), t.rpc.clone())
    };
    publish(app, &a, &rpc);
    if a.state == "playing" {
        spawn_brief(app.clone(), a.universe_id, a.place_id, a.job_id.clone());
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
        let mk = |u, secs, end| Session { universe_id: u, place_id: u, name: format!("G{u}"), start: end - secs, end, secs, job_id: None };
        let p = summarize(vec![mk(1, 100, 1000), mk(2, 500, 2000), mk(1, 100, 3000)], 3000 + MONTH_SECS - 500);
        assert_eq!(p.games.len(), 2);
        assert_eq!(p.games[0].universe_id, 2);
        assert_eq!(p.games[1].sessions, 2);
        assert_eq!(p.recent[0].end, 3000);
        // Nur die letzte Sitzung liegt in den 30 Tagen.
        assert_eq!(p.month.len(), 1);
    }

    #[test]
    fn zeitstempel() {
        assert_eq!(line_time("1970-01-01T00:00:00.000Z,x"), Some(0));
        assert_eq!(line_time("2025-01-01T12:00:00.123Z,1.0,abc,6 [FLog::Output] x"), Some(1_735_732_800));
        assert_eq!(line_time("2024-02-29T23:59:59Z"), Some(1_709_251_199));
        assert_eq!(line_time("irgendeine Zeile ohne Zeit"), None);
        assert_eq!(line_time("2025-13-01T12:00:00.123Z"), None);
    }

    #[test]
    fn nachlesen() {
        let zeile = |t: &str, rest: &str| format!("2025-01-01T12:{t}.000Z,1.0,abc,6 {rest}");
        let mut lines = vec![
            zeile("00:00", "[FLog::Output] ! Joining game '0a1b2c3d-0000-4444-8888-aabbccddeeff' place 5 at 10.0.0.1"),
            zeile("00:02", "[FLog::Network] UDMUX Address = 128.116.44.33, Port = 5 | RCC Server Address = 10.0.0.1, Port = 7"),
            zeile("00:03", "[FLog::GameJoinLoadTime] Report game_join_loadtime: placeid:5, universeid:9, userid:1"),
            zeile("00:05", "[FLog::Network] Replicator created: serverId: 1.2.3.4|5"),
        ];
        let t = replay(&lines, 1);
        assert_eq!(t.activity.state, "playing");
        assert_eq!(t.activity.since, Some(1_735_732_805));
        assert_eq!(t.activity.universe_id, Some(9));
        assert_eq!(t.activity.region.as_deref(), Some("Frankfurt, DE"));
        assert!(t.brief_requested);

        // Danach verlassen: nichts läuft mehr.
        lines.push(zeile("30:00", "[FLog::SingleSurfaceApp] leaveUGCGameInternal"));
        assert_eq!(replay(&lines, 1).activity.state, "idle");
    }
}
