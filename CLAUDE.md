# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
npm install       # install dependencies
npm run dev       # run in development mode (tauri dev)
npm run build     # production build
```

There are no tests or linting configured.

## Architecture

This is a Tauri v2 tray app. Key files:

**[src-tauri/src/lib.rs](src-tauri/src/lib.rs)** — Rust main process. Responsibilities:
- Creates a system tray icon with a right-click menu (Enable/Disable toggle + Quit)
- Double-click tray: toggles the overlay on/off
- Builds a fullscreen transparent overlay window (`overlay.html`) that starts enabled
- Exposes two Tauri commands to JS: `get_cursor` (polled every 16ms for cursor position + click detection) and `hide_overlay` (called after drop animation completes)
- Click detection via Win32 `GetAsyncKeyState(VK_LBUTTON)` — transitions only, with a 500ms suppression window after tray interactions to prevent bleed-through
- Cursor coordinates converted from physical screen pixels to logical window-relative pixels (accounts for DPI scaling and multi-monitor offsets)

**[web/overlay.html](web/overlay.html)** — Renderer. Self-contained canvas physics simulation:
- Whip simulated as a Verlet-integrated chain of 28 segments (`P.segments`)
- Handle (point 0) smoothly chases the cursor with a lerp (`P.handleSmoothing = 0.22`) so the handle carries real velocity into the chain
- Left-click anywhere fires `crackWhip()`: a directed impulse wave through all segments (quadratic forward kick + sine-curve perpendicular bulge) + random crack sound (A–E.mp3)
- Double-click tray → `drop-whip` event → `dropping = true` → whip falls under gravity → `hide_overlay` invoked
- All physics constants in the `P` object at the top — edit there to tune feel
- Rendering uses Catmull-Rom splines (converted to cubic Bézier) for smooth rope drawing
- Cursor position polled from Rust via `invoke('get_cursor')` every 16ms (click-through windows receive no DOM mouse events)

## Platform notes

- **Windows**: Uses `windows-sys` crate for `GetCursorPos` (cursor position) and `GetAsyncKeyState` (click detection). Both require `Win32_UI_WindowsAndMessaging`, `Win32_Foundation`, and `Win32_UI_Input_KeyboardAndMouse` features in `Cargo.toml`.
- **macOS**: Cursor and click detection stubs return `None`/`false` — not yet implemented.
- The overlay uses `transparent`, `decorations: false`, `always_on_top`, `skip_taskbar`, and `set_ignore_cursor_events(true)` so it is fully click-through while still rendering on top.
- A shared Cargo target directory is configured at `~/.cargo/config.toml` (`target-dir = "C:/Users/DrMoh/.cargo/shared-target"`) to avoid duplicating build artifacts across projects.
