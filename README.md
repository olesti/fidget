# Fidget

A small, modular menu-bar / system-tray utility for macOS, Windows and Linux. It sits quietly in the background (no window, no Dock icon on macOS) and collects tiny useful tools in one place.

The first tool is **Jiggle**: nudges the mouse at random intervals so the system doesn't consider you idle.

> **Status:** v0.2 — settings window, Keep Awake, Pause All, launch at login, global shortcuts. See [Roadmap](#roadmap).

## Features

- **Jiggle** — random interval (default 30–120 s), `always` or `idle_only` mode, styles `subtle` / `natural` / `zen`, optional schedule (time window, weekdays only).
- Tray menu is generated from the registered features: toggle, live status line ("Last move: 42 s ago").
- Tray icon reflects state (filled = something active, ring = idle).
- **Keep Awake** — prevents system and display sleep (`⌘/Ctrl+Alt+K`). Jiggle toggles with `⌘/Ctrl+Alt+J`.
- **Pause All** — 15 min / 1 hour / until tomorrow, from the tray menu.
- **Settings** window (Svelte + Tailwind; General / Features / Shortcuts / Permissions / About), editable global shortcuts, English/Türkçe, light/dark theme, **Launch at Login**, settings persisted across restarts.
- Single instance; quit only from the tray menu.

## Run

Requirements: Rust (stable), Node 20+ and pnpm, plus the [Tauri platform prerequisites](https://tauri.app/start/prerequisites/).

```bash
pnpm install
pnpm tauri dev     # run with hot reload
pnpm tauri build   # package
cd src-tauri && cargo test
```

On **macOS** the first time you enable Jiggle, Fidget asks for the **Accessibility** permission (System Settings → Privacy & Security → Accessibility). Without it the cursor can't be moved.

On **Linux**, X11 works out of the box. On Wayland, Fidget uses the wlroots virtual-pointer protocol or the XDG RemoteDesktop portal (the compositor asks for consent once). The Wayland path is untested.

## Adding a feature

1. Create `src-tauri/src/features/<name>/` and implement the `Feature` trait (`src-tauri/src/feature/mod.rs`).
2. Add it to `register_all()` in `src-tauri/src/features/mod.rs`.

The tray section and toggle appear automatically. Platform-specific code lives only under `src-tauri/src/platform/`.

## Roadmap

- **v0.1** — tray, feature trait + manager, Jiggle (`subtle`), macOS permission flow ✅
- **v0.2** — settings window, persisted settings, Keep Awake, Pause All, launch at login, global shortcuts ✅
- **v0.3** — CI builds for 3 platforms, signing/notarization, auto-update

Ideas: Break Reminder, Focus Timer, Clipboard History, Auto Clicker, Quick Notes, Schedule.

## Contributing

Conventional Commits (`feat:`, `fix:`, `chore:` …).

## License

MIT
