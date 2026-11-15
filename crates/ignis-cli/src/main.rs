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
    /// Measured episode metrics (tool latency, tokens, cache hit, cost) from a state directory.
    #[command(subcommand)]
    Metrics(ignis_server::observe::MetricsCmd),
    /// Benchmark runs: a dev task set under a harness variant → TraceBundles + summary (--execute).
    #[command(subcommand)]
    Harness(ignis_server::harness_run::HarnessCmd),
    /// Sealed evaluation: deposit, seal, reveal, score, compare, report.
    #[command(subcommand)]
    Eval(ignis_lab::EvalCmd),
    /// Forecasts through the API's path: one run, or a whole split partition (guarded).
    #[command(subcommand)]
    Infer(ignis_lab::InferCmd),
    /// Inspect, probe and edit model releases (train/validation only).
    #[command(subcommand)]
    Lab(ignis_lab::LabCmd),
    /// Literal weight-space programming: specs, patches, assembly and strict forwards.
    #[command(subcommand)]
    Wsp(ignis_wsp::WspCmd),
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

/// The lane services `/api/v1` and the IGNIS tools run on, each present only when its data is
/// there; an absent one is reported by `/api/v1/capabilities`, never answered with empty results.
/// Models are the served hand-written releases, only with `--enable-models`.
fn research_services(args: Option<&ignis_server::ServeArgs>) -> ignis_server::v1::ResearchServices {
    use std::sync::Arc;
    let knowledge = if ignis_evidence::catalog::default_db().exists() {
        match ignis_evidence::service::EvidenceService::open_default() {
            Ok(s) => Some(Arc::new(s) as Arc<dyn ignis_core::services::KnowledgeService>),
            Err(e) => {
                eprintln!("KnowledgeService not configured: {e:#}");
                None
            }
        }
    } else {
        eprintln!(
            "KnowledgeService not configured: no evidence catalog at {}",
            ignis_evidence::catalog::default_db().display()
        );
        None
    };
    let store = ignis_data::dataset::DatasetStore::default_roots();
    let datasets = if store.builder_root.exists() {
        Some(Arc::new(store) as Arc<dyn ignis_core::services::DatasetService>)
    } else {
        eprintln!(
            "DatasetService not configured: no bundles at {}",
            store.builder_root.display()
        );
        None
    };
    // conditions_query is offered only once its table is built (`ignis data conditions`).
    let conditions = datasets.is_some() && ignis_data::conditions::table_path().exists();
    ignis_server::v1::ResearchServices {
        knowledge,
        datasets,
        conditions,
        models: args.and_then(served_models),
        ..Default::default()
    }
}

/// The served hand-written releases (`IGNIS_WSP_SERVED`, default `configs/models/served` under
/// `--root`): the list and the release bytes come from the host, never from a request. Every
/// listed release is verified at startup; a broken list leaves models unavailable, with the reason.
fn served_models(
    args: &ignis_server::ServeArgs,
) -> Option<std::sync::Arc<dyn ignis_core::services::ModelService>> {
    if !args.enable_models {
        return None;
    }
    let dir = std::env::var_os("IGNIS_WSP_SERVED")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| ignis_wsp_serve::DEFAULT_DIR.into());
    let stored = ignis_wsp::ViewSourcePrefixes::new(ignis_data::views::DataStore::new());
    match ignis_wsp_serve::ServedModels::open(args.root.join(dir), stored) {
        Ok(m) => {
            eprintln!(
                "ModelService: {} served hand-written releases from {}",
                m.count(),
                m.dir().display()
            );
            Some(std::sync::Arc::new(m))
        }
        Err(e) => {
            eprintln!("ModelService not configured: {e:#}");
            None
        }
    }
}

fn main() -> anyhow::Result<()> {
    match Cli::parse().cmd {
        Cmd::Schemas(SchemasCmd::Export { out }) => {
            let n = ignis_core::schema::export(&out)?;
            println!("exported {n} schemas to {}", out.display());
        }
        Cmd::Serve(args) => {
            let research = research_services(Some(&args));
            ignis_server::run(
                args,
                ignis_server::Services {
                    experts: experts(),
                    fixture_experts: ignis_server::fixture::experts(),
                    acquisition: Some(acquisition_scorer()),
                    research,
                },
            )?
        }
        Cmd::Wsp(c) => ignis_wsp::run(c)?,
        Cmd::State(c) => ignis_server::ops::run_state(c)?,
        Cmd::Metrics(c) => ignis_server::observe::run_metrics(c)?,
        Cmd::Harness(c) => ignis_server::harness_run::run(
            c,
            ignis_server::Services {
                experts: experts(),
                fixture_experts: ignis_server::fixture::experts(),
                acquisition: Some(acquisition_scorer()),
                research: research_services(None),
            },
        )?,
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

#[cfg(test)]
mod tests {
    /// clap's own consistency check over the whole CLI, flattened argument groups included: two
    /// fields with the same id (e.g. `runtime` in the agent and runner flags) fail here, not at
    /// `ignis serve` startup in production.
    #[test]
    fn the_cli_definition_is_consistent() {
        use clap::CommandFactory;
        super::Cli::command().debug_assert();
    }

    /// The deployment's environment parses: the agent runtime and the runner runtime are distinct.
    #[test]
    fn serve_parses_with_both_runtimes_set() {
        use clap::Parser;
        let cli = super::Cli::try_parse_from([
            "ignis",
            "serve",
            "--agent-runtime",
            "claude",
            "--runner-runtime",
            "runsc",
        ])
        .expect("serve parses");
        assert!(matches!(cli.cmd, super::Cmd::Serve(_)));
    }
}
