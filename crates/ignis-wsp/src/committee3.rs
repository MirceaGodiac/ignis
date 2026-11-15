//! `acme_normal.wsp12` and `acme_inverse.wsp12` — the committee of `committee2.rs` with members
//! that read a typed fuel table in place of the two fuel indicators.
//!
//! Every rule of `committee2.rs` holds unchanged: ten slots, four weight vectors, the era of the test
//! read from the prefix, three readings of the script's planned stop, the weighted mean of the
//! member tables, one member on one node answering with its own table bit for bit, a member's
//! refusal refusing the forecast. Only the kinds of the members differ:
//!
//! ```text
//! slots a0, a1          tensors `a0.*`, `a1.*`: full `flame5.rs` releases, run by `flame5::forecast`
//! slots b0, b1, b2, b3  tensors `b0.*` ... `b3.*`: full `flame6.rs` releases, run by `flame6::forecast`
//! slots c0, c1, c2, c3  tensors `c0.*` ... `c3.*`: full `flame7.rs` releases, run by `flame7::forecast`
//! ```
//!
//! Each slot carries its own member's fuel table (`<slot>.fuel.*`) like every other member tensor, so
//! each member reads the fuel of the prefix by its own table and refuses by its own gates.
//!
//! # The era
//!
//! As in `committee2.rs`: the date of a test is read from `session_id` (the first valid
//! `YYYY-MM-DD`), otherwise from `run_id` (`YYDDD` after its last `:`), otherwise there is none; a
//! day is `year x 1000 + day of year`;
//!
//! ```text
//! p_late = 0                      day <= era.last_early_day
//!          1                      day >= era.first_late_day
//!          era.p_late_between     strictly between the two
//!          era.p_late_unknown     no date
//!
//! w[i]   = (1 - p_late) early[i] + p_late late[i]      each vector divided by its own sum first
//! ```
//!
//! # The script
//!
//! As in `committee2.rs`: with a resolved, finite, positive condition `programmed_stop`, a positive
//! weight in `mix.ws_early` or `mix.ws_late` and in `script.offset_w`, the pair (`mix.ws_early`,
//! `mix.ws_late`) weighs the slots and the members are run once per offset node with one more
//! control stage `scheduled_stop` at `programmed_stop + script.offset_s[k]`:
//!
//! ```text
//! P(t) = sum_k o[k] sum_i w[i] P[i, k](t)       o = script.offset_w divided by its sum
//! ```
//!
//! `committee2.rs` is not edited and its releases keep their own forward; this file repeats its
//! lines with the member forwards swapped. Normal and inverse share this code and nothing else.
//! Nothing is fitted on this route.

use anyhow::bail;
use serde::Serialize;
use serde_json::{json, Value};

use ignis_core::contracts::{
    weights_digest, AnalysisArtifact, ArtifactKind, ControlStage, Digest, Family, ForwardContract,
    ForwardProfile, ModelRelease, ModelSpec, PrefixBundle, QualificationState, Resolution, SpecId,
    Support, SupportGate, TensorRole,
};
use ignis_core::expert::{Expert, ExpertResult, FitContext, FitData, Refusal};
use ignis_core::prefix::ProspectivePolicy;
use ignis_core::registry::{self, LoadedRelease, TensorMap};
use ignis_core::resources::Profile;
use ignis_core::views::View;

use crate::common::{self, admitted, refuse, Body};
use crate::spec::{authored, Params};
use crate::{flame5, flame6, flame7};

pub const FORWARD_VERSION: &str = "12";
/// Slots of `flame5.rs` members.
pub const A_SLOTS: usize = 2;
/// Slots of `flame6.rs` members.
pub const B_SLOTS: usize = 4;
/// Slots of `flame7.rs` members.
pub const C_SLOTS: usize = 4;
pub const SLOTS: usize = A_SLOTS + B_SLOTS + C_SLOTS;
/// Tensor-name prefixes of the slots, in the order of the weight vectors.
pub const SLOT_PREFIX: [&str; SLOTS] = [
    "a0.", "a1.", "b0.", "b1.", "b2.", "b3.", "c0.", "c1.", "c2.", "c3.",
];
/// Offset nodes of the script's planned stop.
pub const OFFSETS: usize = 3;
/// The committee's own tensors, in spec order (the slots follow them).
pub const OWN_TENSORS: [&str; 10] = [
    "mix.w_early",
    "mix.w_late",
    "mix.ws_early",
    "mix.ws_late",
    "era.last_early_day",
    "era.first_late_day",
    "era.p_late_between",
    "era.p_late_unknown",
    "script.offset_s",
    "script.offset_w",
];
pub use flame6::{CATEGORIES, FORBIDDEN, INTERPRETATION};
/// The condition that carries the script's programmed stop, in seconds from its ignition stamp.
pub const PROGRAMMED_STOP: &str = "programmed_stop";
/// Condition keys a prefix may carry: those of `committee2.rs` (the member forwards' keys and the
/// programmed stop).
pub const ALLOWED: [&str; 10] = [
    "pressure",
    "injected_mass_flow",
    "x_o2_role",
    "x_fuel_role",
    "tad",
    "zst",
    "fuel_is_ethane",
    "fuel_is_butane",
    "fuel_species",
    PROGRAMMED_STOP,
];

pub fn model_name(family: Family) -> &'static str {
    match family {
        Family::AcmeInverse => "acme_inverse.wsp12",
        _ => "acme_normal.wsp12",
    }
}

pub fn family_of_model(name: &str) -> Option<Family> {
    match name {
        "acme_normal.wsp12" => Some(Family::AcmeNormal),
        "acme_inverse.wsp12" => Some(Family::AcmeInverse),
        _ => None,
    }
}

/// The three member specs of a family with their hashes, built once per process.
struct Members {
    a: ModelSpec,
    a_hash: Digest,
    b: ModelSpec,
    b_hash: Digest,
    c: ModelSpec,
    c_hash: Digest,
}

impl Members {
    /// The spec of the members of slot `i` and its hash.
    fn of_slot(&self, i: usize) -> (&ModelSpec, &Digest) {
        if i < A_SLOTS {
            (&self.a, &self.a_hash)
        } else if i < A_SLOTS + B_SLOTS {
            (&self.b, &self.b_hash)
        } else {
            (&self.c, &self.c_hash)
        }
    }
}

fn members(family: Family) -> &'static Members {
    static NORMAL: std::sync::OnceLock<Members> = std::sync::OnceLock::new();
    static INVERSE: std::sync::OnceLock<Members> = std::sync::OnceLock::new();
    let cell = if family == Family::AcmeInverse {
        &INVERSE
    } else {
        &NORMAL
    };
    cell.get_or_init(|| {
        let (a, b, c) = (
            flame5::spec(family),
            flame6::spec(family),
            flame7::spec(family),
        );
        Members {
            a_hash: a.spec_hash(),
            b_hash: b.spec_hash(),
            c_hash: c.spec_hash(),
            a,
            b,
            c,
        }
    })
}

pub fn spec(family: Family) -> ModelSpec {
    assert!(matches!(family, Family::AcmeNormal | Family::AcmeInverse));
    let m = members(family);
    let slots = &[SLOTS as u32];
    let weights = Some((0.0, 1.0e3));
    let days = Some((1_900_001.0, 2_100_366.0));
    let mut tensors = vec![
        authored(
            "mix.w_early",
            slots,
            TensorRole::Weight,
            Some("1"),
            weights,
            "Weights of the ten slots, in the order (a0, a1: flame5.rs members; b0, b1, b2, b3: \
             flame6.rs members; c0, c1, c2, c3: flame7.rs members), for a test of the early era when \
             the prefix does not carry the \
             script's programmed stop. A weight is the author's probability that the member's \
             description holds. The vector is divided by its sum; the weight of a slot is (1 - \
             p_late) times its early weight plus p_late times its late weight, and a slot with \
             weight zero is not run. Not checked and not used when p_late is 1.",
        ),
        authored(
            "mix.w_late",
            slots,
            TensorRole::Weight,
            Some("1"),
            weights,
            "The same for a test of the late era. Not checked and not used when p_late is 0.",
        ),
        authored(
            "mix.ws_early",
            slots,
            TensorRole::Weight,
            Some("1"),
            weights,
            "Weights of the ten slots for a test of the early era when the prefix carries the \
             script's programmed stop (condition programmed_stop). mix.ws_early and mix.ws_late \
             both all zero: the programmed stop is not read and the pair mix.w_early, mix.w_late \
             is used.",
        ),
        authored(
            "mix.ws_late",
            slots,
            TensorRole::Weight,
            Some("1"),
            weights,
            "The same for a test of the late era.",
        ),
        authored(
            "era.last_early_day",
            &[],
            TensorRole::PhysicalParameter,
            Some("day"),
            days,
            "Last calendar day known to be in the early era, as year x 1000 + day of year. A test \
             dated on or before it has p_late = 0. It must be smaller than era.first_late_day.",
        ),
        authored(
            "era.first_late_day",
            &[],
            TensorRole::PhysicalParameter,
            Some("day"),
            days,
            "First calendar day known to be in the late era, as year x 1000 + day of year. A test \
             dated on or after it has p_late = 1.",
        ),
        authored(
            "era.p_late_between",
            &[],
            TensorRole::Weight,
            Some("1"),
            Some((0.0, 1.0)),
            "Probability of the late era for a test dated strictly between era.last_early_day and \
             era.first_late_day.",
        ),
        authored(
            "era.p_late_unknown",
            &[],
            TensorRole::Weight,
            Some("1"),
            Some((0.0, 1.0)),
            "Probability of the late era when the prefix gives no date.",
        ),
        authored(
            "script.offset_s",
            &[OFFSETS as u32],
            TensorRole::PhysicalParameter,
            Some("s"),
            Some((-20.0, 20.0)),
            "Offsets added to the stored programmed stop (the sum of the script's stage durations) \
             to give the time, counted from the established flame as the records count it, at \
             which a scripted stop falls: the scheduled stop handed to the members at node k is \
             programmed_stop + script.offset_s[k].",
        ),
        authored(
            "script.offset_w",
            &[OFFSETS as u32],
            TensorRole::Weight,
            Some("1"),
            weights,
            "Weights of the three offsets, normalised by their sum. A node with weight zero is not \
             run. All zero: the programmed stop is not read.",
        ),
    ];
    for (i, prefix) in SLOT_PREFIX.iter().enumerate() {
        let (member, _) = m.of_slot(i);
        for t in &member.tensors {
            let mut t = t.clone();
            t.name = format!("{prefix}{}", t.name);
            tensors.push(t);
        }
    }
    let spec_id = if family == Family::AcmeInverse {
        "acme_inverse-wsp-v12"
    } else {
        "acme_normal-wsp-v12"
    };
    ModelSpec {
        schema_version: m.b.schema_version,
        spec_id: SpecId::new(spec_id).expect("id"),
        family,
        model_name: model_name(family).into(),
        target: m.b.target.clone(),
        features: m.b.features.clone(),
        domain: m.b.domain.clone(),
        tensors,
        forward: ForwardContract {
            entry_point: "ignis_wsp::committee3::forecast".into(),
            version: FORWARD_VERSION.into(),
            profile: ForwardProfile::StrictNoFitting,
        },
        notes: Some(format!(
            "The rules of the committee of committee2.rs with members that read a fuel table. Ten \
             slots, four weight vectors, an era rule and three offsets of the script's planned \
             stop. The tensors a0.* and a1.* are, name for name after the prefix, the tensors of \
             spec {} ({}), run by ignis_wsp::flame5::forecast; the tensors b0.* to b3.* are those of \
             spec {} ({}), run by ignis_wsp::flame6::forecast; the tensors c0.* to c3.* are those of \
             spec {} ({}), run by ignis_wsp::flame7::forecast. Each slot holds its member's own fuel \
             table (<slot>.fuel.*). Each member is run on the same settings as a release of its own \
             spec would be: its features, its fuel table, its domain, its refusals and its numerics \
             are its own (see the notes of those specs). The features and the domain listed here are \
             those of the flame6.rs and flame7.rs members; the flame5.rs members read the first \
             eight. \
             Date of the test, read from the prefix and never from an outcome: (1) session_id, when \
             the prefix has one: the first substring of the form four digits, hyphen, two digits, \
             hyphen, two digits that is a valid calendar date (year, month, day); (2) otherwise, \
             also when session_id holds no such date, run_id: the part after its last ':'; when its \
             first five characters are ASCII digits YYDDD with DDD between 1 and 366 (no more than \
             365 in a common year), the date is day DDD of year 2000 + YY; (3) otherwise there is no \
             date. The day is year x 1000 + day of year. \
             Era: p_late = 0 when the day is at or before era.last_early_day, 1 when it is at or \
             after era.first_late_day, era.p_late_between strictly between the two, \
             era.p_late_unknown without a date. A release with era.last_early_day >= \
             era.first_late_day cannot be run. \
             Script path: taken when the prefix carries a resolved, finite, positive condition \
             programmed_stop and mix.ws_early or mix.ws_late has a positive entry and \
             script.offset_w has a positive entry. The weight pair is then (mix.ws_early, \
             mix.ws_late), otherwise (mix.w_early, mix.w_late). \
             Weights: a vector of the pair whose era has a non-zero probability must be all >= 0 \
             with a positive sum, otherwise the forecast is refused; it is divided by its sum, and \
             the weight of slot i is w[i] = (1 - p_late) early[i] + p_late late[i] (exactly the \
             common value where the two are equal). A vector whose era has probability zero is not \
             checked and not used. A slot with weight zero is not run. \
             On the script path, for every node k with script.offset_w[k] > 0 (o = script.offset_w \
             divided by its sum; a negative entry is a refusal) every slot with a positive weight \
             is run on the prefix without the condition programmed_stop and with one more control \
             stage, scheduled_stop at programmed_stop + script.offset_s[k] seconds (a stop at or \
             before the cutoff refuses the forecast, as the members define). The forecast is P = \
             sum_k o[k] sum_i w[i] P[i, k], nodes in the outer sum and slots in the inner, for \
             every row of (active_supported, extinction, operator_termination, unresolved_support) \
             at every requested time. Off the script path there is one pass on the prefix as given \
             (a programmed_stop it carries is left in place and not read), P = sum_i w[i] P[i]. \
             When exactly one member is run on exactly one node the forecast is that member's \
             table unchanged. If a member that is run refuses the prefix, the forecast is refused \
             with that member's gate and reasons, prefixed with the slot and, on the script path, \
             the node index. The members must return the same forecast times. Uncertainty kind: \
             assumed_noise.",
            m.a.spec_id, m.a_hash, m.b.spec_id, m.b_hash, m.c.spec_id, m.c_hash
        )),
    }
}

/// The slot's tensors as a release of the member's own spec: the committee's record with the
/// member's spec, forward and tensor list in place. It exists only for the duration of one call and
/// is what lets the member's forward run unchanged; it is never stored and has no id of its own.
fn member_release(
    release: &LoadedRelease,
    spec: &ModelSpec,
    spec_hash: &Digest,
    prefix: &str,
) -> LoadedRelease {
    let params: TensorMap = release
        .params
        .iter()
        .filter_map(|(k, t)| k.strip_prefix(prefix).map(|n| (n.to_string(), t.clone())))
        .collect();
    let frozen: Vec<String> = params.keys().cloned().collect();
    let entries = registry::tensor_entries(&params, &frozen);
    let mut r: ModelRelease = release.release.clone();
    r.model_name = spec.model_name.clone();
    r.feature_order = spec.features.iter().map(|f| f.name.clone()).collect();
    r.units = spec
        .features
        .iter()
        .map(|f| (f.name.clone(), f.unit.clone()))
        .collect();
    if let Some(m) = r.construction.as_mut() {
        m.spec_id = spec.spec_id.clone();
        m.spec_hash = spec_hash.clone();
        m.forward = spec.forward.clone();
        m.forward_hash = crate::audit::forward_hash_of(&spec.model_name);
        m.weights_hash = weights_digest(&entries);
    }
    r.tensors = entries;
    LoadedRelease {
        release: r,
        params,
        posterior: TensorMap::new(),
    }
}

// ---------------------------------------------------------------- the date of a test

fn leap(year: u32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn days_in_month(year: u32, month: u32) -> u32 {
    match month {
        4 | 6 | 9 | 11 => 30,
        2 if leap(year) => 29,
        2 => 28,
        _ => 31,
    }
}

fn days_in_year(year: u32) -> u32 {
    if leap(year) {
        366
    } else {
        365
    }
}

/// Day of the year (1 on 1 January) of a calendar date, or `None` when it is not one.
fn day_of_year(year: u32, month: u32, day: u32) -> Option<u32> {
    if year == 0 || !(1..=12).contains(&month) || day == 0 || day > days_in_month(year, month) {
        return None;
    }
    Some((1..month).map(|m| days_in_month(year, m)).sum::<u32>() + day)
}

/// Where the date of a test was read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DateSource {
    SessionId,
    RunId,
}

/// The calendar day of a test, as the prefix gives it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct TestDate {
    pub year: u32,
    /// 1 on 1 January.
    pub day_of_year: u32,
    pub source: DateSource,
}

impl TestDate {
    /// `year x 1000 + day of year`, the encoding of `era.last_early_day` and `era.first_late_day`.
    pub fn ordinal(&self) -> u32 {
        self.year * 1000 + self.day_of_year
    }

    /// The date as `YYYY-MM-DD`.
    pub fn iso(&self) -> String {
        let (mut month, mut day) = (1, self.day_of_year);
        while day > days_in_month(self.year, month) {
            day -= days_in_month(self.year, month);
            month += 1;
        }
        format!("{:04}-{month:02}-{day:02}", self.year)
    }
}

fn digits(b: &[u8]) -> Option<u32> {
    if b.is_empty() || !b.iter().all(u8::is_ascii_digit) {
        return None;
    }
    Some(b.iter().fold(0, |n, c| n * 10 + u32::from(c - b'0')))
}

/// The first `YYYY-MM-DD` in `text` that is a calendar date.
fn date_in_session(text: &str) -> Option<(u32, u32)> {
    text.as_bytes().windows(10).find_map(|w| {
        if w[4] != b'-' || w[7] != b'-' {
            return None;
        }
        let year = digits(&w[..4])?;
        let doy = day_of_year(year, digits(&w[5..7])?, digits(&w[8..])?)?;
        Some((year, doy))
    })
}

/// `YYDDD` at the start of the part of a run id after its last `:`.
fn date_in_run(run_id: &str) -> Option<(u32, u32)> {
    let tail = run_id.rsplit(':').next().unwrap_or(run_id).as_bytes();
    let head = tail.get(..5)?;
    let year = 2000 + digits(&head[..2])?;
    let doy = digits(&head[2..])?;
    (1..=days_in_year(year))
        .contains(&doy)
        .then_some((year, doy))
}

/// The date of the test of a prefix: from `session_id`, otherwise from `run_id`, otherwise none.
/// Nothing but those two identifiers is read.
pub fn test_date(prefix: &PrefixBundle) -> Option<TestDate> {
    let of = |(year, day_of_year), source| TestDate {
        year,
        day_of_year,
        source,
    };
    prefix
        .session_id
        .as_deref()
        .and_then(date_in_session)
        .map(|d| of(d, DateSource::SessionId))
        .or_else(|| date_in_run(&prefix.run_id).map(|d| of(d, DateSource::RunId)))
}

// ---------------------------------------------------------------- the forecast

/// What one member said on one node (kept for the diagnostics).
#[derive(Debug, Clone, Serialize)]
pub struct MemberOutput {
    pub slot: &'static str,
    /// Index of the offset node the member was run on; `None` off the script path.
    pub node: Option<usize>,
    /// The slot's weight: `(1 - p_late) early + p_late late` of the normalised vectors.
    pub weight: f64,
    /// The node's weight (one off the script path).
    pub node_weight: f64,
    pub p_weak_at_cutoff: f64,
    pub scheduled_stop_s: Option<f64>,
    /// Rows in [`CATEGORIES`] order.
    pub table: [Vec<f64>; 4],
    /// The member's own clocks (a `flame5.rs`, a `flame6.rs` or a `flame7.rs` structure).
    pub clocks: Value,
}

/// One offset node of the script's planned stop that was run.
#[derive(Debug, Clone, Serialize)]
pub struct OffsetNode {
    pub node: usize,
    pub offset_s: f64,
    /// `script.offset_w` of the node divided by the sum.
    pub weight: f64,
    /// The stop handed to the members: `programmed_stop + offset_s`.
    pub scheduled_stop_s: f64,
}

/// The numbers of one forecast.
#[derive(Debug, Clone, Serialize)]
pub struct Forecast {
    /// The members that were run: nodes in the outer order, slots in the inner.
    pub members: Vec<MemberOutput>,
    /// The date of the test, when the prefix gives one.
    pub date: Option<TestDate>,
    pub p_late: f64,
    /// The two weight vectors of the path taken (early, late).
    pub weight_tensors: [&'static str; 2],
    /// The offset nodes that were run; empty off the script path.
    pub offsets: Vec<OffsetNode>,
    pub times_s: Vec<f64>,
    /// Rows in [`CATEGORIES`] order.
    pub table: [Vec<f64>; 4],
}

fn refused_by(slot: &str, node: Option<usize>, s: Support) -> Support {
    let slot = slot.trim_end_matches('.');
    let who = match node {
        Some(k) => format!("member {slot}, node {k}"),
        None => format!("member {slot}"),
    };
    Support {
        admitted: false,
        gate: s.gate,
        reasons: s
            .reasons
            .into_iter()
            .map(|r| format!("{who}: {r}"))
            .collect(),
    }
}

/// A weight vector divided by its sum, or the refusal of one that is not a set of weights.
fn normalised(name: &str, w: &[f64]) -> Result<Vec<f64>, Support> {
    let total: f64 = w.iter().sum();
    if !(w.iter().all(|x| *x >= 0.0) && total > 0.0) {
        return Err(refuse(
            SupportGate::Semantic,
            format!("{name} = {w:?} is not a set of weights: all must be >= 0 and one > 0"),
        ));
    }
    Ok(w.iter().map(|x| x / total).collect())
}

/// One pass over the slots: its weight, and the prefix the members see when it is not the prefix
/// as given.
struct Pass {
    node: Option<usize>,
    weight: f64,
    scripted: Option<PrefixBundle>,
}

/// Strict forward of a committee release on an admitted prefix: `Ok(Err(support))` is a scientific
/// refusal, `Err(refusal)` a release or request that cannot be run at all.
pub fn forecast(
    spec: &ModelSpec,
    release: &LoadedRelease,
    prefix: &PrefixBundle,
    settings: &Value,
) -> Result<Result<Forecast, Support>, Refusal> {
    let params = Params::bind(spec, &release.release, &release.params)?;
    let built_with = release
        .release
        .construction
        .as_ref()
        .map(|m| &m.forward_hash);
    let this = crate::audit::forward_hash_of(&spec.model_name);
    if built_with != Some(&this) {
        return Err(Refusal::unsupported(
            "forward_mismatch",
            format!(
                "release {} was assembled with forward {}, this build runs {this}: a release is its weights and its forward (re-assemble, or run the build it names)",
                release.release.id,
                built_with.map_or("none".to_string(), |d| d.to_string()),
            ),
        ));
    }
    let (last_early, first_late) = (
        params.s("era.last_early_day"),
        params.s("era.first_late_day"),
    );
    if !(last_early < first_late) {
        return Err(Refusal::unsupported(
            "era_days",
            format!(
                "release {}: era.last_early_day = {last_early} is not before era.first_late_day = {first_late}",
                release.release.id
            ),
        ));
    }
    if matches!(
        release.release.state,
        QualificationState::Failed | QualificationState::Unsupported
    ) {
        return Ok(Err(refuse(
            SupportGate::Semantic,
            format!("release state {:?}", release.release.state),
        )));
    }
    // the era of the test, from the date the prefix gives
    let date = test_date(prefix);
    let p_late = match &date {
        None => params.s("era.p_late_unknown"),
        Some(d) => {
            let day = f64::from(d.ordinal());
            if day <= last_early {
                0.0
            } else if day >= first_late {
                1.0
            } else {
                params.s("era.p_late_between")
            }
        }
    };
    if !(0.0..=1.0).contains(&p_late) {
        return Err(Refusal::unsupported(
            "era_probability",
            format!(
                "release {}: the probability of the late era, {p_late}, is not in [0, 1]",
                release.release.id
            ),
        ));
    }
    // the script's programmed stop, when the prefix carries it and the release reads it
    let (ws_early, ws_late) = (params.v("mix.ws_early"), params.v("mix.ws_late"));
    let (offset_s, offset_w) = (params.v("script.offset_s"), params.v("script.offset_w"));
    let programmed = prefix
        .known_conditions
        .get(PROGRAMMED_STOP)
        .filter(|m| m.resolution == Resolution::Resolved)
        .and_then(|m| m.value_si)
        .filter(|v| v.is_finite() && *v > 0.0)
        .filter(|_| ws_early.iter().chain(ws_late).any(|x| *x > 0.0))
        .filter(|_| offset_w.iter().any(|x| *x > 0.0));
    let (early, late, weight_tensors) = match programmed {
        Some(_) => (ws_early, ws_late, ["mix.ws_early", "mix.ws_late"]),
        None => (
            params.v("mix.w_early"),
            params.v("mix.w_late"),
            ["mix.w_early", "mix.w_late"],
        ),
    };
    // a vector whose era has probability zero is neither checked nor used
    let early = if p_late < 1.0 {
        match normalised(weight_tensors[0], early) {
            Ok(w) => Some(w),
            Err(s) => return Ok(Err(s)),
        }
    } else {
        None
    };
    let late = if p_late > 0.0 {
        match normalised(weight_tensors[1], late) {
            Ok(w) => Some(w),
            Err(s) => return Ok(Err(s)),
        }
    } else {
        None
    };
    let w: Vec<f64> = match (early, late) {
        (Some(e), None) => e,
        (None, Some(l)) => l,
        (Some(e), Some(l)) => e
            .iter()
            .zip(&l)
            .map(|(e, l)| {
                if e == l {
                    *e
                } else {
                    (1.0 - p_late) * e + p_late * l
                }
            })
            .collect(),
        (None, None) => unreachable!("p_late is in [0, 1]"),
    };
    // the passes: one per offset node of the planned stop, or one on the prefix as given
    let mut offsets: Vec<OffsetNode> = Vec::new();
    let passes: Vec<Pass> = match programmed {
        Some(t) => {
            let o = match normalised("script.offset_w", offset_w) {
                Ok(o) => o,
                Err(s) => return Ok(Err(s)),
            };
            (0..OFFSETS)
                .filter(|k| offset_w[*k] > 0.0)
                .map(|k| {
                    let stop = t + offset_s[k];
                    offsets.push(OffsetNode {
                        node: k,
                        offset_s: offset_s[k],
                        weight: o[k],
                        scheduled_stop_s: stop,
                    });
                    let mut p = prefix.clone();
                    p.known_conditions.remove(PROGRAMMED_STOP);
                    p.known_controls.push(ControlStage {
                        t_start_s: stop,
                        t_end_s: None,
                        control: "scheduled_stop".into(),
                        value: None,
                        operator_action: false,
                    });
                    Pass {
                        node: Some(k),
                        weight: o[k],
                        scripted: Some(p),
                    }
                })
                .collect()
        }
        None => vec![Pass {
            node: None,
            weight: 1.0,
            scripted: None,
        }],
    };
    let m = members(spec.family);
    // the member records of the slots that are run, built once for all passes
    let slots: Vec<(usize, LoadedRelease)> = (0..SLOTS)
        .filter(|i| w[*i] > 0.0)
        .map(|i| {
            let (member, hash) = m.of_slot(i);
            (i, member_release(release, member, hash, SLOT_PREFIX[i]))
        })
        .collect();
    let mut out: Vec<MemberOutput> = Vec::new();
    let mut times: Option<Vec<f64>> = None;
    for pass in &passes {
        let prefix = pass.scripted.as_ref().unwrap_or(prefix);
        for (i, r) in &slots {
            let (i, slot) = (*i, SLOT_PREFIX[*i]);
            let (t, member) = if i < A_SLOTS {
                match flame5::forecast(&m.a, r, prefix, settings)? {
                    Ok(f) => (
                        f.times_s,
                        MemberOutput {
                            slot,
                            node: pass.node,
                            weight: w[i],
                            node_weight: pass.weight,
                            p_weak_at_cutoff: f.p_weak_at_cutoff,
                            scheduled_stop_s: f.scheduled_stop_s,
                            table: f.table,
                            clocks: json!(f.clocks),
                        },
                    ),
                    Err(s) => return Ok(Err(refused_by(slot, pass.node, s))),
                }
            } else if i < A_SLOTS + B_SLOTS {
                match flame6::forecast(&m.b, r, prefix, settings)? {
                    Ok(f) => (
                        f.times_s,
                        MemberOutput {
                            slot,
                            node: pass.node,
                            weight: w[i],
                            node_weight: pass.weight,
                            p_weak_at_cutoff: f.p_weak_at_cutoff,
                            scheduled_stop_s: f.scheduled_stop_s,
                            table: f.table,
                            clocks: json!({"burner": f.burner, "clocks": f.clocks, "p_held_at_cutoff": f.p_held_at_cutoff}),
                        },
                    ),
                    Err(s) => return Ok(Err(refused_by(slot, pass.node, s))),
                }
            } else {
                match flame7::forecast(&m.c, r, prefix, settings)? {
                    Ok(f) => (
                        f.times_s,
                        MemberOutput {
                            slot,
                            node: pass.node,
                            weight: w[i],
                            node_weight: pass.weight,
                            p_weak_at_cutoff: f.p_weak_at_cutoff,
                            scheduled_stop_s: f.scheduled_stop_s,
                            table: f.table,
                            clocks: json!({"burner": f.burner, "clocks": f.clocks}),
                        },
                    ),
                    Err(s) => return Ok(Err(refused_by(slot, pass.node, s))),
                }
            };
            match &times {
                None => times = Some(t),
                Some(have) if *have == t => {}
                Some(have) => {
                    return Err(Refusal::failed(
                        "committee_times",
                        format!(
                            "member {} returned {} forecast times, the members before it {}",
                            slot.trim_end_matches('.'),
                            t.len(),
                            have.len()
                        ),
                    ))
                }
            }
            out.push(member);
        }
    }
    let times_s = times.expect("one weight is positive and one pass is run");
    // sum_k o[k] sum_i w[i] P[i, k]: nodes in the outer sum, slots in the inner
    let row = |k: usize| -> Vec<f64> {
        (0..times_s.len())
            .map(|j| {
                passes
                    .iter()
                    .zip(out.chunks(slots.len()))
                    .map(|(pass, run)| {
                        pass.weight * run.iter().map(|m| m.weight * m.table[k][j]).sum::<f64>()
                    })
                    .sum()
            })
            .collect()
    };
    // one member on one node answers with its own table, bit for bit
    let table = if out.len() == 1 {
        out[0].table.clone()
    } else {
        [row(0), row(1), row(2), row(3)]
    };
    Ok(Ok(Forecast {
        members: out,
        date,
        p_late,
        weight_tensors,
        offsets,
        times_s,
        table,
    }))
}

fn body(f: Forecast) -> Body {
    let diagnostics = json!({
        "interpretation": INTERPRETATION,
        "test_date": f.date.map(|d| d.iso()),
        "test_date_source": match f.date.map(|d| d.source) {
            Some(DateSource::SessionId) => "session_id",
            Some(DateSource::RunId) => "run_id",
            None => "none",
        },
        "test_date_ordinal": f.date.map(|d| d.ordinal()),
        "p_late": f.p_late,
        "weight_tensors": f.weight_tensors,
        "scheduled_stop_from_script_s": match f.offsets.as_slice() {
            [one] => Some(one.scheduled_stop_s),
            _ => None,
        },
        "script_offsets": f.offsets,
        "members": f.members.iter().map(|m| json!({
            "slot": m.slot.trim_end_matches('.'),
            "node": m.node,
            "forward": match m.slot.as_bytes()[0] {
                b'a' => "ignis_wsp::flame5::forecast",
                b'b' => "ignis_wsp::flame6::forecast",
                _ => "ignis_wsp::flame7::forecast",
            },
            "weight": m.weight,
            "node_weight": m.node_weight,
            "p_weak_at_cutoff": m.p_weak_at_cutoff,
            "scheduled_stop_s": m.scheduled_stop_s,
            "clocks": m.clocks,
            "event_cdf": m.table,
        })).collect::<Vec<_>>(),
        "uncertainty_kind": "assumed_noise",
    });
    Body {
        kind: Some(ArtifactKind::EventProbabilities),
        quantity: None,
        times_s: f.times_s,
        quantile_levels: vec![],
        quantiles: vec![],
        mean: vec![],
        event_categories: CATEGORIES.iter().map(|s| s.to_string()).collect(),
        event_cdf: f.table.to_vec(),
        diagnostics,
    }
}

/// The artifact of a forecast or of a scientific refusal.
pub fn artifact(
    spec: &ModelSpec,
    release: &LoadedRelease,
    prefix: &PrefixBundle,
    settings: &Value,
) -> Result<AnalysisArtifact, Refusal> {
    Ok(match forecast(spec, release, prefix, settings)? {
        Ok(f) => common::artifact(release, prefix, settings, admitted(), body(f)),
        Err(support) => common::artifact(
            release,
            prefix,
            settings,
            support,
            Body {
                kind: Some(ArtifactKind::EventProbabilities),
                ..Body::default()
            },
        ),
    })
}

/// One ACME configuration behind the common expert interface.
pub struct Committee3Wsp {
    spec: ModelSpec,
    policy: ProspectivePolicy,
}

impl Committee3Wsp {
    pub fn new(family: Family) -> Self {
        Committee3Wsp {
            spec: spec(family),
            policy: ProspectivePolicy::new(family, &ALLOWED, &FORBIDDEN),
        }
    }
    pub fn spec(&self) -> &ModelSpec {
        &self.spec
    }
}

impl Expert for Committee3Wsp {
    fn family(&self) -> Family {
        self.spec.family
    }
    fn model_name(&self) -> &str {
        &self.spec.model_name
    }
    fn is_baseline(&self) -> bool {
        false
    }
    fn policy(&self) -> &ProspectivePolicy {
        &self.policy
    }
    fn default_config(&self) -> ExpertResult<Value> {
        Ok(json!({"construction": "literal", "spec_id": self.spec.spec_id}))
    }
    fn estimate_hours(&self, _: &Value, _: usize, _: usize, _: &Profile) -> f64 {
        0.0
    }
    fn fit(&self, _: &FitData<'_>, _: &Value, _: &mut FitContext) -> ExpertResult<ModelRelease> {
        bail!("{} has no fitting procedure: its release is assembled from a literal patch (`ignis wsp assemble`)", self.spec.model_name)
    }
    fn forecast(
        &self,
        release: &LoadedRelease,
        prefix: &PrefixBundle,
        settings: &Value,
    ) -> ExpertResult<AnalysisArtifact> {
        match forecast(&self.spec, release, prefix, settings)? {
            Ok(f) => Ok(common::artifact(
                release,
                prefix,
                settings,
                admitted(),
                body(f),
            )),
            Err(support) => Err(common::refusal_of(&support).into()),
        }
    }
    fn log_likelihood(&self, _: &LoadedRelease, _: &View) -> ExpertResult<f64> {
        bail!(
            "{} exposes event probabilities, not a likelihood",
            self.spec.model_name
        )
    }
}
