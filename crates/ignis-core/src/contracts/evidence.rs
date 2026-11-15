//! Knowledge contracts: evidence references, search, reading, the combustion graph (provenance
//! links), document labels, the wiki and web discovery. Semantic owner: Raul (Iancu and Tudor
//! review numeric consumption).
//!
//! A source *version* is the existing [`Source`](super::Source) record: its `id` is what
//! [`SourceId`] names, and `id_basis` says whether the id proves the bytes. Locators reuse
//! [`SourceLocator`](super::SourceLocator). A search hit is a pointer with a snippet, never the
//! document: reading goes through [`ReadRequest`].

use std::borrow::Cow;
use std::collections::BTreeSet;
use std::fmt;

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{de, Deserialize, Deserializer, Serialize};

use super::common::{
    check_id, ensure, opaque_id, ArtifactRef, ContractError, CorpusSnapshotId, Digest, EntityRef,
    EpisodeId, EvidenceId, Finite, LinkId, Origin, PageCursor, ParseViewId, RunId, ScopeId,
    SourceId, Timestamp, WikiPageId,
};
use super::{
    Derivation, EvidenceClass, Family, Method, Partition, Quantity, Resolution, Rights,
    RightsStatus, SourceKind, SourceLocator,
};

/// Wire tag of the knowledge documents. Wave K1 writes `evidence.v2`; `evidence.v1` (C0) documents
/// are still read. An addition of v2 (a field, a relation, a concept node, a new document type) is
/// refused under the v1 tag, so a v1 reader never receives a document it would misread.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
pub enum EvidenceSchema {
    /// C0. Still read; new documents are written as v2.
    #[serde(rename = "evidence.v1")]
    V1,
    #[default]
    #[serde(rename = "evidence.v2")]
    V2,
}

impl EvidenceSchema {
    fn require_v2(self, contract: &'static str, what: &str) -> Result<(), ContractError> {
        ensure(self == EvidenceSchema::V2, contract, || {
            format!("{what} needs schema_version evidence.v2")
        })
    }
}

opaque_id! {
    /// A domain concept of the combustion graph (`PSI-69`, `fuel:n-heptane`, `hw:cir-mdca`).
    ConceptId;
    /// One itemized entry of a wiki page; stable across revisions of the page.
    WikiItemId;
    /// A section of a document's outline, stable within one parse view.
    OutlineNodeId;
}

/// How much a search or read returns per item: `concise` keeps ids, locators and short text for
/// scanning, `detailed` adds scores, full quotes and context. Absent = the service default. It
/// changes the size of each item, never which items match or their order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ResponseFormat {
    Concise,
    Detailed,
}

fn http_url(s: &str) -> bool {
    (s.starts_with("https://") || s.starts_with("http://"))
        && !s.chars().any(|c| c.is_whitespace() || c.is_control())
}

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
    /// v2.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormat>,
    /// v2. SIRA query expansion (LANES_V2 R12a): concepts, equivalent phrasings and discriminative
    /// terms a relevant passage would contain, never a guessed answer. The service keeps those the
    /// corpus holds, rarest first, and adds them to the lexical score with IDF weights; the page
    /// reports which ones it kept.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub expected_terms: Vec<ExpectedTerm>,
}

/// One expected term of a [`SearchRequest`] and how important the caller thinks it is.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ExpectedTerm {
    pub term: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub importance: Option<TermImportance>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TermImportance {
    High,
    Medium,
    Low,
}

impl TermImportance {
    /// The weight the finalizer gives a term of this importance (Corpus's SIRA: 1, 0.6, 0.3).
    pub fn weight(imp: Option<TermImportance>) -> f64 {
        match imp {
            Some(TermImportance::High) => 1.0,
            Some(TermImportance::Low) => 0.3,
            _ => 0.6,
        }
    }
}

impl SearchRequest {
    pub fn validate(&self) -> Result<(), ContractError> {
        ensure(!self.query.trim().is_empty(), "SearchRequest", || {
            "empty query".into()
        })?;
        ensure(self.page_size != Some(0), "SearchRequest", || {
            "page_size must be positive".into()
        })?;
        if self.response_format.is_some() {
            self.schema_version
                .require_v2("SearchRequest", "response_format")?;
        }
        if !self.expected_terms.is_empty() {
            self.schema_version
                .require_v2("SearchRequest", "expected_terms")?;
            ensure(
                self.expected_terms
                    .iter()
                    .all(|t| !t.term.trim().is_empty()),
                "SearchRequest",
                || "empty expected term".into(),
            )?;
        }
        Ok(())
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
    /// v2.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormat>,
}

impl ReadRequest {
    pub fn validate(&self) -> Result<(), ContractError> {
        if let ReadRange::Pages { first, last } = self.range {
            ensure(1 <= first && first <= last, "ReadRequest", || {
                "pages are 1-based with first <= last".into()
            })?;
        }
        if self.response_format.is_some() {
            self.schema_version
                .require_v2("ReadRequest", "response_format")?;
        }
        Ok(())
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

// ---------------------------------------------------------------- outline (section tree)

/// `sources_outline`: the section tree of one source, or the subtree under `node_id`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OutlineRequest {
    pub schema_version: EvidenceSchema,
    pub source_id: SourceId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parse_view_id: Option<ParseViewId>,
    /// Subtree root; absent = the whole document.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_id: Option<OutlineNodeId>,
    /// Levels below the root to return; deeper children are flagged, not dropped silently.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub depth: Option<u32>,
}

impl OutlineRequest {
    pub fn validate(&self) -> Result<(), ContractError> {
        let c = "OutlineRequest";
        self.schema_version.require_v2(c, c)?;
        ensure(self.depth != Some(0), c, || "depth must be positive".into())
    }
}

/// Text written by a model about a source. It is navigation help, never evidence: a claim cites
/// the passage, not the summary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GeneratedText {
    pub text: String,
    pub model_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_hash: Option<Digest>,
}

/// One section: the document's own heading, its physical page range (1-based, inclusive) and
/// children. `children_omitted` says the request's depth cut children that exist.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OutlineNode {
    pub node_id: OutlineNodeId,
    pub title: String,
    pub page_start: u32,
    pub page_end: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<GeneratedText>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<OutlineNode>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub children_omitted: bool,
}

impl OutlineNode {
    fn check<'a>(
        &'a self,
        parent: Option<(u32, u32)>,
        ids: &mut BTreeSet<&'a OutlineNodeId>,
    ) -> Result<(), ContractError> {
        let c = "OutlineNode";
        ensure(ids.insert(&self.node_id), c, || {
            format!("node {} listed twice", self.node_id)
        })?;
        ensure(
            1 <= self.page_start && self.page_start <= self.page_end,
            c,
            || format!("{}: pages are 1-based with start <= end", self.node_id),
        )?;
        ensure(
            parent.is_none_or(|(a, b)| a <= self.page_start && self.page_end <= b),
            c,
            || format!("{}: outside its parent's pages", self.node_id),
        )?;
        ensure(
            self.summary
                .as_ref()
                .is_none_or(|s| !s.text.trim().is_empty()),
            c,
            || format!("{}: empty summary", self.node_id),
        )?;
        ensure(
            self.children.is_empty() || !self.children_omitted,
            c,
            || format!("{}: children both listed and omitted", self.node_id),
        )?;
        let range = Some((self.page_start, self.page_end));
        self.children.iter().try_for_each(|n| n.check(range, ids))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SourceOutline {
    pub schema_version: EvidenceSchema,
    pub source_id: SourceId,
    pub parse_view_id: ParseViewId,
    pub nodes: Vec<OutlineNode>,
    pub origin: Origin,
}

impl SourceOutline {
    pub fn validate(&self) -> Result<(), ContractError> {
        self.schema_version
            .require_v2("SourceOutline", "SourceOutline")?;
        let mut ids = BTreeSet::new();
        self.nodes.iter().try_for_each(|n| n.check(None, &mut ids))
    }
}

// ---------------------------------------------------------------- source access

/// How the UI reaches an original's bytes without IGNIS storing them (LANES_V2 §7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AccessMode {
    /// The browser streams from NASA (signed PSI S3 URL, NASA media library).
    Direct,
    /// Through the Worker pass-through proxy (NTRS: no CORS); nothing is stored.
    Proxy,
    /// Only a link to the provider's page.
    LinkOut,
}

/// Delivery of one source version. The identity stays the `Source` (geode URL + file version +
/// hash); `url` is a short-lived delivery detail and is never stored in a citation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SourceAccess {
    pub schema_version: EvidenceSchema,
    pub source_id: SourceId,
    pub mode: AccessMode,
    pub url: String,
    /// When a signed `url` stops working; re-resolve after it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<Timestamp>,
    /// The URL answers HTTP Range requests.
    pub range: bool,
    /// The URL is readable by scripts from the app's origin.
    pub cors: bool,
    /// "Open at NASA": the provider's own page for the source.
    pub landing_url: String,
    pub origin: Origin,
}

impl SourceAccess {
    pub fn validate(&self) -> Result<(), ContractError> {
        let c = "SourceAccess";
        self.schema_version.require_v2(c, c)?;
        for u in [&self.url, &self.landing_url] {
            ensure(http_url(u), c, || format!("{u:?} is not an http(s) URL"))?;
        }
        if self.mode == AccessMode::LinkOut {
            ensure(
                !self.range && !self.cors && self.expires_at.is_none(),
                c,
                || "a link out is a page to open, not bytes to stream".into(),
            )?;
        }
        Ok(())
    }
}

// ---------------------------------------------------------------- media indexes (R11)

/// Video codec of an indexed MPEG-TS recording.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VideoCodec {
    H264,
    Hevc,
    Mpeg2,
}

/// Which keyframes an index lists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum KeyframeCoverage {
    /// Every keyframe of the recording (the whole file was read).
    All,
    /// Keyframes found by byte-range probes a few seconds of presentation time apart; enough for
    /// keyframe-aligned segments and I-frame scrubbing, not a list of every keyframe.
    Sampled,
}

/// One keyframe of an MPEG-TS recording.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct KeyframeEntry {
    /// Byte offset of the TS packet that starts the keyframe's PES.
    pub offset: u64,
    /// Bytes from `offset` to the next video PES start: the keyframe's access unit plus the
    /// packets of other streams interleaved with it (whole packets). The I-frame byte range.
    pub length: u64,
    /// Presentation time in seconds (90 kHz PTS, 33-bit wraps unwrapped by the indexer).
    pub pts_s: Finite,
}

/// Keyframe index of one `.ts` source version (R11), the input of M12's byte-range HLS
/// playlists. Written by `ignis sources media keyframes`, read by `/api/v1/sources/{id}/â€¦m3u8`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MediaKeyframeIndex {
    pub schema_version: EvidenceSchema,
    pub source_id: SourceId,
    /// Size of the indexed object; a source of another size makes the index stale.
    pub size_bytes: u64,
    /// Length of the leading packets up to and including the first PMT (whole packets): the
    /// playlists' initialization section (`EXT-X-MAP`).
    pub init_bytes: u64,
    pub video_pid: u16,
    pub codec: VideoCodec,
    /// Presentation time where the last frame ends (last PTS + one frame duration).
    pub end_pts_s: Finite,
    pub coverage: KeyframeCoverage,
    pub keyframes: Vec<KeyframeEntry>,
    /// Bytes the indexer read to build this index (byte ranges, not the whole file).
    pub bytes_read: u64,
    /// Indexer name and version.
    pub indexer: String,
    pub origin: Origin,
}

/// MPEG-TS packet size.
pub const TS_PACKET_BYTES: u64 = 188;

impl MediaKeyframeIndex {
    /// End of the last whole packet.
    pub fn media_end(&self) -> u64 {
        self.size_bytes - self.size_bytes % TS_PACKET_BYTES
    }

    /// PTS span from the first keyframe to the end.
    pub fn duration_s(&self) -> f64 {
        self.keyframes
            .first()
            .map_or(0.0, |k| self.end_pts_s.get() - k.pts_s.get())
    }

    /// Every rule a playable playlist depends on (fail closed).
    pub fn validate(&self) -> Result<(), ContractError> {
        let c = "MediaKeyframeIndex";
        self.schema_version.require_v2(c, c)?;
        let p = TS_PACKET_BYTES;
        ensure(self.init_bytes > 0 && self.init_bytes % p == 0, c, || {
            format!(
                "init_bytes {} is not a whole number of {p}-byte packets",
                self.init_bytes
            )
        })?;
        ensure(!self.indexer.trim().is_empty(), c, || {
            "indexer is empty".into()
        })?;
        let end = self.media_end();
        let (Some(first), Some(last)) = (self.keyframes.first(), self.keyframes.last()) else {
            return Err(ContractError::new(c, "index has no keyframes"));
        };
        ensure(first.offset >= self.init_bytes, c, || {
            "the first keyframe lies inside the init section".into()
        })?;
        for (i, k) in self.keyframes.iter().enumerate() {
            ensure(
                k.offset % p == 0 && k.length % p == 0 && k.length > 0,
                c,
                || format!("keyframe {i}: offset/length are not whole {p}-byte packets"),
            )?;
            let next = self.keyframes.get(i + 1).map_or(end, |n| n.offset);
            ensure(
                k.offset.checked_add(k.length).is_some_and(|e| e <= next),
                c,
                || format!("keyframe {i}: its range runs into the next keyframe or past the end"),
            )?;
            if let Some(n) = self.keyframes.get(i + 1) {
                ensure(
                    n.offset > k.offset && n.pts_s.get() > k.pts_s.get(),
                    c,
                    || {
                        format!(
                            "keyframe {}: offsets and PTS must both increase strictly",
                            i + 1
                        )
                    },
                )?;
            }
        }
        ensure(self.end_pts_s.get() > last.pts_s.get(), c, || {
            "end_pts_s must be after the last keyframe".into()
        })?;
        Ok(())
    }
}

/// One entry of a remote archive's central directory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ArchiveEntry {
    /// Path inside the archive (what `/sources/{id}/archive/entry?name=` takes).
    pub name: String,
    pub size_bytes: u64,
    pub compressed_bytes: u64,
    /// A TIFF frame: served converted to PNG (faithful) or WebP (display) on request.
    pub frame: bool,
}

/// A page of a remote archive's entries (R11), read from its cached central directory; the
/// archive itself stays at NASA.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ArchiveListing {
    pub schema_version: EvidenceSchema,
    pub source_id: SourceId,
    pub archive_size_bytes: u64,
    /// Entries in the archive (all pages).
    pub total_entries: u64,
    /// Index of this page's first entry in the directory order.
    pub first: u64,
    pub entries: Vec<ArchiveEntry>,
    /// `after` for the next page; absent on the last page.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_after: Option<u64>,
    pub origin: Origin,
}

impl ArchiveListing {
    pub fn validate(&self) -> Result<(), ContractError> {
        let c = "ArchiveListing";
        self.schema_version.require_v2(c, c)?;
        ensure(
            self.first + self.entries.len() as u64 <= self.total_entries,
            c,
            || "page runs past the archive's entries".into(),
        )?;
        ensure(
            self.next_after.is_none_or(|n| {
                n == self.first + self.entries.len() as u64 && n < self.total_entries
            }),
            c,
            || "next_after must continue right after this page".into(),
        )?;
        ensure(self.entries.iter().all(|e| !e.name.is_empty()), c, || {
            "an entry has no name".into()
        })
    }
}

// ---------------------------------------------------------------- combustion graph

/// Kind of a domain concept of the combustion graph. Concepts are vocabulary nodes (an
/// investigation, a fuel, a piece of hardware); records (sources, runs, artifacts) are
/// [`EntityRef`]s.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ConceptKind {
    /// A PSI investigation (`id` = its accession, e.g. `PSI-69`) or a ground investigation.
    Investigation,
    Fuel,
    SampleGeometry,
    /// Flight or ground hardware (CIR/MDCA, MSG/SPICE duct, an ACME module).
    Hardware,
    /// A controlled condition (oxygen mole fraction, pressure, flow, gravity level).
    Condition,
    /// An instrument or measurement method (camera, radiometer, thermocouple).
    Diagnostic,
    /// A measured quantity of interest (extinction diameter, spread rate, burning rate).
    Observable,
}

#[derive(
    Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct ConceptRef {
    pub concept: ConceptKind,
    pub id: ConceptId,
}

/// A graph node: an IGNIS record ([`EntityRef`], wire `{kind, id}`) or a domain concept
/// ([`ConceptRef`], wire `{concept, id}`). A v1 link endpoint (always an `EntityRef`) reads
/// unchanged. Passages, figures and tables are `EntityRef::Evidence`; analyses are artifacts.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(untagged)]
pub enum GraphNode {
    Entity(EntityRef),
    Concept(ConceptRef),
}

impl GraphNode {
    fn is_concept(&self, k: ConceptKind) -> bool {
        matches!(self, GraphNode::Concept(c) if c.concept == k)
    }
}

/// Edge type of the graph (master §11.3). The catalog is closed: a new relation is a contract change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Relation {
    Supports,
    /// C0 name, read only: v2 links write `disputes`.
    Contradicts,
    DerivedFrom,
    /// A source describes an entity in general (an investigation, a method), not a specific run.
    Describes,
    /// Exact run identity between two records (identifier, conditions, time or campaign).
    SameRunAs,
    Cites,
    // ---- v2
    /// A source or passage describes one specific run (`to` is a run).
    DescribesRun,
    UsesFuel,
    HasCondition,
    MeasuredBy,
    UsesHardware,
    UsesCalibration,
    /// One result questions another. Two results at other conditions are not a dispute by
    /// themselves: the populations must overlap.
    Disputes,
    PredictedBy,
    /// `from` replaces `to`; descendants of `to` are marked possibly stale, never rewritten.
    Supersedes,
    /// A possible identity found by similarity or a label. Never an exact identity: its resolution
    /// is never `resolved`.
    SameAsCandidate,
}

impl Relation {
    fn in_v1(self) -> bool {
        use Relation::*;
        matches!(
            self,
            Supports | Contradicts | DerivedFrom | Describes | SameRunAs | Cites
        )
    }
}

/// A typed edge between two nodes, with the evidence and method that established it. A purely
/// semantic association (no evidence) is never `resolved`; identity needs an identifier,
/// conditions, time or an explicit reference.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProvenanceLink {
    pub schema_version: EvidenceSchema,
    pub link_id: LinkId,
    pub from: GraphNode,
    pub to: GraphNode,
    pub relation: Relation,
    #[serde(default)]
    pub evidence: Vec<EvidenceId>,
    pub resolution: Resolution,
    pub method: Derivation,
    pub origin: Origin,
}

impl ProvenanceLink {
    pub fn validate(&self) -> Result<(), ContractError> {
        let c = "ProvenanceLink";
        ensure(self.from != self.to, c, || {
            "a link needs two distinct entities".into()
        })?;
        let v2_shape = !self.relation.in_v1()
            || matches!(self.from, GraphNode::Concept(_))
            || matches!(self.to, GraphNode::Concept(_));
        if v2_shape {
            self.schema_version
                .require_v2(c, "a v2 relation or a concept node")?;
        }
        if self.schema_version == EvidenceSchema::V2 {
            ensure(self.relation != Relation::Contradicts, c, || {
                "v2 links write `disputes`, not `contradicts`".into()
            })?;
        }
        let resolved = self.resolution == Resolution::Resolved;
        ensure(
            !(resolved && self.relation == Relation::SameAsCandidate),
            c,
            || "a same_as_candidate link is never a resolved identity".into(),
        )?;
        ensure(
            !(resolved && self.method.method == Method::Semantic && self.evidence.is_empty()),
            c,
            || "a purely semantic association (no evidence) is not resolved".into(),
        )?;
        ensure(
            !(resolved
                && self.method.method == Method::Semantic
                && self.relation == Relation::SameRunAs),
            c,
            || "similarity does not establish run identity".into(),
        )?;
        use ConceptKind as K;
        let to_ok = match self.relation {
            Relation::DescribesRun => matches!(self.to, GraphNode::Entity(EntityRef::Run(_))),
            Relation::UsesFuel => self.to.is_concept(K::Fuel),
            Relation::UsesHardware => self.to.is_concept(K::Hardware),
            Relation::HasCondition => self.to.is_concept(K::Condition),
            Relation::MeasuredBy => self.to.is_concept(K::Diagnostic),
            Relation::PredictedBy => {
                matches!(self.to, GraphNode::Entity(EntityRef::Release(_)))
            }
            _ => true,
        };
        ensure(to_ok, c, || {
            format!("{:?} does not point at {:?}", self.relation, self.to)
        })
    }
}

/// Which side of the requested node an edge sits on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EdgeDirection {
    /// The node is the link's `from`.
    Outgoing,
    /// The node is the link's `to`.
    Incoming,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DirectionFilter {
    Outgoing,
    Incoming,
    #[default]
    Both,
}

/// `graph_neighbors`: the typed neighbours of one node, paged. For joins and filters ("runs that
/// share MDCA"), not community summaries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GraphNeighborsRequest {
    pub schema_version: EvidenceSchema,
    pub node: GraphNode,
    /// Empty = every relation.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub relations: Vec<Relation>,
    #[serde(default)]
    pub direction: DirectionFilter,
    /// Defaults to the snapshot pinned by the scope/episode.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub corpus_snapshot: Option<CorpusSnapshotId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor: Option<PageCursor>,
}

impl GraphNeighborsRequest {
    pub fn validate(&self) -> Result<(), ContractError> {
        let c = "GraphNeighborsRequest";
        self.schema_version.require_v2(c, c)?;
        ensure(self.page_size != Some(0), c, || {
            "page_size must be positive".into()
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GraphNeighbor {
    pub direction: EdgeDirection,
    pub link: ProvenanceLink,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GraphNeighborsPage {
    pub schema_version: EvidenceSchema,
    pub corpus_snapshot: CorpusSnapshotId,
    pub node: GraphNode,
    pub neighbors: Vec<GraphNeighbor>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<PageCursor>,
    /// Exact total, or absent when not counted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_count: Option<u64>,
    pub origin: Origin,
}

impl GraphNeighborsPage {
    pub fn validate(&self) -> Result<(), ContractError> {
        let c = "GraphNeighborsPage";
        self.schema_version.require_v2(c, c)?;
        ensure(
            self.total_count
                .is_none_or(|t| t >= self.neighbors.len() as u64),
            c,
            || "total_count below neighbour count".into(),
        )?;
        let mut ids = BTreeSet::new();
        for n in &self.neighbors {
            n.link.validate()?;
            ensure(ids.insert(&n.link.link_id), c, || {
                format!("link {} listed twice", n.link.link_id)
            })?;
            let end = match n.direction {
                EdgeDirection::Outgoing => &n.link.from,
                EdgeDirection::Incoming => &n.link.to,
            };
            ensure(*end == self.node, c, || {
                format!(
                    "link {} does not touch the node on its side",
                    n.link.link_id
                )
            })?;
        }
        Ok(())
    }
}

// ---------------------------------------------------------------- document labels

/// Closed catalog of document types. A new type is a contract change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DocType {
    /// NASA TP/TM/CR or an equivalent agency report.
    TechnicalReport,
    JournalArticle,
    ConferencePaper,
    Presentation,
    Thesis,
    /// Science requirements (SRD), test matrices.
    RequirementsDocument,
    /// Hardware design, interface and verification documents.
    EngineeringDocument,
    /// Operations, procedures, timelines.
    OperationsDocument,
    /// Readme, data dictionary, file description.
    DataDescription,
    /// Experimental table or measurement file described as a document.
    DataTable,
    Outreach,
    Other,
}

/// Who produced a label: a model (method `semantic`, with model id and prompt hash) or a reviewer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Labeller {
    pub method: Method,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_hash: Option<Digest>,
}

impl Labeller {
    pub fn validate(&self) -> Result<(), ContractError> {
        ensure(
            self.method != Method::Semantic
                || (self.model_id.is_some() && self.prompt_hash.is_some()),
            "Labeller",
            || "a semantic label names its model and prompt hash".into(),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LabelAgreement {
    /// Independent passes run on the document.
    pub passes: u32,
    pub agreed: bool,
}

/// What a document is about, as classified. A label describes; it never establishes identity:
/// the runs it names are mentions, which enter the graph as `same_as_candidate` / `describes_run`
/// links with resolution `unknown` until an identifier or condition match resolves them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SourceLabel {
    pub schema_version: EvidenceSchema,
    pub source_id: SourceId,
    pub doc_type: DocType,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub families: BTreeSet<Family>,
    /// Investigation accessions (`PSI-69`) or registered ground-investigation ids.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub investigation_ids: BTreeSet<String>,
    /// Runs the document mentions (not resolved identities).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub runs: Vec<RunId>,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub conditions_mentioned: BTreeSet<Quantity>,
    /// Passages that support the label.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<EvidenceId>,
    pub labeller: Labeller,
    /// Self-reported or calibrated confidence in [0, 1]; not a probability of truth.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<Finite>,
    pub agreement: LabelAgreement,
    /// `resolved` only when the passes agree (two or more for a model label);
    /// `unresolved_conflict` keeps a disagreement visible.
    pub status: Resolution,
    pub origin: Origin,
}

impl SourceLabel {
    pub fn validate(&self) -> Result<(), ContractError> {
        let c = "SourceLabel";
        self.schema_version.require_v2(c, c)?;
        self.labeller.validate()?;
        ensure(
            self.confidence
                .is_none_or(|x| (0.0..=1.0).contains(&x.get())),
            c,
            || "confidence is in [0, 1]".into(),
        )?;
        ensure(self.agreement.passes >= 1, c, || {
            "a label comes from at least one pass".into()
        })?;
        if self.status == Resolution::Resolved {
            ensure(self.agreement.agreed, c, || {
                "disagreeing passes are not resolved".into()
            })?;
            ensure(
                self.labeller.method != Method::Semantic || self.agreement.passes >= 2,
                c,
                || "a model label is resolved only by two agreeing passes".into(),
            )?;
        }
        ensure(
            self.investigation_ids.iter().all(|i| !i.trim().is_empty()),
            c,
            || "empty investigation id".into(),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LabelCounts {
    pub total: u64,
    pub resolved: u64,
    pub unresolved_conflict: u64,
}

/// Accuracy of the labels on a hand-labelled gold set, measured before the labels are used.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LabelAccuracy {
    pub gold_set: ArtifactRef,
    pub documents: u64,
    pub doc_type_accuracy: Finite,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub investigation_accuracy: Option<Finite>,
}

/// One labelling batch over a corpus snapshot: the labels are a `SourceLabel` JSONL artifact.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LabelSet {
    pub schema_version: EvidenceSchema,
    pub corpus_snapshot: CorpusSnapshotId,
    pub labeller: Labeller,
    pub labels: ArtifactRef,
    pub counts: LabelCounts,
    /// Absent until measured; labels without a measured accuracy are not used for claims.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accuracy: Option<LabelAccuracy>,
    pub origin: Origin,
    pub created_at: Timestamp,
}

impl LabelSet {
    pub fn validate(&self) -> Result<(), ContractError> {
        let c = "LabelSet";
        self.schema_version.require_v2(c, c)?;
        self.labeller.validate()?;
        self.labels.validate()?;
        ensure(
            self.labels.contract.as_deref() == Some("SourceLabel"),
            c,
            || "labels are SourceLabel records".into(),
        )?;
        ensure(
            self.counts.resolved + self.counts.unresolved_conflict <= self.counts.total,
            c,
            || "counts exceed the total".into(),
        )?;
        if let Some(a) = &self.accuracy {
            a.gold_set.validate()?;
            ensure(a.documents > 0, c, || {
                "an empty gold set measures nothing".into()
            })?;
            let unit = |x: Finite| (0.0..=1.0).contains(&x.get());
            ensure(
                unit(a.doc_type_accuracy) && a.investigation_accuracy.is_none_or(unit),
                c,
                || "accuracies are in [0, 1]".into(),
            )?;
        }
        Ok(())
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum WikiItemKind {
    Concept,
    Method,
    Protocol,
    Hypothesis,
    Calculation,
    Contradiction,
    OpenQuestion,
    Lesson,
    Note,
}

/// One itemized entry of a page. Pages change by item deltas, never by rewriting the page, so an
/// edit cannot silently drop what it did not mean to touch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WikiItem {
    pub item_id: WikiItemId,
    pub kind: WikiItemKind,
    /// Markdown.
    pub text: String,
    /// Sources, passages, artifacts, runs or concepts the item rests on.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub refs: Vec<GraphNode>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum WikiOp {
    Add {
        item: WikiItem,
    },
    /// New content for an existing item (same id).
    Revise {
        item: WikiItem,
    },
    Remove {
        item_id: WikiItemId,
        reason: String,
    },
}

impl WikiOp {
    fn item_id(&self) -> &WikiItemId {
        match self {
            WikiOp::Add { item } | WikiOp::Revise { item } => &item.item_id,
            WikiOp::Remove { item_id, .. } => item_id,
        }
    }
}

/// A wiki write against `base_revision` (absent = create the page). The service refuses with
/// `conflict` when the page head moved: the last writer does not win silently. The author, episode
/// and access of the written items are derived by the host, never taken from the patch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WikiPatch {
    pub schema_version: EvidenceSchema,
    pub page_id: WikiPageId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_revision: Option<u64>,
    /// Required to create a page; a rename otherwise.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ops: Vec<WikiOp>,
}

impl WikiPatch {
    pub fn validate(&self) -> Result<(), ContractError> {
        let c = "WikiPatch";
        self.schema_version
            .require_v2(c, "an itemized wiki patch")?;
        ensure(
            self.title.as_deref().is_none_or(|t| !t.trim().is_empty()),
            c,
            || "empty title".into(),
        )?;
        ensure(self.title.is_some() || !self.ops.is_empty(), c, || {
            "an empty patch".into()
        })?;
        if self.base_revision.is_none() {
            ensure(self.title.is_some(), c, || {
                "a new page needs a title".into()
            })?;
            ensure(
                self.ops.iter().all(|o| matches!(o, WikiOp::Add { .. })),
                c,
                || "a new page can only add items".into(),
            )?;
        }
        ensure(self.base_revision != Some(0), c, || {
            "revisions count from 1".into()
        })?;
        let mut touched = BTreeSet::new();
        for op in &self.ops {
            ensure(touched.insert(op.item_id()), c, || {
                format!("item {} touched twice in one patch", op.item_id())
            })?;
            match op {
                WikiOp::Add { item } | WikiOp::Revise { item } => {
                    ensure(!item.text.trim().is_empty(), c, || {
                        format!("item {} is empty", item.item_id)
                    })?
                }
                WikiOp::Remove { reason, .. } => {
                    ensure(!reason.trim().is_empty(), c, || "a removal says why".into())?
                }
            }
        }
        Ok(())
    }
}

/// An item as stored in a revision, with the access the host derived for it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WikiEntry {
    pub item_id: WikiItemId,
    pub kind: WikiItemKind,
    pub text: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub refs: Vec<GraphNode>,
    /// Partitions whose outcomes the item was written under (from the writing scope, never from
    /// the author). A reader needs every one of them: summarizing does not launder a restriction.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub requires_partitions: BTreeSet<Partition>,
    /// The page revision that last added or revised this item: its `author`, `episode_id` and
    /// `scope_id` are the item's provenance (host-derived, never from the patch).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub written_in: Option<u64>,
}

/// An immutable wiki revision: the full item list after `ops`, content-addressed by
/// `content_digest`. The wiki is research memory with references; it is never a primary source
/// and is not indexed as one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WikiRevision {
    pub schema_version: EvidenceSchema,
    pub page_id: WikiPageId,
    pub revision: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_revision: Option<u64>,
    pub title: String,
    pub items: Vec<WikiEntry>,
    /// The delta that produced this revision from its parent.
    pub ops: Vec<WikiOp>,
    /// [`WikiRevision::digest_of`] of `title` and `items`.
    pub content_digest: Digest,
    pub author: Author,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub episode_id: Option<EpisodeId>,
    /// The execution scope that wrote this revision (host-derived).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope_id: Option<ScopeId>,
    pub created_at: Timestamp,
    pub origin: Origin,
}

impl WikiRevision {
    pub fn digest_of(title: &str, items: &[WikiEntry]) -> Digest {
        Digest::of_json(&serde_json::json!({"title": title, "items": items}))
    }

    pub fn validate(&self) -> Result<(), ContractError> {
        let c = "WikiRevision";
        self.schema_version
            .require_v2(c, "an itemized wiki revision")?;
        let ok = match self.parent_revision {
            None => self.revision == 1,
            Some(p) => self.revision == p + 1,
        };
        ensure(ok, c, || "revision is parent + 1 (1 for a new page)".into())?;
        let mut ids = BTreeSet::new();
        for e in &self.items {
            ensure(ids.insert(&e.item_id), c, || {
                format!("item {} listed twice", e.item_id)
            })?;
            ensure(
                e.written_in
                    .is_none_or(|w| (1..=self.revision).contains(&w)),
                c,
                || format!("item {} written in a revision after this one", e.item_id),
            )?;
        }
        for op in &self.ops {
            if let WikiOp::Add { item } | WikiOp::Revise { item } = op {
                let e = self.items.iter().find(|e| e.item_id == item.item_id);
                ensure(
                    e.is_some_and(|e| e.written_in.is_none_or(|w| w == self.revision)),
                    c,
                    || {
                        format!(
                            "item {} written by this revision is missing or misdated",
                            item.item_id
                        )
                    },
                )?;
            }
        }
        let expected = Self::digest_of(&self.title, &self.items);
        ensure(self.content_digest == expected, c, || {
            format!("content_digest does not match title and items (expected {expected})")
        })
    }
}

/// `wiki_read`: one page (head, or a given revision), or the page index when `page_id` is absent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WikiReadRequest {
    pub schema_version: EvidenceSchema,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page_id: Option<WikiPageId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revision: Option<u64>,
    /// Index only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
    /// Index only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor: Option<PageCursor>,
}

impl WikiReadRequest {
    pub fn validate(&self) -> Result<(), ContractError> {
        let c = "WikiReadRequest";
        self.schema_version.require_v2(c, c)?;
        ensure(self.revision.is_none() || self.page_id.is_some(), c, || {
            "a revision belongs to a page".into()
        })?;
        ensure(self.revision != Some(0), c, || {
            "revisions count from 1".into()
        })?;
        ensure(
            self.page_id.is_none() || (self.page_size.is_none() && self.cursor.is_none()),
            c,
            || "paging applies to the index, not to a page".into(),
        )?;
        ensure(self.page_size != Some(0), c, || {
            "page_size must be positive".into()
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WikiPageSummary {
    pub page_id: WikiPageId,
    pub title: String,
    pub head_revision: u64,
    pub item_count: u64,
    pub updated_at: Timestamp,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum WikiReadBody {
    Page {
        revision: Box<WikiRevision>,
    },
    Index {
        pages: Vec<WikiPageSummary>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        next_cursor: Option<PageCursor>,
    },
}

/// Only what the scope may read: items whose `requires_partitions` the scope lacks are withheld
/// by the service (and `withheld_items` says how many), never silently dropped.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WikiReadResult {
    pub schema_version: EvidenceSchema,
    pub body: WikiReadBody,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub withheld_items: Option<u64>,
    pub origin: Origin,
}

impl WikiReadResult {
    pub fn validate(&self) -> Result<(), ContractError> {
        let c = "WikiReadResult";
        self.schema_version.require_v2(c, c)?;
        match &self.body {
            WikiReadBody::Page { revision } => revision.validate(),
            WikiReadBody::Index { .. } => ensure(self.withheld_items.is_none(), c, || {
                "withheld_items counts the items of a page".into()
            }),
        }
    }
}

// ---------------------------------------------------------------- web discovery

/// The provider adapter that answered (Parallel MCP at first, master §13.1). Kept apart from the
/// contract so a provider or URL change does not change the rest of the application.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WebProvider {
    pub name: String,
    pub tool: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum WebContent {
    /// Only the provider's excerpt: not the document. Read the original before a sensitive claim.
    Excerpt,
    /// The bytes were fetched and stored as `snapshot`.
    FullSnapshot,
}

/// A source found on the web. It is not a corpus source until promoted (`promoted_source_id`); an
/// episode may cite it from its snapshot without promotion. Its text is data, never instructions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ExternalSourceCandidate {
    pub schema_version: EvidenceSchema,
    /// As found (search result or request).
    pub url: String,
    /// After redirects; absent when nothing was fetched.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub final_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canonical_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub excerpt: Option<String>,
    pub content: WebContent,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub snapshot: Option<ArtifactRef>,
    pub accessed_at: Timestamp,
    pub provider: WebProvider,
    #[serde(default)]
    pub rights: Rights,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub promoted_source_id: Option<SourceId>,
    pub origin: Origin,
}

impl ExternalSourceCandidate {
    pub fn validate(&self) -> Result<(), ContractError> {
        let c = "ExternalSourceCandidate";
        self.schema_version.require_v2(c, c)?;
        for u in [
            Some(&self.url),
            self.final_url.as_ref(),
            self.canonical_url.as_ref(),
        ]
        .into_iter()
        .flatten()
        {
            ensure(http_url(u), c, || format!("{u:?} is not an http(s) URL"))?;
        }
        ensure(
            (self.content == WebContent::FullSnapshot) == self.snapshot.is_some(),
            c,
            || "a full snapshot is stored as an artifact, an excerpt is not".into(),
        )?;
        if let Some(s) = &self.snapshot {
            s.validate()?;
            ensure(self.final_url.is_some(), c, || {
                "a fetched snapshot records its final URL".into()
            })?;
        }
        if self.promoted_source_id.is_some() {
            ensure(self.content == WebContent::FullSnapshot, c, || {
                "an excerpt is never promoted into the corpus".into()
            })?;
            ensure(
                matches!(
                    self.rights.status,
                    RightsStatus::Verified | RightsStatus::Noncommercial
                ),
                c,
                || "promotion needs verified rights".into(),
            )?;
        }
        Ok(())
    }
}

/// `web_search`. Off in evaluation scopes and protected authoring (master §13).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WebSearchRequest {
    pub schema_version: EvidenceSchema,
    pub query: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor: Option<PageCursor>,
}

impl WebSearchRequest {
    pub fn validate(&self) -> Result<(), ContractError> {
        let c = "WebSearchRequest";
        self.schema_version.require_v2(c, c)?;
        ensure(!self.query.trim().is_empty(), c, || "empty query".into())?;
        ensure(self.page_size != Some(0), c, || {
            "page_size must be positive".into()
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WebSearchPage {
    pub schema_version: EvidenceSchema,
    pub query: String,
    pub provider: WebProvider,
    pub candidates: Vec<ExternalSourceCandidate>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<PageCursor>,
    pub origin: Origin,
}

impl WebSearchPage {
    pub fn validate(&self) -> Result<(), ContractError> {
        let c = "WebSearchPage";
        self.schema_version.require_v2(c, c)?;
        self.candidates
            .iter()
            .try_for_each(ExternalSourceCandidate::validate)
    }
}

/// `web_fetch`: fetch one URL into a snapshot. The host applies the SSRF policy (schemes, DNS/IP
/// after redirects, no private, link-local or metadata addresses); the contract only admits http(s).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WebFetchRequest {
    pub schema_version: EvidenceSchema,
    pub url: String,
}

impl WebFetchRequest {
    pub fn validate(&self) -> Result<(), ContractError> {
        let c = "WebFetchRequest";
        self.schema_version.require_v2(c, c)?;
        ensure(http_url(&self.url), c, || {
            format!("{:?} is not an http(s) URL", self.url)
        })
    }
}
