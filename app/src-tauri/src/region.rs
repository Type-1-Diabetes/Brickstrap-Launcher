//! Region eines Servers aus seiner IP-Adresse, ohne fremden Dienst.
//!
//! Die Tabelle liegt in `resources/regions.json` (eingebettet). Eine Datei
//! `<AppData>/regions.json` überschreibt sie, wie bei der Allowlist. Die Orte stammen aus
//! einer Community-Liste und sind Schätzungen; unbekannte Adressen ergeben `None`.

use std::path::Path;
use std::sync::OnceLock;

use serde::Deserialize;

use crate::state::read_json;

const EMBEDDED: &str = include_str!("../resources/regions.json");

#[derive(Deserialize)]
struct Entry {
    cidr: String,
    city: String,
    #[serde(default)]
    country: String,
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct RegionFile {
    ranges: Vec<Entry>,
}

struct Range {
    base: u32,
    mask: u32,
    label: String,
}

static TABLE: OnceLock<Vec<Range>> = OnceLock::new();

fn parse_ip(s: &str) -> Option<u32> {
    let mut parts = s.trim().split('.');
    let mut out = 0u32;
    for _ in 0..4 {
        out = (out << 8) | u32::from(parts.next()?.parse::<u8>().ok()?);
    }
    parts.next().is_none().then_some(out)
}

/// `a.b.c.d/n` oder eine einzelne Adresse.
fn parse_cidr(s: &str) -> Option<(u32, u32)> {
    let (ip, bits) = match s.split_once('/') {
        Some((ip, b)) => (ip, b.trim().parse::<u32>().ok()?),
        None => (s, 32),
    };
    if bits > 32 {
        return None;
    }
    let mask = if bits == 0 { 0 } else { u32::MAX << (32 - bits) };
    Some((parse_ip(ip)? & mask, mask))
}

fn build(file: RegionFile) -> Vec<Range> {
    let mut out: Vec<Range> = file
        .ranges
        .into_iter()
        .filter_map(|e| {
            let (base, mask) = parse_cidr(&e.cidr)?;
            let label = if e.country.is_empty() { e.city } else { format!("{}, {}", e.city, e.country) };
            Some(Range { base, mask, label })
        })
        .collect();
    // Der engste Bereich gewinnt.
    out.sort_by_key(|r| std::cmp::Reverse(r.mask));
    out
}

fn embedded() -> Vec<Range> {
    build(serde_json::from_str(EMBEDDED).unwrap_or_default())
}

/// Einmal beim Start: eigene Tabelle aus dem Datenordner, sonst die eingebaute.
pub fn init(data_dir: &Path) {
    let user: Option<RegionFile> = read_json(&data_dir.join("regions.json"));
    let table = match user.map(build).filter(|t| !t.is_empty()) {
        Some(t) => t,
        None => embedded(),
    };
    let _ = TABLE.set(table);
}

/// „Frankfurt, DE“ zu einer IPv4-Adresse, falls bekannt.
pub fn lookup(ip: &str) -> Option<String> {
    let n = parse_ip(ip)?;
    TABLE.get_or_init(embedded).iter().find(|r| n & r.mask == r.base).map(|r| r.label.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tabelle() {
        assert!(embedded().len() >= 30);
        assert_eq!(lookup("128.116.44.33").as_deref(), Some("Frankfurt, DE"));
        assert_eq!(lookup("209.206.43.7").as_deref(), Some("San Jose, US"));
        assert_eq!(lookup("10.0.0.1"), None);
        assert_eq!(lookup("kaputt"), None);
        assert_eq!(parse_cidr("1.2.3.4"), Some((0x01020304, u32::MAX)));
        assert_eq!(parse_cidr("1.2.3.4/40"), None);
    }
}
