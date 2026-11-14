//! ACME CFI-G (PSI-159) ingest: normal and inverse experimental tables, kept as separate families.
//!
//! Real source structure (PSI-159 v1, verified):
//!
//! * `PSI-159_Experimental table_CFI-G_Normal Flames.csv` (179 rows): `Flame Type, Day, Fuel, Test,
//!   Cool flame?, ' XO2 before test', 'XO2  after test', 'approx p (bar)', Xf, 'mHC (mg/s)', TFP, 'Tad (K)',
//!   Zst, Hot ignition, 'burn time (sec)', 'Self-ext or flow term', 'Peak burner T (C )', Comments`.
//!   Normal = fuel stream injected through the porous burner into the oxidizing chamber: `XO2` is the
//!   chamber (ambient) oxygen, `Xf` the fuel fraction of the injected stream, `mHC` the injected fuel flow.
//! * `..._Inverse Flames.csv` (1015 lines, 154 non-empty): `Xf before/after test` (chamber fuel), `XO2`
//!   (injected oxidizer stream O2), `HC burnrate (mg/s)`, `O2 flow rate (mg/s)` (injected),
//!   `Peak burner Temp (C)`/`(K)`, `Color`. Inverse = oxidizer injected into a fuel-bearing chamber.
//! * Test ids `21088 C1`: `YYDDD` (2021, day-of-year 088) + test point; the `Day` column is the campaign
//!   day index. Per-test logs (`<test>_data.csv`, `_TP.txt`, `_mfcSetpoints.csv`, `as run<test>.txt`)
//!   exist for timing adjudication.
//!
//! Documented abbreviations (column header): `SE` = self-extinguished, `FT` = flow terminated, `-` = no
//! established flame to terminate. `Hot ignition` is kept raw (Yes/yes/No/no/flash).

use std::collections::{BTreeMap, HashMap};
use std::sync::OnceLock;

use ignis_core::contracts::{
    Alias, Censoring, ControlHistory, ControlStage, EndpointRecord, EvidenceClass, Exclusion,
    ExperimentRecord, Family, Measured, Method, Resolution, Session, SourceLocator,
};
use ignis_core::interpret::FieldRule;
use ignis_core::prefix::{ProspectivePolicy, ALWAYS_FORBIDDEN};
use indexmap::IndexMap;
use regex::Regex;
use serde_json::json;

use super::{r, view_rules, Kept};
use crate::common::{det, loc, measured, parse_number, raw_str, text_value, Bundle, ReviewItem, M};
use crate::sources::manifest as mf;
use crate::text::{
    decode_utf8_sig_lossy, lower, normalize_newlines, py_repr, py_repr_list, strip, table_rows, Row,
};

pub const INV: &str = "PSI-159";
const V: &str = "1";
pub const NORMAL: &str = "PSI-159_Experimental table_CFI-G_Normal Flames.csv";
pub const INVERSE: &str = "PSI-159_Experimental table_CFI-G_Inverse Flames.csv";

const PROSPECTIVE: [&str; 12] = [
    "pressure",
    "injected_mass_flow",
    "x_o2_role",
    "x_fuel_role",
    "tad",
    "zst",
    "fuel_species",
    "fuel_bottle",
    "x_o2_ambient",
    "x_fuel_injected",
    "x_o2_injected",
    "x_fuel_ambient",
];
const FORBIDDEN: [&str; 10] = [
    "x_o2_after",
    "x_fuel_after",
    "burn_time",
    "se_ft",
    "peak_burner_temperature",
    "hot_ignition",
    "tfp",
    "hc_burnrate",
    "color",
    "cool_flame",
];

pub fn policy(fam: Family) -> ProspectivePolicy {
    let mut f: Vec<&str> = FORBIDDEN.to_vec();
    f.extend(ALWAYS_FORBIDDEN);
    ProspectivePolicy::new(fam, &PROSPECTIVE, &f)
}

pub fn rules(fam: Family) -> Vec<FieldRule> {
    use Method::*;
    let f = fam.as_str();
    let mut v = vec![
        r(f, "tad", Deterministic, "acme.cond.tad",
          "Tad is a source-computed adiabatic flame temperature of the admitted mixture: kept as a derived mixture \
           property (source_derived), never an independent control"),
        r(f, "zst", Deterministic, "acme.cond.zst", "stoichiometric mixture fraction, source-computed; same convention as Tad"),
        r(f, "tfp", HumanReview, "acme.tfp.excluded", "TFP is undefined in the published material: raw kept in post facts, quarantined"),
        r(f, "post_test", Deterministic, "acme.post",
          "after-test composition, burn time, SE/FT, peak burner T, hot ignition, color are post-test facts outside \
           conditions; forbidden in prefixes"),
    ];
    v.push(r(f, "comment_semantics", Semantic, "acme.comment_semantics.review",
        "free-text comment adjudicated into population/control/endpoint effect by two independent LLM passes with \
         cited evidence; non_spherical_or_irregular and other leave the cohort, intervention on an SE row makes the \
         endpoint an unresolved SE-vs-intervention conflict, data_loss is recorded in the observation state"));
    if fam == Family::AcmeNormal {
        v.extend([
            r(f, "test_id", Deterministic, "acme.identity.test_id",
              "'21088 C1' -> '21088C1'; YYDDD prefix -> UTC test day (session); raw spelling kept as alias"),
            r(f, "endpoint", Deterministic, "acme.endpoint.se_ft",
              "SE -> extinction, FT -> operator_termination (header abbreviation); burn time integer seconds -> \
               interval [t-0.5, t+0.5] from hot ignition (origin per table, unadjudicated); a comment documenting an \
               intervention/limit stop contradicting SE (e.g. 21088 C1) -> unresolved_conflict with both cited"),
            r(f, "conditions", Deterministic, "acme.cond.role_aware",
              "normal: x_o2_role = chamber XO2 before test, x_fuel_role = Xf of injected stream, injected_mass_flow = \
               mHC (fuel). bar/mg/s -> Pa/kg/s. Missing stays missing"),
            r(f, "control_history", Deterministic, "acme.tp.timeline",
              "<test>_TP.txt script timeline ('Ignition and retraction: <ts>', 'MFCs off: <ts>', '***' = step not run) \
               -> ControlHistory with t=0 at 'Ignition and retraction'; 'MFCs off' is a commanded flow stop \
               (operator_action=True). Wall-clock seconds; whether this origin equals the table burn-time origin is open"),
        ]);
    } else {
        v.extend([
            r(f, "test_id", Deterministic, "acme.identity.test_id", "same rule as normal"),
            r(f, "conditions", Deterministic, "acme.cond.role_aware",
              "inverse: x_o2_role = XO2 of injected oxidizer stream, x_fuel_role = chamber Xf before test, \
               injected_mass_flow = 'O2 flow rate (mg/s)'; 'HC burnrate' is post-factum, not a setpoint"),
        ]);
    }
    v.extend(view_rules(f));
    v.sort_by(|a, b| a.field.cmp(b.field));
    v
}

fn re(cell: &'static OnceLock<Regex>, pat: &str) -> &'static Regex {
    cell.get_or_init(|| Regex::new(pat).unwrap())
}

fn ws_norm(s: &str) -> String {
    static WS: OnceLock<Regex> = OnceLock::new();
    lower(strip(&re(&WS, r"\s+").replace_all(s, " ")))
}

/// Header lookup tolerant of the source's irregular whitespace (`' XO2 before test'`).
pub fn col<'a>(row: &'a Row, names: &[&str]) -> &'a str {
    let mut norm: HashMap<String, &str> = HashMap::new();
    for k in row.cells.keys() {
        norm.insert(ws_norm(k), k.as_str());
    }
    for n in names {
        if let Some(k) = norm.get(&ws_norm(n)) {
            return k;
        }
    }
    panic!("columns {names:?} missing")
}

/// `'21088 C1'` -> (`21088C1`, `21088`, `2021-03-29`).
pub fn test_identity(raw: &str) -> (String, Option<String>, Option<String>) {
    static WS: OnceLock<Regex> = OnceLock::new();
    static ID: OnceLock<Regex> = OnceLock::new();
    let t = re(&WS, r"\s+").replace_all(raw, "").to_string();
    let Some(m) = re(&ID, r"^(\d{2})(\d{3})([A-Z].*)$").captures(&t) else {
        return (t, None, None);
    };
    let (yy, ddd): (i32, i64) = (m[1].parse().unwrap(), m[2].parse().unwrap());
    let day =
        chrono::NaiveDate::from_ymd_opt(2000 + yy, 1, 1).unwrap() + chrono::Duration::days(ddd - 1);
    let prefix = format!("{}{}", &m[1], &m[2]);
    (t, Some(prefix), Some(day.to_string()))
}

fn intervention() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    re(
        &R,
        r"(?i)(burner\s*t(emp(erature)?)?\s*limit|hit (the )?(burner )?t(emp(erature)?)? limit|temperature limit|manually stopped|flow cut|fc\s*=|terminated|aborted|timed out|time[d]? out|fuel flow stopped|flow timed out)",
    )
}

fn no_ignition() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    re(
        &R,
        r"(?i)(no ignition|not ignited|failed to ignite|did not ignite)",
    )
}

fn non_empty(s: &str) -> Option<String> {
    (!s.is_empty()).then(|| s.to_string())
}

fn endpoint_base(
    run_id: &str,
    label: Option<String>,
    comment: Option<String>,
    causes: Vec<String>,
    censoring: Censoring,
    resolution: Resolution,
    ev: Vec<SourceLocator>,
    notes: String,
) -> EndpointRecord {
    EndpointRecord {
        run_id: run_id.into(),
        original_label: label,
        original_comment: comment,
        allowed_causes: causes,
        censoring,
        last_confirmed_active_s: None,
        first_terminal_s: None,
        intervention_s: None,
        intervention_kind: None,
        observation_state: None,
        resolution,
        evidence: ev,
        derivation: det("acme.endpoint.se_ft", V, vec![], Some(notes)),
    }
}

pub fn endpoint(
    run_id: &str,
    row: &Row,
    sha: &str,
    path: &str,
) -> (EndpointRecord, Option<ReviewItem>) {
    static INT: OnceLock<Regex> = OnceLock::new();
    let k_lab = col(row, &["Self-ext or flow term", "Self -ext or flow term"]);
    let k_bt = col(row, &["burn time (sec)"]);
    let k_hot = col(row, &["Hot ignition"]);
    let label = strip(row.get(k_lab)).to_string();
    let comment = strip(row.opt("Comments")).to_string();
    let hot = lower(strip(row.get(k_hot)));
    let ev = vec![
        loc(sha, path, Some(row.row), Some(k_lab)),
        loc(sha, path, Some(row.row), Some(k_bt)),
        loc(sha, path, Some(row.row), Some("Comments")),
    ];
    let bt_raw = strip(row.get(k_bt)).to_string();
    let bt = parse_number(&bt_raw);
    let (mut lo, mut hi) = (None, None);
    if let Some(bt) = bt.filter(|b| *b > 0.0) {
        if re(&INT, r"^\d+$").is_match(&bt_raw) {
            (lo, hi) = (Some((bt - 0.5).max(0.0)), Some(bt + 0.5));
        } else {
            (lo, hi) = (Some(bt), Some(bt));
        }
    }
    let notes = "time origin: hot ignition as implied by 'burn time'; not adjudicated against data.csv/video".to_string();
    if hot != "yes" {
        return (
            endpoint_base(
                run_id,
                non_empty(&label),
                non_empty(&comment),
                vec![],
                Censoring::Unknown,
                Resolution::NotApplicable,
                ev,
                format!(
                    "hot ignition = {}: no established flame population",
                    py_repr(row.get(k_hot))
                ),
            ),
            None,
        );
    }
    let causes: Vec<String> = match label.as_str() {
        "SE" => vec!["extinction".into()],
        "FT" => vec!["operator_termination".into()],
        _ => vec![],
    };
    let conflict = label == "SE" && intervention().is_match(&comment);
    let mut review = None;
    if conflict || (!comment.is_empty() && causes.is_empty()) {
        let mut ctx = serde_json::Map::new();
        ctx.insert("burn_time_s".into(), json!(bt_raw));
        ctx.insert("hot_ignition".into(), json!(row.get(k_hot)));
        review = Some(ReviewItem {
            id: format!("{run_id}#endpoint"), family: "acme".into(), run_id: run_id.into(), field: "endpoint_cause".into(),
            question: "Was the end of the established flame spontaneous extinction, an operator/automatic flow termination \
                       (incl. burner temperature limit), or unclear? Give the time if the log shows it.".into(),
            cited_text: format!("label={}; comment={}", py_repr(&label), py_repr(&comment)),
            locator: loc(sha, path, Some(row.row), Some("Comments")),
            candidates: ["extinction", "operator_termination", "both_possible", "unclear"].map(String::from).to_vec(),
            deterministic_screen: Some(if conflict { "SE label but comment documents an intervention/limit" } else { "no SE/FT label for an ignited test" }.into()),
            context: ctx,
        });
    }
    if conflict {
        let mut e = endpoint_base(
            run_id,
            Some(label),
            Some(comment),
            vec!["extinction".into(), "operator_termination".into()],
            Censoring::Interval,
            Resolution::UnresolvedConflict,
            ev,
            notes,
        );
        e.last_confirmed_active_s = lo;
        e.first_terminal_s = hi;
        return (e, review);
    }
    if causes.is_empty() || lo.is_none() {
        return (
            endpoint_base(
                run_id,
                non_empty(&label),
                non_empty(&comment),
                causes,
                Censoring::Unknown,
                Resolution::Unknown,
                ev,
                notes,
            ),
            review,
        );
    }
    let interv = intervention().is_match(&comment) || label == "FT";
    let ft = label == "FT";
    let mut e = endpoint_base(
        run_id,
        Some(label),
        non_empty(&comment),
        causes,
        if lo == hi {
            Censoring::Event
        } else {
            Censoring::Interval
        },
        Resolution::Resolved,
        ev,
        notes,
    );
    e.last_confirmed_active_s = lo;
    e.first_terminal_s = hi;
    e.intervention_s = if ft { hi } else { None };
    e.intervention_kind = if interv && ft {
        Some("flow_termination".into())
    } else {
        None
    };
    (e, review)
}

pub fn fuel_species(raw: &str) -> (Option<String>, Option<f64>) {
    static R: OnceLock<Regex> = OnceLock::new();
    let Some(m) = re(&R, r"^\s*(?:(\d+(?:\.\d+)?)\s*%\s*)?([A-Za-z0-9]+)").captures(raw) else {
        return (None, None);
    };
    (
        Some(lower(&m[2])),
        m.get(1).map(|x| x.as_str().parse::<f64>().unwrap() / 100.0),
    )
}

fn conditions_normal(row: &Row, sha: &str, path: &str) -> BTreeMap<String, Measured> {
    let rl = |c: &str| Some(loc(sha, path, Some(row.row), Some(c)));
    let k_o2 = col(row, &["XO2 before test"]);
    let k_p = col(row, &["approx p (bar)"]);
    let k_m = col(row, &["mHC (mg/s)"]);
    let mut c = BTreeMap::new();
    c.insert(
        "pressure".into(),
        measured(
            row.get(k_p),
            Some("bar"),
            rl(k_p),
            M::new("acme.cond.pressure")
                .notes("header says 'approx'")
                .flags(&["approximate_per_header"]),
        ),
    );
    c.insert(
        "x_o2_ambient".into(),
        measured(
            row.get(k_o2),
            Some("1"),
            rl(k_o2),
            M::new("acme.cond.x_o2_ambient"),
        ),
    );
    c.insert(
        "x_fuel_injected".into(),
        measured(
            row.get("Xf"),
            Some("1"),
            rl("Xf"),
            M::new("acme.cond.xf_injected"),
        ),
    );
    c.insert(
        "injected_mass_flow".into(),
        measured(
            row.get(k_m),
            Some("mg/s"),
            rl(k_m),
            M::new("acme.cond.injected_fuel_flow").notes("injected fuel (normal)"),
        ),
    );
    c.insert("x_o2_role".into(), c["x_o2_ambient"].clone());
    c.insert("x_fuel_role".into(), c["x_fuel_injected"].clone());
    c
}

fn conditions_inverse(row: &Row, sha: &str, path: &str) -> BTreeMap<String, Measured> {
    let rl = |c: &str| Some(loc(sha, path, Some(row.row), Some(c)));
    let k_xf = col(row, &["Xf before test"]);
    let k_p = col(row, &["approx p (bar)"]);
    let k_m = col(row, &["O2 flow rate (mg/s)"]);
    let mut c = BTreeMap::new();
    c.insert(
        "pressure".into(),
        measured(
            row.get(k_p),
            Some("bar"),
            rl(k_p),
            M::new("acme.cond.pressure").flags(&["approximate_per_header"]),
        ),
    );
    c.insert(
        "x_fuel_ambient".into(),
        measured(
            row.get(k_xf),
            Some("1"),
            rl(k_xf),
            M::new("acme.cond.xf_ambient"),
        ),
    );
    c.insert(
        "x_o2_injected".into(),
        measured(
            row.get("XO2"),
            Some("1"),
            rl("XO2"),
            M::new("acme.cond.xo2_injected"),
        ),
    );
    c.insert(
        "injected_mass_flow".into(),
        measured(
            row.get(k_m),
            Some("mg/s"),
            rl(k_m),
            M::new("acme.cond.injected_o2_flow").notes("injected O2 (inverse)"),
        ),
    );
    c.insert("x_o2_role".into(), c["x_o2_injected"].clone());
    c.insert("x_fuel_role".into(), c["x_fuel_ambient"].clone());
    c
}

fn common_conditions(row: &Row, sha: &str, path: &str) -> Vec<(String, Measured)> {
    let rl = |c: &str| Some(loc(sha, path, Some(row.row), Some(c)));
    let (sp, bottle) = fuel_species(row.get("Fuel"));
    let k_tad = col(row, &["Tad (K)"]);
    vec![
        (
            "fuel_species".into(),
            text_value(
                sp.as_deref(),
                rl("Fuel"),
                "acme.cond.fuel_species",
                Some(format!(
                    "base species parsed from {}",
                    py_repr(row.get("Fuel"))
                )),
                None,
                &[],
            ),
        ),
        // Bottle fraction as written ('30% butane'); an unprefixed name is the gas as delivered and is
        // not converted into a number here (its purity is not stated in the table).
        (
            "fuel_bottle".into(),
            match bottle {
                Some(b) => measured(b, Some("1"), rl("Fuel"), M::new("acme.cond.fuel_bottle")),
                None => text_value(
                    row.get("Fuel"),
                    rl("Fuel"),
                    "acme.cond.fuel_bottle",
                    None,
                    None,
                    &[],
                ),
            },
        ),
        (
            "tad".into(),
            measured(
                row.get(k_tad),
                Some("K"),
                rl(k_tad),
                M::new("acme.cond.tad")
                    .evidence(EvidenceClass::SourceDerived)
                    .flags(&["derived_mixture_property"]),
            ),
        ),
        (
            "zst".into(),
            measured(
                row.get("Zst"),
                Some("1"),
                rl("Zst"),
                M::new("acme.cond.zst")
                    .evidence(EvidenceClass::SourceDerived)
                    .flags(&["derived_mixture_property"]),
            ),
        ),
    ]
}

fn post_facts(row: &Row, sha: &str, path: &str, inverse: bool) -> IndexMap<String, Measured> {
    let rl = |c: &str| Some(loc(sha, path, Some(row.row), Some(c)));
    let tv = |k: &str, mid: &str| text_value(row.get(k), rl(k), mid, None, None, &[]);
    let ms = |k: &str, unit: &str, o: M| measured(row.get(k), Some(unit), rl(k), o);
    let mut out = IndexMap::new();
    out.insert(
        "hot_ignition".into(),
        tv(col(row, &["Hot ignition"]), "acme.post.hot_ignition"),
    );
    out.insert(
        "burn_time".into(),
        ms(
            col(row, &["burn time (sec)"]),
            "s",
            M::new("acme.post.burn_time"),
        ),
    );
    out.insert(
        "se_ft".into(),
        tv(
            col(row, &["Self-ext or flow term", "Self -ext or flow term"]),
            "acme.post.se_ft",
        ),
    );
    out.insert(
        "tfp".into(),
        text_value(
            row.get("TFP"),
            rl("TFP"),
            "acme.tfp.excluded",
            None,
            Some(Resolution::Quarantined),
            &[],
        ),
    );
    out.insert(
        "comments".into(),
        text_value(
            row.opt("Comments"),
            rl("Comments"),
            "acme.post.comments",
            None,
            None,
            &[],
        ),
    );
    if inverse {
        out.insert(
            "x_fuel_after".into(),
            ms(col(row, &["Xf after test"]), "1", M::new("acme.post.after")),
        );
        out.insert(
            "hc_burnrate".into(),
            ms(
                col(row, &["HC burnrate (mg/s)"]),
                "mg/s",
                M::new("acme.post.hc_burnrate").evidence(EvidenceClass::SourceDerived),
            ),
        );
        out.insert(
            "peak_burner_temperature".into(),
            ms(
                col(row, &["Peak burner Temp (C)"]),
                "C",
                M::new("acme.post.peak_t"),
            ),
        );
        out.insert(
            "color".into(),
            text_value(
                row.opt("Color"),
                rl("Color"),
                "acme.post.color",
                None,
                None,
                &[],
            ),
        );
    } else {
        out.insert(
            "x_o2_after".into(),
            ms(
                col(row, &["XO2 after test"]),
                "1",
                M::new("acme.post.after"),
            ),
        );
        out.insert(
            "peak_burner_temperature".into(),
            ms(
                col(row, &["Peak burner T (C )"]),
                "C",
                M::new("acme.post.peak_t"),
            ),
        );
        out.insert(
            "cool_flame".into(),
            text_value(
                row.opt("Cool flame?"),
                rl("Cool flame?"),
                "acme.post.cool_flame",
                None,
                None,
                &[],
            ),
        );
    }
    out
}

fn ingest_table(
    b: &mut Bundle,
    fam: Family,
    name: &str,
    srcs: &BTreeMap<String, ignis_core::contracts::Source>,
) -> anyhow::Result<()> {
    let src = srcs.get(name).ok_or_else(|| {
        anyhow::anyhow!("{name} not in the PSI-159 manifest; run `ignis sources samples PSI-159`")
    })?;
    let sha = mf::sha_of(src);
    let inverse = fam == Family::AcmeInverse;
    let mut seen: HashMap<String, i64> = HashMap::new();
    let p = mf::local_path(src);
    let raw = std::fs::read(&p).map_err(|e| anyhow::anyhow!("{}: {e}", p.display()))?;
    for row in table_rows(&decode_utf8_sig_lossy(&raw), false) {
        let (tid, prefix, day) = test_identity(row.get("Test"));
        if tid.is_empty() {
            b.exclusions.push(Exclusion {
                run_id: format!("{INV}:{}:row{}", fam.as_str(), row.row),
                reason: "row without test id".into(),
            });
            continue;
        }
        let mut run_id = format!("{INV}:{}:{tid}", fam.as_str());
        let mut res = Resolution::Resolved;
        if seen.contains_key(&run_id) {
            res = Resolution::UnresolvedConflict;
            run_id = format!("{run_id}#row{}", row.row);
        }
        seen.insert(run_id.clone(), row.row);
        let session_id = day.as_ref().map(|d| format!("{INV}:day:{d}"));
        if let Some(sid) = &session_id {
            if !b.sessions.contains_key(sid) {
                b.sessions.insert(
                    sid.clone(),
                    Session {
                        id: sid.clone(),
                        investigation_id: INV.into(),
                        campaign_id: None,
                        test_day: day.clone(),
                        gmt_start: None,
                        evidence: vec![loc(&sha, name, Some(row.row), Some("Test"))],
                        resolution: Resolution::Resolved,
                    },
                );
            }
        }
        let mut conds = if inverse {
            conditions_inverse(&row, &sha, name)
        } else {
            conditions_normal(&row, &sha, name)
        };
        conds.extend(common_conditions(&row, &sha, name));
        let aliases = vec![
            Alias {
                name: row.get("Test").into(),
                scheme: "table_test".into(),
                locator: Some(loc(&sha, name, Some(row.row), Some("Test"))),
            },
            Alias {
                name: format!("day{}", strip(row.get("Day"))),
                scheme: "campaign_day".into(),
                locator: Some(loc(&sha, name, Some(row.row), Some("Day"))),
            },
        ];
        let hot = lower(strip(row.get(col(&row, &["Hot ignition"]))));
        b.records.push(ExperimentRecord {
            id: run_id.clone(),
            investigation_id: INV.into(),
            family: fam,
            session_id,
            specimen_id: None,
            aliases,
            fuel_or_material: Some(strip(row.get("Fuel")).into()),
            configuration: Some(if inverse { "inverse" } else { "normal" }.into()),
            gravity: None,
            conditions: conds,
            identity_resolution: res,
            identity_evidence: vec![det(
                "acme.identity.test_id",
                V,
                vec![src.id.clone()],
                Some(format!("YYDDD={}", prefix.as_deref().unwrap_or("None"))),
            )],
            quality_flags: vec![format!(
                "hot_ignition:{}",
                if hot.is_empty() { "missing" } else { &hot }
            )],
            notes: None,
        });
        b.post
            .insert(run_id.clone(), post_facts(&row, &sha, name, inverse));
        let (ep, rv) = endpoint(&run_id, &row, &sha, name);
        b.endpoints.push(ep);
        let comments = row.opt("Comments");
        if let Some(mut rv) = rv {
            rv.family = fam.as_str().into();
            b.review.push(rv);
        } else if !strip(comments).is_empty() && hot == "yes" {
            let screen = if intervention().is_match(comments) {
                Some("intervention pattern".to_string())
            } else if no_ignition().is_match(comments) {
                Some("no-ignition pattern".to_string())
            } else {
                None
            };
            b.review.push(ReviewItem {
                id: format!("{run_id}#comment"), family: fam.as_str().into(), run_id: run_id.clone(), field: "comment_semantics".into(),
                question: "Does this comment change the population membership, control history or endpoint?".into(),
                cited_text: strip(comments).into(), locator: loc(&sha, name, Some(row.row), Some("Comments")),
                candidates: ["no_effect", "intervention", "data_loss", "non_spherical_or_irregular", "other"].map(String::from).to_vec(),
                deterministic_screen: screen, context: Default::default(),
            });
        }
    }
    Ok(())
}

/// `<test>_TP.txt` script timeline -> ControlHistory with t=0 at 'Ignition and retraction'.
pub fn parse_tp(raw: &[u8], run_id: &str, sha: &str, path: &str) -> Option<ControlHistory> {
    static R: OnceLock<Regex> = OnceLock::new();
    let text = normalize_newlines(&String::from_utf8_lossy(raw));
    let rx = re(
        &R,
        r"^([A-Za-z][^:]*?):\s*(\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2})\s*$",
    );
    let mut steps = vec![];
    for (n, line) in text.split('\n').enumerate() {
        if let Some(m) = rx.captures(strip(line)) {
            if let Ok(t) = chrono::NaiveDateTime::parse_from_str(&m[2], "%Y-%m-%d %H:%M:%S") {
                steps.push((strip(&m[1]).to_string(), t, n as i64 + 1));
            }
        }
    }
    let t0 = steps
        .iter()
        .find(|(k, _, _)| lower(k).starts_with("ignition and retraction"))?
        .1;
    let stages = steps
        .iter()
        .map(|(k, t, n)| ControlStage {
            t_start_s: (*t - t0).num_milliseconds() as f64 / 1000.0,
            t_end_s: None,
            control: k.clone(),
            value: Some(text_value(
                k.as_str(),
                Some(SourceLocator {
                    row: Some(*n),
                    ..loc(sha, path, None, None)
                }),
                "acme.tp.timeline",
                None,
                None,
                &[],
            )),
            operator_action: lower(k).starts_with("mfcs off"),
        })
        .collect();
    Some(ControlHistory {
        run_id: run_id.into(),
        time_origin: "TP log 'Ignition and retraction' (wall clock, 1 s resolution)".into(),
        stages,
    })
}

pub fn ingest(family: Family) -> anyhow::Result<Bundle> {
    let srcs = mf::by_name(INV)?;
    let mut b = Bundle::new(family.as_str());
    ingest_table(
        &mut b,
        family,
        if family == Family::AcmeNormal {
            NORMAL
        } else {
            INVERSE
        },
        &srcs,
    )?;
    let mut hist = vec![];
    for rec in &b.records {
        let tid = rec
            .id
            .rsplit(':')
            .next()
            .unwrap()
            .split('#')
            .next()
            .unwrap();
        let name = format!("PSI-159_Raw Data_{tid}_TP.txt");
        if let Some(src) = srcs.get(&name) {
            let p = mf::local_path(src);
            if p.exists() {
                if let Some(ch) = parse_tp(&std::fs::read(&p)?, &rec.id, &mf::sha_of(src), &name) {
                    hist.push(serde_json::to_value(ch)?);
                }
            }
        }
    }
    b.extra.insert("control_history".into(), hist);
    Ok(b)
}

pub const COHORT_CRITERIA: &[(&str, &str)] = &[
    ("population", "established hot flame: Hot ignition == yes (case-insensitive); 'flash'/'no' excluded"),
    ("endpoint", "SE/FT label present; unresolved SE-vs-intervention conflicts excluded from the event likelihood until \
                  adjudicated (kept in records)"),
    ("conditions", "pressure, injected mass flow, role O2 and role fuel fractions all present (no imputation)"),
    ("configuration", "normal and inverse never pooled"),
];

/// Exclusion implied by an adjudicated comment (flag written only by `review::apply`).
fn comment_exclusion(r: &ExperimentRecord) -> Option<&'static str> {
    r.quality_flags
        .iter()
        .rev()
        .find_map(|f| f.strip_prefix("comment_semantics:"))
        .and_then(|v| match v {
            "non_spherical_or_irregular" => {
                Some("non-spherical or irregular flame (adjudicated comment)")
            }
            "other" => Some("comment documents an effect the model does not cover (adjudicated)"),
            "unresolved" => {
                Some("comment semantics unresolved (adjudication passes disagree or invalid)")
            }
            _ => None,
        })
}

pub fn cohort_filter(b: &Bundle) -> Kept {
    let eps: HashMap<&str, &EndpointRecord> =
        b.endpoints.iter().map(|e| (e.run_id.as_str(), e)).collect();
    let (mut keep, mut out) = (vec![], vec![]);
    for r in &b.records {
        let h = &b.post[&r.id]["hot_ignition"];
        let hot = if h.raw_value.is_none() {
            String::new()
        } else {
            lower(strip(&raw_str(h)))
        };
        let e = eps[r.id.as_str()];
        let missing: Vec<&str> = ["pressure", "injected_mass_flow", "x_o2_role", "x_fuel_role"]
            .into_iter()
            .filter(|k| r.conditions[*k].value_si.is_none())
            .collect();
        if hot != "yes" {
            out.push((
                r.id.clone(),
                format!(
                    "hot ignition = {}",
                    if hot.is_empty() { "missing" } else { &hot }
                ),
            ));
        } else if r.identity_resolution != Resolution::Resolved {
            out.push((r.id.clone(), "duplicate test id".into()));
        } else if !missing.is_empty() {
            out.push((
                r.id.clone(),
                format!("missing critical conditions {}", py_repr_list(&missing)),
            ));
        } else if let Some(why) = comment_exclusion(r) {
            out.push((r.id.clone(), why.into()));
        } else if e.resolution != Resolution::Resolved {
            out.push((
                r.id.clone(),
                format!("endpoint {}", super::flex::res_str(e.resolution)),
            ));
        } else {
            keep.push(r.id.clone());
        }
    }
    (keep, out)
}
