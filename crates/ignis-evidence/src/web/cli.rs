//! `ignis evidence web …`: the same [`super::WebDiscovery`] the HTTP API and MCP tools use, under
//! the local research scope plus `network_fetch`. Without `--execute` nothing is sent: the command
//! prints what it would request.

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{bail, Result};
use clap::Subcommand;
use ignis_core::contracts::{
    ArtifactId, Capability, EvidenceSchema, ExecutionScope, Origin, WebFetchRequest,
    WebSearchRequest,
};
use ignis_core::services::ArtifactStore;
use serde_json::json;

use super::fetch::FetchPolicy;
use super::provider::{ParallelSearch, SearchProvider, PARALLEL_KEY_ENV};
use super::WebDiscovery;

#[derive(Debug, Subcommand)]
pub enum WebCmd {
    /// One `web_search` page through the configured provider (network: plan only unless --execute).
    Search {
        query: String,
        #[arg(long)]
        page_size: Option<u32>,
        #[arg(long)]
        execute: bool,
    },
    /// Fetch one http(s) URL into a snapshot under the SSRF policy (network: plan only unless
    /// --execute). Prints the `ExternalSourceCandidate`.
    Fetch {
        url: String,
        #[arg(long)]
        execute: bool,
    },
    /// Read a snapshot back by artifact id (hashes verified); prints its fetch records and,
    /// with --text, the stored text.
    Snapshot {
        artifact_id: String,
        #[arg(long)]
        text: bool,
    },
}

impl WebCmd {
    /// Where the snapshots of this command go (`--state-dir` of the CLI, local backend).
    pub fn needs_store(&self) -> bool {
        !matches!(
            self,
            WebCmd::Search { execute: false, .. } | WebCmd::Fetch { execute: false, .. }
        )
    }
}

/// The local research scope with `network_fetch`.
pub fn cli_scope() -> ExecutionScope {
    let mut s = crate::local_research_scope();
    s.capabilities.insert(Capability::NetworkFetch);
    s
}

fn print(v: &serde_json::Value) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(v)?);
    Ok(())
}

fn emit<T: serde::Serialize>(r: Result<T, ignis_core::contracts::ErrorEnvelope>) -> Result<()> {
    match r {
        Ok(v) => print(&serde_json::to_value(v)?),
        Err(e) => {
            print(&json!({"error": e}))?;
            bail!("{:?}: {}", e.code, e.message)
        }
    }
}

/// Run one command with snapshots in `store` and the fetch log at `log`.
pub fn run(cmd: WebCmd, store: Option<Arc<dyn ArtifactStore>>, log: PathBuf) -> Result<()> {
    let provider = ParallelSearch::from_env().map(|p| Arc::new(p) as Arc<dyn SearchProvider>);
    match (&cmd, store) {
        (
            WebCmd::Search {
                query,
                page_size,
                execute: false,
            },
            _,
        ) => print(&json!({
            "dry_run": true,
            "provider": provider.as_ref().map(|p| p.descriptor()),
            "provider_configured": provider.is_some(),
            "key_env": PARALLEL_KEY_ENV,
            "would_send": ParallelSearch::body(query, page_size.unwrap_or(super::DEFAULT_PAGE_SIZE) as usize + 1),
            "how_to_run": "add --execute (sends the query to the provider)",
        })),
        (
            WebCmd::Fetch {
                url,
                execute: false,
            },
            _,
        ) => {
            let p = FetchPolicy::default();
            print(&json!({
                "dry_run": true,
                "url_allowed_before_dns": super::fetch::check_url(url).is_ok(),
                "policy": {
                    "max_redirects": p.max_redirects, "max_bytes": p.max_bytes,
                    "connect_timeout_s": p.connect_timeout.as_secs(),
                    "total_timeout_s": p.total_timeout.as_secs(),
                },
                "how_to_run": "add --execute (fetches the URL and stores a snapshot)",
            }))
        }
        (_, None) => bail!("this command needs the artifact store"),
        (_, Some(store)) => {
            let web =
                WebDiscovery::new(store, log, provider, FetchPolicy::default(), Origin::System);
            let scope = cli_scope();
            match cmd {
                WebCmd::Search {
                    query, page_size, ..
                } => emit(web.search(
                    &scope,
                    &WebSearchRequest {
                        schema_version: EvidenceSchema::V2,
                        query,
                        page_size,
                        cursor: None,
                    },
                )),
                WebCmd::Fetch { url, .. } => emit(web.fetch(
                    &scope,
                    &WebFetchRequest {
                        schema_version: EvidenceSchema::V2,
                        url,
                    },
                )),
                WebCmd::Snapshot { artifact_id, text } => {
                    let id = ArtifactId::new(artifact_id)?;
                    let snap = web.snapshot(&scope, &id);
                    emit(snap.map(|s| {
                        json!({
                            "artifact": s.artifact,
                            "hash_verified": true,
                            "records": s.records,
                            "text": text.then(|| String::from_utf8_lossy(&s.bytes).into_owned()),
                        })
                    }))
                }
            }
        }
    }
}
