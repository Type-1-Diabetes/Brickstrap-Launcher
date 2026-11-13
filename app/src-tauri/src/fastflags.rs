//! FastFlags: Allowlist laden, Werte speichern und vor dem Start in
//! `ClientSettings\ClientAppSettings.json` schreiben.
//!
//! Nur Dateien, kein Zugriff auf den Roblox-Prozess. Roblox leert den Ordner bei jedem
//! Update, darum wird vor jedem Start neu geschrieben (nur wenn sich etwas ändert).

use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::paths::player_version_dirs;
use crate::state::{now, read_json, write_json, AppState};

const EMBEDDED_ALLOWLIST: &str = include_str!("../resources/allowlist.json");
const STATE_FILE: &str = "fastflags.json";

// ---------- Allowlist ----------

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FlagOption {
    pub value: String,
    pub label: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AllowFlag {
    pub name: String,
    /// "bool" oder "int"
    pub kind: String,
    pub group: String,
    pub label: String,
    pub hint: String,
    pub min: Option<i64>,
    pub max: Option<i64>,
    pub options: Option<Vec<FlagOption>>,
    pub experimental: bool,
    /// Flags mit gleichem Wert gehören zu einer Gruppe, in der nur einer `True` sein darf.
    pub exclusive: Option<String>,
}

impl Default for AllowFlag {
    fn default() -> Self {
        Self {
            name: String::new(),
            kind: "bool".into(),
            group: String::new(),
            label: String::new(),
            hint: String::new(),
            min: None,
            max: None,
            options: None,
            experimental: false,
            exclusive: None,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
struct AllowFile {
    date: String,
    flags: Vec<AllowFlag>,
}

/// Zuerst `data_dir/allowlist.json` (später aktualisierbar), sonst die eingebaute Liste.
fn load_allowlist(state: &AppState) -> AllowFile {
    let user: Option<AllowFile> = read_json(&state.file("allowlist.json"));
    if let Some(f) = user.filter(|f| !f.flags.is_empty()) {
        return f;
    }
    serde_json::from_str(EMBEDDED_ALLOWLIST).unwrap_or_default()
}

// ---------- Gespeicherter Zustand ----------

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
struct FlagsFile {
    values: BTreeMap<String, String>,
    preset: String,
    applied_to: Vec<String>,
    last_applied: Option<u64>,
    /// Inhalt, den Brickstrap zuletzt geschrieben hat (erkennt „von uns geschrieben“).
    last_content: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FlagsView {
    pub allowlist: Vec<AllowFlag>,
    pub values: BTreeMap<String, String>,
    pub preset: String,
    pub allowlist_date: String,
    pub applied_to: Vec<String>,
    pub last_applied: Option<u64>,
}

#[tauri::command]
pub fn flags_get(state: State<'_, AppState>) -> FlagsView {
    let allow = load_allowlist(&state);
    let saved: FlagsFile = read_json(&state.file(STATE_FILE)).unwrap_or_default();
    FlagsView {
        allowlist: allow.flags,
        values: saved.values,
        preset: saved.preset,
        allowlist_date: allow.date,
        applied_to: saved.applied_to,
        last_applied: saved.last_applied,
    }
}

// ---------- Normalisieren ----------

fn norm_bool(v: &str) -> Option<&'static str> {
    match v.trim().to_ascii_lowercase().as_str() {
        "true" | "1" | "an" | "ja" => Some("True"),
        "false" | "0" | "aus" | "nein" => Some("False"),
        _ => None,
    }
}

/// Wert passend zur Art bringen. `Err` mit deutschem Text, wenn er sich nicht lesen lässt.
fn normalize(flag: &AllowFlag, value: &str) -> Result<String, String> {
    if flag.kind == "bool" {
        return norm_bool(value)
            .map(str::to_string)
            .ok_or_else(|| format!("{}: Erwartet True oder False.", flag.name));
    }
    let n: i64 = value
        .trim()
        .parse()
        .map_err(|_| format!("{}: Erwartet eine ganze Zahl.", flag.name))?;
    let lo = flag.min.unwrap_or(i64::MIN);
    let hi = flag.max.unwrap_or(i64::MAX);
    Ok(n.clamp(lo, hi).to_string())
}

#[tauri::command]
pub fn flags_set(state: State<'_, AppState>, values: BTreeMap<String, String>, preset: String) -> Result<(), String> {
    let allow = load_allowlist(&state);
    let mut clean: BTreeMap<String, String> = BTreeMap::new();
    for (name, value) in values {
        let name = name.trim().to_string();
        if name.is_empty() {
            continue;
        }
        match allow.flags.iter().find(|f| f.name == name) {
            Some(flag) => {
                clean.insert(name, normalize(flag, &value)?);
            }
            // Nicht auf der Allowlist: mitspeichern, aber nie schreiben.
            None => {
                clean.insert(name, value.trim().to_string());
            }
        }
    }
    // Grafik-API: nur eine darf True sein.
    let mut groups: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for f in &allow.flags {
        if let Some(g) = f.exclusive.as_deref() {
            if clean.get(&f.name).map(String::as_str) == Some("True") {
                groups.entry(g).or_default().push(&f.name);
            }
        }
    }
    if groups.values().any(|v| v.len() > 1) {
        return Err("Es darf nur eine Grafik-API aktiv sein.".into());
    }

    let path = state.file(STATE_FILE);
    let mut file: FlagsFile = read_json(&path).unwrap_or_default();
    file.values = clean;
    file.preset = preset.trim().to_string();
    write_json(&path, &file)
}

// ---------- Presets ----------

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Preset {
    pub id: String,
    pub name: String,
    pub description: String,
    pub values: BTreeMap<String, String>,
}

fn map(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
    pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
}

const LOD_NAMES: [&str; 4] = [
    "DFIntCSGLevelOfDetailSwitchingDistance",
    "DFIntCSGLevelOfDetailSwitchingDistanceL12",
    "DFIntCSGLevelOfDetailSwitchingDistanceL23",
    "DFIntCSGLevelOfDetailSwitchingDistanceL34",
];

fn max_fps_values() -> BTreeMap<String, String> {
    let mut v = map(&[
        ("DFIntDebugFRMQualityLevelOverride", "1"),
        ("FIntFRMMaxGrassDistance", "0"),
        ("FIntFRMMinGrassDistance", "0"),
        ("DFFlagTextureQualityOverrideEnabled", "True"),
        ("DFIntTextureQualityOverride", "0"),
        ("FIntDebugForceMSAASamples", "0"),
        ("DFFlagDebugPauseVoxelizer", "True"),
    ]);
    for n in LOD_NAMES {
        v.insert(n.into(), "0".into());
    }
    v
}

/// Presets nach Plan §13.6, nur der FastFlag-Teil (FPS-Limit gehört zur XML).
/// Reihenfolge in der Oberfläche: Max FPS, Ausgewogen, Hohe Qualität, dann Aus und Eigene.
#[tauri::command]
pub fn flags_presets() -> Vec<Preset> {
    let mut max_fps = max_fps_values();
    // Experimentell: Wirkung nicht bestätigt (Forum: Wert ist die gewünschte Auflösung).
    max_fps.insert("DFIntDebugDynamicRenderKiloPixels".into(), "720".into());
    vec![
        Preset {
            id: "max-fps".into(),
            name: "Max FPS".into(),
            description: "Niedrigste Stufen, kein Gras, statisches Licht, kleinere Renderauflösung. Für schwache PCs und Laptops.".into(),
            values: max_fps,
        },
        Preset {
            id: "ausgewogen".into(),
            name: "Ausgewogen".into(),
            description: "Hohe Texturen und 2× Kantenglättung. Ein guter Start für die meisten PCs.".into(),
            values: map(&[
                ("DFFlagTextureQualityOverrideEnabled", "True"),
                ("DFIntTextureQualityOverride", "2"),
                ("FIntDebugForceMSAASamples", "2"),
            ]),
        },
        Preset {
            id: "qualitaet".into(),
            name: "Hohe Qualität".into(),
            description: "Beste Texturen, 4× Kantenglättung, höchste Grafikstufe und weites Gras. Kostet viel Leistung.".into(),
            values: map(&[
                ("DFFlagTextureQualityOverrideEnabled", "True"),
                ("DFIntTextureQualityOverride", "3"),
                ("FIntDebugForceMSAASamples", "4"),
                ("DFIntDebugFRMQualityLevelOverride", "21"),
                ("FIntFRMMaxGrassDistance", "400"),
            ]),
        },
    ]
}

// ---------- Vor dem Start schreiben ----------

/// Inhalt der `ClientAppSettings.json`: nur Allowlist-Flags, Werte als Strings.
fn build_content(allow: &AllowFile, saved: &FlagsFile) -> String {
    let mut out: BTreeMap<&str, &str> = BTreeMap::new();
    if saved.preset != "aus" {
        for f in &allow.flags {
            if let Some(v) = saved.values.get(&f.name) {
                out.insert(&f.name, v);
            }
        }
    }
    serde_json::to_string_pretty(&out).unwrap_or_else(|_| "{}".into())
}

fn write_file_if_changed(file: &Path, content: &str) -> Result<bool, String> {
    if std::fs::read_to_string(file).map(|c| c == content).unwrap_or(false) {
        return Ok(false);
    }
    if let Some(dir) = file.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("Ordner anlegen fehlgeschlagen: {e}"))?;
    }
    std::fs::write(file, content).map_err(|e| format!("Schreiben fehlgeschlagen: {e}"))?;
    Ok(true)
}

/// Schreibt die Flags in jeden Versionsordner. Rückgabe: Zahl der geschriebenen Dateien.
pub fn apply_before_launch(state: &AppState) -> Result<usize, String> {
    let allow = load_allowlist(state);
    let path = state.file(STATE_FILE);
    let mut saved: FlagsFile = read_json(&path).unwrap_or_default();
    let content = build_content(&allow, &saved);
    let empty = content.trim() == "{}";

    let dirs = player_version_dirs();
    let mut written = 0usize;
    let mut applied: Vec<String> = Vec::new();
    let mut first_error: Option<String> = None;

    for dir in &dirs {
        let file = dir.join("ClientSettings").join("ClientAppSettings.json");
        if empty {
            // Nur aufräumen, wenn die Datei von Brickstrap stammt. Fremde Dateien bleiben.
            let ours = match (&saved.last_content, std::fs::read_to_string(&file)) {
                (Some(prev), Ok(now_content)) => *prev == now_content && !prev.trim().eq("{}"),
                _ => false,
            };
            if !ours {
                continue;
            }
        }
        match write_file_if_changed(&file, &content) {
            Ok(changed) => {
                if changed {
                    written += 1;
                }
                if let Some(n) = dir.file_name() {
                    applied.push(n.to_string_lossy().into_owned());
                }
            }
            Err(e) => {
                first_error.get_or_insert(format!("{}: {e}", dir.display()));
            }
        }
    }

    if !dirs.is_empty() && first_error.is_none() {
        saved.applied_to = applied;
        saved.last_applied = Some(now());
        saved.last_content = Some(content);
        write_json(&path, &saved)?;
    }
    match first_error {
        Some(e) => Err(e),
        None => Ok(written),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allowlist_ist_gueltig() {
        let a: AllowFile = serde_json::from_str(EMBEDDED_ALLOWLIST).expect("allowlist.json");
        assert_eq!(a.flags.len(), 19);
        assert!(a.flags.iter().any(|f| f.experimental));
    }

    #[test]
    fn normalisieren() {
        let b = AllowFlag { name: "X".into(), kind: "bool".into(), ..Default::default() };
        assert_eq!(normalize(&b, "true").unwrap(), "True");
        assert!(normalize(&b, "vielleicht").is_err());
        let i = AllowFlag { name: "Y".into(), kind: "int".into(), min: Some(0), max: Some(3), ..Default::default() };
        assert_eq!(normalize(&i, " 7 ").unwrap(), "3");
        assert!(normalize(&i, "1.5").is_err());
    }
}
