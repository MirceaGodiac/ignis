//! JAXA DARTS — ISS Kibo PM "Group Combustion" (GCEM) experiment.
//!
//! Catalogue: <https://darts.isas.jaxa.jp/en/datasets/kibo-pm-combusion-gc> (id `darts:kibo-pm-combusion-gc`);
//! raw files: <https://data.darts.isas.jaxa.jp/pub/kibo/gc/> (Apache autoindex, plain HTTP GET/HEAD).
//!
//! Real structure (inspected, see `data/manifests/intl/JAXA-GCEM.jsonl`):
//! `documents/` 5 PDFs + `telemetry_gc.tsv` (one row per telemetry ZIP) + `video_gc.tsv` (one row per
//! video ZIP); `telemetry/` 159 ZIPs `{YYMMDD}-{NN}_{type}-{pattern}.zip` (~72-80 KB), each holding one
//! header-less CSV; `video/` 160 directories `{NNN}/` with `MVI_{NNNN}*.zip` (~250-400 MB) and, for 25
//! runs, a GCEM High Speed Camera ZIP.
//!
//! Telemetry CSV columns are fixed by `documents/gcem_measurement_data_format.pdf` (18 columns):
//! year, month, day, hour, minute, sec, msec, Pressure Sensor 1..4 [kPa], Temp Sensor 1..2 [degC],
//! Constant Current Circuit Voltage [V], Igniter 1..4 status ("OFF (Hi)" / "ON (Low)"). Timestamps are
//! GMT. Rows arrive at ~100 Hz with millisecond stamps quantised to ~15.6 ms, so several rows share
//! one stamp; the row number therefore stays part of every observation id.

use std::io::Read;
use std::sync::OnceLock;

use chrono::{NaiveDate, NaiveDateTime};
use ignis_core::contracts::{
    Alias, ControlStage, Derivation, EvidenceClass, ExperimentRecord, Family, IdBasis,
    Investigation, Method, Observation, Quantity, RawValue, Resolution, Rights, RightsStatus,
    SourceKind, SourceLocator,
};
use regex::Regex;
use serde_json::{json, Value};

use super::common::{self, entry, iso, z};
use crate::sources::http::sha256_bytes;
use crate::text::{py_csv, py_dict_reader};

pub const INVESTIGATION_ID: &str = "JAXA-GCEM";
pub const CATALOG_URL: &str = "https://darts.isas.jaxa.jp/en/datasets/kibo-pm-combusion-gc";
pub const CATALOG_ID: &str = "darts:kibo-pm-combusion-gc";
pub const BASE: &str = "https://data.darts.isas.jaxa.jp/pub/kibo/gc/";
pub const POLICY_URL: &str = "https://www.isas.jaxa.jp/en/researchers/data-policy/";
pub const JAXA_POLICY_URL: &str = "https://global.jaxa.jp/policy.html";
const PARSER_VERSION: &str = "1";

const PRESSURE_COLS: [&str; 4] = [
    "pressure_sensor_1",
    "pressure_sensor_2",
    "pressure_sensor_3",
    "pressure_sensor_4",
];
const TEMP_COLS: [&str; 2] = ["temp_sensor_1", "temp_sensor_2"];
const N_COLS: usize = 18;
const IGNITERS: [&str; 4] = ["igniter_1", "igniter_2", "igniter_3", "igniter_4"];

#[derive(Debug, Clone)]
pub struct IndexEntry {
    pub url: String,
    pub name: String,
    pub is_dir: bool,
    pub last_modified: Option<String>,
    /// Autoindex label, rounded ("72K", "246M"); exact size comes from HEAD.
    pub size_label: Option<String>,
}

fn unescape(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}

pub fn parse_index(html: &str, base_url: &str) -> Vec<IndexEntry> {
    static R: OnceLock<Regex> = OnceLock::new();
    let rx = R.get_or_init(|| {
        Regex::new(r#"<a href="(?P<href>[^"?/][^"]*)">[^<]*</a>\s*</td>\s*<td[^>]*>\s*(?P<date>[0-9]{4}-[0-9]{2}-[0-9]{2} [0-9]{2}:[0-9]{2})?[^<]*</td>\s*<td[^>]*>\s*(?P<size>[^<]*?)\s*</td>"#).unwrap()
    });
    let mut out = vec![];
    for m in rx.captures_iter(html) {
        let href = unescape(&m["href"]);
        if href.starts_with('/') {
            continue; // parent directory
        }
        let size = m["size"].replace("&nbsp;", "").trim().to_string();
        out.push(IndexEntry {
            url: format!("{base_url}{href}"),
            name: href.trim_end_matches('/').to_string(),
            is_dir: href.ends_with('/'),
            last_modified: m.name("date").map(|d| d.as_str().to_string()),
            size_label: if size.is_empty() || size == "-" {
                None
            } else {
                Some(size)
            },
        });
    }
    out
}

pub fn file_kind(name: &str, parent: &str) -> &'static str {
    match parent {
        "telemetry" => "telemetry_zip",
        "video" => "video_zip",
        _ if name.ends_with(".tsv") => "index_tsv",
        _ if name.ends_with(".pdf") => "document_pdf",
        _ => "other",
    }
}

/// `170216-01_2-1` -> `JAXA-GCEM:170216-01` (date + sequence of that day; one telemetry file per run).
pub fn run_id(file_stem: &str) -> String {
    format!(
        "{INVESTIGATION_ID}:{}",
        file_stem.split('_').next().unwrap_or(file_stem)
    )
}

fn dt(s: &str) -> Option<NaiveDateTime> {
    NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").ok()
}

/// One ExperimentRecord per telemetry file; videos joined by param_code and overlapping GMT window.
pub fn experiment_records(
    tele: &[indexmap::IndexMap<String, String>],
    video: &[indexmap::IndexMap<String, String>],
) -> Vec<ExperimentRecord> {
    let g = |r: &indexmap::IndexMap<String, String>, k: &str| r.get(k).cloned().unwrap_or_default();
    let mut recs = vec![];
    for t in tele {
        let stem = g(t, "file_name");
        let (t0, t1) = (dt(&g(t, "start_datetime")), dt(&g(t, "end_datetime")));
        let mut aliases = vec![
            Alias {
                name: stem.clone(),
                scheme: "darts_telemetry_file".into(),
                locator: None,
            },
            Alias {
                name: g(t, "telemetry_id"),
                scheme: "darts_telemetry_id".into(),
                locator: None,
            },
            Alias {
                name: g(t, "param_code"),
                scheme: "gcem_param_code".into(),
                locator: None,
            },
        ];
        for v in video {
            if g(v, "param_code") != g(t, "param_code") {
                continue;
            }
            if let (Some(v0), Some(v1), Some(t0), Some(t1)) = (
                dt(&g(v, "start_datetime")),
                dt(&g(v, "end_datetime")),
                t0,
                t1,
            ) {
                if v0 <= t1 && v1 >= t0 {
                    aliases.push(Alias {
                        name: format!(
                            "video/{}/{}.{}",
                            g(v, "directory_num"),
                            g(v, "file_name"),
                            g(v, "video_file_ext")
                        ),
                        scheme: format!("darts_video:{}", g(v, "camera_name")),
                        locator: None,
                    });
                }
            }
        }
        let flags = if aliases.len() > 3 {
            vec![]
        } else {
            vec!["no_video_matched".to_string()]
        };
        let pc = g(t, "param_code");
        recs.push(ExperimentRecord {
            id: run_id(&stem), investigation_id: INVESTIGATION_ID.into(), family: Family::Other, session_id: None, specimen_id: None,
            aliases, fuel_or_material: None,
            configuration: Some(format!("GCEM experiment type {}", pc.split('-').nth(1).unwrap_or(""))),
            gravity: None, conditions: Default::default(), identity_resolution: Resolution::Resolved,
            identity_evidence: vec![Derivation {
                method: Method::Deterministic, method_id: "intl.darts.run_identity".into(), method_version: PARSER_VERSION.into(),
                inputs: vec![], agent: None, notes: Some("run = telemetry file; videos joined on param_code + GMT overlap".into()),
            }],
            quality_flags: flags,
            notes: Some("fuel not stated in the DARTS documents inspected; droplet size/number/O2 per run are in \
                         documents/gc_parameter.pdf (not yet parsed)".into()),
        });
    }
    recs
}

#[derive(Debug, Clone)]
pub struct TelemetryParse {
    pub run_id: String,
    pub t0: NaiveDateTime,
    pub observations: Vec<Observation>,
    pub controls: Vec<ControlStage>,
    /// Channels without a contract Quantity: (row, time_s, constant_current_circuit_voltage_V).
    pub auxiliary: Vec<(i64, f64, f64)>,
    pub n_rows: usize,
}

pub fn read_telemetry_zip(data: &[u8]) -> anyhow::Result<(String, Vec<u8>)> {
    let mut z = zip::ZipArchive::new(std::io::Cursor::new(data))?;
    let names: Vec<String> = z
        .file_names()
        .filter(|n| n.to_lowercase().ends_with(".csv"))
        .map(String::from)
        .collect();
    anyhow::ensure!(
        names.len() == 1,
        "expected one CSV in telemetry ZIP, found {names:?}"
    );
    let mut body = vec![];
    z.by_name(&names[0])?.read_to_end(&mut body)?;
    Ok((names[0].clone(), body))
}

/// Header-less GCEM CSV -> Observations (pressure, temperature) + igniter ControlStages.
///
/// `time_s` is seconds since the first row of the file (t=0 is the start of the telemetry window,
/// not ignition); `available_at_s == time_s` (no downlink latency is modelled). Pressure sensors are
/// stored as reported; whether they are absolute or gauge is not stated by the source (flag).
pub fn parse_telemetry_csv(
    csv_bytes: &[u8],
    asset_sha256: &str,
    path: &str,
    run: &str,
    max_rows: Option<usize>,
) -> anyhow::Result<TelemetryParse> {
    let der = Derivation {
        method: Method::Deterministic,
        method_id: "intl.darts.gcem_telemetry_csv".into(),
        method_version: PARSER_VERSION.into(),
        inputs: vec![format!("sha256:{asset_sha256}")],
        agent: None,
        notes: Some(
            "columns per documents/gcem_measurement_data_format.pdf; kPa*1e3->Pa; degC+273.15->K"
                .into(),
        ),
    };
    let text = std::str::from_utf8(csv_bytes)?;
    let mut out: Option<TelemetryParse> = None;
    let mut on_since: indexmap::IndexMap<&str, Option<f64>> = indexmap::IndexMap::new();
    for (i, row) in py_csv(text).into_iter().enumerate() {
        if max_rows.is_some_and(|m| i >= m) {
            break;
        }
        anyhow::ensure!(
            row.len() == N_COLS,
            "{path} row {i}: {} columns, expected {N_COLS}",
            row.len()
        );
        let n: Vec<i64> = row[..7]
            .iter()
            .map(|x| x.trim().parse())
            .collect::<Result<_, _>>()?;
        let ts = NaiveDate::from_ymd_opt(n[0] as i32, n[1] as u32, n[2] as u32)
            .and_then(|d| {
                d.and_hms_micro_opt(n[3] as u32, n[4] as u32, n[5] as u32, (n[6] * 1000) as u32)
            })
            .ok_or_else(|| anyhow::anyhow!("{path} row {i}: bad timestamp"))?;
        let o = out.get_or_insert_with(|| TelemetryParse {
            run_id: run.into(),
            t0: ts,
            observations: vec![],
            controls: vec![],
            auxiliary: vec![],
            n_rows: 0,
        });
        let t = (ts - o.t0).num_microseconds().unwrap() as f64 / 1e6;
        let l = |col: &str| SourceLocator {
            asset_sha256: asset_sha256.into(),
            path: Some(path.into()),
            row: Some(i as i64),
            column: Some(col.into()),
            page: None,
            frame: None,
            pts_s: None,
            char_start: None,
            char_end: None,
            ..Default::default()
        };
        for (j, col) in PRESSURE_COLS.iter().enumerate() {
            let raw: f64 = row[7 + j].trim().parse()?;
            o.observations.push(Observation {
                id: format!("{run}:{col}:{i}"),
                run_id: run.into(),
                quantity: Quantity::Pressure,
                channel: Some(format!("GCEM.{col}")),
                time_s: t,
                available_at_s: t,
                raw_value: Some(RawValue::Number(raw)),
                raw_unit: Some("kPa".into()),
                value_si: Some(raw * 1e3),
                unit_si: "Pa".into(),
                uncertainty_si: None,
                calibration_id: None,
                quality_flags: vec!["pressure_reference_unstated".into()],
                evidence_class: EvidenceClass::OriginalObservation,
                locator: l(col),
                derivation: der.clone(),
            });
        }
        for (j, col) in TEMP_COLS.iter().enumerate() {
            let raw: f64 = row[11 + j].trim().parse()?;
            o.observations.push(Observation {
                id: format!("{run}:{col}:{i}"),
                run_id: run.into(),
                quantity: Quantity::Temperature,
                channel: Some(format!("GCEM.{col}")),
                time_s: t,
                available_at_s: t,
                raw_value: Some(RawValue::Number(raw)),
                raw_unit: Some("degC".into()),
                value_si: Some(raw + 273.15),
                unit_si: "K".into(),
                uncertainty_si: None,
                calibration_id: None,
                quality_flags: vec!["sensor_location_unstated".into()],
                evidence_class: EvidenceClass::OriginalObservation,
                locator: l(col),
                derivation: der.clone(),
            });
        }
        o.auxiliary.push((i as i64, t, row[13].trim().parse()?));
        for (j, k) in IGNITERS.iter().enumerate() {
            let on = row[14 + j].trim().to_uppercase().starts_with("ON");
            let since = on_since.get(k).copied().flatten();
            if on && since.is_none() {
                on_since.insert(k, Some(t));
            } else if !on && since.is_some() {
                o.controls.push(ControlStage {
                    t_start_s: since.unwrap(),
                    t_end_s: Some(t),
                    control: format!("{k}_energized"),
                    value: None,
                    operator_action: false,
                });
                on_since.insert(k, None);
            }
        }
        o.n_rows = i + 1;
    }
    let mut out = out.ok_or_else(|| anyhow::anyhow!("{path}: empty telemetry file"))?;
    for (k, t_on) in on_since {
        if let Some(t) = t_on {
            out.controls.push(ControlStage {
                t_start_s: t,
                t_end_s: None,
                control: format!("{k}_energized"),
                value: None,
                operator_action: false,
            });
        }
    }
    Ok(out)
}

pub fn rights(policy_sha256: Option<&str>) -> Rights {
    Rights {
        license: Some("ISAS Data Policy 2018-03-14 sec.6 (free use incl. commercial; credit 'ISAS/JAXA'; declare modifications; \
                       third-party co-producers may add restrictions; stated CC BY 4.0-compatible)".into()),
        // dataset page links a policy, not a dataset licence; co-creators include NASA/universities
        status: RightsStatus::Unverified,
        statement_locator: policy_sha256.map(|s| SourceLocator {
            asset_sha256: s.into(), path: Some(format!("{POLICY_URL}#6")), row: None, column: None, page: None, frame: None,
            pts_s: None, char_start: None, char_end: None,
            ..Default::default()
        }),
    }
}

pub fn build_manifest(head: bool) -> anyhow::Result<Vec<Value>> {
    let retrieved = common::now();
    let c = common::client();
    let policy = c.get(POLICY_URL, &[])?.bytes;
    let policy_sha = sha256_bytes(&policy);
    let catalog = c.get(CATALOG_URL, &[])?.bytes;
    let r = rights(Some(&policy_sha));
    let mut rows: Vec<Value> = vec![];
    let mut pages = vec![];
    let mut p1 = common::source(
        format!("sha256:{policy_sha}"),
        IdBasis::Bytes,
        SourceKind::Webpage,
        INVESTIGATION_ID,
        Rights {
            license: None,
            status: RightsStatus::Verified,
            statement_locator: None,
        },
    );
    p1.title = Some("ISAS Data Policy".into());
    p1.canonical_url = Some(POLICY_URL.into());
    p1.size_bytes = Some(policy.len() as i64);
    p1.media_type = Some("text/html".into());
    p1.retrieved_at = Some(z(&retrieved));
    let mut p2 = common::source(
        format!("sha256:{}", sha256_bytes(&catalog)),
        IdBasis::Bytes,
        SourceKind::Webpage,
        INVESTIGATION_ID,
        r.clone(),
    );
    p2.title = Some("DARTS catalogue: ISS Kibo PM Experiment - Group Combustion Experiment".into());
    p2.canonical_url = Some(CATALOG_URL.into());
    p2.size_bytes = Some(catalog.len() as i64);
    p2.media_type = Some("text/html".into());
    p2.retrieved_at = Some(z(&retrieved));
    pages.push(p1);
    pages.push(p2);
    for p in &pages {
        rows.push(entry("source", p, Some(IdBasis::Bytes), vec![]));
    }
    let listing = |url: &str| -> anyhow::Result<Vec<IndexEntry>> {
        Ok(parse_index(
            &String::from_utf8_lossy(&c.get(url, &[])?.bytes),
            url,
        ))
    };
    let size_of = |url: &str| -> anyhow::Result<Option<i64>> {
        Ok(if head {
            c.head_size(url)?.map(|x| x as i64)
        } else {
            None
        })
    };
    let mut crawled: Vec<(IndexEntry, &'static str, Option<i64>)> = vec![];
    for top in listing(BASE)? {
        if !top.is_dir {
            let s = size_of(&top.url)?;
            crawled.push((top, "other", s));
            continue;
        }
        for e in listing(&top.url)? {
            if e.is_dir {
                for v in listing(&e.url)? {
                    if !v.is_dir {
                        let s = size_of(&v.url)?;
                        crawled.push((v.clone(), file_kind(&v.name, &top.name), s));
                    }
                }
            } else {
                let s = size_of(&e.url)?;
                crawled.push((e.clone(), file_kind(&e.name, &top.name), s));
            }
        }
    }
    let mut files = vec![];
    let mut tsv: std::collections::HashMap<String, String> = Default::default();
    let (mut n_tele, mut n_video) = (0, 0);
    for (e, kind, mut size) in crawled {
        let rel = e.url[BASE.len()..].to_string();
        let mut sha = common::local_sha256(&common::raw_path(INVESTIGATION_ID, &rel));
        if kind == "index_tsv" {
            let body = c.get(&e.url, &[])?.bytes;
            tsv.insert(e.name.clone(), String::from_utf8_lossy(&body).into_owned());
            sha = Some(sha256_bytes(&body));
            size = Some(body.len() as i64);
        }
        let basis = if sha.is_some() {
            IdBasis::Bytes
        } else {
            IdBasis::Listing
        };
        let sid = match &sha {
            Some(s) => format!("sha256:{s}"),
            None => common::listing_id(json!({"url": e.url, "size": size})),
        };
        let media = match kind {
            "telemetry_zip" | "video_zip" => Some("application/zip"),
            "index_tsv" => Some("text/tab-separated-values"),
            "document_pdf" => Some("application/pdf"),
            _ => None,
        };
        let mut s = common::source(
            sid,
            basis,
            if kind == "document_pdf" {
                SourceKind::Protocol
            } else {
                SourceKind::DatasetFile
            },
            INVESTIGATION_ID,
            r.clone(),
        );
        s.canonical_url = Some(e.url.clone());
        s.file_name = Some(rel);
        s.size_bytes = size;
        s.media_type = media.map(String::from);
        s.retrieved_at = sha.as_ref().map(|_| z(&retrieved));
        n_tele += (kind == "telemetry_zip") as usize;
        n_video += (kind == "video_zip") as usize;
        rows.push(entry(
            "source",
            &s,
            Some(basis),
            vec![
                ("kind", json!(kind)),
                ("last_modified", json!(e.last_modified)),
                ("size_label", json!(e.size_label)),
                (
                    "size_from",
                    if head {
                        json!("HEAD content-length")
                    } else {
                        Value::Null
                    },
                ),
                ("listed_at", json!(iso(&retrieved))),
            ],
        ));
        files.push(s);
    }
    let strip_bom = |s: &str| s.trim_start_matches('\u{feff}').to_string();
    let tele = py_dict_reader(
        &strip_bom(
            tsv.get("telemetry_gc.tsv")
                .map(|s| s.as_str())
                .unwrap_or(""),
        ),
        '\t',
    );
    let video = py_dict_reader(
        &strip_bom(tsv.get("video_gc.tsv").map(|s| s.as_str()).unwrap_or("")),
        '\t',
    );
    let inv = Investigation {
        id: INVESTIGATION_ID.into(),
        name: "ISS Kibo PM Experiment - Group Combustion Experiment (GCEM)".into(),
        platform: Some("ISS (Kibo JEM-PM, Chamber for Combustion Experiment)".into()),
        families: vec![Family::Other],
        source_ids: pages
            .iter()
            .chain(files.iter())
            .map(|s| s.id.clone())
            .collect(),
    };
    let total: i64 = files.iter().filter_map(|s| s.size_bytes).sum();
    rows.insert(0, entry("investigation", &inv, None, vec![
        ("catalog_id", json!(CATALOG_ID)), ("catalog_url", json!(CATALOG_URL)), ("raw_root", json!(BASE)),
        ("temporal_coverage_gmt", json!("2017-02-16/2017-07-04")), ("publication_year", json!(2019)),
        ("creators", json!(["MIKAMI Masato (Yamaguchi U.)", "NOMURA Hiroshi (Nihon U.)", "MORIUE Osamu (Kyushu U.)",
            "UMEMURA Akira (Nagoya U.)", "SEO Takehiko (Yamaguchi U.)", "SUGANUMA Yusuke (Nihon U.)", "KIKUCHI Masao (JAXA)",
            "Dietrich, Daniel L. (NASA)"])),
        ("copyright_holder", json!("JAXA / ISAS")), ("license_links", json!([JAXA_POLICY_URL, POLICY_URL])),
        ("credit", json!("ISAS/JAXA")), ("separate_from_nasa_cohorts", json!(true)),
        ("counts", json!({"telemetry_zip": n_tele, "video_zip": n_video, "telemetry_tsv_rows": tele.len(), "video_tsv_rows": video.len()})),
        ("total_listed_bytes", json!(total)),
    ]));
    for rec in experiment_records(&tele, &video) {
        rows.push(entry("experiment", &rec, None, vec![]));
    }
    Ok(rows)
}
