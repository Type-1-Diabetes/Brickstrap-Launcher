# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

(Desktop app for Windows built with Rust + Tauri; the UI is rendered in WebView2, so the design language is web.)

## Users

Power users and fans of Roblox who already know alternative launchers such as Bloxstrap or Fishstrap. They want control over servers, FastFlags, mods and launch behaviour, and they want to get into a game or a specific server with as little friction as possible.

## Product Purpose

Brickstrap is a free, open-source launcher for Roblox on Windows. It starts the official Roblox client from the outside (deeplinks, optionally its own bootstrapper) and adds a fast home screen, game search, a server picker, settings editors and integrations. Success: players reach the server they want faster than with the official app or other launchers, without any risk to their account.

## Positioning

The combination: fast and lightweight (Rust/Tauri, instant start, low RAM), the best server finder (region, small/new servers, follow friends, one-click join) and rule-compliant by design (open source, no injection, login optional).

## Operating Context

- Used on Windows desktops and laptops, often right before playing; sits in the system tray while Roblox runs.
- Lives next to the official Roblox app and Discord.
- Research and plan: `docs/research/roblox-launcher-rust.md`.

## Capabilities and Constraints

- Name: **Brickstrap**. Must never contain "Roblox" or "Blox"; must not imitate Roblox's logo, wordmark or tilted-square mark (trademark).
- Not affiliated with Roblox Corporation.
- No code injection, no memory access to the client.
- Undecided: licence (MIT likely), final feature scope of v1.

## Brand Commitments

- Name: Brickstrap (confirmed 2026-11-13).
- Logo palette (user-pinned 2026-11-13): black, gray, silver, white only.
- Logo subject (user-pinned 2026-11-13): a debug/coding icon, a plain code sign </> (silver brackets, white slash, black tile); final 2026-11-13.

## Evidence on Hand

None yet: no users, downloads, testimonials or benchmarks of Brickstrap itself. Do not fabricate any.

## Product Principles

1. Speed is a feature: every interaction should feel instant.
2. Control without clutter: depth for power users, sensible defaults first.
3. Clean by design: nothing that could get an account banned.
4. Open and transparent: source, behaviour and limits are visible.
