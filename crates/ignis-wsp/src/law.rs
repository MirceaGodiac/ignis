//! `<slug>.wsp` — the generic law forward: one strict forward for any hand-written law that is a
//! small typed network over named input features.
//!
//! A slug's declaration (`configs/models/law/<slug>/decl.json`, compiled in by `law_slugs.rs`)
//! names the features, their transforms and the targets; every number is a tensor of the release.
//!
//! ```text
//! features   z_i = (x - c)/s | ln(x/c)/s | (1/x - 1/c)/s      x = value_si of condition <name>
//!            refused unless lo <= x <= hi and the transform can take x
//! network    N(z) = b0 + skip . z + sum_j w2[j] relu(b1[j] + W1[j, :] . z)      (j < 8)
//! positive   ln(median) = N_median(z), ln-sd = N_spread(z) > 0, two log-normals of one median:
//!            weights (1 - w, w), ln-sds (sd, k sd); quantiles of that mixture
//! binary     P(yes) = logistic(N_logit(z))
//! event      cause c: S_c(t) = 1 / (1 + (t / exp(N_c(z)))^k_c); the causes compete,
//!            conditioned on no event before the cutoff; a scheduled stop takes what is active
//! ```
//!
//! Nothing is fitted, searched or filled: a missing or out-of-domain input is a scientific refusal
//! that names it. Design and tensor table: `docs/wsp/research/law_forward.md`.

use std::sync::OnceLock;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use ignis_core::contracts::{
    AnalysisArtifact, ArtifactKind, Family, FeatureSpec, ForwardContract, ForwardProfile,
    ModelSpec, PrefixBundle, QualificationState, Quantity, Resolution, SpecId, Support,
    SupportGate, TargetSpec, TensorRole, TensorSpec, WspSchema,
};
use ignis_core::expert::Refusal;
use ignis_core::registry::LoadedRelease;

use crate::common::{self, admitted, refuse, Body};
use crate::law_slugs::SLUGS;
use crate::nn::{hinge_net, sigmoid, softplus};
use crate::quad::{normal_cdf, normal_quantile};
use crate::spec::{authored, constant, sparse, Params};

pub const FORWARD_VERSION: &str = "1";
/// Hinge units per network.
pub const HIDDEN: usize = 8;
pub const MAX_FEATURES: usize = 16;
pub const MAX_TARGETS: usize = 8;
pub const MAX_CAUSES: usize = 3;
/// Offset nodes of a declared scheduled stop.
pub const STOP_NODES: usize = 3;
/// Category of the mass still active (event targets).
pub const ACTIVE: &str = "active";
/// Category of the mass a scheduled stop takes (event targets).
pub const STOPPED: &str = "scheduled_stop";
/// Support limit of the unbounded raw tensors (centres, domain bounds).
pub const WIDE: f64 = 1.0e30;
/// Steps of the bisection that inverts the two-component mixture CDF (a fixed distribution).
const BISECTION_STEPS: usize = 200;

// ------------------------------------------------------------------------------- declaration

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    /// A restated law: its releases may be assembled.
    Declared,
    /// Registered before its law was restated: no release can be assembled.
    Placeholder,
    /// Synthetic (tests and documentation); never NASA data.
    Synthetic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Transform {
    Identity,
    Ln,
    Reciprocal,
}

impl Transform {
    pub fn id(self) -> &'static str {
        match self {
            Transform::Identity => "identity",
            Transform::Ln => "ln",
            Transform::Reciprocal => "reciprocal",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Positive,
    Binary,
    Event,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FeatureDecl {
    pub name: String,
    pub unit: String,
    pub transform: Transform,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StopDecl {
    /// Prefix condition holding the planned stop, seconds since the time origin.
    pub condition: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TargetDecl {
    pub name: String,
    pub kind: Kind,
    /// SI unit of a positive target.
    #[serde(default)]
    pub unit: Option<String>,
    /// The shared quantity a positive target is, when it is one.
    #[serde(default)]
    pub shared_quantity: Option<Quantity>,
    /// Competing causes of an event target.
    #[serde(default)]
    pub causes: Vec<String>,
    #[serde(default)]
    pub scheduled_stop: Option<StopDecl>,
    #[serde(default)]
    pub description: String,
}

impl TargetDecl {
    /// Categories of the artifact of an event or binary target.
    pub fn categories(&self) -> Vec<String> {
        match self.kind {
            Kind::Positive => vec![],
            Kind::Binary => vec!["yes".into(), "no".into()],
            Kind::Event => std::iter::once(ACTIVE.to_string())
                .chain(self.causes.iter().cloned())
                .chain(std::iter::once(STOPPED.to_string()))
                .collect(),
        }
    }
}

fn other_family() -> Family {
    Family::Other
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Decl {
    pub slug: String,
    pub status: Status,
    #[serde(default = "other_family")]
    pub family: Family,
    #[serde(default)]
    pub investigation: String,
    #[serde(default)]
    pub description: String,
    pub features: Vec<FeatureDecl>,
    pub targets: Vec<TargetDecl>,
}

fn valid_name(s: &str) -> bool {
    let mut c = s.chars();
    matches!(c.next(), Some('a'..='z'))
        && c.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

impl Decl {
    pub fn validate(&self, slug: &str) -> Result<(), String> {
        if self.slug != slug {
            return Err(format!("declaration names slug {:?}", self.slug));
        }
        if self.features.len() > MAX_FEATURES {
            return Err(format!(
                "{} features (at most {MAX_FEATURES})",
                self.features.len()
            ));
        }
        if self.targets.is_empty() || self.targets.len() > MAX_TARGETS {
            return Err(format!(
                "{} targets (1 to {MAX_TARGETS})",
                self.targets.len()
            ));
        }
        let mut seen = std::collections::BTreeSet::new();
        for f in &self.features {
            if !valid_name(&f.name) || !seen.insert(f.name.as_str()) {
                return Err(format!("feature name {:?} invalid or repeated", f.name));
            }
            if f.unit.trim().is_empty() || f.unit.contains(char::is_whitespace) {
                return Err(format!("feature {}: unit {:?}", f.name, f.unit));
            }
        }
        let mut seen = std::collections::BTreeSet::new();
        for t in &self.targets {
            if !valid_name(&t.name)
                || t.name == "x"
                || t.name == "num"
                || !seen.insert(t.name.as_str())
            {
                return Err(format!("target name {:?} invalid or repeated", t.name));
            }
            match t.kind {
                Kind::Positive => {
                    let unit = t.unit.as_deref().unwrap_or_default();
                    if unit.trim().is_empty() || unit.contains(char::is_whitespace) {
                        return Err(format!("positive target {} needs an SI unit", t.name));
                    }
                    if let Some(q) = t.shared_quantity {
                        if q.unit_si() != unit {
                            return Err(format!(
                                "target {}: unit {unit} is not the unit of {q:?} ({})",
                                t.name,
                                q.unit_si()
                            ));
                        }
                        if q == Quantity::PhaseEndTime {
                            return Err(format!(
                                "target {}: phase_end_time is an event target, not a positive number",
                                t.name
                            ));
                        }
                    }
                }
                Kind::Binary => {
                    if t.unit.is_some() || t.shared_quantity.is_some() {
                        return Err(format!("binary target {} has no unit or quantity", t.name));
                    }
                }
                Kind::Event => {
                    if t.unit.is_some() || t.shared_quantity.is_some() {
                        return Err(format!(
                            "event target {} has no unit or quantity (times are in s)",
                            t.name
                        ));
                    }
                    if t.causes.is_empty() || t.causes.len() > MAX_CAUSES {
                        return Err(format!("event target {}: 1 to {MAX_CAUSES} causes", t.name));
                    }
                    let mut cs = std::collections::BTreeSet::new();
                    for c in &t.causes {
                        if !valid_name(c)
                            || c == ACTIVE
                            || c == STOPPED
                            || c == "stop"
                            || !cs.insert(c)
                        {
                            return Err(format!("event target {}: cause {c:?}", t.name));
                        }
                    }
                    if let Some(s) = &t.scheduled_stop {
                        if !valid_name(&s.condition) {
                            return Err(format!("event target {}: stop condition", t.name));
                        }
                    }
                }
            }
            if t.kind != Kind::Event && (!t.causes.is_empty() || t.scheduled_stop.is_some()) {
                return Err(format!(
                    "target {}: causes and stops belong to events",
                    t.name
                ));
            }
        }
        Ok(())
    }

    pub fn target(&self, name: &str) -> Option<&TargetDecl> {
        self.targets.iter().find(|t| t.name == name)
    }
}

/// The parsed declaration of slug `i` of [`SLUGS`].
///
/// # Panics
/// When a compiled-in declaration does not parse or validate (`tests/law_forward.rs` checks every
/// slug).
pub fn decl(i: usize) -> &'static Decl {
    static CACHE: OnceLock<Vec<Decl>> = OnceLock::new();
    let all = CACHE.get_or_init(|| {
        SLUGS
            .iter()
            .map(|s| {
                let d: Decl = serde_json::from_str(s.decl)
                    .unwrap_or_else(|e| panic!("{}: declaration does not parse: {e}", s.decl_path));
                d.validate(s.slug)
                    .unwrap_or_else(|e| panic!("{}: {e}", s.decl_path));
                d
            })
            .collect()
    });
    &all[i]
}

// ------------------------------------------------------------------------------- spec

fn net_tensors(
    out: &mut Vec<TensorSpec>,
    p: &str,
    n: u32,
    role: TensorRole,
    unit: &str,
    range: (f64, f64),
    what: &str,
) {
    use TensorRole::*;
    out.push(authored(
        &format!("{p}.b0"),
        &[],
        role,
        Some(unit),
        Some(range),
        &format!("Value of {what} at z = 0 before hinge terms ({unit})."),
    ));
    if n == 0 {
        return;
    }
    let h = HIDDEN as u32;
    out.extend([
        authored(
            &format!("{p}.skip"),
            &[n],
            Weight,
            None,
            None,
            &format!("Linear weights of {what} on z, in feature order; every element is stated."),
        ),
        sparse(
            &format!("{p}.W1"),
            &[h, n],
            Weight,
            None,
            &format!("Hinge input weights of {what}; row j reads z in feature order."),
        ),
        sparse(
            &format!("{p}.b1"),
            &[h],
            Bias,
            None,
            &format!("Hinge biases of {what} (hinge positions)."),
        ),
        sparse(
            &format!("{p}.w2"),
            &[h],
            Weight,
            None,
            &format!("Hinge output weights of {what}."),
        ),
    ]);
}

fn feature_transform(f: &FeatureDecl) -> String {
    let (c, s) = (
        format!("x.{}.centre", f.name),
        format!("x.{}.scale", f.name),
    );
    let body = match f.transform {
        Transform::Identity => format!("(x - {c}) / {s}"),
        Transform::Ln => format!("ln(x / {c}) / {s}"),
        Transform::Reciprocal => format!("(1/x - 1/{c}) / {s}"),
    };
    format!(
        "{}: {body}; refused unless x.{n}.lo <= x <= x.{n}.hi @1",
        f.transform.id(),
        n = f.name
    )
}

/// The frozen spec of slug `i`.
pub fn spec(i: usize) -> ModelSpec {
    use TensorRole::*;
    let s = &SLUGS[i];
    let d = decl(i);
    let n = d.features.len() as u32;
    let mut tensors = Vec::new();
    for f in &d.features {
        let scale_unit = match f.transform {
            Transform::Identity => f.unit.clone(),
            Transform::Ln => "1".into(),
            Transform::Reciprocal => format!("1/({})", f.unit),
        };
        let centre_range = match f.transform {
            Transform::Ln => (1.0 / WIDE, WIDE),
            _ => (-WIDE, WIDE),
        };
        tensors.extend([
            authored(
                &format!("x.{}.centre", f.name),
                &[],
                PhysicalParameter,
                Some(&f.unit),
                Some(centre_range),
                &format!(
                    "Centre c of the {} transform of {}.",
                    f.transform.id(),
                    f.name
                ),
            ),
            authored(
                &format!("x.{}.scale", f.name),
                &[],
                PhysicalParameter,
                Some(&scale_unit),
                Some((1.0 / WIDE, WIDE)),
                &format!(
                    "Scale s of the {} transform of {}.",
                    f.transform.id(),
                    f.name
                ),
            ),
            authored(
                &format!("x.{}.lo", f.name),
                &[],
                PhysicalParameter,
                Some(&f.unit),
                Some((-WIDE, WIDE)),
                &format!(
                    "Lower end of the declared domain of {} (raw SI value).",
                    f.name
                ),
            ),
            authored(
                &format!("x.{}.hi", f.name),
                &[],
                PhysicalParameter,
                Some(&f.unit),
                Some((-WIDE, WIDE)),
                &format!(
                    "Upper end of the declared domain of {} (raw SI value).",
                    f.name
                ),
            ),
        ]);
    }
    for t in &d.targets {
        let name = &t.name;
        match t.kind {
            Kind::Positive => {
                let unit = t.unit.clone().unwrap_or_default();
                net_tensors(
                    &mut tensors,
                    &format!("{name}.median"),
                    n,
                    PhysicalParameter,
                    &format!("ln({unit})"),
                    (-80.0, 80.0),
                    &format!("ln(median {name} / 1 {unit})"),
                );
                net_tensors(
                    &mut tensors,
                    &format!("{name}.spread"),
                    n,
                    NoiseParameter,
                    "1",
                    (0.001, 10.0),
                    &format!("the ln-sd of the core component of {name}"),
                );
                tensors.extend([
                    authored(&format!("{name}.tail_weight"), &[], NoiseParameter, Some("1"), Some((0.0, 1.0)),
                             &format!("Weight of the wider component of {name} (0: one log-normal).")),
                    authored(&format!("{name}.tail_width"), &[], NoiseParameter, Some("1"), Some((1.0, 100.0)),
                             &format!("ln-sd of the wider component of {name} over that of the core (1: no wider component).")),
                ]);
            }
            Kind::Binary => net_tensors(
                &mut tensors,
                &format!("{name}.logit"),
                n,
                HazardParameter,
                "logit(1)",
                (-40.0, 40.0),
                &format!("the logit of P({name})"),
            ),
            Kind::Event => {
                for c in &t.causes {
                    net_tensors(
                        &mut tensors,
                        &format!("{name}.{c}.median"),
                        n,
                        PhysicalParameter,
                        "ln(s)",
                        (-20.0, 25.0),
                        &format!("ln(median time of cause {c} of {name} alone / 1 s), since the time origin"),
                    );
                    tensors.push(authored(
                        &format!("{name}.{c}.shape"),
                        &[],
                        HazardParameter,
                        Some("1"),
                        Some((0.1, 50.0)),
                        &format!("Log-logistic shape of cause {c} of {name} (sd of ln T = pi / (shape sqrt 3))."),
                    ));
                }
                if t.scheduled_stop.is_some() {
                    let k = STOP_NODES as u32;
                    tensors.extend([
                        authored(&format!("{name}.stop.offset_s"), &[k], PhysicalParameter, Some("s"), Some((-1.0e5, 1.0e5)),
                                 "Offsets added to the stop condition: node k stops at condition + offset_s[k]."),
                        authored(&format!("{name}.stop.offset_w"), &[k], Weight, Some("1"), Some((0.0, 1.0)),
                                 "Weights of the offset nodes (normalised by their sum); all zero: the stop condition is not read."),
                    ]);
                }
            }
        }
    }
    tensors.push(constant(
        "num.substeps",
        512.0,
        "1",
        "Integration steps per requested interval of an event target (numerical, not physics).",
    ));
    let lane_local: Vec<String> = d
        .targets
        .iter()
        .filter(|t| t.kind != Kind::Event && t.shared_quantity.is_none())
        .map(|t| match &t.unit {
            Some(u) => format!(
                "{}:{} ({}, {u}): lane-local target, not a shared quantity",
                s.slug,
                t.name,
                kind_id(t.kind)
            ),
            None => format!(
                "{}:{} ({}): lane-local target, not a shared quantity",
                s.slug,
                t.name,
                kind_id(t.kind)
            ),
        })
        .collect();
    let primary = d.targets.iter().find_map(|t| match t.kind {
        Kind::Positive => t.shared_quantity,
        Kind::Event => Some(Quantity::PhaseEndTime),
        Kind::Binary => None,
    });
    let mut excludes = lane_local.clone();
    let description = match primary {
        Some(q) => format!(
            "{} ({:?} is the first target with a shared quantity); targets: {}",
            d.description,
            q,
            d.targets
                .iter()
                .map(|t| t.name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ),
        None => {
            excludes.insert(
                0,
                "phase_end_time is named here only because ModelSpec.target requires one shared quantity: \
                 every target of this model is lane-local and no artifact carries phase_end_time"
                    .into(),
            );
            format!(
                "{} (no target is a shared quantity); targets: {}",
                d.description,
                d.targets
                    .iter()
                    .map(|t| t.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
    };
    let quantity = primary.unwrap_or(Quantity::PhaseEndTime);
    ModelSpec {
        schema_version: WspSchema::V1,
        spec_id: SpecId::new(format!("law-{}-v1", s.slug)).expect("id"),
        family: d.family,
        model_name: s.model_name.into(),
        target: TargetSpec {
            quantity,
            unit: quantity.unit_si().into(),
            description,
            excludes,
        },
        features: d
            .features
            .iter()
            .map(|f| FeatureSpec {
                name: f.name.clone(),
                unit: f.unit.clone(),
                source: format!("condition:{}", f.name),
                transform: feature_transform(f),
            })
            .collect(),
        domain: vec![],
        tensors,
        forward: ForwardContract {
            entry_point: "ignis_wsp::law::forecast".into(),
            version: FORWARD_VERSION.into(),
            profile: ForwardProfile::StrictNoFitting,
        },
        notes: Some(format!(
            "Generic law forward law.rs, declaration {} (status {:?}): {}. Each network is \
             N(z) = b0 + skip . z + sum_j w2[j] relu(b1[j] + W1[j, :] . z). A positive target is a \
             mixture of two log-normals of one median exp(N_median), ln-sds N_spread and \
             tail_width * N_spread, weights 1 - tail_weight and tail_weight; a non-positive spread \
             is refused. A binary target is logistic(N_logit). An event target is the competition \
             of log-logistic clocks exp(N_c), shape c.shape, conditioned on no event before the \
             cutoff; the earliest control stage scheduled_stop or flow_stop, and the declared stop \
             condition plus each offset node with a positive weight, take the active mass. Lane-local \
             targets are labelled <slug>:<target> in diagnostics.law and carry no shared quantity. \
             Uncertainty kind: assumed_noise.",
            s.decl_path,
            d.status,
            serde_json::to_string(d).expect("declaration serialises")
        )),
    }
}

pub fn kind_id(k: Kind) -> &'static str {
    match k {
        Kind::Positive => "positive",
        Kind::Binary => "binary",
        Kind::Event => "event",
    }
}

// ------------------------------------------------------------------------------- forward

#[derive(Clone, Copy)]
struct Net<'a> {
    b0: f64,
    skip: &'a [f64],
    w1: &'a [f64],
    b1: &'a [f64],
    w2: &'a [f64],
}

impl<'a> Net<'a> {
    fn bind(p: &Params<'a>, prefix: &str, n: usize) -> Self {
        let b0 = p.s(&format!("{prefix}.b0"));
        if n == 0 {
            return Net {
                b0,
                skip: &[],
                w1: &[],
                b1: &[],
                w2: &[],
            };
        }
        Net {
            b0,
            skip: p.v(&format!("{prefix}.skip")),
            w1: p.v(&format!("{prefix}.W1")),
            b1: p.v(&format!("{prefix}.b1")),
            w2: p.v(&format!("{prefix}.w2")),
        }
    }

    fn eval(&self, z: &[f64]) -> f64 {
        if self.skip.is_empty() {
            return self.b0;
        }
        let mut hidden = [0.0; HIDDEN];
        hinge_net(
            self.w1,
            self.b1,
            self.w2,
            self.skip,
            self.b0,
            z,
            &mut hidden,
        )
    }
}

/// The features of one prefix: raw SI values and normalised inputs.
#[derive(Debug, Clone, Serialize)]
pub struct Features {
    pub names: Vec<String>,
    pub raw: Vec<f64>,
    pub z: Vec<f64>,
}

/// Raw and normalised features, or the reasons (each naming its feature) they cannot be formed.
fn features(
    d: &Decl,
    p: &Params<'_>,
    prefix: &PrefixBundle,
) -> Result<Result<Features, Vec<String>>, Refusal> {
    let mut reasons = Vec::new();
    let (mut raw, mut z) = (Vec::new(), Vec::new());
    for f in &d.features {
        let n = &f.name;
        let (c, s) = (p.s(&format!("x.{n}.centre")), p.s(&format!("x.{n}.scale")));
        let (lo, hi) = (p.s(&format!("x.{n}.lo")), p.s(&format!("x.{n}.hi")));
        if !(lo <= hi)
            || !(s > 0.0)
            || (f.transform == Transform::Ln && !(c > 0.0))
            || (f.transform == Transform::Reciprocal && c == 0.0)
        {
            return Err(Refusal::failed(
                "release_tensors",
                format!("feature {n}: centre {c}, scale {s}, domain [{lo}, {hi}] cannot define a {} transform", f.transform.id()),
            ));
        }
        let x = match common::condition(prefix, n) {
            Ok(x) => x,
            Err(e) => {
                reasons.push(format!("feature {n}: {e}"));
                raw.push(f64::NAN);
                z.push(f64::NAN);
                continue;
            }
        };
        let before = reasons.len();
        let zi = if !(lo <= x && x <= hi) {
            reasons.push(format!(
                "feature {n} = {x} {} is outside its declared domain [{lo}, {hi}]",
                f.unit
            ));
            f64::NAN
        } else {
            match f.transform {
                Transform::Identity => (x - c) / s,
                Transform::Ln if x > 0.0 => (x / c).ln() / s,
                Transform::Ln => {
                    reasons.push(format!(
                        "feature {n} = {x} {} is not positive (transform ln)",
                        f.unit
                    ));
                    f64::NAN
                }
                Transform::Reciprocal if x != 0.0 => (1.0 / x - 1.0 / c) / s,
                Transform::Reciprocal => {
                    reasons.push(format!("feature {n} = 0 {} (transform reciprocal)", f.unit));
                    f64::NAN
                }
            }
        };
        if reasons.len() == before && !zi.is_finite() {
            reasons.push(format!(
                "feature {n} = {x} {} gives a non-finite input",
                f.unit
            ));
        }
        raw.push(x);
        z.push(zi);
    }
    if !reasons.is_empty() {
        return Ok(Err(reasons));
    }
    Ok(Ok(Features {
        names: d.features.iter().map(|f| f.name.clone()).collect(),
        raw,
        z,
    }))
}

/// `q` with `(1 - w) Phi(q / sd) + w Phi(q / (k sd)) = p`: the ln-space quantile of the two-component
/// mixture of one median. Closed form for one component, bisection of the known monotone CDF
/// otherwise (a quantile of a fixed distribution; no parameter is estimated).
pub fn mixture_ln_quantile(p: f64, sd: f64, w: f64, k: f64) -> f64 {
    let z = normal_quantile(p);
    if w == 0.0 || k == 1.0 {
        return sd * z;
    }
    if w == 1.0 {
        return k * sd * z;
    }
    let (a, b) = (sd * z, k * sd * z);
    let (mut lo, mut hi) = if a <= b { (a, b) } else { (b, a) };
    let f = |q: f64| (1.0 - w) * normal_cdf(q / sd) + w * normal_cdf(q / (k * sd)) - p;
    for _ in 0..BISECTION_STEPS {
        let mid = 0.5 * (lo + hi);
        if f(mid) < 0.0 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    0.5 * (lo + hi)
}

/// One log-logistic clock: `ln(median time / 1 s)` and shape.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Clock {
    pub ln_tau: f64,
    pub shape: f64,
}

/// `ln S(t)` of the competing clocks (`S = prod_c 1 / (1 + (t / tau_c)^k_c)`), `t >= 0`.
pub fn ln_survival(clocks: &[Clock], t: f64) -> f64 {
    if !(t > 0.0) {
        return 0.0;
    }
    let lt = t.ln();
    -clocks
        .iter()
        .map(|c| softplus(c.shape * (lt - c.ln_tau)))
        .sum::<f64>()
}

/// Hazard (1/s) of one clock at `t > 0`.
pub fn hazard(c: &Clock, t: f64) -> f64 {
    if !(t > 0.0) {
        return 0.0;
    }
    (c.shape / t) * sigmoid(c.shape * (t.ln() - c.ln_tau))
}

/// Rows `active, cause..., scheduled_stop` at each time (sorted, `>= tau`), conditional on no
/// event before `tau`, with the active mass taken by `stop` (`> tau`) when given.
pub fn event_table(
    clocks: &[Clock],
    tau: f64,
    stop: Option<f64>,
    times: &[f64],
    substeps: usize,
) -> Vec<Vec<f64>> {
    let nc = clocks.len();
    let mut rows = vec![vec![0.0; times.len()]; nc + 2];
    let ln_s_tau = ln_survival(clocks, tau);
    let surv = |t: f64| (ln_survival(clocks, t) - ln_s_tau).exp().min(1.0);
    let mut f = vec![0.0; nc];
    let (mut t, mut s) = (tau, 1.0_f64);
    let mut hz = vec![0.0; nc];
    for (j, &tj) in times.iter().enumerate() {
        let target = stop.map_or(tj, |x| tj.min(x));
        if target > t {
            let h = (target - t) / substeps as f64;
            for i in 0..substeps {
                let u0 = t + i as f64 * h;
                let u1 = if i + 1 == substeps {
                    target
                } else {
                    t + (i + 1) as f64 * h
                };
                let s1 = surv(u1).min(s);
                let lost = s - s1;
                if lost > 0.0 {
                    let mid = 0.5 * (u0 + u1);
                    for (k, c) in clocks.iter().enumerate() {
                        hz[k] = hazard(c, mid);
                    }
                    let total: f64 = hz.iter().sum();
                    for k in 0..nc {
                        f[k] += if total > 0.0 {
                            lost * hz[k] / total
                        } else {
                            lost / nc as f64
                        };
                    }
                }
                s = s1;
            }
            t = target;
        }
        let stopped = stop.is_some_and(|x| tj >= x);
        rows[0][j] = if stopped { 0.0 } else { s };
        for k in 0..nc {
            rows[1 + k][j] = f[k];
        }
        rows[nc + 1][j] = if stopped { s } else { 0.0 };
    }
    rows
}

/// What a forecast of one target holds (reported in the diagnostics too).
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Outcome {
    Positive {
        ln_median: f64,
        ln_sd: f64,
        tail_weight: f64,
        tail_width: f64,
        levels: Vec<f64>,
        quantiles: Vec<f64>,
        mean: Option<f64>,
    },
    Binary {
        logit: f64,
        probability: f64,
    },
    Event {
        causes: Vec<String>,
        clocks: Vec<Clock>,
        /// `(weight, stop, offset node)` of each pass.
        passes: Vec<(f64, Option<f64>, Option<usize>)>,
        stop_condition_read: bool,
        times_s: Vec<f64>,
        table: Vec<Vec<f64>>,
    },
}

/// One forecast of one target.
#[derive(Debug, Clone, Serialize)]
pub struct Forecast {
    pub slug: &'static str,
    pub target: String,
    pub cutoff_s: f64,
    pub features: Features,
    pub outcome: Outcome,
}

fn target_of<'d>(d: &'d Decl, settings: &Value) -> Result<&'d TargetDecl, String> {
    match settings.get("target") {
        None | Some(Value::Null) => Ok(&d.targets[0]),
        Some(Value::String(name)) => d.target(name).ok_or_else(|| {
            format!(
                "target {name:?} is not declared by this law ({})",
                d.targets
                    .iter()
                    .map(|t| t.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }),
        Some(other) => Err(format!(
            "settings.target must be a target name, not {other}"
        )),
    }
}

fn control_stop(prefix: &PrefixBundle) -> Option<f64> {
    prefix
        .known_controls
        .iter()
        .filter(|s| {
            let c = s.control.to_lowercase();
            c == "scheduled_stop" || c == "flow_stop"
        })
        .map(|s| s.t_start_s)
        .filter(|t| t.is_finite())
        .fold(None, |a: Option<f64>, b| Some(a.map_or(b, |x| x.min(b))))
}

/// Strict forward of a law release on a prefix: `Ok(Err(support))` is a scientific refusal,
/// `Err(refusal)` a release or request that cannot be run at all.
pub fn forecast(
    i: usize,
    spec: &ModelSpec,
    release: &LoadedRelease,
    prefix: &PrefixBundle,
    settings: &Value,
) -> Result<Result<Forecast, Support>, Refusal> {
    let d = decl(i);
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
    if matches!(
        release.release.state,
        QualificationState::Failed | QualificationState::Unsupported
    ) {
        return Ok(Err(refuse(
            SupportGate::Semantic,
            format!("release state {:?}", release.release.state),
        )));
    }
    if prefix.family != spec.family {
        return Ok(Err(refuse(
            SupportGate::Semantic,
            format!(
                "a {} prefix cannot be forecast by {} (family {})",
                prefix.family.as_str(),
                spec.model_name,
                spec.family.as_str()
            ),
        )));
    }
    let t = match target_of(d, settings) {
        Ok(t) => t,
        Err(e) => return Ok(Err(refuse(SupportGate::Semantic, e))),
    };
    let tau = prefix.cutoff_s;
    if !(tau.is_finite() && tau >= 0.0) {
        return Ok(Err(refuse(
            SupportGate::Horizon,
            format!("cutoff {tau} s is not a finite time at or after the time origin"),
        )));
    }
    let x = match features(d, &params, prefix)? {
        Ok(x) => x,
        Err(reasons) => {
            return Ok(Err(Support {
                admitted: false,
                gate: SupportGate::Condition,
                reasons,
            }))
        }
    };
    let n = d.features.len();
    let name = &t.name;
    let outcome = match t.kind {
        Kind::Positive => {
            let m = Net::bind(&params, &format!("{name}.median"), n).eval(&x.z);
            let sd = Net::bind(&params, &format!("{name}.spread"), n).eval(&x.z);
            if !(sd > 0.0 && sd.is_finite() && m.is_finite()) {
                return Ok(Err(refuse(
                    SupportGate::Numerical,
                    format!("target {name}: ln(median) = {m}, ln-sd = {sd} at these conditions (the spread must be positive)"),
                )));
            }
            let (w, k) = (
                params.s(&format!("{name}.tail_weight")),
                params.s(&format!("{name}.tail_width")),
            );
            let levels = common::levels_of(settings);
            if levels.iter().any(|q| !(*q > 0.0 && *q < 1.0))
                || levels.windows(2).any(|p| p[1] <= p[0])
            {
                return Ok(Err(refuse(
                    SupportGate::Semantic,
                    "quantile levels must be strictly increasing in (0, 1)",
                )));
            }
            let quantiles: Vec<f64> = levels
                .iter()
                .map(|p| (m + mixture_ln_quantile(*p, sd, w, k)).exp())
                .collect();
            if quantiles.iter().any(|q| !(q.is_finite() && *q > 0.0)) {
                return Ok(Err(refuse(
                    SupportGate::Numerical,
                    format!("target {name}: a quantile is not a finite positive number"),
                )));
            }
            let mean =
                m.exp() * ((1.0 - w) * (0.5 * sd * sd).exp() + w * (0.5 * k * k * sd * sd).exp());
            Outcome::Positive {
                ln_median: m,
                ln_sd: sd,
                tail_weight: w,
                tail_width: k,
                levels,
                quantiles,
                mean: mean.is_finite().then_some(mean),
            }
        }
        Kind::Binary => {
            let l = Net::bind(&params, &format!("{name}.logit"), n).eval(&x.z);
            if !l.is_finite() {
                return Ok(Err(refuse(
                    SupportGate::Numerical,
                    format!("target {name}: logit {l}"),
                )));
            }
            Outcome::Binary {
                logit: l,
                probability: sigmoid(l),
            }
        }
        Kind::Event => {
            let clocks: Vec<Clock> = t
                .causes
                .iter()
                .map(|c| Clock {
                    ln_tau: Net::bind(&params, &format!("{name}.{c}.median"), n).eval(&x.z),
                    shape: params.s(&format!("{name}.{c}.shape")),
                })
                .collect();
            if clocks.iter().any(|c| !c.ln_tau.is_finite()) {
                return Ok(Err(refuse(
                    SupportGate::Numerical,
                    format!("target {name}: a clock is not finite"),
                )));
            }
            if settings.get("times_s").is_none() && settings.get("horizon_s").is_none() {
                return Ok(Err(refuse(
                    SupportGate::Horizon,
                    format!("target {name} is an event in time: the request must give times_s or horizon_s"),
                )));
            }
            let mut times = common::times_of(settings, tau, 0.0, f64::INFINITY);
            times.retain(|t| t.is_finite() && *t >= tau);
            times.dedup();
            if times.is_empty() {
                return Ok(Err(refuse(
                    SupportGate::Horizon,
                    "no forecast time at or after the cutoff",
                )));
            }
            let control = control_stop(prefix);
            let mut passes: Vec<(f64, Option<f64>, Option<usize>)> = Vec::new();
            let mut stop_condition_read = false;
            if let Some(sd) = &t.scheduled_stop {
                let cond = prefix
                    .known_conditions
                    .get(&sd.condition)
                    .filter(|m| m.resolution == Resolution::Resolved)
                    .and_then(|m| m.value_si)
                    .filter(|v| v.is_finite() && *v > 0.0);
                let (os, ow) = (
                    params.v(&format!("{name}.stop.offset_s")),
                    params.v(&format!("{name}.stop.offset_w")),
                );
                let total: f64 = ow.iter().filter(|w| **w > 0.0).sum();
                if let (Some(c), true) = (cond, total > 0.0) {
                    stop_condition_read = true;
                    for k in 0..STOP_NODES {
                        if ow[k] > 0.0 {
                            let node = c + os[k];
                            passes.push((
                                ow[k] / total,
                                Some(control.map_or(node, |s| s.min(node))),
                                Some(k),
                            ));
                        }
                    }
                }
            }
            if passes.is_empty() {
                passes.push((1.0, control, None));
            }
            if let Some(s) = passes.iter().filter_map(|p| p.1).find(|s| *s <= tau) {
                return Ok(Err(refuse(
                    SupportGate::Semantic,
                    format!("a stop at {s} s is at or before the cutoff {tau} s"),
                )));
            }
            let substeps = params.s("num.substeps") as usize;
            let mut table = vec![vec![0.0; times.len()]; clocks.len() + 2];
            for (w, stop, _) in &passes {
                let part = event_table(&clocks, tau, *stop, &times, substeps);
                for (row, prow) in table.iter_mut().zip(&part) {
                    for (v, pv) in row.iter_mut().zip(prow) {
                        *v += w * pv;
                    }
                }
            }
            Outcome::Event {
                causes: t.causes.clone(),
                clocks,
                passes,
                stop_condition_read,
                times_s: times,
                table,
            }
        }
    };
    Ok(Ok(Forecast {
        slug: SLUGS[i].slug,
        target: name.clone(),
        cutoff_s: tau,
        features: x,
        outcome,
    }))
}

/// The lane-local identity of a target, carried by every artifact of it.
pub fn law_block(i: usize, t: &TargetDecl) -> Value {
    let d = decl(i);
    json!({
        "slug": SLUGS[i].slug,
        "status": d.status,
        "target": t.name,
        "kind": kind_id(t.kind),
        "label": format!("{}:{}", SLUGS[i].slug, t.name),
        "unit": match t.kind { Kind::Event => Some("s".to_string()), Kind::Binary => None, Kind::Positive => t.unit.clone() },
        "shared_quantity": t.shared_quantity,
        "description": t.description,
    })
}

fn kind_of(t: &TargetDecl) -> ArtifactKind {
    match t.kind {
        Kind::Positive => ArtifactKind::PrefixForecast,
        _ => ArtifactKind::EventProbabilities,
    }
}

fn body(i: usize, t: &TargetDecl, f: Forecast) -> Body {
    let mut diagnostics = json!({
        "law": law_block(i, t),
        "features": f.features,
        "outcome": f.outcome,
        "uncertainty_kind": "assumed_noise",
    });
    let mut b = Body {
        kind: Some(kind_of(t)),
        quantity: t.shared_quantity,
        times_s: vec![f.cutoff_s],
        ..Body::default()
    };
    match f.outcome {
        Outcome::Positive {
            levels,
            quantiles,
            mean,
            ..
        } => {
            b.quantile_levels = levels;
            b.quantiles = quantiles.into_iter().map(|q| vec![q]).collect();
            b.mean = mean.map(|m| vec![m]).unwrap_or_default();
        }
        Outcome::Binary { probability, .. } => {
            b.event_categories = t.categories();
            b.event_cdf = vec![vec![probability], vec![1.0 - probability]];
        }
        Outcome::Event { times_s, table, .. } => {
            b.times_s = times_s;
            b.event_categories = t.categories();
            b.event_cdf = table;
        }
    }
    diagnostics["times_note"] = json!(match t.kind {
        Kind::Event => "times_s are seconds since the time origin",
        _ => "times_s holds the cutoff only: the target is not a function of time",
    });
    b.diagnostics = diagnostics;
    b
}

/// The artifact of a forecast or of a scientific refusal (which keeps the target's kind, quantity
/// and lane-local label and carries no numbers).
pub fn artifact(
    i: usize,
    spec: &ModelSpec,
    release: &LoadedRelease,
    prefix: &PrefixBundle,
    settings: &Value,
) -> Result<AnalysisArtifact, Refusal> {
    let d = decl(i);
    let t = target_of(d, settings).unwrap_or(&d.targets[0]);
    Ok(match forecast(i, spec, release, prefix, settings)? {
        Ok(f) => common::artifact(release, prefix, settings, admitted(), body(i, t, f)),
        Err(support) => common::artifact(
            release,
            prefix,
            settings,
            support,
            Body {
                kind: Some(kind_of(t)),
                quantity: t.shared_quantity,
                diagnostics: json!({ "law": law_block(i, t) }),
                ..Body::default()
            },
        ),
    })
}
