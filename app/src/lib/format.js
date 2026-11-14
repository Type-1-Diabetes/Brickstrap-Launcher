// Zahlen, Zeiten und Dauern auf Deutsch.

const kurz = new Intl.NumberFormat("de-DE", { notation: "compact", maximumFractionDigits: 1 });
const voll = new Intl.NumberFormat("de-DE");

/** 12.345 → „12.345“, 1.234.567 → „1,2 Mio.“ */
export function zahl(n) {
  if (n == null) return "–";
  return n >= 100_000 ? kurz.format(n) : voll.format(n);
}

/** Anteil positiver Stimmen in Prozent oder null. */
export function bewertung(up, down) {
  const t = (up ?? 0) + (down ?? 0);
  return t > 0 ? Math.round((up / t) * 100) : null;
}

/** Unix-Sekunden → Beginn des lokalen Tages (Unix-Sekunden). */
export function tagBeginn(unix) {
  const d = new Date(unix * 1000);
  d.setHours(0, 0, 0, 0);
  return Math.floor(d.getTime() / 1000);
}

const wochentag = new Intl.DateTimeFormat("de-DE", { weekday: "short" });
const tagMonat = new Intl.DateTimeFormat("de-DE", { day: "numeric", month: "numeric" });
const tagLang = new Intl.DateTimeFormat("de-DE", { weekday: "long", day: "numeric", month: "long" });
const zeitpunkt = new Intl.DateTimeFormat("de-DE", { weekday: "short", day: "numeric", month: "numeric", hour: "2-digit", minute: "2-digit" });

/** Unix-Sekunden → „Mo“, „3.11.“, „Montag, 3. November“ */
export const tagKurz = (unix) => wochentag.format(unix * 1000).replace(".", "");
export const tagDatum = (unix) => tagMonat.format(unix * 1000);
export const tagVoll = (unix) => tagLang.format(unix * 1000);

/** ISO-Zeit → „Sa., 15.11., 18:00“ oder "" */
export function wann(iso) {
  const t = Date.parse(iso ?? "");
  return Number.isNaN(t) ? "" : zeitpunkt.format(t);
}

/** Sekunden → „2 Std. 5 Min.“ */
export function dauer(sek) {
  sek = Math.max(0, Math.round(sek ?? 0));
  const h = Math.floor(sek / 3600);
  const m = Math.floor((sek % 3600) / 60);
  if (h > 0) return m > 0 ? `${h} Std. ${m} Min.` : `${h} Std.`;
  if (m > 0) return `${m} Min.`;
  return "unter 1 Min.";
}

/** Unix-Sekunden → „vor 5 Min.“, „gestern“, Datum */
export function seit(unix) {
  if (!unix) return "";
  const d = Math.floor(Date.now() / 1000) - unix;
  if (d < 45) return "gerade eben";
  if (d < 3600) return `vor ${Math.max(1, Math.round(d / 60))} Min.`;
  if (d < 86400) return `vor ${Math.round(d / 3600)} Std.`;
  if (d < 2 * 86400) return "gestern";
  if (d < 7 * 86400) return `vor ${Math.round(d / 86400)} Tagen`;
  return new Date(unix * 1000).toLocaleDateString("de-DE");
}

const datumKurz = new Intl.DateTimeFormat("de-DE", { day: "numeric", month: "numeric", year: "numeric" });
const monatJahrF = new Intl.DateTimeFormat("de-DE", { month: "long", year: "numeric" });

/** ISO-Zeit → „3.11.2025“ oder "" */
export function datum(iso) {
  const t = Date.parse(iso ?? "");
  return Number.isNaN(t) ? "" : datumKurz.format(t);
}

/** ISO-Zeit → „März 2019“ oder "" */
export function monatJahr(iso) {
  const t = Date.parse(iso ?? "");
  return Number.isNaN(t) ? "" : monatJahrF.format(t);
}

/** Anteil in Prozent → „2,3 %“, „unter 0,1 %“ oder "" */
export function anteil(p) {
  if (p == null) return "";
  if (p < 0.1) return "unter 0,1 %";
  return `${p < 10 ? p.toFixed(1).replace(".", ",") : Math.round(p)} %`;
}

const ASSET_ARTEN = {
  Hat: "Hut", HairAccessory: "Haare", FaceAccessory: "Gesicht", NeckAccessory: "Hals", ShoulderAccessory: "Schulter",
  FrontAccessory: "Vorne", BackAccessory: "Rücken", WaistAccessory: "Hüfte", Face: "Gesicht",
  TShirt: "T-Shirt", Shirt: "Shirt", Pants: "Hose", TShirtAccessory: "T-Shirt", ShirtAccessory: "Shirt", PantsAccessory: "Hose",
  JacketAccessory: "Jacke", SweaterAccessory: "Pullover", ShortsAccessory: "Shorts", DressSkirtAccessory: "Kleid oder Rock",
  LeftShoeAccessory: "Schuh", RightShoeAccessory: "Schuh", Emote: "Emote", EmoteAnimation: "Emote", GamePass: "Game-Pass",
  EyebrowAccessory: "Augenbrauen", EyelashAccessory: "Wimpern", MoodAnimation: "Stimmung",
};

/** Asset-Typ von Roblox (englisch) → deutsches Wort. Animationen heißen „Animation“. */
export function assetArt(kind) {
  if (!kind) return "";
  if (ASSET_ARTEN[kind]) return ASSET_ARTEN[kind];
  if (kind.endsWith("Animation")) return "Animation";
  return kind;
}
