# Brickstrap – Projekt-Kontext

> Arbeitsgrundlage für alle, die am Projekt mitarbeiten (Menschen und KI).
> **Vor jeder Änderung lesen:** [Regeln von Roblox](#regeln-von-roblox-nie-brechen) und [Vorsicht bei Änderungen](#vorsicht-bei-änderungen). Dort steht, was ein Konto sperren lässt oder still kaputtgeht.
> Recherche und Plan: [docs/research/roblox-launcher-rust.md](docs/research/roblox-launcher-rust.md). Produkt: [PRODUCT.md](PRODUCT.md). Logo und Farben: [DESIGN.md](DESIGN.md).
>
> Stand: 13.11.2026 (v0.0.7 released: Tauri 2 + Svelte 5 unter `app/`. Startseite mit Angepinnt, Zuletzt gespielt und Roblox-Reihen; Suche (auch Place-ID/Link); Spielseite mit Server-Picker (Sortierung, Volle ausblenden, „Kleinster Server“, „Bester Ping“, Beitreten per Deeplink); Bibliothek mit Anpinnen-Reihenfolge und Spielzeit; FastFlag-Editor (Allowlist, Presets, JSON teilen); Roblox-Einstellungen (GlobalBasicSettings); Discord-Status und Spielzeit aus der Logdatei; Leiste einklappbar (automatisch unter 1100 px); Infobereich, Einzelinstanz. Dazu: Suchvorschläge, Bewertung, Abzeichen/Game-Pässe/Events auf der Spielseite, Roblox-Update-Erkennung, Region des Servers, „Wieder beitreten“ und Einladung zum Server, Spielzeit nach Zeitraum. Geprüft nur im Browser-Mock und mit `cargo check` für Windows, noch nie auf echtem Windows gestartet.)

## Was ist Brickstrap?

Ein eigener, quelloffener **Launcher für Roblox unter Windows**, gebaut mit Rust und Tauri 2. Er startet den offiziellen Roblox-Client nur von außen (Deeplinks, später ein eigener Bootstrapper) und bietet Startseite, Spielesuche, Server-Picker, Editoren für FastFlags und Einstellungen sowie Discord-Status.

| | |
|---|---|
| Name | **Brickstrap**. Nie „Roblox“ oder „Blox“ im Namen, nichts, was an Logo oder Wortmarke von Roblox erinnert |
| Technik | Tauri 2: Rust-Backend (`reqwest` mit rustls, `serde`, `quick-xml`, `regex`, `discord-rich-presence`), Oberfläche Svelte 5 (Runes) + Vite 7 im WebView2. Schrift Archivo (variabel, selbst gehostet) |
| App-Kennung | `app.brickstrap.launcher` – **nie ändern**, sonst installiert Windows eine zweite App |
| Plattform | nur Windows (x64). Kein Android, kein macOS, kein Linux |
| Nutzer | Roblox-Spieler, die Bloxstrap/Fishstrap kennen und mehr Kontrolle wollen |
| Dienste | nur die Web-APIs von Roblox und GitHub. Kein eigener Server. Das Roblox-Cookie geht an keinen fremden Server |
| Sprache | alles Deutsch: UI-Texte (nur Deutsch, Entscheidung vom 13.11.2026), Kommentare, Commits. Du-Form, kurze Sätze, korrekte Umlaute |
| Repo | privat (`Type-1-Diabetes/Brickstrap-Launcher`). Veröffentlichung als Open Source (Lizenz vermutlich MIT) ist geplant |

**Stufen** (Plan §12.9): 1. MVP ohne Login (Suche, Startseite, Server-Picker, Start per Deeplink) · 2. FastFlag- und Einstellungs-Editor, Discord-Status · 3. Login, Freunde, Region · 4. Mehrere Instanzen, Overlay-Fenster · 5. Eigener Bootstrapper + Mods · 6. Chat, Konto-Wechsel.

## Regeln von Roblox (nie brechen)

- **Tabu:** Code in den laufenden Client einschleusen (DLL-Injection), Speicher des Clients lesen oder schreiben, die Client-Datei patchen, Overlays, die sich in den Roblox-Prozess hängen (ReShade, RTSS), FPS-Unlocker per Speicherzugriff, „Speicher-Bereinigung“ am laufenden Prozess, Macros und automatische Eingaben.
- **Erlaubt bzw. geduldet:** Client herunterladen und starten, Deeplinks, Web-APIs, `ClientAppSettings.json` (nur Flags der Allowlist), `GlobalBasicSettings_13.xml`, Content-Mods (Dateien ersetzen), Logdatei lesen, Windows-Einstellungen für `RobloxPlayerBeta.exe` (Registry), eigene Fenster außerhalb von Roblox.
- Im Zweifel nicht einbauen und den Projektinhaber fragen. Grundsatz aus PRODUCT.md: nichts, was ein Konto sperren lassen könnte.

## Arbeitsweise & Regeln

- **Grundsätze aus PRODUCT.md:** Tempo ist eine Funktion (alles soll sofort reagieren). Kontrolle ohne Unordnung. Sauber von Anfang an. Offen und nachvollziehbar.
- **Workflows kurz und schnell** (Wunsch des Projektinhabers): ein einziger Workflow `release.yml`, ein Linux-Job, der die Windows-exe per `cargo-xwin` cross-kompiliert (Linux-Minuten zählen einfach). Nur die exe, kein Installer. Keine Tests, kein Linting, keine Prüfschritte in GitHub Actions. Normale Pushes bauen nichts.
- **Keine Kosten bei GitHub** (Wunsch des Projektinhabers): siehe [Testbuilds und Kosten](#testbuilds-und-kosten). Keine neuen Workflows, keine Zeitpläne (`schedule`), keine Läufe bei Push oder Pull Request, kein Dependabot-Build.
- **Prüfen passiert lokal** (siehe [Prüfen](#prüfen-lokal-nicht-in-ci)).
- **Login ist Pflicht** (Entscheidung des Projektinhabers vom 13.11.2026): Ohne Anmeldung zeigt die App nur `pages/Anmelden.svelte`. Die Anmeldung läuft über die echte Login-Seite von roblox.com in einem privaten Tauri-Fenster (`auth.rs`, Fenster `login`, ohne Capabilities). Danach liest Rust das Cookie aus diesem Fenster (nie auf dem Hauptthread, sonst Verklemmung unter Windows). OAuth („Mit Roblox anmelden“) bräuchte eine von Roblox geprüfte App und kommt eventuell später. Nur `friends.rs` schickt das Cookie mit (Presence, mit CSRF-Token); alle anderen Abrufe laufen ohne.
- **Das Cookie `.ROBLOSECURITY` ist ein Generalschlüssel:** nur im Rust-Teil, verschlüsselt im Windows-Anmeldeinformations-Manager (`keyring`). Nie an die Oberfläche, nie in Logdatei, Einstellungen, Export, Commit oder an einen anderen Host als `*.roblox.com`.
- **Fremden Code prüfen:** Bloxstrap, Fishstrap, NexStrap (MIT) dürfen als Vorlage dienen, mit Hinweis auf die Herkunft. RoValra und Roblox Account Manager sind **GPL**: nur das Prinzip übernehmen, keinen Code kopieren.
- **Undokumentierte APIs** (Suche, Explore, Chat, `gamejoin`) kapseln, Fehler abfangen, nie die ganze App davon abhängig machen. Sie können jederzeit brechen.
- **UI-Arbeit** mit dem Skill `/impeccable` (Design-System in `DESIGN.md`, Produkt in `PRODUCT.md`, Richtung in `.impeccable/surfaces/app-src-app-svelte.md`). Bewegung nach `/ui-animation` (nur `transform`/`opacity`, 120–250 ms, keine Breiten-Animation). **Texte** nach dem Humanizer-Skill: kurz, informativ, aktiv, keine Floskeln, keine Gedankenstriche als Stilmittel.
- **Look: der bewährte Launcher-Standard** (Wunsch des Projektinhabers: „Standard wie Bloxstrap“). Messlatte: Bloxstrap/Fishstrap, Windows-11-Einstellungen, Discord, Steam/Epic. Flaches Schwarz, kein Mica. Normale Bedienelemente, keine Spielereien.
- **Seitenleiste** klappt unter 1100 px Fensterbreite automatisch ein (nur Symbole); der Knopf oben schaltet von Hand, bis das Fenster die Schwelle wieder kreuzt (Wunsch des Projektinhabers).
- **Farben:** Schwarz `#0B0B0D`, Grau `#8B9099`, Silber `#C3C7CD`, Weiß `#FFFFFF`, dazu Flächenstufen `--s1`…`--s4` (Tokens in `app/src/app.css`). Primärknopf = weiße Fläche, schwarze Schrift. Flach, ohne Verlauf. Nur dunkel. **Statusfarben nur mit Bedeutung** (bestätigt vom Projektinhaber): Grün `--live` für online/spielt, Gelb `--warnung` für experimentell/Warnung, Rot `--fehler` für Fehler. Keine weiteren Farben ohne Rückfrage.
- **Schnell starten** (Plan §15): Fenster erst nach dem ersten Zeichnen zeigen, Daten zuerst aus dem Cache, dann aktualisieren; schwere Arbeit im Rust-Backend (`tokio`); lange Listen virtualisieren; im Tray weiterlaufen statt beenden.
- **Keine Geheimnisse im Repo.** Signierzertifikat nur als GitHub-Secret. Keine Cookies, Konto-Namen oder IDs echter Konten als Testdaten.
- **Nicht raten.** Endpunkte, Registry-Formate und Schlüssel in `GlobalBasicSettings_13.xml` sind teils nur aus Erinnerung oder Foren belegt (im Plan markiert). Vor dem Einbau prüfen, bei Unklarheit nachfragen.

## Testbuilds und Kosten

Das Repo ist **privat**. GitHub Free: 2.000 Actions-Minuten im Monat und 500 MB Speicher für Artefakte. Der Build läuft auf Linux (zählt einfach, Windows würde doppelt zählen). Der alte Windows-Build mit NSIS brauchte kalt 15 Minuten (30 abgerechnete). Release-Dateien kosten nichts. Ohne hinterlegte Zahlungsart entstehen keine Kosten, Läufe stoppen dann nur, wenn das Kontingent leer ist. Sobald das Repo öffentlich ist, sind Actions komplett kostenlos.

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
5. Ergebnis prüfen: Release enthält `Brickstrap_<version>_x64.exe` und `.sha256`. Bei Fehlern das Log ansehen (`mcp__github__get_job_logs`).

**Nie ändern, sobald es Nutzer gibt:** `identifier` in `tauri.conf.json` (sonst installiert Windows eine zweite App neben der alten) und die Asset-Namen (`*-setup.exe`, `<datei>.sha256`), falls ein Update in der App darauf aufbaut.

Optional: Code-Signatur über die Secrets `WINDOWS_CERTIFICATE` (base64 einer `.pfx`) und `WINDOWS_CERTIFICATE_PASSWORD`. Ohne sie baut der Workflow unsigniert, SmartScreen und Defender warnen dann.

## Ordner – was liegt wo

```
Brickstrap-Launcher/
├─ CLAUDE.md                  Diese Datei (für Entwicklung)
├─ PRODUCT.md, DESIGN.md      Produkt und Design-System (impeccable)
├─ .github/workflows/
│  └─ release.yml             Windows-exe auf Linux bauen (cargo-xwin) und releasen, nur auf Anstoß
├─ brand/                     Logo-Kit: logo/ (SVG, PNG 16–2048 px, brickstrap.ico), build_logo.py
├─ docs/research/
│  └─ roblox-launcher-rust.md Recherche, Regeln, APIs, FastFlag-Allowlist, Roadmap
├─ .impeccable/surfaces/      Richtung der Oberfläche (Direction contract)
└─ app/                       Tauri-App (Aufbau unten)
```

```
app/
├─ index.html, vite.config.js (Port 1420, __APP_VERSION__ aus package.json), jsconfig.json, .oxlintrc.json
├─ src/
│  ├─ main.js, App.svelte      Rahmen: Leiste, Suchkopf (Strg+K, Alt+←, Maus-Zurück), Seiten, Dock, Hinweise
│  ├─ app.css                  Tokens (:root), Schrift, Knöpfe, Felder, Schalter, Chips, Bildlaufleisten
│  ├─ fonts/                   Archivo (OFL), latin + latin-ext
│  ├─ lib/
│  │  ├─ bridge.js             einziger Zugang zu Rust (`call`, `on`); im Browser lädt er mock.js
│  │  ├─ mock.js               erfundene Demo-Daten für alle Befehle (Browser-Vorschau)
│  │  ├─ app.svelte.js         Zustand: Route/Verlauf, Config, Angepinnt, Aktivität, Hinweise, Bilder-Bündelung,
│  │  │                        Server-Vorladen (30 s), launch(), Leiste einklappen
│  │  ├─ flags.svelte.js       FastFlag-Zustand für Editor und Dock (Preset, setzen, verzögert speichern)
│  │  ├─ suche.svelte.js       Suchbegriff (Kopf ↔ Suchseite)
│  │  ├─ errors.js, format.js  Fehlercodes → Text; Zahlen, Dauer, „vor X“
│  ├─ ui/                      Leiste, Dock, Toasts, Icon (+icons.js), Bild, Spielkarte, Reihe, Hinweis, Serverliste,
│  │                           Spielextras (Events, Game-Pässe, Abzeichen), Zeitstatistik (7/30 Tage, Gesamt),
│  │                           Freunde (rechte Spalte: online, spielt was, Beitreten)
│  └─ pages/                   Start, Suche, Spiel, Bibliothek, Flags, Roblox, Einstellungen
└─ src-tauri/
   ├─ tauri.conf.json          Fenster (unsichtbar bis gezeichnet), CSP, NSIS (currentUser, Deutsch)
   ├─ capabilities/default.json
   ├─ resources/allowlist.json FastFlag-Allowlist (eingebettet; `<AppData>/allowlist.json` überschreibt sie)
   ├─ resources/regions.json   IP-Bereiche → Ort (eingebettet; `<AppData>/regions.json` überschreibt sie)
   └─ src/
      ├─ lib.rs                Plugins, Infobereich, Schließen = verstecken, Befehle registrieren
      ├─ state.rs              AppState (Datenordner, Config, HTTP-Client, Session-ID), JSON lesen/schreiben, DISCORD_APP_ID
      ├─ paths.rs              Roblox-Pfade, Versionsordner, läuft Roblox? (Toolhelp)
      ├─ roblox_api.rs         Startseite (Cache), Suche + Vorschläge, Details (50 je Abruf), Bewertung, Abzeichen/Pässe/Events,
      │                        Bilder (1 Tag Cache), Serverliste, game_brief, Rohantworten (`roh/`)
      ├─ friends.rs            Online-Freunde: Freundesliste + Presence (mit Cookie, nur *.roblox.com), Avatare, 30 s gemerkt
      ├─ region.rs             Region eines Servers aus der IP (Tabelle, kein fremder Dienst)
      ├─ launch.rs             Start per Deeplink oder `--app`, Roblox-Update prüfen, Einladungslink, Config, Anpinnen
      ├─ fastflags.rs          Allowlist, Presets, ClientAppSettings.json vor jedem Start (nur bei Änderung)
      ├─ basicsettings.rs      GlobalBasicSettings_13.xml lesen/schreiben (Rest der Datei bleibt byte-gleich, Sicherung)
      ├─ activity.rs           Logdatei mitlesen (beim Start auch rückwärts) → Ereignis `activity`, Spielzeit in history.json,
      │                        neue Versionsordner erkennen → FastFlags nachschreiben, Ereignis `roblox-version`
      └─ discord.rs            Rich Presence in eigenem Thread, BloxstrapRPC
```

Daten der App liegen in `%APPDATA%\app.brickstrap.launcher\`: `config.json`, `fastflags.json`, `history.json`, `cache/` (Startseite, Bilder-URLs, Kurzinfos).

Der Workflow erwartet die App unter `app/` mit `app/package-lock.json` und `app/src-tauri/tauri.conf.json`. Wer die Struktur ändert, passt `release.yml` mit an.

## Abhängigkeiten – was hängt woran?

| Was | Stellen |
|---|---|
| Version | `app/package.json` ↔ `app/src-tauri/Cargo.toml` ↔ Workflow-Input `version` |
| Pfade im Workflow | `release.yml` (`app/`, `app/src-tauri`, `bundle/nsis/*-setup.exe`) ↔ Ordnerstruktur ↔ `bundle.targets` in `tauri.conf.json` |
| Farben, Logo | `brand/` ↔ `DESIGN.md` ↔ CSS-Tokens der App (`:root`) ↔ `src-tauri/icons/` (aus `brand/logo/png` und `brickstrap.ico`) |
| FastFlag-Allowlist | eigene Datei (nicht im Code fest eingebaut) ↔ Editor ↔ Plan §5. Roblox ändert die Liste ohne Vorwarnung |
| Rust-Befehle | `#[tauri::command]` ↔ `generate_handler!` in `lib.rs` ↔ `call("…")` in der Oberfläche ↔ **Handler in `mock.js`** (Namen, Argumente in camelCase und JSON-Form gleich halten) |
| JSON-Formen | Rust-Structs mit `rename_all = "camelCase"` ↔ `mock.js` ↔ Seiten. Neues Feld = in allen dreien |
| Fehlercodes | `ApiError.code` bzw. `"CODE|Text"` in Rust ↔ `TEXTE` in `errors.js` (`NETZ`, `TIMEOUT`, `RATE_LIMIT`, `HTTP`, `FORMAT`, `NICHT_INSTALLIERT`, `START`, `LAEUFT`, `SCHREIBGESCHUETZT`, `DATEI`, `UNBEKANNT`) |
| Ereignis `activity` | `activity.rs` (`Activity`) ↔ `app.svelte.js` (`init`) ↔ Dock ↔ `discord.rs` |
| Ereignis `roblox-version` | `activity.rs` (`watch_versions`) ↔ `app.svelte.js` (`init`: Status und Update neu lesen) |
| Discord-ID | `DISCORD_APP_ID` in `state.rs` (eingebaut) ↔ `discordAppId` in den Einstellungen (eigene, optional) ↔ `Config::discord_id()` |
| Externe Hosts | Alle Web-Abrufe laufen in Rust (`reqwest`), nicht im WebView. Bilder lädt das WebView direkt: CSP `img-src https://*.rbxcdn.com`. Neue Bildquelle = CSP ergänzen |

## Vorsicht bei Änderungen

- **`ClientAppSettings.json`** liegt im Versionsordner (`%localappdata%\Roblox\Versions\version-…\ClientSettings\`) und wird bei **jedem Roblox-Update geleert**. Vor jedem Start neu schreiben, nur wenn sich etwas geändert hat (Prüfsumme). Werte als Strings. Nicht erlaubte Flags werden still ignoriert: im Editor als „wirkungslos“ markieren statt zu verstecken.
- **`GlobalBasicSettings_13.xml`** nur schreiben, wenn Roblox **komplett geschlossen** ist, sonst überschreibt Roblox die Änderung beim Beenden. Keinen festen Schlüsselkatalog einbauen, die Werte der Datei des Nutzers lesen und anzeigen.
- **Registry-Einträge pro exe-Pfad** (GPU-Vorgabe, Vollbild-Optimierungen) hängen am Versionsordner und müssen nach jedem Roblox-Update neu gesetzt werden. Formate vorher an einem echten Windows prüfen (Plan §13.3).
- **Protokoll-Handler** (`roblox-player`, `roblox` unter `HKCU\Software\Classes\`) erst mit eigenem Bootstrapper (Stufe 5) umbiegen. Beim Deinstallieren auf den offiziellen Launcher zurückstellen, sonst startet Roblox nicht mehr.
- **Mehrere Instanzen:** Mutex `ROBLOX_singletonEvent` bzw. `ROBLOX_singletonMutex` vor Roblox anlegen und halten. Nicht offiziell unterstützt, Teleports können scheitern. Nur als Option, standardmäßig aus.
- **Serverliste** (`games.roblox.com/v1/games/{placeId}/servers/Public`): Seiten über `cursor`, nach etwa 700 Servern ist Schluss. Abfragen drosseln und zwischenspeichern (Rate-Limit).
- **Konto-Wechsel und mehrere Cookies** sind ein Ziel für Schadsoftware. Erst nach dem MVP und nur mit sicherem Speicher.
- **HTTP nur im Rust-Teil.** Die Oberfläche spricht nie selbst mit Roblox (CSP `connect-src` lässt nur IPC zu).
- **Rate-Limits:** `games?universeIds=` höchstens 50 IDs und ein verstecktes Limit pro IP (~430 IDs/min), Suche ~4/s, Explore 50–120/min. Bilder gebündelt (`wantImage` sammelt 30 ms), Details und Bilder gecacht, Server-Vorladen erst nach 180 ms Hover.
- **Feldnamen der undokumentierten APIs** (omni-search, explore-api, servers) stammen aus Quelltext anderer Launcher, nicht aus eigenem Abruf (Roblox ist aus der Cloud gesperrt). `roblox_api.rs` liest darum mehrere mögliche Namen. Erster echter Test unter Windows: Rohantworten ansehen und anpassen.
- **Rohantworten ansehen:** Gibt es im Datenordner den Ordner `roh`, legt `get_json` jede Antwort dort als JSON ab. So lassen sich die undokumentierten Endpunkte am PC prüfen. Danach den Ordner wieder löschen.
- **Unbestätigte Endpunkte (Stand 13.11.2026):** Vorschläge (`games-autocomplete/v1/get-suggestion`), Game-Pässe (`apis.roblox.com/game-passes/v1/universes/{id}/game-passes`, Rückfall auf den alten Pfad), Events (`virtual-events/v1/universes/{id}/virtual-events`, verlangt eventuell eine Anmeldung). Alle scheitern still: Die Oberfläche zeigt den Teil dann nicht. `game_votes`, Abzeichen und `client-version` sind dokumentiert.
- **Roblox-Update:** `roblox_update` vergleicht die Versionsordner mit `clientVersionUpload` von `clientsettingscdn.roblox.com` (10 Minuten gemerkt). Wer einen anderen Kanal als LIVE nutzt, bekommt dauerhaft „Update steht an“.
- **Regionstabelle** (`regions.json`) stammt aus einer Community-Liste (Quelle in der Datei) und enthält nur auf Roblox registrierte Bereiche. Server bei AWS und unbekannte Adressen zeigen nur die IP. Orte sind Schätzungen.
- **Einladungslink mit Server** (`https://www.roblox.com/games/start?placeId=…&gameInstanceId=…`) und „Wieder beitreten“ (JobId der letzten Sitzung, 3 Stunden lang angeboten): Ob Roblox auf genau diesen Server führt, ist unbestätigt.
- **Discord-Knöpfe** brauchen http(s): „Server beitreten“ zeigt auf `https://www.roblox.com/games/start?placeId=…&gameInstanceId=…` (Format unbestätigt).
- **Schalter (`.schalter`):** Die sichtbare Fläche hat `pointer-events: none`, sonst kommen Mausklicks nicht am Kontrollkästchen an. So lassen.
- **Bool-Flags sind dreiwertig:** kein Wert = „Standard“ (Roblox entscheidet), An = `True`, Aus = `False`. Manche Flags sind ab Werk an.
- **Mock lädt die echte `allowlist.json`** und hat dieselben Presets wie `fastflags.rs`. Presets in beiden ändern. Zustände ansehen über die Adresse: `?leer`, `?offline`, `?ohne-roblox`, `?update`, `?abgemeldet` (kombinierbar).
- **Microsoft-Store-Roblox:** „installiert“ heißt Versionsordner **oder** registriertes `roblox://`. Logs und `GlobalBasicSettings_13.xml` werden auch unter `Packages\ROBLOXCORPORATION.ROBLOX_55nm5eh3cm0pr\LocalState` gesucht. FastFlags lassen sich dort nicht schreiben.
- **FastFlags und Roblox-Updates:** Aktualisiert sich Roblox beim Start, entsteht ein neuer Versionsordner ohne `ClientAppSettings.json`. `activity.rs` prüft jede Sekunde die Versionsordner und schreibt die Flags sofort in einen neuen. Ob das schneller ist als der Start des Clients, ist am echten PC zu prüfen; sonst greifen die Flags erst beim nächsten Start.
- **Fenster-Rückfall:** `lib.rs` zeigt das Fenster nach 4 s selbst, falls die Oberfläche beim Laden scheitert.
- **Neuer Befehl:** Rust-Funktion → `generate_handler!` → Handler in `mock.js` → Aufruf über `call()`. Ohne Mock bricht die Browser-Vorschau.
- **Overlay-Fenster** nur als eigenes transparentes Tauri-Fenster (`always_on_top`, Klick-Durchlass). Es zeigt nur, was der Launcher von außen weiß.
- **Rust lokal in Claude-Cloud-Sessions:** Tauri braucht unter Linux GTK/WebKit. Für den Windows-Teil `cargo check --target x86_64-pc-windows-gnu` (braucht `mingw-w64`). Den echten Windows-Build gibt es nur in GitHub Actions.
- **Workflow:** Kein Schritt darf ohne Grund dazukommen. Gebaut wird `--no-bundle` mit `cargo-xwin` (MSVC-Ziel). Die exe braucht WebView2 auf dem PC. Lokal nicht testbar, weil die Cloud-Session die Microsoft-SDK-Downloads sperrt.

## Prüfen (lokal, nicht in CI)

```bash
cd app && npm install
npm run lint                 # oxlint – muss 0 Befunde haben
npm run check                # svelte-check – 0 Fehler, 0 Warnungen
npm run build                # Vite-Build nach dist/ (braucht cargo check, weil Tauri dist/ einbettet)
npm run dev                  # Browser-Vorschau mit Demo-Daten: http://localhost:1420
npx tauri dev                # als Windows-Programm (nur unter Windows)
cd src-tauri && cargo clippy --target x86_64-pc-windows-gnu   # Rust für Windows prüfen, keine Warnungen
```

In der Cloud: `rustup target add x86_64-pc-windows-gnu` und `apt-get install mingw-w64` einmal pro Sitzung. Oberfläche nach Änderungen bei 1440, 1180 und 940 px Breite ansehen (Playwright), `scrollWidth` darf nicht größer als die Fensterbreite sein. Die reine Logik in `basicsettings.rs` und `activity.rs` hat `#[cfg(test)]`-Tests; unter Linux laufen sie nur in einem Wegwerf-Crate (Tauri braucht GTK).

Nicht in der Cloud prüfbar: Start von Roblox, Registry, Mutex, WebView2, Tray, echter Installer.

## Wie Claude in diesem Projekt antwortet

1. Kurz fassen: was geändert wurde, was der Nutzer selbst tun muss.
2. Am Ende jeder Änderung die neuen und geänderten Dateien nennen.
3. Bei Unklarheiten fragen, besonders bei allem, was die Regeln von Roblox oder das Cookie betrifft.
4. Ergebnisse ehrlich melden: was geprüft ist (lokal, Testbuild, echtes Windows) und was nicht.
5. Releases und Testbuilds nur auf Anweisung starten.

## Offene Punkte

- **Noch nie auf Windows gelaufen.** Erster Testbuild (Run workflow ohne Version) und Test am PC: Start, Infobereich, Deeplink-Beitritt, FastFlags im Versionsordner, XML schreiben, Logdatei, Discord.
- **Discord-Anwendung** „Brickstrap“ ist eingebaut (`DISCORD_APP_ID` = 1570919112279072948, öffentliche ID). Eigene ID in den Einstellungen bleibt optional.
- Echte Antworten von omni-search, explore-api und servers/Public prüfen (Feldnamen, `sortDisplayName`, `contentType`).
- Am PC prüfen (mit Ordner `roh`): Vorschläge, Game-Pässe, Events, Einladungslink mit Server, Flags im neuen Versionsordner nach einem Update, Regionen.
- Lizenz (MIT?).
- Region der Server (braucht Login) und Mod-Manager folgen in späteren Stufen.
- Repo öffentlich machen: dann kosten Actions nichts mehr. Vorher prüfen, dass keine Geheimnisse in der Historie liegen.
- Code-Signatur-Zertifikat (sonst warnen SmartScreen und Defender).
- `brand/build_logo.py` trägt noch Kommentare und Pixel-Tabellen der älteren Noppen-Fassung. Die PNGs sind die schlichte finale Fassung.
- Unbestätigt (Plan §13): Registry-Formate, Schlüssel in `GlobalBasicSettings_13.xml` (die Grafikstufe heißt laut Fishstrap-Vorlage `SavedQualityLevel`), Wirkung von `DFIntDebugDynamicRenderKiloPixels`, Werte der Presets „Hohe Qualität“ und „Max FPS“ (Renderauflösung 720).
