//! Opt-in checks against a REAL `codex` binary. Ignored by default; never run in CI.
//!
//! ```bash
//! # Preflight + handshake + thread creation: no model call, no credentials.
//! IGNIS_CODEX_LIVE_BIN=/path/to/native/codex cargo test -p ignis-research --test runtime_live -- --ignored live_handshake
//! # A real turn: calls the model (costs tokens) and needs CODEX_API_KEY in the environment.
//! IGNIS_CODEX_LIVE_BIN=... IGNIS_CODEX_LIVE_TURN=1 cargo test -p ignis-research --test runtime_live -- --ignored live_turn
//! ```

mod common;

use std::time::Duration;

use common::*;
use ignis_core::contracts::EpisodeStatus;
use ignis_research::runtime::{AgentRuntime, CodexConfig};

fn live_config(dir: &std::path::Path) -> Option<CodexConfig> {
    let bin = std::env::var_os("IGNIS_CODEX_LIVE_BIN")?;
    let mut cfg = CodexConfig::new(bin, dir.join("codex"));
    cfg.request_timeout = Duration::from_secs(120);
    Some(cfg)
}

#[test]
#[ignore = "needs IGNIS_CODEX_LIVE_BIN (a real codex binary)"]
fn live_handshake() {
    let dir = tempfile::tempdir().unwrap();
    let Some(cfg) = live_config(dir.path()) else {
        eprintln!("IGNIS_CODEX_LIVE_BIN not set; skipped");
        return;
    };
    let h = Harness::with_config(dir.path(), cfg);
    eprintln!(
        "{}",
        serde_json::to_string_pretty(h.bridge.manifest()).unwrap()
    );
    let sc = scope("ws-live", false);
    let ep = h.episode(&sc, "live-handshake");
    // No message: the bridge starts the process and the thread, and no turn (no model call).
    h.bridge.kick(&sc, &ep).unwrap();
    let t0 = std::time::Instant::now();
    while h.store.episode(&sc, &ep).unwrap().runtime.is_none() {
        assert!(
            t0.elapsed() < Duration::from_secs(60),
            "no thread from the real app-server"
        );
        std::thread::sleep(Duration::from_millis(100));
    }
    assert!(
        h.events(&sc, &ep).is_empty(),
        "nothing recorded without a message"
    );
}

#[test]
#[ignore = "calls a paid model: needs IGNIS_CODEX_LIVE_BIN, IGNIS_CODEX_LIVE_TURN=1 and CODEX_API_KEY"]
fn live_turn() {
    let dir = tempfile::tempdir().unwrap();
    let (Some(cfg), Ok(_)) = (
        live_config(dir.path()),
        std::env::var("IGNIS_CODEX_LIVE_TURN"),
    ) else {
        eprintln!("live turn not authorized; skipped");
        return;
    };
    let h = Harness::with_config(dir.path(), cfg);
    let sc = scope("ws-live", false);
    let ep = h.episode(&sc, "live-turn");
    h.say(&sc, &ep, "Reply with the single word: ready.");
    let evs = h.wait(&sc, &ep, "turn end", |e| {
        matches!(
            status_of(e),
            Some(EpisodeStatus::Completed | EpisodeStatus::Failed | EpisodeStatus::WaitingUser)
        )
    });
    eprintln!("{:#?}", kinds(&evs));
    ignis_core::contracts::check_event_log(&evs).unwrap();
}
