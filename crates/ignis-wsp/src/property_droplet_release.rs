//! Source-bound release adapter for the frozen clean-room droplet kernel.
//! Internal host API only; no material values or scope are accepted through ForecastSettings.

use std::collections::{BTreeMap, BTreeSet};

use ignis_core::analysis::{validate_artifact, ForecastSettings};
use ignis_core::contracts::{
    AnalysisArtifact, Capability, Digest, EvidenceClass, ExecutionScope, Family, ForecastInput,
    ForecastRequest, HandwrittenWeightPatch, ModelRelease, ModelSpec, PatchId, PrefixBundle,
    Quantity, Resolution, SourceLocator, SpecId, SupportGate,
};
use ignis_core::expert::Refusal;
use ignis_core::hashing::content_hash;
use ignis_core::prefix::{assert_isolated, validate_observation, ALWAYS_FORBIDDEN};
use ignis_core::registry::{self, LoadedRelease};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::common::{self, Body};
use crate::property_droplet_cleanroom::{self as kernel, Chemistry, Input, Origin, Supplied};
use crate::service::Provenance;

type Result<T> = std::result::Result<T, Refusal>;

pub const MODEL_NAME: &str = "flex.property_droplet.source_bound.v1";

/// A distinct adapter identity; the original W0 spec, patch and kernel remain unchanged.
pub fn specification() -> anyhow::Result<ModelSpec> {
    let mut spec = kernel::specification()?;
    spec.spec_id = SpecId::new("flex-property-droplet-source-bound-v1")?;
    spec.model_name = MODEL_NAME.into();
    spec.forward.entry_point = "ignis_wsp::property_droplet_release::forecast".into();
    spec.forward.version = "1".into();
    spec.notes = Some(format!(
        "Source-bound adapter of frozen W0 spec {}. Identical tensors and physical laws; no new physical or numerical assumptions. Explicit initial-state time, full source ledger, prospective prefix, stored tensors, and explicit future times required. Point prediction only; no predictive distribution. Material packet is supplied by the host, never a request argument.",
        kernel::specification()?.spec_hash()
    ));
    spec.validate()?;
    Ok(spec)
}

/// Rebind the unchanged literal values to the adapter spec, preserving author/source provenance.
pub fn authored_patch() -> anyhow::Result<HandwrittenWeightPatch> {
    let mut patch = kernel::authored_patch()?;
    let original = patch.patch_hash();
    patch.patch_id = PatchId::new("flex-property-droplet-source-bound-v1")?;
    patch.spec_hash = specification()?.spec_hash();
    patch.rationale = Some(format!(
        "Integration derivative of frozen physics patch {original}; literal operations are unchanged. Source-bound adapter identity, not a new independent zero-shot scientific experiment. No observed NASA outcomes or calibration values were supplied to this adapter author."
    ));
    patch.validate_against(&specification()?)?;
    Ok(patch)
}

/// Exact-byte dependency identity for this adapter, including algebra and artifact construction.
pub fn forward_hash() -> Digest {
    Digest::of_json(&[
        (
            "property_droplet_release.rs",
            include_str!("property_droplet_release.rs"),
        ),
        (
            "property_droplet_cleanroom.rs",
            include_str!("property_droplet_cleanroom.rs"),
        ),
        ("physical_core.rs", include_str!("physical_core.rs")),
        ("common.rs", include_str!("common.rs")),
    ])
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ReviewedUse {
    SourceOnly,
    /// Host scientific admission artifact, not a generic run-activity marker.
    EstablishedReactionSheet {
        admission_artifact: Digest,
        physical_justification: String,
        known_at_s: f64,
        valid_from_s: f64,
        valid_until_s: f64,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InputEvidence {
    pub source: String,
    /// Original scalar/property locator text retained by the frozen kernel input.
    pub locator_text: String,
    pub locator: SourceLocator,
    pub available_at_s: f64,
    pub observed_at_s: Option<f64>,
    pub evidence_class: EvidenceClass,
    /// Hash of the exact item returned by required_input_items; includes raw/SI/domain/reference.
    pub item_hash: Digest,
    /// Explicit dependencies on admitted data; no full-run or continuation identifiers.
    pub observation_ids: Vec<String>,
    pub condition_keys: Vec<String>,
    pub reviewed_use: ReviewedUse,
}

/// Constructed only by host data/material access. This is not a wire/request contract.
#[derive(Debug)]
pub struct SourceBoundInputs {
    input: Input,
    initial_time_s: f64,
    prefix_hash: String,
    material_snapshot: Digest,
    evidence: BTreeMap<String, InputEvidence>,
    snapshot: Value,
    hash: Digest,
}

/// Host implementations resolve metadata before bytes; scope/partition checks precede resolve.
pub trait InputSource: Send + Sync {
    fn provenance(
        &self,
        scope: &ExecutionScope,
        release: &ModelRelease,
        prefix: &PrefixBundle,
    ) -> Result<Provenance>;
    fn resolve(
        &self,
        scope: &ExecutionScope,
        release: &ModelRelease,
        prefix: &PrefixBundle,
    ) -> Result<SourceBoundInputs>;
}

fn invalid(message: impl Into<String>) -> Refusal {
    Refusal::invalid("droplet_input_binding", message)
}

fn properties(input: &Input) -> BTreeMap<String, &Supplied> {
    let mut values = BTreeMap::new();
    for (index, c) in input.components.iter().enumerate() {
        for (name, p) in [
            ("initial_mass", &c.initial_mass),
            ("molar_mass", &c.molar_mass),
            ("liquid_density", &c.liquid_density),
            ("liquid_heat_capacity", &c.liquid_heat_capacity),
            ("latent_heat_at_reference", &c.latent_heat_at_reference),
            (
                "saturation_pressure_at_reference",
                &c.saturation_pressure_at_reference,
            ),
        ] {
            values.insert(format!("components/{index}/{name}"), p);
        }
        if let Some(p) = &c.complete_combustion_heat {
            values.insert(format!("components/{index}/complete_combustion_heat"), p);
        }
    }
    for (name, p) in [
        ("initial_temperature", &input.initial_temperature),
        ("ambient_temperature", &input.ambient_temperature),
        ("pressure", &input.pressure),
        ("gas_density", &input.gas_density),
        ("gas_heat_capacity", &input.gas_heat_capacity),
        ("gas_conductivity", &input.gas_conductivity),
        ("common_diffusivity", &input.common_diffusivity),
        ("gas_viscosity", &input.gas_viscosity),
        (
            "surface_background_molar_mass",
            &input.surface_background_molar_mass,
        ),
        ("oxygen_molar_mass", &input.oxygen_molar_mass),
        (
            "ambient_oxygen_mass_fraction",
            &input.ambient_oxygen_mass_fraction,
        ),
    ] {
        values.insert(name.into(), p);
    }
    for (i, p) in input.relative_velocity.iter().enumerate() {
        values.insert(format!("relative_velocity/{i}"), p);
    }
    for (i, p) in input.gravity.iter().enumerate() {
        values.insert(format!("gravity/{i}"), p);
    }
    values
}

fn property_value(p: &Supplied) -> Value {
    let uncertainty = match p.uncertainty {
        kernel::Uncertainty::Unquantified => json!({"kind":"unquantified"}),
        kernel::Uncertainty::AbsoluteBoundInSi(v) => {
            json!({"kind":"absolute_bound_in_si","value":v})
        }
    };
    json!({
        "raw_value":p.raw_value,"raw_unit":p.raw_unit,"si_value":p.si_value,
        "si_unit":format!("{:?}",p.si_unit),"phase":format!("{:?}",p.phase),
        "source":p.source,"locator":p.locator,"origin":format!("{:?}",p.origin),
        "uncertainty":uncertainty,"reference_temperature_k":p.reference_temperature_k,
        "reference_pressure_pa":p.reference_pressure_pa,"reference_identity":p.reference_identity,
        "temperature_domain_k":p.temperature_domain_k,"pressure_domain_pa":p.pressure_domain_pa
    })
}

/// Exact required evidence inventory, including closure declarations and all physical metadata.
/// These values are already supplied; this function neither estimates nor fills anything.
pub fn required_input_items(input: &Input, initial_time_s: f64) -> BTreeMap<String, Value> {
    let mut items: BTreeMap<_, _> = properties(input)
        .into_iter()
        .map(|(name, p)| (name, property_value(p)))
        .collect();
    for (i, component) in input.components.iter().enumerate() {
        let a = component.atoms;
        items.insert(format!("components/{i}/identity_and_composition"), json!({"identity":component.identity,"CHONS_He_Ar":[a.carbon,a.hydrogen,a.oxygen,a.nitrogen,a.sulfur,a.helium,a.argon]}));
    }
    items.insert("initial_time_s".into(), json!(initial_time_s));
    items.insert("time_origin".into(), json!(input.time_origin));
    items.insert("coordinate_frame".into(), json!(input.coordinate_frame));
    items.insert(
        "activity_closure".into(),
        match &input.activity {
            kernel::ActivityClosure::IdealSolution {
                source,
                applicability,
            } => json!({"kind":"ideal_solution","source":source,"applicability":applicability}),
            kernel::ActivityClosure::Unsupported { reason } => {
                json!({"kind":"unsupported","reason":reason})
            }
        },
    );
    items.insert("chemistry_closure".into(), match &input.chemistry {
        Chemistry::EvaporationOnly => json!({"kind":"evaporation_only"}),
        Chemistry::EstablishedReactionSheet { source, applicability } => json!({"kind":"established_reaction_sheet","source":source,"applicability":applicability}),
    });
    items.insert(
        "transport_applicability".into(),
        json!(format!("{:?}", input.transport)),
    );
    items.insert("constant_environment".into(), json!("fixed boundaries, infinite reservoir, zero ambient fuel, common gas cp and common species diffusivity over declared domains"));
    items
}

fn prefix_admission(prefix: &PrefixBundle) -> Result<()> {
    if prefix.family != Family::Flex
        || !prefix.cutoff_s.is_finite()
        || prefix.cutoff_s < 0.0
        || prefix.time_origin.trim().is_empty()
    {
        return Err(invalid(
            "FLEX prefix, finite nonnegative cutoff and explicit time origin required",
        ));
    }
    assert_isolated(prefix, &[]).map_err(|e| invalid(e.to_string()))?;
    for key in prefix.known_conditions.keys() {
        if ALWAYS_FORBIDDEN.contains(&key.as_str()) {
            return Err(invalid(format!("forbidden condition {key}")));
        }
    }
    let mut ids = BTreeSet::new();
    for o in &prefix.observations {
        validate_observation(o).map_err(invalid)?;
        if o.run_id != prefix.run_id
            || !o.time_s.is_finite()
            || !o.available_at_s.is_finite()
            || !o.admitted_at(prefix.cutoff_s)
            || !ids.insert(&o.id)
        {
            return Err(invalid("invalid, duplicate or nonprospective observation"));
        }
    }
    for c in &prefix.contours {
        if c.run_id != prefix.run_id
            || !c.time_s.is_finite()
            || !c.available_at_s.is_finite()
            || c.time_s > prefix.cutoff_s
            || c.available_at_s > prefix.cutoff_s
        {
            return Err(invalid("nonprospective contour"));
        }
    }
    if prefix
        .phase_established_at_s
        .is_some_and(|t| !t.is_finite() || t < 0.0 || t > prefix.cutoff_s)
    {
        return Err(invalid("invalid phase establishment time"));
    }
    Ok(())
}

impl SourceBoundInputs {
    /// Read the internal, content-addressed host artifact emitted by snapshot().
    /// It is never accepted as ForecastSettings or a tool argument.
    pub fn from_snapshot(prefix: &PrefixBundle, value: Value) -> Result<Self> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Snapshot {
            schema: String,
            run_id: String,
            prefix_hash: String,
            material_snapshot: Digest,
            items: BTreeMap<String, Value>,
            evidence: BTreeMap<String, InputEvidence>,
        }
        let parsed: Snapshot = serde_json::from_value(value).map_err(|e| invalid(e.to_string()))?;
        if parsed.schema != "droplet.source_bound.internal.v1"
            || parsed.run_id != prefix.run_id
            || parsed.prefix_hash != prefix.input_hash
        {
            return Err(invalid("source packet schema/run/prefix mismatch"));
        }
        let initial_time = read_value(&parsed.items, "initial_time_s")?;
        let input = decode_input(&parsed.items)?;
        if required_input_items(&input, initial_time) != parsed.items {
            return Err(invalid("unrecognized or noncanonical physical input item"));
        }
        Self::bind(
            prefix,
            input,
            initial_time,
            parsed.material_snapshot,
            parsed.evidence,
        )
    }

    pub fn bind(
        prefix: &PrefixBundle,
        input: Input,
        initial_time_s: f64,
        material_snapshot: Digest,
        evidence: BTreeMap<String, InputEvidence>,
    ) -> Result<Self> {
        prefix_admission(prefix)?;
        if !initial_time_s.is_finite()
            || initial_time_s < 0.0
            || initial_time_s > prefix.cutoff_s
            || input.time_origin != prefix.time_origin
        {
            return Err(invalid(
                "initial-state time must precede cutoff in the same explicit time origin",
            ));
        }
        let items = required_input_items(&input, initial_time_s);
        if !items.keys().eq(evidence.keys()) {
            return Err(Refusal::insufficient("missing_physical_evidence", "every physical input and closure needs exactly one source binding; unexpected bindings also refused"));
        }
        for (name, item) in &items {
            let binding = &evidence[name];
            binding
                .locator
                .validate()
                .map_err(|e| invalid(e.to_string()))?;
            if binding.source.trim().is_empty()
                || binding.locator_text.trim().is_empty()
                || Digest::of_json(item) != binding.item_hash
            {
                return Err(invalid(format!("source/item identity mismatch for {name}")));
            }
            if !binding.available_at_s.is_finite()
                || binding.available_at_s > prefix.cutoff_s
                || binding.observed_at_s.is_some_and(|t| {
                    !t.is_finite() || t > binding.available_at_s || t > prefix.cutoff_s
                })
            {
                return Err(invalid(format!(
                    "future/unavailable physical evidence for {name}"
                )));
            }
            if matches!(binding.evidence_class, EvidenceClass::ModelOutput) {
                return Err(Refusal::unsupported(
                    "model_output_as_input",
                    format!(
                        "{name} requires source/measurement evidence, not another model prediction"
                    ),
                ));
            }
            for id in &binding.observation_ids {
                let o = prefix
                    .observations
                    .iter()
                    .find(|o| &o.id == id)
                    .ok_or_else(|| {
                        invalid(format!("{name} depends on non-admitted observation {id}"))
                    })?;
                if o.evidence_class == EvidenceClass::ModelOutput
                    || (o.evidence_class == EvidenceClass::Synthetic)
                        != (binding.evidence_class == EvidenceClass::Synthetic)
                {
                    return Err(invalid(
                        "input observation dependency has incompatible/model-output origin",
                    ));
                }
                if o.available_at_s > binding.available_at_s
                    || o.time_s > binding.observed_at_s.unwrap_or(prefix.cutoff_s)
                {
                    return Err(invalid(
                        "input declared available before its observation dependency",
                    ));
                }
            }
            for key in &binding.condition_keys {
                let c = prefix
                    .known_conditions
                    .get(key)
                    .ok_or_else(|| invalid(format!("missing condition dependency {key}")))?;
                if c.resolution != Resolution::Resolved {
                    return Err(Refusal::insufficient("unresolved_condition", key));
                }
                if c.evidence_class == EvidenceClass::ModelOutput
                    || (c.evidence_class == EvidenceClass::Synthetic)
                        != (binding.evidence_class == EvidenceClass::Synthetic)
                {
                    return Err(invalid(
                        "input condition dependency has incompatible/model-output origin",
                    ));
                }
            }
            if (name.ends_with("/initial_mass") || name == "initial_temperature")
                && binding.observed_at_s != Some(initial_time_s)
            {
                return Err(invalid(format!(
                    "{name} must be an explicit state determination at initial_time_s"
                )));
            }
        }
        for (name, p) in properties(&input) {
            let binding = &evidence[&name];
            if binding.source != p.source || binding.locator_text != p.locator {
                return Err(invalid(format!("property provenance mismatch for {name}")));
            }
            if p.origin == Origin::ExplicitScenarioAssumption {
                return Err(Refusal::unsupported("authored_scenario_requires_candidate", format!("{name}: this binding release admits no new numerical scenario assumption; author it under a separate identity")));
            }
            if (p.origin == Origin::SyntheticFixture)
                != (binding.evidence_class == EvidenceClass::Synthetic)
            {
                return Err(invalid(format!(
                    "synthetic property/evidence mismatch for {name}"
                )));
            }
        }
        if input.transport != kernel::TransportApplicability::RequireQuiescentZeroGravity {
            return Err(Refusal::unsupported("transport_extrapolation_not_admitted", "source-bound FLEX comparison does not admit spherical extrapolation at nonzero gravity/motion"));
        }
        if matches!(input.chemistry, Chemistry::EstablishedReactionSheet { .. }) {
            match &evidence["chemistry_closure"].reviewed_use {
                ReviewedUse::EstablishedReactionSheet { physical_justification, known_at_s, valid_from_s, valid_until_s, .. }
                    if !physical_justification.trim().is_empty() && known_at_s.is_finite() && *known_at_s <= prefix.cutoff_s && valid_from_s.is_finite() && *valid_from_s <= initial_time_s && valid_until_s.is_finite() && *valid_until_s >= initial_time_s => {},
                _ => return Err(Refusal::insufficient("reaction_sheet_not_established", "source-bound scientific admission of the infinitely-fast established-sheet approximation is required; a generic prefix phase marker or diameter alone is insufficient")),
            }
        }
        if prefix.known_controls.iter().any(|c| {
            !c.t_start_s.is_finite()
                || c.t_start_s > initial_time_s
                || c.t_end_s
                    .is_none_or(|t| !t.is_finite() || t > initial_time_s)
        }) {
            return Err(Refusal::unsupported("time_varying_controls", "frozen constant-boundary kernel cannot consume controls intersecting the simulated interval"));
        }
        let snapshot = json!({"schema":"droplet.source_bound.internal.v1","run_id":prefix.run_id,"prefix_hash":prefix.input_hash,"material_snapshot":material_snapshot,"items":items,"evidence":evidence});
        let hash = Digest::of_json(&snapshot);
        Ok(Self {
            input,
            initial_time_s,
            prefix_hash: prefix.input_hash.clone(),
            material_snapshot,
            evidence,
            snapshot,
            hash,
        })
    }

    pub fn input_hash(&self) -> &Digest {
        &self.hash
    }
    pub fn snapshot(&self) -> &Value {
        &self.snapshot
    }
}

fn release_parameters(release: &LoadedRelease) -> Result<kernel::Parameters> {
    let spec = specification().map_err(|e| Refusal::failed("adapter_spec", e.to_string()))?;
    spec.check_release(&release.release)
        .map_err(|e| invalid(e.to_string()))?;
    if release.release.model_name != MODEL_NAME
        || release
            .release
            .construction
            .as_ref()
            .is_none_or(|m| m.forward_hash != forward_hash())
        || !release.posterior.is_empty()
    {
        return Err(Refusal::unsupported("release_binding", "release model/forward identity or posterior is incompatible with the source-bound deterministic adapter"));
    }
    let frozen: Vec<String> = release.params.keys().cloned().collect();
    if registry::tensor_entries(&release.params, &frozen) != release.release.tensors {
        return Err(invalid(
            "stored tensors do not match release manifest entries",
        ));
    }
    kernel::Parameters::from_tensors(
        &kernel::specification().map_err(|e| invalid(e.to_string()))?,
        &release.params,
    )
    .map_err(|e| invalid(e.to_string()))
}

fn requested_times(settings: &ForecastSettings, cutoff: f64) -> Result<&[f64]> {
    settings.validate()?;
    if settings.quantile_levels.is_some() {
        return Err(Refusal::unsupported("predictive_distribution_unavailable", "frozen W0 supplies a deterministic point forecast; no calibrated or assumed distribution exists"));
    }
    if settings.horizon_s.is_some() || settings.n_points.is_some() {
        return Err(Refusal::invalid(
            "explicit_times_required",
            "use times_s alone; no automatic grid or horizon default is introduced",
        ));
    }
    let times = settings.times_s.as_deref().ok_or_else(|| {
        Refusal::invalid(
            "explicit_times_required",
            "explicit future times_s required",
        )
    })?;
    if times.iter().any(|t| *t <= cutoff) {
        return Err(Refusal::invalid(
            "future_times_required",
            "all scored predictions must be strictly after the prefix cutoff",
        ));
    }
    Ok(times)
}

/// Runs only after host scope/partition checks, with no file/network access or parameter estimation.
pub fn forecast(
    scope: &ExecutionScope,
    request: &ForecastRequest,
    release: &LoadedRelease,
    prefix: &PrefixBundle,
    source: &dyn InputSource,
) -> Result<AnalysisArtifact> {
    scope.validate().map_err(|e| invalid(e.to_string()))?;
    for capability in [
        Capability::RunModels,
        Capability::ReadData,
        Capability::ReadSources,
    ] {
        scope
            .require(capability)
            .map_err(|e| invalid(e.to_string()))?;
    }
    request.validate().map_err(|e| invalid(e.to_string()))?;
    if request.release_id.as_str() != release.release.id {
        return Err(invalid("request/release identity mismatch"));
    }
    prefix_admission(prefix)?;
    match &request.input {
        ForecastInput::StoredRun { run_id, cutoff_s }
            if run_id.as_str() == prefix.run_id && cutoff_s.get() == prefix.cutoff_s => {}
        ForecastInput::Prefix { input_hash } if input_hash == &prefix.input_hash => {}
        _ => return Err(invalid("request/prefix identity mismatch")),
    }
    let parameters = release_parameters(release)?;
    let times = requested_times(&request.settings, prefix.cutoff_s)?;
    let provenance = source.provenance(scope, &release.release, prefix)?;
    if let Provenance::Partition(partition) = provenance {
        scope
            .require_partition(partition)
            .map_err(|e| invalid(e.to_string()))?;
    }
    let synthetic = provenance == Provenance::SyntheticFixture;
    let prefix_origins = prefix
        .observations
        .iter()
        .map(|o| o.evidence_class)
        .chain(prefix.contours.iter().map(|c| c.evidence_class))
        .chain(prefix.known_conditions.values().map(|c| c.evidence_class));
    if prefix_origins.into_iter().any(|origin| {
        origin == EvidenceClass::ModelOutput || (origin == EvidenceClass::Synthetic) != synthetic
    }) {
        return Err(invalid(
            "prefix evidence origin conflicts with host provenance or contains model outputs",
        ));
    }
    let packet = match source.resolve(scope, &release.release, prefix) {
        Ok(packet) => packet,
        Err(refusal)
            if matches!(
                refusal.kind,
                ignis_core::expert::RefusalKind::InsufficientInference
                    | ignis_core::expert::RefusalKind::Unsupported
            ) =>
        {
            let a = common::artifact(
                release,
                prefix,
                &request.settings.value(),
                common::refuse(SupportGate::Condition, &refusal.reason),
                Body {
                    quantity: Some(Quantity::LiquidDiameter),
                    diagnostics: json!({"refusal_code":refusal.code,"adapter_forward_hash":forward_hash(),"required_physical_inputs":"explicit initial inventory and liquid temperature; complete material/transport/ambient source packet; no fallback values","uncertainty":"unquantified","no_partial_prediction":true}),
                    ..Body::default()
                },
            );
            validate_artifact(&a, &release.release, prefix)?;
            return Ok(a);
        }
        Err(error) => return Err(error),
    };
    if packet.prefix_hash != prefix.input_hash {
        return Err(invalid(
            "physical input packet belongs to a different prefix",
        ));
    }
    if let ReviewedUse::EstablishedReactionSheet { valid_until_s, .. } =
        &packet.evidence["chemistry_closure"].reviewed_use
    {
        if times.last().is_some_and(|t| t > valid_until_s) {
            return Err(Refusal::unsupported(
                "reaction_sheet_admission_horizon",
                "requested horizon exceeds the source-backed sheet-closure admission interval",
            ));
        }
    }
    if packet
        .evidence
        .values()
        .any(|e| (e.evidence_class == EvidenceClass::Synthetic) != synthetic)
    {
        return Err(invalid(
            "material/evidence origin differs from host provenance",
        ));
    }
    let mut means = Vec::with_capacity(times.len());
    let mut maximum_error: f64 = 0.0;
    let mut mass_residual: f64 = 0.0;
    let mut energy_residual: f64 = 0.0;
    // Each requested endpoint uses the identical frozen integration from the admitted initial state.
    // No interpolation, inferred restart state, or post-cutoff measurement assimilation.
    for &time in times {
        let result = kernel::forward(&packet.input, time - packet.initial_time_s, &parameters);
        let trajectory = match result {
            Ok(t) => t,
            Err(error) => {
                let a = common::artifact(
                    release,
                    prefix,
                    &request.settings.value(),
                    common::refuse(
                        SupportGate::Numerical,
                        format!("frozen physical/numerical closure refused: {error}"),
                    ),
                    Body {
                        quantity: Some(Quantity::LiquidDiameter),
                        diagnostics: json!({"physical_input_hash":packet.hash,"physical_input_snapshot":packet.snapshot,"adapter_forward_hash":forward_hash(),"uncertainty":"unquantified","no_partial_prediction":true}),
                        ..Body::default()
                    },
                );
                validate_artifact(&a, &release.release, prefix)?;
                return Ok(a);
            }
        };
        let state = trajectory
            .states
            .last()
            .ok_or_else(|| Refusal::failed("empty_trajectory", "kernel returned no state"))?;
        let point = kernel::instantaneous(&packet.input, state, &parameters)
            .map_err(|e| Refusal::failed("terminal_state", e.to_string()))?;
        means.push(point.liquid_volume_equivalent_diameter_m);
        maximum_error = maximum_error.max(trajectory.maximum_step_doubling_error_ratio);
        mass_residual = mass_residual.max(trajectory.mass_balance_residual_kg.abs());
        energy_residual = energy_residual.max(trajectory.energy_balance_residual_j.abs());
    }
    let body = Body {
        quantity: Some(Quantity::LiquidDiameter),
        times_s: times.to_vec(),
        mean: means,
        diagnostics: json!({"physical_input_hash":packet.hash,"physical_input_snapshot":packet.snapshot,"material_snapshot":packet.material_snapshot,"adapter_forward_hash":forward_hash(),"frozen_kernel_spec_hash":kernel::specification().map_err(|e|invalid(e.to_string()))?.spec_hash(),"initial_time_s":packet.initial_time_s,"uncertainty":"unquantified; no predictive distribution","prediction_semantics":"deterministic volume-equivalent liquid diameter; mean field is the shared scalar prediction slot","maximum_step_doubling_error_ratio":maximum_error,"maximum_absolute_mass_residual_kg":mass_residual,"maximum_absolute_energy_residual_j":energy_residual,"data_origin":if synthetic {"synthetic_fixture"} else {"source_bound"},"no_parameter_fitting":true}),
        ..Body::default()
    };
    let a = common::artifact(
        release,
        prefix,
        &request.settings.value(),
        common::admitted(),
        body,
    );
    validate_artifact(&a, &release.release, prefix)?;
    // Generic caches keyed only by release/prefix/settings are insufficient: material inputs vary.
    debug_assert_eq!(a.id, content_hash(&a, &["id", "created_at"]));
    Ok(a)
}

fn read_value<T: serde::de::DeserializeOwned>(
    items: &BTreeMap<String, Value>,
    name: &str,
) -> Result<T> {
    serde_json::from_value(
        items
            .get(name)
            .cloned()
            .ok_or_else(|| invalid(format!("missing input {name}")))?,
    )
    .map_err(|e| invalid(format!("{name}: {e}")))
}

fn decode_property(value: Value) -> Result<Supplied> {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Property {
        raw_value: f64,
        raw_unit: String,
        si_value: f64,
        si_unit: String,
        phase: String,
        source: String,
        locator: String,
        origin: String,
        uncertainty: Value,
        reference_temperature_k: f64,
        reference_pressure_pa: f64,
        reference_identity: String,
        temperature_domain_k: [f64; 2],
        pressure_domain_pa: [f64; 2],
    }
    let p: Property = serde_json::from_value(value).map_err(|e| invalid(e.to_string()))?;
    use kernel::Unit::*;
    let si_unit = match p.si_unit.as_str() {
        "Kelvin" => Kelvin,
        "Pascal" => Pascal,
        "Kilogram" => Kilogram,
        "KilogramPerMole" => KilogramPerMole,
        "KilogramPerCubicMetre" => KilogramPerCubicMetre,
        "JoulePerKilogramKelvin" => JoulePerKilogramKelvin,
        "JoulePerKilogram" => JoulePerKilogram,
        "SquareMetrePerSecond" => SquareMetrePerSecond,
        "WattPerMetreKelvin" => WattPerMetreKelvin,
        "PascalSecond" => PascalSecond,
        "MetrePerSecond" => MetrePerSecond,
        "MetrePerSecondSquared" => MetrePerSecondSquared,
        "One" => One,
        _ => return Err(invalid("unsupported property unit")),
    };
    let phase = match p.phase.as_str() {
        "Liquid" => kernel::Phase::Liquid,
        "Gas" => kernel::Phase::Gas,
        "Interface" => kernel::Phase::Interface,
        "Condition" => kernel::Phase::Condition,
        _ => return Err(invalid("unsupported phase")),
    };
    let origin = match p.origin.as_str() {
        "ExternalDetermination" => Origin::ExternalDetermination,
        "SyntheticFixture" => Origin::SyntheticFixture,
        "ExplicitScenarioAssumption" => Origin::ExplicitScenarioAssumption,
        _ => return Err(invalid("unsupported origin")),
    };
    let uncertainty = match p.uncertainty.get("kind").and_then(Value::as_str) {
        Some("unquantified") => kernel::Uncertainty::Unquantified,
        Some("absolute_bound_in_si") => kernel::Uncertainty::AbsoluteBoundInSi(
            p.uncertainty
                .get("value")
                .and_then(Value::as_f64)
                .ok_or_else(|| invalid("uncertainty bound missing"))?,
        ),
        _ => return Err(invalid("unsupported uncertainty status")),
    };
    Ok(Supplied {
        raw_value: p.raw_value,
        raw_unit: p.raw_unit,
        si_value: p.si_value,
        si_unit,
        phase,
        source: p.source,
        locator: p.locator,
        origin,
        uncertainty,
        reference_temperature_k: p.reference_temperature_k,
        reference_pressure_pa: p.reference_pressure_pa,
        reference_identity: p.reference_identity,
        temperature_domain_k: p.temperature_domain_k,
        pressure_domain_pa: p.pressure_domain_pa,
    })
}

fn decode_input(items: &BTreeMap<String, Value>) -> Result<Input> {
    let get = |name: &str| {
        decode_property(
            items
                .get(name)
                .cloned()
                .ok_or_else(|| invalid(format!("missing property {name}")))?,
        )
    };
    let count = items
        .keys()
        .filter(|key| key.starts_with("components/") && key.ends_with("/identity_and_composition"))
        .count();
    let mut components = Vec::with_capacity(count);
    for index in 0..count {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Identity {
            identity: String,
            #[serde(rename = "CHONS_He_Ar")]
            atoms: [u32; 7],
        }
        let identity: Identity = read_value(
            items,
            &format!("components/{index}/identity_and_composition"),
        )?;
        let a = identity.atoms;
        let prop = |name: &str| get(&format!("components/{index}/{name}"));
        components.push(kernel::LiquidComponent {
            identity: identity.identity,
            atoms: crate::physical_core::ElementalComposition {
                carbon: a[0],
                hydrogen: a[1],
                oxygen: a[2],
                nitrogen: a[3],
                sulfur: a[4],
                helium: a[5],
                argon: a[6],
            },
            initial_mass: prop("initial_mass")?,
            molar_mass: prop("molar_mass")?,
            liquid_density: prop("liquid_density")?,
            liquid_heat_capacity: prop("liquid_heat_capacity")?,
            latent_heat_at_reference: prop("latent_heat_at_reference")?,
            saturation_pressure_at_reference: prop("saturation_pressure_at_reference")?,
            complete_combustion_heat: items
                .get(&format!("components/{index}/complete_combustion_heat"))
                .cloned()
                .map(decode_property)
                .transpose()?,
        });
    }
    let activity: Value = read_value(items, "activity_closure")?;
    let text = |value: &Value, key: &str| {
        value
            .get(key)
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| invalid(format!("closure {key} missing")))
    };
    let activity = match activity.get("kind").and_then(Value::as_str) {
        Some("ideal_solution") => kernel::ActivityClosure::IdealSolution {
            source: text(&activity, "source")?,
            applicability: text(&activity, "applicability")?,
        },
        Some("unsupported") => kernel::ActivityClosure::Unsupported {
            reason: text(&activity, "reason")?,
        },
        _ => return Err(invalid("unknown activity closure")),
    };
    let chemistry: Value = read_value(items, "chemistry_closure")?;
    let chemistry = match chemistry.get("kind").and_then(Value::as_str) {
        Some("evaporation_only") => Chemistry::EvaporationOnly,
        Some("established_reaction_sheet") => Chemistry::EstablishedReactionSheet {
            source: text(&chemistry, "source")?,
            applicability: text(&chemistry, "applicability")?,
        },
        _ => return Err(invalid("unknown chemistry closure")),
    };
    let transport: String = read_value(items, "transport_applicability")?;
    let transport = match transport.as_str() {
        "RequireQuiescentZeroGravity" => {
            kernel::TransportApplicability::RequireQuiescentZeroGravity
        }
        "ExplicitSphericalExtrapolation" => {
            kernel::TransportApplicability::ExplicitSphericalExtrapolation
        }
        _ => return Err(invalid("unknown transport applicability")),
    };
    Ok(Input {
        time_origin: read_value(items, "time_origin")?,
        components,
        initial_temperature: get("initial_temperature")?,
        ambient_temperature: get("ambient_temperature")?,
        pressure: get("pressure")?,
        gas_density: get("gas_density")?,
        gas_heat_capacity: get("gas_heat_capacity")?,
        gas_conductivity: get("gas_conductivity")?,
        common_diffusivity: get("common_diffusivity")?,
        gas_viscosity: get("gas_viscosity")?,
        surface_background_molar_mass: get("surface_background_molar_mass")?,
        oxygen_molar_mass: get("oxygen_molar_mass")?,
        ambient_oxygen_mass_fraction: get("ambient_oxygen_mass_fraction")?,
        relative_velocity: [
            get("relative_velocity/0")?,
            get("relative_velocity/1")?,
            get("relative_velocity/2")?,
        ],
        gravity: [get("gravity/0")?, get("gravity/1")?, get("gravity/2")?],
        coordinate_frame: read_value(items, "coordinate_frame")?,
        activity,
        chemistry,
        transport,
    })
}

/// Executes validations and an end-to-end synthetic fixture before any release can be saved.
/// The fixture is not a source for real material inputs or experimental qualification.
pub fn assemble_checked(
    cohort_id: String,
    split_id: String,
    source_manifest_hash: String,
) -> anyhow::Result<LoadedRelease> {
    use ignis_core::contracts::{BuildCheck, BuildCheckKind, CheckStatus, UncertaintyKind};
    anyhow::ensure!(
        !cohort_id.trim().is_empty()
            && !split_id.trim().is_empty()
            && !source_manifest_hash.trim().is_empty(),
        "release population identity required"
    );
    let spec = specification()?;
    spec.validate()?;
    let patch = authored_patch()?;
    patch.validate_against(&spec)?;
    let params = crate::assemble::materialize(&spec, &patch, None)?;
    kernel::Parameters::from_tensors(&kernel::specification()?, &params)?;
    let context = |fixture_status| {
        crate::assemble::ReleaseContext {
        cohort_id:cohort_id.clone(),split_id:split_id.clone(),source_manifest_hash:source_manifest_hash.clone(),
        applicability:json!({"physical_inputs":"complete source-bound host packet required","gravity_and_motion":"quiescent zero-gravity only","qualification":"experimental; not NASA-qualified"}).as_object().unwrap().clone(),
        observation_model:json!({"target":"liquid_volume_equivalent_diameter","distribution":"unavailable; deterministic point only"}).as_object().unwrap().clone(),
        numerics:json!({"kernel_spec":kernel::specification().expect("validated").spec_hash(),"control_source":"stored literal tensors"}).as_object().unwrap().clone(),
        uncertainty_kind:UncertaintyKind::None,forward_hash:forward_hash(),
        checks:vec![BuildCheck{check:BuildCheckKind::StrictForwardAudit,status:CheckStatus::NotRun,detail:Some("No automated whole-route audit is claimed by this constructor; explicit stored-tensor/no-fitting path inspected separately.".into())},BuildCheck{check:BuildCheckKind::FiniteForwardOnFixture,status:fixture_status,detail:Some("Synthetic source packet -> stored tensors -> prospective request -> finite diameter AnalysisArtifact; never NASA evidence.".into())}],
    }
    };
    let mut draft =
        crate::assemble::build_release(&spec, &patch, &params, None, context(CheckStatus::NotRun))?;
    draft.id = content_hash(&draft, &["id", "created_at"]);
    let loaded = LoadedRelease {
        release: draft,
        params: params.clone(),
        posterior: Default::default(),
    };
    let (scope, prefix, input) = synthetic_fixture();
    let source = SyntheticSource {
        snapshot: synthetic_packet(&prefix, input)?.snapshot.clone(),
    };
    let request = synthetic_request(&loaded, &prefix)?;
    let artifact = forecast(&scope, &request, &loaded, &prefix, &source)?;
    anyhow::ensure!(
        artifact.support.admitted
            && artifact.mean.len()
                == request
                    .settings
                    .times_s
                    .as_ref()
                    .expect("fixture grid")
                    .len()
            && artifact.mean.iter().all(|v| v.is_finite() && *v > 0.0),
        "synthetic adapter fixture did not produce finite positive diameter"
    );
    let mut release =
        crate::assemble::build_release(&spec, &patch, &params, None, context(CheckStatus::Passed))?;
    release.id = content_hash(&release, &["id", "created_at"]);
    Ok(LoadedRelease {
        release,
        params,
        posterior: Default::default(),
    })
}

// Deliberately synthetic algebra inputs from the frozen kernel's tests. These are reachable only
// through the explicit release self-check or tests; production input resolution never uses them.
fn synthetic_property(
    value: f64,
    unit: kernel::Unit,
    raw_unit: &str,
    phase: kernel::Phase,
) -> Supplied {
    Supplied {
        raw_value: value,
        raw_unit: raw_unit.into(),
        si_value: value,
        si_unit: unit,
        phase,
        source: "synthetic:adapter-algebra".into(),
        locator: "synthetic fixture declaration".into(),
        origin: Origin::SyntheticFixture,
        uncertainty: kernel::Uncertainty::Unquantified,
        reference_temperature_k: 2.0,
        reference_pressure_pa: 10.0,
        reference_identity: "synthetic-sensible-zero-at-2K".into(),
        temperature_domain_k: [1.0, 1000.0],
        pressure_domain_pa: [1.0, 1000.0],
    }
}

fn synthetic_fixture() -> (ExecutionScope, PrefixBundle, Input) {
    use ignis_core::contracts::{PrincipalId, ScopeId, ScopeRole, WorkspaceId};
    use kernel::{Phase::*, Unit::*};
    let p = synthetic_property;
    let scope = ExecutionScope {
        scope_id: ScopeId::new("synthetic-adapter").unwrap(),
        workspace_id: WorkspaceId::new("synthetic-adapter").unwrap(),
        principal: PrincipalId::new("synthetic-adapter").unwrap(),
        role: ScopeRole::Research,
        partitions: Default::default(),
        capabilities: [
            Capability::ReadSources,
            Capability::ReadData,
            Capability::RunModels,
        ]
        .into_iter()
        .collect(),
        corpus_snapshot: None,
        parent_scope_id: None,
    };
    let mut prefix = PrefixBundle {
        family: Family::Flex,
        run_id: "synthetic-adapter-run".into(),
        session_id: None,
        cutoff_s: 0.0,
        time_origin: "synthetic explicit initial state".into(),
        known_conditions: Default::default(),
        known_controls: vec![],
        observations: vec![],
        contours: vec![],
        phase_established_at_s: None,
        support_evidence: vec![],
        transform_versions: Default::default(),
        input_hash: String::new(),
    };
    prefix.input_hash = content_hash(&prefix, &["input_hash"]);
    let input = Input {
        time_origin: prefix.time_origin.clone(),
        components: vec![kernel::LiquidComponent {
            identity: "synthetic-A".into(),
            atoms: crate::physical_core::ElementalComposition {
                carbon: 1,
                hydrogen: 4,
                oxygen: 0,
                nitrogen: 0,
                sulfur: 0,
                helium: 0,
                argon: 0,
            },
            initial_mass: p(1.0, Kilogram, "kg", Liquid),
            molar_mass: p(1.0, KilogramPerMole, "kg/mol", Liquid),
            liquid_density: p(10.0, KilogramPerCubicMetre, "kg/m3", Liquid),
            liquid_heat_capacity: p(1.0, JoulePerKilogramKelvin, "J/(kg K)", Liquid),
            latent_heat_at_reference: p(1.0, JoulePerKilogram, "J/kg", Interface),
            saturation_pressure_at_reference: p(2.0, Pascal, "Pa", Interface),
            complete_combustion_heat: None,
        }],
        initial_temperature: p(2.0, Kelvin, "K", Condition),
        ambient_temperature: p(2.25, Kelvin, "K", Condition),
        pressure: p(10.0, Pascal, "Pa", Condition),
        gas_density: p(1.0, KilogramPerCubicMetre, "kg/m3", Gas),
        gas_heat_capacity: p(1.0, JoulePerKilogramKelvin, "J/(kg K)", Gas),
        gas_conductivity: p(0.01, WattPerMetreKelvin, "W/(m K)", Gas),
        common_diffusivity: p(0.01, SquareMetrePerSecond, "m2/s", Gas),
        gas_viscosity: p(0.01, PascalSecond, "Pa s", Gas),
        surface_background_molar_mass: p(1.0, KilogramPerMole, "kg/mol", Gas),
        oxygen_molar_mass: p(1.0, KilogramPerMole, "kg/mol", Gas),
        ambient_oxygen_mass_fraction: p(0.25, One, "1", Gas),
        relative_velocity: std::array::from_fn(|_| p(0.0, MetrePerSecond, "m/s", Condition)),
        gravity: std::array::from_fn(|_| p(0.0, MetrePerSecondSquared, "m/s2", Condition)),
        coordinate_frame: "synthetic-cartesian".into(),
        activity: kernel::ActivityClosure::IdealSolution {
            source: "synthetic ideal solution declaration".into(),
            applicability: "algebra only".into(),
        },
        chemistry: Chemistry::EvaporationOnly,
        transport: kernel::TransportApplicability::RequireQuiescentZeroGravity,
    };
    (scope, prefix, input)
}

fn synthetic_packet(prefix: &PrefixBundle, input: Input) -> Result<SourceBoundInputs> {
    let evidence = required_input_items(&input, 0.0)
        .into_iter()
        .map(|(key, item)| {
            let binding = InputEvidence {
                source: item
                    .get("source")
                    .and_then(Value::as_str)
                    .unwrap_or("synthetic:adapter-algebra")
                    .into(),
                locator_text: item
                    .get("locator")
                    .and_then(Value::as_str)
                    .unwrap_or("synthetic fixture declaration")
                    .into(),
                locator: SourceLocator {
                    asset_sha256: Digest::of_bytes(
                        b"synthetic fixture declarations, not NASA source bytes",
                    )
                    .hex()
                    .into(),
                    path: Some("synthetic-fixture".into()),
                    ..SourceLocator::default()
                },
                available_at_s: 0.0,
                observed_at_s: Some(0.0),
                evidence_class: EvidenceClass::Synthetic,
                item_hash: Digest::of_json(&item),
                observation_ids: vec![],
                condition_keys: vec![],
                reviewed_use: ReviewedUse::SourceOnly,
            };
            (key, binding)
        })
        .collect();
    SourceBoundInputs::bind(
        prefix,
        input,
        0.0,
        Digest::of_bytes(b"synthetic material snapshot"),
        evidence,
    )
}

struct SyntheticSource {
    snapshot: Value,
}
impl InputSource for SyntheticSource {
    fn provenance(
        &self,
        _: &ExecutionScope,
        _: &ModelRelease,
        _: &PrefixBundle,
    ) -> Result<Provenance> {
        Ok(Provenance::SyntheticFixture)
    }
    fn resolve(
        &self,
        _: &ExecutionScope,
        _: &ModelRelease,
        prefix: &PrefixBundle,
    ) -> Result<SourceBoundInputs> {
        SourceBoundInputs::from_snapshot(prefix, self.snapshot.clone())
    }
}

fn synthetic_request(
    release: &LoadedRelease,
    prefix: &PrefixBundle,
) -> anyhow::Result<ForecastRequest> {
    Ok(ForecastRequest {
        schema_version: ignis_core::contracts::WspSchema::V1,
        release_id: ignis_core::contracts::ReleaseId::new(&release.release.id)?,
        input: ForecastInput::Prefix {
            input_hash: prefix.input_hash.clone(),
        },
        settings: ForecastSettings {
            times_s: Some(vec![0.001, 0.002]),
            ..ForecastSettings::default()
        },
        client_request_id: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    fn release() -> LoadedRelease {
        assemble_checked(
            "synthetic-cohort".into(),
            "synthetic-split".into(),
            "synthetic-source-manifest".into(),
        )
        .unwrap()
    }

    #[test]
    fn property_droplet_release_executes_stored_release_packet_and_artifact_without_spread() {
        let release = release();
        let (scope, prefix, input) = synthetic_fixture();
        let packet = synthetic_packet(&prefix, input).unwrap();
        let roundtrip = SourceBoundInputs::from_snapshot(&prefix, packet.snapshot.clone()).unwrap();
        assert_eq!(packet.hash, roundtrip.hash);
        let source = SyntheticSource {
            snapshot: packet.snapshot.clone(),
        };
        let a = forecast(
            &scope,
            &synthetic_request(&release, &prefix).unwrap(),
            &release,
            &prefix,
            &source,
        )
        .unwrap();
        assert!(a.support.admitted);
        assert_eq!(a.quantity, Some(Quantity::LiquidDiameter));
        assert_eq!(a.mean.len(), 2);
        assert!(a.mean[1] < a.mean[0]);
        assert!(a.quantiles.is_empty() && a.quantile_levels.is_empty());
        assert_eq!(a.diagnostics["physical_input_hash"], json!(packet.hash));
        assert_eq!(
            release
                .release
                .construction
                .unwrap()
                .checks
                .last()
                .unwrap()
                .status,
            ignis_core::contracts::CheckStatus::Passed
        );
    }

    #[test]
    fn property_droplet_release_refuses_future_missing_changed_and_noncausal_inputs() {
        let (_, prefix, input) = synthetic_fixture();
        let packet = synthetic_packet(&prefix, input).unwrap();
        let mut future = packet.snapshot.clone();
        future["evidence"]["initial_temperature"]["available_at_s"] = json!(1.0);
        assert!(SourceBoundInputs::from_snapshot(&prefix, future).is_err());
        let mut changed = packet.snapshot.clone();
        changed["items"]["initial_temperature"]["si_value"] = json!(2.1);
        assert!(SourceBoundInputs::from_snapshot(&prefix, changed).is_err());
        let mut missing = packet.snapshot.clone();
        missing["evidence"]
            .as_object_mut()
            .unwrap()
            .remove("gas_conductivity");
        assert!(SourceBoundInputs::from_snapshot(&prefix, missing).is_err());
        let mut dependency = packet.snapshot.clone();
        dependency["evidence"]["initial_temperature"]["observation_ids"] = json!(["not-admitted"]);
        assert!(SourceBoundInputs::from_snapshot(&prefix, dependency).is_err());
        let mut no_state_time = packet.snapshot.clone();
        no_state_time["evidence"]["initial_temperature"]["observed_at_s"] = Value::Null;
        assert!(SourceBoundInputs::from_snapshot(&prefix, no_state_time).is_err());
    }

    #[test]
    fn property_droplet_release_scope_denies_partition_before_material_read() {
        struct Denied {
            resolved: AtomicBool,
        }
        impl InputSource for Denied {
            fn provenance(
                &self,
                _: &ExecutionScope,
                _: &ModelRelease,
                _: &PrefixBundle,
            ) -> Result<Provenance> {
                Ok(Provenance::Partition(
                    ignis_core::contracts::Partition::Train,
                ))
            }
            fn resolve(
                &self,
                _: &ExecutionScope,
                _: &ModelRelease,
                _: &PrefixBundle,
            ) -> Result<SourceBoundInputs> {
                self.resolved.store(true, Ordering::SeqCst);
                Err(Refusal::insufficient("unused", "must not resolve"))
            }
        }
        let release = release();
        let (scope, prefix, _) = synthetic_fixture();
        let source = Denied {
            resolved: AtomicBool::new(false),
        };
        assert!(forecast(
            &scope,
            &synthetic_request(&release, &prefix).unwrap(),
            &release,
            &prefix,
            &source
        )
        .is_err());
        assert!(!source.resolved.load(Ordering::SeqCst));
    }

    #[test]
    fn property_droplet_release_missing_packet_is_countable_scientific_refusal() {
        struct Missing;
        impl InputSource for Missing {
            fn provenance(
                &self,
                _: &ExecutionScope,
                _: &ModelRelease,
                _: &PrefixBundle,
            ) -> Result<Provenance> {
                Ok(Provenance::SyntheticFixture)
            }
            fn resolve(
                &self,
                _: &ExecutionScope,
                _: &ModelRelease,
                _: &PrefixBundle,
            ) -> Result<SourceBoundInputs> {
                Err(Refusal::insufficient(
                    "missing_material_packet",
                    "source-admitted gas transport and liquid state missing",
                ))
            }
        }
        let release = release();
        let (scope, prefix, _) = synthetic_fixture();
        let a = forecast(
            &scope,
            &synthetic_request(&release, &prefix).unwrap(),
            &release,
            &prefix,
            &Missing,
        )
        .unwrap();
        assert!(!a.support.admitted && a.mean.is_empty() && a.times_s.is_empty());
        assert_eq!(a.diagnostics["refusal_code"], "missing_material_packet");
    }

    #[test]
    fn property_droplet_release_rejects_tensor_tamper_and_requested_distribution() {
        let mut release = release();
        let (scope, prefix, input) = synthetic_fixture();
        let source = SyntheticSource {
            snapshot: synthetic_packet(&prefix, input).unwrap().snapshot,
        };
        let mut request = synthetic_request(&release, &prefix).unwrap();
        request.settings.quantile_levels = Some(vec![0.5]);
        assert_eq!(
            forecast(&scope, &request, &release, &prefix, &source)
                .unwrap_err()
                .code,
            "predictive_distribution_unavailable"
        );
        request.settings.quantile_levels = None;
        release.params.get_mut("step_seconds").unwrap().data[0] = 0.002;
        assert!(forecast(&scope, &request, &release, &prefix, &source).is_err());
    }

    #[test]
    fn property_droplet_release_generic_activity_marker_does_not_prove_reaction_sheet() {
        let (_, mut prefix, mut input) = synthetic_fixture();
        prefix.phase_established_at_s = Some(0.0);
        prefix.input_hash = content_hash(&prefix, &["input_hash"]);
        input.chemistry = Chemistry::EstablishedReactionSheet {
            source: "synthetic declaration".into(),
            applicability: "test missing admission".into(),
        };
        assert_eq!(
            synthetic_packet(&prefix, input).unwrap_err().code,
            "reaction_sheet_not_established"
        );
    }

    #[test]
    fn property_droplet_release_stale_prefix_and_release_identities_fail_before_use() {
        let mut release = release();
        let (scope, mut prefix, input) = synthetic_fixture();
        let source = SyntheticSource {
            snapshot: synthetic_packet(&prefix, input).unwrap().snapshot,
        };
        let request = synthetic_request(&release, &prefix).unwrap();
        prefix.time_origin = "mutated after prefix hash".into();
        assert!(forecast(&scope, &request, &release, &prefix, &source).is_err());
        let (_, prefix, _) = synthetic_fixture();
        release.release.construction.as_mut().unwrap().forward_hash =
            Digest::of_bytes(b"wrong forward");
        assert!(forecast(&scope, &request, &release, &prefix, &source).is_err());
        release.release.construction.as_mut().unwrap().forward_hash = forward_hash();
        release.release.construction.as_mut().unwrap().spec_hash = Digest::of_bytes(b"wrong spec");
        assert!(forecast(&scope, &request, &release, &prefix, &source).is_err());
    }

    #[test]
    fn property_droplet_release_invalid_packet_is_not_missing_and_synthetic_is_not_real() {
        let release = release();
        let (mut scope, prefix, input) = synthetic_fixture();
        let request = synthetic_request(&release, &prefix).unwrap();
        let bad = SyntheticSource {
            snapshot: json!({"unexpected":"not a physical packet"}),
        };
        let error = forecast(&scope, &request, &release, &prefix, &bad).unwrap_err();
        assert_eq!(error.kind, ignis_core::expert::RefusalKind::InvalidRequest);
        struct WrongOrigin {
            snapshot: Value,
        }
        impl InputSource for WrongOrigin {
            fn provenance(
                &self,
                _: &ExecutionScope,
                _: &ModelRelease,
                _: &PrefixBundle,
            ) -> Result<Provenance> {
                Ok(Provenance::Partition(
                    ignis_core::contracts::Partition::Train,
                ))
            }
            fn resolve(
                &self,
                _: &ExecutionScope,
                _: &ModelRelease,
                prefix: &PrefixBundle,
            ) -> Result<SourceBoundInputs> {
                SourceBoundInputs::from_snapshot(prefix, self.snapshot.clone())
            }
        }
        scope
            .partitions
            .insert(ignis_core::contracts::Partition::Train);
        let wrong = WrongOrigin {
            snapshot: synthetic_packet(&prefix, input).unwrap().snapshot,
        };
        assert_eq!(
            forecast(&scope, &request, &release, &prefix, &wrong)
                .unwrap_err()
                .kind,
            ignis_core::expert::RefusalKind::InvalidRequest
        );
    }

    #[test]
    fn property_droplet_release_rejects_prefix_origin_taint_before_resolving_material() {
        struct RealSource {
            resolved: AtomicBool,
        }
        impl InputSource for RealSource {
            fn provenance(
                &self,
                _: &ExecutionScope,
                _: &ModelRelease,
                _: &PrefixBundle,
            ) -> Result<Provenance> {
                Ok(Provenance::Partition(
                    ignis_core::contracts::Partition::Train,
                ))
            }
            fn resolve(
                &self,
                _: &ExecutionScope,
                _: &ModelRelease,
                _: &PrefixBundle,
            ) -> Result<SourceBoundInputs> {
                self.resolved.store(true, Ordering::SeqCst);
                Err(Refusal::insufficient("unused", "must not resolve"))
            }
        }
        let release = release();
        let (mut scope, mut prefix, input) = synthetic_fixture();
        scope
            .partitions
            .insert(ignis_core::contracts::Partition::Train);
        prefix.known_conditions.insert(
            "source_temperature".into(),
            ignis_core::contracts::Measured {
                raw_value: Some(ignis_core::contracts::RawValue::Number(2.0)),
                raw_unit: Some("K".into()),
                value_si: Some(2.0),
                unit_si: Some("K".into()),
                uncertainty_si: None,
                uncertainty_reason: None,
                evidence_class: EvidenceClass::Synthetic,
                locator: None,
                derivation: None,
                resolution: Resolution::Resolved,
                flags: vec![],
            },
        );
        prefix.input_hash = content_hash(&prefix, &["input_hash"]);
        let source = RealSource {
            resolved: AtomicBool::new(false),
        };
        let request = synthetic_request(&release, &prefix).unwrap();
        assert!(forecast(&scope, &request, &release, &prefix, &source).is_err());
        assert!(!source.resolved.load(Ordering::SeqCst));
        let packet = synthetic_packet(&prefix, input).unwrap();
        prefix
            .known_conditions
            .get_mut("source_temperature")
            .unwrap()
            .evidence_class = EvidenceClass::ModelOutput;
        prefix.input_hash = content_hash(&prefix, &["input_hash"]);
        let mut snapshot = packet.snapshot;
        snapshot["prefix_hash"] = json!(prefix.input_hash);
        snapshot["evidence"]["initial_temperature"]["condition_keys"] =
            json!(["source_temperature"]);
        assert!(SourceBoundInputs::from_snapshot(&prefix, snapshot).is_err());
    }
}
