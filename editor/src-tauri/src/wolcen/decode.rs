use std::fs;
use std::io::Read;
use std::path::Path;
use std::process::Command;

use anyhow::{bail, Context, Result};

use super::paths::Config;

/// Windows: don't pop a console window when running the bundled console exes.
fn hidden_command(program: &Path) -> Command {
    let cmd = Command::new(program);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        let mut cmd = cmd;
        cmd.creation_flags(CREATE_NO_WINDOW);
        return cmd;
    }
    #[allow(unreachable_code)]
    cmd
}

/// True if the file begins with the CryXML binary magic.
pub fn is_cryxml(path: &Path) -> Result<bool> {
    let mut f = fs::File::open(path)?;
    let mut buf = [0u8; 7];
    let n = f.read(&mut buf)?;
    Ok(n == 7 && &buf == b"CryXmlB")
}

/// Decode a file (relative to the extracted Umbra root) into plain XML text.
///
/// Results are cached under `cache_dir`; once a file is decoded it is read
/// straight from the cache (no DataForge2 process spawned) — the game data is
/// frozen so the cache never goes stale.
pub fn read_umbra_xml(cfg: &Config, rel: &str) -> Result<String> {
    let rel_win = rel.replace('/', "\\");
    let cached = cfg.cache_dir.join(&rel_win);
    if cached.exists() {
        return Ok(fs::read_to_string(&cached)?);
    }

    let src = cfg.extracted_umbra.join(&rel_win);
    if !src.exists() {
        bail!("source not found: {}", src.display());
    }
    if let Some(p) = cached.parent() {
        fs::create_dir_all(p)?;
    }

    // Already-plain files: just copy into the cache.
    if !is_cryxml(&src)? {
        fs::copy(&src, &cached)?;
        return Ok(fs::read_to_string(&cached)?);
    }

    // CryXML: DataForge2 turns <name>.xml into <name>.raw (plain XML).
    let feed = cached.with_extension("cryxml.xml");
    fs::copy(&src, &feed)?;
    let df = cfg.tools_bin.join("DataForge2.exe");
    let status = hidden_command(&df)
        .arg(&feed)
        .status()
        .with_context(|| format!("running {}", df.display()))?;
    let raw = feed.with_extension("raw");
    if !raw.exists() {
        bail!(
            "DataForge2 produced no output for {} (status {:?})",
            src.display(),
            status.code()
        );
    }
    let text = fs::read_to_string(&raw)?;
    let _ = fs::rename(&raw, &cached).or_else(|_| fs::copy(&raw, &cached).map(|_| ()));
    let _ = fs::remove_file(&feed);
    Ok(text)
}

/// Read a plain-XML localization sheet from the localization dir (no decode).
pub fn read_localization(cfg: &Config, file: &str) -> Result<String> {
    let p = cfg.localization_dir.join(file);
    if !p.exists() {
        bail!("localization file not found: {}", p.display());
    }
    Ok(fs::read_to_string(&p)?)
}
