//! Name und Avatar des Kontos. Die ID kommt aus der Anmeldung bei Brickstrap (Pflicht).
//! Rückfall, falls die Anmeldung noch lädt: `%localappdata%\Roblox\LocalStorage\appStorage.json`
//! (Feldnamen unbestätigt) oder die letzte `userid:` in den Player-Logdateien. Abrufe ohne Cookie.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::State;

use crate::paths;
use crate::state::{read_json, write_json, AppState};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub user_id: u64,
    pub name: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
}

fn from_app_storage() -> Option<u64> {
    let file = paths::roblox_dir()?.join("LocalStorage").join("appStorage.json");
    let v: Value = serde_json::from_str(&std::fs::read_to_string(file).ok()?).ok()?;
    for k in ["UserId", "userId", "user_id"] {
        match v.get(k) {
            Some(Value::String(s)) => {
                if let Ok(n) = s.parse::<u64>() {
                    if n > 0 {
                        return Some(n);
                    }
                }
            }
            Some(Value::Number(n)) => {
                if let Some(n) = n.as_u64().filter(|n| *n > 0) {
                    return Some(n);
                }
            }
            _ => {}
        }
    }
    None
}

fn from_logs() -> Option<u64> {
    let dir = paths::logs_dir()?;
    let mut files: Vec<_> = std::fs::read_dir(dir)
        .ok()?
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().contains("Player"))
        .filter_map(|e| Some((e.metadata().ok()?.modified().ok()?, e.path())))
        .collect();
    files.sort_by_key(|f| std::cmp::Reverse(f.0));
    let re = regex::Regex::new(r"userid:([0-9]+)").ok()?;
    for (_, f) in files.into_iter().take(8) {
        let text = std::fs::read(&f).ok()?;
        let text = String::from_utf8_lossy(&text);
        if let Some(id) = re.captures_iter(&text).last().and_then(|c| c.get(1)?.as_str().parse().ok()) {
            return Some(id);
        }
    }
    None
}

/// Angemeldetes Konto oder `None`, wenn Roblox noch nie gestartet wurde.
#[tauri::command]
pub async fn account_get(state: State<'_, AppState>) -> Result<Option<Account>, String> {
    let Some(id) = crate::auth::angemeldet_id().or_else(from_app_storage).or_else(from_logs) else {
        return Ok(None);
    };
    let cache = state.file("cache").join("account.json");
    if let Some(a) = read_json::<Account>(&cache).filter(|a| a.user_id == id) {
        return Ok(Some(a));
    }
    let user: Value = match state.http.get(format!("https://users.roblox.com/v1/users/{id}")).send().await {
        Ok(r) if r.status().is_success() => r.json().await.unwrap_or(Value::Null),
        _ => Value::Null,
    };
    let avatar = state
        .http
        .get(format!(
            "https://thumbnails.roblox.com/v1/users/avatar-headshot?userIds={id}&size=150x150&format=Png&isCircular=false"
        ))
        .send()
        .await
        .ok();
    let avatar_url = match avatar {
        Some(r) => r
            .json::<Value>()
            .await
            .ok()
            .and_then(|v| v.get("data")?.as_array()?.first()?.get("imageUrl")?.as_str().map(String::from)),
        None => None,
    };
    let text = |k: &str| user.get(k).and_then(Value::as_str).unwrap_or("").to_string();
    let a = Account {
        user_id: id,
        name: text("name"),
        display_name: text("displayName"),
        avatar_url,
    };
    if !a.name.is_empty() {
        let _ = std::fs::create_dir_all(state.file("cache"));
        let _ = write_json(&cache, &a);
    }
    Ok(Some(a))
}
