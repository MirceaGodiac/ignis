//! `ignis fit` — fitting scientific experts and baselines (opt-in `--execute`).
//!
//! ```text
//! ignis fit list
//! ignis fit plan [--n-runs N --obs-per-run M]
//! ignis fit run <model> --cohort <id> --split <id> --profile cpu-local [--execute] [--resume <release>]
//!               [--fixture synthetic] [--set fit.method=map ...]
//! ```
//!
//! Experts fit on the train partition only (`FitData::partition`); validation is for selection;
//! test is never handed over. Executed fits run inside a `Job` ledger entry and the profile's
//! bounded Rayon pool; MAP checkpoints live under `runs/checkpoints/<model>/<key>/`.

use std::path::PathBuf;

use anyhow::{bail, Context};
use clap::Subcommand;
use serde_json::{json, Value};

use ignis_core::contracts::Partition;
use ignis_core::expert::{find, Expert, FitContext, FitData};
use ignis_core::hashing::{content_hash, hex_part};
use ignis_core::registry::{default_root, list_releases, load};
use ignis_core::resources::{configs_dir, guard, load_profile, runs_dir, Guard, Job};
use ignis_core::views::{View, ViewSource};

use crate::config::load_config;
use crate::{family_of, find_fixture, Fixture};

#[derive(Debug, Subcommand)]
pub enum FitCmd {
    /// Experts available in this build, their configs and saved releases.
    List,
    /// Every fit job with its estimated hours and USD under every resource profile.
    Plan {
        #[arg(long)]
        n_runs: Option<usize>,
        #[arg(long)]
        obs_per_run: Option<usize>,
    },
    /// Fit one model (dry-run plan unless --execute).
    Run {
        /// e.g. flex.v3, flex.v1, flex.d2_population, flex.d2_prefix
        model: String,
        #[arg(long)]
        cohort: Option<String>,
        #[arg(long)]
        split: Option<String>,
        #[arg(long, default_value = "cpu-local")]
        profile: String,
        /// Actually run (default: dry-run plan).
        #[arg(long)]
        execute: bool,
        /// Parent release id (MAP warm start, `fine_tune` child; NUTS refuses).
        #[arg(long)]
        resume: Option<String>,
        /// 'synthetic': generated data, never promotable.
        #[arg(long)]
        fixture: Option<String>,
        #[arg(long, default_value_t = 24)]
        fixture_runs: usize,
        #[arg(long, default_value_t = 0)]
        seed: u64,
        /// Config override, e.g. --set fit.method=map
        #[arg(long = "set")]
        set: Vec<String>,
    },
}

fn n_obs(views: &[&View]) -> usize {
    views.iter().map(|v| v.n_observations()).sum()
}

pub fn run(
    cmd: FitCmd,
    experts: &[Box<dyn Expert>],
    data: &dyn ViewSource,
    fixtures: &[Fixture],
) -> anyhow::Result<()> {
    match cmd {
        FitCmd::List => {
            let mut ex = serde_json::Map::new();
            for e in experts {
                let cfg = match e.default_config() {
                    Ok(c) => c
                        .get("_config_path")
                        .cloned()
                        .unwrap_or(json!("built-in defaults")),
                    Err(err) => json!(format!("config error: {err:#}")),
                };
                ex.insert(
                    e.model_name().into(),
                    json!({"family": e.family(), "baseline": e.is_baseline(), "config": cfg}),
                );
            }
            let rels: Vec<Value> = list_releases(&default_root())
                .iter()
                .map(|r| json!({"id": r.id, "model": r.model_name, "state": r.state, "trained_on": r.trained_on, "created_at": r.created_at, "parent_id": r.parent_id}))
                .collect();
            println!(
                "{}",
                serde_json::to_string_pretty(
                    &json!({"experts": ex, "releases": rels, "fixtures": fixtures.iter().map(|f| json!({"name": f.name, "family": f.family})).collect::<Vec<_>>()})
                )?
            );
        }
        FitCmd::Plan {
            n_runs,
            obs_per_run,
        } => {
            let mut profiles: Vec<String> = std::fs::read_dir(configs_dir().join("resources"))
                .map(|rd| {
                    rd.flatten()
                        .filter_map(|e| {
                            e.path()
                                .file_stem()
                                .map(|s| s.to_string_lossy().to_string())
                        })
                        .collect()
                })
                .unwrap_or_default();
            profiles.sort();
            let mut rows = vec![];
            for e in experts {
                let cfg = match load_config(e.as_ref(), &[]) {
                    Ok(c) => c,
                    Err(err) => {
                        rows.push(json!({"model": e.model_name(), "error": format!("{err:#}")}));
                        continue;
                    }
                };
                let nr = n_runs.unwrap_or(
                    cfg.pointer("/estimate/nominal_runs")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(100) as usize,
                );
                let no = nr
                    * obs_per_run.unwrap_or(
                        cfg.pointer("/estimate/nominal_obs_per_run")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(300) as usize,
                    );
                for p in &profiles {
                    let prof = load_profile(p)?;
                    let h = e.estimate_hours(&cfg, nr, no, &prof);
                    rows.push(json!({"model": e.model_name(), "profile": p, "threads": prof.threads(), "method": cfg.pointer("/fit/method"), "estimated_hours": (h * 1e3).round() / 1e3,
                                     "estimated_usd": (h * prof.usd_per_hour * 100.0).round() / 100.0,
                                     "within_profile_limit": prof.max_hours == 0.0 || h <= prof.max_hours}));
                }
            }
            println!(
                "{}",
                serde_json::to_string_pretty(
                    &json!({"jobs": rows, "basis": "declared planning constants (configs/training/defaults.toml); calibrate from runs/jobs"})
                )?
            );
        }
        FitCmd::Run {
            model,
            cohort,
            split,
            profile,
            execute,
            resume,
            fixture,
            fixture_runs,
            seed,
            set,
        } => {
            let expert = find(experts, &model).with_context(|| {
                format!(
                    "unknown model {model:?}; available: {:?}",
                    experts.iter().map(|e| e.model_name()).collect::<Vec<_>>()
                )
            })?;
            let family = family_of(&model).unwrap_or(expert.family());
            let prof = load_profile(&profile)?;
            let mut cfg = load_config(expert, &set)?;
            let fx_data;
            let (source, cohort_id, split_id): (&dyn ViewSource, String, String) = match fixture
                .as_deref()
            {
                Some(name) => {
                    let fx = find_fixture(fixtures, name, family)?;
                    fx_data = (fx.build)(fixture_runs, seed)?;
                    cfg["fixture"] = json!({"kind": name, "family": family, "seed": seed, "n_runs": fixture_runs});
                    (
                        fx_data.source.as_ref(),
                        fx_data.cohort_id.clone(),
                        fx_data.split_id.clone(),
                    )
                }
                None => {
                    let (Some(c), Some(s)) = (cohort, split) else {
                        bail!("--cohort and --split are required (or --fixture synthetic)")
                    };
                    (data, c, s)
                }
            };
            let coh = source.cohort(&cohort_id)?;
            let sp = source.split(&split_id)?;
            if sp.cohort_id != coh.id {
                bail!(
                    "split {} belongs to cohort {}, not {}",
                    sp.id,
                    sp.cohort_id,
                    coh.id
                );
            }
            let views = source.views(family, &coh.id)?;
            let train: Vec<&View> = views
                .iter()
                .filter(|v| sp.partition_of(v.run_id()) == Some(Partition::Train))
                .collect();
            let n_val = views
                .iter()
                .filter(|v| sp.partition_of(v.run_id()) == Some(Partition::Validation))
                .count();
            let method = cfg
                .pointer("/fit/method")
                .and_then(|v| v.as_str())
                .unwrap_or("map")
                .to_string();
            if let Some(r) = &resume {
                if method == "nuts" {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(
                            &json!({"refused": "NUTS resume is not supported: the adapted step size, mass matrix, chain RNG state and warmup phase are not checkpointed by a tested mechanism; restart the chains (optionally from the parent's MAP via --set fit.method=map, then a fresh NUTS run).", "resume": r})
                        )?
                    );
                    bail!("NUTS resume refused");
                }
            }
            let parent = match &resume {
                Some(r) => Some(load(&default_root(), r)?),
                None => None,
            };
            let nobs = n_obs(&train);
            let hours = expert.estimate_hours(&cfg, train.len(), nobs, &prof);
            let detail = json!({"model": model, "cohort": coh.id, "split": sp.id, "synthetic": fixture.is_some() || coh.source_manifest_hash == "synthetic",
                                "train_runs": train.len(), "validation_runs": n_val, "train_observations": nobs, "method": method,
                                "resume_parent": resume, "threads": prof.threads()});
            if guard(
                &format!("fit {model}"),
                &prof,
                hours,
                execute,
                detail.clone(),
            )? == Guard::DryRun
            {
                return Ok(());
            }
            let full = content_hash(
                &json!({"model": model, "cohort": coh.id, "split": sp.id, "set": set, "resume": resume}),
                &[],
            );
            let key = &hex_part(&full)[..16];
            let checkpoint_dir: PathBuf = runs_dir().join("checkpoints").join(&model).join(key);
            std::fs::create_dir_all(&checkpoint_dir)?;
            let mut job = Job::start(&format!("fit {model}"), &prof, detail)?;
            let mut ctx = FitContext {
                profile: prof.clone(),
                pool: prof.pool(),
                seed,
                registry_root: default_root(),
                checkpoint_dir: checkpoint_dir.clone(),
                resume: parent,
                job_id: Some(job.id().to_string()),
            };
            let fd = FitData {
                views: &views,
                split: &sp,
                cohort_id: &coh.id,
                source_manifest_hash: &coh.source_manifest_hash,
                synthetic: fixture.is_some() || coh.source_manifest_hash == "synthetic",
            };
            let t0 = std::time::Instant::now();
            let rel = expert.fit(&fd, &cfg, &mut ctx)?;
            job.checkpoint(&checkpoint_dir);
            let out = json!({"release": rel.id, "model": model, "state": rel.state, "trained_on": rel.trained_on, "evaluation": rel.evaluation,
                             "inference": rel.numerics.get("inference"), "wall_s": t0.elapsed().as_secs_f64(), "job": job.id()});
            job.finish(json!({"release": rel.id}));
            println!("{}", serde_json::to_string_pretty(&out)?);
        }
    }
    Ok(())
}
