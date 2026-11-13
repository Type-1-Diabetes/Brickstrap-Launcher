//! `GlobalBasicSettings_13.xml`: Einstellungen aus dem Roblox-Menü lesen und ändern.
//!
//! Geschrieben wird nur, wenn Roblox komplett geschlossen ist. Beim Schreiben bleibt alles
//! außer den Textinhalten der geänderten Elemente Byte für Byte gleich (Deklaration,
//! Namensräume, andere Items, Reihenfolge, Leerraum).

use std::path::Path;

use quick_xml::events::Event;
use quick_xml::Reader;
use serde::{Deserialize, Serialize};

use crate::paths::{basic_settings_file, roblox_running};

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BasicEntry {
    /// Elementname: int, float, bool, token, string, …
    pub kind: String,
    pub name: String,
    pub value: String,
    pub editable: bool,
    pub label: Option<String>,
    pub hint: Option<String>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub step: Option<f64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BasicView {
    pub path: String,
    pub exists: bool,
    pub running: bool,
    pub entries: Vec<BasicEntry>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BasicChange {
    pub name: String,
    pub kind: String,
    pub value: String,
}

// ---------- Bekannte Schlüssel ----------

struct Known {
    name: &'static str,
    label: &'static str,
    hint: &'static str,
    min: Option<f64>,
    max: Option<f64>,
    step: Option<f64>,
}

const KNOWN: &[Known] = &[
    Known { name: "FramerateCap", label: "FPS-Limit", hint: "0 = ohne Limit. Roblox bietet im Menü bis 240 an. Höhere Werte wirken eventuell nicht.", min: Some(0.0), max: Some(1000.0), step: Some(1.0) },
    Known { name: "SavedQualityLevel", label: "Grafikqualität", hint: "0 = automatisch, 1 bis 10 = feste Stufe. Niedrigere Stufen bringen mehr FPS.", min: Some(0.0), max: Some(10.0), step: Some(1.0) },
    Known { name: "GraphicsQualityLevel", label: "Grafikqualität (fein)", hint: "Feine Stufe der Grafikqualität von 1 bis 21. Nicht in jeder Datei vorhanden.", min: Some(1.0), max: Some(21.0), step: Some(1.0) },
    Known { name: "Fullscreen", label: "Vollbild", hint: "Roblox im Vollbild starten.", min: None, max: None, step: None },
    Known { name: "MouseSensitivity", label: "Mausempfindlichkeit", hint: "Im Menü nur eingeschränkt einstellbar. Hier sind auch Zwischenwerte möglich.", min: Some(0.01), max: Some(10.0), step: Some(0.01) },
    Known { name: "MasterVolume", label: "Lautstärke", hint: "Gesamtlautstärke von 0 (stumm) bis 1 (volle Lautstärke).", min: Some(0.0), max: Some(1.0), step: Some(0.05) },
    Known { name: "ReducedMotion", label: "Weniger Bewegung", hint: "Reduziert Wackeln und Bewegungseffekte.", min: None, max: None, step: None },
    Known { name: "PreferredTransparency", label: "Transparenz der Oberfläche", hint: "1 = normal, 0 = nicht durchsichtig.", min: Some(0.0), max: Some(1.0), step: Some(0.05) },
    Known { name: "PreferredTextSize", label: "Textgröße", hint: "Stufe der Textgröße in der Oberfläche.", min: None, max: None, step: Some(1.0) },
    Known { name: "CameraMode", label: "Kameramodus", hint: "Kameramodus als Zahl. Die Bedeutung legt Roblox fest, 0 ist der Standard.", min: None, max: None, step: Some(1.0) },
    Known { name: "VREnabled", label: "VR", hint: "VR-Modus von Roblox. Nur sinnvoll mit VR-Brille.", min: None, max: None, step: None },
];

fn known_index(name: &str) -> Option<usize> {
    KNOWN.iter().position(|k| k.name == name)
}

fn is_editable(kind: &str) -> bool {
    matches!(kind, "int" | "float" | "bool" | "token")
}

// ---------- Lesen (reine Funktionen) ----------

fn attr(e: &quick_xml::events::BytesStart<'_>, key: &str) -> Option<String> {
    e.attributes()
        .flatten()
        .find(|a| a.key.as_ref() == key.as_bytes())
        .and_then(|a| a.unescape_value().ok().map(|v| v.into_owned()))
}

fn utf8(b: &[u8]) -> String {
    String::from_utf8_lossy(b).into_owned()
}

/// Zeigt der Stapel auf `Item[class=UserGameSettings]/Properties`?
fn in_properties(stack: &[(String, Option<String>)]) -> bool {
    let n = stack.len();
    n >= 2 && stack[n - 1].0 == "Properties" && stack[n - 2].0 == "Item" && stack[n - 2].1.as_deref() == Some("UserGameSettings")
}

/// Alle Einträge unter `Item[@class='UserGameSettings']/Properties`, bekannte zuerst.
pub fn parse_entries(xml: &str) -> Vec<BasicEntry> {
    let mut reader = Reader::from_str(xml);
    let mut stack: Vec<(String, Option<String>)> = Vec::new();
    // Aktueller Eintrag: Tiefe des Eintrags im Stapel, Daten, gesammelter Text
    let mut cur: Option<(usize, String, String, Vec<String>)> = None;
    let mut found: Vec<BasicEntry> = Vec::new();

    let finish = |c: (usize, String, String, Vec<String>), found: &mut Vec<BasicEntry>| {
        let (_, kind, name, parts) = c;
        let k = known_index(&name).map(|i| &KNOWN[i]);
        found.push(BasicEntry {
            editable: is_editable(&kind),
            kind,
            name,
            value: parts.join(" "),
            label: k.map(|k| k.label.to_string()),
            hint: k.map(|k| k.hint.to_string()),
            min: k.and_then(|k| k.min),
            max: k.and_then(|k| k.max),
            step: k.and_then(|k| k.step),
        });
    };

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let tag = utf8(e.name().as_ref());
                if cur.is_none() && in_properties(&stack) {
                    if let Some(name) = attr(&e, "name") {
                        cur = Some((stack.len(), tag.clone(), name, Vec::new()));
                    }
                }
                let class = attr(&e, "class");
                stack.push((tag, class));
            }
            Ok(Event::Empty(e)) => {
                if cur.is_none() && in_properties(&stack) {
                    if let Some(name) = attr(&e, "name") {
                        finish((stack.len(), utf8(e.name().as_ref()), name, Vec::new()), &mut found);
                    }
                }
            }
            Ok(Event::Text(t)) => {
                if let Some(c) = cur.as_mut() {
                    if let Ok(s) = t.unescape() {
                        let s = s.trim();
                        if !s.is_empty() {
                            c.3.push(s.to_string());
                        }
                    }
                }
            }
            Ok(Event::CData(t)) => {
                if let Some(c) = cur.as_mut() {
                    let s = utf8(&t);
                    let s = s.trim();
                    if !s.is_empty() {
                        c.3.push(s.to_string());
                    }
                }
            }
            Ok(Event::End(_)) => {
                stack.pop();
                if cur.as_ref().map(|c| c.0 == stack.len()).unwrap_or(false) {
                    if let Some(c) = cur.take() {
                        finish(c, &mut found);
                    }
                }
            }
            Ok(Event::Eof) | Err(_) => break,
            Ok(_) => {}
        }
    }

    // Bekannte zuerst (in der Reihenfolge von KNOWN), Rest in Dateireihenfolge.
    let mut keyed: Vec<(usize, usize, BasicEntry)> = found
        .into_iter()
        .enumerate()
        .map(|(i, e)| (known_index(&e.name).unwrap_or(usize::MAX), i, e))
        .collect();
    keyed.sort_by_key(|(k, i, _)| (*k, *i));
    keyed.into_iter().map(|(_, _, e)| e).collect()
}

// ---------- Schreiben (reine Funktionen) ----------

/// Wert prüfen und in die Schreibweise der Datei bringen.
fn check_value(c: &BasicChange) -> Result<String, String> {
    let v = c.value.trim();
    match c.kind.as_str() {
        "int" | "token" => v
            .parse::<i64>()
            .map(|n| n.to_string())
            .map_err(|_| format!("{}: Erwartet eine ganze Zahl.", c.name)),
        "float" => match v.parse::<f64>() {
            Ok(f) if f.is_finite() => Ok(f.to_string()),
            _ => Err(format!("{}: Erwartet eine Zahl.", c.name)),
        },
        "bool" => match v.to_ascii_lowercase().as_str() {
            "true" => Ok("true".into()),
            "false" => Ok("false".into()),
            _ => Err(format!("{}: Erwartet true oder false.", c.name)),
        },
        other => Err(format!("{}: Typ „{other}“ lässt sich nicht ändern.", c.name)),
    }
}

/// Ersetzt die Textinhalte der geänderten Einträge. Alles andere bleibt unverändert.
pub fn apply_changes(xml: &str, changes: &[BasicChange]) -> Result<String, String> {
    let mut wanted: Vec<(&BasicChange, String)> = Vec::new();
    for c in changes {
        wanted.push((c, check_value(c)?));
    }
    let mut done = vec![false; wanted.len()];

    let mut reader = Reader::from_str(xml);
    let mut stack: Vec<(String, Option<String>)> = Vec::new();
    // Ersetzungen als (von, bis, neuer Text) in Byte-Positionen von `xml`
    let mut edits: Vec<(usize, usize, String)> = Vec::new();
    // Offener Eintrag, der geändert wird: (Tiefe, Start des Inhalts, Index, Tag, Wert)
    let mut open: Option<(usize, usize, usize)> = None;

    loop {
        let before = reader.buffer_position() as usize;
        let ev = reader.read_event().map_err(|e| format!("XML nicht lesbar: {e}"))?;
        let after = reader.buffer_position() as usize;
        match ev {
            Event::Start(e) => {
                if open.is_none() && in_properties(&stack) {
                    let tag = utf8(e.name().as_ref());
                    if let Some(name) = attr(&e, "name") {
                        if let Some(i) = wanted.iter().position(|(c, _)| c.name == name && c.kind == tag) {
                            open = Some((stack.len(), after, i));
                        }
                    }
                }
                let class = attr(&e, "class");
                stack.push((utf8(e.name().as_ref()), class));
            }
            Event::Empty(e) => {
                if open.is_none() && in_properties(&stack) {
                    let tag = utf8(e.name().as_ref());
                    if let Some(name) = attr(&e, "name") {
                        if let Some(i) = wanted.iter().position(|(c, _)| c.name == name && c.kind == tag) {
                            // `<int name="x"/>` wird zu `<int name="x">wert</int>`
                            let raw = xml.get(before..after).ok_or("XML-Position ungültig")?;
                            let head = raw.strip_suffix("/>").ok_or("XML nicht lesbar")?.trim_end();
                            edits.push((before, after, format!("{head}>{}</{tag}>", wanted[i].1)));
                            done[i] = true;
                        }
                    }
                }
            }
            Event::End(_) => {
                stack.pop();
                if let Some((depth, start, i)) = open {
                    if depth == stack.len() {
                        edits.push((start, before, wanted[i].1.clone()));
                        done[i] = true;
                        open = None;
                    }
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }

    if let Some(i) = done.iter().position(|d| !d) {
        return Err(format!("UNBEKANNT|Eintrag „{}“ ({}) steht nicht in der Datei.", wanted[i].0.name, wanted[i].0.kind));
    }

    edits.sort_by_key(|e| e.0);
    let mut out = String::with_capacity(xml.len() + 32);
    let mut pos = 0usize;
    for (from, to, text) in edits {
        if from < pos || to < from {
            return Err("Überlappende Änderungen.".into());
        }
        out.push_str(xml.get(pos..from).ok_or("XML-Position ungültig")?);
        out.push_str(&text);
        pos = to;
    }
    out.push_str(xml.get(pos..).ok_or("XML-Position ungültig")?);

    // Gegenprobe: Das Ergebnis muss wieder lesbar sein und die neuen Werte enthalten.
    let mut check = Reader::from_str(&out);
    loop {
        match check.read_event() {
            Ok(Event::Eof) => break,
            Err(e) => return Err(format!("Ergebnis ist kein gültiges XML: {e}")),
            Ok(_) => {}
        }
    }
    Ok(out)
}

// ---------- Befehle ----------

fn read_text(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| format!("Datei nicht lesbar: {e}"))
}

#[tauri::command]
pub fn basic_get() -> Result<BasicView, String> {
    let path = basic_settings_file().ok_or_else(|| "Roblox-Ordner nicht gefunden.".to_string())?;
    let exists = path.is_file();
    let entries = if exists { parse_entries(&read_text(&path)?) } else { Vec::new() };
    Ok(BasicView {
        path: path.to_string_lossy().into_owned(),
        exists,
        running: roblox_running(),
        entries,
    })
}

#[tauri::command]
pub fn basic_set(changes: Vec<BasicChange>) -> Result<(), String> {
    if roblox_running() {
        return Err("LAEUFT|Roblox läuft. Schließ Roblox zuerst, sonst überschreibt es deine Änderungen.".into());
    }
    let path = basic_settings_file().ok_or_else(|| "Roblox-Ordner nicht gefunden.".to_string())?;
    if !path.is_file() {
        return Err("DATEI|Die Einstellungsdatei von Roblox fehlt. Starte Roblox einmal und schließ es wieder.".into());
    }
    if changes.is_empty() {
        return Ok(());
    }
    let xml = read_text(&path)?;
    let next = apply_changes(&xml, &changes)?;
    if next == xml {
        return Ok(());
    }
    // Eine ältere Schreibsperre (z. B. von anderen Launchern) lassen wir unangetastet.
    if std::fs::metadata(&path).map(|m| m.permissions().readonly()).unwrap_or(false) {
        return Err("SCHREIBGESCHUETZT|Die Datei ist schreibgeschützt. Hebe den Schutz in den Dateieigenschaften auf.".into());
    }
    // Einmalige Sicherung, nie überschreiben.
    let backup = path.with_file_name("GlobalBasicSettings_13.xml.brickstrap-sicherung");
    if !backup.exists() {
        std::fs::copy(&path, &backup).map_err(|e| format!("Sicherung fehlgeschlagen: {e}"))?;
    }
    let tmp = path.with_file_name("GlobalBasicSettings_13.xml.brickstrap-tmp");
    std::fs::write(&tmp, next).map_err(|e| format!("Schreiben fehlgeschlagen: {e}"))?;
    std::fs::rename(&tmp, &path).map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        format!("Schreiben fehlgeschlagen: {e}")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "<?xml version=\"1.0\" encoding=\"utf-8\"?>\r\n<roblox xmlns:xmime=\"http://www.w3.org/2005/05/xmlmime\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" xsi:noNamespaceSchemaLocation=\"http://www.roblox.com/roblox.xsd\" version=\"4\">\r\n  <External>null</External>\r\n  <External>nil</External>\r\n  <Item class=\"UserGameSettings\" referent=\"RBX1\">\r\n    <Properties>\r\n      <int name=\"FramerateCap\">60</int>\r\n      <token name=\"SavedQualityLevel\">2</token>\r\n      <float name=\"MouseSensitivity\">1</float>\r\n      <bool name=\"VREnabled\">false</bool>\r\n      <float name=\"PreferredTransparency\">1</float>\r\n      <bool name=\"ReducedMotion\">false</bool>\r\n      <token name=\"PreferredTextSize\">1</token>\r\n      <string name=\"Eigen\">a &amp; b</string>\r\n      <Vector2 name=\"V\"><X>1</X><Y>2</Y></Vector2>\r\n      <int name=\"Leer\"/>\r\n    </Properties>\r\n  </Item>\r\n  <Item class=\"Anderes\" referent=\"RBX2\">\r\n    <Properties>\r\n      <int name=\"FramerateCap\">1</int>\r\n    </Properties>\r\n  </Item>\r\n</roblox>\r\n";

    fn ch(name: &str, kind: &str, value: &str) -> BasicChange {
        BasicChange { name: name.into(), kind: kind.into(), value: value.into() }
    }

    #[test]
    fn lesen() {
        let e = parse_entries(SAMPLE);
        assert_eq!(e.len(), 10);
        assert_eq!(e[0].name, "FramerateCap");
        assert_eq!(e[0].value, "60");
        assert_eq!(e[0].label.as_deref(), Some("FPS-Limit"));
        assert!(e[0].editable);
        let s = e.iter().find(|x| x.name == "Eigen").map(|x| x.value.clone());
        assert_eq!(s.as_deref(), Some("a & b"));
        let v = e.iter().find(|x| x.name == "V");
        assert_eq!(v.map(|x| (x.value.as_str(), x.editable)), Some(("1 2", false)));
        assert!(e.iter().any(|x| x.name == "Leer" && x.value.is_empty()));
    }

    #[test]
    fn schreiben_lässt_rest_unveraendert() {
        let out = apply_changes(SAMPLE, &[ch("FramerateCap", "int", "0"), ch("ReducedMotion", "bool", "TRUE"), ch("Leer", "int", "5")]).unwrap();
        let expect = SAMPLE
            .replace("<int name=\"FramerateCap\">60</int>", "<int name=\"FramerateCap\">0</int>")
            .replace("<bool name=\"ReducedMotion\">false</bool>", "<bool name=\"ReducedMotion\">true</bool>")
            .replace("<int name=\"Leer\"/>", "<int name=\"Leer\">5</int>");
        assert_eq!(out, expect);
        // Das Item mit anderer Klasse bleibt unberührt.
        assert!(out.contains("<int name=\"FramerateCap\">1</int>"));
    }

    #[test]
    fn fehler() {
        assert!(apply_changes(SAMPLE, &[ch("FramerateCap", "int", "abc")]).is_err());
        assert!(apply_changes(SAMPLE, &[ch("Eigen", "string", "x")]).is_err());
        assert!(apply_changes(SAMPLE, &[ch("Gibtsnicht", "int", "1")]).unwrap_err().starts_with("UNBEKANNT|"));
        assert!(apply_changes(SAMPLE, &[ch("FramerateCap", "float", "1")]).is_err());
        assert!(apply_changes(SAMPLE, &[ch("MouseSensitivity", "float", "nan")]).is_err());
    }
}
