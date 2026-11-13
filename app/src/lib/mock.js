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
  pinned: [
    { universeId: 9001, placeId: 70007, name: "Insel-Tycoon", pinnedAt: 0 },
    { universeId: 9009, placeId: 70063, name: "Monsterjagd", pinnedAt: 0 },
    { universeId: 9005, placeId: 70035, name: "Blockbau Arena", pinnedAt: 0 },
  ],
  discord: true,
  discordAppId: "",
  discordJoinButton: true,
  closeToTray: true,
  hideOnLaunch: true,
  hideFullServers: true,
};

const ALLOW = [
  ["FFlagDebugGraphicsPreferD3D11", "bool", "Grafik-API", "Direct3D 11 erzwingen", "Meist am stabilsten. Hilft bei Grafikfehlern unter Vulkan.", { exclusive: "grafik-api" }],
  ["FFlagDebugGraphicsPreferVulkan", "bool", "Grafik-API", "Vulkan erzwingen", "Auf manchen Grafikkarten mehr FPS, auf anderen Ruckler. Ausprobieren.", { exclusive: "grafik-api" }],
  ["FFlagDebugGraphicsPreferOpenGL", "bool", "Grafik-API", "OpenGL erzwingen", "Nur als Notlösung für ältere Grafikkarten.", { exclusive: "grafik-api" }],
  ["DFIntDebugFRMQualityLevelOverride", "int", "Qualität", "Grafikstufe erzwingen", "1 bis 21, feiner als der Regler im Spiel. 0 lässt Roblox entscheiden.", { min: 0, max: 21 }],
  ["DFFlagTextureQualityOverrideEnabled", "bool", "Qualität", "Texturqualität überschreiben", "Schaltet die Einstellung darunter frei.", {}],
  ["DFIntTextureQualityOverride", "int", "Qualität", "Texturqualität", "Niedrig spart Grafikspeicher, hoch sieht schärfer aus.", { options: [["0", "Niedrig"], ["1", "Mittel"], ["2", "Hoch"], ["3", "Sehr hoch"]] }],
  ["FIntDebugForceMSAASamples", "int", "Qualität", "Kantenglättung (MSAA)", "Glättet Kanten. Jede Stufe kostet Leistung.", { options: [["0", "Aus"], ["1", "1×"], ["2", "2×"], ["4", "4×"], ["8", "8×"]] }],
  ["DFFlagDebugPauseVoxelizer", "bool", "Licht und Himmel", "Lichtberechnung anhalten", "Statisches Licht und Schatten. Spürbar mehr FPS, Licht ändert sich nicht mehr.", {}],
  ["FFlagDebugSkyGray", "bool", "Licht und Himmel", "Grauer Himmel", "Ersetzt die Skybox durch Grau.", {}],
  ["DFIntCSGLevelOfDetailSwitchingDistance", "int", "Detailstufen", "Detail-Abstand", "Ab hier werden Modelle vereinfacht. 0 heißt immer die einfachste Stufe.", { min: 0, max: 10000 }],
  ["DFIntCSGLevelOfDetailSwitchingDistanceL12", "int", "Detailstufen", "Abstand Stufe 1 → 2", "Kleiner = früher einfacher.", { min: 0, max: 10000 }],
  ["DFIntCSGLevelOfDetailSwitchingDistanceL23", "int", "Detailstufen", "Abstand Stufe 2 → 3", "Kleiner = früher einfacher.", { min: 0, max: 10000 }],
  ["DFIntCSGLevelOfDetailSwitchingDistanceL34", "int", "Detailstufen", "Abstand Stufe 3 → 4", "Kleiner = früher einfacher.", { min: 0, max: 10000 }],
  ["FIntFRMMaxGrassDistance", "int", "Gras", "Gras bis Entfernung", "0 blendet Gras ganz aus.", { min: 0, max: 1000 }],
  ["FIntFRMMinGrassDistance", "int", "Gras", "Gras ab Entfernung", "Untere Grenze für dichtes Gras.", { min: 0, max: 1000 }],
  ["FIntGrassMovementReducedMotionFactor", "int", "Gras", "Grasbewegung", "Weniger Bewegung, ruhigeres Bild.", { min: 0, max: 100 }],
  ["DFFlagDisableDPIScale", "bool", "Oberfläche", "DPI-Skalierung aus", "Kleinere, schärfere Oberfläche auf hochauflösenden Bildschirmen.", {}],
  ["FFlagHandleAltEnterFullscreenManually", "bool", "Oberfläche", "Alt+Enter wie Fenster-Vollbild", "Aus = echtes Vollbild bei Alt+Enter.", {}],
  ["DFIntDebugDynamicRenderKiloPixels", "int", "Qualität", "Renderauflösung senken", "Für schwache Geräte. Wirkung noch unbestätigt.", { min: 0, max: 4000, experimental: true }],
].map(([name, kind, group, label, hint, o]) => ({
  name, kind, group, label, hint,
  min: o.min ?? null, max: o.max ?? null,
  options: o.options?.map(([value, text]) => ({ value, label: text })) ?? null,
  experimental: !!o.experimental, exclusive: o.exclusive ?? null,
}));

const PRESETS = [
  { id: "max-fps", name: "Max FPS", description: "Alles für flüssiges Bild: niedrige Details, kein Gras, statisches Licht.", values: { DFIntDebugFRMQualityLevelOverride: "1", DFFlagTextureQualityOverrideEnabled: "True", DFIntTextureQualityOverride: "0", FIntDebugForceMSAASamples: "0", DFFlagDebugPauseVoxelizer: "True", FIntFRMMaxGrassDistance: "0", FIntFRMMinGrassDistance: "0", DFIntCSGLevelOfDetailSwitchingDistance: "0", DFIntCSGLevelOfDetailSwitchingDistanceL12: "0", DFIntCSGLevelOfDetailSwitchingDistanceL23: "0", DFIntCSGLevelOfDetailSwitchingDistanceL34: "0" } },
  { id: "ausgewogen", name: "Ausgewogen", description: "Gute Optik bei stabilen FPS.", values: { DFFlagTextureQualityOverrideEnabled: "True", DFIntTextureQualityOverride: "2", FIntDebugForceMSAASamples: "2" } },
  { id: "qualitaet", name: "Qualität", description: "Schärfste Texturen, glatte Kanten, volle Grafikstufe.", values: { DFFlagTextureQualityOverrideEnabled: "True", DFIntTextureQualityOverride: "3", FIntDebugForceMSAASamples: "4", DFIntDebugFRMQualityLevelOverride: "21" } },
  { id: "schwacher-laptop", name: "Schwacher Laptop", description: "Wie Max FPS, dazu niedrigere Renderauflösung (experimentell).", values: { DFIntDebugFRMQualityLevelOverride: "1", DFFlagTextureQualityOverrideEnabled: "True", DFIntTextureQualityOverride: "0", FIntDebugForceMSAASamples: "0", DFFlagDebugPauseVoxelizer: "True", FIntFRMMaxGrassDistance: "0", DFIntDebugDynamicRenderKiloPixels: "720" } },
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
    return servers(placeId, cursor, order, excludeFull);
  },
  async universe_of_place({ placeId }) {
    return games.find((g) => g.placeId === placeId)?.universeId ?? null;
  },
  async roblox_status() { return { installed: true, running: activity.state !== "idle", version: "version-demo" }; },
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
  async config_get() { return structuredClone(config); },
  async config_set({ config: c }) { config = { ...c, pinned: config.pinned }; return structuredClone(config); },
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
  async flags_set({ values, preset }) { flags = { values: { ...values }, preset }; return null; },
  async flags_presets() { return PRESETS; },
  async basic_get() {
    await delay(80);
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
  async playtime_get() { return structuredClone(playtime); },
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
