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
