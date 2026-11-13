// Gemeinsamer Zustand der Oberfläche (Svelte-5-Runes).

import { call, on } from "./bridge.js";
import { fehlerText } from "./errors.js";
import { suche } from "./suche.svelte.js";

export const app = $state({
  /** aktuelle Seite: { name, params } */
  route: { name: "start", params: {} },
  /** Verlauf für Zurück */
  history: [],
  config: null,
  pinned: [],
  activity: { state: "idle" },
  roblox: { installed: true, running: false, version: null },
  toasts: [],
  /** angemeldetes Roblox-Konto (aus Dateien von Roblox, ohne Login) */
  account: null,
  /** Bild-URLs je Universe: icons[id], banners[id] */
  icons: {},
  banners: {},
});

// ---------- Navigation ----------

export function go(name, params = {}) {
  if (app.route.name === name && JSON.stringify(app.route.params) === JSON.stringify(params)) return;
  // Suchfeld leeren, wenn die Suche verlassen wird (außer zum geöffneten Treffer und zurück).
  if (app.route.name === "suche" && name !== "spiel") suche.query = "";
  app.history.push($state.snapshot(app.route));
  if (app.history.length > 30) app.history.shift();
  app.route = { name, params };
}

export function back() {
  const prev = app.history.pop();
  if (prev) app.route = prev;
}

export function openGame(game) {
  go("spiel", { universeId: game.universeId, placeId: game.placeId, name: game.name });
}

// ---------- Hinweise unten rechts ----------

let toastId = 0;
/** Kurzer Hinweis. art: "info" | "fehler" */
export function toast(text, art = "info", hinweis = "") {
  const id = ++toastId;
  app.toasts.push({ id, text, art, hinweis });
  setTimeout(() => {
    const i = app.toasts.findIndex((t) => t.id === id);
    if (i >= 0) app.toasts.splice(i, 1);
  }, art === "fehler" ? 7000 : 3500);
}

/** Text kopieren und Bescheid geben. Die Zwischenablage kann gesperrt sein. */
export async function kopieren(text, meldung) {
  try {
    await navigator.clipboard.writeText(text);
    toast(meldung);
  } catch {
    toast("Kopieren hat nicht geklappt", "fehler", "Die Zwischenablage ist gerade belegt. Versuch es gleich noch einmal.");
  }
}

// ---------- Anpinnen ----------

export const isPinned = (universeId) => app.pinned.some((p) => p.universeId === universeId);

export async function togglePin(game) {
  const pinned = !isPinned(game.universeId);
  app.pinned = await call("pin_set", { universeId: game.universeId, placeId: game.placeId, name: game.name, pinned });
  toast(pinned ? `„${game.name}“ angepinnt` : `„${game.name}“ nicht mehr angepinnt`);
}

/** Eigenes Titelbild eines angepinnten Spiels setzen oder entfernen (null). */
export async function setCover(universeId, cover) {
  app.pinned = await call("pin_cover_set", { universeId, cover });
  toast(cover ? "Titelbild geändert" : "Standard-Titelbild wiederhergestellt");
}

export const coverOf = (universeId) => app.pinned.find((p) => p.universeId === universeId)?.cover ?? null;

/** Bilddatei auf 1280×720 zuschneiden und als WebP-data:-URL liefern (passt in die Einstellungen). */
export function bildVorbereiten(datei) {
  return new Promise((ok, fehler) => {
    const img = new Image();
    img.addEventListener("load", () => {
      const w = 1280, h = 720;
      const c = document.createElement("canvas");
      c.width = w;
      c.height = h;
      const s = Math.max(w / img.width, h / img.height);
      const dw = img.width * s, dh = img.height * s;
      c.getContext("2d").drawImage(img, (w - dw) / 2, (h - dh) / 2, dw, dh);
      URL.revokeObjectURL(img.src);
      ok(c.toDataURL("image/webp", 0.85));
    });
    img.addEventListener("error", () => fehler(new Error("Bild nicht lesbar")));
    img.src = URL.createObjectURL(datei);
  });
}

// ---------- Bilder (gebündelt laden) ----------

const queue = { icon: new Set(), banner: new Set() };
let timer = null;

/** Bild für ein Spiel anfordern; die URL landet in app.icons bzw. app.banners. */
export function wantImage(universeId, kind = "icon") {
  const store = kind === "banner" ? app.banners : app.icons;
  if (!universeId || universeId in store) return;
  store[universeId] = null; // markiert als angefragt
  queue[kind].add(universeId);
  timer ??= setTimeout(flush, 30);
}

async function flush() {
  timer = null;
  for (const kind of ["icon", "banner"]) {
    const ids = [...queue[kind]];
    queue[kind].clear();
    if (!ids.length) continue;
    try {
      const map = await call("thumbnails", { universeIds: ids, kind });
      const store = kind === "banner" ? app.banners : app.icons;
      for (const id of ids) store[id] = map[id] ?? "";
    } catch {
      // Bilder sind nicht kritisch; die Kachel zeigt dann den Platzhalter.
      const store = kind === "banner" ? app.banners : app.icons;
      for (const id of ids) store[id] = "";
    }
  }
}

// ---------- Serverliste vorladen ----------

const serverCache = new Map();

/** Erste Seite der Serverliste; Ergebnisse 30 s im Speicher. */
export function firstServers(placeId, order, excludeFull) {
  const key = `${placeId}|${order}|${excludeFull}`;
  const hit = serverCache.get(key);
  if (hit && Date.now() - hit.t < 30_000) return hit.p;
  const p = call("servers", { placeId, cursor: null, order, excludeFull });
  serverCache.set(key, { t: Date.now(), p });
  p.catch(() => serverCache.delete(key));
  return p;
}

/** Beim Überfahren einer Kachel: Serverliste schon anfragen, damit der Klick sofort wirkt. */
export function prefetch(game) {
  if (!game?.placeId || !app.config) return;
  firstServers(game.placeId, "Desc", app.config.hideFullServers ?? false);
}

// ---------- Start ----------

export async function init() {
  const [config, roblox, activity] = await Promise.all([
    call("config_get"),
    call("roblox_status").catch(() => app.roblox),
    call("activity_get").catch(() => ({ state: "idle" })),
  ]);
  app.config = config;
  app.pinned = config.pinned;
  app.roblox = roblox;
  if (activity?.state) app.activity = activity;
  call("account_get").then((a) => (app.account = a)).catch(() => {});
  on("activity", (a) => {
    app.activity = a;
    app.roblox.running = a.state !== "idle";
  });
}

export async function saveConfig(patch) {
  app.config = await call("config_set", { config: { ...$state.snapshot(app.config), ...patch } });
}

// ---------- Roblox starten ----------


export const starting = $state({ key: null });

/** Spiel (optional bestimmter Server) oder ohne Spiel die Roblox-App starten. */
export async function launch(game = null, jobId = null) {
  const key = game ? `${game.placeId}|${jobId ?? ""}` : "app";
  if (starting.key) return;
  starting.key = key;
  try {
    await call("launch", { placeId: game?.placeId ?? null, jobId });
    toast(game ? (jobId ? `Roblox startet ${game.name} auf dem gewählten Server` : `Roblox startet ${game.name}`) : "Roblox startet");
  } catch (e) {
    const f = fehlerText(e);
    toast(f.titel, "fehler", f.hinweis);
  } finally {
    setTimeout(() => (starting.key = null), 1200);
  }
}

// ---------- Seitenleiste einklappen ----------
// Automatisch eingeklappt bei schmalem Fenster. Ein Klick auf den Knopf gilt,
// bis das Fenster die Schwelle wieder überschreitet; dann übernimmt die Automatik.

const SCHWELLE = 1100;

function gelesen() {
  try {
    const v = localStorage.getItem("brickstrap.leiste");
    return v === "zu" ? true : v === "offen" ? false : null;
  } catch {
    return null;
  }
}

export const leiste = $state({
  breite: typeof window !== "undefined" ? window.innerWidth : 1280,
  manuell: gelesen(),
});

export const eingeklappt = () => leiste.manuell ?? leiste.breite < SCHWELLE;

export function leisteUmschalten() {
  leiste.manuell = !eingeklappt();
  try {
    localStorage.setItem("brickstrap.leiste", leiste.manuell ? "zu" : "offen");
  } catch {
    // Ohne Speicher gilt die Wahl nur bis zum Neustart.
  }
}

if (typeof window !== "undefined") {
  window.addEventListener("resize", () => {
    const vorher = leiste.breite < SCHWELLE;
    leiste.breite = window.innerWidth;
    if (vorher !== leiste.breite < SCHWELLE && leiste.manuell !== null) {
      leiste.manuell = null;
      try {
        localStorage.removeItem("brickstrap.leiste");
      } catch {
        // egal
      }
    }
  });
}
