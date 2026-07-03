use std::path::PathBuf;

/// Resolved filesystem locations the core needs.
///
/// For now these default to the dev workspace, overridable via env vars so the
/// app is not hard-wired to one machine. Later this becomes user-configurable
/// via a settings screen (pick the Wolcen install dir, everything else derives).
#[derive(Clone, Debug)]
pub struct Config {
    /// Wolcen install root (contains `Game\`, `win_x64\`, ...).
    pub game_dir: PathBuf,
    /// Folder with keydumper/PakDecrypt/DataForge2 exes.
    pub tools_bin: PathBuf,
    /// Already-extracted (still CryXML) Umbra data root: `.../Umbra/Umbra`.
    pub extracted_umbra: PathBuf,
    /// Extracted English localization folder.
    pub localization_dir: PathBuf,
    /// Where decoded plain-XML is cached.
    pub cache_dir: PathBuf,
    /// Where exported mods are written.
    pub mods_dir: PathBuf,
}

fn env_or(key: &str, default: &str) -> PathBuf {
    match std::env::var(key) {
        Ok(v) if !v.trim().is_empty() => PathBuf::from(v),
        _ => PathBuf::from(default),
    }
}

impl Config {
    pub fn dev() -> Self {
        let base = env_or("WOLCEN_WORKSPACE", r"E:\Desenvolvimento\WolcenModding");
        Config {
            game_dir: env_or(
                "WOLCEN_GAME_DIR",
                r"D:\SteamLibrary\steamapps\common\Wolcen",
            ),
            tools_bin: base.join(r"tools\WolcenExtractor\bin"),
            extracted_umbra: base.join(r"extracted\Umbra\Umbra"),
            localization_dir: base.join(r"extracted\localization_en"),
            cache_dir: base.join(r"editor\.cache"),
            mods_dir: base.join("mods"),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config::dev()
    }
}
