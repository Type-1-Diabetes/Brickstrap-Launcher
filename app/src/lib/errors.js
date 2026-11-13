// Fehlercodes aus dem Rust-Teil → Text für Menschen.

const TEXTE = {
  NETZ: ["Keine Verbindung zu Roblox", "Prüf deine Internetverbindung und versuch es noch einmal."],
  TIMEOUT: ["Roblox antwortet nicht", "Die Anfrage hat zu lange gedauert. Versuch es gleich noch einmal."],
  RATE_LIMIT: ["Zu viele Anfragen", "Roblox bremst gerade. Warte kurz, dann klappt es wieder."],
  HTTP: ["Roblox hat die Anfrage abgelehnt", "Das kann an einer Änderung bei Roblox liegen."],
  FORMAT: ["Unerwartete Antwort", "Roblox hat das Format geändert. Bitte melde das im Repo."],
  ANTWORT: ["Antwort nicht lesbar", "Roblox hat etwas Unerwartetes geschickt."],
  NICHT_INSTALLIERT: ["Roblox ist nicht installiert", "Installier Roblox einmal von roblox.com, dann startet Brickstrap es."],
  START: ["Roblox ließ sich nicht starten", "Ist Roblox installiert? Öffne es einmal direkt und versuch es dann erneut."],
  LAEUFT: ["Roblox läuft noch", "Schließ Roblox zuerst. Sonst überschreibt es deine Änderungen beim Beenden."],
};

/** Liefert { titel, hinweis } für einen Fehler. */
export function fehlerText(err) {
  const [titel, hinweis] = TEXTE[err?.code] ?? ["Etwas ist schiefgelaufen", err?.message || "Unbekannter Fehler."];
  if (err?.code === "RATE_LIMIT" && /^\d+$/.test(err.message ?? "")) {
    return { titel, hinweis: `Roblox bremst gerade. Neuer Versuch in ${err.message} s.` };
  }
  if (err?.code === "HTTP" && err.message) return { titel: `${titel} (${err.message})`, hinweis };
  return { titel, hinweis };
}
