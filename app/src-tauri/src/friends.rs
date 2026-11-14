//! Freunde, die gerade online sind, und was sie spielen.
//!
//! Freundesliste: `friends.roblox.com/v1/users/{id}/friends` (dokumentiert).
//! Status: `presence.roblox.com/v1/presence/users` (dokumentiert). Ohne Cookie zeigt Roblox
//! nur online/offline; mit Cookie auch das Spiel, sofern der Freund es freigibt.
//! Das Cookie geht nur an `*.roblox.com` und bleibt im Rust-Teil.

use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde::Serialize;
use serde_json::{json, Value};
use tauri::State;

use crate::state::AppState;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Friend {
    pub user_id: u64,
    pub name: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    /// "online", "spielt" oder "studio"
    pub status: String,
    /// Spielname laut Roblox (fehlt, wenn der Freund es verbirgt)
    pub game_name: Option<String>,
    pub place_id: Option<u64>,
    pub universe_id: Option<u64>,
    /// Server-ID; nur gesetzt, wenn Roblox den Beitritt erlaubt
    pub job_id: Option<String>,
}

/// Abrufe höchstens alle 30 Sekunden (Rate-Limit von presence).
const FRISCH: Duration = Duration::from_secs(30);
static LETZTE: Mutex<Option<(Instant, u64, Vec<Friend>)>> = Mutex::new(None);

fn u(v: &Value, k: &str) -> Option<u64> {
    v.get(k).and_then(Value::as_u64).filter(|n| *n > 0)
}
fn s(v: &Value, k: &str) -> String {
    v.get(k).and_then(Value::as_str).unwrap_or("").to_string()
}

/// POST mit Cookie. Roblox verlangt bei POST ein CSRF-Token: Die erste Antwort (403)
/// liefert es im Kopf `x-csrf-token`, dann einmal neu senden.
pub(crate) async fn post_auth(state: &AppState, url: &str, body: &Value, cookie: Option<&str>) -> Option<Value> {
    let mut token: Option<String> = None;
    for _ in 0..2 {
        let mut req = state.http.post(url).json(body);
        if let Some(c) = cookie {
            req = req.header("Cookie", format!(".ROBLOSECURITY={c}"));
        }
        if let Some(t) = &token {
            req = req.header("x-csrf-token", t);
        }
        let r = req.send().await.ok()?;
        if r.status().as_u16() == 403 {
            token = r.headers().get("x-csrf-token").and_then(|h| h.to_str().ok()).map(String::from);
            if token.is_some() {
                continue;
            }
        }
        if !r.status().is_success() {
            return None;
        }
        return r.json().await.ok();
    }
    None
}

pub(crate) async fn get(state: &AppState, url: &str) -> Option<Value> {
    let r = state.http.get(url).send().await.ok()?;
    if !r.status().is_success() {
        return None;
    }
    r.json().await.ok()
}

/// Online-Freunde des angemeldeten Kontos, spielende zuerst. Leer ohne Anmeldung.
#[tauri::command]
pub async fn friends_online(state: State<'_, AppState>, fresh: Option<bool>) -> Result<Vec<Friend>, String> {
    let Some(me) = crate::auth::angemeldet_id() else { return Ok(Vec::new()) };
    if !fresh.unwrap_or(false) {
        if let Some((t, id, list)) = LETZTE.lock().ok().and_then(|g| g.clone()) {
            if id == me && t.elapsed() < FRISCH {
                return Ok(list);
            }
        }
    }
    let cookie = crate::auth::cookie();

    // 1. Freundesliste (IDs und, je nach Roblox-Version, Namen)
    let liste = get(&state, &format!("https://friends.roblox.com/v1/users/{me}/friends"))
        .await
        .ok_or("NETZ|Freundesliste nicht erreichbar.")?;
    let freunde: Vec<(u64, String, String)> = liste
        .get("data")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|f| Some((u(f, "id")?, s(f, "name"), s(f, "displayName"))))
        .collect();
    if freunde.is_empty() {
        return Ok(Vec::new());
    }

    // 2. Status in Paketen zu 100
    let mut online: Vec<Friend> = Vec::new();
    for paket in freunde.chunks(100) {
        let ids: Vec<u64> = paket.iter().map(|f| f.0).collect();
        let Some(v) = post_auth(&state, "https://presence.roblox.com/v1/presence/users", &json!({ "userIds": ids }), cookie.as_deref()).await
        else {
            continue;
        };
        for p in v.get("userPresences").and_then(Value::as_array).into_iter().flatten() {
            let status = match p.get("userPresenceType").and_then(Value::as_u64) {
                Some(1) => "online",
                Some(2) => "spielt",
                Some(3) => "studio",
                _ => continue,
            };
            let Some(id) = u(p, "userId") else { continue };
            let (name, display) = paket.iter().find(|f| f.0 == id).map(|f| (f.1.clone(), f.2.clone())).unwrap_or_default();
            let ort = s(p, "lastLocation");
            online.push(Friend {
                user_id: id,
                name,
                display_name: display,
                avatar_url: None,
                status: status.into(),
                game_name: (status == "spielt" && !ort.is_empty()).then_some(ort),
                place_id: u(p, "rootPlaceId").or_else(|| u(p, "placeId")),
                universe_id: u(p, "universeId"),
                job_id: Some(s(p, "gameId")).filter(|g| g.len() == 36),
            });
        }
    }

    // 3. Fehlende Namen (neuere Antworten der Freundesliste enthalten nur IDs)
    let ohne: Vec<u64> = online.iter().filter(|f| f.name.is_empty()).map(|f| f.user_id).collect();
    if !ohne.is_empty() {
        if let Some(v) = post_auth(&state, "https://users.roblox.com/v1/users", &json!({ "userIds": ohne, "excludeBannedUsers": true }), None).await {
            for n in v.get("data").and_then(Value::as_array).into_iter().flatten() {
                if let Some(f) = online.iter_mut().find(|f| Some(f.user_id) == u(n, "id")) {
                    f.name = s(n, "name");
                    f.display_name = s(n, "displayName");
                }
            }
        }
    }

    // 4. Avatare (ein Abruf für alle)
    if !online.is_empty() {
        let ids: Vec<String> = online.iter().take(100).map(|f| f.user_id.to_string()).collect();
        let url = format!(
            "https://thumbnails.roblox.com/v1/users/avatar-headshot?userIds={}&size=48x48&format=Png&isCircular=false",
            ids.join(",")
        );
        if let Some(v) = get(&state, &url).await {
            for b in v.get("data").and_then(Value::as_array).into_iter().flatten() {
                if let Some(f) = online.iter_mut().find(|f| Some(f.user_id) == u(b, "targetId")) {
                    f.avatar_url = Some(s(b, "imageUrl")).filter(|x| x.starts_with("https://"));
                }
            }
        }
    }

    let rang = |f: &Friend| match f.status.as_str() {
        "spielt" => 0,
        "studio" => 1,
        _ => 2,
    };
    online.sort_by(|a, b| rang(a).cmp(&rang(b)).then_with(|| a.display_name.to_lowercase().cmp(&b.display_name.to_lowercase())));
    if let Ok(mut g) = LETZTE.lock() {
        *g = Some((Instant::now(), me, online.clone()));
    }
    Ok(online)
}

/// Status eines beliebigen Kontos (für die Profilseite). Mit Cookie zeigt Roblox das Spiel,
/// sofern die Person es für dich freigibt; sonst nur online/offline.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Presence {
    /// "offline", "online", "spielt" oder "studio"
    pub status: String,
    pub game_name: Option<String>,
    pub place_id: Option<u64>,
    pub universe_id: Option<u64>,
    pub job_id: Option<String>,
}

pub async fn presence_of(state: &AppState, user_id: u64) -> Option<Presence> {
    let cookie = crate::auth::cookie();
    let v = post_auth(state, "https://presence.roblox.com/v1/presence/users", &json!({ "userIds": [user_id] }), cookie.as_deref()).await?;
    let p = v.get("userPresences")?.as_array()?.first()?.clone();
    let status = match p.get("userPresenceType").and_then(Value::as_u64) {
        Some(1) => "online",
        Some(2) => "spielt",
        Some(3) => "studio",
        _ => "offline",
    };
    let ort = s(&p, "lastLocation");
    Some(Presence {
        status: status.into(),
        game_name: (status == "spielt" && !ort.is_empty()).then_some(ort),
        place_id: u(&p, "rootPlaceId").or_else(|| u(&p, "placeId")),
        universe_id: u(&p, "universeId"),
        job_id: Some(s(&p, "gameId")).filter(|g| g.len() == 36),
    })
}
