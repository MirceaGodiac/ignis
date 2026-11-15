//! `wiki_read` / `wiki_patch` through the tool dispatcher (R4): registered only with a wiki store,
//! author and episode from the host (never the arguments), compare-and-swap surfaced as a refusal,
//! and wiki text never made citable.

mod common;

use std::sync::Arc;

use common::*;
use ignis_core::contracts::*;
use ignis_core::services::{KnowledgeService, ServiceResult};
use ignis_evidence::wiki::WikiStore;
use ignis_mcp::{ToolHost, ToolServices};
use ignis_research::{LocalArtifactStore, ResearchStore};
use serde_json::json;

/// A KnowledgeService whose only capability is the real wiki store; every cited evidence id
/// resolves except `held-out-*`.
struct WikiOnly(WikiStore);

impl KnowledgeService for WikiOnly {
    fn search(&self, _: &ExecutionScope, _: &SearchRequest) -> ServiceResult<SearchPage> {
        unreachable!()
    }
    fn read(&self, _: &ExecutionScope, _: &ReadRequest) -> ServiceResult<ReadResult> {
        unreachable!()
    }
    fn resolve_evidence(&self, _: &ExecutionScope, _: &EvidenceId) -> ServiceResult<EvidenceRef> {
        unreachable!()
    }
    fn wiki_read(&self, s: &ExecutionScope, r: &WikiReadRequest) -> ServiceResult<WikiReadResult> {
        self.0.read(s, r)
    }
    fn wiki_patch(
        &self,
        s: &ExecutionScope,
        a: &Author,
        e: Option<&EpisodeId>,
        p: &WikiPatch,
    ) -> ServiceResult<WikiRevision> {
        self.0.patch(s, a, e, p, &|id| {
            if id.as_str().starts_with("held-out-") {
                Err(ErrorEnvelope::new(
                    ErrorCode::ScopeDenied,
                    "outside the scope's partitions",
                    RetryPolicy::Never,
                ))
            } else {
                Ok(())
            }
        })
    }
}

struct Wiki {
    _dir: tempfile::TempDir,
    host: ToolHost,
    store: Arc<ResearchStore>,
}

fn wiki_host(wiki: bool) -> Wiki {
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(ResearchStore::open(dir.path().join("state.sqlite")).unwrap());
    let artifacts = Arc::new(LocalArtifactStore::open(dir.path().join("artifacts")).unwrap());
    let w = WikiStore::open(&dir.path().join("wiki.sqlite"), Origin::SyntheticFixture).unwrap();
    let host = ToolHost::new(
        ToolServices {
            knowledge: Some(Arc::new(WikiOnly(w))),
            wiki,
            ..Default::default()
        },
        store.clone(),
        artifacts,
        Origin::SyntheticFixture,
        Arc::new(|| {}),
    );
    Wiki {
        _dir: dir,
        host,
        store,
    }
}

fn status(r: &ToolResult) -> ExecutionStatus {
    r.validate().unwrap();
    r.execution_status
}

fn code(r: &ToolResult) -> ErrorCode {
    assert_eq!(status(r), ExecutionStatus::Refused, "{r:#?}");
    r.error.as_ref().unwrap().code
}

fn create_args() -> serde_json::Value {
    json!({
        "schema_version": "evidence.v2",
        "page_id": "time-zero",
        "title": "Time zero",
        "ops": [{"op": "add", "item": {
            "item_id": "t0", "kind": "method", "text": "t = 0 at igniter retraction",
            "refs": [{"kind": "evidence", "id": "fixture-evidence-01"}]}}]
    })
}

#[test]
fn the_wiki_tools_are_registered_only_with_a_wiki_store() {
    let names =
        |h: &ToolHost| -> Vec<String> { h.registered().iter().map(|d| d.name.clone()).collect() };
    let on = wiki_host(true);
    for t in ["wiki_read", "wiki_patch"] {
        assert!(
            names(&on.host).contains(&t.to_string()),
            "{t} not registered"
        );
    }
    let off = wiki_host(false);
    assert!(!names(&off.host).iter().any(|n| n.starts_with("wiki_")));
    let why: Vec<_> = ignis_mcp::catalog(&ToolServices {
        knowledge: Some(Arc::new(WikiOnly(
            WikiStore::open(&off._dir.path().join("w2.sqlite"), Origin::SyntheticFixture).unwrap(),
        ))),
        ..Default::default()
    })
    .into_iter()
    .filter(|t| t.descriptor.name.starts_with("wiki_"))
    .map(|t| t.unavailable)
    .collect();
    assert_eq!(why, vec![Some(ignis_mcp::tools::WIKI_NOT_CONFIGURED); 2]);
}

#[test]
fn a_patch_through_the_tool_reads_back_with_host_provenance() {
    let w = wiki_host(true);
    let sc = scope("ws");
    let ep = w
        .store
        .create_episode(
            &sc,
            "Synthetic fixture goal",
            &ClientRequestId::new("crid-wiki-1").unwrap(),
            EpisodePins::default(),
            Origin::SyntheticFixture,
        )
        .unwrap()
        .0
        .episode_id;
    let r = w.host.call_in(&sc, Some(&ep), "wiki_patch", create_args());
    assert_eq!(status(&r), ExecutionStatus::Completed, "{r:#?}");
    assert_eq!(r.scientific_status, ScientificStatus::NotAssessed);
    let rev: WikiRevision =
        serde_json::from_value(r.result.as_ref().unwrap().value.clone()).unwrap();
    rev.validate().unwrap();
    assert_eq!(rev.revision, 1);
    assert_eq!(rev.episode_id.as_ref(), Some(&ep), "the grant's episode");
    assert_eq!(rev.scope_id.as_ref(), Some(&sc.scope_id));
    assert_eq!(
        rev.author,
        Author {
            kind: AuthorKind::Agent,
            id: sc.principal.as_str().into()
        }
    );

    let r = w.host.call_in(
        &sc,
        Some(&ep),
        "wiki_read",
        json!({"schema_version": "evidence.v2", "page_id": "time-zero"}),
    );
    assert_eq!(status(&r), ExecutionStatus::Completed, "{r:#?}");
    let read: WikiReadResult =
        serde_json::from_value(r.result.as_ref().unwrap().value.clone()).unwrap();
    let WikiReadBody::Page { revision } = read.body else {
        panic!("page")
    };
    assert_eq!(*revision, rev);

    // The wiki is memory, not a source: reading it records nothing an answer could cite.
    let ev = EvidenceId::new("fixture-evidence-01").unwrap();
    assert!(w
        .store
        .episode_evidence(&sc, &ep, [&ev])
        .unwrap()
        .is_empty());
}

#[test]
fn the_tool_refuses_what_the_host_owns_and_a_stale_base() {
    let w = wiki_host(true);
    let sc = scope("ws");
    assert_eq!(
        status(&w.host.call(&sc, "wiki_patch", create_args())),
        ExecutionStatus::Completed
    );
    // Author, episode and access are the host's: an argument naming them is refused.
    for (k, v) in [
        ("author", json!({"kind": "human", "id": "someone-else"})),
        ("episode_id", json!("ep-forged")),
        ("requires_partitions", json!([])),
    ] {
        let mut a = create_args();
        a["base_revision"] = json!(1);
        a[k] = v;
        assert_eq!(
            code(&w.host.call(&sc, "wiki_patch", a)),
            ErrorCode::InvalidRequest,
            "{k}"
        );
    }
    // A moved head is a conflict, not a silent last-writer-wins.
    let delta = |id: &str| {
        json!({"schema_version": "evidence.v2", "page_id": "time-zero", "base_revision": 1,
               "ops": [{"op": "add", "item": {"item_id": id, "kind": "note", "text": id}}]})
    };
    assert_eq!(
        status(&w.host.call(&sc, "wiki_patch", delta("first"))),
        ExecutionStatus::Completed
    );
    assert_eq!(
        code(&w.host.call(&sc, "wiki_patch", delta("second"))),
        ErrorCode::Conflict
    );
    // Citing what the scope cannot resolve is refused.
    let mut held = create_args();
    held["page_id"] = json!("other");
    held["ops"][0]["item"]["refs"][0]["id"] = json!("held-out-01");
    assert_eq!(
        code(&w.host.call(&sc, "wiki_patch", held)),
        ErrorCode::ScopeDenied
    );
    // Writing needs `wiki_write` in the host-derived scope.
    let mut ro = sc.clone();
    ro.capabilities.remove(&Capability::WikiWrite);
    assert_eq!(
        code(&w.host.call(&ro, "wiki_patch", delta("third"))),
        ErrorCode::ScopeDenied
    );
}
