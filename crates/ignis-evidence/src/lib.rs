//! Scientific retrieval (V3.1 §18): PDF parsing with page/char locators (MuPDF), SQLite/FTS5
//! catalog, dense Qwen3 embeddings and bounded Qwen3 reranking served by the DeepInfra API, RRF
//! fusion, and the leak firewall.
//!
//! Evidence content is explanation, never a predictor input: [`firewall::scan`] proves from the
//! cargo dependency graph that the scientific crates do not depend on this crate, and
//! [`firewall::strip_for_sealed`] strips outcome-bearing chunks for runs under sealed evaluation.
//!
//! Commands (`ignis evidence ...`, JSON on stdout): `ingest`, `embed [--execute] [--force]`,
//! `search`, `firewall`. The API key is read only from `DEEPINFRA_API_KEY`.

pub mod catalog;
pub mod deepinfra;
pub mod dense;
pub mod firewall;
#[cfg(feature = "pdf")]
pub mod pdf;
pub mod records;
pub mod search;

use std::collections::HashMap;
use std::path::PathBuf;
#[cfg(feature = "pdf")]
use std::time::Instant;

use anyhow::{bail, Context, Result};
use clap::Subcommand;
use ignis_core::resources::{guard, load_profile, Guard, Job};
use serde_json::{json, Value};

pub const PARSER_VERSION: &str = "0.2.0-mupdf";

#[derive(Debug, Subcommand)]
pub enum EvidenceCmd {
    /// Parse PDFs (page, char spans, captions) and upsert publications + chunks into the catalog.
    Ingest {
        /// a PDF or a directory of PDFs (NTRS `<stem>.json` sidecars are read)
        path: PathBuf,
        #[arg(long)]
        db: Option<PathBuf>,
        /// investigation ids to attach, e.g. PSI-25
        #[arg(long)]
        investigation: Vec<String>,
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
    /// FTS5 + dense (if embedded and a key is set) -> RRF k=60 -> API rerank top-30 (fallback RRF). Prints locators.
    Search {
        query: String,
        #[arg(long)]
        db: Option<PathBuf>,
        #[arg(long, default_value_t = 5)]
        limit: usize,
        #[arg(long)]
        family: Option<String>,
        #[arg(long)]
        investigation: Option<String>,
        #[arg(long)]
        publication_id: Option<String>,
        #[arg(long)]
        kind: Option<String>,
        #[arg(long)]
        year_min: Option<i64>,
        #[arg(long)]
        year_max: Option<i64>,
        #[arg(long)]
        no_dense: bool,
        #[arg(long)]
        no_rerank: bool,
        /// override reranker top_n (default 30)
        #[arg(long)]
        rerank_top: Option<usize>,
        /// seconds; on timeout the order stays RRF
        #[arg(long)]
        rerank_timeout: Option<f64>,
        /// strip outcome-bearing evidence for these sealed runs
        #[arg(long)]
        sealed_run: Vec<String>,
        #[arg(long)]
        full_text: bool,
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

pub fn run(cmd: EvidenceCmd) -> Result<()> {
    match cmd {
        #[cfg(not(feature = "pdf"))]
        EvidenceCmd::Ingest { .. } => anyhow::bail!(
            "PDF ingest needs the `pdf` feature (MuPDF): cargo build -p ignis-cli --features pdf              (on Windows set CLANG_PATH to LLVM's clang.exe and build from a VS vcvars64 shell)"
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
                let t = Instant::now();
                let (publ, chunks, rep) = pdf::parse_pdf(p, &opts, &rp)?;
                let n = catalog::upsert_publication(&mut con, &publ, &chunks)?;
                out.push(json!({"file": publ.file_name, "publication_id": publ.id, "title": publ.title, "pages": publ.n_pages,
                    "chunks": n, "families": publ.families, "report": rep, "seconds": t.elapsed().as_secs_f64()}));
            }
            print(&json!({"db": db.display().to_string(), "ingested": out}))
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
        EvidenceCmd::Search {
            query, db, limit, family, investigation, publication_id, kind, year_min, year_max, no_dense, no_rerank,
            rerank_top, rerank_timeout, sealed_run, full_text,
        } => {
            let db = db.unwrap_or_else(catalog::default_db);
            let filters = catalog::Filters { family, investigation, publication_id, kind, year_min, year_max, ..Default::default() };
            let mut r = search::Retriever::open(&db, cfg()?)?;
            let opts = search::SearchOpts { limit, use_dense: !no_dense, use_rerank: !no_rerank, rerank_top, rerank_timeout_s: rerank_timeout };
            let mut res = r.search(&query, &filters, &opts)?;
            // API calls made by this search go to the cost ledger
            if res["usage"].as_object().is_some_and(|u| !u.is_empty()) {
                let prof = load_profile("cpu-local")?;
                let mut job = Job::start("evidence.search", &prof, json!({"query_chars": query.chars().count(), "limit": limit}))?;
                let sum = |k: &str| res["usage"].as_object().map(|u| u.values().filter_map(|x| x[k].as_f64()).sum::<f64>()).unwrap_or(0.0);
                job.set_api_cost(sum("estimated_usd"), sum("billed_usd"));
                res["job"] = json!(job.id());
                job.finish(json!({"stages": res["stages"], "usage": res["usage"], "timings": res["timings"]}));
            }
            if !sealed_run.is_empty() {
                let items: Vec<Value> = res["results"].as_array().cloned().unwrap_or_default();
                let (kept, stripped) = firewall::strip_for_sealed(items, &sealed_run, &HashMap::new(), true);
                res["results"] = json!(kept);
                res["stripped_for_sealed"] = json!(stripped);
            }
            if !full_text {
                for r in res["results"].as_array_mut().into_iter().flatten() {
                    let t = r["text"].as_str().unwrap_or_default();
                    let short: String = t.chars().take(280).collect::<String>().replace('\n', " ");
                    let more = t.chars().count() > 280;
                    r["text"] = json!(if more { format!("{short}...") } else { short });
                }
            }
            print(&res)
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
