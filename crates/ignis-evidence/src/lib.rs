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

pub mod attribution;
pub mod catalog;
pub mod combined;
pub mod corpus;
pub mod deepinfra;
pub mod dense;
pub mod docling_view;
pub mod firewall;
pub mod graph;
pub mod labels;
pub mod llm_pass;
pub mod outline;
#[cfg(feature = "pdf")]
pub mod pdf;
pub mod records;
pub mod retrieval_eval;
pub mod search;
pub mod service;
pub mod sira;
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

/// `ignis evidence graph ...` (R9b).
#[derive(Debug, Subcommand)]
pub enum GraphCmd {
    /// Build the combustion graph next to the catalog (`graph.sqlite`) from the PSI manifests, the
    /// conditions table (`ignis data conditions`), the catalog's active parse views and the R8
    /// labels when present; prints the build report (links per relation, gaps that are not links).
    Build {
        #[arg(long)]
        db: Option<PathBuf>,
        /// Default: `data/manifests`.
        #[arg(long)]
        manifests: Option<PathBuf>,
        /// Default: the table `ignis data conditions` wrote.
        #[arg(long)]
        conditions: Option<PathBuf>,
        /// Default: `<catalog dir>/graph.sqlite`.
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// One `graph_neighbors` page (a `GraphNeighborsRequest` as JSON) under the local research
    /// scope, through the same service as the HTTP API and the MCP tool.
    Neighbors {
        request: String,
        #[arg(long)]
        db: Option<PathBuf>,
    },
}

#[derive(Debug, Subcommand)]
pub enum EvidenceCmd {
    #[command(subcommand)]
    Corpus(CorpusCmd),
    #[command(subcommand)]
    Graph(GraphCmd),
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
    /// Import a Docling parse view (`sidecar/docling/ignis_docling.py` output) of a PDF as its active
    /// view; other views of the PDF stay stored, so their citations still resolve. Refused if the PDF
    /// bytes differ from the view's source hash or the view has failed pages.
    ImportView {
        /// the view JSON (Docling or PDFium)
        view: PathBuf,
        /// the PDF the view was made from (its bytes are hashed against the view)
        #[arg(long)]
        pdf: PathBuf,
        #[arg(long)]
        db: Option<PathBuf>,
        /// investigation ids to attach (the storage location adds its own)
        #[arg(long)]
        investigation: Vec<String>,
    },
    /// Import every view under a directory (`<root>/**/<inv>/<stem>.{pdfium,docling}.json`) whose
    /// PDF is found under `--pdf-root` as `<inv>/<file_name>`. PDFium views go first, so a Docling
    /// view of the same PDF ends active. A view already stored is skipped (incremental); a refused
    /// view is reported and the others still import.
    ImportViews {
        /// directories of views (repeatable), e.g. data/views/pdfium-5.15.2 data/views/docling-2.150.1
        #[arg(required = true)]
        roots: Vec<PathBuf>,
        /// where the PDFs are, as `<inv>/<file name>`
        #[arg(long)]
        pdf_root: PathBuf,
        #[arg(long)]
        db: Option<PathBuf>,
        /// re-import views already stored
        #[arg(long)]
        force: bool,
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
        /// SIRA query expansion: a term or phrase a relevant passage would contain (repeatable)
        #[arg(long = "expect")]
        expect: Vec<String>,
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
    /// SIRA document expansion (R12b): load a page-card pack (pages.jsonl + MANIFEST.json, keyed by
    /// NASA file sha256 and page) into the expansion index; raises outcome_bearing where the pack
    /// protects a page, never lowers it; then rebuilds the term statistics.
    Enrich {
        /// the pack directory (e.g. /srv/ignis/data/enrichment/docs-v1)
        pack: PathBuf,
        #[arg(long)]
        db: Option<PathBuf>,
        /// DF ceiling: expansion words held by more than this share of chunks are dropped
        #[arg(long, default_value_t = sira::DEFAULT_TAU)]
        tau: f64,
    },
    /// Corpus statistics: rebuild the term table (`--build`), or show how many chunks hold each
    /// expression and the query finalizer's choice among them.
    Terms {
        expressions: Vec<String>,
        #[arg(long)]
        db: Option<PathBuf>,
        #[arg(long)]
        build: bool,
    },
    /// Section outlines (R7b) from the active parse views: one source's tree as JSON, or with
    /// `--all` a report of every publication (method, sections, depth) written as JSONL.
    Outline {
        /// Source ids (`sha256:<pdf>`); none with `--all`.
        sources: Vec<String>,
        #[arg(long)]
        all: bool,
        /// Levels to print below the document.
        #[arg(long)]
        depth: Option<u32>,
        /// Report path for `--all` (default `<catalog dir>/outline_report.jsonl`).
        #[arg(long)]
        report: Option<PathBuf>,
        #[arg(long)]
        db: Option<PathBuf>,
    },
    /// Section summaries (R7b): build the requests and print the cost estimate; with `--execute`,
    /// call the model (`claude -p`) and append answers to `<catalog dir>/outline_summaries.jsonl`,
    /// which `sources_outline` attaches as generated text. Spend needs the user's approval.
    OutlineSummaries {
        sources: Vec<String>,
        #[arg(long)]
        all: bool,
        /// Summarise parents too (their pages repeat their children's).
        #[arg(long)]
        parents: bool,
        #[arg(long, default_value = "haiku")]
        model: String,
        /// Price per million input / output tokens for the estimate (USD).
        #[arg(long, default_value_t = 1.0)]
        usd_per_mtok_in: f64,
        #[arg(long, default_value_t = 5.0)]
        usd_per_mtok_out: f64,
        /// Where the requests are written for review.
        #[arg(long)]
        requests: Option<PathBuf>,
        #[arg(long)]
        db: Option<PathBuf>,
        #[arg(long)]
        execute: bool,
    },
    /// Document labels (R8), two passes per document: build the requests and print the cost
    /// estimate; with `--execute`, run the missing passes (`claude -p`) into
    /// `<catalog dir>/label_answers.jsonl`. Then (always) combine the answers present into
    /// `<catalog dir>/source_labels.jsonl` (SourceLabel JSONL) and print the counts.
    Labels {
        sources: Vec<String>,
        #[arg(long)]
        all: bool,
        #[arg(long, default_value = "haiku")]
        model: String,
        #[arg(long, default_value_t = 1.0)]
        usd_per_mtok_in: f64,
        #[arg(long, default_value_t = 5.0)]
        usd_per_mtok_out: f64,
        #[arg(long)]
        db: Option<PathBuf>,
        #[arg(long)]
        execute: bool,
    },
    /// Gold set (R8): select about N documents stratified across investigations and kinds and
    /// write them as a labelling template with EMPTY labels (a person fills them in).
    GoldSelect {
        #[arg(long, default_value_t = 100)]
        n: usize,
        #[arg(long, default_value_t = 2026)]
        seed: u64,
        /// Default `data/labels/gold/gold_template.jsonl`; an existing file is never overwritten.
        #[arg(long)]
        out: Option<PathBuf>,
        #[arg(long)]
        db: Option<PathBuf>,
    },
    /// Score a labeller's SourceLabel JSONL against the person-labelled rows of a gold file.
    GoldEval {
        #[arg(long)]
        gold: PathBuf,
        /// Default `<catalog dir>/source_labels.jsonl`.
        #[arg(long)]
        labels: Option<PathBuf>,
        #[arg(long)]
        db: Option<PathBuf>,
    },
    /// The combined model pass (R7b summaries + R8 label pass a + R12b expansion vocabulary), one
    /// call per leaf section: requests + cost estimate; `--execute` runs it and splits the answers
    /// into `outline_summaries.jsonl`, `label_answers.jsonl` and an expansion pack
    /// (`<catalog dir>/expansion-llm/`, load with `ignis evidence enrich`). Needs approved spend.
    LlmPass {
        sources: Vec<String>,
        #[arg(long)]
        all: bool,
        #[arg(long, default_value = "haiku")]
        model: String,
        #[arg(long, default_value_t = 1.0)]
        usd_per_mtok_in: f64,
        #[arg(long, default_value_t = 5.0)]
        usd_per_mtok_out: f64,
        #[arg(long)]
        db: Option<PathBuf>,
        #[arg(long)]
        execute: bool,
    },
    /// Retrieval evaluation and τ/w sweep (R12b) on the T10 **dev** tasks: nDCG@10 and recall@10
    /// per question type for plain BM25, query expansion (`--expected-terms`), document expansion
    /// and both. Refuses `holdout/`, any non-dev task and a scope that may read the test
    /// partition. Writes `report.json`, `report.md` and `proposal.json` (never applied).
    RetrievalEval {
        /// GoldTask array (`benchmarks/t10/dev/tasks.json`), GoldSet or JSONL; dev split only.
        #[arg(long)]
        tasks: PathBuf,
        #[arg(long)]
        db: Option<PathBuf>,
        /// Expansion pack (`<catalog dir>/expansion-llm`); needed to sweep τ (re-enriches a copy).
        #[arg(long)]
        pack: Option<PathBuf>,
        /// `{"<task_id>": [ExpectedTerm, ...]}` from the agent's sketches; enables the query arms.
        #[arg(long)]
        expected_terms: Option<PathBuf>,
        /// τ grid, comma separated (default 0.01,0.02,0.05,0.1,0.2 with --pack).
        #[arg(long)]
        tau: Option<String>,
        /// w grid, comma separated (default 0.1,0.2,0.4,0.6,0.8,1,2,4).
        #[arg(long)]
        w: Option<String>,
        /// Output directory (default `<catalog dir>/retrieval_eval`).
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Dependency-graph check: protected scientific crates must not depend on ignis-evidence.
    Firewall {
        #[arg(long, default_value = "Cargo.toml")]
        manifest: PathBuf,
    },
}

/// The publications named by source id (`sha256:<hex>` or the bare hex), or every one with `all`.
fn select_pubs(
    con: &rusqlite::Connection,
    sources: &[String],
    all: bool,
) -> Result<Vec<records::Publication>> {
    let pubs = catalog::publications(con)?;
    if all {
        return Ok(pubs);
    }
    if sources.is_empty() {
        bail!("name sources (sha256:<pdf>) or pass --all");
    }
    sources
        .iter()
        .map(|s| {
            let sha = s.trim_start_matches("sha256:");
            pubs.iter()
                .find(|p| p.source_sha256 == sha)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("{s} is not in the catalog"))
        })
        .collect()
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
        EvidenceCmd::Graph(GraphCmd::Build { db, manifests, conditions, out }) => {
            let db = db.unwrap_or_else(catalog::default_db);
            let con = catalog::connect(&db, true)?;
            let manifests = manifests.unwrap_or_else(ignis_data::paths::manifests);
            let cpath = conditions.unwrap_or_else(ignis_data::conditions::table_path);
            let table: Option<ignis_core::contracts::ConditionsTable> = if cpath.exists() {
                Some(serde_json::from_str(&std::fs::read_to_string(&cpath)?).with_context(|| cpath.display().to_string())?)
            } else {
                None
            };
            let labels = labels::read_labels(&db.with_file_name(service::LABELS_FILE))?;
            let lockbox = service::Lockbox::load()?;
            let out = out.unwrap_or_else(|| graph::path_for(&db));
            let report = graph::build(&out, &graph::Inputs {
                catalog: &con,
                manifests: Some(&manifests),
                conditions: table.as_ref(),
                labels: &labels,
                aliases: lockbox.aliases(),
                origin: ignis_core::contracts::Origin::System,
            })?;
            print(&report)
        }
        EvidenceCmd::Graph(GraphCmd::Neighbors { request, db }) => {
            let req: ignis_core::contracts::GraphNeighborsRequest = serde_json::from_str(&request).context("request")?;
            emit(open_service(db, true)?.graph_neighbors(&local_research_scope(), &req))
        }
        EvidenceCmd::ImportView {
            view,
            pdf,
            db,
            investigation,
        } => {
            let db = db.unwrap_or_else(catalog::default_db);
            let v: docling_view::DoclingView = serde_json::from_str(&std::fs::read_to_string(&view)?)?;
            let bytes = std::fs::read(&pdf)?;
            let rp = attribution::RunPatterns::load(&attribution::RunPatterns::default_path())?;
            let iv = docling_view::import(&v, &pdf, &bytes, &rp, &investigation)?;
            let mut con = catalog::connect(&db, false)?;
            let n = docling_view::store(&mut con, &iv)?;
            catalog::record_location(&con, &iv.publication.source_sha256, &pdf.display().to_string())?;
            print(&json!({"db": db.display().to_string(), "publication_id": iv.publication.id,
                "parse_view_id": records::parse_view_id(&iv.publication.id, &iv.publication.parser_version),
                "active": catalog::active_parser(&con, &iv.publication.id)?,
                "chunks": n, "views": catalog::views(&con, &iv.publication.id)?}))
        }
        EvidenceCmd::ImportViews {
            roots,
            pdf_root,
            db,
            force,
        } => {
            let db = db.unwrap_or_else(catalog::default_db);
            let rp = attribution::RunPatterns::load(&attribution::RunPatterns::default_path())?;
            print(&import_views(&roots, &pdf_root, &db, &rp, force)?)
        }
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
                if p.parser_version.starts_with("docling-") || p.parser_version.starts_with("pdfium-") {
                    // Sidecar views: spans, line boxes and page bounds are checked at import
                    // (`docling_view::import`); here the stored bytes must still be the view's source.
                    let ok = ignis_core::hashing::sha256_hex(&bytes) == p.source_sha256;
                    if !ok {
                        failed += 1;
                    }
                    out.push(json!({"publication_id": p.id, "source_id": format!("sha256:{}", p.source_sha256),
                        "parse_view_id": records::parse_view_id(&p.id, &p.parser_version),
                        "bytes_match": ok, "check": "source hash; spans and boxes checked at import", "passed": ok}));
                    continue;
                }
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
        EvidenceCmd::Search { query, db, mode, page_size, cursor, family, investigation, source, offline, expect } => {
            let svc = open_service(db, offline)?;
            let mode: SearchMode = serde_json::from_value(json!(mode)).context("--mode hybrid|lexical|dense")?;
            let families = family
                .iter()
                .map(|f| serde_json::from_value::<Family>(json!(f)).with_context(|| format!("--family {f}")))
                .collect::<Result<Vec<_>>>()?;
            let req = SearchRequest {
                response_format: None,
                schema_version: if expect.is_empty() { EvidenceSchema::V1 } else { EvidenceSchema::V2 },
                expected_terms: expect
                    .iter()
                    .map(|t| ignis_core::contracts::ExpectedTerm { term: t.clone(), importance: None })
                    .collect(),
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
            let req = ReadRequest { schema_version: EvidenceSchema::V1, source_id: SourceId::new(source)?, parse_view_id: None, range, response_format: None };
            emit(svc.read(&local_research_scope(), &req))
        }
        EvidenceCmd::Resolve { evidence_id, db } => {
            let svc = open_service(db, true)?;
            emit(svc.resolve_evidence(&local_research_scope(), &EvidenceId::new(evidence_id)?))
        }
        EvidenceCmd::Enrich { pack, db, tau } => {
            let db = db.unwrap_or_else(catalog::default_db);
            let mut con = catalog::connect(&db, false)?;
            let expansion = sira::enrich(&mut con, &pack, tau)?;
            let terms = sira::build_term_stats(&mut con)?;
            print(&json!({"db": db.display().to_string(), "expansion": expansion, "term_stats": terms}))
        }
        EvidenceCmd::Terms { expressions, db, build } => {
            let db = db.unwrap_or_else(catalog::default_db);
            let mut con = catalog::connect(&db, !build)?;
            let built = if build { Some(sira::build_term_stats(&mut con)?) } else { None };
            let proposals: Vec<(String, f64)> = expressions.iter().map(|e| (e.clone(), 1.0)).collect();
            let choice = sira::select_expressions(&con, &proposals)?;
            print(&json!({"db": db.display().to_string(), "built": built, "expressions": choice}))
        }
        EvidenceCmd::Outline { sources, all, depth, report, db } => {
            let db = db.unwrap_or_else(catalog::default_db);
            let con = catalog::connect(&db, true)?;
            let pubs = select_pubs(&con, &sources, all)?;
            let mut lines = Vec::new();
            let mut by_method = std::collections::BTreeMap::<&str, usize>::new();
            for p in &pubs {
                let chunks = outline::load_chunks(&con, &p.id)?;
                let (heads, method) = outline::headings(&chunks);
                let title = p.title.clone().or_else(|| p.file_name.clone()).unwrap_or_default();
                let roots = outline::build(&heads, p.n_pages, &title)?;
                *by_method.entry(method.as_str()).or_default() += 1;
                if all {
                    lines.push(json!({"source_id": format!("sha256:{}", p.source_sha256), "file": p.file_name,
                        "pages": p.n_pages, "method": method.as_str(), "sections": outline::count(&roots),
                        "top_level": roots.len()}));
                } else {
                    let nodes = outline::select(roots, None, depth).unwrap_or_default();
                    print(&json!({"source_id": format!("sha256:{}", p.source_sha256), "method": method.as_str(), "nodes": nodes}))?;
                }
            }
            if all {
                let path = report.unwrap_or_else(|| db.with_file_name("outline_report.jsonl"));
                let mut text = String::new();
                for l in &lines {
                    text.push_str(&l.to_string());
                    text.push('\n');
                }
                std::fs::write(&path, text)?;
                print(&json!({"publications": pubs.len(), "by_method": by_method, "report": path.display().to_string()}))?;
            }
            Ok(())
        }
        EvidenceCmd::OutlineSummaries { sources, all, parents, model, usd_per_mtok_in, usd_per_mtok_out, requests, db, execute } => {
            let db = db.unwrap_or_else(catalog::default_db);
            let con = catalog::connect(&db, true)?;
            let pubs = select_pubs(&con, &sources, all)?;
            let reqs = outline::summary_requests(&con, &pubs, !parents)?;
            let req_path = requests.unwrap_or_else(|| db.with_file_name("outline_summary_requests.jsonl"));
            llm_pass::write_requests(&req_path, &reqs)?;
            let est = llm_pass::estimate(&reqs, usd_per_mtok_in, usd_per_mtok_out);
            let out = db.with_file_name(service::SUMMARIES_FILE);
            print(&json!({"publications": pubs.len(), "requests_file": req_path.display().to_string(),
                "estimate": est, "model": model, "answers": out.display().to_string(),
                "status": if execute { "running" } else { "dry run: nothing was sent to a model; add --execute after the spend is approved" }}))?;
            if execute {
                let r = llm_pass::run(&reqs, &llm_pass::ClaudeCli::new(&model), &out, 5)?;
                print(&serde_json::to_value(&r)?)?;
            }
            Ok(())
        }
        EvidenceCmd::Labels { sources, all, model, usd_per_mtok_in, usd_per_mtok_out, db, execute } => {
            let db = db.unwrap_or_else(catalog::default_db);
            let con = catalog::connect(&db, true)?;
            let pubs = select_pubs(&con, &sources, all)?;
            let reqs = labels::label_requests(&con, &pubs)?;
            let answers_path = db.with_file_name(service::LABEL_ANSWERS_FILE);
            let done: BTreeSet<String> = llm_pass::read_answers(&answers_path)?.into_iter().map(|a| a.custom_id).collect();
            let todo: Vec<llm_pass::Request> = reqs.iter().filter(|r| !done.contains(&r.custom_id)).cloned().collect();
            let req_path = db.with_file_name("label_requests.jsonl");
            llm_pass::write_requests(&req_path, &todo)?;
            print(&json!({"publications": pubs.len(), "passes_to_run": todo.len(), "passes_done": reqs.len() - todo.len(),
                "requests_file": req_path.display().to_string(), "model": model,
                "estimate": llm_pass::estimate(&todo, usd_per_mtok_in, usd_per_mtok_out),
                "status": if execute { "running" } else { "dry run: nothing was sent to a model; add --execute after the spend is approved" }}))?;
            if execute {
                let r = llm_pass::run(&todo, &llm_pass::ClaudeCli::new(&model), &answers_path, 5)?;
                print(&serde_json::to_value(&r)?)?;
            }
            let built = labels::build_labels(&llm_pass::read_answers(&answers_path)?)?;
            let out = db.with_file_name(service::LABELS_FILE);
            let mut text = String::new();
            for l in &built.labels {
                text.push_str(&serde_json::to_string(l)?);
                text.push('\n');
            }
            std::fs::write(&out, text)?;
            print(&json!({"labels": out.display().to_string(), "counts": built.counts, "problems": built.problems}))
        }
        EvidenceCmd::GoldSelect { n, seed, out, db } => {
            let db = db.unwrap_or_else(catalog::default_db);
            let con = catalog::connect(&db, true)?;
            let pubs = catalog::publications(&con)?;
            let rows = labels::gold_select(&pubs, n, seed);
            let out = out.unwrap_or_else(|| PathBuf::from("data/labels/gold/gold_template.jsonl"));
            if out.exists() {
                bail!("{} exists: a gold file may hold labels written by a person; choose another --out", out.display());
            }
            if let Some(d) = out.parent() {
                std::fs::create_dir_all(d)?;
            }
            let mut text = String::new();
            for r in &rows {
                text.push_str(&serde_json::to_string(r)?);
                text.push('\n');
            }
            std::fs::write(&out, text)?;
            let mut strata = std::collections::BTreeMap::<String, usize>::new();
            for r in &rows {
                *strata.entry(format!("{} / {}", r.stratum_investigation, r.stratum_kind)).or_default() += 1;
            }
            print(&json!({"template": out.display().to_string(), "documents": rows.len(), "of": pubs.len(), "seed": seed, "strata": strata,
                "next": "a person fills doc_type, families, investigation_ids, runs, labelled_by, labelled_at; then `ignis evidence gold-eval --gold <file>`"}))
        }
        EvidenceCmd::GoldEval { gold, labels: labels_path, db } => {
            let db = db.unwrap_or_else(catalog::default_db);
            let labels_path = labels_path.unwrap_or_else(|| db.with_file_name(service::LABELS_FILE));
            let g = labels::read_gold(&gold)?;
            let l = labels::read_labels(&labels_path)?;
            print(&labels::to_value(&labels::gold_eval(&g, &l)?))
        }
        EvidenceCmd::LlmPass { sources, all, model, usd_per_mtok_in, usd_per_mtok_out, db, execute } => {
            let db = db.unwrap_or_else(catalog::default_db);
            let con = catalog::connect(&db, true)?;
            let pubs = select_pubs(&con, &sources, all)?;
            let reqs = combined::combined_requests(&con, &pubs)?;
            let req_path = db.with_file_name("combined_requests.jsonl");
            llm_pass::write_requests(&req_path, &reqs)?;
            let answers = db.with_file_name("combined_answers.jsonl");
            print(&json!({"publications": pubs.len(), "requests_file": req_path.display().to_string(), "model": model,
                "estimate": llm_pass::estimate(&reqs, usd_per_mtok_in, usd_per_mtok_out),
                "then": "label pass b: `ignis evidence labels --all --execute` (runs only the missing passes)",
                "status": if execute { "running" } else { "dry run: nothing was sent to a model; add --execute after the spend is approved" }}))?;
            if execute {
                let r = llm_pass::run(&reqs, &llm_pass::ClaudeCli::new(&model), &answers, 5)?;
                print(&serde_json::to_value(&r)?)?;
                let s = combined::split(
                    &llm_pass::read_answers(&answers)?,
                    &db.with_file_name(service::SUMMARIES_FILE),
                    &db.with_file_name("expansion-llm"),
                    &db.with_file_name(service::LABEL_ANSWERS_FILE),
                )?;
                print(&serde_json::to_value(&s)?)?;
            }
            Ok(())
        }
        EvidenceCmd::RetrievalEval { tasks, db, pack, expected_terms, tau, w, out } => {
            let db = db.unwrap_or_else(catalog::default_db);
            let scope = local_research_scope();
            retrieval_eval::check_scope(&scope)?;
            let svc = service::EvidenceService::open(&db, cfg()?, service::Lockbox::load()?)?.offline();
            let withheld = |id: &str| svc.withholds_chunk(&scope, id);
            let taus = tau.as_deref().map(retrieval_eval::parse_list).transpose()?;
            let ws = match w.as_deref() {
                Some(s) => retrieval_eval::parse_list(s)?,
                None => retrieval_eval::DEFAULT_WS.to_vec(),
            };
            let out = out.unwrap_or_else(retrieval_eval::default_out);
            let args = retrieval_eval::SweepArgs {
                tasks: &tasks,
                db: &db,
                pack: pack.as_deref(),
                expected_terms: expected_terms.as_deref(),
                taus,
                ws,
                out: &out,
            };
            let r = retrieval_eval::sweep(&args, &scope, &withheld)?;
            print(&json!({"out": out.display().to_string(), "origin": r.origin,
                "scored": r.preparation.tasks.len(), "excluded": r.preparation.excluded.len(),
                "baseline": r.baseline.arms, "proposal": r.proposal}))
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

/// Batch form of `import-view` (see [`EvidenceCmd::ImportViews`]).
pub fn import_views(
    roots: &[PathBuf],
    pdf_root: &std::path::Path,
    db: &std::path::Path,
    rp: &attribution::RunPatterns,
    force: bool,
) -> Result<Value> {
    fn walk(dir: &std::path::Path, ext: &str, out: &mut Vec<PathBuf>) -> Result<()> {
        for e in std::fs::read_dir(dir).with_context(|| format!("read {}", dir.display()))? {
            let p = e?.path();
            if p.is_dir() {
                walk(&p, ext, out)?;
            } else if p.to_string_lossy().ends_with(ext) {
                out.push(p);
            }
        }
        Ok(())
    }
    let mut views = Vec::new();
    for ext in [".pdfium.json", ".docling.json"] {
        let mut v = Vec::new();
        for r in roots {
            walk(r, ext, &mut v)?;
        }
        v.sort();
        views.extend(v);
    }
    let mut con = catalog::connect(db, false)?;
    let t = std::time::Instant::now();
    let (mut imported, mut skipped, mut chunks) = (0usize, 0usize, 0usize);
    let mut refused = Vec::new();
    for (i, path) in views.iter().enumerate() {
        let mut one = || -> Result<Option<usize>> {
            let v: docling_view::DoclingView =
                serde_json::from_str(&std::fs::read_to_string(path)?)?;
            let pub_id = format!("pub-{}", &v.source.sha256[..16.min(v.source.sha256.len())]);
            let view_id = records::parse_view_id(&pub_id, &docling_view::parser_version(&v));
            if !force && catalog::views(&con, &pub_id)?.contains(&view_id) {
                return Ok(None);
            }
            let inv = path
                .parent()
                .and_then(|p| p.file_name())
                .context("a view sits in its investigation's directory")?;
            let pdf = pdf_root.join(inv).join(&v.source.file_name);
            let bytes = std::fs::read(&pdf).with_context(|| format!("read {}", pdf.display()))?;
            let iv = docling_view::import(&v, &pdf, &bytes, rp, &[])?;
            let n = docling_view::store(&mut con, &iv)?;
            catalog::record_location(
                &con,
                &iv.publication.source_sha256,
                &pdf.display().to_string(),
            )?;
            Ok(Some(n))
        };
        match one() {
            Ok(Some(n)) => {
                imported += 1;
                chunks += n;
            }
            Ok(None) => skipped += 1,
            Err(e) => {
                refused.push(json!({"view": path.display().to_string(), "error": format!("{e:#}")}))
            }
        }
        if (i + 1) % 100 == 0 {
            eprintln!(
                "import-views: {}/{} ({imported} imported, {skipped} already stored, {} refused) {:.0} s",
                i + 1,
                views.len(),
                refused.len(),
                t.elapsed().as_secs_f64()
            );
        }
    }
    Ok(
        json!({"db": db.display().to_string(), "views": views.len(), "imported": imported,
        "already_stored": skipped, "chunks": chunks, "refused": refused,
        "seconds": t.elapsed().as_secs_f64()}),
    )
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
