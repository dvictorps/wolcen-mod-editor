use anyhow::Result;
use quick_xml::events::{BytesStart, Event};
use quick_xml::reader::Reader;
use serde::Serialize;

use super::decode::read_umbra_xml;
use super::paths::Config;

const PLAYER_SHEET: &str = "CharacterSheets/Player/DefaultSheet.xml";

#[derive(Serialize, Clone, Debug)]
pub struct PlayerField {
    pub attr: String,
    pub value: f64,
}

#[derive(Serialize, Clone, Debug)]
pub struct StatGroup {
    pub element: String,
    pub fields: Vec<PlayerField>,
}

#[derive(Serialize, Clone, Debug)]
pub struct PlayerStats {
    pub file: String,
    pub groups: Vec<StatGroup>,
}

fn group_from(e: &BytesStart, lname: &[u8]) -> Option<StatGroup> {
    if lname == b"Progression" {
        return None; // character level/xp, not a stat to mod
    }
    let mut fields = Vec::new();
    for a in e.attributes().flatten() {
        let v = a.unescape_value().unwrap_or_default();
        if let Ok(n) = v.parse::<f64>() {
            fields.push(PlayerField {
                attr: String::from_utf8_lossy(a.key.as_ref()).into_owned(),
                value: n,
            });
        }
    }
    if fields.is_empty() {
        return None;
    }
    Some(StatGroup {
        element: String::from_utf8_lossy(lname).into_owned(),
        fields,
    })
}

/// Editable base player stats (direct children of <PlayerStats> in DefaultSheet.xml).
pub fn get_player_stats(cfg: &Config) -> Result<PlayerStats> {
    let xml = read_umbra_xml(cfg, PLAYER_SHEET)?;
    let mut reader = Reader::from_str(&xml);
    let mut stack: Vec<Vec<u8>> = Vec::new();
    let mut groups: Vec<StatGroup> = Vec::new();

    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                let lname = e.local_name().as_ref().to_vec();
                if stack.last().map(|s| s.as_slice()) == Some(b"PlayerStats".as_ref()) {
                    if let Some(g) = group_from(&e, &lname) {
                        groups.push(g);
                    }
                }
                stack.push(lname);
            }
            Event::Empty(e) => {
                let lname = e.local_name().as_ref().to_vec();
                if stack.last().map(|s| s.as_slice()) == Some(b"PlayerStats".as_ref()) {
                    if let Some(g) = group_from(&e, &lname) {
                        groups.push(g);
                    }
                }
            }
            Event::End(_) => {
                stack.pop();
            }
            Event::Eof => break,
            _ => {}
        }
    }

    Ok(PlayerStats {
        file: PLAYER_SHEET.to_string(),
        groups,
    })
}
