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

async fn get_json(state: &AppState, url: &str) -> ApiResult<Value> {
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
    Ok(resp.json::<Value>().await?)
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
            })
        })
        .collect();
    Ok(ServerPage {
        servers,
        next_cursor: v.get("nextPageCursor").and_then(Value::as_str).map(String::from),
    })
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
