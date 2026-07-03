use anyhow::Result;
use quick_xml::events::{BytesStart, Event};
use quick_xml::reader::Reader;
use serde::Serialize;
use walkdir::WalkDir;

use super::decode::read_umbra_xml;
use super::paths::Config;

const PASSIVE_TREES_DIR: &str = "Skills/Trees/PassiveSkills";

#[derive(Serialize, Debug, Clone)]
pub struct PassiveNode {
    pub name: String,
    pub rarity: u32,
    /// 0..1 within the section; drives radial layout.
    pub angle: f64,
    /// 0..1 along the spoke; drives radial layout.
    pub pos: f64,
    /// Connected node names (graph edges) — includes ring anchors like "begin".
    pub unlock: Vec<String>,
}

#[derive(Serialize, Debug, Clone)]
pub struct PassiveSection {
    pub name: String,
    pub ui_name: String,
    pub nodes: Vec<PassiveNode>,
}

#[derive(Serialize, Debug, Clone)]
pub struct SectionSummary {
    pub name: String,
    pub file: String,
}

fn attr(e: &BytesStart, key: &[u8]) -> Option<String> {
    for a in e.attributes().flatten() {
        if a.key.as_ref() == key {
            return Some(a.unescape_value().unwrap_or_default().into_owned());
        }
    }
    None
}

/// List the 21 passive sub-tree sections (Melee, Warrior, ...).
pub fn list_sections(cfg: &Config) -> Result<Vec<SectionSummary>> {
    let dir = cfg.extracted_umbra.join(PASSIVE_TREES_DIR.replace('/', "\\"));
    let mut out = Vec::new();
    if !dir.exists() {
        return Ok(out);
    }
    for entry in WalkDir::new(&dir).max_depth(1).into_iter().flatten() {
        let path = entry.path();
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n,
            None => continue,
        };
        if !name.ends_with("_tree.xml") {
            continue;
        }
        let section = name.trim_end_matches("_tree.xml").to_string();
        out.push(SectionSummary {
            name: section,
            file: format!("{PASSIVE_TREES_DIR}/{name}"),
        });
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

pub fn get_section(cfg: &Config, section: &str) -> Result<PassiveSection> {
    let file = format!("{PASSIVE_TREES_DIR}/{section}_tree.xml");
    let xml = read_umbra_xml(cfg, &file)?;

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
                    let rarity = attr(&e, b"Rarity").and_then(|s| s.parse().ok()).unwrap_or(1);
                    let angle = attr(&e, b"Angle").and_then(|s| s.parse().ok()).unwrap_or(0.5);
                    let pos = attr(&e, b"Pos").and_then(|s| s.parse().ok()).unwrap_or(0.5);
                    let unlock = attr(&e, b"Unlock")
                        .unwrap_or_default()
                        .split(',')
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string())
                        .collect();
                    nodes.push(PassiveNode {
                        name,
                        rarity,
                        angle,
                        pos,
                        unlock,
                    });
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }

    Ok(PassiveSection {
        name: section.to_string(),
        ui_name,
        nodes,
    })
}
