//! Hybrid retrieval: FTS5 lexical + exact dense -> RRF (k=60) -> bounded rerank with declared fallback.
//!
//! Every result carries its per-retriever ranks, the fused RRF score, the reranker score when it ran,
//! and the exact page/char locator. The response states which stages ran and why others did not,
//! and how long each took. The query embedding and the rerank scores come from the DeepInfra API;
//! without a key (`DEEPINFRA_API_KEY`) or network, dense is skipped and rerank falls back to RRF with
//! an explicit status, and lexical FTS5 still answers. [`Retriever`] keeps the vector matrix and the
//! HTTP client in memory, so a long-lived caller pays the load once.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;

use anyhow::Result;
use rusqlite::Connection;
use serde_json::{json, Value};

use crate::catalog::{self, Filters};
use crate::deepinfra::{redact, ApiKey, ApiUsage, Client};
use crate::dense::{self, RetrievalCfg, VectorStore};

/// RRF(d) = sum_m 1 / (k + r_m(d)), ranks 1-based; absence from a list contributes zero.
/// Ties are broken by id.
pub fn rrf(rankings: &[&[String]], k: f64) -> Vec<(String, f64)> {
    let mut score: HashMap<&str, f64> = HashMap::new();
    for ids in rankings {
        for (r, d) in ids.iter().enumerate() {
            *score.entry(d.as_str()).or_default() += 1.0 / (k + (r + 1) as f64);
        }
    }
    let mut v: Vec<(String, f64)> = score.into_iter().map(|(d, s)| (d.to_string(), s)).collect();
    v.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    v
}

#[derive(Debug, Clone)]
pub struct SearchOpts {
    pub limit: usize,
    pub use_dense: bool,
    pub use_rerank: bool,
    pub rerank_top: Option<usize>,
    pub rerank_timeout_s: Option<f64>,
}

impl Default for SearchOpts {
    fn default() -> Self {
        SearchOpts {
            limit: 10,
            use_dense: true,
            use_rerank: true,
            rerank_top: None,
            rerank_timeout_s: None,
        }
    }
}

/// Catalog + lazily loaded vectors and API client (kept for the life of the value).
pub struct Retriever {
    pub cfg: RetrievalCfg,
    pub db: PathBuf,
    pub con: Connection,
    /// from `DEEPINFRA_API_KEY` at open; None = offline (lexical only, declared)
    pub api_key: Option<ApiKey>,
    vectors: Option<Result<Arc<VectorStore>, String>>,
    client: Option<Arc<Client>>,
}

fn ms(t: Instant) -> f64 {
    (t.elapsed().as_secs_f64() * 1e4).round() / 10.0
}

impl Retriever {
    pub fn open(db: &Path, cfg: RetrievalCfg) -> Result<Retriever> {
        let api_key = ApiKey::from_env();
        Ok(Retriever {
            cfg,
            db: db.to_path_buf(),
            con: catalog::connect(db, true)?,
            api_key,
            vectors: None,
            client: None,
        })
    }

    fn vectors(&mut self) -> Result<Arc<VectorStore>, String> {
        let dir = self.db.parent().unwrap_or(Path::new(".")).to_path_buf();
        self.vectors
            .get_or_insert_with(|| {
                if !VectorStore::exists(&dir) {
                    return Err("no vector file (run `ignis evidence embed --execute`)".into());
                }
                VectorStore::load(&dir)
                    .map(Arc::new)
                    .map_err(|e| e.to_string())
            })
            .clone()
    }

    fn client(&mut self) -> Result<Arc<Client>, String> {
        if let Some(c) = &self.client {
            return Ok(c.clone());
        }
        let key = self
            .api_key
            .clone()
            .ok_or_else(|| format!("no API key ({} unset)", crate::deepinfra::KEY_ENV))?;
        let c = Arc::new(Client::new(key, self.cfg.provider.clone()));
        self.client = Some(c.clone());
        Ok(c)
    }

    pub fn search(&mut self, query: &str, filters: &Filters, o: &SearchOpts) -> Result<Value> {
        let t_all = Instant::now();
        let k = self.cfg.fusion.rrf_k as f64;
        let mut stages = serde_json::Map::new();
        let mut timings = serde_json::Map::new();
        let mut usage = serde_json::Map::new();
        let t = Instant::now();
        let lex = catalog::lexical(&self.con, query, filters, self.cfg.fusion.lexical_depth)?;
        timings.insert("lexical_ms".into(), json!(ms(t)));
        stages.insert("lexical".into(), json!(format!("ok ({})", lex.len())));
        let lex_ids: Vec<String> = lex.iter().map(|r| r.chunk_id.clone()).collect();
        let mut dense_rows = Vec::new();
        if !o.use_dense {
            stages.insert("dense".into(), json!("disabled"));
        } else {
            let r = (|| -> Result<String, String> {
                let t = Instant::now();
                let vs = self.vectors()?;
                timings.insert("vectors_load_ms".into(), json!(ms(t)));
                dense::check_compatible(&vs.meta, &self.cfg.embedding.identity())?;
                let client = self.client()?;
                let e = &self.cfg.embedding;
                let t = Instant::now();
                let deadline =
                    Instant::now() + Duration::from_secs_f64(self.cfg.provider.timeout_s.max(0.1));
                let (mut v, u) = client
                    .embed(
                        &e.model,
                        &[e.format_query(query)],
                        e.dimensions,
                        1,
                        usize::MAX,
                        e.usd_per_mtok,
                        Some(deadline),
                    )
                    .map_err(|e| redact(&format!("{e:#}"), self.api_key.as_ref()))?;
                timings.insert("query_embed_ms".into(), json!(ms(t)));
                usage.insert("embed".into(), json!(u));
                let q = v.remove(0);
                let t = Instant::now();
                let allowed =
                    catalog::allowed_ids(&self.con, filters).map_err(|e| e.to_string())?;
                dense_rows = vs
                    .search(&q, allowed.as_ref(), self.cfg.fusion.dense_depth)
                    .map_err(|e| e.to_string())?;
                timings.insert("dense_score_ms".into(), json!(ms(t)));
                Ok(format!(
                    "ok ({}; {}/{} dim {})",
                    dense_rows.len(),
                    vs.provider(),
                    vs.model(),
                    vs.dim()
                ))
            })();
            stages.insert(
                "dense".into(),
                json!(r.unwrap_or_else(|e| format!("skipped: {e}"))),
            );
        }
        let dense_ids: Vec<String> = dense_rows.iter().map(|r| r.chunk_id.clone()).collect();
        // a skipped dense stage leaves an empty list, which contributes nothing to RRF
        let rankings: Vec<&[String]> = vec![&lex_ids, &dense_ids];
        let fused = rrf(&rankings, k);
        let mut order: Vec<String> = fused.iter().map(|(c, _)| c.clone()).collect();
        let top_n = o.rerank_top.unwrap_or(self.cfg.reranker.top_n);
        let mut rerank: HashMap<String, f32> = HashMap::new();
        if !o.use_rerank {
            stages.insert("rerank".into(), json!("disabled; order = RRF"));
        } else {
            match self.client() {
                Err(e) => {
                    stages.insert("rerank".into(), json!(format!("fallback_rrf: {e}")));
                }
                Ok(client) => {
                    let head: Vec<String> = order.iter().take(top_n).cloned().collect();
                    let rows = catalog::get_chunks(&self.con, &head)?;
                    let docs: Vec<String> = head
                        .iter()
                        .map(|c| rows[c]["text"].as_str().unwrap_or_default().to_string())
                        .collect();
                    let timeout = o.rerank_timeout_s.unwrap_or(self.cfg.reranker.timeout_s);
                    let rc = &self.cfg.reranker;
                    let instr = (!rc.instruction.is_empty()).then_some(rc.instruction.as_str());
                    let t = Instant::now();
                    let deadline = Instant::now() + Duration::from_secs_f64(timeout.max(0.0));
                    let res = if head.is_empty() {
                        Ok((Vec::new(), ApiUsage::default()))
                    } else {
                        client.rerank(
                            &rc.model,
                            query,
                            &docs,
                            instr,
                            rc.batch_size,
                            rc.max_batch_chars,
                            rc.usd_per_mtok,
                            Some(deadline),
                        )
                    };
                    timings.insert("rerank_ms".into(), json!(ms(t)));
                    match res {
                        Ok((sc, u)) => {
                            usage.insert("rerank".into(), json!(u));
                            rerank = head.iter().cloned().zip(sc).collect();
                            let mut h: Vec<(usize, String)> =
                                head.iter().cloned().enumerate().collect();
                            h.sort_by(|a, b| {
                                rerank[&b.1].total_cmp(&rerank[&a.1]).then(a.0.cmp(&b.0))
                            });
                            let tail = order.split_off(head.len());
                            order = h.into_iter().map(|(_, c)| c).chain(tail).collect();
                            stages.insert(
                                "rerank".into(),
                                json!(format!(
                                    "ok (top {}; {}/{})",
                                    head.len(),
                                    crate::deepinfra::PROVIDER,
                                    rc.model
                                )),
                            );
                        }
                        Err(e) if e.to_string().starts_with("timeout") => {
                            stages.insert(
                                "rerank".into(),
                                json!(format!("fallback_rrf: timeout after {timeout}s")),
                            );
                        }
                        Err(e) => {
                            stages.insert(
                                "rerank".into(),
                                json!(format!(
                                    "fallback_rrf: {}",
                                    redact(&format!("{e:#}"), self.api_key.as_ref())
                                )),
                            );
                        }
                    }
                }
            }
        }
        let fused_score: HashMap<String, f64> = fused.into_iter().collect();
        let lex_by: HashMap<&str, &catalog::LexHit> =
            lex.iter().map(|r| (r.chunk_id.as_str(), r)).collect();
        let den_by: HashMap<&str, &dense::DenseHit> = dense_rows
            .iter()
            .map(|r| (r.chunk_id.as_str(), r))
            .collect();
        let top: Vec<String> = order.into_iter().take(o.limit).collect();
        let rows = catalog::get_chunks(&self.con, &top)?;
        let results: Vec<Value> = top
            .iter()
            .map(|cid| {
                let r = &rows[cid];
                let l = lex_by.get(cid.as_str());
                let d = den_by.get(cid.as_str());
                json!({
                    "chunk_id": cid, "publication_id": r["publication_id"], "title": r["title"], "year": r["year"],
                    "page": r["page"], "char_start": r["char_start"], "char_end": r["char_end"], "kind": r["kind"],
                    "section": r["section"], "locator": r["locator"], "canonical_url": r["canonical_url"],
                    "lexical_rank": l.map(|x| x.lexical_rank), "bm25": l.map(|x| x.bm25),
                    "dense_rank": d.map(|x| x.dense_rank), "cosine": d.map(|x| x.cosine),
                    "rrf": fused_score.get(cid), "rerank": rerank.get(cid),
                    "outcome_bearing": r["outcome_bearing"], "describes_runs": r["describes_runs"], "text": r["text"],
                })
            })
            .collect();
        timings.insert("total_ms".into(), json!(ms(t_all)));
        Ok(
            json!({"query": query, "filters": filters, "rrf_k": k, "stages": stages, "timings": timings, "usage": usage, "results": results}),
        )
    }
}
