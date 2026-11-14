//! Visual observation bundles (workboard R5): clip → run identity, the bundles written by
//! [`crate::pipeline::run_contours`], and the scope checks for serving them.
//!
//! **Identity.** [`resolve_identity`] cross-checks the run a spec or the command line declares with the
//! data lane's clip → run map (`bass.video.map`: test id + GMT day + sample). Agreement is
//! `resolved`; a map that names another run is `unresolved_conflict`; no map entry is `unknown`
//! (declared, not cross-checked). Only a resolved identity can feed a metric forecast
//! ([`VisualObservationBundle::require_metric`]).
//!
//! **Serving.** [`VisualBundleStore::bundle`] needs `read_data`, and the partition of the bundle's run
//! in the frozen splits must be granted (contours show how a run burned: they are outcomes). A clip
//! whose identity is not resolved could show a sealed run, so only an evaluation scope receives it.
//! A run outside every frozen split is not in the lockbox.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use ignis_core::contracts::{
    Capability, Digest, ErrorCode, ErrorEnvelope, ExecutionScope, Origin, Partition, Resolution,
    RetryPolicy, RunId, RunIdentity, ScopeRole, Split, VisualBundleId, VisualObservationBundle,
};
use ignis_core::services::ServiceResult;

/// One row of the data lane's clip → run map (the fields identity needs).
#[derive(Debug, Clone, PartialEq)]
pub struct ClipMapEntry {
    pub file_name: String,
    pub run_id: Option<String>,
    /// `resolved` | other (as written by the map)
    pub resolution: String,
}

/// The clip → run map of the ingested BASS records (empty when they are not ingested).
pub fn clip_map() -> anyhow::Result<Vec<ClipMapEntry>> {
    let d = ignis_data::paths::work(&["records", "bass"]);
    if !d.join("experiments.jsonl").exists() {
        return Ok(Vec::new());
    }
    let b = ignis_data::store::load("bass")?;
    Ok(ignis_data::families::bass::video_map(&b)
        .into_iter()
        .map(|v| ClipMapEntry {
            file_name: v.file_name,
            run_id: v.run_id,
            resolution: v.resolution,
        })
        .collect())
}

/// The run identity of `file_name`: `declared` (command line, else a single `applies_to` of the
/// spec) cross-checked against the clip map.
pub fn resolve_identity(
    file_name: &str,
    declared: Option<&str>,
    applies_to: &[String],
    map: &[ClipMapEntry],
) -> anyhow::Result<RunIdentity> {
    let declared = declared
        .map(String::from)
        .or_else(|| (applies_to.len() == 1).then(|| applies_to[0].clone()));
    let entry = map.iter().find(|e| e.file_name == file_name);
    let mapped = entry
        .filter(|e| e.resolution == "resolved")
        .and_then(|e| e.run_id.clone());
    let (run, resolution) = match (declared, mapped) {
        (Some(d), Some(m)) if d == m => (Some(d), Resolution::Resolved),
        (Some(d), Some(_)) => (Some(d), Resolution::UnresolvedConflict),
        (None, Some(m)) => (Some(m), Resolution::Resolved),
        (Some(d), None) => (Some(d), Resolution::Unknown),
        (None, None) => (None, Resolution::Unknown),
    };
    Ok(RunIdentity {
        run_id: run.map(RunId::new).transpose()?,
        resolution,
    })
}

/// Partition of every run in the frozen splits.
#[derive(Debug, Clone, Default)]
pub struct RunPartitions {
    by_run: HashMap<String, Partition>,
}

impl RunPartitions {
    pub fn new(splits: &[Split]) -> RunPartitions {
        let mut by_run = HashMap::new();
        for s in splits {
            for r in s.run_to_group.keys() {
                if let Some(p) = s.partition_of(r) {
                    by_run.insert(r.clone(), p);
                }
            }
        }
        RunPartitions { by_run }
    }

    /// Every frozen split under `data/splits/` (content hash verified).
    pub fn load() -> anyhow::Result<RunPartitions> {
        let mut splits = Vec::new();
        for f in ignis_data::cohort::split_files() {
            splits.push(ignis_data::cohort::load_split(&f.display().to_string())?);
        }
        Ok(RunPartitions::new(&splits))
    }

    pub fn partition(&self, run: &str) -> Option<Partition> {
        self.by_run.get(run).copied()
    }
}

fn err(code: ErrorCode, msg: impl Into<String>) -> ErrorEnvelope {
    ErrorEnvelope::new(code, msg, RetryPolicy::Never)
}

/// Bundles under the vision artifact root (`artifacts/vision/**/visual_bundle.json`).
#[derive(Debug, Clone)]
pub struct VisualBundleStore {
    pub root: PathBuf,
    pub partitions: RunPartitions,
}

impl VisualBundleStore {
    pub fn open(root: PathBuf, partitions: RunPartitions) -> VisualBundleStore {
        VisualBundleStore { root, partitions }
    }

    /// `artifacts/vision/` and the frozen splits.
    pub fn open_default() -> anyhow::Result<VisualBundleStore> {
        Ok(VisualBundleStore::open(
            crate::pipeline::default_out_root(),
            RunPartitions::load()?,
        ))
    }

    /// Every bundle on disk with its directory (each one validated; an invalid one is an error).
    pub fn all(&self) -> anyhow::Result<Vec<(PathBuf, VisualObservationBundle)>> {
        let mut out = Vec::new();
        if !self.root.exists() {
            return Ok(out);
        }
        let mut stack = vec![self.root.clone()];
        while let Some(d) = stack.pop() {
            for e in std::fs::read_dir(&d)? {
                let p = e?.path();
                if p.is_dir() {
                    stack.push(p);
                } else if p.file_name().is_some_and(|n| n == "visual_bundle.json") {
                    let b: VisualObservationBundle =
                        serde_json::from_str(&std::fs::read_to_string(&p)?)
                            .map_err(|e| anyhow::anyhow!("{}: {e}", p.display()))?;
                    b.validate()
                        .map_err(|e| anyhow::anyhow!("{}: {e}", p.display()))?;
                    out.push((p.parent().unwrap().to_path_buf(), b));
                }
            }
        }
        out.sort_by(|a, b| a.1.bundle_id.cmp(&b.1.bundle_id));
        Ok(out)
    }

    /// Whether `scope` may receive `b` (see the module docs).
    pub fn admit(&self, scope: &ExecutionScope, b: &VisualObservationBundle) -> ServiceResult<()> {
        scope
            .validate()
            .map_err(|e| err(ErrorCode::ScopeDenied, e.to_string()))?;
        scope.require(Capability::ReadData)?;
        match (&b.run.run_id, b.run.resolution) {
            (Some(r), Resolution::Resolved) => {
                if let Some(p) = self.partitions.partition(r.as_str()) {
                    scope.require_partition(p)?;
                }
                Ok(())
            }
            (None, _) if b.origin == Origin::SyntheticFixture => Ok(()),
            _ if scope.role == ScopeRole::Evaluation => Ok(()),
            _ => Err(err(
                ErrorCode::ScopeDenied,
                format!(
                    "{}: the clip's run identity is {:?}; it could show a sealed run",
                    b.bundle_id, b.run.resolution
                ),
            )),
        }
    }

    fn find(&self, id: &VisualBundleId) -> ServiceResult<(PathBuf, VisualObservationBundle)> {
        self.all()
            .map_err(|e| err(ErrorCode::Internal, format!("{e:#}")))?
            .into_iter()
            .find(|(_, b)| b.bundle_id == *id)
            .ok_or_else(|| {
                err(
                    ErrorCode::NotFound,
                    format!("visual bundle {id} is not in this store"),
                )
            })
    }

    pub fn bundle(
        &self,
        scope: &ExecutionScope,
        id: &VisualBundleId,
    ) -> ServiceResult<VisualObservationBundle> {
        let (_, b) = self.find(id)?;
        self.admit(scope, &b)?;
        Ok(b)
    }

    /// The bundle's contour records after the same checks, verified against their digest.
    pub fn records_path(
        &self,
        scope: &ExecutionScope,
        id: &VisualBundleId,
    ) -> ServiceResult<PathBuf> {
        let (dir, b) = self.find(id)?;
        self.admit(scope, &b)?;
        let name = match b.contours.contract.as_deref() {
            Some(ignis_core::contracts::IMAGE_CONTOURS) => "image_contours.jsonl",
            _ => "contours.jsonl",
        };
        let p = dir.join(name);
        let bytes = std::fs::read(&p)
            .map_err(|e| err(ErrorCode::SourceMissing, format!("{}: {e}", p.display())))?;
        if Digest::of_bytes(&bytes) != b.contours.sha256 {
            return Err(err(
                ErrorCode::Internal,
                format!("{}: bytes do not match the bundle", p.display()),
            ));
        }
        Ok(p)
    }
}

/// `contours.jsonl` files written before 0.3.0 whose records hold pixel coordinates under the metric
/// contract (`ContourObservation`, flagged only in a note). They are loaded as metres by readers of
/// `contours.jsonl`; [`quarantine_legacy`] renames them so nothing reads them as metres again.
pub fn legacy_pixel_contours(root: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    if !root.exists() {
        return Ok(out);
    }
    let mut stack = vec![root.to_path_buf()];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d)? {
            let p = e?.path();
            if p.is_dir() {
                stack.push(p);
            } else if p.file_name().is_some_and(|n| n == "contours.jsonl") {
                use std::io::BufRead;
                let mut first = String::new();
                std::io::BufReader::new(std::fs::File::open(&p)?).read_line(&mut first)?;
                if first.contains("UNCALIBRATED: s_m/f_m hold pixel coordinates") {
                    out.push(p);
                }
            }
        }
    }
    out.sort();
    Ok(out)
}

/// Rename each legacy file to `contours.pixel-legacy.jsonl` (reversible; bytes unchanged).
pub fn quarantine_legacy(files: &[PathBuf]) -> anyhow::Result<Vec<PathBuf>> {
    let mut moved = Vec::new();
    for f in files {
        let to = f.with_file_name("contours.pixel-legacy.jsonl");
        anyhow::ensure!(!to.exists(), "{} already exists", to.display());
        std::fs::rename(f, &to)?;
        moved.push(to);
    }
    Ok(moved)
}
