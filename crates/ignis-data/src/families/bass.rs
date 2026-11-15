//! BASS-II (PSI-25) and BASS (PSI-26) tabular ingest + video <-> run <-> specimen mapping.
//!
//! Video decoding belongs to the vision lane; this module only lists files and maps them.
//!
//! Real source structure (verified):
//!
//! * PSI-25 `Experimental table_BASS-II.csv` (129 rows): `PI, Test #, Date, GMT` (GMT = day of year),
//!   `Sample #` (107 labels; specimens are reused: `205M` burns as O3a and O3b), `Fuel Sample Material`
//!   (free text), `Flow restrictor` (0/1/2 perforated plates), `Fan display` (fan volts x10 and pot
//!   settings, often several per test), `Air display` (duct flow speed, cm/s, often a sequence),
//!   `Radiometer gain`, `Total Frames Shot`, `Calibrated initial/final O2 % by vol`, `Initial/Final CO2
//!   % by vol`, `Initial/Final CO (ppm)`. `BASS-II_ReadMe.docx` documents these columns and that the
//!   *video display date is one day later than the actual date* (on-board leap-year bug).
//! * PSI-25 videos: `BASS_GMT<doy>_SAMPLE_<sample>_TEST_<test>[ _2ND_VIEW| _Still_Sequence].mp4` and
//!   `... original videos.zip`; still-image archives `GMT2014_<doy>_BASS_S<sample>M_TP<test>.zip` and
//!   `..._ProofSheet.pdf`; `iss0NNeNNNNNN.zip` crew image dumps (no run key).
//! * PSI-26 `Experimental table_BASS.csv` (122 rows): `Date, GMT, As Run Test #, Test #, Sample #, Fuel
//!   Sample Material`; `As Run_Test Matrix.xlsx` adds 'Date on videos and images' and 'Approx Start Time'.
//!   PSI-26 videos are named by recording time only: `<yyyymmdd> <hh.mm.ss> iss035m<id>.mp4`.
//!   Its sheets 2012/2013 also give each burn's 'O2 % by volume' (written as a fraction, 0.221) and
//!   'pressure, atm' (0.97-0.98): the measured atmosphere of the burn (verified 2026-11-15).
//! * PSI-25 has no per-test pressure. The BASS-II Summary Report states "Ambient pressure was 1 atm" for
//!   the T'ien tests (p. 102) and takes P = 1 atm for the MSG work volume in its O2 consumption
//!   analysis of every test (p. 19); that 1 atm is kept as a report value, flagged approximate.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::Path;
use std::sync::OnceLock;

use calamine::{Data, Reader};
use chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime};
use ignis_core::contracts::{
    Alias, EvidenceClass, ExperimentRecord, Family, Measured, Method, Resolution, Session, Source,
    SourceLocator, Specimen,
};
use ignis_core::interpret::FieldRule;
use ignis_core::prefix::{ProspectivePolicy, ALWAYS_FORBIDDEN};
use indexmap::IndexMap;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::{r, view_rules, Kept};
use crate::common::{det, loc, measured, parse_number, text_value, Bundle, ReviewItem, M};
use crate::paths;
use crate::sources::manifest as mf;
use crate::sources::psi::{read_listing, RemoteFile};
use crate::text::{decode_utf8_sig_lossy, lower, strip, table_rows, Row};

const FAM: &str = "bass";
const V: &str = "1";
pub const T25: &str = "PSI-25_Experimental table_BASS-II.csv";
pub const T26: &str = "PSI-26_Experimental table_BASS.csv";
pub const ASRUN26: &str = "PSI-26_Science Documents_As Run_Test Matrix.xlsx";
/// The BASS-II Summary Report (PSI-25 Reports): the only source of a PSI-25 pressure. Not a sample file of
/// the manifest; located by its file hash, which is also its source id in the evidence corpus.
pub const REPORT25: &str = "PSI-25_Reports_BASS-II Summary Report.pdf";
pub const REPORT25_SHA: &str = "5ffa911fd9b6356f204646003fb9342e341810dcdfdf5cb15756e7f01ec5f770";

pub fn policy() -> ProspectivePolicy {
    let mut f = vec!["x_o2_final", "x_co2_final", "x_co_final", "total_frames"];
    f.extend(ALWAYS_FORBIDDEN);
    ProspectivePolicy::new(
        Family::Bass,
        &[
            "x_o2",
            "pressure",
            "x_co2",
            "x_co",
            "flow_velocity",
            "flow_restrictor",
            "fan_display",
        ],
        &f,
    )
}

pub fn rules() -> Vec<FieldRule> {
    use Method::*;
    let mut v = vec![
        r(FAM, "x_o2", Deterministic, "bass.cond.o2_vol_pct",
          "'Calibrated initial O2 % by vol' /100; volume % taken as mole fraction (ideal gas), flagged"),
        r(FAM, "pressure", Deterministic, "bass.cond.pressure",
          "PSI-26: the as-run matrix 'pressure, atm' of the burn (bass.cond.pressure_asrun), an original \
           observation. PSI-25: no per-test reading; the BASS-II Summary Report's 1 atm (stated for the T'ien \
           tests p. 102, assumed for the MSG work volume p. 19) as source_derived, flagged approximate \
           (bass.cond.pressure_report)"),
        r(FAM, "x_o2_asrun", Deterministic, "bass.cond.o2_asrun",
          "PSI-26: the as-run matrix 'O2 % by volume' of the burn; the sheet writes fractions (0.221) under a % \
           header, so a value <= 1 is a fraction and a value > 1 is percent; both flagged with the reading"),
        r(FAM, "flow_velocity", Deterministic, "bass.cond.air_display",
          "'Air display' is duct flow speed in cm/s (ReadMe). A single number -> m/s; a sequence of settings has no \
           times in the table -> kept as text, unresolved (needs the video overlay)"),
        r(FAM, "post_test", Deterministic, "bass.post", "final O2/CO2/CO and total frames are after-test facts, never prospective"),
        r(FAM, "geometry", Deterministic, "bass.material.parse",
          "regex over the material text: class (planar_fabric/planar_film/slab/rod/sphere/candle), width cm, \
           thickness (micron|mm), rod diameter (inch fractions); unparsed parts stay text"),
        r(FAM, "video_map", Deterministic, "bass.video.map",
          "PSI-25: test id in the file name -> table row, corroborated by GMT day and sample number (any mismatch -> \
           unresolved_conflict). PSI-26: recording date = 'Date on videos' and start time within [-20, +90] min of the \
           as-run 'Approx Start Time'; exactly one candidate -> resolved (flagged approximate), several -> \
           unresolved_conflict, none -> unknown"),
    ];
    v.extend(view_rules(FAM));
    v.sort_by(|a, b| a.field.cmp(b.field));
    v
}

fn re(cell: &'static OnceLock<Regex>, pat: &str) -> &'static Regex {
    cell.get_or_init(|| Regex::new(pat).unwrap())
}

fn col<'a>(row: &'a Row, prefix: &str) -> &'a str {
    static WS: OnceLock<Regex> = OnceLock::new();
    row.cells
        .keys()
        .find(|k| lower(&re(&WS, r"\s+").replace_all(k, " ")).starts_with(&lower(prefix)))
        .map(|s| s.as_str())
        .unwrap_or_else(|| panic!("no column starting with {prefix:?}"))
}

pub type Geo = Vec<(&'static str, f64, &'static str)>;

pub fn parse_material(text: &str) -> (Option<&'static str>, &'static str, Geo) {
    static CM: OnceLock<Regex> = OnceLock::new();
    static UM: OnceLock<Regex> = OnceLock::new();
    static MM: OnceLock<Regex> = OnceLock::new();
    static IN: OnceLock<Regex> = OnceLock::new();
    let t = lower(text);
    let has = |s: &str| t.contains(s);
    let material = if has("sibal") {
        Some("SIBAL")
    } else if has("nomex") {
        Some("Nomex")
    } else if has("wax") || has("candle") {
        Some("wax")
    } else if has("pmma") || has("film") || has("rod") || has("sphere") {
        Some("PMMA")
    } else {
        None
    };
    let cls = if has("candle") {
        "candle"
    } else if has("sphere") || has("spherical") {
        "sphere"
    } else if has("rod") {
        "rod"
    } else if has("sibal") || has("nomex") || has("fabric") {
        "planar_fabric"
    } else if has("film") || has("micron") {
        "planar_film"
    } else if has("thick") || has("sided") {
        "planar_slab"
    } else {
        "unknown"
    };
    let mut geo: Geo = vec![];
    if let Some(m) = re(&CM, r"(\d+(?:\.\d+)?)\s*cm(?:\s*wide)?").captures(&t) {
        geo.push((
            if cls == "sphere" {
                "sphere_diameter"
            } else {
                "width"
            },
            m[1].parse().unwrap(),
            "cm",
        ));
    }
    if let Some(m) = re(&UM, r"(\d+(?:\.\d+)?)\s*micron").captures(&t) {
        geo.push(("thickness", m[1].parse().unwrap(), "um"));
    } else if let Some(m) = re(&MM, r"(\d+(?:\.\d+)?)\s*mm(?:\s*thick)?").captures(&t) {
        geo.push(("thickness", m[1].parse().unwrap(), "mm"));
    }
    if cls == "rod" {
        if let Some(m) = re(&IN, r#"(\d+)\s*/\s*(\d+)\s*""#).captures(&t) {
            geo.push((
                "diameter",
                m[1].parse::<f64>().unwrap() / m[2].parse::<f64>().unwrap(),
                "in",
            ));
        }
    }
    if has("1 sided") || has("one sided") {
        geo.push(("burn_sides", 1.0, "1"));
    } else if has("2 sided") {
        geo.push(("burn_sides", 2.0, "1"));
    }
    (material, cls, geo)
}

/// m/d/yyyy or m/d/yy; stray non-date characters (e.g. '6/29/`12') are removed and flagged.
pub fn parse_date(raw: &str) -> (Option<NaiveDate>, Vec<String>) {
    static NON: OnceLock<Regex> = OnceLock::new();
    let s = strip(raw);
    let clean = re(&NON, r"[^0-9/]").replace_all(s, "").to_string();
    let flags = if clean != s {
        vec![format!("date_cell_cleaned:{s}")]
    } else {
        vec![]
    };
    if let Some(d) = super::flex::parse_us_date(&clean) {
        return (Some(d), flags);
    }
    (
        None,
        if s.is_empty() {
            vec![]
        } else {
            vec![format!("unparsed_date:{s}")]
        },
    )
}

fn norm_sample(s: &str) -> String {
    static WS: OnceLock<Regex> = OnceLock::new();
    re(&WS, r"\s+").replace_all(s, "").to_uppercase()
}

/// Numeric core used to compare file-name and table spellings ('153m' ~ '153M' ~ 'S153M').
fn sample_key(s: &str) -> String {
    static D: OnceLock<Regex> = OnceLock::new();
    match re(&D, r"(\d+)").captures(s) {
        Some(m) => m[1]
            .parse::<u128>()
            .map(|n| n.to_string())
            .unwrap_or_else(|_| m[1].to_string()),
        None => s.to_uppercase(),
    }
}

fn read_table(src: &Source) -> anyhow::Result<Vec<Row>> {
    let p = mf::local_path(src);
    let raw = std::fs::read(&p).map_err(|e| anyhow::anyhow!("{}: {e}", p.display()))?;
    Ok(table_rows(&decode_utf8_sig_lossy(&raw), true))
}

fn geometry(geo: &Geo, l: &SourceLocator) -> BTreeMap<String, Measured> {
    geo.iter()
        .map(|(k, v, u)| {
            (
                k.to_string(),
                measured(*v, Some(u), Some(l.clone()), M::new("bass.material.parse")),
            )
        })
        .collect()
}

fn session(b: &mut Bundle, inv: &str, d: Option<NaiveDate>, ev: SourceLocator) -> Option<String> {
    let d = d?;
    let sid = format!("{inv}:day:{d}");
    if !b.sessions.contains_key(&sid) {
        b.sessions.insert(
            sid.clone(),
            Session {
                id: sid.clone(),
                investigation_id: inv.into(),
                campaign_id: None,
                test_day: Some(d.to_string()),
                gmt_start: None,
                evidence: vec![ev],
                resolution: Resolution::Resolved,
            },
        );
    }
    Some(sid)
}

fn bass2(
    b: &mut Bundle,
    srcs: &BTreeMap<String, Source>,
) -> anyhow::Result<HashMap<String, ExperimentRecord>> {
    let src = srcs.get(T25).ok_or_else(|| {
        anyhow::anyhow!("{T25} not in manifest; run `ignis sources samples PSI-25`")
    })?;
    let sha = mf::sha_of(src);
    let mut out = HashMap::new();
    for row in read_table(src)? {
        let rl = |c: &str| loc(&sha, T25, Some(row.row), Some(c));
        let test = strip(row.get("Test #")).to_string();
        let run_id = format!("PSI-25:{test}");
        let (d, mut flags) = parse_date(row.get("Date"));
        let doy = parse_number(row.get("GMT"));
        if let (Some(d), Some(doy)) = (d, doy) {
            if d.ordinal() as i64 != doy.trunc() as i64 {
                flags.push(format!(
                    "gmt_day_{}_vs_date_doy_{}",
                    doy.trunc() as i64,
                    d.ordinal()
                ));
            }
        }
        let session_id = session(b, "PSI-25", d, rl("Date"));
        let sample = norm_sample(row.get("Sample #"));
        let spec_id = format!("PSI-25:sample:{sample}");
        let (material, cls, geo) = parse_material(row.get("Fuel Sample Material"));
        if !b.specimens.contains_key(&spec_id) {
            b.specimens.insert(
                spec_id.clone(),
                Specimen {
                    id: spec_id.clone(),
                    investigation_id: "PSI-25".into(),
                    material: material.map(String::from),
                    geometry: geometry(&geo, &rl("Fuel Sample Material")),
                    reuse_group: Some(format!("PSI-25:reuse:{}", sample_key(&sample))),
                },
            );
        }
        let air = row.get("Air display");
        let air_num = parse_number(air);
        let k_o2 = col(&row, "Calibrated initial O2");
        let mut conds = BTreeMap::new();
        conds.insert(
            "x_o2".into(),
            measured(
                row.get(k_o2),
                Some("%"),
                Some(rl(k_o2)),
                M::new("bass.cond.o2_vol_pct")
                    .flags(&["volume_percent_as_mole_fraction"])
                    .notes(
                        "sensor offset-corrected against the ISS MCA reading of the day (ReadMe)",
                    ),
            ),
        );
        let pi = lower(strip(row.get("PI")));
        let tien = pi.starts_with("t'ien") || pi.starts_with("tien") || pi.starts_with("t’ien");
        let (page, how, note) = if tien {
            (102, "stated_by_report", "BASS-II Summary Report, T'ien chapter: 'Ambient pressure was 1 atm'; no per-test reading")
        } else {
            (19, "assumed_by_report", "BASS-II Summary Report takes P = 1 atm for the MSG work volume in its O2 consumption analysis; no per-test reading")
        };
        conds.insert(
            "pressure".into(),
            measured(
                1.0,
                Some("atm"),
                Some(SourceLocator {
                    page: Some(page),
                    ..loc(REPORT25_SHA, REPORT25, None, None)
                }),
                M::new("bass.cond.pressure_report")
                    .evidence(EvidenceClass::SourceDerived)
                    .flags(&["approximate", how])
                    .notes(note),
            ),
        );
        conds.insert(
            "x_co2".into(),
            measured(
                row.get("Initial CO2 % by vol"),
                Some("%"),
                Some(rl("Initial CO2 % by vol")),
                M::new("bass.cond.co2").flags(&["volume_percent_as_mole_fraction"]),
            ),
        );
        conds.insert(
            "x_co".into(),
            measured(
                row.get("Initial CO (ppm)"),
                Some("ppm"),
                Some(rl("Initial CO (ppm)")),
                M::new("bass.cond.co"),
            ),
        );
        conds.insert(
            "flow_restrictor".into(),
            text_value(
                row.get("Flow restrictor"),
                Some(rl("Flow restrictor")),
                "bass.cond.restrictor",
                Some("0 = highest flow / least restriction, 2 = lowest flow".into()),
                None,
                &[],
            ),
        );
        conds.insert(
            "fan_display".into(),
            text_value(
                row.get("Fan display"),
                Some(rl("Fan display")),
                "bass.cond.fan_display",
                Some("fan volts x10 and/or pot settings; untimed sequence if several".into()),
                None,
                &[],
            ),
        );
        conds.insert(
            "flow_velocity".into(),
            match air_num {
                Some(a) => measured(
                    a,
                    Some("cm/s"),
                    Some(rl("Air display")),
                    M::new("bass.cond.air_display")
                        .notes("duct anemometer, ~1 min response (ReadMe)"),
                ),
                None => text_value(
                    air,
                    Some(rl("Air display")),
                    "bass.cond.air_display",
                    None,
                    Some(Resolution::Unknown),
                    &["multiple_settings_untimed"],
                ),
            },
        );
        let rec = ExperimentRecord {
            id: run_id.clone(),
            investigation_id: "PSI-25".into(),
            family: Family::Bass,
            session_id,
            specimen_id: Some(spec_id),
            aliases: vec![
                Alias {
                    name: test.clone(),
                    scheme: "test_number".into(),
                    locator: Some(rl("Test #")),
                },
                Alias {
                    name: row.get("Sample #").into(),
                    scheme: "sample_number".into(),
                    locator: Some(rl("Sample #")),
                },
                Alias {
                    name: row.get("PI").into(),
                    scheme: "pi".into(),
                    locator: Some(rl("PI")),
                },
            ],
            fuel_or_material: Some(row.get("Fuel Sample Material").into()),
            configuration: Some(cls.into()),
            gravity: None,
            conditions: conds,
            identity_resolution: Resolution::Resolved,
            identity_evidence: vec![det(
                "bass.identity.test_number",
                V,
                vec![src.id.clone()],
                None,
            )],
            quality_flags: flags,
            notes: None,
        };
        out.insert(test.to_uppercase(), rec.clone());
        b.records.push(rec);
        let mut post = IndexMap::new();
        for (k, c, u) in [
            ("x_o2_final", "Calibrated final O2 % by vol", "%"),
            ("x_co2_final", "Final CO2 % by vol", "%"),
            ("x_co_final", "Final CO (ppm)", "ppm"),
        ] {
            post.insert(
                k.to_string(),
                measured(row.get(c), Some(u), Some(rl(c)), M::new("bass.post")),
            );
        }
        post.insert(
            "total_frames".into(),
            text_value(
                row.get("Total Frames Shot"),
                Some(rl("Total Frames Shot")),
                "bass.post",
                None,
                None,
                &[],
            ),
        );
        b.post.insert(run_id, post);
    }
    Ok(out)
}

fn bass1(b: &mut Bundle, srcs: &BTreeMap<String, Source>) -> anyhow::Result<Vec<ExperimentRecord>> {
    let src = &srcs[T26];
    let sha = mf::sha_of(src);
    let mut recs = vec![];
    let mut seen = HashSet::new();
    for row in read_table(src)? {
        let rl = |c: &str| loc(&sha, T26, Some(row.row), Some(c));
        let (asrun, test) = (row.get("As Run Test #"), row.get("Test #"));
        let mut run_id = format!("PSI-26:asrun{asrun}:test{test}");
        let mut res = Resolution::Resolved;
        if seen.contains(&run_id) {
            res = Resolution::UnresolvedConflict;
            run_id = format!("{run_id}#row{}", row.row);
        }
        seen.insert(run_id.clone());
        let (d, dflags) = parse_date(row.get("Date"));
        let session_id = session(b, "PSI-26", d, rl("Date"));
        let sample = norm_sample(row.get("Sample #"));
        let spec_id = format!("PSI-26:sample:{sample}");
        let (material, cls, geo) = parse_material(row.get("Fuel Sample Material"));
        if !b.specimens.contains_key(&spec_id) {
            b.specimens.insert(
                spec_id.clone(),
                Specimen {
                    id: spec_id.clone(),
                    investigation_id: "PSI-26".into(),
                    material: material.map(String::from),
                    geometry: geometry(&geo, &rl("Fuel Sample Material")),
                    reuse_group: Some(format!("PSI-26:reuse:{}", sample_key(&sample))),
                },
            );
        }
        let rec = ExperimentRecord {
            id: run_id,
            investigation_id: "PSI-26".into(),
            family: Family::Bass,
            session_id,
            specimen_id: Some(spec_id),
            aliases: vec![
                Alias {
                    name: asrun.into(),
                    scheme: "as_run_number".into(),
                    locator: Some(rl("As Run Test #")),
                },
                Alias {
                    name: test.into(),
                    scheme: "test_number".into(),
                    locator: Some(rl("Test #")),
                },
                Alias {
                    name: row.get("Sample #").into(),
                    scheme: "sample_number".into(),
                    locator: Some(rl("Sample #")),
                },
            ],
            fuel_or_material: Some(row.get("Fuel Sample Material").into()),
            configuration: Some(cls.into()),
            gravity: None,
            conditions: BTreeMap::new(),
            identity_resolution: res,
            identity_evidence: vec![det(
                "bass.identity.asrun_test",
                V,
                vec![src.id.clone()],
                None,
            )],
            quality_flags: dflags,
            notes: None,
        };
        recs.push(rec.clone());
        b.records.push(rec);
    }
    Ok(recs)
}

/// One row of the video <-> run map (`extra_video_map.jsonl`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VideoMap {
    pub investigation: String,
    pub file_name: String,
    pub file_version: Option<i64>,
    pub stable_url: String,
    pub size_bytes: Option<i64>,
    pub kind: String,
    pub variant: String,
    pub run_id: Option<String>,
    pub specimen_id: Option<String>,
    pub resolution: String,
    pub evidence: String,
    pub method_id: String,
    pub method_version: String,
}

fn vm(
    f: &RemoteFile,
    run_id: Option<String>,
    spec: Option<String>,
    res: Resolution,
    why: String,
    kind: Option<&str>,
    variant: &str,
) -> VideoMap {
    let ext = Path::new(&f.file_name)
        .extension()
        .map(|e| lower(&e.to_string_lossy()))
        .unwrap_or_default();
    VideoMap {
        investigation: f.investigation.clone(),
        file_name: f.file_name.clone(),
        file_version: f.file_version,
        stable_url: f.stable_url(),
        size_bytes: f.size_bytes,
        kind: kind.map(String::from).unwrap_or_else(|| {
            if ext == "mp4" || ext == "ts" {
                "video".into()
            } else {
                "archive".into()
            }
        }),
        variant: variant.into(),
        run_id,
        specimen_id: spec,
        resolution: super::flex::res_str(res),
        evidence: why,
        method_id: "bass.video.map".into(),
        method_version: V.into(),
    }
}

/// One as-run matrix row (PSI-26 `As Run_Test Matrix.xlsx`, sheets 2012/2013).
#[derive(Debug, Clone)]
pub struct AsRun {
    pub sheet: String,
    pub row: i64,
    pub asrun: String,
    pub test: String,
    pub video_date: Option<NaiveDate>,
    pub start: Option<NaiveTime>,
    /// 'O2 % by volume' as written (a fraction on these sheets).
    pub o2: Option<f64>,
    /// 'pressure, atm' as written.
    pub pressure_atm: Option<f64>,
}

fn alias<'a>(r: &'a ExperimentRecord, scheme: &str) -> &'a str {
    r.aliases
        .iter()
        .find(|a| a.scheme == scheme)
        .map(|a| a.name.as_str())
        .unwrap_or("")
}

fn strip_chars<'a>(s: &'a str, cs: &[char]) -> &'a str {
    s.trim_matches(|c| cs.contains(&c))
}

pub fn map_videos(
    files25: &[RemoteFile],
    runs25: &HashMap<String, ExperimentRecord>,
    files26: &[RemoteFile],
    asrun26: &[AsRun],
    runs26: &[ExperimentRecord],
) -> Vec<VideoMap> {
    static V25: OnceLock<Regex> = OnceLock::new();
    static S25: OnceLock<Regex> = OnceLock::new();
    static V26: OnceLock<Regex> = OnceLock::new();
    let v25 = re(
        &V25,
        r"(?i)^PSI-25_Raw Data_BASS_GMT(?P<doy>\d+)_SAMPLE_?(?P<sample>[0-9A-Za-z]+)_TEST[_-]?(?P<test>[A-Za-z]+\d+[a-z]?)(?P<segment>[_-](?:First|Second)_Ignition_Attempts|[_-]Part-\d+|[_-]LOS[-_][Pp]ortion(?:_[\d-]+)?|-shifted-video)?(?P<variant>_2ND_VIEW|_Still_Sequence| original videos)?\.(?P<ext>mp4|zip)$",
    );
    let s25 = re(
        &S25,
        r"(?i)^PSI-25_Raw Data_GMT(?P<year>\d{4})_(?P<doy>\d{3})_BASS_S(?P<sample>[0-9A-Za-z]+?)M?_TP(?P<test>[A-Za-z]+\d+[a-z]?)(?:_ProofSheet)?\.(?P<ext>pdf|zip)$",
    );
    let v26 = re(
        &V26,
        r"(?i)^PSI-26_Raw Data_(?P<ymd>\d{8}) (?P<hms>\d{2}\.\d{2}\.\d{2}) iss\w*\.mp4$",
    );
    let mut out = vec![];
    for f in files25 {
        let lname = lower(&f.file_name);
        let (m, is_v25) = match v25.captures(&f.file_name) {
            Some(m) => (m, true),
            None => match s25.captures(&f.file_name) {
                Some(m) => (m, false),
                None => {
                    if lname.ends_with(".mp4") || lname.ends_with(".zip") {
                        out.push(vm(
                            f,
                            None,
                            None,
                            Resolution::Unknown,
                            "no run key in file name".into(),
                            None,
                            "main",
                        ));
                    }
                    continue;
                }
            },
        };
        let kind = if is_v25 && lower(&m["ext"]) == "mp4" {
            "video"
        } else {
            "image_archive"
        };
        let mut variant = m
            .name("variant")
            .map(|x| strip(x.as_str()).to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "main".into());
        if let Some(seg) = m.name("segment").filter(|s| !s.as_str().is_empty()) {
            variant = format!(
                "segment:{}{}",
                strip_chars(seg.as_str(), &['_', '-']),
                if variant == "main" {
                    String::new()
                } else {
                    format!("/{variant}")
                }
            );
        }
        let test = m["test"].to_uppercase();
        let Some(rec) = runs25.get(&test) else {
            out.push(vm(
                f,
                None,
                None,
                Resolution::Unknown,
                format!("test {test} not in master table"),
                Some(kind),
                &variant,
            ));
            continue;
        };
        let mut why = vec![];
        let doy: i64 = m["doy"].parse().unwrap();
        if let Some(sd) = rec
            .session_id
            .as_ref()
            .and_then(|s| s.rsplit(':').next())
            .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        {
            let table_doy = sd.ordinal() as i64;
            if doy == table_doy + 1 {
                why.push(
                    "file GMT day = actual + 1 (video display leap-year offset, ReadMe)"
                        .to_string(),
                );
            } else if doy != table_doy {
                why.push(format!("file GMT day {doy} != table day {table_doy}"));
            }
        }
        if sample_key(&m["sample"]) != sample_key(alias(rec, "sample_number")) {
            why.push(format!("file sample {} != table sample", &m["sample"]));
        }
        let conflict = why.iter().any(|w| w.contains("!="));
        let ev = if why.is_empty() {
            "test id + GMT day + sample agree".to_string()
        } else {
            why.join("; ")
        };
        out.push(vm(
            f,
            Some(rec.id.clone()),
            rec.specimen_id.clone(),
            if conflict {
                Resolution::UnresolvedConflict
            } else {
                Resolution::Resolved
            },
            ev,
            Some(kind),
            &variant,
        ));
    }
    // PSI-26: time-window match against the as-run matrix
    let mut by_key: HashMap<(String, String), &ExperimentRecord> = HashMap::new();
    for r in runs26 {
        by_key.insert(
            (
                alias(r, "as_run_number").into(),
                alias(r, "test_number").into(),
            ),
            r,
        );
    }
    for f in files26 {
        let Some(m) = v26.captures(&f.file_name) else {
            if lower(&f.file_name).ends_with(".mp4") {
                out.push(vm(
                    f,
                    None,
                    None,
                    Resolution::Unknown,
                    "no timestamp in file name".into(),
                    None,
                    "main",
                ));
            }
            continue;
        };
        let Ok(vt) =
            NaiveDateTime::parse_from_str(&format!("{}{}", &m["ymd"], &m["hms"]), "%Y%m%d%H.%M.%S")
        else {
            continue;
        };
        let mut cands: Vec<(f64, &AsRun)> = vec![];
        for a in asrun26 {
            let (Some(vd), Some(st)) = (a.video_date, a.start) else {
                continue;
            };
            if vd != vt.date() {
                continue;
            }
            let dt = (vt - vd.and_time(st)).num_seconds() as f64 / 60.0;
            if (-20.0..=90.0).contains(&dt) {
                cands.push((dt, a));
            }
        }
        if cands.len() == 1 {
            let (dt, a) = cands[0];
            let rec = by_key.get(&(a.asrun.clone(), a.test.clone()));
            out.push(vm(
                f,
                rec.map(|r| r.id.clone()),
                rec.and_then(|r| r.specimen_id.clone()),
                if rec.is_some() {
                    Resolution::Resolved
                } else {
                    Resolution::Unknown
                },
                format!(
                    "approx_start_time_match dt={dt:+.0} min (as-run row {}){}",
                    a.row,
                    if rec.is_some() {
                        ""
                    } else {
                        "; as-run row not in master table"
                    }
                ),
                None,
                "main",
            ));
        } else if !cands.is_empty() {
            let list: Vec<String> = cands
                .iter()
                .map(|(_, a)| format!("{}/{}", a.asrun, a.test))
                .collect();
            out.push(vm(
                f,
                None,
                None,
                Resolution::UnresolvedConflict,
                format!("{} as-run rows in window: {}", cands.len(), list.join(", ")),
                None,
                "main",
            ));
        } else {
            out.push(vm(
                f,
                None,
                None,
                Resolution::Unknown,
                "no as-run row within the time window".into(),
                None,
                "main",
            ));
        }
    }
    out
}

/// Python `str(cell)` of an openpyxl value (ints print without a decimal point).
fn cell_str(d: &Data) -> String {
    match d {
        Data::Empty => "None".into(),
        Data::String(s) => s.clone(),
        Data::Int(i) => i.to_string(),
        Data::Float(x) if x.fract() == 0.0 && x.abs() < 1e15 => format!("{}", *x as i64),
        Data::Float(x) => crate::text::py_float(*x),
        Data::Bool(b) => {
            if *b {
                "True".into()
            } else {
                "False".into()
            }
        }
        Data::DateTime(dt) => match dt.as_datetime() {
            Some(t) if dt.as_f64() < 1.0 => t.time().format("%H:%M:%S").to_string(),
            Some(t) => t.format("%Y-%m-%d %H:%M:%S").to_string(),
            None => dt.as_f64().to_string(),
        },
        Data::DateTimeIso(s) | Data::DurationIso(s) => s.clone(),
        Data::Error(e) => format!("{e:?}"),
    }
}

fn is_empty(d: Option<&Data>) -> bool {
    matches!(d, None | Some(Data::Empty))
}

/// First `(?<!\d)(\d{1,2}):(\d{2})` with hour < 24 and minute < 60 (re.finditer semantics).
fn first_clock(s: &str) -> Option<NaiveTime> {
    let c: Vec<char> = s.chars().collect();
    let mut pos = 0;
    while pos < c.len() {
        let mut found = None;
        for i in pos..c.len() {
            if i > 0 && c[i - 1].is_ascii_digit() {
                continue;
            }
            for hl in [2usize, 1] {
                if i + hl + 3 <= c.len()
                    && c[i..i + hl].iter().all(|x| x.is_ascii_digit())
                    && c[i + hl] == ':'
                    && c[i + hl + 1].is_ascii_digit()
                    && c[i + hl + 2].is_ascii_digit()
                {
                    found = Some((i, hl));
                    break;
                }
            }
            if found.is_some() {
                break;
            }
        }
        let (i, hl) = found?;
        let h: u32 = c[i..i + hl].iter().collect::<String>().parse().ok()?;
        let m: u32 = c[i + hl + 1..i + hl + 3]
            .iter()
            .collect::<String>()
            .parse()
            .ok()?;
        if h < 24 && m < 60 {
            return NaiveTime::from_hms_opt(h, m, 0);
        }
        pos = i + hl + 3;
    }
    None
}

const O2_ASRUN: &str = "O2 % by volume";
const P_ASRUN: &str = "pressure, atm";

/// PSI-26 burns get the atmosphere the as-run matrix measured for them: the row with the same as-run and
/// test numbers. A run with no such row, or with several, keeps no value (never a neighbour's).
pub fn atmosphere26(b: &mut Bundle, asrun: &[AsRun], sha: &str) -> (usize, usize) {
    let mut by_key: HashMap<(String, String), Vec<&AsRun>> = HashMap::new();
    for a in asrun {
        by_key
            .entry((norm_num(&a.asrun), norm_num(&a.test)))
            .or_default()
            .push(a);
    }
    let (mut o2n, mut pn) = (0, 0);
    for r in b
        .records
        .iter_mut()
        .filter(|r| r.investigation_id == "PSI-26")
    {
        let key = (
            norm_num(alias(r, "as_run_number")),
            norm_num(alias(r, "test_number")),
        );
        let Some([a]) = by_key.get(&key).map(|v| v.as_slice()) else {
            continue;
        };
        let at = |h: &str| {
            Some(loc(
                sha,
                ASRUN26,
                Some(a.row),
                Some(&format!("{}!{h}", a.sheet)),
            ))
        };
        if let Some(x) = a.o2 {
            let (unit, how) = if x <= 1.0 {
                ("1", "percent_header_fraction_values")
            } else {
                ("%", "volume_percent_as_mole_fraction")
            };
            r.conditions.insert(
                "x_o2".into(),
                measured(
                    x,
                    Some(unit),
                    at(O2_ASRUN),
                    M::new("bass.cond.o2_asrun")
                        .flags(&[how])
                        .notes("as-run matrix O2 of this burn"),
                ),
            );
            o2n += 1;
        }
        if let Some(p) = a.pressure_atm {
            r.conditions.insert(
                "pressure".into(),
                measured(
                    p,
                    Some("atm"),
                    at(P_ASRUN),
                    M::new("bass.cond.pressure_asrun").notes("as-run matrix pressure of this burn"),
                ),
            );
            pn += 1;
        }
    }
    (o2n, pn)
}

/// "12" and "12.0" are the same test number; text ids ("12-R2", "53a") stay as written.
fn norm_num(s: &str) -> String {
    let t = strip(s);
    match t.parse::<f64>() {
        Ok(x) if x.fract() == 0.0 => format!("{}", x as i64),
        _ => t.to_string(),
    }
}

pub fn parse_asrun26(raw_path: &Path) -> anyhow::Result<Vec<AsRun>> {
    let mut wb = calamine::open_workbook_auto(raw_path)?;
    let names = wb.sheet_names().to_vec();
    let mut out = vec![];
    for sheet in ["2012", "2013"] {
        if !names.iter().any(|n| n == sheet) {
            continue;
        }
        let rng = wb.worksheet_range(sheet)?;
        let Some((_, _)) = rng.start() else { continue };
        let (er, ec) = rng.end().unwrap();
        // Absolute grid from A1, like openpyxl's rows.
        let rows: Vec<Vec<Data>> = (0..=er)
            .map(|r| {
                (0..=ec)
                    .map(|c| rng.get_value((r, c)).cloned().unwrap_or(Data::Empty))
                    .collect()
            })
            .collect();
        let hi = rows
            .iter()
            .position(|r| !is_empty(r.first()) && cell_str(&r[0]).starts_with("Actual Date"))
            .ok_or_else(|| anyhow::anyhow!("{sheet}: no 'Actual Date' header"))?;
        let header: Vec<String> = rows[hi]
            .iter()
            .map(|h| {
                if is_empty(Some(h)) {
                    String::new()
                } else {
                    strip(&cell_str(h)).to_string()
                }
            })
            .collect();
        let ix = |k: &str| {
            header
                .iter()
                .position(|h| h == k)
                .ok_or_else(|| anyhow::anyhow!("{sheet}: column {k:?} missing"))
        };
        let (i_act, i_vd, i_st, i_ar, i_t) = (
            ix("Actual Date  (for SPICE Display Date, add 1 for Leap Year)")?,
            ix("Date on videos and images")?,
            ix("Approx Start Time ( with GMT)")?,
            ix("As run test no.")?,
            ix("Test no.")?,
        );
        let opt = |k: &str| header.iter().position(|h| h == k);
        let (i_o2, i_p) = (opt(O2_ASRUN), opt(P_ASRUN));
        let num = |r: &Vec<Data>, i: Option<usize>| match i.and_then(|i| r.get(i)) {
            Some(Data::Float(x)) => Some(*x),
            Some(Data::Int(x)) => Some(*x as f64),
            Some(Data::String(s)) => strip(s).parse::<f64>().ok(),
            _ => None,
        };
        for (k, r) in rows.iter().enumerate().skip(hi + 1) {
            if is_empty(r.get(i_act)) {
                continue;
            }
            let video_date = match r.get(i_vd) {
                Some(Data::String(s)) => NaiveDate::parse_from_str(strip(s), "%m/%d/%Y").ok(),
                Some(Data::DateTime(dt)) => dt.as_datetime().map(|t| t.date()),
                _ => None,
            };
            let st = r
                .get(i_st)
                .filter(|d| !is_empty(Some(d)))
                .map(cell_str)
                .unwrap_or_default();
            out.push(AsRun {
                sheet: sheet.into(),
                row: k as i64 + 1,
                asrun: strip(&r.get(i_ar).map(cell_str).unwrap_or_else(|| "None".into()))
                    .to_string(),
                test: strip(&r.get(i_t).map(cell_str).unwrap_or_else(|| "None".into())).to_string(),
                video_date,
                start: first_clock(&st),
                o2: num(r, i_o2),
                pressure_atm: num(r, i_p),
            });
        }
    }
    Ok(out)
}

pub fn ingest() -> anyhow::Result<Bundle> {
    let mut b = Bundle::new(FAM);
    let (s25, s26) = (mf::by_name("PSI-25")?, mf::by_name("PSI-26")?);
    let runs25 = bass2(&mut b, &s25)?;
    let runs26 = if s26.contains_key(T26) {
        bass1(&mut b, &s26)?
    } else {
        vec![]
    };
    let asrun = match s26.get(ASRUN26) {
        Some(src) => {
            let rows = parse_asrun26(&mf::local_path(src))?;
            atmosphere26(&mut b, &rows, &mf::sha_of(src));
            rows
        }
        None => vec![],
    };
    let listing = |inv: &str| -> anyhow::Result<Vec<RemoteFile>> {
        Ok(
            read_listing(&paths::manifests().join(format!("{inv}.listing.jsonl")))?
                .into_iter()
                .filter(|f| f.category == "Raw Data")
                .collect(),
        )
    };
    let v = map_videos(
        &listing("PSI-25")?,
        &runs25,
        &listing("PSI-26")?,
        &asrun,
        &runs26,
    );
    let mut listing_sha = HashMap::new();
    for inv in ["PSI-25", "PSI-26"] {
        listing_sha.insert(
            inv,
            ignis_core::hashing::file_sha256(
                &paths::manifests().join(format!("{inv}.listing.jsonl")),
            )?,
        );
    }
    for x in &v {
        if x.resolution != "resolved" && x.kind == "video" {
            let mut ctx = serde_json::Map::new();
            ctx.insert("stable_url".into(), json!(x.stable_url));
            ctx.insert(
                "note".into(),
                json!("locator = the committed listing manifest entry; video bytes not fetched"),
            );
            b.review.push(ReviewItem {
                id: format!("video:{}", x.file_name),
                family: FAM.into(),
                run_id: x.run_id.clone().unwrap_or_default(),
                field: "video_run".into(),
                question: "Which run (test number) and specimen does this video record?".into(),
                cited_text: x.file_name.clone(),
                locator: SourceLocator {
                    column: Some(x.file_name.clone()),
                    ..loc(
                        &listing_sha[x.investigation.as_str()],
                        &format!("{}.listing.jsonl", x.investigation),
                        None,
                        None,
                    )
                },
                candidates: ["<run_id>", "not_a_test_recording", "unclear"]
                    .map(String::from)
                    .to_vec(),
                deterministic_screen: Some(x.evidence.clone()),
                context: ctx,
            });
        }
    }
    b.extra.insert(
        "video_map".into(),
        v.iter().map(|x| serde_json::to_value(x).unwrap()).collect(),
    );
    Ok(b)
}

pub const COHORT_CRITERIA: &[(&str, &str)] = &[
    ("investigation", "PSI-25 (BASS-II) only; PSI-26 is a related but different protocol, never pooled"),
    ("material", "SIBAL (75/25 cotton/fiberglass fabric), planar_fabric geometry"),
    ("video", "at least one resolved main-view MP4 mapped to the run"),
    ("qualification", "DECLARED CANDIDATE, NOT QUALIFIED: pixel->m calibration, plane validity, phase and flow history are \
                       not established; the vision lane must qualify each run"),
];

pub fn video_map(b: &Bundle) -> Vec<VideoMap> {
    b.extra
        .get("video_map")
        .into_iter()
        .flatten()
        .filter_map(|v| serde_json::from_value(v.clone()).ok())
        .collect()
}

pub fn cohort_filter(b: &Bundle) -> Kept {
    let mapped: HashSet<String> = video_map(b)
        .into_iter()
        .filter(|v| v.resolution == "resolved" && v.kind == "video" && v.variant == "main")
        .filter_map(|v| v.run_id)
        .collect();
    let (mut keep, mut out) = (vec![], vec![]);
    for r in &b.records {
        let cfg = r.configuration.as_deref().unwrap_or("None");
        if r.investigation_id != "PSI-25" {
            out.push((
                r.id.clone(),
                "PSI-26 protocol not pooled with BASS-II".into(),
            ));
        } else if !lower(r.fuel_or_material.as_deref().unwrap_or("")).contains("sibal")
            || cfg != "planar_fabric"
        {
            out.push((
                r.id.clone(),
                format!("material/geometry {cfg}: not planar SIBAL"),
            ));
        } else if !mapped.contains(&r.id) {
            out.push((r.id.clone(), "no resolved main-view video".into()));
        } else {
            keep.push(r.id.clone());
        }
    }
    (keep, out)
}
