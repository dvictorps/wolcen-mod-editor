First public build of the **Wolcen Mod Editor** — a visual GUI to make mods for Wolcen (skills, Gate of Fates passives, player stats) and export them ready to install. No hand-editing XML, and **the decryptor is bundled** (no extra downloads).

## Download (Windows)
- **Portable (recommended):** `Wolcen-Mod-Editor-v0.1.0-portable-win64.zip` — unzip and run `Wolcen Mod Editor.exe`. Keep the `tools` folder next to the exe.
- **Installer:** `Wolcen Mod Editor_0.1.0_x64-setup.exe`.

## First run
Point it at your Wolcen install folder (auto-detected if you have it on Steam). It decrypts what it needs **once**, then you edit and export.

## Export → install a mod
Edit values → name your mod → **Export** → copy the generated `Umbra` folder into `<Wolcen>\Game\` → launch the game.

## Notes
- You need your own legally-owned copy of Wolcen. Mods are **offline-only** (sanctioned by the studio).
- If decryption fails, install the **Microsoft Visual C++ 2015–2019 Redistributable (x86)**.
- Decryption builds on prior work by **atom0s** and **gabriel-dehan/WolcenExtractor** — thanks to them.
- Early build — feedback, bug reports and PRs welcome.
