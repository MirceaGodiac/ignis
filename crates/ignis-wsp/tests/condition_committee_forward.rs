//! Structural checks on synthetic inputs. The new node numbers are test data, not candidates.

mod common;

use ignis_core::contracts::{Family, RawValue};
use ignis_core::registry::{LoadedRelease, TensorMap};
use ignis_wsp::{
    acme_tools, assemble, audit, committee, condition_committee as conditions, models, text,
};
use serde_json::json;

const FAMILIES: [(Family, &str); 2] = [
    (Family::AcmeNormal, "normal"),
    (Family::AcmeInverse, "inverse"),
];
const TIMES: [f64; 7] = [3.0, 6.0, 15.0, 40.0, 90.0, 200.0, 500.0];
const NEUTRAL: &str = "
condition.w[0:5] = [1.0, 0.0, 0.0, 0.0, 0.0]
condition.flow_multiplier[0:5] = 1.0
condition.pressure_multiplier[0:5] = 1.0
";
const SPREAD: &str = "
condition.w[0:5] = [2.0, 1.0, 1.0, 0.0, 0.0]
condition.flow_multiplier[0:5] = [1.0, 0.95, 1.05, 1.0, 1.0]
condition.pressure_multiplier[0:5] = [1.0, 1.02, 0.98, 1.0, 1.0]
";

fn copy_release(r: &LoadedRelease) -> LoadedRelease {
    LoadedRelease {
        release: r.release.clone(),
        params: r.params.clone(),
        posterior: r.posterior.clone(),
    }
}

fn release(family: Family, dir: &str, nodes: Option<&str>) -> LoadedRelease {
    let generation = if nodes.is_some() { 6 } else { 5 };
    let key = format!("{}_{}{}", "acme", dir, generation);
    let model = models::by_key(&key).expect("registered model");
    let spec = model.spec();
    assert_eq!(spec.family, family);
    let base = std::fs::read_to_string(format!(
        "{}/../../configs/models/acme/wsp5/{dir}/baseline-set/edits.wsp",
        env!("CARGO_MANIFEST_DIR"),
    ))
    .unwrap();
    let src = format!("{base}\n{}", nodes.unwrap_or_default());
    let ops = text::compile(&src, &spec).expect("literal test patch");
    let patch = assemble::make_patch(&spec, common::meta("test-condition-committee"), ops);
    let params = assemble::materialize(&spec, &patch, None).unwrap();
    let release = model
        .build_release(&patch, &params, None, &common::binding())
        .unwrap();
    LoadedRelease {
        release,
        params,
        posterior: TensorMap::new(),
    }
}

#[test]
fn neutral_nodes_equal_predecessor_exactly_and_leave_inputs_unchanged() {
    for (family, dir) in FAMILIES {
        let predecessor = release(family, dir, None);
        for nodes in [
            NEUTRAL,
            SPREAD
                .replace("[1.0, 0.95, 1.05, 1.0, 1.0]", "1.0")
                .replace("[1.0, 1.02, 0.98, 1.0, 1.0]", "1.0")
                .as_str(),
        ] {
            let r = release(family, dir, Some(nodes));
            let prefix = acme_tools::default_fixture(family);
            let before = serde_json::to_value(&prefix).unwrap();
            let tensors_before = r.params.clone();
            let settings = json!({"times_s": TIMES});
            let old =
                committee::forecast(&committee::spec(family), &predecessor, &prefix, &settings)
                    .unwrap()
                    .unwrap();
            let new = conditions::forecast(&conditions::spec(family), &r, &prefix, &settings)
                .unwrap()
                .unwrap();
            assert_eq!(new.times_s, old.times_s);
            assert_eq!(new.table, old.table);
            assert_eq!(serde_json::to_value(&prefix).unwrap(), before);
            for (key, tensor) in tensors_before {
                assert_eq!(r.params[&key].data, tensor.data);
            }
        }
    }
}

#[test]
fn nodes_mix_complete_paths_with_fixed_positive_normalized_masses() {
    for (family, dir) in FAMILIES {
        let r = release(family, dir, Some(SPREAD));
        let old_r = release(family, dir, None);
        let prefix = acme_tools::default_fixture(family);
        let before = serde_json::to_value(&prefix).unwrap();
        let settings = json!({"times_s": TIMES});
        let f = conditions::forecast(&conditions::spec(family), &r, &prefix, &settings)
            .unwrap()
            .unwrap();
        assert_eq!(f.nodes.len(), 3);
        assert_eq!(f.nodes.iter().map(|n| n.weight).sum::<f64>(), 1.0);
        assert!(f.nodes.iter().all(|n| n.weight > 0.0));
        for node in &f.nodes {
            let mut p = prefix.clone();
            p.known_conditions
                .get_mut("injected_mass_flow")
                .unwrap()
                .value_si = Some(node.flow_kg_s);
            p.known_conditions.get_mut("pressure").unwrap().value_si = Some(node.pressure_pa);
            let direct = committee::forecast(&committee::spec(family), &old_r, &p, &settings)
                .unwrap()
                .unwrap();
            assert_eq!(node.forecast.table, direct.table);
        }
        for j in 0..f.times_s.len() {
            assert!((f.table.iter().map(|row| row[j]).sum::<f64>() - 1.0).abs() < 1e-12);
            for k in 0..4 {
                let expected: f64 = f
                    .nodes
                    .iter()
                    .map(|n| n.weight * n.forecast.table[k][j])
                    .sum();
                assert!((f.table[k][j] - expected).abs() < 1e-14);
                assert!(f.table[k][j] >= 0.0 && f.table[k][j] <= 1.0);
                if j > 0 {
                    if k == 0 {
                        assert!(f.table[k][j] <= f.table[k][j - 1] + 1e-14);
                    } else {
                        assert!(f.table[k][j] + 1e-14 >= f.table[k][j - 1]);
                    }
                }
            }
        }
        let short = conditions::forecast(
            &conditions::spec(family),
            &r,
            &prefix,
            &json!({"times_s": [15.0, 90.0]}),
        )
        .unwrap()
        .unwrap();
        for k in 0..4 {
            for (short_index, full_index) in [(0, 2), (1, 4)] {
                // Separate horizon grids can change the rounding of member CDF arithmetic.
                // Probabilities are bounded by one, so an absolute 32-epsilon tolerance is
                // a tight roundoff allowance; neutral predecessor identity stays exact above.
                let difference = (short.table[k][short_index] - f.table[k][full_index]).abs();
                assert!(
                    difference <= 32.0 * f64::EPSILON,
                    "category {k}, horizon {}: short {}, full {}, difference {difference}",
                    short.times_s[short_index],
                    short.table[k][short_index],
                    f.table[k][full_index],
                );
            }
        }
        assert_eq!(serde_json::to_value(&prefix).unwrap(), before);
        let artifact =
            conditions::artifact(&conditions::spec(family), &r, &prefix, &settings).unwrap();
        artifact.validate().unwrap();
        assert_eq!(artifact.prefix_hash, prefix.input_hash);
        assert_eq!(artifact.model_release_id, r.release.id);
        assert!(artifact.diagnostics["condition_weight_scope"]
            .as_str()
            .unwrap()
            .contains("cutoff"));
        assert!(artifact.diagnostics["condition_scope"]
            .as_str()
            .unwrap()
            .contains("no_equilibrium"));
    }
}

#[test]
fn invalid_nodes_and_composition_fail_closed_including_inactive_slots() {
    let family = Family::AcmeNormal;
    let r = release(family, "normal", Some(NEUTRAL));
    let prefix = acme_tools::default_fixture(family);
    let spec = conditions::spec(family);
    for (tensor, values) in [
        ("condition.w", vec![0.0; 5]),
        ("condition.w", vec![-1.0, 1.0, 0.0, 0.0, 0.0]),
        ("condition.flow_multiplier", vec![1.0, 1.0, 1.0, 1.0, 0.0]),
        (
            "condition.pressure_multiplier",
            vec![1.0, 1.0, 1.0, 1.0, -1.0],
        ),
        ("condition.pressure_multiplier", vec![f64::MAX; 5]),
    ] {
        let mut invalid = copy_release(&r);
        invalid.params.get_mut(tensor).unwrap().data = values;
        assert!(
            conditions::forecast(&spec, &invalid, &prefix, &json!(null))
                .unwrap()
                .is_err(),
            "{tensor}"
        );
    }
    for value in [f64::NAN, f64::INFINITY] {
        let mut invalid = copy_release(&r);
        invalid.params.get_mut("condition.w").unwrap().data[0] = value;
        assert!(conditions::forecast(&spec, &invalid, &prefix, &json!(null)).is_err());
    }
    for (key, value) in [
        ("x_o2_role", 1.01),
        ("x_fuel_role", 0.0),
        ("zst", 1.0),
        ("tad", -1.0),
    ] {
        let mut invalid = prefix.clone();
        invalid.known_conditions.get_mut(key).unwrap().value_si = Some(value);
        assert!(conditions::forecast(&spec, &r, &invalid, &json!(null))
            .unwrap()
            .is_err());
    }
    let mut missing = prefix.clone();
    missing.known_conditions.remove("pressure");
    assert!(conditions::forecast(&spec, &r, &missing, &json!(null))
        .unwrap()
        .is_err());
}

#[test]
fn scheduled_stop_is_the_same_atom_in_every_node() {
    for (family, dir) in FAMILIES {
        let mut r = release(family, dir, Some(SPREAD));
        r.params.get_mut("mix.w_script").unwrap().data = r.params["mix.w"].data.clone();
        let mut prefix = acme_tools::default_fixture(family);
        let mut stop = prefix.known_conditions["pressure"].clone();
        stop.raw_value = Some(RawValue::Number(100.0));
        stop.value_si = Some(100.0);
        stop.raw_unit = Some("s".into());
        stop.unit_si = Some("s".into());
        prefix
            .known_conditions
            .insert(committee::PROGRAMMED_STOP.into(), stop);
        let before = serde_json::to_value(&prefix).unwrap();
        let f = conditions::forecast(
            &conditions::spec(family),
            &r,
            &prefix,
            &json!({"times_s": TIMES}),
        )
        .unwrap()
        .unwrap();
        let stops: Vec<_> = f
            .nodes
            .iter()
            .map(|n| n.forecast.scheduled_stop_s.unwrap())
            .collect();
        assert!(stops.iter().all(|s| *s == stops[0]));
        assert_eq!(f.table[0][6], 0.0);
        assert_eq!(serde_json::to_value(&prefix).unwrap(), before);
    }
}

#[test]
fn new_identity_covers_every_called_forward_without_changing_the_member_spec() {
    for (family, _) in FAMILIES {
        let old = committee::spec(family);
        let new = conditions::spec(family);
        new.validate().unwrap();
        assert_eq!(&new.tensors[..old.tensors.len()], old.tensors);
        assert_eq!(new.tensors.len(), old.tensors.len() + 3);
        assert_eq!(new.tensors.last().unwrap().shape, vec![5]);
        let sources = audit::forward_sources_of(&new.model_name).unwrap();
        for file in [
            "committee.rs",
            "condition_committee.rs",
            "flame.rs",
            "flame3.rs",
        ] {
            assert!(sources.iter().any(|(name, _)| *name == file));
        }
        assert_ne!(
            audit::forward_hash_of(&new.model_name),
            audit::forward_hash_of(&old.model_name)
        );
    }
}
