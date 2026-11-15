//! Size of the fixed per-request prefix the runtime re-sends with every model request: the IGNIS
//! base instructions (Codex), the harness bundle (investigator, tool section, skill index) and the
//! MCP tool definitions (description + slim input schema). Prints a table with
//! `cargo test -p ignis-mcp --test prefix_size -- --nocapture` and fails when the prefix grows past
//! its ceiling, so a longer description is a deliberate change (docs/research/HARNESS_COST.md).
//!
//! Tokens are estimated as bytes / 4 (English prose and JSON land between 3.5 and 4.5 bytes per
//! token with the GPT and Claude tokenizers); the live `usage.reported` counts are the truth.

use ignis_core::contracts::tool_catalog;
use ignis_mcp::schemas::input_schema;
use ignis_research::runtime::{codex_base_instructions, HarnessBundle, ToolDescriptions};
use serde_json::json;

fn tok(bytes: usize) -> usize {
    bytes.div_ceil(4)
}

/// Every catalog tool as the runtime receives it over MCP (`tools/list`).
fn tool_defs() -> Vec<(String, usize, usize)> {
    let mut v: Vec<_> = tool_catalog()
        .into_iter()
        .map(|d| {
            let schema = input_schema(&d.input_contract).expect("a schema per contract");
            let def = json!({"name": d.name, "description": d.summary, "inputSchema": schema});
            let total = serde_json::to_vec(&def).unwrap().len();
            (d.name.clone(), d.summary.len(), total)
        })
        .collect();
    v.sort_by(|a, b| b.2.cmp(&a.2));
    v
}

/// Ceilings (bytes) of the parts of the prefix; raise one only on purpose.
const MAX_HARNESS_BYTES: usize = 35_500;
const MAX_TOOL_DEFS_BYTES: usize = 46_500;

#[test]
fn the_fixed_prefix_stays_within_its_budget() {
    let names: Vec<String> = tool_catalog().into_iter().map(|t| t.name).collect();
    let bundle = HarnessBundle::embedded(&names);
    let text = bundle.text();
    let section = |start: &str, end: Option<&str>| -> usize {
        let Some(i) = text.find(start) else { return 0 };
        let rest = &text[i..];
        end.and_then(|e| rest[1..].find(e).map(|j| j + 1))
            .unwrap_or(rest.len())
    };
    let tools_section = section("## IGNIS tools", Some("\n## Skills"));
    let skills_index = section("\n## Skills", None);
    let investigator = text.len() - tools_section - skills_index;
    let base = codex_base_instructions().len();
    let defs = tool_defs();
    let defs_total: usize = defs.iter().map(|d| d.2).sum();
    // Claude Code sends the definitions itself, so its prefix lists the tools only.
    let claude = HarnessBundle::embedded_with(&names, ToolDescriptions::Index);

    println!("\nFixed prefix (bytes, ~tokens = bytes/4)");
    println!("{:<40} {:>8} {:>8}", "part", "bytes", "~tokens");
    for (k, b) in [
        ("codex base instructions", base),
        ("harness: investigator", investigator),
        ("harness: tool section", tools_section),
        ("harness: skill index", skills_index),
        ("harness total", text.len()),
        ("MCP tool definitions (all)", defs_total),
        ("Codex: base + harness (tools deferred)", base + text.len()),
        ("Claude: harness (index section)", claude.text().len()),
        (
            "Claude: harness + tool definitions",
            claude.text().len() + defs_total,
        ),
    ] {
        println!("{k:<40} {b:>8} {:>8}", tok(b));
    }
    println!(
        "\n{:<20} {:>12} {:>12} {:>8}",
        "tool", "description", "definition", "~tokens"
    );
    for (n, d, t) in &defs {
        println!("{n:<20} {d:>12} {t:>12} {:>8}", tok(*t));
    }
    for s in bundle.skills() {
        println!(
            "skill {:<28} {:>6} bytes (served on demand by skills_read)",
            s.name,
            s.body.len()
        );
    }

    // `IGNIS_PREFIX_DUMP=<dir>`: write the prefix parts as the runtime receives them, e.g. to
    // replay them through a capturing model provider.
    if let Some(dir) = std::env::var_os("IGNIS_PREFIX_DUMP") {
        let dir = std::path::PathBuf::from(dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("harness.md"), text).unwrap();
        std::fs::write(dir.join("codex_base.md"), codex_base_instructions()).unwrap();
        let tools: Vec<_> = tool_catalog()
            .into_iter()
            .map(|d| {
                let s = input_schema(&d.input_contract).unwrap();
                json!({"name": d.name, "description": d.summary, "inputSchema": s})
            })
            .collect();
        std::fs::write(
            dir.join("tools.json"),
            serde_json::to_vec_pretty(&tools).unwrap(),
        )
        .unwrap();
    }

    assert!(
        text.len() <= MAX_HARNESS_BYTES,
        "harness prefix {} bytes > {MAX_HARNESS_BYTES}",
        text.len()
    );
    assert!(claude.text().len() < text.len());
    assert!(
        defs_total <= MAX_TOOL_DEFS_BYTES,
        "tool definitions {defs_total} bytes > {MAX_TOOL_DEFS_BYTES}"
    );
}
