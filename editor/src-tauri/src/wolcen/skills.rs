use anyhow::{bail, Result};
use quick_xml::events::{BytesStart, Event};
use quick_xml::reader::Reader;
use serde::Serialize;
use walkdir::WalkDir;

use super::decode::read_umbra_xml;
use super::localization::{load_active_skills, Strings};
use super::paths::Config;

const PLAYER_SKILLS_DIR: &str = "Skills/NewSkills/Player";

/// One editable numeric parameter of a perk (e.g. damage-per-ailment-stack).
#[derive(Serialize, Debug, Clone)]
pub struct Field {
    /// Owning effect element, e.g. "BaseDamageMultiplier".
    pub element: String,
    /// Attribute name, e.g. "AdditionalMultiplierFactorPerAilmentStack".
    pub attr: String,
    pub value: f64,
}

/// A perk (variant) of a skill.
#[derive(Serialize, Debug, Clone)]
pub struct Variant {
    pub uid: String,
    pub number: Option<u32>,
    pub name: String,
    pub description: String,
    pub fields: Vec<Field>,
}

#[derive(Serialize, Debug, Clone)]
pub struct SkillDetail {
    pub internal_name: String,
    pub display_name: String,
    pub file: String,
    pub variants: Vec<Variant>,
}

#[derive(Serialize, Debug, Clone)]
pub struct SkillSummary {
    pub internal_name: String,
    pub display_name: String,
    pub file: String,
}

fn attr<'a>(e: &'a BytesStart, key: &[u8]) -> Option<String> {
    for a in e.attributes().flatten() {
        if a.key.as_ref() == key {
            return Some(a.unescape_value().unwrap_or_default().into_owned());
        }
    }
    None
}

/// internal name from "Player_Laceration.xml" -> "Laceration".
fn internal_from_file(file_stem: &str) -> String {
    file_stem
        .strip_prefix("Player_")
        .unwrap_or(file_stem)
        .to_string()
}

/// List all player active skills (name resolved via localization `ui_AST_<Name>`).
pub fn list_skills(cfg: &Config) -> Result<Vec<SkillSummary>> {
    let strings = load_active_skills(cfg)?;
    let dir = cfg.extracted_umbra.join(PLAYER_SKILLS_DIR.replace('/', "\\"));
    let mut out = Vec::new();
    if !dir.exists() {
        return Ok(out);
    }
    for entry in WalkDir::new(&dir).max_depth(1).into_iter().flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n,
            None => continue,
        };
        if !name.starts_with("Player_") || !name.ends_with(".xml") {
            continue;
        }
        let stem = &name[..name.len() - 4];
        let internal = internal_from_file(stem);
        let display = strings
            .get(&format!("ui_AST_{internal}"))
            .unwrap_or(&internal)
            .to_string();
        out.push(SkillSummary {
            internal_name: internal,
            display_name: display,
            file: format!("{PLAYER_SKILLS_DIR}/{name}"),
        });
    }
    out.sort_by(|a, b| a.display_name.to_lowercase().cmp(&b.display_name.to_lowercase()));
    Ok(out)
}

fn variant_number(uid: &str) -> Option<u32> {
    uid.rsplit("_variant_").next().and_then(|s| s.parse().ok())
}

/// Elements that carry no editable gameplay numbers (presentation/FX/audio).
fn is_presentational(name: &[u8]) -> bool {
    matches!(
        name,
        b"HUD" | b"Particle" | b"ParticleList" | b"SoundTrigger" | b"Animation" | b"Entry"
    )
}

fn record_fields(e: &BytesStart, lname: &[u8], v: &mut Variant) {
    if is_presentational(lname) {
        return;
    }
    for a in e.attributes().flatten() {
        let val = a.unescape_value().unwrap_or_default();
        if let Ok(num) = val.parse::<f64>() {
            v.fields.push(Field {
                element: String::from_utf8_lossy(lname).into_owned(),
                attr: String::from_utf8_lossy(a.key.as_ref()).into_owned(),
                value: num,
            });
        }
    }
}

fn begin_variant(e: &BytesStart, internal: &str, strings: &Strings) -> Option<Variant> {
    let uid = attr(e, b"UID")?;
    if !uid.contains("_variant_") {
        return None;
    }
    let number = variant_number(&uid);
    let name_key = number
        .map(|n| format!("ui_Variant_{internal}_variant_{n}"))
        .unwrap_or_default();
    let name = strings.get(&name_key).unwrap_or("").to_string();
    let description = strings.get(&format!("{name_key}_desc")).unwrap_or("").to_string();
    Some(Variant {
        uid,
        number,
        name,
        description,
        fields: Vec::new(),
    })
}

fn parse_variants(xml: &str, internal: &str, strings: &Strings) -> Result<Vec<Variant>> {
    let mut reader = Reader::from_str(xml);
    let mut variants: Vec<Variant> = Vec::new();
    let mut current: Option<Variant> = None;
    // When inside a child container element (e.g. Damage_Conversion) we skip its
    // grandchildren so only direct-child effect elements of a variant are read.
    let mut skip_until: Option<Vec<u8>> = None;

    loop {
        let ev = reader.read_event()?;
        match ev {
            Event::Start(e) => {
                let lname = e.local_name().as_ref().to_vec();
                if skip_until.is_some() {
                    continue; // inside a skipped container
                }
                if lname == b"Skill" {
                    // Any Skill boundary closes the previous variant's collection.
                    if let Some(v) = current.take() {
                        variants.push(v);
                    }
                    current = begin_variant(&e, internal, strings);
                } else if let Some(v) = current.as_mut() {
                    record_fields(&e, &lname, v);
                    // This start-tag has children; skip them until its close.
                    skip_until = Some(lname);
                }
            }
            Event::Empty(e) => {
                if skip_until.is_some() {
                    continue;
                }
                let lname = e.local_name().as_ref().to_vec();
                if lname == b"Skill" {
                    if let Some(v) = current.take() {
                        variants.push(v);
                    }
                    current = begin_variant(&e, internal, strings);
                } else if let Some(v) = current.as_mut() {
                    record_fields(&e, &lname, v);
                }
            }
            Event::End(e) => {
                let lname = e.local_name().as_ref().to_vec();
                if let Some(open) = &skip_until {
                    if *open == lname {
                        skip_until = None;
                    }
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }
    if let Some(v) = current.take() {
        variants.push(v);
    }
    Ok(variants)
}

pub fn get_skill(cfg: &Config, internal_name: &str) -> Result<SkillDetail> {
    let file = format!("{PLAYER_SKILLS_DIR}/Player_{internal_name}.xml");
    let xml = read_umbra_xml(cfg, &file)?;
    if xml.is_empty() {
        bail!("empty skill file: {file}");
    }
    let strings = load_active_skills(cfg)?;
    let display = strings
        .get(&format!("ui_AST_{internal_name}"))
        .unwrap_or(internal_name)
        .to_string();
    let mut variants = parse_variants(&xml, internal_name, &strings)?;
    variants.sort_by_key(|v| v.number.unwrap_or(u32::MAX));
    Ok(SkillDetail {
        internal_name: internal_name.to_string(),
        display_name: display,
        file,
        variants,
    })
}
