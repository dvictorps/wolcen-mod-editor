use std::collections::HashMap;

use anyhow::{bail, Result};
use quick_xml::events::{BytesStart, Event};
use quick_xml::reader::Reader;
use serde::Serialize;
use walkdir::WalkDir;

use super::decode::read_umbra_xml;
use super::localization::{load_eim, load_passive_skills};
use super::paths::Config;

const PASSIVE_TREES_DIR: &str = "Skills/Trees/PassiveSkills";
const PST_DIR: &str = "Skills/Passive/PST";

/// One editable number of a node's magic effect.
#[derive(Serialize, Debug, Clone)]
pub struct NumField {
    pub attr: String,
    pub value: f64,
}

/// One magic effect (EIM) granted by a passive node.
#[derive(Serialize, Debug, Clone)]
pub struct NodeEffect {
    pub eim: String,
    pub label: String,
    pub fields: Vec<NumField>,
}

#[derive(Serialize, Debug, Clone)]
pub struct PassiveNode {
    pub name: String,
    pub display_name: String,
    pub rarity: u32,
    pub angle: f64,
    pub pos: f64,
    pub unlock: Vec<String>,
    pub effects: Vec<NodeEffect>,
}

#[derive(Serialize, Debug, Clone)]
pub struct PassiveSection {
    pub name: String,
    pub ui_name: String,
    /// PST file backing this section's node stats (needed for export edit keys).
    pub pst_file: String,
    pub nodes: Vec<PassiveNode>,
}

#[derive(Serialize, Debug, Clone)]
pub struct SectionSummary {
    pub name: String,
    pub file: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct NodeDetail {
    pub node: String,
    pub display_name: String,
    pub file: String,
    pub effects: Vec<NodeEffect>,
}

fn attr(e: &BytesStart, key: &[u8]) -> Option<String> {
    for a in e.attributes().flatten() {
        if a.key.as_ref() == key {
            return Some(a.unescape_value().unwrap_or_default().into_owned());
        }
    }
    None
}

pub fn list_sections(cfg: &Config) -> Result<Vec<SectionSummary>> {
    let dir = cfg.extracted_umbra.join(PASSIVE_TREES_DIR.replace('/', "\\"));
    let mut out = Vec::new();
    if !dir.exists() {
        return Ok(out);
    }
    for entry in WalkDir::new(&dir).max_depth(1).into_iter().flatten() {
        let name = match entry.path().file_name().and_then(|n| n.to_str()) {
            Some(n) => n,
            None => continue,
        };
        if !name.ends_with("_tree.xml") {
            continue;
        }
        out.push(SectionSummary {
            name: name.trim_end_matches("_tree.xml").to_string(),
            file: format!("{PASSIVE_TREES_DIR}/{name}"),
        });
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

fn pst_file_for(cfg: &Config, section: &str) -> Option<String> {
    let dir = cfg.extracted_umbra.join(PST_DIR.replace('/', "\\"));
    let want = format!("_{}.xml", section.to_lowercase());
    for entry in WalkDir::new(&dir).max_depth(1).into_iter().flatten() {
        let name = entry.path().file_name()?.to_str()?.to_string();
        if name.to_lowercase().ends_with(&want) {
            return Some(format!("{PST_DIR}/{name}"));
        }
    }
    None
}

/// Parse a section's PST file into node-name -> effects.
fn parse_pst(cfg: &Config, section: &str) -> Result<(String, HashMap<String, Vec<NodeEffect>>)> {
    let file = match pst_file_for(cfg, section) {
        Some(f) => f,
        None => return Ok((String::new(), HashMap::new())),
    };
    let xml = read_umbra_xml(cfg, &file)?;
    let eim_strings = load_eim(cfg).unwrap_or_default();

    let mut reader = Reader::from_str(&xml);
    let mut map: HashMap<String, Vec<NodeEffect>> = HashMap::new();
    let mut node: Option<String> = None;
    let mut effects: Vec<NodeEffect> = Vec::new();
    let mut cur_eim: Option<NodeEffect> = None;

    loop {
        match reader.read_event()? {
            Event::Start(e) | Event::Empty(e) => {
                let lname = e.local_name().as_ref().to_vec();
                match lname.as_slice() {
                    b"Spell" => {
                        node = attr(&e, b"Name");
                        effects = Vec::new();
                        cur_eim = None;
                    }
                    b"EIM" if node.is_some() => {
                        if let Some(eff) = cur_eim.take() {
                            effects.push(eff);
                        }
                        let name = attr(&e, b"Name").unwrap_or_default();
                        let label = attr(&e, b"HUDDesc")
                            .and_then(|k| eim_strings.get(&k).map(String::from))
                            .unwrap_or_else(|| name.clone());
                        cur_eim = Some(NodeEffect { eim: name, label, fields: Vec::new() });
                    }
                    b"Semantics" => {
                        if let Some(eff) = cur_eim.as_mut() {
                            for a in e.attributes().flatten() {
                                let v = a.unescape_value().unwrap_or_default();
                                if let Ok(n) = v.parse::<f64>() {
                                    eff.fields.push(NumField {
                                        attr: String::from_utf8_lossy(a.key.as_ref()).into_owned(),
                                        value: n,
                                    });
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            Event::End(e) => match e.local_name().as_ref() {
                b"EIM" => {
                    if let Some(eff) = cur_eim.take() {
                        effects.push(eff);
                    }
                }
                b"Spell" => {
                    if let Some(eff) = cur_eim.take() {
                        effects.push(eff);
                    }
                    if let Some(nm) = node.take() {
                        map.insert(nm, std::mem::take(&mut effects));
                    }
                }
                _ => {}
            },
            Event::Eof => break,
            _ => {}
        }
    }
    Ok((file, map))
}

pub fn get_section(cfg: &Config, section: &str) -> Result<PassiveSection> {
    let file = format!("{PASSIVE_TREES_DIR}/{section}_tree.xml");
    let xml = read_umbra_xml(cfg, &file)?;
    let strings = load_passive_skills(cfg).unwrap_or_default();
    let (pst_file, mut effects_map) = parse_pst(cfg, section).unwrap_or_default();

    let mut reader = Reader::from_str(&xml);
    let mut ui_name = String::new();
    let mut nodes: Vec<PassiveNode> = Vec::new();

    loop {
        match reader.read_event()? {
            Event::Start(e) | Event::Empty(e) => {
                let lname = e.local_name().as_ref().to_vec();
                if lname == b"Tree" {
                    ui_name = attr(&e, b"UIName").unwrap_or_default();
                } else if lname == b"Skill" {
                    let name = attr(&e, b"Name").unwrap_or_default();
                    if name.is_empty() {
                        continue;
                    }
                    let display_name = strings
                        .get(&format!("ui_{name}_name"))
                        .unwrap_or(&name)
                        .to_string();
                    let effects = effects_map.remove(&name).unwrap_or_default();
                    nodes.push(PassiveNode {
                        rarity: attr(&e, b"Rarity").and_then(|s| s.parse().ok()).unwrap_or(1),
                        angle: attr(&e, b"Angle").and_then(|s| s.parse().ok()).unwrap_or(0.5),
                        pos: attr(&e, b"Pos").and_then(|s| s.parse().ok()).unwrap_or(0.5),
                        unlock: attr(&e, b"Unlock")
                            .unwrap_or_default()
                            .split(',')
                            .filter(|s| !s.is_empty())
                            .map(String::from)
                            .collect(),
                        display_name,
                        name,
                        effects,
                    });
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }

    Ok(PassiveSection { name: section.to_string(), ui_name, pst_file, nodes })
}

/// Single-node effects (kept for the headless probe / direct queries).
pub fn get_node_effects(cfg: &Config, section: &str, node: &str) -> Result<NodeDetail> {
    let (file, mut map) = parse_pst(cfg, section)?;
    if file.is_empty() {
        bail!("no PST file for section {section}");
    }
    let strings = load_passive_skills(cfg).unwrap_or_default();
    let display_name = strings.get(&format!("ui_{node}_name")).unwrap_or(node).to_string();
    Ok(NodeDetail {
        node: node.to_string(),
        display_name,
        file,
        effects: map.remove(node).unwrap_or_default(),
    })
}
