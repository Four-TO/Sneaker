# Sneaker

A Windows desktop floating note & task app built with **Tauri 2 + Svelte 5 + Rust**.

Focus mode by default: frameless, translucent, always within reach, never in the way.

---

## Features

### Window & appearance
- Frameless, transparent, always-on-top toggleable
- `Ctrl + MouseWheel` to adjust opacity on the fly · slider in Settings
- Fully-transparent-background mode (opacity applies only to text/controls)
- Dark / Light theme with auto-matching bg color
- Windows Acrylic / Mica blur (`window-vibrancy`)
- Title bar & bottom bar can be shown/hidden independently
- Non-passthrough: hold modifier (Alt / Ctrl / Shift / Win) + left-drag to move window anywhere

### Interaction
- Click-through **three-state** (off / semi / full); semi disabled when title bar is hidden
- Tray icon with full menu (show / hide / top / passthrough / settings / lock / quit)
  - Left double-click → restore & focus main window
  - Right-click → menu
  - Taskbar icon toggleable via `WS_EX_TOOLWINDOW`
- Global hotkeys (all rebindable, per-field conflict detection, auto-revert on failure)
  - Toggle window visibility
  - Toggle always-on-top
  - Cycle passthrough
  - Boss key (instant hide + lock)
  - Quick capture (pops window + jumps to new-task input)
- Local shortcuts: `Ctrl+1` notes · `Ctrl+2` tasks · `Ctrl+,` settings · `Ctrl+N` focus input · `Ctrl+B` toggle sidebar

### Tasks
- Groups: 🔥 Urgent pinned · ▶ Working · 📋 Todo · ✓ Done-today · ⌄ Earlier (auto-folded)
- Quick input with prefix shortcuts: `!xxx` → pinned · `>xxx` → directly Working
- Double-click title to rename; hover for ▶/⏸, 📌 pin, ✕ delete
- Stored as a single `tasks.json` (easy to sync/diff/edit by hand)

### Notes
- Drag `.txt` / `.md` / `.log` / `.json` / ... onto the window to open as plain text (no rendering)
- Sidebar list with fuzzy search, rename, delete, auto-save
- Plain `.md` files under `%APPDATA%\Sneaker\notes\`

### Security
- Optional master password (Argon2 KDF), Telegram-style two-level
- Boss key triggers instant hide **and** lock
- Auto-lock after configurable idle
- Per-note encryption backend ready (ChaCha20-Poly1305; UI at M3)

### Persistence
- Window position/size restored on startup (`tauri-plugin-window-state`)
- Settings saved to `%APPDATA%\Sneaker\settings.json`
- Opt-in autostart (`tauri-plugin-autostart`)

---

## Build

### Prerequisites
- Node ≥ 20, pnpm ≥ 10
- Rust ≥ 1.80, `rustup target add x86_64-pc-windows-msvc`
- Visual Studio 2022 Build Tools (C++ workload)
- WebView2 Runtime (preinstalled on Win11)

### Commands
```bash
pnpm install           # install JS deps
pnpm tauri dev         # dev with HMR
pnpm tauri build       # release exe + NSIS installer
```

Artifacts:
- `src-tauri/target/release/sneaker.exe` — portable binary
- `src-tauri/target/release/bundle/nsis/Sneaker_*-setup.exe` — installer

---

## Data locations

| Kind | Path |
|---|---|
| Settings | `%APPDATA%\Sneaker\settings.json` |
| Notes | `%APPDATA%\Sneaker\notes\*.md` |
| Tasks | `%APPDATA%\Sneaker\tasks.json` |
| Window state | `%APPDATA%\Sneaker\.window-state.json` |

All plain text / JSON — syncable via Git, OneDrive, Dropbox, etc.

---

## Architecture

```
┌─── Frontend (Svelte 5 + Vite + TS) ────────────────┐
│  App.svelte           state + view router          │
│  views/Main.svelte    notes                        │
│  views/Tasks.svelte   tasks with groups            │
│  views/Settings.svelte                             │
│  views/Lock.svelte    master password screen       │
│  components/          TitleBar, BottomBar          │
│  lib/api.ts           tauri invoke wrappers        │
│  lib/store.ts         Svelte stores                │
└─────────────────────────────────────────────────────┘
                           ▲
                   invoke  │  events
                           ▼
┌─── Backend (Rust + Tauri 2) ───────────────────────┐
│  lib.rs         wiring & setup                     │
│  commands.rs    #[tauri::command] handlers         │
│  settings.rs    typed settings                     │
│  storage.rs     notes, argon2, chacha20poly1305    │
│  tasks.rs       tasks JSON store                   │
│  tray.rs        tray icon + menu                   │
│  hotkeys.rs     global shortcut registry           │
│  win_util.rs    WS_EX_TOOLWINDOW, WS_EX_TRANSPARENT│
└─────────────────────────────────────────────────────┘
```

---

## Shortcuts cheatsheet

### Global (rebindable in settings)
| Action | Default |
|---|---|
| Toggle window | `Ctrl+Alt+N` |
| Toggle always-on-top | `Ctrl+Alt+T` |
| Cycle passthrough | `Ctrl+Alt+P` |
| Boss key (hide + lock) | `Ctrl+Alt+H` |
| Quick capture to task | `Ctrl+Alt+Q` |

### Local
| Action | Key |
|---|---|
| Notes view | `Ctrl+1` |
| Tasks view | `Ctrl+2` |
| Settings view | `Ctrl+,` |
| Focus new-task input | `Ctrl+N` (while in Tasks) |
| Toggle sidebar | `Ctrl+B` |
| Adjust opacity | `Ctrl + MouseWheel` |
| Drag window | `Alt + LeftDrag` (modifier configurable) |

---

## Roadmap

- [ ] Per-note encryption UI (backend ready)
- [ ] Tag / backlink `[[note]]`
- [ ] Multi-sticky independent windows
- [ ] Sync providers (WebDAV / Git) — trait interface stubbed
- [ ] Markdown editor mode
- [ ] Task drag reorder

---

## License

MIT
