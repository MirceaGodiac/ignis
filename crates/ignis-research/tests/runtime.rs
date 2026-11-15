//! Conformance of the Codex bridge against the fake app-server (no model, no network): handshake,
//! a turn with deltas / completed message / tool call, approvals granted, denied and refused by
//! policy, unknown notifications archived as `runtime.opaque`, pause / resume / cancel, resume after
//! a host restart without duplicated events, and episodes that never mix.

mod common;

use std::io::Read;

use common::*;
use ignis_core::contracts::*;
use ignis_core::services::ArtifactStore;
use ignis_research::runtime::{preflight, AgentRuntime, StopReason};
use ignis_research::store::EpisodeAction;

#[test]
fn handshake_then_a_turn_with_deltas_a_message_and_a_tool_call() {
    let dir = tempfile::tempdir().unwrap();
    let h = Harness::new(dir.path());
    let id = h.bridge.manifest().runtime_build_id.to_string();
    assert!(id.starts_with("codex-app-server-0.169.0+"), "{id}");
    assert_eq!(h.bridge.manifest().transport, "stdio");

    let sc = scope("ws-a", false);
    let ep = h.episode(&sc, "turn");
    h.say(&sc, &ep, "search the corpus with a tool");
    let evs = h.wait_status(&sc, &ep, EpisodeStatus::Completed);
    check_event_log(&evs).unwrap();
    let k = kinds(&evs);
    assert_eq!(k[0], "message.user");
    assert_eq!(k[1], "status Ready->Running");
    assert_eq!(k.last().unwrap(), "status Running->Completed");
    let tool_finished = evs
        .iter()
        .find_map(|e| match &e.body {
            EventBody::ToolFinished { result } => Some((result.clone(), e.causation.clone())),
            _ => None,
        })
        .expect("tool.finished");
    assert_eq!(tool_finished.0.tool, "sources_search");
    assert_eq!(tool_finished.0.execution_status, ExecutionStatus::Completed);
    assert!(tool_finished.1.runtime_turn_id.is_some() && tool_finished.1.tool_call_id.is_some());
    let pos = |kind: &str| k.iter().position(|x| x == kind).unwrap();
    assert!(pos("tool.started") < pos("tool.finished"));
    assert!(pos("tool.finished") < pos("message.delta"));
    assert!(pos("message.delta") < pos("message.completed"));
    assert_eq!(completed_texts(&evs), vec!["Fixture answer"]);

    // The thread mapping is recorded on the episode; the handshake came first.
    let rec = h.store.episode(&sc, &ep).unwrap();
    let rt = rec.runtime.expect("runtime thread");
    assert_eq!(rt.runtime_build_id.to_string(), id);
    let reqs = h.requests("ws-a");
    assert_eq!(reqs[..3], ["initialize", "thread/start", "turn/start"]);
    // Commits were announced up to the last event.
    let last = rec.last_seq;
    h.wait(&sc, &ep, "commit hook", |_| {
        h.commits
            .lock()
            .unwrap()
            .iter()
            .any(|(e, s)| *e == ep && *s == last)
    });
}

#[test]
fn a_tool_call_records_its_arguments_and_the_message_that_introduced_it() {
    let dir = tempfile::tempdir().unwrap();
    let h = Harness::new(dir.path());
    let sc = scope("ws-a", false);
    let ep = h.episode(&sc, "explain");
    h.say(&sc, &ep, "explain then compute");
    let evs = h.wait_status(&sc, &ep, EpisodeStatus::Completed);
    check_event_log(&evs).unwrap();
    assert_explained_call(&evs, h.artifacts.as_ref(), &sc, &["/api_key"]);
}

#[test]
fn a_host_refusal_is_recorded_as_refused_with_its_reason() {
    let dir = tempfile::tempdir().unwrap();
    let h = Harness::new(dir.path());
    let sc = scope("ws-refuse", false);
    let ep = h.episode(&sc, "refuse");
    h.say(&sc, &ep, "submit an answer that gets a refusal");
    let evs = h.wait_status(&sc, &ep, EpisodeStatus::Completed);
    check_event_log(&evs).unwrap();
    let finished: Vec<_> = evs
        .iter()
        .filter_map(|e| match &e.body {
            EventBody::ToolFinished { result } => Some((result.clone(), e.causation.clone())),
            _ => None,
        })
        .collect();
    // Structured content and text-only content both keep the refusal, never "internal".
    assert_eq!(finished.len(), 2);
    for (r, c) in finished {
        assert_eq!(r.tool, "answer_submit");
        assert_eq!(r.execution_status, ExecutionStatus::Refused);
        let e = r.error.expect("refusal reason");
        assert_eq!(e.code, ErrorCode::InvalidRequest);
        assert!(e.message.contains("unknown_evidence"), "{}", e.message);
        assert_eq!(
            Some(&r.call_id),
            c.tool_call_id.as_ref(),
            "re-keyed to the runtime item"
        );
    }
}

#[test]
fn approvals_are_resolved_by_the_host_granted_or_denied() {
    let dir = tempfile::tempdir().unwrap();
    let h = Harness::new(dir.path());
    let writer = scope("ws-ap", true);
    let reader = scope("ws-ap", false);
    for (goal, decision, expect) in [
        ("grant", ApprovalDecision::Approved, "accept"),
        ("deny", ApprovalDecision::Denied, "decline"),
    ] {
        let ep = h.episode(&writer, goal);
        h.say(&writer, &ep, "needs an approval");
        let evs = h.wait_status(&writer, &ep, EpisodeStatus::WaitingApproval);
        let approval = evs
            .iter()
            .find_map(|e| match &e.body {
                EventBody::ApprovalRequested { approval_id, .. } => Some(approval_id.clone()),
                _ => None,
            })
            .unwrap();
        if decision == ApprovalDecision::Approved {
            // Approving a workspace write needs that capability in the caller's scope.
            let e = h
                .bridge
                .resolve_approval(&reader, &ep, &approval, decision)
                .unwrap_err();
            assert_eq!(e.code, ErrorCode::ScopeDenied);
        }
        // A caller cannot pick "expired" or "cancelled"; another workspace cannot see it.
        assert!(h
            .bridge
            .resolve_approval(&writer, &ep, &approval, ApprovalDecision::Expired)
            .is_err());
        assert!(h
            .bridge
            .resolve_approval(&scope("ws-other", true), &ep, &approval, decision)
            .is_err());
        h.bridge
            .resolve_approval(&writer, &ep, &approval, decision)
            .unwrap();
        let again = h
            .bridge
            .resolve_approval(&writer, &ep, &approval, decision)
            .unwrap_err();
        assert_eq!(again.code, ErrorCode::NotFound, "resolved once");
        let evs = h.wait_status(&writer, &ep, EpisodeStatus::Completed);
        check_event_log(&evs).unwrap();
        let resolved: Vec<_> = evs
            .iter()
            .filter_map(|e| match &e.body {
                EventBody::ApprovalResolved {
                    approval_id,
                    decision,
                } => Some((approval_id.clone(), *decision)),
                _ => None,
            })
            .collect();
        assert_eq!(resolved, vec![(approval.clone(), decision)]);
        assert_eq!(
            completed_texts(&evs),
            vec![format!("item/fileChange/requestApproval -> {expect}")]
        );
        let k = kinds(&evs);
        assert!(k.contains(&"status Running->WaitingApproval".to_string()));
        assert!(k.contains(&"status WaitingApproval->Running".to_string()));
    }
}

#[test]
fn shell_and_permission_requests_are_denied_by_the_public_profile() {
    let dir = tempfile::tempdir().unwrap();
    let h = Harness::new(dir.path());
    let sc = scope("ws-sh", true);
    let ep = h.episode(&sc, "shell");
    h.say(&sc, &ep, "run a shell command");
    let evs = h.wait_status(&sc, &ep, EpisodeStatus::Completed);
    let requested = evs
        .iter()
        .find_map(|e| match &e.body {
            EventBody::ApprovalRequested { effect, action, .. } => Some((*effect, action.clone())),
            _ => None,
        })
        .unwrap();
    assert_eq!(requested.0, ToolEffect::Compute);
    assert!(requested.1.contains("ls /"));
    assert!(evs.iter().any(|e| matches!(
        e.body,
        EventBody::ApprovalResolved {
            decision: ApprovalDecision::Denied,
            ..
        }
    )));
    assert!(
        !kinds(&evs).iter().any(|k| k.contains("WaitingApproval")),
        "no host wait"
    );
    assert_eq!(
        completed_texts(&evs),
        vec!["item/commandExecution/requestApproval -> decline"]
    );

    let ep = h.episode(&sc, "perm");
    h.say(&sc, &ep, "widen permissions");
    let evs = h.wait_status(&sc, &ep, EpisodeStatus::Completed);
    assert_eq!(
        completed_texts(&evs),
        vec!["item/permissions/requestApproval -> decline"]
    );
    // The profile is enforced at spawn time, not by the prompt.
    let overrides = &h.bridge.manifest().config_overrides;
    assert!(overrides.contains(&"features.shell_tool=false".to_string()));
    assert!(overrides.contains(&"sandbox_mode=\"read-only\"".to_string()));
}

#[test]
fn unknown_notifications_are_archived_as_runtime_opaque() {
    let dir = tempfile::tempdir().unwrap();
    let h = Harness::new(dir.path());
    let sc = scope("ws-op", false);
    let ep = h.episode(&sc, "opaque");
    h.say(&sc, &ep, "send something opaque");
    let evs = h.wait_status(&sc, &ep, EpisodeStatus::Completed);
    let opaque: Vec<_> = evs
        .iter()
        .filter_map(|e| match &e.body {
            EventBody::RuntimeOpaque { method, payload } => Some((method.clone(), payload.clone())),
            _ => None,
        })
        .collect();
    let methods: Vec<_> = opaque.iter().map(|(m, _)| m.as_str()).collect();
    assert!(methods.contains(&"x/unknownNotification"), "{methods:?}");
    assert!(methods.contains(&"thread/tokenUsage/updated"));
    // Opted out at the source / not attributable to a thread: never recorded.
    assert!(!methods.contains(&"account/rateLimits/updated"));
    let (_, payload) = opaque
        .iter()
        .find(|(m, _)| m == "x/unknownNotification")
        .unwrap();
    let a = payload.as_ref().expect("payload archived");
    let mut text = String::new();
    h.artifacts
        .open(&sc, &a.artifact_id, 0, None)
        .unwrap()
        .read_to_string(&mut text)
        .unwrap();
    let doc: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(doc["params"]["anything"], serde_json::json!([1, 2, 3]));
    assert_eq!(a.origin, Origin::SyntheticFixture);
}

#[test]
fn a_failed_turn_fails_the_episode_with_its_reason() {
    let dir = tempfile::tempdir().unwrap();
    let h = Harness::new(dir.path());
    let sc = scope("ws-f", false);
    let ep = h.episode(&sc, "fail");
    h.say(&sc, &ep, "this will fail");
    let evs = h.wait_status(&sc, &ep, EpisodeStatus::Failed);
    let reason = evs
        .iter()
        .rev()
        .find_map(|e| match &e.body {
            EventBody::StatusChanged { reason, .. } => reason.clone(),
            _ => None,
        })
        .unwrap();
    assert!(reason.contains("model refused"), "{reason}");
    assert!(completed_texts(&evs).is_empty());
}

#[test]
fn pause_stops_at_a_checkpoint_resume_runs_only_new_messages_cancel_ends() {
    let dir = tempfile::tempdir().unwrap();
    let h = Harness::new(dir.path());
    let sc = scope("ws-p", false);
    let ep = h.episode(&sc, "pause");
    h.say(&sc, &ep, "hang until told");
    h.wait(&sc, &ep, "first delta", |e| count(e, "message.delta") >= 1);
    // Host action first, then the runtime is interrupted.
    h.store.act(&sc, &ep, EpisodeAction::Pause).unwrap();
    h.bridge.interrupt(&sc, &ep, StopReason::Pause).unwrap();
    h.wait(&sc, &ep, "turn closed", |_| {
        h.store.active_turn(&sc, &ep).unwrap().is_none()
    });
    // A message while paused is recorded, not run.
    h.say(&sc, &ep, "follow-up");
    std::thread::sleep(std::time::Duration::from_millis(300));
    assert_eq!(status_of(&h.events(&sc, &ep)), Some(EpisodeStatus::Paused));
    h.store.act(&sc, &ep, EpisodeAction::Resume).unwrap();
    h.bridge.resume(&sc, &ep).unwrap();
    let evs = h.wait_status(&sc, &ep, EpisodeStatus::Completed);
    check_event_log(&evs).unwrap();
    assert_eq!(completed_texts(&evs), vec!["Fixture answer"]);
    // The interrupted turn's message was not re-sent: exactly two turns.
    let turns = h
        .requests("ws-p")
        .iter()
        .filter(|m| *m == "turn/start")
        .count();
    assert_eq!(turns, 2);

    let ep2 = h.episode(&sc, "cancel");
    h.say(&sc, &ep2, "hang again");
    h.wait(&sc, &ep2, "first delta", |e| count(e, "message.delta") >= 1);
    h.store.act(&sc, &ep2, EpisodeAction::Cancel).unwrap();
    h.bridge.interrupt(&sc, &ep2, StopReason::Cancel).unwrap();
    h.wait(&sc, &ep2, "turn closed", |_| {
        h.store.active_turn(&sc, &ep2).unwrap().is_none()
    });
    let evs = h.events(&sc, &ep2);
    check_event_log(&evs).unwrap();
    assert_eq!(status_of(&evs), Some(EpisodeStatus::Cancelled));
    assert!(h
        .store
        .post_message(&sc, &ep2, "more", &ClientRequestId::new("late").unwrap())
        .is_err());
}

#[test]
fn messages_during_a_turn_wait_for_the_next_turn() {
    let dir = tempfile::tempdir().unwrap();
    let h = Harness::new(dir.path());
    let sc = scope("ws-q", false);
    let ep = h.episode(&sc, "queue");
    h.say(&sc, &ep, "hang first");
    h.wait(&sc, &ep, "first delta", |e| count(e, "message.delta") >= 1);
    h.say(&sc, &ep, "second");
    h.say(&sc, &ep, "third");
    std::thread::sleep(std::time::Duration::from_millis(200));
    assert_eq!(
        h.requests("ws-q")
            .iter()
            .filter(|m| *m == "turn/start")
            .count(),
        1,
        "never two turns on one episode"
    );
    // Stopping the first turn (as a pause would, then resume) lets the queued messages run together.
    h.store.act(&sc, &ep, EpisodeAction::Pause).unwrap();
    h.bridge.interrupt(&sc, &ep, StopReason::Pause).unwrap();
    h.wait(&sc, &ep, "turn closed", |_| {
        h.store.active_turn(&sc, &ep).unwrap().is_none()
    });
    h.store.act(&sc, &ep, EpisodeAction::Resume).unwrap();
    h.bridge.resume(&sc, &ep).unwrap();
    let evs = h.wait_status(&sc, &ep, EpisodeStatus::Completed);
    check_event_log(&evs).unwrap();
    assert_eq!(
        h.requests("ws-q")
            .iter()
            .filter(|m| *m == "turn/start")
            .count(),
        2
    );
}

#[test]
fn resume_after_a_host_restart_does_not_duplicate_events() {
    let dir = tempfile::tempdir().unwrap();
    let sc = scope("ws-r", false);
    let (ep, thread, before) = {
        let h = Harness::new(dir.path());
        let ep = h.episode(&sc, "restart");
        h.say(&sc, &ep, "use a tool then hang");
        h.wait(&sc, &ep, "tool and delta", |e| {
            count(e, "tool.finished") == 1 && count(e, "message.delta") >= 1
        });
        let thread = h
            .store
            .episode(&sc, &ep)
            .unwrap()
            .runtime
            .unwrap()
            .thread_id;
        h.bridge.shutdown(); // the host dies: nothing more is recorded
        let n = h.events(&sc, &ep).len();
        (ep, thread, n)
    };
    let h = Harness::new(dir.path());
    assert!(
        h.store.active_turn(&sc, &ep).unwrap().is_some(),
        "left open"
    );
    h.bridge.recover().unwrap();
    let evs = h.events(&sc, &ep);
    check_event_log(&evs).unwrap();
    assert_eq!(status_of(&evs), Some(EpisodeStatus::WaitingUser));
    assert_eq!(
        count(&evs, "tool.started"),
        1,
        "replayed tool start not duplicated"
    );
    assert_eq!(
        count(&evs, "tool.finished"),
        1,
        "replayed tool result not duplicated"
    );
    assert!(h.store.active_turn(&sc, &ep).unwrap().is_none());
    // Recovery added only the settlement: one status change.
    assert_eq!(evs.len(), before + 1, "{:#?}", kinds(&evs));
    // A second recovery is a no-op.
    h.bridge.recover().unwrap();
    assert_eq!(h.events(&sc, &ep).len(), before + 1);

    // The same thread continues with a new message.
    h.say(&sc, &ep, "go on");
    let evs = h.wait_status(&sc, &ep, EpisodeStatus::Completed);
    check_event_log(&evs).unwrap();
    assert_eq!(count(&evs, "tool.finished"), 1);
    assert_eq!(completed_texts(&evs), vec!["Fixture answer"]);
    assert_eq!(
        h.store
            .episode(&sc, &ep)
            .unwrap()
            .runtime
            .unwrap()
            .thread_id,
        thread
    );
    assert!(h.requests("ws-r").contains(&"thread/resume".to_string()));
}

#[test]
fn concurrent_episodes_never_mix() {
    let dir = tempfile::tempdir().unwrap();
    let h = Harness::new(dir.path());
    let a = scope("ws-x", false);
    let b = scope("ws-y", false);
    let eps = [
        (a.clone(), h.episode(&a, "x1"), "tool one"),
        (a.clone(), h.episode(&a, "x2"), "opaque two"),
        (b.clone(), h.episode(&b, "y1"), "tool three"),
    ];
    for (sc, ep, text) in &eps {
        h.say(sc, ep, text);
    }
    let mut threads = std::collections::BTreeSet::new();
    for (sc, ep, text) in &eps {
        let evs = h.wait_status(sc, ep, EpisodeStatus::Completed);
        check_event_log(&evs).unwrap();
        assert!(evs.iter().all(|e| e.episode_id == *ep));
        let users: Vec<_> = evs
            .iter()
            .filter_map(|e| match &e.body {
                EventBody::UserMessage { text, .. } => Some(text.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(users, vec![*text]);
        assert_eq!(
            count(&evs, "tool.finished"),
            usize::from(text.contains("tool"))
        );
        assert_eq!(count(&evs, "runtime.opaque") > 0, text.contains("opaque"));
        threads.insert(h.store.episode(sc, ep).unwrap().runtime.unwrap().thread_id);
    }
    assert_eq!(threads.len(), 3, "one thread per episode");
    // Another workspace cannot reach the episode through the runtime either.
    assert!(h.bridge.kick(&b, &eps[0].1).is_err());
    // One app-server process per workspace.
    assert_eq!(
        h.requests("ws-x")
            .iter()
            .filter(|m| *m == "initialize")
            .count(),
        1
    );
    assert_eq!(
        h.requests("ws-y")
            .iter()
            .filter(|m| *m == "initialize")
            .count(),
        1
    );
}

#[test]
fn preflight_refuses_a_binary_that_is_not_the_pinned_one() {
    let dir = tempfile::tempdir().unwrap();
    let mut cfg = config(dir.path());
    cfg.expected_version = "0.1.0".into();
    let e = preflight(&cfg).unwrap_err();
    assert!(e.contains("not the pinned 0.1.0"), "{e}");

    let mut cfg = config(dir.path());
    cfg.expected_sha256 = Some("00".repeat(32));
    let e = preflight(&cfg).unwrap_err();
    assert!(e.contains("does not match the pinned"), "{e}");

    let mut cfg = config(dir.path());
    cfg.binary = dir.path().join("no-codex.exe");
    assert!(preflight(&cfg).unwrap_err().contains("not found"));

    let cfg = config(dir.path());
    let m = preflight(&cfg).unwrap();
    assert!(!m.binary_hash_pinned);
    let mut pinned = config(dir.path());
    pinned.expected_sha256 = Some(m.binary_sha256.clone());
    let p = preflight(&pinned).unwrap();
    assert!(p.binary_hash_pinned);
    assert_eq!(p.runtime_build_id, m.runtime_build_id);
}

#[test]
fn a_thread_the_runtime_lost_is_replaced_visibly() {
    let dir = tempfile::tempdir().unwrap();
    let sc = scope("ws-lost", false);
    let (ep, old) = {
        let h = Harness::new(dir.path());
        let ep = h.episode(&sc, "lost");
        h.say(&sc, &ep, "first");
        h.wait_status(&sc, &ep, EpisodeStatus::Completed);
        h.bridge.shutdown();
        let old = h
            .store
            .episode(&sc, &ep)
            .unwrap()
            .runtime
            .unwrap()
            .thread_id;
        (ep, old)
    };
    // The runtime's state directory is gone (e.g. restored from a backup without it).
    std::fs::remove_dir_all(dir.path().join("codex")).unwrap();
    let h = Harness::new(dir.path());
    h.say(&sc, &ep, "second");
    let evs = h.wait(&sc, &ep, "second answer", |e| completed_texts(e).len() == 2);
    check_event_log(&evs).unwrap();
    let new = h
        .store
        .episode(&sc, &ep)
        .unwrap()
        .runtime
        .unwrap()
        .thread_id;
    assert_ne!(new, old);
    let reasons: Vec<String> = evs
        .iter()
        .filter_map(|e| match &e.body {
            EventBody::StatusChanged {
                reason: Some(r), ..
            } => Some(r.clone()),
            _ => None,
        })
        .collect();
    assert!(
        reasons.iter().any(|r| r.contains(&old) && r.contains(&new)),
        "{reasons:#?}"
    );
    // Only the new message went to the new thread.
    let user_messages_sent = h
        .requests("ws-lost")
        .iter()
        .filter(|m| *m == "turn/start")
        .count();
    assert_eq!(user_messages_sent, 1);
}

#[test]
fn a_signed_in_login_home_is_shared_and_required() {
    let dir = tempfile::tempdir().unwrap();
    let home = dir.path().join("login");
    std::fs::create_dir_all(&home).unwrap();
    let mut cfg = config(dir.path());
    cfg.login_home = Some(home.clone());
    let e = preflight(&cfg).unwrap_err();
    assert!(e.contains("no Codex login"), "{e}");

    // What `codex login` leaves behind (the content is the runtime's business, never read here).
    std::fs::write(home.join("auth.json"), "{}").unwrap();
    let h = Harness::with_config(dir.path(), cfg);
    assert_eq!(
        h.bridge.manifest().credentials.as_str(),
        "shared_login_home"
    );
    let (a, b) = (scope("ws-l1", false), scope("ws-l2", false));
    let (ea, eb) = (h.episode(&a, "l1"), h.episode(&b, "l2"));
    h.say(&a, &ea, "one");
    h.say(&b, &eb, "two");
    h.wait_status(&a, &ea, EpisodeStatus::Completed);
    h.wait_status(&b, &eb, EpisodeStatus::Completed);
    // Both workspaces' app-servers ran in the signed-in home; no per-workspace homes were made.
    let log = std::fs::read_to_string(home.join("fake-requests.log")).unwrap();
    // Episode working directories stay in IGNIS's state, never in the (possibly personal) login home.
    assert!(!home.join("episodes").exists());
    assert_eq!(log.lines().filter(|l| *l == "initialize").count(), 2);
    // IGNIS's per-workspace folders hold only episode working directories: no app-server ran there.
    for e in dir.path().join("codex").read_dir().unwrap() {
        let p = e.unwrap().path();
        if p.file_name().unwrap().to_string_lossy().starts_with("ws-") {
            let names: Vec<_> = p
                .read_dir()
                .unwrap()
                .map(|x| x.unwrap().file_name().to_string_lossy().into_owned())
                .collect();
            assert_eq!(names, vec!["episodes"], "{}", p.display());
        }
    }
}

/// Reasons of the episode's `-> Running` status changes, in order (where turn notes go).
fn running_reasons(evs: &[ResearchEvent]) -> Vec<String> {
    evs.iter()
        .filter_map(|e| match &e.body {
            EventBody::StatusChanged { to, reason, .. } if *to == EpisodeStatus::Running => {
                Some(reason.clone().unwrap_or_default())
            }
            _ => None,
        })
        .collect()
}

/// `developerInstructions` of every thread method the workspace's app-server received.
fn instructions(dir: &std::path::Path, ws: &str) -> Vec<(String, Option<String>)> {
    let want = ignis_core::hashing::sha256_hex(ws.as_bytes());
    let file = dir
        .join("codex")
        .join(format!("ws-{}", &want[..16]))
        .join("fake-instructions.jsonl");
    std::fs::read_to_string(file)
        .unwrap_or_default()
        .lines()
        .map(|l| {
            let v: serde_json::Value = serde_json::from_str(l).unwrap();
            (
                v["method"].as_str().unwrap().to_string(),
                v["developerInstructions"].as_str().map(str::to_string),
            )
        })
        .collect()
}

#[test]
fn the_harness_bundle_reaches_every_thread_and_is_pinned_on_the_episode() {
    use ignis_research::runtime::{HarnessBundle, HarnessSource};
    let dir = tempfile::tempdir().unwrap();
    let tools: Vec<String> = vec!["sources_search".into(), "answer_submit".into()];
    let h1 = HarnessBundle::embedded(&tools);
    let with = |b: &HarnessBundle| {
        let mut cfg = config(dir.path());
        cfg.harness = Some(b.clone());
        Harness::with_config(dir.path(), cfg)
    };
    let sc = scope("ws-h", false);
    let (ep, build) = {
        let h = with(&h1);
        let ep = h.episode(&sc, "harness");
        h.say(&sc, &ep, "first question");
        let evs = h.wait_status(&sc, &ep, EpisodeStatus::Completed);
        check_event_log(&evs).unwrap();
        // Visible in the log: the first turn says which bundle the runtime was started with.
        let reasons = running_reasons(&evs);
        assert!(
            reasons[0].contains(&format!("harness {} injected", h1.short())),
            "{reasons:?}"
        );
        let rec = h.store.episode(&sc, &ep).unwrap();
        assert_eq!(rec.schema_version, ResearchSchema::V2);
        rec.validate().unwrap();
        let pin = rec.pins.harness.expect("harness pin");
        assert_eq!(&pin.digest, h1.digest());
        assert_eq!(pin.label, "h1");
        assert_eq!(pin.model, "fake", "the model the runtime answered with");
        assert_eq!(pin.effort.as_deref(), Some("medium"));
        assert_eq!(pin.runtime_build, h.bridge.manifest().runtime_build_id);
        let sent = instructions(dir.path(), "ws-h");
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0].0, "thread/start");
        assert_eq!(sent[0].1.as_deref(), Some(h1.text()));
        h.bridge.shutdown();
        (ep, pin.runtime_build)
    };

    // A new host process resumes the thread with the same bytes; the pin does not move.
    {
        let h = with(&h1);
        h.say(&sc, &ep, "second question");
        let evs = h.wait(&sc, &ep, "second answer", |e| completed_texts(e).len() == 2);
        h.wait_status(&sc, &ep, EpisodeStatus::Completed);
        let reasons = running_reasons(&evs);
        assert!(!reasons[1].contains("harness"), "{reasons:?}");
        let sent = instructions(dir.path(), "ws-h");
        assert_eq!(sent[1].0, "thread/resume");
        assert_eq!(
            sent[1].1.as_deref(),
            Some(h1.text()),
            "byte-identical prefix"
        );
        let pin = h.store.episode(&sc, &ep).unwrap().pins.harness.unwrap();
        assert_eq!(&pin.digest, h1.digest());
        assert_eq!(pin.runtime_build, build);
        h.bridge.shutdown();
    }

    // A host with another bundle: Codex applies it on resume, and the switch is said in the log.
    let mut src = HarnessSource::embedded();
    src.investigator
        .push_str("\nAn extra rule for this variant.\n");
    let h2 = HarnessBundle::render("h2", &src, &tools).unwrap();
    let h = with(&h2);
    h.say(&sc, &ep, "third question");
    let evs = h.wait(&sc, &ep, "third answer", |e| completed_texts(e).len() == 3);
    let reasons = running_reasons(&evs);
    assert!(
        reasons[2].contains(&format!(
            "harness switched from {} to {}",
            h1.short(),
            h2.short()
        )),
        "{reasons:?}"
    );
    assert_eq!(
        instructions(dir.path(), "ws-h")[2].1.as_deref(),
        Some(h2.text())
    );
    let pin = h.store.episode(&sc, &ep).unwrap().pins.harness.unwrap();
    assert_eq!(&pin.digest, h2.digest());
    assert_eq!(pin.label, "h2");
}

/// The cached prefix is the same bytes for episodes of different workspaces and scopes: nothing
/// episode-, scope- or time-specific enters it.
#[test]
fn the_harness_prefix_is_byte_identical_across_episodes_and_scopes() {
    use ignis_research::runtime::HarnessBundle;
    let dir = tempfile::tempdir().unwrap();
    // As served: skills on demand (the prefix carries the skill index, not the texts).
    let bundle = HarnessBundle::embedded(&[
        "sources_search".into(),
        "answer_submit".into(),
        "skills_read".into(),
    ]);
    assert!(!bundle.text().contains("<skill "));
    let mut cfg = config(dir.path());
    cfg.harness = Some(bundle.clone());
    let h = Harness::with_config(dir.path(), cfg);
    let (a, b) = (scope("ws-p1", false), scope("ws-p2", true));
    let (ea, eb) = (h.episode(&a, "goal-a"), h.episode(&b, "goal-b"));
    h.say(&a, &ea, "a question about flames");
    h.say(&b, &eb, "a different question");
    h.wait_status(&a, &ea, EpisodeStatus::Completed);
    h.wait_status(&b, &eb, EpisodeStatus::Completed);
    let sa = instructions(dir.path(), "ws-p1");
    let sb = instructions(dir.path(), "ws-p2");
    assert_eq!(sa[0].1.as_deref(), Some(bundle.text()));
    assert_eq!(sa[0].1, sb[0].1, "byte-identical prefix");
    let pin = |sc: &ExecutionScope, ep: &EpisodeId| {
        h.store
            .episode(sc, ep)
            .unwrap()
            .pins
            .harness
            .unwrap()
            .digest
    };
    assert_eq!(pin(&a, &ea), pin(&b, &eb));
}

/// One line per record of a fake app-server log of the workspace's `CODEX_HOME`.
fn fake_log(dir: &std::path::Path, ws: &str, file: &str) -> Vec<serde_json::Value> {
    let want = ignis_core::hashing::sha256_hex(ws.as_bytes());
    let path = dir
        .join("codex")
        .join(format!("ws-{}", &want[..16]))
        .join(file);
    std::fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect()
}

/// Codex's coding-agent base prompt is replaced by the IGNIS base (`model_instructions_file`), the
/// built-in tools and prompt sections a research agent never uses are switched off, and the whole
/// prefix the model sees before the turn input (base + harness) is the same bytes for every
/// workspace, scope and process, so a prompt cache keyed on it can serve every episode.
#[test]
fn the_base_prompt_replaces_codex_defaults_and_the_whole_prefix_is_stable() {
    use ignis_research::runtime::{codex_base_instructions, HarnessBundle, CODEX_BASE_LABEL};
    let dir = tempfile::tempdir().unwrap();
    let bundle = HarnessBundle::embedded(&["sources_search".into(), "answer_submit".into()]);
    let mut cfg = config(dir.path());
    cfg.harness = Some(bundle.clone());
    let h = Harness::with_config(dir.path(), cfg);
    let base = codex_base_instructions();
    assert!(!base.contains('\r') && base.contains("IGNIS"));

    // The manifest names the base prompt by content, so traces say which bytes ran.
    let overrides = &h.bridge.manifest().config_overrides;
    let digest = ignis_core::hashing::sha256_hex(base.as_bytes());
    assert!(
        overrides.contains(&format!(
            "model_instructions_file={CODEX_BASE_LABEL} sha256:{digest}"
        )),
        "{overrides:?}"
    );

    let (a, b) = (scope("ws-b1", false), scope("ws-b2", true));
    let (ea, eb) = (h.episode(&a, "base-a"), h.episode(&b, "base-b"));
    h.say(&a, &ea, "a question about flames");
    h.say(&b, &eb, "a different question");
    h.wait_status(&a, &ea, EpisodeStatus::Completed);
    h.wait_status(&b, &eb, EpisodeStatus::Completed);
    // A second process for the first workspace (a host restart) resumes with the same prefix.
    h.bridge.shutdown();
    let h = {
        let mut cfg = config(dir.path());
        cfg.harness = Some(bundle.clone());
        Harness::with_config(dir.path(), cfg)
    };
    h.say(&a, &ea, "a follow-up");
    h.wait(&a, &ea, "second answer", |e| completed_texts(e).len() == 2);

    let mut sent = fake_log(dir.path(), "ws-b1", "fake-instructions.jsonl");
    sent.extend(fake_log(dir.path(), "ws-b2", "fake-instructions.jsonl"));
    let methods: Vec<_> = sent.iter().map(|v| v["method"].as_str().unwrap()).collect();
    assert_eq!(methods, ["thread/start", "thread/resume", "thread/start"]);
    for v in &sent {
        assert_eq!(v["baseInstructions"].as_str(), Some(base.as_str()), "{v}");
        assert_eq!(
            v["developerInstructions"].as_str(),
            Some(bundle.text()),
            "{v}"
        );
    }

    // Every app-server process got the same switches (they are process-wide, not per thread).
    let mut procs = fake_log(dir.path(), "ws-b1", "fake-overrides.jsonl");
    procs.extend(fake_log(dir.path(), "ws-b2", "fake-overrides.jsonl"));
    assert_eq!(procs.len(), 3);
    for p in &procs {
        let o: Vec<&str> = p
            .as_array()
            .unwrap()
            .iter()
            .map(|x| x.as_str().unwrap())
            .collect();
        for off in [
            "features.shell_tool=false",
            "features.goals=false",
            "features.view_image=false",
            "features.sleep_tool=false",
            "skills.bundled.enabled=false",
            "skills.include_instructions=false",
            "include_permissions_instructions=false",
            "include_collaboration_mode_instructions=false",
            "include_apps_instructions=false",
            "agents.enabled=false",
            "include_environment_context=false",
            "web_search=\"disabled\"",
        ] {
            assert!(o.contains(&off), "{off} missing from {o:?}");
        }
        let file: Vec<_> = o
            .iter()
            .filter_map(|x| x.strip_prefix("model_instructions_file="))
            .collect();
        assert_eq!(file.len(), 1, "{o:?}");
        assert_eq!(
            file[0],
            procs[0]
                .as_array()
                .unwrap()
                .iter()
                .find_map(|x| { x.as_str().unwrap().strip_prefix("model_instructions_file=") })
                .unwrap(),
            "one file for every process"
        );
        let path: String = serde_json::from_str(file[0]).unwrap();
        assert!(std::path::Path::new(&path).is_absolute(), "{path}");
        assert!(path.contains(&digest[..16]), "named by its content: {path}");
    }
}
