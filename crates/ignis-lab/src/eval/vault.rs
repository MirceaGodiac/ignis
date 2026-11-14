//! Sealed vault: hash commitments before continuations are opened.
//!
//! Layout
//! ```text
//! holdout/<split-hex>/<partition>/<sha(run_id)[:32]>.json   continuation payloads (gitignored root)
//! holdout/<split-hex>/<partition>/<sha(run_id)[:32]>.cutoff-<g>.json  continuation of one landmark
//! runs/evaluation/artifacts/<artifact-hex>.json             exact bytes of every sealed artifact
//! runs/evaluation/seals/<artifact-hex>.json                 seal record (SealedPrediction + binding)
//! runs/evaluation/splits/<split-hex>.json                   frozen copy of the split used at seal time
//! runs/evaluation/reveal_log.jsonl                          every reveal attempt, granted or refused
//! ```
//!
//! Continuation payload (written by the data pipeline through [`deposit_continuation`]):
//! `{"run_id", "session_id"?, "time_origin"?, "cutoff_s"?, "observations": [{"time_s", "value_si",
//! "quantity", "valid"}], "endpoint": {"censoring", "t_lower_s", "t_upper_s", "allowed_causes"},
//! "front": {"t_s", "s_m", "f_m"}}`. Every section is optional; scorers report what they could not
//! compute and why.
//!
//! Isolation: the holdout root is read only by [`reveal`], which needs a [`VaultAccess`] token that
//! only the `eval` module can mint, and which refuses (and logs) while any lab session is active.

use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::Context;
use ignis_core::contracts::{AnalysisArtifact, Partition, SealedPrediction, Split, SplitPartition};
use ignis_core::hashing::{content_hash, hex_part, key_of};
use ignis_core::registry::load_release;
use serde_json::{json, Map, Value};

use super::VaultAccess;
use crate::access;
use crate::util::{eval_root, fmt_g, holdout_root, now_rfc3339, refused, sha256_hex, write_once};

/// Commitment over every field of the artifact (identity, numbers, status, timestamps).
pub fn artifact_hash(a: &AnalysisArtifact) -> String {
    key_of(&serde_json::to_value(a).expect("artifact serializes"))
}

fn run_key(run_id: &str) -> String {
    sha256_hex(run_id.as_bytes())[..32].to_string()
}

/// File stem of a continuation: per run, or per run and landmark.
fn continuation_stem(run_id: &str, cutoff_s: Option<f64>) -> String {
    match cutoff_s {
        None => run_key(run_id),
        Some(c) => format!("{}.cutoff-{}", run_key(run_id), fmt_g(c)),
    }
}

/// Ids become file names: keep them inside their directory.
fn safe_name(id: &str) -> anyhow::Result<&str> {
    let h = hex_part(id);
    if h.is_empty()
        || !h
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
        || h.contains("..")
    {
        return Err(refused(format!(
            "id {id:?} is not a valid file-safe identifier"
        )));
    }
    Ok(h)
}

fn partition_name(p: Partition) -> &'static str {
    match p {
        Partition::Train => "train",
        Partition::Validation => "validation",
        Partition::Test => "test",
    }
}

fn split_text(split: &Split) -> String {
    serde_json::to_string_pretty(split).expect("split serializes")
}

fn split_hash(split: &Split) -> String {
    content_hash(&serde_json::to_value(split).expect("split serializes"), &[])
}

fn frozen_split(split_id: &str) -> anyhow::Result<Split> {
    let p = eval_root()
        .join("splits")
        .join(format!("{}.json", safe_name(split_id)?));
    Ok(serde_json::from_str(
        &std::fs::read_to_string(&p).with_context(|| format!("frozen split {}", p.display()))?,
    )?)
}

fn user() -> String {
    std::env::var("USERNAME")
        .or_else(|_| std::env::var("USER"))
        .unwrap_or_else(|_| "unknown".into())
}

fn host() -> String {
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "unknown".into())
}

fn log_path() -> PathBuf {
    eval_root().join("reveal_log.jsonl")
}

fn log(entry: &Map<String, Value>) -> anyhow::Result<()> {
    let p = log_path();
    if let Some(d) = p.parent() {
        std::fs::create_dir_all(d)?;
    }
    let mut e = Map::new();
    e.insert("at".into(), json!(now_rfc3339()));
    e.insert("user".into(), json!(user()));
    e.insert("host".into(), json!(host()));
    for (k, v) in entry {
        e.insert(k.clone(), v.clone());
    }
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&p)?;
    writeln!(f, "{}", serde_json::to_string(&Value::Object(e))?)?;
    Ok(())
}

/// Every reveal attempt, granted or refused, in order.
pub fn reveal_log() -> Vec<Value> {
    std::fs::read_to_string(log_path())
        .map(|t| {
            t.lines()
                .filter(|l| !l.trim().is_empty())
                .filter_map(|l| serde_json::from_str(l).ok())
                .collect()
        })
        .unwrap_or_default()
}

// ------------------------------------------------------------------------------------ continuations

/// Store a validation/test continuation. Train runs are not held out and are refused.
pub fn deposit_continuation(
    split: &Split,
    run_id: &str,
    payload: Value,
) -> anyhow::Result<PathBuf> {
    let part = split
        .partition_of(run_id)
        .ok_or_else(|| refused(format!("run {run_id} is not in split {}", split.id)))?;
    if part == Partition::Train {
        return Err(refused(format!(
            "{run_id} is a train run; only validation/test continuations are sealed"
        )));
    }
    let Value::Object(mut body) = payload else {
        return Err(refused("continuation payload must be a JSON object"));
    };
    if body
        .get("run_id")
        .is_some_and(|r| r.as_str() != Some(run_id))
    {
        return Err(refused("payload run_id differs from the declared run"));
    }
    body.insert("run_id".into(), json!(run_id));
    body.insert("split_id".into(), json!(split.id));
    body.insert("partition".into(), json!(partition_name(part)));
    let path = holdout_root()
        .join(safe_name(&split.id)?)
        .join(partition_name(part))
        .join(format!("{}.json", run_key(run_id)));
    write_once(&path, &serde_json::to_string_pretty(&Value::Object(body))?)?;
    Ok(path)
}

/// Store the continuation of a validation/test run at one landmark. Refuses any observation at or
/// before the cutoff: a value a forecast may see is never part of the hidden side.
pub fn deposit_landmark(
    split: &Split,
    run_id: &str,
    cutoff_s: f64,
    payload: Value,
) -> anyhow::Result<PathBuf> {
    let part = split
        .partition_of(run_id)
        .ok_or_else(|| refused(format!("run {run_id} is not in split {}", split.id)))?;
    if part == Partition::Train {
        return Err(refused(format!(
            "{run_id} is a train run; only validation/test continuations are sealed"
        )));
    }
    let Value::Object(mut body) = payload else {
        return Err(refused("continuation payload must be a JSON object"));
    };
    if body
        .get("run_id")
        .is_some_and(|r| r.as_str() != Some(run_id))
    {
        return Err(refused("payload run_id differs from the declared run"));
    }
    let obs = body
        .get("observations")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if let Some(o) = obs.iter().find(|o| {
        o.get("time_s")
            .and_then(Value::as_f64)
            .is_none_or(|t| t <= cutoff_s)
    }) {
        return Err(refused(format!("continuation of {run_id} at {cutoff_s} s holds an observation that is not after the cutoff: {o}")));
    }
    body.insert("run_id".into(), json!(run_id));
    body.insert("cutoff_s".into(), json!(cutoff_s));
    body.insert("split_id".into(), json!(split.id));
    body.insert("partition".into(), json!(partition_name(part)));
    let path = holdout_root()
        .join(safe_name(&split.id)?)
        .join(partition_name(part))
        .join(format!(
            "{}.json",
            continuation_stem(run_id, Some(cutoff_s))
        ));
    write_once(&path, &serde_json::to_string_pretty(&Value::Object(body))?)?;
    Ok(path)
}

/// The continuation a seal refers to: the landmark file for its cutoff, else the run-level deposit.
fn read_continuation(
    _access: &VaultAccess,
    split_id: &str,
    partition: &str,
    run_id: &str,
    cutoff_s: Option<f64>,
) -> anyhow::Result<Value> {
    access::assert_not_lab()?;
    let dir = holdout_root().join(safe_name(split_id)?).join(partition);
    let landmark = cutoff_s
        .map(|_| dir.join(format!("{}.json", continuation_stem(run_id, cutoff_s))))
        .filter(|p| p.exists());
    let path =
        landmark.unwrap_or_else(|| dir.join(format!("{}.json", continuation_stem(run_id, None))));
    if !path.exists() {
        anyhow::bail!("no {partition} continuation deposited for run {run_id}");
    }
    let data: Value = serde_json::from_str(&std::fs::read_to_string(&path)?)?;
    if data.get("run_id").and_then(Value::as_str) != Some(run_id)
        || data.get("partition").and_then(Value::as_str) != Some(partition)
    {
        return Err(refused(format!(
            "continuation file {} does not belong to {run_id}/{partition}",
            path.display()
        )));
    }
    Ok(data)
}

// ---------------------------------------------------------------------------------------- sealing

fn seal_path(artifact_id: &str) -> anyhow::Result<PathBuf> {
    Ok(eval_root()
        .join("seals")
        .join(format!("{}.json", safe_name(artifact_id)?)))
}

fn artifact_path(artifact_id: &str) -> anyhow::Result<PathBuf> {
    Ok(eval_root()
        .join("artifacts")
        .join(format!("{}.json", safe_name(artifact_id)?)))
}

/// Commit to an artifact before its continuation is opened.
///
/// The partition comes from the frozen split, never from the caller. A test run whose continuation
/// has already been revealed cannot receive new sealed predictions (they would be development).
pub fn seal(
    artifact: &AnalysisArtifact,
    run_id: &str,
    split: &Split,
    artifacts: &Path,
) -> anyhow::Result<Value> {
    seal_at(artifact, run_id, split, artifacts, None)
}

/// [`seal`], recording the cutoff of the forecast so that a reveal opens that landmark's continuation.
pub fn seal_at(
    artifact: &AnalysisArtifact,
    run_id: &str,
    split: &Split,
    artifacts: &Path,
    cutoff_s: Option<f64>,
) -> anyhow::Result<Value> {
    if seal_path(&artifact.id)?.exists() {
        return Err(refused(format!(
            "artifact {} is already sealed (one commitment per artifact)",
            artifact.id
        )));
    }
    let (release, _, _) = load_release(artifacts, &artifact.model_release_id, None)?;
    if release.split_id != split.id {
        return Err(refused(format!(
            "release {} was fitted against split {}, not {}",
            release.id, release.split_id, split.id
        )));
    }
    let part = split
        .partition_of(run_id)
        .ok_or_else(|| refused(format!("run {run_id} is not in split {}", split.id)))?;
    if part == Partition::Train {
        return Err(refused(format!("{run_id} is a train run; predictions on train are fit diagnostics, not sealed forecasts")));
    }
    let previously = reveal_log().iter().any(|e| {
        e.get("granted") == Some(&json!(true))
            && e.get("run_id").and_then(Value::as_str) == Some(run_id)
            && e.get("split_id").and_then(Value::as_str) == Some(split.id.as_str())
    });
    if part == Partition::Test && previously {
        return Err(refused(format!("test continuation of {run_id} was already revealed; new test predictions are not prospective")));
    }
    let sealed_at = chrono::Utc::now();
    if let Ok(created) = chrono::DateTime::parse_from_rfc3339(&artifact.created_at) {
        if created > sealed_at {
            return Err(refused("artifact claims a creation time after the seal"));
        }
    }
    // The commitment is over the stored bytes as they will be read back at reveal time.
    let art_text = serde_json::to_string_pretty(artifact)?;
    let stored: AnalysisArtifact = serde_json::from_str(&art_text)?;
    let sp = SealedPrediction {
        artifact_id: artifact.id.clone(),
        artifact_hash: artifact_hash(&stored),
        sealed_at: sealed_at.to_rfc3339_opts(chrono::SecondsFormat::Micros, true),
        split_partition: if part == Partition::Test {
            SplitPartition::Test
        } else {
            SplitPartition::Validation
        },
        run_id: Some(run_id.to_string()),
        split_id: Some(split.id.clone()),
        prefix_hash: Some(artifact.prefix_hash.clone()),
    };
    let mut body = Map::new();
    body.insert("sealed".into(), serde_json::to_value(&sp)?);
    body.insert("run_id".into(), json!(run_id));
    body.insert("split_id".into(), json!(split.id));
    body.insert("split_hash".into(), json!(split_hash(split)));
    body.insert("model_release_id".into(), json!(artifact.model_release_id));
    body.insert("prefix_hash".into(), json!(artifact.prefix_hash));
    body.insert("settings_hash".into(), json!(artifact.settings_hash));
    body.insert("continuation_previously_revealed".into(), json!(previously));
    if let Some(c) = cutoff_s {
        body.insert("cutoff_s".into(), json!(c));
    }
    let rh = content_hash(&Value::Object(body.clone()), &[]);
    body.insert("record_hash".into(), json!(rh));
    write_once(
        &eval_root()
            .join("splits")
            .join(format!("{}.json", safe_name(&split.id)?)),
        &split_text(split),
    )?;
    write_once(&artifact_path(&artifact.id)?, &art_text)?;
    let body = Value::Object(body);
    write_once(
        &seal_path(&artifact.id)?,
        &serde_json::to_string_pretty(&body)?,
    )?;
    Ok(body)
}

pub fn load_seal(artifact_id: &str) -> anyhow::Result<Option<Value>> {
    let p = seal_path(artifact_id)?;
    if !p.exists() {
        return Ok(None);
    }
    Ok(Some(serde_json::from_str(&std::fs::read_to_string(p)?)?))
}

pub fn list_seals() -> Vec<Value> {
    let d = eval_root().join("seals");
    let Ok(rd) = std::fs::read_dir(&d) else {
        return vec![];
    };
    let mut paths: Vec<PathBuf> = rd
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    paths.sort();
    paths
        .iter()
        .filter_map(|p| std::fs::read_to_string(p).ok())
        .filter_map(|t| serde_json::from_str(&t).ok())
        .collect()
}

/// The stored artifact (predictions only; no outcomes).
pub fn sealed_artifact(artifact_id: &str) -> anyhow::Result<AnalysisArtifact> {
    let p = artifact_path(artifact_id)?;
    let text =
        std::fs::read_to_string(&p).with_context(|| format!("sealed artifact {artifact_id}"))?;
    Ok(serde_json::from_str(&text)?)
}

// ---------------------------------------------------------------------------------------- reveal

pub struct Revealed {
    pub seal: Value,
    pub artifact: AnalysisArtifact,
    pub continuation: Value,
    pub group_id: Option<String>,
}

/// Open the continuation for a sealed artifact. The only reader of the holdout root.
pub fn reveal(
    access: &VaultAccess,
    artifact_id: &str,
    final_test: bool,
    purpose: &str,
) -> anyhow::Result<Revealed> {
    let mut entry = Map::new();
    entry.insert("artifact_id".into(), json!(artifact_id));
    entry.insert("final_test".into(), json!(final_test));
    entry.insert("purpose".into(), json!(purpose));
    let refuse = |entry: &Map<String, Value>, reason: String| -> anyhow::Error {
        let mut e = entry.clone();
        e.insert("granted".into(), json!(false));
        e.insert("reason".into(), json!(reason));
        if let Err(le) = log(&e) {
            return anyhow::anyhow!("reveal log unwritable ({le}); refused: {reason}");
        }
        refused(reason)
    };
    if let Err(e) = access::assert_not_lab() {
        return Err(refuse(&entry, e.to_string()));
    }
    let rec = match load_seal(artifact_id) {
        Ok(Some(r)) => r,
        Ok(None) => {
            return Err(refuse(
                &entry,
                format!(
                    "no SealedPrediction recorded for {artifact_id}; run `ignis eval seal` first"
                ),
            ))
        }
        Err(e) => return Err(refuse(&entry, e.to_string())),
    };
    let mut unhashed = rec.as_object().cloned().unwrap_or_default();
    let recorded = unhashed.remove("record_hash");
    if recorded.as_ref().and_then(Value::as_str)
        != Some(content_hash(&Value::Object(unhashed), &[]).as_str())
    {
        return Err(refuse(
            &entry,
            "seal record was modified after it was written".into(),
        ));
    }
    let sp: SealedPrediction = match serde_json::from_value(rec["sealed"].clone()) {
        Ok(s) => s,
        Err(e) => return Err(refuse(&entry, format!("seal record unreadable: {e}"))),
    };
    let run_id = rec["run_id"].as_str().unwrap_or_default().to_string();
    let split_id = rec["split_id"].as_str().unwrap_or_default().to_string();
    let sp_part = match sp.split_partition {
        SplitPartition::Validation => "validation",
        SplitPartition::Test => "test",
    };
    entry.insert("run_id".into(), json!(run_id));
    entry.insert("split_id".into(), json!(split_id));
    entry.insert("partition".into(), json!(sp_part));
    let art = match sealed_artifact(artifact_id) {
        Ok(a) => a,
        Err(_) => return Err(refuse(&entry, "sealed artifact bytes are missing".into())),
    };
    if artifact_hash(&art) != sp.artifact_hash {
        return Err(refuse(
            &entry,
            "stored artifact no longer matches its commitment".into(),
        ));
    }
    let split = match frozen_split(&split_id) {
        Ok(s) => s,
        Err(e) => return Err(refuse(&entry, format!("frozen split copy unreadable: {e}"))),
    };
    if Some(split_hash(&split).as_str()) != rec["split_hash"].as_str() {
        return Err(refuse(
            &entry,
            "frozen split copy differs from the one recorded at seal time".into(),
        ));
    }
    let part = split.partition_of(&run_id).map(partition_name);
    if part != Some(sp_part) {
        return Err(refuse(
            &entry,
            "partition recorded in the seal disagrees with the frozen split".into(),
        ));
    }
    if sp_part == "test" && !final_test {
        return Err(refuse(
            &entry,
            "test-partition reveal requires --final-test (final, logged, not repeatable as development)".into(),
        ));
    }
    let cont = match read_continuation(
        access,
        &split_id,
        sp_part,
        &run_id,
        rec.get("cutoff_s").and_then(Value::as_f64),
    ) {
        Ok(c) => c,
        Err(e) => return Err(refuse(&entry, format!("{e:#}"))),
    };
    let group = split.run_to_group.get(&run_id).cloned();
    let mut granted = entry.clone();
    granted.insert("granted".into(), json!(true));
    log(&granted)?;
    Ok(Revealed {
        seal: rec,
        artifact: art,
        continuation: cont,
        group_id: group,
    })
}
