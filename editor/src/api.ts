import { invoke } from "@tauri-apps/api/core";

// Mirrors the Rust serde structs (snake_case fields).
export interface SkillSummary {
  internal_name: string;
  display_name: string;
  file: string;
}

export interface Field {
  element: string;
  attr: string;
  value: number;
}

export interface Variant {
  uid: string;
  number: number | null;
  name: string;
  description: string;
  fields: Field[];
}

export interface SkillDetail {
  internal_name: string;
  display_name: string;
  file: string;
  variants: Variant[];
}

export interface SectionSummary {
  name: string;
  file: string;
}

export interface NumField {
  attr: string;
  value: number;
}

export interface NodeEffect {
  eim: string;
  label: string;
  fields: NumField[];
}

export interface PassiveNode {
  name: string;
  display_name: string;
  rarity: number;
  angle: number;
  pos: number;
  unlock: string[];
  effects: NodeEffect[];
}

export interface PassiveSection {
  name: string;
  ui_name: string;
  pst_file: string;
  nodes: PassiveNode[];
}

export interface NodeDetail {
  node: string;
  display_name: string;
  file: string;
  effects: NodeEffect[];
}

export interface StatGroup {
  element: string;
  fields: NumField[];
}
export interface PlayerStats {
  file: string;
  groups: StatGroup[];
}

export interface SkillEditReq {
  file: string;
  uid: string;
  element: string;
  attr: string;
  value: number;
}
export interface PlayerEditReq {
  file: string;
  element: string;
  attr: string;
  value: number;
}
export interface PassiveEditReq {
  file: string;
  node: string;
  eim: string;
  attr: string;
  value: number;
}
export interface ExportRequest {
  mod_name: string;
  skill_edits: SkillEditReq[];
  passive_edits: PassiveEditReq[];
  player_edits: PlayerEditReq[];
}
export interface ExportResult {
  pak: string;
  folder: string;
  files: number;
  changes: number;
}

export interface AppStateInfo {
  game_dir: string | null;
  prepared: boolean;
  detected: string | null;
  tools_ok: boolean;
}

export const api = {
  getState: () => invoke<AppStateInfo>("get_state"),
  setGameDir: (dir: string) => invoke<void>("set_game_dir", { dir }),
  prepareData: () => invoke<void>("prepare_data"),
  listSkills: () => invoke<SkillSummary[]>("list_skills"),
  getSkill: (name: string) => invoke<SkillDetail>("get_skill", { name }),
  listSections: () => invoke<SectionSummary[]>("list_sections"),
  getSection: (section: string) => invoke<PassiveSection>("get_section", { section }),
  getNodeEffects: (section: string, node: string) =>
    invoke<NodeDetail>("get_node_effects", { section, node }),
  getPlayerStats: () => invoke<PlayerStats>("get_player_stats"),
  exportMod: (request: ExportRequest) => invoke<ExportResult>("export_mod", { request }),
};
