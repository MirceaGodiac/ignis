//! Test double for `claude -p` (built as `ignis-fake-claude`, selected with `IGNIS_CLAUDE_BIN`).
//! The behaviour is taken from the item id in the prompt (`agree`, `disagree`, `fabricate`, `recover`,
//! `ratelimit`); every call is counted in `<IGNIS_FAKE_DIR>/calls.log`.

use std::io::{Read, Write};

fn result(text: &str, is_error: bool) -> String {
    serde_json::json!({
        "type": "result", "subtype": "success", "is_error": is_error, "num_turns": 1, "duration_ms": 12, "result": text,
        "usage": {"input_tokens": 100, "output_tokens": 20, "cache_read_input_tokens": 0, "cache_creation_input_tokens": 0},
        "modelUsage": {"fake-model": {}},
    })
    .to_string()
}

fn main() {
    let mut prompt = String::new();
    std::io::stdin().read_to_string(&mut prompt).unwrap();
    let field = |key: &str| {
        prompt
            .lines()
            .find_map(|l| l.strip_prefix(key))
            .unwrap_or("")
            .trim()
            .to_string()
    };
    let id = field("ITEM: ");
    let candidates: Vec<String> = field("CANDIDATES: ")
        .split(" | ")
        .map(String::from)
        .collect();
    let dir = std::env::var("IGNIS_FAKE_DIR").unwrap();
    let mut log = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(format!("{dir}/calls.log"))
        .unwrap();
    writeln!(log, "{id}").unwrap();
    drop(log);
    let calls = std::fs::read_to_string(format!("{dir}/calls.log"))
        .unwrap()
        .lines()
        .filter(|l| *l == id)
        .count();
    if id.contains("ratelimit") {
        println!("{}", result("You've hit your limit", true));
        std::process::exit(1);
    }
    let first_line = prompt
        .lines()
        .find(|l| l.starts_with("[L0001] "))
        .unwrap_or("[L0001] x");
    let quote = first_line
        .trim_start_matches("[L0001] ")
        .split_whitespace()
        .take(2)
        .collect::<Vec<_>>()
        .join(" ");
    let value = if id.contains("disagree") {
        candidates[(calls - 1) % candidates.len()].clone()
    } else {
        candidates[0].clone()
    };
    let fabricated = id.contains("fabricate") || (id.contains("recover") && calls == 1);
    let line_id = if fabricated { "L9999" } else { "L0001" };
    let answer = serde_json::json!({"value": value, "rationale": "fixture", "evidence": [{"id": line_id, "quote": quote}]});
    println!("{}", result(&answer.to_string(), false));
}
