//! `ignis metrics` (task M5): the same measurements `/api/v1/episodes/{id}/metrics` and
//! `/api/v1/metrics` serve, read from a state directory on this host. JSON on stdout.
//!
//! The scope is derived like the server's `local` auth derives it (from a principal key), never
//! read from the data. Read-only: nothing is written to the state.

use std::path::{Path, PathBuf};

use ignis_core::contracts::{EpisodeId, ExecutionScope};
use ignis_research::metrics::{self, PriceList};

#[derive(clap::Args, Debug, Clone)]
pub struct StateScope {
    /// The server's state directory (`state.sqlite` inside).
    #[arg(long, env = "IGNIS_STATE_DIR", default_value = "runs/server")]
    pub state_dir: PathBuf,
    /// Principal whose workspace is read (`local` is the workspace of `--auth local`).
    #[arg(long, default_value = "local")]
    pub principal: String,
    /// Versioned price list; unset = the one built into the binary.
    #[arg(long, env = "IGNIS_PRICE_LIST")]
    pub price_list: Option<PathBuf>,
}

impl StateScope {
    pub fn scope(&self) -> ExecutionScope {
        crate::v1::io::research_scope(&self.principal, false)
    }

    pub fn prices(&self) -> anyhow::Result<PriceList> {
        match &self.price_list {
            Some(p) => PriceList::load(p).map_err(|e| anyhow::anyhow!(e)),
            None => Ok(PriceList::builtin()),
        }
    }

    pub fn store(&self) -> anyhow::Result<ignis_research::ResearchStore> {
        open_store(&self.state_dir)
    }
}

pub fn open_store(dir: &Path) -> anyhow::Result<ignis_research::ResearchStore> {
    let db = dir.join("state.sqlite");
    anyhow::ensure!(db.exists(), "no state store at {}", db.display());
    ignis_research::ResearchStore::open(&db).map_err(|e| anyhow::anyhow!("{}: {e}", db.display()))
}

#[derive(clap::Subcommand, Debug)]
pub enum MetricsCmd {
    /// One episode: tool latency p50/p95, tokens and cache hit per turn and question, cost.
    Episode {
        episode_id: String,
        #[command(flatten)]
        at: StateScope,
    },
    /// Every episode of the principal's workspace, aggregated.
    Workspace {
        #[command(flatten)]
        at: StateScope,
    },
}

pub fn run_metrics(cmd: MetricsCmd) -> anyhow::Result<()> {
    let out = match cmd {
        MetricsCmd::Episode { episode_id, at } => {
            let id = EpisodeId::new(episode_id)?;
            let m = metrics::measure_episode(&at.store()?, &at.scope(), &id, &at.prices()?)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            serde_json::to_string_pretty(&m.metrics)?
        }
        MetricsCmd::Workspace { at } => {
            let m = metrics::measure_workspace(&at.store()?, &at.scope(), &at.prices()?)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            serde_json::to_string_pretty(&m)?
        }
    };
    println!("{out}");
    Ok(())
}
