//! Test double of headless Claude Code (`claude -p --output-format stream-json`) for the offline
//! tests of `ClaudeBridge`. It checks the isolation flags and the cleared environment, keeps
//! sessions as files in `CLAUDE_CONFIG_DIR`, and acts on keywords in the prompt:
//! `tool` (one IGNIS tool call), `usage` (the result carries usage and cost), `delegate` (a subtask through the subagent tool), `nested` (a
//! subagent that tries to delegate again), `explain` (a message, then a call it introduces, with arguments
//! larger than an event inlines), `slow` (runs 30 s), `limit` (plan limit result), `crash` (exits
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
    // The only built-in tool ever enabled is the subagent tool, and only with an IGNIS subagent
    // definition whose tools are IGNIS MCP tools and include no subagent tool.
    let tools_ok = match flag("--tools").as_deref() {
        Some("") => flag("--agents").is_none(),
        Some("Agent") => flag("--agents").is_some_and(|a| {
            let v: serde_json::Value = serde_json::from_str(&a).unwrap_or_default();
            let defs = v.as_object().cloned().unwrap_or_default();
            !defs.is_empty()
                && defs.values().all(|d| {
                    d["tools"].as_array().is_some_and(|t| {
                        t.iter().all(|n| {
                            n.as_str().is_some_and(|n| {
                                n.starts_with("mcp__ignis__") && n != "mcp__ignis__answer_submit"
                            })
                        })
                    })
                })
        }),
        _ => false,
    };
    if !tools_ok || !args.iter().any(|a| a == "--strict-mcp-config") {
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
    // `delegate`: one subtask through the subagent tool, the subagent makes an IGNIS call and
    // reports; `nested` adds a subagent trying to delegate again (the host must stop the turn).
    if prompt.contains("delegate") || prompt.contains("nested") {
        if flag("--agents").is_none() {
            fail("delegation requested without a subagent definition");
        }
        let sub = "tu-agent-1";
        emit(
            serde_json::json!({"type":"assistant","parent_tool_use_id":null,"message":{"id":"m-del","content":[
            {"type":"text","text":"Delegating the aggregation."},
            {"type":"tool_use","id":sub,"name":"Agent","input":{"subagent_type":"ignis-subinvestigator","description":"aggregate","prompt":"aggregate burn rates"}}]}}),
        );
        emit(
            serde_json::json!({"type":"assistant","parent_tool_use_id":sub,"message":{"id":"m-sub-1","content":[
            {"type":"text","text":"Sub: searching."},
            {"type":"tool_use","id":"tu-sub-1","name":"mcp__ignis__sources_search","input":{"query":"burn rate"}}]}}),
        );
        emit(
            serde_json::json!({"type":"user","parent_tool_use_id":sub,"message":{"content":[
            {"type":"tool_result","tool_use_id":"tu-sub-1","content":[{"type":"text","text":"{\"hits\":[]}"}]}]}}),
        );
        if prompt.contains("nested") {
            emit(
                serde_json::json!({"type":"assistant","parent_tool_use_id":sub,"message":{"id":"m-sub-2","content":[
                {"type":"tool_use","id":"tu-sub-2","name":"Agent","input":{"prompt":"go deeper"}}]}}),
            );
            // A real CLI would keep going; the host must stop it here.
            std::thread::sleep(std::time::Duration::from_secs(30));
        }
        emit(
            serde_json::json!({"type":"user","parent_tool_use_id":null,"message":{"content":[
            {"type":"tool_result","tool_use_id":sub,"content":[{"type":"text","text":"Report: no hits for burn rate."}]}]}}),
        );
    }
    if prompt.contains("tool") {
        emit(
            serde_json::json!({"type":"assistant","message":{"id":"m-tool","content":[
            {"type":"tool_use","id":"tu-1","name":"mcp__ignis__sources_search","input":{"query":"q"}}]}}),
        );
        emit(serde_json::json!({"type":"user","message":{"content":[
            {"type":"tool_result","tool_use_id":"tu-1","content":[{"type":"text","text":"{\"hits\":[]}"}]}]}}));
    }
    if prompt.contains("explain") {
        let source = format!(
            "print(1)
{}",
            "# padding
"
            .repeat(10_000)
        );
        emit(
            serde_json::json!({"type":"assistant","message":{"id":"m-explain","content":[
            {"type":"thinking","thinking":"hidden reasoning that is never logged"},
            {"type":"text","text":"Computing the spread rate from the cited table."},
            {"type":"tool_use","id":"tu-x","name":"mcp__ignis__analysis_execute",
             "input":{"language":"python","source":source}}]}}),
        );
        emit(serde_json::json!({"type":"user","message":{"content":[
            {"type":"tool_result","tool_use_id":"tu-x","content":[{"type":"text","text":"{\"ok\":true}"}]}]}}));
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
    let mut result = serde_json::json!({"type":"result","subtype":"success","is_error":false,"result":"ok","session_id":session});
    if prompt.contains("usage") {
        // What Claude Code reports: Anthropic usage (cache reads/writes apart from input) and
        // per-model usage with its own cost.
        result["usage"] = serde_json::json!({"input_tokens": 200, "cache_read_input_tokens": 6000,
            "cache_creation_input_tokens": 1800, "output_tokens": 500});
        result["total_cost_usd"] = serde_json::json!(0.0254);
        result["modelUsage"] = serde_json::json!({"claude-opus-5-5": {"inputTokens": 200,
            "outputTokens": 500, "cacheReadInputTokens": 6000, "cacheCreationInputTokens": 1800,
            "costUSD": 0.0254}});
    }
    emit(result);
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
