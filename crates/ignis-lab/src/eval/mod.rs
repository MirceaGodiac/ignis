//! Sealed evaluation: commitments before reveal, proper scores, group-cluster uncertainty.
//!
//! Isolation rules enforced in code:
//! * continuations of validation/test runs live under a separate root (`IGNIS_HOLDOUT`, default
//!   `holdout/`, gitignored) and are read only by [`vault::reveal`];
//! * `reveal` needs a [`VaultAccess`] token, which only this module can mint (the lab module
//!   cannot construct one), and refuses while a lab session is active ([`crate::access`]);
//! * `reveal` requires a `SealedPrediction` recorded earlier for the exact artifact bytes;
//! * a test-partition reveal requires `--final-test` and every attempt is logged.

pub mod bootstrap;
pub mod compare;
pub mod deposit;
pub mod protocol;
pub mod report;
pub mod scorecard;
pub mod scores;
pub mod scoring;
pub mod service;
pub mod vault;

use std::path::PathBuf;

use clap::Subcommand;
use ignis_core::contracts::AnalysisArtifact;
use ignis_core::hashing::hex_part;
use serde_json::{json, Value};

use crate::util::{artifacts_root, emit, load_split, parse_cutoffs, refused};

/// Capability to read sealed continuations. Constructible only inside `crate::eval`.
pub struct VaultAccess {
    _private: (),
}

fn grant() -> VaultAccess {
    VaultAccess { _private: () }
}

#[derive(Debug, Subcommand)]
pub enum EvalCmd {
    /// Store a validation/test continuation in the holdout root (data pipeline hand-off).
    Deposit {
        /// Split JSON path or id.
        split: String,
        /// Continuation payload JSON.
        payload: PathBuf,
        /// Run id the continuation belongs to.
        #[arg(long)]
        run: String,
    },
    /// Build and store the hidden continuations of a cohort's validation/test runs at the landmarks.
    DepositCohort {
        #[arg(long)]
        cohort: String,
        #[arg(long)]
        split: String,
        /// Cutoffs in s, comma-separated (default: configs/evaluation/landmarks.toml).
        #[arg(long)]
        cutoffs: Option<String>,
        /// validation | test | all
        #[arg(long, default_value = "all")]
        partition: String,
        /// 'synthetic': the generated fixture the release was fitted on.
        #[arg(long)]
        fixture: Option<String>,
        #[arg(long, default_value_t = 24)]
        fixture_runs: usize,
        #[arg(long, default_value_t = 0)]
        seed: u64,
    },
    /// Record a SealedPrediction (hash commitment) for an AnalysisArtifact JSON file.
    Seal {
        artifact: PathBuf,
        /// Run id the artifact forecasts.
        #[arg(long)]
        run: String,
        /// Split JSON path or id.
        #[arg(long)]
        split: String,
    },
    /// List seal records (no outcomes).
    Seals {
        #[arg(long)]
        release: Option<String>,
        #[arg(long)]
        partition: Option<String>,
    },
    /// Open the continuation of a sealed artifact (logged). Refuses without a prior seal.
    Reveal {
        artifact_id: String,
        /// Required for test runs.
        #[arg(long)]
        final_test: bool,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Reveal and score sealed artifacts; aggregate with group-cluster bootstrap and refusal accounting.
    Score {
        /// Sealed artifact ids.
        artifact_ids: Vec<String>,
        /// Score every sealed artifact of this release.
        #[arg(long)]
        release: Option<String>,
        /// With --release: validation | test.
        #[arg(long, default_value = "validation")]
        partition: String,
        #[arg(long)]
        final_test: bool,
        /// Event horizons in s, comma-separated.
        #[arg(long)]
        horizons: Option<String>,
        #[arg(long, default_value_t = 2000)]
        n_boot: usize,
        #[arg(long, default_value_t = 0)]
        seed: u64,
        #[arg(long)]
        out: Option<PathBuf>,
        /// Omit the per-case table.
        #[arg(long)]
        no_cases: bool,
    },
    /// Score releases on exactly the same admissible cases, same metrics; paired cluster-bootstrap deltas.
    Compare {
        #[arg(required = true)]
        release_ids: Vec<String>,
        #[arg(long, default_value = "validation")]
        partition: String,
        #[arg(long)]
        final_test: bool,
        #[arg(long)]
        allow_feature_mismatch: bool,
        #[arg(long)]
        allow_unit_mismatch: bool,
        #[arg(long)]
        allow_cohort_mismatch: bool,
        /// Restrict to cases sealed by every release.
        #[arg(long)]
        intersect: bool,
        #[arg(long)]
        horizons: Option<String>,
        #[arg(long, default_value_t = 2000)]
        n_boot: usize,
        #[arg(long, default_value_t = 0)]
        seed: u64,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Summarise a score/compare JSON result (JSON summary; Markdown with --md).
    Report {
        result: PathBuf,
        /// Also write Markdown here.
        #[arg(long)]
        md: Option<PathBuf>,
    },
    /// Render an EvaluationResult JSON as a Markdown scorecard (validates the result first).
    Scorecard {
        result: PathBuf,
        /// Write the Markdown here instead of printing it in the JSON output.
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Freeze an evaluation protocol JSON (write-once per id and version) and print its frozen_hash.
    FreezeProtocol {
        protocol: PathBuf,
        #[arg(long, default_value = "configs/evaluation/protocols")]
        dir: PathBuf,
    },
}

impl EvalCmd {
    /// The generated data source this command asks for (`--fixture`), as (name, runs, seed).
    pub fn fixture(&self) -> Option<(&str, usize, u64)> {
        match self {
            EvalCmd::DepositCohort {
                fixture: Some(f),
                fixture_runs,
                seed,
                ..
            } => Some((f.as_str(), *fixture_runs, *seed)),
            _ => None,
        }
    }
}

fn horizons(h: &Option<String>) -> anyhow::Result<Option<Vec<f64>>> {
    match h {
        None => Ok(None),
        Some(s) => Ok(Some(
            s.split(',')
                .filter(|x| !x.trim().is_empty())
                .map(|x| x.trim().parse::<f64>())
                .collect::<Result<_, _>>()?,
        )),
    }
}

fn read_json(p: &PathBuf) -> anyhow::Result<Value> {
    Ok(serde_json::from_str(&std::fs::read_to_string(p)?)?)
}

/// Score sealed artifacts by id (reveals are sequential and logged; scoring is parallel).
pub fn score(
    ids: &[String],
    final_test: bool,
    horizons: Option<&[f64]>,
    n_boot: usize,
    seed: u64,
    with_cases: bool,
) -> anyhow::Result<Value> {
    if ids.is_empty() {
        return Err(refused(
            "nothing to score: give artifact ids or --release with sealed artifacts",
        ));
    }
    let access = grant();
    let arts: Vec<AnalysisArtifact> = ids
        .iter()
        .map(|i| vault::sealed_artifact(i))
        .collect::<anyhow::Result<_>>()?;
    let spec = scoring::common_spec(&arts.iter().collect::<Vec<_>>(), horizons);
    let mut revealed = vec![];
    for a in &arts {
        revealed.push(vault::reveal(&access, &a.id, final_test, "score")?);
    }
    let mut rows = scoring::score_all(&revealed, &spec);
    let agg = scoring::aggregate(&mut rows, &spec, n_boot, seed);
    let cases = if with_cases {
        Value::Array(rows.iter().map(scoring::public_case).collect())
    } else {
        Value::Null
    };
    Ok(json!({"aggregate": agg, "cases": cases}))
}

/// The cutoff of a stored analysis, when the artifact came from `artifacts/analysis/`.
fn stored_cutoff(artifact_id: &str) -> Option<f64> {
    ignis_core::analysis::AnalysisStore::new(artifacts_root())
        .load(artifact_id)
        .ok()
        .map(|s| s.context.cutoff_s)
}

/// `ignis eval ...`. Prints JSON; returns the process exit code (2 on refusal/error). Experts are
/// accepted for a uniform signature; apart from `deposit-cohort`, which reads run records from
/// `views`, evaluation reads only sealed artifacts.
pub fn run(
    cmd: EvalCmd,
    _experts: &[Box<dyn ignis_core::expert::Expert>],
    views: &dyn ignis_core::views::ViewSource,
) -> i32 {
    match cmd {
        EvalCmd::Deposit {
            split,
            payload,
            run,
        } => emit(
            (|| {
                let p =
                    vault::deposit_continuation(&load_split(&split)?, &run, read_json(&payload)?)?;
                Ok(json!({"deposited": run, "path": p.display().to_string()}))
            })(),
            None,
        ),
        EvalCmd::DepositCohort {
            cohort,
            split,
            cutoffs,
            partition,
            ..
        } => emit(
            (|| {
                let c = cutoffs.as_deref().map(parse_cutoffs).transpose()?;
                deposit::deposit_cohort(views, &cohort, &split, c, &partition)
            })(),
            None,
        ),
        EvalCmd::Seal {
            artifact,
            run,
            split,
        } => emit(
            (|| {
                let a: AnalysisArtifact =
                    serde_json::from_str(&std::fs::read_to_string(&artifact)?)?;
                vault::seal_at(
                    &a,
                    &run,
                    &load_split(&split)?,
                    &artifacts_root(),
                    stored_cutoff(&a.id),
                )
            })(),
            None,
        ),
        EvalCmd::Seals { release, partition } => {
            let rows: Vec<Value> = vault::list_seals()
                .into_iter()
                .filter(|s| {
                    release.as_ref().map_or(true, |r| {
                        s["model_release_id"]
                            .as_str()
                            .is_some_and(|m| m.ends_with(hex_part(r)))
                    })
                })
                .filter(|s| {
                    partition.as_ref().map_or(true, |p| {
                        s["sealed"]["split_partition"].as_str() == Some(p.as_str())
                    })
                })
                .collect();
            emit(Ok(json!({"n": rows.len(), "seals": rows})), None)
        }
        EvalCmd::Reveal {
            artifact_id,
            final_test,
            out,
        } => emit(
            vault::reveal(&grant(), &artifact_id, final_test, "reveal").map(
                |r| json!({"seal": r.seal, "group_id": r.group_id, "continuation": r.continuation}),
            ),
            out.as_deref(),
        ),
        EvalCmd::Score {
            artifact_ids,
            release,
            partition,
            final_test,
            horizons: h,
            n_boot,
            seed,
            out,
            no_cases,
        } => emit(
            (|| {
                let mut ids = artifact_ids.clone();
                if let Some(r) = &release {
                    ids.extend(
                        vault::list_seals()
                            .iter()
                            .filter(|s| {
                                s["model_release_id"].as_str().map(hex_part) == Some(hex_part(r))
                            })
                            .filter(|s| {
                                s["sealed"]["split_partition"].as_str() == Some(partition.as_str())
                            })
                            .filter_map(|s| s["sealed"]["artifact_id"].as_str().map(String::from)),
                    );
                }
                score(
                    &ids,
                    final_test,
                    horizons(&h)?.as_deref(),
                    n_boot,
                    seed,
                    !no_cases,
                )
            })(),
            out.as_deref(),
        ),
        EvalCmd::Compare {
            release_ids,
            partition,
            final_test,
            allow_feature_mismatch,
            allow_unit_mismatch,
            allow_cohort_mismatch,
            intersect,
            horizons: h,
            n_boot,
            seed,
            out,
        } => emit(
            (|| {
                let o = compare::CompareOpts {
                    partition: &partition,
                    final_test,
                    allow: compare::Allow {
                        feature_mismatch: allow_feature_mismatch,
                        unit_mismatch: allow_unit_mismatch,
                        cohort_mismatch: allow_cohort_mismatch,
                    },
                    intersect,
                    horizons: horizons(&h)?,
                    n_boot,
                    seed,
                };
                compare::compare(&grant(), &release_ids, &o)
            })(),
            out.as_deref(),
        ),
        EvalCmd::Scorecard { result, out } => emit(
            (|| {
                let r: ignis_core::contracts::EvaluationResult =
                    serde_json::from_str(&std::fs::read_to_string(&result)?)?;
                r.validate()?;
                let md = scorecard::render(&r);
                match &out {
                    Some(p) => {
                        std::fs::write(p, &md)?;
                        Ok(json!({"scorecard": p.display().to_string(), "result_id": r.result_id}))
                    }
                    None => Ok(json!({"result_id": r.result_id, "markdown": md})),
                }
            })(),
            None,
        ),
        EvalCmd::FreezeProtocol {
            protocol: path,
            dir,
        } => emit(
            (|| {
                let p: protocol::Protocol = serde_json::from_str(&std::fs::read_to_string(&path)?)?;
                let f = protocol::freeze(&dir, &p)?;
                Ok(
                    json!({"protocol_id": f.protocol.protocol_id, "version": f.protocol.version, "frozen_hash": f.frozen_hash}),
                )
            })(),
            None,
        ),
        EvalCmd::Report { result, md } => emit(
            (|| {
                let data = read_json(&result)?;
                let text = report::render(&data);
                if let Some(m) = &md {
                    std::fs::write(m, &text)?;
                }
                Ok(report::summary(
                    &data,
                    md.as_ref()
                        .map(|m| m.to_string_lossy().to_string())
                        .as_deref(),
                ))
            })(),
            None,
        ),
    }
}

/// Library entry for in-process callers (tests, other tools): compare with the evaluator's access.
pub fn compare_releases(
    release_ids: &[String],
    o: &compare::CompareOpts<'_>,
) -> anyhow::Result<Value> {
    compare::compare(&grant(), release_ids, o)
}

/// Library entry: reveal one sealed artifact as the evaluator (logged; refused inside a lab session).
pub fn reveal(artifact_id: &str, final_test: bool) -> anyhow::Result<vault::Revealed> {
    vault::reveal(&grant(), artifact_id, final_test, "reveal")
}
