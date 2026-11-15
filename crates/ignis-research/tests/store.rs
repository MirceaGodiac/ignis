//! Conformance of the research state store: idempotency, workspace isolation, gap-free replay,
//! leases with fencing, reconciliation and restart.

use std::collections::BTreeSet;
use std::io::Read;
use std::time::Duration;

use ignis_core::contracts::*;
use ignis_core::services::{ArtifactDraft, ArtifactStore};
use ignis_research::store::{
    EpisodeAction, JobOutcome, JobPriority, NewJob, Replay, UploadSpec, UploadStatus,
};
use ignis_research::{LocalArtifactStore, ResearchStore};
use serde_json::json;

fn scope(ws: &str) -> ExecutionScope {
    ExecutionScope {
        scope_id: ScopeId::new(format!("scope-{ws}")).unwrap(),
        workspace_id: WorkspaceId::new(ws).unwrap(),
        principal: PrincipalId::new(format!("user-{ws}")).unwrap(),
        role: ScopeRole::Research,
        partitions: BTreeSet::from([Partition::Train]),
        capabilities: BTreeSet::from([Capability::ReadSources, Capability::RunModels]),
        corpus_snapshot: None,
        parent_scope_id: None,
    }
}

fn crid(s: &str) -> ClientRequestId {
    ClientRequestId::new(s).unwrap()
}

fn all_events(store: &ResearchStore, sc: &ExecutionScope, id: &EpisodeId) -> Vec<ResearchEvent> {
    match store.replay(sc, id, 0, 10_000).unwrap() {
        Replay::Events { events, .. } => events,
        Replay::Gone { .. } => panic!("gone"),
    }
}

#[test]
fn create_is_idempotent_and_workspaces_are_isolated() {
    let dir = tempfile::tempdir().unwrap();
    let store = ResearchStore::open(dir.path().join("state.sqlite")).unwrap();
    let a = scope("ws-a");
    let (e1, created) = store
        .create_episode(
            &a,
            "flame spread",
            &crid("c1"),
            Default::default(),
            Origin::System,
        )
        .unwrap();
    assert!(created);
    let (e2, created) = store
        .create_episode(
            &a,
            "flame spread",
            &crid("c1"),
            Default::default(),
            Origin::System,
        )
        .unwrap();
    assert!(!created);
    assert_eq!(e1.episode_id, e2.episode_id);
    let err = store
        .create_episode(
            &a,
            "other goal",
            &crid("c1"),
            Default::default(),
            Origin::System,
        )
        .unwrap_err();
    assert_eq!(err.code, ErrorCode::Conflict);

    // Another workspace neither sees the episode nor collides on the same request id.
    let b = scope("ws-b");
    assert_eq!(
        store.episode(&b, &e1.episode_id).unwrap_err().code,
        ErrorCode::NotFound
    );
    let (eb, created) = store
        .create_episode(
            &b,
            "flame spread",
            &crid("c1"),
            Default::default(),
            Origin::System,
        )
        .unwrap();
    assert!(created);
    assert_ne!(eb.episode_id, e1.episode_id);
    assert_eq!(store.episodes(&a, 10).unwrap().len(), 1);
    assert_eq!(
        store
            .post_message(&b, &e1.episode_id, "hi", &crid("m"))
            .unwrap_err()
            .code,
        ErrorCode::NotFound
    );
}

#[test]
fn messages_are_idempotent_and_the_log_replays_gap_free() {
    let dir = tempfile::tempdir().unwrap();
    let store = ResearchStore::open(dir.path().join("state.sqlite")).unwrap();
    let sc = scope("ws");
    let (ep, _) = store
        .create_episode(&sc, "goal", &crid("c"), Default::default(), Origin::System)
        .unwrap();
    let id = ep.episode_id;
    let (m1, fresh) = store.post_message(&sc, &id, "first", &crid("m1")).unwrap();
    assert!(fresh);
    let (again, fresh) = store.post_message(&sc, &id, "first", &crid("m1")).unwrap();
    assert!(!fresh);
    assert_eq!(again, m1);
    assert_eq!(
        store
            .post_message(&sc, &id, "changed", &crid("m1"))
            .unwrap_err()
            .code,
        ErrorCode::Conflict
    );
    store.post_message(&sc, &id, "second", &crid("m2")).unwrap();
    store.act(&sc, &id, EpisodeAction::Pause).unwrap();
    store.act(&sc, &id, EpisodeAction::Pause).unwrap(); // no-op
    store.act(&sc, &id, EpisodeAction::Resume).unwrap();

    let events = all_events(&store, &sc, &id);
    check_event_log(&events).unwrap();
    assert_eq!(
        events.iter().map(|e| e.seq).collect::<Vec<_>>(),
        vec![1, 2, 3, 4]
    );
    assert_eq!(store.episode(&sc, &id).unwrap().last_seq, 4);

    // Resume after a cursor: only later events; a cursor ahead of the log is Gone.
    match store.replay(&sc, &id, 2, 100).unwrap() {
        Replay::Events { events, last_seq } => {
            assert_eq!(events.iter().map(|e| e.seq).collect::<Vec<_>>(), vec![3, 4]);
            assert_eq!(last_seq, 4);
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(
        store.replay(&sc, &id, 99, 100).unwrap(),
        Replay::Gone { last_seq: 4 }
    );

    // Disallowed transition and a cancelled episode refusing messages.
    assert_eq!(
        store
            .act(&sc, &id, EpisodeAction::Resume)
            .map(|e| e.status)
            .unwrap(),
        EpisodeStatus::Ready
    );
    store.act(&sc, &id, EpisodeAction::Cancel).unwrap();
    assert_eq!(
        store.act(&sc, &id, EpisodeAction::Pause).unwrap_err().code,
        ErrorCode::Conflict
    );
    assert_eq!(
        store
            .post_message(&sc, &id, "late", &crid("m3"))
            .unwrap_err()
            .code,
        ErrorCode::Conflict
    );
}

fn job(episode: Option<&EpisodeId>, idempotent: bool) -> NewJob {
    NewJob {
        kind: "models_forecast".into(),
        input: json!({"x": 1}),
        episode_id: episode.cloned(),
        client_request_id: None,
        idempotent,
        priority: JobPriority::Interactive,
        causation: Causation::default(),
    }
}

#[test]
fn jobs_use_leases_and_fencing_tokens() {
    let dir = tempfile::tempdir().unwrap();
    let store = ResearchStore::open(dir.path().join("state.sqlite")).unwrap();
    let sc = scope("ws");
    let (ep, _) = store
        .create_episode(&sc, "goal", &crid("c"), Default::default(), Origin::System)
        .unwrap();
    let j = store
        .submit_job(&sc, job(Some(&ep.episode_id), true))
        .unwrap();
    assert_eq!(j.status, JobStatus::Queued);
    assert!(store
        .claim_job("r1", &["other_kind"], Duration::from_secs(30))
        .unwrap()
        .is_none());

    let c1 = store
        .claim_job("r1", &["models_forecast"], Duration::from_millis(1))
        .unwrap()
        .unwrap();
    assert_eq!(
        c1.scope, sc,
        "the job runs under the scope it was submitted with"
    );
    std::thread::sleep(Duration::from_millis(10));
    // The lease expired: the idempotent job is re-queued and claimed by another runner.
    let reaped = store.reap_expired(false).unwrap();
    assert_eq!(reaped[0].status, JobStatus::Queued);
    let c2 = store
        .claim_job("r2", &["models_forecast"], Duration::from_secs(30))
        .unwrap()
        .unwrap();
    assert!(c2.fence > c1.fence);
    assert_eq!(c2.attempt, 2);

    // The first runner wakes up: its token is stale, it cannot heartbeat nor finish.
    let stale = store
        .finish_job(
            &c1.job_id,
            c1.fence,
            JobOutcome::Succeeded { outputs: vec![] },
        )
        .unwrap_err();
    assert_eq!(stale.code, ErrorCode::Conflict);
    assert!(store
        .heartbeat(&c1.job_id, c1.fence, Duration::from_secs(1), None)
        .is_err());

    let progress = JobProgress {
        completed: 1,
        total: Some(2),
        unit: "draws".into(),
    };
    assert!(!store
        .heartbeat(
            &c2.job_id,
            c2.fence,
            Duration::from_secs(30),
            Some(progress.clone())
        )
        .unwrap());
    let done = store
        .finish_job(
            &c2.job_id,
            c2.fence,
            JobOutcome::Succeeded { outputs: vec![] },
        )
        .unwrap();
    assert_eq!(done.status, JobStatus::Succeeded);
    assert_eq!(store.job(&sc, &j.job_id).unwrap().progress, Some(progress));
    assert_eq!(
        store.job(&scope("other"), &j.job_id).unwrap_err().code,
        ErrorCode::NotFound
    );

    // Every status change of the job is in the episode log, in order, and passes the log checks.
    let events = all_events(&store, &sc, &ep.episode_id);
    check_event_log(&events).unwrap();
    let statuses: Vec<JobStatus> = events
        .iter()
        .filter_map(|e| match &e.body {
            EventBody::JobUpdated { job } => Some(job.status),
            _ => None,
        })
        .collect();
    assert_eq!(
        statuses,
        vec![
            JobStatus::Queued,
            JobStatus::Running,
            JobStatus::Queued,
            JobStatus::Running,
            JobStatus::Running, // progress report
            JobStatus::Succeeded
        ]
    );
}

#[test]
fn interrupted_effects_are_reconciled_and_state_survives_restart() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("state.sqlite");
    let sc = scope("ws");
    let (ep_id, effectful, pure, to_cancel) = {
        let store = ResearchStore::open(&path).unwrap();
        let (ep, _) = store
            .create_episode(&sc, "goal", &crid("c"), Default::default(), Origin::System)
            .unwrap();
        let effectful = store.submit_job(&sc, job(None, false)).unwrap();
        let pure = store.submit_job(&sc, job(None, true)).unwrap();
        let to_cancel = store
            .submit_job(&sc, job(Some(&ep.episode_id), true))
            .unwrap();
        for _ in 0..3 {
            store
                .claim_job(
                    "old-process",
                    &["models_forecast"],
                    Duration::from_secs(3600),
                )
                .unwrap()
                .unwrap();
        }
        store.cancel_job(&sc, &to_cancel.job_id).unwrap();
        (
            ep.episode_id,
            effectful.job_id,
            pure.job_id,
            to_cancel.job_id,
        )
        // the process "dies" here with three leases held
    };
    let store = ResearchStore::open(&path).unwrap();
    assert_eq!(store.episode(&sc, &ep_id).unwrap().goal, "goal");
    store.reap_expired(true).unwrap();
    assert_eq!(
        store.job(&sc, &effectful).unwrap().status,
        JobStatus::Reconciling
    );
    assert_eq!(store.job(&sc, &pure).unwrap().status, JobStatus::Queued);
    assert_eq!(
        store.job(&sc, &to_cancel).unwrap().status,
        JobStatus::Cancelled
    );
    check_event_log(&all_events(&store, &sc, &ep_id)).unwrap();

    // Cancelling the episode cancels its remaining jobs.
    let more = store.submit_job(&sc, job(Some(&ep_id), true)).unwrap();
    store.act(&sc, &ep_id, EpisodeAction::Cancel).unwrap();
    assert_eq!(
        store.job(&sc, &more.job_id).unwrap().status,
        JobStatus::Cancelled
    );
    check_event_log(&all_events(&store, &sc, &ep_id)).unwrap();
}

#[test]
fn job_submission_is_idempotent() {
    let dir = tempfile::tempdir().unwrap();
    let store = ResearchStore::open(dir.path().join("state.sqlite")).unwrap();
    let sc = scope("ws");
    let mut j = job(None, true);
    j.client_request_id = Some(crid("k"));
    let a = store.submit_job(&sc, j.clone()).unwrap();
    let b = store.submit_job(&sc, j.clone()).unwrap();
    assert_eq!(a.job_id, b.job_id);
    j.input = json!({"x": 2});
    assert_eq!(
        store.submit_job(&sc, j).unwrap_err().code,
        ErrorCode::Conflict
    );
}

fn draft() -> ArtifactDraft {
    ArtifactDraft {
        media_type: "application/json".into(),
        contract: Some("AnalysisArtifact".into()),
        produced_by: Producer::default(),
        inputs: vec![],
        origin: Origin::SyntheticFixture,
    }
}

#[test]
fn artifacts_are_content_addressed_ranged_and_scoped() {
    let dir = tempfile::tempdir().unwrap();
    let store = LocalArtifactStore::open(dir.path()).unwrap();
    let sc = scope("ws");
    let bytes = b"{\"hello\": \"world\"}".to_vec();
    let a = store.put(&sc, draft(), &mut bytes.as_slice()).unwrap();
    assert_eq!(a.sha256, Digest::of_bytes(&bytes));
    assert_eq!(a.byte_length, bytes.len() as u64);
    let again = store.put(&sc, draft(), &mut bytes.as_slice()).unwrap();
    assert_eq!(again, a, "same bytes + metadata = same reference");
    assert_eq!(store.stat(&sc, &a.artifact_id).unwrap(), a);

    let mut part = String::new();
    store
        .open(&sc, &a.artifact_id, 2, Some(5))
        .unwrap()
        .read_to_string(&mut part)
        .unwrap();
    assert_eq!(part, "hello");
    let mut tail = Vec::new();
    store
        .open(&sc, &a.artifact_id, 10, None)
        .unwrap()
        .read_to_end(&mut tail)
        .unwrap();
    assert_eq!(tail, bytes[10..]);
    assert!(store.open(&sc, &a.artifact_id, 1000, None).is_err());

    // Another workspace cannot see it, and garbage ids are simply not found.
    assert_eq!(
        store
            .stat(&scope("other"), &a.artifact_id)
            .unwrap_err()
            .code,
        ErrorCode::NotFound
    );
    let bad = ArtifactId::new("art_../../etc").unwrap();
    assert_eq!(store.stat(&sc, &bad).unwrap_err().code, ErrorCode::NotFound);
    assert_eq!(
        std::fs::read_dir(dir.path().join("staging"))
            .unwrap()
            .count(),
        0
    );
}

#[test]
fn higher_priority_jobs_are_claimed_first() {
    let dir = tempfile::tempdir().unwrap();
    let store = ResearchStore::open(dir.path().join("state.sqlite")).unwrap();
    let sc = scope("ws");
    let mut ids = Vec::new();
    for p in [
        JobPriority::Ingestion,
        JobPriority::Evaluation,
        JobPriority::Interactive,
    ] {
        let mut j = job(None, true);
        j.priority = p;
        ids.push(store.submit_job(&sc, j).unwrap().job_id);
    }
    let order: Vec<JobId> = (0..3)
        .map(|_| {
            store
                .claim_job("r", &["models_forecast"], Duration::from_secs(30))
                .unwrap()
                .unwrap()
                .job_id
        })
        .collect();
    assert_eq!(order, vec![ids[2].clone(), ids[1].clone(), ids[0].clone()]);
}

#[test]
fn a_version_1_store_is_migrated_in_place() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("state.sqlite");
    {
        let c = rusqlite::Connection::open(&path).unwrap();
        c.execute_batch(
            "CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
             INSERT INTO meta VALUES ('schema_version', '1');
             CREATE TABLE jobs (job_id TEXT PRIMARY KEY, workspace_id TEXT NOT NULL, episode_id TEXT,
               kind TEXT NOT NULL, status TEXT NOT NULL, record TEXT NOT NULL, input TEXT NOT NULL,
               scope TEXT NOT NULL, idempotent INTEGER NOT NULL, lease_owner TEXT, lease_expires_ms INTEGER,
               fence INTEGER NOT NULL DEFAULT 0, attempts INTEGER NOT NULL DEFAULT 0, queued_ms INTEGER NOT NULL);",
        )
        .unwrap();
    }
    let store = ResearchStore::open(&path).unwrap();
    store.submit_job(&scope("ws"), job(None, true)).unwrap();
    drop(store);
    let c = rusqlite::Connection::open(&path).unwrap();
    let v: String = c
        .query_row(
            "SELECT value FROM meta WHERE key = 'schema_version'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(v, "5");
    // 2 -> 3 adds the agent-runtime tables, 3 -> 4 the episode evidence of `answer_submit`,
    // 4 -> 5 the scopes recovery drives unsent messages with.
    let tables: i64 = c
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE name IN ('runtime_turns', 'runtime_marks', 'episode_evidence', 'episode_drivers')",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(tables, 4);
}

/// 4 -> 5: an existing `runtime_turns` gains the columns that bound the host's continuation, and
/// turns run on it.
#[test]
fn a_version_4_store_gains_the_turn_columns() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("state.sqlite");
    {
        let c = rusqlite::Connection::open(&path).unwrap();
        c.execute_batch(
            "CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
             INSERT INTO meta VALUES ('schema_version', '4');
             CREATE TABLE runtime_turns (turn_key TEXT PRIMARY KEY, episode_id TEXT NOT NULL,
               workspace_id TEXT NOT NULL, scope TEXT NOT NULL, runtime_turn_id TEXT,
               through_seq INTEGER NOT NULL, state TEXT NOT NULL, outcome TEXT,
               created_ms INTEGER NOT NULL);",
        )
        .unwrap();
    }
    let store = ResearchStore::open(&path).unwrap();
    let sc = scope("ws");
    let (ep, _) = store
        .create_episode(&sc, "g", &crid("c"), Default::default(), Origin::System)
        .unwrap();
    store
        .post_message(&sc, &ep.episode_id, "q", &crid("m"))
        .unwrap();
    let t = store
        .begin_turn(&sc, &ep.episode_id, None, true)
        .unwrap()
        .unwrap();
    assert!(store.continuation_due(&ep.episode_id, &t.turn_key).unwrap());
}

#[test]
fn uploads_are_scoped_and_move_through_their_states() {
    let dir = tempfile::tempdir().unwrap();
    let store = ResearchStore::open(dir.path().join("state.sqlite")).unwrap();
    let sc = scope("ws");
    let spec = UploadSpec {
        media_type: "text/csv".into(),
        byte_length: 3,
        sha256: Digest::of_bytes(b"a,b"),
        contract: None,
    };
    let u = store
        .create_upload(&sc, spec.clone(), Some(&crid("u")), Duration::from_secs(60))
        .unwrap();
    assert_eq!(u.status, UploadStatus::Initiated);
    let again = store
        .create_upload(&sc, spec, Some(&crid("u")), Duration::from_secs(60))
        .unwrap();
    assert_eq!(again.upload_id, u.upload_id);
    assert_eq!(
        store
            .upload(&scope("other"), &u.upload_id)
            .unwrap_err()
            .code,
        ErrorCode::NotFound
    );
    let fake = LocalArtifactStore::open(dir.path().join("a"))
        .unwrap()
        .put(&sc, draft(), &mut b"a,b".as_slice())
        .unwrap();
    assert_eq!(
        store
            .upload_completed(&sc, &u.upload_id, &fake)
            .unwrap_err()
            .code,
        ErrorCode::Conflict,
        "no bytes yet"
    );
    store.upload_received(&sc, &u.upload_id).unwrap();
    let done = store.upload_completed(&sc, &u.upload_id, &fake).unwrap();
    assert_eq!(done.status, UploadStatus::Completed);
    assert_eq!(done.artifact, Some(fake));
}

#[test]
fn the_c0_research_fixture_imports_as_a_replayable_episode() {
    let root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/contracts/research");
    let episode: ResearchEpisode =
        serde_json::from_str(&std::fs::read_to_string(root.join("episode.json")).unwrap()).unwrap();
    let events: Vec<ResearchEvent> =
        serde_json::from_str(&std::fs::read_to_string(root.join("events.json")).unwrap()).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let store = ResearchStore::open(dir.path().join("state.sqlite")).unwrap();
    let sc = scope("ws");
    assert!(store
        .import_episode(&sc, episode.clone(), events.clone())
        .unwrap());
    assert!(!store
        .import_episode(&sc, episode.clone(), events.clone())
        .unwrap());
    let ep = store.episode(&sc, &episode.episode_id).unwrap();
    assert_eq!((ep.last_seq, ep.status), (15, EpisodeStatus::Completed));
    assert_eq!(ep.origin, Origin::SyntheticFixture);
    assert_eq!(all_events(&store, &sc, &ep.episode_id), events);
    // Continuing the imported episode appends after the fixture log.
    let (m, _) = store
        .post_message(&sc, &ep.episode_id, "follow-up", &crid("f"))
        .unwrap();
    assert_eq!(m.seq, 16);
}

#[test]
fn a_first_turn_receives_unanswered_messages_even_after_job_events() {
    let dir = tempfile::tempdir().unwrap();
    let store = ResearchStore::open(dir.path().join("state.sqlite")).unwrap();
    let sc = scope("ws");
    let (ep, _) = store
        .create_episode(&sc, "g", &crid("c"), Default::default(), Origin::System)
        .unwrap();
    let id = ep.episode_id;
    store
        .post_message(&sc, &id, "first question", &crid("m1"))
        .unwrap();
    // A job attached to the episode is not an answer to the message.
    store.submit_job(&sc, job(Some(&id), true)).unwrap();
    let t = store
        .begin_turn(&sc, &id, None, false)
        .unwrap()
        .expect("a turn");
    assert_eq!(
        t.messages.iter().map(|m| m.1.as_str()).collect::<Vec<_>>(),
        vec!["first question"]
    );
    // One open turn per episode; its messages are never handed out again.
    assert!(store.begin_turn(&sc, &id, None, false).unwrap().is_none());
    store.finish_turn(&t.turn_key, "completed").unwrap();
    assert!(store.begin_turn(&sc, &id, None, false).unwrap().is_none());
}

/// The host continues a turn at most once, and only when it ended without an accepted answer: a
/// turn with an `answer.completed` is not continued (a pending continuation then completes), and a
/// continuation is never continued again. A resume continues an unanswered question once.
#[test]
fn a_turn_is_continued_once_and_only_without_an_answer() {
    let dir = tempfile::tempdir().unwrap();
    let store = ResearchStore::open(dir.path().join("state.sqlite")).unwrap();
    let artifacts = LocalArtifactStore::open(dir.path().join("artifacts")).unwrap();
    let sc = scope("ws");
    let (ep, _) = store
        .create_episode(&sc, "g", &crid("c"), Default::default(), Origin::System)
        .unwrap();
    let id = ep.episode_id;
    let status = |s: &ResearchStore| s.episode(&sc, &id).unwrap().status;

    // No answer: due; the continuation's input is one host note, and it is never continued.
    store.post_message(&sc, &id, "q1", &crid("m1")).unwrap();
    let t1 = store.begin_turn(&sc, &id, None, true).unwrap().unwrap();
    assert!(!t1.continuation);
    assert!(store.continuation_due(&id, &t1.turn_key).unwrap());
    store.finish_turn(&t1.turn_key, "completed").unwrap();
    let c1 = store.begin_turn(&sc, &id, None, true).unwrap().unwrap();
    assert!(c1.continuation);
    assert_eq!(c1.messages.len(), 1);
    assert!(
        c1.messages[0].1.contains("answer_submit"),
        "{:?}",
        c1.messages
    );
    assert!(!store.continuation_due(&id, &c1.turn_key).unwrap());
    store.finish_turn(&c1.turn_key, "completed").unwrap();
    // Left running by a bridge that skipped the status: completes, no third round.
    assert!(store.begin_turn(&sc, &id, None, true).unwrap().is_none());
    assert_eq!(status(&store), EpisodeStatus::Completed);

    // An accepted answer: not due; a pending continuation completes instead.
    store.post_message(&sc, &id, "q2", &crid("m2")).unwrap();
    let t2 = store.begin_turn(&sc, &id, None, true).unwrap().unwrap();
    let art = artifacts.put(&sc, draft(), &mut &b"{}"[..]).unwrap();
    store
        .complete_answer(
            &sc,
            &id,
            None,
            Causation::default(),
            EventBody::AnswerCompleted {
                answer_id: AnswerId::new("ans_1").unwrap(),
                status: AnswerStatus::Complete,
                answer: art,
            },
        )
        .unwrap();
    assert!(!store.continuation_due(&id, &t2.turn_key).unwrap());
    store.finish_turn(&t2.turn_key, "completed").unwrap();
    assert!(store.begin_turn(&sc, &id, None, true).unwrap().is_none());
    assert_eq!(status(&store), EpisodeStatus::Completed);

    // Paused mid-question, then resumed: one continuation quoting the question.
    store.post_message(&sc, &id, "q3", &crid("m3")).unwrap();
    let t3 = store.begin_turn(&sc, &id, None, true).unwrap().unwrap();
    store.act(&sc, &id, EpisodeAction::Pause).unwrap();
    store.finish_turn(&t3.turn_key, "interrupted").unwrap();
    assert!(store.begin_turn(&sc, &id, None, true).unwrap().is_none());
    store.act(&sc, &id, EpisodeAction::Resume).unwrap();
    let c3 = store.begin_turn(&sc, &id, None, true).unwrap().unwrap();
    assert!(c3.continuation);
    assert!(c3.messages[0].1.contains("> q3"), "{:?}", c3.messages);
    check_event_log(&all_events(&store, &sc, &id)).unwrap();
}

/// After a restart, an episode whose message no turn received is driven again (with the scope of
/// that message); a paused one, or one whose messages were all handed to a turn, is not.
#[test]
fn messages_no_turn_received_are_found_for_recovery() {
    let dir = tempfile::tempdir().unwrap();
    let store = ResearchStore::open(dir.path().join("state.sqlite")).unwrap();
    let sc = scope("ws");
    let new = |k: &str| {
        store
            .create_episode(&sc, k, &crid(k), Default::default(), Origin::System)
            .unwrap()
            .0
            .episode_id
    };
    // `d` was created and never messaged: an idle conversation, not a stranded one.
    let (stranded, paused, sent, _idle) = (new("a"), new("b"), new("c"), new("d"));
    for (ep, k) in [(&stranded, "m1"), (&paused, "m2"), (&sent, "m3")] {
        store.post_message(&sc, ep, "q", &crid(k)).unwrap();
    }
    store.act(&sc, &paused, EpisodeAction::Pause).unwrap();
    let t = store.begin_turn(&sc, &sent, None, false).unwrap().unwrap();
    store.finish_turn(&t.turn_key, "completed").unwrap();
    store
        .append(
            &sc,
            &sent,
            vec![(
                Causation::default(),
                EventBody::StatusChanged {
                    from: EpisodeStatus::Running,
                    to: EpisodeStatus::Completed,
                    reason: None,
                },
            )],
        )
        .unwrap();
    let found = store.episodes_to_drive().unwrap();
    let ids: Vec<&EpisodeId> = found.iter().map(|(e, _)| e).collect();
    assert_eq!(ids, vec![&stranded], "{found:?}");
    assert_eq!(found[0].1, sc);
}

/// A job submitted through the MCP server knows only its workspace; the runtime's tool result that
/// carries its handle attaches it to the episode, so its updates reach the log and an episode
/// cancel reaches it. A job of another workspace, or one already attached, is left alone.
#[test]
fn a_tool_result_attaches_its_workspace_job_to_the_episode() {
    use ignis_research::store::{RuntimeChange, RuntimeRecord};
    let dir = tempfile::tempdir().unwrap();
    let store = ResearchStore::open(dir.path().join("state.sqlite")).unwrap();
    let (a, b) = (scope("ws-a"), scope("ws-b"));
    let (ep, _) = store
        .create_episode(&a, "q", &crid("e1"), Default::default(), Origin::System)
        .unwrap();
    let id = ep.episode_id.clone();
    let mine = store.submit_job(&a, job(None, true)).unwrap();
    let foreign = store.submit_job(&b, job(None, true)).unwrap();
    let finished = |call: &str, j: &JobRef| RuntimeRecord {
        mark: Some(format!("tool-{call}")),
        causation: Causation::default(),
        change: RuntimeChange::Event(EventBody::ToolFinished {
            result: ToolResult {
                schema_version: ToolsSchema::V1,
                call_id: ToolCallId::new(call).unwrap(),
                tool: "models_forecast".into(),
                execution_status: ExecutionStatus::Accepted,
                scientific_status: ScientificStatus::NotAssessed,
                result: None,
                job: Some(j.clone()),
                artifact_refs: vec![],
                error: None,
                origin: Origin::System,
            },
        }),
    };
    let out = store
        .record_runtime(
            &a,
            &id,
            vec![finished("c1", &mine), finished("c2", &foreign)],
        )
        .unwrap();
    let kinds: Vec<&str> = out
        .iter()
        .map(|e| match &e.body {
            EventBody::ToolFinished { .. } => "tool.finished",
            EventBody::JobUpdated { .. } => "job.updated",
            _ => "other",
        })
        .collect();
    assert_eq!(kinds, ["tool.finished", "job.updated", "tool.finished"]);
    match &out[1].body {
        EventBody::JobUpdated { job } => assert_eq!(job.job_id, mine.job_id),
        _ => unreachable!(),
    }
    assert_eq!(out[1].causation.job_id.as_ref(), Some(&mine.job_id));
    check_event_log(&all_events(&store, &a, &id)).unwrap();

    // Replayed records (same marks) attach nothing twice.
    assert!(store
        .record_runtime(&a, &id, vec![finished("c1", &mine)])
        .unwrap()
        .is_empty());

    // Cancelling the episode now reaches the attached job, not the other workspace's.
    store.act(&a, &id, EpisodeAction::Cancel).unwrap();
    assert_eq!(
        store.job(&a, &mine.job_id).unwrap().status,
        JobStatus::Cancelled
    );
    assert_eq!(
        store.job(&b, &foreign.job_id).unwrap().status,
        JobStatus::Queued
    );
}
