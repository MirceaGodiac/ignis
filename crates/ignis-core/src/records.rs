//! Read-only run-record registry loaded from JSON lines: what `ignis serve` and `ignis infer` read.
//!
//! Layout: every `*.jsonl` file under the data directory whose name ends with `experiments.jsonl`,
//! `observations.jsonl`, `contours.jsonl`, `control_histories.jsonl` (or the ingest's
//! `extra_control_history.jsonl`), `endpoints.jsonl`, `splits.jsonl`, `sources.jsonl` or
//! `video_map.jsonl`; one contract object per line, validated by the Rust mirrors. Invalid lines are
//! reported, never guessed. [`RegistryPaths`] adds the frozen splits, the cohorts and the vision
//! contours, which live outside the records directory.

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::contracts::{
    Cohort, ContourObservation, ControlHistory, EndpointRecord, EvidenceClass, ExperimentRecord,
    Observation, Source, Split,
};
use crate::views::{family_time_origin, RunArchive};

/// Locations outside the records directory. Each is optional.
#[derive(Debug, Clone, Default)]
pub struct RegistryPaths {
    /// `data/work/cohorts/<hex>.json`
    pub cohorts: Option<PathBuf>,
    /// `data/splits/<family>-<hex16>.json` (frozen split files, each embedding its cohort)
    pub splits: Option<PathBuf>,
    /// `artifacts/vision/` (searched recursively for `contours.jsonl`)
    pub vision: Option<PathBuf>,
}

#[derive(Default)]
pub struct Registry {
    pub dir: PathBuf,
    pub experiments: BTreeMap<String, ExperimentRecord>,
    /// Sorted by (time_s, id), like `build_prefix`.
    pub observations: HashMap<String, Vec<Observation>>,
    pub contours: HashMap<String, Vec<ContourObservation>>,
    pub controls: HashMap<String, ControlHistory>,
    pub endpoints: HashMap<String, EndpointRecord>,
    pub splits: Vec<Split>,
    pub sources: BTreeMap<String, Source>,
    pub cohorts: BTreeMap<String, Cohort>,
    /// `video_map` rows of each run (BASS recordings mapped to runs).
    pub videos: HashMap<String, Vec<Value>>,
    pub files: Vec<String>,
    pub errors: Vec<String>,
}

const KINDS: &[&str] = &[
    "experiments",
    "observations",
    "contours",
    "control_histories",
    "control_history",
    "endpoints",
    "splits",
    "sources",
    "video_map",
];

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<PathBuf> = rd.filter_map(|e| e.ok().map(|e| e.path())).collect();
    entries.sort();
    for p in entries {
        if p.is_dir() {
            walk(&p, out);
        } else if p.extension().is_some_and(|e| e == "jsonl") {
            out.push(p);
        }
    }
}

fn parse_lines<T: DeserializeOwned>(path: &Path, text: &str, errors: &mut Vec<String>) -> Vec<T> {
    let mut out = Vec::new();
    for (i, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        match serde_json::from_str::<T>(line) {
            Ok(v) => out.push(v),
            Err(e) => errors.push(format!("{}:{}: {e}", path.display(), i + 1)),
        }
    }
    out
}

impl Registry {
    pub fn load(dir: &Path) -> Registry {
        Self::load_with(dir, &RegistryPaths::default())
    }

    pub fn load_with(dir: &Path, paths: &RegistryPaths) -> Registry {
        let mut r = Registry {
            dir: dir.to_path_buf(),
            ..Default::default()
        };
        let mut files = Vec::new();
        walk(dir, &mut files);
        if let Some(v) = &paths.vision {
            let mut extra = Vec::new();
            walk(v, &mut extra);
            files.extend(
                extra
                    .into_iter()
                    .filter(|p| p.file_name().is_some_and(|n| n == "contours.jsonl")),
            );
        }
        for path in files {
            let name = path.file_name().unwrap().to_string_lossy().to_string();
            let Some(kind) = KINDS.iter().find(|k| name.ends_with(&format!("{k}.jsonl"))) else {
                continue;
            };
            let text = match std::fs::read_to_string(&path) {
                Ok(t) => t,
                Err(e) => {
                    r.errors.push(format!("{}: {e}", path.display()));
                    continue;
                }
            };
            r.files.push(path.display().to_string());
            let errs = &mut r.errors;
            match *kind {
                "experiments" => {
                    for x in parse_lines::<ExperimentRecord>(&path, &text, errs) {
                        if r.experiments.insert(x.id.clone(), x).is_some() {
                            errs.push(format!("{}: duplicate experiment id", path.display()));
                        }
                    }
                }
                "observations" => {
                    for x in parse_lines::<Observation>(&path, &text, errs) {
                        r.observations.entry(x.run_id.clone()).or_default().push(x);
                    }
                }
                "contours" => {
                    for x in parse_lines::<ContourObservation>(&path, &text, errs) {
                        r.contours.entry(x.run_id.clone()).or_default().push(x);
                    }
                }
                "control_histories" | "control_history" => {
                    for x in parse_lines::<ControlHistory>(&path, &text, errs) {
                        r.controls.insert(x.run_id.clone(), x);
                    }
                }
                "endpoints" => {
                    for x in parse_lines::<EndpointRecord>(&path, &text, errs) {
                        r.endpoints.insert(x.run_id.clone(), x);
                    }
                }
                "splits" => r.splits.extend(parse_lines::<Split>(&path, &text, errs)),
                "sources" => {
                    for x in parse_lines::<Source>(&path, &text, errs) {
                        r.sources.insert(x.id.clone(), x);
                    }
                }
                "video_map" => {
                    for x in parse_lines::<Value>(&path, &text, errs) {
                        if let Some(run) = x.get("run_id").and_then(Value::as_str) {
                            r.videos.entry(run.to_string()).or_default().push(x.clone());
                        }
                    }
                }
                _ => unreachable!(),
            }
        }
        r.load_cohorts_and_splits(paths);
        for v in r.observations.values_mut() {
            v.sort_by(|a, b| a.time_s.total_cmp(&b.time_s).then_with(|| a.id.cmp(&b.id)));
        }
        for v in r.contours.values_mut() {
            v.sort_by(|a, b| a.time_s.total_cmp(&b.time_s).then_with(|| a.id.cmp(&b.id)));
            v.dedup_by(|a, b| a.id == b.id);
        }
        r
    }

    fn load_cohorts_and_splits(&mut self, paths: &RegistryPaths) {
        for (dir, is_split) in [(&paths.cohorts, false), (&paths.splits, true)] {
            let Some(dir) = dir else { continue };
            let Ok(rd) = std::fs::read_dir(dir) else {
                continue;
            };
            let mut json: Vec<PathBuf> = rd
                .flatten()
                .map(|e| e.path())
                .filter(|p| p.extension().is_some_and(|x| x == "json"))
                .collect();
            json.sort();
            for path in json {
                match Self::read_set(&path, is_split) {
                    Ok((cohort, split)) => {
                        if let Some(c) = cohort {
                            self.cohorts.entry(c.id.clone()).or_insert(c);
                        }
                        if let Some(s) = split {
                            if !self.splits.iter().any(|x| x.id == s.id) {
                                self.splits.push(s);
                            }
                        }
                    }
                    Err(e) => self.errors.push(format!("{}: {e}", path.display())),
                }
            }
        }
    }

    /// A cohort file, or a frozen split file (`{"split": .., "cohort": ..}`).
    fn read_set(path: &Path, is_split: bool) -> anyhow::Result<(Option<Cohort>, Option<Split>)> {
        let v: Value = serde_json::from_str(&std::fs::read_to_string(path)?)?;
        if !is_split {
            return Ok((Some(serde_json::from_value(v)?), None));
        }
        let split = serde_json::from_value(v.get("split").cloned().unwrap_or(v.clone()))?;
        let cohort = v
            .get("cohort")
            .map(|c| serde_json::from_value(c.clone()))
            .transpose()?;
        Ok((cohort, Some(split)))
    }

    /// The stored records of one run (owned; the forecast and deposit paths read this).
    pub fn archive(&self, run: &str) -> Option<RunArchive> {
        let record = self.experiments.get(run)?.clone();
        let control = self.controls.get(run).cloned();
        let time_origin = control
            .as_ref()
            .map(|c| c.time_origin.clone())
            .unwrap_or_else(|| family_time_origin(record.family).to_string());
        Some(RunArchive {
            observations: self.observations(run).to_vec(),
            contours: self.contours(run).to_vec(),
            endpoint: self.endpoints.get(run).cloned(),
            control,
            time_origin,
            record,
        })
    }

    pub fn observations(&self, run: &str) -> &[Observation] {
        self.observations.get(run).map(Vec::as_slice).unwrap_or(&[])
    }

    pub fn contours(&self, run: &str) -> &[ContourObservation] {
        self.contours.get(run).map(Vec::as_slice).unwrap_or(&[])
    }

    /// A run is synthetic when every observation is labelled synthetic (or the record says so).
    pub fn is_synthetic(&self, run: &str) -> bool {
        let obs = self.observations(run);
        let rec_flag = self
            .experiments
            .get(run)
            .is_some_and(|r| r.quality_flags.iter().any(|f| f == "synthetic"));
        rec_flag
            || (!obs.is_empty()
                && obs
                    .iter()
                    .all(|o| o.evidence_class == EvidenceClass::Synthetic))
    }
}
