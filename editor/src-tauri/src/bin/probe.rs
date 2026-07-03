//! Headless verification harness: parses real game data with no GUI so we can
//! confirm the core works. Run with: `cargo run --bin wolcen_probe`.

use editor_lib::wolcen::export::{self, ExportRequest, SkillEdit};
use editor_lib::wolcen::{passives, skills, Config};

fn main() -> anyhow::Result<()> {
    let cfg = Config::dev();

    println!("== SKILLS ==");
    let list = skills::list_skills(&cfg)?;
    println!("player skills found: {}", list.len());
    for s in list.iter().take(8) {
        println!("  {} -> {}", s.internal_name, s.display_name);
    }

    println!("\n== BLEEDING EDGE (Laceration) ==");
    let bleed = skills::get_skill(&cfg, "Laceration")?;
    println!("display: {}  variants: {}", bleed.display_name, bleed.variants.len());
    for v in &bleed.variants {
        if v.number == Some(11) {
            println!("  >>> variant 11: {}", v.name);
            println!("      desc: {}", v.description);
            for f in &v.fields {
                println!("      [{}] {} = {}", f.element, f.attr, f.value);
            }
        }
    }

    println!("\n== PASSIVE SECTIONS ==");
    let sections = passives::list_sections(&cfg)?;
    println!("sections: {}", sections.len());
    let melee = passives::get_section(&cfg, "Melee")?;
    println!("Melee ui_name={} nodes={}", melee.ui_name, melee.nodes.len());
    if let Some(n) = melee.nodes.first() {
        println!(
            "  first node: {} '{}' rarity={} unlock={:?}",
            n.name, n.display_name, n.rarity, n.unlock
        );
    }

    println!("\n== NODE EFFECTS (Melee / MELEE_1) ==");
    let nd = passives::get_node_effects(&cfg, "Melee", "MELEE_1")?;
    println!("node {} = '{}'  file={}", nd.node, nd.display_name, nd.file);
    for eff in &nd.effects {
        println!("  EIM {} [{}]", eff.eim, eff.label);
        for f in &eff.fields {
            println!("     {} = {}", f.attr, f.value);
        }
    }

    println!("\n== EXPORT TEST ==");
    let req = ExportRequest {
        mod_name: "ProbeTest".into(),
        skill_edits: vec![SkillEdit {
            file: "Skills/NewSkills/Player/Player_Laceration.xml".into(),
            uid: "player_laceration_variant_11".into(),
            element: "BaseDamageMultiplier".into(),
            attr: "AdditionalMultiplierFactorPerAilmentStack".into(),
            value: 0.1,
        }],
        passive_edits: vec![],
        player_edits: vec![],
    };
    let res = export::export(&cfg, req)?;
    println!("pak: {}", res.pak);
    println!("files: {}  changes: {}", res.files, res.changes);

    println!("\n== IMPORT TEST (read the mod back into edits) ==");
    let imp = editor_lib::wolcen::import::import_mod(
        &cfg,
        r"E:\Desenvolvimento\WolcenModding\mods\ProbeTest",
    )?;
    println!("files: {}  skill_edits: {}", imp.files, imp.skill_edits.len());
    for e in &imp.skill_edits {
        println!("  {} | {} | {} = {}", e.uid, e.element, e.attr, e.value);
    }

    println!("\n== SELF-CONTAINED SETUP TEST (bundled exe -> decrypt real game) ==");
    println!("detected: {:?}", editor_lib::wolcen::setup::detect_game());
    let test_root = std::path::PathBuf::from(r"E:\Desenvolvimento\WolcenModding\editor\.setuptest");
    let scfg = Config {
        game_dir: std::path::PathBuf::from(r"D:\SteamLibrary\steamapps\common\Wolcen"),
        tools_bin: std::path::PathBuf::from(
            r"E:\Desenvolvimento\WolcenModding\editor\src-tauri\resources\tools",
        ),
        extracted_umbra: test_root.join("gamedata").join("Umbra"),
        localization_dir: test_root.join("gamedata").join("localization"),
        cache_dir: test_root.join("cache"),
        mods_dir: test_root.join("mods"),
        data_root: test_root.clone(),
    };
    editor_lib::wolcen::setup::prepare_data(&scfg)?;
    println!("prepared: {}", scfg.is_prepared());
    let _ = std::fs::remove_dir_all(&test_root);

    Ok(())
}
