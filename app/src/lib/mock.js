import allowlist from "../../src-tauri/resources/allowlist.json";
// Browser-Mock: erfundene Demo-Daten, damit sich die Oberfläche ohne Tauri und ohne
// Roblox-Zugang entwickeln und prüfen lässt. Alle Spiele, Zahlen und Bilder sind synthetisch.

const SPIELE = [
  ["Turmlauf", 18200, 4100, 820, 60],
  ["Insel-Tycoon", 92800, 18000, 2100, 210],
  ["Neon-Rennen", 6400, 1600, 400, 10],
  ["Schattenwald", 31800, 7200, 900, 330],
  ["Pizzabude Simulator", 12500, 3300, 500, 25],
  ["Blockbau Arena", 44100, 9900, 1500, 270],
  ["Gefängnisausbruch", 27600, 5200, 1900, 0],
  ["Weltraumstation 9", 3900, 1100, 210, 190],
  ["Parkour Profis", 15400, 3800, 640, 120],
  ["Monsterjagd", 52300, 12100, 2400, 350],
  ["Bahnhof Simulator", 2100, 900, 90, 230],
  ["Versteckspiel XXL", 8800, 2600, 300, 45],
  ["Drachenreiter", 23900, 6100, 1300, 15],
  ["Feuerwehr Einsatz", 11200, 2900, 450, 5],
  ["Obby der 1000 Stufen", 39700, 8800, 2600, 140],
  ["Hai-Überleben", 7300, 2100, 520, 200],
];

const games = SPIELE.map(([name, playing, up, down, hue], i) => ({
  universeId: 9000 + i,
  placeId: 70000 + i * 7,
  name,
  playing,
  upVotes: up * 10,
  downVotes: down * 10,
  hue,
}));

const delay = (ms) => new Promise((r) => setTimeout(r, ms));

// Zustände zum Ansehen über die Adresse, z. B. http://localhost:1420/?leer&offline&ohne-roblox
const modus = new Set(typeof location !== "undefined" ? [...new URLSearchParams(location.search).keys()] : []);

// Flaches Demo-Bild aus Blöcken (kein Verlauf), je Spiel eigene Farbe.
function bild(g, w, h) {
  const c = (l, s = 55) => `hsl(${g.hue} ${s}% ${l}%)`;
  const seed = g.universeId;
  let blocks = "";
  for (let i = 0; i < 7; i++) {
    const x = ((seed * (i + 3) * 37) % 100) / 100;
    const bh = 0.25 + (((seed * (i + 5)) % 50) / 100);
    blocks += `<rect x="${(x * w).toFixed(0)}" y="${(h * (1 - bh)).toFixed(0)}" width="${(w * 0.18).toFixed(0)}" height="${(h * bh).toFixed(0)}" fill="${c(28 + i * 5, 40)}"/>`;
  }
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="${w}" height="${h}" viewBox="0 0 ${w} ${h}"><rect width="${w}" height="${h}" fill="${c(62, 60)}"/><circle cx="${w * 0.78}" cy="${h * 0.28}" r="${h * 0.12}" fill="${c(86, 70)}"/>${blocks}<rect y="${h * 0.86}" width="${w}" height="${h * 0.14}" fill="${c(20, 35)}"/></svg>`;
  return "data:image/svg+xml;utf8," + encodeURIComponent(svg);
}

let config = {
  pinned: modus.has("leer") ? [] : [
    { universeId: 9001, placeId: 70007, name: "Insel-Tycoon", pinnedAt: 0 },
    { universeId: 9009, placeId: 70063, name: "Monsterjagd", pinnedAt: 0 },
    { universeId: 9005, placeId: 70035, name: "Blockbau Arena", pinnedAt: 0 },
  ],
  discord: true,
  discordAppId: "",
  discordJoinButton: true,
  closeToTray: true,
  hideOnLaunch: true,
  hideFullServers: false,
  showOnExit: true,
};

// Gleiche Allowlist wie im Rust-Teil (eingebettete Datei)
const ALLOW = allowlist.flags.map((f) => ({ min: null, max: null, options: null, experimental: false, exclusive: null, ...f }));

// Gleiche Presets wie flags_presets() in fastflags.rs
const LOD = ["", "L12", "L23", "L34"].map((s) => "DFIntCSGLevelOfDetailSwitchingDistance" + s);
const MAX_FPS = {
  DFIntDebugFRMQualityLevelOverride: "1", FIntFRMMaxGrassDistance: "0", FIntFRMMinGrassDistance: "0",
  DFFlagTextureQualityOverrideEnabled: "True", DFIntTextureQualityOverride: "0", FIntDebugForceMSAASamples: "0",
  DFFlagDebugPauseVoxelizer: "True", ...Object.fromEntries(LOD.map((n) => [n, "0"])),
};
const PRESETS = [
  { id: "max-fps", name: "Max FPS", description: "Niedrigste Stufen, kein Gras, statisches Licht. Das Bild wird schlichter, die FPS steigen.", values: MAX_FPS },
  { id: "ausgewogen", name: "Ausgewogen", description: "Hohe Texturen und 2× Kantenglättung. Ein guter Start für die meisten PCs.", values: { DFFlagTextureQualityOverrideEnabled: "True", DFIntTextureQualityOverride: "2", FIntDebugForceMSAASamples: "2" } },
  { id: "qualitaet", name: "Qualität", description: "Beste Texturen, 4× Kantenglättung, höchste Grafikstufe und weites Gras. Kostet viel Leistung.", values: { DFFlagTextureQualityOverrideEnabled: "True", DFIntTextureQualityOverride: "3", FIntDebugForceMSAASamples: "4", DFIntDebugFRMQualityLevelOverride: "21", FIntFRMMaxGrassDistance: "400" } },
  { id: "schwacher-laptop", name: "Schwacher Laptop", description: "Wie Max FPS, dazu eine niedrigere Renderauflösung (experimentell, Wirkung unklar).", values: { ...MAX_FPS, DFIntDebugDynamicRenderKiloPixels: "720" } },
];

let flags = { values: { ...PRESETS[1].values }, preset: "ausgewogen" };

let basic = [
  ["int", "FramerateCap", "240", "FPS-Limit", "0 = ohne Limit. Roblox bietet offiziell bis 240.", 0, 1000, 1],
  ["token", "SavedQualityLevel", "6", "Grafikqualität", "0 = automatisch, 1 bis 10 = fest.", 0, 10, 1],
  ["float", "MouseSensitivity", "0.62", "Mausempfindlichkeit", "Wie im Spielmenü, hier auch feiner einstellbar.", 0.01, 4, 0.01],
  ["float", "MasterVolume", "0.8", "Lautstärke", "0 bis 1.", 0, 1, 0.05],
  ["bool", "Fullscreen", "false", "Vollbild", "Startet Roblox im Vollbild.", null, null, null],
  ["bool", "ReducedMotion", "false", "Weniger Bewegung", "Dämpft Kamerawackeln und Effekte.", null, null, null],
  ["float", "PreferredTransparency", "1", "Transparenz der Menüs", "1 = normal, 0 = undurchsichtig.", 0, 1, 0.05],
  ["token", "CameraMode", "0", null, null, null, null, null],
  ["bool", "VREnabled", "false", null, null, null, null, null],
  ["string", "StartScreenPosition", "0,0", null, null, null, null, null],
].map(([kind, name, value, label, hint, min, max, step]) => ({
  kind, name, value, label, hint, min, max, step,
  editable: ["int", "float", "bool", "token"].includes(kind),
}));

const jetzt = () => Math.floor(Date.now() / 1000);
const playtime = {
  games: [
    { universeId: 9001, placeId: 70007, name: "Insel-Tycoon", totalSecs: 41 * 3600 + 1200, sessions: 63, lastPlayed: jetzt() - 3 * 3600 },
    { universeId: 9009, placeId: 70063, name: "Monsterjagd", totalSecs: 17 * 3600, sessions: 28, lastPlayed: jetzt() - 26 * 3600 },
    { universeId: 9014, placeId: 70098, name: "Obby der 1000 Stufen", totalSecs: 5 * 3600 + 900, sessions: 11, lastPlayed: jetzt() - 4 * 86400 },
    { universeId: 9002, placeId: 70014, name: "Neon-Rennen", totalSecs: 2 * 3600, sessions: 6, lastPlayed: jetzt() - 9 * 86400 },
  ],
  recent: [],
};

let activity = { state: "idle", placeId: null, jobId: null, universeId: null, serverIp: null, since: null, gameName: null, iconUrl: null };
const listeners = new Map();

function servers(placeId, cursor, order, excludeFull) {
  const g = games.find((x) => x.placeId === placeId) ?? games[0];
  const max = g.playing > 40000 ? 50 : g.playing > 10000 ? 30 : 12;
  const page = Number(cursor || 0);
  const total = Math.min(700, Math.max(8, Math.round(g.playing / (max * 0.7))));
  let list = [];
  for (let i = 0; i < total; i++) {
    const r = ((i + 1) * 7919 + g.universeId) % 997;
    const r2 = (((i + 3) * 104729 + g.universeId * 13) % 1000) / 1000;
    // grob: viele volle, viele halbvolle, einige fast leere Server
    const anteil = r2 < 0.3 ? 1 : r2 < 0.85 ? 0.2 + (r2 - 0.3) * 1.3 : 0.04 + (r2 - 0.85) * 2;
    const playing = Math.max(1, Math.round(max * Math.min(1, anteil)));
    list.push({
      id: `${(r * 104729).toString(16).padStart(8, "0").slice(0, 8)}-${(i + 4096).toString(16)}-4c1a-9e2f-${(r * 31337 + i).toString(16).padStart(12, "0").slice(-12)}`,
      playing: Math.min(max, playing),
      maxPlayers: max,
      ping: 18 + (r % 160),
      fps: 59.5 + (r % 10) / 20,
      playerTokens: Array.from({ length: Math.min(max, playing) }, (_, k) => `T${g.universeId}-${i}-${k}`),
    });
  }
  if (excludeFull) list = list.filter((s) => s.playing < s.maxPlayers);
  list = list.toSorted((a, b) => (order === "Asc" ? a.playing - b.playing : b.playing - a.playing));
  const slice = list.slice(page * 100, page * 100 + 100);
  return { servers: slice, nextCursor: (page + 1) * 100 < list.length ? String(page + 1) : null };
}

const handlers = {
  async home_feed({ fresh }) {
    await delay(fresh ? 600 : 5);
    if (!fresh) return null;
    if (modus.has("offline")) throw { code: "NETZ", message: "Demo: keine Verbindung" };
    const pick = (o) => games.filter((_, i) => (i + o) % 3 !== 0);
    return {
      fetchedAt: jetzt(),
      sorts: [
        { id: "top-trending", title: "Gerade beliebt", games: pick(0) },
        { id: "up-and-coming", title: "Im Kommen", games: pick(1).toReversed() },
        { id: "top-playing-now", title: "Meiste Spieler jetzt", games: games.toSorted((a, b) => b.playing - a.playing) },
      ],
    };
  },
  async search({ query }) {
    await delay(350);
    const q = query.trim().toLowerCase();
    if (q === "fehler") throw { code: "RATE_LIMIT", message: "30" };
    return { games: games.filter((g) => g.name.toLowerCase().includes(q)), nextPage: null };
  },
  async game_details({ universeIds }) {
    await delay(250);
    if (modus.has("offline")) throw { code: "NETZ", message: "Demo" };
    return universeIds.map((id) => games.find((g) => g.universeId === id)).filter(Boolean).map((g) => ({
      universeId: g.universeId, placeId: g.placeId, name: g.name,
      description: `Demo-Beschreibung für „${g.name}“. Alle Daten in der Vorschau sind erfunden.\n\nBaue, sammle und spiel mit Freunden. Neue Inhalte jeden Freitag.`,
      creatorName: "Demo Studio", creatorVerified: g.universeId % 2 === 0,
      playing: g.playing, visits: g.playing * 51234, maxPlayers: g.playing > 40000 ? 50 : g.playing > 10000 ? 30 : 12,
      favorites: g.playing * 812, genre: "Abenteuer", created: "2021-04-02T10:00:00Z", updated: "2026-11-10T18:00:00Z",
    }));
  },
  async thumbnails({ universeIds, kind }) {
    await delay(120);
    const out = {};
    for (const id of universeIds) {
      const g = games.find((x) => x.universeId === id);
      if (g) out[id] = kind === "banner" ? bild(g, 768, 432) : bild(g, 256, 256);
    }
    return out;
  },
  async servers({ placeId, cursor, order, excludeFull }) {
    await delay(500);
    if (modus.has("offline")) throw { code: "RATE_LIMIT", message: "30" };
    return servers(placeId, cursor, order, excludeFull);
  },
  async universe_of_place({ placeId }) {
    return games.find((g) => g.placeId === placeId)?.universeId ?? null;
  },
  async roblox_status() { return { installed: !modus.has("ohne-roblox"), running: activity.state !== "idle", version: "version-demo" }; },
  async launch({ placeId, jobId }) {
    await delay(300);
    const g = games.find((x) => x.placeId === placeId);
    if (!g) return null;
    setTimeout(() => emit({ ...activity, state: "joining", placeId, jobId: jobId ?? null, universeId: g.universeId, since: jetzt() }), 400);
    setTimeout(() => emit({ ...activity, state: "playing", gameName: g.name, iconUrl: bild(g, 256, 256) }), 1800);
    return null;
  },
  async invite_link({ placeId }) { return `https://www.roblox.com/games/${placeId}`; },
  async discord_builtin() { return false; },
  async server_avatars({ tokens }) {
    await delay(300);
    return tokens.map((t, k) => bild({ hue: (k * 47) % 360, universeId: 300 + k }, 150, 150));
  },
  async account_get() { return modus.has("ohne-roblox") ? null : { userId: 1, name: "DemoSpieler", displayName: "Demo", avatarUrl: null }; },
  async game_media({ universeId }) {
    await delay(200);
    const g = games.find((x) => x.universeId === universeId) ?? games[0];
    return [0, 40, 90, 150, 210, 280].map((d) => bild({ ...g, hue: (g.hue + d) % 360, universeId: g.universeId + d }, 768, 432));
  },
  async pin_cover_set({ universeId, cover }) {
    const p = config.pinned.find((x) => x.universeId === universeId);
    if (p) p.cover = cover ?? null;
    return structuredClone(config.pinned);
  },
  async config_get() { return structuredClone(config); },
  async config_set({ config: c }) { config = { ...c, pinned: config.pinned, discordAppId: String(c.discordAppId ?? "").replace(/\D/g, "") }; return structuredClone(config); },
  async pin_set({ universeId, placeId, name, pinned }) {
    config.pinned = config.pinned.filter((p) => p.universeId !== universeId);
    if (pinned) config.pinned.unshift({ universeId, placeId, name, pinnedAt: jetzt() });
    return structuredClone(config.pinned);
  },
  async pin_order({ order }) {
    config.pinned.sort((a, b) => order.indexOf(a.universeId) - order.indexOf(b.universeId));
    return structuredClone(config.pinned);
  },
  async flags_get() {
    return { allowlist: ALLOW, values: { ...flags.values }, preset: flags.preset, allowlistDate: "2025-09-29", appliedTo: ["version-demo"], lastApplied: jetzt() - 7200 };
  },
  async flags_set({ values, preset }) {
    // wie fastflags.rs: Zahlen auf den erlaubten Bereich kappen
    const v = { ...values };
    for (const f of ALLOW) if (f.kind === "int" && v[f.name] != null) v[f.name] = String(Math.min(f.max ?? Infinity, Math.max(f.min ?? -Infinity, Number(v[f.name]))));
    flags = { values: v, preset };
    return null;
  },
  async flags_presets() { return PRESETS; },
  async basic_get() {
    await delay(80);
    if (modus.has("ohne-roblox")) return { path: "C:\\Users\\Demo\\AppData\\Local\\Roblox\\GlobalBasicSettings_13.xml", exists: false, running: false, entries: [] };
    return { path: "C:\\Users\\Demo\\AppData\\Local\\Roblox\\GlobalBasicSettings_13.xml", exists: true, running: activity.state !== "idle", entries: structuredClone(basic) };
  },
  async basic_set({ changes }) {
    await delay(200);
    if (activity.state !== "idle") throw { code: "LAEUFT", message: "" };
    for (const c of changes) {
      const e = basic.find((b) => b.name === c.name);
      if (e) e.value = c.value;
    }
    return null;
  },
  async activity_get() { return { ...activity }; },
  async playtime_get() { return modus.has("leer") ? { games: [], recent: [] } : structuredClone(playtime); },
};

function emit(a) {
  activity = a;
  for (const h of listeners.get("activity") ?? []) h({ ...a });
}

export async function call(cmd, args) {
  const h = handlers[cmd];
  if (!h) throw { code: "UNBEKANNT", message: `Mock kennt ${cmd} nicht` };
  return h(args);
}

export function on(event, handler) {
  if (!listeners.has(event)) listeners.set(event, new Set());
  listeners.get(event).add(handler);
  return () => listeners.get(event).delete(handler);
}

// Für Tests in der Konsole: demoBeenden() beendet das Demo-Spiel.
if (typeof window !== "undefined") {
  window.demoBeenden = () => emit({ state: "idle", placeId: null, jobId: null, universeId: null, serverIp: null, since: null, gameName: null, iconUrl: null });
}
