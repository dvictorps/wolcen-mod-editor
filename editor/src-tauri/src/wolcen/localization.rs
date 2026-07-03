use std::collections::HashMap;

use anyhow::Result;
use quick_xml::events::Event;
use quick_xml::reader::Reader;

use super::decode::read_localization;
use super::paths::Config;

/// Localization strings: internal key -> English text.
///
/// The localization files are SpreadsheetML (Excel XML), plain text. Each `<Row>`
/// holds `<Cell><Data>KEY</Data></Cell><Cell><Data>VALUE</Data></Cell>`.
#[derive(Default, Debug)]
pub struct Strings {
    map: HashMap<String, String>,
}

impl Strings {
    pub fn get(&self, key: &str) -> Option<&str> {
        let key = key.strip_prefix('@').unwrap_or(key);
        self.map.get(key).map(|s| s.as_str())
    }

    pub fn get_or<'a>(&'a self, key: &'a str, fallback: &'a str) -> &'a str {
        self.get(key).unwrap_or(fallback)
    }
}

fn parse_sheet(xml: &str, out: &mut HashMap<String, String>) -> Result<()> {
    let mut reader = Reader::from_str(xml);
    let mut in_data = false;
    let mut buf = String::new();
    let mut row: Vec<String> = Vec::new();

    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                let name = e.local_name();
                match name.as_ref() {
                    b"Row" => row.clear(),
                    b"Data" => {
                        in_data = true;
                        buf.clear();
                    }
                    _ => {}
                }
            }
            Event::Text(e) => {
                if in_data {
                    buf.push_str(&e.unescape().unwrap_or_default());
                }
            }
            Event::End(e) => {
                let name = e.local_name();
                match name.as_ref() {
                    b"Data" => {
                        in_data = false;
                        row.push(buf.clone());
                    }
                    b"Row" => {
                        if row.len() >= 2 && !row[0].is_empty() {
                            out.insert(row[0].clone(), row[1].clone());
                        }
                    }
                    _ => {}
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }
    Ok(())
}

/// Load the active-skills strings (skill + variant names and descriptions).
pub fn load_active_skills(cfg: &Config) -> Result<Strings> {
    let mut map = HashMap::new();
    let xml = read_localization(cfg, "text_ui_Activeskills.xml")?;
    parse_sheet(&xml, &mut map)?;
    Ok(Strings { map })
}

/// Load the passive-skills strings (section + node names).
pub fn load_passive_skills(cfg: &Config) -> Result<Strings> {
    let mut map = HashMap::new();
    if let Ok(xml) = read_localization(cfg, "text_ui_passiveskills.xml") {
        parse_sheet(&xml, &mut map)?;
    }
    Ok(Strings { map })
}

/// Load the EIM (magic-effect) descriptions used by passive nodes.
pub fn load_eim(cfg: &Config) -> Result<Strings> {
    let mut map = HashMap::new();
    if let Ok(xml) = read_localization(cfg, "text_ui_EIM.xml") {
        parse_sheet(&xml, &mut map)?;
    }
    Ok(Strings { map })
}
