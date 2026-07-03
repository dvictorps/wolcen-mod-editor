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

export interface PassiveNode {
  name: string;
  rarity: number;
  angle: number;
  pos: number;
  unlock: string[];
}

export interface PassiveSection {
  name: string;
  ui_name: string;
  nodes: PassiveNode[];
}

export const api = {
  listSkills: () => invoke<SkillSummary[]>("list_skills"),
  getSkill: (name: string) => invoke<SkillDetail>("get_skill", { name }),
  listSections: () => invoke<SectionSummary[]>("list_sections"),
  getSection: (section: string) => invoke<PassiveSection>("get_section", { section }),
};
