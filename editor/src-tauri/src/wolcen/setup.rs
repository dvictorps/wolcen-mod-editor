use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use anyhow::{bail, Context, Result};
use serde::Serialize;

use super::paths::Config;

#[derive(Serialize)]
pub struct AppState {
    pub game_dir: Option<String>,
    pub prepared: bool,
    pub detected: Option<String>,
    pub tools_ok: bool,
}

fn looks_like_wolcen(dir: &Path) -> bool {
    dir.join("Game").join("Umbra.pak").exists()
        || dir.join("win_x64").join("Wolcen.exe").exists()
}

/// Best-effort auto-detection of the Wolcen install (Steam).
pub fn detect_game() -> Option<String> {
    let mut candidates: Vec<PathBuf> = Vec::new();
    for base in [
        r"C:\Program Files (x86)\Steam",
        r"C:\Program Files\Steam",
    ] {
        candidates.push(
            PathBuf::from(base)
                .join("steamapps")
                .join("common")
                .join("Wolcen"),
        );
    }
    for vdf in [
        r"C:\Program Files (x86)\Steam\steamapps\libraryfolders.vdf",
        r"C:\Program Files (x86)\Steam\config\libraryfolders.vdf",
    ] {
        if let Ok(txt) = fs::read_to_string(vdf) {
            for line in txt.lines() {
                if line.to_lowercase().contains("\"path\"") {
                    if let Some(p) = line.split('"').filter(|s| !s.trim().is_empty()).last() {
                        let lib = p.replace("\\\\", "\\");
                        candidates.push(
                            PathBuf::from(lib)
                                .join("steamapps")
                                .join("common")
                                .join("Wolcen"),
                        );
                    }
                }
            }
        }
    }
    for drive in ["C", "D", "E", "F", "G"] {
        candidates.push(PathBuf::from(format!(
            r"{drive}:\SteamLibrary\steamapps\common\Wolcen"
        )));
    }
    candidates
        .into_iter()
        .find(|c| looks_like_wolcen(c))
        .map(|c| c.to_string_lossy().into_owned())
}

pub fn get_state(cfg: &Config) -> AppState {
    let game_dir = if cfg.game_dir.as_os_str().is_empty() {
        None
    } else {
        Some(cfg.game_dir.to_string_lossy().into_owned())
    };
    AppState {
        game_dir,
        prepared: cfg.is_prepared(),
        detected: detect_game(),
        tools_ok: cfg.tools_bin.join("PakDecrypt.exe").exists(),
    }
}

fn pakdecrypt(cfg: &Config, src: &Path, dst: &Path) -> Result<()> {
    let exe = cfg.tools_bin.join("PakDecrypt.exe");
    if !exe.exists() {
        bail!("bundled PakDecrypt.exe not found ({})", exe.display());
    }
    if !src.exists() {
        bail!("pak not found: {}", src.display());
    }
    Command::new(&exe)
        .arg(src)
        .arg(dst)
        .status()
        .with_context(|| format!("running {}", exe.display()))?;
    if !dst.exists() {
        bail!(
            "decryption produced no output for {} — the Microsoft Visual C++ 2015-2019 x86 runtime may be missing.",
            src.file_name().unwrap_or_default().to_string_lossy()
        );
    }
    Ok(())
}

fn unzip(zip_path: &Path, dest: &Path) -> Result<()> {
    fs::create_dir_all(dest)?;
    let f = fs::File::open(zip_path)?;
    let mut ar = zip::ZipArchive::new(f)?;
    ar.extract(dest)?;
    Ok(())
}

/// One-time first-run: decrypt Umbra.pak + English localization into the app-data folder.
pub fn prepare_data(cfg: &Config) -> Result<()> {
    if cfg.game_dir.as_os_str().is_empty() {
        bail!("No Wolcen folder set.");
    }
    if !looks_like_wolcen(&cfg.game_dir) {
        bail!("That folder doesn't look like a Wolcen install (no Game\\Umbra.pak).");
    }
    let tmp = cfg.data_root.join("tmp");
    fs::create_dir_all(&tmp)?;
    let gamedata = cfg.data_root.join("gamedata");

    // Umbra.pak -> gamedata/Umbra/... (zip entries are already prefixed with Umbra/)
    let umbra_zip = tmp.join("Umbra.zip");
    let _ = fs::remove_file(&umbra_zip);
    pakdecrypt(cfg, &cfg.game_dir.join("Game").join("Umbra.pak"), &umbra_zip)?;
    unzip(&umbra_zip, &gamedata)?;

    // english localization -> gamedata/localization/...
    let loc_zip = tmp.join("loc.zip");
    let _ = fs::remove_file(&loc_zip);
    pakdecrypt(
        cfg,
        &cfg.game_dir.join("localization").join("english_xml.pak"),
        &loc_zip,
    )?;
    unzip(&loc_zip, &cfg.localization_dir)?;

    let _ = fs::remove_dir_all(&tmp);
    if !cfg.is_prepared() {
        bail!("Setup finished but expected files are missing.");
    }
    Ok(())
}
