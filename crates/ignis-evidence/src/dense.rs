//! Retrieval configuration, vector identity and the vector file.
//!
//! Embeddings and reranking are served by the DeepInfra API ([`crate::deepinfra`]) with Qwen3
//! (pretrained only; never trained here). The input formatting is part of the experiment (V3.1
//! §18.2): queries are `"Instruct: {task}\nQuery:{query}"` (Qwen3-Embedding model card), documents
//! carry no instruction. Every vector file records its [`VectorIdentity`] (provider, model,
//! dimension, query template) and creation date; vectors of a different identity are never mixed:
//! search refuses them, `embed` refuses to overwrite without `--force`.

use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::deepinfra::ProviderCfg;
use ignis_core::hashing::sha256_hex;

// ------------------------------------------------------------------ config

#[derive(Debug, Clone, Deserialize)]
pub struct EmbeddingCfg {
    pub model: String,
    /// MRL truncation sent as `dimensions` (None: the model's native size)
    #[serde(default)]
    pub dimensions: Option<usize>,
    #[serde(default = "d32")]
    pub batch_size: usize,
    #[serde(default = "d100k")]
    pub max_batch_chars: usize,
    #[serde(default)]
    pub usd_per_mtok: f64,
    #[serde(default)]
    pub query_instruction: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RerankerCfg {
    pub model: String,
    #[serde(default = "d32")]
    pub batch_size: usize,
    #[serde(default = "d100k")]
    pub max_batch_chars: usize,
    #[serde(default)]
    pub usd_per_mtok: f64,
    #[serde(default = "d30")]
    pub top_n: usize,
    #[serde(default = "d20")]
    pub timeout_s: f64,
    #[serde(default)]
    pub instruction: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FusionCfg {
    #[serde(default = "d60")]
    pub rrf_k: usize,
    #[serde(default = "d100")]
    pub lexical_depth: usize,
    #[serde(default = "d100")]
    pub dense_depth: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RetrievalCfg {
    #[serde(default)]
    pub provider: ProviderCfg,
    pub embedding: EmbeddingCfg,
    pub reranker: RerankerCfg,
    pub fusion: FusionCfg,
}

fn d20() -> f64 {
    20.0
}
fn d30() -> usize {
    30
}
fn d32() -> usize {
    32
}
fn d60() -> usize {
    60
}
fn d100() -> usize {
    100
}
fn d100k() -> usize {
    100_000
}

impl RetrievalCfg {
    pub fn default_path() -> PathBuf {
        ignis_core::resources::configs_dir()
            .join("evidence")
            .join("retrieval.toml")
    }
    pub fn load(path: &Path) -> Result<RetrievalCfg> {
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("retrieval config {}", path.display()))?;
        Ok(toml::from_str(&text)?)
    }
}

impl EmbeddingCfg {
    pub fn format_query(&self, q: &str) -> String {
        if self.query_instruction.is_empty() {
            q.to_string()
        } else {
            format!("Instruct: {}\nQuery:{q}", self.query_instruction)
        }
    }
    /// What a vector file must match to be searched with this config.
    pub fn identity(&self) -> VectorIdentity {
        VectorIdentity {
            provider: crate::deepinfra::PROVIDER.into(),
            model: self.model.clone(),
            dim: self.dimensions,
            query_instruction: self.format_query("{query}"),
        }
    }
}

// ------------------------------------------------------------------ identity

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct VectorIdentity {
    pub provider: String,
    pub model: String,
    /// None = whatever the model returns natively (checked against the file only when set)
    pub dim: Option<usize>,
    /// the query template, `{query}` marks the query
    pub query_instruction: String,
}

/// `Ok` when vectors described by `meta` (a `vectors.ids.json`) can be compared with queries
/// embedded under `want`; otherwise the reason. Files written before providers were recorded
/// (local candle Qwen3) have no `provider` and never match.
pub fn check_compatible(meta: &Value, want: &VectorIdentity) -> Result<(), String> {
    let provider = meta["provider"].as_str().unwrap_or("local");
    let model = meta["model"].as_str().unwrap_or("?");
    let mut why = Vec::new();
    if provider != want.provider {
        why.push(format!("provider {provider} != {}", want.provider));
    }
    if model != want.model {
        why.push(format!("model {model} != {}", want.model));
    }
    if let (Some(d), Some(have)) = (want.dim, meta["dim"].as_u64()) {
        if have as usize != d {
            why.push(format!("dim {have} != {d}"));
        }
    }
    if meta["query_instruction"].as_str().unwrap_or_default() != want.query_instruction {
        why.push("query instruction differs".into());
    }
    if why.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "vector set {provider}/{model} is not comparable with the configured embedder: {}",
            why.join("; ")
        ))
    }
}

// ------------------------------------------------------------------ vector file

/// `vectors.f32` (N x dim, normalized little-endian f32) + `vectors.ids.json`; exact dot scoring.
pub struct VectorStore {
    pub dir: PathBuf,
    pub meta: Value,
    ids: Vec<String>,
    dim: usize,
    data: Vec<f32>,
}

#[derive(Debug, Clone)]
pub struct DenseHit {
    pub chunk_id: String,
    pub cosine: f32,
    pub dense_rank: usize,
}

impl VectorStore {
    pub fn vec_path(dir: &Path) -> PathBuf {
        dir.join("vectors.f32")
    }
    pub fn ids_path(dir: &Path) -> PathBuf {
        dir.join("vectors.ids.json")
    }
    pub fn exists(dir: &Path) -> bool {
        Self::vec_path(dir).exists() && Self::ids_path(dir).exists()
    }
    /// The metadata of an existing set without reading the matrix.
    pub fn read_meta(dir: &Path) -> Result<Option<Value>> {
        if !Self::ids_path(dir).exists() {
            return Ok(None);
        }
        let mut m: Value = serde_json::from_slice(&std::fs::read(Self::ids_path(dir))?)?;
        if let Value::Object(o) = &mut m {
            o.remove("ids");
        }
        Ok(Some(m))
    }

    pub fn write(
        dir: &Path,
        ids: &[String],
        vectors: &[Vec<f32>],
        id: &VectorIdentity,
        revision: &str,
    ) -> Result<Value> {
        let dim = vectors.first().map(|v| v.len()).unwrap_or(0);
        if vectors.len() != ids.len() || vectors.iter().any(|v| v.len() != dim) {
            bail!("vectors must be aligned with ids and share one dimension");
        }
        if id.dim.is_some_and(|d| d != dim) {
            bail!(
                "vectors have dimension {dim}, the identity says {:?}",
                id.dim
            );
        }
        if vectors
            .iter()
            .any(|v| (v.iter().map(|x| x * x).sum::<f32>().sqrt() - 1.0).abs() > 1e-3)
        {
            bail!("vectors must be L2-normalized");
        }
        std::fs::create_dir_all(dir)?;
        let mut bytes = Vec::with_capacity(vectors.len() * dim * 4);
        for v in vectors {
            for x in v {
                bytes.extend_from_slice(&x.to_le_bytes());
            }
        }
        let tmp = dir.join("vectors.f32.tmp");
        std::fs::write(&tmp, &bytes)?;
        std::fs::rename(&tmp, Self::vec_path(dir))?;
        let meta = json!({
            "ids": ids, "provider": id.provider, "model": id.model, "revision": revision, "dim": dim, "n": vectors.len(),
            "dtype": "float32-le", "normalized": true, "query_instruction": id.query_instruction, "sha256": sha256_hex(&bytes),
            "created_at": chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, false),
        });
        std::fs::write(Self::ids_path(dir), serde_json::to_vec(&meta)?)?;
        Ok(meta)
    }

    /// Load and verify (sha256) once; keep the matrix in memory for repeated queries.
    pub fn load(dir: &Path) -> Result<VectorStore> {
        let meta: Value = serde_json::from_slice(&std::fs::read(Self::ids_path(dir))?)?;
        let raw = std::fs::read(Self::vec_path(dir))?;
        if meta["sha256"].as_str() != Some(sha256_hex(&raw).as_str()) {
            bail!("vectors.f32 does not match its recorded sha256");
        }
        let n = meta["n"].as_u64().context("n")? as usize;
        let dim = meta["dim"].as_u64().context("dim")? as usize;
        if raw.len() != n * dim * 4 {
            bail!(
                "vectors.f32 has {} bytes, expected {n} x {dim} x 4",
                raw.len()
            );
        }
        let data = raw
            .chunks_exact(4)
            .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .collect();
        let ids = meta["ids"]
            .as_array()
            .context("ids")?
            .iter()
            .map(|x| x.as_str().unwrap_or_default().to_string())
            .collect();
        Ok(VectorStore {
            dir: dir.to_path_buf(),
            meta,
            ids,
            dim,
            data,
        })
    }

    pub fn model(&self) -> &str {
        self.meta["model"].as_str().unwrap_or_default()
    }
    pub fn provider(&self) -> &str {
        self.meta["provider"].as_str().unwrap_or("local")
    }
    pub fn len(&self) -> usize {
        self.ids.len()
    }
    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }
    pub fn dim(&self) -> usize {
        self.dim
    }
    pub fn row(&self, i: usize) -> &[f32] {
        &self.data[i * self.dim..(i + 1) * self.dim]
    }
    pub fn ids(&self) -> &[String] {
        &self.ids
    }

    pub fn search(
        &self,
        q: &[f32],
        allowed: Option<&std::collections::HashSet<String>>,
        limit: usize,
    ) -> Result<Vec<DenseHit>> {
        if q.len() != self.dim {
            bail!("query dim {} != vector dim {}", q.len(), self.dim);
        }
        let mut s: Vec<(usize, f32)> = (0..self.ids.len())
            .map(|i| (i, self.row(i).iter().zip(q).map(|(a, b)| a * b).sum()))
            .collect();
        s.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0))); // stable: ties keep row order
        let mut out = Vec::new();
        for (i, c) in s {
            if allowed.is_some_and(|a| !a.contains(&self.ids[i])) {
                continue;
            }
            out.push(DenseHit {
                chunk_id: self.ids[i].clone(),
                cosine: c,
                dense_rank: out.len() + 1,
            });
            if out.len() >= limit {
                break;
            }
        }
        Ok(out)
    }
}

/// Decide whether `embed --execute` may (re)write the vector file next to the catalog.
pub fn embed_permission(
    existing: Option<&Value>,
    want: &VectorIdentity,
    force: bool,
) -> Result<(), String> {
    let Some(m) = existing else { return Ok(()) };
    if force {
        return Ok(());
    }
    let what = format!(
        "{}/{} dim {} n {} created {}",
        m["provider"].as_str().unwrap_or("local"),
        m["model"].as_str().unwrap_or("?"),
        m["dim"],
        m["n"],
        m["created_at"].as_str().unwrap_or("?")
    );
    match check_compatible(m, want) {
        Ok(()) => Err(format!("a vector set already exists ({what}); re-embedding is refused without --force")),
        Err(e) => Err(format!("{e}; vectors of different models are never mixed: pass --force to replace the set ({what})")),
    }
}
