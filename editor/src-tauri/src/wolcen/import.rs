use std::collections::HashMap;
use std::path::Path;

use anyhow::Result;
use quick_xml::events::{BytesStart, Event};
use quick_xml::reader::Reader;
use serde::Serialize;
use walkdir::WalkDir;

use super::decode::read_umbra_xml;
use super::paths::Config;

#[derive(Serialize)]
pub struct SkillEditOut {
    pub file: String,
    pub uid: String,
    pub element: String,
    pub attr: String,
    pub value: f64,
}
#[derive(Serialize)]
pub struct PassiveEditOut {
    pub file: String,
    pub node: String,
    pub eim: String,
    pub attr: String,
    pub value: f64,
}
#[derive(Serialize)]
pub struct PlayerEditOut {
    pub file: String,
    pub element: String,
    pub attr: String,
    pub value: f64,
}

#[derive(Serialize, Default)]
pub struct ImportResult {
    pub skill_edits: Vec<SkillEditOut>,
    pub passive_edits: Vec<PassiveEditOut>,
    pub player_edits: Vec<PlayerEditOut>,
    pub files: usize,
    /// Files in the mod that couldn't be matched to the original data (skipped).
    pub skipped: Vec<String>,
}

impl ImportResult {
    fn total(&self) -> usize {
        self.skill_edits.len() + self.passive_edits.len() + self.player_edits.len()
    }
}

fn get_attr(e: &BytesStart, key: &[u8]) -> Option<String> {
    for a in e.attributes().flatten() {
        if a.key.as_ref() == key {
            return Some(a.unescape_value().unwrap_or_default().into_owned());
        }
    }
    None
}

/// Path -> rel under the pak's `Umbra/` root (e.g. Skills/NewSkills/Player/Foo.xml).
fn rel_from_path(path: &Path) -> Option<String> {
    let comps: Vec<String> = path
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect();
    let idx = comps.iter().position(|c| c.eq_ignore_ascii_case("Umbra"))?;
    let rel = comps[idx + 1..].join("/");
    if rel.is_empty() {
        None
    } else {
        Some(rel)
    }
}

fn key_of(lname: &[u8], e: &BytesStart) -> Option<String> {
    match lname {
        b"Skill" => get_attr(e, b"UID"),
        b"Spell" | b"EIM" => get_attr(e, b"Name"),
        _ => None,
    }
}

fn ctx_of(stack: &[(Vec<u8>, Option<String>)]) -> (Option<String>, Option<String>, Option<String>) {
    let (mut uid, mut spell, mut eim) = (None, None, None);
    for (ln, key) in stack.iter().rev() {
        match ln.as_slice() {
            b"Skill" if uid.is_none() => uid = key.clone(),
            b"Spell" if spell.is_none() => spell = key.clone(),
            b"EIM" if eim.is_none() => eim = key.clone(),
            _ => {}
        }
    }
    (uid, spell, eim)
}

fn compare(
    oe: &BytesStart,
    me: &BytesStart,
    lname: &[u8],
    stack: &[(Vec<u8>, Option<String>)],
    rel: &str,
    out: &mut ImportResult,
) {
    let mut omap: HashMap<Vec<u8>, String> = HashMap::new();
    for a in oe.attributes().flatten() {
        omap.insert(
            a.key.as_ref().to_vec(),
            a.unescape_value().unwrap_or_default().into_owned(),
        );
    }
    for a in me.attributes().flatten() {
        let key = a.key.as_ref().to_vec();
        let mv = a.unescape_value().unwrap_or_default().into_owned();
        if omap.get(&key).map(|s| s.as_str()) == Some(mv.as_str()) {
            continue; // unchanged
        }
        let val = match mv.parse::<f64>() {
            Ok(v) => v,
            Err(_) => continue,
        };
        let attr = String::from_utf8_lossy(&key).into_owned();
        let element = String::from_utf8_lossy(lname).into_owned();
        let (uid, spell, eim) = ctx_of(stack);

        if rel.contains("/NewSkills/Player/") {
            if let Some(uid) = uid {
                out.skill_edits.push(SkillEditOut {
                    file: rel.to_string(),
                    uid,
                    element,
                    attr,
                    value: val,
                });
            }
        } else if rel.contains("/Passive/PST/") {
            if lname == b"Semantics" {
                if let (Some(node), Some(eim)) = (spell, eim) {
                    out.passive_edits.push(PassiveEditOut {
                        file: rel.to_string(),
                        node,
                        eim,
                        attr,
                        value: val,
                    });
                }
            }
        } else if rel.ends_with("DefaultSheet.xml") {
            out.player_edits.push(PlayerEditOut {
                file: rel.to_string(),
                element,
                attr,
                value: val,
            });
        }
    }
}

/// Advance a reader to the next structural event (skipping text/whitespace/comments).
fn next_struct<'a>(reader: &mut Reader<&'a [u8]>) -> Result<Event<'a>> {
    loop {
        match reader.read_event()? {
            e @ (Event::Start(_) | Event::Empty(_) | Event::End(_) | Event::Eof) => return Ok(e),
            _ => continue,
        }
    }
}

fn diff(orig: &str, modx: &str, rel: &str, out: &mut ImportResult) -> Result<()> {
    let mut ro = Reader::from_str(orig);
    let mut rm = Reader::from_str(modx);
    let mut stack: Vec<(Vec<u8>, Option<String>)> = Vec::new();

    loop {
        let eo = next_struct(&mut ro)?;
        let em = next_struct(&mut rm)?;
        match (eo, em) {
            (Event::Eof, _) | (_, Event::Eof) => break,
            (Event::Start(oe), Event::Start(me)) => {
                let lname = me.local_name().as_ref().to_vec();
                compare(&oe, &me, &lname, &stack, rel, out);
                stack.push((lname.clone(), key_of(&lname, &me)));
            }
            (Event::Empty(oe), Event::Empty(me)) => {
                let lname = me.local_name().as_ref().to_vec();
                compare(&oe, &me, &lname, &stack, rel, out);
            }
            (Event::End(_), Event::End(_)) => {
                stack.pop();
            }
            _ => break, // structure diverged — stop safely
        }
    }
    Ok(())
}

/// Import a mod folder (containing an `Umbra\...` tree) by diffing each file
/// against the original game data to reconstruct the edits.
pub fn import_mod(cfg: &Config, mod_dir: &str) -> Result<ImportResult> {
    let mut out = ImportResult::default();
    for entry in WalkDir::new(mod_dir).into_iter().flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("xml") {
            continue;
        }
        let rel = match rel_from_path(path) {
            Some(r) => r,
            None => continue,
        };
        let orig = match read_umbra_xml(cfg, &rel) {
            Ok(x) => x,
            Err(_) => {
                out.skipped.push(rel);
                continue;
            }
        };
        let modx = match std::fs::read_to_string(path) {
            Ok(x) => x,
            Err(_) => continue,
        };
        let before = out.total();
        if diff(&orig, &modx, &rel, &mut out).is_ok() && out.total() > before {
            out.files += 1;
        }
    }
    Ok(out)
}
