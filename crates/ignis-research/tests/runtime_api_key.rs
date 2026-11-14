//! API-key mode: Codex App Server ignores a key in its environment, so the bridge logs every
//! app-server process in with `account/login/start` right after the handshake (its own binary: the
//! key is a process-wide environment variable).

mod common;

use common::*;
use ignis_core::contracts::*;

#[test]
fn every_process_is_logged_in_with_the_api_key_before_any_thread() {
    std::env::set_var("CODEX_API_KEY", "sk-test-fake-key");
    let dir = tempfile::tempdir().unwrap();
    let h = Harness::new(dir.path());
    for ws in ["ws-k1", "ws-k2"] {
        let sc = scope(ws, false);
        let ep = h.episode(&sc, ws);
        h.say(&sc, &ep, "hello");
        h.wait_status(&sc, &ep, EpisodeStatus::Completed);
        let reqs = h.requests(ws);
        assert_eq!(
            reqs[..4],
            [
                "initialize",
                "account/login/start",
                "thread/start",
                "turn/start"
            ],
            "{reqs:?}"
        );
    }
    // The key never reaches the request log.
    for e in walk(dir.path()) {
        let s = std::fs::read_to_string(&e).unwrap_or_default();
        assert!(
            !s.contains("sk-test-fake-key"),
            "key found in {}",
            e.display()
        );
    }
}

fn walk(p: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut out = vec![];
    for e in std::fs::read_dir(p).into_iter().flatten().flatten() {
        let path = e.path();
        if path.is_dir() {
            out.extend(walk(&path));
        } else if path.extension().is_some_and(|x| x == "log") {
            out.push(path);
        }
    }
    out
}
