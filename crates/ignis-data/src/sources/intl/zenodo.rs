//! ZARM / DLR FLARE-G II & III infrared drop-tower dataset on Zenodo (record 18479713).
//!
//! Same dataset as register entry D20, registered once as investigation
//! `ZARM-FLARE`. Real record (REST API): DOI 10.5281/zenodo.18479713 (concept
//! 10.5281/zenodo.18479712), licence `cc-by-nc-4.0`, 50 files / 3,845,807,014 bytes: 30 `<ExpID>.mat` IR
//! cubes (109-156 MB each), `index.csv` (31 experiments), `Sample_Shapes.csv` and Figure_0N png/svg/csv.
//! `index.csv` lists experiment 148 whose `148.mat` is absent from the record.

use std::collections::{BTreeMap, HashSet};

use ignis_core::contracts::{
    Alias, Derivation, EvidenceClass, ExperimentRecord, Family, IdBasis, Investigation, Measured,
    Method, RawValue, Resolution, Rights, RightsStatus, SourceKind, SourceLocator,
};
use serde_json::{json, Value};

use super::common::{self, entry, z};
use crate::sources::http::sha256_bytes;
use crate::text::{py_dict_reader, strip};

pub const INVESTIGATION_ID: &str = "ZARM-FLARE";
pub const RECORD_ID: &str = "18479713";
pub const REGISTER_REF: &str = "D20";
const PARSER_VERSION: &str = "1";

pub fn api() -> String {
    format!("https://zenodo.org/api/records/{RECORD_ID}")
}

pub fn file_url(key: &str) -> String {
    format!("{}/files/{key}/content", api())
}

fn sl(sha: &str, path: &str) -> SourceLocator {
    SourceLocator {
        asset_sha256: sha.into(),
        path: Some(path.into()),
        row: None,
        column: None,
        page: None,
        frame: None,
        pts_s: None,
        char_start: None,
        char_end: None,
        ..Default::default()
    }
}

pub fn rights(record_sha: &str, license_id: Option<&str>) -> Rights {
    let nc = license_id.is_some_and(|l| l.contains("-nc"));
    Rights {
        license: license_id.map(String::from),
        status: if nc {
            RightsStatus::Noncommercial
        } else {
            RightsStatus::Unverified
        },
        statement_locator: Some(sl(record_sha, &format!("{}#metadata.license.id", api()))),
    }
}

fn m(
    raw: &str,
    unit: &str,
    f: f64,
    si: &str,
    loc: SourceLocator,
    std: Option<&str>,
) -> anyhow::Result<Option<Measured>> {
    if strip(raw).is_empty() {
        return Ok(None);
    }
    let v: f64 = strip(raw).parse()?;
    let unc = match std.map(strip) {
        Some(s) if !s.is_empty() => Some(s.parse::<f64>()? * f),
        _ => None,
    };
    Ok(Some(Measured {
        raw_value: Some(RawValue::Number(v)),
        raw_unit: Some(unit.into()),
        value_si: Some(v * f),
        unit_si: Some(si.into()),
        uncertainty_si: unc,
        uncertainty_reason: if unc.is_none() {
            Some("source gives no spread for this field".into())
        } else {
            None
        },
        evidence_class: EvidenceClass::SourceDerived,
        locator: Some(loc),
        derivation: None,
        resolution: Resolution::Resolved,
        flags: if unc.is_some() {
            vec!["uncertainty_is_source_std".into()]
        } else {
            vec![]
        },
    }))
}

/// index.csv -> one ExperimentRecord per experiment + its reported outcome (kept out of `conditions`).
pub fn parse_index(
    text: &str,
    asset_sha256: &str,
    present: &HashSet<String>,
) -> anyhow::Result<Vec<(ExperimentRecord, Value)>> {
    let der = Derivation {
        method: Method::Deterministic,
        method_id: "intl.zenodo.flare_index_csv".into(),
        method_version: PARSER_VERSION.into(),
        inputs: vec![format!("sha256:{asset_sha256}")],
        agent: None,
        notes: None,
    };
    let mut out = vec![];
    for (i, row) in py_dict_reader(text.trim_start_matches('\u{feff}'), ',')
        .into_iter()
        .enumerate()
    {
        let g = |k: &str| row.get(k).map(|s| s.as_str()).unwrap_or("");
        let loc = |col: &str| SourceLocator {
            row: Some(i as i64),
            column: Some(col.into()),
            ..sl(asset_sha256, "index.csv")
        };
        let mut conds = BTreeMap::new();
        for (k, v) in [
            (
                "oxygen_mole_fraction",
                m(
                    g("Oxygen [mole/mole]"),
                    "mole/mole",
                    1.0,
                    "1",
                    loc("Oxygen [mole/mole]"),
                    Some(g("Oxygen (std.) [mole/mole]")),
                )?,
            ),
            (
                "pressure",
                m(
                    g("Pressure [kPa]"),
                    "kPa",
                    1e3,
                    "Pa",
                    loc("Pressure [kPa]"),
                    Some(g("Pressure (std.) [kPa]")),
                )?,
            ),
            (
                "flow_velocity",
                m(
                    g("Flow Velocity [mm/s]"),
                    "mm/s",
                    1e-3,
                    "m/s",
                    loc("Flow Velocity [mm/s]"),
                    Some(g("Flow Velocity (std.) [mm/s]")),
                )?,
            ),
            (
                "sample_thickness",
                m(
                    g("Thickness [mm]"),
                    "mm",
                    1e-3,
                    "m",
                    loc("Thickness [mm]"),
                    None,
                )?,
            ),
            (
                "ir_pixel_ratio",
                m(
                    g("PixelRatio [mm/px]"),
                    "mm/px",
                    1e-3,
                    "m/px",
                    loc("PixelRatio [mm/px]"),
                    None,
                )?,
            ),
        ] {
            if let Some(v) = v {
                conds.insert(k.to_string(), v);
            }
        }
        let gaps = strip(g("Gaps [Sequence]"));
        let mat = strip(g("IR Data File [Path]")).to_string();
        let flags = if present.contains(&mat) {
            vec![]
        } else {
            vec!["ir_asset_missing_from_record".to_string()]
        };
        let exp_id = strip(g("Experiment [ID]")).to_string();
        let rec = ExperimentRecord {
            id: format!("{INVESTIGATION_ID}:{exp_id}"),
            investigation_id: INVESTIGATION_ID.into(),
            family: Family::Other,
            session_id: None,
            specimen_id: None,
            aliases: vec![
                Alias {
                    name: exp_id.clone(),
                    scheme: "zarm_experiment_id".into(),
                    locator: Some(loc("Experiment [ID]")),
                },
                Alias {
                    name: mat,
                    scheme: "zenodo_file_key".into(),
                    locator: None,
                },
            ],
            fuel_or_material: Some("PMMA".into()),
            configuration: Some(if gaps.is_empty() {
                "full sample".into()
            } else {
                format!("gap sample: {gaps} mm {}", strip(g("Filler [Material]")))
            }),
            gravity: None,
            conditions: conds,
            identity_resolution: Resolution::Resolved,
            identity_evidence: vec![der.clone()],
            quality_flags: flags,
            notes: Some(format!(
                "test date {}; flow direction {}; drop tower (ZARM Bremen) per record description",
                g("Date [yyyy-mm-dd]"),
                g("Flow Direction [opposed/concurrent]")
            )),
        };
        let outcome = m(
            g("Propagation Rate [mm/s]"),
            "mm/s",
            1e-3,
            "m/s",
            loc("Propagation Rate [mm/s]"),
            Some(g("Propagation Rate (std.) [mm/s]")),
        )?;
        let note = if outcome.is_some() {
            Value::Null
        } else {
            json!("blank in index.csv (gap samples)")
        };
        out.push((
            rec,
            json!({"propagation_rate": outcome, "propagation_rate_note": note}),
        ));
    }
    Ok(out)
}

pub fn build_manifest() -> anyhow::Result<Vec<Value>> {
    let retrieved = common::now();
    let got = common::client().get(&api(), &[])?;
    let record: Value = serde_json::from_slice(&got.bytes)?;
    let record_sha = sha256_bytes(&got.bytes);
    let meta = &record["metadata"];
    let lic = meta["license"]["id"].as_str();
    let r = rights(&record_sha, lic);
    let mut rows = vec![];
    let mut page = common::source(
        format!("sha256:{record_sha}"),
        IdBasis::Bytes,
        SourceKind::Webpage,
        INVESTIGATION_ID,
        r.clone(),
    );
    page.title = Some(format!("Zenodo REST record {RECORD_ID}"));
    page.canonical_url = Some(api());
    page.size_bytes = Some(got.bytes.len() as i64);
    page.media_type = Some("application/json".into());
    page.retrieved_at = Some(z(&retrieved));
    rows.push(entry(
        "source",
        &page,
        Some(IdBasis::Bytes),
        vec![(
            "note",
            json!("API response bytes include live counters; hash identifies this retrieval only"),
        )],
    ));
    let mut present = HashSet::new();
    let mut ids = vec![page.id.clone()];
    let (mut index_text, mut index_sha) = (None, None);
    let mut files: Vec<&Value> = record["files"]
        .as_array()
        .map(|a| a.iter().collect())
        .unwrap_or_default();
    files.sort_by(|a, b| a["key"].as_str().cmp(&b["key"].as_str()));
    let revision = match &record["revision"] {
        Value::Null => "None".to_string(),
        Value::String(s) => s.clone(),
        v => v.to_string(),
    };
    for f in &files {
        let key = f["key"].as_str().unwrap_or_default().to_string();
        present.insert(key.clone());
        let ck = f["checksum"].as_str().unwrap_or("");
        let md5 = ck.strip_prefix("md5:").map(String::from);
        let local = common::raw_path(INVESTIGATION_ID, &key);
        let sha = common::local_sha256(&local);
        if key == "index.csv" && sha.is_some() {
            index_text = Some(std::fs::read_to_string(&local)?);
            index_sha = sha.clone();
        }
        let kind = if key.ends_with(".mat") {
            "ir_mat"
        } else if key.ends_with(".csv") && !key.starts_with("Figure") {
            "metadata_csv"
        } else if key.starts_with("Figure") {
            "figure_data"
        } else {
            "other"
        };
        let size = f["size"].as_i64();
        let id = match &sha {
            Some(s) => format!("sha256:{s}"),
            None => common::listing_id(json!({"url": file_url(&key), "size": size, "md5": md5})),
        };
        let basis = if sha.is_some() {
            IdBasis::Bytes
        } else {
            IdBasis::Listing
        };
        let mut s = common::source(
            id,
            basis,
            SourceKind::DatasetFile,
            INVESTIGATION_ID,
            r.clone(),
        );
        s.title = meta["title"].as_str().map(String::from);
        s.canonical_url = Some(file_url(&key));
        s.file_name = Some(key.clone());
        s.file_version = Some(revision.clone());
        s.doi = record["doi"].as_str().map(String::from);
        s.size_bytes = size;
        s.retrieved_at = sha.as_ref().map(|_| z(&retrieved));
        ids.push(s.id.clone());
        rows.push(entry(
            "source",
            &s,
            Some(basis),
            vec![
                ("kind", json!(kind)),
                ("md5", json!(md5)),
                ("file_id", f["id"].clone()),
            ],
        ));
    }
    let inv = Investigation {
        id: INVESTIGATION_ID.into(),
        name: format!(
            "ZARM/DLR FLARE-G II & III: {}",
            meta["title"].as_str().unwrap_or("")
        ),
        platform: Some("ZARM drop tower (Bremen)".into()),
        families: vec![Family::Other],
        source_ids: ids,
    };
    let related: Vec<Value> = meta["related_identifiers"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|x| x["identifier"].clone())
        .collect();
    let creators: Vec<Value> = meta["creators"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|c| c["name"].clone())
        .collect();
    let total: i64 = files.iter().filter_map(|f| f["size"].as_i64()).sum();
    rows.insert(0, entry("investigation", &inv, None, vec![
        ("register_ref", json!(REGISTER_REF)), ("zenodo_record", json!(RECORD_ID)), ("doi", record["doi"].clone()),
        ("concept_doi", record["conceptdoi"].clone()), ("license", json!(lic)), ("access_right", meta["access_right"].clone()),
        ("publication_date", meta["publication_date"].clone()), ("creators", json!(creators)), ("described_by", json!(related)),
        ("funding", json!("German Federal Ministry of Research, Technology and Space via DLR, FLARE-G II & III (grants 50WM2160 & \
                          50WM2456) per record description")),
        ("file_count", json!(files.len())), ("total_bytes", json!(total)),
        ("mat_files", json!(present.iter().filter(|k| k.ends_with(".mat")).count())),
        ("visual_data", json!("not included (available on request per record description)")), ("separate_from_nasa_cohorts", json!(true)),
    ]));
    match (index_text, index_sha) {
        (Some(t), Some(sha)) => {
            for (rec, outcome) in parse_index(&t, &sha, &present)? {
                rows.push(entry("experiment", &rec, None, vec![("source_outcome", outcome)]));
            }
        }
        _ => rows.push(entry("note", json!({"text": "index.csv not fetched; run `ignis sources intl fetch ZARM-FLARE --execute` then \
                                                    rebuild to add experiment records"}), None, vec![])),
    }
    Ok(rows)
}
