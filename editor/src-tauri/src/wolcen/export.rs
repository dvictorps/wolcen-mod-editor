use std::collections::BTreeMap;
use std::fs;
use std::io::Write;

use anyhow::{Context, Result};
use quick_xml::events::{BytesStart, Event};
use quick_xml::reader::Reader;
use quick_xml::writer::Writer;
use serde::{Deserialize, Serialize};
use zip::write::SimpleFileOptions;

use super::decode::read_umbra_xml;
use super::paths::Config;

#[derive(Deserialize, Debug)]
pub struct SkillEdit {
    pub file: String,
    pub uid: String,
    pub element: String,
    pub attr: String,
    pub value: f64,
}

#[derive(Deserialize, Debug)]
pub struct PassiveEdit {
    pub file: String,
    pub node: String,
    pub eim: String,
    pub attr: String,
    pub value: f64,
}

#[derive(Deserialize, Debug)]
pub struct ExportRequest {
    pub mod_name: String,
    pub skill_edits: Vec<SkillEdit>,
    pub passive_edits: Vec<PassiveEdit>,
}

#[derive(Serialize, Debug)]
pub struct ExportResult {
    pub pak: String,
    pub folder: String,
    pub files: usize,
    pub changes: usize,
}

/// A single attribute change scoped to a context inside a file.
enum Rule {
    /// Set `attr` on element `element` inside `<Skill UID=uid>`.
    Skill { uid: String, element: String, attr: String, value: f64 },
    /// Set `attr` on `<Semantics>` inside `<Spell Name=node><EIM Name=eim>`.
    Passive { node: String, eim: String, attr: String, value: f64 },
}

#[derive(Default)]
struct Ctx {
    skill_uid: Option<String>,
    spell: Option<String>,
    eim: Option<String>,
}

fn attr(e: &BytesStart, key: &[u8]) -> Option<String> {
    for a in e.attributes().flatten() {
        if a.key.as_ref() == key {
            return Some(a.unescape_value().unwrap_or_default().into_owned());
        }
    }
    None
}

fn fmt(v: f64) -> String {
    // Rust's Display drops the trailing ".0" (2.0 -> "2") and keeps decimals (0.01 -> "0.01").
    format!("{v}")
}

/// Which (attr -> value) changes apply to this element in this context.
fn targets(lname: &[u8], ctx: &Ctx, rules: &[Rule]) -> Vec<(String, f64)> {
    let mut out = Vec::new();
    for r in rules {
        match r {
            Rule::Skill { uid, element, attr, value } => {
                if lname == element.as_bytes() && ctx.skill_uid.as_deref() == Some(uid) {
                    out.push((attr.clone(), *value));
                }
            }
            Rule::Passive { node, eim, attr, value } => {
                if lname == b"Semantics"
                    && ctx.spell.as_deref() == Some(node)
                    && ctx.eim.as_deref() == Some(eim)
                {
                    out.push((attr.clone(), *value));
                }
            }
        }
    }
    out
}

/// Rebuild a start tag with some attribute values replaced. Returns None if nothing changes.
fn modify(e: &BytesStart, changes: &[(String, f64)]) -> Option<(BytesStart<'static>, usize)> {
    if changes.is_empty() {
        return None;
    }
    let name = String::from_utf8_lossy(e.name().as_ref()).into_owned();
    let mut nb = BytesStart::new(name);
    let mut applied = 0;
    for a in e.attributes().flatten() {
        let key = String::from_utf8_lossy(a.key.as_ref()).into_owned();
        let mut val = a.unescape_value().unwrap_or_default().into_owned();
        if let Some((_, v)) = changes.iter().find(|(k, _)| *k == key) {
            val = fmt(*v);
            applied += 1;
        }
        nb.push_attribute((key.as_str(), val.as_str()));
    }
    Some((nb, applied))
}

/// Apply all rules for one file to its decoded XML, returning (new_xml, changes_applied).
fn rewrite(xml: &str, rules: &[Rule]) -> Result<(String, usize)> {
    let mut reader = Reader::from_str(xml);
    let mut writer = Writer::new(Vec::new());
    let mut stack: Vec<(Vec<u8>, Option<String>)> = Vec::new();
    let mut applied = 0usize;

    let ctx_of = |stack: &[(Vec<u8>, Option<String>)]| -> Ctx {
        let mut c = Ctx::default();
        for (ln, key) in stack.iter().rev() {
            match ln.as_slice() {
                b"Skill" if c.skill_uid.is_none() => c.skill_uid = key.clone(),
                b"Spell" if c.spell.is_none() => c.spell = key.clone(),
                b"EIM" if c.eim.is_none() => c.eim = key.clone(),
                _ => {}
            }
        }
        c
    };
    let key_of = |lname: &[u8], e: &BytesStart| -> Option<String> {
        match lname {
            b"Skill" => attr(e, b"UID"),
            b"Spell" | b"EIM" => attr(e, b"Name"),
            _ => None,
        }
    };

    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                let lname = e.local_name().as_ref().to_vec();
                let ctx = ctx_of(&stack);
                let ch = targets(&lname, &ctx, rules);
                match modify(&e, &ch) {
                    Some((nb, n)) => {
                        applied += n;
                        writer.write_event(Event::Start(nb))?;
                    }
                    None => writer.write_event(Event::Start(e.clone()))?,
                }
                stack.push((lname.clone(), key_of(&lname, &e)));
            }
            Event::Empty(e) => {
                let lname = e.local_name().as_ref().to_vec();
                let ctx = ctx_of(&stack);
                let ch = targets(&lname, &ctx, rules);
                match modify(&e, &ch) {
                    Some((nb, n)) => {
                        applied += n;
                        writer.write_event(Event::Empty(nb))?;
                    }
                    None => writer.write_event(Event::Empty(e.clone()))?,
                }
            }
            Event::End(e) => {
                stack.pop();
                writer.write_event(Event::End(e))?;
            }
            Event::Eof => break,
            other => writer.write_event(other)?,
        }
    }

    let bytes = writer.into_inner();
    Ok((String::from_utf8(bytes)?, applied))
}

fn sanitize(name: &str) -> String {
    let s: String = name
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '_' || c == '-' { c } else { '_' })
        .collect();
    let s = s.trim_matches('_').to_string();
    if s.is_empty() { "MyWolcenMod".into() } else { s }
}

pub fn export(cfg: &Config, req: ExportRequest) -> Result<ExportResult> {
    let mut by_file: BTreeMap<String, Vec<Rule>> = BTreeMap::new();
    for e in req.skill_edits {
        by_file.entry(e.file).or_default().push(Rule::Skill {
            uid: e.uid,
            element: e.element,
            attr: e.attr,
            value: e.value,
        });
    }
    for e in req.passive_edits {
        by_file.entry(e.file).or_default().push(Rule::Passive {
            node: e.node,
            eim: e.eim,
            attr: e.attr,
            value: e.value,
        });
    }

    let mod_name = sanitize(&req.mod_name);
    let out_dir = cfg.mods_dir.join(&mod_name);
    let _ = fs::remove_dir_all(&out_dir); // rebuild the mod folder fresh each export
    // Nexus convention: the mod is just an `Umbra` folder dropped into <Wolcen>\Game\.
    let umbra_dir = out_dir.join("Umbra");
    fs::create_dir_all(&out_dir).with_context(|| format!("creating {}", out_dir.display()))?;

    let pak_path = out_dir.join(format!("zzz_{mod_name}.pak"));
    let pak_file = fs::File::create(&pak_path)?;
    let mut zip = zip::ZipWriter::new(pak_file);
    let opts = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    let mut total_changes = 0usize;
    for (file, rules) in &by_file {
        let xml = read_umbra_xml(cfg, file)?;
        let (modified, n) = rewrite(&xml, rules)?;
        total_changes += n;

        // pak entry: Umbra/<rel> with forward slashes
        zip.start_file(format!("Umbra/{file}"), opts)?;
        zip.write_all(modified.as_bytes())?;

        // loose file (Nexus style): <out>/Umbra/<rel>
        let loose_path = umbra_dir.join(file.replace('/', std::path::MAIN_SEPARATOR_STR));
        if let Some(p) = loose_path.parent() {
            fs::create_dir_all(p)?;
        }
        fs::write(&loose_path, modified.as_bytes())?;
    }
    zip.finish()?;

    let readme = format!(
        "Wolcen mod: {mod_name}\r\n\r\n\
         Install (pick ONE):\r\n\r\n\
         A) LOOSE FILES (Nexus style, recommended): copy the \"Umbra\" folder into\r\n   \
         <Wolcen>\\Game\\   (you'll get <Wolcen>\\Game\\Umbra\\Skills\\...)\r\n\r\n\
         B) PAK: copy \"zzz_{mod_name}.pak\" into <Wolcen>\\Game\\\r\n\r\n\
         Uninstall: delete the files you copied (A) or the pak (B).\r\n\
         Files changed: {}  |  Attribute edits: {total_changes}\r\n",
        by_file.len()
    );
    fs::write(out_dir.join("HOW_TO_INSTALL.txt"), readme)?;

    Ok(ExportResult {
        pak: pak_path.display().to_string(),
        folder: out_dir.display().to_string(),
        files: by_file.len(),
        changes: total_changes,
    })
}
