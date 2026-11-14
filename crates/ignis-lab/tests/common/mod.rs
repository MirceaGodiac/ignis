#![allow(dead_code)]

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

use ignis_core::contracts::{AnalysisArtifact, ModelRelease, Split};
use ignis_core::hashing::content_hash;
use ignis_core::registry::{save_release, TensorMap};
use serde_json::json;

static LOCK: Mutex<()> = Mutex::new(());

/// Fresh roots under a temp dir; holds a process-wide lock because roots come from env vars.
pub struct Env {
    pub root: PathBuf,
    _g: MutexGuard<'static, ()>,
}

pub fn env(tag: &str) -> Env {
    let g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let root = std::env::temp_dir().join(format!("ignis-lab-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    std::env::set_var("IGNIS_ARTIFACTS", root.join("artifacts"));
    std::env::set_var("IGNIS_HOLDOUT", root.join("holdout"));
    std::env::set_var("IGNIS_RUNS", root.join("runs"));
    std::env::set_var("IGNIS_SPLITS", root.join("splits"));
    Env { root, _g: g }
}

impl Env {
    pub fn artifacts(&self) -> PathBuf {
        self.root.join("artifacts")
    }
}

pub fn split() -> Split {
    let runs: BTreeMap<String, String> = [
        ("r_tr", "g1"),
        ("r_va1", "g2"),
        ("r_va2", "g3"),
        ("r_te", "g4"),
    ]
    .iter()
    .map(|(a, b)| (a.to_string(), b.to_string()))
    .collect();
    let mut s = Split {
        id: String::new(),
        cohort_id: "c1".into(),
        group_key: "session".into(),
        rule: "fixture".into(),
        seed: 0,
        run_to_group: runs,
        train: vec!["g1".into()],
        validation: vec!["g2".into(), "g3".into()],
        test: vec!["g4".into()],
        frozen_at: "2026-11-14T00:00:00Z".into(),
    };
    s.id = content_hash(&s, &["id"]);
    s
}

/// Tiny synthetic release (`trained_on="synthetic"`).
pub fn release(
    env: &Env,
    split_id: &str,
    features: &[&str],
    arch: serde_json::Value,
    params: &TensorMap,
    post: &TensorMap,
) -> ModelRelease {
    let r: ModelRelease = serde_json::from_value(json!({
        "id": "", "family": "flex", "model_name": "fixture", "is_baseline": true, "architecture": arch, "config": {},
        "feature_order": features, "units": {"a": "Pa"}, "normalization": {}, "cohort_id": "c1", "split_id": split_id,
        "source_manifest_hash": "m", "code_version": "test", "trained_on": "synthetic", "created_at": ""}))
    .unwrap();
    save_release(&env.artifacts(), r, params, post, &[]).unwrap()
}

pub fn artifact(release_id: &str, run: &str, shift: f64) -> AnalysisArtifact {
    let t = [1.0, 2.0, 3.0];
    let med: Vec<f64> = t.iter().map(|x| 1e-3 * (2.0 - 0.1 * x) + shift).collect();
    let mut a: AnalysisArtifact = serde_json::from_value(json!({
        "id": "", "kind": "prefix_forecast", "model_release_id": release_id, "prefix_hash": format!("prefix-{run}"),
        "settings_hash": "s", "status": "forecast_qualified", "quantity": "liquid_diameter", "unit": "m", "times_s": t,
        "quantile_levels": [0.05, 0.5, 0.95],
        "quantiles": [med.iter().map(|m| m - 1e-4).collect::<Vec<_>>(), med.clone(), med.iter().map(|m| m + 1e-4).collect::<Vec<_>>()],
        "support": {"admitted": true, "gate": "passed"},
        "created_at": chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true)}))
    .unwrap();
    a.id = content_hash(&a, &["id", "created_at"]);
    a
}

pub fn is_refusal(e: &anyhow::Error, needle: &str) -> bool {
    e.downcast_ref::<ignis_lab::util::Refused>().is_some() && e.to_string().contains(needle)
}
