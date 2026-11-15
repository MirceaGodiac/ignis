//! Persistent input-condition nodes around the unchanged ACME `committee` forward.
//!
//! A node is one whole trajectory, with an authored probability mass conditional on the
//! admitted cutoff. It is not an ignition prior, and is not survivor-updated again. Law
//! disagreement remains inside the predecessor committee. The outer five-node rule can
//! express a central node and paired flow/pressure deviations, or correlated joint nodes;
//! its coordinates and masses are all authored, never generated or selected from outcomes.
//!
//! Only reactant mass flow and pressure vary. Composition, supplied `zst` and supplied
//! `tad` stay fixed together: the predecessor does not recompute equilibrium chemistry.
//! Pressure scenarios therefore explicitly condition on the supplied flame temperature.
//! Burner quantities derived from flow are recomputed by the member forwards. This is
//! assumed input uncertainty, not measured apparatus precision or calibrated coverage.

use anyhow::bail;
use serde::Serialize;
use serde_json::{json, Value};

use ignis_core::contracts::{
    weights_digest, AnalysisArtifact, ArtifactKind, EvidenceClass, Family, Measured, ModelRelease,
    ModelSpec, PrefixBundle, Resolution, SpecId, Support, SupportGate, TensorRole,
};
use ignis_core::expert::{Expert, ExpertResult, FitContext, FitData, Refusal};
use ignis_core::prefix::ProspectivePolicy;
use ignis_core::registry::{self, LoadedRelease, TensorMap};
use ignis_core::resources::Profile;
use ignis_core::views::View;

use crate::committee;
use crate::common::{self, admitted, refuse, Body};
use crate::spec::{authored, Params};

pub const FORWARD_VERSION: &str = "6";
/// Fixed quadrature capacity: one central node and two pairs of condition deviations.
pub const NODES: usize = 5;
pub use committee::{ALLOWED, CATEGORIES, FORBIDDEN, INTERPRETATION};

pub fn model_name(family: Family) -> &'static str {
    match family {
        Family::AcmeInverse => "acme_inverse.wsp6",
        _ => "acme_normal.wsp6",
    }
}

pub fn family_of_model(name: &str) -> Option<Family> {
    match name {
        "acme_normal.wsp6" => Some(Family::AcmeNormal),
        "acme_inverse.wsp6" => Some(Family::AcmeInverse),
        _ => None,
    }
}

pub fn spec(family: Family) -> ModelSpec {
    let mut s = committee::spec(family);
    let predecessor_notes = s.notes.take().unwrap_or_default();
    s.spec_id = SpecId::new(if family == Family::AcmeInverse {
        "acme_inverse-wsp-v6"
    } else {
        "acme_normal-wsp-v6"
    })
    .expect("id");
    s.model_name = model_name(family).into();
    s.forward.entry_point = "ignis_wsp::condition_committee::forecast".into();
    s.forward.version = FORWARD_VERSION.into();
    for (name, role, description) in [
        (
            "condition.w",
            TensorRole::Weight,
            "Five finite nonnegative node masses, at least one positive. Normalized once at the admitted cutoff; fixed for every horizon and every event category. Not ignition priors or calibrated probabilities.",
        ),
        (
            "condition.flow_multiplier",
            TensorRole::PhysicalParameter,
            "Five finite strictly positive multipliers of injected_mass_flow, one per persistent node. One is neutral. Every member recomputes its flow-derived features.",
        ),
        (
            "condition.pressure_multiplier",
            TensorRole::PhysicalParameter,
            "Five finite strictly positive multipliers of pressure, one per persistent node. One is neutral. Composition, zst and supplied tad remain fixed; this is conditional pressure uncertainty without an equilibrium-temperature recomputation.",
        ),
    ] {
        s.tensors.push(authored(
            name,
            &[NODES as u32],
            role,
            Some("1"),
            None,
            description,
        ));
    }
    s.notes = Some(format!(
        "Persistent condition mixture over the unchanged wsp5 committee. All inherited tensors \
         retain their meaning. condition.w is normalized once and applies to complete event tables, \
         conditional on the admitted cutoff; no extra survivor update or observation likelihood. \
         Five authored joint nodes vary only pressure and reactant mass flow. Composition, zst and \
         tad are held fixed together; pressure variation is conditional on supplied tad, not a \
         thermochemical equilibrium calculation. All nodes must be finite with positive multipliers, \
         even at zero mass. Any positive-mass node refusal refuses the entire mixture. A single \
         neutral node is exactly the predecessor. Uncertainty kind: assumed_noise. Predecessor: \
         {predecessor_notes}"
    ));
    s
}

/// Ephemeral adapter only; no synthetic member record is saved as a release or evidence.
fn predecessor_release(release: &LoadedRelease, spec: &ModelSpec) -> LoadedRelease {
    let params: TensorMap = spec
        .tensors
        .iter()
        .map(|t| (t.name.clone(), release.params[&t.name].clone()))
        .collect();
    let frozen = params.keys().cloned().collect::<Vec<_>>();
    let entries = registry::tensor_entries(&params, &frozen);
    let mut r = release.release.clone();
    r.model_name = spec.model_name.clone();
    r.feature_order = spec.features.iter().map(|f| f.name.clone()).collect();
    r.units = spec
        .features
        .iter()
        .map(|f| (f.name.clone(), f.unit.clone()))
        .collect();
    if let Some(m) = r.construction.as_mut() {
        m.spec_id = spec.spec_id.clone();
        m.spec_hash = spec.spec_hash();
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

#[derive(Debug, Clone, Serialize)]
pub struct NodeOutput {
    pub node: usize,
    pub weight: f64,
    pub flow_multiplier: f64,
    pub pressure_multiplier: f64,
    pub flow_kg_s: f64,
    pub pressure_pa: f64,
    /// Complete predecessor forecast, including its distinct law weights and event tables.
    pub forecast: committee::Forecast,
}

#[derive(Debug, Clone, Serialize)]
pub struct Forecast {
    pub nodes: Vec<NodeOutput>,
    pub times_s: Vec<f64>,
    /// Rows in [`CATEGORIES`] order; one common set of weights at every horizon.
    pub table: [Vec<f64>; 4],
}

pub fn forecast(
    spec: &ModelSpec,
    release: &LoadedRelease,
    prefix: &PrefixBundle,
    settings: &Value,
) -> Result<Result<Forecast, Support>, Refusal> {
    let params = Params::bind(spec, &release.release, &release.params)?;
    let this = crate::audit::forward_hash_of(&spec.model_name);
    if release
        .release
        .construction
        .as_ref()
        .map(|m| &m.forward_hash)
        != Some(&this)
    {
        return Err(Refusal::unsupported(
            "forward_mismatch",
            "condition committee forward differs from release construction",
        ));
    }
    let (w, flow, pressure) = (
        params.v("condition.w"),
        params.v("condition.flow_multiplier"),
        params.v("condition.pressure_multiplier"),
    );
    if [w.len(), flow.len(), pressure.len()] != [NODES; 3] {
        return Err(Refusal::failed(
            "release_tensors",
            "condition tensor data length differs from its shape",
        ));
    }
    if w.iter().any(|v| *v < 0.0)
        || !w.iter().any(|v| *v > 0.0)
        || flow.iter().chain(pressure).any(|v| *v <= 0.0)
    {
        return Ok(Err(refuse(
            SupportGate::Semantic,
            "condition nodes require nonnegative masses, one positive mass, and strictly positive flow/pressure multipliers in every slot",
        )));
    }
    // Scaling before summation avoids overflow for finite literal masses.
    let max_w = w.iter().copied().fold(0.0_f64, f64::max);
    let scaled_total: f64 = w.iter().map(|v| v / max_w).sum();
    let weights: Vec<_> = w.iter().map(|v| (v / max_w) / scaled_total).collect();
    if w.iter()
        .zip(&weights)
        .any(|(raw, normalized)| *raw > 0.0 && *normalized == 0.0)
    {
        return Ok(Err(refuse(
            SupportGate::Semantic,
            "condition mass normalization underflows a positive node",
        )));
    }
    let mut nominal = [0.0; 2];
    for (i, key) in ["injected_mass_flow", "pressure"].iter().enumerate() {
        nominal[i] = match common::condition(prefix, key) {
            Ok(v) if v > 0.0 => v,
            Ok(_) => {
                return Ok(Err(refuse(
                    SupportGate::Semantic,
                    format!("{key} must be positive"),
                )));
            }
            Err(reason) => return Ok(Err(refuse(SupportGate::Semantic, reason))),
        };
    }
    // The two mole fractions describe DIFFERENT streams: they need not sum to one.
    // Never perturb them while leaving their supplied thermochemical features stale.
    for (key, upper) in [
        ("x_o2_role", 1.0),
        ("x_fuel_role", 1.0),
        ("zst", 1.0),
        ("tad", f64::INFINITY),
    ] {
        match common::condition(prefix, key) {
            Ok(v) if v > 0.0 && v <= upper && (key != "zst" || v < 1.0) => {}
            Ok(v) => {
                return Ok(Err(refuse(
                    SupportGate::Semantic,
                    format!("{key} = {v} is physically invalid"),
                )));
            }
            Err(reason) => return Ok(Err(refuse(SupportGate::Semantic, reason))),
        }
    }
    let predecessor = committee::spec(spec.family);
    let member_release = predecessor_release(release, &predecessor);
    let mut nodes: Vec<NodeOutput> = Vec::new();
    for i in 0..NODES {
        if w[i] == 0.0 {
            continue;
        }
        let (flow_kg_s, pressure_pa) = (nominal[0] * flow[i], nominal[1] * pressure[i]);
        if ![flow_kg_s, pressure_pa]
            .iter()
            .all(|v| v.is_finite() && *v > 0.0)
        {
            return Ok(Err(refuse(
                SupportGate::Semantic,
                format!("condition node {i}: transformed flow/pressure is not finite and positive"),
            )));
        }
        // A private calculation copy, never emitted as a measurement or a new input artifact.
        // Replace perturbed entries with explicitly labelled model conditions, not stale raw data.
        let mut node_prefix = prefix.clone();
        for (key, value, unit, multiplier) in [
            ("injected_mass_flow", flow_kg_s, "kg/s", flow[i]),
            ("pressure", pressure_pa, "Pa", pressure[i]),
        ] {
            if multiplier == 1.0 {
                continue;
            }
            node_prefix.known_conditions.insert(
                key.into(),
                Measured {
                    raw_value: None,
                    raw_unit: None,
                    value_si: Some(value),
                    unit_si: Some(unit.into()),
                    uncertainty_si: None,
                    uncertainty_reason: Some(
                        "Authored persistent condition scenario, not a measurement".into(),
                    ),
                    evidence_class: EvidenceClass::ModelOutput,
                    locator: None,
                    derivation: None,
                    resolution: Resolution::Resolved,
                    flags: vec![
                        format!("authored_condition_node_{i}"),
                        "not_measured".into(),
                    ],
                },
            );
        }
        node_prefix.input_hash = ignis_core::hashing::content_hash(&node_prefix, &["input_hash"]);
        let forecast =
            match committee::forecast(&predecessor, &member_release, &node_prefix, settings)? {
                Ok(f) => f,
                Err(s) => {
                    return Ok(Err(Support {
                        admitted: false,
                        gate: s.gate,
                        reasons: s
                            .reasons
                            .into_iter()
                            .map(|r| format!("condition node {i}: {r}"))
                            .collect(),
                    }));
                }
            };
        if nodes
            .first()
            .is_some_and(|first| first.forecast.times_s != forecast.times_s)
        {
            return Err(Refusal::failed(
                "condition_times",
                "condition nodes returned different forecast times",
            ));
        }
        nodes.push(NodeOutput {
            node: i,
            weight: weights[i],
            flow_multiplier: flow[i],
            pressure_multiplier: pressure[i],
            flow_kg_s,
            pressure_pa,
            forecast,
        });
    }
    let times_s = nodes[0].forecast.times_s.clone();
    let table = if nodes.len() == 1
        || nodes
            .iter()
            .all(|n| n.forecast.table == nodes[0].forecast.table)
    {
        nodes[0].forecast.table.clone()
    } else {
        std::array::from_fn(|k| {
            (0..times_s.len())
                .map(|j| {
                    nodes
                        .iter()
                        .map(|n| n.weight * n.forecast.table[k][j])
                        .sum()
                })
                .collect()
        })
    };
    Ok(Ok(Forecast {
        nodes,
        times_s,
        table,
    }))
}

fn body(f: Forecast) -> Body {
    Body {
        kind: Some(ArtifactKind::EventProbabilities),
        times_s: f.times_s,
        event_categories: CATEGORIES.iter().map(|s| s.to_string()).collect(),
        event_cdf: f.table.to_vec(),
        diagnostics: json!({
            "interpretation": INTERPRETATION,
            "uncertainty_kind": "assumed_noise",
            "condition_weight_scope": "conditional_on_admitted_cutoff_constant_over_whole_trajectory",
            "condition_scope": "pressure_and_reactant_mass_flow_only; composition_zst_and_supplied_tad_fixed; no_equilibrium_recomputation",
            "conditioning": "no_additional_survivor_update; predecessor_prefix_conditioning_unchanged",
            "input_semantics": "node_values_are_assumed_model_conditions_not_measurements; original_prefix_preserved",
            "nodes": f.nodes,
        }),
        ..Body::default()
    }
}

pub fn artifact(
    spec: &ModelSpec,
    release: &LoadedRelease,
    prefix: &PrefixBundle,
    settings: &Value,
) -> Result<AnalysisArtifact, Refusal> {
    Ok(match forecast(spec, release, prefix, settings)? {
        Ok(f) => common::artifact(release, prefix, settings, admitted(), body(f)),
        Err(s) => common::artifact(
            release,
            prefix,
            settings,
            s,
            Body {
                kind: Some(ArtifactKind::EventProbabilities),
                ..Body::default()
            },
        ),
    })
}

pub struct ConditionCommitteeWsp {
    spec: ModelSpec,
    policy: ProspectivePolicy,
}

impl ConditionCommitteeWsp {
    pub fn new(family: Family) -> Self {
        Self {
            spec: spec(family),
            policy: ProspectivePolicy::new(family, &ALLOWED, &FORBIDDEN),
        }
    }
    pub fn spec(&self) -> &ModelSpec {
        &self.spec
    }
}

impl Expert for ConditionCommitteeWsp {
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
        bail!(
            "{} has no fitting procedure; assemble a literal patch",
            self.spec.model_name
        )
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
            Err(s) => Err(common::refusal_of(&s).into()),
        }
    }
    fn log_likelihood(&self, _: &LoadedRelease, _: &View) -> ExpertResult<f64> {
        bail!(
            "{} exposes event probabilities, not a likelihood",
            self.spec.model_name
        )
    }
}
