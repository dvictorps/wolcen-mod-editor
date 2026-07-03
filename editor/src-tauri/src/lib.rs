pub mod wolcen;

use wolcen::passives::{PassiveSection, SectionSummary};
use wolcen::skills::{SkillDetail, SkillSummary};
use wolcen::Config;

fn cfg() -> Config {
    Config::dev()
}

fn to_str<T>(r: anyhow::Result<T>) -> Result<T, String> {
    r.map_err(|e| format!("{e:#}"))
}

#[tauri::command]
fn list_skills() -> Result<Vec<SkillSummary>, String> {
    to_str(wolcen::skills::list_skills(&cfg()))
}

#[tauri::command]
fn get_skill(name: String) -> Result<SkillDetail, String> {
    to_str(wolcen::skills::get_skill(&cfg(), &name))
}

#[tauri::command]
fn list_sections() -> Result<Vec<SectionSummary>, String> {
    to_str(wolcen::passives::list_sections(&cfg()))
}

#[tauri::command]
fn get_section(section: String) -> Result<PassiveSection, String> {
    to_str(wolcen::passives::get_section(&cfg(), &section))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            list_skills,
            get_skill,
            list_sections,
            get_section
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
