# Brickstrap

Ein schneller, quelloffener Launcher für Roblox unter Windows, gebaut mit Rust und Tauri. Er startet den offiziellen Roblox-Client nur von außen: keine Injection, keine Änderung an der Client-Datei.

Kein Produkt von Roblox Corporation.

**Stand:** Planung. Noch kein App-Code.

- Plan und Recherche: [docs/research/roblox-launcher-rust.md](docs/research/roblox-launcher-rust.md)
- Produkt: [PRODUCT.md](PRODUCT.md) · Logo und Farben: [DESIGN.md](DESIGN.md) · Logo-Dateien: [brand/](brand/)
- Für Entwicklung: [CLAUDE.md](CLAUDE.md)

## Release

GitHub → *Actions → Release → Run workflow*, Branch `main`, Version z. B. `v0.1.0`. Der Workflow baut den Windows-Installer und hängt ihn mit Prüfsumme an das Release. Normale Pushes bauen nichts.
