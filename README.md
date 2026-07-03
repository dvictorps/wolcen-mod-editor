# Wolcen Mod Editor

A desktop GUI for creating mods for **Wolcen: Lords of Mayhem** — visually edit the
game's gameplay data (skills, the Gate of Fates passive tree) and export a
ready-to-install mod. No hand-editing XML.

> **Status: working prototype.** Skills editing, the Gate of Fates wheel (with search &
> node-effect editing), and mod export all work. See "Roadmap" for what's next.

Wolcen's development ended and its servers are offline; the studio sanctioned modding for
offline play but never shipped tools. The community only had an outdated one-way unpacker.
This project gives modders a real round-trip editor so a modding scene can grow around the
(now frozen) game.

![status](https://img.shields.io/badge/platform-Windows-blue) ![status](https://img.shields.io/badge/status-prototype-orange)

## What it does

- Decodes the game's encrypted data into a clean, editable model.
- **Skills tab** — every player skill, its perks (variants), and the editable numbers;
  toggle individual modifiers on/off.
- **Gate of Fates tab** — the full 21-section passive wheel rendered from real data, with
  zoom/pan, **search by node name or by what the modifier does**, and click-to-edit node stats.
- **Export** — writes a ready-to-install mod: a Nexus-style `Umbra\` folder (drop into
  `<Wolcen>\Game\`) plus an overlay `.pak`, and an install readme.

## Download & use (players / modders)

> Grab the latest build from the [**Releases**](../../releases) page (Windows).

1. Download and run the app.
2. Point it at your Wolcen install folder (e.g.
   `...\steamapps\common\Wolcen`). First run decrypts the data it needs (one time).
3. Edit skills / passive nodes, give your mod a name, click **Export mod**.
4. Copy the exported `Umbra` folder into `<Wolcen>\Game\` and launch the game.

You need your own legally-owned copy of Wolcen. Mods are offline-only.

## Run from source (developers)

Prerequisites (Windows):

- **Rust** (stable, MSVC toolchain) — https://rustup.rs
- **Windows SDK** + MSVC build tools (Visual Studio 2022 with "Desktop development with C++",
  which includes the Windows SDK). The Rust MSVC linker needs these.
- **Node.js 18+** and **npm**.
- **WebView2** runtime (preinstalled on Windows 11).

```bash
cd editor
npm install
npm run tauri dev      # dev app with hot reload
npm run tauri build    # production build -> installer + exe under src-tauri/target/release/
```

> If `link.exe` / `kernel32.lib` errors appear, run the build from a **"x64 Native Tools
> Command Prompt for VS 2022"** (or `call vcvars64.bat` first) so the MSVC environment is set.

### Project layout

- `editor/src-tauri/src/wolcen/` — Rust core: `decode`, `localization`, `skills`, `passives`,
  `export` (+ `bin/probe.rs`, a headless test harness).
- `editor/src/` — React UI (`tabs/SkillsTab.tsx`, `tabs/GateTab.tsx`).
- `docs/` — [`game-mapping.md`](docs/game-mapping.md) (reverse-engineered data map) and
  [`tool-design.md`](docs/tool-design.md) (design).

## Roadmap

- Self-contained first run (pick Wolcen folder → auto-decrypt) — required for public builds.
- Loot / item editing; more Gate polish; mod manager (load order).
- Reimplement decryption in Rust to drop the third-party exes.

## Legal & credits

- **Game data is © Wolcen Studio and is NOT included** in this repo or in any release.
  You need your own copy of Wolcen. Modding is sanctioned for offline play by the studio.
- Decryption builds on prior work by **atom0s** and **[gabriel-dehan/WolcenExtractor](https://github.com/gabriel-dehan/WolcenExtractor)**.
- Unofficial; not affiliated with Wolcen Studio.
