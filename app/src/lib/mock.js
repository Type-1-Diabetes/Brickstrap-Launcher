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

let angemeldet = !modus.has("abgemeldet");

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
  updateCheck: true,
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
  { id: "max-fps", name: "Max FPS", description: "Niedrigste Stufen, kein Gras, statisches Licht, kleinere Renderauflösung. Für schwache PCs und Laptops.", values: { ...MAX_FPS, DFIntDebugDynamicRenderKiloPixels: "720" } },
  { id: "ausgewogen", name: "Ausgewogen", description: "Hohe Texturen und 2× Kantenglättung. Ein guter Start für die meisten PCs.", values: { DFFlagTextureQualityOverrideEnabled: "True", DFIntTextureQualityOverride: "2", FIntDebugForceMSAASamples: "2" } },
  { id: "qualitaet", name: "Hohe Qualität", description: "Beste Texturen, 4× Kantenglättung, höchste Grafikstufe und weites Gras. Kostet viel Leistung.", values: { DFFlagTextureQualityOverrideEnabled: "True", DFIntTextureQualityOverride: "3", FIntDebugForceMSAASamples: "4", DFIntDebugFRMQualityLevelOverride: "21", FIntFRMMaxGrassDistance: "400" } },
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
  month: [],
};
// Sitzungen der letzten 30 Tage, gleichmäßig erfunden (kein Zufall, damit die Vorschau stabil bleibt).
{
  const heute = jetzt();
  const wer = [playtime.games[0], playtime.games[1], playtime.games[0], playtime.games[2], playtime.games[3], playtime.games[0]];
  for (let tag = 0; tag < 30; tag++) {
    if (tag % 7 === 4) continue; // ein Tag Pause pro Woche
    const n = 1 + ((tag * 5) % 3);
    for (let k = 0; k < n; k++) {
      const g = wer[(tag + k * 2) % wer.length];
      const secs = 900 + ((tag * 37 + k * 211) % 80) * 60;
      const end = heute - tag * 86400 - 1800 - k * 7200;
      playtime.month.push({ universeId: g.universeId, placeId: g.placeId, name: g.name, start: end - secs, end, secs, jobId: "0a1b2c3d-0000-4444-8888-aabbccddeeff" });
    }
  }
  playtime.month.sort((a, b) => b.start - a.start);
  playtime.recent = playtime.month.slice(0, 30);
}

let activity = { state: "idle", placeId: null, jobId: null, universeId: null, serverIp: null, region: null, serverPlaying: null, serverMax: null, since: null, gameName: null, iconUrl: null };
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
  // Der vierte Server der ersten Seite ist der von „Feuerfuchs“ (friends_online), damit die Markierung zu sehen ist.
  if (g === games[0] && page === 0 && slice[3]) slice[3] = { ...slice[3], id: "0a1b2c3d-0000-4444-8888-aabbccddeeff" };
  return { servers: slice, nextCursor: (page + 1) * 100 < list.length ? String(page + 1) : null };
}


// ---------- Spieler (erfunden) ----------

const LEUTE = [
  [11, "FeuerFuchs", "Feuerfuchs", true, 20],
  [12, "Kiesel_07", "Kiesel", false, 200],
  [13, "Nachtwache", "Nachtwache", false, 260],
  [14, "lenny_b", "Lenny", false, 120],
  [15, "Turmbauer2011", "Turmbauer", false, 40],
  [16, "MiraMond", "Mira", false, 300],
  [17, "Feuerwehr_Fan", "Florian", false, 5],
  [18, "pixelpilot", "Pixelpilot", false, 180],
  [1, "DemoSpieler", "Demo", false, 220],
];
const leute = LEUTE.map(([userId, name, displayName, verified, hue]) => ({ userId, name, displayName, verified, hue }));

// Flache Figur aus Blöcken als Avatar (kein Verlauf).
function figur(u, size, nurKopf = false) {
  const c = (l, s = 45) => `hsl(${u.hue} ${s}% ${l}%)`;
  const haut = "hsl(30 45% 72%)";
  const body = nurKopf
    ? `<rect width="100" height="100" fill="${c(30, 20)}"/><rect x="28" y="22" width="44" height="46" rx="6" fill="${haut}"/><rect x="26" y="16" width="48" height="14" fill="${c(30, 50)}"/><rect x="38" y="40" width="5" height="6" fill="#222"/><rect x="57" y="40" width="5" height="6" fill="#222"/><rect x="18" y="72" width="64" height="28" fill="${c(55)}"/>`
    : `<rect x="38" y="8" width="24" height="22" rx="3" fill="${haut}"/><rect x="36" y="5" width="28" height="8" fill="${c(30, 50)}"/><rect x="32" y="31" width="36" height="30" fill="${c(55)}"/><rect x="18" y="31" width="13" height="30" fill="${c(48)}"/><rect x="69" y="31" width="13" height="30" fill="${c(48)}"/><rect x="33" y="62" width="16" height="30" fill="${c(24, 25)}"/><rect x="51" y="62" width="16" height="30" fill="${c(24, 25)}"/>`;
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="${size}" height="${size}" viewBox="0 0 100 100">${body}</svg>`;
  return "data:image/svg+xml;utf8," + encodeURIComponent(svg);
}
const kleinBild = (hue, seed) => bild({ hue, universeId: seed }, 150, 150);
const BADGE_NAMEN = ["Willkommen", "Erste Runde", "10 Siege", "Sammler", "Nachtschicht", "Ohne Sturz", "Schnellster Lauf", "Entdecker", "Teamspieler", "100 Stunden", "Geheimer Raum", "Meister"];
// Welche Abzeichen der Demo-Spieler hat: je Spiel ein paar.
const BESITZ = { 9001: [0, 1, 2, 3, 5, 7, 9], 9009: [0, 1, 4], 9013: [0, 1, 2, 3, 4, 5, 6, 7, 8], 9005: [0, 2], 9014: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11], 9003: [0] };
const besitzListe = Object.entries(BESITZ).flatMap(([uid, ks]) => ks.map((k) => ({ uid: Number(uid), k })))
  .toSorted((a, b) => ((a.uid * 7 + a.k * 13) % 31) - ((b.uid * 7 + b.k * 13) % 31));
const SACHEN = {
  accessoires: [["Rote Kappe", "Hat"], ["Seitenscheitel braun", "HairAccessory"], ["Sonnenbrille", "FaceAccessory"], ["Schal gestreift", "NeckAccessory"], ["Rucksack", "BackAccessory"], ["Gürteltasche", "WaistAccessory"]],
  kleidung: [["Jeansjacke mit Hoodie", "JacketAccessory"], ["Schwarze Jeans", "Pants"], ["Feuerwehr-Shirt", "Shirt"], ["Weißes T-Shirt", "TShirt"], ["Turnschuh links", "LeftShoeAccessory"], ["Turnschuh rechts", "RightShoeAccessory"]],
  emotes: [["Salutieren", "EmoteAnimation"], ["Applaus", "EmoteAnimation"], ["Winken", "EmoteAnimation"]],
  gamepaesse: [["VIP", "GamePass"], ["Doppelte Münzen", "GamePass"]],
};

let vollbildAn = false;
const handlers = {
  discord_seite: () => null,
  // Update-Dialog ansehen: Adresse mit ?neue-version
  async update_check() {
    await delay(500);
    return modus.has("neue-version") ? { version: "0.0.8", current: __APP_VERSION__, size: 11_800_000, notes: "## Neu\n- Freunde online in der rechten Spalte\n- Discord zeigt Region und Spielerzahl\n\n## Behoben\n- Spielerkennung mit CrashHandler-Logs" } : null;
  },
  async update_install() {
    const total = 11_800_000;
    for (let done = 0; done <= total; done += 900_000) {
      for (const h of listeners.get("update-progress") ?? []) h({ done: Math.min(done, total), total });
      await delay(120);
    }
    throw "FORMAT|Vorschau: Im Browser wird nichts installiert.";
  },
  async friends_online() {
    await delay(300);
    if (modus.has("leer") || modus.has("abgemeldet")) return [];
    const g = games[0];
    return [
      { userId: 11, name: "FeuerFuchs", displayName: "Feuerfuchs", avatarUrl: null, status: "spielt", gameName: g.name, placeId: g.placeId, universeId: g.universeId, jobId: "0a1b2c3d-0000-4444-8888-aabbccddeeff" },
      { userId: 12, name: "Kiesel_07", displayName: "Kiesel", avatarUrl: null, status: "spielt", gameName: games[1]?.name ?? g.name, placeId: games[1]?.placeId ?? g.placeId, universeId: games[1]?.universeId ?? g.universeId, jobId: null },
      { userId: 13, name: "Nachtwache", displayName: "Nachtwache", avatarUrl: null, status: "studio", gameName: null, placeId: null, universeId: null, jobId: null },
      { userId: 14, name: "lenny_b", displayName: "Lenny", avatarUrl: null, status: "online", gameName: null, placeId: null, universeId: null, jobId: null },
    ];
  },
  async user_search({ query, cursor }) {
    await delay(400);
    if (modus.has("offline")) throw { code: "NETZ", message: "Demo" };
    const q = query.trim().replace(/^@/, "").toLowerCase();
    if (q === "fehler") throw { code: "RATE_LIMIT", message: "30" };
    const treffer = leute.filter((u) => String(u.userId) === q || u.name.toLowerCase().includes(q) || u.displayName.toLowerCase().includes(q));
    const page = Number(cursor || 0);
    return {
      users: treffer.slice(page * 6, page * 6 + 6).map(({ hue, ...u }) => ({ ...u, avatarUrl: figur({ hue }, 150, true) })),
      nextCursor: (page + 1) * 6 < treffer.length ? String(page + 1) : null,
    };
  },
  async user_profile({ userId }) {
    await delay(450);
    if (modus.has("offline")) throw { code: "NETZ", message: "Demo" };
    const u = leute.find((x) => x.userId === userId) ?? leute[0];
    const g = games[(userId * 3) % games.length];
    const status = userId === 11 ? "spielt" : userId === 14 ? "online" : userId === 13 ? "studio" : "offline";
    return {
      userId: u.userId, name: u.name, displayName: u.displayName, verified: u.verified, banned: false,
      description: u.userId === 12 ? "" : `Demo-Profil von ${u.displayName}. Alles hier ist erfunden.\nBaue gern Türme und spiele abends mit Freunden.`,
      created: "2019-03-14T16:20:00Z",
      friends: 120 + u.userId * 37, followers: u.userId * 3, following: 6 + u.userId,
      avatarUrl: figur(u, 420), headshotUrl: figur(u, 150, true),
      presence: { status, gameName: status === "spielt" ? g.name : null, placeId: status === "spielt" ? g.placeId : null, universeId: status === "spielt" ? g.universeId : null, jobId: status === "spielt" ? "0a1b2c3d-0000-4444-8888-aabbccddeeff" : null },
      inventoryPublic: userId !== 13,
      wearing: [...SACHEN.kleidung.slice(0, 3), ...SACHEN.accessoires.slice(1, 2), ...SACHEN.emotes.slice(0, 2)].map(([name, kind], k) => ({ id: 1000 + k, name, kind, iconUrl: kleinBild((u.hue + k * 40) % 360, 400 + k) })),
      games: userId === 15 ? games.slice(10, 13) : userId === 11 ? games.slice(2, 4) : [],
    };
  },
  async user_badges({ cursor }) {
    await delay(500);
    const page = Number(cursor || 0);
    const slice = besitzListe.slice(page * 25, page * 25 + 25);
    const ids = [...new Set(slice.map((b) => b.uid))];
    return {
      badges: slice.map(({ uid, k }) => {
        const g = games.find((x) => x.universeId === uid);
        return { id: uid * 100 + k, name: BADGE_NAMEN[k], description: `Demo-Abzeichen aus „${g.name}“.`, iconUrl: kleinBild((g.hue + k * 53) % 360, g.universeId + k * 11), universeId: k === 6 && uid === 9013 ? null : uid };
      }),
      games: ids.map((id) => games.find((x) => x.universeId === id)).map((g) => ({ universeId: g.universeId, placeId: g.placeId, name: g.name })),
      nextCursor: (page + 1) * 25 < besitzListe.length ? String(page + 1) : null,
    };
  },
  async user_game_badges({ universeId }) {
    await delay(450);
    const g = games.find((x) => x.universeId === universeId) ?? games[0];
    const hat = BESITZ[universeId] ?? [];
    return BADGE_NAMEN.map((name, k) => ({
      id: universeId * 100 + k, name, description: `Demo-Abzeichen aus „${g.name}“.`, iconUrl: kleinBild((g.hue + k * 53) % 360, g.universeId + k * 11),
      winRate: k === 11 ? 0.04 : Math.round(8000 / (k + 1)) / 100,
      awardedAt: hat.includes(k) ? new Date(Date.now() - (k + 1) * 9 * 86400e3).toISOString() : null,
    }));
  },
  async user_inventory({ category }) {
    await delay(400);
    if (category === "gamepaesse" && modus.has("offline")) throw { code: "HTTP", message: "403" };
    return { items: (SACHEN[category] ?? []).map(([name, kind], k) => ({ id: 2000 + k + category.length * 50, name, kind, iconUrl: kleinBild((k * 61 + category.length * 30) % 360, 700 + k + category.length) })), nextCursor: null };
  },
  vollbild: () => (vollbildAn = !vollbildAn),
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
    setTimeout(() => emit({ ...activity, state: "joining", placeId, jobId: jobId ?? "0a1b2c3d-0000-4444-8888-aabbccddeeff", universeId: g.universeId, serverIp: "128.116.44.33", region: "Frankfurt, DE", serverPlaying: 7, serverMax: 30, since: jetzt() }), 400);
    setTimeout(() => emit({ ...activity, state: "playing", gameName: g.name, iconUrl: bild(g, 256, 256) }), 1800);
    return null;
  },
  async invite_link({ placeId, jobId }) {
    return jobId ? `https://www.roblox.com/games/start?placeId=${placeId}&gameInstanceId=${jobId}` : `https://www.roblox.com/games/${placeId}`;
  },
  async roblox_update() {
    return { installed: "version-demo", latest: modus.has("update") ? "version-neu" : "version-demo", updateAvailable: modus.has("update") };
  },
  async game_votes({ universeIds }) {
    await delay(150);
    return Object.fromEntries(universeIds.map((id) => games.find((g) => g.universeId === id)).filter(Boolean).map((g) => [g.universeId, { upVotes: g.upVotes, downVotes: g.downVotes }]));
  },
  async search_suggest({ query }) {
    await delay(80);
    const q = query.trim().toLowerCase();
    if (q.length < 2 || modus.has("offline")) return [];
    return games.filter((g) => g.name.toLowerCase().includes(q)).slice(0, 6).map((g) => g.name.toLowerCase());
  },
  async game_extras({ universeId }) {
    await delay(400);
    if (modus.has("offline") || modus.has("leer")) return { badges: [], passes: [], events: [] };
    const g = games.find((x) => x.universeId === universeId) ?? games[0];
    const mini = (k) => bild({ hue: (g.hue + k * 53) % 360, universeId: g.universeId + k * 11 }, 150, 150);
    const namen = ["Willkommen", "Erste Runde", "10 Siege", "Sammler", "Nachtschicht", "Ohne Sturz", "Schnellster Lauf", "Entdecker", "Teamspieler", "100 Stunden", "Geheimer Raum", "Meister"];
    return {
      badges: namen.map((name, k) => ({
        id: 500 + k, name, description: k % 3 === 0 ? "" : `Demo-Abzeichen Nummer ${k + 1}.`, iconUrl: mini(k),
        awarded: Math.round(g.playing * 900 / (k + 1)), winRate: k === 11 ? 0.04 : Math.round(8000 / (k + 1)) / 100,
      })),
      passes: [["VIP", 399], ["Doppelte Münzen", 149], ["Großes Lager", 75], ["Altes Abo", null], ["Radio", 250]].map(([name, price], k) => ({ id: 800 + k, name, price, iconUrl: mini(k + 20) })),
      events: g.universeId % 2 ? [] : [
        { id: "e1", title: "Winter-Update", subtitle: "Neue Karte und Schneebälle", start: new Date(Date.now() - 3600e3).toISOString(), end: new Date(Date.now() + 2 * 86400e3).toISOString() },
        { id: "e2", title: "Doppelte Punkte", subtitle: "", start: new Date(Date.now() + 3 * 86400e3).toISOString(), end: new Date(Date.now() + 4 * 86400e3).toISOString() },
      ],
    };
  },
  async discord_builtin() { return true; },
  async auth_status() { return angemeldet ? { loggedIn: true, userId: 1, name: "DemoSpieler", displayName: "Demo" } : { loggedIn: false }; },
  async auth_login() {
    setTimeout(() => { angemeldet = true; for (const h of listeners.get("auth") ?? []) h({ loggedIn: true, userId: 1, name: "DemoSpieler", displayName: "Demo" }); }, 1200);
    return null;
  },
  async auth_logout() { angemeldet = false; for (const h of listeners.get("auth") ?? []) h({ loggedIn: false }); return null; },
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
  async playtime_get() { return modus.has("leer") ? { games: [], recent: [], month: [] } : structuredClone(playtime); },
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

// Wie in Rust: Suche kurz nach dem Start meldet ein Update per Ereignis.
setTimeout(async () => {
  const info = await handlers.update_check();
  if (info) for (const h of listeners.get("update") ?? []) h(info);
}, 1500);

export function on(event, handler) {
  if (!listeners.has(event)) listeners.set(event, new Set());
  listeners.get(event).add(handler);
  return () => listeners.get(event).delete(handler);
}

// Für Tests in der Konsole: demoBeenden() beendet das Demo-Spiel.
if (typeof window !== "undefined") {
  window.demoBeenden = () => emit({ state: "idle", placeId: null, jobId: null, universeId: null, serverIp: null, region: null, serverPlaying: null, serverMax: null, since: null, gameName: null, iconUrl: null });
}
