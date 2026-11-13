# Brickstrap – Projekt-Kontext

> Arbeitsgrundlage für alle, die am Projekt mitarbeiten (Menschen und KI).
> **Vor jeder Änderung lesen:** [Regeln von Roblox](#regeln-von-roblox-nie-brechen) und [Vorsicht bei Änderungen](#vorsicht-bei-änderungen). Dort steht, was ein Konto sperren lässt oder still kaputtgeht.
> Recherche und Plan: [docs/research/roblox-launcher-rust.md](docs/research/roblox-launcher-rust.md). Produkt: [PRODUCT.md](PRODUCT.md). Logo und Farben: [DESIGN.md](DESIGN.md).
>
> Stand: 13.11.2026 (Repo angelegt: Release-Workflow, Logo-Kit in `brand/`, Plan in `docs/research/`. Noch kein App-Code.)

## Was ist Brickstrap?

Ein eigener, quelloffener **Launcher für Roblox unter Windows**, gebaut mit Rust und Tauri 2. Er startet den offiziellen Roblox-Client nur von außen (Deeplinks, später ein eigener Bootstrapper) und bietet Startseite, Spielesuche, Server-Picker, Editoren für FastFlags und Einstellungen sowie Discord-Status.

| | |
|---|---|
| Name | **Brickstrap**. Nie „Roblox“ oder „Blox“ im Namen, nichts, was an Logo oder Wortmarke von Roblox erinnert |
| Technik | Tauri 2: Rust-Backend (`tokio`, `reqwest`, `serde`), Oberfläche im WebView2 (Frontend-Framework noch offen, Plan: Svelte oder SolidJS mit Vite) |
| Plattform | nur Windows (x64). Kein Android, kein macOS, kein Linux |
| Nutzer | Roblox-Spieler, die Bloxstrap/Fishstrap kennen und mehr Kontrolle wollen |
| Dienste | nur die Web-APIs von Roblox und GitHub. Kein eigener Server. Das Roblox-Cookie geht an keinen fremden Server |
| Sprache | Code-Kommentare, Commits und Doku auf Deutsch. Du-Form, kurze Sätze, korrekte Umlaute. UI-Sprache offen (Deutsch zuerst, Englisch geplant) |
| Repo | privat (`Type-1-Diabetes/Brickstrap-Launcher`). Veröffentlichung als Open Source (Lizenz vermutlich MIT) ist geplant |

**Stufen** (Plan §12.9): 1. MVP ohne Login (Suche, Startseite, Server-Picker, Start per Deeplink) · 2. FastFlag- und Einstellungs-Editor, Discord-Status · 3. Login, Freunde, Region · 4. Mehrere Instanzen, Overlay-Fenster · 5. Eigener Bootstrapper + Mods · 6. Chat, Konto-Wechsel.

## Regeln von Roblox (nie brechen)

- **Tabu:** Code in den laufenden Client einschleusen (DLL-Injection), Speicher des Clients lesen oder schreiben, die Client-Datei patchen, Overlays, die sich in den Roblox-Prozess hängen (ReShade, RTSS), FPS-Unlocker per Speicherzugriff, „Speicher-Bereinigung“ am laufenden Prozess, Macros und automatische Eingaben.
- **Erlaubt bzw. geduldet:** Client herunterladen und starten, Deeplinks, Web-APIs, `ClientAppSettings.json` (nur Flags der Allowlist), `GlobalBasicSettings_13.xml`, Content-Mods (Dateien ersetzen), Logdatei lesen, Windows-Einstellungen für `RobloxPlayerBeta.exe` (Registry), eigene Fenster außerhalb von Roblox.
- Im Zweifel nicht einbauen und den Projektinhaber fragen. Grundsatz aus PRODUCT.md: nichts, was ein Konto sperren lassen könnte.

## Arbeitsweise & Regeln

- **Grundsätze aus PRODUCT.md:** Tempo ist eine Funktion (alles soll sofort reagieren). Kontrolle ohne Unordnung. Sauber von Anfang an. Offen und nachvollziehbar.
- **Workflows kurz und schnell** (Wunsch des Projektinhabers): ein einziger Workflow `release.yml`, nur ein Windows-Job. Keine Tests, kein Linting, keine Prüfschritte in GitHub Actions. Normale Pushes bauen nichts.
- **Keine Kosten bei GitHub** (Wunsch des Projektinhabers): siehe [Testbuilds und Kosten](#testbuilds-und-kosten). Keine neuen Workflows, keine Zeitpläne (`schedule`), keine Läufe bei Push oder Pull Request, kein Dependabot-Build.
- **Prüfen passiert lokal** (siehe [Prüfen](#prüfen-lokal-nicht-in-ci)).
- **Login ist optional.** Alles, was ohne Cookie geht (Suche, Startseite, Serverliste, Start per Deeplink), muss ohne Login laufen.
- **Das Cookie `.ROBLOSECURITY` ist ein Generalschlüssel:** nur im Rust-Teil, verschlüsselt im Windows-Anmeldeinformations-Manager (`keyring`). Nie an die Oberfläche, nie in Logdatei, Einstellungen, Export, Commit oder an einen anderen Host als `*.roblox.com`.
- **Fremden Code prüfen:** Bloxstrap, Fishstrap, NexStrap (MIT) dürfen als Vorlage dienen, mit Hinweis auf die Herkunft. RoValra und Roblox Account Manager sind **GPL**: nur das Prinzip übernehmen, keinen Code kopieren.
- **Undokumentierte APIs** (Suche, Explore, Chat, `gamejoin`) kapseln, Fehler abfangen, nie die ganze App davon abhängig machen. Sie können jederzeit brechen.
- **UI-Arbeit** mit dem Skill `/impeccable` (Design-System in `DESIGN.md`, Produkt in `PRODUCT.md`). **Texte** nach dem Humanizer-Skill: kurz, informativ, aktiv, keine Floskeln, keine Gedankenstriche als Stilmittel.
- **Farben:** nur Schwarz `#0B0B0D`, Grau `#8B9099`, Silber `#C3C7CD`, Weiß `#FFFFFF` für die Marke. Flach, ohne Verlauf und Schatten. **Dunkel ist Standard.** Weitere UI-Farben (Fehler, Erfolg) erst mit dem Projektinhaber festlegen und in `DESIGN.md` eintragen.
- **Schnell starten** (Plan §15): Fenster erst nach dem ersten Zeichnen zeigen, Daten zuerst aus dem Cache, dann aktualisieren; schwere Arbeit im Rust-Backend (`tokio`); lange Listen virtualisieren; im Tray weiterlaufen statt beenden.
- **Keine Geheimnisse im Repo.** Signierzertifikat nur als GitHub-Secret. Keine Cookies, Konto-Namen oder IDs echter Konten als Testdaten.
- **Nicht raten.** Endpunkte, Registry-Formate und Schlüssel in `GlobalBasicSettings_13.xml` sind teils nur aus Erinnerung oder Foren belegt (im Plan markiert). Vor dem Einbau prüfen, bei Unklarheit nachfragen.

## Testbuilds und Kosten

Das Repo ist **privat**. GitHub Free: 2.000 Actions-Minuten im Monat und 500 MB Speicher für Artefakte. **Windows-Minuten zählen doppelt.** Ein Release-Lauf mit warmem Cache braucht etwa 6–10 Minuten (also 12–20 abgerechnete), der erste Lauf ohne Cache deutlich länger. Release-Dateien kosten nichts. Ohne hinterlegte Zahlungsart entstehen keine Kosten, Läufe stoppen dann nur, wenn das Kontingent leer ist. Sobald das Repo öffentlich ist, sind Actions komplett kostenlos.

- **Kein Testbuild vor einem Release.** Ein Release baut selbst; schlägt der Build fehl, entsteht kein Release.
- **Kein Testbuild bei normalen Änderungen** an Rust, Oberfläche oder Doku. Die laufen lokal (`cargo check`, Lint, `npm run build`).
- **Testbuild** (Run workflow **ohne** Version) nur, wenn sich `tauri.conf.json` (Bundle, NSIS), Icons, Abhängigkeiten mit Windows-eigenem Code oder der Workflow selbst ändern und die lokale Prüfung das nicht abdeckt, oder wenn der Projektinhaber ihn will. Vorher kurz Bescheid sagen.
- Der Workflow lädt ein Artefakt nur beim Testbuild hoch und hält es 1 Tag. Laufzeit höchstens 30 Minuten, ein neuer Start bricht den alten ab.
- **Cache:** `rust-cache` speichert nur auf `main`. Releases darum per „Run workflow“ auf `main` starten, nicht per Tag (Tag-Läufe sehen keinen gespeicherten Cache und bauen kalt).

## Release erstellen (nur auf Anweisung)

1. Version hochzählen: `app/package.json` → `"version"` **und** `app/src-tauri/Cargo.toml` → `version` (gleich halten, z. B. `0.1.0`). `tauri.conf.json` übernimmt die Version aus `package.json`.
2. Lokal prüfen (siehe [Prüfen](#prüfen-lokal-nicht-in-ci)).
3. Committen und auf `main` pushen.
4. Release starten: GitHub → *Actions → Release → Run workflow*, Branch `main`, Feld **Version = `v0.1.0`** (mit `v`, gleich wie in `package.json`). Aus einer Claude-Session geht kein Tag-Push (HTTP 403): stattdessen `mcp__github__actions_run_trigger` mit `method: run_workflow`, `workflow_id: release.yml`, `ref: main`, `inputs: {version: "v0.1.0"}`. Leeres Feld = nur Testbuild als Artefakt, kein Release.
5. Ergebnis prüfen: Release enthält `Brickstrap_<version>_x64-setup.exe` und `.sha256`. Bei Fehlern das Log ansehen (`mcp__github__get_job_logs`).

**Nie ändern, sobald es Nutzer gibt:** `identifier` in `tauri.conf.json` (sonst installiert Windows eine zweite App neben der alten) und die Asset-Namen (`*-setup.exe`, `<datei>.sha256`), falls ein Update in der App darauf aufbaut.

Optional: Code-Signatur über die Secrets `WINDOWS_CERTIFICATE` (base64 einer `.pfx`) und `WINDOWS_CERTIFICATE_PASSWORD`. Ohne sie baut der Workflow unsigniert, SmartScreen und Defender warnen dann.

## Ordner – was liegt wo

```
Brickstrap-Launcher/
├─ CLAUDE.md                  Diese Datei (für Entwicklung)
├─ PRODUCT.md, DESIGN.md      Produkt und Design-System (impeccable)
├─ .github/workflows/
│  └─ release.yml             Windows-Installer (NSIS) bauen und releasen, nur auf Anstoß
├─ brand/                     Logo-Kit: logo/ (SVG, PNG 16–2048 px, brickstrap.ico), build_logo.py
├─ docs/research/
│  └─ roblox-launcher-rust.md Recherche, Regeln, APIs, FastFlag-Allowlist, Roadmap
└─ app/                       (geplant) Tauri-App
   ├─ package.json            Version der App, Skripte (dev, build, lint)
   ├─ src/                    Oberfläche
   └─ src-tauri/              Rust-Kern, tauri.conf.json, capabilities/, icons/
```

Der Workflow erwartet die App unter `app/` mit `app/package-lock.json` und `app/src-tauri/tauri.conf.json`. Wer die Struktur ändert, passt `release.yml` mit an.

## Abhängigkeiten – was hängt woran?

| Was | Stellen |
|---|---|
| Version | `app/package.json` ↔ `app/src-tauri/Cargo.toml` ↔ Workflow-Input `version` |
| Pfade im Workflow | `release.yml` (`app/`, `app/src-tauri`, `bundle/nsis/*-setup.exe`) ↔ Ordnerstruktur ↔ `bundle.targets` in `tauri.conf.json` |
| Farben, Logo | `brand/` ↔ `DESIGN.md` ↔ CSS-Tokens der App (`:root`) ↔ `src-tauri/icons/` (aus `brand/logo/png` und `brickstrap.ico`) |
| FastFlag-Allowlist | eigene Datei (nicht im Code fest eingebaut) ↔ Editor ↔ Plan §5. Roblox ändert die Liste ohne Vorwarnung |
| Rust-Befehle | `#[tauri::command]` ↔ `generate_handler!` ↔ `invoke("…")` in der Oberfläche (Namen gleich halten) |
| Externe Hosts | `capabilities/*.json` (HTTP-Scope) ↔ CSP in `tauri.conf.json` ↔ tatsächlich genutzte Endpunkte |

## Vorsicht bei Änderungen

- **`ClientAppSettings.json`** liegt im Versionsordner (`%localappdata%\Roblox\Versions\version-…\ClientSettings\`) und wird bei **jedem Roblox-Update geleert**. Vor jedem Start neu schreiben, nur wenn sich etwas geändert hat (Prüfsumme). Werte als Strings. Nicht erlaubte Flags werden still ignoriert: im Editor als „wirkungslos“ markieren statt zu verstecken.
- **`GlobalBasicSettings_13.xml`** nur schreiben, wenn Roblox **komplett geschlossen** ist, sonst überschreibt Roblox die Änderung beim Beenden. Keinen festen Schlüsselkatalog einbauen, die Werte der Datei des Nutzers lesen und anzeigen.
- **Registry-Einträge pro exe-Pfad** (GPU-Vorgabe, Vollbild-Optimierungen) hängen am Versionsordner und müssen nach jedem Roblox-Update neu gesetzt werden. Formate vorher an einem echten Windows prüfen (Plan §13.3).
- **Protokoll-Handler** (`roblox-player`, `roblox` unter `HKCU\Software\Classes\`) erst mit eigenem Bootstrapper (Stufe 5) umbiegen. Beim Deinstallieren auf den offiziellen Launcher zurückstellen, sonst startet Roblox nicht mehr.
- **Mehrere Instanzen:** Mutex `ROBLOX_singletonEvent` bzw. `ROBLOX_singletonMutex` vor Roblox anlegen und halten. Nicht offiziell unterstützt, Teleports können scheitern. Nur als Option, standardmäßig aus.
- **Serverliste** (`games.roblox.com/v1/games/{placeId}/servers/Public`): Seiten über `cursor`, nach etwa 700 Servern ist Schluss. Abfragen drosseln und zwischenspeichern (Rate-Limit).
- **Konto-Wechsel und mehrere Cookies** sind ein Ziel für Schadsoftware. Erst nach dem MVP und nur mit sicherem Speicher.
- **HTTP aus der App** nur zu Hosts im Scope der Capabilities. Neue API → Scope und CSP ergänzen, sonst schlägt die Anfrage still fehl.
- **Overlay-Fenster** nur als eigenes transparentes Tauri-Fenster (`always_on_top`, Klick-Durchlass). Es zeigt nur, was der Launcher von außen weiß.
- **Rust lokal in Claude-Cloud-Sessions:** Tauri braucht unter Linux GTK/WebKit. Für den Windows-Teil `cargo check --target x86_64-pc-windows-gnu` (braucht `mingw-w64`). Den echten Windows-Build gibt es nur in GitHub Actions.
- **Workflow:** Kein Schritt darf ohne Grund dazukommen. Jede Minute zählt doppelt (Windows).

## Prüfen (lokal, nicht in CI)

```bash
cd app && npm install
npm run lint                 # Lint der Oberfläche – muss 0 Befunde haben
npm run dev                  # Oberfläche im Browser
npx tauri dev                # als Windows-Programm (nur unter Windows)
cd src-tauri && cargo check  # Rust; in der Cloud: cargo check --target x86_64-pc-windows-gnu
cargo clippy                 # Rust-Lint, keine Warnungen
```

Die genauen Skripte stehen fest, sobald `app/` angelegt ist. Dann diesen Abschnitt anpassen.

Nicht in der Cloud prüfbar: Start von Roblox, Registry, Mutex, WebView2, Tray, echter Installer.

## Wie Claude in diesem Projekt antwortet

1. Kurz fassen: was geändert wurde, was der Nutzer selbst tun muss.
2. Am Ende jeder Änderung die neuen und geänderten Dateien nennen.
3. Bei Unklarheiten fragen, besonders bei allem, was die Regeln von Roblox oder das Cookie betrifft.
4. Ergebnisse ehrlich melden: was geprüft ist (lokal, Testbuild, echtes Windows) und was nicht.
5. Releases und Testbuilds nur auf Anweisung starten.

## Offene Punkte

- Frontend-Framework (Svelte oder SolidJS), Lizenz (MIT?), Umfang von v1.
- `app/` anlegen (Tauri 2, Identifier festlegen, Icons aus `brand/`).
- Repo öffentlich machen: dann kosten Actions nichts mehr. Vorher prüfen, dass keine Geheimnisse in der Historie liegen.
- Code-Signatur-Zertifikat (sonst warnen SmartScreen und Defender).
- `brand/build_logo.py` trägt noch Kommentare und Pixel-Tabellen der älteren Noppen-Fassung. Die PNGs sind die schlichte finale Fassung.
- Unbestätigt (Plan §13): Registry-Formate, Schlüssel in `GlobalBasicSettings_13.xml`, Wirkung von `DFIntDebugDynamicRenderKiloPixels`.
