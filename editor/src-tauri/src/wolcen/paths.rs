use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Resolved filesystem locations the core needs.
#[derive(Clone, Debug)]
pub struct Config {
    /// Wolcen install root (empty if not chosen yet).
    pub game_dir: PathBuf,
    /// Folder with the bundled PakDecrypt/DataForge2 exes.
    pub tools_bin: PathBuf,
    /// Decrypted (still-CryXML) Umbra data root, populated on first-run setup.
    pub extracted_umbra: PathBuf,
    /// Decrypted English localization folder.
    pub localization_dir: PathBuf,
    /// Where decoded plain-XML is cached.
    pub cache_dir: PathBuf,
    /// Where exported mods are written.
    pub mods_dir: PathBuf,
    /// App data root (settings, gamedata, cache, mods live here).
    pub data_root: PathBuf,
}

#[derive(Serialize, Deserialize, Default)]
pub struct Settings {
    pub game_dir: Option<String>,
}

fn env_or(key: &str, default: &str) -> PathBuf {
    match std::env::var(key) {
        Ok(v) if !v.trim().is_empty() => PathBuf::from(v),
        _ => PathBuf::from(default),
    }
}

fn read_settings(data_root: &std::path::Path) -> Settings {
    let p = data_root.join("settings.json");
    std::fs::read_to_string(p)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

/// Find the bundled tools. Order: next to the exe (portable zip) -> Tauri
/// resource dir (installer) -> source tree (dev).
fn resolve_tools(resource: &std::path::Path) -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let c = dir.join("tools");
            if c.join("PakDecrypt.exe").exists() {
                return c;
            }
        }
    }
    let prod = resource.join("tools");
    if prod.join("PakDecrypt.exe").exists() {
        return prod;
    }
    let dev = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("resources")
        .join("tools");
    if dev.join("PakDecrypt.exe").exists() {
        return dev;
    }
    prod
}

impl Config {
    /// Dev config (used by the headless probe) — points at the pre-extracted workspace.
    pub fn dev() -> Self {
        let base = env_or("WOLCEN_WORKSPACE", r"E:\Desenvolvimento\WolcenModding");
        Config {
            game_dir: env_or("WOLCEN_GAME_DIR", r"D:\SteamLibrary\steamapps\common\Wolcen"),
            tools_bin: base.join(r"tools\WolcenExtractor\bin"),
            extracted_umbra: base.join(r"extracted\Umbra\Umbra"),
            localization_dir: base.join(r"extracted\localization_en"),
            cache_dir: base.join(r"editor\.cache"),
            mods_dir: base.join("mods"),
            data_root: base.join("editor"),
        }
    }

    /// Runtime config for the shipped app: data under the OS app-data dir, tools bundled.
    pub fn from_app(app: &tauri::AppHandle) -> Self {
        use tauri::Manager;
        let data = app
            .path()
            .app_data_dir()
            .unwrap_or_else(|_| PathBuf::from("."));
        let resource = app.path().resource_dir().unwrap_or_else(|_| PathBuf::from("."));
        let settings = read_settings(&data);
        Config {
            game_dir: settings.game_dir.map(PathBuf::from).unwrap_or_default(),
            tools_bin: resolve_tools(&resource),
            extracted_umbra: data.join("gamedata").join("Umbra"),
            localization_dir: data.join("gamedata").join("localization"),
            cache_dir: data.join("cache"),
            mods_dir: data.join("mods"),
            data_root: data,
        }
    }

    pub fn settings_path(&self) -> PathBuf {
        self.data_root.join("settings.json")
    }

    pub fn save_game_dir(&self, dir: &str) -> anyhow::Result<()> {
        std::fs::create_dir_all(&self.data_root)?;
        let s = Settings {
            game_dir: Some(dir.to_string()),
        };
        std::fs::write(self.settings_path(), serde_json::to_string_pretty(&s)?)?;
        Ok(())
    }

    /// True once the game data has been decrypted into the app-data folder.
    pub fn is_prepared(&self) -> bool {
        self.extracted_umbra
            .join("Skills")
            .join("Trees")
            .join("PassiveSkills")
            .exists()
    }
}
