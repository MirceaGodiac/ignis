//! Knowledge contracts: evidence references, search, reading, provenance links and the wiki.
//! Semantic owner: Raul (Iancu and Tudor review numeric consumption).
//!
//! A source *version* is the existing [`Source`](super::Source) record: its `id` is what
//! [`SourceId`] names, and `id_basis` says whether the id proves the bytes. Locators reuse
//! [`SourceLocator`](super::SourceLocator). A search hit is a pointer with a snippet, never the
//! document: reading goes through [`ReadRequest`].

use std::collections::BTreeSet;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::common::{
    ensure, schema_tag, ContractError, CorpusSnapshotId, EntityRef, EpisodeId, EvidenceId, Finite,
    LinkId, Origin, PageCursor, ParseViewId, SourceId, Timestamp, WikiPageId,
};
use super::{Derivation, EvidenceClass, Family, Resolution, SourceKind, SourceLocator};

schema_tag!(
    /// Wire tag of search, read, provenance and wiki documents.
    EvidenceSchema = "evidence.v1"
);

// ---------------------------------------------------------------- evidence

/// A citable pointer into one source version. Verifiable only against that version's asset and
/// parse view. Holding an `EvidenceRef` grants no access to restricted originals.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EvidenceRef {
    pub evidence_id: EvidenceId,
    pub source_id: SourceId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parse_view_id: Option<ParseViewId>,
    /// Absent for a metadata-only citation (bytes not obtained: listing or access-gated record).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locator: Option<SourceLocator>,
    pub evidence_class: EvidenceClass,
    /// Exact citable text from the parse view (not the normalized index text).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quote: Option<String>,
    pub origin: Origin,
}

impl EvidenceRef {
    pub fn validate(&self) -> Result<(), ContractError> {
        let c = "EvidenceRef";
        if let Some(l) = &self.locator {
            l.validate()?;
            if l.char_start.is_some() {
                ensure(
                    self.parse_view_id.as_ref().map(ParseViewId::as_str)
                        == l.parse_view_id.as_deref(),
                    c,
                    || "character offsets must count in the evidence's own parse view".into(),
                )?;
            }
        }
        ensure(self.locator.is_some() || self.quote.is_none(), c, || {
            "a quote needs a locator".into()
        })
    }
}

// ---------------------------------------------------------------- search

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SearchMode {
    #[default]
    Hybrid,
    Lexical,
    Dense,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SearchFilters {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub families: Vec<Family>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_kinds: Vec<SourceKind>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub investigation_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_ids: Vec<SourceId>,
}

/// A search. The access scope is not here: the host passes it to the service separately.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SearchRequest {
    pub schema_version: EvidenceSchema,
    pub query: String,
    #[serde(default)]
    pub mode: SearchMode,
    #[serde(default)]
    pub filters: SearchFilters,
    /// Defaults to the snapshot pinned by the scope/episode.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub corpus_snapshot: Option<CorpusSnapshotId>,
    /// Requested page size; the service may return fewer and pages on with `next_cursor`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor: Option<PageCursor>,
}

impl SearchRequest {
    pub fn validate(&self) -> Result<(), ContractError> {
        ensure(!self.query.trim().is_empty(), "SearchRequest", || {
            "empty query".into()
        })?;
        ensure(self.page_size != Some(0), "SearchRequest", || {
            "page_size must be positive".into()
        })
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum RetrievalBackend {
    Lexical,
    Dense,
    Rerank,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BackendStatus {
    Used,
    /// Requested but not available: results are degraded and say so.
    Unavailable,
    NotRequested,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BackendReport {
    pub backend: RetrievalBackend,
    pub status: BackendStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// Kind of a ranking score. Scores are not probabilities of truth and are not comparable across kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ScoreKind {
    Bm25,
    Cosine,
    Rrf,
    Rerank,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RetrievalScore {
    pub kind: ScoreKind,
    pub value: Finite,
}

/// One hit: a citable evidence pointer and a short snippet. The snippet is never the full passage;
/// read the source to get it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SearchHit {
    /// 1-based rank across the whole result list (continues across pages).
    pub rank: u32,
    pub evidence: EvidenceRef,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub snippet: String,
    #[serde(default)]
    pub scores: Vec<RetrievalScore>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SearchPage {
    pub schema_version: EvidenceSchema,
    pub corpus_snapshot: CorpusSnapshotId,
    pub mode: SearchMode,
    pub hits: Vec<SearchHit>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<PageCursor>,
    /// Exact total, or absent when the backend does not count exhaustively. An empty page does not
    /// certify that the corpus lacks the information.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_count: Option<u64>,
    pub backends: Vec<BackendReport>,
    pub origin: Origin,
}

impl SearchPage {
    /// True when a requested backend was unavailable (e.g. lexical-only fallback).
    pub fn degraded(&self) -> bool {
        self.backends
            .iter()
            .any(|b| b.status == BackendStatus::Unavailable)
    }

    pub fn validate(&self) -> Result<(), ContractError> {
        let c = "SearchPage";
        ensure(
            self.hits.windows(2).all(|w| w[0].rank < w[1].rank),
            c,
            || "ranks must increase".into(),
        )?;
        ensure(self.hits.first().is_none_or(|h| h.rank >= 1), c, || {
            "ranks are 1-based".into()
        })?;
        ensure(
            self.total_count.is_none_or(|t| t >= self.hits.len() as u64),
            c,
            || "total_count below hit count".into(),
        )?;
        let reported: BTreeSet<_> = self.backends.iter().map(|b| b.backend).collect();
        ensure(reported.len() == self.backends.len(), c, || {
            "backend reported twice".into()
        })?;
        let needed: &[RetrievalBackend] = match self.mode {
            SearchMode::Hybrid => &[RetrievalBackend::Lexical, RetrievalBackend::Dense],
            SearchMode::Lexical => &[RetrievalBackend::Lexical],
            SearchMode::Dense => &[RetrievalBackend::Dense],
        };
        ensure(needed.iter().all(|b| reported.contains(b)), c, || {
            format!("mode {:?} must report {needed:?}", self.mode)
        })?;
        self.hits.iter().try_for_each(|h| h.evidence.validate())
    }
}

// ---------------------------------------------------------------- reading

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ReadRange {
    /// Physical pages, 1-based, inclusive.
    Pages {
        first: u32,
        last: u32,
    },
    /// The passage an evidence reference points at, with its neighbourhood as the service defines.
    Passage {
        evidence_id: EvidenceId,
    },
    Table {
        table_id: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReadRequest {
    pub schema_version: EvidenceSchema,
    pub source_id: SourceId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parse_view_id: Option<ParseViewId>,
    pub range: ReadRange,
}

impl ReadRequest {
    pub fn validate(&self) -> Result<(), ContractError> {
        match self.range {
            ReadRange::Pages { first, last } => {
                ensure(1 <= first && first <= last, "ReadRequest", || {
                    "pages are 1-based with first <= last".into()
                })
            }
            _ => Ok(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReadSegment {
    pub locator: SourceLocator,
    pub text: String,
}

/// What was read and how to continue. `complete = false` always comes with `next`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReadResult {
    pub schema_version: EvidenceSchema,
    pub source_id: SourceId,
    pub parse_view_id: ParseViewId,
    pub segments: Vec<ReadSegment>,
    pub complete: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next: Option<ReadRange>,
    pub origin: Origin,
}

impl ReadResult {
    pub fn validate(&self) -> Result<(), ContractError> {
        ensure(self.complete == self.next.is_none(), "ReadResult", || {
            "incomplete reads say how to continue (and only they)".into()
        })?;
        self.segments.iter().try_for_each(|s| s.locator.validate())
    }
}

// ---------------------------------------------------------------- provenance links

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Relation {
    Supports,
    Contradicts,
    DerivedFrom,
    Describes,
    SameRunAs,
    Cites,
}

/// A typed edge between entities, with the evidence and method that established it. A purely
/// semantic association is `resolution = unknown`, never an exact identity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProvenanceLink {
    pub schema_version: EvidenceSchema,
    pub link_id: LinkId,
    pub from: EntityRef,
    pub to: EntityRef,
    pub relation: Relation,
    #[serde(default)]
    pub evidence: Vec<EvidenceId>,
    pub resolution: Resolution,
    pub method: Derivation,
    pub origin: Origin,
}

impl ProvenanceLink {
    pub fn validate(&self) -> Result<(), ContractError> {
        ensure(self.from != self.to, "ProvenanceLink", || {
            "a link needs two distinct entities".into()
        })
    }
}

// ---------------------------------------------------------------- wiki

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AuthorKind {
    Agent,
    Human,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Author {
    pub kind: AuthorKind,
    pub id: String,
}

/// A wiki write against `base_revision` (absent = create). The service refuses with `conflict` when
/// the page head moved: the last writer does not win silently.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WikiPatch {
    pub schema_version: EvidenceSchema,
    pub page_id: WikiPageId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_revision: Option<u64>,
    pub title: String,
    pub body_markdown: String,
    #[serde(default)]
    pub links: Vec<EntityRef>,
}

/// An immutable wiki revision. The wiki is research memory with references; it is never a primary
/// source and is not indexed as one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WikiRevision {
    pub schema_version: EvidenceSchema,
    pub page_id: WikiPageId,
    pub revision: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_revision: Option<u64>,
    pub title: String,
    pub body_markdown: String,
    #[serde(default)]
    pub links: Vec<EntityRef>,
    pub author: Author,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub episode_id: Option<EpisodeId>,
    pub created_at: Timestamp,
    pub origin: Origin,
}

impl WikiRevision {
    pub fn validate(&self) -> Result<(), ContractError> {
        let ok = match self.parent_revision {
            None => self.revision == 1,
            Some(p) => self.revision == p + 1,
        };
        ensure(ok, "WikiRevision", || {
            "revision is parent + 1 (1 for a new page)".into()
        })
    }
}
