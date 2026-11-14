//! One synthetic fit -> save -> load -> forecast round trip, the semantic refusal, and VOI never
//! reading candidate values or the continuation. Not NASA validation.

use serde_json::json;

use ignis_core::contracts::{DecisionPolicy, EvidenceClass, Family};
use ignis_core::expert::{find, FitContext, FitData, Refusal};
use ignis_core::registry::load;
use ignis_core::resources::load_profile;
use ignis_core::views::ViewSource;
use ignis_flex::synthetic::archive_of;
use ignis_flex::synthetic::make_dataset;
use ignis_train::decide::{build_episode, decide, replay};

#[test]
fn fit_save_load_forecast_and_voi_isolation() {
    let root = std::env::temp_dir().join(format!("ignis-rt-{}", std::process::id()));
    std::env::set_var(
        "IGNIS_CONFIGS",
        concat!(env!("CARGO_MANIFEST_DIR"), "/../../configs"),
    );
    let ds = make_dataset(12, 1, 3, 0.1).unwrap();
    let experts = ignis_flex::experts();
    let ex = find(&experts, "flex.v1").unwrap();
    let cfg = json!({"fit": {"method": "map", "map": {"epochs": 2, "steps_per_epoch": 40, "learning_rate": 0.02}},
                     "eval": {"n_global": 1, "n_local": 32}, "support": {"k": 1, "min_groups": 1, "radius_percentile": 100.0}});
    let views = ds.views(Family::Flex, &ds.cohort.id).unwrap();
    let prof = load_profile("cpu-local").unwrap();
    let mut ctx = FitContext {
        pool: prof.pool(),
        profile: prof,
        seed: 0,
        registry_root: root.clone(),
        checkpoint_dir: root.join("ckpt"),
        resume: None,
        job_id: None,
    };
    let fd = FitData {
        views: &views,
        split: &ds.split,
        cohort_id: &ds.cohort.id,
        source_manifest_hash: "synthetic",
        synthetic: true,
    };
    let rel = ex.fit(&fd, &cfg, &mut ctx).unwrap();
    assert_eq!(rel.trained_on, ignis_core::contracts::TrainedOn::Synthetic);
    assert_eq!(rel.applicability["promotable"], json!(false));

    let loaded = load(&root, &rel.id).unwrap();
    assert_eq!(loaded.release.feature_order, rel.feature_order);
    let v = ds
        .views
        .iter()
        .find(|v| {
            ds.split.partition_of(&v.run_id) == Some(ignis_core::contracts::Partition::Validation)
        })
        .unwrap();
    let archive = archive_of(v);
    let (rec, obs) = (archive.record.clone(), archive.observations.clone());
    let prefix = ignis_core::prefix::build_prefix(
        &rec,
        &obs,
        2.0,
        ex.policy(),
        &v.time_origin,
        ignis_core::prefix::PrefixInputs {
            phase_established_at_s: Some(0.0),
            ..Default::default()
        },
    )
    .unwrap();
    let art = ex
        .forecast(&loaded, &prefix, &json!({"times_s": [2.0, 3.0, 5.0]}))
        .unwrap();
    art.validate().unwrap();
    assert_eq!(
        art.event_categories,
        [
            "active_supported",
            "hot_phase_end",
            "disruption",
            "unresolved"
        ]
    );
    for j in 0..3 {
        let s: f64 = art.event_cdf.iter().map(|r| r[j]).sum();
        assert!((s - 1.0).abs() < 1e-9);
        assert!(art.quantiles[0][j] <= art.quantiles[4][j]);
    }
    // a log likelihood of a held-out view is finite
    assert!(ex
        .log_likelihood(&loaded, &ignis_core::views::View::Flex(v.clone()))
        .unwrap()
        .is_finite());
    // refusal: synthetic releases never forecast non-synthetic observations
    let nasa_like: Vec<_> = obs
        .iter()
        .cloned()
        .map(|mut o| {
            o.evidence_class = EvidenceClass::SourceDerived;
            o
        })
        .collect();
    let p2 = ignis_core::prefix::build_prefix(
        &rec,
        &nasa_like,
        2.0,
        ex.policy(),
        &v.time_origin,
        ignis_core::prefix::PrefixInputs {
            phase_established_at_s: Some(0.0),
            ..Default::default()
        },
    )
    .unwrap();
    let err = ex.forecast(&loaded, &p2, &json!({})).unwrap_err();
    assert_eq!(Refusal::from_error(&err).unwrap().code, "semantic");

    // VOI: corrupt every hidden value and the whole continuation; the ranking must be identical.
    let m = ex.candidate_predictive().unwrap();
    let s = json!({"n_outcomes": 8, "target_horizon_s": 1.0});
    let (pre, pool, vault) = build_episode(&archive, 2.0, 6, 6, ex.policy()).unwrap();
    assert!(!pool.is_empty() && pool.iter().all(|a| a.time_s <= 2.0));
    let t1 = decide(m, &loaded, &pre, &pool, DecisionPolicy::Voi, &s).unwrap();
    let corrupted: Vec<_> = obs
        .iter()
        .cloned()
        .map(|mut o| {
            if vault.contains_key(&o.id) || o.time_s > 2.0 {
                o.value_si = Some(1.0);
            }
            o
        })
        .collect();
    let mut hidden_changed = archive.clone();
    hidden_changed.observations = corrupted;
    let (pre2, pool2, _) = build_episode(&hidden_changed, 2.0, 6, 6, ex.policy()).unwrap();
    assert_eq!(pre2.input_hash, pre.input_hash);
    let t2 = decide(m, &loaded, &pre2, &pool2, DecisionPolicy::Voi, &s).unwrap();
    assert!(t1.chosen_action_id.is_some());
    assert_eq!(t1.id, t2.id);
    // The server's acquisition scoring is the same decision and reads only the prefix and the actions.
    let via_server =
        ignis_train::decide::score_candidates(&experts, &loaded, &pre2, &pool2, &s).unwrap();
    assert_eq!(via_server.id, t1.id);
    // replay: values only through reveal, only for the chosen action, only before the cutoff
    let mut seen = vec![];
    let mut reveal = |id: &str| {
        seen.push(id.to_string());
        vault[id].clone()
    };
    let r = replay(
        ex,
        m,
        &loaded,
        &rec,
        &pre,
        &pool,
        &mut reveal,
        1,
        DecisionPolicy::Voi,
        &s,
        &json!({"times_s": [2.0, 3.0]}),
    )
    .unwrap();
    let chosen = pool
        .iter()
        .find(|a| Some(&a.id) == t1.chosen_action_id.as_ref())
        .unwrap();
    assert_eq!(seen, vec![chosen.observation_id.clone()]);
    assert_eq!(r.final_prefix.cutoff_s, 2.0);
    assert!(r.final_prefix.observations.iter().all(|o| o.time_s <= 2.0));
    let _ = std::fs::remove_dir_all(&root);
}
