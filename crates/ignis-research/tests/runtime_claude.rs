//! `ClaudeBridge` over the fake headless Claude Code (`ignis-fake-claude-code`): turns, session
//! resume, failure reasons (plan limit, bad credential), cancel, isolation of the child.

mod common;

use std::path::Path;
use std::sync::{Arc, Once};

use common::*;
use ignis_core::contracts::*;
use ignis_research::runtime::{
    claude_preflight, AgentRuntime, ClaudeBridge, ClaudeConfig, StopReason,
};
use ignis_research::store::EpisodeAction;
use ignis_research::{LocalArtifactStore, ResearchStore};

static ENV: Once = Once::new();

/// The fake's credential and a host secret the child must never see.
fn env() {
    ENV.call_once(|| {
        std::env::set_var("IGNIS_FAKE_CLAUDE_TOKEN", "fake-plan-token");
        std::env::set_var("IGNIS_TEST_HOST_SECRET", "must-not-leak");
    });
}

fn config(dir: &Path) -> ClaudeConfig {
    env();
    let mut cfg = ClaudeConfig::new(
        env!("CARGO_BIN_EXE_ignis-fake-claude-code"),
        dir.join("claude"),
    );
    cfg.pass_env = vec!["IGNIS_FAKE_CLAUDE_TOKEN".into()];
    cfg
}

struct H {
    store: Arc<ResearchStore>,
    bridge: ClaudeBridge,
}

fn harness(dir: &Path) -> H {
    let store = Arc::new(ResearchStore::open(dir.join("state.sqlite")).unwrap());
    let artifacts = Arc::new(LocalArtifactStore::open(dir.join("artifacts")).unwrap());
    let cfg = config(dir);
    let manifest = claude_preflight(&cfg).expect("preflight of the fake");
    let bridge = ClaudeBridge::new(
        cfg,
        manifest,
        store.clone(),
        artifacts,
        Arc::new(|_: &EpisodeId, _| {}),
    );
    H { store, bridge }
}

impl H {
    fn episode(&self, sc: &ExecutionScope, goal: &str) -> EpisodeId {
        self.store
            .create_episode(
                sc,
                goal,
                &ClientRequestId::new(format!("c-{goal}")).unwrap(),
                Default::default(),
                Origin::SyntheticFixture,
            )
            .unwrap()
            .0
            .episode_id
    }
    fn say(&self, sc: &ExecutionScope, ep: &EpisodeId, text: &str) {
        let key = ClientRequestId::new(format!("m-{}", uuid_like())).unwrap();
        self.store.post_message(sc, ep, text, &key).unwrap();
        self.bridge.kick(sc, ep).unwrap();
    }
    fn wait(
        &self,
        sc: &ExecutionScope,
        ep: &EpisodeId,
        what: &str,
        pred: impl Fn(&[ResearchEvent]) -> bool,
    ) -> Vec<ResearchEvent> {
        let t0 = std::time::Instant::now();
        loop {
            let evs = events(&self.store, sc, ep);
            if pred(&evs) {
                return evs;
            }
            assert!(
                t0.elapsed().as_secs() < 30,
                "timed out waiting for {what}; log: {:#?}",
                kinds(&evs)
            );
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
    }
    fn idle(
        &self,
        sc: &ExecutionScope,
        ep: &EpisodeId,
        status: EpisodeStatus,
        turns: usize,
    ) -> Vec<ResearchEvent> {
        self.wait(sc, ep, &format!("{status:?} after {turns} turns"), |e| {
            status_of(e) == Some(status)
                && kinds(e).iter().filter(|k| k.ends_with("->Running")).count() >= turns
                && self.store.active_turn(sc, ep).unwrap().is_none()
        })
    }
}

fn reason(evs: &[ResearchEvent]) -> String {
    evs.iter()
        .rev()
        .find_map(|e| match &e.body {
            EventBody::StatusChanged { reason, .. } => reason.clone(),
            _ => None,
        })
        .unwrap_or_default()
}

#[test]
fn a_turn_with_a_tool_call_then_a_resumed_session() {
    let dir = tempfile::tempdir().unwrap();
    let h = harness(dir.path());
    assert!(h
        .bridge
        .manifest()
        .runtime_build_id
        .to_string()
        .starts_with("claude-code-2.1.273+"));
    let sc = scope("ws-c", false);
    let ep = h.episode(&sc, "claude");
    h.say(&sc, &ep, "use a tool please");
    let evs = h.idle(&sc, &ep, EpisodeStatus::Completed, 1);
    check_event_log(&evs).unwrap();
    let k = kinds(&evs);
    assert_eq!(k[0], "message.user");
    assert!(
        k.iter().position(|x| x == "tool.started") < k.iter().position(|x| x == "tool.finished")
    );
    assert_eq!(completed_texts(&evs), vec!["Answer: use a tool please"]);
    let rt = h
        .store
        .episode(&sc, &ep)
        .unwrap()
        .runtime
        .expect("runtime thread");
    assert_eq!(rt.runtime, RuntimeKind::ClaudeCode);

    h.say(&sc, &ep, "and again");
    let evs = h.idle(&sc, &ep, EpisodeStatus::Completed, 2);
    check_event_log(&evs).unwrap();
    assert_eq!(
        completed_texts(&evs),
        vec!["Answer: use a tool please", "Answer: and again"]
    );
    // Same session: started once, resumed once.
    let ws = std::fs::read_dir(dir.path().join("claude"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let argv = std::fs::read_to_string(ws.join("config").join("argv.log")).unwrap();
    let lines: Vec<&str> = argv.lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(
        lines[0].contains(&format!("\"--session-id\",\"{}\"", rt.thread_id)),
        "{}",
        lines[0]
    );
    assert!(
        lines[1].contains(&format!("\"--resume\",\"{}\"", rt.thread_id)),
        "{}",
        lines[1]
    );
}

#[test]
fn a_plan_limit_or_a_bad_credential_fails_the_turn_with_its_reason() {
    let dir = tempfile::tempdir().unwrap();
    let h = harness(dir.path());
    let sc = scope("ws-f", false);
    let ep = h.episode(&sc, "limit");
    h.say(&sc, &ep, "hit the limit");
    let evs = h.idle(&sc, &ep, EpisodeStatus::Failed, 1);
    assert!(
        reason(&evs).contains("usage limit reached"),
        "{}",
        reason(&evs)
    );

    let ep = h.episode(&sc, "crash");
    h.say(&sc, &ep, "crash now");
    let evs = h.idle(&sc, &ep, EpisodeStatus::Failed, 1);
    // Also proves the host secret never reached the child (the fake would fail differently).
    assert!(reason(&evs).contains("Invalid API key"), "{}", reason(&evs));
}

#[test]
fn cancel_kills_the_turn_and_ends_the_episode() {
    let dir = tempfile::tempdir().unwrap();
    let h = harness(dir.path());
    let sc = scope("ws-x", false);
    let ep = h.episode(&sc, "cancel");
    h.say(&sc, &ep, "slow work");
    h.wait(&sc, &ep, "first message", |e| {
        count(e, "message.completed") >= 1
    });
    h.store.act(&sc, &ep, EpisodeAction::Cancel).unwrap();
    h.bridge.interrupt(&sc, &ep, StopReason::Cancel).unwrap();
    let evs = h.wait(&sc, &ep, "turn closed", |_| {
        h.store.active_turn(&sc, &ep).unwrap().is_none()
    });
    check_event_log(&evs).unwrap();
    assert_eq!(
        status_of(&events(&h.store, &sc, &ep)),
        Some(EpisodeStatus::Cancelled)
    );
}

#[test]
fn preflight_refuses_without_a_credential() {
    let dir = tempfile::tempdir().unwrap();
    let mut cfg = config(dir.path());
    cfg.pass_env = vec!["IGNIS_NO_SUCH_CREDENTIAL".into()];
    let e = claude_preflight(&cfg).unwrap_err();
    assert!(e.contains("no Claude credential"), "{e}");
}

/// Opt-in, against the real binary (`IGNIS_LIVE_CLAUDE_BIN=<path to claude>`), with an empty,
/// signed-out config directory: no credential is touched and no model is called. The real CLI must
/// accept every flag the bridge passes, and its sign-in failure must become the turn's reason.
#[test]
#[ignore]
fn live_binary_accepts_the_flags_and_reports_a_missing_sign_in() {
    let Some(bin) = std::env::var_os("IGNIS_LIVE_CLAUDE_BIN") else {
        return;
    };
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(ResearchStore::open(dir.path().join("state.sqlite")).unwrap());
    let artifacts = Arc::new(LocalArtifactStore::open(dir.path().join("artifacts")).unwrap());
    let login = dir.path().join("signed-out");
    std::fs::create_dir_all(&login).unwrap();
    let mut cfg = ClaudeConfig::new(bin, dir.path().join("claude"));
    cfg.pass_env = vec![];
    cfg.login_home = Some(login);
    let manifest = claude_preflight(&cfg).expect("preflight of the real binary");
    println!("build: {}", manifest.runtime_build_id);
    let h = H {
        store: store.clone(),
        bridge: ClaudeBridge::new(cfg, manifest, store, artifacts, Arc::new(|_: &EpisodeId, _| {})),
    };
    let sc = scope("ws-live", false);
    let ep = h.episode(&sc, "live");
    h.say(&sc, &ep, "Say hello.");
    let evs = h.idle(&sc, &ep, EpisodeStatus::Failed, 1);
    let why = reason(&evs);
    println!("reason: {why}");
    assert!(!why.contains("unknown option") && !why.contains("error: "), "{why}");
}
