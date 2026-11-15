//! `acme_normal.wsp5` and `acme_inverse.wsp5` — a committee of hand-written scenarios for the ACME
//! burner flames.
//!
//! A law typed from seven test days says one thing about a flame. The train tables say more than one
//! thing in several places: a lean inverse flame at a high burner flow held to the script stop where
//! its neighbours went out in seconds; butane flames at 2 atm cycled between hot and cool where the
//! same flow a little higher went out for good. A single law has to call such a flame one way and is
//! then certain of it. A committee release holds up to six complete sets of numbers, each a full
//! release of one of the two existing burner-flame forwards, with a typed weight each:
//!
//! ```text
//! P(t) = sum_k w_k P_k(t) / sum_k w_k        row by row of
//!        (active_supported, extinction, operator_termination, unresolved_support)
//!
//! slots a0, a1          tensors `a0.*`, `a1.*`: full `flame.rs` releases, run by `flame::forecast`
//! slots b0, b1, b2, b3  tensors `b0.*` ... `b3.*`: full `flame3.rs` releases, run by `flame3::forecast`
//! w                     the tensor `mix.w`, six typed numbers in slot order
//! ```
//!
//! Read as a model, a weight is the author's probability that the member's description is the one
//! that holds for a flame; the forecast is the total probability over the descriptions. Members
//! that agree add nothing; where they differ the forecast says how sure the author is. The weights
//! are constants: nothing in a prefix moves them.
//!
//! Every member is run unchanged, on the same prefix and the same forecast times, by the code its
//! own releases use. A weighted mean of partitions of one is a partition of one, and a weighted mean
//! of cumulative incidences is a cumulative incidence, so the output is a forecast of the same kind.
//! A slot with weight zero is not run (its tensors still have to be complete: a release is its whole
//! spec). If a member that is run refuses the prefix, the forecast is refused with that member's
//! reasons.
//!
//! # The script
//!
//! A test of this campaign is run by a script whose stage durations are set before ignition. If the
//! flame is still burning when the constant-flow stage ends, the script stops the flow: that stop is
//! known in advance to the second, and the member laws have had to guess it as campaign policy
//! (stops at about 61, 98, 126, 275 s with typed frequencies). When a prefix carries the condition
//! `programmed_stop` (the sum of the script's stage durations up to the end of its last
//! constant-flow stage, as the test's parameter file gives them) this forward
//!
//! * hands every member the prefix with a control stage `scheduled_stop` at `programmed_stop -
//!   script.onset_s` (the records count time from the established flame; `script.onset_s` is the
//!   typed offset between the stage sum and the time on that clock at which a scripted stop
//!   falls), which the members already know how to use: the mass still active at that time goes
//!   to operator termination; and
//! * weighs the members with `mix.w_script` instead of `mix.w`, so that the slots used then can hold
//!   the same laws with their guessed stops switched off.
//!
//! Without the condition, or with `mix.w_script` all zero, nothing changes.
//!
//! This generalises `blend.rs` (one member of each kind), whose releases keep their own forward.
//! Normal and inverse share this code and nothing else. Nothing is fitted on this route.

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
use crate::{flame, flame3};

pub const FORWARD_VERSION: &str = "5";
/// Slots of `flame.rs` members.
pub const A_SLOTS: usize = 2;
/// Slots of `flame3.rs` members.
pub const B_SLOTS: usize = 4;
pub const SLOTS: usize = A_SLOTS + B_SLOTS;
/// Tensor-name prefixes of the slots, in the order of `mix.w`.
pub const SLOT_PREFIX: [&str; SLOTS] = ["a0.", "a1.", "b0.", "b1.", "b2.", "b3."];
pub use flame3::{CATEGORIES, FORBIDDEN, INTERPRETATION};
/// The condition that carries the script's programmed stop, in seconds from its ignition stamp.
pub const PROGRAMMED_STOP: &str = "programmed_stop";
/// Condition keys a prefix may carry: those of the member forwards, and the programmed stop.
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
        Family::AcmeInverse => "acme_inverse.wsp5",
        _ => "acme_normal.wsp5",
    }
}

pub fn family_of_model(name: &str) -> Option<Family> {
    match name {
        "acme_normal.wsp5" => Some(Family::AcmeNormal),
        "acme_inverse.wsp5" => Some(Family::AcmeInverse),
        _ => None,
    }
}

/// The two member specs of a family with their hashes, built once per process.
struct Members {
    a: ModelSpec,
    a_hash: Digest,
    b: ModelSpec,
    b_hash: Digest,
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
        let (a, b) = (flame::spec(family), flame3::spec(family));
        Members {
            a_hash: a.spec_hash(),
            b_hash: b.spec_hash(),
            a,
            b,
        }
    })
}

pub fn spec(family: Family) -> ModelSpec {
    assert!(matches!(family, Family::AcmeNormal | Family::AcmeInverse));
    let m = members(family);
    let mut tensors = vec![authored(
        "mix.w",
        &[SLOTS as u32],
        TensorRole::Weight,
        Some("1"),
        Some((0.0, 1.0e3)),
        "Weights of the six slots, in the order (a0, a1: flame.rs members; b0, b1, b2, b3: flame3.rs \
         members). A weight is the author's probability that the member's description holds; the \
         forecast is the weighted mean of the members' event tables, the weights divided by their \
         sum. A slot with weight zero is not run.",
    )];
    tensors.push(authored(
        "mix.w_script",
        &[SLOTS as u32],
        TensorRole::Weight,
        Some("1"),
        Some((0.0, 1.0e3)),
        "Weights of the six slots when the prefix carries the script's programmed stop (condition \
         programmed_stop). All zero: the programmed stop is not read and mix.w is used.",
    ));
    tensors.push(authored(
        "script.onset_s",
        &[],
        TensorRole::PhysicalParameter,
        Some("s"),
        Some((0.0, 20.0)),
        "Offset between the stored programmed stop (the sum of the script's stage durations) and the \
         time, counted from the established flame as the records count it, at which a scripted stop \
         falls: the scheduled stop handed to the members is programmed_stop minus this.",
    ));
    for (i, prefix) in SLOT_PREFIX.iter().enumerate() {
        let member = if i < A_SLOTS { &m.a } else { &m.b };
        for t in &member.tensors {
            let mut t = t.clone();
            t.name = format!("{prefix}{}", t.name);
            tensors.push(t);
        }
    }
    let spec_id = if family == Family::AcmeInverse {
        "acme_inverse-wsp-v5"
    } else {
        "acme_normal-wsp-v5"
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
            entry_point: "ignis_wsp::committee::forecast".into(),
            version: FORWARD_VERSION.into(),
            profile: ForwardProfile::StrictNoFitting,
        },
        notes: Some(format!(
            "Six slots and six weights. The tensors a0.* and a1.* are, name for name after the \
             prefix, the tensors of spec {} ({}), run by ignis_wsp::flame::forecast; the tensors \
             b0.* to b3.* are those of spec {} ({}), run by ignis_wsp::flame3::forecast. Each member \
             is run on the same prefix and settings as a release of its own spec would be: its \
             features, its domain, its refusals and its numerics are its own (see the notes of those \
             specs). The forecast is P = sum_k mix.w[k] P_k / sum_k mix.w[k] over the slots with a \
             positive weight, for every row of (active_supported, extinction, operator_termination, \
             unresolved_support) at every requested time. A slot with weight zero is not run; all \
             weights zero, or a negative weight, is a refusal. If a member that is run refuses the \
             prefix, the forecast is refused with that member's gate and reasons. The members must \
             return the same forecast times. The features and the domain listed here are those of \
             the flame3.rs members; the flame.rs members read the first eight. When the prefix carries \
             a resolved, finite condition programmed_stop and mix.w_script has a positive entry, the \
             weights are mix.w_script and every member is run on the prefix without that condition and \
             with one more control stage, scheduled_stop at programmed_stop - script.onset_s seconds \
             (a stop at or before the cutoff refuses the forecast, as the members define). Uncertainty \
             kind: assumed_noise.",
            m.a.spec_id, m.a_hash, m.b.spec_id, m.b_hash
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

/// What one member said (kept for the diagnostics).
#[derive(Debug, Clone, Serialize)]
pub struct MemberOutput {
    pub slot: &'static str,
    /// The slot's weight divided by the sum of the weights.
    pub weight: f64,
    pub p_weak_at_cutoff: f64,
    pub scheduled_stop_s: Option<f64>,
    /// Rows in [`CATEGORIES`] order.
    pub table: [Vec<f64>; 4],
    /// The member's own clocks (a `flame.rs` or a `flame3.rs` structure).
    pub clocks: Value,
}

/// The numbers of one forecast.
#[derive(Debug, Clone, Serialize)]
pub struct Forecast {
    /// The members that were run, in slot order.
    pub members: Vec<MemberOutput>,
    /// The stop handed to the members from the script's programmed stop, when the prefix carried it.
    pub scheduled_stop_s: Option<f64>,
    pub times_s: Vec<f64>,
    /// Rows in [`CATEGORIES`] order.
    pub table: [Vec<f64>; 4],
}

fn refused_by(slot: &str, s: Support) -> Support {
    Support {
        admitted: false,
        gate: s.gate,
        reasons: s
            .reasons
            .into_iter()
            .map(|r| format!("member {}: {r}", slot.trim_end_matches('.')))
            .collect(),
    }
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
    if matches!(
        release.release.state,
        QualificationState::Failed | QualificationState::Unsupported
    ) {
        return Ok(Err(refuse(
            SupportGate::Semantic,
            format!("release state {:?}", release.release.state),
        )));
    }
    // the script's programmed stop, when the prefix carries it and the release reads it
    let w_script = params.v("mix.w_script");
    let programmed = prefix
        .known_conditions
        .get(PROGRAMMED_STOP)
        .filter(|m| m.resolution == Resolution::Resolved)
        .and_then(|m| m.value_si)
        .filter(|v| v.is_finite() && *v > 0.0)
        .filter(|_| w_script.iter().any(|x| *x > 0.0));
    let scripted: Option<PrefixBundle> = programmed.map(|t| {
        let mut p = prefix.clone();
        p.known_conditions.remove(PROGRAMMED_STOP);
        p.known_controls.push(ControlStage {
            t_start_s: t - params.s("script.onset_s"),
            t_end_s: None,
            control: "scheduled_stop".into(),
            value: None,
            operator_action: false,
        });
        p
    });
    let (w, prefix) = match &scripted {
        Some(p) => (w_script, p),
        None => (params.v("mix.w"), prefix),
    };
    let total: f64 = w.iter().sum();
    if !(w.iter().all(|x| *x >= 0.0) && total > 0.0) {
        return Ok(Err(refuse(
            SupportGate::Semantic,
            format!("mix.w or mix.w_script = {w:?} is not a set of weights: all must be >= 0 and one > 0"),
        )));
    }
    let m = members(spec.family);
    let mut out: Vec<MemberOutput> = Vec::new();
    let mut times: Option<Vec<f64>> = None;
    for (i, slot) in SLOT_PREFIX.iter().enumerate() {
        if w[i] <= 0.0 {
            continue;
        }
        let (t, member) = if i < A_SLOTS {
            let r = member_release(release, &m.a, &m.a_hash, slot);
            match flame::forecast(&m.a, &r, prefix, settings)? {
                Ok(f) => (
                    f.times_s,
                    MemberOutput {
                        slot,
                        weight: w[i] / total,
                        p_weak_at_cutoff: f.p_weak_at_cutoff,
                        scheduled_stop_s: f.scheduled_stop_s,
                        table: f.table,
                        clocks: json!(f.clocks),
                    },
                ),
                Err(s) => return Ok(Err(refused_by(slot, s))),
            }
        } else {
            let r = member_release(release, &m.b, &m.b_hash, slot);
            match flame3::forecast(&m.b, &r, prefix, settings)? {
                Ok(f) => (
                    f.times_s,
                    MemberOutput {
                        slot,
                        weight: w[i] / total,
                        p_weak_at_cutoff: f.p_weak_at_cutoff,
                        scheduled_stop_s: f.scheduled_stop_s,
                        table: f.table,
                        clocks: json!({"burner": f.burner, "clocks": f.clocks, "p_held_at_cutoff": f.p_held_at_cutoff}),
                    },
                ),
                Err(s) => return Ok(Err(refused_by(slot, s))),
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
    let times_s = times.expect("one weight is positive");
    let row = |k: usize| -> Vec<f64> {
        (0..times_s.len())
            .map(|j| out.iter().map(|m| m.weight * m.table[k][j]).sum())
            .collect()
    };
    // one member alone answers with its own table, bit for bit
    let table = if out.len() == 1 {
        out[0].table.clone()
    } else {
        [row(0), row(1), row(2), row(3)]
    };
    Ok(Ok(Forecast {
        members: out,
        scheduled_stop_s: scripted.as_ref().and_then(|p| {
            p.known_controls
                .iter()
                .rev()
                .find(|c| c.control == "scheduled_stop")
                .map(|c| c.t_start_s)
        }),
        times_s,
        table,
    }))
}

fn body(f: Forecast) -> Body {
    let diagnostics = json!({
        "interpretation": INTERPRETATION,
        "scheduled_stop_from_script_s": f.scheduled_stop_s,
        "members": f.members.iter().map(|m| json!({
            "slot": m.slot.trim_end_matches('.'),
            "forward": if m.slot.starts_with('a') { "ignis_wsp::flame::forecast" } else { "ignis_wsp::flame3::forecast" },
            "weight": m.weight,
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
pub struct CommitteeWsp {
    spec: ModelSpec,
    policy: ProspectivePolicy,
}

impl CommitteeWsp {
    pub fn new(family: Family) -> Self {
        CommitteeWsp {
            spec: spec(family),
            policy: ProspectivePolicy::new(family, &ALLOWED, &FORBIDDEN),
        }
    }
    pub fn spec(&self) -> &ModelSpec {
        &self.spec
    }
}

impl Expert for CommitteeWsp {
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
