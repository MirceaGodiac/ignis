//! Everything about the `law.rs` models that is not their forward: the model records of the slug
//! table, release declarations, the synthetic fixture, the build check and the neutral root. Kept
//! out of `law.rs` so that a change here never changes a law release's forward identity.

use std::collections::BTreeMap;

use serde_json::{json, Value};

use ignis_core::contracts::{
    AuthorModel, BuildCheck, BuildCheckKind, CheckStatus, DataExposure, EvidenceClass, Measured,
    ModelSpec, Origin, PatchId, PrefixBundle, RawValue, Resolution, TensorFill,
};
use ignis_core::registry::LoadedRelease;

use crate::assemble::PatchMeta;
use crate::law::{self, Kind, Status};
use crate::law_slugs::{self, SLUGS};
use crate::models::{Forward, Model};

/// The model record of slug `i` (key = the slug, `model_name` = `<slug>.wsp`).
pub fn model(i: usize) -> Model {
    Model {
        key: SLUGS[i].slug,
        model_name: SLUGS[i].model_name,
        family: law::decl(i).family,
        forward: Forward::Law(i),
    }
}

pub fn model_by_key(key: &str) -> Option<Model> {
    law_slugs::by_key(key).map(model)
}

pub fn model_by_name(name: &str) -> Option<Model> {
    law_slugs::by_model_name(name).map(model)
}

/// What the registry record of a law release declares besides spec, patch and tensors.
pub fn release_notes(i: usize) -> (BTreeMap<&'static str, Value>, Value, Value) {
    let d = law::decl(i);
    let targets: Vec<Value> = d.targets.iter().map(|t| law::law_block(i, t)).collect();
    let applicability = BTreeMap::from([
        ("slug", json!(SLUGS[i].slug)),
        ("declaration_path", json!(SLUGS[i].decl_path)),
        ("declaration", json!(d)),
        ("status", json!(d.status)),
        ("investigation", json!(d.investigation)),
        ("features", json!(d.features)),
        ("targets", json!(targets)),
        (
            "meaning",
            json!("declared domain of the handwritten law (tensors x.<feature>.lo, x.<feature>.hi), not a training-support region"),
        ),
    ]);
    let observation_model = json!({
        "targets": targets,
        "lane_local": "a target without a shared quantity is labelled <slug>:<target> in diagnostics.law; its artifacts carry quantity = null",
        "noise": "assumed: the spread tensors of each target are the author's statement of how unsure the law is",
    });
    let numerics = json!({
        "event_integration": "num.substeps equal steps per requested interval; exact survival, causes share each step's loss by their midpoint hazards",
        "positive_quantiles": "closed form for one log-normal; bisection of the fixed two-component mixture CDF otherwise",
        "request_time_fitting": "none",
    });
    (applicability, observation_model, numerics)
}

fn measured(v: f64, unit: &str) -> Measured {
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

/// A synthetic prefix of slug `i` (`evidence_class = synthetic`, never NASA data) with the given
/// value of every declared feature, cutoff 0.
pub fn prefix_with(i: usize, values: &[f64], run_id: &str) -> PrefixBundle {
    let d = law::decl(i);
    let mut p = PrefixBundle {
        family: d.family,
        run_id: run_id.into(),
        session_id: None,
        cutoff_s: 0.0,
        time_origin: "synthetic fixture: the time origin of the law".into(),
        known_conditions: d
            .features
            .iter()
            .zip(values)
            .map(|(f, v)| (f.name.clone(), measured(*v, &f.unit)))
            .collect(),
        known_controls: vec![],
        observations: vec![],
        contours: vec![],
        phase_established_at_s: Some(0.0),
        support_evidence: vec!["synthetic fixture".into()],
        transform_versions: BTreeMap::new(),
        input_hash: String::new(),
    };
    p.input_hash = ignis_core::hashing::content_hash(&p, &["input_hash"]);
    p
}

/// The fixture prefix without a release: every feature at 1 (a value every transform can take).
pub fn fixture_prefix(i: usize) -> PrefixBundle {
    let n = law::decl(i).features.len();
    prefix_with(i, &vec![1.0; n], &format!("wsp-fixture-{}", SLUGS[i].slug))
}

/// A point of the release's declared domain for each feature: the centre when it lies in the
/// domain, otherwise the middle of the domain.
fn fixture_values(i: usize, release: &LoadedRelease) -> Vec<f64> {
    let s = |name: String| {
        release
            .params
            .get(&name)
            .and_then(|t| t.data.first().copied())
            .unwrap_or(f64::NAN)
    };
    law::decl(i)
        .features
        .iter()
        .map(|f| {
            let n = &f.name;
            let (c, lo, hi) = (
                s(format!("x.{n}.centre")),
                s(format!("x.{n}.lo")),
                s(format!("x.{n}.hi")),
            );
            if lo <= c && c <= hi {
                c
            } else {
                0.5 * (lo + hi)
            }
        })
        .collect()
}

/// Settings that ask a forecast of target `t` on the fixture.
pub fn fixture_settings(t: &law::TargetDecl) -> Value {
    match t.kind {
        Kind::Event => json!({"target": t.name, "times_s": [0.0, 1.0, 10.0, 100.0, 1000.0]}),
        _ => json!({"target": t.name}),
    }
}

/// The build check of a law release: every target forecast on the synthetic fixture is a valid,
/// finite, admitted artifact. A placeholder declaration fails it, so no placeholder release exists.
pub fn fixture_check(i: usize, model: Model, release: &LoadedRelease) -> BuildCheck {
    let outcome: anyhow::Result<String> = (|| {
        let d = law::decl(i);
        if d.status == Status::Placeholder {
            anyhow::bail!(
                "{} is a placeholder declaration: restate the law ({}) before assembling a release",
                SLUGS[i].slug,
                SLUGS[i].decl_path
            );
        }
        let prefix = prefix_with(
            i,
            &fixture_values(i, release),
            &format!("wsp-fixture-{}", SLUGS[i].slug),
        );
        for t in &d.targets {
            let a = model
                .artifact(release, &prefix, &fixture_settings(t))
                .map_err(|r| anyhow::anyhow!("{r}"))?;
            a.validate().map_err(anyhow::Error::msg)?;
            if !a.support.admitted {
                anyhow::bail!(
                    "fixture refused for target {}: {}",
                    t.name,
                    a.support.reasons.join("; ")
                );
            }
        }
        Ok(format!(
            "synthetic fixture prefix: {} targets, valid finite artifacts",
            d.targets.len()
        ))
    })();
    let (status, detail) = match outcome {
        Ok(detail) => (CheckStatus::Passed, detail),
        Err(e) => (CheckStatus::Failed, format!("{e:#}")),
    };
    BuildCheck {
        check: BuildCheckKind::FiniteForwardOnFixture,
        status,
        detail: Some(detail),
    }
}

fn number(v: f64) -> String {
    let s = format!("{v:?}");
    if s.contains(['.', 'e', 'E']) {
        s
    } else {
        format!("{s}.0")
    }
}

/// The neutral root of a slug as an edits file, generated from the spec alone (no claim): every
/// `b0` 0 but the spread `b0` at its upper bound, skips 0, no hinge, no wider component, shapes at
/// their lower bound, stop offsets and weights 0, centres and scales 1, domain bounds at the
/// tensor bounds.
pub fn root_edits(spec: &ModelSpec) -> String {
    let mut out = format!(
        "# {} — neutral root of spec {}, generated by `ignis wsp law-root` from the spec alone.\n\
         # No claim: no feature weight, no hinge, the widest spread and shapes the spec allows, no domain\n\
         # beyond the tensor bounds. A lineage root, never a candidate. Not typed by any author.\n",
        spec.model_name, spec.spec_id
    );
    for t in &spec.tensors {
        if matches!(
            t.fill,
            TensorFill::Fixed { .. } | TensorFill::StructuralZero
        ) {
            continue;
        }
        let (lo, hi) = t
            .bounds
            .as_ref()
            .map_or((f64::NAN, f64::NAN), |b| (b.min.get(), b.max.get()));
        let n = &t.name;
        let v = if n.ends_with(".spread.b0") {
            hi
        } else if n.ends_with(".shape") || n.ends_with(".tail_width") || n.ends_with(".lo") {
            lo
        } else if n.ends_with(".hi") {
            hi
        } else if n.ends_with(".centre") || n.ends_with(".scale") {
            1.0
        } else {
            0.0
        };
        if t.shape.is_empty() {
            out.push_str(&format!("{n} = {}\n", number(v)));
        } else {
            let sel = vec![":"; t.shape.len()].join(", ");
            out.push_str(&format!("{n}[{sel}] = {}\n", number(v)));
        }
    }
    out
}

/// The provenance of the neutral root.
pub fn root_meta(spec: &ModelSpec) -> PatchMeta {
    PatchMeta {
        patch_id: PatchId::new(format!("{}-root", spec.spec_id)).expect("id"),
        parent_release_id: None,
        author: AuthorModel {
            provider: "ignis".into(),
            model_id: "ignis-wsp-law-root".into(),
            effort: None,
        },
        exposure: DataExposure::default(),
        rationale: Some(
            "Neutral root of a law slug, generated from the spec alone by `ignis wsp law-root`: no claim \
             (no feature weight, no hinge, the widest spread and clock shapes the spec allows, domain at \
             the tensor bounds). A lineage root for literal revisions; never a candidate."
                .into(),
        ),
        origin: Origin::System,
    }
}
