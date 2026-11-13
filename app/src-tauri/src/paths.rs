//! Pfade von Roblox unter Windows und Prozess-Prüfung.
//! Unter anderen Systemen liefern die Funktionen leere Ergebnisse, damit der Code kompiliert.

use std::path::PathBuf;

/// `%localappdata%\Roblox`
pub fn roblox_dir() -> Option<PathBuf> {
    std::env::var_os("LOCALAPPDATA").map(|p| PathBuf::from(p).join("Roblox"))
}

/// Datenordner der Microsoft-Store-Version:
/// `%localappdata%\Packages\ROBLOXCORPORATION.ROBLOX_55nm5eh3cm0pr\LocalState`
fn store_dir() -> Option<PathBuf> {
    std::env::var_os("LOCALAPPDATA").map(|p| {
        PathBuf::from(p)
            .join("Packages")
            .join("ROBLOXCORPORATION.ROBLOX_55nm5eh3cm0pr")
            .join("LocalState")
    })
}

/// Erster vorhandener Pfad: normale Installation vor Store-Version.
fn first_existing(rel: &str) -> Option<PathBuf> {
    let normal = roblox_dir().map(|d| d.join(rel));
    let store = store_dir().map(|d| d.join(rel));
    [normal.clone(), store]
        .into_iter()
        .flatten()
        .find(|p| p.exists())
        .or(normal)
}

/// `%localappdata%\Roblox\logs` (oder der Ordner der Store-Version)
pub fn logs_dir() -> Option<PathBuf> {
    first_existing("logs")
}

/// Alle vorhandenen Log-Ordner: normale Installation und Store-Version.
/// Sind beide da, kann jede die aktuelle sein, darum beide durchsuchen.
pub fn all_logs_dirs() -> Vec<PathBuf> {
    [roblox_dir(), store_dir()]
        .into_iter()
        .flatten()
        .map(|d| d.join("logs"))
        .filter(|d| d.is_dir())
        .collect()
}

/// `%localappdata%\Roblox\GlobalBasicSettings_13.xml` (oder die Datei der Store-Version)
pub fn basic_settings_file() -> Option<PathBuf> {
    first_existing("GlobalBasicSettings_13.xml")
}

/// Ist das Protokoll `roblox://` registriert? Dann öffnet Windows Roblox per Link,
/// auch ohne Versionsordner (z. B. Microsoft-Store-Version).
#[cfg(windows)]
pub fn protocol_registered() -> bool {
    use windows_sys::Win32::System::Registry::{RegCloseKey, RegOpenKeyExW, HKEY, HKEY_CLASSES_ROOT, KEY_READ};
    let name: Vec<u16> = "roblox\0".encode_utf16().collect();
    let mut key: HKEY = std::ptr::null_mut();
    // SAFETY: gültiger nullterminierter Name, Schlüssel wird bei Erfolg geschlossen.
    unsafe {
        if RegOpenKeyExW(HKEY_CLASSES_ROOT, name.as_ptr(), 0, KEY_READ, &mut key) == 0 {
            RegCloseKey(key);
            true
        } else {
            false
        }
    }
}

#[cfg(not(windows))]
pub fn protocol_registered() -> bool {
    false
}

/// Alle Versionsordner, die einen Player enthalten (neuester zuerst).
/// Sucht in `%localappdata%\Roblox\Versions` und im alten Ort unter `Program Files (x86)`.
pub fn player_version_dirs() -> Vec<PathBuf> {
    let mut roots: Vec<PathBuf> = Vec::new();
    if let Some(d) = roblox_dir() {
        roots.push(d.join("Versions"));
    }
    if let Some(p) = std::env::var_os("ProgramFiles(x86)") {
        roots.push(PathBuf::from(p).join("Roblox").join("Versions"));
    }
    let mut found: Vec<(std::time::SystemTime, PathBuf)> = Vec::new();
    for root in roots {
        let Ok(entries) = std::fs::read_dir(&root) else { continue };
        for e in entries.flatten() {
            let dir = e.path();
            let exe = dir.join("RobloxPlayerBeta.exe");
            if exe.is_file() {
                let t = std::fs::metadata(&exe)
                    .and_then(|m| m.modified())
                    .unwrap_or(std::time::UNIX_EPOCH);
                found.push((t, dir));
            }
        }
    }
    found.sort_by_key(|f| std::cmp::Reverse(f.0));
    found.into_iter().map(|(_, d)| d).collect()
}

/// Neueste `RobloxPlayerBeta.exe`, falls installiert.
pub fn player_exe() -> Option<PathBuf> {
    player_version_dirs()
        .into_iter()
        .next()
        .map(|d| d.join("RobloxPlayerBeta.exe"))
}

/// Läuft gerade ein Roblox-Player?
#[cfg(windows)]
pub fn roblox_running() -> bool {
    use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
    };
    // SAFETY: Standard-Aufrufe der Toolhelp-API; der Handle wird immer geschlossen.
    unsafe {
        let snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snap == INVALID_HANDLE_VALUE {
            return false;
        }
        let mut entry: PROCESSENTRY32W = std::mem::zeroed();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
        let mut found = false;
        if Process32FirstW(snap, &mut entry) != 0 {
            loop {
                let len = entry.szExeFile.iter().position(|&c| c == 0).unwrap_or(entry.szExeFile.len());
                let name = String::from_utf16_lossy(&entry.szExeFile[..len]);
                if name.eq_ignore_ascii_case("RobloxPlayerBeta.exe") {
                    found = true;
                    break;
                }
                if Process32NextW(snap, &mut entry) == 0 {
                    break;
                }
            }
        }
        CloseHandle(snap);
        found
    }
}

#[cfg(not(windows))]
pub fn roblox_running() -> bool {
    false
}
