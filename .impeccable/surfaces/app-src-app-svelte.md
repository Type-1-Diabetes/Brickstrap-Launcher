---
version: 1
slug: "app-src-app-svelte"
primary_target: "app/src/App.svelte"
related_targets: []
---

# Surface brief: Brickstrap app shell (alle Seiten)

Scope: die ganze Desktop-App (Startseite, Suche, Spielseite mit Server-Picker, Favoriten, FastFlags, Roblox-Einstellungen, Verlauf/Spielzeit, App-Einstellungen). Modus: Operate.
Audience: Roblox-Power-User am Windows-PC, kurz vor dem Spielen, Discord daneben, oft abends.
Job: Spiel finden oder aus Favoriten wählen → Server wählen → beitreten; oder einfach mit den eigenen Einstellungen schnell starten.
Constraints: nur Deutsch; Logo-Palette Schwarz/Grau/Silber/Weiß; flaches Schwarz, kein Mica; Anmeldung ist Pflicht (echte Login-Seite von Roblox, Entscheidung vom 13.11.2026); nichts, was an Roblox-Logo erinnert.
Unresolved: Status-Farben (online/voll/Fehler) als neutrale Abstufungen plus eine sparsame Signalfarbe nur für Fehler; mit dem Nutzer nachschärfen.

## Direction contract

THESIS: Der bewährte Launcher-Standard (Navigation links, Spielseite mit großem Spielen-Knopf, Einstellungsgruppen mit Schaltern), gespielt ohne Ironie und auf dem Niveau von Windows-11-Einstellungen, Discord und Steam. Verweigert: Glas, Neon, Verläufe, Kartenwust.
OWN-WORLD: Flaches Schwarz #0B0B0D als Grund, zwei bis drei Graustufen als Ebenen (Leiste, Karte, Hover), Silber #C3C7CD für Sekundärtext und Symbole, Weiß für Primärtext und den Hauptknopf (weiße Fläche, schwarze Schrift). Eine Schriftfamilie (Archivo, normale Breite) für alles, tabellarische Ziffern in Listen. Ecken 8 px, Kacheln 12 px. Feine 1-px-Trennlinien nur in Tabellen.
STORY: Der Nutzer sieht sofort seine angepinnten Spiele, versteht, dass ein Klick zur Serverliste führt, und tritt mit einem zweiten Klick bei. Er vertraut, weil alles erklärt ist und die Anmeldung nur bei Roblox selbst passiert.
FIRST VIEWPORT: Links 220 px Leiste (Logo, Start, Suchen, Favoriten, FastFlags, Roblox-Einstellungen, Verlauf; unten App-Einstellungen). Oben Suchfeld über die ganze Inhaltsbreite. Inhalt: Reihe „Angepinnt“ als große Spielkacheln (Bild 16:9, Name, Spieler online), darunter „Weiterspielen“ und „Beliebt“. Unten rechts fester Knopf „Roblox starten“ mit aktivem Preset (wird „Roblox läuft“, solange gespielt wird). Angepasst beim Bau: Leiste 232 px mit Liste der angepinnten Spiele, einklappbar (automatisch unter 1100 px, Wunsch des Nutzers); Favoriten und Verlauf zusammen in „Bibliothek“; Suchfeld höchstens 620 px breit; Kacheln mit sekundärem „Spielen“-Knopf; Dock 52 px. Spielseite: Kopfbild, Name, Spielen-Knopf, Anpinnen; darunter Server-Picker als sortierbare Tabelle (Spieler mit Füllbalken, Ping, Beitreten).
FORM: Standard (Canon), vom Nutzer aus der Würfelrunde gewählt (Runde lief eingeschränkt ohne Challenger; gewürfelt war Kandidat 5 „Highscore-Tafel“, angeboten auch „Sendegrafik“), seed:4ac060aeabc2b9f8 – „Standard wie Bloxstrap“, Vorbilder Bloxstrap/Fishstrap, Windows-11-Einstellungen, Discord, Steam/Epic. Signatur-Interaktion: Hover über eine Spielkachel lädt die Serverliste vor; Serverliste füllt sich ohne Springen, Füllbalken wachsen kurz an.
FINISH: unreviewed and undocumented is unfinished; this build ends with the finish review, the verdict, DESIGN.md, and every shipping raster carrying its provenance
