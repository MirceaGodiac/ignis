//! `acme_normal.wsp4` and `acme_inverse.wsp4` — two hand-written laws of the ACME burner flames,
//! answering together.
//!
//! The second generation (`flame.rs`) wrote the life of a flame as hinge laws in the conditions; the
//! growth-clock forward (`flame3.rs`) wrote it as the published physics of a growing spherical flame
//! with a burner-heating law. They were typed from the same train table and make different mistakes:
//! one is an interpolation of the lives the table shows, the other a mechanism with few constants.
//! This forward holds one complete set of numbers for each and gives the weighted mean of their
//! answers:
//!
//! ```text
//! P(t) = (w_a P_a(t) + w_b P_b(t)) / (w_a + w_b)        row by row of
//!        (active_supported, extinction, operator_termination, unresolved_support)
//!
//! a   the tensors `a.*`: a full `flame.rs` release (spec acme_*-wsp-v2), run by `flame::forecast`
//! b   the tensors `b.*`: a full `flame3.rs` release (spec acme_*-wsp-v3), run by `flame3::forecast`
//! w   the tensor `mix.w`, two typed numbers
//! ```
//!
//! Each member is run unchanged, on the same prefix and the same forecast times, by the code its own
//! releases use. A weighted mean of two partitions of one is a partition of one, and a weighted mean
//! of two cumulative incidences is a cumulative incidence, so the output is a valid forecast of the
//! same kind. Read as a model, it says: with probability `w_a / (w_a + w_b)` the hinge law is the
//! right description of this flame, otherwise the growth law is. The weights are constants; nothing
//! in a prefix moves them.
//!
//! A member with weight zero is not run. If a member that is run refuses the prefix, the forecast is
//! refused with that member's reasons: the two laws are never silently swapped for one.
//!
//! Normal and inverse share this code and nothing else. Nothing is fitted on this route.

use anyhow::bail;
use serde::Serialize;
use serde_json::{json, Value};

use ignis_core::contracts::{
    weights_digest, AnalysisArtifact, ArtifactKind, Digest, Family, ForwardContract,
    ForwardProfile, ModelRelease, ModelSpec, PrefixBundle, QualificationState, SpecId, Support,
    SupportGate, TensorRole,
};
use ignis_core::expert::{Expert, ExpertResult, FitContext, FitData, Refusal};
use ignis_core::prefix::ProspectivePolicy;
use ignis_core::registry::{self, LoadedRelease, TensorMap};
use ignis_core::resources::Profile;
use ignis_core::views::View;

use crate::common::{self, admitted, refuse, Body};
use crate::spec::{authored, Params};
use crate::{flame, flame3};

pub const FORWARD_VERSION: &str = "4";
/// Tensor-name prefix of the `flame.rs` member.
pub const MEMBER_A: &str = "a.";
/// Tensor-name prefix of the `flame3.rs` member.
pub const MEMBER_B: &str = "b.";
pub use flame3::{ALLOWED, CATEGORIES, FORBIDDEN, INTERPRETATION};

pub fn model_name(family: Family) -> &'static str {
    match family {
        Family::AcmeInverse => "acme_inverse.wsp4",
        _ => "acme_normal.wsp4",
    }
}

pub fn family_of_model(name: &str) -> Option<Family> {
    match name {
        "acme_normal.wsp4" => Some(Family::AcmeNormal),
        "acme_inverse.wsp4" => Some(Family::AcmeInverse),
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
        &[2],
        TensorRole::Weight,
        Some("1"),
        Some((0.0, 1.0e3)),
        "Weights of the two members, (a: the hinge-law clocks of flame.rs, b: the growth clock of \
         flame3.rs). The forecast is the weighted mean of the members' event tables, the weights \
         divided by their sum. A member with weight zero is not run.",
    )];
    for (prefix, member) in [(MEMBER_A, &m.a), (MEMBER_B, &m.b)] {
        for t in &member.tensors {
            let mut t = t.clone();
            t.name = format!("{prefix}{}", t.name);
            tensors.push(t);
        }
    }
    let spec_id = if family == Family::AcmeInverse {
        "acme_inverse-wsp-v4"
    } else {
        "acme_normal-wsp-v4"
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
            entry_point: "ignis_wsp::blend::forecast".into(),
            version: FORWARD_VERSION.into(),
            profile: ForwardProfile::StrictNoFitting,
        },
        notes: Some(format!(
            "Two members and two weights. The tensors a.* are, name for name after the prefix, the \
             tensors of spec {} ({}), run by ignis_wsp::flame::forecast; the tensors b.* are those of \
             spec {} ({}), run by ignis_wsp::flame3::forecast. Each member is run on the same prefix \
             and settings as a release of its own spec would be: its features, its domain, its \
             refusals and its numerics are its own (see the notes of those specs). The forecast is \
             P = (mix.w[0] P_a + mix.w[1] P_b) / (mix.w[0] + mix.w[1]) for every row of (active_supported, \
             extinction, operator_termination, unresolved_support) at every requested time. A member \
             with weight zero is not run; both weights zero, or a negative weight, is a refusal. If a \
             member that is run refuses the prefix, the forecast is refused with that member's gate \
             and reasons. The members must return the same forecast times. The features and the \
             domain listed here are those of member b; member a reads the first eight. Uncertainty \
             kind: assumed_noise.",
            m.a.spec_id, m.a_hash, m.b.spec_id, m.b_hash
        )),
    }
}

/// The member's tensors as a release of the member's own spec: the blend's record with the member's
/// spec, forward and tensor list in place. It exists only for the duration of one call and is what
/// lets the member's forward run unchanged; it is never stored and has no id of its own.
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

/// The numbers of one forecast.
#[derive(Debug, Clone, Serialize)]
pub struct Forecast {
    /// The two weights divided by their sum.
    pub weights: [f64; 2],
    /// The members' own forecasts; `None` for a member with weight zero.
    pub a: Option<flame::Forecast>,
    pub b: Option<flame3::Forecast>,
    pub times_s: Vec<f64>,
    /// Rows in [`CATEGORIES`] order.
    pub table: [Vec<f64>; 4],
}

fn refused_by(member: &str, s: Support) -> Support {
    Support {
        admitted: false,
        gate: s.gate,
        reasons: s
            .reasons
            .into_iter()
            .map(|r| format!("member {member}: {r}"))
            .collect(),
    }
}

/// Strict forward of a blend release on an admitted prefix: `Ok(Err(support))` is a scientific
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
    let w = params.v("mix.w");
    let (wa, wb) = (w[0], w[1]);
    if !(wa >= 0.0 && wb >= 0.0 && wa + wb > 0.0) {
        return Ok(Err(refuse(
            SupportGate::Semantic,
            format!("mix.w = [{wa}, {wb}] is not a pair of weights: both must be >= 0 and one > 0"),
        )));
    }
    let weights = [wa / (wa + wb), wb / (wa + wb)];
    let m = members(spec.family);
    let a = if wa > 0.0 {
        let r = member_release(release, &m.a, &m.a_hash, MEMBER_A);
        match flame::forecast(&m.a, &r, prefix, settings)? {
            Ok(f) => Some(f),
            Err(s) => return Ok(Err(refused_by("a (flame.rs)", s))),
        }
    } else {
        None
    };
    let b = if wb > 0.0 {
        let r = member_release(release, &m.b, &m.b_hash, MEMBER_B);
        match flame3::forecast(&m.b, &r, prefix, settings)? {
            Ok(f) => Some(f),
            Err(s) => return Ok(Err(refused_by("b (flame3.rs)", s))),
        }
    } else {
        None
    };
    let (times_s, table) = match (&a, &b) {
        (Some(fa), Some(fb)) => {
            if fa.times_s != fb.times_s {
                return Err(Refusal::failed(
                    "blend_times",
                    format!(
                        "the members returned different forecast times ({} and {})",
                        fa.times_s.len(),
                        fb.times_s.len()
                    ),
                ));
            }
            let row = |k: usize| -> Vec<f64> {
                fa.table[k]
                    .iter()
                    .zip(&fb.table[k])
                    .map(|(x, y)| weights[0] * x + weights[1] * y)
                    .collect()
            };
            (fa.times_s.clone(), [row(0), row(1), row(2), row(3)])
        }
        (Some(fa), None) => (fa.times_s.clone(), fa.table.clone()),
        (None, Some(fb)) => (fb.times_s.clone(), fb.table.clone()),
        (None, None) => unreachable!("one weight is positive"),
    };
    Ok(Ok(Forecast {
        weights,
        a,
        b,
        times_s,
        table,
    }))
}

fn body(f: Forecast) -> Body {
    let diagnostics = json!({
        "interpretation": INTERPRETATION,
        "weights": {"a": f.weights[0], "b": f.weights[1]},
        "members": {
            "a": f.a.as_ref().map(|m| json!({
                "forward": "ignis_wsp::flame::forecast",
                "clocks": m.clocks,
                "p_weak_at_cutoff": m.p_weak_at_cutoff,
                "scheduled_stop_s": m.scheduled_stop_s,
                "event_cdf": m.table,
            })),
            "b": f.b.as_ref().map(|m| json!({
                "forward": "ignis_wsp::flame3::forecast",
                "burner": m.burner,
                "clocks": m.clocks,
                "p_weak_at_cutoff": m.p_weak_at_cutoff,
                "p_held_at_cutoff": m.p_held_at_cutoff,
                "scheduled_stop_s": m.scheduled_stop_s,
                "event_cdf": m.table,
            })),
        },
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
pub struct BlendWsp {
    spec: ModelSpec,
    policy: ProspectivePolicy,
}

impl BlendWsp {
    pub fn new(family: Family) -> Self {
        BlendWsp {
            spec: spec(family),
            policy: ProspectivePolicy::new(family, &ALLOWED, &FORBIDDEN),
        }
    }
    pub fn spec(&self) -> &ModelSpec {
        &self.spec
    }
}

impl Expert for BlendWsp {
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
