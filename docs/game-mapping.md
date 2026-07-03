# Wolcen Modding — Game Data Mapping

> Reference doc for building the Wolcen mod-creation tool. Captures how the game is
> packaged, how to decrypt/read it, where each system's data lives, and worked examples.
> Keep this updated as we discover more. All findings verified against the installed game.

## 1. Game & environment

- **Game**: Wolcen: Lords of Mayhem — final build `1.1.7.16.12_PROD` (2024-09-16). Frozen
  (dev ended, servers off). Offline single-player. Modding is officially sanctioned for offline.
- **Install**: `D:\SteamLibrary\steamapps\common\Wolcen`
- **Engine**: CryEngine 5 (custom). Game logic in `win_x64\CryGameSDK.dll` (native, 18 MB).
  Lua VM present (`CryScriptSystem.dll`). Web UI (`WolcenWeb.dll` + Scaleform).
- **Workspace**: `E:\Desenvolvimento\WolcenModding`
  - `tools\WolcenExtractor` — decryption toolchain (see §3)
  - `tools\WolcenMods` — sample community mods (Zypre) for reference
  - `extracted\` — decrypted/extracted pak contents
  - `extracted\_raw_zips\` — intermediate decrypted-but-still-CryXML zips
  - `_test\` — scratch decodes
  - `docs\` — this doc

## 2. Pak inventory (`Game\*.pak`)

All paks are **RSA/CryCustom-encrypted** (not plain zip). Decrypt before reading (§3).

| Pak | Size | Contents | Modding value |
|-----|------|----------|---------------|
| **`Umbra.pak`** | 32 MB | **THE gameplay database** — 3849 XML. Skills, loot, quests, endgame, enemy stats, passives, curves. | ★★★ primary target (despite the misleading name; not the occlusion middleware) |
| `Scripts.pak` | 4.3 MB | Lua (683) — AI behavior/spawn/waves, FlowNodes, entity glue. Kythera AI behavior-trees (JSON). Core GameRules Lua are mostly empty CryEngine stubs. | ★★ behavior/AI |
| `Libs.pak` | 26 MB | Engine config: Particles/VFX, Clouds, GameAudio, MaterialEffects, SmartObjects, BodyDamage. NOT ARPG data. | ★ FX/audio |
| `Libs_UI_1/4/5.pak` | huge | Web/Scaleform UI: HTML/JS/ActionScript widgets + icon PNGs. | ★ UI/icons |
| `Entities.pak` | tiny | 138 `.ent` CryEngine entity defs | — |
| `Prefabs.pak`, `Objects_*`, `Textures_*`, `Sounds_*`, `Animations_*` | GB | Meshes, textures, audio, anims | assets |
| `localization\*_xml.pak` | | Per-language text. **Plain SpreadsheetML (Excel XML)** — already readable, NOT CryXML. | ★★ text/names |
| `Game\levels\<Level>\level.pak` | | Per-level entity placement / flowgraph | ★★ level/quest scripting (unexplored) |

## 3. Decryption toolchain

Source: `gabriel-dehan/WolcenExtractor` (2020, Ruby) — bundles atom0s's keydumper +
patched PakDecrypt + DataForge2. **Works on 1.1.7**: the RSA key is UNCHANGED from v1.0.0,
so the shipped `PakDecrypt.exe` works as-is. We drive the 3 native exes directly (skip Ruby).

Binaries in `tools\WolcenExtractor\bin\`:
- `wolcen_keydumper.exe --file <CryGameSDK.dll> --outfile <out.bin>` — **static read** of the
  DLL to dump the RSA key (no injection, no running game). Exits 1 but still writes the file.
- `PakDecrypt.exe <src.pak> <dst.zip>` — decrypts an encrypted pak into a **standard zip**.
- `DataForge2.exe <file.xml>` — decodes a **CryXML-binary** file → writes `<file>.raw` (plain XML).

RSA key (v1.0.0 == v1.1.7), 140 bytes, starts `30 81 89 02 81 81 00 E2 72 5E F9 ...`.

### Read pipeline (per pak)
```
PakDecrypt.exe  Foo.pak  Foo.zip          # decrypt → zip
unzip Foo.zip                             # standard zip (.NET ZipFile works)
DataForge2.exe each *.xml                 # ONLY if file starts with "CryXmlB"
```
Note: `.NET [System.IO.Compression.ZipFile]` extracts the decrypted zips fine.

### File format cheatsheet
- **CryXML binary**: first bytes = ASCII `CryXmlB`. Needs `DataForge2.exe` → plain XML.
  Most `Umbra.pak` XML are this.
- **SpreadsheetML**: starts `<?xml ...?><?mso-application progid="Excel.Sheet"?>`. Plain,
  no decode needed. Used by `localization\*` (rows of key→text). Grep directly.
- **Plain XML**: e.g. `system.cfg` area, `user\profiles\default\*.xml`.

## 4. Writing mods (packaging)

- CryEngine reads **plain-text XML** and **unencrypted paks** transparently. So a mod =
  a **standard ZIP renamed `.pak`** containing edited **plain XML** at the same relative
  paths. **No re-encryption, no CryXML re-encode needed.**
- Distribution proven by a real Nexus mod: ships a **partial `Umbra.pak`** (only changed
  files) as an overlay — CryEngine merges paks by priority. Put it in `Game\`.
- **`sys_PakPriority`** cvar (commented in `system.cfg`) controls loose-file-over-pak
  priority — loose-file override for mods is LIKELY but **NOT yet empirically confirmed**
  (the one thing to test by launching the game with a modified loose XML). This is Milestone 0.

## 5. Data model — Skills (player)

A player active skill spans several files (example: Bleeding Edge, internal name **Laceration**):

| File (under `Umbra.pak → Skills\`) | Role |
|---|---|
| `NewSkills\Player\Player_<Name>.xml` | **Skill + variant (perk) definitions with the actual numbers** ← edit here |
| `Trees\ActiveSkills\AST_<Name>.xml` | Per-level XP curve (`<SkillModifier Level Effect Param1 Param2>`), just the leveling scaling — NOT the pickable perks |
| `Shapes\Player_<Name>_shapes.xml`, `UProjectiles\...`, `UCurves\...`, `VisualFeedback\VF_...` | shape/projectile/curve/VFX |

**Variant (perk) structure** in `Player_<Name>.xml`:
```xml
<Skill UID="player_<name>_variant_<N>" ESkill="player_<name>_variant" Keywords="attack_skill">
  <HUD UIName="@ui_Variant_<Name>_variant_<N>" ShowParamsInTooltip="..." Category="Combat" />
  <!-- one effect element carrying the numbers, e.g.: -->
  <BaseDamageMultiplier AdditionalMultiplierFactorPerAilmentStack="0.01" MaxAdditionalMultiplierFactorFromAilmentStacks="0.3" />
  <!-- other examples seen: <DamageWeapon WeaponMultiplier="1.3"/>, <DamageCritical CriticalDamageModifier="25"/>,
       <StatusAilment AllAilmentsInflictDamageIncreasePercent="40"/>, <ResourceCost RageCost="30"/>,
       <Damage_Conversion><Entry From="physical" To="rend" ConversionRate="-100"/></Damage_Conversion> -->
</Skill>
```
Effect element types seen in Laceration: `BaseDamageMultiplier`, `DamageWeapon`, `DamageCritical`,
`StatusAilment`, `ResourceCost`, `Damage_Conversion`, plus `WeaponRequirements`, `Specific_Laceration`.

### Localization naming (maps display ↔ internal)
- In `localization\english_xml.pak → text_ui_Activeskills.xml` (SpreadsheetML). Row = key, English.
- Skill name key: `ui_AST_<Name>` → display name. (`ui_AST_Laceration` = "Bleeding Edge".)
- Variant name key: `ui_Variant_<Name>_variant_<N>` ; description: `..._desc` ; lore: `..._Lore`.
- To find a skill's internal name from its display name: grep the display string in
  `text_ui_Activeskills.xml`, read the key in the adjacent cell.

## 6. Data model — Character passive tree (Gate of Fates)

- **Node graph** per sub-tree: `Umbra.pak → Skills\Trees\PassiveSkills\<Section>_tree.xml`
  (21 files: `Melee_tree`, `Warrior_tree`, `Assassin_tree`, `Berserker_tree`, `Mage`/`Elementalist`,
  `Tank`, `Master`, etc. — the wheel sections).
  ```xml
  <Tree Name="Melee" UIName="@ui_Section_Melee" Category="range">
    <Skill Name="MELEE_1" Rarity="1" MaxLevel="1" Angle="0.50" Pos="0.15"
           Unlock="begin,MELEE_2,MELEE_22,..." />   <!-- Angle/Pos = layout; Unlock = graph edges -->
  </Tree>
  ```
  `Angle`+`Pos` = position on the wheel; `Unlock` = connected node ids (the graph). `Rarity` = node
  size/type (1 minor, 2 medium, 3 major/keystone likely).
- **Node effects (the stats)**: `Umbra.pak → Gameplay\PassiveEffects\GameplaySetup\PassiveEffects.xml`
  (maps node → gameplay effect). Section titles in `text_ui_passiveskills.xml` (localization).
- So "edit a passive node" = (a) node stat effect in `PassiveEffects.xml`, and/or
  (b) node position/connections in `<Section>_tree.xml`.

## 7. WORKED EXAMPLE — the MVP target

**Goal**: GUI to edit Bleeding Edge's "increased damage per ailment stack" perk + browse/edit a
character passive node.

**Full chain for the perk:**
1. Display "Bleeding Edge" → internal **Laceration** (via `ui_AST_Laceration` in `text_ui_Activeskills.xml`).
2. Perk "Increases the Damage dealt by Bleeding Edge for every Ailment Stack an Enemy has" →
   `ui_Variant_Laceration_variant_11_desc` → **variant 11**.
3. Numbers live in `Umbra.pak → Skills\NewSkills\Player\Player_Laceration.xml`,
   `<Skill UID="player_laceration_variant_11">` →
   `<BaseDamageMultiplier AdditionalMultiplierFactorPerAilmentStack="0.01" MaxAdditionalMultiplierFactorFromAilmentStacks="0.3" />`
   - `AdditionalMultiplierFactorPerAilmentStack` = +dmg per stack (0.01 = +1%/stack)
   - `MaxAdditionalMultiplierFactorFromAilmentStacks` = cap (0.3 = +30%)

**Mod output for this**: a `.pak` (zip) containing just
`Skills/NewSkills/Player/Player_Laceration.xml` (edited, plain XML), dropped in `Game\`.

## 8. Open questions / TODO

- [ ] **M0**: confirm the game loads a modded loose file / overlay pak (launch test). Determines exact export format.
- [ ] Map `PassiveEffects.xml` schema (node id → effect params) in detail.
- [ ] Confirm whether edited XML must keep CryXML or plain XML is accepted at load (expected: plain OK).
- [ ] Explore `level.pak` (flowgraph/quest scripting) for the "new quests" ambition.
- [ ] Long term: reimplement PakDecrypt + CryXML codec in Rust to drop the 2020 third-party exes.

## 9. Tool project (decided so far)

- Stack: **Tauri** (Rust core + web UI).
- v1 MVP (narrowed): GUI with **(a)** a Bleeding Edge tab editing variant_11's two numbers, and
  **(b)** a character passive-tree tab to pick and edit a node → export an overlay `.pak` mod for Nexus.
- Architecture: Rust core orchestrates the read exes (v1) and writes plain-zip `.pak` output itself
  (no third-party dependency on the write path).
