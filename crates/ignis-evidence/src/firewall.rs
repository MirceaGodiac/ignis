//! Leak firewall: evidence content never reaches a prospective forecast (V3.1 §4.4, §18, §24.8).
//!
//! Two mechanisms, both in code:
//!
//! 1. [`scan`]: the workspace dependency graph from `cargo metadata --no-deps` (declared
//!    dependencies of every member, all kinds: normal, build, dev; renamed packages resolve to their
//!    real name). It fails if any protected crate (`configs/evidence/firewall.toml`: the scientific
//!    crates that fit, forecast or train) reaches `ignis-evidence` directly or through other members.
//!
//! 2. [`strip_for_sealed`]: the helper the API layer calls before returning evidence while runs are
//!    under sealed evaluation: outcome-bearing chunks/findings that describe a sealed run are removed;
//!    outcome-bearing items with *no* run attribution are removed too unless the caller explicitly
//!    opts out (attribution recall is imperfect, so the default is conservative).

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};
use std::path::Path;
use std::process::Command;

use anyhow::{bail, Context, Result};
use serde::Serialize;
use serde_json::Value;

use crate::records::{Chunk, Finding};

pub const EVIDENCE_CRATE: &str = "ignis-evidence";
pub const DEFAULT_PROTECTED: [&str; 6] = [
    "ignis-science",
    "ignis-flex",
    "ignis-acme",
    "ignis-bass",
    "ignis-accel",
    "ignis-train",
];

#[derive(Debug, Clone, Serialize)]
pub struct Violation {
    pub krate: String,
    /// protected crate -> ... -> ignis-evidence, with the dependency kind of each edge
    pub chain: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScanResult {
    pub passed: bool,
    pub protected: Vec<String>,
    /// protected crates that exist in this workspace
    pub present: Vec<String>,
    pub members_scanned: usize,
    pub violations: Vec<Violation>,
}

pub fn protected_from_config(path: &Path) -> Result<Vec<String>> {
    if !path.exists() {
        return Ok(DEFAULT_PROTECTED.iter().map(|s| s.to_string()).collect());
    }
    let v: toml::Value = toml::from_str(&std::fs::read_to_string(path)?)?;
    Ok(v.get("protected")
        .and_then(|p| p.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_else(|| DEFAULT_PROTECTED.iter().map(|s| s.to_string()).collect()))
}

/// `cargo metadata --no-deps` of the workspace whose root manifest is `manifest`.
pub fn cargo_metadata(manifest: &Path) -> Result<Value> {
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let out = Command::new(cargo)
        .args([
            "metadata",
            "--format-version",
            "1",
            "--no-deps",
            "--offline",
            "--manifest-path",
        ])
        .arg(manifest)
        .output()
        .context("run `cargo metadata`")?;
    if !out.status.success() {
        bail!(
            "cargo metadata failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    Ok(serde_json::from_slice(&out.stdout)?)
}

/// Graph `member -> [(dependency package name, kind)]` restricted to workspace members.
pub fn member_graph(meta: &Value) -> BTreeMap<String, Vec<(String, String)>> {
    let mut g = BTreeMap::new();
    for p in meta["packages"].as_array().into_iter().flatten() {
        let name = p["name"].as_str().unwrap_or_default().to_string();
        let deps = p["dependencies"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|d| {
                (
                    d["name"].as_str().unwrap_or_default().to_string(),
                    d["kind"].as_str().unwrap_or("normal").to_string(),
                )
            })
            .collect();
        g.insert(name, deps);
    }
    g
}

pub fn scan_graph(
    graph: &BTreeMap<String, Vec<(String, String)>>,
    protected: &[String],
) -> ScanResult {
    let mut violations = Vec::new();
    let present: Vec<String> = protected
        .iter()
        .filter(|p| graph.contains_key(*p))
        .cloned()
        .collect();
    for start in &present {
        let mut prev: HashMap<String, Option<(String, String)>> =
            HashMap::from([(start.clone(), None)]);
        let mut q = VecDeque::from([start.clone()]);
        while let Some(m) = q.pop_front() {
            if m == EVIDENCE_CRATE {
                let mut chain = Vec::new();
                let mut cur = m.clone();
                loop {
                    match prev.get(&cur) {
                        Some(Some((p, kind))) => {
                            chain.push(format!("{cur} [{kind}]"));
                            cur = p.clone();
                        }
                        _ => {
                            chain.push(cur.clone());
                            break;
                        }
                    }
                }
                chain.reverse();
                violations.push(Violation {
                    krate: start.clone(),
                    chain,
                });
                break;
            }
            let mut next: Vec<&(String, String)> = graph
                .get(&m)
                .map(|d| d.iter().collect())
                .unwrap_or_default();
            next.sort();
            for (d, kind) in next {
                // only members (and the evidence crate itself) carry edges further
                if (graph.contains_key(d) || d == EVIDENCE_CRATE) && !prev.contains_key(d) {
                    prev.insert(d.clone(), Some((m.clone(), kind.clone())));
                    q.push_back(d.clone());
                }
            }
        }
    }
    ScanResult {
        passed: violations.is_empty(),
        protected: protected.to_vec(),
        present,
        members_scanned: graph.len(),
        violations,
    }
}

/// Scan the workspace rooted at `manifest` (its `Cargo.toml`).
pub fn scan(manifest: &Path, protected: &[String]) -> Result<ScanResult> {
    Ok(scan_graph(
        &member_graph(&cargo_metadata(manifest)?),
        protected,
    ))
}

/// Anything the sealed-run filter can judge.
pub trait SealedItem {
    fn item_id(&self) -> String;
    fn outcome_bearing(&self) -> bool;
    fn describes_runs(&self) -> Vec<String>;
}

impl SealedItem for Value {
    fn item_id(&self) -> String {
        self.get("id")
            .or_else(|| self.get("chunk_id"))
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string()
    }
    fn outcome_bearing(&self) -> bool {
        // missing flag = outcome-bearing (conservative)
        self.get("outcome_bearing")
            .map(|v| v.as_bool().unwrap_or(v.as_i64().unwrap_or(1) != 0))
            .unwrap_or(true)
    }
    fn describes_runs(&self) -> Vec<String> {
        self.get("describes_runs")
            .and_then(|v| v.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|x| x.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default()
    }
}

impl SealedItem for Chunk {
    fn item_id(&self) -> String {
        self.id.clone()
    }
    fn outcome_bearing(&self) -> bool {
        self.outcome_bearing
    }
    fn describes_runs(&self) -> Vec<String> {
        self.describes_runs.clone()
    }
}

impl SealedItem for Finding {
    fn item_id(&self) -> String {
        self.id.clone()
    }
    fn outcome_bearing(&self) -> bool {
        self.outcome_bearing
    }
    fn describes_runs(&self) -> Vec<String> {
        self.describes_runs.clone()
    }
}

/// Return `(kept, stripped_ids)` for chunks/findings under sealed evaluation of `sealed_runs`.
/// `alias_to_run` maps text aliases (`alias:bass2:testT12`) to `ExperimentRecord` ids.
pub fn strip_for_sealed<T: SealedItem>(
    items: Vec<T>,
    sealed_runs: &[String],
    alias_to_run: &HashMap<String, String>,
    strip_unattributed_outcomes: bool,
) -> (Vec<T>, Vec<String>) {
    let sealed: HashSet<&str> = sealed_runs.iter().map(|s| s.as_str()).collect();
    let mut kept = Vec::new();
    let mut stripped = Vec::new();
    for it in items {
        if sealed.is_empty() || !it.outcome_bearing() {
            kept.push(it);
            continue;
        }
        let runs: BTreeSet<String> = it
            .describes_runs()
            .into_iter()
            .map(|r| alias_to_run.get(&r).cloned().unwrap_or(r))
            .collect();
        if runs.iter().any(|r| sealed.contains(r.as_str()))
            || (runs.is_empty() && strip_unattributed_outcomes)
        {
            stripped.push(it.item_id());
        } else {
            kept.push(it);
        }
    }
    (kept, stripped)
}
