# Brickstrap

Ein schneller, quelloffener Launcher für Roblox unter Windows, gebaut mit Rust und Tauri. Er startet den offiziellen Roblox-Client nur von außen: keine Injection, keine Änderung an der Client-Datei.

Kein Produkt von Roblox Corporation.

**Stand:** v0.1.0 in Arbeit, noch nicht auf Windows getestet.

## Was es kann

- Startseite mit angepinnten Spielen, zuletzt gespielten und beliebten Spielen
- Suche nach Name, Place-ID oder Link zur Spielseite
- Serverliste je Spiel: sortieren, volle Server ausblenden, kleinsten Server oder besten Ping mit einem Klick betreten
- FastFlags (nur die von Roblox erlaubten) mit Presets wie „Max FPS“
- Roblox-Einstellungen wie FPS-Limit und Mausempfindlichkeit
- Discord-Status und Spielzeit, gelesen aus der Logdatei von Roblox
- Kein Login nötig. Brickstrap startet Roblox nur per Link und ändert nichts am Client.

## Entwickeln

```
cd app
npm install
npm run dev        # Browser-Vorschau mit Demo-Daten
npx tauri dev      # als Programm (Windows)
```

- Plan und Recherche: [docs/research/roblox-launcher-rust.md](docs/research/roblox-launcher-rust.md)
- Produkt: [PRODUCT.md](PRODUCT.md) · Logo und Farben: [DESIGN.md](DESIGN.md) · Logo-Dateien: [brand/](brand/)
- Für Entwicklung: [CLAUDE.md](CLAUDE.md)

## Release

GitHub → *Actions → Release → Run workflow*, Branch `main`, Version z. B. `v0.1.0`. Der Workflow baut den Windows-Installer und hängt ihn mit Prüfsumme an das Release. Normale Pushes bauen nichts.
