# Platinum Shell GTK

Parallel shell prototype for migrating Platinum Launcher from `React/Tauri` toward a native Linux shell built with `GTK4 + Relm4`.

This folder is intentionally isolated from the existing `platinum-launcher` app.
Use it to develop the future device shell without breaking the current launcher.

## Why this exists

- keep the current `React/Tauri` build working
- start native Linux shell work in parallel
- validate `GTK4 + Relm4` on Raspberry Pi / Wayland
- gradually move shared device logic from frontend code into Rust services

## Proposed runtime stack

- compositor: `labwc`, `sway`, `wayfire`, or `weston`
- shell UI: this crate (`GTK4 + Relm4`)
- hardware/service layer: Rust daemons or IPC services
- session startup: compositor autostarts `platinum-shell-gtk`

## Project layout

```text
platinum-shell-gtk/
├── Cargo.toml
├── README.md
└── src
    ├── app.rs
    ├── device.rs
    └── main.rs
```

## Suggested migration path

1. Keep `platinum-launcher` as the reference UI for screen layout and interaction.
2. Rebuild only the shell chrome here first:
   - status header
   - main content card
   - bottom device navigation
3. Move device data providers into Rust modules/services.
4. Replace placeholder data with real system reads for battery, GPS, radio, and LTE.
5. Wire compositor startup so this app becomes the visible shell on boot.

## Run locally

You need GTK4 development packages installed on the host system.

```bash
cargo run
```

## Run on macOS

Install the native GTK toolchain first:

```bash
brew install gtk4 libadwaita pkgconf
```

Then open a new shell and run:

```bash
cd platinum-shell-gtk
cargo run
```

If `pkg-config` still cannot find GTK, check the Homebrew prefix:

```bash
brew --prefix gtk4
pkg-config --cflags gtk4
```

On Apple Silicon this is usually under `/opt/homebrew`.

## Raspberry Pi notes

- test under Wayland first, not X11
- keep the window fixed to portrait `720x1560`
- run fullscreen or kiosk-like from compositor session config
- avoid heavy CSS and animation while the shell is still stabilising
