//! The research wiki (R4): workspace notes kept across episodes, written as itemized deltas.
//!
//! A page changes only by [`WikiOp`]s (add, revise or remove one item by id) against the head it
//! was read at; there is no "set page" operation, and a patch that would replace every item of a
//! page is refused as a whole-page rewrite (ACE: incremental deltas avoid context collapse). A
//! moved head is refused as `conflict`, never merged. Every revision is immutable and stored by
//! its content digest ([`WikiRevision::digest_of`]): `wiki_blobs` is a content-addressed table
//! that is verified on every read, and `wiki_revisions` keeps the whole history.
//!
//! An item is a claim of kind interpretation. It carries its provenance (the revision that wrote
//! it, whose author, episode and scope the host derived) and the references it rests on, but it is
//! never evidence: the wiki is not indexed by search, and `wiki_read` does not make anything
//! citable. Access follows the writer, not the reader: a written item requires every partition of
//! the scope that wrote it, so a summary written under `test` never reaches a research scope, and
//! a writer may only cite evidence its own scope can resolve.
//!
//! Storage is one SQLite file (WAL). Reads load one revision of one page; the index is paged by
//! page id. Nothing loads the whole wiki.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use ignis_core::contracts::{
    Author, Capability, Digest, EntityRef, EpisodeId, ErrorCode, ErrorEnvelope, EvidenceId,
    EvidenceSchema, ExecutionScope, GraphNode, Origin, PageCursor, Partition, RetryPolicy, ScopeId,
    Timestamp, WikiEntry, WikiItemId, WikiOp, WikiPageId, WikiPageSummary, WikiPatch, WikiReadBody,
    WikiReadRequest, WikiReadResult, WikiRevision,
};
use ignis_core::services::ServiceResult;
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use serde::{Deserialize, Serialize};

/// Index page size when the request names none.
pub const DEFAULT_INDEX_PAGE: u32 = 50;

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS wiki_blobs (
    digest TEXT PRIMARY KEY,
    body   TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS wiki_revisions (
    workspace_id      TEXT NOT NULL,
    page_id           TEXT NOT NULL,
    revision          INTEGER NOT NULL,
    parent_revision   INTEGER,
    content_digest    TEXT NOT NULL REFERENCES wiki_blobs(digest),
    ops               TEXT NOT NULL,
    author            TEXT NOT NULL,
    episode_id        TEXT,
    scope_id          TEXT NOT NULL,
    writer_partitions TEXT NOT NULL,
    created_at        TEXT NOT NULL,
    origin            TEXT NOT NULL,
    PRIMARY KEY (workspace_id, page_id, revision)
);
CREATE TABLE IF NOT EXISTS wiki_pages (
    workspace_id  TEXT NOT NULL,
    page_id       TEXT NOT NULL,
    title         TEXT NOT NULL,
    head_revision INTEGER NOT NULL,
    item_count    INTEGER NOT NULL,
    updated_at    TEXT NOT NULL,
    PRIMARY KEY (workspace_id, page_id)
);
";

/// The content-addressed part of a revision (what [`WikiRevision::digest_of`] hashes).
#[derive(Serialize, Deserialize)]
struct Content {
    title: String,
    items: Vec<WikiEntry>,
}

fn err(code: ErrorCode, msg: impl Into<String>) -> ErrorEnvelope {
    ErrorEnvelope::new(code, msg, RetryPolicy::Never)
}

fn internal(e: impl std::fmt::Display) -> ErrorEnvelope {
    err(ErrorCode::Internal, format!("wiki store: {e}"))
}

fn to_json<T: Serialize>(v: &T) -> ServiceResult<String> {
    serde_json::to_string(v).map_err(internal)
}

fn from_json<T: for<'de> Deserialize<'de>>(s: &str) -> ServiceResult<T> {
    serde_json::from_str(s).map_err(internal)
}

/// `runs/wiki/wiki.sqlite` under `IGNIS_RUNS` (default `runs`), next to the other run state.
pub fn default_path() -> PathBuf {
    std::env::var_os("IGNIS_RUNS")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("runs"))
        .join("wiki")
        .join("wiki.sqlite")
}

/// The wiki of every workspace in one store (rows are keyed by the scope's workspace).
pub struct WikiStore {
    con: Mutex<Connection>,
    origin: Origin,
}

/// One stored revision row (everything but the content blob).
struct Row {
    parent: Option<u64>,
    digest: String,
    ops: String,
    author: String,
    episode: Option<String>,
    scope: String,
    writer: String,
    created_at: String,
}

impl WikiStore {
    /// Open (or create) the store at `path`. `origin` marks what it holds: `system` for a
    /// deployment, `synthetic_fixture` for tests and the dev fixture.
    pub fn open(path: &Path, origin: Origin) -> anyhow::Result<WikiStore> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let con = Connection::open(path)?;
        con.pragma_update(None, "journal_mode", "WAL")?;
        con.pragma_update(None, "foreign_keys", true)?;
        con.busy_timeout(std::time::Duration::from_secs(5))?;
        con.execute_batch(SCHEMA)?;
        Ok(WikiStore {
            con: Mutex::new(con),
            origin,
        })
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.con.lock().unwrap_or_else(|p| p.into_inner())
    }

    /// Apply `patch` as the next revision of its page. `author` and `episode` come from the host;
    /// `cites` resolves an evidence id under `scope` and refuses what the scope cannot read.
    pub fn patch(
        &self,
        scope: &ExecutionScope,
        author: &Author,
        episode: Option<&EpisodeId>,
        patch: &WikiPatch,
        cites: &dyn Fn(&EvidenceId) -> ServiceResult<()>,
    ) -> ServiceResult<WikiRevision> {
        scope
            .validate()
            .map_err(|e| err(ErrorCode::ScopeDenied, e.to_string()))?;
        scope.require(Capability::WikiWrite)?;
        patch
            .validate()
            .map_err(|e| err(ErrorCode::InvalidRequest, e.to_string()))?;
        // A writer cites only what its own scope may read (a held-out passage, a forged id).
        for op in &patch.ops {
            if let WikiOp::Add { item } | WikiOp::Revise { item } = op {
                for r in &item.refs {
                    if let GraphNode::Entity(EntityRef::Evidence(id)) = r {
                        cites(id).map_err(|e| {
                            err(
                                ErrorCode::ScopeDenied,
                                format!(
                                    "item {} cites evidence {id} this scope cannot resolve: {}",
                                    item.item_id, e.message
                                ),
                            )
                        })?;
                    }
                }
            }
        }

        let ws = scope.workspace_id.as_str();
        let page = patch.page_id.as_str();
        let mut con = self.lock();
        // IMMEDIATE: the head check and the insert are one compare-and-swap, across processes too.
        let tx = con
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(internal)?;
        let head: Option<(u64, String)> = tx
            .query_row(
                "SELECT head_revision, title FROM wiki_pages WHERE workspace_id = ?1 AND page_id = ?2",
                params![ws, page],
                |r| Ok((r.get::<_, i64>(0)? as u64, r.get(1)?)),
            )
            .optional()
            .map_err(internal)?;
        let (parent, base) = match (patch.base_revision, head) {
            (None, None) => (None, Content {
                title: String::new(),
                items: Vec::new(),
            }),
            (None, Some((h, _))) => {
                return Err(conflict(format!(
                    "page {page} already exists at revision {h}: read it with wiki_read and patch against it"
                )))
            }
            (Some(b), None) => {
                return Err(err(
                    ErrorCode::NotFound,
                    format!("no wiki page {page} (revision {b} was given): create it without base_revision"),
                ))
            }
            (Some(b), Some((h, _))) if b != h => {
                return Err(conflict(format!(
                    "page {page} moved from revision {b} to {h}: read it with wiki_read and reapply the ops"
                )))
            }
            (Some(b), Some(_)) => {
                let digest: String = tx
                    .query_row(
                        "SELECT content_digest FROM wiki_revisions
                         WHERE workspace_id = ?1 AND page_id = ?2 AND revision = ?3",
                        params![ws, page, b as i64],
                        |r| r.get(0),
                    )
                    .map_err(internal)?;
                (Some(b), load_blob(&tx, &digest)?)
            }
        };
        let revision = parent.map_or(1, |p| p + 1);
        let items = apply(&base.items, &patch.ops, scope, revision)?;
        let title = patch.title.clone().unwrap_or(base.title);
        let content_digest = WikiRevision::digest_of(&title, &items);
        let rev = WikiRevision {
            schema_version: EvidenceSchema::V2,
            page_id: patch.page_id.clone(),
            revision,
            parent_revision: parent,
            title,
            items,
            ops: patch.ops.clone(),
            content_digest,
            author: author.clone(),
            episode_id: episode.cloned(),
            scope_id: Some(scope.scope_id.clone()),
            created_at: Timestamp::now(),
            origin: self.origin,
        };
        rev.validate().map_err(internal)?;

        let body = to_json(&Content {
            title: rev.title.clone(),
            items: rev.items.clone(),
        })?;
        tx.execute(
            "INSERT OR IGNORE INTO wiki_blobs (digest, body) VALUES (?1, ?2)",
            params![rev.content_digest.to_string(), body],
        )
        .map_err(internal)?;
        tx.execute(
            "INSERT INTO wiki_revisions (workspace_id, page_id, revision, parent_revision,
               content_digest, ops, author, episode_id, scope_id, writer_partitions, created_at, origin)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                ws,
                page,
                revision as i64,
                parent.map(|p| p as i64),
                rev.content_digest.to_string(),
                to_json(&rev.ops)?,
                to_json(&rev.author)?,
                episode.map(|e| e.as_str().to_string()),
                scope.scope_id.as_str(),
                to_json(&scope.partitions)?,
                rev.created_at.as_str(),
                to_json(&rev.origin)?,
            ],
        )
        .map_err(internal)?;
        tx.execute(
            "INSERT INTO wiki_pages (workspace_id, page_id, title, head_revision, item_count, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT (workspace_id, page_id) DO UPDATE SET
               title = excluded.title, head_revision = excluded.head_revision,
               item_count = excluded.item_count, updated_at = excluded.updated_at",
            params![
                ws,
                page,
                rev.title,
                revision as i64,
                rev.items.len() as i64,
                rev.created_at.as_str()
            ],
        )
        .map_err(internal)?;
        tx.commit().map_err(internal)?;
        Ok(rev)
    }

    /// One page at its head or a given revision, or the page index (paged by page id).
    pub fn read(
        &self,
        scope: &ExecutionScope,
        req: &WikiReadRequest,
    ) -> ServiceResult<WikiReadResult> {
        scope
            .validate()
            .map_err(|e| err(ErrorCode::ScopeDenied, e.to_string()))?;
        scope.require(Capability::ReadSources)?;
        req.validate()
            .map_err(|e| err(ErrorCode::InvalidRequest, e.to_string()))?;
        let ws = scope.workspace_id.as_str();
        let con = self.lock();
        let Some(page) = &req.page_id else {
            return self.index(&con, ws, req);
        };
        let revision = match req.revision {
            Some(r) => r,
            None => con
                .query_row(
                    "SELECT head_revision FROM wiki_pages WHERE workspace_id = ?1 AND page_id = ?2",
                    params![ws, page.as_str()],
                    |r| r.get::<_, i64>(0),
                )
                .optional()
                .map_err(internal)?
                .ok_or_else(|| err(ErrorCode::NotFound, format!("no wiki page {page}")))?
                as u64,
        };
        let row = con
            .query_row(
                "SELECT parent_revision, content_digest, ops, author, episode_id, scope_id,
                        writer_partitions, created_at
                 FROM wiki_revisions WHERE workspace_id = ?1 AND page_id = ?2 AND revision = ?3",
                params![ws, page.as_str(), revision as i64],
                |r| {
                    Ok(Row {
                        parent: r.get::<_, Option<i64>>(0)?.map(|p| p as u64),
                        digest: r.get(1)?,
                        ops: r.get(2)?,
                        author: r.get(3)?,
                        episode: r.get(4)?,
                        scope: r.get(5)?,
                        writer: r.get(6)?,
                        created_at: r.get(7)?,
                    })
                },
            )
            .optional()
            .map_err(internal)?
            .ok_or_else(|| {
                err(
                    ErrorCode::NotFound,
                    format!("wiki page {page} has no revision {revision}"),
                )
            })?;
        let content = load_blob(&con, &row.digest)?;
        drop(con);

        let readable = |needs: &BTreeSet<Partition>| needs.is_subset(&scope.partitions);
        let total = content.items.len();
        let items: Vec<WikiEntry> = content
            .items
            .into_iter()
            .filter(|e| readable(&e.requires_partitions))
            .collect();
        let withheld = (total - items.len()) as u64;
        // The delta of a revision is the writer's text: shown only to a scope that covers it.
        let writer: BTreeSet<Partition> = from_json(&row.writer)?;
        let ops: Vec<WikiOp> = if readable(&writer) {
            from_json(&row.ops)?
        } else {
            Vec::new()
        };
        // A redacted view is addressed by what it shows (`withheld_items` says it is a view); the
        // stored revision keeps its own digest, verified in `load_blob`.
        let content_digest = if withheld == 0 {
            Digest::parse(&row.digest).map_err(internal)?
        } else {
            WikiRevision::digest_of(&content.title, &items)
        };
        let rev = WikiRevision {
            schema_version: EvidenceSchema::V2,
            page_id: page.clone(),
            revision,
            parent_revision: row.parent,
            title: content.title,
            items,
            ops,
            content_digest,
            author: from_json(&row.author)?,
            episode_id: row
                .episode
                .map(EpisodeId::new)
                .transpose()
                .map_err(internal)?,
            scope_id: Some(ScopeId::new(row.scope).map_err(internal)?),
            created_at: Timestamp::parse(&row.created_at).map_err(internal)?,
            origin: self.origin,
        };
        let out = WikiReadResult {
            schema_version: EvidenceSchema::V2,
            body: WikiReadBody::Page {
                revision: Box::new(rev),
            },
            withheld_items: (withheld > 0).then_some(withheld),
            origin: self.origin,
        };
        out.validate().map_err(internal)?;
        Ok(out)
    }

    fn index(
        &self,
        con: &Connection,
        ws: &str,
        req: &WikiReadRequest,
    ) -> ServiceResult<WikiReadResult> {
        let after = match &req.cursor {
            None => String::new(),
            Some(c) => decode_cursor(c)?,
        };
        let size = req.page_size.unwrap_or(DEFAULT_INDEX_PAGE) as i64;
        let mut st = con
            .prepare(
                "SELECT page_id, title, head_revision, item_count, updated_at FROM wiki_pages
                 WHERE workspace_id = ?1 AND page_id > ?2 ORDER BY page_id LIMIT ?3",
            )
            .map_err(internal)?;
        let rows = st
            .query_map(params![ws, after, size + 1], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, i64>(2)?,
                    r.get::<_, i64>(3)?,
                    r.get::<_, String>(4)?,
                ))
            })
            .map_err(internal)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(internal)?;
        let more = rows.len() as i64 > size;
        let pages = rows
            .into_iter()
            .take(size as usize)
            .map(|(id, title, head, n, at)| {
                Ok(WikiPageSummary {
                    page_id: WikiPageId::new(id).map_err(internal)?,
                    title,
                    head_revision: head as u64,
                    item_count: n as u64,
                    updated_at: Timestamp::parse(&at).map_err(internal)?,
                })
            })
            .collect::<ServiceResult<Vec<_>>>()?;
        let next_cursor = match (more, pages.last()) {
            (true, Some(last)) => Some(encode_cursor(last.page_id.as_str())),
            _ => None,
        };
        Ok(WikiReadResult {
            schema_version: EvidenceSchema::V2,
            body: WikiReadBody::Index { pages, next_cursor },
            withheld_items: None,
            origin: self.origin,
        })
    }
}

fn conflict(msg: String) -> ErrorEnvelope {
    // Re-read, then retry: the head moved under the caller.
    ErrorEnvelope::new(ErrorCode::Conflict, msg, RetryPolicy::Never)
}

fn encode_cursor(page_id: &str) -> PageCursor {
    PageCursor::new(format!("wiki1.{}", hex::encode(page_id))).expect("hex has no spaces")
}

fn decode_cursor(c: &PageCursor) -> ServiceResult<String> {
    c.as_str()
        .strip_prefix("wiki1.")
        .and_then(|h| hex::decode(h).ok())
        .and_then(|b| String::from_utf8(b).ok())
        .ok_or_else(|| {
            err(
                ErrorCode::InvalidRequest,
                format!("{c} is not a wiki index cursor"),
            )
        })
}

/// The blob of `digest`, verified against it (a corrupted or edited row is an error, not content).
fn load_blob(con: &Connection, digest: &str) -> ServiceResult<Content> {
    let body: String = con
        .query_row(
            "SELECT body FROM wiki_blobs WHERE digest = ?1",
            params![digest],
            |r| r.get(0),
        )
        .map_err(internal)?;
    let content: Content = from_json(&body)?;
    let actual = WikiRevision::digest_of(&content.title, &content.items);
    if actual.to_string() != digest {
        return Err(internal(format!(
            "blob {digest} does not hash to its address (got {actual})"
        )));
    }
    Ok(content)
}

/// The items after `ops`. Every op names one item: an add of an existing id, a revise or remove
/// of a missing one, and a patch that would replace every item of a page are refused.
fn apply(
    base: &[WikiEntry],
    ops: &[WikiOp],
    scope: &ExecutionScope,
    revision: u64,
) -> ServiceResult<Vec<WikiEntry>> {
    let has = |id: &WikiItemId| base.iter().any(|e| &e.item_id == id);
    let touched_existing = ops
        .iter()
        .filter(|o| matches!(o, WikiOp::Revise { .. } | WikiOp::Remove { .. }))
        .count();
    if base.len() >= 2 && touched_existing == base.len() {
        return Err(err(
            ErrorCode::InvalidRequest,
            format!(
                "this patch revises or removes all {} items of the page: that is a whole-page \
                 rewrite; change the items that need it, one delta at a time",
                base.len()
            ),
        ));
    }
    let entry = |item: &ignis_core::contracts::WikiItem| WikiEntry {
        item_id: item.item_id.clone(),
        kind: item.kind,
        text: item.text.clone(),
        refs: item.refs.clone(),
        // From the writing scope: summarizing does not launder a restriction.
        requires_partitions: scope.partitions.clone(),
        written_in: Some(revision),
    };
    let mut items = base.to_vec();
    for op in ops {
        // An item this scope may not read is not this scope's to change (it cannot see it).
        let target = match op {
            WikiOp::Revise { item } => Some(&item.item_id),
            WikiOp::Remove { item_id, .. } => Some(item_id),
            WikiOp::Add { .. } => None,
        };
        if let Some(e) = target.and_then(|id| base.iter().find(|e| &e.item_id == id)) {
            if !e.requires_partitions.is_subset(&scope.partitions) {
                return Err(err(
                    ErrorCode::ScopeDenied,
                    format!("item {} is outside this scope's partitions", e.item_id),
                ));
            }
        }
        match op {
            WikiOp::Add { item } => {
                if has(&item.item_id) {
                    return Err(err(
                        ErrorCode::InvalidRequest,
                        format!("item {} already exists: revise it instead", item.item_id),
                    ));
                }
                items.push(entry(item));
            }
            WikiOp::Revise { item } => {
                let Some(slot) = items.iter_mut().find(|e| e.item_id == item.item_id) else {
                    return Err(err(
                        ErrorCode::InvalidRequest,
                        format!("no item {} to revise", item.item_id),
                    ));
                };
                *slot = entry(item);
            }
            WikiOp::Remove { item_id, .. } => {
                let before = items.len();
                items.retain(|e| &e.item_id != item_id);
                if items.len() == before {
                    return Err(err(
                        ErrorCode::InvalidRequest,
                        format!("no item {item_id} to remove"),
                    ));
                }
            }
        }
    }
    Ok(items)
}
