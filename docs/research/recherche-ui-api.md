# Recherche: UI-Vorlagen und Roblox-Web-APIs

Stand: 13.11.2026. Ergänzt [roblox-launcher-rust.md](roblox-launcher-rust.md). Was dort schon steht (Allowlist-Basis, Deeplink-Grundformen, 700-Server-Grenze, Mutex, Bootstrapper-Ablauf), wird hier nicht wiederholt.

## 0. Lesehinweise und Grenzen der Recherche

Kennzeichnung: **[bestätigt]** = aus abgerufenem Quelltext oder Dokument. **[Bericht]** = aus einer Fremdquelle, die selbst Messungen oder Beispiele zeigt, nicht von mir abgerufen. **[unsicher]** = Erinnerung, Suchtreffer-Zusammenfassung oder Schluss.

**Netzwerk in der Cloud-Session:**
- Alle Roblox-Hosts sind **nicht erreichbar**: `curl` auf `apis.roblox.com` und `games.roblox.com` scheitert mit `CONNECT tunnel failed, response 403`. Ich habe also **keine Endpunkte selbst abgerufen**. Alle Antwortfelder unten stammen aus Quelltext und Berichten.
- Nicht auflösbar (WebFetch): `devforum.roblox.com`, `create.roblox.com`, `docs.rs`, `deepwiki.com`. Zum Freischalten müssten diese Hosts in den Netzwerkeinstellungen der Session erlaubt werden.
- Erreichbar: `raw.githubusercontent.com` (Quelltexte, README). `api.github.com` liefert nur für nicht angehängte Repos 403.
- WebSearch liefert nur Zusammenfassungen. Reddit-Threads und Screenshots waren nicht lesbar.

**Lizenz:** Bloxstrap und Fishstrap sind MIT (Hinweis auf Herkunft nötig, wenn Code übernommen wird). Von mir gelesen, nur Muster und Formate notiert, kein Code kopiert. RoValra (GPL) nicht gelesen.

---

## Teil A: UI-Vorlagen

### A1. Die Strap-Familie

**Bloxstrap** [bestätigt, `Bloxstrap/UI/Elements/Settings/MainWindow.xaml`, `Strings.resx`]
- Der Projektstatus: README sagt „Bloxstrap is not under active development anymore“. Als Vorlage taugt es, als lebendiger Maßstab nicht mehr.
- Navigation: linke Leiste (WPF-UI „NavigationFluent“) mit Seiten **Integrations, Behaviour, Mods, FastFlags, Appearance, Shortcuts**. Unten im Fuß: eine Bloxstrap-Seite (Daten exportieren, Analytics) und „About“. Der FastFlag-Editor ist eine **versteckte Unterseite** (nur über die FastFlags-Seite erreichbar).
- Unten rechts feste Knopfleiste: **Save and Launch, Save, Close**. Einstellungen werden also gesammelt gespeichert, nicht live.
- Integrations: Aktivitäts-Tracking, „Query server location“ (ipinfo.io), Discord „Show game activity“, „Allow activity joining“, eigene Integrationen (Befehle) und Desktop-App-Optionen.
- Mods: Presets (Mauszeiger, Emoji-Typ, alter Avatar-Editor, alte Charakter-Sounds), eigene Schrift, „Open mods folder“.
- FastFlags-Seite (nach Allowlist stark geschrumpft): Schalter „Manager aktiv“, Presets für **MSAA, Display-Skalierung (DPI), Texturqualität**, „Reset“.
- FastFlag-Editor: Suchfeld, **Tabelle Name | Wert** (Doppelklick zum Bearbeiten), Knöpfe „Add new“, „Delete selected“, „Show preset flags“, „Export JSON“, Banner „Misusing this can lead to instability“, Link zum Allowlist-Thread.

**Fishstrap** (MIT, Bloxstrap-Fork) [bestätigt, README und `MainWindow.xaml`]
- Gleiche Struktur, zusätzlich: **Deployment/Channel**-Seite und eine eigene **Global Settings**-Seite (Editor für `GlobalBasicSettings_13.xml`, siehe B7).
- Serverinfo über die RoValra-API („Detailed server information“), eigene **Einladungslinks** (`https://www.fishstrap.app/v1/joingame?placeId=…`), Cache-Cleaner, Channel-Wechsel, Studio-Unterstützung.
- README nennt ausdrücklich „You cannot apply FastFlags not present in the allowlist“.

**Voidstrap, GokuTrap, NexStrap:** README und Oberflächen-Quelltext waren nicht abrufbar (404 bzw. gesperrt). Aus Suchtreffern **[unsicher]**: Voidstrap hat Base64-Kopieren/Einfügen im Flag-Editor und einen „FFlag Bypass“-Schalter (aus). Ein Drittanbieter-FastFlag-Manager gruppiert Flags zu **Presets**, ordnet sie und teilt sie als JSON-Datei oder Base64-Text. Das Muster „Preset = benannte Flaggruppe, teilbar als Text“ ist übernehmenswert, wenn nur Allowlist-Flags enthalten sind.

**Kritik von Nutzern:** Reddit und Issue-Threads waren nicht lesbar, ich habe **keine belegten Nutzerzitate**. Ableitbar aus bestätigten Fakten:
- Der Editor ist seit der Allowlist (29.09.2025) größtenteils wirkungslos, weil fast alle Flags ignoriert werden. Das Bloxstrap-Banner erklärt das nur in einem Link.
- Alle Einstellungen hängen an „Save“, es gibt keine Rückmeldung, was wirklich wirkt. Brickstrap sollte „wirkungslos“ markieren (steht schon in CLAUDE.md).
- Die Seiten sind Einstellungs-zentriert. Es gibt keine Spiele- oder Serverseite (das ist die Lücke für Brickstrap).
- Weitere Punkte (Fehlalarme von Virenscannern, Sicherheitsbedenken bei Forks) tauchen in Suchtreffern auf **[unsicher]**. Signatur und offene Doku sind die Gegenmittel.

### A2. Desktop-Launcher: Bibliothek, Favoriten, Server-Listen

Belegt sind hier nur wenige Punkte. Der Rest ist Allgemeinwissen **[unsicher]**.

| Launcher | Muster | Quelle |
|---|---|---|
| Heroic | Einheitliche Bibliothek als Karten-Raster, **umschaltbar auf Liste**. Oben eine **konfigurierbare Reihe „Favoriten“ oder „Zuletzt gespielt“**. Seitenleiste einklappbar. | [Bericht] gamingonlinux, linuxcompatible (Release-Notes 2.3 bis 2.16) |
| Prism | Instanz-Liste mit Gruppen, rechts Aktionsleiste (Start, Ordner, Einstellungen). Kein belegtes Raster. | [unsicher] |
| Steam-Serverbrowser | Tabelle, **Spaltenköpfe klickbar zum Sortieren**. Filter: Standort, Modus, Karte, Ping, „leere ausblenden“, „volle ausblenden“, Passwort, max. Spieler. Tabs für Internet, Favoriten, Verlauf, LAN. | [Bericht] TF2- und CS-Wiki |
| Battle.net, Lunar Client, Discord | Großer „Spielen“-Knopf je Spiel, zuletzt gespielt zuerst, Status-Chip „läuft“. Discord zeigt „Beitreten“ als Knopf im Status. Nicht abgerufen. | [unsicher] |

**Bewährte Muster für einen Server-Browser** (aus Steam-Wiki-Belegen plus Plausibilität):
1. Tabelle mit sortierbaren Spalten: **Spieler (x/max), Ping, FPS, Alter, Region**. Standardsortierung nach dem, was der Nutzer meistens sucht.
2. Schnellfilter als Chips statt Formular: „Nicht voll“, „Wenig Spieler“, „Niedriger Ping“, „Neu“.
3. Ping als Zahl **und** als Farbe (grün/gelb/rot), Farben aber nur mit Zusatzsignal (Zahl), nicht allein.
4. **Ein-Klick-Beitritt** je Zeile, Doppelklick wie Beitreten. Zusätzlich ein „Bester Server“-Knopf oben.
5. Favoriten und „zuletzt besucht“ als eigene Tabs. Die Serverliste nur manuell oder alle 30 bis 60 s aktualisieren (Rate-Limit).

### A3. RoValra / RoSeal beim Server-Picker (nur Prinzip)

Direkt abgerufen habe ich weder RoValra noch RoSeal (GPL bzw. nicht gelesen). Aus dem bestehenden Plan und einem MIT-nahen Beispiel **[Bericht]**:
- **Region:** Für jeden Server die Serveradresse über `gamejoin.roblox.com` ermitteln (braucht Anmeldung), die IP einem Standort zuordnen, dann nach Region filtern oder „bevorzugte Region“-Knopf anbieten. Das Chrome-Projekt [Naxumi/roblox-server-finder](https://github.com/Naxumi/roblox-server-finder) beschreibt genau diesen Ablauf: Serverliste holen, je Server `POST gamejoin.roblox.com/v1/join-game-instance`, `Address` aus der Antwort lesen, IP gegen eine fest eingebaute IP-Bereichsliste prüfen, bei Treffer beitreten.
- **Server-Alter und kleine Server:** Die Liste liefert weder Alter noch Startzeit. Plan: „Alter“ nur schätzen oder weglassen. Kleine Server filtern über `playing` (siehe B5).
- **UI-Idee:** Filterleiste über der Tabelle, Region als Dropdown, „Bevorzugte Region“-Knopf als primäre Aktion, Fortschrittsanzeige beim Region-Scan („12 von 40 Servern geprüft“).
- **Kosten:** Ein Join-Aufruf pro Server. Vor dem Bau messen, nur für die ersten N sichtbaren Server, mit Cache je Server-ID.

---

## Teil B: Roblox-Web-APIs

Alle undokumentiert außer den offiziellen Domain-Seiten (`games`, `thumbnails`). Roblox kann sie jederzeit ändern. Quelle für viele Messwerte: das README von [MoulinLouis/roblox-trends](https://github.com/MoulinLouis/roblox-trends), Abschnitt „Verified external API behavior“, Messungen vom Oktober 2026, **[Bericht]**. Lizenz des Repos unklar: nur Fakten übernommen.

### B1. Suche: `omni-search`

```
GET https://apis.roblox.com/search-api/omni-search
    ?searchQuery=<Text>&sessionId=<GUID>&pageType=all[&pageToken=<nextPageToken>]
```
- Parameter [Bericht roblox-trends]: `searchQuery`, `sessionId` (zufällige GUID, pro Sitzung gleich lassen), `pageType=all`, `pageToken`. Das BTRoblox-README nennt `SearchQuery` und `SessionId` großgeschrieben [bestätigt]. Groß/klein ist wahrscheinlich egal **[unsicher]**.
- Antwort [Bericht]: `searchResults[]`, jedes mit `contents[]`. Jeder Eintrag hat `contentType`, nur `"Game"` behalten. Ein `nextPageToken` für die nächste Seite. Erste Seite hatte 40 Ergebnisse.
- Felder je Spiel [Bericht, Namen teils unsicher]: `universeId`, `rootPlaceId`, `name`, `playerCount`, Stimmen (`totalUpVotes`, `totalDownVotes` waren in meinen Quellen nicht namentlich bestätigt, nur „vote/maturity fields“). **Beim ersten echten Aufruf Felder prüfen.**
- **Limit [Bericht]:** Von einer Server-IP nur **1 Anfrage pro Minute** (`x-ratelimit-limit: 1, 1;w=60`), von einer Heim-IP etwa 4 pro Sekunde. Beim zweiten Aufruf kommt 429. Für Brickstrap (Heim-IP): Eingabe entprellen (300 bis 400 ms), laufende Anfragen abbrechen, `x-ratelimit-remaining` lesen und bei 0 pausieren.
- Autovervollständigung `games-autocomplete/v1/get-suggestion/<Text>` steht im Plan, von mir nicht geprüft.

### B2. Startseite: `explore-api`

```
GET https://apis.roblox.com/explore-api/v1/get-sorts?sessionId=<GUID>[&device=computer&country=<xx>]
GET https://apis.roblox.com/explore-api/v1/get-sort-content?sessionId=<GUID>&sortId=<id>[&device=…&country=…]
```
- BTRoblox-README listet nur die zwei Pfade mit `sessionId` und `sortId` [bestätigt]. `device` und `country` stammen aus roblox-trends [Bericht].
- `get-sorts` liefert ein `sorts`-Array, Spiele-Sorts enthalten ihre `games` schon eingebettet. Bekannte `sortId`s: `top-trending`, `up-and-coming`, `top-playing-now`, `top-rated`, `most-popular`, `top-paid-access`, `top-earning`, `trending-in-*` (13 Genres). Manche gültige Sorts fehlen in `get-sorts`.
- `get-sort-content` liefert **ein** Sort-Objekt mit `games[]`. Je Spiel `universeId`, `rootPlaceId`, `name`, `playerCount`, Stimmen, Altersfreigabe. **Kein Fortsetzungs-Token** und höchstens etwa 100 Spiele je Sort [Bericht, DevForum-Treffer].
- `country` ändert die Reihung wirklich (Beispiel im Bericht). Für Brickstrap: Land aus dem Windows-Gebietsschema, mit Einstellung überschreibbar.
- Limits laut Antwort-Headern [Bericht]: 50 Anfragen/min für Sort-Inhalt, 120/min für Sorts. Header sind beobachtet, nicht zugesagt.
- Seit Oktober 2026 liefert `games/recommendations/game/{universeId}` ohne Anmeldung eine leere Liste [Bericht]. **„Ähnliche Spiele“ nicht darauf bauen.**

### B3. Spieldetails, Stimmen, Place zu Universe

| Endpunkt | Antwort | Bemerkung |
|---|---|---|
| `GET games.roblox.com/v1/games?universeIds=1,2,3` | `data[]` mit `id` (Universe), `rootPlaceId`, `name`, `description`, `sourceName`, `creator{id,name,type,hasVerifiedBadge}`, `price`, `playing`, `visits`, `maxPlayers`, `created`, `updated`, `genre`, `favoritedCount`, `createVipServersAllowed`, `universeAvatarType`, `copyingAllowed` … | Felder [bestätigt, Bloxstrap `GameDetailResponse.cs`] |
| `GET games.roblox.com/v1/games/votes?universeIds=…` | `data[]` mit `id`, `upVotes`, `downVotes` | ohne Anmeldung [Bericht] |
| `GET apis.roblox.com/universes/v1/places/{placeId}/universe` | `{"universeId": 123}` | Place ohne Universe: HTTP 200 mit `{"universeId": null}`, nicht 404 [Bericht] |

**Korrektur zum Plan (§12.3):** „bis 100 pro Abruf“ stimmt für `games?universeIds=` **nicht**. Gemessen: **50 IDs sind das Maximum, 100 ergibt HTTP 400** [Bericht, Okt. 2026]. Außerdem gibt es ein verstecktes **Limit pro IP, das IDs zählt**: 50 IDs alle 2 s gaben bei der 6. Anfrage 429, dauerhaft verträglich waren etwa 430 IDs pro Minute (eine Anfrage à 50 IDs alle 7 s). Die `x-ratelimit-*`-Header (300/min) bilden das nicht ab. Für Brickstrap: Detailabfragen bündeln (50), cachen, nur für sichtbare Karten laden. Gelöschte oder gesperrte Spiele kommen als Platzhalter mit `id` 0 und Name `[TITLE UNAVAILABLE]` zurück, die Einträge überspringen. `votes` teilt dieses Limit nicht (3.928 Spiele in 39 s ohne 429).

### B4. Thumbnails

```
GET https://thumbnails.roblox.com/v1/games/icons
    ?universeIds=1,2&returnPolicy=PlaceHolder&size=150x150&format=Webp&isCircular=false
GET https://thumbnails.roblox.com/v1/games/multiget/thumbnails
    ?universeIds=1,2&countPerUniverse=3&defaults=false&size=768x432&format=Png&isCircular=false
```
- `icons`: Antwort `data[]` mit `targetId` (Universe), `state`, `imageUrl`, optional `errorCode`, `errorMessage`, `requestId` [bestätigt, Bloxstrap-Modell]. `state` ist `Completed`, `Pending`, `InReview`, `Blocked`, `TemporarilyUnavailable` oder `Error`. Bloxstrap nutzt `size=128x128&format=Png`. `format=Webp` und `150x150` nutzt roblox-trends, „nicht einzeln verifiziert“ [Bericht].
- `multiget/thumbnails`: `data[]` mit `universeId`, `error`, `thumbnails[{targetId,state,imageUrl}]`, geprüft am 20.10.2026 mit **50 Universen je Aufruf** [Bericht]. Mit `defaults=true` kommen gemeinsame Platzhalterbilder als `Completed`, darum `defaults=false`.
- Zulässige Größen (Auswahl 150x150, 256x144, 768x432 usw.) und Webp-Unterstützung je Größe: **[unsicher]**, beim ersten Aufruf prüfen.
- Bild-URLs tragen ein Ablaufpräfix (`180DAY-`) und laufen wahrscheinlich ab: im Cache nach Media-ID, nicht nach URL schlüsseln [Bericht].
- Batch-Größe: mit **50** planen (der Plan nennt „viele IDs“, die offizielle Grenze von 100 ist unbestätigt).

### B5. Serverliste

```
GET https://games.roblox.com/v1/games/{placeId}/servers/Public
    ?sortOrder=Asc|Desc&limit=10|25|50|100&excludeFullGames=true|false&cursor=<token>
```
- Beispielantwort [Bericht, README von Naxumi/roblox-server-finder, älteres Beispiel]:
  ```json
  { "previousPageCursor": null, "nextPageCursor": "eyJz…",
    "data": [ { "id": "63e76d8a-…", "maxPlayers": 12, "playing": 12,
                "playerTokens": ["3A18…", "8D14…"], "players": [],
                "fps": 59.95, "ping": 44 } ] }
  ```
- `id` = JobId = `gameInstanceId`. `playerTokens` sind verschleierte Kennungen (nicht zu Nutzern auflösbar), `players` ist leer. `fps` ist die Server-Bildrate, `ping` laut Forum die Latenz des Servers (Messpunkt **[unsicher]**, nicht als Ping des Nutzers anzeigen, besser „Server-Ping (Roblox)“).
- `sortOrder` `Asc` oder `Desc` (nach Spielerzahl) [Bericht]. `limit` 10/25/50/100 ist aus meiner Erinnerung **[unsicher]**. `excludeFullGames=false` lieferte laut Forum teils private Server in der Liste **[Bericht]**.
- Grenze etwa 700 Server je Abfrage steht im Plan. Ratenlimit speziell für diesen Endpunkt habe ich **nicht gefunden**. Vorsichtig planen (Abstand 1 s, 429 mit `Retry-After` und Backoff, wie der roblox-trends-Client).
- Fehlende Felder: kein Alter, keine Region, keine Spielernamen. Region nur über B6 (mit Anmeldung).

### B6. Deeplinks, Beitritt, Region

- **Bestätigt (Bloxstrap `ActivityData.GetInviteDeeplink`):**
  - öffentlicher Server: `roblox://experiences/start?placeId={PlaceId}&gameInstanceId={JobId}`
  - privater Server: `roblox://experiences/start?placeId={PlaceId}&accessCode={AccessCode}`
  - optional `&launchData=<URL-kodiert>`
  - Bloxstrap startet damit den Client direkt: `Process.Start(playerPath, deeplink)`.
- Der Beitritt läuft über das **im Roblox-Client angemeldete Konto**. Brickstrap braucht dafür kein Cookie.
- **Zuverlässigkeit von `gameInstanceId`:** Bloxstrap baut seine „Join server“-Knöpfe in Discord darauf [bestätigt]. Dass es immer klappt, ist **nicht belegt**. Gründe für Fehlschläge **[unsicher]**: Server ist inzwischen voll oder geschlossen, Spiel verwendet eigene Reservierung (Teleport-Spiele), Server in der Liste ist schon weg. Für die UI: Fehlschlag erwarten, „Server nicht mehr verfügbar, anderen wählen“ anbieten, nie als garantiert darstellen.
- **Alternativen:** Das alte `roblox-player:1+launchmode:play+…+placelauncherurl:…`-Format braucht ein Authentifizierungs-Ticket (Konto-Wechsel, siehe Plan §12.6). Für ein Launcher-MVP ohne Login ist `roblox://experiences/start` der Weg. Fishstrap stellt für Einladungen Web-Links bereit (`https://www.fishstrap.app/v1/joingame?placeId=…`), die an den Launcher übergeben werden [bestätigt, README]. Offizielle Web-Variante ohne Launcher: `https://www.roblox.com/games/start?placeId=…&launchData=…` **[unsicher]**.
- **Region ohne Anmeldung:** nur **nach** dem Beitritt über die Logdatei (UDMUX-Zeile, siehe B8) plus IP-Zuordnung. Vor dem Beitritt braucht man `gamejoin.roblox.com/v1/join-game-instance` mit Anmeldung (Cookie, CSRF-Token, User-Agent `Roblox/WinInet`; Körper `{placeId, isTeleport, gameId, gameJoinAttemptId}`, Antwort enthält `joinScript` mit `Address`) [Bericht, Naxumi-README]. Bloxstrap fragt `https://ipinfo.io/{ip}/json` ab (City, Region, Country) [bestätigt]. Das schickt IP-Adressen des Spieleservers an einen Drittdienst. Alternative laut Naxumi: **eigene IP-Bereichsliste** (Roblox-Server-Regionen) lokal, ohne Netzabfrage. Datenschutz-freundlicher.

### B7. `GlobalBasicSettings_13.xml`

Pfad `%localappdata%\Roblox\GlobalBasicSettings_13.xml` [bestätigt, Fishstrap `Paths.Roblox`].
Echter Aufbau laut Fishstrap-Vorlage (`GlobalBasicSettings_Template.xml`) [bestätigt]:

```xml
<?xml version="1.0" encoding="utf-8"?>
<roblox xmlns:xmime="http://www.w3.org/2005/05/xmlmime"
        xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
        xsi:noNamespaceSchemaLocation="http://www.roblox.com/roblox.xsd" version="4">
  <External>null</External>
  <External>nil</External>
  <Item class="UserGameSettings" referent="RBX…">
    <Properties>
      <int name="FramerateCap">60</int>
      <token name="SavedQualityLevel">2</token>
      <float name="MouseSensitivity">1</float>
      <bool name="VREnabled">false</bool>
      <float name="PreferredTransparency">1</float>
      <bool name="ReducedMotion">false</bool>
      <token name="PreferredTextSize">1</token>
    </Properties>
  </Item>
</roblox>
```
- XPath, das Fishstrap nutzt: `//Item[@class='UserGameSettings']/Properties/<typ>[@name='<Name>']` [bestätigt]. Typen in dieser Vorlage: `int`, `float`, `bool`, `token`. `SavedQualityLevel` 0 = automatisch.
- **Korrektur zum Plan (§13.2):** Der Schlüssel heißt in der Vorlage **`SavedQualityLevel`** (Typ `token`). `GraphicsQualityLevel` konnte ich **nicht belegen**. `Fullscreen`, `MasterVolume` stehen in dieser Vorlage nicht, sind aber in echten Dateien wahrscheinlich vorhanden **[unsicher]**. Die Vorlage ist minimal, eine echte Datei enthält mehr (Kameramodus, Lautstärke u. a., ein Bloxstrap-Issue zeigt `<token name="CameraMode">`) **[Bericht, Suchtreffer]**.
- Die Typen `string` und `Vector2` aus der Aufgabenstellung konnte ich **nicht belegen**. Nicht fest einbauen, sondern beim Lesen den Elementnamen als Typ nehmen und unbekannte Typen als Rohtext anzeigen, bearbeitbar nur für `int`, `float`, `bool`, `token`.
- Fishstrap setzt die Datei bei Bedarf **schreibgeschützt** (`FileAttributes.ReadOnly`), damit Roblox sie nicht überschreibt [bestätigt]. Das ist ein Hack, Roblox kann dann nichts mehr speichern. Für Brickstrap lieber: nur schreiben, wenn kein Roblox-Prozess läuft (so steht es in CLAUDE.md).
- FPS-Limit: Ein DevForum-Thread meldet 240 FPS als harte Obergrenze trotz `FramerateCap`, Fishstrap-Doku behauptet das Gegenteil **[unsicher, widersprüchlich]**. Im UI nicht mehr als 240 zusichern.

### B8. Logdatei

Ordner `%localappdata%\Roblox\logs` (`Paths.RobloxLogs`) [bestätigt]. Bloxstrap nimmt die neueste Datei, deren Name „Player“ enthält und die vor höchstens 15 s erstellt wurde, und liest sie im Takt von 1 s mit `FileShare.ReadWrite` [bestätigt, `ActivityWatcher.cs`].

Bestätigte Zeilen und Muster (aus `ActivityWatcher.cs`):

| Ereignis | Zeilenanfang | Regex / Inhalt |
|---|---|---|
| Beitritt beginnt | `[FLog::Output] ! Joining game` | `! Joining game '([0-9a-f\-]{36})' place ([0-9]+) at ([0-9\.]+)` → JobId, PlaceId, IP |
| Universum, Nutzer | `[FLog::GameJoinLoadTime] Report game_join_loadtime:` | `universeid:([0-9]+).*userid:([0-9]+)`; `referral_page:` kann auf privaten Server hinweisen |
| UDMUX-Proxy | `[FLog::Network] UDMUX Address = ` | `UDMUX Address = ([0-9\.]+), Port = [0-9]+ \| RCC Server Address = ([0-9\.]+), Port = [0-9]+` → erste IP ist die echte Maschine für Standort |
| Beitritt bestätigt | `[FLog::Network] Replicator created: ` | `serverId: ([0-9\.]+)\|[0-9]+` |
| Verlassen | `[FLog::Network] Time to disconnect replication data:` | Ende der Sitzung |
| Teleport | `[FLog::UgcExperienceController] UgcExperienceController: doTeleport: joinScriptUrl` | `JoinTypeId"%3a(\d+)%2c` |
| Spiel verlassen (Menü) | `[FLog::SingleSurfaceApp] leaveUGCGameInternal` | |
| BloxstrapRPC | `[FLog::CreatorOutput] [BloxstrapRPC]` | `\[BloxstrapRPC\] (.*)` → JSON |

Hinweis im Quelltext: Diese Zeilen sind **flüchtig**, sie hängen am eingestellten FLog-Level und können sich ändern. Alles abfangen, bei Formatabweichung nur loggen. „Connection lost“ ist in den gelesenen Dateien **nicht** als Muster enthalten, die Trennung erkennt Bloxstrap über „Time to disconnect replication data“.

**BloxstrapRPC-Format** [bestätigt, `Message.cs`, `RichPresence.cs`, `DiscordRichPresence.cs`]
- Spiel schreibt `print("[BloxstrapRPC] " .. json)`, JSON `{"command": "...", "data": {...}}`. Befehle in dem Teil des Codes, den ich gelesen habe: `SetRichPresence`, `SetLaunchData`. Weitere (z. B. Fensterbefehle) **[unsicher]**.
- `SetRichPresence`-Daten: `details`, `state` (je max. 128 Zeichen, `"<reset>"` stellt zurück), `timeStart`, `timeEnd`, `smallImage`, `largeImage` (jeweils Objekt mit Bild-Asset und `hoverText`, löschen/zurücksetzen möglich).
- Rate-Limit: Nachrichten innerhalb von 1 s nach der letzten werden verworfen.

### B9. Discord Rich Presence

- Bloxstrap nutzt die Discord-Anwendungs-ID **`1005469189907173486`** [bestätigt, Quelltext `DiscordRichPresence.cs`]. Das ist die **Anwendung von Bloxstrap** (Name und Bilder gehören ihr). **Brickstrap braucht eine eigene Anwendung** im Discord Developer Portal (kostenlos, ID ist öffentlich, kein Geheimnis), sonst steht „Bloxstrap“ im Status.
- Felder: `details` = Spielname, `state` = „by &lt;Ersteller&gt;“ (mit Verifiziert-Zeichen), großes Bild = Spielicon, kleines Bild = Avatar-Kopf des Nutzers mit Text „Playing on &lt;Anzeigename&gt; (@name)“ (nur mit Konto-Anzeige), Startzeit, zwei Knöpfe: **„Join server“** (= Deeplink aus B6) und **„See game page“** (`https://www.roblox.com/games/{PlaceId}`) [bestätigt]. Die Knöpfe lassen sich ausblenden (`HideRPCButtons`).
- Bilder: Discord erlaubt externe Bild-URLs in `large_image`/`small_image`, Bloxstrap setzt direkt die Roblox-Thumbnail-URL (`imageUrl` aus B4) [bestätigt]. Brickstrap muss also keine Bilder bei Discord hochladen.
- Die `discord-rich-presence`-Crate (Plan §3) deckt Details, State, Assets, Timestamps und Buttons ab **[unsicher, nicht geprüft]**.

### B10. FastFlag-Allowlist (Stand)

Der DevForum-Thread war nicht abrufbar. Abgeglichen habe ich mit der Drittliste [LeventGameing/allowlist](https://github.com/LeventGameing/allowlist) (README und `allowlist.json` mit Standardwerten, abgerufen). Ergebnis: **identisch mit der Liste in Plan §5, 18 Flags**. Neu hinzu kommt nur der Typ und der Standardwert:

| Flag | Typ | Standardwert (Drittliste) |
|---|---|---|
| `DFIntCSGLevelOfDetailSwitchingDistance` / `…L12` / `…L23` / `…L34` | Int | 250 / 500 / 750 / 1000 |
| `FFlagHandleAltEnterFullscreenManually` | Bool | true |
| `DFFlagTextureQualityOverrideEnabled` | Bool | false |
| `DFIntTextureQualityOverride` | Int | 3 |
| `FIntDebugForceMSAASamples` | Int | 0 |
| `DFFlagDisableDPIScale` | Bool | false |
| `FFlagDebugGraphicsPreferD3D11` / `…Vulkan` / `…OpenGL` | Bool | false |
| `FFlagDebugSkyGray` | Bool | false |
| `DFFlagDebugPauseVoxelizer` | Bool | false |
| `DFIntDebugFRMQualityLevelOverride` | Int | 0 |
| `FIntFRMMaxGrassDistance` / `FIntFRMMinGrassDistance` | Int | 290 / 100 |
| `FIntGrassMovementReducedMotionFactor` | Int | 5 |

Die Standardwerte sind **[unsicher]** (Drittliste, nicht von Roblox). Nicht als Wahrheit anzeigen. Ein DevForum-Treffer meldet für den 26.01.2026, Roblox arbeite an einem Prozess für Änderungen und habe bis dahin nichts geändert **[Bericht]**. Ob seitdem Flags hinzukamen, ist **nicht geprüft**. Der Launcher muss die Liste als aktualisierbare Datei führen (so schon in CLAUDE.md).

---

## Teil C: Empfehlungen für das Layout (Fenster ca. 1100 × 720)

1. **Linke schmale Seitenleiste (ca. 72 px, ausklappbar auf 220 px)** mit vier Zielen: **Start, Suche, Server, Einstellungen**. Oben ein dauerhafter „Weiterspielen“-Block. Anders als bei der Strap-Familie steht das Spiel im Mittelpunkt, nicht die Einstellungsseiten. (Belegt: Heroic kann die Seitenleiste einklappen.)
2. **Startseite: „Weiterspielen“ als große Karte oben**, darunter horizontale Reihen aus `get-sorts` (Top Trending, Up-and-Coming, zuletzt gespielt). Reihen virtualisieren, Icons erst beim Sichtbarwerden laden. Cache zuerst, dann aktualisieren (CLAUDE.md). Je Karte ein Hover-Knopf „Spielen“ plus „Server wählen“.
3. **Spielseite als Zweispalter:** links Kopf (Icon, Name, Spieler, Stimmen, Ersteller), rechts oder darunter die **Serverliste als Tabelle** mit Spalten **Spieler (x/max), Ping, FPS, Beitreten**, sortierbare Köpfe. Über der Tabelle Chips: „Nicht voll“, „Wenig Spieler“, „Niedriger Ping“. Primärer Knopf oben: **„Bester Server“ (Spielen)**, daneben „Schnellstart ohne Auswahl“. Ein Klick auf „Beitreten“ in der Zeile startet sofort (Muster Steam, belegt für sortierbare Spalten und Filter).
4. **Region erst in Stufe 3:** In der Tabelle eine Spalte „Region“ nur anzeigen, wenn angemeldet. Sonst Hinweis „Region nach dem Beitritt sichtbar“ statt leerer Spalte. Region-Scan mit Fortschritt und Abbrechen. Eigene IP-Bereichsliste statt Drittdienst (B6).
5. **Schnellstart-Leiste unten, immer sichtbar:** Dropdown **Preset (Max FPS, Ausgewogen, Qualität, Eigene)**, Anzeige „FPS-Limit 0, Grafik manuell“, ein großer Knopf **„Spielen“**. Ändern eines Presets wirkt vor dem Start (Prüfsumme, siehe CLAUDE.md), nicht erst nach „Speichern“. Das schließt die Lücke der Strap-Familie, die „Save“ und „Save and Launch“ trennt.
6. **Einstellungen als Seiten mit Live-Wirkung:** Grafik (FastFlags der Allowlist + `GlobalBasicSettings`), Start (Priorität, GPU-Vorgabe), Integration (Discord, Logdatei), Aussehen. Flag-Tabelle im Stil Bloxstrap (Suche, Name | Wert), aber **mit Spalte „Wirkung“**: „wirkt“, „experimentell“ (z. B. `DFIntDebugDynamicRenderKiloPixels`), „wirkungslos (nicht auf der Allowlist)“. Presets als teilbarer Text (JSON oder Base64), nur mit Allowlist-Flags. Allowlist-Hinweis direkt in der Seite, nicht nur als Link.
7. **Statusleiste/Tray statt Extra-Fenster:** Unten rechts ein Chip „Roblox läuft: Spiel X, Server in Region Y, seit 12 min“ aus der Logdatei. Klick öffnet Einladungslink kopieren. Im Tray weiterlaufen (CLAUDE.md). Aktualisieren der Serverliste nur manuell oder mit Zeitgeber (30 bis 60 s), sichtbar „zuletzt aktualisiert vor 20 s“, weil das Ratenlimit unbekannt ist.
8. **Fehlerzustände einplanen:** Beitritt scheitert (Server voll oder weg) → Hinweis plus „Nächsten Server probieren“. API 429 → „Zu viele Anfragen, neuer Versuch in 30 s“ mit Zähler, Daten aus dem Cache bleiben stehen. Undokumentierte Endpunkte (Suche, Explore) bei Ausfall kapseln: Startseite zeigt dann „zuletzt gespielt“ und Favoriten.

---

## Offene Punkte für den Projektinhaber

1. Wenn echte Antwortfelder gewünscht sind (omni-search, explore-api, servers/Public): die Hosts `apis.roblox.com`, `games.roblox.com`, `thumbnails.roblox.com` müssten in der Cloud-Session erlaubt oder die Abrufe lokal auf Windows gemacht werden (je ein `curl`, Ausgabe in `docs/research/` ablegen). Bis dahin sind die Feldnamen in B1, B2, B5 **nicht aus erster Hand**.
2. Eigene Discord-Anwendung anlegen (B9). Name „Brickstrap“, Bilder aus `brand/`.
3. Entscheidung Region: Drittdienst `ipinfo.io` (wie Bloxstrap) oder lokale IP-Liste. Empfehlung lokal.
4. `GlobalBasicSettings`: Mit einer echten Datei von Windows gegenprüfen (Schlüssel, Typen `string`/`Vector2`). Plan §13.2 und CLAUDE.md nennen noch `GraphicsQualityLevel`, in der Vorlage heißt es `SavedQualityLevel`.
5. Plan §12.3 und §15.3 korrigieren: `games?universeIds=` höchstens **50** IDs je Aufruf, verstecktes Limit pro IP beachten.

## Quellen

- Bloxstrap (MIT): [ActivityWatcher.cs](https://raw.githubusercontent.com/bloxstraplabs/bloxstrap/main/Bloxstrap/Integrations/ActivityWatcher.cs), [DiscordRichPresence.cs](https://raw.githubusercontent.com/bloxstraplabs/bloxstrap/main/Bloxstrap/Integrations/DiscordRichPresence.cs), [ActivityData.cs](https://raw.githubusercontent.com/bloxstraplabs/bloxstrap/main/Bloxstrap/Models/Entities/ActivityData.cs), [UniverseDetails.cs](https://raw.githubusercontent.com/bloxstraplabs/bloxstrap/main/Bloxstrap/Models/Entities/UniverseDetails.cs), [GameDetailResponse.cs](https://raw.githubusercontent.com/bloxstraplabs/bloxstrap/main/Bloxstrap/Models/APIs/Roblox/GameDetailResponse.cs), [MainWindow.xaml](https://raw.githubusercontent.com/bloxstraplabs/bloxstrap/main/Bloxstrap/UI/Elements/Settings/MainWindow.xaml), [Strings.resx](https://raw.githubusercontent.com/bloxstraplabs/bloxstrap/main/Bloxstrap/Resources/Strings.resx), [FastFlagManager.cs](https://raw.githubusercontent.com/bloxstraplabs/bloxstrap/main/Bloxstrap/FastFlagManager.cs), [README](https://raw.githubusercontent.com/bloxstraplabs/bloxstrap/main/README.md)
- Fishstrap (MIT): [README](https://raw.githubusercontent.com/fishstrap/fishstrap/main/README.md), [GlobalSettingsManager.cs](https://raw.githubusercontent.com/fishstrap/fishstrap/main/Bloxstrap/GlobalSettingsManager.cs), [GlobalBasicSettings_Template.xml](https://raw.githubusercontent.com/fishstrap/fishstrap/main/Bloxstrap/Resources/GlobalBasicSettings_Template.xml), [MainWindow.xaml](https://raw.githubusercontent.com/fishstrap/fishstrap/main/Bloxstrap/UI/Elements/Settings/MainWindow.xaml)
- [roblox-trends README, „Verified external API behavior“](https://raw.githubusercontent.com/MoulinLouis/roblox-trends/main/README.md) (Messungen Okt. 2026)
- [BTRoblox README](https://raw.githubusercontent.com/AntiBoomz/BTRoblox/master/README.md) (Endpunktliste)
- [Naxumi/roblox-server-finder README](https://raw.githubusercontent.com/Naxumi/roblox-server-finder/main/README.md) (Serverliste, gamejoin, Region)
- [LeventGameing/allowlist](https://raw.githubusercontent.com/LeventGameing/allowlist/main/README.md) und [allowlist.json](https://raw.githubusercontent.com/LeventGameing/allowlist/main/allowlist.json)
- Nur über Suchtreffer (nicht selbst abgerufen): DevForum [Allowlist-Thread](https://devforum.roblox.com/t/allowlist-for-local-client-configuration-via-fast-flags/3966569), [Serverliste-Tutorial](https://devforum.roblox.com/t/getting-a-places-server-list/796653), [700-Server-Grenze](https://devforum.roblox.com/t/unable-to-fetch-more-than-700-servers-servers-using-the-games-v1-api-gamesrobloxcomv1/4294098), [Explore-API ohne Paging](https://devforum.roblox.com/t/api-feature-that-allows-for-infinite-scrolling-on-front-page-pagination-for-apisrobloxcomexplore-apiv1get-sort-content/4040481), [FPS-Cap 240](https://devforum.roblox.com/t/roblox-caps-fps-at-240-regardless-of-globalbasicsettings13xml-framerate-cap/4451048); [Fishstrap-Doku zu FastFlags](https://docs-1w2.pages.dev/manual/fast-flags-client); Steam-Serverbrowser: [TF2-Wiki](https://wiki.teamfortress.com/wiki/ServerBrowser), [CS-Wiki](https://counterstrike.fandom.com/wiki/Server_Browser); Heroic: [gamingonlinux](https://www.gamingonlinux.com/2022/05/heroic-games-launcher-for-epic-a-gog-gets-themes-platform-selection-favourites), [linuxcompatible](https://www.linuxcompatible.org/story/heroic-games-launcher-240-beta-released/)
