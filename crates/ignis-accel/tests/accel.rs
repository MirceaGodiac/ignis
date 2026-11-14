//! Essential accelerator checks (V3.1 §26): parameter inventory, tensor RQ spline = FP64 reference
//! (inverse, derivative), exact flow proposal density, and a tasks → train → save → benchmark smoke.

use candle_core::{DType, Device, Tensor};
use ignis_accel::nn::{self, Params};
use ignis_science::flows as fl;
use serde_json::json;

#[test]
fn parameter_inventory() {
    assert_eq!(nn::encoder_params(), 73856);
    let fams = ["flex", "acme_normal", "acme_inverse", "bass"];
    let tot: Vec<usize> = fams.iter().map(|f| nn::total_params(f)).collect();
    assert_eq!(tot, vec![167316, 161080, 161080, 223440]);
    assert_eq!(tot.iter().sum::<usize>(), 712916);
}

#[test]
fn tensor_spline_matches_fp64_reference() {
    let dev = Device::Cpu;
    let n = 400;
    let mut seed = 3u64;
    let mut rnd = || {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((seed >> 11) as f64 / (1u64 << 53) as f64 - 0.5) * 4.0
    };
    let raw: Vec<f64> = (0..n * 23).map(|_| rnd()).collect();
    let x: Vec<f64> = (0..n)
        .map(|i| -6.0 + 12.0 * i as f64 / (n - 1) as f64)
        .collect();
    let xt = Tensor::from_vec(x.clone(), (n, 1), &dev).unwrap();
    let rt = Tensor::from_vec(raw.clone(), (n, 1, 23), &dev).unwrap();
    let (y, ld) = nn::rq_forward(&xt, &rt).unwrap();
    let (xb, ldi) = nn::rq_inverse(&y, &rt).unwrap();
    let (y, ld, xb, ldi): (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>) = (
        y.flatten_all().unwrap().to_vec1().unwrap(),
        ld.flatten_all().unwrap().to_vec1().unwrap(),
        xb.flatten_all().unwrap().to_vec1().unwrap(),
        ldi.flatten_all().unwrap().to_vec1().unwrap(),
    );
    let (mut e_ref, mut e_inv) = (0.0f64, 0.0f64);
    for i in 0..n {
        let (yr, ldr) = fl::rq_forward(x[i], &raw[i * 23..(i + 1) * 23], 8, 5.0);
        e_ref = e_ref.max((y[i] - yr).abs()).max((ld[i] - ldr).abs());
        e_inv = e_inv.max((xb[i] - x[i]).abs()).max((ld[i] + ldi[i]).abs());
    }
    assert!(e_ref < 1e-12, "tensor vs FP64 reference {e_ref}");
    assert!(e_inv < 1e-10, "inverse {e_inv}");
}

/// Deterministic standard-normal draws (splitmix64 + Box-Muller): the check must not depend on luck.
fn normals(n: usize, seed: u64, scale: f64) -> Vec<f64> {
    let mut x = seed;
    let mut next = || {
        x = x.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = x;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        ((z ^ (z >> 31)) >> 11) as f64 / (1u64 << 53) as f64
    };
    (0..n)
        .map(|_| {
            let (u1, u2) = (next().max(1e-300), next());
            scale * (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
        })
        .collect()
}

#[test]
fn flow_proposal_density_is_exact() {
    let dev = Device::Cpu;
    let dim = 3;
    let p = Params::init(dim, 1, DType::F64, &dev).unwrap();
    // move the last layers off the identity
    for (k, v) in &p.vars {
        if k.ends_with("W3") {
            let n: usize = v.dims().iter().product();
            let noise =
                Tensor::from_vec(normals(n, 20261107 + n as u64, 0.3), v.dims(), &dev).unwrap();
            let t = (v.as_tensor() + noise).unwrap();
            v.set(&t).unwrap();
        }
    }
    let ctx = Tensor::from_vec(
        (0..64)
            .map(|i| -1.0 + 2.0 * i as f64 / 63.0)
            .collect::<Vec<f64>>(),
        (1, 64),
        &dev,
    )
    .unwrap()
    .broadcast_as((64, 64))
    .unwrap()
    .contiguous()
    .unwrap();
    let g = Tensor::from_vec(
        (0..16).map(|i| i as f64 / 15.0).collect::<Vec<f64>>(),
        (1, 16),
        &dev,
    )
    .unwrap()
    .broadcast_as((64, 16))
    .unwrap()
    .contiguous()
    .unwrap();
    let eps = Tensor::from_vec(normals(64 * dim, 7, 1.0), (64, dim), &dev).unwrap();
    let (z, lq) = nn::sample_from(&p, &eps, &ctx, &g).unwrap();
    let lp = nn::log_prob(&p, &z, &ctx, &g).unwrap();
    let d = (lq - lp)
        .unwrap()
        .abs()
        .unwrap()
        .max(0)
        .unwrap()
        .to_scalar::<f64>()
        .unwrap();
    assert!(d < 1e-9, "sample log q vs log_prob: {d}");
}

#[test]
fn tasks_train_benchmark_smoke() {
    std::env::set_var(
        "IGNIS_CONFIGS",
        concat!(env!("CARGO_MANIFEST_DIR"), "/../../configs"),
    );
    use ignis_core::expert::{Expert, FitContext, FitData};
    let rows = ignis_acme::synthetic::default_rows("normal", 1);
    let mut groups: Vec<String> = rows.iter().map(|r| r.group_id.clone()).collect();
    groups.sort();
    groups.dedup();
    let split = ignis_core::contracts::Split {
        id: "s".into(),
        cohort_id: "c".into(),
        group_key: "session".into(),
        rule: "fixture".into(),
        seed: 0,
        run_to_group: rows
            .iter()
            .map(|r| (r.run_id.clone(), r.group_id.clone()))
            .collect(),
        train: groups.clone(),
        validation: vec![],
        test: vec![],
        frozen_at: "x".into(),
    };
    let views: Vec<_> = rows
        .into_iter()
        .map(ignis_core::views::View::Acme)
        .collect();
    let dir = std::env::temp_dir().join(format!("ignis-accel-smoke-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let mut ctx = FitContext {
        profile: serde_json::from_value(json!({"name": "t", "device": "cpu"})).unwrap(),
        pool: rayon::ThreadPoolBuilder::new().build().unwrap(),
        seed: 0,
        registry_root: dir.clone(),
        checkpoint_dir: dir.join("ck"),
        resume: None,
        job_id: None,
    };
    let e = ignis_acme::expert::AcmeExpert::new("normal", "hazard").unwrap();
    let rel = e
        .fit(
            &FitData {
                views: &views,
                split: &split,
                cohort_id: "c",
                source_manifest_hash: "syn",
                synthetic: true,
            },
            &json!({"inference": "map"}),
            &mut ctx,
        )
        .unwrap();
    let loaded = ignis_core::registry::load(&dir, &rel.id).unwrap();
    let path = dir.join("tasks.npz");
    let t = std::time::Instant::now();
    let m =
        ignis_accel::tasks::generate(&loaded, 2000, &path, 0, &json!({"stop_choices_s": [60.0]}))
            .unwrap();
    eprintln!("ACME tasks: 2000 in {:.3} s", t.elapsed().as_secs_f64());
    assert_eq!(m["latent_dim"], 2);
    let (arr, manifest) = ignis_accel::tasks::load_tasks(&path).unwrap();
    let cfg = json!({"precision": "fp32", "steps": 150, "batch_size": 128, "lr": 1e-3, "clip_norm": 1.0, "validation_fraction": 0.1, "seed": 0});
    let res = ignis_accel::train::train(&arr, 2, &cfg, false).unwrap();
    eprintln!("flow training: {} steps (batch 128, FP32 CPU) in {:.2} s = {:.1} steps/s; nll {:.3} -> {:.3}, validation {:.3}",
        res.history.len(), res.seconds, res.history.len() as f64 / res.seconds, res.history[0], res.history.last().unwrap(), res.validation_nll);
    assert!(res.history.last().unwrap() < &res.history[0]);
    let arel = ignis_accel::train::save(&dir, &res, &manifest, &cfg).unwrap();
    let acc = ignis_core::registry::load(&dir, &arel.id).unwrap();
    let out =
        ignis_accel::benchmark::acme_benchmark(&loaded, &arr, Some(&acc), 10, 256, 0.1, 0).unwrap();
    eprintln!("benchmark summary: {}", out["summary"]);
    assert!(out["summary"]["flow_ess_fraction"].as_f64().unwrap() > 0.0);
    assert!(out["summary"]["prior_abs_error"].as_f64().unwrap() < 0.5);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn bass_tasks_smoke() {
    std::env::set_var(
        "IGNIS_CONFIGS",
        concat!(env!("CARGO_MANIFEST_DIR"), "/../../configs"),
    );
    use ignis_core::expert::{Expert, FitContext, FitData};
    let seqs = ignis_bass::synthetic::synthetic_sequences(3, 1, 4);
    let groups: Vec<String> = seqs.iter().map(|s| s.group_id.clone()).collect();
    let split = ignis_core::contracts::Split {
        id: "s".into(),
        cohort_id: "c".into(),
        group_key: "specimen".into(),
        rule: "fixture".into(),
        seed: 0,
        run_to_group: seqs
            .iter()
            .map(|s| (s.run_id.clone(), s.group_id.clone()))
            .collect(),
        train: groups,
        validation: vec![],
        test: vec![],
        frozen_at: "x".into(),
    };
    let views: Vec<_> = seqs
        .into_iter()
        .map(ignis_core::views::View::Bass)
        .collect();
    let dir = std::env::temp_dir().join(format!("ignis-accel-bass-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let mut ctx = FitContext {
        profile: serde_json::from_value(json!({"name": "t", "device": "cpu"})).unwrap(),
        pool: rayon::ThreadPoolBuilder::new().build().unwrap(),
        seed: 0,
        registry_root: dir.clone(),
        checkpoint_dir: dir.join("ck"),
        resume: None,
        job_id: None,
    };
    let e = ignis_bass::expert::BassOperatorExpert::new().unwrap();
    let rel = e
        .fit(
            &FitData {
                views: &views,
                split: &split,
                cohort_id: "c",
                source_manifest_hash: "syn",
                synthetic: true,
            },
            &json!({"train": {"steps": 5, "n_members": 2}}),
            &mut ctx,
        )
        .unwrap();
    let loaded = ignis_core::registry::load(&dir, &rel.id).unwrap();
    let path = dir.join("bass_tasks.npz");
    let t = std::time::Instant::now();
    let m = ignis_accel::tasks::generate(
        &loaded,
        50,
        &path,
        0,
        &json!({"frame_dt_s": 1.0, "max_frames": 8}),
    )
    .unwrap();
    eprintln!("BASS tasks: 50 in {:.3} s", t.elapsed().as_secs_f64());
    assert_eq!(m["latent_dim"], 12);
    let (arr, _) = ignis_accel::tasks::load_tasks(&path).unwrap();
    assert_eq!(arr["tokens"].0, vec![50, 64, 24]);
    let res =
        ignis_accel::train::train(&arr, 12, &json!({"steps": 3, "batch_size": 16}), false).unwrap();
    assert!(res.validation_nll.is_finite());
    let _ = std::fs::remove_dir_all(&dir);
}
