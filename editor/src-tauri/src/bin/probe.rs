//! Headless verification harness: parses real game data with no GUI so we can
//! confirm the core works. Run with: `cargo run --bin wolcen_probe`.

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
            "  first node: {} rarity={} angle={} pos={} unlock={:?}",
            n.name, n.rarity, n.angle, n.pos, n.unlock
        );
    }

    Ok(())
}
