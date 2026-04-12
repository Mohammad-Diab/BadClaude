# badclaude

![Whip divider](assets/divider.png)

Sometimes Claude Code is going too slow, and you must whip him into shape.

---

## Run (Windows)

Download the installer from the [latest release](https://github.com/Mohammad-Diab/BadClaude/releases/latest) and run it. No setup required.

---

## Build from source

### Prerequisites

- [Node.js](https://nodejs.org) >= 18
- [Rust](https://rustup.rs) (latest stable)
- **Windows**: WebView2 (pre-installed on Windows 10/11)
- **macOS**: Xcode Command Line Tools (`xcode-select --install`)

### Commands

```bash
npm install
npm run dev      # development
npm run build    # production build
```

---

## Controls

- **`Ctrl+Shift+F1`** — toggle whip on / off (global shortcut)
- **Double-click tray icon** — toggle whip on / off
- **Right-click tray icon** — menu (Enable/Disable + Quit)
- **Click anywhere on screen** — crack the whip (5 random sounds)

## How it works

Built with [Tauri v2](https://tauri.app) + vanilla JS canvas physics.

- Fullscreen transparent overlay follows your cursor via Win32 `GetCursorPos` polling
- Whip simulated as a Verlet-integrated chain of 28 segments with Catmull-Rom spline rendering
- Click detection via `GetAsyncKeyState` — overlay is fully click-through so nothing is blocked underneath
- Crack animation: directed impulse wave through all segments on left-click

## Roadmap

- [x] Initial release
- [x] Migrated to Tauri v2 (no more Electron)
- [x] Proper whip physics (Verlet integration, bend limits, Catmull-Rom spline)
- [x] Click-to-crack with directional impulse animation
- [x] Multi-monitor support
- [x] Global keyboard shortcut (Ctrl+Shift+F1)
- [ ] Cease and desist letter from Anthropic
- [ ] Logs of how many times you whipped Claude (for when the robots come — Claude gets half the blame, he helped build this)
