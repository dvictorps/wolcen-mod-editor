# Wolcen Mod Editor

A desktop GUI for creating mods for **Wolcen: Lords of Mayhem** — visually edit the
game's gameplay data (skills, the Gate of Fates passive tree, and more) and export a
ready-to-publish overlay mod. No hand-editing XML.

> **Status: early WIP.** The data-mapping and Rust core (reading/parsing real game data)
> work; the GUI and exporter are in progress. Not yet a usable release.

Wolcen's development ended and its servers are offline; the studio sanctioned modding for
offline play but never shipped tools. The community only had an outdated one-way unpacker.
This project aims to give modders a real round-trip editor so a modding scene can grow
around the (now frozen) game.

## What it does

- Opens and decodes the game's encrypted data (via the existing decryption toolchain).
- Parses it into a clean model: skills and their perks (variants), passive-tree sections
  and nodes, with the actual editable numbers.
- Lets you edit values in a GUI — including a rendered **Gate of Fates** wheel you click.
- Exports your changes as an overlay `.pak` mod you drop into `Game\`.

## Architecture (4 pieces)

- **Decode tools** — third-party exes (atom0s's keydumper + PakDecrypt + DataForge2) that
  decrypt the `.pak` archives and CryXML into plain XML. We orchestrate them (and may
  reimplement in Rust later).
- **Core (Rust)** — `editor/src-tauri/src/wolcen/`: decodes on demand, parses skills and
  passives into JSON for the UI, and writes the mod on export.
- **UI (React/TypeScript)** — `editor/src/`: skill list + perk editor, Gate of Fates canvas.
- **Tauri** — bundles the web UI + Rust core into a single Windows app.

See [`docs/tool-design.md`](docs/tool-design.md) for the design and
[`docs/game-mapping.md`](docs/game-mapping.md) for the reverse-engineered data map.

## Build

Prerequisites: Rust (MSVC toolchain + **Windows SDK**), Node.js + pnpm, WebView2.

```bash
cd editor
pnpm install
pnpm tauri dev
```

## Legal & credits

- **Game data is © Wolcen Studio and is NOT included** in this repo. You need your own
  legally-owned copy of Wolcen. Modding is sanctioned for offline play by the studio.
- Decryption relies on prior work by **atom0s** and **gabriel-dehan/WolcenExtractor**.
- This tool is unofficial and not affiliated with Wolcen Studio.
