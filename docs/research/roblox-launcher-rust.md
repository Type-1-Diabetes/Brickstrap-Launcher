# Eigener Roblox-Launcher in Rust: Recherche & Plan

Stand: 13.11.2026

## Ziel

Ein eigener, quelloffener Roblox-Launcher für Windows, gebaut mit **Rust**. Er startet den offiziellen Roblox-Client nur von außen (keine Injection, keine Veränderung der Client-Datei). So bleibt er innerhalb der Regeln von Roblox.

## Name & Logo

**Name: Brickstrap**
- „Brick“ für den Baustein, das Urbild von Roblox, und „-strap“ als bekannte Endung der Bootstrapper (Bloxstrap, Fishstrap).
- Enthält weder „Roblox“ noch „Blox“.
- Bei der Suche auf GitHub am 13.11.2026 gab es kein Roblox-Projekt mit diesem Namen.

**Logo: Code-Zeichen `</>`**
- **Zeichen:** ein schlichtes Coding-/Debug-Symbol `</>`: silberne Klammern und ein weißer Schrägstrich auf einer schwarzen, abgerundeten Kachel. Alle Enden sind rund.
- **Verlauf:** Zuerst gab es den „Ping-Stein“ (Bernstein), dann eine Umstellung auf Schwarz/Grau/Silber/Weiß, dann ein `</>` mit Noppen-Schrägstrich und zuletzt das schlichte `</>`. Das ist die finale Fassung vom 13.11.2026.
- **Farben:** Schwarz `#0B0B0D` (Kachel), Silber `#C3C7CD` (Klammern) und Weiß `#FFFFFF` (Schrägstrich, Schriftzug auf dunklem Grund). Grau `#8B9099` ist eine Reservefarbe für die Oberfläche. Alles ist flach, ohne Verlauf und Schatten.
- **Geometrie:**
  - Grundlage ist ein 64er-Raster mit Linienstärke 6 und runden Enden.
  - Die Klammern laufen von (22,21) über (10,32) nach (22,43), rechts gespiegelt.
  - Der Schrägstrich geht von (28.5,45) nach (35.5,19).
  - Der Kachel-Radius beträgt 14/64.
  - Es ist reiner Vektor und bleibt in jeder Größe scharf (PNG bis 2048 px).
- **Schriftzug:** „brickstrap“ in Kleinbuchstaben, Archivo mit Gewicht 850 und Breite 118, Laufweite −0,03 em, in Formen umgewandelt. In der Kombination reicht die Oberlänge bis zur Oberkante des Zeichens und die Grundlinie liegt auf seiner Unterkante.
- **Einsatz:**
  - Auf hellem Grund: die schwarze Kachel mit dem Schriftzug in Schwarz, oder alles einfarbig schwarz.
  - Auf dunklem Grund: das Zeichen ohne Kachel mit dem Schriftzug in Weiß.
- **Dateien:**
  - `brickstrap-logo.zip` enthält SVG, PNG 16–1024 px, Windows-`.ico`, den Generator `build_logo.py` und `DESIGN.md` mit Farben, Maßen und Regeln für die spätere Oberfläche.
  - Arbeitsordner: `brickstrap-logo/`.
- **Tabu:** nichts, das an das Roblox-Logo oder die Roblox-Wortmarke erinnert.

---

## 1. Rahmen & Regeln

- Roblox bietet **kein offizielles SDK** für eigene Launcher.
- Roblox geht automatisch gegen veränderte Clients vor. Bloxstrap ist nicht betroffen, **weil es die Client-Datei nicht verändert**. Offiziell unterstützt wird solche Drittsoftware trotzdem nicht. ([DevForum](https://devforum.roblox.com/t/an-update-on-automated-action-against-modified-clients/3640609/1))
- **Tabu:** Code in den laufenden Client einschleusen (DLL-Injection), Speicher auslesen oder ändern, die Client-Datei patchen.
- **Erlaubt bzw. geduldet:** Client herunterladen und starten, Deeplinks, Web-APIs, `ClientAppSettings.json` (nur Flags auf der Allowlist), `GlobalBasicSettings_13.xml`, Content-Mods (Dateien ersetzen), Logdatei lesen.
- Offizieller Weg nur für Spieleentwickler: von Overwolf gebaute Launcher für Studios und angekündigte Standalone-Apps („Roblox Everywhere“). Für einen allgemeinen Launcher ist beides nicht relevant.

---

## 2. Drei Ansätze

| Ansatz | Aufwand | Beschreibung |
|---|---|---|
| **A: Nur Oberfläche** | gering | Eigene Oberfläche für Spieleliste, Server-Picker und Freunde. Startet Roblox per Deeplink. Das offizielle Roblox bleibt installiert und aktualisiert sich selbst. |
| **B: Eigener Bootstrapper** | hoch | Lädt und installiert Roblox selbst, verwaltet Versionen, Mods und FastFlags, registriert das `roblox-player:`-Protokoll. Funktioniert wie Bloxstrap. |
| **C: Bloxstrap forken** | mittel | C#/.NET statt Rust, aber alles ist schon fertig. |

**Empfehlung:** Mit **A** in Rust/Tauri anfangen und später schrittweise zu **B** ausbauen.

---

## 3. Tech-Stack (Rust)

| Zweck | Crate / Tool |
|---|---|
| Oberfläche | **Tauri** (Rust-Backend und Web-Frontend, z. B. Svelte oder React). Alternativ `egui`/`iced` für reines Rust. |
| HTTP | `reqwest` + `tokio` |
| Roblox-Web-API | **`roboat`**: Rust-Schnittstelle zur Roblox-API (Users, Presence, Chat, Thumbnails, Games …, MIT) |
| JSON / XML | `serde`, `serde_json`, `quick-xml` |
| ZIP entpacken (Bootstrapper) | `zip` |
| Registry (Protokoll-Handler) | `winreg` |
| Mutex (mehrere Instanzen) | `windows` (Win32 `CreateMutexW`) |
| Logdatei beobachten | `notify` |
| Discord Rich Presence | `discord-rich-presence` |
| Login-Cookie sicher speichern | `keyring` (Windows Credential Manager) |

---

## 4. Funktionen & wie man sie umsetzt

### 4.1 Roblox starten (Deeplinks)
- Spiel starten: `roblox://experiences/start?placeId=<ID>`
- **Bestimmten Server betreten:** `roblox://experiences/start?placeId=<ID>&gameInstanceId=<JobID>`
- Weitere Parameter: `accessCode`, `linkCode` (private Server), `launchData`, `userId` (einem Freund folgen)
- Altes Format: `roblox-player:1+launchmode:play+...`. Der Bootstrapper gibt die Protokoll-URL als erstes Argument an `RobloxPlayerBeta.exe` weiter.

### 4.2 Server-Picker
- `GET https://games.roblox.com/v1/games/{placeId}/servers/Public?sortOrder=Asc&limit=100&excludeFullGames=true`
- Liefert Server-ID (JobId), Spielerzahl und Ping. Seiten mit `cursor` durchblättern.
- Bekannte Grenze: Nach etwa **700 Servern** bricht die Seitenliste ab. ([DevForum](https://devforum.roblox.com/t/unable-to-fetch-more-than-700-servers-servers-using-the-games-v1-api-gamesrobloxcomv1/4294098))
- Die Liste lässt sich ohne Login abrufen, zum Beitreten braucht man das Cookie.
- Region: Bloxstrap zeigt den Serverstandort per IP-Abfrage an (über ipinfo.io).

### 4.3 Login, Freunde, Chat, Empfehlungen
- Alles über die Web-APIs mit dem Cookie `.ROBLOSECURITY` (z. B. `users`, `friends`, `presence`, `chat`, `thumbnails`, `games`).
- Übersetzte Spielnamen kommen aus der Games-API, gesteuert über das Locale.
- **Sicherheit:** Das Cookie gibt vollen Zugriff auf das Konto. Es gehört verschlüsselt gespeichert (keyring) und darf an keinen fremden Server gehen.
- Zum Einloggen ein WebView-Fenster mit der echten Roblox-Login-Seite öffnen und das Cookie daraus übernehmen (geht mit Tauri).

### 4.4 Mehrere Instanzen
- Roblox verhindert eine zweite Instanz mit dem Mutex **`ROBLOX_singletonEvent`** (neuere Clients: `ROBLOX_singletonMutex`).
- Lösung: Den Mutex **vor** Roblox selbst anlegen und halten. ([MultiBloxy](https://github.com/zgoly/multibloxy))
- Bekanntes Problem: Teleports können fehlschlagen, wenn mehrere Clients laufen.

### 4.5 Aktivitäten-Tracking & Discord Rich Presence
- Die Roblox-Logdatei unter `%localappdata%\Roblox\logs\` live mitlesen. Daraus ergeben sich Spiel, Server und Join/Leave.
- **BloxstrapRPC:** Spiele können per `print("[BloxstrapRPC] {json}")` Daten an den Launcher schicken, etwa einen eigenen Discord-Status. Der Launcher erkennt das am Präfix in der Logdatei. Es lohnt sich, das kompatibel nachzubauen. ([Wiki](https://github.com/pizzaboxer/bloxstrap/wiki/Integrating-Bloxstrap-functionality-into-your-game))

### 4.6 Mods (Content-Dateien ersetzen)
- Ein Mods-Ordner liegt wie eine Ebene über dem Client-Ordner. Bei jedem Start wird er in `Versions\version-xxx\` kopiert und **nach jedem Update erneut angewendet**.
- Typische Ziele:
  - Mauszeiger: `content\textures\KeyboardMouse\Cursors\`
  - Sounds (z. B. der alte „oof“-Sound): `content\sounds\ouch.ogg`
  - Schriften: `content\fonts\`
  - Außerdem Skyboxen, Emojis, Texturen
- Funktioniert nur sauber mit eigenem Bootstrapper (Ansatz B), weil Roblox sonst bei jedem Update überschreibt.

---

## 5. `ClientAppSettings.json`: alle aktuell erlaubten FastFlags

**Pfad:** `<Roblox-Versionsordner>\ClientSettings\ClientAppSettings.json`
(z. B. `%localappdata%\Roblox\Versions\version-xxxxxxxx\ClientSettings\`). Der Ordner wird bei **jedem Update geleert**, der Launcher muss die Datei also vor jedem Start neu schreiben.

**Format:** Die Werte stehen meist als Strings, z. B. `{"FFlagDebugSkyGray": "True", "DFIntTextureQualityOverride": "3"}`

**Allowlist (seit 29.09.2025):** Nur diese Flags wirken. Alle anderen werden **still ignoriert, eine Strafe gibt es nicht**. Für Roblox Studio gilt die Allowlist nicht. Roblox ändert die Liste ohne Vorwarnung. ([DevForum-Post](https://devforum.roblox.com/t/allowlist-for-local-client-configuration-via-fast-flags/3966569))

### Geometrie
| Flag | Typ | Wirkung |
|---|---|---|
| `DFIntCSGLevelOfDetailSwitchingDistance` | Int | Entfernung, ab der Mesh/CSG-Objekte auf eine niedrigere Detailstufe wechseln |
| `DFIntCSGLevelOfDetailSwitchingDistanceL12` | Int | Wechsel von Detailstufe 1 zu 2 |
| `DFIntCSGLevelOfDetailSwitchingDistanceL23` | Int | Wechsel von Detailstufe 2 zu 3 |
| `DFIntCSGLevelOfDetailSwitchingDistanceL34` | Int | Wechsel von Detailstufe 3 zu 4 |

Niedrige Werte bringen mehr FPS und weniger Details. Alle auf `0` gesetzt heißt immer die niedrigste Detailstufe.

### Rendering
| Flag | Typ | Wirkung |
|---|---|---|
| `FFlagDebugGraphicsPreferD3D11` | Bool | Direct3D 11 erzwingen |
| `FFlagDebugGraphicsPreferVulkan` | Bool | Vulkan erzwingen |
| `FFlagDebugGraphicsPreferOpenGL` | Bool | OpenGL erzwingen |
| `DFFlagTextureQualityOverrideEnabled` | Bool | Überschreiben der Texturqualität aktivieren |
| `DFIntTextureQualityOverride` | Int (0–3) | Texturqualität (0 = niedrig, 3 = hoch) |
| `FIntDebugForceMSAASamples` | Int (0/1/2/4/8) | Kantenglättung (MSAA) erzwingen |
| `DFIntDebugFRMQualityLevelOverride` | Int (1–21) | Grafikqualitätsstufe erzwingen, feiner als der Regler im Spiel |
| `FIntFRMMaxGrassDistance` | Int | Maximale Grasweite (`0` = kein Gras) |
| `FIntFRMMinGrassDistance` | Int | Minimale Grasweite |
| `DFFlagDebugPauseVoxelizer` | Bool | Pausiert die Voxel-Lichtberechnung (statisches Licht und Schatten, mehr FPS) |
| `DFFlagDisableDPIScale` | Bool | DPI-Skalierung aus (schärfere, kleinere Oberfläche auf HiDPI-Monitoren) |
| `FFlagHandleAltEnterFullscreenManually` | Bool | Vollbildverhalten bei Alt+Enter (`False` = exklusives Vollbild) |
| `FFlagDebugSkyGray` | Bool | Grauer Himmel statt Skybox |

### Oberfläche
| Flag | Typ | Wirkung |
|---|---|---|
| `FIntGrassMovementReducedMotionFactor` | Int | Stärke der Grasbewegung (weniger Bewegung) |

### Kandidaten ohne bestätigte Freischaltung
- `DFIntDebugDynamicRenderKiloPixels` (Renderauflösung für schwache Geräte): **im Januar 2026 vom Rendering-Team genehmigt**, Nutzer berichteten im Februar 2026 aber, dass die Flag noch keine Wirkung hat. ([DevForum](https://devforum.roblox.com/t/fflag-allowlist-request-towards-users-with-low-end-devices/4275059))
- **Nicht erlaubt** (wird ignoriert): `DFIntTaskSchedulerTargetFps`, `FFlagTaskSchedulerLimitTargetFpsTo2402` (alte FPS-Unlocker), `DFIntConnectionMTUSize`, Telemetrie-Flags, `FLogNetwork` u. a.

**Für den Launcher:** Die Allowlist nicht fest einbauen, sondern als Datei pflegen, die sich aktualisieren lässt. Im Editor nur erlaubte Flags anbieten und alle anderen als „wirkungslos“ markieren.

---

## 6. `GlobalBasicSettings_13.xml`: die Nutzereinstellungen

Das ist die zweite und für FPS wichtigere Stellschraube: In dieser Datei speichert Roblox die **Einstellungen aus dem Spielmenü**.

- **Pfad (Windows):** `%localappdata%\Roblox\GlobalBasicSettings_13.xml` (Microsoft Store: `...\Packages\ROBLOXCORPORATION.ROBLOX_55nm5eh3cm0pr\LocalState\`)
- **Wichtig:** Roblox muss **vollständig geschlossen** sein. Sonst überschreibt Roblox die Änderungen beim Beenden.
- **FPS:** `<int name="FramerateCap">0</int>`, wobei `0` = ohne Limit. Im Menü bietet Roblox offiziell bis **240 FPS** an, mehr unterstützt es nicht. Über die XML lässt sich das Limit nach Nutzerberichten trotzdem entfernen. ([DevForum](https://devforum.roblox.com/t/roblox-caps-fps-at-240-regardless-of-globalbasicsettings13xml-framerate-cap/4451048))
- Weitere Werte dort: Grafikqualität, Mausempfindlichkeit (auch Werte, die im Menü nicht einstellbar sind), Lautstärke, Oberfläche u. a.
- Vorbild: GokuTrap hat einen eigenen Bereich „Global Basic Settings“ für Grafik, Framerate, Oberfläche und Empfindlichkeit.

**Für den Launcher:** Vor dem Start die XML mit `quick-xml` einlesen, die gewünschten Werte setzen und zurückschreiben. Läuft Roblox gerade, nichts schreiben.

---

## 7. Eigener Bootstrapper (Ansatz B): Ablauf

1. **Version abfragen:** `GET https://clientsettings.roblox.com/v2/client-version/WindowsPlayer` (optional mit `/channel/{name}`)
2. **Paketliste laden:** `https://setup.rbxcdn.com/{version}-rbxPkgManifest.txt` mit Dateiname, MD5, gepackter und entpackter Größe
3. **Pakete laden:** `https://setup.rbxcdn.com/{version}-{Datei}.zip`, MD5 prüfen, in den richtigen Unterordner entpacken (`content\`, `PlatformContent\`, `ExtraContent\` …)
4. `AppSettings.xml` anlegen
5. `ClientSettings\ClientAppSettings.json` schreiben und Mods darüberkopieren
6. Protokolle `roblox-player` und `roblox` unter `HKCU\Software\Classes\` auf den eigenen Launcher umbiegen
7. Start: `RobloxPlayerBeta.exe <Protokoll-URL>` (ohne URL: `--app`)

Ersatz-CDNs: `setup-ak.rbxcdn.com`, `roblox-setup.cachefly.net`
Die beste Doku dazu: [A deep dive on how the Roblox bootstrapper works](https://github.com/pizzaboxer/bloxstrap/wiki/A-deep-dive-on-how-the-Roblox-bootstrapper-works)

---

## 8. Vorbilder

### Rust
| Projekt | Was man daraus lernen kann |
|---|---|
| [rsblox](https://github.com/idkncc/rsblox) | **Rust + Tauri + Svelte**, minimalistischer Launcher (Login per Cookie). Die nächste Vorlage für Ansatz A. |
| [ApplejuiceCLI](https://github.com/WaviestBalloon/ApplejuiceCLI) | Roblox-Bootstrapper in Rust (Linux/Proton). Zeigt, wie Download und Installation in Rust gehen. |
| [roboat](https://github.com/fekie/roboat) | Rust-Bibliothek für die Roblox-Web-API |

### C# / andere Sprachen (Funktionen und Logik)
| Projekt | Besonderheit |
|---|---|
| [Bloxstrap](https://github.com/bloxstraplabs/bloxstrap) | Das Original (C#/WPF, MIT). Wiki mit der Bootstrapper-Doku. |
| [Fishstrap](https://github.com/fishstrap/fishstrap) | Fork von Bloxstrap: Multi-Instanz, Spieler- und Chat-Logs |
| Voidstrap, Nyxstrap | Forks von Bloxstrap: Server-Infos (mit RoValra), Multi-Instanz |
| [NexStrap](https://github.com/k153636/nexstrap) | C#/Avalonia, MIT: Spielstatistik, Freunde-Tracking, FastFlags ohne Neustart neu laden |
| [GokuTrap](https://github.com/gokuthug1/GokuTrap/wiki) | Sehr gutes Wiki: GlobalBasicSettings-Editor, Region Radar, Account-Profile, Crosshair-Overlay |
| [MultiBloxy](https://github.com/zgoly/multibloxy) | Multi-Instanz über den Mutex |
| [Vinegar](https://github.com/vinegarhq/vinegar) | Go, Linux: sauberer, minimaler Bootstrapper |
| [Roblox Client Tracker](https://github.com/MaximumADHD/Roblox-Client-Tracker) | Verfolgt Versionen, Manifeste und FastFlags. Nützlich für Updates. |

### Angekündigt (noch kein Code)
- **Rocket Launcher** (@RLauncherRoblox): angekündigt als Open Source für Windows, mit Server-Picker, Freunde-Chat, mehr FPS und übersetzten Spielnamen. Bisher ist kein Repo öffentlich.
- **Lunar for Roblox** (lunar.gg, von den Machern von Lunar Client): Instanzen, FPS, Macros, HUD. Nicht Open Source angekündigt.

---

## 9. Ideen für eigene Funktionen

- Startseite mit „Weiterspielen“ und Empfehlungen (Games-API)
- Server-Picker mit Filtern (Spielerzahl, Ping, Region) und **Freund beitreten**
- Editor für FastFlags, der nur Flags der Allowlist anbietet, mit Presets („Performance“, „Qualität“)
- Editor für `GlobalBasicSettings_13.xml` (FPS-Limit, Empfindlichkeit)
- Mod-Manager (Cursor, Sounds, Schriften, Skybox)
- Discord Rich Presence + BloxstrapRPC-kompatibel
- Mehrere Konten und Instanzen (Konto-Profile)
- Spielzeit-Statistik (aus der Logdatei)
- Kanal- bzw. Versionswahl (nur Ansatz B)

---

## 10. Roadmap

1. **MVP (Ansatz A):** Tauri-App, Login per WebView, Spielesuche, Server-Picker, Start per Deeplink
2. Freunde und Presence, Discord RPC über die Logdatei
3. FastFlag-Editor und GlobalBasicSettings-Editor
4. Multi-Instanz (Mutex)
5. **Ansatz B:** eigener Bootstrapper (Manifest, Download, Protokoll-Registrierung) und Mods
6. Veröffentlichung auf GitHub (Lizenz z. B. MIT), signierte Releases

## 12. Nachtrag: weitere Möglichkeiten & Machbarkeits-Check

### 12.1 Wichtigste Erkenntnis: Das MVP braucht gar keinen Login
Ein Deeplink startet Roblox mit dem Konto, das in der Roblox-App schon angemeldet ist. Suche, Spieldetails, Bilder und die öffentliche Serverliste gehen ohne Cookie. **Ein Launcher mit Suche, Startseite und Server-Picker kommt also ganz ohne Zugangsdaten aus** und hat damit kein Konto-Risiko. Das Cookie braucht man erst für Freunde, Chat, Region und Konto-Wechsel.

### 12.2 Offizieller Login: OAuth 2.0 („Mit Roblox anmelden“)
- Roblox bietet OAuth 2.0 mit PKCE für Drittanbieter-Apps. Desktop-Apps gelten als „public client“, eigene URL-Schemata wie `my-app://` sind als Redirect erlaubt. ([Doku](https://create.roblox.com/docs/cloud/auth/oauth2-overview))
- Bedingungen: Der Entwickler muss per Ausweis verifiziert sein, die App geht durch eine Prüfung, und alle Scopes müssen in **eine** Kategorie der „Creator Third-Party App Policy“ passen. Nutzer müssen 13+ sein.
- Grenze: OAuth liefert Identität (`openid`, `profile`) und Open-Cloud-Zugriffe für Entwickler. Freunde, Chat und Serverbeitritt laufen nicht darüber.
- **Fazit:** gut für ein sauberes „Angemeldet als …“ ohne Cookie. Ersetzt das Cookie für Spielerfunktionen nicht.

### 12.3 Entdecken & Suche (ohne Login, aber undokumentiert)
- Suche: `https://apis.roblox.com/search-api/omni-search?searchQuery=…&sessionId=<zufällige GUID>`
- Startseiten-Reihen: `https://apis.roblox.com/explore-api/v1/get-sorts?sessionId=…` und `…/get-sort-content?sessionId=…&sortId=…`
- Autovervollständigung: `https://apis.roblox.com/games-autocomplete/v1/get-suggestion/<Text>`
- Details zu vielen Spielen auf einmal: `https://games.roblox.com/v1/games?universeIds=…` (höchstens **50** pro Abruf, 100 ergibt HTTP 400; dazu ein Limit pro IP, siehe recherche-ui-api.md B3)
- Place-ID → Universe-ID: `https://apis.roblox.com/universes/v1/places/<PlaceId>/universe`
- Sammlung undokumentierter Endpunkte: [BTRoblox-README](https://github.com/AntiBoomz/BTRoblox)
- Such- und Explore-API sind nicht offiziell dokumentiert und können jederzeit brechen. Zugriffe kapseln und Fehler abfangen.

### 12.4 Chat
- Endpunkte (undokumentiert): `apis.roblox.com/platform-chat-api/v1/` mit `get-conversation-metadata`, `send-messages`, `create-conversations`, `update-typing-status`. Cookie nötig.
- Seit Januar 2026 verlangt Roblox weltweit eine Altersprüfung zum Chatten. Der Launcher muss damit rechnen, dass Chat für ein Konto gesperrt ist.
- **Fazit:** baubar, aber die wackeligste Funktion. Spät einplanen.

### 12.5 Server-Region & Uptime
- Vorbild: [RoValra](https://github.com/NotValra/RoValra) (Browser-Erweiterung, GPL-3.0): Regionswahl, Server-Laufzeit, „Bevorzugte Region“-Knopf. Nutzt u. a. `gamejoin.roblox.com`.
- Prinzip: Beitrittsdaten eines Servers abfragen (Cookie nötig), daraus die Server-Adresse lesen, IP einer Region zuordnen.
- Achtung Lizenz: GPL-Code darf nicht in ein MIT-Projekt kopiert werden. Nur das Prinzip übernehmen und selbst schreiben.
- **Fazit:** baubar mit Cookie. Viele Abfragen nötig, also zwischenspeichern und langsam abfragen.

### 12.6 Konto-Wechsel / mehrere Konten
- So machen es Account-Manager: mit dem Cookie ein kurzlebiges Ticket holen (`POST https://auth.roblox.com/v1/authentication-ticket`, Antwort-Header `RBX-Authentication-Ticket`) und Roblox über das alte `roblox-player:`-Format mit diesem Ticket starten. In `roblox://`-Links lässt sich kein Konto mitgeben.
- Roblox' eigene Doku nennt inzwischen einen zusätzlichen Schritt (`GET /v1/client-assertion`) für die Ticket-Erzeugung.
- Vorbild: [Roblox Account Manager](https://github.com/ic3w0lf22/Roblox-Account-Manager) (GPLv3).
- **Fazit:** technisch baubar, aber Grauzone: mehrere gespeicherte Cookies sind ein lohnendes Ziel für Schadsoftware. Nur mit sicherem Speicher und erst nach dem MVP.

### 12.7 Overlays (Fadenkreuz, zweiter-Bildschirm-HUD)
- Erlaubte Bauart: ein **eigenes, transparentes, immer-oben-Fenster**, das nichts im Spiel liest oder verändert. Funktioniert nur im randlosen Fenstermodus, nicht im exklusiven Vollbild.
- Tabu: alles, was sich in den Roblox-Prozess einhängt. Hyperion blockiert solche Overlays (RTSS/MSI Afterburner, ReShade).
- In Tauri: zweites Fenster mit `transparent`, `always_on_top` und Klick-Durchlass.
- Ein HUD kann nur zeigen, was der Launcher von außen weiß: Server, Region, Spielzeit, Freunde (aus Logdatei und Web-API). Keine Spieldaten wie Leben oder Gegner.
- Eine offizielle Aussage von Roblox zu Fadenkreuz-Overlays habe ich nicht gefunden. Einzelne Spiele können das als unfair werten.

### 12.8 Machbarkeit auf einen Blick

| Funktion | Selbst baubar? | Cookie nötig? | Risiko |
|---|---|---|---|
| Suche, Startseite, Spieldetails | ja | nein | API undokumentiert |
| Server-Picker + Start per Deeplink | ja | nein | gering |
| FastFlag-Editor (Allowlist) | ja | nein | gering |
| FPS-Limit & Einstellungen (XML) | ja | nein | gering |
| Mods (Cursor, Sounds, Schriften) | ja, sauber nur mit Bootstrapper | nein | gering |
| Discord-Status, Spielzeit (Logdatei) | ja | nein | gering |
| Mehrere Instanzen (Mutex) | ja | nein | nicht offiziell unterstützt |
| „Mit Roblox anmelden“ (OAuth) | ja, nach Prüfung durch Roblox | nein | Ablehnung möglich |
| Freunde, Presence, Freund beitreten | ja | ja | Cookie-Sicherheit |
| Server-Region | ja | ja | viele Abfragen |
| Chat | ja | ja | undokumentiert, Altersprüfung |
| Konto-Wechsel | ja | ja (mehrere) | Grauzone |
| Fadenkreuz / HUD als externes Fenster | ja | nein | je nach Spiel |
| Macros / automatische Eingaben | nicht geprüft | – | bewusst weglassen, in vielen Spielen verboten |
| Mehr als von außen sichtbar (Spieldaten, FPS per Speicher) | **nein** | – | Sperre |

### 12.9 Angepasste Reihenfolge
1. MVP **ohne Login**: Suche, Startseite, Server-Picker, Deeplink-Start
2. FastFlag- und Einstellungs-Editor, Discord-Status
3. Login (WebView-Cookie, optional OAuth), Freunde, Region
4. Mehrere Instanzen, Overlay-Fenster
5. Eigener Bootstrapper + Mods
6. Chat, Konto-Wechsel

## 13. Nachtrag: weitere Grafik- & Performance-Stellschrauben

Es gibt drei Ebenen: FastFlags (Abschnitt 5), die Einstellungsdatei von Roblox (Abschnitt 6) und Windows selbst. Die Windows-Ebene ist neu und komplett im erlaubten Rahmen, weil sie den Client nicht berührt.

### 13.1 FastFlags: aktueller Stand
- Die Allowlist aus Abschnitt 5 ist weiterhin die Basis. Laut Roblox durchlaufen neue Einträge ein Freigabeverfahren.
- **`DFIntDebugDynamicRenderKiloPixels`** (Renderauflösung senken, großer FPS-Gewinn auf schwachen Geräten): Status **widersprüchlich**. Im Januar 2026 genehmigt, ein Video vom Februar 2026 meldet sie als freigeschaltet, Forennutzer schrieben zur selben Zeit, genehmigte Flags seien nie aktiviert worden. Im Launcher als „experimentell“ anbieten und selbst testen. Wert laut Forum: die gewünschte Auflösung, z. B. `"720"`. ([DevForum](https://devforum.roblox.com/t/fflag-allowlist-request-towards-users-with-low-end-devices/4275059))
- Ein Nutzer berichtete Anfang 2026, die Textur-Flags (`DFIntTextureQualityOverride`) seien zeitweise wirkungslos gewesen. Also auch erlaubte Flags regelmäßig gegenprüfen.
- Hinweis zu `DFIntDebugFRMQualityLevelOverride`: Stufen 1–6 gelten als niedrige, darüber als hohe Grafik (wie der Regler mit 21 Stufen).
- Grafik-API (D3D11 / Vulkan / OpenGL): Welche schneller ist, hängt vom Gerät ab. Der Launcher kann einen „Ausprobieren“-Schalter anbieten.

### 13.2 `GlobalBasicSettings_13.xml`: mehr als nur FPS
- Bestätigt: `FramerateCap` (0 = ohne Limit).
- Weitere Schlüssel dort (aus meiner Erinnerung, **vor dem Bau an einer echten Datei prüfen**; Nachtrag: in der Fishstrap-Vorlage heißt die Grafikstufe `SavedQualityLevel`, `GraphicsQualityLevel` ist nicht belegt): `SavedQualityLevel`, `Fullscreen`, `MouseSensitivity`, `MasterVolume`, `ReducedMotion`, `PreferredTransparency`.
- Empfehlung: keinen festen Schlüsselkatalog einbauen. Der Launcher liest die Datei des Nutzers ein und zeigt alle gefundenen Werte als bearbeitbare Liste. So bleibt er robust, wenn Roblox Schlüssel ändert.

### 13.3 Windows-Ebene (vom Launcher setzbar)

| Stellschraube | Umsetzung | Hinweis |
|---|---|---|
| **Starke Grafikkarte erzwingen** (Laptops!) | Registry `HKCU\Software\Microsoft\DirectX\UserGpuPreferences`, Wertname = voller Pfad zu `RobloxPlayerBeta.exe`, Daten `GpuPreference=2;` | Entspricht „Einstellungen → Anzeige → Grafik → Hohe Leistung“. Der Pfad ändert sich mit jedem Roblox-Update, der Launcher muss den Eintrag also nach jedem Update neu setzen. Größter Einzelgewinn auf Laptops, die sonst die integrierte Grafik nehmen. |
| **Vollbild-Optimierungen aus** | Registry `HKCU\Software\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Layers`, Wertname = exe-Pfad, Daten `~ DISABLEDXMAXIMIZEDWINDOWEDMODE` | Entspricht dem Häkchen unter Eigenschaften → Kompatibilität. Wirkung je nach System, als Option anbieten. Ebenfalls pro Update neu setzen. |
| **Prozess-Priorität** | Beim Start per `SetPriorityClass` auf „Höher als normal“ oder „Hoch“ | Machen Voidstrap und GokuTrap („Saiyan Mode“, dort auch CPU-Kernzuweisung). Nie „Echtzeit“. Bringt wenig, wenn die CPU ohnehin voll ausgelastet ist. |
| **Gestreckte Auflösung** | Desktop-Auflösung vor dem Start umstellen, danach zurück | Hat NexStrap als „Stretch resolution helper“. |
| **Hintergrund aufräumen** | Liste störender Programme zeigen (Aufnahme-Software, Overlays), Nutzer entscheidet | Nichts automatisch beenden. |

Die ersten zwei Registry-Formate stammen aus meinem Windows-Wissen, nicht aus einer heute geprüften Quelle. Vor dem Einbau kurz testen: Einstellung einmal von Hand in Windows setzen und nachsehen, was in der Registry steht.

### 13.4 Nur als Hinweis anzeigen (systemweit, nicht ungefragt ändern)
- Energiesparplan „Höchstleistung“, Laptop am Netzteil
- Hardwarebeschleunigte GPU-Planung, Spielmodus
- Xbox Game Bar und Hintergrundaufnahme aus
- Grafiktreiber-Profil für `RobloxPlayerBeta.exe` (NVIDIA Systemsteuerung / AMD Adrenalin)
- Im Spiel: Grafikmodus „Manuell“ und Qualität senken. Roblox hängt stark an der CPU, eine starke Grafikkarte allein hilft wenig.

Idee: ein **„Performance-Check“** im Launcher, der diese Punkte ausliest und als Ampelliste zeigt, mit Knopf zur jeweiligen Windows-Einstellung.

### 13.5 Bewusst weglassen
- **FPS-Unlocker per Speicherzugriff** (alte „rbxfpsunlocker“-Methode): Hyperion versucht, Speicheränderungen von außen zu erkennen. Tabu.
- **„Speicher-Bereinigung“ am laufenden Roblox-Prozess** (hat Voidstrap): greift in den Prozess ein, Nutzen fraglich. Nicht nachbauen.
- **ReShade / Shader-Injektion**: wird von Roblox blockiert.

### 13.6 Presets für den Launcher

| Preset | Inhalt |
|---|---|
| **Max FPS** | `FramerateCap` 0, FRM-Qualität niedrig, Gras 0, Textur 0, MSAA 0, Voxelizer pausiert, LOD-Distanzen 0, hohe Leistung-GPU, Priorität „Höher als normal“ |
| **Ausgewogen** | `FramerateCap` = Bildwiederholrate, Textur 2, MSAA 2, hohe Leistung-GPU |
| **Qualität** | Textur 3, MSAA 4–8, FRM-Qualität 21, Gras weit |
| **Schwacher Laptop** | wie Max FPS, dazu experimentell reduzierte Renderauflösung |

## 14. Feature-Ideen aus anderen Apps

### 14.1 Besonders spannend für uns
1. **Play-Knopf „bevorzugte Region“**: Ein Klick, und der Launcher tritt einem Server in der gewählten Region bei.
2. **Spiele-Playlists und eine anpassbare Startseite**: eigene Listen, Reihen selbst anordnen.
3. **Spielverlauf und Spielzeit-Statistik** aus der Logdatei.
4. **Einladungslink zum aktuellen Server über das Tray-Menü**: Der Link funktioniert auch für Leute ohne unseren Launcher (wie bei Fishstrap).
5. **Kleine oder neue Server finden**: Filter nach Spielerzahl und Laufzeit des Servers.
6. **Spieler-Log (wer kommt, wer geht)**: Das geht nur, soweit die Roblox-Logdatei es hergibt.

### 14.2 Browser-Erweiterungen (gute Ideen für die Oberfläche)
- **RoSeal**: Reihen der Startseite selbst anordnen, Gruppen per Drag-and-drop in Ordnern sortieren, Abzeichen zweier Nutzer vergleichen. Dazu Spiele-Playlists und Filter auf der Entdecken-Seite. ([Microsoft Edge Add-ons](https://microsoftedge.microsoft.com/addons/detail/emjkdffoonmeappoffiaalofdiflmela), [Firefox Add-ons](https://addons.mozilla.org/firefox/addon/roseal/))
- **RoValra**: Server nach Laufzeit filtern, Region wählen und ein Play-Knopf für die bevorzugte Region. ([Firefox Add-ons](https://addons.mozilla.org/addon/rovalra-rav4/), Quellcode unter GPL-3.0: nur das Prinzip übernehmen)

### 14.3 Weitere Funktionen anderer Launcher (zum Vergleich)
- **Fishstrap**: Spieler- und Chat-Log, Profile für FastFlags, Tray-Menü mit Spielverlauf, Serverstandort und Server-Laufzeit (über RoValra). ([Doku](https://docs-1w2.pages.dev/manual/integrations))
- **Voidstrap**: Prozess-Priorität und CPU-Kernzuweisung.
- **GokuTrap**: Konto-Profile, „Region Radar“, Fadenkreuz-Overlay.
- **Roblox Account Manager** (seit Oktober 2024 archiviert): zufälliger Server bei jedem Beitritt, kleine Server in Spielen mit Lobby betreten, Region und Ping pro Server, zuletzt gespielte Spiele mit automatisch ergänzter Place-ID, FPS-Limit pro Konto, Prüfung ob ein Freund online und erreichbar ist. ([GitHub](https://github.com/ic3w0lf22/roblox-account-manager))

---

## 15. Wie der Launcher schnell startet und reagiert

### 15.1 Warum Tauri
- Ein Vergleich vom Juli 2026 misst einen **3,7-mal schnelleren Kaltstart** (380 ms statt 1.420 ms) und **75 % weniger Arbeitsspeicher** im Leerlauf (42 MB statt 168 MB) gegenüber Electron. ([tech-insider](https://tech-insider.org/tauri-vs-electron-2026/))
- Ein anderer Test fand beim Start kaum einen Unterschied. ([Hopp](https://dev.to/gethopp/tauri-vs-electron-performance-bundle-size-and-the-real-trade-offs-1el4)) Den Ausschlag gibt also, wie wir die App bauen.

### 15.2 Start (die ersten Sekunden)
- **Größte Bremse ist WebView2 beim Kaltstart.** Laut Microsoft ist das Erzeugen der WebView und das erste Laden einer Seite ein häufiger Engpass, weil Prozesse und Disk-Caches erst hochfahren müssen. Läuft Edge in derselben Version schon, liegen die Dateien bereits im Speicher und der Start geht schneller. ([Microsoft](https://learn.microsoft.com/microsoft-edge/webview2/concepts/performance))
- **Im Tray weiterlaufen statt beenden:** Ein warmer Start ist fast sofort da. Ein Entwickler maß bei Tauri etwa 1 Sekunde bis zum fertig gezeichneten Fenster beim Kaltstart. Für Apps, die ohnehin im Hintergrund bleiben, ist das aber die falsche Messgröße. ([dev.to](https://dev.to/tinyforge/tauri-cold-launch-was-1s-heres-how-i-got-it-to-feel-instant-32lh)) Optional bieten wir „Mit Windows starten (versteckt)“ an.
- **Kein weißer Blitz:** Das Fenster erst zeigen, wenn die Oberfläche gezeichnet ist (`visible: false`, danach `show()`), oder einen Splashscreen nutzen. Für den Splashscreen eine Zeitgrenze setzen, damit er nicht hängen bleibt. ([Tauri Splashscreen](https://v2.tauri.app/learn/splashscreen/))
- **Schlankes Frontend:** Svelte oder SolidJS statt eines schweren Frameworks, dazu Vite mit Code-Splitting. Selten genutzte Seiten (Einstellungen, Mods) erst bei Bedarf laden. In einem Praxisbeispiel brachten Svelte, Code-Splitting und spätes Laden einzelner Module den Start von 2,5 s auf 1,2 s. ([Fallstudie](https://offline-pixel.github.io/case-studies/improving-desktop-app-startup-performance))

### 15.3 Laden der Daten
- **Erst aus dem Cache zeigen, dann aktualisieren:** Die letzte Startseite, Spielnamen und Bilder lokal speichern (SQLite oder `redb`) und sofort anzeigen. Neue Daten im Hintergrund holen und austauschen. So steht die Oberfläche in Millisekunden, auch wenn die Roblox-API langsam ist.
- **Bilder:** Thumbnails gebündelt abfragen (`thumbnails.roblox.com` erlaubt viele IDs pro Abruf), auf der Festplatte zwischenspeichern und erst beim Scrollen laden (`loading="lazy"`). Kleine Formate (WebP) wählen.
- **Parallel und gebündelt:** Ein einziger `reqwest`-Client mit Verbindungspool (HTTP/2, Keep-Alive). Spieldetails bis 50 pro Abruf holen (100 ergibt HTTP 400). Unabhängige Abfragen gleichzeitig starten.
- **Vorladen:** Fährt die Maus über ein Spiel, schon die Serverliste anfragen. Der Klick fühlt sich dann sofort an.

### 15.4 Bedienung
- Schwere Arbeit (Netz, Dateien, Logdatei mitlesen, entpacken) gehört ins Rust-Backend und läuft asynchron (`tokio`). Die Oberfläche bekommt nur fertige, kleine Daten.
- Lange Listen wie die Serverliste virtualisieren: nur die sichtbaren Zeilen zeichnen.
- Optimistische Oberfläche: Knöpfe reagieren sofort, das Ergebnis kommt nach.

### 15.5 Roblox selbst schneller starten (wie Bloxstrap)
- Updates im Hintergrund vorab laden, während der Launcher offen ist, nicht erst beim Klick auf „Spielen“.
- Mods und FastFlags nur neu schreiben, wenn sich etwas geändert hat (Prüfsumme vergleichen).

*Hinweis: Cache-zuerst, Vorladen, Virtualisierung und Verbindungspool sind bewährte Praxis, keine Rechercheergebnisse.*

## 16. Offene Punkte / Risiken
- Die Allowlist kann sich jederzeit ändern, also regelmäßig gegenprüfen.
- Die Web-APIs sind teils nicht offiziell dokumentiert und können sich ändern.
- Der Name darf weder „Roblox“ noch „Blox“ enthalten (Markenrecht bzw. AGB).
- Windows Defender meldet unsignierte Launcher oft fälschlich als Virus. Code-Signing einplanen.

---

## 17. Quellen (alle Recherchen, Stand 13.11.2026)

### Angekündigte Launcher (Anlass)
- [Rocket Launcher auf X (@RLauncherRoblox)](https://x.com/RLauncherRoblox/status/2120975825113870474): öffentlich nicht abrufbar, Inhalt per Screenshot
- [Lunar for Roblox](https://www.lunar.gg/roblox)

### Regeln & offizielle Roblox-Doku
- [DevForum: Automated Action Against Modified Clients](https://devforum.roblox.com/t/an-update-on-automated-action-against-modified-clients/3640609/1)
- [DevForum: Fast Flag Allowlist (Originalpost)](https://devforum.roblox.com/t/allowlist-for-local-client-configuration-via-fast-flags/3966569) · [Seite 9](https://devforum.roblox.com/t/allowlist-for-local-client-configuration-via-fast-flags/3966569?page=9) · [Seite 20](https://devforum.roblox.com/t/allowlist-for-local-client-configuration-via-fast-flags/3966569?page=20)
- [DevForum: Allowlist-Anfrage DFIntDebugDynamicRenderKiloPixels](https://devforum.roblox.com/t/fflag-allowlist-request-towards-users-with-low-end-devices/4275059)
- [DevForum: FPS-Limit über GlobalBasicSettings_13.xml](https://devforum.roblox.com/t/roblox-caps-fps-at-240-regardless-of-globalbasicsettings13xml-framerate-cap/4451048)
- [DevForum: RTSS/MSI Afterburner von Hyperion blockiert](https://devforum.roblox.com/t/rtss-msi-afterburner-blocked-by-hyperion/3274953)
- [DevForum: Roblox FPS Uncapper (Speicherzugriff)](https://devforum.roblox.com/t/roblox-fps-uncapper-for-web-client-runtime/2971591)
- [Roblox Doku: ClientSettings API](https://create.roblox.com/docs/cloud/reference/domains/clientsettings)
- [Roblox Doku: Deep Links](https://create.roblox.com/docs/en-us/production/promotion/deeplinks)
- [Roblox Doku: OAuth 2.0 Überblick](https://create.roblox.com/docs/cloud/auth/oauth2-overview) · [App-Registrierung](https://create.roblox.com/docs/cloud/auth/oauth2-registration.md) · [Implementierung](https://create.roblox.com/docs/en-us/cloud/auth/oauth2-develop)
- [Roblox Doku: Private Server Endpunkte](https://create.roblox.com/docs/en-us/cloud/reference/features/private-servers.md)
- [Overwolf Roblox Launcher Terms](https://legal.overwolf.com/docs/roblox-launchers/terms-and-conditions)
- [Tubefilter: Roblox Everywhere](https://www.tubefilter.com/?p=194121) · [mobilegamer.biz](https://mobilegamer.biz/roblox-to-let-creators-turn-their-games-into-standalone-apps/)

### Technik: Bootstrapper, Launch, APIs
- [Bloxstrap Wiki: A deep dive on how the Roblox bootstrapper works](https://github.com/pizzaboxer/bloxstrap/wiki/A-deep-dive-on-how-the-Roblox-bootstrapper-works)
- [Bloxstrap Wiki: BloxstrapRPC ins Spiel integrieren](https://github.com/pizzaboxer/bloxstrap/wiki/Integrating-Bloxstrap-functionality-into-your-game)
- [Bloxstrap Wiki: Modding](https://bloxstraplabs.com/wiki/features/modding/)
- [Roblox Client Watch (Manifest-Formate)](https://github.com/CloneTrooper1019/Roblox-Client-Watch) · [Roblox Client Tracker](https://github.com/MaximumADHD/Roblox-Client-Tracker)
- [DevForum: Deeplink mit gameInstanceId](https://devforum.roblox.com/t/developer-deeplinking-beta/1904069/179)
- [DevForum: Serverliste & Region](https://devforum.roblox.com/t/how-to-find-the-server-region-from-the-website/2862705) · [700-Server-Grenze](https://devforum.roblox.com/t/unable-to-fetch-more-than-700-servers-servers-using-the-games-v1-api-gamesrobloxcomv1/4294098)
- [DevForum: Search- und Explore-API](https://devforum.roblox.com/t/robloxs-games-api/3483783)
- [BTRoblox README (undokumentierte APIs)](https://github.com/AntiBoomz/BTRoblox)
- [Roblox Wrapper (Startargumente -a/-t/-j)](https://github.1485827954.workers.dev/Hexcede/Roblox-Wrapper)
- [npm „roblox“ (Authentication-Ticket)](https://www.npmjs.com/package/roblox)
- [MultiBloxy (Mutex, mehrere Instanzen)](https://github.com/zgoly/multibloxy) · [DevForum: Teleport bei mehreren Clients](https://devforum.roblox.com/t/teleportserviceteleportasync-fails-if-there-are-multiple-clients-running-on-the-same-machine-byfron-issue/2324412)
- [glitchingdocs: FPS-Limit-Methoden](https://glitchingdocs.gitlab.io/tips/fpslockingmethods.html)

### Vorbilder (Launcher & Tools)
- [Bloxstrap](https://github.com/bloxstraplabs/bloxstrap) · [Webseite](https://bloxstraplabs.com/)
- [Fishstrap](https://github.com/imide/fishstrap) · [Fishstrap Integrationen](https://docs-1w2.pages.dev/manual/integrations)
- [Voidstrap: Performance-Optimierungen](https://deepwiki.com/voidstrap/Voidstrap/5.3-performance-optimizations)
- [NexStrap](https://github.com/k153636/nexstrap) · [Nyxstrap](https://github.com/nyxstrap/Nyxstrap)
- [GokuTrap Wiki](https://github.com/gokuthug1/GokuTrap/wiki)
- [Vinegar (Linux, Go)](https://github.com/vinegarhq/vinegar)
- [Roblox Account Manager](https://github.com/ic3w0lf22/roblox-account-manager) · [Releases](https://github.com/ic3w0lf22/roblox-account-manager/releases)
- [RoValra (Firefox)](https://addons.mozilla.org/addon/rovalra-rav4/) · [RoSeal (Edge)](https://microsoftedge.microsoft.com/addons/detail/emjkdffoonmeappoffiaalofdiflmela) · [RoSeal (Firefox)](https://addons.mozilla.org/firefox/addon/roseal/)

### Rust
- [rsblox (Rust + Tauri + Svelte)](https://github.com/idkncc/rsblox)
- [ApplejuiceCLI (Rust-Bootstrapper)](https://github.com/WaviestBalloon/ApplejuiceCLI)
- [roboat (Rust-Bibliothek für die Roblox-API)](https://lib.rs/crates/roboat) · [DevForum-Vorstellung](https://devforum.roblox.com/t/roboat-a-roblox-api-library-for-rust/2388303)

### Performance (Windows & App-Start)
- [GamerHardware: Roblox FPS (GPU-Einstellung)](https://gamerhardware.org/boost-fps-roblox-settings/)
- [Tauri vs Electron 2026 (tech-insider)](https://tech-insider.org/tauri-vs-electron-2026/) · [Hopp-Benchmark](https://dev.to/gethopp/tauri-vs-electron-performance-bundle-size-and-the-real-trade-offs-1el4)
- [Microsoft: WebView2 Performance](https://learn.microsoft.com/microsoft-edge/webview2/concepts/performance)
- [Tauri: Splashscreen](https://v2.tauri.app/learn/splashscreen/) · [Tauri Kaltstart „instant“](https://dev.to/tinyforge/tauri-cold-launch-was-1s-heres-how-i-got-it-to-feel-instant-32lh)
- [Fallstudie: Startzeit 8 s → 1,2 s](https://offline-pixel.github.io/case-studies/improving-desktop-app-startup-performance)
- [Luair (externes Overlay-Fenster)](https://luair.xmit.dev/)

### Logo & Schrift
- [Archivo (Fontsource, variable)](https://fontsource.org/fonts/archivo): Grundlage des Schriftzugs, SIL Open Font License
