//! Scientific retrieval and knowledge (workboard R1/R3): PDF parse views with verifiable locators
//! (MuPDF), the corpus manifest and snapshots, a SQLite/FTS5 catalog, dense Qwen3 embeddings and
//! bounded Qwen3 reranking served by the DeepInfra API, RRF fusion, the leak firewall, and
//! [`service::EvidenceService`], the one `KnowledgeService` implementation.
//!
//! Evidence content is explanation, never a predictor input: [`firewall::scan`] proves from the
//! cargo dependency graph that the scientific crates do not depend on this crate, and the service
//! withholds outcome-bearing passages the scope's partitions do not grant ([`service::Lockbox`]).
//!
//! Commands (`ignis evidence ...`, JSON on stdout): `corpus status|fetch|ingest|snapshot`, `ingest`,
//! `verify`, `embed [--execute] [--force]`, `search`, `read`, `resolve`, `firewall`. `search`, `read`
//! and `resolve` print the contract documents (`SearchPage`, `ReadResult`, `EvidenceRef`) of the same
//! service the HTTP API and MCP tools use. The API key is read only from `DEEPINFRA_API_KEY`.

pub mod catalog;
pub mod corpus;
pub mod deepinfra;
pub mod dense;
pub mod firewall;
#[cfg(feature = "pdf")]
pub mod pdf;
pub mod records;
pub mod search;
pub mod service;
#[cfg(feature = "pdf")]
pub mod verify;

use std::collections::BTreeSet;
use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use clap::Subcommand;
use ignis_core::contracts::{
    Capability, EvidenceId, EvidenceSchema, ExecutionScope, Family, PageCursor, Partition,
    PrincipalId, ReadRange, ReadRequest, ScopeId, ScopeRole, SearchFilters, SearchMode,
    SearchRequest, SourceId, WorkspaceId,
};
use ignis_core::resources::{guard, load_profile, Guard, Job};
use ignis_core::services::KnowledgeService;
use serde_json::{json, Value};

/// Version of the parse view definition (page text, chunking, locator fields). 0.3.0 adds the
/// parse view id, the offset unit and normalized bboxes to every locator and stores page texts.
pub const PARSER_VERSION: &str = "0.3.0-mupdf";

#[derive(Debug, Subcommand)]
pub enum CorpusCmd {
    /// Derive the status of every seed (configs/evidence/corpus.toml) and write data/corpus/manifest.jsonl.
    Status {
        #[arg(long)]
        db: Option<PathBuf>,
        /// print only; do not rewrite the manifest
        #[arg(long)]
        no_write: bool,
    },
    /// Fetch NTRS citation records and PDFs of the seeds (network: dry-run plan unless --execute).
    Fetch {
        #[arg(long)]
        execute: bool,
    },
    /// Parse and index every downloaded, non-rejected, non-restricted seed (needs the `pdf` feature).
    Ingest {
        #[arg(long)]
        db: Option<PathBuf>,
    },
    /// Per publication: passages, and how many the local research scope (train + validation) cannot read.
    Lockbox {
        #[arg(long)]
        db: Option<PathBuf>,
    },
    /// Write the catalog's snapshot (members + id) to data/corpus/snapshots/<id>.json.
    Snapshot {
        #[arg(long)]
        db: Option<PathBuf>,
    },
}

#[derive(Debug, Subcommand)]
pub enum EvidenceCmd {
    #[command(subcommand)]
    Corpus(CorpusCmd),
    /// Parse PDFs (page texts, char spans, bboxes, captions) and upsert their parse views into the catalog.
    Ingest {
        /// a PDF or a directory of PDFs (NTRS `<stem>.json` sidecars are read)
        path: PathBuf,
        #[arg(long)]
        db: Option<PathBuf>,
        /// investigation ids to attach, e.g. PSI-25
        #[arg(long)]
        investigation: Vec<String>,
    },
    /// Re-derive every stored parse view from the original bytes (hash, page texts, chunk spans, locators).
    Verify {
        #[arg(long)]
        db: Option<PathBuf>,
    },
    /// Embed every chunk via DeepInfra to vectors.f32 + vectors.ids.json next to the catalog
    /// (dry-run with token/USD estimate unless --execute; an existing set is replaced only with --force).
    Embed {
        #[arg(long)]
        db: Option<PathBuf>,
        #[arg(long, default_value = "cpu-local")]
        profile: String,
        #[arg(long)]
        execute: bool,
        /// replace an existing vector set (of any model)
        #[arg(long)]
        force: bool,
    },
    /// Hybrid search through the KnowledgeService; prints a SearchPage (evidence refs + snippets).
    Search {
        query: String,
        #[arg(long)]
        db: Option<PathBuf>,
        /// hybrid | lexical | dense
        #[arg(long, default_value = "hybrid")]
        mode: String,
        #[arg(long)]
        page_size: Option<u32>,
        /// continue a previous page (its `next_cursor`)
        #[arg(long)]
        cursor: Option<String>,
        /// flex | acme_normal | acme_inverse | bass (repeatable)
        #[arg(long)]
        family: Vec<String>,
        #[arg(long)]
        investigation: Vec<String>,
        /// source id (`sha256:<hex>`), repeatable
        #[arg(long)]
        source: Vec<String>,
        /// declare the provider unavailable (no network); degradation is reported
        #[arg(long)]
        offline: bool,
    },
    /// Read a source through the KnowledgeService; prints a ReadResult.
    Read {
        /// source id (`sha256:<hex>`)
        source: String,
        /// physical pages, `3` or `3-5`
        #[arg(long, conflicts_with = "passage")]
        pages: Option<String>,
        /// the passage of an evidence id, with its neighbours
        #[arg(long)]
        passage: Option<String>,
        #[arg(long)]
        db: Option<PathBuf>,
    },
    /// Resolve an evidence id to its EvidenceRef (with the exact quote).
    Resolve {
        evidence_id: String,
        #[arg(long)]
        db: Option<PathBuf>,
    },
    /// Dependency-graph check: protected scientific crates must not depend on ignis-evidence.
    Firewall {
        #[arg(long, default_value = "Cargo.toml")]
        manifest: PathBuf,
    },
}

fn print(v: &Value) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(v)?);
    Ok(())
}

fn cfg() -> Result<dense::RetrievalCfg> {
    dense::RetrievalCfg::load(&dense::RetrievalCfg::default_path())
}

/// The scope of a local researcher at this terminal: research role, train + validation outcomes,
/// read-only sources. The CLI never grants the test partition (only an evaluation scope may).
pub fn local_research_scope() -> ExecutionScope {
    ExecutionScope {
        scope_id: ScopeId::new("local-cli").expect("id"),
        workspace_id: WorkspaceId::new("local").expect("id"),
        principal: PrincipalId::new("local-researcher").expect("id"),
        role: ScopeRole::Research,
        partitions: BTreeSet::from([Partition::Train, Partition::Validation]),
        capabilities: BTreeSet::from([Capability::ReadSources]),
        corpus_snapshot: None,
        parent_scope_id: None,
    }
}

fn open_service(db: Option<PathBuf>, offline: bool) -> Result<service::EvidenceService> {
    let db = db.unwrap_or_else(catalog::default_db);
    if !db.exists() {
        bail!(
            "no catalog at {} (run `ignis evidence corpus ingest`)",
            db.display()
        );
    }
    let s = service::EvidenceService::open(&db, cfg()?, service::Lockbox::load()?)?;
    Ok(if offline { s.offline() } else { s })
}

/// Service errors keep their typed envelope on stdout and fail the command.
fn emit<T: serde::Serialize>(r: Result<T, ignis_core::contracts::ErrorEnvelope>) -> Result<()> {
    match r {
        Ok(v) => print(&serde_json::to_value(v)?),
        Err(e) => {
            print(&json!({"error": e}))?;
            bail!("{:?}: {}", e.code, e.message)
        }
    }
}

fn parse_pages(s: &str) -> Result<(u32, u32)> {
    let (a, b) = s.split_once('-').unwrap_or((s, s));
    Ok((
        a.trim().parse().context("--pages")?,
        b.trim().parse().context("--pages")?,
    ))
}

#[cfg(feature = "pdf")]
fn ingest_one(
    con: &mut rusqlite::Connection,
    path: &std::path::Path,
    opts: &pdf::ParseOptions,
    rp: &pdf::RunPatterns,
    families: Option<&[String]>,
    outcome_free: Option<&str>,
) -> Result<Value> {
    let t = std::time::Instant::now();
    let mut parsed = pdf::parse_pdf(path, opts, rp)?;
    if let Some(f) = families.filter(|f| !f.is_empty()) {
        parsed.publication.families = f.to_vec(); // the corpus seed is the authority, not keyword hints
    }
    if let Some(why) = outcome_free {
        parsed.publication.outcome_bearing = false;
        parsed
            .chunks
            .iter_mut()
            .for_each(|c| c.outcome_bearing = false);
        parsed.report["outcome_free"] = json!(why);
    }
    let publ = &parsed.publication;
    let n = catalog::upsert_parse_view(
        con,
        publ,
        &parsed.chunks,
        &parsed.pages,
        Some(&path.display().to_string()),
    )?;
    Ok(
        json!({"file": publ.file_name, "source_id": format!("sha256:{}", publ.source_sha256),
        "publication_id": publ.id, "title": publ.title, "pages": publ.n_pages, "chunks": n,
        "families": publ.families, "report": parsed.report, "seconds": t.elapsed().as_secs_f64()}),
    )
}

pub fn run(cmd: EvidenceCmd) -> Result<()> {
    match cmd {
        EvidenceCmd::Corpus(c) => run_corpus(c),
        #[cfg(not(feature = "pdf"))]
        EvidenceCmd::Ingest { .. } | EvidenceCmd::Verify { .. } => anyhow::bail!(
            "PDF ingest/verify needs the `pdf` feature (MuPDF): cargo build -p ignis-cli --features pdf \
             (on Windows set CLANG_PATH to LLVM's clang.exe and build from a VS vcvars64 shell)"
        ),
        #[cfg(feature = "pdf")]
        EvidenceCmd::Ingest { path, db, investigation } => {
            let db = db.unwrap_or_else(catalog::default_db);
            let pdfs: Vec<PathBuf> = if path.is_dir() {
                let mut v = Vec::new();
                walk_pdfs(&path, &mut v)?;
                v.sort();
                v
            } else {
                vec![path]
            };
            let rp = pdf::RunPatterns::load(&pdf::RunPatterns::default_path())?;
            let opts = pdf::ParseOptions { investigation_ids: investigation, ..Default::default() };
            let mut con = catalog::connect(&db, false)?;
            let mut out = Vec::new();
            for p in &pdfs {
                out.push(ingest_one(&mut con, p, &opts, &rp, None, None)?);
            }
            print(&json!({"db": db.display().to_string(), "ingested": out}))
        }
        #[cfg(feature = "pdf")]
        EvidenceCmd::Verify { db } => {
            let db = db.unwrap_or_else(catalog::default_db);
            let con = catalog::connect(&db, true)?;
            let mut out = Vec::new();
            let mut failed = 0;
            for p in catalog::publications(&con)? {
                let Some(rel) = catalog::asset_location(&con, &p.source_sha256)? else {
                    failed += 1;
                    out.push(json!({"publication_id": p.id, "passed": false, "error": "no recorded location of the bytes"}));
                    continue;
                };
                let bytes = std::fs::read(&rel).with_context(|| format!("read {rel}"))?;
                let r = verify::verify_view(&con, &p, &bytes)?;
                if !r.passed() {
                    failed += 1;
                }
                let mut v = serde_json::to_value(&r)?;
                v["passed"] = json!(r.passed());
                out.push(v);
            }
            print(&json!({"db": db.display().to_string(), "parser_version": PARSER_VERSION, "views": out,
                "failed": failed}))?;
            if failed > 0 {
                bail!("{failed} parse view(s) failed verification");
            }
            Ok(())
        }
        EvidenceCmd::Embed { db, profile, execute, force } => {
            let db = db.unwrap_or_else(catalog::default_db);
            let c = cfg()?;
            let e = &c.embedding;
            let con = catalog::connect(&db, false)?;
            let rows = catalog::all_chunk_texts(&con)?;
            let prof = load_profile(&profile)?;
            let dir = db.parent().unwrap_or(std::path::Path::new(".")).to_path_buf();
            let want = e.identity();
            let existing = dense::VectorStore::read_meta(&dir)?;
            let permission = dense::embed_permission(existing.as_ref(), &want, force);
            let texts: Vec<String> = rows.iter().map(|r| r.1.clone()).collect();
            let lens: Vec<usize> = texts.iter().map(|t| t.chars().count()).collect();
            let n_req = deepinfra::batches(&lens, e.batch_size, e.max_batch_chars).len();
            let est_tokens: u64 = texts.iter().map(|t| deepinfra::estimate_tokens(t)).sum();
            let est_usd = deepinfra::usd(est_tokens, e.usd_per_mtok);
            let key = deepinfra::ApiKey::from_env();
            let detail = json!({"provider": deepinfra::PROVIDER, "model": e.model, "dimensions": e.dimensions,
                "chunks": rows.len(), "chars": lens.iter().sum::<usize>(), "requests": n_req, "estimated_tokens": est_tokens,
                "usd_per_mtok": e.usd_per_mtok, "estimated_usd": est_usd, "api_key_set": key.is_some(),
                "query_instruction": want.query_instruction, "existing_set": existing,
                "permission": permission.as_ref().map(|_| "ok".to_string()).unwrap_or_else(|e| format!("refused: {e}"))});
            // ~1.5 s per request, measured on DeepInfra; the guard's local-hours rate is not what is billed here
            if guard("evidence.embed", &prof, n_req as f64 * 1.5 / 3600.0, execute, detail)? == Guard::DryRun {
                return Ok(());
            }
            if let Err(msg) = permission {
                bail!("{msg}");
            }
            let key = key.with_context(|| format!("{} is not set (the key is read only from that variable)", deepinfra::KEY_ENV))?;
            let mut job = Job::start("evidence.embed", &prof, json!({"provider": deepinfra::PROVIDER, "model": e.model,
                "dimensions": e.dimensions, "n": rows.len(), "estimated_tokens": est_tokens}))?;
            let client = deepinfra::Client::new(key, c.provider.clone());
            let (vecs, usage) = client.embed(&e.model, &texts, e.dimensions, e.batch_size, e.max_batch_chars, e.usd_per_mtok, None)?;
            job.set_api_cost(usage.estimated_usd, usage.billed_usd);
            let ids: Vec<String> = rows.iter().map(|r| r.0.clone()).collect();
            let revision = format!("api:{}", chrono::Utc::now().format("%Y-%m-%d"));
            let meta = dense::VectorStore::write(&dir, &ids, &vecs, &want, &revision)?;
            con.execute("DELETE FROM embedding_sets", [])?; // one vector file next to the catalog: one row
            con.execute(
                "INSERT OR REPLACE INTO embedding_sets(id, provider, model, revision, dim, n, instruction, vectors_path, ids_path, created_at)                  VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
                rusqlite::params![meta["sha256"].as_str(), meta["provider"].as_str(), meta["model"].as_str(), meta["revision"].as_str(),
                    meta["dim"].as_i64(), meta["n"].as_i64(), meta["query_instruction"].as_str(), "vectors.f32", "vectors.ids.json",
                    meta["created_at"].as_str()],
            )?;
            let mut out = meta.clone();
            if let Value::Object(m) = &mut out {
                m.remove("ids");
                m.insert("usage".into(), json!(usage));
                m.insert("s_per_chunk".into(), json!(usage.latency_ms / 1e3 / ids.len().max(1) as f64));
                m.insert("job".into(), json!(job.id()));
            }
            job.finish(out.clone());
            print(&out)
        }
        EvidenceCmd::Search { query, db, mode, page_size, cursor, family, investigation, source, offline } => {
            let svc = open_service(db, offline)?;
            let mode: SearchMode = serde_json::from_value(json!(mode)).context("--mode hybrid|lexical|dense")?;
            let families = family
                .iter()
                .map(|f| serde_json::from_value::<Family>(json!(f)).with_context(|| format!("--family {f}")))
                .collect::<Result<Vec<_>>>()?;
            let req = SearchRequest {
                schema_version: EvidenceSchema::V1,
                query,
                mode,
                filters: SearchFilters {
                    families,
                    source_kinds: Vec::new(),
                    investigation_ids: investigation,
                    source_ids: source.into_iter().map(SourceId::new).collect::<Result<_, _>>()?,
                },
                corpus_snapshot: None,
                page_size,
                cursor: cursor.map(PageCursor::new).transpose()?,
            };
            emit(svc.search(&local_research_scope(), &req))
        }
        EvidenceCmd::Read { source, pages, passage, db } => {
            let svc = open_service(db, true)?;
            let range = match (pages, passage) {
                (Some(p), _) => {
                    let (first, last) = parse_pages(&p)?;
                    ReadRange::Pages { first, last }
                }
                (None, Some(e)) => ReadRange::Passage { evidence_id: EvidenceId::new(e)? },
                (None, None) => bail!("give --pages or --passage"),
            };
            let req = ReadRequest { schema_version: EvidenceSchema::V1, source_id: SourceId::new(source)?, parse_view_id: None, range };
            emit(svc.read(&local_research_scope(), &req))
        }
        EvidenceCmd::Resolve { evidence_id, db } => {
            let svc = open_service(db, true)?;
            emit(svc.resolve_evidence(&local_research_scope(), &EvidenceId::new(evidence_id)?))
        }
        EvidenceCmd::Firewall { manifest } => {
            let protected = firewall::protected_from_config(&ignis_core::resources::configs_dir().join("evidence").join("firewall.toml"))?;
            let r = firewall::scan(&manifest, &protected)?;
            print(&serde_json::to_value(&r)?)?;
            if !r.passed {
                bail!("firewall: {} protected crate(s) reach {}", r.violations.len(), firewall::EVIDENCE_CRATE);
            }
            Ok(())
        }
    }
}

fn run_corpus(cmd: CorpusCmd) -> Result<()> {
    let seeds = corpus::load_seeds(&corpus::default_seeds_path())?;
    match cmd {
        CorpusCmd::Status { db, no_write } => {
            let db = db.unwrap_or_else(catalog::default_db);
            let con = if db.exists() { Some(catalog::connect(&db, true)?) } else { None };
            let rows = corpus::status(&seeds, con.as_ref())?;
            let path = corpus::corpus_dir().join("manifest.jsonl");
            if !no_write {
                corpus::write_manifest(&path, &rows)?;
            }
            let mut counts = std::collections::BTreeMap::<String, usize>::new();
            for r in &rows {
                *counts.entry(serde_json::to_value(r.status)?.as_str().unwrap_or("?").to_string()).or_default() += 1;
            }
            print(&json!({"manifest": (!no_write).then(|| path.display().to_string()), "counts": counts, "entries": rows}))
        }
        CorpusCmd::Fetch { execute } => {
            let todo: Vec<&corpus::Seed> = seeds
                .iter()
                .filter(|s| s.ntrs_id.is_some() && s.rejected.is_none())
                .filter(|s| !corpus::local_path(s).exists())
                .collect();
            let plan: Vec<Value> = todo
                .iter()
                .map(|s| {
                    let n = s.ntrs_id.as_deref().unwrap_or_default();
                    json!({"seed": s.id, "citation": format!("{}/api/citations/{n}", corpus::NTRS_API),
                        "pdf": corpus::ntrs_pdf(n).display().to_string()})
                })
                .collect();
            let psi_missing: Vec<&str> = seeds
                .iter()
                .filter(|s| s.psi_file.is_some() && s.rejected.is_none() && !corpus::local_path(s).exists())
                .map(|s| s.id.as_str())
                .collect();
            if !execute {
                return print(&json!({"dry_run": true, "network": "ntrs.nasa.gov (GET, public records)", "fetch": plan,
                    "psi_not_local": psi_missing, "hint": "re-run with --execute"}));
            }
            let http = ignis_data::sources::http::Http::new(corpus::USER_AGENT, 300);
            let mut out = Vec::new();
            for s in todo {
                let n = s.ntrs_id.as_deref().unwrap_or_default();
                out.push(match corpus::fetch_ntrs(&http, n) {
                    Ok(v) => v,
                    Err(e) => json!({"ntrs_id": n, "error": format!("{e:#}")}),
                });
            }
            print(&json!({"fetched": out, "psi_not_local": psi_missing}))
        }
        #[cfg(not(feature = "pdf"))]
        CorpusCmd::Ingest { .. } => bail!("corpus ingest needs the `pdf` feature: cargo run -p ignis-cli --features pdf -- evidence corpus ingest"),
        #[cfg(feature = "pdf")]
        CorpusCmd::Ingest { db } => {
            let db = db.unwrap_or_else(catalog::default_db);
            let rp = pdf::RunPatterns::load(&pdf::RunPatterns::default_path())?;
            let mut con = catalog::connect(&db, false)?;
            let rows = corpus::status(&seeds, Some(&con))?;
            let mut out = Vec::new();
            for (s, r) in seeds.iter().zip(&rows) {
                use corpus::CorpusStatus as S;
                match r.status {
                    S::Downloaded | S::Parsed | S::Indexed => {}
                    st => {
                        out.push(json!({"seed": s.id, "skipped": st, "note": r.note}));
                        continue;
                    }
                }
                let opts = pdf::ParseOptions { investigation_ids: s.investigations.clone(), ..Default::default() };
                let mut v = ingest_one(&mut con, &corpus::local_path(s), &opts, &rp, Some(&s.families), s.outcome_free.as_deref())?;
                v["seed"] = json!(s.id);
                out.push(v);
            }
            print(&json!({"db": db.display().to_string(), "ingested": out}))
        }
        CorpusCmd::Lockbox { db } => {
            let svc = open_service(db, true)?;
            let scope = local_research_scope();
            let rows = service::withheld_by_publication(&svc, &scope)?;
            let titles: std::collections::HashMap<String, Option<String>> =
                svc.snapshot().members.iter().map(|m| (m.publication_id.clone(), m.title.clone())).collect();
            let out: Vec<Value> = rows
                .iter()
                .map(|(p, (n, w))| json!({"publication_id": p, "title": titles.get(p).cloned().flatten(),
                    "passages": n, "withheld": w, "readable": n - w}))
                .collect();
            print(&json!({"scope": scope.scope_id, "partitions": scope.partitions, "publications": out}))
        }
        CorpusCmd::Snapshot { db } => {
            let db = db.unwrap_or_else(catalog::default_db);
            let con = catalog::connect(&db, true)?;
            let snap = corpus::snapshot(&con)?;
            let path = corpus::corpus_dir().join("snapshots").join(format!("{}.json", snap.snapshot_id));
            std::fs::create_dir_all(path.parent().unwrap())?;
            std::fs::write(&path, serde_json::to_string_pretty(&snap)? + "\n")?;
            print(&json!({"snapshot": path.display().to_string(), "snapshot_id": snap.snapshot_id,
                "members": snap.members.len(), "embedding_set": snap.embedding_set}))
        }
    }
}

#[cfg(feature = "pdf")]
fn walk_pdfs(dir: &std::path::Path, out: &mut Vec<PathBuf>) -> Result<()> {
    for e in std::fs::read_dir(dir)? {
        let p = e?.path();
        if p.is_dir() {
            walk_pdfs(&p, out)?;
        } else if p.extension().is_some_and(|x| x.eq_ignore_ascii_case("pdf")) {
            out.push(p);
        }
    }
    Ok(())
}
