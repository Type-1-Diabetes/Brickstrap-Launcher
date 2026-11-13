//! Zugriff auf die Web-APIs von Roblox (ohne Login).
//!
//! Suche und Explore sind nicht offiziell dokumentiert. Darum werden alle Antworten
//! nachsichtig gelesen (mehrere mögliche Feldnamen) und Fehler sauber gemeldet.
//! Feldnamen: docs/research/recherche-ui-api.md, Teil B.

use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::State;

use crate::state::{now, read_json, write_json, AppState};

/// Fehler an die Oberfläche: `code` steuert den Text, `message` ist die Rohmeldung.
#[derive(Debug, Serialize)]
pub struct ApiError {
    pub code: &'static str,
    pub message: String,
}

impl ApiError {
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self { code, message: message.into() }
    }
}

impl From<reqwest::Error> for ApiError {
    fn from(e: reqwest::Error) -> Self {
        if e.is_timeout() {
            ApiError::new("TIMEOUT", e.to_string())
        } else if e.is_connect() || e.is_request() {
            ApiError::new("NETZ", e.to_string())
        } else {
            ApiError::new("ANTWORT", e.to_string())
        }
    }
}

type ApiResult<T> = Result<T, ApiError>;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameCard {
    pub universe_id: u64,
    pub place_id: u64,
    pub name: String,
    pub playing: u64,
    pub up_votes: u64,
    pub down_votes: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sort {
    pub id: String,
    pub title: String,
    pub games: Vec<GameCard>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HomeFeed {
    pub sorts: Vec<Sort>,
    /// Unix-Zeit des Abrufs
    pub fetched_at: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameDetail {
    pub universe_id: u64,
    pub place_id: u64,
    pub name: String,
    pub description: String,
    pub creator_name: String,
    pub creator_verified: bool,
    pub playing: u64,
    pub visits: u64,
    pub max_players: u64,
    pub favorites: u64,
    pub genre: String,
    pub created: String,
    pub updated: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Server {
    pub id: String,
    pub playing: u64,
    pub max_players: u64,
    /// Server-Ping laut Roblox (nicht der eigene Ping)
    pub ping: Option<u64>,
    pub fps: Option<f64>,
    /// Verschleierte Kennungen der Spieler (nur für Avatar-Bilder nutzbar)
    pub player_tokens: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerPage {
    pub servers: Vec<Server>,
    pub next_cursor: Option<String>,
}

/// Kurzinfo für Discord-Status und Verlauf.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameBrief {
    pub universe_id: u64,
    pub place_id: u64,
    pub name: String,
    pub creator: String,
    pub icon_url: Option<String>,
}

// ---------- Hilfen zum nachsichtigen Lesen ----------

fn num(v: &Value, keys: &[&str]) -> u64 {
    for k in keys {
        match v.get(*k) {
            Some(Value::Number(n)) => return n.as_u64().unwrap_or_else(|| n.as_f64().unwrap_or(0.0) as u64),
            Some(Value::String(s)) => {
                if let Ok(n) = s.parse() {
                    return n;
                }
            }
            _ => {}
        }
    }
    0
}

fn text(v: &Value, keys: &[&str]) -> String {
    for k in keys {
        if let Some(s) = v.get(*k).and_then(Value::as_str) {
            return s.to_string();
        }
    }
    String::new()
}

fn card(v: &Value) -> Option<GameCard> {
    let universe_id = num(v, &["universeId", "UniverseId", "id"]);
    let place_id = num(v, &["rootPlaceId", "placeId", "RootPlaceId"]);
    let name = text(v, &["name", "Name"]);
    if universe_id == 0 || name.is_empty() || name == "[TITLE UNAVAILABLE]" {
        return None;
    }
    Some(GameCard {
        universe_id,
        place_id,
        name,
        playing: num(v, &["playerCount", "playing", "PlayerCount"]),
        up_votes: num(v, &["totalUpVotes", "upVotes"]),
        down_votes: num(v, &["totalDownVotes", "downVotes"]),
    })
}

/// Rohantworten ablegen, wenn es im Datenordner den Ordner `roh` gibt. Zum Prüfen der
/// undokumentierten Endpunkte am echten PC: Ordner anlegen, App benutzen, Dateien ansehen.
fn dump_raw(state: &AppState, url: &str, v: &Value) {
    let dir = state.file("roh");
    if !dir.is_dir() {
        return;
    }
    let name: String = url
        .trim_start_matches("https://")
        .split('?')
        .next()
        .unwrap_or("antwort")
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '.' || c == '-' { c } else { '_' })
        .take(120)
        .collect();
    let _ = write_json(&dir.join(format!("{name}.json")), v);
}

pub(crate) async fn get_json(state: &AppState, url: &str) -> ApiResult<Value> {
    let resp = state.http.get(url).send().await?;
    let status = resp.status();
    if status.as_u16() == 429 {
        let wait = resp
            .headers()
            .get("retry-after")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("30")
            .to_string();
        return Err(ApiError::new("RATE_LIMIT", wait));
    }
    if !status.is_success() {
        return Err(ApiError::new("HTTP", status.as_u16().to_string()));
    }
    let v = resp.json::<Value>().await?;
    dump_raw(state, url, &v);
    Ok(v)
}

fn cache_path(state: &AppState, key: &str) -> PathBuf {
    let dir = state.file("cache");
    let _ = std::fs::create_dir_all(&dir);
    dir.join(format!("{key}.json"))
}

fn ids(list: &[u64]) -> String {
    list.iter().map(u64::to_string).collect::<Vec<_>>().join(",")
}

// ---------- Startseite ----------

/// Startseiten-Reihen. `fresh = false` liefert nur den Cache (sofort), `true` lädt neu.
#[tauri::command]
pub async fn home_feed(state: State<'_, AppState>, fresh: bool) -> ApiResult<Option<HomeFeed>> {
    let path = cache_path(&state, "home");
    if !fresh {
        return Ok(read_json(&path));
    }
    let url = format!(
        "https://apis.roblox.com/explore-api/v1/get-sorts?sessionId={}&device=computer",
        state.session_id
    );
    let v = get_json(&state, &url).await?;
    let mut sorts = Vec::new();
    if let Some(list) = v.get("sorts").and_then(Value::as_array) {
        for s in list {
            let games: Vec<GameCard> = s
                .get("games")
                .and_then(Value::as_array)
                .map(|g| g.iter().filter_map(card).collect())
                .unwrap_or_default();
            if games.is_empty() {
                continue;
            }
            sorts.push(Sort {
                id: text(s, &["sortId", "id"]),
                title: text(s, &["sortDisplayName", "displayName", "title"]),
                games,
            });
        }
    }
    if sorts.is_empty() {
        return Err(ApiError::new("FORMAT", "Startseite ohne Spiele"));
    }
    let feed = HomeFeed { sorts, fetched_at: now() };
    let _ = write_json(&path, &feed);
    Ok(Some(feed))
}

// ---------- Suche ----------

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchPage {
    pub games: Vec<GameCard>,
    pub next_page: Option<String>,
}

#[tauri::command]
pub async fn search(state: State<'_, AppState>, query: String, page: Option<String>) -> ApiResult<SearchPage> {
    let q = query.trim();
    if q.is_empty() {
        return Ok(SearchPage { games: vec![], next_page: None });
    }
    // Eine Place-ID oder ein Spiel-Link: direkt auflösen.
    if let Some(place) = place_id_from_input(q) {
        if let Some(b) = game_brief_by_place(&state, place).await {
            return Ok(SearchPage {
                games: vec![GameCard {
                    universe_id: b.universe_id,
                    place_id: b.place_id,
                    name: b.name,
                    playing: 0,
                    up_votes: 0,
                    down_votes: 0,
                }],
                next_page: None,
            });
        }
    }
    let mut url = reqwest::Url::parse("https://apis.roblox.com/search-api/omni-search")
        .map_err(|e| ApiError::new("INTERN", e.to_string()))?;
    url.query_pairs_mut()
        .append_pair("searchQuery", q)
        .append_pair("sessionId", &state.session_id)
        .append_pair("pageType", "all");
    if let Some(p) = &page {
        url.query_pairs_mut().append_pair("pageToken", p);
    }
    let v = get_json(&state, url.as_str()).await?;
    let mut games = Vec::new();
    for r in v.get("searchResults").and_then(Value::as_array).into_iter().flatten() {
        let ty = text(r, &["contentGroupType", "contentType"]);
        for c in r.get("contents").and_then(Value::as_array).into_iter().flatten() {
            let cty = text(c, &["contentType"]);
            if (cty.is_empty() && ty != "Game") || (!cty.is_empty() && cty != "Game") {
                continue;
            }
            if let Some(g) = card(c) {
                games.push(g);
            }
        }
    }
    let next = v.get("nextPageToken").and_then(Value::as_str).filter(|s| !s.is_empty()).map(String::from);
    Ok(SearchPage { games, next_page: next })
}

/// Vorschläge beim Tippen. Der Endpunkt ist undokumentiert: Jeder Fehler ergibt eine leere
/// Liste, die Suche selbst hängt nicht davon ab.
#[tauri::command]
pub async fn search_suggest(state: State<'_, AppState>, query: String) -> Result<Vec<String>, ()> {
    let q = query.trim();
    if q.chars().count() < 2 || place_id_from_input(q).is_some() {
        return Ok(Vec::new());
    }
    let url = format!("https://apis.roblox.com/games-autocomplete/v1/get-suggestion/{}", urlencode(q));
    let Ok(v) = get_json(&state, &url).await else { return Ok(Vec::new()) };
    let mut out: Vec<String> = Vec::new();
    for e in v.get("entries").and_then(Value::as_array).into_iter().flatten() {
        let s = text(e, &["searchQuery", "canonicalTitle", "query", "name"]);
        let s = s.trim();
        if !s.is_empty() && !out.iter().any(|o| o.eq_ignore_ascii_case(s)) {
            out.push(s.to_string());
        }
        if out.len() >= 6 {
            break;
        }
    }
    Ok(out)
}

/// Erkennt `123456`, `roblox.com/games/123456/...` und `placeId=123456`.
fn place_id_from_input(q: &str) -> Option<u64> {
    if q.chars().all(|c| c.is_ascii_digit()) && q.len() >= 3 {
        return q.parse().ok();
    }
    let re = regex::Regex::new(r"(?:/games/|placeId=)(\d{3,})").ok()?;
    re.captures(q)?.get(1)?.as_str().parse().ok()
}

// ---------- Spieldetails ----------

fn detail_from(v: &Value) -> Option<GameDetail> {
    let universe_id = num(v, &["id"]);
    let name = text(v, &["name"]);
    if universe_id == 0 || name == "[TITLE UNAVAILABLE]" {
        return None;
    }
    let creator = v.get("creator").cloned().unwrap_or(Value::Null);
    Some(GameDetail {
        universe_id,
        place_id: num(v, &["rootPlaceId"]),
        name,
        description: text(v, &["description"]),
        creator_name: text(&creator, &["name"]),
        creator_verified: creator.get("hasVerifiedBadge").and_then(Value::as_bool).unwrap_or(false),
        playing: num(v, &["playing"]),
        visits: num(v, &["visits"]),
        max_players: num(v, &["maxPlayers"]),
        favorites: num(v, &["favoritedCount"]),
        genre: text(v, &["genre_l1", "genre"]),
        created: text(v, &["created"]),
        updated: text(v, &["updated"]),
    })
}

/// Details zu mehreren Spielen. Roblox erlaubt höchstens 50 IDs je Abruf.
#[tauri::command]
pub async fn game_details(state: State<'_, AppState>, universe_ids: Vec<u64>) -> ApiResult<Vec<GameDetail>> {
    fetch_details(&state, &universe_ids).await
}

pub async fn fetch_details(state: &AppState, universe_ids: &[u64]) -> ApiResult<Vec<GameDetail>> {
    let mut out = Vec::new();
    for chunk in universe_ids.chunks(50) {
        let url = format!("https://games.roblox.com/v1/games?universeIds={}", ids(chunk));
        let v = get_json(state, &url).await?;
        out.extend(v.get("data").and_then(Value::as_array).into_iter().flatten().filter_map(detail_from));
    }
    Ok(out)
}

// ---------- Bewertung ----------

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Votes {
    pub up_votes: u64,
    pub down_votes: u64,
}

/// Daumen hoch und runter je Spiel (öffentlich, ohne Login).
#[tauri::command]
pub async fn game_votes(state: State<'_, AppState>, universe_ids: Vec<u64>) -> ApiResult<HashMap<u64, Votes>> {
    let mut out = HashMap::new();
    for chunk in universe_ids.chunks(50) {
        let url = format!("https://games.roblox.com/v1/games/votes?universeIds={}", ids(chunk));
        let v = get_json(&state, &url).await?;
        for d in v.get("data").and_then(Value::as_array).into_iter().flatten() {
            let id = num(d, &["id", "universeId"]);
            if id != 0 {
                out.insert(id, Votes { up_votes: num(d, &["upVotes"]), down_votes: num(d, &["downVotes"]) });
            }
        }
    }
    Ok(out)
}

// ---------- Abzeichen, Game-Pässe, Events ----------

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Badge {
    pub id: u64,
    pub name: String,
    pub description: String,
    pub icon_url: Option<String>,
    /// Wie oft insgesamt vergeben
    pub awarded: u64,
    /// Anteil der Spieler von gestern, die es bekommen haben (0 bis 100)
    pub win_rate: Option<f64>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GamePass {
    pub id: u64,
    pub name: String,
    /// Preis in Robux; `None` = nicht im Verkauf
    pub price: Option<u64>,
    pub icon_url: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameEvent {
    pub id: String,
    pub title: String,
    pub subtitle: String,
    /// ISO-Zeiten wie von Roblox geliefert (UTC)
    pub start: String,
    pub end: String,
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameExtras {
    pub badges: Vec<Badge>,
    pub passes: Vec<GamePass>,
    pub events: Vec<GameEvent>,
}

/// Bild-URLs aus einer Thumbnail-Antwort (`targetId` → `imageUrl`). Fehler ergeben eine leere Karte.
async fn thumb_map(state: &AppState, url: &str) -> HashMap<u64, String> {
    let Ok(v) = get_json(state, url).await else { return HashMap::new() };
    v.get("data")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|d| text(d, &["state"]) == "Completed")
        .map(|d| (num(d, &["targetId"]), text(d, &["imageUrl"])))
        .filter(|(id, u)| *id != 0 && !u.is_empty())
        .collect()
}

async fn fetch_badges(state: &AppState, universe_id: u64) -> Vec<Badge> {
    let url = format!("https://badges.roblox.com/v1/universes/{universe_id}/badges?limit=100&sortOrder=Asc");
    let Ok(v) = get_json(state, &url).await else { return Vec::new() };
    let mut out: Vec<Badge> = v
        .get("data")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|b| b.get("enabled").and_then(Value::as_bool).unwrap_or(true))
        .filter_map(|b| {
            let id = num(b, &["id"]);
            let stats = b.get("statistics").cloned().unwrap_or(Value::Null);
            (id != 0).then(|| Badge {
                id,
                name: text(b, &["displayName", "name"]),
                description: text(b, &["displayDescription", "description"]),
                icon_url: None,
                awarded: num(&stats, &["awardedCount"]),
                // Roblox liefert einen Anteil von 0 bis 1.
                win_rate: stats.get("winRatePercentage").and_then(Value::as_f64).map(|r| r * 100.0),
            })
        })
        .collect();
    if !out.is_empty() {
        let list: Vec<u64> = out.iter().map(|b| b.id).collect();
        let icons = thumb_map(
            state,
            &format!("https://thumbnails.roblox.com/v1/badges/icons?badgeIds={}&size=150x150&format=Png&isCircular=false", ids(&list)),
        )
        .await;
        for b in &mut out {
            b.icon_url = icons.get(&b.id).cloned();
        }
    }
    out
}

/// Game-Pässe. Roblox hat 2026 alte Endpunkte abgeschaltet: zuerst der neue unter
/// `apis.roblox.com`, dann der alte. Feldnamen unbestätigt, darum mehrere Namen.
async fn fetch_passes(state: &AppState, universe_id: u64) -> Vec<GamePass> {
    let urls = [
        format!("https://apis.roblox.com/game-passes/v1/universes/{universe_id}/game-passes?passView=Full&pageSize=100"),
        format!("https://games.roblox.com/v1/games/{universe_id}/game-passes?limit=100&sortOrder=Asc"),
    ];
    let mut list: Vec<Value> = Vec::new();
    for url in &urls {
        if let Ok(v) = get_json(state, url).await {
            if let Some(a) = v.get("gamePasses").or_else(|| v.get("data")).and_then(Value::as_array) {
                list = a.clone();
                break;
            }
        }
    }
    let mut out: Vec<GamePass> = list
        .iter()
        .filter_map(|p| {
            let id = num(p, &["id", "gamePassId"]);
            let name = text(p, &["displayName", "name"]);
            let for_sale = p.get("isForSale").and_then(Value::as_bool).unwrap_or(true);
            let price = ["price", "priceInRobux"]
                .iter()
                .find_map(|k| p.get(*k).and_then(Value::as_u64))
                .or_else(|| p.get("priceInformation")?.get("defaultPriceInRobux")?.as_u64());
            (id != 0 && !name.is_empty()).then(|| GamePass { id, name, price: price.filter(|_| for_sale), icon_url: None })
        })
        .collect();
    if !out.is_empty() {
        let list: Vec<u64> = out.iter().map(|p| p.id).collect();
        let icons = thumb_map(
            state,
            &format!("https://thumbnails.roblox.com/v1/game-passes?gamePassIds={}&size=150x150&format=Png&isCircular=false", ids(&list)),
        )
        .await;
        for p in &mut out {
            p.icon_url = icons.get(&p.id).cloned();
        }
    }
    out
}

/// Events eines Spiels. Unbestätigt (Pfad und Felder), verlangt eventuell eine Anmeldung:
/// dann bleibt die Liste leer.
async fn fetch_events(state: &AppState, universe_id: u64) -> Vec<GameEvent> {
    let url = format!("https://apis.roblox.com/virtual-events/v1/universes/{universe_id}/virtual-events");
    let Ok(v) = get_json(state, &url).await else { return Vec::new() };
    v.get("data")
        .or_else(|| v.get("gameEvents"))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|e| !matches!(text(e, &["eventStatus", "status"]).to_ascii_lowercase().as_str(), "cancelled" | "canceled" | "moderated"))
        .filter_map(|e| {
            let time = e.get("eventTime").cloned().unwrap_or(Value::Null);
            let pick = |a: &str, b: &str| {
                let t = text(&time, &[a]);
                if t.is_empty() { text(e, &[b]) } else { t }
            };
            let title = text(e, &["displayTitle", "title"]);
            let id = match e.get("id") {
                Some(Value::String(s)) => s.clone(),
                Some(Value::Number(n)) => n.to_string(),
                _ => String::new(),
            };
            (!title.is_empty()).then(|| GameEvent {
                id,
                title,
                subtitle: text(e, &["displaySubtitle", "subtitle"]),
                start: pick("startUtc", "startTime"),
                end: pick("endUtc", "endTime"),
            })
        })
        .collect()
}

/// Abzeichen, Game-Pässe und Events für die Spielseite. Jeder Teil darf scheitern
/// und ist dann leer; die Spielseite hängt nicht davon ab.
#[tauri::command]
pub async fn game_extras(state: State<'_, AppState>, universe_id: u64) -> Result<GameExtras, ()> {
    let (badges, passes, events) =
        tokio::join!(fetch_badges(&state, universe_id), fetch_passes(&state, universe_id), fetch_events(&state, universe_id));
    Ok(GameExtras { badges, passes, events })
}

/// Universe-ID zu einer Place-ID.
#[tauri::command]
pub async fn universe_of_place(state: State<'_, AppState>, place_id: u64) -> ApiResult<Option<u64>> {
    let url = format!("https://apis.roblox.com/universes/v1/places/{place_id}/universe");
    let v = get_json(&state, &url).await?;
    Ok(v.get("universeId").and_then(Value::as_u64))
}

// ---------- Bilder ----------

/// Bild-URLs je Universe. `kind`: "icon" (quadratisch) oder "banner" (16:9).
/// URLs werden einen Tag lang zwischengespeichert.
#[tauri::command]
pub async fn thumbnails(state: State<'_, AppState>, universe_ids: Vec<u64>, kind: String) -> ApiResult<HashMap<u64, String>> {
    let banner = kind == "banner";
    let key = if banner { "thumbs-banner" } else { "thumbs-icon" };
    let path = cache_path(&state, key);
    let mut cache: HashMap<u64, (String, u64)> = read_json(&path).unwrap_or_default();
    let fresh_after = now().saturating_sub(24 * 3600);
    let missing: Vec<u64> = universe_ids
        .iter()
        .copied()
        .filter(|id| cache.get(id).map(|(_, t)| *t < fresh_after).unwrap_or(true))
        .collect();
    for chunk in missing.chunks(50) {
        let url = if banner {
            format!(
                "https://thumbnails.roblox.com/v1/games/multiget/thumbnails?universeIds={}&countPerUniverse=1&defaults=false&size=768x432&format=Webp&isCircular=false",
                ids(chunk)
            )
        } else {
            format!(
                "https://thumbnails.roblox.com/v1/games/icons?universeIds={}&returnPolicy=PlaceHolder&size=256x256&format=Webp&isCircular=false",
                ids(chunk)
            )
        };
        let v = match get_json(&state, &url).await {
            Ok(v) => v,
            Err(e) if !cache.is_empty() => {
                // Bilder sind nicht kritisch: mit dem Cache weitermachen.
                eprintln!("Bilder: {}", e.message);
                break;
            }
            Err(e) => return Err(e),
        };
        for d in v.get("data").and_then(Value::as_array).into_iter().flatten() {
            let (id, url) = if banner {
                let id = num(d, &["universeId"]);
                let url = d
                    .get("thumbnails")
                    .and_then(Value::as_array)
                    .and_then(|t| t.iter().find(|x| text(x, &["state"]) == "Completed"))
                    .map(|x| text(x, &["imageUrl"]))
                    .unwrap_or_default();
                (id, url)
            } else {
                let ok = text(d, &["state"]) == "Completed";
                (num(d, &["targetId"]), if ok { text(d, &["imageUrl"]) } else { String::new() })
            };
            if id != 0 && !url.is_empty() {
                cache.insert(id, (url, now()));
            }
        }
    }
    if !missing.is_empty() {
        let _ = write_json(&path, &cache);
    }
    Ok(universe_ids
        .into_iter()
        .filter_map(|id| cache.get(&id).map(|(u, _)| (id, u.clone())))
        .collect())
}

/// Alle Vorschaubilder eines Spiels (16:9), in der Reihenfolge von Roblox.
#[tauri::command]
pub async fn game_media(state: State<'_, AppState>, universe_id: u64) -> ApiResult<Vec<String>> {
    let url = format!(
        "https://thumbnails.roblox.com/v1/games/multiget/thumbnails?universeIds={universe_id}&countPerUniverse=25&defaults=false&size=768x432&format=Webp&isCircular=false"
    );
    let v = get_json(&state, &url).await?;
    Ok(v.get("data")
        .and_then(Value::as_array)
        .and_then(|d| d.first())
        .and_then(|d| d.get("thumbnails")?.as_array().cloned())
        .unwrap_or_default()
        .iter()
        .filter(|t| text(t, &["state"]) == "Completed")
        .map(|t| text(t, &["imageUrl"]))
        .filter(|u| !u.is_empty())
        .collect())
}

/// Avatar-Köpfe zu den Spieler-Kennungen eines Servers (ohne Login).
/// Roblox verrät dabei keine Namen; die Bilder kommen über den Sammel-Endpunkt `thumbnails/v1/batch`.
#[tauri::command]
pub async fn server_avatars(state: State<'_, AppState>, tokens: Vec<String>) -> ApiResult<Vec<String>> {
    let mut out = Vec::new();
    for chunk in tokens.chunks(100).take(2) {
        let body: Vec<Value> = chunk
            .iter()
            .map(|t| {
                serde_json::json!({
                    "requestId": format!("0:{t}:AvatarHeadshot:150x150:png:regular"),
                    "type": "AvatarHeadShot",
                    "targetId": 0,
                    "token": t,
                    "format": "png",
                    "size": "150x150"
                })
            })
            .collect();
        let resp = state.http.post("https://thumbnails.roblox.com/v1/batch").json(&body).send().await?;
        if resp.status().as_u16() == 429 {
            return Err(ApiError::new("RATE_LIMIT", "30"));
        }
        if !resp.status().is_success() {
            return Err(ApiError::new("HTTP", resp.status().as_u16().to_string()));
        }
        let v: Value = resp.json().await?;
        out.extend(
            v.get("data")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter(|d| text(d, &["state"]) == "Completed")
                .map(|d| text(d, &["imageUrl"]))
                .filter(|u| !u.is_empty()),
        );
    }
    Ok(out)
}

// ---------- Serverliste ----------

/// Eine Seite öffentlicher Server. `order`: "Asc" (wenige Spieler zuerst) oder "Desc".
#[tauri::command]
pub async fn servers(
    state: State<'_, AppState>,
    place_id: u64,
    cursor: Option<String>,
    order: Option<String>,
    exclude_full: bool,
) -> ApiResult<ServerPage> {
    let order = match order.as_deref() {
        Some("Asc") => "Asc",
        _ => "Desc",
    };
    let mut url = format!(
        "https://games.roblox.com/v1/games/{place_id}/servers/Public?sortOrder={order}&limit=100&excludeFullGames={exclude_full}"
    );
    if let Some(c) = cursor.filter(|c| !c.is_empty()) {
        url.push_str("&cursor=");
        url.push_str(&urlencode(&c));
    }
    let v = get_json(&state, &url).await?;
    let servers = v
        .get("data")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|s| {
            let id = text(s, &["id"]);
            if id.is_empty() {
                return None;
            }
            Some(Server {
                id,
                playing: num(s, &["playing"]),
                max_players: num(s, &["maxPlayers"]),
                ping: s.get("ping").and_then(Value::as_u64),
                fps: s.get("fps").and_then(Value::as_f64),
                player_tokens: s
                    .get("playerTokens")
                    .and_then(Value::as_array)
                    .map(|a| a.iter().filter_map(|t| t.as_str().map(String::from)).collect())
                    .unwrap_or_default(),
            })
        })
        .collect();
    Ok(ServerPage {
        servers,
        next_cursor: v.get("nextPageCursor").and_then(Value::as_str).map(String::from),
    })
}

/// Spieler auf einem bestimmten Server (für Discord). Sucht höchstens drei Seiten
/// (300 Server, volle zuerst). Bei großen Spielen kann der Server fehlen: dann `None`.
pub async fn server_count(state: &AppState, place_id: u64, job_id: &str) -> Option<(u64, u64)> {
    let mut cursor: Option<String> = None;
    for _ in 0..3 {
        let mut url = format!("https://games.roblox.com/v1/games/{place_id}/servers/Public?sortOrder=Desc&limit=100");
        if let Some(c) = &cursor {
            url.push_str("&cursor=");
            url.push_str(&urlencode(c));
        }
        let v = get_json(state, &url).await.ok()?;
        let list = v.get("data").and_then(Value::as_array)?;
        if let Some(s) = list.iter().find(|s| text(s, &["id"]).eq_ignore_ascii_case(job_id)) {
            return Some((num(s, &["playing"]), num(s, &["maxPlayers"])));
        }
        cursor = Some(v.get("nextPageCursor").and_then(Value::as_str)?.to_string());
    }
    None
}

fn urlencode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

// ---------- Kurzinfo (für Discord und Verlauf) ----------

/// Name, Ersteller und Icon eines Spiels; zwischengespeichert in `cache/briefs.json`.
pub async fn game_brief(state: &AppState, universe_id: u64) -> Option<GameBrief> {
    let path = cache_path(state, "briefs");
    let mut all: HashMap<u64, GameBrief> = read_json(&path).unwrap_or_default();
    if let Some(b) = all.get(&universe_id) {
        return Some(b.clone());
    }
    let d = fetch_details(state, &[universe_id]).await.ok()?.into_iter().next()?;
    let icon = {
        let url = format!(
            "https://thumbnails.roblox.com/v1/games/icons?universeIds={universe_id}&returnPolicy=PlaceHolder&size=512x512&format=Png&isCircular=false"
        );
        get_json(state, &url)
            .await
            .ok()
            .and_then(|v| v.get("data")?.as_array()?.first().map(|x| text(x, &["imageUrl"])))
            .filter(|u| !u.is_empty())
    };
    let b = GameBrief {
        universe_id,
        place_id: d.place_id,
        name: d.name,
        creator: d.creator_name,
        icon_url: icon,
    };
    all.insert(universe_id, b.clone());
    let _ = write_json(&path, &all);
    Some(b)
}

/// Wie `game_brief`, aber ausgehend von einer Place-ID.
pub async fn game_brief_by_place(state: &AppState, place_id: u64) -> Option<GameBrief> {
    let url = format!("https://apis.roblox.com/universes/v1/places/{place_id}/universe");
    let uid = get_json(state, &url).await.ok()?.get("universeId")?.as_u64()?;
    game_brief(state, uid).await
}
