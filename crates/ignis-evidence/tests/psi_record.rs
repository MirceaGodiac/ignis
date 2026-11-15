//! PSI investigation records as citable sources: rendering from saved PSI metadata (fixture
//! `tests/fixtures/psi_record`), ingest into the catalog, and search that finds the record's DOI,
//! size and hardware with a valid locator, also in a scope the lockbox restricts.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};

use ignis_core::contracts::{
    Capability, EvidenceSchema, ExecutionScope, Family, OffsetUnit, Partition, PrincipalId,
    ReadRange, ReadRequest, ScopeId, ScopeRole, SearchFilters, SearchMode, SearchPage,
    SearchRequest, SourceLocator, Split, WorkspaceId,
};
use ignis_core::hashing::sha256_hex;
use ignis_core::services::KnowledgeService;
use ignis_evidence::service::{EvidenceService, Lockbox};
use ignis_evidence::{catalog, dense, psi_record, records};
use serde_json::{json, Value};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/psi_record")
}

fn tmp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("ignis-psi-record-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// Manifests and raw metadata as `ignis sources inspect` leaves them: PSI-69 with its raw response
/// (cited by hash from the investigation manifest), PSI-68 with the investigation manifest only.
fn layout(dir: &Path) -> (PathBuf, PathBuf) {
    let (manifests, meta) = (dir.join("manifests"), dir.join("meta"));
    std::fs::create_dir_all(&manifests).unwrap();
    std::fs::create_dir_all(&meta).unwrap();
    let raw = std::fs::read(fixtures().join("PSI-69.json")).unwrap();
    std::fs::write(meta.join("PSI-69.json"), &raw).unwrap();
    let mut info: Value = serde_json::from_slice(
        &std::fs::read(fixtures().join("PSI-69.investigation.json")).unwrap(),
    )
    .unwrap();
    info["metadata_url_sha256"] = json!(sha256_hex(&raw));
    std::fs::write(
        manifests.join("PSI-69.investigation.json"),
        serde_json::to_string_pretty(&info).unwrap(),
    )
    .unwrap();
    std::fs::copy(
        fixtures().join("PSI-69.listing.jsonl"),
        manifests.join("PSI-69.listing.jsonl"),
    )
    .unwrap();
    std::fs::copy(
        fixtures().join("PSI-68.investigation.json"),
        manifests.join("PSI-68.investigation.json"),
    )
    .unwrap();
    (manifests, meta)
}

fn full_text(r: &psi_record::Record) -> String {
    String::from_utf8(r.bytes()).unwrap()
}

#[test]
fn renders_one_fact_per_line_from_psi_metadata() {
    let d = tmp("render");
    let (manifests, meta) = layout(&d);
    let (rec, report) = psi_record::load("PSI-69", &manifests, &meta).unwrap();
    assert_eq!(report["raw_metadata"], "used");
    let t = full_text(&rec);
    for line in [
        "PSI investigation number (accession): PSI-69",
        "Investigation acronym: FLEX",
        "Investigation title: Flame Extinguishment Experiment",
        "Dataset DOI: 10.60555/mbq8-0451",
        "DOI link: https://doi.org/10.60555/mbq8-0451",
        "Dataset (investigation) version at PSI: 5 (Metadata update)",
        "Licence: CC0-1.0 (Creative Commons Zero v1.0 Universal), https://spdx.org/licenses/CC0-1.0.html",
        "PSI landing page: https://psi.nasa.gov/physci/repo/data/investigations/PSI-69",
        "ISS hardware / apparatus: Combustion Integrated Rack (CIR)-Fluids and Combustion Facility (FCF), Multi-user Droplet Combustion Appartatus (MDCA)",
        "Managing NASA center: Glenn Research Center (GRC)",
        "Principal Investigator (PI): Forman A Williams, University of California, San Diego",
        "Investigation start date: 2007-08-01 (PSI: 08-01-2007)",
        "Data release date: 2016-07-31 (PSI: 31-Jul-2016)",
        "Total dataset size (sum of the PSI file listing): 0.00 GB (4.98 MB; 4,977,492 bytes) in 12 files",
        "Category Raw Data: 2 files,",
        "Total file size stated by PSI metadata: 923.83 GB (923,832,535,657 bytes)",
        "Publications listed by PSI: 2",
    ] {
        assert!(
            t.lines().any(|l| l.starts_with(line)),
            "missing line {line:?} in\n{t}"
        );
    }
    // PI before co-investigators; no contact details, no results summary
    let pi = t.find("Principal Investigator (PI):").unwrap();
    let coi = t.find("Co-Investigator (Co-I):").unwrap();
    assert!(pi < coi);
    assert!(!t.contains("redacted@example.org") && !t.contains("555-0100"));
    assert!(!t.contains("SECRET-RESULT-SUMMARY"));
    // every page names the investigation, so a search on any field finds its record
    for (_, p) in &rec.pages {
        assert!(p.contains("Investigation: PSI-69 FLEX (Flame Extinguishment Experiment)"));
    }
    // deterministic: the same inputs give the same bytes
    let (again, _) = psi_record::load("PSI-69", &manifests, &meta).unwrap();
    assert_eq!(again.bytes(), rec.bytes());

    // manifest-only record (no raw response, no listing): identity, hardware and PSI's total size
    let (r68, rep68) = psi_record::load("PSI-68", &manifests, &meta).unwrap();
    assert!(rep68["raw_metadata"]
        .as_str()
        .unwrap()
        .starts_with("absent"));
    let t = full_text(&r68);
    assert!(t.contains("Dataset DOI: 10.60555/byp8-h018"), "{t}");
    assert!(t.contains("Investigation acronym: FLEX-2"));
    assert!(t.contains(
        "Total file size stated by PSI metadata: 1,224.75 GB (1.22 TB; 1,224,752,113,090 bytes)"
    ));

    // a raw response whose hash the manifest does not cite is not used
    std::fs::write(meta.join("PSI-69.json"), b"{\"doi\": \"10.0/forged\"}").unwrap();
    let (r, rep) = psi_record::load("PSI-69", &manifests, &meta).unwrap();
    assert!(rep["raw_metadata"]
        .as_str()
        .unwrap()
        .starts_with("not used"));
    assert!(!full_text(&r).contains("forged"));
}

fn scope(partitions: &[Partition]) -> ExecutionScope {
    ExecutionScope {
        scope_id: ScopeId::new("test-scope").unwrap(),
        workspace_id: WorkspaceId::new("ws").unwrap(),
        principal: PrincipalId::new("tester").unwrap(),
        role: ScopeRole::Research,
        partitions: partitions.iter().copied().collect(),
        capabilities: BTreeSet::from([Capability::ReadSources]),
        corpus_snapshot: None,
        parent_scope_id: None,
    }
}

/// PSI-69 runs split train / test, so a train+validation scope has a sealed partition.
fn lockbox() -> Lockbox {
    let split = Split {
        id: "split".into(),
        cohort_id: "cohort".into(),
        group_key: "session".into(),
        rule: "fixture".into(),
        seed: 0,
        run_to_group: BTreeMap::from([
            ("PSI-69:R1".to_string(), "g1".to_string()),
            ("PSI-69:R2".to_string(), "g2".to_string()),
        ]),
        train: vec!["g1".into()],
        validation: vec![],
        test: vec!["g2".into()],
        frozen_at: "2026-11-07T00:00:00Z".into(),
    };
    Lockbox::new(&[(split, Family::Flex)], HashMap::new())
}

/// A FLEX report that mentions the investigation (distractors for the records), with enough
/// pages of other text that BM25 term weights behave as in the real corpus.
fn add_report(db: &Path) {
    let sha = "abababababababababababababababababababababababababababababababab";
    let mut texts: Vec<String> = vec![
        "FLEX heptane droplets burned in the MDCA; the dataset of the FLEX investigation holds the droplet histories.".into(),
    ];
    for i in 0..60 {
        texts.push(format!(
            "Section {i}: flame spread over thin cellulose and PMMA samples in a low-speed flow;              soot radiation and quenching distance at ambient oxygen {i} percent."
        ));
    }
    let mut con = catalog::connect(db, false).unwrap();
    let publ = records::Publication {
        id: format!("pub-{}", &sha[..16]),
        source_sha256: sha.into(),
        title: Some("FLEX droplet combustion report".into()),
        authors: vec![],
        year: Some(2015),
        ntrs_id: None,
        doi: None,
        canonical_url: None,
        investigation_ids: vec!["PSI-69".into()],
        families: vec!["flex".into()],
        file_name: Some("report.pdf".into()),
        n_pages: texts.len() as i64,
        rights_status: "verified".into(),
        license: None,
        describes_runs: vec![],
        outcome_bearing: true,
        ingested_at: "2026-11-13T00:00:00Z".into(),
        parser_version: "test".into(),
    };
    let view = records::parse_view_id(&publ.id, "test");
    let chunks: Vec<records::Chunk> = texts
        .iter()
        .enumerate()
        .map(|(i, text)| {
            let page = i as i64 + 1;
            let n = text.chars().count() as i64;
            records::Chunk {
                id: format!("{view}:p{page}:{i:05}"),
                publication_id: publ.id.clone(),
                ordinal: i as i64,
                kind: "text".into(),
                page,
                char_start: Some(0),
                char_end: Some(n),
                section: None,
                text: text.clone(),
                context: json!({}),
                locator: SourceLocator {
                    asset_sha256: sha.into(),
                    path: Some("report.pdf".into()),
                    page: Some(page),
                    char_start: Some(0),
                    char_end: Some(n),
                    parse_view_id: Some(view.clone()),
                    offset_unit: Some(OffsetUnit::UnicodeScalar),
                    ..Default::default()
                },
                describes_runs: vec![],
                continues_runs: vec![],
                outcome_bearing: true,
                evidence_class: "source_derived".into(),
            }
        })
        .collect();
    catalog::upsert_parse_view(&mut con, &publ, &chunks, &texts, None).unwrap();
}

fn search(svc: &EvidenceService, s: &ExecutionScope, q: &str) -> SearchPage {
    svc.search(
        s,
        &SearchRequest {
            response_format: None,
            expected_terms: Vec::new(),
            schema_version: EvidenceSchema::V1,
            query: q.into(),
            mode: SearchMode::Hybrid,
            filters: SearchFilters::default(),
            corpus_snapshot: None,
            page_size: Some(5),
            cursor: None,
        },
    )
    .unwrap()
}

#[test]
fn ingested_records_are_found_cited_and_read() {
    let d = tmp("ingest");
    let (manifests, meta) = layout(&d);
    let db = d.join("evidence.sqlite");
    add_report(&db);
    let out = d.join("records");
    let mut con = catalog::connect(&db, false).unwrap();
    let rows = psi_record::ingest(
        &mut con,
        &["PSI-69", "PSI-68", "PSI-999"],
        &manifests,
        &meta,
        &out,
    )
    .unwrap();
    assert!(rows[2]["skipped"].is_string(), "{rows:?}");
    let sid69 = rows[0]["source_id"].as_str().unwrap().to_string();
    // idempotent: re-ingest keeps the ids and does not duplicate
    let again =
        psi_record::ingest(&mut con, &["PSI-69", "PSI-68"], &manifests, &meta, &out).unwrap();
    assert_eq!(again[0]["source_id"], json!(sid69));
    let n: i64 = con
        .query_row(
            "SELECT count(*) FROM publications WHERE parser_version LIKE 'psi-record-%'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(n, 2);
    // the bytes on disk are the source; verification re-derives pages and spans
    for p in catalog::publications(&con).unwrap() {
        if !psi_record::is_record(&p) {
            continue;
        }
        let rel = catalog::asset_location(&con, &p.source_sha256)
            .unwrap()
            .unwrap();
        let bytes = std::fs::read(&rel).unwrap();
        let v = psi_record::verify(&con, &p, &bytes).unwrap();
        assert_eq!(v["passed"], json!(true), "{v}");
        assert_eq!(
            p.canonical_url.as_deref(),
            Some(psi_record::landing_url(&p.investigation_ids[0]).as_str())
        );
    }
    // records.jsonl maps each source id to its landing page
    let m = std::fs::read_to_string(out.join(psi_record::RECORDS_MANIFEST)).unwrap();
    let first: Value = serde_json::from_str(m.lines().next().unwrap()).unwrap();
    assert_eq!(first["source_id"], json!(sid69));
    assert_eq!(
        first["landing_url"],
        "https://psi.nasa.gov/physci/repo/data/investigations/PSI-69"
    );
    drop(con);

    let cfg = dense::RetrievalCfg::load(&root().join("configs/evidence/retrieval.toml")).unwrap();
    let mut svc = EvidenceService::open(&db, cfg, lockbox())
        .unwrap()
        .offline();
    svc.ledger = false;
    // a scope with the test partition sealed still reads the record (it carries no outcomes)
    let research = scope(&[Partition::Train, Partition::Validation]);
    for (q, want) in [
        ("DOI FLEX PSI-69 dataset", "Dataset DOI: 10.60555/mbq8-0451"),
        ("FLEX-2 dataset size", "1,224.75 GB"),
        ("FLEX hardware apparatus", "Multi-user Droplet Combustion"),
        ("principal investigator of FLEX", "Forman A Williams"),
    ] {
        let page = search(&svc, &research, q);
        let top = page.hits.first().unwrap_or_else(|| panic!("{q}: no hit"));
        let e = &top.evidence;
        let loc = e.locator.as_ref().unwrap();
        loc.validate().unwrap();
        assert_eq!(format!("sha256:{}", loc.asset_sha256), e.source_id.as_str());
        assert!(e
            .parse_view_id
            .as_ref()
            .unwrap()
            .as_str()
            .ends_with("@psi-record-1"));
        // the cited passage holds the fact, and reading it returns the same span
        let read = svc
            .read(
                &research,
                &ReadRequest {
                    response_format: None,
                    schema_version: EvidenceSchema::V1,
                    source_id: e.source_id.clone(),
                    parse_view_id: e.parse_view_id.clone(),
                    range: ReadRange::Passage {
                        evidence_id: e.evidence_id.clone(),
                    },
                },
            )
            .unwrap();
        let span: String = read.segments.iter().map(|s| s.text.as_str()).collect();
        assert!(span.contains(want), "{q}: top hit lacks {want:?}:\n{span}");
    }
}
