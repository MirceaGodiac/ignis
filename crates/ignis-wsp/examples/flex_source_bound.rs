//! Explicit local host consumer. No network, dataset discovery, fitting or scoring.
//! TRAIN access is verified against a supplied frozen split before a source packet is read.

use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{ensure, Context};
use clap::{Parser, Subcommand};
use ignis_core::contracts::{
    Capability, ExecutionScope, ForecastRequest, ModelRelease, Partition, PrefixBundle,
    PrincipalId, ScopeId, ScopeRole, Split, WorkspaceId,
};
use ignis_core::expert::Refusal;
use ignis_core::registry;
use ignis_wsp::property_droplet_release::{self as adapter, InputSource, SourceBoundInputs};
use ignis_wsp::service::Provenance;
use serde::Deserialize;
use serde_json::Value;

#[derive(Parser)]
struct Args {
    #[arg(long)]
    execute: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Assemble {
        #[arg(long)]
        binding: PathBuf,
        #[arg(long)]
        registry_root: PathBuf,
        #[arg(long)]
        receipt: Option<PathBuf>,
    },
    Forecast {
        #[arg(long)]
        registry_root: PathBuf,
        #[arg(long)]
        release_id: String,
        /// Host run identity checked against the split before the prefix artifact is read.
        #[arg(long)]
        run_id: String,
        #[arg(long)]
        prefix: PathBuf,
        #[arg(long)]
        request: PathBuf,
        /// Frozen split whose id matches the release. Only its TRAIN members are admitted.
        #[arg(long)]
        split_file: PathBuf,
        /// Host artifact in adapter::SourceBoundInputs::snapshot format; absence is a refusal.
        #[arg(long)]
        packet: Option<PathBuf>,
        #[arg(long)]
        output: Option<PathBuf>,
    },
}

fn read<T: serde::de::DeserializeOwned>(path: &Path) -> anyhow::Result<T> {
    serde_json::from_slice(&fs::read(path).with_context(|| format!("read {}", path.display()))?)
        .with_context(|| format!("parse {}", path.display()))
}

fn emit(value: &Value, path: Option<&Path>) -> anyhow::Result<()> {
    let bytes = serde_json::to_vec_pretty(value)?;
    if let Some(path) = path {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .with_context(|| {
                format!(
                    "create new output {}; existing files are preserved",
                    path.display()
                )
            })?;
        file.write_all(&bytes)?;
        file.write_all(b"\n")?;
    } else {
        println!("{}", String::from_utf8(bytes)?);
    }
    Ok(())
}

struct HostPacket {
    path: Option<PathBuf>,
}
impl InputSource for HostPacket {
    fn provenance(
        &self,
        _: &ExecutionScope,
        _: &ModelRelease,
        _: &PrefixBundle,
    ) -> Result<Provenance, Refusal> {
        Ok(Provenance::Partition(Partition::Train))
    }
    fn resolve(
        &self,
        _: &ExecutionScope,
        _: &ModelRelease,
        prefix: &PrefixBundle,
    ) -> Result<SourceBoundInputs, Refusal> {
        let path=self.path.as_ref().ok_or_else(||Refusal::insufficient("missing_material_packet","No admitted physical packet supplied. Initial liquid inventory/temperature and complete material, gas-transport, boundary and closure evidence are required; no defaults were substituted."))?;
        let value: Value =
            read(path).map_err(|e| Refusal::invalid("material_packet", e.to_string()))?;
        SourceBoundInputs::from_snapshot(prefix, value)
    }
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    ensure!(
        args.execute,
        "no computation or writes without explicit --execute"
    );
    match args.command {
        Command::Assemble {
            binding,
            registry_root,
            receipt,
        } => {
            #[derive(Deserialize)]
            #[serde(deny_unknown_fields)]
            struct Binding {
                cohort_id: String,
                split_id: String,
                source_manifest_hash: String,
            }
            let b: Binding = read(&binding)?;
            let loaded =
                adapter::assemble_checked(b.cohort_id, b.split_id, b.source_manifest_hash)?;
            let saved = ignis_wsp::assemble::save(&registry_root, loaded.release, &loaded.params)?;
            emit(
                &serde_json::json!({"release_id":saved.id,"registry_root":registry_root,"model_name":saved.model_name,"cohort_id":saved.cohort_id,"split_id":saved.split_id,"source_manifest_hash":saved.source_manifest_hash,"state":saved.state,"trained_on":saved.trained_on,"forward_hash":adapter::forward_hash(),"construction":saved.construction}),
                receipt.as_deref(),
            )?;
        }
        Command::Forecast {
            registry_root,
            release_id,
            run_id,
            prefix,
            request,
            split_file,
            packet,
            output,
        } => {
            let release = registry::load(&registry_root, &release_id)?;
            let split: Split = read(&split_file)?;
            ensure!(
                split.id == release.release.split_id
                    && split.cohort_id == release.release.cohort_id,
                "frozen split does not match release population"
            );
            ensure!(
                split.partition_of(&run_id) == Some(Partition::Train),
                "run is not in the pinned TRAIN partition"
            );
            let prefix: PrefixBundle = read(&prefix)?;
            ensure!(prefix.run_id == run_id, "prefix belongs to another run");
            let request: ForecastRequest = read(&request)?;
            // Host-derived scope; no caller-controlled role, partition grant or workspace in request.
            let scope = ExecutionScope {
                scope_id: ScopeId::new("local-flex-train-consumer")?,
                workspace_id: WorkspaceId::new("local-ignis")?,
                principal: PrincipalId::new("local-operator")?,
                role: ScopeRole::Research,
                partitions: [Partition::Train].into_iter().collect(),
                capabilities: [
                    Capability::RunModels,
                    Capability::ReadData,
                    Capability::ReadSources,
                ]
                .into_iter()
                .collect(),
                corpus_snapshot: None,
                parent_scope_id: None,
            };
            let source = HostPacket { path: packet };
            let artifact = adapter::forecast(&scope, &request, &release, &prefix, &source)?;
            emit(&serde_json::to_value(artifact)?, output.as_deref())?;
        }
    }
    Ok(())
}
