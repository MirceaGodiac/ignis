//! `ignis infer`: forecasts from the command line, through the same path as `POST /forecast`
//! (`ignis_core::analysis`), one run at a time or over a split partition.

use std::path::PathBuf;

use clap::Subcommand;
use ignis_core::analysis::{
    at_risk, execute_forecast, landmarks, plan_forecast, AnalysisStore, ForecastSettings,
    StoredAnalysis,
};
use ignis_core::contracts::{ModelRelease, Partition, Split};
use ignis_core::expert::{Expert, Refusal};
use ignis_core::registry::{default_root, load, LoadedRelease};
use ignis_core::resources::{guard, load_profile, Guard, Job};
use ignis_core::views::{RunArchive, ViewSource};
use rayon::prelude::*;
use serde_json::{json, Map, Value};

use crate::eval::vault;
use crate::util::{artifacts_root, dumps, emit, parse_cutoffs, refused, write_json_file};

#[derive(Debug, Subcommand)]
pub enum InferCmd {
    /// Forecast one run from its causal prefix and store the content-addressed AnalysisArtifact.
    Forecast {
        #[arg(long)]
        release: String,
        #[arg(long)]
        run: String,
        #[arg(long)]
        cutoff: f64,
        /// Settings JSON (strict): seed, quantile_levels, horizon_s, n_points, times_s.
        #[arg(long)]
        settings: Option<String>,
        /// Also write the artifact to this file.
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Forecast every admitted case of a split partition (dry-run unless --execute).
    ForecastCohort {
        #[arg(long)]
        release: String,
        /// Split id, 16-hex prefix or file path (must be the release's split).
        #[arg(long)]
        split: String,
        /// validation | test (test needs --final-test).
        #[arg(long, default_value = "validation")]
        partition: String,
        /// Cutoffs in s, comma-separated (default: configs/evaluation/landmarks.toml).
        #[arg(long)]
        cutoffs: Option<String>,
        #[arg(long)]
        settings: Option<String>,
        #[arg(long, default_value = "cpu-local")]
        profile: String,
        /// Actually run.
        #[arg(long)]
        execute: bool,
        /// Seal every produced artifact (hash commitment before any reveal).
        #[arg(long)]
        seal: bool,
        #[arg(long)]
        final_test: bool,
    },
}

/// Which data a release's forecasts read: the data store, or the fixture it was fitted on.
pub type SourceFor<'a> = dyn Fn(&ModelRelease) -> anyhow::Result<Box<dyn ViewSource>> + 'a;

fn from_fixture(rel: &ModelRelease) -> bool {
    rel.config.contains_key("fixture")
}

fn load_release(id: &str) -> Result<LoadedRelease, Refusal> {
    load(&default_root(), id).map_err(|e| {
        Refusal::failed(
            "release_verification",
            format!("release {id} failed verification: {e:#}"),
        )
    })
}

fn now_s() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// One forecast through the shared path: plan (prefix), run, store.
pub fn forecast_run(
    experts: &[Box<dyn Expert>],
    loaded: &LoadedRelease,
    archive: &RunArchive,
    cutoff_s: f64,
    settings: &ForecastSettings,
    store: &AnalysisStore,
    fixture_source: bool,
) -> Result<StoredAnalysis, Refusal> {
    let rel = &loaded.release;
    let label = if fixture_source {
        "development fixture — synthetic"
    } else {
        "registry"
    };
    let (ei, prefix, value) = plan_forecast(
        experts,
        rel,
        archive,
        cutoff_s,
        settings,
        fixture_source,
        label,
    )?;
    execute_forecast(
        experts[ei].as_ref(),
        loaded,
        prefix,
        value,
        label,
        store,
        now_s(),
    )
}

fn print_refusal(r: &Refusal) -> i32 {
    println!("{}", dumps(&json!({"refused": true, "refusal": r})));
    2
}

fn one(
    experts: &[Box<dyn Expert>],
    source_for: &SourceFor<'_>,
    release: &str,
    run: &str,
    cutoff: f64,
    settings: Option<&str>,
    out: Option<&PathBuf>,
) -> i32 {
    let settings = match ForecastSettings::parse(settings) {
        Ok(s) => s,
        Err(r) => return print_refusal(&r),
    };
    let loaded = match load_release(release) {
        Ok(l) => l,
        Err(r) => return print_refusal(&r),
    };
    let source = match source_for(&loaded.release) {
        Ok(s) => s,
        Err(e) => return emit(Err(e), None),
    };
    let archive = match source.run_archive(loaded.release.family, run) {
        Ok(Some(a)) => a,
        Ok(None) => {
            return print_refusal(&Refusal::not_found(
                "run_not_found",
                format!(
                    "run {run} is not in the {} records",
                    loaded.release.family.as_str()
                ),
            ))
        }
        Err(e) => return emit(Err(e), None),
    };
    let store = AnalysisStore::new(default_root());
    match forecast_run(
        experts,
        &loaded,
        &archive,
        cutoff,
        &settings,
        &store,
        from_fixture(&loaded.release),
    ) {
        Err(r) => print_refusal(&r),
        Ok(s) => {
            if let Some(o) = out {
                if let Err(e) = write_json_file(o, &serde_json::to_value(&s.artifact).unwrap()) {
                    return emit(Err(e), None);
                }
            }
            let dir = store
                .dir(&s.artifact.id)
                .map(|d| d.display().to_string())
                .unwrap_or_default();
            emit(
                Ok(
                    json!({"artifact_id": s.artifact.id, "run_id": s.context.run_id, "cutoff_s": cutoff, "status": s.artifact.status,
                           "kind": s.artifact.kind, "support": s.artifact.support, "prefix_hash": s.artifact.prefix_hash,
                           "stored_at": dir, "out": out.map(|p| p.display().to_string())}),
                ),
                None,
            )
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn cohort(
    experts: &[Box<dyn Expert>],
    source_for: &SourceFor<'_>,
    release: &str,
    split: &str,
    partition: &str,
    cutoffs: Option<&str>,
    settings: Option<&str>,
    profile: &str,
    execute: bool,
    seal: bool,
    final_test: bool,
) -> anyhow::Result<Value> {
    let settings = ForecastSettings::parse(settings).map_err(|r| refused(r.reason))?;
    let part = match partition {
        "validation" => Partition::Validation,
        "test" if final_test => Partition::Test,
        "test" => return Err(refused("the test partition is forecast once, by the team, after everything is frozen; pass --final-test")),
        other => return Err(refused(format!("partition {other:?}: validation or test"))),
    };
    let loaded = load_release(release).map_err(|r| refused(r.reason))?;
    let rel = &loaded.release;
    let source = source_for(rel)?;
    let sp: Split = source.split(split)?;
    if sp.id != rel.split_id {
        return Err(refused(format!(
            "release {} was fitted against split {}, not {}",
            rel.id, rel.split_id, sp.id
        )));
    }
    let cutoffs = match cutoffs {
        Some(c) => parse_cutoffs(c)?,
        None => landmarks(rel.family)?,
    };
    let archives: Vec<RunArchive> = source
        .run_archives(rel.family)?
        .into_iter()
        .filter(|a| sp.partition_of(&a.record.id) == Some(part))
        .collect();
    let mut cases = vec![];
    let mut not_at_risk = vec![];
    for a in &archives {
        for &c in &cutoffs {
            if at_risk(a, c) {
                cases.push((a, c))
            } else {
                not_at_risk
                    .push(json!({"run_id": a.record.id, "cutoff_s": c, "status": "not_at_risk"}))
            }
        }
    }
    let prof = load_profile(profile)?;
    let per_case = 1.0; // seconds; a declared planning constant, replaced by measured wall time in the job ledger
    let detail = json!({"release": rel.id, "model": rel.model_name, "cohort": rel.cohort_id, "split": sp.id, "partition": partition, "cutoffs_s": cutoffs,
                        "cases": cases.len(), "not_at_risk_cases": not_at_risk.len(), "runs": archives.len(), "threads": prof.threads(), "seal": seal,
                        "basis": "planning constant: 1 s per forecast case"});
    if guard(
        &format!("infer forecast-cohort {}", rel.model_name),
        &prof,
        cases.len() as f64 * per_case / 3600.0,
        execute,
        detail.clone(),
    )? == Guard::DryRun
    {
        return Ok(Value::Null);
    }
    let mut job = Job::start(
        &format!("infer forecast-cohort {}", rel.model_name),
        &prof,
        detail,
    )?;
    let store = AnalysisStore::new(default_root());
    let t0 = std::time::Instant::now();
    let results: Vec<(String, f64, Result<StoredAnalysis, Refusal>)> = prof.pool().install(|| {
        cases
            .par_iter()
            .map(|(a, c)| {
                (
                    a.record.id.clone(),
                    *c,
                    forecast_run(
                        experts,
                        &loaded,
                        a,
                        *c,
                        &settings,
                        &store,
                        from_fixture(rel),
                    ),
                )
            })
            .collect()
    });
    job.checkpoint(&artifacts_root().join("analysis"));
    let (mut n_ok, mut n_sealed) = (0usize, 0usize);
    let mut refusal_counts: Map<String, Value> = Map::new();
    let mut rows = not_at_risk;
    for (run, cutoff, res) in results {
        match res {
            Ok(s) => {
                n_ok += 1;
                let mut seal_id = Value::Null;
                if seal {
                    if vault::load_seal(&s.artifact.id)?.is_none() {
                        vault::seal_at(&s.artifact, &run, &sp, &artifacts_root(), Some(cutoff))?;
                    }
                    n_sealed += 1;
                    seal_id = json!(s.artifact.id);
                }
                rows.push(json!({"run_id": run, "cutoff_s": cutoff, "status": "artifact", "artifact_id": s.artifact.id, "sealed": seal_id}));
            }
            Err(r) => {
                let n = refusal_counts.entry(r.code.clone()).or_insert(json!(0));
                *n = json!(n.as_u64().unwrap_or(0) + 1);
                rows.push(
                    json!({"run_id": run, "cutoff_s": cutoff, "status": "refused", "refusal": r}),
                );
            }
        }
    }
    job.finish(json!({"artifacts": n_ok, "sealed": n_sealed}));
    Ok(
        json!({"release": rel.id, "model": rel.model_name, "split": sp.id, "partition": partition, "cutoffs_s": cutoffs,
              "artifacts": n_ok, "sealed": n_sealed, "refused": rows.iter().filter(|r| r["status"] == "refused").count(),
              "refusal_counts": refusal_counts, "not_at_risk": rows.iter().filter(|r| r["status"] == "not_at_risk").count(),
              "wall_s": t0.elapsed().as_secs_f64(), "cases": rows}),
    )
}

/// `ignis infer ...`. Prints JSON; returns the process exit code (2 on refusal or error).
pub fn run(cmd: InferCmd, experts: &[Box<dyn Expert>], source_for: &SourceFor<'_>) -> i32 {
    match cmd {
        InferCmd::Forecast {
            release,
            run,
            cutoff,
            settings,
            out,
        } => one(
            experts,
            source_for,
            &release,
            &run,
            cutoff,
            settings.as_deref(),
            out.as_ref(),
        ),
        InferCmd::ForecastCohort {
            release,
            split,
            partition,
            cutoffs,
            settings,
            profile,
            execute,
            seal,
            final_test,
        } => {
            match cohort(
                experts,
                source_for,
                &release,
                &split,
                &partition,
                cutoffs.as_deref(),
                settings.as_deref(),
                &profile,
                execute,
                seal,
                final_test,
            ) {
                Ok(Value::Null) => 0,
                other => emit(other, None),
            }
        }
    }
}
