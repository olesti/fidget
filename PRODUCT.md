# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Users

General everyday users on macOS, Windows and Linux who want a few tiny, useful tools one click away without learning a new app. Typical situations: staying "active" during a call or long task, keeping the machine awake while something runs. No technical background is assumed.

## Product Purpose

Fidget is a small menu-bar / system-tray utility that collects tiny tools in one place. It runs quietly in the background with no Dock icon and no main window on macOS. Current tools: **Jiggle** (random mouse nudges so the system doesn't see you as idle) and **Keep Awake** (prevents system and display sleep), plus Pause All timers, global shortcuts and launch at login. Success is that the user sets it and forgets it, and opens it only to toggle or check status.

## Positioning

One modular, cross-platform, open-source tray home for many tiny tools, not a single-purpose utility. Each tool is a `Feature` that registers itself, so the tray, toggles and settings grow automatically. Planned ideas: Break Reminder, Focus Timer, Clipboard History, Auto Clicker, Quick Notes, Schedule.

## Operating Context

- Primary surface is the tray icon and its panel: Keep Awake timers, pause, quick actions. Click opens the panel on macOS/Windows; Linux uses a native menu.
- Secondary surface is a Settings window with General, Features, Shortcuts, Permissions and About.
- The tray icon reflects state: filled means something is active, ring means idle.
- macOS needs the Accessibility permission for Jiggle. Wayland support is untested.
- Distribution is via GitHub Releases (`.dmg`, `.deb`, others), with the Tauri updater pointed at `latest.json`.

## Capabilities and Constraints

- Stack: Tauri 2 (Rust backend), Svelte 5, Tailwind 4, Vite. Platform-specific code lives only under `src-tauri/src/platform/`.
- Single instance. Quit only from the tray menu.
- macOS builds are Apple Silicon only, minimum macOS 11.
- Jiggle: random interval (default 30-120 s), `always` or `idle_only`, styles `subtle` / `natural` / `zen`, optional schedule (time window, weekdays only).
- Default shortcuts: Keep Awake `Cmd/Ctrl+Alt+K`, Jiggle `Cmd/Ctrl+Alt+J`.
- Auto-update is configured but not confirmed as shipped.

## Brand Commitments

- Name: Fidget. Quiet, small, unobtrusive. It must never feel like it demands attention.
- Bilingual: every user-facing string ships in English and Türkçe.
- Light and dark themes are both first-class and follow the system.
- No telemetry, no accounts. MIT licensed.

## Evidence on Hand

- README.md documents features, roadmap and run instructions.
- App icons in `src-tauri/icons/`.
- No testimonials, user counts, benchmarks or press exist. Future work must not invent any.

## Product Principles

1. **Stay out of the way.** The tray is home. No Dock icon or main window on macOS, and nothing that interrupts.
2. **Glanceable state.** The user can tell whether anything is running without opening a window.
3. **Small tools, one shape.** Every new feature fits the same toggle/status/settings pattern instead of adding its own UI paradigm.
4. **Respect the OS.** Follow native conventions (tray behavior, permissions, themes) per platform.
5. **Private by default.** Nothing leaves the machine.

## Accessibility & Inclusion

- English and Türkçe parity for all copy.
- Light and dark theme parity.
- Keyboard-reachable global shortcuts for the main toggles.
