//! `ignis` — one binary. Each command group lives in the crate that owns it and exposes a clap
//! `Subcommand` enum plus `run(cmd)`. Expensive commands are dry-runs unless `--execute` is given.
//!
//! Groups: schemas (here) · sources/data (ignis-data) · fit/decide (ignis-train) ·
//! accel (ignis-accel) · eval/infer/lab (ignis-lab) · evidence (ignis-evidence) · vision (ignis-vision) ·
//! serve (ignis-server).

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "ignis", version, about = "IGNIS scientific laboratory")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Shared contract schemas.
    #[command(subcommand)]
    Schemas(SchemasCmd),
    /// Runtime API (Axum) with in-process experts; `--dev` adds the synthetic /fixtures origin.
    Serve(ignis_server::ServeArgs),
    /// Server state snapshots: backup (optionally to R2), verify, restore onto an empty volume.
    #[command(subcommand)]
    State(ignis_server::ops::StateCmd),
    /// Sealed evaluation: deposit, seal, reveal, score, compare, report.
    #[command(subcommand)]
    Eval(ignis_lab::EvalCmd),
    /// Forecasts through the API's path: one run, or a whole split partition (guarded).
    #[command(subcommand)]
    Infer(ignis_lab::InferCmd),
    /// Inspect, probe and edit model releases (train/validation only).
    #[command(subcommand)]
    Lab(ignis_lab::LabCmd),
    /// BASS visual metrology: probe, frames, contours.
    #[command(subcommand)]
    Vision(ignis_vision::VisionCmd),
    /// NASA PSI + international sources: inspect, samples, guarded fetch, manifests.
    #[command(subcommand)]
    Sources(ignis_data::SourcesCmd),
    /// Curation: ingest, rules, cohorts, frozen splits, views, review.
    #[command(subcommand)]
    Data(ignis_data::DataCmd),
    /// Scientific retrieval: ingest, embed, search, firewall, models.
    #[command(subcommand)]
    Evidence(ignis_evidence::EvidenceCmd),
    /// Optional inference accelerators: proposal tasks, encoder/flow training, benchmarks (guarded).
    #[command(subcommand)]
    Accel(ignis_accel::AccelCmd),
    /// Fit experts and baselines (dry-run unless --execute).
    #[command(subcommand)]
    Fit(ignis_train::fit::FitCmd),
    /// Observation-value ranking and equal-budget replay.
    #[command(subcommand)]
    Decide(ignis_train::decide::DecideCmd),
}

#[derive(Subcommand)]
enum SchemasCmd {
    /// Write one JSON Schema per contract to `schemas/` (web client, SAM2 sidecar).
    Export {
        #[arg(long, default_value = "schemas")]
        out: std::path::PathBuf,
    },
}

/// Every scientific expert compiled into this binary (model crates are added here as they land).
fn experts() -> Vec<Box<dyn ignis_core::expert::Expert>> {
    let mut v = ignis_acme::experts();
    v.extend(ignis_bass::experts());
    v.extend(ignis_flex::experts());
    v
}

/// Generated (never NASA) archives for `--fixture`.
fn fixtures() -> Vec<ignis_train::Fixture> {
    vec![ignis_train::Fixture {
        name: "synthetic",
        family: ignis_core::contracts::Family::Flex,
        build: |n, seed| {
            let ds = ignis_flex::synthetic::make_dataset(n, seed, 3, 0.1)?;
            let (cohort_id, split_id) = (ds.cohort.id.clone(), ds.split.id.clone());
            Ok(ignis_train::FixtureData {
                source: Box::new(ds),
                cohort_id,
                split_id,
            })
        },
    }]
}

/// The data a release's forecasts read: the fixture it was fitted on, or the data store.
fn source_for(
    release: &ignis_core::contracts::ModelRelease,
) -> anyhow::Result<Box<dyn ignis_core::views::ViewSource>> {
    let Some(fx) = release.config.get("fixture") else {
        return Ok(Box::new(ignis_data::views::DataStore::new()));
    };
    let kind = fx.get("kind").and_then(|k| k.as_str()).unwrap_or_default();
    let (n, seed) = (
        fx.get("n_runs").and_then(|v| v.as_u64()).unwrap_or(24) as usize,
        fx.get("seed").and_then(|v| v.as_u64()).unwrap_or(0),
    );
    Ok((ignis_train::find_fixture(&fixtures(), kind, release.family)?.build)(n, seed)?.source)
}

/// The eval command's data: the named fixture when asked for, otherwise the data store.
fn eval_source(c: &ignis_lab::EvalCmd) -> anyhow::Result<Box<dyn ignis_core::views::ViewSource>> {
    match c.fixture() {
        Some((kind, n, seed)) => {
            let f = fixtures()
                .into_iter()
                .find(|f| f.name == kind)
                .ok_or_else(|| anyhow::anyhow!("no fixture {kind:?}"))?;
            Ok((f.build)(n, seed)?.source)
        }
        None => Ok(Box::new(ignis_data::views::DataStore::new())),
    }
}

/// `POST /acquisition` scoring: VOI ranking with the release's own expert.
fn acquisition_scorer() -> ignis_server::AcquisitionScorer {
    let experts = experts();
    std::sync::Arc::new(move |release, prefix, actions, settings| {
        ignis_train::decide::score_candidates(&experts, release, prefix, actions, settings)
    })
}

fn main() -> anyhow::Result<()> {
    match Cli::parse().cmd {
        Cmd::Schemas(SchemasCmd::Export { out }) => {
            let n = ignis_core::schema::export(&out)?;
            println!("exported {n} schemas to {}", out.display());
        }
        Cmd::Serve(args) => ignis_server::run(
            args,
            ignis_server::Services {
                experts: experts(),
                fixture_experts: ignis_server::fixture::experts(),
                acquisition: Some(acquisition_scorer()),
                // Lane services are wired here as each lane delivers its constructor (R3, R2, I2).
                research: Default::default(),
            },
        )?,
        Cmd::State(c) => ignis_server::ops::run_state(c)?,
        Cmd::Eval(c) => {
            let source = eval_source(&c)?;
            std::process::exit(ignis_lab::eval::run(c, &experts(), source.as_ref()))
        }
        Cmd::Infer(c) => std::process::exit(ignis_lab::infer::run(c, &experts(), &source_for)),
        Cmd::Lab(c) => std::process::exit(ignis_lab::lab::run(
            c,
            &experts(),
            &ignis_data::views::DataStore::new(),
        )),
        Cmd::Vision(c) => ignis_vision::run(c)?,
        Cmd::Sources(c) => c.run()?,
        Cmd::Data(c) => c.run()?,
        Cmd::Evidence(c) => ignis_evidence::run(c)?,
        Cmd::Accel(cmd) => ignis_accel::run(cmd)?,
        Cmd::Fit(cmd) => ignis_train::fit::run(
            cmd,
            &experts(),
            &ignis_data::views::DataStore::new(),
            &fixtures(),
        )?,
        Cmd::Decide(cmd) => ignis_train::decide::run(
            cmd,
            &experts(),
            &ignis_data::views::DataStore::new(),
            &fixtures(),
        )?,
    }
    Ok(())
}
