pub mod wolcen;

use tauri::AppHandle;

use wolcen::export::{ExportRequest, ExportResult};
use wolcen::passives::{NodeDetail, PassiveSection, SectionSummary};
use wolcen::player::PlayerStats;
use wolcen::setup::AppState;
use wolcen::skills::{SkillDetail, SkillSummary};
use wolcen::Config;

fn cfg(app: &AppHandle) -> Config {
    Config::from_app(app)
}

fn to_str<T>(r: anyhow::Result<T>) -> Result<T, String> {
    r.map_err(|e| format!("{e:#}"))
}

// --- setup / first run ---

#[tauri::command]
fn get_state(app: AppHandle) -> AppState {
    wolcen::setup::get_state(&cfg(&app))
}

#[tauri::command]
fn set_game_dir(app: AppHandle, dir: String) -> Result<(), String> {
    to_str(cfg(&app).save_game_dir(&dir))
}

#[tauri::command]
fn prepare_data(app: AppHandle) -> Result<(), String> {
    to_str(wolcen::setup::prepare_data(&cfg(&app)))
}

// --- editor ---

#[tauri::command]
fn list_skills(app: AppHandle) -> Result<Vec<SkillSummary>, String> {
    to_str(wolcen::skills::list_skills(&cfg(&app)))
}

#[tauri::command]
fn get_skill(app: AppHandle, name: String) -> Result<SkillDetail, String> {
    to_str(wolcen::skills::get_skill(&cfg(&app), &name))
}

#[tauri::command]
fn list_sections(app: AppHandle) -> Result<Vec<SectionSummary>, String> {
    to_str(wolcen::passives::list_sections(&cfg(&app)))
}

#[tauri::command]
fn get_section(app: AppHandle, section: String) -> Result<PassiveSection, String> {
    to_str(wolcen::passives::get_section(&cfg(&app), &section))
}

#[tauri::command]
fn get_node_effects(app: AppHandle, section: String, node: String) -> Result<NodeDetail, String> {
    to_str(wolcen::passives::get_node_effects(&cfg(&app), &section, &node))
}

#[tauri::command]
fn get_player_stats(app: AppHandle) -> Result<PlayerStats, String> {
    to_str(wolcen::player::get_player_stats(&cfg(&app)))
}

#[tauri::command]
fn export_mod(app: AppHandle, request: ExportRequest) -> Result<ExportResult, String> {
    to_str(wolcen::export::export(&cfg(&app), request))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_state,
            set_game_dir,
            prepare_data,
            list_skills,
            get_skill,
            list_sections,
            get_section,
            get_node_effects,
            get_player_stats,
            export_mod
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
