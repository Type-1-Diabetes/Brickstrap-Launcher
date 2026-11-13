// Einziger Zugang zum Rust-Teil. Im Browser (npm run dev ohne Tauri) antwortet der Mock.

const inTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

let mock = null;
async function mockCall(cmd, args) {
  mock ??= await import("./mock.js");
  return mock.call(cmd, args ?? {});
}

/** Befehl im Rust-Teil aufrufen. Fehler kommen als { code, message } zurück. */
export async function call(cmd, args) {
  try {
    if (!inTauri) return await mockCall(cmd, args);
    const { invoke } = await import("@tauri-apps/api/core");
    return await invoke(cmd, args);
  } catch (e) {
    throw normalize(e);
  }
}

/** Ereignis aus dem Rust-Teil abonnieren. Gibt eine Abmelde-Funktion zurück. */
export async function on(event, handler) {
  if (!inTauri) {
    mock ??= await import("./mock.js");
    return mock.on(event, handler);
  }
  const { listen } = await import("@tauri-apps/api/event");
  return listen(event, (e) => handler(e.payload));
}

/** Fenster zeigen, sobald die Oberfläche gezeichnet ist (kein weißer Blitz). */
export async function showWindow() {
  if (!inTauri) return;
  const { getCurrentWindow } = await import("@tauri-apps/api/window");
  const w = getCurrentWindow();
  await w.show();
  await w.setFocus();
}

/** Link im Standardbrowser öffnen. */
export async function openExternal(url) {
  if (!inTauri) return window.open(url, "_blank", "noopener");
  const { openUrl } = await import("@tauri-apps/plugin-opener");
  return openUrl(url);
}

export const isDemo = !inTauri;

// Fehler vereinheitlichen: Rust liefert { code, message } (ApiError) oder "CODE|Text".
function normalize(e) {
  if (e && typeof e === "object" && "code" in e) return e;
  const s = String(e?.message ?? e ?? "");
  const i = s.indexOf("|");
  if (i > 0 && /^[A-Z_]+$/.test(s.slice(0, i))) return { code: s.slice(0, i), message: s.slice(i + 1) };
  return { code: "UNBEKANNT", message: s };
}
