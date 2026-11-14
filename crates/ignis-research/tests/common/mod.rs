//! Shared harness of the runtime conformance tests: a state store, an artifact store and a
//! `CodexBridge` over the fake app-server (`ignis-fake-codex`), all in a temp directory.

#![allow(dead_code)]

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use ignis_core::contracts::*;
use ignis_research::runtime::{preflight, AgentRuntime, CodexBridge, CodexConfig};
use ignis_research::store::Replay;
use ignis_research::{LocalArtifactStore, ResearchStore};

pub fn scope(ws: &str, write: bool) -> ExecutionScope {
    let mut caps = BTreeSet::from([Capability::ReadSources, Capability::RunModels]);
    if write {
        caps.insert(Capability::WorkspaceWrite);
    }
    ExecutionScope {
        scope_id: ScopeId::new(format!("scope-{ws}")).unwrap(),
        workspace_id: WorkspaceId::new(ws).unwrap(),
        principal: PrincipalId::new(format!("user-{ws}")).unwrap(),
        role: ScopeRole::Research,
        partitions: BTreeSet::from([Partition::Train]),
        capabilities: caps,
        corpus_snapshot: None,
        parent_scope_id: None,
    }
}

pub fn config(dir: &Path) -> CodexConfig {
    let mut cfg = CodexConfig::new(env!("CARGO_BIN_EXE_ignis-fake-codex"), dir.join("codex"));
    // The fake reads its framing mode from the environment; forward it like a credential.
    cfg.pass_env.push("IGNIS_FAKE_CODEX_FRAMING".into());
    cfg.approval_timeout = Duration::from_secs(30);
    cfg.request_timeout = Duration::from_secs(20);
    cfg
}

pub struct Harness {
    pub dir: PathBuf,
    pub store: Arc<ResearchStore>,
    pub artifacts: Arc<LocalArtifactStore>,
    pub bridge: CodexBridge,
    /// Every (episode, seq) the commit hook reported.
    pub commits: Arc<Mutex<Vec<(EpisodeId, u64)>>>,
}

impl Harness {
    pub fn new(dir: &Path) -> Self {
        Self::with_config(dir, config(dir))
    }

    pub fn with_config(dir: &Path, cfg: CodexConfig) -> Self {
        let store = Arc::new(ResearchStore::open(dir.join("state.sqlite")).unwrap());
        let artifacts = Arc::new(LocalArtifactStore::open(dir.join("artifacts")).unwrap());
        let manifest = preflight(&cfg).expect("preflight of the fake app-server");
        let commits = Arc::new(Mutex::new(Vec::new()));
        let c = commits.clone();
        let bridge = CodexBridge::new(
            cfg,
            manifest,
            store.clone(),
            artifacts.clone(),
            Arc::new(move |ep: &EpisodeId, seq| c.lock().unwrap().push((ep.clone(), seq))),
        );
        Harness {
            dir: dir.to_path_buf(),
            store,
            artifacts,
            bridge,
            commits,
        }
    }

    pub fn episode(&self, sc: &ExecutionScope, goal: &str) -> EpisodeId {
        self.store
            .create_episode(
                sc,
                goal,
                &ClientRequestId::new(format!("c-{goal}")).unwrap(),
                Default::default(),
                Origin::SyntheticFixture,
            )
            .unwrap()
            .0
            .episode_id
    }

    /// Record a user message and ask the runtime to act on it (what the host does on POST).
    pub fn say(&self, sc: &ExecutionScope, ep: &EpisodeId, text: &str) {
        let key = ClientRequestId::new(format!("m-{}", uuid_like())).unwrap();
        self.store.post_message(sc, ep, text, &key).unwrap();
        self.bridge.kick(sc, ep).unwrap();
    }

    pub fn events(&self, sc: &ExecutionScope, ep: &EpisodeId) -> Vec<ResearchEvent> {
        events(&self.store, sc, ep)
    }

    /// Wait until the log satisfies `pred` (fails after 30 s with the log).
    pub fn wait(
        &self,
        sc: &ExecutionScope,
        ep: &EpisodeId,
        what: &str,
        pred: impl Fn(&[ResearchEvent]) -> bool,
    ) -> Vec<ResearchEvent> {
        let t0 = Instant::now();
        loop {
            let evs = self.events(sc, ep);
            if pred(&evs) {
                return evs;
            }
            if t0.elapsed() > Duration::from_secs(30) {
                panic!("timed out waiting for {what}; log: {:#?}", kinds(&evs));
            }
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    pub fn wait_status(
        &self,
        sc: &ExecutionScope,
        ep: &EpisodeId,
        s: EpisodeStatus,
    ) -> Vec<ResearchEvent> {
        self.wait(sc, ep, &format!("status {s:?}"), |evs| {
            status_of(evs) == Some(s)
        })
    }

    /// Methods the fake received on the workspace's app-server, in order.
    pub fn requests(&self, ws: &str) -> Vec<String> {
        let home = std::fs::read_dir(self.dir.join("codex"))
            .unwrap()
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.is_dir() && p.file_name().unwrap().to_string_lossy().starts_with("ws-"))
            .find(|p| {
                let want = ignis_core::hashing::sha256_hex(ws.as_bytes());
                p.file_name().unwrap().to_string_lossy() == format!("ws-{}", &want[..16])
            })
            .expect("workspace home");
        std::fs::read_to_string(home.join("fake-requests.log"))
            .unwrap_or_default()
            .lines()
            .map(str::to_string)
            .collect()
    }
}

pub fn events(store: &ResearchStore, sc: &ExecutionScope, ep: &EpisodeId) -> Vec<ResearchEvent> {
    match store.replay(sc, ep, 0, 100_000).unwrap() {
        Replay::Events { events, .. } => events,
        Replay::Gone { .. } => panic!("gone"),
    }
}

pub fn status_of(evs: &[ResearchEvent]) -> Option<EpisodeStatus> {
    evs.iter().rev().find_map(|e| match e.body {
        EventBody::StatusChanged { to, .. } => Some(to),
        _ => None,
    })
}

pub fn kind(e: &ResearchEvent) -> String {
    serde_json::to_value(&e.body).unwrap()["kind"]
        .as_str()
        .unwrap()
        .to_string()
}

pub fn kinds(evs: &[ResearchEvent]) -> Vec<String> {
    evs.iter()
        .map(|e| match &e.body {
            EventBody::StatusChanged { from, to, .. } => format!("status {from:?}->{to:?}"),
            EventBody::RuntimeOpaque { method, .. } => format!("runtime.opaque {method}"),
            _ => kind(e),
        })
        .collect()
}

pub fn count(evs: &[ResearchEvent], k: &str) -> usize {
    evs.iter().filter(|e| kind(e) == k).count()
}

pub fn completed_texts(evs: &[ResearchEvent]) -> Vec<String> {
    evs.iter()
        .filter_map(|e| match &e.body {
            EventBody::AssistantMessage { text, .. } => Some(text.clone()),
            _ => None,
        })
        .collect()
}

pub fn delta_text(evs: &[ResearchEvent], message: &MessageId) -> String {
    evs.iter()
        .filter_map(|e| match &e.body {
            EventBody::AssistantDelta { message_id, text } if message_id == message => {
                Some(text.as_str())
            }
            _ => None,
        })
        .collect()
}

pub fn uuid_like() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static N: AtomicU64 = AtomicU64::new(0);
    format!(
        "{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::SeqCst)
    )
}

/// The full turn scenario (tool call, granted approval, opaque notifications, streamed answer),
/// checked end to end. Used by the framing variants.
pub fn full_turn(h: &Harness) {
    let sc = scope("ws-frame", true);
    let ep = h.episode(&sc, "framing");
    h.say(&sc, &ep, "tool opaque approval please");
    let evs = h.wait(&sc, &ep, "approval request", |e| {
        count(e, "approval.requested") == 1
    });
    let approval = evs
        .iter()
        .find_map(|e| match &e.body {
            EventBody::ApprovalRequested {
                approval_id,
                effect,
                ..
            } => {
                assert_eq!(*effect, ToolEffect::WorkspaceWrite);
                Some(approval_id.clone())
            }
            _ => None,
        })
        .unwrap();
    h.wait_status(&sc, &ep, EpisodeStatus::WaitingApproval);
    h.bridge
        .resolve_approval(&sc, &ep, &approval, ApprovalDecision::Approved)
        .unwrap();
    let evs = h.wait_status(&sc, &ep, EpisodeStatus::Completed);
    check_event_log(&evs).unwrap();
    assert_eq!(count(&evs, "tool.started"), 1);
    assert_eq!(count(&evs, "tool.finished"), 1);
    let texts = completed_texts(&evs);
    assert_eq!(texts, vec!["item/fileChange/requestApproval -> accept"]);
    let msg = evs
        .iter()
        .find_map(|e| match &e.body {
            EventBody::AssistantMessage { message_id, .. } => Some(message_id.clone()),
            _ => None,
        })
        .unwrap();
    // Deltas may be coalesced, never lost: together they are the completed message.
    assert_eq!(delta_text(&evs, &msg).trim_end(), texts[0]);
    let opaque: Vec<_> = evs
        .iter()
        .filter_map(|e| match &e.body {
            EventBody::RuntimeOpaque { method, .. } => Some(method.as_str()),
            _ => None,
        })
        .collect();
    assert!(opaque.contains(&"x/unknownNotification"), "{opaque:?}");
}
