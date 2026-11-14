//! Spieler suchen, Profil und öffentliches Inventar.
//!
//! Alle Abrufe laufen ohne Cookie. Nur der Status (`friends::presence_of`) schickt es mit,
//! damit Roblox das Spiel zeigt, wenn die Person es freigibt.
//!
//! Dokumentiert: `users/search`, `usernames/users`, `users/{id}`, Zähler für Freunde und
//! Follower, `can-view-inventory`, Abzeichen eines Kontos, `awarded-dates`, Avatar-Bilder.
//! Unbestätigt (Feldnamen aus Erinnerung): `avatar/v1/users/{id}/avatar` (was jemand trägt),
//! `games/v2/users/{id}/games` (eigene Spiele), `inventory/v2/.../inventory` (Kategorien),
//! `awarder`/`awardingUniverse` in der Abzeichenliste. Diese Teile scheitern still.

use std::collections::{HashMap, HashSet};

use serde::Serialize;
use serde_json::{json, Value};
use tauri::State;

use crate::friends::{post_auth, presence_of, Presence};
use crate::roblox_api::{
    badge_icons, fetch_details, get_json, ids, num, text, thumb_map, universe_badges, urlencode, ApiError, GameCard,
};
use crate::state::AppState;

type ApiResult<T> = Result<T, ApiError>;

// ---------- Suche ----------

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserCard {
    pub user_id: u64,
    pub name: String,
    pub display_name: String,
    pub verified: bool,
    pub avatar_url: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserPage {
    pub users: Vec<UserCard>,
    pub next_cursor: Option<String>,
}

fn user_card(v: &Value) -> Option<UserCard> {
    let user_id = num(v, &["id", "userId"]);
    let name = text(v, &["name", "username"]);
    (user_id != 0 && !name.is_empty()).then(|| UserCard {
        user_id,
        display_name: Some(text(v, &["displayName"])).filter(|d| !d.is_empty()).unwrap_or_else(|| name.clone()),
        name,
        verified: v.get("hasVerifiedBadge").and_then(Value::as_bool).unwrap_or(false),
        avatar_url: None,
    })
}

/// Benutzer-ID aus Eingabe: reine Zahl oder Profil-Link (`/users/123/profile`).
fn user_id_from_input(q: &str) -> Option<u64> {
    if let Ok(n) = q.parse::<u64>() {
        return Some(n);
    }
    let rest = q.split("/users/").nth(1)?;
    rest.split(|c: char| !c.is_ascii_digit()).next()?.parse().ok()
}

/// Kopfbilder (rund zugeschnitten macht die Oberfläche) für bis zu 100 Konten.
async fn headshots(state: &AppState, user_ids: &[u64], size: &str) -> HashMap<u64, String> {
    let mut out = HashMap::new();
    for chunk in user_ids.chunks(100) {
        out.extend(
            thumb_map(
                state,
                &format!(
                    "https://thumbnails.roblox.com/v1/users/avatar-headshot?userIds={}&size={size}&format=Png&isCircular=false",
                    ids(chunk)
                ),
            )
            .await,
        );
    }
    out
}

/// Spieler suchen. Erste Seite: genauer Name (oder ID, Profil-Link) zuerst, dann die
/// Stichwortsuche. Die Stichwortsuche bremst Roblox stark; scheitert sie, bleibt der
/// genaue Treffer.
#[tauri::command]
pub async fn user_search(state: State<'_, AppState>, query: String, cursor: Option<String>) -> ApiResult<UserPage> {
    let q = query.trim().trim_start_matches('@').trim().to_string();
    if q.is_empty() {
        return Ok(UserPage { users: Vec::new(), next_cursor: None });
    }
    let mut users: Vec<UserCard> = Vec::new();

    if cursor.is_none() {
        if let Some(id) = user_id_from_input(&q) {
            if let Ok(v) = get_json(&state, &format!("https://users.roblox.com/v1/users/{id}")).await {
                users.extend(user_card(&v));
            }
        }
        // Genauer Benutzername (auch ein früherer Name findet das Konto)
        if q.len() >= 3 && q.len() <= 20 && q.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            if let Some(v) = post_auth(
                &state,
                "https://users.roblox.com/v1/usernames/users",
                &json!({ "usernames": [q], "excludeBannedUsers": true }),
                None,
            )
            .await
            {
                users.extend(v.get("data").and_then(Value::as_array).into_iter().flatten().filter_map(user_card));
            }
        }
    }

    let mut next_cursor = None;
    let mut url = format!("https://users.roblox.com/v1/users/search?keyword={}&limit=25", urlencode(&q));
    if let Some(c) = &cursor {
        url.push_str(&format!("&cursor={}", urlencode(c)));
    }
    match get_json(&state, &url).await {
        Ok(v) => {
            let mut seen: HashSet<u64> = users.iter().map(|u| u.user_id).collect();
            for u in v.get("data").and_then(Value::as_array).into_iter().flatten().filter_map(user_card) {
                if seen.insert(u.user_id) {
                    users.push(u);
                }
            }
            next_cursor = v.get("nextPageCursor").and_then(Value::as_str).map(String::from);
        }
        Err(e) if users.is_empty() => return Err(e),
        Err(_) => {}
    }

    let bilder = headshots(&state, &users.iter().map(|u| u.user_id).collect::<Vec<_>>(), "150x150").await;
    for u in &mut users {
        u.avatar_url = bilder.get(&u.user_id).cloned();
    }
    Ok(UserPage { users, next_cursor })
}

// ---------- Profil ----------

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WornItem {
    pub id: u64,
    pub name: String,
    /// Asset-Typ laut Roblox (englisch, z. B. "Hat", "Shirt"); die Oberfläche übersetzt
    pub kind: String,
    pub icon_url: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserProfile {
    pub user_id: u64,
    pub name: String,
    pub display_name: String,
    pub description: String,
    /// ISO-Zeit der Anmeldung bei Roblox
    pub created: String,
    pub verified: bool,
    pub banned: bool,
    pub friends: Option<u64>,
    pub followers: Option<u64>,
    pub following: Option<u64>,
    pub avatar_url: Option<String>,
    pub headshot_url: Option<String>,
    pub presence: Option<Presence>,
    /// Inventar öffentlich? `None` = unbekannt
    pub inventory_public: Option<bool>,
    pub wearing: Vec<WornItem>,
    pub games: Vec<GameCard>,
}

async fn count(state: &AppState, url: &str) -> Option<u64> {
    get_json(state, url).await.ok()?.get("count")?.as_u64()
}

/// Bild-URLs zu Assets (Kleidung, Accessoires, Emotes), in Paketen zu 100.
async fn asset_icons(state: &AppState, asset_ids: &[u64]) -> HashMap<u64, String> {
    let mut out = HashMap::new();
    for chunk in asset_ids.chunks(100) {
        out.extend(
            thumb_map(
                state,
                &format!(
                    "https://thumbnails.roblox.com/v1/assets?assetIds={}&returnPolicy=PlaceHolder&size=150x150&format=Png&isCircular=false",
                    ids(chunk)
                ),
            )
            .await,
        );
    }
    out
}

/// Was jemand gerade trägt. Körperteile lässt die Liste weg.
async fn wearing(state: &AppState, user_id: u64) -> Vec<WornItem> {
    const KOERPER: [&str; 7] = ["Head", "Torso", "LeftArm", "RightArm", "LeftLeg", "RightLeg", "DynamicHead"];
    let Ok(v) = get_json(state, &format!("https://avatar.roblox.com/v1/users/{user_id}/avatar")).await else {
        return Vec::new();
    };
    let mut out: Vec<WornItem> = v
        .get("assets")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|a| {
            let id = num(a, &["id"]);
            let kind = a.get("assetType").map(|t| text(t, &["name"])).unwrap_or_default();
            (id != 0 && !KOERPER.contains(&kind.as_str())).then(|| WornItem { id, name: text(a, &["name"]), kind, icon_url: None })
        })
        .collect();
    out.extend(v.get("emotes").and_then(Value::as_array).into_iter().flatten().filter_map(|e| {
        let id = num(e, &["assetId", "id"]);
        (id != 0).then(|| WornItem { id, name: text(e, &["assetName", "name"]), kind: "Emote".into(), icon_url: None })
    }));
    out.truncate(40);
    let icons = asset_icons(state, &out.iter().map(|w| w.id).collect::<Vec<_>>()).await;
    for w in &mut out {
        w.icon_url = icons.get(&w.id).cloned();
    }
    out
}

/// Öffentliche Spiele eines Kontos, mit Spielerzahlen.
async fn own_games(state: &AppState, user_id: u64) -> Vec<GameCard> {
    let url = format!("https://games.roblox.com/v2/users/{user_id}/games?accessFilter=2&limit=50&sortOrder=Desc");
    let Ok(v) = get_json(state, &url).await else { return Vec::new() };
    let liste: Vec<u64> = v
        .get("data")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(|g| num(g, &["id", "universeId"]))
        .filter(|id| *id != 0)
        .collect();
    let Ok(details) = fetch_details(state, &liste).await else { return Vec::new() };
    let mut out: Vec<GameCard> = details
        .into_iter()
        .map(|d| GameCard { universe_id: d.universe_id, place_id: d.place_id, name: d.name, playing: d.playing, up_votes: 0, down_votes: 0 })
        .collect();
    // Beliebteste zuerst
    out.sort_by_key(|g| std::cmp::Reverse(g.playing));
    out
}

#[tauri::command]
pub async fn user_profile(state: State<'_, AppState>, user_id: u64) -> ApiResult<UserProfile> {
    let st: &AppState = &state;
    let u_basis = format!("https://users.roblox.com/v1/users/{user_id}");
    let u_freunde = format!("https://friends.roblox.com/v1/users/{user_id}/friends/count");
    let u_follower = format!("https://friends.roblox.com/v1/users/{user_id}/followers/count");
    let u_folgt = format!("https://friends.roblox.com/v1/users/{user_id}/followings/count");
    let u_ganz = format!("https://thumbnails.roblox.com/v1/users/avatar?userIds={user_id}&size=420x420&format=Png&isCircular=false");
    let u_inventar = format!("https://inventory.roblox.com/v1/users/{user_id}/can-view-inventory");
    let ich = [user_id];
    let basis = get_json(st, &u_basis);
    let freunde = count(st, &u_freunde);
    let follower = count(st, &u_follower);
    let folgt = count(st, &u_folgt);
    let ganz = thumb_map(st, &u_ganz);
    let kopf = headshots(st, &ich, "150x150");
    let status = presence_of(st, user_id);
    let inventar = async {
        get_json(st, &u_inventar)
            .await
            .ok()
            .and_then(|v| v.get("canView").and_then(Value::as_bool))
    };
    let (basis, freunde, follower, folgt, ganz, kopf, status, inventar, traegt, spiele) = tokio::join!(
        basis,
        freunde,
        follower,
        folgt,
        ganz,
        kopf,
        status,
        inventar,
        wearing(st, user_id),
        own_games(st, user_id)
    );
    let v = basis?;
    let name = text(&v, &["name"]);
    if name.is_empty() {
        return Err(ApiError::new("FORMAT", "Profil ohne Namen"));
    }
    Ok(UserProfile {
        user_id,
        display_name: Some(text(&v, &["displayName"])).filter(|d| !d.is_empty()).unwrap_or_else(|| name.clone()),
        name,
        description: text(&v, &["description"]),
        created: text(&v, &["created"]),
        verified: v.get("hasVerifiedBadge").and_then(Value::as_bool).unwrap_or(false),
        banned: v.get("isBanned").and_then(Value::as_bool).unwrap_or(false),
        friends: freunde,
        followers: follower,
        following: folgt,
        avatar_url: ganz.get(&user_id).cloned(),
        headshot_url: kopf.get(&user_id).cloned(),
        presence: status,
        inventory_public: inventar,
        wearing: traegt,
        games: spiele,
    })
}

// ---------- Abzeichen eines Kontos ----------

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OwnedBadge {
    pub id: u64,
    pub name: String,
    pub description: String,
    pub icon_url: Option<String>,
    /// Spiel, das das Abzeichen vergibt (fehlt, wenn Roblox es nicht verrät)
    pub universe_id: Option<u64>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BadgeGame {
    pub universe_id: u64,
    pub place_id: u64,
    pub name: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BadgePage {
    pub badges: Vec<OwnedBadge>,
    /// Spiele zu den Abzeichen dieser Seite
    pub games: Vec<BadgeGame>,
    pub next_cursor: Option<String>,
}

/// Spiele zu IDs aus `awarder`. Roblox liefert dort je nach Alter des Abzeichens eine
/// Universe- oder eine Place-ID (unbestätigt). Erst als Universe lesen, den Rest als Place.
async fn awarder_games(state: &AppState, raw: &[u64]) -> (Vec<BadgeGame>, HashMap<u64, u64>) {
    let mut map: HashMap<u64, u64> = HashMap::new();
    let mut games: Vec<BadgeGame> = Vec::new();
    if let Ok(d) = fetch_details(state, raw).await {
        for g in d {
            map.insert(g.universe_id, g.universe_id);
            games.push(BadgeGame { universe_id: g.universe_id, place_id: g.place_id, name: g.name });
        }
    }
    let rest: Vec<u64> = raw.iter().copied().filter(|id| !map.contains_key(id)).take(20).collect();
    let mut neu: Vec<u64> = Vec::new();
    for place in rest {
        let url = format!("https://apis.roblox.com/universes/v1/places/{place}/universe");
        if let Some(uid) = get_json(state, &url).await.ok().and_then(|v| v.get("universeId").and_then(Value::as_u64)) {
            map.insert(place, uid);
            if !games.iter().any(|g| g.universe_id == uid) {
                neu.push(uid);
            }
        }
    }
    if !neu.is_empty() {
        if let Ok(d) = fetch_details(state, &neu).await {
            games.extend(d.into_iter().map(|g| BadgeGame { universe_id: g.universe_id, place_id: g.place_id, name: g.name }));
        }
    }
    (games, map)
}

/// Abzeichen eines Kontos, neueste zuerst, 100 je Seite. Nur bei öffentlichem Inventar.
#[tauri::command]
pub async fn user_badges(state: State<'_, AppState>, user_id: u64, cursor: Option<String>) -> ApiResult<BadgePage> {
    let mut url = format!("https://badges.roblox.com/v1/users/{user_id}/badges?limit=100&sortOrder=Desc");
    if let Some(c) = &cursor {
        url.push_str(&format!("&cursor={}", urlencode(c)));
    }
    let v = get_json(&state, &url).await?;
    let mut games_by_universe: HashMap<u64, BadgeGame> = HashMap::new();
    let mut awarder: Vec<u64> = Vec::new();
    let mut badges: Vec<(OwnedBadge, Option<u64>)> = Vec::new();
    for b in v.get("data").and_then(Value::as_array).into_iter().flatten() {
        let id = num(b, &["id"]);
        if id == 0 {
            continue;
        }
        // Neuere Antworten nennen das Spiel direkt.
        let uni = b.get("awardingUniverse").map(|u| (num(u, &["id"]), num(u, &["rootPlaceId"]), text(u, &["name"])));
        let mut universe_id = None;
        let mut roh = None;
        if let Some((uid, pid, name)) = uni.filter(|(uid, _, _)| *uid != 0) {
            universe_id = Some(uid);
            if !name.is_empty() {
                games_by_universe.entry(uid).or_insert(BadgeGame { universe_id: uid, place_id: pid, name });
            }
        } else if let Some(a) = b.get("awarder") {
            let aid = num(a, &["id"]);
            if aid != 0 {
                roh = Some(aid);
                if !awarder.contains(&aid) {
                    awarder.push(aid);
                }
            }
        }
        badges.push((
            OwnedBadge {
                id,
                name: text(b, &["displayName", "name"]),
                description: text(b, &["displayDescription", "description"]),
                icon_url: None,
                universe_id,
            },
            roh,
        ));
    }
    let (gefunden, map) = if awarder.is_empty() { (Vec::new(), HashMap::new()) } else { awarder_games(&state, &awarder).await };
    for g in gefunden {
        games_by_universe.entry(g.universe_id).or_insert(g);
    }
    let icons = badge_icons(&state, &badges.iter().map(|(b, _)| b.id).collect::<Vec<_>>()).await;
    let badges: Vec<OwnedBadge> = badges
        .into_iter()
        .map(|(mut b, roh)| {
            if b.universe_id.is_none() {
                b.universe_id = roh.and_then(|r| map.get(&r).copied());
            }
            b.icon_url = icons.get(&b.id).cloned();
            b
        })
        .collect();
    Ok(BadgePage {
        badges,
        games: games_by_universe.into_values().collect(),
        next_cursor: v.get("nextPageCursor").and_then(Value::as_str).map(String::from),
    })
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameBadge {
    pub id: u64,
    pub name: String,
    pub description: String,
    pub icon_url: Option<String>,
    /// Anteil der Spieler, die es haben (0 bis 100)
    pub win_rate: Option<f64>,
    /// Wann das Konto es bekommen hat; `None` = fehlt noch
    pub awarded_at: Option<String>,
}

/// Alle Abzeichen eines Spiels mit dem Datum, wann das Konto sie bekommen hat.
/// So zeigt der Filter auch die fehlenden, unabhängig davon, wie viele Seiten geladen sind.
#[tauri::command]
pub async fn user_game_badges(state: State<'_, AppState>, user_id: u64, universe_id: u64) -> ApiResult<Vec<GameBadge>> {
    let alle = universe_badges(&state, universe_id, 10).await;
    let mut daten: HashMap<u64, String> = HashMap::new();
    for chunk in alle.chunks(100) {
        let url = format!(
            "https://badges.roblox.com/v1/users/{user_id}/badges/awarded-dates?badgeIds={}",
            ids(&chunk.iter().map(|b| b.id).collect::<Vec<_>>())
        );
        let v = get_json(&state, &url).await?;
        for d in v.get("data").and_then(Value::as_array).into_iter().flatten() {
            let id = num(d, &["badgeId"]);
            let t = text(d, &["awardedDate"]);
            if id != 0 && !t.is_empty() {
                daten.insert(id, t);
            }
        }
    }
    Ok(alle
        .into_iter()
        .map(|b| GameBadge {
            awarded_at: daten.get(&b.id).cloned(),
            id: b.id,
            name: b.name,
            description: b.description,
            icon_url: b.icon_url,
            win_rate: b.win_rate,
        })
        .collect())
}

// ---------- Weitere Kategorien des Inventars ----------

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryPage {
    pub items: Vec<WornItem>,
    pub next_cursor: Option<String>,
}

/// Asset-Typen je Kategorie der Oberfläche.
fn asset_types(kategorie: &str) -> Option<&'static str> {
    Some(match kategorie {
        "accessoires" => "Hat,HairAccessory,FaceAccessory,NeckAccessory,ShoulderAccessory,FrontAccessory,BackAccessory,WaistAccessory",
        "kleidung" => "TShirt,Shirt,Pants,TShirtAccessory,ShirtAccessory,PantsAccessory,JacketAccessory,SweaterAccessory,ShortsAccessory,DressSkirtAccessory,LeftShoeAccessory,RightShoeAccessory",
        "emotes" => "EmoteAnimation",
        "gamepaesse" => "GamePass",
        _ => return None,
    })
}

/// Eine Kategorie des Inventars, neueste zuerst, 100 je Seite. Unbestätigt: Pfad und Felder
/// von `inventory/v2`. Game-Pässe haben eigene Bilder.
#[tauri::command]
pub async fn user_inventory(
    state: State<'_, AppState>,
    user_id: u64,
    category: String,
    cursor: Option<String>,
) -> ApiResult<InventoryPage> {
    let typen = asset_types(&category).ok_or_else(|| ApiError::new("FORMAT", "Unbekannte Kategorie"))?;
    let mut url = format!("https://inventory.roblox.com/v2/users/{user_id}/inventory?assetTypes={typen}&limit=100&sortOrder=Desc");
    if let Some(c) = &cursor {
        url.push_str(&format!("&cursor={}", urlencode(c)));
    }
    let v = get_json(&state, &url).await?;
    let mut items: Vec<WornItem> = v
        .get("data")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|a| {
            let id = num(a, &["assetId", "id", "gamePassId"]);
            let kind = match a.get("assetType") {
                Some(Value::String(s)) => s.clone(),
                Some(t) => text(t, &["name"]),
                None => String::new(),
            };
            (id != 0).then(|| WornItem { id, name: text(a, &["name", "assetName"]), kind, icon_url: None })
        })
        .collect();
    let liste: Vec<u64> = items.iter().map(|i| i.id).collect();
    let icons = if category == "gamepaesse" {
        let mut m = HashMap::new();
        for chunk in liste.chunks(100) {
            m.extend(
                thumb_map(
                    &state,
                    &format!("https://thumbnails.roblox.com/v1/game-passes?gamePassIds={}&size=150x150&format=Png&isCircular=false", ids(chunk)),
                )
                .await,
            );
        }
        m
    } else {
        asset_icons(&state, &liste).await
    };
    for i in &mut items {
        i.icon_url = icons.get(&i.id).cloned();
    }
    Ok(InventoryPage { items, next_cursor: v.get("nextPageCursor").and_then(Value::as_str).map(String::from) })
}

#[cfg(test)]
mod tests {
    use super::user_id_from_input;

    #[test]
    fn id_aus_eingabe() {
        assert_eq!(user_id_from_input("12345"), Some(12345));
        assert_eq!(user_id_from_input("https://www.roblox.com/users/987/profile"), Some(987));
        assert_eq!(user_id_from_input("SomeName"), None);
    }
}
