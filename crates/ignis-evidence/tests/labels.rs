//! R8 on a fixture catalog with a fake model (no model is ever called): two independent passes make
//! a `resolved` label only when they agree; a disagreement stays `unresolved_conflict`; answers are
//! parsed robustly; the gold set is stratified, reproducible and EMPTY; the eval refuses an
//! unlabelled gold set and scores a labelled one; and the combined pass splits into summaries, an
//! expansion pack that `enrich` loads, and label pass `a`.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use ignis_core::contracts::{
    Capability, DocType, ExecutionScope, Family, OffsetUnit, Partition, PrincipalId, Quantity,
    Resolution, ScopeId, ScopeRole, SourceId, SourceLocator, WorkspaceId,
};
use ignis_core::services::KnowledgeService;
use ignis_evidence::labels::{self, GoldRow};
use ignis_evidence::llm_pass::{self, Model, Task};
use ignis_evidence::service::{EvidenceService, Lockbox, LABELS_FILE};
use ignis_evidence::{catalog, combined, dense, records, sira};
use serde_json::json;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

fn sha(n: u8) -> String {
    format!("{n:02x}").repeat(32)
}

/// Publication `n` of investigation `inv`, file `file`, with `pages` of text (a heading per page).
fn add(con: &mut rusqlite::Connection, n: u8, inv: &str, file: &str, pages: &[&str]) {
    let s = sha(n);
    let publ = records::Publication {
        id: format!("pub-{}", &s[..16]),
        source_sha256: s.clone(),
        title: Some(format!("Synthetic {file}")),
        authors: vec![],
        year: None,
        ntrs_id: None,
        doi: None,
        canonical_url: None,
        investigation_ids: vec![inv.into()],
        families: vec![],
        file_name: Some(file.into()),
        n_pages: pages.len() as i64,
        rights_status: "verified".into(),
        license: None,
        describes_runs: vec![],
        outcome_bearing: false,
        ingested_at: "2026-11-15T00:00:00Z".into(),
        parser_version: "test".into(),
    };
    let view = records::parse_view_id(&publ.id, &publ.parser_version);
    let mut page_text = Vec::new();
    let mut chunks = Vec::new();
    for (i, body) in pages.iter().enumerate() {
        let page = i as i64 + 1;
        let heading = format!("{page} Section {page}");
        let text = format!("{heading}\n{body}");
        for (k, (t, kind)) in [(heading.as_str(), "heading"), (*body, "text")]
            .iter()
            .enumerate()
        {
            let a = if k == 0 {
                0
            } else {
                heading.chars().count() + 1
            };
            chunks.push(records::Chunk {
                id: format!("{view}:p{page}:{:05}", chunks.len()),
                publication_id: publ.id.clone(),
                ordinal: chunks.len() as i64,
                kind: (*kind).into(),
                page,
                char_start: Some(a as i64),
                char_end: Some((a + t.chars().count()) as i64),
                section: None,
                text: (*t).into(),
                context: json!({}),
                locator: SourceLocator {
                    asset_sha256: s.clone(),
                    page: Some(page),
                    parse_view_id: Some(view.clone()),
                    offset_unit: Some(OffsetUnit::UnicodeScalar),
                    ..Default::default()
                },
                describes_runs: vec![],
                continues_runs: vec![],
                outcome_bearing: false,
                evidence_class: "source_derived".into(),
            });
        }
        page_text.push(text);
    }
    catalog::upsert_parse_view(con, &publ, &chunks, &page_text, None).unwrap();
}

fn fixture(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("ignis-labels-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let db = dir.join("evidence.sqlite");
    let mut con = catalog::connect(&db, false).unwrap();
    add(
        &mut con,
        1,
        "PSI-68",
        "PSI-68_Reports_FLEX-2 Summary Report.pdf",
        &[
            "Droplet tests in the CIR.",
            "Results of FLEX-2 heptane tests.",
            "Conclusions.",
        ],
    );
    add(
        &mut con,
        2,
        "PSI-39",
        "PSI-39_Science Documents_CFI SRD.pdf",
        &[
            "Science requirements for CFI.",
            "Test matrix: pressure and oxygen.",
            "Ignition.",
            "Diagnostics.",
            "Operations.",
            "Appendix.",
        ],
    );
    add(
        &mut con,
        3,
        "PSI-39",
        "PSI-39_Presentation_CFI overview.pptx.pdf",
        &["Overview slides."],
    );
    add(
        &mut con,
        4,
        "PSI-107",
        "PSI-107_Reports_SPICE Final Report.pdf",
        &["Smoke point measurements.", "Laminar jet flames."],
    );
    db
}

/// Answers by document and pass: doc 2's passes disagree on the type.
struct Fake;
impl Model for Fake {
    fn id(&self) -> &str {
        "fake-labeller"
    }
    fn call(&self, system: &str, user: &str, _: u32) -> anyhow::Result<String> {
        let file = user.lines().next().unwrap_or_default();
        // Pass b never sees page 1 of a document longer than four pages.
        let pass_b = !user.contains("[page 1]\n");
        if system == combined::COMBINED_SYSTEM {
            let pages: Vec<i64> = user
                .lines()
                .filter_map(|l| l.strip_prefix("[page ")?.strip_suffix(']')?.parse().ok())
                .collect();
            let label = user.contains("LABEL: yes").then(|| {
                json!({"doc_type": "technical_report", "families": [], "investigation_ids": ["PSI-68"], "runs": ["FLEX-2-001"], "conditions_mentioned": [], "confidence": 0.8})
            });
            return Ok(format!(
                "Here is the JSON:\n{}",
                json!({"summary": "Covers droplet tests.", "pages": pages.iter().map(|p| json!({"page": p, "expansion": ["n-heptane droplet", "CIR"]})).collect::<Vec<_>>(), "label": label})
            ));
        }
        let (ty, inv) = if file.contains("FLEX-2") {
            ("technical_report", "PSI-68")
        } else if file.contains("SRD") {
            (
                if pass_b {
                    "engineering_document"
                } else {
                    "requirements_document"
                },
                "PSI-39",
            )
        } else if file.contains("Presentation") {
            ("presentation", "PSI-39")
        } else {
            ("technical_report", "psi-107")
        };
        Ok(format!(
            "```json\n{{\"doc_type\": \"{ty}\", \"families\": [], \"investigation_ids\": [\"{inv}\"], \"runs\": [\"T{{1}}\"], \"conditions_mentioned\": [\"pressure\"], \"confidence\": 0.9}}\n```"
        ))
    }
}

#[test]
fn two_agreeing_passes_resolve_a_label_and_a_disagreement_stays_visible() {
    let db = fixture("passes");
    let con = catalog::connect(&db, true).unwrap();
    let pubs = catalog::publications(&con).unwrap();
    let reqs = labels::label_requests(&con, &pubs).unwrap();
    assert_eq!(reqs.len(), 2 * pubs.len());
    assert!(reqs
        .iter()
        .all(|r| r.task == Task::SourceLabel && r.system == labels::LABEL_SYSTEM));
    // The passes see different pages of a long document.
    let (a, b) = labels::pass_pages(40);
    assert_eq!(a, vec![1, 2, 3]);
    assert_eq!(b, vec![20, 21, 39, 40]);

    let answers = db.with_file_name("label_answers.jsonl");
    llm_pass::run(&reqs, &Fake, &answers, 3).unwrap();
    let built = labels::build_labels(&llm_pass::read_answers(&answers).unwrap()).unwrap();
    assert!(built.problems.is_empty(), "{:?}", built.problems);
    assert_eq!(
        (
            built.counts.total,
            built.counts.resolved,
            built.counts.unresolved_conflict
        ),
        (4, 3, 1)
    );
    let by = |n: u8| {
        built
            .labels
            .iter()
            .find(|l| l.source_id.as_str() == format!("sha256:{}", sha(n)))
            .unwrap()
    };
    let flex2 = by(1);
    assert_eq!(flex2.status, Resolution::Resolved);
    assert_eq!(flex2.doc_type, DocType::TechnicalReport);
    assert!(flex2.agreement.agreed && flex2.agreement.passes == 2);
    assert_eq!(flex2.labeller.model_id.as_deref(), Some("fake-labeller"));
    assert_eq!(
        flex2.investigation_ids,
        BTreeSet::from(["PSI-68".to_string()])
    );
    let srd = by(2);
    assert_eq!(
        srd.status,
        Resolution::UnresolvedConflict,
        "pass b said engineering_document"
    );
    assert!(!srd.agreement.agreed);
    assert_eq!(
        by(4).investigation_ids,
        BTreeSet::from(["PSI-107".to_string()]),
        "accessions normalized"
    );

    // The service serves them per source.
    let mut text = String::new();
    for l in &built.labels {
        text.push_str(&serde_json::to_string(l).unwrap());
        text.push('\n');
    }
    std::fs::write(db.with_file_name(LABELS_FILE), text).unwrap();
    let cfg = dense::RetrievalCfg::load(&root().join("configs/evidence/retrieval.toml")).unwrap();
    let mut s = EvidenceService::open(&db, cfg, Lockbox::new(&[], Default::default()))
        .unwrap()
        .offline();
    s.ledger = false;
    let sc = ExecutionScope {
        scope_id: ScopeId::new("s").unwrap(),
        workspace_id: WorkspaceId::new("w").unwrap(),
        principal: PrincipalId::new("p").unwrap(),
        role: ScopeRole::Research,
        partitions: [Partition::Train].into(),
        capabilities: [Capability::ReadSources].into(),
        corpus_snapshot: None,
        parent_scope_id: None,
    };
    let got = s
        .labels(&sc, &SourceId::new(format!("sha256:{}", sha(2))).unwrap())
        .unwrap();
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].status, Resolution::UnresolvedConflict);
}

#[test]
fn answers_are_parsed_robustly_and_refused_when_not_a_label() {
    let d = labels::parse_label(r#"Sure. {"doc_type": "thesis", "investigation_ids": [" psi-26 "], "runs": ["a {b} c"], "confidence": 1.7} done"#).unwrap();
    assert_eq!(d.doc_type, DocType::Thesis);
    assert_eq!(d.investigation_ids, BTreeSet::from(["PSI-26".to_string()]));
    assert_eq!(d.runs, vec!["a {b} c".to_string()], "braces inside strings");
    assert_eq!(d.confidence, None, "out-of-range confidence dropped");
    assert!(labels::parse_label("no json here").is_err());
    assert!(
        labels::parse_label(r#"{"doc_type": "novel"}"#).is_err(),
        "closed doc types"
    );
    assert!(labels::parse_label(r#"{"doc_type": "other""#).is_err());
}

#[test]
fn the_gold_set_is_stratified_reproducible_and_empty() {
    let db = fixture("gold");
    let con = catalog::connect(&db, true).unwrap();
    let pubs = catalog::publications(&con).unwrap();
    let rows = labels::gold_select(&pubs, 3, 7);
    assert_eq!(rows.len(), 3);
    // Round-robin over strata: three different strata before any repeats.
    let strata: BTreeSet<(String, String)> = rows
        .iter()
        .map(|r| (r.stratum_investigation.clone(), r.stratum_kind.clone()))
        .collect();
    assert_eq!(strata.len(), 3);
    assert_eq!(
        labels::gold_select(&pubs, 3, 7),
        rows,
        "same seed, same selection"
    );
    assert_eq!(
        labels::gold_select(&pubs, 10, 7).len(),
        4,
        "never more than the corpus"
    );
    // No label is ever filled by code.
    assert!(rows.iter().all(|r| r.doc_type.is_none()
        && r.families.is_empty()
        && r.investigation_ids.is_empty()
        && r.labelled_by.is_none()));
    assert_eq!(labels::kind_hint(&pubs[2]), "presentation");

    // An unlabelled gold set measures nothing.
    let e = labels::gold_eval(&rows, &[]).unwrap_err();
    assert!(e.to_string().contains("no labelled rows"), "{e}");
}

#[test]
fn the_eval_scores_a_labeller_against_person_labelled_rows() {
    let db = fixture("eval");
    let con = catalog::connect(&db, true).unwrap();
    let pubs = catalog::publications(&con).unwrap();
    let reqs = labels::label_requests(&con, &pubs).unwrap();
    let answers = db.with_file_name("label_answers.jsonl");
    llm_pass::run(&reqs, &Fake, &answers, 3).unwrap();
    let built = labels::build_labels(&llm_pass::read_answers(&answers).unwrap()).unwrap();
    // Test-only rows standing in for a person's labels (synthetic, not corpus gold).
    let row = |n: u8, ty: DocType, inv: &str, fam: &[Family]| GoldRow {
        source_id: format!("sha256:{}", sha(n)),
        file_name: None,
        n_pages: 1,
        stratum_investigation: inv.into(),
        stratum_kind: "x".into(),
        doc_type: Some(ty),
        families: fam.iter().copied().collect(),
        investigation_ids: [inv.to_string()].into(),
        runs: vec![],
        conditions_mentioned: Default::default(),
        labelled_by: Some("test-fixture".into()),
        labelled_at: None,
        notes: String::new(),
    };
    let mut unfilled = row(3, DocType::Presentation, "PSI-39", &[]);
    unfilled.labelled_by = None;
    // Every label carries the run `T{1}` and the condition `pressure` (see `Fake`).
    let mut one = row(1, DocType::TechnicalReport, "PSI-68", &[]);
    one.runs = vec![" t{1} ".into()];
    one.conditions_mentioned = [Quantity::Pressure].into();
    let mut four = row(4, DocType::JournalArticle, "PSI-107", &[Family::Other]);
    four.conditions_mentioned = [Quantity::Pressure, Quantity::Temperature].into();
    let gold = vec![
        one,
        row(2, DocType::RequirementsDocument, "PSI-39", &[]),
        four,
        unfilled,
    ];
    let s = labels::gold_eval(&gold, &built.labels).unwrap();
    assert_eq!(
        (s.labelled, s.unlabelled, s.scored, s.missing_labels),
        (3, 1, 3, 0)
    );
    assert_eq!(
        s.doc_type_accuracy,
        Some(2.0 / 3.0),
        "doc 4 is a journal article, labelled a report"
    );
    assert_eq!(s.investigation_accuracy, Some(1.0));
    assert_eq!(s.family_f1, Some(0.0), "one gold family, none predicted");
    assert_eq!(
        s.runs_f1,
        Some(0.5),
        "doc 1's run matches after normalising; docs 2 and 4 name a run the gold lacks"
    );
    assert_eq!(
        s.conditions_f1,
        Some(2.0 / 3.0),
        "2 matched, 1 extra (doc 2), 1 missed (doc 4's temperature)"
    );
    assert_eq!(
        (s.resolved_scored, s.resolved_doc_type_accuracy),
        (2, Some(0.5))
    );
    assert_eq!(s.confusion["journal_article"]["technical_report"], 1);
}

#[test]
fn the_combined_pass_splits_into_summaries_a_pack_and_label_pass_a() {
    let db = fixture("combined");
    let con = catalog::connect(&db, true).unwrap();
    let pubs: Vec<_> = catalog::publications(&con)
        .unwrap()
        .into_iter()
        .filter(|p| p.source_sha256 == sha(1))
        .collect();
    let reqs = combined::combined_requests(&con, &pubs).unwrap();
    assert_eq!(reqs.len(), 3, "one call per leaf section");
    assert_eq!(
        reqs.iter()
            .filter(|r| r.user.contains("LABEL: yes"))
            .count(),
        1,
        "the label once per document"
    );
    let est = llm_pass::estimate(&reqs, 1.0, 5.0);
    assert!(est.prefix_tokens > 0 && est.usd > 0.0);
    drop(con);

    let answers = db.with_file_name("combined_answers.jsonl");
    llm_pass::run(&reqs, &Fake, &answers, 3).unwrap();
    let pack = db.with_file_name("expansion-llm");
    let rep = combined::split(
        &llm_pass::read_answers(&answers).unwrap(),
        &db.with_file_name("outline_summaries.jsonl"),
        &pack,
        &db.with_file_name("label_answers.jsonl"),
    )
    .unwrap();
    assert_eq!((rep.summaries, rep.pack_pages, rep.label_passes), (3, 3, 1));
    assert!(rep.unparseable.is_empty());

    // The pack loads into the expansion index (R12b), never as citable text.
    let mut con = catalog::connect(&db, false).unwrap();
    let r = sira::enrich(&mut con, &pack, 1.0).unwrap();
    let n: i64 = con
        .query_row("SELECT count(*) FROM expansion_fts", [], |r| r.get(0))
        .unwrap();
    assert!(n > 0, "{r}");
    drop(con);

    // Label pass b (the label prompt) completes the document's label; it names both prompts.
    let con = catalog::connect(&db, true).unwrap();
    let b: Vec<_> = labels::label_requests(&con, &pubs)
        .unwrap()
        .into_iter()
        .filter(|r| r.custom_id.ends_with("#b"))
        .collect();
    llm_pass::run(&b, &Fake, &db.with_file_name("label_answers.jsonl"), 3).unwrap();
    let built = labels::build_labels(
        &llm_pass::read_answers(&db.with_file_name("label_answers.jsonl")).unwrap(),
    )
    .unwrap();
    assert_eq!(built.labels.len(), 1);
    let l = &built.labels[0];
    assert_eq!(l.status, Resolution::Resolved);
    assert_ne!(
        l.labeller.prompt_hash.as_ref().unwrap(),
        &llm_pass::prompt_hash(labels::LABEL_SYSTEM)
    );
}
