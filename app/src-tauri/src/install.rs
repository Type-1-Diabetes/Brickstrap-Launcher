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

/// Verknüpfungen auf dem Desktop und im Startmenü anlegen oder auf den festen Platz umbiegen,
/// falls sie noch auf eine alte exe (z. B. in „Downloads“) zeigen.
#[cfg(windows)]
fn verknuepfungen(exe: &Path) {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let pfad = exe.display().to_string().replace('\'', "''");
    let ordner = exe.parent().map(|p| p.display().to_string()).unwrap_or_default().replace('\'', "''");
    let skript = format!(
        "$w=New-Object -ComObject WScript.Shell;\
         foreach($o in @([Environment]::GetFolderPath('Desktop'),[Environment]::GetFolderPath('Programs'))){{\
         $l=Join-Path $o 'Brickstrap.lnk';$s=$w.CreateShortcut($l);\
         if($s.TargetPath -ne '{pfad}'){{$s.TargetPath='{pfad}';$s.WorkingDirectory='{ordner}';\
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

/// Andere laufende Brickstrap-Instanzen aus einem anderen Ordner beenden (z. B. eine alte
/// Version im Infobereich). Sonst holt die Einzelinstanz beim Start einer neuen exe nur die
/// alte nach vorn. Muss vor dem Plugin `single-instance` laufen. Gleicher Pfad = normale
/// Einzelinstanz (Fenster nach vorn), darum bleibt die unangetastet.
#[cfg(windows)]
pub fn andere_beenden() {
    use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE, WAIT_TIMEOUT};
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
    };
    use windows_sys::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, TerminateProcess, WaitForSingleObject, PROCESS_QUERY_LIMITED_INFORMATION,
        PROCESS_SYNCHRONIZE, PROCESS_TERMINATE,
    };
    if cfg!(debug_assertions) {
        return;
    }
    let Ok(ich) = std::env::current_exe() else { return };
    let eigene = std::process::id();
    // SAFETY: Standard-Aufrufe der Toolhelp- und Prozess-API; jeder Handle wird geschlossen.
    unsafe {
        let snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snap == INVALID_HANDLE_VALUE {
            return;
        }
        let mut entry: PROCESSENTRY32W = std::mem::zeroed();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
        let mut ok = Process32FirstW(snap, &mut entry) != 0;
        while ok {
            let len = entry.szExeFile.iter().position(|&c| c == 0).unwrap_or(entry.szExeFile.len());
            let name = String::from_utf16_lossy(&entry.szExeFile[..len]).to_ascii_lowercase();
            if entry.th32ProcessID != eigene && name.starts_with("brickstrap") && name.ends_with(".exe") {
                let h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_TERMINATE | PROCESS_SYNCHRONIZE, 0, entry.th32ProcessID);
                if !h.is_null() {
                    let mut buf = [0u16; 1024];
                    let mut n = buf.len() as u32;
                    let pfad = (QueryFullProcessImageNameW(h, 0, buf.as_mut_ptr(), &mut n) != 0)
                        .then(|| std::path::PathBuf::from(String::from_utf16_lossy(&buf[..n as usize])));
                    if pfad.is_some_and(|p| !gleich(&p, &ich)) && TerminateProcess(h, 0) != 0 {
                        // Kurz warten, bis die Sperre der Einzelinstanz frei ist.
                        if WaitForSingleObject(h, 3000) == WAIT_TIMEOUT {
                            eprintln!("Alte Instanz beendet sich nicht rechtzeitig");
                        }
                    }
                    CloseHandle(h);
                }
            }
            ok = Process32NextW(snap, &mut entry) != 0;
        }
        CloseHandle(snap);
    }
}

#[cfg(not(windows))]
pub fn andere_beenden() {}
