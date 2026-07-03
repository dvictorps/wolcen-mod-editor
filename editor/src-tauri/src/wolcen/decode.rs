use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};

use super::paths::Config;

/// True if the file begins with the CryXML binary magic.
pub fn is_cryxml(path: &Path) -> Result<bool> {
    let mut f = fs::File::open(path)?;
    let mut buf = [0u8; 7];
    let n = f.read(&mut buf)?;
    Ok(n == 7 && &buf == b"CryXmlB")
}

/// Decode a file (relative to the extracted Umbra root) into plain XML text.
///
/// If it is CryXML it is run through DataForge2; if already plain it is read
/// as-is. Decoded output is cached under `cache_dir` so repeat reads are cheap.
pub fn read_umbra_xml(cfg: &Config, rel: &str) -> Result<String> {
    let rel_win = rel.replace('/', "\\");
    let src = cfg.extracted_umbra.join(&rel_win);
    if !src.exists() {
        bail!("source not found: {}", src.display());
    }

    // Non-CryXML files (rare here) are already plain text.
    if !is_cryxml(&src)? {
        return Ok(fs::read_to_string(&src)?);
    }

    let cached = cfg.cache_dir.join(&rel_win); // e.g. .cache\Skills\...\Foo.xml
    if let Some(p) = cached.parent() {
        fs::create_dir_all(p)?;
    }

    // DataForge2 turns <name>.xml (CryXML) into <name>.raw (plain). Feed it a
    // copy that ends in .xml so the .raw naming is predictable.
    let feed = cached.with_extension("cryxml.xml");
    fs::copy(&src, &feed)?;
    let df = cfg.tools_bin.join("DataForge2.exe");
    let status = Command::new(&df)
        .arg(&feed)
        .status()
        .with_context(|| format!("running {}", df.display()))?;
    // DataForge2 exits 0 on success; some builds return non-zero even so — rely
    // on the presence of the .raw output rather than the exit code.
    let raw = feed.with_extension("raw");
    if !raw.exists() {
        bail!(
            "DataForge2 produced no output for {} (status {:?})",
            src.display(),
            status.code()
        );
    }
    let text = fs::read_to_string(&raw)?;
    // Tidy the intermediate; keep the .raw as the cache artifact.
    let _ = fs::remove_file(&feed);
    let final_path: PathBuf = cached; // keep decoded under the clean name too
    let _ = fs::copy(&raw, &final_path);
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
