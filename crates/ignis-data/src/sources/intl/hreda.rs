//! ESA HREDA (Human and Robotic Exploration Data Archive) — access-gated investigations, metadata only.
//!
//! The data sit behind an ESA login at <https://hreda.esac.esa.int> (never used here). DataCite carries
//! only a UUID title for these DOIs and the landing pages are a client-side app, so the descriptive
//! metadata are read from ESA's public DOI registry TAP service (table `dois.esa_dois_hre_missions`),
//! which is what that landing app itself queries. Each investigation becomes an `Investigation` plus a
//! `Source(kind="webpage")` for the DOI with `Rights(status="restricted")` and `access: login_required`.

use std::collections::BTreeMap;

use ignis_core::contracts::{
    Family, IdBasis, Investigation, Rights, RightsStatus, SourceKind, SourceLocator,
};
use serde_json::{json, Map, Value};

use super::common::{self, entry, z};

pub const TAP: &str = "https://esdcdoi.esac.esa.int/registry-sl-tap/tap/sync";
pub const TABLE: &str = "dois.esa_dois_hre_missions";

/// (investigation id, registry name, DOI).
pub const TARGETS: [(&str, &str, &str); 5] = [
    (
        "ESA-HREDA-DropletCombustion_DROP",
        "DropletCombustion_DROP",
        "10.57780/esa-fggmoj8",
    ),
    (
        "ESA-HREDA-FlameSpreading_SR",
        "FlameSpreading_SR",
        "10.57780/esa-q19vqps",
    ),
    (
        "ESA-HREDA-FlameSpreading_SolidFuels_SR",
        "FlameSpreading_SolidFuels_SR",
        "10.57780/esa-7xxl71e",
    ),
    (
        "ESA-HREDA-CombustionPMMA_Samples2_PARA",
        "CombustionPMMA_Samples2_PARA",
        "10.57780/esa-x47pu33",
    ),
    (
        "ESA-HREDA-SOOT_LII_DROP",
        "SOOT_LII_DROP",
        "10.57780/esa-5d5p5aq",
    ),
];

const KEEP: [&str; 23] = [
    "doi_number",
    "investigation_name",
    "operational_nomenclature",
    "investigation_id",
    "mission",
    "platform",
    "temporal_coverage",
    "description",
    "short_description",
    "team_coordinator",
    "credit_guidelines",
    "publication",
    "data_policy_id",
    "data_policy_description",
    "data_policy_url",
    "download_url",
    "link_to_landing_page",
    "hre_e3p_category",
    "version",
    "date_published",
    "generation_date",
    "refresh_date",
    "related_identifiers",
];

pub fn query(dois: &[&str]) -> anyhow::Result<Vec<Map<String, Value>>> {
    let inlist: Vec<String> = dois.iter().map(|d| format!("'{d}'")).collect();
    let q = format!(
        "select * from {TABLE} where doi_number in ({})",
        inlist.join(",")
    );
    let got = common::client().get(
        TAP,
        &[
            ("REQUEST", "doQuery"),
            ("LANG", "ADQL"),
            ("FORMAT", "json"),
            ("QUERY", &q),
        ],
    )?;
    let d: Value = serde_json::from_slice(&got.bytes)?;
    let cols: Vec<String> = d["metadata"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|c| c["name"].as_str().unwrap_or("").to_string())
        .collect();
    Ok(d["data"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|row| {
            cols.iter()
                .cloned()
                .zip(row.as_array().cloned().unwrap_or_default())
                .collect()
        })
        .collect())
}

fn null(v: &Value) -> Value {
    match v.as_str() {
        Some("") | Some("not known") | Some("No publications found") => Value::Null,
        _ => v.clone(),
    }
}

fn truthy(v: &Value) -> bool {
    !(v.is_null() || v.as_str() == Some("") || v == &json!(false))
}

pub fn build(investigation_id: &str, doi: &str, row: Option<&Map<String, Value>>) -> Vec<Value> {
    let retrieved = common::now();
    let Some(row) = row else {
        return vec![entry(
            "note",
            json!({"investigation_id": investigation_id, "doi": doi,
            "text": "DOI not found in the public ESA DOI registry; nothing registered"}),
            None,
            vec![],
        )];
    };
    let g = |k: &str| row.get(k).cloned().unwrap_or(Value::Null);
    let kept: Map<String, Value> = KEEP.iter().map(|k| (k.to_string(), g(k))).collect();
    let rid = common::record_id(&Value::Object(kept.clone()));
    let rights = Rights {
        license: null(&g("data_policy_id")).as_str().map(String::from),
        status: RightsStatus::Restricted,
        statement_locator: Some(SourceLocator {
            asset_sha256: rid.split_once(':').unwrap().1.into(),
            path: Some(format!("{TAP}#{TABLE}/{doi}/data_policy_id")),
            row: None,
            column: None,
            page: None,
            frame: None,
            pts_s: None,
            char_start: None,
            char_end: None,
            ..Default::default()
        }),
    };
    let s = |v: Value| match v {
        Value::String(s) => Some(s),
        Value::Null => None,
        v => Some(v.to_string()),
    };
    let mut landing = common::source(
        rid.clone(),
        IdBasis::MetadataRecord,
        SourceKind::Webpage,
        investigation_id,
        rights,
    );
    landing.title = s(g("investigation_name"));
    landing.canonical_url = Some(format!("https://doi.org/{doi}"));
    landing.doi = Some(doi.into());
    landing.file_version = s(g("version"));
    landing.retrieved_at = Some(z(&retrieved));
    let cov_s = g("temporal_coverage").as_str().unwrap_or("").to_string();
    let cov: Vec<&str> = cov_s.split('/').collect();
    let year = |x: &str| {
        if x.is_empty() {
            Value::Null
        } else {
            json!(x.chars().take(4).collect::<String>())
        }
    };
    let py = |v: Value| match v {
        Value::Null => "None".to_string(),
        Value::String(s) => s,
        v => v.to_string(),
    };
    let inv = Investigation {
        id: investigation_id.into(),
        name: s(g("investigation_name")).unwrap_or_default(),
        platform: Some(format!("{}: {}", py(g("platform")), py(g("mission")))),
        families: vec![Family::Other],
        source_ids: vec![landing.id.clone()],
    };
    let desc = null(&g("description"));
    vec![
        entry(
            "investigation",
            &inv,
            None,
            vec![
                ("doi", json!(doi)),
                ("access", json!("login_required")),
                (
                    "access_note",
                    json!("HREDA data require an ESA account; not logged in, nothing downloaded"),
                ),
                ("data_portal_url", g("download_url")),
                ("year_start", year(cov[0])),
                ("year_end", year(cov.last().copied().unwrap_or(""))),
                (
                    "description_reason",
                    if truthy(&desc) {
                        Value::Null
                    } else {
                        json!("registry says 'not known'")
                    },
                ),
                ("description", desc),
                ("team_coordinator", g("team_coordinator")),
                ("publications", null(&g("publication"))),
                ("license", g("data_policy_description")),
                ("separate_from_nasa_cohorts", json!(true)),
            ],
        ),
        entry(
            "source",
            &landing,
            Some(IdBasis::MetadataRecord),
            vec![
                ("access", json!("login_required")),
                ("metadata_source", json!(format!("{TAP} ({TABLE})"))),
                ("metadata_record", Value::Object(kept)),
            ],
        ),
    ]
}

pub fn build_all() -> anyhow::Result<BTreeMap<String, Vec<Value>>> {
    let dois: Vec<&str> = TARGETS.iter().map(|t| t.2).collect();
    let rows: BTreeMap<String, Map<String, Value>> = query(&dois)?
        .into_iter()
        .map(|r| {
            (
                r.get("doi_number")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_lowercase(),
                r,
            )
        })
        .collect();
    Ok(TARGETS
        .iter()
        .map(|(inv, _, doi)| {
            (
                inv.to_string(),
                build(inv, doi, rows.get(&doi.to_lowercase())),
            )
        })
        .collect())
}
