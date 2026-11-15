//! R9b: the combustion graph. `ProvenanceLink` edges in an indexed SQLite file next to the catalog
//! (`graph.sqlite`), built by `ignis evidence graph build` and read by
//! [`KnowledgeService::graph_neighbors`](ignis_core::services::KnowledgeService::graph_neighbors).
//! The graph is for joins and filters ("investigations sharing MDCA", "runs with O2 ≥ 30 %"), not
//! community summaries.
//!
//! **Edges come only from data that exists** and each one says where it came from:
//!
//! | relation | from → to | read from | provenance on the link |
//! |---|---|---|---|
//! | `part_of` | source → investigation | PSI manifests (`data/manifests/PSI-*.jsonl`) | `method.inputs` = manifest file and line |
//! | `uses_hardware` | investigation → hardware | PSI investigation metadata (`hardware` field) | `value` = the field's text and locator |
//! | `part_of` | run → investigation | conditions table (R9a) | `method.inputs` = table digest, run and the row's locator |
//! | `describes_run` | experimental table source → run | conditions table | the row's locator |
//! | `has_condition` / `uses_fuel` / `measured_by` / `uses_hardware` | run → condition / fuel / diagnostic / hardware | conditions table cells | `value` = the cell (raw value, raw unit, SI value, locator) |
//! | same relations | investigation → condition / fuel / … | cells of a test point without a run identity (campaign or test-day rows) | `value` = the cell; `method.inputs` = table, row and key |
//! | `describes` | source → investigation | evidence catalog publications | `method.inputs` = publication id |
//! | `part_of` | passage / table / caption → source | evidence catalog, active parse view | `evidence` = the chunk |
//! | `describes_run` | passage → run | run identifiers matched in the chunk (`describes_runs`) | `evidence` = the chunk |
//! | `describes` / `describes_run` | source → investigation / run | R8 `SourceLabel`s, when present | `evidence` = the label's passages; runs stay `unknown` |
//!
//! **Gaps are links too.** A missing, not-applicable or conflicting cell becomes a link with that
//! resolution (`unknown`, `not_applicable`, `unresolved_conflict`, one link per candidate) and the
//! reason in `method.notes`; numeric keys always point at the key's condition node, so "what is
//! the O2 of this run" answers even when the table does not say. What cannot become a link (a
//! test point without a run, a run name that resolves to no run, a hardware phrase that names no
//! hardware) is counted in the build report, never dropped silently.
//!
//! **Node ids.** Investigations are their accession (`PSI-69`); conditions `cond:<key>` (an O2
//! that belongs to a burner stream, not the atmosphere: `cond:oxygen_mole_fraction:oxidizer_stream:burner`);
//! fuels `fuel:<slug of NASA's text>`; hardware `hw:<acronym>` when NASA's text gives one
//! (`Multi-user Droplet Combustion Apparatus (MDCA)` → `hw:mdca`), else `hw:<slug>`; diagnostics
//! `diag:<slug>`. Passages, tables and captions are `EntityRef::Evidence` (the chunk id).
//!
//! **Lockbox.** The file holds every link; the service filters at read time with the scope's
//! sealed partitions: a link touching a run of a sealed partition, or a passage (endpoint or
//! evidence) the lockbox withholds, is not returned and not counted.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;

use anyhow::{Context, Result};
use ignis_core::contracts::{
    ConceptId, ConceptKind, ConceptRef, ConditionCell, ConditionKey, ConditionsTable, Derivation,
    DirectionFilter, EntityRef, EvidenceClass, EvidenceId, EvidenceSchema, GraphNode, LinkId,
    Measured, Method, Origin, ProvenanceLink, RawValue, Relation, Resolution, RunId, SourceId,
    SourceLabel, SourceLocator,
};
use ignis_core::hashing::{canonical_json, sha256_hex};
use regex::Regex;
use rusqlite::{params, params_from_iter, Connection, OpenFlags};
use serde_json::{json, Value};
use unicode_normalization::UnicodeNormalization;

use crate::catalog;
use crate::corpus;
use crate::records::{locator_json, parse_view_id};

/// The graph file next to the catalog.
pub const GRAPH_FILE: &str = "graph.sqlite";
/// Version of the edge rules (part of every link's method).
pub const GRAPH_VERSION: &str = "1";

const DDL: &str = "
CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
-- seq is the read order (deterministic: the build walks its inputs in sorted order)
CREATE TABLE links (
  seq INTEGER PRIMARY KEY,
  link_id TEXT NOT NULL UNIQUE,
  from_key TEXT NOT NULL,
  to_key TEXT NOT NULL,
  relation TEXT NOT NULL,
  value_si REAL,
  link_json TEXT NOT NULL
);
CREATE INDEX links_from ON links(from_key, seq);
CREATE INDEX links_to ON links(to_key, seq);
";

/// Default path of the graph for a catalog.
pub fn path_for(catalog_db: &Path) -> std::path::PathBuf {
    catalog_db.with_file_name(GRAPH_FILE)
}

/// Canonical key of a node (the indexed columns): `e:<kind>:<id>` or `c:<concept>:<id>`.
pub fn node_key(n: &GraphNode) -> String {
    let v = serde_json::to_value(n).expect("node serializes");
    match n {
        GraphNode::Entity(_) => format!(
            "e:{}:{}",
            v["kind"].as_str().unwrap_or_default(),
            v["id"].as_str().unwrap_or_default()
        ),
        GraphNode::Concept(_) => format!(
            "c:{}:{}",
            v["concept"].as_str().unwrap_or_default(),
            v["id"].as_str().unwrap_or_default()
        ),
    }
}

/// Wire name of a relation (`has_condition`).
pub fn relation_name(r: Relation) -> String {
    serde_json::to_value(r)
        .ok()
        .and_then(|v| v.as_str().map(String::from))
        .expect("relation serializes as a string")
}

fn key_name(k: ConditionKey) -> String {
    serde_json::to_value(k)
        .ok()
        .and_then(|v| v.as_str().map(String::from))
        .expect("condition key serializes as a string")
}

/// Lower-case ASCII-ish slug of NASA's text: NFKC, letters and digits kept, every other run of
/// characters one `-`. Never empty for a text with a letter or digit.
pub fn slug(text: &str) -> String {
    let mut out = String::new();
    let mut dash = false;
    for c in text.nfkc().flat_map(char::to_lowercase) {
        if c.is_alphanumeric() {
            if dash && !out.is_empty() {
                out.push('-');
            }
            dash = false;
            out.push(c);
        } else {
            dash = true;
        }
    }
    out
}

fn concept(kind: ConceptKind, id: impl Into<String>) -> Result<GraphNode> {
    Ok(GraphNode::Concept(ConceptRef {
        concept: kind,
        id: ConceptId::new(id)?,
    }))
}

fn investigation(id: &str) -> Result<GraphNode> {
    concept(ConceptKind::Investigation, id)
}

fn run_node(id: &RunId) -> GraphNode {
    GraphNode::Entity(EntityRef::Run(id.clone()))
}

fn source_node(sha_or_id: &str) -> Result<GraphNode> {
    let id = if sha_or_id.starts_with("sha256:") {
        sha_or_id.to_string()
    } else {
        format!("sha256:{sha_or_id}")
    };
    Ok(GraphNode::Entity(EntityRef::Source(SourceId::new(id)?)))
}

fn det(method_id: &str, inputs: Vec<String>, notes: Option<String>) -> Derivation {
    Derivation {
        method: Method::Deterministic,
        method_id: method_id.into(),
        method_version: GRAPH_VERSION.into(),
        inputs,
        agent: None,
        notes,
    }
}

// ------------------------------------------------------------------ hardware names

/// Hardware phrases of NASA's metadata that name no hardware (counted in the report, no link).
const NOT_A_NAME: [&str; 1] = ["refer to final report"];

/// `Full Name (ACR)` pairs, learned from every hardware text, so a bare `Microgravity Science
/// Glovebox` resolves to the `MSG` another investigation spells out. A full name seen with two
/// acronyms is ambiguous and not used.
#[derive(Debug, Default)]
pub struct HardwareNames {
    full_to_acr: HashMap<String, Option<String>>,
    acronyms: BTreeSet<String>,
}

fn norm_words(s: &str) -> String {
    s.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim_matches(|c: char| !c.is_alphanumeric())
        .to_lowercase()
}

fn acr_re() -> Regex {
    Regex::new(r"([^()]*?)\s*\(([A-Za-z][A-Za-z0-9-]*)\)").expect("regex")
}

fn segments(text: &str) -> impl Iterator<Item = &str> {
    text.split([',', ';'])
        .map(str::trim)
        .filter(|s| !s.is_empty())
}

impl HardwareNames {
    pub fn learn<'a>(texts: impl IntoIterator<Item = &'a str>) -> HardwareNames {
        let re = acr_re();
        let mut h = HardwareNames::default();
        for t in texts {
            for seg in segments(t) {
                for c in re.captures_iter(seg) {
                    let acr = c[2].to_lowercase();
                    h.acronyms.insert(acr.clone());
                    let full = norm_words(&c[1]);
                    if full.is_empty() {
                        continue;
                    }
                    let e = h.full_to_acr.entry(full).or_insert(Some(acr.clone()));
                    if e.as_ref().is_some_and(|a| *a != acr) {
                        *e = None;
                    }
                }
            }
        }
        h
    }

    /// The hardware ids one text names, each with the segment that named it; `Err(segment)` for a
    /// phrase that names no hardware.
    pub fn resolve<'a>(&self, text: &'a str) -> Vec<Result<(String, &'a str), &'a str>> {
        let re = acr_re();
        let mut out = Vec::new();
        for seg in segments(text) {
            let acrs: Vec<String> = re.captures_iter(seg).map(|c| c[2].to_lowercase()).collect();
            if !acrs.is_empty() {
                out.extend(acrs.into_iter().map(|a| Ok((format!("hw:{a}"), seg))));
                continue;
            }
            let n = norm_words(seg);
            if NOT_A_NAME.contains(&n.as_str()) || slug(seg).is_empty() {
                out.push(Err(seg));
            } else if let Some(Some(a)) = self.full_to_acr.get(&n) {
                out.push(Ok((format!("hw:{a}"), seg)));
            } else if self.acronyms.contains(&n) {
                out.push(Ok((format!("hw:{n}"), seg)));
            } else {
                out.push(Ok((format!("hw:{}", slug(seg)), seg)));
            }
        }
        out
    }
}

// ------------------------------------------------------------------ build

/// What the build reads. Every input is optional except the catalog (the graph is bound to its
/// corpus snapshot); an absent input is reported, not guessed.
pub struct Inputs<'a> {
    /// The evidence catalog (read-only).
    pub catalog: &'a Connection,
    /// `data/manifests` (PSI investigation metadata and source manifests).
    pub manifests: Option<&'a Path>,
    /// The R9a conditions table.
    pub conditions: Option<&'a ConditionsTable>,
    /// R8 labels (none run yet: empty).
    pub labels: &'a [SourceLabel],
    /// Report aliases (`alias:flex:FLEX-020`) → run ids, from the ingested records.
    pub aliases: &'a HashMap<String, String>,
    /// `synthetic_fixture` in tests.
    pub origin: Origin,
}

struct Writer<'c> {
    tx: rusqlite::Transaction<'c>,
    origin: Origin,
    by_relation: BTreeMap<String, u64>,
    by_method: BTreeMap<String, u64>,
    duplicates: u64,
}

impl Writer<'_> {
    #[allow(clippy::too_many_arguments)]
    fn add(
        &mut self,
        from: GraphNode,
        to: GraphNode,
        relation: Relation,
        evidence: Vec<EvidenceId>,
        resolution: Resolution,
        method: Derivation,
        value: Option<Measured>,
    ) -> Result<()> {
        let identity = json!({
            "from": from, "to": to, "relation": relation, "method_id": method.method_id,
            "inputs": method.inputs, "evidence": evidence,
            "value_locator": value.as_ref().and_then(|v| v.locator.as_ref()).map(locator_json),
        });
        let link_id = format!(
            "lnk-{}",
            &sha256_hex(canonical_json(&identity).as_bytes())[..24]
        );
        let link = ProvenanceLink {
            schema_version: EvidenceSchema::V2,
            link_id: LinkId::new(link_id)?,
            from,
            to,
            relation,
            evidence,
            resolution,
            method,
            value,
            origin: self.origin,
        };
        link.validate()
            .map_err(|e| anyhow::anyhow!("{e} ({})", serde_json::to_string(&link).unwrap()))?;
        let n = self.tx.execute(
            "INSERT OR IGNORE INTO links(link_id, from_key, to_key, relation, value_si, link_json) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                link.link_id.as_str(),
                node_key(&link.from),
                node_key(&link.to),
                relation_name(link.relation),
                link.value.as_ref().and_then(|v| v.value_si),
                serde_json::to_string(&link)?
            ],
        )?;
        if n == 0 {
            self.duplicates += 1;
            return Ok(());
        }
        *self
            .by_relation
            .entry(relation_name(link.relation))
            .or_default() += 1;
        *self
            .by_method
            .entry(link.method.method_id.clone())
            .or_default() += 1;
        Ok(())
    }
}

/// Build the graph at `out` (written to a temporary file, then moved into place). Returns the
/// build report: links per relation and method, and every gap that could not become a link.
pub fn build(out: &Path, inp: &Inputs) -> Result<Value> {
    let snapshot = corpus::snapshot(inp.catalog)?;
    let tmp = out.with_extension("sqlite.tmp");
    let _ = std::fs::remove_file(&tmp);
    if let Some(d) = out.parent() {
        std::fs::create_dir_all(d)?;
    }
    let mut con = Connection::open(&tmp).with_context(|| tmp.display().to_string())?;
    con.execute_batch(DDL)?;
    let mut report = json!({});
    let conditions_digest = inp
        .conditions
        .map(|t| sha256_hex(canonical_json(&serde_json::to_value(t).expect("table")).as_bytes()));
    {
        let mut w = Writer {
            tx: con.transaction()?,
            origin: inp.origin,
            by_relation: BTreeMap::new(),
            by_method: BTreeMap::new(),
            duplicates: 0,
        };
        report["manifests"] = match inp.manifests {
            Some(dir) if dir.exists() => manifests(&mut w, dir)?,
            Some(dir) => json!({"status": "absent", "path": dir.display().to_string()}),
            None => json!({"status": "not_requested"}),
        };
        report["conditions"] = match (inp.conditions, &conditions_digest) {
            (Some(t), Some(d)) => conditions(&mut w, t, d)?,
            _ => json!({"status": "absent: run `ignis data conditions` to build the table"}),
        };
        report["catalog"] = catalog_links(&mut w, inp.catalog, inp.aliases)?;
        report["labels"] = labels(&mut w, inp.labels)?;
        report["links_by_relation"] = json!(w.by_relation);
        report["links_by_method"] = json!(w.by_method);
        report["duplicate_links"] = json!(w.duplicates);
        let total: u64 = w.by_relation.values().sum();
        report["links"] = json!(total);
        let meta = [
            ("graph_version", GRAPH_VERSION.to_string()),
            ("corpus_snapshot", snapshot.snapshot_id.clone()),
            (
                "conditions_digest",
                conditions_digest.clone().unwrap_or_default(),
            ),
            ("origin", serde_json::to_value(inp.origin)?.to_string()),
            ("built_at", chrono::Utc::now().to_rfc3339()),
            ("links", total.to_string()),
        ];
        for (k, v) in meta {
            w.tx.execute(
                "INSERT INTO meta(key, value) VALUES (?1, ?2)",
                params![k, v],
            )?;
        }
        w.tx.commit()?;
    }
    con.execute_batch("ANALYZE")?;
    drop(con);
    std::fs::rename(&tmp, out)?;
    report["corpus_snapshot"] = json!(snapshot.snapshot_id);
    report["path"] = json!(out.display().to_string());
    Ok(report)
}

/// PSI manifests: investigation → hardware (metadata), source → investigation (file listings).
fn manifests(w: &mut Writer, dir: &Path) -> Result<Value> {
    let mut files: Vec<_> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.is_file())
        .collect();
    files.sort();
    let name = |p: &Path| {
        p.file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default()
    };
    let metas: Vec<(String, Value)> = files
        .iter()
        .filter(|p| name(p).ends_with(".investigation.json"))
        .map(|p| -> Result<(String, Value)> {
            let v: Value = serde_json::from_str(&std::fs::read_to_string(p)?)
                .with_context(|| p.display().to_string())?;
            Ok((name(p), v))
        })
        .collect::<Result<_>>()?;
    let names = HardwareNames::learn(metas.iter().filter_map(|(_, v)| v["hardware"].as_str()));
    let mut not_names = Vec::new();
    let mut without_hardware = Vec::new();
    for (file, v) in &metas {
        let Some(inv) = v["investigation"].as_str() else {
            continue;
        };
        let text = v["hardware"].as_str().unwrap_or_default();
        if text.trim().is_empty() {
            without_hardware.push(inv.to_string());
            continue;
        }
        let sha = v["metadata_url_sha256"].as_str();
        for r in names.resolve(text) {
            match r {
                Err(seg) => not_names.push(json!({"investigation": inv, "text": seg})),
                Ok((hw, seg)) => {
                    // the metadata record is the original: its `hardware` field, quoted
                    let value = sha.map(|sha| Measured {
                        raw_value: Some(RawValue::Text(seg.to_string())),
                        raw_unit: None,
                        value_si: None,
                        unit_si: None,
                        uncertainty_si: None,
                        uncertainty_reason: None,
                        evidence_class: EvidenceClass::OriginalObservation,
                        locator: Some(SourceLocator {
                            asset_sha256: sha.to_string(),
                            path: Some("hardware".into()),
                            ..Default::default()
                        }),
                        derivation: None,
                        resolution: Resolution::Resolved,
                        flags: vec![],
                    });
                    w.add(
                        investigation(inv)?,
                        concept(ConceptKind::Hardware, hw)?,
                        Relation::UsesHardware,
                        vec![],
                        Resolution::Resolved,
                        det(
                            "graph.psi_metadata",
                            vec![format!("manifest:{file}#hardware")],
                            Some(format!("NASA PSI metadata names the hardware: {text}")),
                        ),
                        value,
                    )?;
                }
            }
        }
    }
    let mut sources = 0u64;
    let mut without_investigation = 0u64;
    for p in files.iter().filter(|p| {
        let n = name(p);
        n.starts_with("PSI-") && n.ends_with(".jsonl") && !n.ends_with(".listing.jsonl")
    }) {
        let file = name(p);
        let reader = std::io::BufReader::new(std::fs::File::open(p)?);
        for (i, line) in std::io::BufRead::lines(reader).enumerate() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let v: Value =
                serde_json::from_str(&line).with_context(|| format!("{file}:{}", i + 1))?;
            let (Some(id), Some(inv)) = (v["id"].as_str(), v["investigation_id"].as_str()) else {
                without_investigation += 1;
                continue;
            };
            w.add(
                source_node(id)?,
                investigation(inv)?,
                Relation::PartOf,
                vec![],
                Resolution::Resolved,
                det(
                    "graph.psi_manifest",
                    vec![format!("manifest:{file}:{}", i + 1)],
                    v["kind"]
                        .as_str()
                        .map(|k| format!("{k} in the investigation's PSI archive")),
                ),
                None,
            )?;
            sources += 1;
        }
    }
    Ok(json!({
        "investigations": metas.len(), "sources": sources,
        "sources_without_investigation": without_investigation,
        "investigations_without_hardware": without_hardware,
        "hardware_phrases_not_names": not_names,
    }))
}

/// Where a conditions cell points, by key.
fn cell_target(
    key: ConditionKey,
    text: Option<&str>,
    flags: &[String],
) -> Result<(Relation, GraphNode)> {
    let k = key_name(key);
    let categorical =
        |kind: ConceptKind, rel: Relation, prefix: &str| -> Result<(Relation, GraphNode)> {
            match text.map(slug).filter(|s| !s.is_empty()) {
                Some(s) => Ok((rel, concept(kind, format!("{prefix}:{s}"))?)),
                None => Ok((
                    Relation::HasCondition,
                    concept(ConceptKind::Condition, format!("cond:{k}"))?,
                )),
            }
        };
    match key {
        ConditionKey::Fuel => categorical(ConceptKind::Fuel, Relation::UsesFuel, "fuel"),
        ConditionKey::Hardware => {
            match text.and_then(|t| {
                HardwareNames::default()
                    .resolve(t)
                    .into_iter()
                    .find_map(|r| r.ok())
            }) {
                Some((hw, _)) => Ok((Relation::UsesHardware, concept(ConceptKind::Hardware, hw)?)),
                None => Ok((
                    Relation::HasCondition,
                    concept(ConceptKind::Condition, format!("cond:{k}"))?,
                )),
            }
        }
        ConditionKey::Diagnostics => {
            categorical(ConceptKind::Diagnostic, Relation::MeasuredBy, "diag")
        }
        ConditionKey::Diluent | ConditionKey::SampleGeometry => categorical(
            ConceptKind::Condition,
            Relation::HasCondition,
            &format!("cond:{k}"),
        ),
        _ => {
            // a value that is not the test atmosphere (a burner stream's O2) is its own condition
            let other: Vec<&str> = flags
                .iter()
                .map(String::as_str)
                .filter(|f| ignis_data::conditions::NOT_ATMOSPHERE.contains(f))
                .collect();
            let id = if other.is_empty() {
                format!("cond:{k}")
            } else {
                format!("cond:{k}:{}", other.join(":"))
            };
            Ok((Relation::HasCondition, concept(ConceptKind::Condition, id)?))
        }
    }
}

fn text_of(m: &Measured) -> Option<&str> {
    match &m.raw_value {
        Some(RawValue::Text(t)) => Some(t),
        _ => None,
    }
}

/// Conditions table (R9a): run → investigation, table → run, run → each cell's condition.
fn conditions(w: &mut Writer, t: &ConditionsTable, digest: &str) -> Result<Value> {
    let table = format!("conditions_table:sha256:{}", &digest[..16]);
    let mut runs = 0u64;
    let mut without_run: BTreeMap<String, u64> = BTreeMap::new();
    let mut gaps: BTreeMap<String, u64> = BTreeMap::new();
    let mut test_point_gaps: BTreeMap<String, u64> = BTreeMap::new();
    for row in &t.rows {
        let row_loc = row.cells.iter().find_map(|c| match c {
            ConditionCell::Value { value, .. } => value.locator.clone(),
            ConditionCell::Conflicting { candidates, .. } => {
                candidates.iter().find_map(|m| m.locator.clone())
            }
            _ => None,
        });
        let row_only = row_loc.as_ref().map(|l| SourceLocator {
            column: None,
            ..l.clone()
        });
        let Some(run) = &row.run.run_id else {
            // A test point without a run identity (a campaign or test-day row): its stated values
            // are the investigation's, at that row; its gaps are counted.
            *without_run.entry(row.investigation_id.clone()).or_default() += 1;
            let inv = investigation(&row.investigation_id)?;
            let at = row_only
                .as_ref()
                .and_then(|l| l.row)
                .map_or_else(String::new, |r| format!("&row={r}"));
            let note = "a test point of the investigation without a run identity (campaign or test-day row)";
            for cell in &row.cells {
                let key = cell.key();
                let mut input = vec![format!(
                    "{table}#investigation={}{at}&key={}",
                    row.investigation_id,
                    key_name(key)
                )];
                input.extend(
                    row_only
                        .iter()
                        .map(|l| format!("locator:{}", locator_json(l))),
                );
                let (values, resolution) = match cell {
                    ConditionCell::Value { value, .. } => (vec![value], Resolution::Resolved),
                    ConditionCell::Conflicting { candidates, .. } => {
                        (candidates.iter().collect(), Resolution::UnresolvedConflict)
                    }
                    _ => {
                        *test_point_gaps.entry(key_name(key)).or_default() += 1;
                        continue;
                    }
                };
                for m in values {
                    let (rel, to) = cell_target(key, text_of(m), &m.flags)?;
                    w.add(
                        inv.clone(),
                        to,
                        rel,
                        vec![],
                        resolution,
                        det("graph.conditions_table", input.clone(), Some(note.into())),
                        Some(m.clone()),
                    )?;
                }
            }
            continue;
        };
        runs += 1;
        let row_input = |extra: &str| {
            let mut v = vec![format!("{table}#run={run}{extra}")];
            v.extend(
                row_only
                    .iter()
                    .map(|l| format!("locator:{}", locator_json(l))),
            );
            v
        };
        w.add(
            run_node(run),
            investigation(&row.investigation_id)?,
            Relation::PartOf,
            vec![],
            row.run.resolution,
            det("graph.conditions_table", row_input(""), None),
            None,
        )?;
        if let Some(l) = &row_loc {
            w.add(
                source_node(&l.asset_sha256)?,
                run_node(run),
                Relation::DescribesRun,
                vec![],
                row.run.resolution,
                det(
                    "graph.conditions_table",
                    row_input(""),
                    Some("the investigation's experimental table has a row for this run".into()),
                ),
                None,
            )?;
        }
        for cell in &row.cells {
            let key = cell.key();
            let input = row_input(&format!("&key={}", key_name(key)));
            match cell {
                ConditionCell::Value { value, .. } => {
                    let (rel, to) = cell_target(key, text_of(value), &value.flags)?;
                    w.add(
                        run_node(run),
                        to,
                        rel,
                        vec![],
                        Resolution::Resolved,
                        det("graph.conditions_table", input, None),
                        Some(value.clone()),
                    )?;
                }
                ConditionCell::Conflicting {
                    candidates, reason, ..
                } => {
                    *gaps
                        .entry(format!("{}:conflicting", key_name(key)))
                        .or_default() += 1;
                    for m in candidates {
                        let (rel, to) = cell_target(key, text_of(m), &m.flags)?;
                        w.add(
                            run_node(run),
                            to,
                            rel,
                            vec![],
                            Resolution::UnresolvedConflict,
                            det(
                                "graph.conditions_table",
                                input.clone(),
                                Some(reason.clone()),
                            ),
                            Some(m.clone()),
                        )?;
                    }
                }
                ConditionCell::Missing { reason, .. }
                | ConditionCell::NotApplicable { reason, .. } => {
                    let (status, resolution) = match cell {
                        ConditionCell::Missing { .. } => ("missing", Resolution::Unknown),
                        _ => ("not_applicable", Resolution::NotApplicable),
                    };
                    *gaps
                        .entry(format!("{}:{status}", key_name(key)))
                        .or_default() += 1;
                    let (rel, to) = cell_target(key, None, &[])?;
                    w.add(
                        run_node(run),
                        to,
                        rel,
                        vec![],
                        resolution,
                        det("graph.conditions_table", input, Some(reason.clone())),
                        None,
                    )?;
                }
            }
        }
    }
    Ok(json!({"table": table, "runs": runs, "gaps": gaps,
        "test_points_without_run": without_run, "test_point_gaps": test_point_gaps}))
}

/// Evidence catalog: publication → investigation, chunks of the active view → source, chunk → run.
fn catalog_links(
    w: &mut Writer,
    con: &Connection,
    aliases: &HashMap<String, String>,
) -> Result<Value> {
    let mut unresolved: BTreeMap<String, u64> = BTreeMap::new();
    let mut chunks = 0u64;
    let pubs = catalog::publications(con)?;
    for p in &pubs {
        let source = source_node(&p.source_sha256)?;
        for inv in &p.investigation_ids {
            w.add(
                source.clone(),
                investigation(inv)?,
                Relation::Describes,
                vec![],
                Resolution::Resolved,
                det(
                    "graph.catalog_publication",
                    vec![format!("catalog:publication:{}", p.id)],
                    Some("the corpus seed assigns the publication to the investigation".into()),
                ),
                None,
            )?;
        }
        let view = parse_view_id(&p.id, &p.parser_version);
        let prefix = format!("{view}:");
        let mut st = con.prepare(
            "SELECT id, kind, describes_runs_json FROM chunks \
             WHERE publication_id = ?1 AND substr(id, 1, length(?2)) = ?2 ORDER BY ordinal",
        )?;
        let mut rows = st.query(params![p.id, prefix])?;
        while let Some(r) = rows.next()? {
            let id: String = r.get(0)?;
            let kind: String = r.get(1)?;
            let runs: Vec<String> =
                serde_json::from_str(&r.get::<_, String>(2)?).unwrap_or_default();
            let ev = EvidenceId::new(id.clone())?;
            let passage = GraphNode::Entity(EntityRef::Evidence(ev.clone()));
            chunks += 1;
            w.add(
                passage.clone(),
                source.clone(),
                Relation::PartOf,
                vec![ev.clone()],
                Resolution::Resolved,
                det(
                    "graph.catalog_chunk",
                    vec![format!("parse_view:{view}")],
                    Some(format!("{kind} chunk of the active parse view")),
                ),
                None,
            )?;
            for name in runs {
                let resolved = match aliases.get(&name) {
                    Some(r) => Some(r.clone()),
                    None if !name.starts_with("alias:") && name.contains(':') => Some(name.clone()),
                    None => None,
                };
                let Some(run) = resolved.and_then(|r| RunId::new(r).ok()) else {
                    *unresolved.entry(name).or_default() += 1;
                    continue;
                };
                w.add(
                    passage.clone(),
                    run_node(&run),
                    Relation::DescribesRun,
                    vec![ev.clone()],
                    Resolution::Resolved,
                    det(
                        "graph.run_identifier",
                        vec![name.clone()],
                        Some("the passage names the run's identifier (run patterns)".into()),
                    ),
                    None,
                )?;
            }
        }
    }
    Ok(json!({"publications": pubs.len(), "chunks": chunks, "unresolved_run_names": unresolved}))
}

/// R8 labels: source → investigation (the label's status), source → run (mentions: `unknown`).
fn labels(w: &mut Writer, labels: &[SourceLabel]) -> Result<Value> {
    if labels.is_empty() {
        return Ok(json!({"status": "absent: no R8 labels have been run"}));
    }
    let mut n = 0u64;
    for l in labels {
        l.validate()?;
        let source = GraphNode::Entity(EntityRef::Source(l.source_id.clone()));
        let method = |notes: &str| Derivation {
            method: l.labeller.method,
            method_id: "graph.source_label".into(),
            method_version: GRAPH_VERSION.into(),
            inputs: l.labeller.model_id.iter().cloned().collect(),
            agent: l.labeller.model_id.clone(),
            notes: Some(notes.into()),
        };
        // a semantic label without passages is an association, never a resolved link
        let status = if l.labeller.method == Method::Semantic && l.evidence.is_empty() {
            match l.status {
                Resolution::Resolved => Resolution::Unknown,
                s => s,
            }
        } else {
            l.status
        };
        for inv in &l.investigation_ids {
            w.add(
                source.clone(),
                investigation(inv)?,
                Relation::Describes,
                l.evidence.clone(),
                status,
                method("document label (R8)"),
                None,
            )?;
            n += 1;
        }
        for run in &l.runs {
            w.add(
                source.clone(),
                run_node(run),
                Relation::DescribesRun,
                l.evidence.clone(),
                Resolution::Unknown,
                method("the label mentions the run; identity not established"),
                None,
            )?;
            n += 1;
        }
    }
    Ok(json!({"labels": labels.len(), "links": n}))
}

// ------------------------------------------------------------------ read

/// A read-only graph file.
pub struct Store {
    con: Connection,
    pub corpus_snapshot: String,
    pub origin: Origin,
    /// Identity of this build (snapshot, conditions digest, build time), part of cursors.
    pub build_id: String,
}

/// One stored link with its read-order position.
pub struct Row {
    pub seq: i64,
    pub link: ProvenanceLink,
}

/// What one `graph_neighbors` request asks the file for.
pub struct Scan<'a> {
    pub node_key: &'a str,
    pub direction: DirectionFilter,
    pub relations: &'a [Relation],
    pub min_si: Option<f64>,
    pub max_si: Option<f64>,
    /// Links without an SI value only (the `without_value` count).
    pub without_value: bool,
}

impl Store {
    /// `None` when no graph has been built.
    pub fn open(path: &Path) -> Result<Option<Store>> {
        if !path.exists() {
            return Ok(None);
        }
        let con = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;
        let meta = |k: &str| -> Result<String> {
            Ok(con.query_row("SELECT value FROM meta WHERE key = ?1", [k], |r| r.get(0))?)
        };
        let corpus_snapshot = meta("corpus_snapshot")?;
        let origin: Origin = serde_json::from_str(&meta("origin")?)?;
        let build_id = sha256_hex(
            format!(
                "{}|{}|{}|{}",
                meta("graph_version")?,
                corpus_snapshot,
                meta("conditions_digest")?,
                meta("built_at")?
            )
            .as_bytes(),
        )[..16]
            .to_string();
        Ok(Some(Store {
            con,
            corpus_snapshot,
            origin,
            build_id,
        }))
    }

    fn filter_sql(s: &Scan, args: &mut Vec<rusqlite::types::Value>) -> String {
        use rusqlite::types::Value as V;
        let mut sql = match s.direction {
            DirectionFilter::Outgoing => "from_key = ?".to_string(),
            DirectionFilter::Incoming => "to_key = ?".to_string(),
            DirectionFilter::Both => "(from_key = ? OR to_key = ?)".to_string(),
        };
        args.push(V::Text(s.node_key.to_string()));
        if s.direction == DirectionFilter::Both {
            args.push(V::Text(s.node_key.to_string()));
        }
        if !s.relations.is_empty() {
            let marks = vec!["?"; s.relations.len()].join(", ");
            sql.push_str(&format!(" AND relation IN ({marks})"));
            args.extend(s.relations.iter().map(|r| V::Text(relation_name(*r))));
        }
        if s.without_value {
            sql.push_str(" AND value_si IS NULL");
        } else {
            if let Some(m) = s.min_si {
                sql.push_str(" AND value_si >= ?");
                args.push(V::Real(m));
            }
            if let Some(m) = s.max_si {
                sql.push_str(" AND value_si <= ?");
                args.push(V::Real(m));
            }
        }
        sql
    }

    /// Up to `limit` links after `after` in read order.
    pub fn scan(&self, s: &Scan, after: i64, limit: usize) -> Result<Vec<Row>> {
        let mut args = Vec::new();
        let filter = Self::filter_sql(s, &mut args);
        args.push(rusqlite::types::Value::Integer(after));
        args.push(rusqlite::types::Value::Integer(
            i64::try_from(limit).unwrap_or(i64::MAX),
        ));
        let mut st = self.con.prepare_cached(&format!(
            "SELECT seq, link_json FROM links WHERE {filter} AND seq > ? ORDER BY seq LIMIT ?"
        ))?;
        let rows = st
            .query_map(params_from_iter(args), |r| {
                Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        rows.into_iter()
            .map(|(seq, j)| {
                Ok(Row {
                    seq,
                    link: serde_json::from_str(&j)?,
                })
            })
            .collect()
    }

    /// Exact number of stored links matching `s` (no lockbox).
    pub fn count(&self, s: &Scan) -> Result<u64> {
        let mut args = Vec::new();
        let filter = Self::filter_sql(s, &mut args);
        let n: i64 = self.con.query_row(
            &format!("SELECT count(*) FROM links WHERE {filter}"),
            params_from_iter(args),
            |r| r.get(0),
        )?;
        Ok(n as u64)
    }
}
