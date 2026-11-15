//! The droplet forward for eight fuels (`droplet8.rs`): its specs are those of `droplet6.rs` with the
//! fuel list, the domain of CFI and FLEX-2 and the horizon of section 3.1 of
//! `docs/wsp/research/phys_droplet_fuels.md`, and nothing else; the committed carries of the fifth
//! generation give the forecasts of the `droplet6.rs` releases they copy, number for number; each
//! committed one-shot candidate of a new fuel assembles, forecasts on a synthetic prefix of its fuel
//! and refuses outside its declared domain. Every prefix here is synthetic.

mod common;

use ignis_core::contracts::{
    CheckStatus, ModelRelease, ModelSpec, PrefixBundle, Quantity, RawValue, ReleaseId, TensorFill,
};
use ignis_core::hashing::content_hash;
use ignis_core::registry::{LoadedRelease, TensorMap};
use ignis_wsp::droplet8::{self, Fuel};
use ignis_wsp::{assemble, audit, droplet6, droplet8_tools, flex_tools, models, text};
use serde_json::{json, Value};

const K_HOT: f64 = 0.5e-6;

fn config(path: &str) -> String {
    let path = format!(
        "{}/../../configs/models/flex/{path}",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

fn dir(fuel: Fuel) -> &'static str {
    match fuel {
        Fuel::Methanol => "methanol",
        Fuel::Heptane => "heptane",
        Fuel::Octane => "octane",
        Fuel::Isooctane => "isooctane",
        Fuel::Decane => "decane",
        Fuel::Dodecane => "dodecane",
        Fuel::Farnesane => "farnesane",
        Fuel::Ethanol => "ethanol",
    }
}

fn key(fuel: Fuel) -> String {
    format!("flex_{}7", dir(fuel))
}

/// The fuel of `droplet6.rs` whose tensor set a fuel of this forward uses.
fn anchor(fuel: Fuel) -> droplet6::Fuel {
    if fuel.has_cool() {
        droplet6::Fuel::Heptane
    } else {
        droplet6::Fuel::Methanol
    }
}

fn loaded(mut release: ModelRelease, params: TensorMap) -> LoadedRelease {
    release.id = content_hash(&release, &["id", "created_at"]);
    LoadedRelease {
        release,
        params,
        posterior: TensorMap::new(),
    }
}

fn root(key: &str, src: &str) -> LoadedRelease {
    let model = models::by_key(key).expect("model");
    let spec = model.spec();
    let ops = text::compile(src, &spec).unwrap_or_else(|e| panic!("{key}: {e}"));
    let patch = assemble::make_patch(&spec, common::meta("test-droplet8"), ops);
    let tensors = assemble::materialize(&spec, &patch, None).expect("materialize");
    let release = model
        .build_release(&patch, &tensors, None, &common::binding())
        .expect("release");
    loaded(release, tensors)
}

fn revise(key: &str, parent: &LoadedRelease, src: &str) -> LoadedRelease {
    let model = models::by_key(key).expect("model");
    let spec = model.spec();
    let mut meta = common::meta("test-revision");
    meta.parent_release_id = Some(ReleaseId::new(parent.release.id.clone()).unwrap());
    let ops = text::compile(src, &spec).unwrap_or_else(|e| panic!("{key} revision: {e}"));
    let patch = assemble::make_patch(&spec, meta, ops);
    let tensors = assemble::materialize(&spec, &patch, Some(parent))
        .unwrap_or_else(|e| panic!("{key} revision: {e:#}"));
    let release = model
        .build_release(&patch, &tensors, Some(parent), &common::binding())
        .expect("revision release");
    loaded(release, tensors)
}

/// A committed chain `generation/dir/[steps]`, built in memory from its edits.
fn chain(key: &str, generation: &str, dir: &str, steps: &[&str]) -> Vec<LoadedRelease> {
    let mut out: Vec<LoadedRelease> = Vec::new();
    for step in steps {
        let src = config(&format!("{generation}/{dir}/{step}/edits.wsp"));
        let r = match out.last() {
            None => root(key, &src),
            Some(parent) => revise(key, parent, &src),
        };
        out.push(r);
    }
    out
}

fn rehash(mut p: PrefixBundle) -> PrefixBundle {
    p.input_hash = content_hash(&p, &["input_hash"]);
    p
}

#[derive(Clone, Copy)]
struct Gas {
    x_o2: f64,
    x_co2: f64,
    x_he: f64,
    p: f64,
}

const AIR: Gas = Gas {
    x_o2: 0.21,
    x_co2: 0.0,
    x_he: 0.0,
    p: 101_325.0,
};
const HELIUM_CO2: Gas = Gas {
    x_o2: 0.21,
    x_co2: 0.10,
    x_he: 0.20,
    p: 150_000.0,
};

/// A droplet whose fuel condition reads `label`, of initial diameter `d0` and D^2 `q(t)`, observed
/// at 30 Hz to `cutoff`, in `gas`.
fn prefix(label: &str, d0: f64, cutoff: f64, gas: Gas, q: impl Fn(f64) -> f64) -> PrefixBundle {
    let mut p = flex_tools::fixture_prefix(d0, K_HOT, cutoff, gas.x_o2, gas.x_co2, gas.x_he, gas.p);
    if let Some(f) = p.known_conditions.get_mut("fuel") {
        f.raw_value = Some(RawValue::Text(label.into()));
    }
    for o in &mut p.observations {
        o.value_si = Some(q(o.time_s).sqrt());
    }
    rehash(p)
}

fn with_flame(mut p: PrefixBundle, times: impl IntoIterator<Item = f64>) -> PrefixBundle {
    let template = p.observations[0].clone();
    for (i, t) in times.into_iter().enumerate() {
        let mut o = template.clone();
        o.id = format!("uv/{i}");
        o.quantity = Quantity::UvFlameDiameter;
        o.channel = Some("LLUVData".into());
        o.time_s = t;
        o.available_at_s = t;
        o.value_si = Some(0.012);
        p.observations.push(o);
    }
    rehash(p)
}

fn frames(from: f64, to: f64) -> Vec<f64> {
    let n = ((to - from) * 30.0).floor() as usize;
    (0..=n).map(|i| from + i as f64 / 30.0).collect()
}

fn run8(
    fuel: Fuel,
    r: &LoadedRelease,
    p: &PrefixBundle,
    settings: &Value,
) -> Result<droplet8::Forecast, Vec<String>> {
    droplet8::forecast(&droplet8::spec(fuel), r, p, settings)
        .expect("runnable")
        .map_err(|s| s.reasons)
}

fn constant(spec: &ModelSpec, name: &str) -> f64 {
    match &spec.tensors.iter().find(|t| t.name == name).unwrap().fill {
        TensorFill::Fixed { values } => values[0].get(),
        _ => panic!("{name} is not a constant"),
    }
}

#[test]
fn the_specs_are_those_of_droplet6_with_the_fuels_domain_and_horizon_of_section_3_1() {
    for fuel in Fuel::ALL {
        let spec = droplet8::spec(fuel);
        spec.validate().expect("spec valid");
        assert_eq!(
            models::by_key(&key(fuel)).unwrap().spec().spec_hash(),
            spec.spec_hash()
        );
        assert_eq!(spec.model_name, fuel.model_name());
        let on_disk: ModelSpec =
            serde_json::from_str(&config(&format!("wsp7/{}/spec.json", dir(fuel)))).unwrap();
        assert_eq!(
            on_disk.spec_hash(),
            spec.spec_hash(),
            "configs/models/flex/wsp7/{}/spec.json drifted from droplet8::spec(): a changed spec is a \
             new spec id (re-export with `ignis wsp spec --family {} --out …`)",
            dir(fuel),
            key(fuel)
        );
        // the tensor set of the anchor fuel, every tensor unchanged but the horizon
        let old = droplet6::spec(anchor(fuel));
        assert_eq!(spec.tensors.len(), old.tensors.len());
        for (a, b) in spec.tensors.iter().zip(&old.tensors) {
            if a.name == "num.t_valid_s" {
                assert_eq!(constant(&spec, &a.name), 200.0);
                assert_eq!(constant(&old, &a.name), 45.0);
            } else {
                assert_eq!(a, b, "{}: {}", fuel.label(), a.name);
            }
        }
        assert_eq!(spec.features, old.features);
        // the domain: c_logp to 1.7 and c_he to 4.25, the rest as droplet6.rs
        assert_eq!(spec.domain.len(), old.domain.len());
        for (a, b) in spec.domain.iter().zip(&old.domain) {
            assert_eq!((&a.feature, a.min), (&b.feature, b.min));
            let hi = a.max.unwrap().get();
            match a.feature.as_str() {
                "c_logp" => assert_eq!((hi, b.max.unwrap().get()), (1.7, 1.5)),
                "c_he" => assert_eq!((hi, b.max.unwrap().get()), (4.25, 3.0)),
                _ => assert_eq!(a.max, b.max, "{}", a.feature),
            }
        }
        assert_eq!(fuel.has_cool(), anchor(fuel) == droplet6::Fuel::Heptane);
        // CFI's 506.6 kPa and FLEX-2's 83 % helium are inside
        assert!((506_600.0f64 / 1.0e5).ln() < 1.7 && 0.83 / 0.20 < 4.25);
    }
}

#[test]
fn a_fuel_condition_names_the_fuel_after_normalisation() {
    assert!(Fuel::Heptane.accepts("Heptane"));
    assert!(Fuel::Heptane.accepts("n-heptane"));
    assert!(Fuel::Methanol.accepts("Methanol"));
    assert!(Fuel::Octane.accepts("n_octane"));
    assert!(Fuel::Octane.accepts("n-Octane"));
    assert!(Fuel::Isooctane.accepts("iso-octane"));
    assert!(Fuel::Isooctane.accepts("isooctane"));
    assert!(Fuel::Isooctane.accepts("2,2,4-Trimethylpentane"));
    assert!(Fuel::Dodecane.accepts("dodecane"));
    assert!(!Fuel::Dodecane.accepts("iso_dodecane"));
    assert!(!Fuel::Dodecane.accepts("dodecane+iso_dodecane"));
    assert!(!Fuel::Octane.accepts("isooctane"));
    assert!(!Fuel::Heptane.accepts("heptane+isooctane"));
    assert!(!Fuel::Ethanol.accepts("methanol"));
    for fuel in Fuel::ALL {
        assert!(fuel.accepts(fuel.label()));
        assert_eq!(Fuel::of_model(fuel.model_name()), Some(fuel));
        for other in Fuel::ALL.into_iter().filter(|o| *o != fuel) {
            assert!(
                !other.accepts(fuel.label()),
                "{} accepts {}",
                other.label(),
                fuel.label()
            );
        }
    }
}

/// The committed fifth-generation chains and their copies on this forward.
const CARRIES: [(Fuel, droplet6::Fuel, &str, [&str; 4], [&str; 4]); 2] = [
    (
        Fuel::Methanol,
        droplet6::Fuel::Methanol,
        "methanol",
        ["baseline-d2", "carry-d3", "p0-fable", "p1-fable"],
        ["baseline-d2", "carry-d3", "carry-p0", "carry-p1"],
    ),
    (
        Fuel::Heptane,
        droplet6::Fuel::Heptane,
        "heptane",
        ["baseline-d2", "carry-d2", "p0-fable", "p1-fable"],
        ["baseline-d2", "carry-d2", "carry-p0", "carry-p1"],
    ),
];

#[test]
fn the_committed_carries_give_the_forecasts_of_the_droplet6_releases_they_copy() {
    let settings = json!({ "horizon_s": 12.0, "n_points": 25 });
    for (fuel, old_fuel, d, old_steps, new_steps) in CARRIES {
        let old_key = format!("flex_{d}5");
        let old = chain(&old_key, "wsp5", d, &old_steps);
        let new = chain(&key(fuel), "wsp7", d, &new_steps);
        for (i, (a, b)) in old.iter().zip(&new).enumerate() {
            // the same tensors, element for element (the horizon constant aside)
            assert_eq!(a.params.len(), b.params.len());
            for (name, t) in &a.params {
                if name != "num.t_valid_s" {
                    assert_eq!(&b.params[name], t, "{d} {}: {name}", new_steps[i]);
                }
            }
            // the committed copy holds these weights, was assembled with this forward and passed
            // every construction check; the committed droplet6 release still matches its chain
            let path = format!("wsp7/{d}/{}/release.json", new_steps[i]);
            let on_disk: ModelRelease = serde_json::from_str(&config(&path)).unwrap();
            let c = on_disk.construction.as_ref().expect("literal");
            assert_eq!(
                c.weights_hash,
                b.release.construction.as_ref().unwrap().weights_hash,
                "{path}"
            );
            assert_eq!(c.spec_hash, droplet8::spec(fuel).spec_hash(), "{path}");
            assert_eq!(
                c.forward_hash,
                audit::forward_hash_of(fuel.model_name()),
                "{path}"
            );
            assert!(
                c.checks.iter().all(|k| k.status == CheckStatus::Passed),
                "{path}"
            );
            let old_path = format!("wsp5/{d}/{}/release.json", old_steps[i]);
            let old_disk: ModelRelease = serde_json::from_str(&config(&old_path)).unwrap();
            assert_eq!(
                old_disk.construction.as_ref().unwrap().weights_hash,
                a.release.construction.as_ref().unwrap().weights_hash,
                "{old_path}"
            );
        }
        let hot_then = |t1: f64, k1: f64, t2: f64, k2: f64, d0: f64| {
            move |t: f64| {
                let q = d0 * d0 - K_HOT * t.min(t1);
                q - k1 * (t.min(t2) - t1).max(0.0) - k2 * (t - t2).max(0.0)
            }
        };
        let (small, big) = (3.0e-3, 4.0e-3);
        let label = old_fuel.label();
        let burning = prefix(label, small, 3.0, AIR, |t| small * small - K_HOT * t);
        let lost = prefix(label, big, 6.0, AIR, hot_then(4.0, 0.03e-6, 9.0, 0.0, big));
        let prefixes = [
            burning.clone(),
            prefix(
                label,
                big,
                5.0,
                HELIUM_CO2,
                hot_then(9.0, 0.0, 9.0, 0.0, big),
            ),
            lost.clone(),
            prefix(label, big, 7.0, AIR, hot_then(4.0, 0.25e-6, 9.0, 0.0, big)),
            prefix(
                label,
                big,
                8.0,
                AIR,
                hot_then(3.0, 0.25e-6, 6.0, 0.02e-6, big),
            ),
            prefix(
                label,
                small,
                5.0,
                HELIUM_CO2,
                hot_then(3.5, 0.02e-6, 9.0, 0.02e-6, small),
            ),
            with_flame(burning, frames(0.3, 3.0)),
            with_flame(lost, frames(0.3, 4.0)),
        ];
        let mut reached = [false; 3];
        for (a, b) in old.iter().zip(&new).skip(1) {
            for p in &prefixes {
                let f6 = droplet6::forecast(&droplet6::spec(old_fuel), a, p, &settings)
                    .expect("runnable")
                    .expect("admitted");
                let f8 = run8(fuel, b, p, &settings).expect("admitted");
                assert_eq!(f8.times_s, f6.times_s);
                assert_eq!(f8.quantiles, f6.quantiles, "{d}, cutoff {}", p.cutoff_s);
                assert_eq!(f8.latent_quantiles, f6.latent_quantiles);
                assert_eq!(f8.mean, f6.mean);
                assert_eq!(f8.events, f6.events);
                assert_eq!(f8.state_probability, f6.state_probability);
                assert_eq!(f8.encoded.state, f6.encoded.state);
                assert_eq!(f8.encoded.b, f6.encoded.b);
                assert_eq!((f8.rk4_steps, f8.max_atoms), (f6.rk4_steps, f6.max_atoms));
                for (i, r) in reached.iter_mut().enumerate() {
                    *r |= f8.state_probability[i].iter().any(|v| *v > 0.3);
                }
            }
            // and on the fixture of the build check
            let fx = droplet8_tools::fixture_prefix(fuel);
            let f6 = droplet6::forecast(&droplet6::spec(old_fuel), a, &fx, &Value::Null)
                .unwrap()
                .unwrap();
            let f8 = run8(fuel, b, &fx, &Value::Null).unwrap();
            assert_eq!(f8.quantiles, f6.quantiles);
            assert_eq!(f8.events, f6.events);
        }
        assert!(
            reached[0] && reached[2],
            "{d}: states not reached {reached:?}"
        );
        if fuel.has_cool() {
            assert!(reached[1], "{d}: no case with a cool flame");
        }
        // where they differ: the widened domain and horizon
        let p5 = prefix(
            label,
            big,
            3.0,
            Gas {
                p: 506_600.0,
                ..AIR
            },
            |t| big * big - K_HOT * t,
        );
        let carry = &new[1];
        assert!(
            droplet6::forecast(&droplet6::spec(old_fuel), &old[1], &p5, &settings)
                .unwrap()
                .is_err()
        );
        assert!(run8(fuel, carry, &p5, &settings).is_ok());
        let late = json!({ "times_s": [10.0, 60.0, 150.0] });
        assert!(run8(fuel, carry, &burning_of(label), &late).is_ok());
        assert!(run8(
            fuel,
            carry,
            &burning_of(label),
            &json!({ "times_s": [201.0] })
        )
        .is_err());
    }
}

fn burning_of(label: &str) -> PrefixBundle {
    prefix(label, 3.0e-3, 3.0, AIR, |t| 9.0e-6 - K_HOT * t)
}

const CANDIDATES: [Fuel; 6] = [
    Fuel::Octane,
    Fuel::Isooctane,
    Fuel::Decane,
    Fuel::Dodecane,
    Fuel::Farnesane,
    Fuel::Ethanol,
];

#[test]
fn each_candidate_forecasts_its_fuel_and_refuses_outside_its_declared_domain() {
    let settings = json!({ "horizon_s": 20.0, "n_points": 21 });
    for fuel in CANDIDATES {
        let d = dir(fuel);
        let releases = chain(&key(fuel), "wsp7", d, &["baseline-d2", "w0-fable"]);
        // the root is the anchor's baseline, unchanged
        let anchor_root = chain(
            &format!(
                "flex_{}5",
                if fuel.has_cool() {
                    "heptane"
                } else {
                    "methanol"
                }
            ),
            "wsp5",
            if fuel.has_cool() {
                "heptane"
            } else {
                "methanol"
            },
            &["baseline-d2"],
        );
        assert_eq!(
            config(&format!("wsp7/{d}/baseline-d2/edits.wsp"))
                .lines()
                .filter(|l| !l.starts_with('#'))
                .collect::<Vec<_>>(),
            config(&format!(
                "wsp5/{}/baseline-d2/edits.wsp",
                if fuel.has_cool() {
                    "heptane"
                } else {
                    "methanol"
                }
            ))
            .lines()
            .filter(|l| !l.starts_with('#'))
            .collect::<Vec<_>>()
        );
        for (name, t) in &anchor_root[0].params {
            if name != "num.t_valid_s" {
                assert_eq!(&releases[0].params[name], t, "{d} root: {name}");
            }
        }
        let cand = &releases[1];
        for (step, r) in [("baseline-d2", &releases[0]), ("w0-fable", cand)] {
            let path = format!("wsp7/{d}/{step}/release.json");
            let on_disk: ModelRelease = serde_json::from_str(&config(&path)).unwrap();
            let c = on_disk.construction.as_ref().expect("literal");
            assert_eq!(
                c.weights_hash,
                r.release.construction.as_ref().unwrap().weights_hash
            );
            assert_eq!(c.forward_hash, audit::forward_hash_of(fuel.model_name()));
            assert!(
                c.checks.iter().all(|k| k.status == CheckStatus::Passed),
                "{path}"
            );
        }
        let committed: ModelRelease =
            serde_json::from_str(&config(&format!("wsp7/{d}/w0-fable/release.json"))).unwrap();
        let root_disk: ModelRelease =
            serde_json::from_str(&config(&format!("wsp7/{d}/baseline-d2/release.json"))).unwrap();
        assert_eq!(committed.parent_id.as_deref(), Some(root_disk.id.as_str()));

        // a forecast on a synthetic prefix of its fuel, in air and in a diluted atmosphere
        let label = fuel.label();
        for p in [
            droplet8_tools::fixture_prefix(fuel),
            prefix(label, 3.0e-3, 3.0, HELIUM_CO2, |t| 9.0e-6 - K_HOT * t),
            prefix(
                label,
                5.0e-3,
                5.0,
                Gas {
                    p: 506_600.0,
                    ..AIR
                },
                |t| 25.0e-6 - 0.4e-6 * t,
            ),
            prefix(
                label,
                3.0e-3,
                3.0,
                Gas {
                    x_o2: 0.17,
                    x_co2: 0.0,
                    x_he: 0.83,
                    p: 101_325.0,
                },
                |t| 9.0e-6 - K_HOT * t,
            ),
        ] {
            let f =
                run8(fuel, cand, &p, &settings).unwrap_or_else(|r| panic!("{d}: refused {r:?}"));
            assert!(f
                .quantiles
                .iter()
                .flatten()
                .all(|v| v.is_finite() && *v >= 0.0));
            assert!(f.mean.iter().all(|v| v.is_finite() && *v > 0.0));
        }
        let long = json!({ "times_s": [10.0, 60.0, 120.0, 199.0] });
        assert!(run8(fuel, cand, &droplet8_tools::fixture_prefix(fuel), &long).is_ok());
        // refusals: above the pressure and helium bounds, another fuel, past the horizon
        let outside = [
            prefix(
                label,
                3.0e-3,
                3.0,
                Gas {
                    p: 600_000.0,
                    ..AIR
                },
                |t| 9.0e-6 - K_HOT * t,
            ),
            prefix(
                label,
                3.0e-3,
                3.0,
                Gas {
                    x_o2: 0.10,
                    x_co2: 0.0,
                    x_he: 0.90,
                    p: 101_325.0,
                },
                |t| 9.0e-6 - K_HOT * t,
            ),
            prefix(label, 3.0e-3, 3.0, Gas { x_o2: 0.08, ..AIR }, |t| {
                9.0e-6 - K_HOT * t
            }),
            prefix("Methanol", 3.0e-3, 3.0, AIR, |t| 9.0e-6 - K_HOT * t),
            prefix("dodecane+iso_dodecane", 3.0e-3, 3.0, AIR, |t| {
                9.0e-6 - K_HOT * t
            }),
        ];
        for (i, p) in outside.iter().enumerate() {
            assert!(
                run8(fuel, cand, p, &settings).is_err(),
                "{d}: case {i} admitted"
            );
        }
        assert!(run8(
            fuel,
            cand,
            &droplet8_tools::fixture_prefix(fuel),
            &json!({ "times_s": [250.0] })
        )
        .is_err());
    }
}
