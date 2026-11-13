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
