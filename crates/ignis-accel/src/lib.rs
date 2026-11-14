//! Optional inference accelerators (V3.1 §10.3–§10.6).
//!
//! Encoder + rational-quadratic spline flow proposals ([`nn`], candle autograd), defensive-mixture
//! importance reweighting ([`proposal`]) and model-generated proposal tasks ([`tasks`]). Every expert
//! works without this crate: ACME serves through 2-D Gauss–Hermite quadrature, BASS through
//! Laplace/defensive importance sampling. A proposal can only change efficiency, never the target.
//!
//! `ignis accel tasks|train|benchmark` ([`AccelCmd`]): `tasks` and `train` are guarded — without
//! `--execute` they print the plan and the estimated cost and stop.

pub mod benchmark;
pub mod nn;
pub mod proposal;
pub mod tasks;
pub mod train;

use std::path::PathBuf;

use anyhow::Context;
use clap::Subcommand;
use ignis_core::registry::{default_root, load};
use ignis_core::resources::{guard, load_profile, Guard, Job};
use serde_json::{json, Value};

#[derive(Subcommand, Debug)]
pub enum AccelCmd {
    /// Generate statistical proposal tasks from a fitted release (labelled model-generated).
    Tasks {
        #[arg(long)]
        release: String,
        #[arg(long, default_value_t = 100_000)]
        n: usize,
        #[arg(long, default_value = "runs/accel/tasks.npz")]
        out: PathBuf,
        /// configs/accel/<family>.toml
        #[arg(long)]
        config: Option<PathBuf>,
        #[arg(long, default_value = "cpu-local")]
        profile: String,
        #[arg(long, default_value_t = 0)]
        seed: u64,
        #[arg(long)]
        root: Option<PathBuf>,
        #[arg(long)]
        execute: bool,
    },
    /// Train encoder + spline flow on generated tasks (dry-run unless --execute).
    Train {
        #[arg(long)]
        tasks: PathBuf,
        #[arg(long)]
        config: Option<PathBuf>,
        #[arg(long)]
        out_dir: Option<PathBuf>,
        #[arg(long, default_value = "cpu-local")]
        profile: String,
        #[arg(long)]
        execute: bool,
    },
    /// ESS / error / latency of the proposal against the Gauss–Hermite reference (ACME).
    Benchmark {
        #[arg(long)]
        release: String,
        #[arg(long)]
        tasks: PathBuf,
        /// accelerator release id (omit: prior IS vs reference)
        #[arg(long)]
        accel: Option<String>,
        #[arg(long, default_value_t = 20)]
        n: usize,
        #[arg(long, default_value_t = 256)]
        particles: usize,
        #[arg(long, default_value_t = 0.1)]
        epsilon: f64,
        #[arg(long)]
        root: Option<PathBuf>,
        #[arg(long, default_value_t = 0)]
        seed: u64,
    },
}

fn read_cfg(p: &Option<PathBuf>) -> anyhow::Result<Value> {
    Ok(match p {
        Some(p) => {
            ignis_acme::common::load_toml(p)?.with_context(|| format!("config {}", p.display()))?
        }
        None => json!({}),
    })
}

pub fn run(cmd: AccelCmd) -> anyhow::Result<()> {
    match cmd {
        AccelCmd::Tasks {
            release,
            n,
            out,
            config,
            profile,
            seed,
            root,
            execute,
        } => {
            let prof = load_profile(&profile)?;
            let cfg = read_cfg(&config)?;
            let hours = n as f64
                * cfg["tasks"]["seconds_per_task_estimate"]
                    .as_f64()
                    .unwrap_or(2e-3)
                / 3600.0;
            if guard(
                "accel.tasks",
                &prof,
                hours,
                execute,
                json!({"release": release, "n_tasks": n, "out": out.display().to_string()}),
            )? == Guard::DryRun
            {
                return Ok(());
            }
            let job = Job::start("accel.tasks", &prof, json!({"release": release, "n": n}))?;
            let r = load(&root.unwrap_or_else(default_root), &release)?;
            let m = prof
                .pool()
                .install(|| tasks::generate(&r, n, &out, seed, &cfg["tasks"]["options"]))?;
            job.finish(m.clone());
            println!("{}", serde_json::to_string_pretty(&m)?);
        }
        AccelCmd::Train {
            tasks: path,
            config,
            out_dir,
            profile,
            execute,
        } => {
            let prof = load_profile(&profile)?;
            let cfg = read_cfg(&config)?["train"].clone();
            let hours = cfg["steps"].as_f64().unwrap_or(10000.0)
                * cfg["seconds_per_step_estimate"].as_f64().unwrap_or(0.05)
                / 3600.0;
            let mut detail = json!({"tasks": path.display().to_string()});
            if let (Some(d), Some(c)) = (detail.as_object_mut(), cfg.as_object()) {
                d.extend(c.clone());
            }
            if guard("accel.train", &prof, hours, execute, detail)? == Guard::DryRun {
                return Ok(());
            }
            let mut job = Job::start("accel.train", &prof, cfg.clone())?;
            let (arr, manifest) = tasks::load_tasks(&path)?;
            let dim = manifest["latent_dim"].as_u64().context("latent_dim")? as usize;
            let gpu = prof.device == "gpu";
            let res = prof.pool().install(|| train::train(&arr, dim, &cfg, gpu))?;
            let root = out_dir.unwrap_or_else(default_root);
            let rel = train::save(&root, &res, &manifest, &cfg)?;
            job.checkpoint(
                &root
                    .join("models")
                    .join(ignis_core::hashing::hex_part(&rel.id)),
            );
            let out = json!({"release": rel.id, "evaluation": rel.evaluation});
            job.finish(out.clone());
            println!("{}", serde_json::to_string_pretty(&out)?);
        }
        AccelCmd::Benchmark {
            release,
            tasks: path,
            accel,
            n,
            particles,
            epsilon,
            root,
            seed,
        } => {
            let root = root.unwrap_or_else(default_root);
            let r = load(&root, &release)?;
            let (arr, _) = tasks::load_tasks(&path)?;
            let a = accel.map(|id| load(&root, &id)).transpose()?;
            let out = benchmark::acme_benchmark(&r, &arr, a.as_ref(), n, particles, epsilon, seed)?;
            println!("{}", serde_json::to_string_pretty(&out)?);
        }
    }
    Ok(())
}
