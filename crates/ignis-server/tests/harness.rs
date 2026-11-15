//! M10 end to end: `ignis harness run` (parsed like the CLI) over the dev fixture task set with the
//! fake Codex app-server → one validated, replayable `TraceBundle` per task and a summary; the same
//! episode exports to the same bytes; a second run has the same shape; a hold-out path, a test
//! task set and a test scope are refused before anything starts.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use clap::Parser;
use ignis_core::contracts::*;
use ignis_research::metrics::{measure, PriceList};
use ignis_research::trace::{build_trace, replay_events};
use ignis_server::harness_run::{check_scope, run_tasks, HarnessCmd, RunSummary};
use ignis_server::{fixture, Services};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    cmd: HarnessCmd,
}

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn services() -> Services {
    Services {
        experts: Vec::new(),
        fixture_experts: fixture::experts(),
        acquisition: None,
        research: Default::default(),
    }
}

fn parse(extra: &[&str]) -> ignis_server::harness_run::HarnessRunArgs {
    let mut argv = vec!["ignis", "run"];
    argv.extend_from_slice(extra);
    let HarnessCmd::Run(a) = Cli::try_parse_from(argv).expect("parses").cmd;
    a
}

fn run(out: &Path, variant: &str, tasks: &Path) -> RunSummary {
    let fake = env!("CARGO_BIN_EXE_ignis-server-fake-codex");
    let a = parse(&[
        "--variant",
        variant,
        "--tasks",
        tasks.to_str().unwrap(),
        "--out",
        out.to_str().unwrap(),
        "--codex-bin",
        fake,
        "--execute",
    ]);
    run_tasks(&a, services()).unwrap().expect("executed")
}

fn trace_of(out: &Path, s: &RunSummary, task: &str) -> TraceBundle {
    let e = s.runs.iter().find(|r| r.task_id == task).unwrap();
    let text = std::fs::read_to_string(out.join(e.trace_file.as_ref().unwrap())).unwrap();
    serde_json::from_str(&text).unwrap()
}

/// What two runs of the same tasks must share (ids and timestamps differ per run).
fn shape(t: &TraceBundle) -> (Vec<String>, Vec<TokenUsage>, bool, Vec<Partition>) {
    (
        t.tool_calls.iter().map(|c| c.tool.clone()).collect(),
        t.usage.clone(),
        t.answer.is_some(),
        t.partitions_touched.clone(),
    )
}

#[test]
fn a_harness_run_writes_replayable_traces_and_a_summary() {
    let dir = tempfile::tempdir().unwrap();
    let tasks = repo().join("fixtures/harness/dev_tasks.json");
    let variant = repo().join("harness");
    let out = dir.path().join("run-1");
    let s = run(&out, variant.to_str().unwrap(), &tasks);

    assert_eq!(s.task_set, "synthetic-dev-pilot");
    assert_eq!(s.task_origin, Origin::SyntheticFixture);
    assert_eq!(s.runs.len(), 3);
    let h = s
        .harness
        .clone()
        .expect("the episodes are pinned to the variant");
    assert_eq!(h.label, "harness");
    assert_eq!(s.aggregate.episodes, 3);
    assert_eq!(s.aggregate.questions, 3);
    let on_disk: RunSummary =
        serde_json::from_slice(&std::fs::read(out.join("summary.json")).unwrap()).unwrap();
    assert_eq!(on_disk, s);

    for r in &s.runs {
        assert_eq!(r.outcome, "completed", "{r:?}");
        let t = trace_of(&out, &s, &r.task_id);
        t.validate().unwrap();
        assert_eq!(t.trace_id.as_str(), r.trace_id.as_deref().unwrap());
        assert_eq!(t.harness, h);
        let run = t.run.as_ref().unwrap();
        assert_eq!(
            (run.task_id.as_str(), run.repetition),
            (r.task_id.as_str(), 0)
        );
        assert_eq!(t.partitions_touched, vec![Partition::Train]);
        assert!(t.tool_calls.iter().all(|c| c.result.is_some()));
    }
    // The search task: one traced call with its arguments and result, and measured usage
    // (the fake Codex reports thread totals; its model "fake" is not priced: no cost, not zero).
    let search = trace_of(&out, &s, "dev-search-usage");
    assert_eq!(search.tool_calls.len(), 1);
    assert_eq!(search.tool_calls[0].tool, "sources_search");
    assert!(search.tool_calls[0].arguments.is_some());
    assert_eq!(search.usage.len(), 1);
    assert_eq!(search.usage[0].model, "fake");
    assert_eq!(search.usage[0].input_tokens, 1000);
    assert_eq!(search.usage[0].cached_input_tokens, Some(400));
    assert!(search.cost.is_none());
    // The explain task traces a partial-preview argument by its full artifact.
    let explain = trace_of(&out, &s, "dev-explain");
    assert_eq!(explain.tool_calls[0].tool, "analysis_execute");

    // Replay: the event log reads back, checks out, and reproduces the episode's numbers.
    let state = out.join("state");
    let store = ignis_research::ResearchStore::open(state.join("state.sqlite")).unwrap();
    let artifacts = ignis_research::LocalArtifactStore::open(state.join("artifacts")).unwrap();
    let scope = ignis_server::v1::io::research_scope("local", false);
    let events = replay_events(&artifacts, &scope, &search).unwrap();
    assert_eq!(events.len() as u64, search.events.last_seq);
    let ep = store.episode(&scope, &search.episode_id).unwrap();
    let prices = PriceList::builtin();
    let replayed = measure(&ep, &events, &prices);
    assert_eq!(replayed.token_usage(), search.usage);
    // Into an empty store, the replayed log is the same log.
    let fresh = tempfile::tempdir().unwrap();
    let other = ignis_research::ResearchStore::open(fresh.path().join("s.sqlite")).unwrap();
    assert!(other
        .import_episode(&scope, ep.clone(), events.clone())
        .unwrap());
    assert_eq!(
        ignis_research::metrics::read_log(&other, &scope, &search.episode_id).unwrap(),
        events
    );

    // Content-addressed: exporting the same episode again gives the same bundle, byte for byte;
    // a tampered bundle no longer replays.
    let again = build_trace(
        &store,
        &artifacts,
        &scope,
        &search.episode_id,
        search.run.clone(),
        &prices,
    )
    .unwrap();
    assert_eq!(
        serde_json::to_vec(&again).unwrap(),
        serde_json::to_vec(&search).unwrap()
    );
    let mut tampered = search.clone();
    tampered.wall_time_s = Finite::new(tampered.wall_time_s.get() + 1.0).unwrap();
    assert!(replay_events(&artifacts, &scope, &tampered).is_err());

    // Deterministic shape: a second run (variant named by the digest it pinned) traces the same
    // calls, usage, answers and partitions.
    let out2 = dir.path().join("run-2");
    let s2 = run(&out2, h.digest.as_str(), &tasks);
    let _ = s2; // the built-in bundle may differ from the directory's: checked below
    for task in ["dev-search-usage", "dev-explain", "dev-plain"] {
        let a = trace_of(&out, &s, task);
        let b = trace_of(&out2, &s2, task);
        assert_eq!(shape(&a), shape(&b), "{task}");
    }

    // Re-running into the same output resumes: the same episodes, no new turns.
    let s3 = run(&out, variant.to_str().unwrap(), &tasks);
    let ids = |s: &RunSummary| {
        s.runs
            .iter()
            .map(|r| r.episode_id.clone())
            .collect::<BTreeSet<_>>()
    };
    assert_eq!(ids(&s3), ids(&s));
    assert_eq!(s3.aggregate.tokens, s.aggregate.tokens);
}

#[test]
fn holdout_paths_test_task_sets_and_test_scopes_are_refused() {
    let dir = tempfile::tempdir().unwrap();
    let tasks = repo().join("fixtures/harness/dev_tasks.json");
    let variant = repo().join("harness");
    let refused = |tasks: &Path, out: &Path, variant: &str| {
        let a = parse(&[
            "--variant",
            variant,
            "--tasks",
            tasks.to_str().unwrap(),
            "--out",
            out.to_str().unwrap(),
            "--execute",
        ]);
        run_tasks(&a, services()).unwrap_err().to_string()
    };
    let v = variant.to_str().unwrap();

    // A task file under holdout/ (however spelled), even a copy of the dev set.
    let hold = dir.path().join("data").join("HoldOut");
    std::fs::create_dir_all(&hold).unwrap();
    std::fs::copy(&tasks, hold.join("tasks.json")).unwrap();
    let e = refused(&hold.join("tasks.json"), &dir.path().join("o1"), v);
    assert!(e.contains("holdout"), "{e}");
    let sneaky = dir
        .path()
        .join("data")
        .join("x")
        .join("..")
        .join("HoldOut")
        .join("tasks.json");
    let e = refused(&sneaky, &dir.path().join("o2"), v);
    assert!(e.contains("holdout"), "{e}");
    // Output or variant under holdout/.
    let e = refused(&tasks, &hold.join("out"), v);
    assert!(e.contains("holdout"), "{e}");

    // A task set declaring the test partition.
    let mut set: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&tasks).unwrap()).unwrap();
    set["partition"] = "test".into();
    let test_set = dir.path().join("test_tasks.json");
    std::fs::write(&test_set, set.to_string()).unwrap();
    let e = refused(&test_set, &dir.path().join("o3"), v);
    assert!(e.contains("development tasks only"), "{e}");

    // The T10 loader is a seam that says why it is not there yet.
    let a = parse(&[
        "--variant",
        v,
        "--tasks",
        tasks.to_str().unwrap(),
        "--tasks-format",
        "gold",
        "--out",
        dir.path().join("o4").to_str().unwrap(),
    ]);
    let e = run_tasks(&a, services()).unwrap_err().to_string();
    assert!(e.contains("GoldTask"), "{e}");

    // Without --execute nothing starts and nothing is written.
    let out = dir.path().join("dry");
    let a = parse(&[
        "--variant",
        v,
        "--tasks",
        tasks.to_str().unwrap(),
        "--out",
        out.to_str().unwrap(),
    ]);
    assert!(run_tasks(&a, services()).unwrap().is_none());
    assert!(!out.exists());

    // A scope granting test, or an evaluation scope, is refused.
    let mut scope = ignis_server::v1::io::research_scope("local", false);
    check_scope(&scope).unwrap();
    scope.partitions.insert(Partition::Test);
    assert!(check_scope(&scope).is_err());
    let mut scope = ignis_server::v1::io::research_scope("local", false);
    scope.role = ScopeRole::Evaluation;
    assert!(check_scope(&scope).is_err());
}
