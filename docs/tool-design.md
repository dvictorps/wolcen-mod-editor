# Wolcen Mod Editor — Tool Design (v1)

> Design for the GUI mod-creation tool. Data/format details live in `game-mapping.md`.
> Status: design consolidated, ready to build. Build M0 (load test) before the exporter.

## Goal

A desktop GUI where anyone opens the tool, visually edits Wolcen gameplay data (skills, passive
tree), and exports a ready-to-publish overlay mod for Nexus — no XML by hand. Lowers the barrier
so a modding scene can grow around the (now frozen) game.

## Stack

- **Tauri** — Rust core + web frontend (React).
- **Rust core**: orchestrates the read exes (v1) and writes the mod `.pak` itself (plain zip, no
  third-party dependency on the write path). Later: reimplement CryXML/pak decode in Rust to drop
  the 2020 exes.
- **Web UI**: React + SVG/Canvas for the visual editors.

## v1 MVP scope (two tabs)

### Tab A — Skills
- **Left rail**: list of player skills (real icon PNG from `u_resources` + display name from
  `text_ui_Activeskills.xml`). Click a skill.
- **Main panel**: all that skill's **variants (perks)** parsed from
  `Umbra.pak/Skills/NewSkills/Player/Player_<Name>.xml`. Each perk = a card showing its effect
  element's editable attributes (numbers) with the localized perk name/description.
- **Anchor example**: Bleeding Edge (Laceration) → variant_11 →
  `<BaseDamageMultiplier AdditionalMultiplierFactorPerAilmentStack MaxAdditionalMultiplierFactorFromAilmentStacks>`.

### Tab B — Gate of Fates (character passive tree)
- **Full wheel** of all 21 sections rendered from data (`Skills/Trees/PassiveSkills/<Section>_tree.xml`):
  nodes placed by `Angle`/`Pos`, edges from `Unlock`, size/style by `Rarity`, real node icons.
- Click a node → panel shows/edits its effect (from
  `Gameplay/PassiveEffects/GameplaySetup/PassiveEffects.xml`).
- **Fidelity**: faithful topology + real icons; functional graph, not a 1:1 art clone.
- **Known gap**: the ring ORDER of the 21 sections isn't in extracted data (likely `Libs_UI_4`/native).
  v1 uses a hardcoded/configurable section order, calibrated once by looking at the in-game wheel.
  Each section's internal layout is exact from data; only the wheel rotation is the manual bit.

### Export
- "Export mod" → Rust writes a standard zip renamed `.pak` containing only the edited plain-XML files
  at their original relative paths → user drops it in `Game\` (or we install it). Nexus-ready.
- Non-destructive: original decrypted data is the read-only base; edits tracked as overrides; export
  emits only changed files (overlay).

## Data flow

```
Open game dir
  → core: keydumper (once) → PakDecrypt Umbra.pak + localization → DataForge2 CryXML → plain XML cache
  → parse skills + passive trees + localization → structured model → UI
UI edits (typed fields) → override store
Export → serialize changed files as plain XML → zip → <mod>.pak
```

## Architecture notes

- **Read path v1**: shell out to `wolcen_keydumper.exe`, `PakDecrypt.exe`, `DataForge2.exe`
  (bundled). Cache decoded XML under the workspace so re-open is instant.
- **Write path v1**: pure Rust zip. Plain XML (CryEngine loads it). No encryption, no CryXML re-encode.
- **Model**: parse only what the tabs need in v1 (Player_*.xml variants, *_tree.xml nodes,
  PassiveEffects.xml, localization). Generic full-data editor is a later phase.

## Milestones

- **M0 — Load test (do first, before exporter)**: manually edit one plain XML (e.g. Player_Laceration
  variant_11), pack as overlay `.pak` (or loose file), launch game, confirm the change takes effect.
  Locks the exact export format (overlay pak vs loose file, plain XML accepted?).
- **M1 — Skeleton**: Tauri app; core opens game dir, runs read pipeline, lists player skills.
- **M2 — Tab A**: skill → perks → edit numbers → export working overlay pak. First end-to-end mod.
- **M3 — Tab B**: render one section graph from data; click node → edit effect.
- **M4 — Full wheel**: all 21 sections arranged in the ring (calibrated order); polish.
- **M5 — Packaging/UX**: mod metadata, icons, one-click export, README for Nexus.

## Open questions (see game-mapping.md §8)

- Exact export format (M0 answers).
- PassiveEffects.xml node→effect schema detail (needed for M3).
- Section ring order (calibrate from game for M4).
