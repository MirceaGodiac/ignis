//! The research wiki (R4) through the real `EvidenceService`: itemized deltas against a base
//! revision (compare-and-swap), no whole-page rewrite, history kept by content digest, per-item
//! provenance, and the lockbox: an item citing a held-out passage never reaches a research scope.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};

use ignis_core::contracts::{
    Author, AuthorKind, Capability, EntityRef, EpisodeId, ErrorCode, EvidenceId, EvidenceSchema,
    ExecutionScope, Family, GraphNode, OffsetUnit, Origin, Partition, PrincipalId, ScopeId,
    ScopeRole, SourceLocator, Split, WikiItem, WikiItemId, WikiItemKind, WikiOp, WikiPageId,
    WikiPatch, WikiReadBody, WikiReadRequest, WikiReadResult, WikiRevision, WorkspaceId,
};
use ignis_core::services::KnowledgeService;
use ignis_evidence::service::{EvidenceService, Lockbox};
use ignis_evidence::wiki::WikiStore;
use ignis_evidence::{catalog, dense, records};
use serde_json::json;

const SHA: &str = "abababababababababababababababababababababababababababababababab";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

fn tmp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("ignis-wiki-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn scope(ws: &str, role: ScopeRole, partitions: &[Partition]) -> ExecutionScope {
    ExecutionScope {
        scope_id: ScopeId::new(format!("scope-{ws}-{role:?}")).unwrap(),
        workspace_id: WorkspaceId::new(ws).unwrap(),
        principal: PrincipalId::new("tester").unwrap(),
        role,
        partitions: partitions.iter().copied().collect(),
        capabilities: BTreeSet::from([Capability::ReadSources, Capability::WikiWrite]),
        corpus_snapshot: None,
        parent_scope_id: None,
    }
}

fn research() -> ExecutionScope {
    scope(
        "ws",
        ScopeRole::Research,
        &[Partition::Train, Partition::Validation],
    )
}

fn evaluation() -> ExecutionScope {
    scope(
        "ws",
        ScopeRole::Evaluation,
        &[Partition::Train, Partition::Validation, Partition::Test],
    )
}

fn agent() -> Author {
    Author {
        kind: AuthorKind::Agent,
        id: "tester".into(),
    }
}

/// A catalog with two outcome passages: one on train run FLEX-001, one on test run FLEX-002.
fn catalog(dir: &Path) -> (PathBuf, String, String) {
    let db = dir.join("evidence.sqlite");
    let mut con = catalog::connect(&db, false).unwrap();
    let publ = records::Publication {
        id: format!("pub-{}", &SHA[..16]),
        source_sha256: SHA.into(),
        title: Some("Synthetic FLEX report".into()),
        authors: vec![],
        year: Some(2015),
        ntrs_id: None,
        doi: None,
        canonical_url: None,
        investigation_ids: vec!["PSI-69".into()],
        families: vec!["flex".into()],
        file_name: Some("synthetic.pdf".into()),
        n_pages: 1,
        rights_status: "verified".into(),
        license: None,
        describes_runs: vec![],
        outcome_bearing: true,
        ingested_at: "2026-11-15T00:00:00Z".into(),
        parser_version: "test".into(),
    };
    let view = records::parse_view_id(&publ.id, &publ.parser_version);
    let texts = [
        ("Test FLEX-001 extinction at 2.1 mm.", "alias:flex:FLEX-001"),
        ("Test FLEX-002 extinction at 1.7 mm.", "alias:flex:FLEX-002"),
    ];
    let mut page = String::new();
    let mut chunks = Vec::new();
    for (i, (text, run)) in texts.iter().enumerate() {
        let a = page.chars().count() as i64;
        page.push_str(text);
        let b = page.chars().count() as i64;
        page.push('\n');
        chunks.push(records::Chunk {
            id: format!("{view}:p1:{i:05}"),
            publication_id: publ.id.clone(),
            ordinal: i as i64,
            kind: "text".into(),
            page: 1,
            char_start: Some(a),
            char_end: Some(b),
            section: None,
            text: (*text).into(),
            context: json!({}),
            locator: SourceLocator {
                asset_sha256: SHA.into(),
                path: Some("synthetic.pdf".into()),
                page: Some(1),
                char_start: Some(a),
                char_end: Some(b),
                parse_view_id: Some(view.clone()),
                offset_unit: Some(OffsetUnit::UnicodeScalar),
                bbox: Some([0.1, 0.1, 0.9, 0.2]),
                ..Default::default()
            },
            describes_runs: vec![(*run).into()],
            continues_runs: vec![],
            outcome_bearing: true,
            evidence_class: "source_derived".into(),
        });
    }
    let (train, test) = (chunks[0].id.clone(), chunks[1].id.clone());
    catalog::upsert_parse_view(&mut con, &publ, &chunks, &[page], None).unwrap();
    (db, train, test)
}

/// FLEX-001 is a train run, FLEX-002 a test run.
fn lockbox() -> Lockbox {
    let split = Split {
        id: "split".into(),
        cohort_id: "cohort".into(),
        group_key: "session".into(),
        rule: "fixture".into(),
        seed: 0,
        run_to_group: BTreeMap::from([
            ("PSI-69:R1".to_string(), "g1".to_string()),
            ("PSI-69:R2".to_string(), "g2".to_string()),
        ]),
        train: vec!["g1".into()],
        validation: vec![],
        test: vec!["g2".into()],
        frozen_at: "2026-11-14T00:00:00Z".into(),
    };
    let aliases = HashMap::from([
        ("alias:flex:FLEX-001".to_string(), "PSI-69:R1".to_string()),
        ("alias:flex:FLEX-002".to_string(), "PSI-69:R2".to_string()),
    ]);
    Lockbox::new(&[(split, Family::Flex)], aliases)
}

struct World {
    dir: PathBuf,
    svc: EvidenceService,
    train_ev: String,
    test_ev: String,
}

impl Drop for World {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

fn world(name: &str) -> World {
    let dir = tmp(name);
    let (db, train_ev, test_ev) = catalog(&dir);
    let cfg = dense::RetrievalCfg::load(&root().join("configs/evidence/retrieval.toml")).unwrap();
    let wiki =
        WikiStore::open(&dir.join("runs/wiki/wiki.sqlite"), Origin::SyntheticFixture).unwrap();
    let mut svc = EvidenceService::open(&db, cfg, lockbox())
        .unwrap()
        .offline()
        .with_wiki(wiki);
    svc.ledger = false;
    World {
        dir,
        svc,
        train_ev,
        test_ev,
    }
}

fn item(id: &str, text: &str, cites: &[&str]) -> WikiItem {
    WikiItem {
        item_id: WikiItemId::new(id).unwrap(),
        kind: WikiItemKind::Method,
        text: text.into(),
        refs: cites
            .iter()
            .map(|e| GraphNode::Entity(EntityRef::Evidence(EvidenceId::new(*e).unwrap())))
            .collect(),
    }
}

fn add(id: &str, text: &str, cites: &[&str]) -> WikiOp {
    WikiOp::Add {
        item: item(id, text, cites),
    }
}

fn revise(id: &str, text: &str) -> WikiOp {
    WikiOp::Revise {
        item: item(id, text, &[]),
    }
}

fn remove(id: &str) -> WikiOp {
    WikiOp::Remove {
        item_id: WikiItemId::new(id).unwrap(),
        reason: "superseded".into(),
    }
}

fn patch(page: &str, base: Option<u64>, title: Option<&str>, ops: Vec<WikiOp>) -> WikiPatch {
    WikiPatch {
        schema_version: EvidenceSchema::V2,
        page_id: WikiPageId::new(page).unwrap(),
        base_revision: base,
        title: title.map(Into::into),
        ops,
    }
}

fn read(svc: &EvidenceService, s: &ExecutionScope, page: &str, rev: Option<u64>) -> WikiReadResult {
    svc.wiki_read(
        s,
        &WikiReadRequest {
            schema_version: EvidenceSchema::V2,
            page_id: Some(WikiPageId::new(page).unwrap()),
            revision: rev,
            page_size: None,
            cursor: None,
        },
    )
    .unwrap()
}

fn page_of(r: &WikiReadResult) -> &WikiRevision {
    match &r.body {
        WikiReadBody::Page { revision } => revision,
        WikiReadBody::Index { .. } => panic!("an index, not a page"),
    }
}

fn ids(r: &WikiRevision) -> Vec<&str> {
    r.items.iter().map(|e| e.item_id.as_str()).collect()
}

#[test]
fn a_patch_reads_back_with_its_provenance() {
    let w = world("roundtrip");
    let s = research();
    let ep = EpisodeId::new("ep-1").unwrap();
    let r1 = w
        .svc
        .wiki_patch(
            &s,
            &agent(),
            Some(&ep),
            &patch(
                "time-zero",
                None,
                Some("Time zero of a FLEX run"),
                vec![
                    add("t0", "Take t = 0 at igniter retraction.", &[&w.train_ev]),
                    add("q1", "Does the trend hold at 1 atm?", &[]),
                ],
            ),
        )
        .unwrap();
    assert_eq!(r1.revision, 1);
    let got = read(&w.svc, &s, "time-zero", None);
    got.validate().unwrap();
    assert_eq!(got.withheld_items, None);
    let rev = page_of(&got);
    assert_eq!(rev, &r1, "what was written is what is read");
    assert_eq!(ids(rev), ["t0", "q1"]);
    assert_eq!(rev.episode_id.as_ref(), Some(&ep));
    assert_eq!(rev.scope_id.as_ref(), Some(&s.scope_id));
    assert_eq!(rev.author, agent());
    for e in &rev.items {
        assert_eq!(e.written_in, Some(1));
        assert_eq!(
            e.requires_partitions, s.partitions,
            "access is the writer's"
        );
    }
    assert_eq!(
        rev.items[0].refs,
        vec![GraphNode::Entity(EntityRef::Evidence(
            EvidenceId::new(&w.train_ev).unwrap()
        ))]
    );

    // A second delta revises one item and adds one: the untouched item keeps its provenance.
    let ep2 = EpisodeId::new("ep-2").unwrap();
    let r2 = w
        .svc
        .wiki_patch(
            &s,
            &agent(),
            Some(&ep2),
            &patch(
                "time-zero",
                Some(1),
                None,
                vec![
                    revise("q1", "Does the trend hold at 0.5 atm?"),
                    add("l1", "Check the time origin first.", &[]),
                ],
            ),
        )
        .unwrap();
    assert_eq!((r2.revision, r2.parent_revision), (2, Some(1)));
    assert_eq!(r2.title, "Time zero of a FLEX run");
    assert_eq!(ids(&r2), ["t0", "q1", "l1"]);
    let written: Vec<_> = r2.items.iter().map(|e| e.written_in).collect();
    assert_eq!(written, [Some(1), Some(2), Some(2)]);

    // The index lists the page at its head; another workspace sees nothing.
    let idx = w
        .svc
        .wiki_read(
            &s,
            &WikiReadRequest {
                schema_version: EvidenceSchema::V2,
                page_id: None,
                revision: None,
                page_size: None,
                cursor: None,
            },
        )
        .unwrap();
    let WikiReadBody::Index { pages, next_cursor } = idx.body else {
        panic!("index")
    };
    assert_eq!(pages.len(), 1);
    assert_eq!((pages[0].head_revision, pages[0].item_count), (2, 3));
    assert!(next_cursor.is_none());
    let other = scope("other", ScopeRole::Research, &[Partition::Train]);
    let e = w
        .svc
        .wiki_read(
            &other,
            &WikiReadRequest {
                schema_version: EvidenceSchema::V2,
                page_id: Some(WikiPageId::new("time-zero").unwrap()),
                revision: None,
                page_size: None,
                cursor: None,
            },
        )
        .unwrap_err();
    assert_eq!(e.code, ErrorCode::NotFound);
}

#[test]
fn a_stale_base_is_refused_never_merged() {
    let w = world("stale");
    let s = research();
    w.svc
        .wiki_patch(
            &s,
            &agent(),
            None,
            &patch("p", None, Some("P"), vec![add("a", "first", &[])]),
        )
        .unwrap();
    // Two writers read revision 1; the first wins the swap.
    w.svc
        .wiki_patch(
            &s,
            &agent(),
            None,
            &patch("p", Some(1), None, vec![add("b", "mine", &[])]),
        )
        .unwrap();
    let e = w
        .svc
        .wiki_patch(
            &s,
            &agent(),
            None,
            &patch("p", Some(1), None, vec![add("c", "theirs", &[])]),
        )
        .unwrap_err();
    assert_eq!(e.code, ErrorCode::Conflict, "{e:?}");
    // Creating an existing page is also a stale base.
    let e = w
        .svc
        .wiki_patch(
            &s,
            &agent(),
            None,
            &patch("p", None, Some("P"), vec![add("d", "x", &[])]),
        )
        .unwrap_err();
    assert_eq!(e.code, ErrorCode::Conflict);
    let head = read(&w.svc, &s, "p", None);
    assert_eq!(page_of(&head).revision, 2);
    assert_eq!(
        ids(page_of(&head)),
        ["a", "b"],
        "the refused delta left no trace"
    );
}

#[test]
fn a_whole_page_rewrite_is_refused() {
    let w = world("rewrite");
    let s = research();
    w.svc
        .wiki_patch(
            &s,
            &agent(),
            None,
            &patch(
                "p",
                None,
                Some("P"),
                vec![add("a", "A", &[]), add("b", "B", &[])],
            ),
        )
        .unwrap();
    // Replacing every item, by revision or by removal plus new items, is a rewrite.
    for ops in [
        vec![revise("a", "A2"), revise("b", "B2")],
        vec![remove("a"), remove("b"), add("c", "everything anew", &[])],
    ] {
        let e = w
            .svc
            .wiki_patch(&s, &agent(), None, &patch("p", Some(1), None, ops))
            .unwrap_err();
        assert_eq!(e.code, ErrorCode::InvalidRequest, "{e:?}");
        assert!(e.message.contains("whole-page rewrite"), "{}", e.message);
    }
    // Every op names one item that must (or must not) exist: no op silently replaces another.
    for ops in [
        vec![add("a", "dup", &[])],
        vec![revise("zz", "nothing there")],
        vec![remove("zz")],
    ] {
        let e = w
            .svc
            .wiki_patch(&s, &agent(), None, &patch("p", Some(1), None, ops))
            .unwrap_err();
        assert_eq!(e.code, ErrorCode::InvalidRequest, "{e:?}");
    }
    // By construction: a patch has no page-body field and no "set page" op.
    for bad in [
        json!({"schema_version": "evidence.v2", "page_id": "p", "base_revision": 1,
               "items": [{"item_id": "a", "kind": "note", "text": "x"}]}),
        json!({"schema_version": "evidence.v2", "page_id": "p", "base_revision": 1,
               "ops": [{"op": "set_page", "items": []}]}),
    ] {
        assert!(serde_json::from_value::<WikiPatch>(bad).is_err());
    }
    // One item at a time still works.
    let r = w
        .svc
        .wiki_patch(
            &s,
            &agent(),
            None,
            &patch("p", Some(1), None, vec![revise("a", "A2")]),
        )
        .unwrap();
    assert_eq!(ids(&r), ["a", "b"]);
}

#[test]
fn history_is_kept_by_content_digest() {
    let w = world("history");
    let s = research();
    let mut written = Vec::new();
    written.push(
        w.svc
            .wiki_patch(
                &s,
                &agent(),
                None,
                &patch(
                    "p",
                    None,
                    Some("P"),
                    vec![add("a", "A", &[]), add("b", "B", &[])],
                ),
            )
            .unwrap(),
    );
    written.push(
        w.svc
            .wiki_patch(
                &s,
                &agent(),
                None,
                &patch("p", Some(1), None, vec![revise("a", "A2")]),
            )
            .unwrap(),
    );
    written.push(
        w.svc
            .wiki_patch(
                &s,
                &agent(),
                None,
                &patch("p", Some(2), Some("P renamed"), vec![remove("b")]),
            )
            .unwrap(),
    );
    let digests: BTreeSet<_> = written.iter().map(|r| r.content_digest.clone()).collect();
    assert_eq!(digests.len(), 3, "each revision has its own address");
    for r in &written {
        let back = read(&w.svc, &s, "p", Some(r.revision));
        let got = page_of(&back);
        assert_eq!(got, r, "revision {} unchanged by later writes", r.revision);
        assert_eq!(
            got.content_digest,
            WikiRevision::digest_of(&got.title, &got.items)
        );
    }
    assert_eq!(page_of(&read(&w.svc, &s, "p", Some(1))).items[0].text, "A");

    // A stored blob that no longer hashes to its address is an error, never served as content.
    let con = rusqlite::Connection::open(w.dir.join("runs/wiki/wiki.sqlite")).unwrap();
    let n = con
        .execute(
            "UPDATE wiki_blobs SET body = replace(body, '\"A\"', '\"forged\"') WHERE digest = ?1",
            [written[0].content_digest.to_string()],
        )
        .unwrap();
    assert_eq!(n, 1);
    drop(con);
    let e = w
        .svc
        .wiki_read(
            &s,
            &WikiReadRequest {
                schema_version: EvidenceSchema::V2,
                page_id: Some(WikiPageId::new("p").unwrap()),
                revision: Some(1),
                page_size: None,
                cursor: None,
            },
        )
        .unwrap_err();
    assert_eq!(e.code, ErrorCode::Internal);
    assert!(
        e.message.contains("does not hash to its address"),
        "{}",
        e.message
    );
}

#[test]
fn an_item_citing_held_out_evidence_never_reaches_a_research_scope() {
    let w = world("lockbox");
    let res = research();
    let eval = evaluation();
    // A research scope cannot cite the test passage (it could not have read it).
    let e = w
        .svc
        .wiki_patch(
            &res,
            &agent(),
            None,
            &patch(
                "p",
                None,
                Some("Extinction"),
                vec![add("x", "1.7 mm", &[&w.test_ev])],
            ),
        )
        .unwrap_err();
    assert_eq!(e.code, ErrorCode::ScopeDenied, "{e:?}");
    w.svc
        .wiki_patch(
            &res,
            &agent(),
            None,
            &patch(
                "p",
                None,
                Some("Extinction"),
                vec![add("train", "2.1 mm on FLEX-001", &[&w.train_ev])],
            ),
        )
        .unwrap();
    // The evaluator may cite it; its item then requires the test partition.
    let r2 = w
        .svc
        .wiki_patch(
            &eval,
            &agent(),
            None,
            &patch(
                "p",
                Some(1),
                None,
                vec![add(
                    "held",
                    "FLEX-002 extinguished at 1.7 mm",
                    &[&w.test_ev],
                )],
            ),
        )
        .unwrap();
    assert!(r2.items[1].requires_partitions.contains(&Partition::Test));

    let seen = read(&w.svc, &res, "p", None);
    seen.validate().unwrap();
    assert_eq!(seen.withheld_items, Some(1), "withheld and counted");
    let text = serde_json::to_string(&seen).unwrap();
    assert!(
        !text.contains(&w.test_ev),
        "the held-out id reached research: {text}"
    );
    assert!(
        !text.contains("1.7 mm"),
        "the held-out item text reached research"
    );
    assert_eq!(ids(page_of(&seen)), ["train"]);
    assert!(
        page_of(&seen).ops.is_empty(),
        "the evaluator's delta is its text too"
    );
    // Nor can research change what it cannot see.
    let e = w
        .svc
        .wiki_patch(
            &res,
            &agent(),
            None,
            &patch("p", Some(2), None, vec![remove("held")]),
        )
        .unwrap_err();
    assert_eq!(e.code, ErrorCode::ScopeDenied);
    // The evaluator sees the whole revision under its stored digest.
    let full = read(&w.svc, &eval, "p", None);
    assert_eq!(full.withheld_items, None);
    assert_eq!(page_of(&full), &r2);
}

#[test]
fn the_index_pages_by_page_id() {
    let w = world("index");
    let s = research();
    for p in ["c", "a", "b"] {
        w.svc
            .wiki_patch(
                &s,
                &agent(),
                None,
                &patch(p, None, Some(p), vec![add("i", "x", &[])]),
            )
            .unwrap();
    }
    let mut seen = Vec::new();
    let mut cursor = None;
    loop {
        let r = w
            .svc
            .wiki_read(
                &s,
                &WikiReadRequest {
                    schema_version: EvidenceSchema::V2,
                    page_id: None,
                    revision: None,
                    page_size: Some(2),
                    cursor: cursor.take(),
                },
            )
            .unwrap();
        let WikiReadBody::Index { pages, next_cursor } = r.body else {
            panic!("index")
        };
        seen.extend(pages.into_iter().map(|p| p.page_id.as_str().to_string()));
        match next_cursor {
            Some(c) => cursor = Some(c),
            None => break,
        }
    }
    assert_eq!(seen, ["a", "b", "c"]);
}
