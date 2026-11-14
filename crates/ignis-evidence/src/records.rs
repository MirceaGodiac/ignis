//! Evidence-layer records (V3.1 §18.1, §18.5). Local to `ignis-evidence`; not shared contracts.
//!
//! `describes_runs` lists `ExperimentRecord` ids (or aliases) the text reports on. `outcome_bearing`
//! is true when the text could reveal how a run ended (results, extinction, burn time, final
//! diameter, conclusions). Unknown attribution is treated conservatively by the firewall.

use ignis_core::contracts::{Derivation, SourceLocator};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Publication {
    /// `pub-<sha16>`
    pub id: String,
    /// sha256 of the PDF bytes (also the Source id without the prefix)
    pub source_sha256: String,
    pub title: Option<String>,
    pub authors: Vec<String>,
    pub year: Option<i64>,
    pub ntrs_id: Option<String>,
    pub doi: Option<String>,
    pub canonical_url: Option<String>,
    /// e.g. `["PSI-25"]`
    pub investigation_ids: Vec<String>,
    /// evidence families (`flex`, `bass`, ...), not predictor inputs
    pub families: Vec<String>,
    pub file_name: Option<String>,
    pub n_pages: i64,
    /// verified | unverified | restricted | noncommercial
    pub rights_status: String,
    pub license: Option<String>,
    pub describes_runs: Vec<String>,
    pub outcome_bearing: bool,
    /// RFC 3339 UTC
    pub ingested_at: String,
    pub parser_version: String,
}

/// A span of one page. For text/caption chunks `text == page_text(pdf, page)[char_start..char_end]`
/// (char indices; see [`crate::pdf::page_text`]).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Chunk {
    pub id: String,
    pub publication_id: String,
    pub ordinal: i64,
    /// text | table | caption
    pub kind: String,
    /// 1-based
    pub page: i64,
    pub char_start: Option<i64>,
    pub char_end: Option<i64>,
    pub section: Option<String>,
    pub text: String,
    /// captions/tables on the same page (`{"captions": [...], "tables": [...]}`)
    pub context: Value,
    pub locator: SourceLocator,
    pub describes_runs: Vec<String>,
    pub outcome_bearing: bool,
    pub evidence_class: String,
}

/// One atomic claim (V3.1 §18.1): population, conditions, result, evidence type, locator.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Finding {
    pub id: String,
    pub publication_id: String,
    #[serde(default)]
    pub chunk_id: Option<String>,
    pub claim: String,
    /// e.g. "n-heptane droplets, ISS CIR/MDCA, 2009-2013"
    pub population: String,
    /// as written, with units
    #[serde(default)]
    pub conditions: serde_json::Map<String, Value>,
    pub result: String,
    /// observed | author_conclusion | independent_calculation | model_inference | mechanism |
    /// hypothesis | absence_of_evidence
    pub evidence_type: String,
    pub locator: SourceLocator,
    #[serde(default)]
    pub describes_runs: Vec<String>,
    #[serde(default = "yes")]
    pub outcome_bearing: bool,
    pub derivation: Derivation,
}

fn yes() -> bool {
    true
}

pub const EVIDENCE_TYPES: [&str; 7] = [
    "observed",
    "author_conclusion",
    "independent_calculation",
    "model_inference",
    "mechanism",
    "hypothesis",
    "absence_of_evidence",
];

/// `SourceLocator` as compact JSON without null fields (the catalog's `locator_json`).
pub fn locator_json(l: &SourceLocator) -> String {
    let mut v = serde_json::to_value(l).expect("locator serializes");
    if let Value::Object(m) = &mut v {
        m.retain(|_, x| !x.is_null());
    }
    v.to_string()
}
