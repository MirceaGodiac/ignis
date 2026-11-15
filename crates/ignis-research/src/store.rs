//! SQLite state store: episodes, event logs, idempotency keys and durable jobs.
//!
//! One writer connection (behind a mutex) serializes every write in short transactions; readers use
//! their own connections, which WAL lets run concurrently with the writer. The event `seq` is
//! allocated inside the append transaction, so a committed log is always gap-free; notification of
//! listeners happens after commit, in the host (outbox after commit). The database is the replay
//! authority: a missed notification only delays a reader, it never loses an event.
//!
//! Jobs: a runner claims a queued job in one transaction (status `running`, a lease and a new fencing
//! token). Heartbeats extend the lease; finishing requires the current token, so a runner whose lease
//! expired cannot overwrite the outcome of a newer attempt. An expired lease of an idempotent job
//! re-queues it; for a non-idempotent job the effect is uncertain and it goes to `reconciling`
//! (never re-run blindly).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Duration;

use ignis_core::contracts::{
    check_event_log, ArtifactRef, Causation, ClientRequestId, EpisodeId, EpisodePins,
    EpisodeStatus, ErrorCode, ErrorEnvelope, EventBody, EventId, EvidenceId, EvidenceRef,
    ExecutionScope, HarnessRef, JobId, JobProgress, JobRef, JobStatus, MessageId, Origin,
    ResearchEpisode, ResearchEvent, ResearchSchema, RetryPolicy, RuntimeThreadRef, Timestamp,
};
use ignis_core::hashing::sha256_hex;
use ignis_core::services::ServiceResult;
use rusqlite::{params, Connection, OptionalExtension, Transaction};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;

use crate::{conflict, internal, new_id, not_found, now_ms};

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS episodes (
    episode_id   TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL,
    principal    TEXT NOT NULL,
    record       TEXT NOT NULL,
    -- lowest seq still stored (events below it were compacted away; replay before it is 410 Gone)
    log_floor    INTEGER NOT NULL DEFAULT 1
);
CREATE INDEX IF NOT EXISTS episodes_ws ON episodes (workspace_id);
CREATE TABLE IF NOT EXISTS events (
    episode_id TEXT NOT NULL REFERENCES episodes (episode_id),
    seq        INTEGER NOT NULL,
    event_id   TEXT NOT NULL UNIQUE,
    kind       TEXT NOT NULL,
    record     TEXT NOT NULL,
    PRIMARY KEY (episode_id, seq)
) WITHOUT ROWID;
CREATE TABLE IF NOT EXISTS requests (
    workspace_id      TEXT NOT NULL,
    client_request_id TEXT NOT NULL,
    operation         TEXT NOT NULL,
    fingerprint       TEXT NOT NULL,
    response          TEXT NOT NULL,
    PRIMARY KEY (workspace_id, client_request_id)
) WITHOUT ROWID;
CREATE TABLE IF NOT EXISTS jobs (
    job_id           TEXT PRIMARY KEY,
    workspace_id     TEXT NOT NULL,
    episode_id       TEXT,
    kind             TEXT NOT NULL,
    status           TEXT NOT NULL,
    record           TEXT NOT NULL,
    input            TEXT NOT NULL,
    scope            TEXT NOT NULL,
    idempotent       INTEGER NOT NULL,
    lease_owner      TEXT,
    lease_expires_ms INTEGER,
    fence            INTEGER NOT NULL DEFAULT 0,
    attempts         INTEGER NOT NULL DEFAULT 0,
    queued_ms        INTEGER NOT NULL,
    -- JobPriority: 0 interactive, 1 evaluation, 2 ingestion (claimed in this order)
    priority         INTEGER NOT NULL DEFAULT 1
);
CREATE INDEX IF NOT EXISTS jobs_queue ON jobs (status, queued_ms);
CREATE TABLE IF NOT EXISTS uploads (
    upload_id    TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL,
    status       TEXT NOT NULL,
    spec         TEXT NOT NULL,
    artifact     TEXT,
    created_ms   INTEGER NOT NULL,
    expires_ms   INTEGER NOT NULL
);
-- Agent-runtime turns (M1). One row per turn IGNIS asked the runtime for; the intent is written
-- before `turn/start` is sent, so a crash leaves a visible, reconcilable record.
CREATE TABLE IF NOT EXISTS runtime_turns (
    turn_key        TEXT PRIMARY KEY,
    episode_id      TEXT NOT NULL REFERENCES episodes (episode_id),
    workspace_id    TEXT NOT NULL,
    -- host-derived scope at turn start (recovery after a restart has no caller)
    scope           TEXT NOT NULL,
    runtime_turn_id TEXT,
    -- user messages with seq <= through_seq were handed to this turn (never re-sent)
    through_seq     INTEGER NOT NULL,
    -- starting | running | finished
    state           TEXT NOT NULL,
    outcome         TEXT,
    created_ms      INTEGER NOT NULL
);
-- At most one unfinished turn per episode, whatever process asks.
CREATE UNIQUE INDEX IF NOT EXISTS runtime_turns_active ON runtime_turns (episode_id)
    WHERE state IN ('starting', 'running');
-- Evidence a retrieval tool returned in an episode (M7): `answer_submit` resolves the answer's
-- citations against exactly this set, so an answer cannot cite what its episode never saw.
CREATE TABLE IF NOT EXISTS episode_evidence (
    episode_id  TEXT NOT NULL REFERENCES episodes (episode_id),
    evidence_id TEXT NOT NULL,
    record      TEXT NOT NULL,
    PRIMARY KEY (episode_id, evidence_id)
) WITHOUT ROWID;
-- Runtime facts already recorded in an episode log (e.g. `item:<id>:completed`), so replaying a
-- runtime thread after a restart never appends the same event twice.
CREATE TABLE IF NOT EXISTS runtime_marks (
    episode_id TEXT NOT NULL,
    mark       TEXT NOT NULL,
    seq        INTEGER NOT NULL,
    PRIMARY KEY (episode_id, mark)
) WITHOUT ROWID;
"#;

/// 1 -> 2: `jobs.priority`, `uploads`. 2 -> 3: `runtime_turns`, `runtime_marks`.
/// 3 -> 4: `episode_evidence`.
const SCHEMA_VERSION: &str = "4";

/// Upgrade an older store in place (additive changes only; new tables come from `SCHEMA`).
fn migrate(conn: &Connection, from: &str) -> ServiceResult<()> {
    match from {
        "1" | "2" | "3" => {}
        _ => {
            return Err(internal(
                "state store",
                format!("schema version {from} is not supported (expected {SCHEMA_VERSION})"),
            ))
        }
    }
    if from == "1" {
        let has_priority = conn
            .prepare("SELECT 1 FROM pragma_table_info('jobs') WHERE name = 'priority'")
            .and_then(|mut st| st.exists([]))
            .map_err(db("migrate"))?;
        if !has_priority {
            conn.execute_batch("ALTER TABLE jobs ADD COLUMN priority INTEGER NOT NULL DEFAULT 1;")
                .map_err(db("migrate"))?;
        }
    }
    conn.execute(
        "UPDATE meta SET value = ?1 WHERE key = 'schema_version'",
        [SCHEMA_VERSION],
    )
    .map_err(db("migrate"))?;
    Ok(())
}

fn db(context: &'static str) -> impl FnOnce(rusqlite::Error) -> ErrorEnvelope {
    move |e| internal(context, e)
}

fn to_json<T: Serialize>(v: &T) -> ServiceResult<String> {
    serde_json::to_string(v).map_err(|e| internal("serialize", e))
}

fn from_json<T: DeserializeOwned>(s: &str) -> ServiceResult<T> {
    serde_json::from_str(s).map_err(|e| internal("stored record violates its contract", e))
}

/// The snake_case wire name of a unit enum (`JobStatus::CancelRequested` → `cancel_requested`).
fn wire<T: Serialize>(v: &T) -> String {
    match serde_json::to_value(v) {
        Ok(Value::String(s)) => s,
        _ => String::new(),
    }
}

fn fingerprint<T: Serialize>(v: &T) -> String {
    sha256_hex(serde_json::to_string(v).unwrap_or_default().as_bytes())
}

/// Status changes a host action may request (a contract: `POST /api/v1/episodes/{id}/{action}`).
/// The runtime bridge (M1) drives the others.
pub use ignis_core::contracts::EpisodeAction;

/// Result of a replay request.
#[derive(Debug, Clone, PartialEq)]
pub enum Replay {
    /// Events with `seq > after`, in order (possibly empty), and the episode's `last_seq`.
    Events {
        events: Vec<ResearchEvent>,
        last_seq: u64,
    },
    /// The log no longer covers the cursor (compacted, or the cursor is ahead of the log): the
    /// client reloads the episode and continues from `last_seq`.
    Gone { last_seq: u64 },
}

/// A job request. `input` is the decoded contract argument (e.g. a `ForecastRequest`).
#[derive(Debug, Clone)]
pub struct NewJob {
    pub kind: String,
    pub input: Value,
    pub episode_id: Option<EpisodeId>,
    pub client_request_id: Option<ClientRequestId>,
    /// Re-running an interrupted attempt has no external effect (pure computation).
    pub idempotent: bool,
    pub priority: JobPriority,
    pub causation: Causation,
}

/// Scheduling class. Queued jobs are claimed interactive first, then requested evaluations, then
/// background ingestion; the runner also caps how many of the lower classes run at once.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum JobPriority {
    Interactive = 0,
    Evaluation = 1,
    Ingestion = 2,
}

/// What a client declared when starting an upload; the bytes must match it exactly.
#[derive(Debug, Clone, PartialEq, Serialize, serde::Deserialize)]
pub struct UploadSpec {
    pub media_type: String,
    pub byte_length: u64,
    pub sha256: ignis_core::contracts::Digest,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contract: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UploadStatus {
    /// Waiting for the bytes.
    Initiated,
    /// Bytes received and verified against the spec (length and sha256).
    Received,
    /// Published as an artifact.
    Completed,
}

#[derive(Debug, Clone)]
pub struct UploadRecord {
    pub upload_id: String,
    pub status: UploadStatus,
    pub spec: UploadSpec,
    pub artifact: Option<ArtifactRef>,
    pub expires_ms: i64,
}

/// A job handed to a runner. The runner must present `fence` on every heartbeat and on finish.
#[derive(Debug, Clone)]
pub struct ClaimedJob {
    pub job_id: JobId,
    pub fence: i64,
    pub kind: String,
    pub input: Value,
    /// The scope the job was submitted under (host-derived, persisted with the job).
    pub scope: ExecutionScope,
    pub episode_id: Option<EpisodeId>,
    pub attempt: i64,
}

/// How a runner ends an attempt.
#[derive(Debug, Clone)]
pub enum JobOutcome {
    Succeeded { outputs: Vec<ArtifactRef> },
    Failed { error: ErrorEnvelope },
    Cancelled,
}

pub struct ResearchStore {
    path: PathBuf,
    writer: Mutex<Connection>,
    readers: Mutex<Vec<Connection>>,
}

impl ResearchStore {
    /// Open (and create or migrate) the store at `path`.
    pub fn open(path: impl AsRef<Path>) -> ServiceResult<Self> {
        let path = path.as_ref().to_path_buf();
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| internal("create state dir", e))?;
        }
        let conn = Self::connect(&path)?;
        conn.execute_batch(SCHEMA).map_err(db("schema"))?;
        let version: Option<String> = conn
            .query_row(
                "SELECT value FROM meta WHERE key = 'schema_version'",
                [],
                |r| r.get(0),
            )
            .optional()
            .map_err(db("schema version"))?;
        match version.as_deref() {
            None => {
                conn.execute(
                    "INSERT INTO meta (key, value) VALUES ('schema_version', ?1)",
                    [SCHEMA_VERSION],
                )
                .map_err(db("schema version"))?;
            }
            Some(SCHEMA_VERSION) => {}
            Some(other) => migrate(&conn, other)?,
        }
        conn.execute_batch(
            "CREATE INDEX IF NOT EXISTS jobs_claim ON jobs (status, priority, queued_ms);",
        )
        .map_err(db("schema"))?;
        Ok(ResearchStore {
            path,
            writer: Mutex::new(conn),
            readers: Mutex::new(Vec::new()),
        })
    }

    fn connect(path: &Path) -> ServiceResult<Connection> {
        let conn = Connection::open(path).map_err(db("open state store"))?;
        conn.busy_timeout(Duration::from_secs(10))
            .map_err(db("busy timeout"))?;
        conn.execute_batch(
            "PRAGMA journal_mode = WAL; PRAGMA synchronous = NORMAL; PRAGMA foreign_keys = ON;",
        )
        .map_err(db("pragmas"))?;
        Ok(conn)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn write<T>(&self, f: impl FnOnce(&Transaction) -> ServiceResult<T>) -> ServiceResult<T> {
        let mut conn = self.writer.lock().unwrap_or_else(|p| p.into_inner());
        let tx = conn
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
            .map_err(db("begin"))?;
        let out = f(&tx)?;
        tx.commit().map_err(db("commit"))?;
        Ok(out)
    }

    fn read<T>(&self, f: impl FnOnce(&Connection) -> ServiceResult<T>) -> ServiceResult<T> {
        let conn = match self.readers.lock().unwrap_or_else(|p| p.into_inner()).pop() {
            Some(c) => c,
            None => Self::connect(&self.path)?,
        };
        let out = f(&conn);
        let mut pool = self.readers.lock().unwrap_or_else(|p| p.into_inner());
        if pool.len() < 8 {
            pool.push(conn);
        }
        out
    }

    // ------------------------------------------------------------------------ episodes

    /// Create an episode in the scope's workspace. Idempotent on `client_request_id`: the same key
    /// with the same goal returns the existing episode (`created = false`); with a different goal it
    /// is a conflict.
    pub fn create_episode(
        &self,
        scope: &ExecutionScope,
        goal: &str,
        client_request_id: &ClientRequestId,
        pins: EpisodePins,
        origin: Origin,
    ) -> ServiceResult<(ResearchEpisode, bool)> {
        if goal.trim().is_empty() {
            return Err(ErrorEnvelope::new(
                ErrorCode::InvalidRequest,
                "goal must not be empty",
                RetryPolicy::Never,
            ));
        }
        let fp = fingerprint(&(goal, &pins));
        self.write(|tx| {
            if let Some(prev) = prior_request(tx, scope, client_request_id, "create_episode", &fp)?
            {
                let id = EpisodeId::new(prev).map_err(|e| internal("stored id", e))?;
                return Ok((load_episode(tx, scope, &id)?, false));
            }
            let now = Timestamp::now();
            let ep = ResearchEpisode {
                schema_version: ResearchSchema::V1,
                episode_id: EpisodeId::new(new_id("ep")).expect("generated id"),
                workspace_id: scope.workspace_id.clone(),
                goal: goal.to_string(),
                status: EpisodeStatus::Ready,
                runtime: None,
                parent: None,
                pins,
                scope_id: scope.scope_id.clone(),
                revision: 1,
                last_seq: 0,
                created_at: now.clone(),
                updated_at: now,
                origin,
            };
            tx.execute(
                "INSERT INTO episodes (episode_id, workspace_id, principal, record) VALUES (?1, ?2, ?3, ?4)",
                params![
                    ep.episode_id.as_str(),
                    scope.workspace_id.as_str(),
                    scope.principal.as_str(),
                    to_json(&ep)?
                ],
            )
            .map_err(db("insert episode"))?;
            record_request(
                tx,
                scope,
                client_request_id,
                "create_episode",
                &fp,
                ep.episode_id.as_str(),
            )?;
            Ok((ep, true))
        })
    }

    pub fn episode(
        &self,
        scope: &ExecutionScope,
        id: &EpisodeId,
    ) -> ServiceResult<ResearchEpisode> {
        self.read(|c| load_episode(c, scope, id))
    }

    /// Episodes of the scope's workspace, newest first.
    pub fn episodes(
        &self,
        scope: &ExecutionScope,
        limit: usize,
    ) -> ServiceResult<Vec<ResearchEpisode>> {
        self.read(|c| {
            let mut st = c
                .prepare(
                    "SELECT record FROM episodes WHERE workspace_id = ?1 ORDER BY rowid DESC LIMIT ?2",
                )
                .map_err(db("list episodes"))?;
            let rows = st
                .query_map(params![scope.workspace_id.as_str(), limit as i64], |r| {
                    r.get::<_, String>(0)
                })
                .map_err(db("list episodes"))?;
            rows.map(|r| r.map_err(db("list episodes")).and_then(|s| from_json(&s)))
                .collect()
        })
    }

    /// Append events to an episode in one transaction (the internal entry point of the runtime
    /// bridge and of the host). Status changes must start from the current status.
    pub fn append(
        &self,
        scope: &ExecutionScope,
        id: &EpisodeId,
        events: Vec<(Causation, EventBody)>,
    ) -> ServiceResult<Vec<ResearchEvent>> {
        self.write(|tx| {
            let mut ep = load_episode(tx, scope, id)?;
            append_tx(tx, &mut ep, events)
        })
    }

    /// Record a user message. Idempotent on `client_request_id` (a retried POST returns the event
    /// that was appended the first time and appends nothing).
    pub fn post_message(
        &self,
        scope: &ExecutionScope,
        id: &EpisodeId,
        text: &str,
        client_request_id: &ClientRequestId,
    ) -> ServiceResult<(ResearchEvent, bool)> {
        if text.trim().is_empty() {
            return Err(ErrorEnvelope::new(
                ErrorCode::InvalidRequest,
                "message text must not be empty",
                RetryPolicy::Never,
            ));
        }
        let op = format!("message:{id}");
        let fp = fingerprint(&text);
        self.write(|tx| {
            let mut ep = load_episode(tx, scope, id)?;
            if let Some(prev) = prior_request(tx, scope, client_request_id, &op, &fp)? {
                return Ok((load_event(tx, &prev)?, false));
            }
            if matches!(ep.status, EpisodeStatus::Cancelled | EpisodeStatus::Failed) {
                return Err(conflict(format!(
                    "episode {id} is {}; start a new episode",
                    wire(&ep.status)
                )));
            }
            let causation = Causation {
                client_request_id: Some(client_request_id.clone()),
                ..Default::default()
            };
            let body = EventBody::UserMessage {
                message_id: MessageId::new(new_id("msg")).expect("generated id"),
                text: text.to_string(),
            };
            let ev = append_tx(tx, &mut ep, vec![(causation, body)])?.remove(0);
            record_request(tx, scope, client_request_id, &op, &fp, ev.event_id.as_str())?;
            Ok((ev, true))
        })
    }

    /// Apply a host action. Repeating an action that already holds is a no-op; an action that is
    /// not allowed from the current status is a conflict. Cancelling an episode also cancels its
    /// queued jobs and asks running ones to stop.
    pub fn act(
        &self,
        scope: &ExecutionScope,
        id: &EpisodeId,
        action: EpisodeAction,
    ) -> ServiceResult<ResearchEpisode> {
        use EpisodeStatus as S;
        self.write(|tx| {
            let mut ep = load_episode(tx, scope, id)?;
            let from = ep.status;
            let to = match (action, from) {
                (EpisodeAction::Pause, S::Paused)
                | (EpisodeAction::Resume, S::Ready)
                | (EpisodeAction::Cancel, S::Cancelled) => return Ok(ep),
                (EpisodeAction::Pause, S::Ready | S::Running | S::WaitingTools | S::WaitingApproval | S::WaitingUser) => S::Paused,
                (EpisodeAction::Resume, S::Paused) => S::Ready,
                (EpisodeAction::Cancel, s) if s != S::Failed => S::Cancelled,
                (a, s) => {
                    return Err(conflict(format!(
                        "cannot {a:?} an episode that is {}",
                        wire(&s)
                    )
                    .to_lowercase()))
                }
            };
            let reason = format!("{action:?} requested by {}", scope.principal).to_lowercase();
            append_tx(
                tx,
                &mut ep,
                vec![(
                    Causation::default(),
                    EventBody::StatusChanged {
                        from,
                        to,
                        reason: Some(reason),
                    },
                )],
            )?;
            if to == S::Cancelled {
                let ids: Vec<String> = {
                    let mut st = tx
                        .prepare(
                            "SELECT job_id FROM jobs WHERE episode_id = ?1 AND status IN ('queued', 'running', 'reconciling')",
                        )
                        .map_err(db("episode jobs"))?;
                    let rows = st
                        .query_map([id.as_str()], |r| r.get::<_, String>(0))
                        .map_err(db("episode jobs"))?;
                    rows.collect::<Result<_, _>>().map_err(db("episode jobs"))?
                };
                for j in ids {
                    cancel_job_tx(tx, &JobId::new(j).map_err(|e| internal("stored id", e))?)?;
                }
                ep = load_episode(tx, scope, id)?;
            }
            Ok(ep)
        })
    }

    /// Events after `after`, at most `limit`.
    pub fn replay(
        &self,
        scope: &ExecutionScope,
        id: &EpisodeId,
        after: u64,
        limit: usize,
    ) -> ServiceResult<Replay> {
        self.read(|c| {
            let ep = load_episode(c, scope, id)?;
            let floor: i64 = c
                .query_row(
                    "SELECT log_floor FROM episodes WHERE episode_id = ?1",
                    [id.as_str()],
                    |r| r.get(0),
                )
                .map_err(db("log floor"))?;
            if after > ep.last_seq || after + 1 < floor as u64 {
                return Ok(Replay::Gone {
                    last_seq: ep.last_seq,
                });
            }
            let mut st = c
                .prepare(
                    "SELECT record FROM events WHERE episode_id = ?1 AND seq > ?2 ORDER BY seq LIMIT ?3",
                )
                .map_err(db("replay"))?;
            let rows = st
                .query_map(params![id.as_str(), after as i64, limit as i64], |r| {
                    r.get::<_, String>(0)
                })
                .map_err(db("replay"))?;
            let events: Vec<ResearchEvent> = rows
                .map(|r| r.map_err(db("replay")).and_then(|s| from_json(&s)))
                .collect::<ServiceResult<_>>()?;
            check_event_log(&events).map_err(|e| internal("stored event log", e))?;
            Ok(Replay::Events {
                events,
                last_seq: ep.last_seq,
            })
        })
    }

    // ------------------------------------------------------------------------ answers (M7)

    /// Remember the evidence a tool call returned in episode `id` (idempotent; the first record of
    /// an id is kept).
    pub fn record_episode_evidence(
        &self,
        scope: &ExecutionScope,
        id: &EpisodeId,
        evidence: &[EvidenceRef],
    ) -> ServiceResult<()> {
        if evidence.is_empty() {
            return Ok(());
        }
        self.write(|tx| {
            load_episode(tx, scope, id)?;
            let mut st = tx
                .prepare_cached(
                    "INSERT OR IGNORE INTO episode_evidence (episode_id, evidence_id, record) VALUES (?1, ?2, ?3)",
                )
                .map_err(db("record evidence"))?;
            for e in evidence {
                st.execute(params![id.as_str(), e.evidence_id.as_str(), to_json(e)?])
                    .map_err(db("record evidence"))?;
            }
            Ok(())
        })
    }

    /// The evidence among `ids` that a tool returned in episode `id`; ids it never returned are
    /// absent from the map.
    pub fn episode_evidence<'a>(
        &self,
        scope: &ExecutionScope,
        id: &EpisodeId,
        ids: impl IntoIterator<Item = &'a EvidenceId>,
    ) -> ServiceResult<BTreeMap<EvidenceId, EvidenceRef>> {
        self.read(|c| {
            load_episode(c, scope, id)?;
            let mut st = c
                .prepare_cached(
                    "SELECT record FROM episode_evidence WHERE episode_id = ?1 AND evidence_id = ?2",
                )
                .map_err(db("episode evidence"))?;
            let mut out = BTreeMap::new();
            for e in ids {
                let rec: Option<String> = st
                    .query_row(params![id.as_str(), e.as_str()], |r| r.get(0))
                    .optional()
                    .map_err(db("episode evidence"))?;
                if let Some(rec) = rec {
                    out.insert(e.clone(), from_json(&rec)?);
                }
            }
            Ok(out)
        })
    }

    /// The `answer.completed` event an earlier submission with the same idempotency key appended,
    /// if any (a different submission under the same key is a conflict).
    pub fn prior_answer(
        &self,
        scope: &ExecutionScope,
        id: &EpisodeId,
        crid: &ClientRequestId,
        fingerprint: &str,
    ) -> ServiceResult<Option<ResearchEvent>> {
        let op = format!("answer:{id}");
        self.write(|tx| {
            load_episode(tx, scope, id)?;
            match prior_request(tx, scope, crid, &op, fingerprint)? {
                Some(ev) => load_event(tx, &ev).map(Some),
                None => Ok(None),
            }
        })
    }

    /// Append the `answer.completed` event of a validated answer. With an idempotency key, a
    /// concurrent duplicate returns the event appended first (`false`: nothing new was appended).
    pub fn complete_answer(
        &self,
        scope: &ExecutionScope,
        id: &EpisodeId,
        idempotency: Option<(&ClientRequestId, &str)>,
        causation: Causation,
        body: EventBody,
    ) -> ServiceResult<(ResearchEvent, bool)> {
        if !matches!(body, EventBody::AnswerCompleted { .. }) {
            return Err(internal("complete_answer", "not an answer.completed event"));
        }
        let op = format!("answer:{id}");
        self.write(|tx| {
            let mut ep = load_episode(tx, scope, id)?;
            if let Some((crid, fp)) = idempotency {
                if let Some(prev) = prior_request(tx, scope, crid, &op, fp)? {
                    return Ok((load_event(tx, &prev)?, false));
                }
            }
            if matches!(ep.status, EpisodeStatus::Cancelled | EpisodeStatus::Failed) {
                return Err(conflict(format!(
                    "episode {id} is {}; its answer can no longer change",
                    wire(&ep.status)
                )));
            }
            let ev = append_tx(tx, &mut ep, vec![(causation, body)])?.remove(0);
            if let Some((crid, fp)) = idempotency {
                record_request(tx, scope, crid, &op, fp, ev.event_id.as_str())?;
            }
            Ok((ev, true))
        })
    }

    // ------------------------------------------------------------------------ jobs

    /// Queue a job. Idempotent on `client_request_id` (same key + same kind/input → same job).
    pub fn submit_job(&self, scope: &ExecutionScope, job: NewJob) -> ServiceResult<JobRef> {
        let fp = fingerprint(&(&job.kind, &job.input, &job.episode_id));
        self.write(|tx| {
            if let Some(crid) = &job.client_request_id {
                if let Some(prev) = prior_request(tx, scope, crid, "submit_job", &fp)? {
                    let id = JobId::new(prev).map_err(|e| internal("stored id", e))?;
                    return load_job(tx, Some(scope), &id).map(|(j, _)| j);
                }
            }
            let mut ep = match &job.episode_id {
                Some(e) => Some(load_episode(tx, scope, e)?),
                None => None,
            };
            let now = Timestamp::now();
            let r = JobRef {
                job_id: JobId::new(new_id("job")).expect("generated id"),
                kind: job.kind.clone(),
                status: JobStatus::Queued,
                progress: None,
                outputs: vec![],
                error: None,
                submitted_at: now.clone(),
                updated_at: now,
            };
            r.validate().map_err(|e| internal("job", e))?;
            tx.execute(
                "INSERT INTO jobs (job_id, workspace_id, episode_id, kind, status, record, input, scope, idempotent, queued_ms, priority)
                 VALUES (?1, ?2, ?3, ?4, 'queued', ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    r.job_id.as_str(),
                    scope.workspace_id.as_str(),
                    job.episode_id.as_ref().map(EpisodeId::as_str),
                    job.kind,
                    to_json(&r)?,
                    to_json(&job.input)?,
                    to_json(scope)?,
                    job.idempotent,
                    now_ms(),
                    job.priority as i64
                ],
            )
            .map_err(db("insert job"))?;
            if let Some(crid) = &job.client_request_id {
                record_request(tx, scope, crid, "submit_job", &fp, r.job_id.as_str())?;
            }
            if let Some(ep) = ep.as_mut() {
                let causation = Causation {
                    job_id: Some(r.job_id.clone()),
                    ..job.causation.clone()
                };
                append_tx(tx, ep, vec![(causation, EventBody::JobUpdated { job: r.clone() })])?;
            }
            Ok(r)
        })
    }

    pub fn job(&self, scope: &ExecutionScope, id: &JobId) -> ServiceResult<JobRef> {
        self.read(|c| load_job(c, Some(scope), id).map(|(j, _)| j))
    }

    /// Ask a job to stop. A queued job is cancelled at once; a running one becomes
    /// `cancel_requested` and its runner sees it on the next heartbeat.
    pub fn cancel_job(&self, scope: &ExecutionScope, id: &JobId) -> ServiceResult<JobRef> {
        self.write(|tx| {
            load_job(tx, Some(scope), id)?;
            cancel_job_tx(tx, id)
        })
    }

    /// Claim the oldest queued job of one of `kinds` for `lease`. Returns `None` when nothing is
    /// queued.
    pub fn claim_job(
        &self,
        owner: &str,
        kinds: &[&str],
        lease: Duration,
    ) -> ServiceResult<Option<ClaimedJob>> {
        if kinds.is_empty() {
            return Ok(None);
        }
        self.write(|tx| {
            let marks = vec!["?"; kinds.len()].join(",");
            let sql = format!(
                "SELECT job_id FROM jobs WHERE status = 'queued' AND kind IN ({marks}) ORDER BY priority, queued_ms, rowid LIMIT 1"
            );
            let id: Option<String> = tx
                .query_row(&sql, rusqlite::params_from_iter(kinds.iter()), |r| r.get(0))
                .optional()
                .map_err(db("claim"))?;
            let Some(id) = id else { return Ok(None) };
            let id = JobId::new(id).map_err(|e| internal("stored id", e))?;
            tx.execute(
                "UPDATE jobs SET lease_owner = ?2, lease_expires_ms = ?3, fence = fence + 1, attempts = attempts + 1
                 WHERE job_id = ?1",
                params![id.as_str(), owner, now_ms() + lease.as_millis() as i64],
            )
            .map_err(db("claim"))?;
            let (fence, attempt, kind, input, scope, episode): (i64, i64, String, String, String, Option<String>) = tx
                .query_row(
                    "SELECT fence, attempts, kind, input, scope, episode_id FROM jobs WHERE job_id = ?1",
                    [id.as_str()],
                    |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?)),
                )
                .map_err(db("claim"))?;
            set_job_status(tx, &id, JobStatus::Running, None, None, None)?;
            Ok(Some(ClaimedJob {
                job_id: id,
                fence,
                kind,
                input: from_json(&input)?,
                scope: from_json(&scope)?,
                episode_id: episode
                    .map(EpisodeId::new)
                    .transpose()
                    .map_err(|e| internal("stored id", e))?,
                attempt,
            }))
        })
    }

    /// Extend the lease of the attempt holding `fence`, optionally reporting progress. Returns
    /// whether cancellation was requested. A stale fence (the lease expired and the job moved on) is
    /// a conflict: the runner must stop without writing anything.
    pub fn heartbeat(
        &self,
        id: &JobId,
        fence: i64,
        lease: Duration,
        progress: Option<JobProgress>,
    ) -> ServiceResult<bool> {
        self.write(|tx| {
            let status = check_fence(tx, id, fence)?;
            tx.execute(
                "UPDATE jobs SET lease_expires_ms = ?2 WHERE job_id = ?1",
                params![id.as_str(), now_ms() + lease.as_millis() as i64],
            )
            .map_err(db("heartbeat"))?;
            if let Some(p) = progress {
                let (job, _) = load_job(tx, None, id)?;
                if job.progress.as_ref() != Some(&p) {
                    set_job_status(tx, id, status, Some(Some(p)), None, None)?;
                }
            }
            Ok(status == JobStatus::CancelRequested)
        })
    }

    /// End the attempt holding `fence`.
    pub fn finish_job(&self, id: &JobId, fence: i64, outcome: JobOutcome) -> ServiceResult<JobRef> {
        self.write(|tx| {
            check_fence(tx, id, fence)?;
            let (status, outputs, error) = match outcome {
                JobOutcome::Succeeded { outputs } => (JobStatus::Succeeded, Some(outputs), None),
                JobOutcome::Failed { error } => (JobStatus::Failed, None, Some(error)),
                JobOutcome::Cancelled => (
                    JobStatus::Cancelled,
                    None,
                    Some(ErrorEnvelope::new(
                        ErrorCode::Cancelled,
                        "cancelled on request",
                        RetryPolicy::Never,
                    )),
                ),
            };
            tx.execute(
                "UPDATE jobs SET lease_owner = NULL, lease_expires_ms = NULL WHERE job_id = ?1",
                [id.as_str()],
            )
            .map_err(db("finish"))?;
            set_job_status(tx, id, status, None, outputs, error)
        })
    }

    // ------------------------------------------------------------------------ uploads

    /// Register an upload in the scope's workspace. Idempotent on `client_request_id`.
    pub fn create_upload(
        &self,
        scope: &ExecutionScope,
        spec: UploadSpec,
        client_request_id: Option<&ClientRequestId>,
        ttl: Duration,
    ) -> ServiceResult<UploadRecord> {
        let fp = fingerprint(&spec);
        self.write(|tx| {
            if let Some(crid) = client_request_id {
                if let Some(prev) = prior_request(tx, scope, crid, "upload", &fp)? {
                    return load_upload(tx, scope, &prev);
                }
            }
            let id = new_id("upl");
            let now = now_ms();
            tx.execute(
                "INSERT INTO uploads (upload_id, workspace_id, status, spec, created_ms, expires_ms)
                 VALUES (?1, ?2, 'initiated', ?3, ?4, ?5)",
                params![id, scope.workspace_id.as_str(), to_json(&spec)?, now, now + ttl.as_millis() as i64],
            )
            .map_err(db("insert upload"))?;
            if let Some(crid) = client_request_id {
                record_request(tx, scope, crid, "upload", &fp, &id)?;
            }
            load_upload(tx, scope, &id)
        })
    }

    pub fn upload(&self, scope: &ExecutionScope, id: &str) -> ServiceResult<UploadRecord> {
        self.read(|c| load_upload(c, scope, id))
    }

    /// Mark the bytes of an upload as received and verified.
    pub fn upload_received(&self, scope: &ExecutionScope, id: &str) -> ServiceResult<UploadRecord> {
        self.write(|tx| {
            let u = load_upload(tx, scope, id)?;
            match u.status {
                UploadStatus::Initiated | UploadStatus::Received => {
                    tx.execute(
                        "UPDATE uploads SET status = 'received' WHERE upload_id = ?1",
                        [id],
                    )
                    .map_err(db("upload"))?;
                    load_upload(tx, scope, id)
                }
                UploadStatus::Completed => {
                    Err(conflict(format!("upload {id} is already completed")))
                }
            }
        })
    }

    /// Record the artifact an upload was published as.
    pub fn upload_completed(
        &self,
        scope: &ExecutionScope,
        id: &str,
        artifact: &ArtifactRef,
    ) -> ServiceResult<UploadRecord> {
        self.write(|tx| {
            let u = load_upload(tx, scope, id)?;
            if u.status == UploadStatus::Initiated {
                return Err(conflict(format!("upload {id} has not received its bytes")));
            }
            tx.execute(
                "UPDATE uploads SET status = 'completed', artifact = ?2 WHERE upload_id = ?1",
                params![id, to_json(artifact)?],
            )
            .map_err(db("upload"))?;
            load_upload(tx, scope, id)
        })
    }

    // ------------------------------------------------------------------------ fixtures

    /// Import a hand-made episode and its log (C0 contract fixtures) into the scope's workspace,
    /// keeping their ids. The log must pass `check_event_log`; the record's status and `last_seq`
    /// are taken from the log. Returns false when the episode already exists.
    pub fn import_episode(
        &self,
        scope: &ExecutionScope,
        mut episode: ResearchEpisode,
        events: Vec<ResearchEvent>,
    ) -> ServiceResult<bool> {
        check_event_log(&events).map_err(ErrorEnvelope::from)?;
        if events.first().is_some_and(|e| e.seq != 1)
            || events.iter().any(|e| e.episode_id != episode.episode_id)
        {
            return Err(ErrorEnvelope::new(
                ErrorCode::InvalidRequest,
                "an imported log starts at seq 1 and belongs to the episode",
                RetryPolicy::Never,
            ));
        }
        self.write(|tx| {
            let exists: bool = tx
                .query_row(
                    "SELECT 1 FROM episodes WHERE episode_id = ?1",
                    [episode.episode_id.as_str()],
                    |_| Ok(()),
                )
                .optional()
                .map_err(db("import"))?
                .is_some();
            if exists {
                return Ok(false);
            }
            episode.workspace_id = scope.workspace_id.clone();
            episode.last_seq = events.last().map_or(0, |e| e.seq);
            if let Some(s) = events.iter().rev().find_map(|e| match e.body {
                EventBody::StatusChanged { to, .. } => Some(to),
                _ => None,
            }) {
                episode.status = s;
            }
            tx.execute(
                "INSERT INTO episodes (episode_id, workspace_id, principal, record) VALUES (?1, ?2, ?3, ?4)",
                params![
                    episode.episode_id.as_str(),
                    scope.workspace_id.as_str(),
                    scope.principal.as_str(),
                    to_json(&episode)?
                ],
            )
            .map_err(db("import"))?;
            for ev in &events {
                let kind = serde_json::to_value(&ev.body)
                    .ok()
                    .and_then(|v| v.get("kind").and_then(Value::as_str).map(str::to_string))
                    .unwrap_or_default();
                tx.execute(
                    "INSERT INTO events (episode_id, seq, event_id, kind, record) VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![ev.episode_id.as_str(), ev.seq as i64, ev.event_id.as_str(), kind, to_json(ev)?],
                )
                .map_err(db("import"))?;
            }
            Ok(true)
        })
    }

    /// Settle attempts whose lease expired before `now_ms` (all running attempts when `all` is set,
    /// e.g. at startup of the single-host runner, whose previous process is gone). Idempotent jobs
    /// are re-queued; others go to `reconciling`; a pending cancellation becomes `cancelled`.
    pub fn reap_expired(&self, all: bool) -> ServiceResult<Vec<JobRef>> {
        self.write(|tx| {
            let rows: Vec<(String, String, bool)> = {
                let mut st = tx
                    .prepare(
                        "SELECT job_id, status, idempotent FROM jobs
                         WHERE status IN ('running', 'cancel_requested') AND (?1 OR lease_expires_ms < ?2)",
                    )
                    .map_err(db("reap"))?;
                let rows = st
                    .query_map(params![all, now_ms()], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
                    .map_err(db("reap"))?;
                rows.collect::<Result<_, _>>().map_err(db("reap"))?
            };
            let mut out = Vec::new();
            for (id, status, idempotent) in rows {
                let id = JobId::new(id).map_err(|e| internal("stored id", e))?;
                tx.execute(
                    "UPDATE jobs SET lease_owner = NULL, lease_expires_ms = NULL, fence = fence + 1 WHERE job_id = ?1",
                    [id.as_str()],
                )
                .map_err(db("reap"))?;
                let r = if status == "cancel_requested" {
                    set_job_status(
                        tx,
                        &id,
                        JobStatus::Cancelled,
                        None,
                        None,
                        Some(ErrorEnvelope::new(
                            ErrorCode::Cancelled,
                            "cancelled; the interrupted attempt was abandoned",
                            RetryPolicy::Never,
                        )),
                    )?
                } else if idempotent {
                    tx.execute("UPDATE jobs SET queued_ms = ?2 WHERE job_id = ?1", params![id.as_str(), now_ms()])
                        .map_err(db("reap"))?;
                    set_job_status(tx, &id, JobStatus::Queued, None, None, None)?
                } else {
                    set_job_status(tx, &id, JobStatus::Reconciling, None, None, None)?
                };
                out.push(r);
            }
            Ok(out)
        })
    }
}

// ---------------------------------------------------------------------------- agent-runtime state

/// A change the runtime bridge asks for. Status changes are expressed as a target: the store fills
/// in `from` with the status it actually finds, and applies the change only when that status is one
/// of `from_any`. A host action that already moved the episode (pause, cancel) therefore wins over
/// a late runtime report, without a conflict.
#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeChange {
    Status {
        to: EpisodeStatus,
        reason: Option<String>,
        from_any: Vec<EpisodeStatus>,
    },
    Event(EventBody),
}

/// One runtime fact to record. `mark` names it (e.g. `item:<id>:completed`); a mark already
/// recorded for the episode makes the record a no-op, so a thread replayed after a restart does not
/// append it again.
#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeRecord {
    pub mark: Option<String>,
    pub causation: Causation,
    pub change: RuntimeChange,
}

/// What [`ResearchStore::pin_harness`] changed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HarnessPinChange {
    Unchanged,
    /// The episode had no harness pin.
    Pinned,
    /// The episode was pinned to this harness before.
    Replaced(HarnessRef),
}

/// The user messages handed to a new runtime turn.
#[derive(Debug, Clone)]
pub struct TurnStart {
    pub turn_key: String,
    pub episode: ResearchEpisode,
    /// `(seq, text)` of every user message not yet handed to a turn, in order.
    pub messages: Vec<(u64, String)>,
}

/// A turn that has not finished (for interrupts, resume and recovery).
#[derive(Debug, Clone)]
pub struct ActiveTurn {
    pub turn_key: String,
    pub episode_id: EpisodeId,
    pub scope: ExecutionScope,
    pub runtime_turn_id: Option<String>,
    pub through_seq: u64,
}

impl ResearchStore {
    /// Record runtime facts in one transaction (see [`RuntimeChange`], [`RuntimeRecord`]).
    pub fn record_runtime(
        &self,
        scope: &ExecutionScope,
        id: &EpisodeId,
        records: Vec<RuntimeRecord>,
    ) -> ServiceResult<Vec<ResearchEvent>> {
        self.write(|tx| {
            let mut ep = load_episode(tx, scope, id)?;
            let mut out = Vec::new();
            for r in records {
                if let Some(mark) = &r.mark {
                    let seen = tx
                        .query_row(
                            "SELECT 1 FROM runtime_marks WHERE episode_id = ?1 AND mark = ?2",
                            params![id.as_str(), mark],
                            |_| Ok(()),
                        )
                        .optional()
                        .map_err(db("runtime mark"))?
                        .is_some();
                    if seen {
                        continue;
                    }
                }
                let body = match r.change {
                    RuntimeChange::Event(body) => Some(body),
                    RuntimeChange::Status {
                        to,
                        reason,
                        from_any,
                    } => (ep.status != to && from_any.contains(&ep.status)).then_some(
                        EventBody::StatusChanged {
                            from: ep.status,
                            to,
                            reason,
                        },
                    ),
                };
                // A job submitted through the MCP server does not know its episode (the server only
                // knows the workspace); the tool result that returns its handle attaches it, in the
                // same transaction, so its updates reach this log and episode cancel reaches it.
                let attach = match &body {
                    Some(EventBody::ToolFinished { result }) => result.job.as_ref().map(|j| {
                        (
                            j.job_id.clone(),
                            Causation {
                                job_id: Some(j.job_id.clone()),
                                ..r.causation.clone()
                            },
                        )
                    }),
                    _ => None,
                };
                let seq = match body {
                    Some(body) => {
                        let ev = append_tx(tx, &mut ep, vec![(r.causation, body)])?.remove(0);
                        let seq = ev.seq;
                        out.push(ev);
                        if let Some((job_id, causation)) = attach {
                            let attached = tx
                                .execute(
                                    "UPDATE jobs SET episode_id = ?1
                                     WHERE job_id = ?2 AND workspace_id = ?3 AND episode_id IS NULL",
                                    params![
                                        id.as_str(),
                                        job_id.as_str(),
                                        scope.workspace_id.as_str()
                                    ],
                                )
                                .map_err(db("attach job"))?;
                            if attached == 1 {
                                let (job, _) = load_job(tx, Some(scope), &job_id)?;
                                out.extend(append_tx(
                                    tx,
                                    &mut ep,
                                    vec![(causation, EventBody::JobUpdated { job })],
                                )?);
                            }
                        }
                        seq
                    }
                    None => 0,
                };
                if let Some(mark) = &r.mark {
                    tx.execute(
                        "INSERT INTO runtime_marks (episode_id, mark, seq) VALUES (?1, ?2, ?3)",
                        params![id.as_str(), mark, seq as i64],
                    )
                    .map_err(db("runtime mark"))?;
                }
            }
            Ok(out)
        })
    }

    /// Record the runtime thread hosting an episode. Set once; replacing it needs the thread being
    /// replaced (`replacing`), so two writers cannot silently swap it.
    pub fn set_runtime_thread(
        &self,
        scope: &ExecutionScope,
        id: &EpisodeId,
        thread: RuntimeThreadRef,
        replacing: Option<&str>,
    ) -> ServiceResult<ResearchEpisode> {
        self.write(|tx| {
            let mut ep = load_episode(tx, scope, id)?;
            match &ep.runtime {
                Some(t) if t.thread_id == thread.thread_id => return Ok(ep),
                Some(t) if Some(t.thread_id.as_str()) == replacing => {}
                Some(t) => {
                    return Err(conflict(format!(
                        "episode {id} is already hosted by runtime thread {}",
                        t.thread_id
                    )))
                }
                None => {}
            }
            ep.runtime = Some(thread);
            ep.revision += 1;
            ep.updated_at = Timestamp::now();
            tx.execute(
                "UPDATE episodes SET record = ?2 WHERE episode_id = ?1",
                params![id.as_str(), to_json(&ep)?],
            )
            .map_err(db("update episode"))?;
            Ok(ep)
        })
    }

    /// Pin the harness an episode's runtime was started with (`pins.harness`). The pin is a
    /// research.v2 field, so the episode record becomes v2. Returns what changed, so the caller can
    /// say so in the log; an identical pin changes nothing.
    pub fn pin_harness(
        &self,
        scope: &ExecutionScope,
        id: &EpisodeId,
        harness: HarnessRef,
    ) -> ServiceResult<HarnessPinChange> {
        harness.validate().map_err(ErrorEnvelope::from)?;
        self.write(|tx| {
            let mut ep = load_episode(tx, scope, id)?;
            let change = match &ep.pins.harness {
                Some(h) if *h == harness => return Ok(HarnessPinChange::Unchanged),
                Some(h) => HarnessPinChange::Replaced(h.clone()),
                None => HarnessPinChange::Pinned,
            };
            ep.pins.harness = Some(harness);
            ep.schema_version = ResearchSchema::V2;
            ep.validate().map_err(ErrorEnvelope::from)?;
            ep.revision += 1;
            ep.updated_at = Timestamp::now();
            tx.execute(
                "UPDATE episodes SET record = ?2 WHERE episode_id = ?1",
                params![id.as_str(), to_json(&ep)?],
            )
            .map_err(db("update episode"))?;
            Ok(change)
        })
    }

    /// Open a turn for the user messages not yet handed to one, and move the episode to `running`,
    /// in one transaction. `None` when there is nothing to send, the episode is paused or ended, or
    /// another turn is still open (never two uncoordinated turns on one episode).
    ///
    /// Before an episode's first turn, only the user messages after its last assistant output
    /// (`message.completed` or `answer.completed`) count: an imported log may hold messages that
    /// were already answered. `note` is added to the reason of the status change.
    pub fn begin_turn(
        &self,
        scope: &ExecutionScope,
        id: &EpisodeId,
        note: Option<&str>,
    ) -> ServiceResult<Option<TurnStart>> {
        use EpisodeStatus as S;
        self.write(|tx| {
            let mut ep = load_episode(tx, scope, id)?;
            if matches!(ep.status, S::Paused | S::Cancelled | S::Failed) {
                return Ok(None);
            }
            let open = tx
                .query_row(
                    "SELECT 1 FROM runtime_turns WHERE episode_id = ?1 AND state IN ('starting', 'running')",
                    [id.as_str()],
                    |_| Ok(()),
                )
                .optional()
                .map_err(db("begin turn"))?
                .is_some();
            if open {
                return Ok(None);
            }
            // Messages already handed to a turn are never re-sent, whatever that turn's outcome.
            let turns: Option<i64> = tx
                .query_row(
                    "SELECT MAX(through_seq) FROM runtime_turns WHERE episode_id = ?1",
                    [id.as_str()],
                    |r| r.get(0),
                )
                .map_err(db("begin turn"))?;
            let sent: i64 = match turns {
                Some(s) => s,
                None => tx
                    .query_row(
                        "SELECT COALESCE(MAX(seq), 0) FROM events WHERE episode_id = ?1
                         AND kind IN ('message.completed', 'answer.completed')",
                        [id.as_str()],
                        |r| r.get(0),
                    )
                    .map_err(db("begin turn"))?,
            };
            let messages: Vec<(u64, String)> = {
                let mut st = tx
                    .prepare(
                        "SELECT seq, record FROM events WHERE episode_id = ?1 AND kind = 'message.user' AND seq > ?2 ORDER BY seq",
                    )
                    .map_err(db("begin turn"))?;
                let rows = st
                    .query_map(params![id.as_str(), sent], |r| {
                        Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?))
                    })
                    .map_err(db("begin turn"))?;
                let mut out = Vec::new();
                for row in rows {
                    let (seq, rec) = row.map_err(db("begin turn"))?;
                    let ev: ResearchEvent = from_json(&rec)?;
                    if let EventBody::UserMessage { text, .. } = ev.body {
                        out.push((seq as u64, text));
                    }
                }
                out
            };
            let (Some(&(first, _)), Some(&(through, _))) = (messages.first(), messages.last())
            else {
                return Ok(None);
            };
            let turn_key = new_id("rt");
            tx.execute(
                "INSERT INTO runtime_turns (turn_key, episode_id, workspace_id, scope, through_seq, state, created_ms)
                 VALUES (?1, ?2, ?3, ?4, ?5, 'starting', ?6)",
                params![turn_key, id.as_str(), scope.workspace_id.as_str(), to_json(scope)?, through as i64, now_ms()],
            )
            .map_err(db("begin turn"))?;
            if ep.status != S::Running {
                let from = ep.status;
                append_tx(
                    tx,
                    &mut ep,
                    vec![(
                        Causation {
                            caused_by_seq: Some(first),
                            ..Default::default()
                        },
                        EventBody::StatusChanged {
                            from,
                            to: S::Running,
                            reason: Some(match note {
                                Some(n) => format!("runtime turn started; {n}"),
                                None => "runtime turn started".into(),
                            }),
                        },
                    )],
                )?;
            }
            Ok(Some(TurnStart {
                turn_key,
                episode: ep,
                messages,
            }))
        })
    }

    /// The runtime accepted the turn and gave it an id.
    pub fn turn_running(&self, turn_key: &str, runtime_turn_id: &str) -> ServiceResult<()> {
        self.write(|tx| {
            tx.execute(
                "UPDATE runtime_turns SET state = 'running', runtime_turn_id = ?2
                 WHERE turn_key = ?1 AND state IN ('starting', 'running')",
                params![turn_key, runtime_turn_id],
            )
            .map_err(db("turn running"))?;
            Ok(())
        })
    }

    /// Close a turn with its outcome (`completed`, `interrupted`, `failed`, `ambiguous`, ...).
    pub fn finish_turn(&self, turn_key: &str, outcome: &str) -> ServiceResult<()> {
        self.write(|tx| {
            tx.execute(
                "UPDATE runtime_turns SET state = 'finished', outcome = ?2
                 WHERE turn_key = ?1 AND state IN ('starting', 'running')",
                params![turn_key, outcome],
            )
            .map_err(db("finish turn"))?;
            Ok(())
        })
    }

    /// Whether a runtime fact (mark) is already recorded for the episode (host-internal).
    pub fn runtime_marked(&self, id: &EpisodeId, mark: &str) -> ServiceResult<bool> {
        self.read(|c| {
            c.query_row(
                "SELECT 1 FROM runtime_marks WHERE episode_id = ?1 AND mark = ?2",
                params![id.as_str(), mark],
                |_| Ok(()),
            )
            .optional()
            .map(|r| r.is_some())
            .map_err(db("runtime mark"))
        })
    }

    /// The open turn of an episode, if any.
    pub fn active_turn(
        &self,
        scope: &ExecutionScope,
        id: &EpisodeId,
    ) -> ServiceResult<Option<ActiveTurn>> {
        self.read(|c| {
            load_episode(c, scope, id)?;
            Ok(active_turns_where(c, Some(id))?.into_iter().next())
        })
    }

    /// Every open turn of every workspace (host-internal: recovery at startup, like
    /// [`ResearchStore::reap_expired`]). Each carries the scope it was started under.
    pub fn active_turns(&self) -> ServiceResult<Vec<ActiveTurn>> {
        self.read(|c| active_turns_where(c, None))
    }
}

fn active_turns_where(
    c: &Connection,
    episode: Option<&EpisodeId>,
) -> ServiceResult<Vec<ActiveTurn>> {
    let mut st = c
        .prepare(
            "SELECT turn_key, episode_id, scope, runtime_turn_id, through_seq FROM runtime_turns
             WHERE state IN ('starting', 'running') AND (?1 IS NULL OR episode_id = ?1)
             ORDER BY created_ms",
        )
        .map_err(db("active turns"))?;
    let rows = st
        .query_map([episode.map(EpisodeId::as_str)], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, Option<String>>(3)?,
                r.get::<_, i64>(4)?,
            ))
        })
        .map_err(db("active turns"))?;
    let mut out = Vec::new();
    for row in rows {
        let (turn_key, ep, scope, runtime_turn_id, through) = row.map_err(db("active turns"))?;
        out.push(ActiveTurn {
            turn_key,
            episode_id: EpisodeId::new(ep).map_err(|e| internal("stored id", e))?,
            scope: from_json(&scope)?,
            runtime_turn_id,
            through_seq: through as u64,
        });
    }
    Ok(out)
}

// ---------------------------------------------------------------------------- transaction helpers

fn load_episode(
    c: &Connection,
    scope: &ExecutionScope,
    id: &EpisodeId,
) -> ServiceResult<ResearchEpisode> {
    let row: Option<(String, String)> = c
        .query_row(
            "SELECT workspace_id, record FROM episodes WHERE episode_id = ?1",
            [id.as_str()],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()
        .map_err(db("load episode"))?;
    match row {
        // Another workspace's episode is indistinguishable from a missing one.
        Some((ws, rec)) if ws == scope.workspace_id.as_str() => from_json(&rec),
        _ => Err(not_found(format!("episode {id}"))),
    }
}

fn append_tx(
    tx: &Transaction,
    ep: &mut ResearchEpisode,
    events: Vec<(Causation, EventBody)>,
) -> ServiceResult<Vec<ResearchEvent>> {
    let mut out = Vec::with_capacity(events.len());
    for (causation, body) in events {
        if let EventBody::StatusChanged { from, to, .. } = &body {
            if *from != ep.status {
                return Err(conflict(format!(
                    "status change from {} but episode {} is {}",
                    wire(from),
                    ep.episode_id,
                    wire(&ep.status)
                )));
            }
            ep.status = *to;
        }
        let ev = ResearchEvent {
            schema_version: ResearchSchema::V2,
            event_id: EventId::new(new_id("ev")).expect("generated id"),
            episode_id: ep.episode_id.clone(),
            seq: ep.last_seq + 1,
            occurred_at: Timestamp::now(),
            causation,
            body,
            origin: ep.origin,
        };
        ev.validate().map_err(|e| ErrorEnvelope::from(e))?;
        let kind = serde_json::to_value(&ev.body)
            .ok()
            .and_then(|v| v.get("kind").and_then(Value::as_str).map(str::to_string))
            .unwrap_or_default();
        tx.execute(
            "INSERT INTO events (episode_id, seq, event_id, kind, record) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![ev.episode_id.as_str(), ev.seq as i64, ev.event_id.as_str(), kind, to_json(&ev)?],
        )
        .map_err(db("append event"))?;
        ep.last_seq = ev.seq;
        out.push(ev);
    }
    if !out.is_empty() {
        ep.revision += 1;
        ep.updated_at = Timestamp::now();
        tx.execute(
            "UPDATE episodes SET record = ?2 WHERE episode_id = ?1",
            params![ep.episode_id.as_str(), to_json(ep)?],
        )
        .map_err(db("update episode"))?;
    }
    Ok(out)
}

fn load_event(c: &Connection, event_id: &str) -> ServiceResult<ResearchEvent> {
    c.query_row(
        "SELECT record FROM events WHERE event_id = ?1",
        [event_id],
        |r| r.get::<_, String>(0),
    )
    .map_err(db("load event"))
    .and_then(|s| from_json(&s))
}

fn prior_request(
    tx: &Transaction,
    scope: &ExecutionScope,
    crid: &ClientRequestId,
    operation: &str,
    fp: &str,
) -> ServiceResult<Option<String>> {
    let row: Option<(String, String, String)> = tx
        .query_row(
            "SELECT operation, fingerprint, response FROM requests WHERE workspace_id = ?1 AND client_request_id = ?2",
            params![scope.workspace_id.as_str(), crid.as_str()],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()
        .map_err(db("idempotency key"))?;
    match row {
        None => Ok(None),
        Some((op, f, resp)) if op == operation && f == fp => Ok(Some(resp)),
        Some(_) => Err(conflict(format!(
            "client_request_id {crid} was already used for a different request"
        ))),
    }
}

fn record_request(
    tx: &Transaction,
    scope: &ExecutionScope,
    crid: &ClientRequestId,
    operation: &str,
    fp: &str,
    response: &str,
) -> ServiceResult<()> {
    tx.execute(
        "INSERT INTO requests (workspace_id, client_request_id, operation, fingerprint, response) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![scope.workspace_id.as_str(), crid.as_str(), operation, fp, response],
    )
    .map_err(db("idempotency key"))?;
    Ok(())
}

fn load_job(
    c: &Connection,
    scope: Option<&ExecutionScope>,
    id: &JobId,
) -> ServiceResult<(JobRef, Option<EpisodeId>)> {
    let row: Option<(String, String, Option<String>)> = c
        .query_row(
            "SELECT workspace_id, record, episode_id FROM jobs WHERE job_id = ?1",
            [id.as_str()],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()
        .map_err(db("load job"))?;
    match row {
        Some((ws, rec, ep)) if scope.is_none_or(|s| s.workspace_id.as_str() == ws) => Ok((
            from_json(&rec)?,
            ep.map(EpisodeId::new)
                .transpose()
                .map_err(|e| internal("stored id", e))?,
        )),
        _ => Err(not_found(format!("job {id}"))),
    }
}

fn load_upload(c: &Connection, scope: &ExecutionScope, id: &str) -> ServiceResult<UploadRecord> {
    let row: Option<(String, String, String, Option<String>, i64)> = c
        .query_row(
            "SELECT workspace_id, status, spec, artifact, expires_ms FROM uploads WHERE upload_id = ?1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
        )
        .optional()
        .map_err(db("load upload"))?;
    match row {
        Some((ws, status, spec, artifact, expires_ms)) if ws == scope.workspace_id.as_str() => {
            Ok(UploadRecord {
                upload_id: id.to_string(),
                status: match status.as_str() {
                    "initiated" => UploadStatus::Initiated,
                    "received" => UploadStatus::Received,
                    _ => UploadStatus::Completed,
                },
                spec: from_json(&spec)?,
                artifact: artifact.map(|a| from_json(&a)).transpose()?,
                expires_ms,
            })
        }
        _ => Err(not_found(format!("upload {id}"))),
    }
}

fn check_fence(tx: &Transaction, id: &JobId, fence: i64) -> ServiceResult<JobStatus> {
    let row: Option<(i64, String)> = tx
        .query_row(
            "SELECT fence, record FROM jobs WHERE job_id = ?1",
            [id.as_str()],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()
        .map_err(db("fence"))?;
    let Some((current, rec)) = row else {
        return Err(not_found(format!("job {id}")));
    };
    let job: JobRef = from_json(&rec)?;
    if current != fence || !matches!(job.status, JobStatus::Running | JobStatus::CancelRequested) {
        return Err(ErrorEnvelope::new(
            ErrorCode::Conflict,
            format!("stale fencing token for job {id}: the attempt no longer owns the job"),
            RetryPolicy::Never,
        ));
    }
    Ok(job.status)
}

fn cancel_job_tx(tx: &Transaction, id: &JobId) -> ServiceResult<JobRef> {
    let (job, _) = load_job(tx, None, id)?;
    match job.status {
        JobStatus::Queued | JobStatus::Reconciling => set_job_status(
            tx,
            id,
            JobStatus::Cancelled,
            None,
            None,
            Some(ErrorEnvelope::new(
                ErrorCode::Cancelled,
                "cancelled on request",
                RetryPolicy::Never,
            )),
        ),
        JobStatus::Running => set_job_status(tx, id, JobStatus::CancelRequested, None, None, None),
        _ => Ok(job),
    }
}

/// Update a job's public record and status and, when it belongs to an episode, append the
/// `job.updated` event in the same transaction.
fn set_job_status(
    tx: &Transaction,
    id: &JobId,
    status: JobStatus,
    progress: Option<Option<JobProgress>>,
    outputs: Option<Vec<ArtifactRef>>,
    error: Option<ErrorEnvelope>,
) -> ServiceResult<JobRef> {
    let (mut job, episode) = load_job(tx, None, id)?;
    if job.status.is_terminal() && job.status != status {
        return Err(conflict(format!(
            "job {id} is already {}",
            wire(&job.status)
        )));
    }
    job.status = status;
    if let Some(p) = progress {
        job.progress = p;
    }
    if let Some(o) = outputs {
        job.outputs = o;
    }
    job.error = error;
    job.updated_at = Timestamp::now();
    job.validate().map_err(|e| internal("job", e))?;
    tx.execute(
        "UPDATE jobs SET status = ?2, record = ?3 WHERE job_id = ?1",
        params![id.as_str(), wire(&status), to_json(&job)?],
    )
    .map_err(db("update job"))?;
    if let Some(ep_id) = episode {
        let rec: String = tx
            .query_row(
                "SELECT record FROM episodes WHERE episode_id = ?1",
                [ep_id.as_str()],
                |r| r.get(0),
            )
            .map_err(db("job episode"))?;
        let mut ep: ResearchEpisode = from_json(&rec)?;
        let causation = Causation {
            job_id: Some(id.clone()),
            ..Default::default()
        };
        append_tx(
            tx,
            &mut ep,
            vec![(causation, EventBody::JobUpdated { job: job.clone() })],
        )?;
    }
    Ok(job)
}
