//! Immutable, content-addressed model registry.
//!
//! Layout: `artifacts/models/<release-hex>/{release.json, params.npz, posterior.npz?}`. A release
//! directory is write-once. Loading verifies every tensor hash and (optionally) the declared feature
//! order, so a checkpoint cannot silently change semantics between training and serving. Tensors
//! are flat `name -> f64 array` maps in uncompressed npz (readable by NumPy, the lab tools, the SAM2
//! sidecar and any external analysis).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context};

use crate::contracts::{ModelRelease, TensorEntry};
use crate::hashing::{content_hash, hex_part};
use crate::npz::{read_npz, tensor_sha256, write_npz, NpyArray};

#[derive(Debug, Clone, PartialEq)]
pub struct Tensor {
    pub shape: Vec<usize>,
    pub data: Vec<f64>,
}

impl Tensor {
    pub fn new(shape: Vec<usize>, data: Vec<f64>) -> Self {
        assert_eq!(
            shape.iter().product::<usize>(),
            data.len(),
            "tensor shape/len mismatch"
        );
        Tensor { shape, data }
    }
    pub fn scalar(x: f64) -> Self {
        Tensor {
            shape: vec![],
            data: vec![x],
        }
    }
    pub fn vector(v: Vec<f64>) -> Self {
        Tensor {
            shape: vec![v.len()],
            data: v,
        }
    }
    pub fn len(&self) -> usize {
        self.data.len()
    }
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
    fn to_npy(&self) -> NpyArray {
        NpyArray::from_f64(self.shape.clone(), &self.data)
    }
    fn from_npy(a: &NpyArray) -> Self {
        Tensor {
            shape: a.shape.clone(),
            data: a.to_f64(),
        }
    }
    pub fn sha256(&self) -> String {
        tensor_sha256(&self.to_npy())
    }
}

pub type TensorMap = BTreeMap<String, Tensor>;

pub fn default_root() -> PathBuf {
    std::env::var_os("IGNIS_ARTIFACTS")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("artifacts"))
}

pub fn release_dir(root: &Path, release_id: &str) -> PathBuf {
    root.join("models").join(hex_part(release_id))
}

pub fn tensor_entries(tensors: &TensorMap, frozen: &[String]) -> Vec<TensorEntry> {
    tensors
        .iter()
        .map(|(name, t)| TensorEntry {
            name: name.clone(),
            shape: t.shape.iter().map(|&d| d as i64).collect(),
            dtype: "float64".into(),
            sha256: t.sha256(),
            trainable: !frozen.contains(name),
        })
        .collect()
}

/// Git commit of the working tree (`+dirty` when modified), or "unknown".
pub fn code_version() -> String {
    let run = |args: &[&str]| {
        std::process::Command::new("git")
            .args(args)
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
    };
    match run(&["rev-parse", "HEAD"]) {
        Some(sha) => {
            let dirty = run(&["status", "--porcelain"])
                .map(|s| !s.is_empty())
                .unwrap_or(false);
            if dirty {
                format!("{sha}+dirty")
            } else {
                sha
            }
        }
        None => "unknown".into(),
    }
}

/// Create a new release. `release` carries every field; `id`, `tensors`, `posterior` and
/// `created_at` are filled here. Idempotent: an identical release returns the stored one.
pub fn save_release(
    root: &Path,
    mut release: ModelRelease,
    params: &TensorMap,
    posterior: &TensorMap,
    frozen: &[String],
) -> anyhow::Result<ModelRelease> {
    release.tensors = tensor_entries(params, frozen);
    release.posterior = tensor_entries(posterior, &[]);
    if release.code_version.is_empty() {
        release.code_version = code_version();
    }
    release.check_construction()?;
    release.id = content_hash(&release, &["id", "created_at"]);
    release.created_at = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true);
    let dir = release_dir(root, &release.id);
    if dir.exists() {
        return Ok(load_release(root, &release.id, None)?.0);
    }
    let tmp = dir.with_extension("tmp");
    if tmp.exists() {
        std::fs::remove_dir_all(&tmp)?;
    }
    std::fs::create_dir_all(&tmp)?;
    let npy = |m: &TensorMap| {
        m.iter()
            .map(|(k, t)| (k.clone(), t.to_npy()))
            .collect::<BTreeMap<_, _>>()
    };
    write_npz(&tmp.join("params.npz"), &npy(params))?;
    if !posterior.is_empty() {
        write_npz(&tmp.join("posterior.npz"), &npy(posterior))?;
    }
    std::fs::write(
        tmp.join("release.json"),
        serde_json::to_vec_pretty(&release)?,
    )?;
    std::fs::rename(&tmp, &dir)?;
    Ok(release)
}

/// Reads a `release.json`, applying the one documented legacy rule: a schema-2.0.0 file that has
/// neither `trained_on` nor `construction` keeps the old implicit `trained_on = "nasa"`. A file with a
/// `construction` record must state `trained_on`, and every release passes
/// [`ModelRelease::check_construction`], so no handwritten release can be read as NASA-trained.
pub fn parse_release(text: &str) -> anyhow::Result<ModelRelease> {
    let mut v: serde_json::Value = serde_json::from_str(text)?;
    if let Some(m) = v.as_object_mut() {
        if !m.contains_key("trained_on") && !m.contains_key("construction") {
            m.insert(
                "trained_on".into(),
                serde_json::Value::String("nasa".into()),
            );
        }
    }
    let release: ModelRelease = serde_json::from_value(v)?;
    release.check_construction()?;
    Ok(release)
}

pub struct LoadedRelease {
    pub release: ModelRelease,
    pub params: TensorMap,
    pub posterior: TensorMap,
}

pub fn load_release(
    root: &Path,
    release_id: &str,
    expect_features: Option<&[String]>,
) -> anyhow::Result<(ModelRelease, TensorMap, TensorMap)> {
    let dir = release_dir(root, release_id);
    let text = std::fs::read_to_string(dir.join("release.json"))
        .with_context(|| format!("release {release_id}"))?;
    let release = parse_release(&text)?;
    if hex_part(&release.id) != hex_part(release_id) {
        bail!("release id mismatch");
    }
    let read = |name: &str| -> anyhow::Result<TensorMap> {
        let p = dir.join(name);
        if !p.exists() {
            return Ok(TensorMap::new());
        }
        Ok(read_npz(&p)?
            .iter()
            .map(|(k, a)| (k.clone(), Tensor::from_npy(a)))
            .collect())
    };
    let params = read("params.npz")?;
    let posterior = read("posterior.npz")?;
    for (entries, tensors) in [
        (&release.tensors, &params),
        (&release.posterior, &posterior),
    ] {
        if entries.len() != tensors.len() || entries.iter().any(|e| !tensors.contains_key(&e.name))
        {
            bail!("tensor set differs from manifest");
        }
        for e in entries {
            if tensors[&e.name].sha256() != e.sha256 {
                bail!("tensor {} does not match its recorded hash", e.name);
            }
        }
    }
    if let Some(f) = expect_features {
        if f != release.feature_order.as_slice() {
            bail!(
                "feature order mismatch: {:?} != {:?}",
                release.feature_order,
                f
            );
        }
    }
    Ok((release, params, posterior))
}

pub fn load(root: &Path, release_id: &str) -> anyhow::Result<LoadedRelease> {
    let (release, params, posterior) = load_release(root, release_id, None)?;
    Ok(LoadedRelease {
        release,
        params,
        posterior,
    })
}

pub fn list_releases(root: &Path) -> Vec<ModelRelease> {
    let Ok(rd) = std::fs::read_dir(root.join("models")) else {
        return vec![];
    };
    let mut out: Vec<ModelRelease> = rd
        .flatten()
        .filter_map(|e| std::fs::read_to_string(e.path().join("release.json")).ok())
        .filter_map(|t| parse_release(&t).ok())
        .collect();
    out.sort_by(|a, b| a.created_at.cmp(&b.created_at));
    out
}
