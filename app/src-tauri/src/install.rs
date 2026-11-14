//! Fester Platz für Brickstrap, damit Verknüpfungen und Updates immer dieselbe Datei treffen.
//!
//! Beim Start aus einem anderen Ordner (z. B. „Downloads“) kopiert sich Brickstrap nach
//! `%LOCALAPPDATA%\Brickstrap\Brickstrap.exe`, legt Verknüpfungen auf dem Desktop und im
//! Startmenü an und startet von dort neu. Eine ältere Download-exe überschreibt eine neuere
//! installierte nie (`version.txt` neben der exe).

use std::path::{Path, PathBuf};

const ORDNER: &str = "Brickstrap";
const DATEI: &str = "Brickstrap.exe";

/// Ziel der Installation.
pub fn ziel() -> Option<PathBuf> {
    std::env::var_os("LOCALAPPDATA").map(|p| PathBuf::from(p).join(ORDNER).join(DATEI))
}

fn versionsdatei(exe: &Path) -> PathBuf {
    exe.with_file_name("version.txt")
}

/// Merkt die Version der installierten exe (nach Installation und Update).
pub fn version_merken(exe: &Path, version: &str) {
    let _ = std::fs::write(versionsdatei(exe), version);
}

fn teile(v: &str) -> Option<(u64, u64, u64)> {
    let mut it = v.trim().trim_start_matches('v').split(['.', '-']).map(|p| p.parse::<u64>());
    Some((it.next()?.ok()?, it.next()?.ok()?, it.next()?.ok()?))
}

fn gleich(a: &Path, b: &Path) -> bool {
    match (a.canonicalize(), b.canonicalize()) {
        (Ok(x), Ok(y)) => x.to_string_lossy().eq_ignore_ascii_case(&y.to_string_lossy()),
        _ => a.to_string_lossy().eq_ignore_ascii_case(&b.to_string_lossy()),
    }
}

/// Verknüpfungen auf dem Desktop und im Startmenü (nur anlegen, wenn sie fehlen).
#[cfg(windows)]
fn verknuepfungen(exe: &Path) {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let pfad = exe.display().to_string().replace('\'', "''");
    let ordner = exe.parent().map(|p| p.display().to_string()).unwrap_or_default().replace('\'', "''");
    let skript = format!(
        "$w=New-Object -ComObject WScript.Shell;\
         foreach($o in @([Environment]::GetFolderPath('Desktop'),[Environment]::GetFolderPath('Programs'))){{\
         $l=Join-Path $o 'Brickstrap.lnk';\
         if(-not (Test-Path $l)){{$s=$w.CreateShortcut($l);$s.TargetPath='{pfad}';$s.WorkingDirectory='{ordner}';\
         $s.IconLocation='{pfad},0';$s.Description='Brickstrap';$s.Save()}}}}"
    );
    let _ = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", &skript])
        .creation_flags(CREATE_NO_WINDOW)
        .status();
}

#[cfg(not(windows))]
fn verknuepfungen(_exe: &Path) {}

/// Läuft Brickstrap nicht vom festen Platz, dorthin kopieren und von dort starten.
/// `true` heißt: Diese Instanz soll sich jetzt beenden.
pub fn sicherstellen() -> bool {
    if cfg!(debug_assertions) {
        return false; // `tauri dev` nicht umleiten
    }
    let (Ok(exe), Some(ziel)) = (std::env::current_exe(), ziel()) else { return false };
    let jetzt = env!("CARGO_PKG_VERSION");
    if gleich(&exe, &ziel) {
        // Schon am festen Platz: Version merken, fehlende Verknüpfungen nachholen.
        version_merken(&ziel, jetzt);
        // Im Hintergrund, damit der Start nicht auf PowerShell wartet.
        std::thread::spawn(move || verknuepfungen(&ziel));
        return false;
    }
    let installiert = std::fs::read_to_string(versionsdatei(&ziel)).ok();
    let aelter_installiert = match installiert.as_deref().and_then(teile) {
        Some(v) => teile(jetzt).is_some_and(|j| j > v),
        None => true,
    };
    if !ziel.exists() || aelter_installiert {
        let Some(dir) = ziel.parent() else { return false };
        if std::fs::create_dir_all(dir).is_err() || std::fs::copy(&exe, &ziel).is_err() {
            return false; // dann eben von hier laufen
        }
        version_merken(&ziel, jetzt);
    }
    verknuepfungen(&ziel);
    crate::update::neu_starten(&ziel).is_ok()
}
