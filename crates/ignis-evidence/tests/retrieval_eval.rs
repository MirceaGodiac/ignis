//! `ignis evidence retrieval-eval` (R12b) on the synthetic dev fixture over the PSI-69 FLEX Test
//! Plan (`tests/fixtures/retrieval_eval`, `origin = synthetic_fixture`; never a result):
//! - known rankings give known nDCG@10 / recall@10, and the relevance rule matches span, page, row
//!   and path locators;
//! - the sweep runs end to end (all four arms, per question type, every (τ, w)), writes the report
//!   and a proposal that is never applied, and leaves the input catalog unchanged;
//! - hold-out paths, task files with hold-out tasks, hold-out gold sets and scopes that may read the
//!   test partition are refused.

use std::path::{Path, PathBuf};

use ignis_core::contracts::{
    Capability, ExecutionScope, OffsetUnit, Partition, PrincipalId, ScopeId, ScopeRole,
    SourceLocator, WorkspaceId,
};
use ignis_evidence::attribution::RunPatterns;
use ignis_evidence::docling_view::{import, DoclingView};
use ignis_evidence::retrieval_eval::{self as re, Arm, SweepArgs};
use ignis_evidence::service::{EvidenceService, Lockbox};
use ignis_evidence::{catalog, dense};
use serde_json::{json, Value};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

fn fx(rel: &str) -> PathBuf {
    root()
        .join("crates/ignis-evidence/tests/fixtures")
        .join(rel)
}

fn tmp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("ignis-reval-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn scope(role: ScopeRole, partitions: &[Partition]) -> ExecutionScope {
    ExecutionScope {
        scope_id: ScopeId::new("retrieval-eval-test").unwrap(),
        workspace_id: WorkspaceId::new("ws").unwrap(),
        principal: PrincipalId::new("tester").unwrap(),
        role,
        partitions: partitions.iter().copied().collect(),
        capabilities: [Capability::ReadSources].into_iter().collect(),
        corpus_snapshot: None,
        parent_scope_id: None,
    }
}

fn research() -> ExecutionScope {
    scope(
        ScopeRole::Research,
        &[Partition::Train, Partition::Validation],
    )
}

const PDF: &str = "PSI-69/PSI-69_Science Documents_FLEX_Test_Plan.pdf";
const VIEW: &str = "PSI-69/PSI-69_Science Documents_FLEX_Test_Plan.docling.json";

/// The FLEX Test Plan's Docling view as a one-publication catalog.
fn test_plan_catalog(dir: &Path) -> PathBuf {
    let view: DoclingView =
        serde_json::from_str(&std::fs::read_to_string(fx(VIEW)).unwrap()).unwrap();
    let rp = RunPatterns::load(&root().join("configs/evidence/run_patterns.toml")).unwrap();
    let bytes = std::fs::read(fx(PDF)).unwrap();
    let iv = import(&view, &fx(PDF), &bytes, &rp, &[]).unwrap();
    let db = dir.join("evidence.sqlite");
    let mut con = catalog::connect(&db, false).unwrap();
    catalog::upsert_parse_view(&mut con, &iv.publication, &iv.chunks, &iv.pages, None).unwrap();
    db
}

fn no_lockbox(_: &str) -> anyhow::Result<bool> {
    Ok(false)
}

#[test]
fn known_rankings_give_known_ndcg_and_recall() {
    let eq = |a: f64, b: f64| assert!((a - b).abs() < 1e-9, "{a} != {b}");
    // two cited locators, found at ranks 2 and 4
    let (n, r) = re::ndcg_recall(&[vec![], vec![0], vec![], vec![1]], 2, 10);
    let dcg = 1.0 / 3f64.log2() + 1.0 / 5f64.log2();
    eq(n, dcg / (1.0 + 1.0 / 3f64.log2()));
    eq(r, 1.0);
    // perfect ranking; several chunks of one cited page count once
    let (n, r) = re::ndcg_recall(&[vec![0], vec![0], vec![1]], 2, 10);
    eq(n, (1.0 + 1.0 / 4f64.log2()) / (1.0 + 1.0 / 3f64.log2()));
    eq(r, 1.0);
    let (n, r) = re::ndcg_recall(&[vec![0], vec![1]], 2, 10);
    eq(n, 1.0);
    eq(r, 1.0);
    // a hit below the cut-off does not count
    let mut covers = vec![vec![]; 10];
    covers.push(vec![0]);
    assert_eq!(re::ndcg_recall(&covers, 1, 10), (0.0, 0.0));
    // one of three found, at rank 1
    let (n, r) = re::ndcg_recall(&[vec![2]], 3, 10);
    eq(n, 1.0 / (1.0 + 1.0 / 3f64.log2() + 0.5));
    eq(r, 1.0 / 3.0);
}

#[test]
fn the_relevance_rule_matches_span_page_row_and_path() {
    let a = "aa".repeat(32);
    let base = SourceLocator {
        asset_sha256: a.clone(),
        ..Default::default()
    };
    let hit = SourceLocator {
        page: Some(4),
        char_start: Some(100),
        char_end: Some(200),
        parse_view_id: Some("pv-x@docling".into()),
        offset_unit: Some(OffsetUnit::UnicodeScalar),
        ..base.clone()
    };
    let span = |s, e| SourceLocator {
        char_start: Some(s),
        char_end: Some(e),
        ..hit.clone()
    };
    assert!(re::relevant(&hit, &span(150, 400)), "overlap");
    assert!(
        !re::relevant(&hit, &span(200, 300)),
        "adjacent half-open spans do not overlap"
    );
    let other_view = SourceLocator {
        parse_view_id: Some("pv-x@mupdf".into()),
        ..span(500, 600)
    };
    assert!(
        re::relevant(&hit, &other_view),
        "another view falls back to the page"
    );
    let page = SourceLocator {
        page: Some(4),
        ..base.clone()
    };
    assert!(re::relevant(&hit, &page));
    assert!(!re::relevant(
        &hit,
        &SourceLocator {
            page: Some(5),
            ..base.clone()
        }
    ));
    let other_file = SourceLocator {
        asset_sha256: "bb".repeat(32),
        page: Some(4),
        ..Default::default()
    };
    assert!(!re::relevant(&hit, &other_file));
    // T10's experimental-table and record locators
    let row_hit = SourceLocator {
        row: Some(3),
        table_id: Some("t2".into()),
        ..base.clone()
    };
    assert!(re::relevant(
        &row_hit,
        &SourceLocator {
            row: Some(3),
            column: Some("Fuel".into()),
            ..base.clone()
        }
    ));
    assert!(!re::relevant(
        &row_hit,
        &SourceLocator {
            row: Some(3),
            table_id: Some("t1".into()),
            ..base.clone()
        }
    ));
    let path_hit = SourceLocator {
        path: Some("acronym".into()),
        ..base.clone()
    };
    assert!(re::relevant(
        &path_hit,
        &SourceLocator {
            path: Some("acronym".into()),
            ..base.clone()
        }
    ));
    // a locator naming only the file grades nothing
    assert!(!re::gradable(&base));
    assert!(!re::relevant(&hit, &base));
}

#[test]
fn the_sweep_runs_end_to_end_and_only_proposes() {
    let d = tmp("sweep");
    let db = test_plan_catalog(&d);
    let s = research();
    let out = d.join("out");
    let tasks = fx("retrieval_eval/dev_tasks.synthetic.json");
    let terms = fx("retrieval_eval/expected_terms.synthetic.json");
    let pack = fx("retrieval_eval/pack");
    let args = SweepArgs {
        tasks: &tasks,
        db: &db,
        pack: Some(&pack),
        expected_terms: Some(&terms),
        taus: Some(vec![0.05, 0.2]),
        ws: vec![0.0, 0.4, 1.0, 4.0],
        out: &out,
    };
    let r = re::sweep(&args, &s, &no_lockbox).unwrap();

    // the fixture's tasks: five retrieval questions, one outside the sub-set
    assert_eq!(r.origin, ignis_core::contracts::Origin::SyntheticFixture);
    assert_eq!(r.preparation.tasks.len(), 5);
    assert_eq!(r.preparation.excluded.len(), 1);
    assert_eq!(r.preparation.excluded[0].task_id, "synthetic-retrieval-06");
    let bm25 = &r.baseline.arms[&Arm::Bm25];
    assert_eq!(
        (
            bm25["single_hop"].n,
            bm25["multi_hop"].n,
            bm25["figure"].n,
            bm25["all"].n
        ),
        (3, 1, 1, 5)
    );
    // the query arm runs on the tasks with a sketch, with its paired BM25 baseline
    let qe = &r.baseline.arms[&Arm::QueryExpansion]["all"];
    assert_eq!(qe.n, 3);
    assert!(qe.bm25_ndcg_same_tasks.is_some());
    assert_eq!(r.grid.len(), 2 * 4);
    for c in &r.grid {
        for arm in [Arm::DocExpansion, Arm::QueryDocExpansion] {
            for g in c.arms[&arm].values() {
                for v in [g.ndcg_at_10, g.recall_at_10].into_iter().flatten() {
                    assert!((0.0..=1.0).contains(&v));
                }
            }
        }
    }
    // w = 0 is plain BM25, whatever τ
    let task = |c: &re::Cell, arm: Arm, id: &str| {
        c.per_task[&arm]
            .iter()
            .find(|t| t.task_id == id)
            .unwrap()
            .clone()
    };
    let b_cell = &r.baseline;
    for c in r.grid.iter().filter(|c| c.w == 0.0) {
        for t in &b_cell.per_task[&Arm::Bm25] {
            assert_eq!(task(c, Arm::DocExpansion, &t.task_id).top, t.top);
        }
    }
    // document expansion ("sulfur hexafluoride" on the SF6 page) never ranks the cited page lower
    // below w = 1 the generic question words outrank the expansion-only page; at w = 4 it leads
    let w1 = r
        .grid
        .iter()
        .find(|c| c.tau == Some(0.05) && c.w == 4.0)
        .unwrap();
    let id = "synthetic-retrieval-02";
    assert!(task(w1, Arm::DocExpansion, id).ndcg_at_10 >= task(b_cell, Arm::Bm25, id).ndcg_at_10);
    assert!(task(w1, Arm::DocExpansion, id).recall_at_10 > 0.0);

    // a proposal, never applied; synthetic numbers are flagged as such
    assert!(!r.proposal.applied);
    assert_eq!(r.proposal.status, "synthetic_fixture_not_a_result");
    assert!(r.proposal.proposed.is_some());
    assert!(r.proposal.objective.contains("doc_expansion"));
    for f in ["report.json", "report.md", "proposal.json"] {
        assert!(out.join(f).exists(), "{f}");
    }
    assert!(
        !out.join("work.sqlite").exists(),
        "the working copy is removed"
    );
    let md = std::fs::read_to_string(out.join("report.md")).unwrap();
    assert!(md.contains("Synthetic fixture") && md.contains("| bm25 |"));
    let p: Value =
        serde_json::from_slice(&std::fs::read(out.join("proposal.json")).unwrap()).unwrap();
    assert_eq!(p["applied"], false);

    // the input catalog is untouched (no expansion loaded into it)
    let con = catalog::connect(&db, true).unwrap();
    let n: i64 = con
        .query_row("SELECT count(*) FROM expansion_fts", [], |r| r.get(0))
        .unwrap();
    assert_eq!(n, 0);
    drop(con);

    // the scope's lockbox decides what is ranked: what it withholds is never a hit
    let sealed = |_: &str| -> anyhow::Result<bool> { Ok(true) };
    let r = re::sweep(&args, &s, &sealed).unwrap();
    assert!(r.baseline.per_task[&Arm::Bm25]
        .iter()
        .all(|t| t.top.is_empty() && t.recall_at_10 == 0.0));
    // and the service's own lockbox runs through the same filter
    std::env::set_var("IGNIS_DATA", root().join("data"));
    let svc = EvidenceService::open(
        &db,
        dense::RetrievalCfg::load(&root().join("configs/evidence/retrieval.toml")).unwrap(),
        Lockbox::load().unwrap(),
    )
    .unwrap()
    .offline();
    let withheld = |id: &str| svc.withholds_chunk(&s, id);
    let r = re::sweep(&args, &s, &withheld).unwrap();
    // every retrieval task is either scored or excluded with the locators it could not see
    let p = &r.preparation;
    assert_eq!(p.tasks.len() + p.excluded.len(), 6);
    assert_eq!(
        p.tasks.iter().map(|t| t.cited.len()).sum::<usize>() + p.cited_not_visible,
        6
    );
    assert!(
        svc.withholds_chunk(&s, "no-such-chunk").unwrap(),
        "unknown chunks are withheld"
    );
    let _ = std::fs::remove_dir_all(d);
}

fn args<'a>(tasks: &'a Path, db: &'a Path, out: &'a Path) -> SweepArgs<'a> {
    SweepArgs {
        tasks,
        db,
        pack: None,
        expected_terms: None,
        taus: None,
        ws: vec![0.4],
        out,
    }
}

fn write(dir: &Path, name: &str, v: &Value) -> PathBuf {
    let p = dir.join(name);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(&p, serde_json::to_vec(v).unwrap()).unwrap();
    p
}

#[test]
fn hold_out_tasks_paths_and_test_scopes_are_refused() {
    let d = tmp("refuse");
    let tasks: Value = serde_json::from_slice(
        &std::fs::read(fx("retrieval_eval/dev_tasks.synthetic.json")).unwrap(),
    )
    .unwrap();
    let err = |r: anyhow::Result<Vec<re::GoldTaskView>>| r.unwrap_err().to_string();

    // the dev file loads, as an array and as a GoldSet
    assert_eq!(
        re::load_tasks(&fx("retrieval_eval/dev_tasks.synthetic.json"))
            .unwrap()
            .len(),
        6
    );
    let set = write(&d, "set.json", &json!({"split": "dev", "tasks": tasks}));
    assert_eq!(re::load_tasks(&set).unwrap().len(), 6);

    // a path under holdout/ is refused before it is read (it does not even exist here)
    let e = err(re::load_tasks(&d.join("benchmarks/t10/holdout/tasks.json")));
    assert!(e.contains("holdout/"), "{e}");
    let e = err(re::load_tasks(&d.join("HoldOut/x.json")));
    assert!(e.contains("holdout/"), "{e}");

    // one hold-out task anywhere in the file refuses the whole file
    let mut mixed = tasks.clone();
    mixed[3]["split"] = json!("holdout");
    let e = err(re::load_tasks(&write(&d, "mixed.json", &mixed)));
    assert!(
        e.contains("synthetic-retrieval-04") && e.contains("refused"),
        "{e}"
    );
    // a hold-out GoldSet is refused
    let e = err(re::load_tasks(&write(
        &d,
        "hset.json",
        &json!({"split": "holdout", "tasks": []}),
    )));
    assert!(e.contains("hold-out"), "{e}");
    // JSONL with a hold-out line
    let lines: Vec<String> = mixed
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t.to_string())
        .collect();
    let jl = d.join("mixed.jsonl");
    std::fs::write(&jl, lines.join("\n")).unwrap();
    assert!(err(re::load_tasks(&jl)).contains("outside the dev split"));

    // scopes: an evaluation scope, and any scope granting test, are refused
    assert!(re::check_scope(&research()).is_ok());
    let eval = scope(
        ScopeRole::Evaluation,
        &[Partition::Train, Partition::Validation, Partition::Test],
    );
    assert!(re::check_scope(&eval).is_err());
    assert!(re::check_scope(&scope(ScopeRole::Evaluation, &[Partition::Train])).is_err());
    let with_test = scope(ScopeRole::Research, &[Partition::Train, Partition::Test]);
    assert!(
        re::check_scope(&with_test).is_err(),
        "fail closed on an invalid scope too"
    );

    // the sweep itself refuses them before reading anything
    let db = d.join("missing.sqlite");
    let out = d.join("out");
    let dev = fx("retrieval_eval/dev_tasks.synthetic.json");
    let e = re::sweep(&args(&dev, &db, &out), &eval, &no_lockbox)
        .unwrap_err()
        .to_string();
    assert!(e.contains("test partition"), "{e}");
    let ho = d.join("holdout/out");
    let e = re::sweep(&args(&dev, &db, &ho), &research(), &no_lockbox)
        .unwrap_err()
        .to_string();
    assert!(e.contains("holdout/"), "{e}");
    assert!(
        !d.join("holdout").exists(),
        "nothing written under holdout/"
    );

    // τ without a pack, and sketches for tasks the dev file does not hold, are refused
    let real_db = test_plan_catalog(&d);
    let mut a = args(&dev, &db, &out);
    a.db = &real_db;
    a.taus = Some(vec![0.05]);
    assert!(re::sweep(&a, &research(), &no_lockbox)
        .unwrap_err()
        .to_string()
        .contains("--pack"));
    let bad_terms = write(
        &d,
        "terms.json",
        &json!({"not-a-dev-task": [{"term": "LOI"}]}),
    );
    let mut a = args(&dev, &db, &out);
    a.db = &real_db;
    a.expected_terms = Some(&bad_terms);
    assert!(re::sweep(&a, &research(), &no_lockbox)
        .unwrap_err()
        .to_string()
        .contains("not a task of the dev file"));
    let _ = std::fs::remove_dir_all(d);
}
