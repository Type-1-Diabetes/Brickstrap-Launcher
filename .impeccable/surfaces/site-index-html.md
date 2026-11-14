---
version: 1
slug: "site-index-html"
primary_target: "site/index.html"
related_targets: []
---

# Surface brief: Download-Seite (GitHub Pages, Branch gh-pages)

Scope: eine Seite, `index.html` im Branch `gh-pages`, veröffentlicht unter type-1-diabetes.github.io/Brickstrap-Launcher. Modus: Persuade.
Audience: Roblox-Spieler, die Bloxstrap/Fishstrap kennen, misstrauisch gegenüber Launchern, die am Client herumschrauben.
Job: verstehen, was Brickstrap macht und was nie, dann die exe laden.
Proof: Screenshots der App mit Demo-Daten (docs/bilder), Regeln aus CLAUDE.md (keine Injection, Cookie nur an roblox.com, kein eigener Server), echte Release-Daten von der GitHub-API (Version, Größe, Datum, SHA-256).
Constraints: nur Deutsch, Du-Form; Palette und Schrift aus DESIGN.md; kein Bezug auf Roblox-Logo; kein Build-Schritt, keine Workflows; Bilder als Demo-Daten kennzeichnen; Hinweis „kein Produkt von Roblox Corporation“.
Unresolved: Lizenz; ob ein eigener Domainname kommt.

## Direction contract

THESIS: Die Seite verkauft durch Verzicht: ein zweispaltiges Register „macht“ gegen „macht nie“ ist der Hauptinhalt, nicht ein Feature-Kartenraster. Verweigert: Split-Hero mit Gerät und Kartenreihe.
OWN-WORLD: Flaches Schwarz #0B0B0D, Flächen s1–s4, Silber und Grau für Text, ein weißer Primärknopf mit schwarzer Schrift. Archivo, Überschrift weit (112–118 %) und schwer. Register-Zeilen mit 1-px-Linien, eigene SVG-Haken und Kreuze, Kreuze in Grau, nie in Signalfarben.
STORY: Der Besucher liest in einer Zeile, dass Roblox unangetastet bleibt, sieht die Grenze schwarz auf weiß, prüft Version und Prüfsumme und lädt.
FIRST VIEWPORT: Kopfleiste 64 px: Lockup links, Sprungmarken, rechts weißer Knopf „Herunterladen“ mit Version. Darunter Überschrift etwa 64 px über die halbe Breite, eine Zeile Erklärung, Meta-Zeile (Version · Größe · Datum · SHA-256 kopieren). Direkt darunter das Register in zwei gleich breiten Spalten auf s1, Kopf „Brickstrap macht“ / „Brickstrap macht nie“.
FORM: Tut / tut nicht, Position 7 der eigenen Liste, seed:602f876d151cfc8e. Signatur: Register-Zeilen bauen sich beim Laden gestaffelt auf; der Download-Knopf füllt Version und Größe live aus der GitHub-API, mit festem Rückfall auf /releases/latest.
FINISH: unreviewed and undocumented is unfinished; this build ends with the finish review, the verdict, DESIGN.md, and every shipping raster carrying its provenance
