# badclaude

![Whip divider](assets/divider.png)

Sometimes Claude Code is going too slow, and you must whip him into shape.

## Install + run

```bash
npm install
npm run dev       # development
npm run build     # production build
```

## Controls

- **Double-click tray icon** — toggle whip on / off
- **Right-click tray icon** — menu (Enable/Disable toggle + Quit)
- **Click anywhere on screen** — crack the whip (plays one of 5 sounds)

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
- [ ] Cease and desist letter from Anthropic
- [ ] Logs of how many times you whipped Claude (for when the robots come — Claude gets half the blame, he helped build this)
