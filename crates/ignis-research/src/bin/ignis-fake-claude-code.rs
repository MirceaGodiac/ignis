//! Test double of headless Claude Code (`claude -p --output-format stream-json`) for the offline
//! tests of `ClaudeBridge`. It checks the isolation flags and the cleared environment, keeps
//! sessions as files in `CLAUDE_CONFIG_DIR`, and acts on keywords in the prompt:
//! `tool` (one IGNIS tool call), `slow` (runs 30 s), `limit` (plan limit result), `crash` (exits
//! without a result). It appends its argv to `<CLAUDE_CONFIG_DIR>/argv.log` and the text of
//! `--append-system-prompt-file` to `<CLAUDE_CONFIG_DIR>/system-prompts.jsonl`.

use std::io::{Read, Write};
use std::path::PathBuf;

fn emit(v: serde_json::Value) {
    let mut out = std::io::stdout().lock();
    writeln!(out, "{v}").unwrap();
    out.flush().unwrap();
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("--version") {
        println!("2.1.273 (Claude Code)");
        return;
    }
    let cfg = PathBuf::from(std::env::var("CLAUDE_CONFIG_DIR").expect("CLAUDE_CONFIG_DIR"));
    std::fs::create_dir_all(cfg.join("sessions")).unwrap();
    let mut log = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(cfg.join("argv.log"))
        .unwrap();
    writeln!(log, "{}", serde_json::to_string(&args).unwrap()).unwrap();

    let flag = |f: &str| {
        args.iter()
            .position(|a| a == f)
            .map(|i| args.get(i + 1).cloned().unwrap_or_default())
    };
    let fail = |msg: &str| {
        eprintln!("{msg}");
        std::process::exit(2);
    };
    if flag("--tools").as_deref() != Some("") || !args.iter().any(|a| a == "--strict-mcp-config") {
        fail("isolation flags missing");
    }
    if std::env::var_os("IGNIS_TEST_HOST_SECRET").is_some() {
        fail("host environment leaked into the runtime");
    }
    let session = match (flag("--session-id"), flag("--resume")) {
        (Some(s), None) => s,
        (None, Some(s)) => {
            if !cfg.join("sessions").join(&s).exists() {
                fail(&format!("No conversation found with session ID: {s}"));
            }
            s
        }
        _ => fail("exactly one of --session-id / --resume"),
    };
    std::fs::write(cfg.join("sessions").join(&session), b"").unwrap();
    // Like the real CLI: a missing file fails the run; the text read is logged for the tests.
    if let Some(file) = flag("--append-system-prompt-file") {
        match std::fs::read_to_string(&file) {
            Ok(text) => {
                let mut f = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(cfg.join("system-prompts.jsonl"))
                    .unwrap();
                writeln!(
                    f,
                    "{}",
                    serde_json::json!({"session": session, "append": text})
                )
                .unwrap();
            }
            Err(_) => {
                fail(&format!(
                    "Error: Append system prompt file not found: {file}"
                ));
            }
        }
    }
    let mut prompt = String::new();
    std::io::stdin().read_to_string(&mut prompt).unwrap();

    emit(
        serde_json::json!({"type":"system","subtype":"init","session_id":session,"mcp_servers":[],"model":"fake-claude-model"}),
    );
    if prompt.contains("crash") {
        fail("Invalid API key · Please run /login");
    }
    if prompt.contains("tool") {
        emit(
            serde_json::json!({"type":"assistant","message":{"id":"m-tool","content":[
            {"type":"tool_use","id":"tu-1","name":"mcp__ignis__sources_search","input":{"query":"q"}}]}}),
        );
        emit(serde_json::json!({"type":"user","message":{"content":[
            {"type":"tool_result","tool_use_id":"tu-1","content":[{"type":"text","text":"{\"hits\":[]}"}]}]}}));
    }
    if prompt.contains("slow") {
        emit(
            serde_json::json!({"type":"assistant","message":{"id":"m-slow","content":[{"type":"text","text":"Working."}]}}),
        );
        std::thread::sleep(std::time::Duration::from_secs(30));
    }
    if prompt.contains("limit") {
        emit(
            serde_json::json!({"type":"result","subtype":"success","is_error":true,"result":"Claude usage limit reached","session_id":session}),
        );
        std::process::exit(1);
    }
    let first = prompt.lines().next().unwrap_or("").to_string();
    emit(
        serde_json::json!({"type":"assistant","message":{"id":format!("m-{}", uuid_like()),"content":[{"type":"text","text":format!("Answer: {first}")}]}}),
    );
    emit(
        serde_json::json!({"type":"result","subtype":"success","is_error":false,"result":"ok","session_id":session}),
    );
}

fn uuid_like() -> String {
    format!(
        "{:x}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    )
}
