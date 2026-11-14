---
name: Brickstrap
description: A plain code sign </>: silver brackets, white slash, black tile.
colors:
  black: "#0B0B0D"
  gray: "#8B9099"
  silver: "#C3C7CD"
  white: "#FFFFFF"
  surface-1: "#121215"
  surface-2: "#18191D"
  surface-3: "#212227"
  surface-4: "#2B2C33"
  line: "#24252B"
  line-strong: "#34353D"
  line-hover: "#43444D"
  text-1: "#F3F4F6"
  text-4: "#62666F"
  primary-hover: "#E4E6EA"
  live: "#5AD08B"
  warning: "#E2B34A"
  warning-surface: "#29220F"
  error: "#F0525A"
  error-surface: "#2A1416"
typography:
  wordmark:
    fontFamily: "Archivo, sans-serif"
    fontWeight: 850
    letterSpacing: "-0.03em"
    fontVariation: "'wght' 850, 'wdth' 118"
  headline:
    fontFamily: "Archivo, Segoe UI Variable Text, Segoe UI, system-ui, sans-serif"
    fontSize: "26px"
    fontWeight: 800
    lineHeight: 1.15
    letterSpacing: "-0.02em"
    fontVariation: "'wdth' 112"
  title:
    fontFamily: "Archivo, Segoe UI Variable Text, Segoe UI, system-ui, sans-serif"
    fontSize: "17px"
    fontWeight: 700
    lineHeight: 1.25
    letterSpacing: "-0.01em"
    fontVariation: "'wdth' 106"
  subtitle:
    fontFamily: "Archivo, Segoe UI Variable Text, Segoe UI, system-ui, sans-serif"
    fontSize: "14px"
    fontWeight: 700
    lineHeight: 1.3
    letterSpacing: "-0.01em"
  body:
    fontFamily: "Archivo, Segoe UI Variable Text, Segoe UI, system-ui, sans-serif"
    fontSize: "14px"
    fontWeight: 400
    lineHeight: 1.45
  label:
    fontFamily: "Archivo, Segoe UI Variable Text, Segoe UI, system-ui, sans-serif"
    fontSize: "14px"
    fontWeight: 600
    lineHeight: 1.45
  hint:
    fontFamily: "Archivo, Segoe UI Variable Text, Segoe UI, system-ui, sans-serif"
    fontSize: "13px"
    fontWeight: 400
    lineHeight: 1.45
  caption:
    fontFamily: "Archivo, Segoe UI Variable Text, Segoe UI, system-ui, sans-serif"
    fontSize: "12px"
    fontWeight: 600
    lineHeight: 1.45
  data:
    fontFamily: "Archivo, Segoe UI Variable Text, Segoe UI, system-ui, sans-serif"
    fontSize: "14px"
    fontWeight: 400
    fontFeature: "'tnum' 1"
  mono:
    fontFamily: "Cascadia Mono, Consolas, ui-monospace, monospace"
    fontSize: "12.5px"
    fontWeight: 400
rounded:
  tile: "21.875%"
  sm: "6px"
  md: "8px"
  lg: "12px"
  control-lg: "10px"
  xl: "16px"
spacing:
  row-y: "14px"
  row-x: "18px"
  tile-gap: "16px"
  section-gap: "28px"
  page-x: "28px"
components:
  app-icon:
    backgroundColor: "{colors.black}"
    textColor: "{colors.silver}"
    rounded: "{rounded.tile}"
  lockup-dark:
    backgroundColor: "{colors.black}"
    textColor: "{colors.white}"
  lockup-light-mono:
    backgroundColor: "{colors.white}"
    textColor: "{colors.black}"
  button-default:
    backgroundColor: "{colors.surface-3}"
    textColor: "{colors.text-1}"
    typography: "{typography.label}"
    rounded: "{rounded.md}"
    padding: "0 14px"
    height: "34px"
  button-default-hover:
    backgroundColor: "{colors.surface-4}"
  button-primary:
    backgroundColor: "{colors.white}"
    textColor: "{colors.black}"
    typography: "{typography.label}"
    rounded: "{rounded.md}"
    padding: "0 14px"
    height: "34px"
  button-primary-hover:
    backgroundColor: "{colors.primary-hover}"
  button-flach:
    backgroundColor: "transparent"
    textColor: "{colors.silver}"
    rounded: "{rounded.md}"
    height: "34px"
  button-flach-hover:
    backgroundColor: "{colors.surface-3}"
    textColor: "{colors.text-1}"
  button-klein:
    padding: "0 10px"
    height: "28px"
  button-gross:
    padding: "0 20px"
    height: "42px"
  button-rund:
    padding: "0"
    width: "34px"
    height: "34px"
  field:
    backgroundColor: "{colors.surface-1}"
    textColor: "{colors.text-1}"
    typography: "{typography.data}"
    rounded: "{rounded.md}"
    padding: "0 12px"
    height: "34px"
  field-focus:
    backgroundColor: "{colors.surface-2}"
  switch-off:
    backgroundColor: "{colors.black}"
    textColor: "{colors.silver}"
    width: "40px"
    height: "22px"
  switch-on:
    backgroundColor: "{colors.white}"
    textColor: "{colors.black}"
    width: "40px"
    height: "22px"
  chip:
    backgroundColor: "transparent"
    textColor: "{colors.silver}"
    padding: "0 12px"
    height: "30px"
  chip-selected:
    backgroundColor: "{colors.text-1}"
    textColor: "{colors.black}"
  marke:
    backgroundColor: "{colors.surface-3}"
    textColor: "{colors.silver}"
    padding: "0 7px"
    height: "20px"
  marke-warning:
    backgroundColor: "{colors.warning-surface}"
    textColor: "{colors.warning}"
  settings-card:
    backgroundColor: "{colors.surface-1}"
    rounded: "{rounded.lg}"
  settings-row:
    padding: "14px 16px 14px 18px"
  server-table:
    backgroundColor: "{colors.surface-1}"
    rounded: "{rounded.lg}"
  server-row:
    padding: "0 12px 0 18px"
    height: "48px"
  server-row-hover:
    backgroundColor: "{colors.surface-2}"
  game-tile:
    backgroundColor: "{colors.surface-1}"
    rounded: "{rounded.lg}"
  game-tile-hover:
    backgroundColor: "{colors.surface-2}"
  game-card:
    rounded: "{rounded.lg}"
    width: "152px"
  rail:
    backgroundColor: "{colors.surface-1}"
    width: "232px"
  rail-collapsed:
    width: "68px"
  rail-item:
    textColor: "{colors.silver}"
    rounded: "{rounded.md}"
    padding: "0 10px"
    height: "38px"
  rail-item-active:
    backgroundColor: "{colors.surface-3}"
    textColor: "{colors.text-1}"
  dock:
    backgroundColor: "{colors.surface-1}"
    height: "52px"
  toast:
    backgroundColor: "{colors.surface-3}"
    textColor: "{colors.text-1}"
    padding: "11px 14px"
  hinweis:
    backgroundColor: "{colors.surface-1}"
    rounded: "{rounded.lg}"
    padding: "20px"
  hinweis-error-symbol:
    backgroundColor: "{colors.error-surface}"
    textColor: "{colors.error}"
  skeleton:
    backgroundColor: "{colors.surface-2}"
    rounded: "{rounded.md}"
  chip-small:
    textColor: "{colors.silver}"
    padding: "0 10px"
    height: "26px"
  settings-row-surface:
    backgroundColor: "{colors.surface-1}"
    rounded: "{rounded.control-lg}"
    padding: "14px 16px 14px 18px"
  settings-row-surface-hover:
    backgroundColor: "{colors.surface-2}"
  side-panel:
    backgroundColor: "{colors.black}"
    width: "296px"
  side-block:
    backgroundColor: "{colors.surface-1}"
    rounded: "{rounded.lg}"
    padding: "14px"
  player-row:
    textColor: "{colors.text-1}"
    rounded: "{rounded.lg}"
    padding: "8px 10px"
  player-row-hover:
    backgroundColor: "{colors.surface-1}"
  profile-avatar:
    backgroundColor: "{colors.surface-1}"
    rounded: "{rounded.lg}"
    width: "260px"
  item-row:
    rounded: "{rounded.md}"
    padding: "7px 8px"
  item-row-hover:
    backgroundColor: "{colors.surface-1}"
  progress-track:
    backgroundColor: "{colors.surface-3}"
    textColor: "{colors.silver}"
    height: "6px"
  game-summary:
    backgroundColor: "{colors.surface-1}"
    rounded: "{rounded.lg}"
    padding: "12px 16px 12px 12px"
  dropdown:
    backgroundColor: "{colors.surface-3}"
    rounded: "{rounded.control-lg}"
    padding: "4px"
  dialog:
    backgroundColor: "{colors.surface-1}"
    rounded: "{rounded.xl}"
    padding: "28px"
    width: "520px"
---

# Design System: Brickstrap

Recorded from the shipped logo system (`brand/build_logo.py`, `brand/logo/`) and from the shipped launcher UI (`app/src/app.css`, `app/src/ui/`, `app/src/pages/`). The identity sections are unchanged. The UI sections describe the build as it stands, not the plan (refreshed 2026-11-14: player search, profile, inventory, side panel, dialogs).

## Overview

**Creative North Star: "The Launcher Standard, Played Straight"**

The mark is a plain code sign, `</>`: silver chevrons and a white slash on a black rounded tile. It says "tool for people who tinker" at a glance. The identity is achromatic and flat: neutrals only, no hue, no gradients, glow or shadow. It is pure vector and scales cleanly to any size.

The launcher UI keeps the category's proven layout without irony: a navigation rail on the left, a game page with one big play button, settings in grouped cards with switches, a status dock at the bottom. It sits at the level of Windows 11 Settings, Discord and Steam, built from the four logo neutrals stretched into a short ladder of greys. Hierarchy comes from surface brightness, never from shadow or gradient. The page is dense, quiet and dark, so the white primary button and the few signal colours carry all the emphasis.

**Key Characteristics:**
- Flat black ground (#0B0B0D) with four tonal steps above it.
- One white primary with black text per view; everything else is grey.
- Signal colours appear only where they carry meaning: online, warning, error.
- One family, Archivo, self-hosted; headings slightly wide, body at normal width.
- Short, transform-only motion that respects reduced motion.

## Colors

Achromatic by default: black ground, grey surfaces, silver and white text, plus three small signal colours for state.

### Primary
- **White** (#FFFFFF): the slash, and the wordmark on dark grounds. White on black is 19.66:1. In the UI, the one primary button fill (`--primaer`), with black text on it (`--auf-primaer`).
- **Pressed White** (primary-hover): the primary button's hover fill. Black on it is 15.74:1.

### Neutral
- **Black** (#0B0B0D): the tile, the dark ground, and the only ink on light grounds. In the UI, the window ground (`--s0`) and the text on the primary button.
- **Silver** (#C3C7CD): the chevrons. Silver on black is 11.58:1. In the UI, secondary text, icons, the flat button label and the server fill bars (`--t2`).
- **Gray** (#8B9099): reserved for UI (secondary text, dividers). It is not used in the mark. Gray on black is 6.13:1. In the UI, hints, table headers, meta lines and placeholders (`--t3`).
- **Rail Grey** (surface-1, `--s1`): rail, dock, cards, server table, empty-state boxes, fields.
- **Card Hover** (surface-2, `--s2`): row and tile hover, focused field, skeleton blocks.
- **Raised Grey** (surface-3, `--s3`): default button, active rail item, toast, badges, progress track.
- **Top Grey** (surface-4, `--s4`): default button hover, pressed toggle button, unlit signal bars.
- **Hairline** (line, `--linie`) and **Strong Hairline** (line-strong, `--linie-stark`): 1 px dividers between rows and around the rail and dock; field and chip borders. **Hover Hairline** (line-hover) brightens a field border on hover.
- **Near White** (text-1, `--t1`): primary text, active states, selected chips, lit signal bars, focus rings.
- **Ash** (text-4, `--t4`): decoration only (image placeholder initials, a full server's bar). Ash is 3.25:1 on Rail Grey, too low for text.

### Signal colours (user-confirmed 2026-11-13)
- **Live Green** (live): online and playing. Only as a small dot (6–8 px) next to a player count or in the dock lamp.
- **Warning Amber** (warning on warning-surface): experimental flags, "status off" notices, the dock lamp in a warning state. 8.11:1 on its surface.
- **Error Red** (error on error-surface): errors only: error toasts, error notices, inline validation text. 5.01:1 on its surface, 5.67:1 on black.

### Named Rules
**The Four Neutrals Rule.** The identity uses black, gray, silver and white only. Fills are flat. No gradients, glow or shadow.

**The Black Ground Rule.** The two-tone mark (silver plus white) sits only on black, inside the tile or on a black ground. On light grounds, use the single-ink black mark.

**The Outlined Wordmark Rule.** The wordmark ships as outlines. Never retype it in a live font, uppercase it, or change its tracking.

**The Meaning-Only Signal Rule.** Green, amber and red appear only where a state needs them: green for online or playing, amber for experimental or warning, red for errors. Never as decoration, accent or brand colour.

**The Brightness Ladder Rule.** Depth is one step up the surface ladder (s0 → s1 → s2 → s3 → s4). A hovered or raised element moves one step brighter; it does not gain a shadow.

## Typography

**UI Font:** Archivo variable (weights 100–900, widths 62–125 %), self-hosted as WOFF2, with Segoe UI Variable Text, Segoe UI and system-ui as fallbacks.
**Mono Font:** Cascadia Mono (with Consolas, ui-monospace).

**Character:** one grotesque for everything. Headings widen a little (106–112 %) and get heavy; body text stays at normal width so lists read cleanly.

- **Wordmark:** "brickstrap" in lowercase Archivo, wght 850 / wdth 118, tracking −0.03em, outlined.

### Hierarchy
- **Headline** (800, 26 px, 1.15, width 112 %, −0.02em): page titles. The game page raises its title to 30 px (24 px below 1100 px).
- **Title** (700, 17 px, 1.25, width 106 %, −0.01em): section headings above cards and rows.
- **Subtitle** (700, 14 px, 1.3): small headings inside notices.
- **Body** (400, 14 px, 1.45): the base size for everything. Notices cap prose at 60ch, hints at 64ch, leads at 68ch.
- **Label** (600, 14 px): buttons, setting labels, game names. Rail items use 500, the active item 600.
- **Hint** (400, 13 px, Gray) and meta lines (12.5 px, Gray): explanations under labels, player counts, dock sub-lines.
- **Caption** (600, 12 px, Gray): table headers, the rail's "Angepinnt" group title, value labels on the game page. Sentence case, no tracking.
- **Data** (tabular numerals): every count, ping, time and number field (`.zahl`, `.feld`).
- **Mono** (12.5 px; 11.5 px for flag names): server IDs, FastFlag names, setting keys, file paths, JSON import.

### Named Rules
**The Tabular Data Rule.** Numbers that sit in lists or change live use tabular numerals, so columns do not jitter.

**The Mono-For-Identifiers Rule.** Monospace marks machine identifiers only: server IDs, flag names, setting keys, file paths. Never for headings, labels or prose.

## Layout

A fixed desktop shell: a grid of rail plus main column over a full-width dock.

- **Rail:** 232 px wide. Below a window width of 1100 px it collapses to 68 px (icons only, names as tooltips). A click on the collapse button overrides the automatic state until the window crosses 1100 px again. The rail switches between the two widths without animating its width.
- **Dock:** 52 px tall, spans the full window width at the bottom.
- **Header:** 60 px, holding the search field (max 620 px, 38 px tall).
- **Content:** page padding 8 px top, 28 px sides, 40 px bottom; the page caps at 1600 px. Settings, FastFlags and list pages cap at 1180 px (`--inhalt-max`).
- **Side panel:** from a window width of 1360 px a 296 px column opens on the right (account, friends online, what is running, quick start, playtime). Below 1360 px it disappears and the account moves into the rail.
- **Rhythm:** 28 px between sections (36 px on the start page), 10 px from a section heading to its card, 16 px between tiles, 14 × 18 px padding inside setting rows.
- **Grids:** pinned tiles fill `auto-fill, minmax(236px, 1fr)`. Game card rows scroll sideways in 152 px columns with snap. FastFlag presets run in five columns, three below 1050 px. The game page header is a two-column grid (image 300–440 px, info). The profile header uses the same grid with a square 260 px avatar (200 px below 1100 px). Player results fill `auto-fill, minmax(280px, 1fr)`, inventory items `minmax(240px, 1fr)`, worn items `minmax(112px, 1fr)`.

## Elevation & Depth

Flat by default. Depth comes from tonal layering on the brightness ladder: ground (s0), rail and cards (s1), hover (s2), raised controls (s3), pressed (s4). Lines are 1 px hairlines between rows and along the rail and dock edges. Images and preset cards carry a 1 px inset light hairline (`inset 0 0 0 1px #ffffff12`) that works as a border; selection and hover thicken it to 2 px in Silver or Near White.

### Named Rules
**The No-Shadow Rule.** Surfaces do not cast shadows. Separation comes from a brighter surface step or a hairline.

**The Floating-Layer Exception.** Only layers that float above the page carry one soft drop shadow: the search suggestions (`0 12px 28px #00000073`) and the update dialog (`0 24px 64px -12px #000c`, over a Black scrim at 80 %). Anything that sits in the page flow stays flat.

## Shapes

Gently rounded, never pill-shaped containers.

- **Small** (6 px): pinned-game thumbnails in the rail, the pin badge, focus-ring corners.
- **Standard** (8 px): buttons, fields, rail items, skeleton blocks.
- **Large** (12 px): cards, the server table, game tiles and cards, notices, the save bar, side-panel blocks, the profile avatar, player rows.
- **Control large** (10 px): big buttons, the header search field, settings rows, dropdown lists.
- **Extra large** (16 px): the update dialog and the sign-in card.
- **Full round:** switches, chips (half their height), status dots, player avatars in lists.
- **Badges:** 4 px corners.
- **Icons:** own SVG set on a 24 grid, stroke 1.75, round caps and joins like the logo.

### Logo geometry
- **Canvas:** 64 units, with a tile corner radius of 14 (21.875%).
- **Strokes:** width 6, round caps and joins.
- **Chevrons:** (22,21) → (10,32) → (22,43), mirrored to (42,21) → (54,32) → (42,43).
- **Slash:** (28.5,45) → (35.5,19).
- **Lockups:** the wordmark ascender sits at y 19 and the baseline at y 45. The gap between mark and wordmark is 16 units.

## Components

### Identity
- **App icon:** `brand/logo/brickstrap-icon.svg`. PNG exports run from 16 to 2048 px, and `brickstrap.ico` holds frames from 16 to 256. Every size is rendered from the vector.
- **Lockups:**
  - light: black tile plus black wordmark
  - light-mono: all black
  - dark: mark without a tile plus white wordmark (`-dark-bg` adds a black ground)
- **Single ink:** `mark-dark` (two-tone, for dark grounds only), `mark-black`, `mark-silver`, `mark-white`; `wordmark-black`, `wordmark-white`.

### Buttons
Solid, quiet and quick.
- **Shape:** standard corners (8 px), 34 px tall, 14 px side padding, weight 600, icon gap 8 px.
- **Default:** Raised Grey with Near White text; hover one step brighter (Top Grey).
- **Primary:** White fill, Black text; hover Pressed White. The play button, "Roblox starten" in the dock, "Speichern".
- **Flach:** transparent with Silver text; hover Raised Grey and Near White text. Toolbar and row actions.
- **Klein:** 28 px tall, 10 px padding, 13 px text. **Gross:** 42 px tall, 20 px padding, 15 px text, 10 px corners. **Rund:** square icon button at the current height.
- **Toggle:** `aria-pressed="true"` holds Top Grey.
- **States:** press scales to 0.97 (140 ms, enter curve); disabled drops to 45 % opacity.

### Inputs / Fields
- **Style:** Rail Grey fill, 1 px Strong Hairline border, 8 px corners, 34 px tall, tabular numerals.
- **Hover:** border to Hover Hairline. **Focus:** border Silver, fill Card Hover, no glow.
- **Select:** the same field with a Silver chevron on the right; options on Card Hover.
- **Search (header):** the field pattern at 38 px with 10 px corners, a search icon and a keyboard hint (`kbd` on Raised Grey).

### Switch (Schalter)
40 × 22 px track. Off: black track, 1 px grey border, Silver knob. On: White track, Black knob slid 18 px. The knob moves with transform only (180 ms, enter curve). Focus ring on the track.

### Chips
- **Style:** 30 px tall, full round, 1 px Strong Hairline border, transparent, Silver 13 px text at weight 500.
- **Hover:** Card Hover fill, Near White text. **Selected:** Near White fill, Black text. Used as sort and filter choices, and as a radio group for search mode (Spiele | Spieler) and inventory categories. **Small chip:** 26 px tall, 10 px padding, 12.5 px text, for secondary filters (Alle, Erhalten, Fehlen).

### Badges (Marke)
20 px tall, 4 px corners, Raised Grey with Silver 11 px text at weight 600. The warning variant uses Warning Amber on its surface ("Experimentell", "Status aus: keine ID").

### Settings rows
Each row is its own Rail Grey surface with 10 px corners, stacked with a 4 px gap like Windows 11 Settings; hover moves the row to Card Hover. Each row puts a label (600) and a Gray hint on the left and the control (switch, select, number field, flat icon button) on the right. Row padding 14 × 16 × 14 × 18 px. Used for app settings, Roblox settings and FastFlags; FastFlag rows show the flag name in mono below the hint.

### Server table
The signature list of the game page.
- **Container:** Rail Grey, 12 px corners, rows of at least 48 px split by hairlines.
- **Header:** sticky at the top, caption style in Gray on Rail Grey.
- **Columns:** player count in 15 px bold, load as a fill bar, server ping with three signal bars, FPS, server ID (mono, copy icon on row hover), join action.
- **Fill bar:** 6 px Raised Grey track, Silver fill grown with `scaleX` (420 ms, staggered 18 ms per row). A full server turns its count, ping and bar grey.
- **Signal bars:** three bars (5, 8, 12 px); lit bars Near White, unlit Top Grey.
- **Row hover:** Card Hover. The "Beitreten" button turns White with Black text on the hovered row only.
- **Refreshing:** the table dims to 60 % while new data loads, so nothing jumps.

### Game tile and game card
- **Tile (start page, pinned):** Rail Grey block, 12 px corners, 16:9 image on top, a footer with name (700, 15 px), a green dot plus player count, and a small "Spielen" button. Hover moves the block to Card Hover.
- **Card (rows, search):** a square image in a 12 px frame with a light inset hairline, name below (two lines max), meta line in Gray. On hover the frame lifts 2 px and its hairline turns Silver; on press it scales to 0.98. A pinned card shows a pin badge in the image corner.
- **Images:** fade in over 220 ms on Card Hover; until then the game's initials in Ash.

### Rail (Leiste)
Rail Grey column with a hairline on the right. Logo mark and wordmark on top, then the main destinations, then a scrolling list of pinned games, settings at the bottom behind a hairline. Items are 38 px, Silver at weight 500; hover Card Hover and Near White; active Raised Grey, weight 600, with a 3 px Near White bar on the left edge. Collapsed, only icons and thumbnails remain.

### Dock
52 px Rail Grey bar with a hairline on top. Left: a status lamp (8 px; Ash when idle, Live Green while Roblox runs, Warning Amber on a warning), a title and a Gray sub-line, plus flat small actions. Right: the FastFlag preset select and the primary "Roblox starten" button, which becomes a disabled default button reading "Roblox läuft" while a game is open.

### Side panel (RechteSpalte)
296 px Black column with a hairline on its left edge, 16 × 14 px padding, blocks 10 px apart. Each block is Rail Grey with 12 px corners and 14 px padding, headed by a 13 px Silver title at normal width. List rows are 5 × 6 px, hover Card Hover. Friends show a 32 px round avatar with a 10 px status dot ringed in Rail Grey (Live Green while playing, Gray when only online). Playtime uses 4 px tracks with a Silver `scaleX` fill.

### Account (Konto)
Avatar (36 px, 28 px compact), display name at 600 and @name in Gray. It is a button: hover Card Hover, click opens the own profile.

### Player row
The search result for people. 52 px round avatar on Card Hover with an inset hairline, display name 600 at 15 px, @name in Gray 13 px, a 15 px Silver check for verified accounts. The whole row is a button with 12 px corners; hover Rail Grey, and a chevron slides in from the right (opacity plus 4 px `translateX`, 160 ms). Initials in Ash until the avatar fades in.

### Profile header
Same two-column grid as the game page. Left: a square Rail Grey frame with the full-body avatar, `object-fit: contain`, rising 6 px while it fades in. Right: name as a 30 px headline with the verified check, @name in Silver plus badges ("Das bist du", "Gesperrt" in warning), a status line with an 8 px dot (Live Green playing, Silver online, Ash offline), four values in the game page's value style (Freunde, Follower, Folgt, Dabei seit), then actions: "Beitreten" as the one primary only when the server is joinable, "Spiel ansehen" default, copy and browser as flat round icons.

### Inventory
Category chips, then a toolbar with a game select (optgroups: games from loaded badges with counts, then pinned games) and a filter field with a search icon. Items are rows of a 48 px image (8 px corners, inset hairline), name at 600 and a Gray meta line; hover Rail Grey. Under "Alle Spiele" the meta line is the game name as a quiet link that applies the filter.
- **Game summary:** a Rail Grey bar with the game icon (40 px), name and chevron (opens the game), "7 von 12 erhalten" with a 140 × 6 px progress track, and small chips Alle, Erhalten, Fehlen on the right.
- **Missing badges:** image at 40 % opacity and greyscale, name in Gray, meta "Fehlt · 2,3 % der Spieler". Earned ones read "Erhalten am 3.11.2026" and sort first.
- **Private:** a compact notice instead of tabs.

### Progress track
6 px Raised Grey track (4 px in the side panel and playtime list), Silver fill grown with `scaleX` from the left over 420 ms. The update download uses a White fill with a linear 120 ms transition.

### Dropdown list (Auswahl, search suggestions)
Raised Grey list (Card Hover for suggestions) with a strong hairline, 10 px corners, 4–5 px padding, rows 34 px with 6 px corners; the active row is Top Grey. Opens with a 150 ms scale from 0.96 (suggestions: 3 px rise, 120 ms). Arrow keys, Enter and Escape work.

### Dialog
Update dialog: centred over a Black scrim at 80 %, Rail Grey with a strong hairline, 16 px corners, 28 px padding, max 520 px. The 48 px app icon, a 24 px title, release notes on Card Hover, a progress track while downloading, the primary action stretched next to "Später". The sign-in page reuses the card shape (460 px, 32 px padding) on the Black ground.

### Slideshow (Diashow)
The game page image cycles through screenshots with a 600 ms crossfade and a slow 1.02 → 1 scale. Arrows and actions appear on hover on Black scrim circles; dots sit in a scrim pill at the bottom, the current one stretched to 16 px.

### Toast
Bottom right above the dock, up to 380 px wide, Raised Grey with 10 px corners, icon in Silver (Error Red for errors), title at 600, text in Silver 13 px. Enters with a fade and 8 px rise from 0.97 scale (220 ms).

### Notice (Hinweis)
The empty and error state. Rail Grey box, 12 px corners, 20 px padding (14 × 16 px compact), up to 560 px wide. A 36 px icon square on Raised Grey (error: Error Red on its surface), a subtitle, a Gray explanation and optional buttons. Every empty or failed list uses it, with copy that says what happened and what to do next.

### Skeleton
Card Hover blocks in the shape of the coming content, 8 px or 12 px corners, pulsing opacity to 55 % over 1.6 s.

### Motion
- **Enter curve:** `cubic-bezier(0.22, 1, 0.36, 1)` for anything that arrives or moves; `cubic-bezier(0.25, 1, 0.5, 1)` is defined for movement.
- **Durations:** 100–160 ms for colour changes on hover, 180–250 ms for entrances (page 180 ms, toast 220 ms), 420 ms for fill bars.
- **Properties:** only `transform` and `opacity` move; colour states fade. Fill bars grow with `scaleX`, never with `width`.
- **Reduced motion:** all animations and transitions shrink to 1 ms.

## Do's and Don'ts

### Do:
- **Do** keep the slash white and the chevrons silver on the two-tone mark.
- **Do** use the black mark on light grounds.
- **Do** use one white primary button per view; every other action is default or flach.
- **Do** step one surface brighter for hover and raised states (s1 → s2 → s3 → s4).
- **Do** use Live Green only for online or playing, Warning Amber only for experimental or warning, Error Red only for errors.
- **Do** set counts, pings and times in tabular numerals, and identifiers (server IDs, flag names, keys, paths) in mono.
- **Do** animate only `transform` and `opacity`, 120–250 ms on the enter curve, and grow bars with `scaleX`.
- **Do** write UI copy in German, du form, short active sentences, with correct umlauts and no dashes as a stylistic device.
- **Do** give every empty or failed list a notice that says what happened and what to do.
- **Do** make a whole row the click target (player rows, friends, account) and let a chevron or hover step show it.

### Don't:
- **Don't** imitate Roblox's logo, or use "Roblox" or "Blox" in any name.
- **Don't** add colour, gradients or glow, and no shadow outside floating layers.
- **Don't** tilt the mark or change its stroke weight.
- **Don't** use glass, blur, Mica or neon effects.
- **Don't** use Ash (#62666F) for readable text.
- **Don't** animate width, height or layout; the rail switches between two fixed widths.
- **Don't** use signal colours as decoration or to make a screen livelier.
