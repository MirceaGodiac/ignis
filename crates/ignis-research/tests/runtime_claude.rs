//! `ClaudeBridge` over the fake headless Claude Code (`ignis-fake-claude-code`): turns, session
//! resume, failure reasons (plan limit, bad credential), cancel, isolation of the child.

mod common;

use std::path::Path;
use std::sync::{Arc, Once};

use common::*;
use ignis_core::contracts::*;
use ignis_research::runtime::{
    claude_preflight, AgentRuntime, ClaudeBridge, ClaudeConfig, McpAttachment, McpGrants,
    StopReason,
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
    artifacts: Arc<LocalArtifactStore>,
    bridge: ClaudeBridge,
}

fn harness(dir: &Path) -> H {
    harness_with(dir, |_| {})
}

fn harness_with(dir: &Path, tune: impl FnOnce(&mut ClaudeConfig)) -> H {
    let store = Arc::new(ResearchStore::open(dir.join("state.sqlite")).unwrap());
    let artifacts = Arc::new(LocalArtifactStore::open(dir.join("artifacts")).unwrap());
    let mut cfg = config(dir);
    tune(&mut cfg);
    let manifest = claude_preflight(&cfg).expect("preflight of the fake");
    let bridge = ClaudeBridge::new(
        cfg,
        manifest,
        store.clone(),
        artifacts.clone(),
        Arc::new(|_: &EpisodeId, _| {}),
    );
    H {
        store,
        artifacts,
        bridge,
    }
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
fn a_tool_call_records_its_arguments_and_the_message_that_introduced_it() {
    let dir = tempfile::tempdir().unwrap();
    let h = harness(dir.path());
    let sc = scope("ws-c", false);
    let ep = h.episode(&sc, "explain");
    h.say(&sc, &ep, "explain then compute");
    let evs = h.idle(&sc, &ep, EpisodeStatus::Completed, 1);
    check_event_log(&evs).unwrap();
    assert_explained_call(&evs, h.artifacts.as_ref(), &sc, &[]);
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

/// The prompts the fake received, in order.
fn prompts(dir: &Path) -> Vec<String> {
    let ws = std::fs::read_dir(dir.join("claude"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    std::fs::read_to_string(ws.join("config").join("prompts.jsonl"))
        .unwrap_or_default()
        .lines()
        .map(|l| {
            let v: serde_json::Value = serde_json::from_str(l).unwrap();
            v["prompt"].as_str().unwrap().to_string()
        })
        .collect()
}

fn transitions(evs: &[ResearchEvent]) -> Vec<(EpisodeStatus, EpisodeStatus)> {
    evs.iter()
        .filter_map(|e| match &e.body {
            EventBody::StatusChanged { from, to, .. } => Some((*from, *to)),
            _ => None,
        })
        .collect()
}

/// A plan limit, a bad credential or a process that dies without a result does not end the
/// conversation: the episode waits for the user with the reason, and the next message resumes the
/// session (or starts it, when the dead turn never created it).
#[test]
fn a_plan_limit_or_a_bad_credential_waits_for_the_user_and_the_session_goes_on() {
    let dir = tempfile::tempdir().unwrap();
    let h = harness(dir.path());
    let sc = scope("ws-f", false);
    let ep = h.episode(&sc, "limit");
    h.say(&sc, &ep, "hit the limit");
    let evs = h.idle(&sc, &ep, EpisodeStatus::WaitingUser, 1);
    assert!(
        reason(&evs).contains("usage limit reached"),
        "{}",
        reason(&evs)
    );
    h.say(&sc, &ep, "and now");
    let evs = h.idle(&sc, &ep, EpisodeStatus::Completed, 2);
    check_event_log(&evs).unwrap();
    assert_eq!(completed_texts(&evs), vec!["Answer: and now"]);

    let ep = h.episode(&sc, "crash");
    h.say(&sc, &ep, "crash now");
    let evs = h.idle(&sc, &ep, EpisodeStatus::WaitingUser, 1);
    // Also proves the host secret never reached the child (the fake would fail differently).
    assert!(reason(&evs).contains("Invalid API key"), "{}", reason(&evs));

    // Dead before any output: the session never existed, so the next turn starts it.
    let ep = h.episode(&sc, "mute");
    h.say(&sc, &ep, "mute now");
    let evs = h.idle(&sc, &ep, EpisodeStatus::WaitingUser, 1);
    assert!(reason(&evs).contains("overloaded"), "{}", reason(&evs));
    h.say(&sc, &ep, "try again");
    let evs = h.idle(&sc, &ep, EpisodeStatus::Completed, 2);
    assert_eq!(completed_texts(&evs), vec!["Answer: try again"]);
    let sid = h
        .store
        .episode(&sc, &ep)
        .unwrap()
        .runtime
        .unwrap()
        .thread_id;
    let runs: Vec<_> = argv(dir.path())
        .into_iter()
        .filter(|a| a.contains(&sid))
        .collect();
    assert_eq!(runs.len(), 2);
    assert!(runs.iter().all(|a| a.iter().any(|x| x == "--session-id")));
}

/// A turn past its budget is stopped and said so; the conversation goes on.
#[test]
fn a_turn_past_its_budget_waits_for_the_user() {
    let dir = tempfile::tempdir().unwrap();
    let h = harness_with(dir.path(), |c| {
        c.turn_timeout = std::time::Duration::from_millis(800)
    });
    let sc = scope("ws-b", false);
    let ep = h.episode(&sc, "budget");
    h.say(&sc, &ep, "slow work");
    let evs = h.idle(&sc, &ep, EpisodeStatus::WaitingUser, 1);
    assert!(reason(&evs).contains("budget of 0.8 s"), "{}", reason(&evs));
    h.say(&sc, &ep, "quick one");
    let evs = h.idle(&sc, &ep, EpisodeStatus::Completed, 2);
    check_event_log(&evs).unwrap();
    assert!(completed_texts(&evs).contains(&"Answer: quick one".to_string()));
}

/// Resume after a pause continues the interrupted question with one host turn quoting it.
#[test]
fn resume_continues_the_interrupted_question() {
    let dir = tempfile::tempdir().unwrap();
    let h = harness(dir.path());
    let sc = scope("ws-r", false);
    let ep = h.episode(&sc, "resume");
    h.say(&sc, &ep, "slow work");
    h.wait(&sc, &ep, "first message", |e| {
        count(e, "message.completed") >= 1
    });
    h.store.act(&sc, &ep, EpisodeAction::Pause).unwrap();
    h.bridge.interrupt(&sc, &ep, StopReason::Pause).unwrap();
    h.wait(&sc, &ep, "turn closed", |_| {
        h.store.active_turn(&sc, &ep).unwrap().is_none()
    });
    h.store.act(&sc, &ep, EpisodeAction::Resume).unwrap();
    h.bridge.resume(&sc, &ep).unwrap();
    let evs = h.idle(&sc, &ep, EpisodeStatus::Completed, 2);
    check_event_log(&evs).unwrap();
    let sent = prompts(dir.path());
    assert_eq!(sent.len(), 2, "{sent:?}");
    assert!(sent[1].starts_with("IGNIS host:"), "{}", sent[1]);
    assert!(sent[1].contains("> slow work"), "{}", sent[1]);
    let args = &argv(dir.path())[1];
    assert!(args.iter().any(|a| a == "--resume"), "{args:?}");
}

/// With `answer_submit` attached, a turn that ends without an accepted answer gets exactly one
/// host continuation (the episode never looks completed in between); then it completes.
#[test]
fn a_turn_without_an_accepted_answer_is_continued_once() {
    let dir = tempfile::tempdir().unwrap();
    let h = harness_with(dir.path(), |c| {
        c.mcp = Some(McpAttachment {
            url: "http://127.0.0.1:9/mcp".into(),
            grants: McpGrants::new(),
            enabled_tools: vec!["sources_search".into(), "answer_submit".into()],
            approved_tools: vec!["sources_search".into(), "answer_submit".into()],
        });
    });
    let sc = scope("ws-a", false);
    let ep = h.episode(&sc, "answer");
    h.say(&sc, &ep, "a question");
    let evs = h.idle(&sc, &ep, EpisodeStatus::Completed, 2);
    check_event_log(&evs).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(300));
    assert_eq!(events(&h.store, &sc, &ep).len(), evs.len());
    use EpisodeStatus as S;
    assert_eq!(
        transitions(&evs),
        vec![
            (S::Ready, S::Running),
            (S::Running, S::Running),
            (S::Running, S::Completed)
        ]
    );
    let sent = prompts(dir.path());
    assert_eq!(sent.len(), 2, "{sent:?}");
    assert!(
        sent[1].contains("Submit your answer with answer_submit"),
        "{}",
        sent[1]
    );
}

/// A message no turn received is sent by recovery.
#[test]
fn a_message_no_turn_received_is_sent_on_recovery() {
    let dir = tempfile::tempdir().unwrap();
    let h = harness(dir.path());
    let sc = scope("ws-s", false);
    let ep = h.episode(&sc, "stranded");
    h.store
        .post_message(
            &sc,
            &ep,
            "nobody sent me",
            &ClientRequestId::new("s1").unwrap(),
        )
        .unwrap();
    h.bridge.recover().unwrap();
    let evs = h.idle(&sc, &ep, EpisodeStatus::Completed, 1);
    assert_eq!(completed_texts(&evs), vec!["Answer: nobody sent me"]);
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
    cfg.harness = Some(ignis_research::runtime::HarnessBundle::embedded(&[]));
    // With IGNIS_LIVE_CLAUDE_DELEGATION set, also the subagent flags (`--tools Agent --agents …`).
    if std::env::var_os("IGNIS_LIVE_CLAUDE_DELEGATION").is_some() {
        delegating(&mut cfg);
    }
    let manifest = claude_preflight(&cfg).expect("preflight of the real binary");
    println!("build: {}", manifest.runtime_build_id);
    let h = H {
        store: store.clone(),
        artifacts: artifacts.clone(),
        bridge: ClaudeBridge::new(
            cfg,
            manifest,
            store,
            artifacts,
            Arc::new(|_: &EpisodeId, _| {}),
        ),
    };
    let sc = scope("ws-live", false);
    let ep = h.episode(&sc, "live");
    h.say(&sc, &ep, "Say hello.");
    let evs = h.idle(&sc, &ep, EpisodeStatus::WaitingUser, 1);
    let why = reason(&evs);
    println!("reason: {why}");
    assert!(
        !why.contains("unknown option") && !why.contains("error: "),
        "{why}"
    );
}

#[test]
fn the_harness_bundle_is_appended_identically_every_turn_and_pinned() {
    use ignis_research::runtime::{HarnessBundle, HarnessSource};
    let dir = tempfile::tempdir().unwrap();
    // As served: skills on demand, so the appended prefix carries the skill index, not the texts.
    let tools: Vec<String> = vec![
        "sources_search".into(),
        "answer_submit".into(),
        "skills_read".into(),
    ];
    let h1 = HarnessBundle::embedded(&tools);
    assert!(h1.text().contains("`skills_read`") && !h1.text().contains("<skill "));
    let with = |b: &HarnessBundle| {
        let store = Arc::new(ResearchStore::open(dir.path().join("state.sqlite")).unwrap());
        let artifacts = Arc::new(LocalArtifactStore::open(dir.path().join("artifacts")).unwrap());
        let mut cfg = config(dir.path());
        cfg.harness = Some(b.clone());
        let manifest = claude_preflight(&cfg).unwrap();
        H {
            store: store.clone(),
            artifacts: artifacts.clone(),
            bridge: ClaudeBridge::new(
                cfg,
                manifest,
                store,
                artifacts,
                Arc::new(|_: &EpisodeId, _| {}),
            ),
        }
    };
    let running = |evs: &[ResearchEvent]| -> Vec<String> {
        evs.iter()
            .filter_map(|e| match &e.body {
                EventBody::StatusChanged { to, reason, .. } if *to == EpisodeStatus::Running => {
                    Some(reason.clone().unwrap_or_default())
                }
                _ => None,
            })
            .collect()
    };
    let sc = scope("ws-hc", false);
    let h = with(&h1);
    let ep = h.episode(&sc, "harness");
    h.say(&sc, &ep, "first question");
    h.idle(&sc, &ep, EpisodeStatus::Completed, 1);
    h.say(&sc, &ep, "second question");
    let evs = h.idle(&sc, &ep, EpisodeStatus::Completed, 2);
    check_event_log(&evs).unwrap();
    let reasons = running(&evs);
    assert!(
        reasons[0].contains(&format!("harness {} injected", h1.short())),
        "{reasons:?}"
    );
    assert!(!reasons[1].contains("harness"), "{reasons:?}");
    let rec = h.store.episode(&sc, &ep).unwrap();
    rec.validate().unwrap();
    let pin = rec.pins.harness.expect("harness pin");
    assert_eq!(&pin.digest, h1.digest());
    assert_eq!(
        pin.model, "fake-claude-model",
        "the model Claude Code reported"
    );
    assert_eq!(pin.runtime_build, h.bridge.manifest().runtime_build_id);
    // Both turns got exactly the bundle's bytes.
    let cfg_dir = std::fs::read_dir(dir.path().join("claude"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path()
        .join("config");
    let logged: Vec<serde_json::Value> =
        std::fs::read_to_string(cfg_dir.join("system-prompts.jsonl"))
            .unwrap()
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect();
    assert_eq!(logged.len(), 2);
    assert!(logged.iter().all(|v| v["append"] == h1.text()));
    let argv = std::fs::read_to_string(cfg_dir.join("argv.log")).unwrap();
    assert!(argv.contains("--exclude-dynamic-system-prompt-sections"));
    drop(h);

    // Another bundle on the host: the resumed session keeps the prompt it started with, so the
    // pin stays and the log says why; a new episode starts with the new bundle.
    let mut src = HarnessSource::embedded();
    src.investigator.push_str(
        "
An extra rule for this variant.
",
    );
    let h2 = HarnessBundle::render("h2", &src, &tools).unwrap();
    let h = with(&h2);
    h.say(&sc, &ep, "third question");
    let evs = h.idle(&sc, &ep, EpisodeStatus::Completed, 3);
    let reasons = running(&evs);
    assert!(
        reasons[2].contains(&format!("stays on {}", h1.short())),
        "{reasons:?}"
    );
    let pin = h.store.episode(&sc, &ep).unwrap().pins.harness.unwrap();
    assert_eq!(&pin.digest, h1.digest());
    let ep2 = h.episode(&sc, "fresh");
    h.say(&sc, &ep2, "a question");
    h.idle(&sc, &ep2, EpisodeStatus::Completed, 1);
    let pin = h.store.episode(&sc, &ep2).unwrap().pins.harness.unwrap();
    assert_eq!(&pin.digest, h2.digest());
}

/// The IGNIS MCP server as the bridge sees it (nothing listens: the fake never connects).
fn delegating(cfg: &mut ClaudeConfig) {
    cfg.delegation = true;
    cfg.mcp = Some(McpAttachment {
        url: "http://127.0.0.1:9/mcp".into(),
        grants: McpGrants::new(),
        enabled_tools: vec!["sources_search".into(), "answer_submit".into()],
        approved_tools: vec!["sources_search".into(), "answer_submit".into()],
    });
}

fn argv(dir: &Path) -> Vec<Vec<String>> {
    let ws = std::fs::read_dir(dir.join("claude"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    std::fs::read_to_string(ws.join("config").join("argv.log"))
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect()
}

#[test]
fn a_delegated_subtask_runs_in_the_parent_episode_and_cannot_delegate_again() {
    let dir = tempfile::tempdir().unwrap();
    let off = harness(&dir.path().join("off"));
    let h = harness_with(dir.path(), delegating);
    // Delegation is part of the runtime build: the two arms of an ablation never share an id.
    assert_ne!(
        h.bridge.manifest().runtime_build_id,
        off.bridge.manifest().runtime_build_id
    );

    let sc = scope("ws-d", false);
    let ep = h.episode(&sc, "delegate");
    h.say(&sc, &ep, "delegate the aggregation");
    let evs = h.idle(&sc, &ep, EpisodeStatus::Completed, 1);
    check_event_log(&evs).unwrap();

    // The child got exactly one subagent: IGNIS tools only, never answer_submit or a subagent tool.
    let args = &argv(dir.path())[0];
    let flag = |f: &str| {
        let i = args.iter().position(|a| a == f).unwrap();
        args[i + 1].clone()
    };
    assert_eq!(flag("--tools"), "Agent");
    let agents: serde_json::Value = serde_json::from_str(&flag("--agents")).unwrap();
    let defs = agents.as_object().unwrap();
    assert_eq!(defs.len(), 1);
    let tools = &defs["ignis-subinvestigator"]["tools"];
    assert_eq!(tools, &serde_json::json!(["mcp__ignis__sources_search"]));
    assert!(flag("--allowedTools").split(',').any(|t| t == "Agent"));

    // The subagent's IGNIS call and the subagent call itself are in the parent episode's log;
    // the subagent's text is archived, never an assistant message of the episode.
    let started: Vec<String> = evs
        .iter()
        .filter_map(|e| match &e.body {
            EventBody::ToolStarted { tool, .. } => Some(tool.clone()),
            _ => None,
        })
        .collect();
    assert!(started.contains(&"claude.Agent".to_string()), "{started:?}");
    assert!(
        started.contains(&"sources_search".to_string()),
        "{started:?}"
    );
    let finished = kinds(&evs).iter().filter(|k| *k == "tool.finished").count();
    assert_eq!(finished, 2, "{:#?}", kinds(&evs));
    let texts = completed_texts(&evs);
    assert!(!texts.iter().any(|t| t.starts_with("Sub:")), "{texts:?}");
    assert!(texts.contains(&"Delegating the aggregation.".to_string()));

    // A subagent that tries to delegate again: the host stops the turn and says why.
    let ep2 = h.episode(&sc, "nested");
    h.say(&sc, &ep2, "nested delegation");
    let evs = h.idle(&sc, &ep2, EpisodeStatus::Failed, 1);
    check_event_log(&evs).unwrap();
    assert!(reason(&evs).contains("depth 1"), "{}", reason(&evs));
}

#[test]
fn a_turn_s_usage_and_reported_cost_are_recorded_and_measured() {
    let dir = tempfile::tempdir().unwrap();
    let h = harness(dir.path());
    let sc = scope("ws-u", false);
    let ep = h.episode(&sc, "usage");
    h.say(&sc, &ep, "search with a tool and report usage");
    let evs = h.idle(&sc, &ep, EpisodeStatus::Completed, 1);
    check_event_log(&evs).unwrap();
    let usage: Vec<_> = evs
        .iter()
        .filter_map(|e| match &e.body {
            EventBody::UsageReported { usage } => Some((usage.clone(), e.causation.clone())),
            _ => None,
        })
        .collect();
    let [(u, c)] = &usage[..] else {
        panic!("{usage:?}")
    };
    // Anthropic counts cache reads and writes apart from input; the report folds them in.
    assert_eq!(u.accounting, UsageAccounting::Increment);
    assert_eq!(u.model.as_deref(), Some("claude-opus-5-5"));
    assert_eq!(u.input_tokens, 200 + 6000 + 1800);
    assert_eq!(
        (u.cached_input_tokens, u.cache_write_tokens),
        (Some(6000), Some(1800))
    );
    assert_eq!(u.output_tokens, 500);
    let reported = u.reported_cost.as_ref().unwrap();
    assert_eq!(reported.basis, CostBasis::ProviderReported);
    assert!(c.runtime_turn_id.is_some());

    // The metrics price it from the price list (not from the reported cost).
    let m = ignis_research::metrics::measure_episode(
        &h.store,
        &sc,
        &ep,
        &ignis_research::metrics::PriceList::builtin(),
    )
    .unwrap()
    .metrics;
    m.validate().unwrap();
    assert_eq!(m.turns.len(), 1);
    assert_eq!(m.turns[0].tool_calls, 1);
    assert_eq!(m.tools[0].tool, "sources_search");
    assert_eq!(m.tools[0].latency.as_ref().unwrap().samples, 1);
    let pct = m.tokens.cache_hit_pct.unwrap().get();
    assert!((pct - 75.0).abs() < 1e-9, "{pct}");
    let CostStatus::Priced { cost } = &m.questions[0].cost else {
        panic!("{:?}", m.questions[0].cost)
    };
    // 200 × 4 + 6000 × 0.20 + 1800 × 5 + 500 × 20, per million tokens.
    let want = (200.0 * 4.0 + 6000.0 * 0.20 + 1800.0 * 5.0 + 500.0 * 20.0) / 1e6;
    assert!(
        (cost.amount.get() - want).abs() < 1e-12,
        "{}",
        cost.amount.get()
    );
    assert_eq!(
        cost.price_list.as_deref(),
        Some("ignis-llm-prices@2026-11-15b")
    );
}
