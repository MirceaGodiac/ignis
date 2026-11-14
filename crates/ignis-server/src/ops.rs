//! Deployment operations shared by `ignis serve` and `ignis state`: the artifact-store backend
//! (local directory or Cloudflare R2) and state snapshots (backup / restore / verify).

use std::path::PathBuf;
use std::sync::Arc;

use ignis_core::services::ArtifactStore;
use ignis_research::s3::{S3Client, S3Config};

/// Where `/api/v1` keeps artifact bytes and manifests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, clap::ValueEnum)]
pub enum ArtifactBackend {
    /// `<state>/artifacts` on the local disk (development).
    #[default]
    Local,
    /// A Cloudflare R2 bucket (S3 API); the local directory only stages uploads and caches
    /// manifests.
    R2,
}

/// R2 bucket and credentials. The secret is read from the environment (the deployment's
/// entrypoint exports it from a secret file); never pass it on a command line.
#[derive(clap::Args, Debug, Clone, Default)]
pub struct R2Args {
    /// `https://<account-id>.r2.cloudflarestorage.com`.
    #[arg(long = "r2-endpoint", env = "IGNIS_R2_ENDPOINT")]
    pub endpoint: Option<String>,
    #[arg(long = "r2-bucket", env = "IGNIS_R2_BUCKET")]
    pub bucket: Option<String>,
    #[arg(long = "r2-access-key-id", env = "IGNIS_R2_ACCESS_KEY_ID")]
    pub access_key_id: Option<String>,
    #[arg(skip)]
    pub secret_access_key: Option<String>,
}

pub const R2_SECRET_ENV: &str = "IGNIS_R2_SECRET_ACCESS_KEY";

impl R2Args {
    pub fn config(&self) -> Result<S3Config, String> {
        let need = |v: &Option<String>, what: &str| {
            v.as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .ok_or_else(|| format!("the R2 artifact store needs {what}"))
        };
        let secret = match &self.secret_access_key {
            Some(s) => Some(s.clone()),
            None => std::env::var(R2_SECRET_ENV).ok(),
        };
        Ok(S3Config {
            endpoint: need(&self.endpoint, "--r2-endpoint / IGNIS_R2_ENDPOINT")?,
            bucket: need(&self.bucket, "--r2-bucket / IGNIS_R2_BUCKET")?,
            region: "auto".into(),
            access_key_id: need(
                &self.access_key_id,
                "--r2-access-key-id / IGNIS_R2_ACCESS_KEY_ID",
            )?,
            secret_access_key: need(&secret, R2_SECRET_ENV)?,
        })
    }

    pub fn client(&self) -> Result<S3Client, String> {
        S3Client::new(self.config()?)
    }
}

/// The artifact store of one `/api/v1` instance (`name` separates instances sharing a bucket).
pub fn artifact_store(
    backend: ArtifactBackend,
    r2: &R2Args,
    local_dir: PathBuf,
    name: &str,
) -> Result<Arc<dyn ArtifactStore>, String> {
    match backend {
        ArtifactBackend::Local => ignis_research::LocalArtifactStore::open(&local_dir)
            .map(|s| Arc::new(s) as Arc<dyn ArtifactStore>)
            .map_err(|e| e.to_string()),
        ArtifactBackend::R2 => {
            let s = ignis_research::r2::R2ArtifactStore::open(r2.config()?, name, &local_dir)
                .map_err(|e| e.to_string())?;
            Ok(Arc::new(s))
        }
    }
}

/// Startup check of the R2 backend: the bucket answers with these credentials.
pub fn probe_r2(r2: &R2Args) -> Result<(), String> {
    let dir = std::env::temp_dir().join(format!("ignis-r2-probe-{}", std::process::id()));
    let s = ignis_research::r2::R2ArtifactStore::open(r2.config()?, "probe", &dir)
        .map_err(|e| e.to_string())?;
    let r = s.probe();
    let _ = std::fs::remove_dir_all(&dir);
    r.map_err(|e| format!("R2 bucket not reachable: {e}"))
}

/// `ignis state …`: consistent snapshots of a server state directory.
#[derive(clap::Subcommand, Debug)]
pub enum StateCmd {
    /// Snapshot the state directory into a new directory (optionally push it to R2).
    Backup {
        #[arg(long, env = "IGNIS_STATE_DIR", default_value = "runs/server")]
        state_dir: PathBuf,
        /// New directory to write the snapshot into.
        #[arg(long)]
        out: PathBuf,
        /// Also upload it to the R2 bucket under `<prefix><name>/`.
        #[arg(long)]
        push: bool,
        #[arg(long, requires = "push")]
        name: Option<String>,
        #[arg(long, env = "IGNIS_BACKUP_PREFIX", default_value = "backups/")]
        prefix: String,
        #[command(flatten)]
        r2: R2Args,
    },
    /// Check a local snapshot against its manifest.
    Verify { snapshot: PathBuf },
    /// Restore a snapshot onto an empty state directory.
    Restore {
        /// A local snapshot directory …
        #[arg(conflicts_with = "pull")]
        snapshot: Option<PathBuf>,
        /// … or the name of one in the bucket (downloaded to --download-to first).
        #[arg(long, requires = "download_to")]
        pull: Option<String>,
        #[arg(long)]
        download_to: Option<PathBuf>,
        #[arg(long, env = "IGNIS_BACKUP_PREFIX", default_value = "backups/")]
        prefix: String,
        /// Empty (or missing) state directory to restore into.
        #[arg(long)]
        target: PathBuf,
        #[command(flatten)]
        r2: R2Args,
    },
}

pub fn run_state(cmd: StateCmd) -> anyhow::Result<()> {
    use ignis_research::backup;
    let err = |e: String| anyhow::anyhow!(e);
    match cmd {
        StateCmd::Backup {
            state_dir,
            out,
            push,
            name,
            prefix,
            r2,
        } => {
            let m = backup::snapshot(&state_dir, &out).map_err(err)?;
            println!("snapshot {} ({} files)", out.display(), m.files.len());
            if push {
                let name = name.unwrap_or_else(|| {
                    out.file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_else(|| "snapshot".into())
                });
                backup::push(&r2.client().map_err(err)?, &out, &prefix, &name).map_err(err)?;
                println!("pushed {prefix}{name}/");
            }
        }
        StateCmd::Verify { snapshot } => {
            let m = backup::verify(&snapshot).map_err(err)?;
            println!("ok: {} files, taken {}", m.files.len(), m.created_at);
        }
        StateCmd::Restore {
            snapshot,
            pull,
            download_to,
            prefix,
            target,
            r2,
        } => {
            let snap = match (snapshot, pull, download_to) {
                (Some(s), None, _) => s,
                (None, Some(name), Some(dir)) => {
                    backup::pull(&r2.client().map_err(err)?, &prefix, &name, &dir).map_err(err)?;
                    dir
                }
                _ => {
                    anyhow::bail!("give a snapshot directory or --pull <name> --download-to <dir>")
                }
            };
            let m = backup::restore(&snap, &target).map_err(err)?;
            println!(
                "restored {} files (snapshot of {}) into {}",
                m.files.len(),
                m.created_at,
                target.display()
            );
        }
    }
    Ok(())
}
