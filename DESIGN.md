---
name: Brickstrap
description: A plain code sign </>: silver brackets, white slash, black tile.
colors:
  black: "#0B0B0D"
  gray: "#8B9099"
  silver: "#C3C7CD"
  white: "#FFFFFF"
typography:
  wordmark:
    fontFamily: "Archivo, sans-serif"
    fontWeight: 850
    letterSpacing: "-0.03em"
    fontVariation: "'wght' 850, 'wdth' 118"
rounded:
  tile: "21.875%"
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
---

# Design System: Brickstrap

Recorded from the shipped logo system (`brand/build_logo.py`, `brand/logo/`). Only the identity is proven. Anything the future Tauri launcher UI has not yet shown is marked **[open]**.

## Overview

The mark is a plain code sign, `</>`: silver chevrons and a white slash on a black rounded tile. It says "tool for people who tinker" at a glance. The identity is achromatic and flat: neutrals only, no hue, no gradients, glow or shadow. It is pure vector and scales cleanly to any size.

## Colors

- **Black** (#0B0B0D): the tile, the dark ground, and the only ink on light grounds.
- **Silver** (#C3C7CD): the chevrons. Silver on black is 11.58:1.
- **White** (#FFFFFF): the slash, and the wordmark on dark grounds. White on black is 19.66:1.
- **Gray** (#8B9099): reserved for UI (secondary text, dividers). It is not used in the mark. Gray on black is 6.13:1.
- UI surfaces and other neutrals are **[open]**.

### Named Rules
**The Four Neutrals Rule.** The identity uses black, gray, silver and white only. Fills are flat. No gradients, glow or shadow.

**The Black Ground Rule.** The two-tone mark (silver plus white) sits only on black, inside the tile or on a black ground. On light grounds, use the single-ink black mark.

**The Outlined Wordmark Rule.** The wordmark ships as outlines. Never retype it in a live font, uppercase it, or change its tracking.

## Typography

- **Wordmark:** "brickstrap" in lowercase Archivo, wght 850 / wdth 118, tracking −0.03em, outlined.
- **UI type:** heavy, wide Archivo suits headlines. Body text and labels are **[open]**.

## Geometry

- **Canvas:** 64 units, with a tile corner radius of 14 (21.875%).
- **Strokes:** width 6, round caps and joins.
- **Chevrons:** (22,21) → (10,32) → (22,43), mirrored to (42,21) → (54,32) → (42,43).
- **Slash:** (28.5,45) → (35.5,19).
- **Lockups:** the wordmark ascender sits at y 19 and the baseline at y 45. The gap between mark and wordmark is 16 units.

## Components

- **App icon:** `brand/logo/brickstrap-icon.svg`. PNG exports run from 16 to 2048 px, and `brickstrap.ico` holds frames from 16 to 256. Every size is rendered from the vector.
- **Lockups:**
  - light: black tile plus black wordmark
  - light-mono: all black
  - dark: mark without a tile plus white wordmark (`-dark-bg` adds a black ground)
- **Single ink:** `mark-dark` (two-tone, for dark grounds only), `mark-black`, `mark-silver`, `mark-white`; `wordmark-black`, `wordmark-white`.
- **UI components:** **[open]**.

## Do's and Don'ts

- **Do** keep the slash white and the chevrons silver on the two-tone mark.
- **Do** use the black mark on light grounds.
- **Don't** imitate Roblox's logo, or use "Roblox" or "Blox" in any name.
- **Don't** add colour, gradients, glow or shadow.
- **Don't** tilt the mark or change its stroke weight.
