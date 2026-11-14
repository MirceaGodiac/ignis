//! Evaluation protocols and their freeze (master design §16.8).
//!
//! A protocol fixes what an evaluation measures: target and unit, the dataset bundle, split and
//! partition, the cutoffs and the versioned metrics. `frozen_hash` is the digest of the canonical
//! JSON of the parsed protocol (so formatting does not change it). Freezing is write-once: the same
//! id and version can never be frozen again with different content; a changed protocol is a new
//! version, and old results keep pointing at the old one.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use ignis_core::contracts::{
    Digest, EvaluationRequest, Finite, MetricSpec, Partition, ProtocolRef, Quantity,
};
use serde::{Deserialize, Serialize};

pub const PROTOCOL_SCHEMA: &str = "evaluation_protocol.v1";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Protocol {
    pub schema_version: String,
    pub protocol_id: String,
    pub version: String,
    pub target: Quantity,
    pub unit: String,
    pub dataset_hash: Digest,
    pub split_id: String,
    pub partition: Partition,
    pub cutoffs_s: Vec<Finite>,
    pub metrics: Vec<MetricSpec>,
    /// Human description of targets, groups and exclusions (part of the hash: wording is reviewed too).
    #[serde(default)]
    pub notes: String,
}

impl Protocol {
    pub fn frozen_hash(&self) -> Digest {
        Digest::of_json(self)
    }

    pub fn reference(&self) -> ProtocolRef {
        ProtocolRef {
            protocol_id: self.protocol_id.clone(),
            version: self.version.clone(),
            frozen_hash: Some(self.frozen_hash()),
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != PROTOCOL_SCHEMA {
            return Err(format!("schema_version must be {PROTOCOL_SCHEMA}"));
        }
        if !file_safe(&self.protocol_id) || !file_safe(&self.version) {
            return Err(format!(
                "protocol id and version use only letters, digits, '-', '_' and '.', and do not start with '.' (got {:?}@{:?})",
                self.protocol_id, self.version
            ));
        }
        if self.unit != self.target.unit_si() {
            return Err(format!(
                "{:?} is scored in {}",
                self.target,
                self.target.unit_si()
            ));
        }
        if self.metrics.is_empty() || self.cutoffs_s.is_empty() {
            return Err("a protocol needs metrics and cutoffs".into());
        }
        if self.cutoffs_s[0].get() < 0.0
            || self.cutoffs_s.windows(2).any(|w| w[0].get() >= w[1].get())
        {
            return Err("cutoffs are non-negative and strictly increasing".into());
        }
        Ok(())
    }

    /// Every protocol field of the request must equal the frozen protocol.
    pub fn check_request(&self, req: &EvaluationRequest) -> Result<(), String> {
        let mut diff = vec![];
        let r = &req.protocol;
        if r.protocol_id != self.protocol_id || r.version != self.version {
            diff.push(format!(
                "protocol {}@{} (frozen: {}@{})",
                r.protocol_id, r.version, self.protocol_id, self.version
            ));
        }
        if r.frozen_hash.as_ref() != Some(&self.frozen_hash()) {
            diff.push("frozen_hash".into());
        }
        if req.target != self.target {
            diff.push("target".into());
        }
        if req.unit != self.unit {
            diff.push("unit".into());
        }
        if req.dataset_hash != self.dataset_hash {
            diff.push("dataset_hash".into());
        }
        if req.split_id != self.split_id {
            diff.push("split_id".into());
        }
        if req.partition != self.partition {
            diff.push("partition".into());
        }
        if req.cutoffs_s != self.cutoffs_s {
            diff.push("cutoffs_s".into());
        }
        if req.metrics != self.metrics {
            diff.push("metrics".into());
        }
        if diff.is_empty() {
            Ok(())
        } else {
            Err(format!(
                "request differs from the frozen protocol: {}",
                diff.join(", ")
            ))
        }
    }
}

/// A frozen protocol as stored: the protocol plus the hash recorded at freeze time.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrozenProtocol {
    pub protocol: Protocol,
    pub frozen_hash: Digest,
}

impl FrozenProtocol {
    /// Re-derives the hash: a hand-edited frozen file no longer matches and is refused.
    pub fn verify(&self) -> Result<(), String> {
        self.protocol.validate()?;
        if self.protocol.frozen_hash() != self.frozen_hash {
            return Err(format!(
                "frozen protocol {}@{} was modified after its freeze",
                self.protocol.protocol_id, self.protocol.version
            ));
        }
        Ok(())
    }
}

/// Ids and versions are used as file names as they are: no character is replaced, so two different
/// protocols can never share one frozen file.
fn file_safe(s: &str) -> bool {
    !s.is_empty()
        && !s.starts_with('.')
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || "-_.".contains(c))
}

fn frozen_path(dir: &Path, p: &Protocol) -> PathBuf {
    dir.join(format!("{}@{}.frozen.json", p.protocol_id, p.version))
}

/// Freeze `p` under `dir`. Idempotent for identical content; refuses a different protocol under an
/// already frozen id and version.
pub fn freeze(dir: &Path, p: &Protocol) -> anyhow::Result<FrozenProtocol> {
    p.validate().map_err(anyhow::Error::msg)?;
    let frozen = FrozenProtocol {
        protocol: p.clone(),
        frozen_hash: p.frozen_hash(),
    };
    let path = frozen_path(dir, p);
    if path.exists() {
        let old: FrozenProtocol = serde_json::from_str(&std::fs::read_to_string(&path)?)?;
        old.verify().map_err(anyhow::Error::msg)?;
        if old.frozen_hash != frozen.frozen_hash {
            anyhow::bail!(
                "protocol {}@{} is already frozen with {}; a changed protocol needs a new version",
                p.protocol_id,
                p.version,
                old.frozen_hash
            );
        }
        return Ok(old);
    }
    std::fs::create_dir_all(dir)?;
    // Atomic and write-once: the complete file is written and synced under a temporary name, then
    // hard-linked to its final name, which fails if that name exists. A crash leaves at most a
    // temporary file, never an empty or partial frozen protocol.
    let tmp = dir.join(format!(
        ".{}@{}.{}.tmp",
        p.protocol_id,
        p.version,
        std::process::id()
    ));
    {
        let mut f = std::fs::File::create(&tmp)?;
        std::io::Write::write_all(&mut f, serde_json::to_string_pretty(&frozen)?.as_bytes())?;
        f.sync_all()?;
    }
    let linked = std::fs::hard_link(&tmp, &path);
    let _ = std::fs::remove_file(&tmp);
    match linked {
        Ok(()) => Ok(frozen),
        // A concurrent freeze won: accept it only if it froze the same content.
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => freeze(dir, p),
        Err(e) => Err(e.into()),
    }
}

/// Load every `*.frozen.json` under `dir`, keyed by (protocol_id, version). Fails closed on any
/// file that does not verify.
pub fn load_frozen(dir: &Path) -> anyhow::Result<BTreeMap<(String, String), FrozenProtocol>> {
    let mut out = BTreeMap::new();
    if !dir.exists() {
        return Ok(out);
    }
    let mut paths: Vec<PathBuf> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.to_string_lossy().ends_with(".frozen.json"))
        .collect();
    paths.sort();
    for p in paths {
        let f: FrozenProtocol = serde_json::from_str(&std::fs::read_to_string(&p)?)
            .map_err(|e| anyhow::anyhow!("{}: {e}", p.display()))?;
        f.verify()
            .map_err(|e| anyhow::anyhow!("{}: {e}", p.display()))?;
        let key = (f.protocol.protocol_id.clone(), f.protocol.version.clone());
        if out.insert(key.clone(), f).is_some() {
            anyhow::bail!(
                "protocol {}@{} is frozen twice under {}",
                key.0,
                key.1,
                dir.display()
            );
        }
    }
    Ok(out)
}
