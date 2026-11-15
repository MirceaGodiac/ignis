//! The hand-written (literal WSP) releases a deployment serves, as one [`ModelService`].
//!
//! *Which* releases are served is data: `served.json` in the served directory (default
//! `configs/models/served`) lists each release with what the deployment states about it (the
//! [`ModelCard`] fields: investigation, what it forecasts, inputs, zero-shot or data-informed,
//! reported development scores with their source, limits). The release bytes sit beside it in the
//! content-addressed layout `models/<hex>/` and are never edited. Adding a release is a list entry
//! and its directory; a rebuild is needed only when it runs on a forward this build lacks.
//!
//! [`ServedModels::open`] fails closed: every listed release must load through the registry
//! checks (tensor digests, construction), carry the id that is the content hash of its record,
//! name a literal forward compiled into `ignis-wsp` whose identity (`forward_hash`) equals the one
//! in its manifest, and agree with its list entry on model name and family. A deployment with a
//! broken list serves no models (the host reports why) instead of serving some.
//!
//! Forecasts run the strict forward of `ignis-wsp` through [`ignis_wsp::WspModelService`]: no
//! fitting, sampling or search at request time; scientific refusals come back as artifacts with
//! their reasons. Inputs are a stored run (through the host's [`PrefixSource`], which checks the
//! run's partition against the scope before anything is read) or a typed scenario admitted with
//! [`ModelService::admit_scenario`]: conditions a person typed, admitted as a synthetic prefix
//! (`evidence_class = synthetic`), never a NASA run.

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use anyhow::{bail, ensure, Context};
use serde::{Deserialize, Serialize};

use ignis_core::contracts::{AnalysisArtifact, Finite};
use ignis_core::contracts::{
    Authorship, Capability, ControlStage, ErrorCode, ErrorEnvelope, EvidenceClass, ExecutionScope,
    Family, ForecastInput, ForecastRequest, Measured, ModelCard, ModelInput, ModelInputKind,
    ModelRelease, PrefixBundle, RawValue, ReleaseId, ReportedScore, Resolution, RetryPolicy,
    ScenarioAdmitted, ScenarioRequest, ScenarioValue, WspSchema,
};
use ignis_core::hashing::{content_hash, hex_part};
use ignis_core::prefix::ProspectivePolicy;
use ignis_core::registry;
use ignis_core::services::{ModelService, ServiceResult};
use ignis_wsp::{PrefixSource, Provenance, WspModelService};

/// Wire tag of `served.json`.
pub const SERVED_SCHEMA: &str = "ignis.served-models/1";
/// Default served directory, relative to the repository (image) root.
pub const DEFAULT_DIR: &str = "configs/models/served";
/// The list file inside the served directory.
pub const LIST_FILE: &str = "served.json";

fn envelope(code: ErrorCode, message: impl Into<String>) -> ErrorEnvelope {
    ErrorEnvelope::new(code, message, RetryPolicy::Never)
}

// ---------------------------------------------------------------- the list

/// One release of `served.json`: the [`ModelCard`] fields the record does not carry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServedEntry {
    pub release_id: ReleaseId,
    pub model_name: String,
    pub family: Family,
    pub investigation: String,
    pub candidate: String,
    pub forecasts: String,
    pub authorship: Authorship,
    pub authorship_note: String,
    pub inputs: Vec<ModelInput>,
    pub stored_runs: bool,
    pub inputs_note: String,
    pub scores: Vec<ReportedScore>,
    pub limits: Vec<String>,
    pub cite: String,
    #[serde(default)]
    pub card_doc: Option<String>,
    /// For a generic law release: the declared targets that are the model (others are refused,
    /// and the first is the default). Absent = every declared target.
    #[serde(default)]
    pub targets: Option<Vec<String>>,
    /// Why the other declared targets are not served (returned with their refusal).
    #[serde(default)]
    pub targets_note: Option<String>,
    /// Refuse a scenario whose fuel is a blend (`fuel_species` naming two species with `+`, or a
    /// `fuel_blend_x` condition): the release carries no blend law.
    #[serde(default)]
    pub refuse_blends: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServedList {
    pub schema: String,
    #[serde(default)]
    pub note: Option<String>,
    pub releases: Vec<ServedEntry>,
}

impl ServedList {
    pub fn read(path: &Path) -> anyhow::Result<ServedList> {
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("served list {}", path.display()))?;
        let list: ServedList = serde_json::from_str(&text)
            .with_context(|| format!("served list {}", path.display()))?;
        ensure!(
            list.schema == SERVED_SCHEMA,
            "served list {}: schema {:?}, expected {SERVED_SCHEMA:?}",
            path.display(),
            list.schema
        );
        let mut seen = std::collections::BTreeSet::new();
        for e in &list.releases {
            ensure!(
                seen.insert(e.release_id.as_str().to_string()),
                "release {} is listed twice",
                e.release_id
            );
            ensure!(
                !e.inputs.is_empty() && !e.limits.is_empty() && !e.cite.trim().is_empty(),
                "release {}: inputs, limits and cite are required",
                e.release_id
            );
        }
        Ok(list)
    }
}

/// Every identity check a served release must pass, on its loaded record. Returns the record.
///
/// * the registry load (tensor digests against the manifest, construction rules);
/// * the id is the content hash of the record without `id` and `created_at` (how
///   `registry::save_release` names a release), so the bytes are the release the id names;
/// * the model is a literal forward of this build and its forward identity equals the manifest's
///   `forward_hash` (a changed forward source would be a different release);
/// * the record agrees with its list entry on model name and family.
pub fn verify_release(root: &Path, entry: &ServedEntry) -> anyhow::Result<ModelRelease> {
    let id = entry.release_id.as_str();
    ensure!(
        ignis_core::hashing::is_digest(id),
        "release id {id} is not a digest"
    );
    let loaded = registry::load(root, id).with_context(|| format!("release {id}"))?;
    let r = loaded.release;
    let recomputed = content_hash(&r, &["id", "created_at"]);
    ensure!(
        recomputed == r.id,
        "release {id}: the record hashes to {recomputed}, not to its id"
    );
    ensure!(
        hex_part(&r.id) == hex_part(id),
        "release directory {id} holds {}",
        r.id
    );
    ensure!(
        r.model_name == entry.model_name && r.family == entry.family,
        "release {id} is {} ({}), the list says {} ({})",
        r.model_name,
        r.family.as_str(),
        entry.model_name,
        entry.family.as_str()
    );
    ensure!(
        ignis_wsp::service::runnable(&r),
        "release {id} ({}) is not a literal release this build can run",
        r.model_name
    );
    let manifest = r
        .construction
        .as_ref()
        .context("no construction manifest")?;
    let forward = ignis_wsp::audit::forward_hash_of(&r.model_name);
    ensure!(
        manifest.forward_hash == forward,
        "release {id}: forward identity of {} in this build is {forward}, the release was built \
         with {}",
        r.model_name,
        manifest.forward_hash
    );
    Ok(r)
}

// ---------------------------------------------------------------- typed scenarios

/// Admitted typed scenarios by `input_hash`, shared by the service and its prefix source.
#[derive(Default, Clone)]
struct Scenarios(Arc<Mutex<HashMap<String, PrefixBundle>>>);

impl Scenarios {
    fn get(&self, hash: &str) -> Option<PrefixBundle> {
        self.0
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .get(hash)
            .cloned()
    }
    fn put(&self, p: PrefixBundle) {
        self.0
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .insert(p.input_hash.clone(), p);
    }
}

/// Stored runs through the host's source; `ForecastInput::Prefix` through the admitted scenarios.
pub struct ServedPrefixes<S> {
    stored: S,
    scenarios: Scenarios,
}

impl<S: PrefixSource> PrefixSource for ServedPrefixes<S> {
    fn provenance(
        &self,
        scope: &ExecutionScope,
        release: &ModelRelease,
        input: &ForecastInput,
    ) -> ServiceResult<Provenance> {
        match input {
            ForecastInput::Prefix { input_hash } => match self.scenarios.get(input_hash) {
                Some(p) if p.family == release.family => Ok(Provenance::SyntheticFixture),
                Some(_) => Err(envelope(
                    ErrorCode::InvalidRequest,
                    format!("scenario {input_hash} was admitted for another family"),
                )),
                None => Err(envelope(
                    ErrorCode::NotFound,
                    format!(
                        "no admitted scenario {input_hash}: admit one with POST \
                         /api/v1/models/{{id}}/scenarios (typed by a person), or forecast a stored run"
                    ),
                )),
            },
            ForecastInput::StoredRun { .. } => self.stored.provenance(scope, release, input),
        }
    }

    fn prefix(
        &self,
        scope: &ExecutionScope,
        release: &ModelRelease,
        policy: &ProspectivePolicy,
        input: &ForecastInput,
    ) -> ServiceResult<PrefixBundle> {
        match input {
            ForecastInput::Prefix { input_hash } => self
                .scenarios
                .get(input_hash)
                .ok_or_else(|| envelope(ErrorCode::NotFound, "scenario not admitted")),
            ForecastInput::StoredRun { .. } => self.stored.prefix(scope, release, policy, input),
        }
    }
}

// ---------------------------------------------------------------- the service

/// The served hand-written releases. `S` builds prefixes of stored runs (in production
/// `ignis_wsp::ViewSourcePrefixes` over the data lane's store).
pub struct ServedModels<S> {
    inner: WspModelService<ServedPrefixes<S>>,
    /// Releases of the generic law forward (no expert, so not runnable through `inner`), loaded.
    law: BTreeMap<String, Arc<registry::LoadedRelease>>,
    /// Served targets of law releases that restrict them (release id -> targets).
    targets: BTreeMap<String, (Vec<String>, Option<String>)>,
    /// Releases that carry no blend law (blend scenarios are refused).
    no_blends: std::collections::BTreeSet<String>,
    records: Vec<ModelRelease>,
    cards: Vec<ModelCard>,
    policies: BTreeMap<String, ProspectivePolicy>,
    scenarios: Scenarios,
    dir: PathBuf,
}

impl<S: PrefixSource> ServedModels<S> {
    /// Reads `<dir>/served.json` and verifies every listed release in `<dir>/models/` (see
    /// [`verify_release`]). Any failure refuses the whole list.
    pub fn open(dir: impl Into<PathBuf>, stored: S) -> anyhow::Result<Self> {
        let dir = dir.into();
        let list = ServedList::read(&dir.join(LIST_FILE))?;
        ensure!(
            !list.releases.is_empty(),
            "served list {} names no release",
            dir.display()
        );
        let mut records = Vec::new();
        let mut cards = Vec::new();
        for e in &list.releases {
            let r = verify_release(&dir, e)?;
            cards.push(ModelCard {
                schema_version: WspSchema::default(),
                release_id: e.release_id.clone(),
                model_name: r.model_name.clone(),
                family: r.family,
                investigation: e.investigation.clone(),
                candidate: e.candidate.clone(),
                forecasts: e.forecasts.clone(),
                authorship: e.authorship,
                authorship_note: e.authorship_note.clone(),
                trained_on: r.trained_on,
                state: r.state,
                inputs: e.inputs.clone(),
                stored_runs: e.stored_runs,
                inputs_note: e.inputs_note.clone(),
                scores: e.scores.clone(),
                limits: e.limits.clone(),
                cite: e.cite.clone(),
                card_doc: e.card_doc.clone(),
            });
            records.push(r);
        }
        let mut policies: BTreeMap<String, ProspectivePolicy> = ignis_wsp::experts()
            .iter()
            .map(|x| (x.model_name().to_string(), x.policy().clone()))
            .collect();
        // Generic law releases have no expert: their prospective conditions are the declared
        // features, nothing else.
        let mut law = BTreeMap::new();
        for r in &records {
            if let Some(i) = law_index(&r.model_name) {
                let d = ignis_wsp::law::decl(i);
                let names: Vec<&str> = d.features.iter().map(|f| f.name.as_str()).collect();
                policies.insert(
                    r.model_name.clone(),
                    ProspectivePolicy::new(r.family, &names, &[]),
                );
                law.insert(
                    r.id.clone(),
                    Arc::new(registry::load(&dir, &r.id).with_context(|| r.id.clone())?),
                );
            }
        }
        let scenarios = Scenarios::default();
        let inner = WspModelService::new(
            dir.clone(),
            ServedPrefixes {
                stored,
                scenarios: scenarios.clone(),
            },
        );
        let mut targets = BTreeMap::new();
        for e in &list.releases {
            if let Some(t) = &e.targets {
                let i = law_index(&e.model_name).with_context(|| {
                    format!(
                        "{}: targets are listed for a release that is not a law release",
                        e.release_id
                    )
                })?;
                let d = ignis_wsp::law::decl(i);
                for name in t {
                    ensure!(
                        d.target(name).is_some(),
                        "{}: target {name} is not declared by {}",
                        e.release_id,
                        e.model_name
                    );
                }
                ensure!(!t.is_empty(), "{}: empty target list", e.release_id);
                targets.insert(
                    e.release_id.as_str().to_string(),
                    (t.clone(), e.targets_note.clone()),
                );
            }
        }
        let no_blends = list
            .releases
            .iter()
            .filter(|e| e.refuse_blends)
            .map(|e| e.release_id.as_str().to_string())
            .collect();
        Ok(ServedModels {
            inner,
            law,
            targets,
            no_blends,
            records,
            cards,
            policies,
            scenarios,
            dir,
        })
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// How many releases are served.
    pub fn count(&self) -> usize {
        self.records.len()
    }

    fn served(&self, id: &ReleaseId) -> ServiceResult<&ModelRelease> {
        self.records
            .iter()
            .find(|r| hex_part(&r.id) == hex_part(id.as_str()) && r.id == id.as_str())
            .ok_or_else(|| {
                // A model name is the usual mistake: say which id it has, and list what is served,
                // so the caller retries with an id instead of concluding nothing is served.
                let named = self.records.iter().find(|r| r.model_name == id.as_str());
                let served: Vec<String> = self
                    .records
                    .iter()
                    .map(|r| format!("{} = {}", r.model_name, r.id))
                    .collect();
                let message = match named {
                    Some(r) => format!(
                        "{id} is a model name, not a release_id; it is served as release_id {}: call again with that id",
                        r.id
                    ),
                    None => format!(
                        "release {id} is not served by this deployment; served release_ids: {}",
                        served.join("; ")
                    ),
                };
                envelope(ErrorCode::NotFound, message)
            })
    }

    /// One paragraph for the model tools' descriptions: what is served and what each needs.
    pub fn served_note(&self) -> String {
        let mut s = String::from(
            "Served hand-written releases (literal weights, not trained, experimental; outputs \
             are model_output):",
        );
        for c in &self.cards {
            let needs: Vec<&str> = c
                .inputs
                .iter()
                .filter(|i| i.required)
                .map(|i| i.name.as_str())
                .collect();
            let warn = c
                .limits
                .iter()
                .find(|l| l.starts_with("Do not use"))
                .map(|l| format!(" {l}"))
                .unwrap_or_default();
            s.push_str(&format!(
                " {} {} `{}` ({}, {}; needs {}; {}).{warn}",
                c.model_name,
                c.candidate,
                c.release_id,
                c.investigation,
                match c.authorship {
                    Authorship::ZeroShot => "zero-shot",
                    Authorship::DataInformed => "data-informed",
                },
                needs.join(", "),
                if c.stored_runs {
                    "stored runs work"
                } else {
                    "stored runs are refused: only a scenario a person admitted in the forecast lab"
                }
            ));
        }
        s
    }
}

/// Index of a generic law slug by its model name (`<slug>.wsp`).
pub fn law_index(model_name: &str) -> Option<usize> {
    ignis_wsp::law_slugs::SLUGS
        .iter()
        .position(|s| s.model_name == model_name)
}

fn measured_number(v: f64, unit: &str) -> Measured {
    Measured {
        raw_value: Some(RawValue::Number(v)),
        raw_unit: Some(unit.into()),
        value_si: Some(v),
        unit_si: Some(unit.into()),
        uncertainty_si: None,
        uncertainty_reason: None,
        evidence_class: EvidenceClass::Synthetic,
        locator: None,
        derivation: None,
        resolution: Resolution::Resolved,
        flags: vec![],
    }
}

/// Builds the synthetic prefix of a typed scenario for `release`. Every value is marked
/// `synthetic`; nothing is filled in that the person did not type.
pub fn scenario_prefix(
    release: &ModelRelease,
    card: &ModelCard,
    policy: &ProspectivePolicy,
    req: &ScenarioRequest,
) -> anyhow::Result<PrefixBundle> {
    let model = ignis_wsp::models::by_name(&release.model_name)
        .with_context(|| format!("no literal model {}", release.model_name))?;
    // The fixture of the model is used only for its layout (time origin, the observed quantity,
    // observation and control templates); none of its values reach the scenario.
    let template = model.fixture_prefix();
    let unit_of = |name: &str, kind: ModelInputKind| {
        card.inputs
            .iter()
            .find(|i| i.name == name && i.kind == kind)
            .map(|i| i.unit_si.clone())
    };
    let mut known = BTreeMap::new();
    for (k, v) in &req.conditions {
        if !policy.allowed_conditions.contains(k) || policy.forbidden_conditions.contains(k) {
            bail!(
                "{} does not read condition {k:?}; it reads {:?}",
                release.model_name,
                policy.allowed_conditions
            );
        }
        let unit = unit_of(k, ModelInputKind::Condition)
            .or_else(|| {
                template
                    .known_conditions
                    .get(k)
                    .and_then(|m| m.unit_si.clone())
            })
            .with_context(|| format!("no unit is declared for condition {k:?}"))?;
        let m = match v {
            ScenarioValue::Number(x) => measured_number(x.get(), &unit),
            ScenarioValue::Text(t) => {
                let mut m = measured_number(0.0, &unit);
                m.raw_value = Some(RawValue::Text(t.trim().to_string()));
                m.value_si = None;
                m
            }
        };
        known.insert(k.clone(), m);
    }
    let mut controls = Vec::new();
    for c in &req.controls {
        let unit = unit_of(&c.control, ModelInputKind::Control)
            .or_else(|| {
                template
                    .known_controls
                    .iter()
                    .find(|s| s.control == c.control)
                    .and_then(|s| s.value.as_ref().and_then(|v| v.unit_si.clone()))
            })
            .unwrap_or_else(|| "1".into());
        controls.push(ControlStage {
            t_start_s: c.t_start_s.get(),
            t_end_s: c.t_end_s.map(Finite::get),
            control: c.control.clone(),
            value: c.value_si.map(|v| measured_number(v.get(), &unit)),
            operator_action: false,
        });
    }
    let mut observations = Vec::new();
    if !req.observations.is_empty() {
        let Some(tpl) = template.observations.first() else {
            bail!(
                "{} reads no observations; give conditions only",
                release.model_name
            );
        };
        for (i, p) in req.observations.iter().enumerate() {
            let mut o = tpl.clone();
            o.id = format!("typed-scenario:{i}");
            o.run_id = "typed-scenario".into();
            o.time_s = p.time_s.get();
            o.available_at_s = p.time_s.get();
            o.value_si = Some(p.value_si.get());
            o.raw_value = Some(RawValue::Number(p.value_si.get()));
            o.raw_unit = Some(o.unit_si.clone());
            o.evidence_class = EvidenceClass::Synthetic;
            o.quality_flags = vec![];
            o.derivation.method_id = "ignis_wsp_serve.typed_scenario".into();
            o.derivation.method_version = "1".into();
            o.derivation.notes =
                Some("typed by a person in the forecast lab; not a measurement".into());
            o.derivation.inputs = vec![];
            observations.push(o);
        }
    }
    let time_origin = release
        .applicability
        .get("time_origin")
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .unwrap_or(template.time_origin.clone());
    let mut bundle = PrefixBundle {
        family: release.family,
        run_id: String::new(),
        session_id: req.test_date.clone(),
        cutoff_s: req.cutoff_s.get(),
        time_origin,
        known_conditions: known,
        known_controls: controls,
        observations,
        contours: vec![],
        phase_established_at_s: req.phase_established_at_s.map(Finite::get),
        support_evidence: vec!["typed scenario (not a NASA run)".into()],
        transform_versions: BTreeMap::new(),
        input_hash: String::new(),
    };
    // The run id names the scenario by its content, so equal scenarios are one prefix.
    let tag = content_hash(&bundle, &["input_hash", "run_id"]);
    bundle.run_id = format!("typed-scenario-{}", &hex_part(&tag)[..16]);
    for o in &mut bundle.observations {
        o.run_id = bundle.run_id.clone();
    }
    bundle.input_hash = content_hash(&bundle, &["input_hash"]);
    Ok(bundle)
}

impl<S: PrefixSource> ModelService for ServedModels<S> {
    fn releases(&self, _: &ExecutionScope) -> ServiceResult<Vec<ModelRelease>> {
        Ok(self.records.clone())
    }

    fn release(&self, scope: &ExecutionScope, id: &ReleaseId) -> ServiceResult<ModelRelease> {
        let r = self.served(id)?;
        if self.law.contains_key(&r.id) {
            return Ok(r.clone());
        }
        self.inner.release(scope, id)
    }

    fn forecast(
        &self,
        scope: &ExecutionScope,
        req: &ForecastRequest,
    ) -> ServiceResult<AnalysisArtifact> {
        let release = self.served(&req.release_id)?;
        let Some(loaded) = self.law.get(&release.id) else {
            return self.inner.forecast(scope, req);
        };
        // The generic law forward: the same checks as the service of ignis-wsp, then the strict
        // forward on an admitted typed scenario (law releases have no stored runs).
        req.validate()?;
        scope.require(Capability::RunModels)?;
        scope.require(Capability::ReadData)?;
        let ForecastInput::Prefix { input_hash } = &req.input else {
            return Err(envelope(
                ErrorCode::CapabilityUnavailable,
                format!(
                    "{} reads typed conditions only (no stored run of this investigation \
                     carries its features): it forecasts only the input_hash of a scenario a \
                     person admitted in the forecast lab (POST /api/v1/models/{{id}}/scenarios); \
                     without one, report the release and the inputs it needs instead of a number",
                    release.model_name
                ),
            ));
        };
        let prefix = self.scenarios.get(input_hash).ok_or_else(|| {
            envelope(
                ErrorCode::NotFound,
                format!("no admitted scenario {input_hash}"),
            )
        })?;
        if prefix.family != release.family {
            return Err(envelope(
                ErrorCode::InvalidRequest,
                format!("scenario {input_hash} was admitted for another family"),
            ));
        }
        let mut settings = req.settings.clone();
        if let Some((served, note)) = self.targets.get(&release.id) {
            match &settings.target {
                None => settings.target = Some(served[0].clone()),
                Some(t) if served.contains(t) => {}
                Some(t) => {
                    return Err(envelope(
                        ErrorCode::InvalidRequest,
                        format!(
                            "target {t} of {} is not served: {}; served: {}",
                            release.model_name,
                            note.as_deref().unwrap_or("it is not the model"),
                            served.join(", ")
                        ),
                    ))
                }
            }
        }
        let artifact = ignis_wsp::artifact_of(loaded, &prefix, &settings.value())
            .map_err(ignis_wsp::service::envelope_of)?;
        ignis_core::analysis::validate_artifact(&artifact, release, &prefix)
            .map_err(ignis_wsp::service::envelope_of)?;
        Ok(artifact)
    }

    fn tool_note(&self) -> Option<String> {
        Some(self.served_note())
    }

    fn cards(&self, _: &ExecutionScope) -> ServiceResult<Vec<ModelCard>> {
        Ok(self.cards.clone())
    }

    fn admit_scenario(
        &self,
        scope: &ExecutionScope,
        req: &ScenarioRequest,
    ) -> ServiceResult<ScenarioAdmitted> {
        req.validate()?;
        scope.require(Capability::RunModels)?;
        let release = self.served(&req.release_id)?;
        if self.no_blends.contains(&release.id) {
            let blend = req.conditions.contains_key("fuel_blend_x")
                || req.conditions.values().any(|v| {
                    matches!(v, ScenarioValue::Text(t) if t.contains('+'))
                        && req.conditions.get("fuel_species").is_some_and(|f| f == v)
                });
            if blend {
                return Err(envelope(
                    ErrorCode::InvalidRequest,
                    format!(
                        "{}: the fuel is a blend and there is no blend law in this release",
                        release.model_name
                    ),
                ));
            }
        }
        let card = self
            .cards
            .iter()
            .find(|c| c.release_id == req.release_id)
            .expect("a served release has a card");
        let policy = self.policies.get(&release.model_name).ok_or_else(|| {
            envelope(
                ErrorCode::Unsupported,
                format!("no prospective policy for {}", release.model_name),
            )
        })?;
        let prefix = scenario_prefix(release, card, policy, req)
            .map_err(|e| envelope(ErrorCode::InvalidRequest, format!("{e:#}")))?;
        let admitted = ScenarioAdmitted {
            release_id: req.release_id.clone(),
            input_hash: prefix.input_hash.clone(),
            evidence_class: EvidenceClass::Synthetic,
            cutoff_s: req.cutoff_s,
            condition_keys: prefix.known_conditions.keys().cloned().collect(),
            observation_count: prefix.observations.len(),
        };
        self.scenarios.put(prefix);
        Ok(admitted)
    }
}
