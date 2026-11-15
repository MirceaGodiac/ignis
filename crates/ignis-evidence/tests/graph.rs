//! R9b: the combustion graph through `KnowledgeService::graph_neighbors`, built from the real NASA
//! inputs the repo carries (the PSI investigation metadata in `data/manifests`, the experimental
//! tables of `crates/ignis-data/tests/fixtures` through the R9a conditions builder) and a synthetic
//! catalog publication (passages that name runs). Covers the two acceptance queries ("investigations
//! sharing MDCA", "runs with O2 ≥ 30 %") with the cells and locators they rest on, the provenance of
//! each kind of edge, explicit gaps, paging, and the lockbox: a run of a sealed partition and an
//! outcome-bearing passage about it never reach a research scope, nor its counts.

use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use ignis_core::contracts::{
    Capability, ConceptId, ConceptKind, ConceptRef, ConditionCell, ConditionFilter, ConditionKey,
    ConditionsQuery, ConditionsTable, DatasetSchema, DirectionFilter, EdgeDirection, EntityRef,
    ErrorCode, EvidenceId, EvidenceSchema, ExecutionScope, Family, Finite, GraphNeighborsPage,
    GraphNeighborsRequest, GraphNode, OffsetUnit, Origin, Partition, PrincipalId, ProvenanceLink,
    RawValue, Relation, Resolution, RunId, ScopeId, ScopeRole, SourceId, SourceLocator, Split,
    WorkspaceId,
};
use ignis_core::services::KnowledgeService;
use ignis_evidence::service::{EvidenceService, Lockbox};
use ignis_evidence::{catalog, dense, graph, records};
use serde_json::{json, Value};

const PUB_SHA: &str = "c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0";
/// Investigations whose NASA tables are fixtures: FLEX and BASS-II through their family records
/// (ingested from the fixtures), the others through the table specs.
const FIXTURE_INVS: [&str; 6] = ["PSI-20", "PSI-23", "PSI-25", "PSI-68", "PSI-69", "PSI-98"];
const SPEC_INVS: [&str; 4] = ["PSI-20", "PSI-23", "PSI-68", "PSI-98"];

static LOCK: Mutex<()> = Mutex::new(());

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

fn scope(role: ScopeRole, partitions: &[Partition]) -> ExecutionScope {
    ExecutionScope {
        scope_id: ScopeId::new("test-scope").unwrap(),
        workspace_id: WorkspaceId::new("ws").unwrap(),
        principal: PrincipalId::new("tester").unwrap(),
        role,
        partitions: partitions.iter().copied().collect(),
        capabilities: BTreeSet::from([Capability::ReadSources]),
        corpus_snapshot: None,
        parent_scope_id: None,
    }
}

fn research() -> ExecutionScope {
    scope(
        ScopeRole::Research,
        &[Partition::Train, Partition::Validation],
    )
}

fn evaluation() -> ExecutionScope {
    scope(
        ScopeRole::Evaluation,
        &[Partition::Train, Partition::Validation, Partition::Test],
    )
}

/// `IGNIS_DATA` with the fixture tables and their manifest lines, plus every PSI investigation
/// metadata file of the repo (the graph's hardware source). Caller holds LOCK.
fn data_root(dir: &Path) -> PathBuf {
    let root = dir.join("data");
    let m = root.join("manifests");
    std::fs::create_dir_all(&m).unwrap();
    for inv in FIXTURE_INVS {
        let fx = repo().join("crates/ignis-data/tests/fixtures").join(inv);
        let raw = root.join("raw/samples").join(inv);
        std::fs::create_dir_all(&raw).unwrap();
        let mut names = vec![];
        for e in std::fs::read_dir(&fx).unwrap().flatten() {
            std::fs::copy(e.path(), raw.join(e.file_name())).unwrap();
            names.push(e.file_name().to_string_lossy().into_owned());
        }
        let all =
            std::fs::read_to_string(repo().join(format!("data/manifests/{inv}.jsonl"))).unwrap();
        let lines: Vec<&str> = all
            .lines()
            .filter(|l| {
                let v: Value = serde_json::from_str(l).unwrap();
                names.iter().any(|n| v["file_name"] == *n)
            })
            .collect();
        assert_eq!(
            lines.len(),
            names.len(),
            "{inv}: a fixture without its Source record"
        );
        std::fs::write(m.join(format!("{inv}.jsonl")), lines.join("\n") + "\n").unwrap();
    }
    for e in std::fs::read_dir(repo().join("data/manifests"))
        .unwrap()
        .flatten()
    {
        let n = e.file_name().to_string_lossy().into_owned();
        if n.ends_with(".investigation.json") || n.ends_with(".listing.jsonl") {
            std::fs::copy(e.path(), m.join(&n)).unwrap();
        }
    }
    root
}

/// (text, runs it names, outcome-bearing)
const PASSAGES: [(&str, &[&str], bool); 5] = [
    (
        "The MDCA chamber was filled to 1 atm before ignition.",
        &[],
        false,
    ),
    (
        "Test FLEX-001 burned to extinction at 2.1 mm.",
        &["alias:flex:FLEX-001"],
        true,
    ),
    (
        "Test FLEX-002 extinguished radiatively at 4.8 mm.",
        &["alias:flex:FLEX-002"],
        true,
    ),
    (
        "Test FLEX-003 extinguished with a residue.",
        &["alias:flex:FLEX-003"],
        true,
    ),
    (
        "Run PSI-69:R1 is the reference for the series.",
        &["PSI-69:R1"],
        false,
    ),
];

fn chunk_id(i: usize) -> String {
    let view = records::parse_view_id(&format!("pub-{}", &PUB_SHA[..16]), "test");
    format!("{view}:p1:{i:05}")
}

fn add_publication(con: &mut rusqlite::Connection) {
    let publ = records::Publication {
        id: format!("pub-{}", &PUB_SHA[..16]),
        source_sha256: PUB_SHA.into(),
        title: Some("Synthetic FLEX flight report".into()),
        authors: vec![],
        year: Some(2015),
        ntrs_id: None,
        doi: None,
        canonical_url: None,
        investigation_ids: vec!["PSI-69".into()],
        families: vec!["flex".into()],
        file_name: Some("synthetic.pdf".into()),
        n_pages: 1,
        rights_status: "verified".into(),
        license: None,
        describes_runs: vec![],
        outcome_bearing: true,
        ingested_at: "2026-11-15T00:00:00Z".into(),
        parser_version: "test".into(),
    };
    let view = records::parse_view_id(&publ.id, &publ.parser_version);
    let mut page = String::new();
    let mut chunks = Vec::new();
    for (i, (text, runs, ob)) in PASSAGES.iter().enumerate() {
        let a = page.chars().count();
        page.push_str(text);
        let b = page.chars().count();
        page.push('\n');
        chunks.push(records::Chunk {
            id: chunk_id(i),
            publication_id: publ.id.clone(),
            ordinal: i as i64,
            kind: "text".into(),
            page: 1,
            char_start: Some(a as i64),
            char_end: Some(b as i64),
            section: None,
            text: (*text).into(),
            context: json!({}),
            locator: SourceLocator {
                asset_sha256: PUB_SHA.into(),
                path: Some("synthetic.pdf".into()),
                page: Some(1),
                char_start: Some(a as i64),
                char_end: Some(b as i64),
                parse_view_id: Some(view.clone()),
                offset_unit: Some(OffsetUnit::UnicodeScalar),
                ..Default::default()
            },
            describes_runs: runs.iter().map(|r| r.to_string()).collect(),
            continues_runs: vec![],
            outcome_bearing: *ob,
            evidence_class: "source_derived".into(),
        });
    }
    catalog::upsert_parse_view(con, &publ, &chunks, &[page], None).unwrap();
}

fn o2(r: &ignis_core::contracts::ConditionsRow) -> Option<f64> {
    r.cells.iter().find_map(|c| match c {
        ConditionCell::Value {
            key: ConditionKey::OxygenMoleFraction,
            value,
        } if !value
            .flags
            .iter()
            .any(|f| ignis_data::conditions::NOT_ATMOSPHERE.contains(&f.as_str())) =>
        {
            value.value_si
        }
        _ => None,
    })
}

struct Fx {
    _dir: PathBuf,
    db: PathBuf,
    svc: EvidenceService,
    table: ConditionsTable,
    report: Value,
    /// A PSI-23 run with O2 ≥ 30 % that the frozen split puts in the test partition.
    sealed_run: RunId,
}

/// The catalog, the conditions table of the fixture tables, the graph built from them, and the
/// service with a lockbox: FLEX R1 train, R2 test (FLEX-003 names no known run), one s-Flame run test.
fn setup(name: &str) -> Fx {
    let _g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let dir = std::env::temp_dir().join(format!("ignis-graph-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let data = data_root(&dir);
    std::env::set_var("IGNIS_DATA", &data);
    ignis_data::store::save(&ignis_data::families::flex::ingest().unwrap()).unwrap();
    ignis_data::store::save(&ignis_data::families::bass::ingest().unwrap()).unwrap();
    let specs: Vec<_> = ignis_data::conditions::SPECS
        .iter()
        .copied()
        .filter(|s| SPEC_INVS.contains(&s.investigation))
        .collect();
    let table = ignis_data::conditions::build_from(&["flex", "bass"], &specs).unwrap();
    let sealed_run = table
        .rows
        .iter()
        .filter(|r| r.investigation_id == "PSI-69")
        .find(|r| o2(r).is_some_and(|x| x >= 0.3))
        .and_then(|r| r.run.run_id.clone())
        .expect("a FLEX run at O2 >= 30 %");

    let db = dir.join("evidence.sqlite");
    let mut con = catalog::connect(&db, false).unwrap();
    add_publication(&mut con);
    drop(con);

    let split = |runs: &[(&str, &str)], train: &str, test: &str| Split {
        id: "split".into(),
        cohort_id: "cohort".into(),
        group_key: "session".into(),
        rule: "fixture".into(),
        seed: 0,
        run_to_group: runs
            .iter()
            .map(|(r, g)| (r.to_string(), g.to_string()))
            .collect(),
        train: vec![train.into()],
        validation: vec![],
        test: vec![test.into()],
        frozen_at: "2026-11-14T00:00:00Z".into(),
    };
    let aliases = HashMap::from([
        ("alias:flex:FLEX-001".to_string(), "PSI-69:R1".to_string()),
        ("alias:flex:FLEX-002".to_string(), "PSI-69:R2".to_string()),
    ]);
    let lockbox = Lockbox::new(
        &[
            (
                split(&[("PSI-69:R1", "g1"), ("PSI-69:R2", "g2")], "g1", "g2"),
                Family::Flex,
            ),
            (
                split(&[(sealed_run.as_str(), "t")], "none", "t"),
                Family::Flex,
            ),
        ],
        aliases.clone(),
    );

    let ro = catalog::connect(&db, true).unwrap();
    let report = graph::build(
        &graph::path_for(&db),
        &graph::Inputs {
            catalog: &ro,
            manifests: Some(&data.join("manifests")),
            conditions: Some(&table),
            labels: &[],
            aliases: &aliases,
            origin: Origin::SyntheticFixture,
        },
    )
    .unwrap();
    drop(ro);
    let cfg = dense::RetrievalCfg::load(&repo().join("configs/evidence/retrieval.toml")).unwrap();
    let mut svc = EvidenceService::open(&db, cfg, lockbox).unwrap().offline();
    svc.ledger = false;
    Fx {
        _dir: dir,
        db,
        svc,
        table,
        report,
        sealed_run,
    }
}

fn node_concept(kind: ConceptKind, id: &str) -> GraphNode {
    GraphNode::Concept(ConceptRef {
        concept: kind,
        id: ConceptId::new(id).unwrap(),
    })
}

fn req(
    node: GraphNode,
    relations: &[Relation],
    direction: DirectionFilter,
) -> GraphNeighborsRequest {
    GraphNeighborsRequest {
        schema_version: EvidenceSchema::V2,
        node,
        relations: relations.to_vec(),
        direction,
        corpus_snapshot: None,
        value_min_si: None,
        value_max_si: None,
        page_size: None,
        cursor: None,
    }
}

/// Every page of a request, concatenated (checks that cursors continue without repeats).
fn all_pages(
    svc: &EvidenceService,
    s: &ExecutionScope,
    mut r: GraphNeighborsRequest,
) -> Vec<ProvenanceLink> {
    let mut out = Vec::new();
    let mut ids = BTreeSet::new();
    loop {
        let p: GraphNeighborsPage = svc.graph_neighbors(s, &r).unwrap();
        p.validate().unwrap();
        for n in p.neighbors {
            assert!(
                ids.insert(n.link.link_id.clone()),
                "a link repeats across pages"
            );
            out.push(n.link);
        }
        match p.next_cursor {
            Some(c) => r.cursor = Some(c),
            None => return out,
        }
    }
}

fn investigation_of(l: &ProvenanceLink) -> Option<&str> {
    match &l.from {
        GraphNode::Concept(c) if c.concept == ConceptKind::Investigation => Some(c.id.as_str()),
        _ => None,
    }
}

fn run_of(n: &GraphNode) -> Option<&RunId> {
    match n {
        GraphNode::Entity(EntityRef::Run(r)) => Some(r),
        _ => None,
    }
}

#[test]
fn investigations_sharing_mdca_come_from_nasa_metadata_with_locators() {
    let fx = setup("mdca");
    let r = req(
        node_concept(ConceptKind::Hardware, "hw:mdca"),
        &[Relation::UsesHardware],
        DirectionFilter::Incoming,
    );
    let page = fx.svc.graph_neighbors(&research(), &r).unwrap();
    let invs: BTreeSet<&str> = page
        .neighbors
        .iter()
        .filter_map(|n| investigation_of(&n.link))
        .collect();
    assert_eq!(invs, BTreeSet::from(["PSI-39", "PSI-68", "PSI-69"]));
    assert_eq!(page.total_count, Some(3));
    for n in &page.neighbors {
        assert_eq!(n.direction, EdgeDirection::Incoming);
        let l = &n.link;
        assert_eq!(l.resolution, Resolution::Resolved);
        assert_eq!(l.method.method_id, "graph.psi_metadata");
        // the original value: NASA's words and the metadata record they were read from
        let v = l.value.as_ref().expect("a metadata value");
        let Some(RawValue::Text(t)) = &v.raw_value else {
            panic!("hardware is text")
        };
        assert!(t.contains("(MDCA)"), "{t}");
        let loc = v.locator.as_ref().unwrap();
        assert_eq!(loc.path.as_deref(), Some("hardware"));
        let inv = investigation_of(l).unwrap();
        let meta: Value = serde_json::from_str(
            &std::fs::read_to_string(
                repo().join(format!("data/manifests/{inv}.investigation.json")),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(
            meta["metadata_url_sha256"].as_str(),
            Some(loc.asset_sha256.as_str())
        );
        assert!(meta["hardware"].as_str().unwrap().contains(t.as_str()));
    }

    // A bare "Microgravity Science Glovebox" (BASS) resolves to the MSG another investigation
    // spells out; a phrase that names no hardware is reported, not linked.
    let msg = fx
        .svc
        .graph_neighbors(
            &research(),
            &req(
                node_concept(ConceptKind::Hardware, "hw:msg"),
                &[],
                DirectionFilter::Incoming,
            ),
        )
        .unwrap();
    let msg: BTreeSet<&str> = msg
        .neighbors
        .iter()
        .filter_map(|n| investigation_of(&n.link))
        .collect();
    assert!(
        msg.is_superset(&BTreeSet::from(["PSI-25", "PSI-26", "PSI-107"])),
        "{msg:?}"
    );
    let not_names = fx.report["manifests"]["hardware_phrases_not_names"].to_string();
    assert!(
        not_names.contains("Refer to final report") && not_names.contains("PSI-101"),
        "{not_names}"
    );
    assert_eq!(
        fx.report["manifests"]["investigations_without_hardware"],
        json!(["PSI-106"])
    );
}

#[test]
fn runs_with_o2_at_least_30_percent_match_the_conditions_table_cell_by_cell() {
    let fx = setup("o2");
    let eval = evaluation();
    let mut r = req(
        node_concept(ConceptKind::Condition, "cond:oxygen_mole_fraction"),
        &[Relation::HasCondition],
        DirectionFilter::Incoming,
    );
    r.value_min_si = Some(Finite::new(0.3).unwrap());
    r.page_size = Some(2);
    let links = all_pages(&fx.svc, &eval, r.clone());
    assert!(!links.is_empty());

    // The same answer as conditions_query with the filter O2 >= 0.3, run by run.
    let q = ConditionsQuery {
        schema_version: DatasetSchema::V2,
        investigation_ids: vec![],
        run_ids: vec![],
        keys: vec![],
        filters: vec![ConditionFilter {
            key: ConditionKey::OxygenMoleFraction,
            min_si: Some(Finite::new(0.3).unwrap()),
            max_si: None,
            any_of: Default::default(),
        }],
        include_unknown: false,
        include_envelopes: false,
        page_size: None,
        cursor: None,
    };
    let rows = ignis_data::conditions::query(&fx.table, &q, |_| true).unwrap();
    let want: BTreeSet<&RunId> = rows
        .rows
        .iter()
        .filter_map(|r| r.run.run_id.as_ref())
        .collect();
    let got: BTreeSet<&RunId> = links.iter().filter_map(|l| run_of(&l.from)).collect();
    assert_eq!(got, want);
    assert!(!got.is_empty(), "FLEX runs at O2 >= 30 %");
    // Test points without a run identity (s-Flame test days) answer at the investigation level,
    // one link per matching row of conditions_query.
    let points: Vec<_> = links.iter().filter(|l| run_of(&l.from).is_none()).collect();
    assert_eq!(
        points.len(),
        rows.rows.iter().filter(|r| r.run.run_id.is_none()).count()
    );
    assert!(points
        .iter()
        .all(|l| investigation_of(l) == Some("PSI-23") && l.method.notes.is_some()));

    // Each link carries the cell it was read from: raw value, unit, SI value and the cell locator.
    for l in &links {
        let v = l.value.as_ref().expect("a cell");
        assert!(v.value_si.unwrap() >= 0.3);
        assert!(matches!(v.raw_value, Some(RawValue::Number(_))));
        let loc = v.locator.as_ref().unwrap();
        assert!(loc.row.is_some() && loc.column.is_some(), "{loc:?}");
        assert!(loc.path.as_deref().unwrap().contains("Experimental table"));
        // the cell is the table's own cell, and the link names the table, row and key
        assert!(fx.table.rows.iter().any(|row| row
            .cells
            .iter()
            .any(|c| matches!(c, ConditionCell::Value { value, .. } if value == v))));
        let at = match run_of(&l.from) {
            Some(run) => format!("run={run}&key=oxygen_mole_fraction"),
            None => format!("&row={}&key=oxygen_mole_fraction", loc.row.unwrap()),
        };
        assert!(
            l.method.inputs.iter().any(|i| i.contains(&at)),
            "{:?}",
            l.method.inputs
        );
    }

    // Runs whose O2 is not one stated value are counted, never matched.
    let p = fx.svc.graph_neighbors(&eval, &r).unwrap();
    assert!(p.without_value.unwrap() > 0);
    let gaps = all_pages(
        &fx.svc,
        &eval,
        req(
            node_concept(ConceptKind::Condition, "cond:oxygen_mole_fraction"),
            &[Relation::HasCondition],
            DirectionFilter::Incoming,
        ),
    );
    let unknown: Vec<_> = gaps.iter().filter(|l| l.value.is_none()).collect();
    assert_eq!(unknown.len() as u64, p.without_value.unwrap());
    assert!(unknown.iter().all(|l| matches!(
        l.resolution,
        Resolution::Unknown | Resolution::NotApplicable
    ) && l.method.notes.is_some()));
    assert!(unknown.iter().any(|l| l
        .method
        .notes
        .as_deref()
        .unwrap()
        .contains("several tested values")));

    // Second hop: the runs' investigations, from the same table rows.
    let run = links.iter().find_map(|l| run_of(&l.from)).unwrap().clone();
    let up = fx
        .svc
        .graph_neighbors(
            &eval,
            &req(
                GraphNode::Entity(EntityRef::Run(run.clone())),
                &[Relation::PartOf],
                DirectionFilter::Outgoing,
            ),
        )
        .unwrap();
    assert_eq!(up.neighbors.len(), 1);
    assert_eq!(
        up.neighbors[0].link.to,
        node_concept(
            ConceptKind::Investigation,
            run.as_str().split(':').next().unwrap()
        )
    );

    // A cursor is bound to its request.
    let first = fx.svc.graph_neighbors(&eval, &r).unwrap();
    let mut other = r.clone();
    other.value_min_si = Some(Finite::new(0.25).unwrap());
    other.cursor = first.next_cursor.clone();
    if other.cursor.is_some() {
        let e = fx.svc.graph_neighbors(&eval, &other).unwrap_err();
        assert_eq!(e.code, ErrorCode::InvalidRequest);
    }
}

#[test]
fn every_edge_says_where_it_came_from() {
    let fx = setup("provenance");
    let s = evaluation();
    let source = GraphNode::Entity(EntityRef::Source(
        SourceId::new(format!("sha256:{PUB_SHA}")).unwrap(),
    ));

    // Passages of the active view are part of their source, with the chunk as evidence.
    let parts = all_pages(
        &fx.svc,
        &s,
        req(
            source.clone(),
            &[Relation::PartOf],
            DirectionFilter::Incoming,
        ),
    );
    assert_eq!(parts.len(), PASSAGES.len());
    for l in &parts {
        let GraphNode::Entity(EntityRef::Evidence(e)) = &l.from else {
            panic!("a passage")
        };
        assert_eq!(l.evidence, vec![e.clone()]);
        fx.svc.resolve_evidence(&s, e).unwrap();
    }
    // ... and the publication describes its investigation (catalog seed).
    let out = fx
        .svc
        .graph_neighbors(&s, &req(source, &[], DirectionFilter::Outgoing))
        .unwrap();
    assert_eq!(out.neighbors.len(), 1);
    assert_eq!(out.neighbors[0].link.relation, Relation::Describes);
    assert_eq!(
        out.neighbors[0].link.to,
        node_concept(ConceptKind::Investigation, "PSI-69")
    );

    // A run named in a passage (directly or through a report alias) is linked with the passage as
    // evidence; a name that resolves to no run is reported, not linked.
    let r2 = GraphNode::Entity(EntityRef::Run(RunId::new("PSI-69:R2").unwrap()));
    let about = fx
        .svc
        .graph_neighbors(
            &s,
            &req(r2, &[Relation::DescribesRun], DirectionFilter::Incoming),
        )
        .unwrap();
    assert_eq!(about.neighbors.len(), 1);
    let l = &about.neighbors[0].link;
    assert_eq!(l.evidence, vec![EvidenceId::new(chunk_id(2)).unwrap()]);
    assert_eq!(l.method.inputs, vec!["alias:flex:FLEX-002".to_string()]);
    assert_eq!(
        fx.report["catalog"]["unresolved_run_names"],
        json!({"alias:flex:FLEX-003": 1})
    );

    // Table rows: the run is part of its investigation and described by the table it is a row of.
    let row = fx
        .table
        .rows
        .iter()
        .find(|r| r.investigation_id == "PSI-98")
        .unwrap();
    let run = GraphNode::Entity(EntityRef::Run(row.run.run_id.clone().unwrap()));
    let links = all_pages(&fx.svc, &s, req(run, &[], DirectionFilter::Both));
    assert_eq!(
        links.len(),
        2 + row.cells.len(),
        "part_of, describes_run, one link per cell"
    );
    let table_src = links
        .iter()
        .find(|l| l.relation == Relation::DescribesRun)
        .unwrap();
    let csv: Vec<Value> = std::fs::read_to_string(repo().join("data/manifests/PSI-98.jsonl"))
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .filter(|v: &Value| {
            v["file_name"]
                .as_str()
                .is_some_and(|f| f.contains("Experimental table"))
        })
        .collect();
    assert_eq!(
        table_src.from,
        GraphNode::Entity(EntityRef::Source(
            SourceId::new(csv[0]["id"].as_str().unwrap()).unwrap()
        ))
    );
    assert!(table_src
        .method
        .inputs
        .iter()
        .any(|i| i.starts_with("locator:") && i.contains("\"row\"")));
    // Gravity is stated by no table: an explicit unknown with the reason, never a value.
    let g = links
        .iter()
        .find(|l| l.to == node_concept(ConceptKind::Condition, "cond:gravity_level"))
        .unwrap();
    assert_eq!(g.resolution, Resolution::Unknown);
    assert!(g.value.is_none() && g.method.notes.is_some());
    // The fuel is NASA's text as a fuel node, with its cell.
    let fuel = links.iter().find(|l| l.relation == Relation::UsesFuel);
    if let Some(f) = fuel {
        assert!(f
            .value
            .as_ref()
            .unwrap()
            .locator
            .as_ref()
            .unwrap()
            .column
            .is_some());
    }

    // The table's file is part of its investigation's archive (manifest line).
    let csv_src = GraphNode::Entity(EntityRef::Source(
        SourceId::new(csv[0]["id"].as_str().unwrap()).unwrap(),
    ));
    let arch = fx
        .svc
        .graph_neighbors(
            &s,
            &req(csv_src, &[Relation::PartOf], DirectionFilter::Outgoing),
        )
        .unwrap();
    assert_eq!(
        arch.neighbors[0].link.method.inputs,
        vec!["manifest:PSI-98.jsonl:1".to_string()]
    );
    assert_eq!(arch.neighbors[0].link.origin, Origin::SyntheticFixture);
    assert_eq!(
        fx.report["labels"]["status"],
        json!("absent: no R8 labels have been run")
    );
}

#[test]
fn the_lockbox_keeps_sealed_runs_and_their_passages_from_a_research_scope() {
    let fx = setup("lockbox");
    let res = research();
    let eval = evaluation();
    let o2 = || {
        let mut r = req(
            node_concept(ConceptKind::Condition, "cond:oxygen_mole_fraction"),
            &[Relation::HasCondition],
            DirectionFilter::Incoming,
        );
        r.value_min_si = Some(Finite::new(0.3).unwrap());
        r
    };
    let seen = |s: &ExecutionScope| -> BTreeSet<RunId> {
        all_pages(&fx.svc, s, o2())
            .iter()
            .filter_map(|l| run_of(&l.from).cloned())
            .collect()
    };
    assert!(seen(&eval).contains(&fx.sealed_run));
    assert!(!seen(&res).contains(&fx.sealed_run));
    // Counts are taken under the lockbox too.
    let p = fx.svc.graph_neighbors(&res, &o2()).unwrap();
    assert!(p
        .total_count
        .is_none_or(|t| t == all_pages(&fx.svc, &res, o2()).len() as u64));
    let all = |s: &ExecutionScope| {
        all_pages(
            &fx.svc,
            s,
            req(
                node_concept(ConceptKind::Condition, "cond:oxygen_mole_fraction"),
                &[],
                DirectionFilter::Incoming,
            ),
        )
        .len()
    };
    assert_eq!(
        all(&eval) - all(&res),
        1,
        "exactly the sealed run's O2 link is hidden"
    );

    // The sealed run itself has no visible neighbours; its investigation does not list it.
    let sealed = GraphNode::Entity(EntityRef::Run(fx.sealed_run.clone()));
    let p = fx
        .svc
        .graph_neighbors(&res, &req(sealed.clone(), &[], DirectionFilter::Both))
        .unwrap();
    assert!(p.neighbors.is_empty() && p.next_cursor.is_none());
    assert!(!fx
        .svc
        .graph_neighbors(&eval, &req(sealed, &[], DirectionFilter::Both))
        .unwrap()
        .neighbors
        .is_empty());
    let members = all_pages(
        &fx.svc,
        &res,
        req(
            node_concept(ConceptKind::Investigation, "PSI-23"),
            &[Relation::PartOf],
            DirectionFilter::Incoming,
        ),
    );
    assert!(members
        .iter()
        .all(|l| run_of(&l.from) != Some(&fx.sealed_run)));

    // Passages: the outcome of test run R2 and an unattributable outcome stay withheld; methods
    // and the train run's passage stay readable.
    let source = GraphNode::Entity(EntityRef::Source(
        SourceId::new(format!("sha256:{PUB_SHA}")).unwrap(),
    ));
    let visible: BTreeSet<String> = all_pages(
        &fx.svc,
        &res,
        req(source, &[Relation::PartOf], DirectionFilter::Incoming),
    )
    .iter()
    .flat_map(|l| l.evidence.iter().map(|e| e.as_str().to_string()))
    .collect();
    assert_eq!(
        visible,
        BTreeSet::from([chunk_id(0), chunk_id(1), chunk_id(4)])
    );
    let r2 = GraphNode::Entity(EntityRef::Run(RunId::new("PSI-69:R2").unwrap()));
    assert!(fx
        .svc
        .graph_neighbors(&res, &req(r2, &[], DirectionFilter::Both))
        .unwrap()
        .neighbors
        .is_empty());
    let passage = GraphNode::Entity(EntityRef::Evidence(EvidenceId::new(chunk_id(2)).unwrap()));
    assert!(fx
        .svc
        .graph_neighbors(&res, &req(passage, &[], DirectionFilter::Both))
        .unwrap()
        .neighbors
        .is_empty());

    // The scope comes from the host: a request cannot carry one (unknown fields are refused).
    let forged = json!({"schema_version": "evidence.v2", "node": {"kind": "run", "id": fx.sealed_run.as_str()},
        "scope": {"partitions": ["test"]}});
    assert!(serde_json::from_value::<GraphNeighborsRequest>(forged).is_err());
}

#[test]
fn a_missing_or_stale_graph_is_refused_not_guessed() {
    let fx = setup("stale");
    let r = req(
        node_concept(ConceptKind::Hardware, "hw:mdca"),
        &[],
        DirectionFilter::Incoming,
    );
    // stale: the catalog changed after the build
    let con = catalog::connect(&fx.db, false).unwrap();
    con.execute(
        "UPDATE chunks SET text = text || ' ' WHERE id = ?1",
        [chunk_id(0)],
    )
    .unwrap();
    drop(con);
    let cfg = dense::RetrievalCfg::load(&repo().join("configs/evidence/retrieval.toml")).unwrap();
    let svc = EvidenceService::open(&fx.db, cfg.clone(), Lockbox::default())
        .unwrap()
        .offline();
    assert_eq!(
        svc.graph_neighbors(&research(), &r).unwrap_err().code,
        ErrorCode::Conflict
    );
    // missing
    std::fs::remove_file(graph::path_for(&fx.db)).unwrap();
    let svc = EvidenceService::open(&fx.db, cfg, Lockbox::default())
        .unwrap()
        .offline();
    assert_eq!(
        svc.graph_neighbors(&research(), &r).unwrap_err().code,
        ErrorCode::CapabilityUnavailable
    );
}
