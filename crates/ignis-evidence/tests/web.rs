//! Web discovery (R4), offline: a local fake HTTP server, the real content-addressed artifact store,
//! a fake search provider, and the MCP tool dispatcher over the real `EvidenceService`.
//!
//! The fake server listens on loopback, which the SSRF policy blocks; each test exempts exactly its
//! own server's `ip:port` (`FetchPolicy::allow_addrs`), so every other loopback address (another
//! port, `localhost`) is still refused, which is what the redirect tests prove.

use std::collections::BTreeSet;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener};
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use ignis_core::contracts::{
    Capability, ErrorCode, EvidenceSchema, ExecutionScope, ExecutionStatus, Origin, Partition,
    PrincipalId, RightsStatus, ScopeId, ScopeRole, WebContent, WebFetchRequest, WebProvider,
    WebSearchRequest, WorkspaceId,
};
use ignis_core::services::{ArtifactStore, KnowledgeService};
use ignis_evidence::web::fetch::FetchPolicy;
use ignis_evidence::web::provider::{ProviderHit, SearchProvider};
use ignis_evidence::web::rights::RightsPolicy;
use ignis_evidence::web::WebDiscovery;
use ignis_research::LocalArtifactStore;
use serde_json::{json, Value};

// ------------------------------------------------------------------ fake HTTP server

struct Reply {
    status: u16,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
    /// Send `Content-Length` (otherwise the body ends when the connection closes).
    length: bool,
}

fn ok(ct: &str, body: impl Into<Vec<u8>>) -> Reply {
    Reply {
        status: 200,
        headers: vec![("Content-Type".into(), ct.into())],
        body: body.into(),
        length: true,
    }
}

fn redirect(to: &str) -> Reply {
    Reply {
        status: 302,
        headers: vec![("Location".into(), to.into())],
        body: Vec::new(),
        length: true,
    }
}

type Routes = Arc<dyn Fn(&str) -> Reply + Send + Sync>;

struct Server {
    addr: SocketAddr,
    hits: Arc<AtomicUsize>,
}

fn serve(routes: Routes) -> Server {
    let l = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = l.local_addr().unwrap();
    let hits = Arc::new(AtomicUsize::new(0));
    let h = hits.clone();
    std::thread::spawn(move || {
        for s in l.incoming() {
            let Ok(mut s) = s else { continue };
            h.fetch_add(1, Ordering::SeqCst);
            let mut buf = Vec::new();
            let mut b = [0u8; 1024];
            while !buf.windows(4).any(|w| w == b"\r\n\r\n") {
                match s.read(&mut b) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => buf.extend_from_slice(&b[..n]),
                }
            }
            let req = String::from_utf8_lossy(&buf);
            let path = req.split_whitespace().nth(1).unwrap_or("/").to_string();
            let r = routes(&path);
            let mut head = format!("HTTP/1.1 {} X\r\nConnection: close\r\n", r.status);
            for (k, v) in &r.headers {
                head.push_str(&format!("{k}: {v}\r\n"));
            }
            if r.length {
                head.push_str(&format!("Content-Length: {}\r\n", r.body.len()));
            }
            head.push_str("\r\n");
            let _ = s.write_all(head.as_bytes());
            let _ = s.write_all(&r.body);
        }
    });
    Server { addr, hits }
}

// ------------------------------------------------------------------ fixtures

fn scope(caps: &[Capability], role: ScopeRole) -> ExecutionScope {
    ExecutionScope {
        scope_id: ScopeId::new("scope_web").unwrap(),
        workspace_id: WorkspaceId::new("ws_web").unwrap(),
        principal: PrincipalId::new("researcher").unwrap(),
        role,
        partitions: BTreeSet::from([Partition::Train, Partition::Validation]),
        capabilities: caps.iter().copied().collect(),
        corpus_snapshot: None,
        parent_scope_id: None,
    }
}

fn research() -> ExecutionScope {
    scope(
        &[Capability::ReadSources, Capability::NetworkFetch],
        ScopeRole::Research,
    )
}

struct FakeProvider {
    hits: Vec<ProviderHit>,
    asked: Mutex<Vec<usize>>,
}

impl SearchProvider for FakeProvider {
    fn descriptor(&self) -> WebProvider {
        WebProvider {
            name: "fake-provider".into(),
            tool: "search".into(),
            version: None,
        }
    }
    fn search(
        &self,
        _q: &str,
        max: usize,
    ) -> Result<Vec<ProviderHit>, ignis_core::contracts::ErrorEnvelope> {
        self.asked.lock().unwrap().push(max);
        Ok(self.hits.iter().take(max).cloned().collect())
    }
}

fn provider(n: usize) -> Arc<FakeProvider> {
    let hits = (0..n)
        .map(|i| ProviderHit {
            url: if i == 0 {
                "https://www.nasa.gov/flame-spread".into()
            } else {
                format!("https://example.org/paper-{i}")
            },
            title: Some(format!("Result {i}")),
            excerpt: Some(format!("excerpt {i}")),
        })
        .collect();
    Arc::new(FakeProvider {
        hits,
        asked: Mutex::new(Vec::new()),
    })
}

struct Env {
    _dir: tempfile::TempDir,
    store: Arc<LocalArtifactStore>,
    web: Arc<WebDiscovery>,
}

fn env(allow: &[SocketAddr], max_bytes: u64, p: Option<Arc<FakeProvider>>) -> Env {
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(LocalArtifactStore::open(dir.path().join("artifacts")).unwrap());
    let policy = FetchPolicy {
        max_bytes,
        allow_addrs: allow.iter().copied().collect(),
        ..FetchPolicy::default()
    };
    let web = Arc::new(WebDiscovery::new(
        store.clone(),
        dir.path().join("web/fetches.jsonl"),
        p.map(|p| p as Arc<dyn SearchProvider>),
        policy,
        Origin::System,
    ));
    Env {
        _dir: dir,
        store,
        web,
    }
}

fn fetch_req(url: &str) -> WebFetchRequest {
    WebFetchRequest {
        schema_version: EvidenceSchema::V2,
        url: url.into(),
    }
}

const PAGE: &str = r#"<!doctype html><html><head><title>Flame spread over thin fuels</title>
<link rel="canonical" href="/papers/flame">
<script>var secret = "never in the snapshot";</script><style>p{color:red}</style></head>
<body><h1>Flame spread</h1><p>Opposed-flow flame spread &amp; extinction in microgravity.</p>
<p>Second paragraph.</p></body></html>"#;

// ------------------------------------------------------------------ fetch → snapshot → read back

#[test]
fn fetch_stores_a_snapshot_that_reads_back_hash_verified() {
    let version = Arc::new(AtomicUsize::new(0));
    let v = version.clone();
    let srv = serve(Arc::new(move |path| match path {
        "/paper" if v.load(Ordering::SeqCst) == 0 => ok("text/html; charset=utf-8", PAGE),
        "/paper" => ok(
            "text/html",
            "<html><body><p>Revised page.</p></body></html>",
        ),
        _ => Reply {
            status: 404,
            ..ok("text/plain", "")
        },
    }));
    let e = env(&[srv.addr], 1 << 20, None);
    let s = research();
    let url = format!("http://{}/paper", srv.addr);
    let c = e.web.fetch(&s, &fetch_req(&url)).unwrap();
    c.validate().unwrap();
    assert_eq!(c.content, WebContent::FullSnapshot);
    assert_eq!(c.final_url.as_deref(), Some(url.as_str()));
    assert_eq!(c.title.as_deref(), Some("Flame spread over thin fuels"));
    assert_eq!(
        c.canonical_url,
        Some(format!("http://{}/papers/flame", srv.addr))
    );
    assert_eq!(c.provider.tool, "web_fetch");
    assert!(
        c.promoted_source_id.is_none(),
        "never promoted into the corpus"
    );
    let snap = c.snapshot.clone().unwrap();
    assert!(snap.media_type.starts_with("text/plain"));

    // read back by id: bytes, hash and the fetch record
    let back = e.web.snapshot(&s, &snap.artifact_id).unwrap();
    assert_eq!(back.artifact.sha256, snap.sha256);
    let text = String::from_utf8(back.bytes.clone()).unwrap();
    assert!(text.contains("Opposed-flow flame spread & extinction in microgravity."));
    assert!(
        !text.contains("never in the snapshot"),
        "scripts are dropped"
    );
    assert!(!text.contains("color:red"), "styles are dropped");
    let r = &back.records[0];
    assert_eq!(r.http_status, 200);
    assert_eq!(r.content_type.as_deref(), Some("text/html; charset=utf-8"));
    assert!(r.extracted_text);
    assert_eq!(r.source_bytes, PAGE.len() as u64);
    assert_eq!(r.fetched_at, c.accessed_at);
    assert!(r.fetched_at.as_str().ends_with('Z'));

    // the same bytes through the artifact store (what `artifacts_read` serves)
    let mut via_store = Vec::new();
    e.store
        .open(&s, &snap.artifact_id, 0, None)
        .unwrap()
        .read_to_end(&mut via_store)
        .unwrap();
    assert_eq!(via_store, back.bytes);

    // another workspace does not see it
    let mut other = research();
    other.workspace_id = WorkspaceId::new("ws_other").unwrap();
    assert!(e.web.snapshot(&other, &snap.artifact_id).is_err());

    // re-fetch after the page changed: a new snapshot, the old one still resolves
    version.store(1, Ordering::SeqCst);
    let c2 = e.web.fetch(&s, &fetch_req(&url)).unwrap();
    let snap2 = c2.snapshot.unwrap();
    assert_ne!(snap2.artifact_id, snap.artifact_id);
    assert_eq!(
        String::from_utf8(e.web.snapshot(&s, &snap2.artifact_id).unwrap().bytes).unwrap(),
        "Revised page."
    );
    assert_eq!(
        e.web.snapshot(&s, &snap.artifact_id).unwrap().bytes,
        back.bytes
    );
}

#[test]
fn a_non_html_document_is_stored_byte_for_byte() {
    let pdf: Vec<u8> = b"%PDF-1.4 synthetic fixture bytes \x00\x01\x02".to_vec();
    let body = pdf.clone();
    let srv = serve(Arc::new(move |_| ok("application/pdf", body.clone())));
    let e = env(&[srv.addr], 1 << 20, None);
    let s = research();
    let c = e
        .web
        .fetch(&s, &fetch_req(&format!("http://{}/doc.pdf", srv.addr)))
        .unwrap();
    let snap = c.snapshot.unwrap();
    assert_eq!(snap.media_type, "application/pdf");
    let back = e.web.snapshot(&s, &snap.artifact_id).unwrap();
    assert_eq!(back.bytes, pdf);
    assert!(!back.records[0].extracted_text);
    assert_eq!(back.records[0].source_sha256, snap.sha256);
}

// ------------------------------------------------------------------ redirects and SSRF

#[test]
fn redirects_are_followed_and_recorded() {
    let srv = serve(Arc::new(|path| match path {
        "/old" => redirect("/mid"),
        "/mid" => redirect("paper"),
        _ => ok("text/html", PAGE),
    }));
    let e = env(&[srv.addr], 1 << 20, None);
    let s = research();
    let c = e
        .web
        .fetch(&s, &fetch_req(&format!("http://{}/old", srv.addr)))
        .unwrap();
    let final_url = format!("http://{}/paper", srv.addr);
    assert_eq!(c.final_url.as_deref(), Some(final_url.as_str()));
    assert_eq!(c.url, format!("http://{}/old", srv.addr));
    let r = &e
        .web
        .snapshot(&s, &c.snapshot.unwrap().artifact_id)
        .unwrap()
        .records[0];
    assert_eq!(
        r.redirects,
        vec![format!("http://{}/mid", srv.addr), final_url.clone()]
    );
}

#[test]
fn a_redirect_to_loopback_or_metadata_is_blocked_on_that_hop() {
    // the target server would answer if it were ever reached
    let inner = serve(Arc::new(|_| ok("text/plain", "internal secret")));
    let inner_addr = inner.addr;
    let srv = serve(Arc::new(move |path| match path {
        "/to-loopback" => redirect(&format!("http://127.0.0.1:{}/admin", inner_addr.port())),
        "/to-localhost" => redirect(&format!("http://localhost:{}/admin", inner_addr.port())),
        "/to-metadata" => redirect("http://169.254.169.254/latest/meta-data/"),
        "/to-file" => redirect("file:///etc/passwd"),
        _ => ok("text/plain", "public"),
    }));
    let e = env(&[srv.addr], 1 << 20, None);
    let s = research();
    for path in ["/to-loopback", "/to-localhost", "/to-metadata"] {
        let err = e
            .web
            .fetch(&s, &fetch_req(&format!("http://{}{path}", srv.addr)))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::ScopeDenied, "{path}: {}", err.message);
        assert!(err.message.contains("SSRF"), "{}", err.message);
    }
    let err = e
        .web
        .fetch(&s, &fetch_req(&format!("http://{}/to-file", srv.addr)))
        .unwrap_err();
    assert_eq!(err.code, ErrorCode::InvalidRequest);
    assert_eq!(
        inner.hits.load(Ordering::SeqCst),
        0,
        "the internal server was never reached"
    );
    // nor directly
    let err = e
        .web
        .fetch(
            &s,
            &fetch_req(&format!("http://127.0.0.1:{}/", inner_addr.port())),
        )
        .unwrap_err();
    assert_eq!(err.code, ErrorCode::ScopeDenied);
    assert_eq!(inner.hits.load(Ordering::SeqCst), 0);
    // nothing was snapshotted
    assert!(!Path::new(&e._dir.path().join("web/fetches.jsonl")).exists());
}

#[test]
fn redirect_chains_are_bounded() {
    let srv = serve(Arc::new(|path| {
        let n: u32 = path.trim_start_matches("/r").parse().unwrap_or(0);
        redirect(&format!("/r{}", n + 1))
    }));
    let e = env(&[srv.addr], 1 << 20, None);
    let err = e
        .web
        .fetch(&research(), &fetch_req(&format!("http://{}/r0", srv.addr)))
        .unwrap_err();
    assert_eq!(err.code, ErrorCode::InvalidRequest);
    assert!(
        err.message.contains("redirects more than 5"),
        "{}",
        err.message
    );
    assert_eq!(srv.hits.load(Ordering::SeqCst), 6);
}

// ------------------------------------------------------------------ size budget

#[test]
fn a_body_over_the_size_budget_is_refused_not_truncated() {
    let srv = serve(Arc::new(|path| {
        let mut r = ok("application/octet-stream", vec![b'x'; 4096]);
        r.length = path == "/declared";
        r
    }));
    let e = env(&[srv.addr], 1024, None);
    for path in ["/declared", "/streamed"] {
        let err = e
            .web
            .fetch(
                &research(),
                &fetch_req(&format!("http://{}{path}", srv.addr)),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::Unsupported, "{path}: {}", err.message);
        assert!(err.message.contains("1024 bytes"));
    }
    assert!(!e._dir.path().join("web/fetches.jsonl").exists());
}

// ------------------------------------------------------------------ rights

#[test]
fn unknown_rights_are_explicitly_unverified() {
    let srv = serve(Arc::new(|path| match path {
        "/licensed" => ok(
            "text/html",
            r#"<html><head><link rel="license" href="https://creativecommons.org/licenses/by-nc/4.0/"></head><body><p>x</p></body></html>"#,
        ),
        _ => ok("text/html", PAGE),
    }));
    let e = env(&[srv.addr], 1 << 20, None);
    let c = e
        .web
        .fetch(
            &research(),
            &fetch_req(&format!("http://{}/plain", srv.addr)),
        )
        .unwrap();
    assert_eq!(c.rights.status, RightsStatus::Unverified);
    assert_eq!(c.rights.license, None);
    let v = serde_json::to_value(&c).unwrap();
    assert_eq!(
        v["rights"]["status"], "unverified",
        "the status is written, not omitted"
    );

    let c = e
        .web
        .fetch(
            &research(),
            &fetch_req(&format!("http://{}/licensed", srv.addr)),
        )
        .unwrap();
    assert_eq!(c.rights.status, RightsStatus::Noncommercial);
    assert!(c
        .rights
        .license
        .unwrap()
        .starts_with("declared by the page:"));

    let p = RightsPolicy::default();
    assert_eq!(
        p.rights("https://www.nasa.gov/x", None).status,
        RightsStatus::Verified
    );
    let ntrs = p.rights("https://ntrs.nasa.gov/citations/2020", None);
    assert_eq!(
        ntrs.status,
        RightsStatus::Unverified,
        "NTRS is per document"
    );
    assert_eq!(ntrs.license, None);
    assert_eq!(
        p.rights("https://notnasa.gov.example.com/", None).status,
        RightsStatus::Unverified
    );
}

// ------------------------------------------------------------------ search

#[test]
fn search_pages_provider_results_as_excerpt_candidates() {
    let p = provider(23);
    let e = env(&[], 1 << 20, Some(p.clone()));
    let s = research();
    let mut req = WebSearchRequest {
        schema_version: EvidenceSchema::V2,
        query: "opposed flow flame spread".into(),
        page_size: Some(10),
        cursor: None,
    };
    let mut seen = Vec::new();
    loop {
        let page = e.web.search(&s, &req).unwrap();
        page.validate().unwrap();
        assert_eq!(page.provider.name, "fake-provider");
        for c in &page.candidates {
            assert_eq!(c.content, WebContent::Excerpt);
            assert!(c.snapshot.is_none() && c.final_url.is_none());
            seen.push(c.url.clone());
        }
        match page.next_cursor {
            Some(cur) => req.cursor = Some(cur),
            None => break,
        }
    }
    assert_eq!(seen.len(), 23);
    assert_eq!(seen[22], "https://example.org/paper-22");
    assert_eq!(*p.asked.lock().unwrap(), vec![11, 21, 31]);
    // rights from the domain rule only (nothing fetched)
    let first = e
        .web
        .search(
            &s,
            &WebSearchRequest {
                cursor: None,
                ..req.clone()
            },
        )
        .unwrap();
    assert_eq!(first.candidates[0].rights.status, RightsStatus::Verified);
    assert_eq!(first.candidates[1].rights.status, RightsStatus::Unverified);
    // a cursor of another query is refused
    let other = WebSearchRequest {
        query: "something else".into(),
        cursor: first.next_cursor.clone(),
        ..req.clone()
    };
    assert_eq!(
        e.web.search(&s, &other).unwrap_err().code,
        ErrorCode::InvalidRequest
    );
}

#[test]
fn evaluation_scopes_and_scopes_without_network_fetch_are_refused() {
    let e = env(&[], 1 << 20, Some(provider(3)));
    let req = WebSearchRequest {
        schema_version: EvidenceSchema::V2,
        query: "flame".into(),
        page_size: None,
        cursor: None,
    };
    let eval = scope(
        &[Capability::ReadSources, Capability::NetworkFetch],
        ScopeRole::Evaluation,
    );
    let no_net = scope(&[Capability::ReadSources], ScopeRole::Research);
    for s in [&eval, &no_net] {
        assert_eq!(
            e.web.search(s, &req).unwrap_err().code,
            ErrorCode::ScopeDenied
        );
        assert_eq!(
            e.web
                .fetch(s, &fetch_req("https://example.org/"))
                .unwrap_err()
                .code,
            ErrorCode::ScopeDenied
        );
    }
    let no_provider = env(&[], 1 << 20, None);
    assert_eq!(
        no_provider.web.search(&research(), &req).unwrap_err().code,
        ErrorCode::CapabilityUnavailable
    );
}

// ------------------------------------------------------------------ MCP tool → service

fn evidence_service(
    dir: &Path,
    web: Option<Arc<WebDiscovery>>,
) -> ignis_evidence::service::EvidenceService {
    let db = dir.join("evidence.sqlite");
    drop(ignis_evidence::catalog::connect(&db, false).unwrap());
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let cfg =
        ignis_evidence::dense::RetrievalCfg::load(&root.join("configs/evidence/retrieval.toml"))
            .unwrap();
    let mut s = ignis_evidence::service::EvidenceService::open(
        &db,
        cfg,
        ignis_evidence::service::Lockbox::default(),
    )
    .unwrap()
    .offline();
    s.ledger = false;
    match web {
        Some(w) => s.with_web(w),
        None => s,
    }
}

fn tool_host(e: &Env, web: ignis_mcp::WebTools, attach: bool) -> ignis_mcp::ToolHost {
    let state = ignis_research::ResearchStore::open(e._dir.path().join("state.sqlite")).unwrap();
    let knowledge = evidence_service(e._dir.path(), attach.then(|| e.web.clone()));
    ignis_mcp::ToolHost::new(
        ignis_mcp::ToolServices {
            knowledge: Some(Arc::new(knowledge) as Arc<dyn KnowledgeService>),
            web,
            ..Default::default()
        },
        Arc::new(state),
        e.store.clone(),
        Origin::System,
        Arc::new(|| {}),
    )
}

#[test]
fn mcp_web_tools_reach_the_service_and_are_off_by_default() {
    let srv = serve(Arc::new(|_| ok("text/html", PAGE)));
    let e = env(&[srv.addr], 1 << 20, Some(provider(4)));
    let s = research();

    // default deployment: not registered, and the catalog says why
    let off = tool_host(&e, ignis_mcp::WebTools::default(), false);
    assert!(!off.registered().iter().any(|d| d.name.starts_with("web_")));
    let cat = ignis_mcp::catalog(&ignis_mcp::ToolServices {
        knowledge: Some(Arc::new(evidence_service(e._dir.path(), None))),
        ..Default::default()
    });
    for t in cat.iter().filter(|t| t.descriptor.name.starts_with("web_")) {
        assert_eq!(t.unavailable, Some(ignis_mcp::WEB_DISABLED));
    }
    let r = off.call(
        &s,
        "web_fetch",
        json!({"schema_version": "evidence.v2", "url": "https://example.org/"}),
    );
    assert_eq!(r.error.unwrap().code, ErrorCode::NotFound);

    // fetch only: web_search reports the missing provider
    let cat = ignis_mcp::catalog(&ignis_mcp::ToolServices {
        knowledge: Some(Arc::new(evidence_service(e._dir.path(), None))),
        web: ignis_mcp::WebTools::FetchOnly,
        ..Default::default()
    });
    let reason = |n: &str| {
        cat.iter()
            .find(|t| t.descriptor.name == n)
            .unwrap()
            .unavailable
    };
    assert_eq!(
        reason("web_search"),
        Some(ignis_mcp::WEB_SEARCH_NO_PROVIDER)
    );
    assert_eq!(reason("web_fetch"), None);

    // enabled: both tools go through the EvidenceService into WebDiscovery
    let on = tool_host(&e, ignis_mcp::WebTools::SearchAndFetch, true);
    let r = on.call(
        &s,
        "web_fetch",
        json!({"schema_version": "evidence.v2", "url": format!("http://{}/paper", srv.addr)}),
    );
    assert_eq!(
        r.execution_status,
        ExecutionStatus::Completed,
        "{:?}",
        r.error
    );
    let v: Value = r.result.unwrap().value;
    assert_eq!(v["content"], "full_snapshot");
    let id = v["snapshot"]["artifact_id"].as_str().unwrap().to_string();
    // the snapshot is an artifact of the caller's workspace: artifacts_read opens it
    let r = on.call(&s, "artifacts_read", json!({"artifact_id": id}));
    assert_eq!(
        r.execution_status,
        ExecutionStatus::Completed,
        "{:?}",
        r.error
    );
    assert!(serde_json::to_string(&r.result.unwrap().value)
        .unwrap()
        .contains("Opposed-flow flame spread"));

    let r = on.call(
        &s,
        "web_search",
        json!({"schema_version": "evidence.v2", "query": "flame spread", "page_size": 2}),
    );
    assert_eq!(
        r.execution_status,
        ExecutionStatus::Completed,
        "{:?}",
        r.error
    );
    let v = r.result.unwrap().value;
    assert_eq!(v["candidates"].as_array().unwrap().len(), 2);
    assert!(v["next_cursor"].is_string());

    // the host's capability check: no network_fetch, refused before the service
    let no_net = scope(&[Capability::ReadSources], ScopeRole::Research);
    let r = on.call(
        &no_net,
        "web_search",
        json!({"schema_version": "evidence.v2", "query": "x"}),
    );
    assert_eq!(r.error.unwrap().code, ErrorCode::ScopeDenied);
}

// ------------------------------------------------------------------ live (opt-in)

/// Live Parallel search: `PARALLEL_API_KEY=... cargo test -p ignis-evidence --test web -- --ignored live_`.
#[test]
#[ignore = "live network: needs PARALLEL_API_KEY and explicit opt-in"]
fn live_parallel_search() {
    let p = ignis_evidence::web::provider::ParallelSearch::from_env().expect("PARALLEL_API_KEY");
    let hits = p.search("microgravity flame spread thin fuel", 3).unwrap();
    assert!(!hits.is_empty());
    assert!(hits.iter().all(|h| h.url.starts_with("http")));
}

/// Live fetch of a public NASA page through the real SSRF policy.
#[test]
#[ignore = "live network: explicit opt-in"]
fn live_fetch_nasa_page() {
    let e = env(&[], 32 << 20, None);
    let c = e
        .web
        .fetch(&research(), &fetch_req("https://www.nasa.gov/"))
        .unwrap();
    assert_eq!(c.rights.status, RightsStatus::Verified);
}
