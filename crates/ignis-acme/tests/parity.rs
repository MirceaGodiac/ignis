//! Parity with the frozen Python/JAX reference (tools/parity/acme_parity.py), at fixed parameters.
use ignis_acme::model::*;
use ignis_core::views::AcmeEventRow;
use serde_json::Value;

fn fixture() -> Value {
    serde_json::from_str(
        &std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/acme_parity.json"
        ))
        .unwrap(),
    )
    .unwrap()
}
fn vecf(v: &Value) -> Vec<f64> {
    v.as_array()
        .unwrap()
        .iter()
        .flat_map(|x| {
            if x.is_array() {
                vecf(x)
            } else {
                vec![x.as_f64().unwrap()]
            }
        })
        .collect()
}
pub fn spec_of(s: &Value) -> AcmeSpec {
    AcmeSpec {
        causes: s["causes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c.as_str().unwrap().to_string())
            .collect(),
        bin_lower_edges_s: vecf(&s["bin_lower_edges_s"]),
        t_ref_s: s["t_ref_s"].as_f64().unwrap(),
        n_features: s["n_features"].as_u64().unwrap() as usize,
        linear: s["linear"].as_bool().unwrap(),
        neural: s["neural"].as_bool().unwrap(),
        frailty: s["frailty"].as_bool().unwrap(),
        hidden: s["hidden"].as_u64().unwrap() as usize,
        cells_per_bin: s["cells_per_bin"].as_u64().unwrap() as usize,
    }
}
fn rel(a: f64, b: f64) -> f64 {
    (a - b).abs() / b.abs().max(1e-300)
}

#[test]
fn gauss_hermite_matches_numpy() {
    let fx = fixture();
    for (n, v) in fx["gh"].as_object().unwrap() {
        let (x, w) = ignis_science::gauss_hermite::hermite_rule(n.parse().unwrap());
        let (xr, wr) = (vecf(&v[0]), vecf(&v[1]));
        for i in 0..x.len() {
            assert!(
                (x[i] - xr[i]).abs() < 1e-14 * xr[i].abs().max(1.0),
                "n={n} x"
            );
            assert!(rel(w[i], wr[i]) < 1e-12, "n={n} w {} {}", w[i], wr[i]);
        }
    }
}

#[test]
fn likelihood_gradient_and_forecast_parity() {
    let fx = fixture();
    let rows: Vec<AcmeEventRow> = fx["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| serde_json::from_value(r.clone()).unwrap())
        .collect();
    let refs: Vec<&AcmeEventRow> = rows.iter().collect();
    let c: Vec<Vec<f64>> = fx["c"].as_array().unwrap().iter().map(vecf).collect();
    let mut worst: f64 = 0.0;
    for case in fx["cases"].as_array().unwrap() {
        let spec = spec_of(&case["spec"]);
        let lay = Layout::new(&spec);
        let mut p = vec![0.0; lay.len];
        for (name, _, off) in &lay.blocks {
            let v = vecf(&case["params"][name]);
            p[*off..off + v.len()].copy_from_slice(&v);
        }
        let enc = encode_rows(&refs, &c, &spec, None).unwrap();
        let q = quadrature(&spec, 12);
        let rl = row_loglik_nodes(&p, &spec, &lay, &enc, &q);
        let rr = vecf(&case["row_loglik_nodes"]);
        for i in 0..rl.len() {
            worst = worst.max(rel(rl[i], rr[i]));
            assert!(rel(rl[i], rr[i]) < 1e-11, "row ll {i}: {} {}", rl[i], rr[i]);
        }
        let ml = marginal_loglik(&p, &spec, &lay, &enc, &q, None);
        assert!(rel(ml, case["marginal_loglik"].as_f64().unwrap()) < 1e-11);
        let pr = prior_settings(&spec, &Value::Null);
        assert!(
            rel(
                log_prior(&p, &spec, &lay, &pr, None),
                case["log_prior"].as_f64().unwrap()
            ) < 1e-12
        );
        let mut g = vec![0.0; lay.len];
        let v = log_posterior(&p, &spec, &lay, &pr, &enc, &q, Some(&mut g));
        assert!(rel(v, case["log_posterior"].as_f64().unwrap()) < 1e-11);
        let gmax = vecf_all(&case["grad"]);
        let scale = gmax.iter().fold(0.0f64, |a, b| a.max(b.abs()));
        for (name, _, off) in &lay.blocks {
            let gr = vecf(&case["grad"][name]);
            for i in 0..gr.len() {
                let e = (g[off + i] - gr[i]).abs() / scale;
                worst = worst.max(e);
                assert!(
                    e < 1e-10,
                    "{name}[{i}] {} vs {} ({:?})",
                    g[off + i],
                    gr[i],
                    spec
                );
            }
        }
        // finite-difference check on the analytic gradient
        for i in 0..lay.len {
            let h = 1e-6 * p[i].abs().max(1.0);
            let (mut a, mut b) = (p.clone(), p.clone());
            a[i] += h;
            b[i] -= h;
            let fd = (log_posterior(&a, &spec, &lay, &pr, &enc, &q, None)
                - log_posterior(&b, &spec, &lay, &pr, &enc, &q, None))
                / (2.0 * h);
            assert!(
                (fd - g[i]).abs() < 1e-5 * scale.max(1.0),
                "fd {i}: {fd} {}",
                g[i]
            );
        }
        // speed of value + gradient on the same data as the JAX number
        let t = std::time::Instant::now();
        let mut n = 0;
        while t.elapsed().as_secs_f64() < 1.0 {
            log_posterior(&p, &spec, &lay, &pr, &enc, &q, Some(&mut g));
            n += 1;
        }
        eprintln!(
            "K={} neural={} frailty={}: rust value+grad/s {:.0} vs jax {:.0}",
            spec.k(),
            spec.neural,
            spec.frailty,
            n as f64 / t.elapsed().as_secs_f64(),
            case["jax_value_and_grad_per_s"].as_f64().unwrap()
        );
        if let Some(f) = case.get("forecast") {
            let cc = vecf(&f["c"]);
            let times = vecf(&f["times"]);
            let shift = f["draw_shift"].as_f64().unwrap();
            let draws = vec![p.clone(), p.iter().map(|x| x + shift).collect::<Vec<_>>()];
            for (key, res) in f["results"].as_object().unwrap() {
                let (q2, extra) = if key == "known_session_1" {
                    let mut tab = vec![];
                    for d in &draws {
                        let t = session_loglik_nodes(d, &spec, &lay, &enc, &q);
                        tab.extend_from_slice(&t[q.n()..2 * q.n()]);
                    }
                    (
                        quadrature(&spec, 12),
                        Some(normalise_session_table(&tab, &q)),
                    )
                } else {
                    (quadrature(&spec, key.parse().unwrap()), None)
                };
                let fc = predict_partition(
                    &draws,
                    &spec,
                    &lay,
                    &cc,
                    f["t_entry"].as_f64().unwrap(),
                    f["tau"].as_f64().unwrap(),
                    &times,
                    f["stop"].as_f64().unwrap(),
                    f["horizon"].as_f64().unwrap(),
                    &q2,
                    extra.as_deref(),
                );
                for (name, mine) in [
                    ("active", &fc.part.active),
                    ("F", &fc.part.f),
                    ("atom", &fc.part.atom),
                    ("unresolved", &fc.part.unresolved),
                    ("global", &fc.global),
                ] {
                    let r = vecf(&res[name]);
                    for i in 0..r.len() {
                        let e = (mine[i] - r[i]).abs();
                        worst = worst.max(e);
                        assert!(e < 1e-12, "{key} {name}[{i}] {} {}", mine[i], r[i]);
                    }
                }
                assert!(fc.part.max_partition_error() < 1e-12);
            }
        }
    }
    eprintln!("worst parity error {worst:e}");
}

fn vecf_all(v: &Value) -> Vec<f64> {
    v.as_object().unwrap().values().flat_map(vecf).collect()
}

/// V3.1 §26: 24 vs 48 GH nodes agree; the prefix re-weights global draws jointly; known-session
/// tables normalised per draw leave the global weights at exactly 1/G when τ = entry (review-B fix 1).
#[test]
fn gauss_hermite_joint_normalisation_and_known_session() {
    let fx = fixture();
    let case = fx["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| {
            c["spec"]["frailty"].as_bool().unwrap()
                && c["spec"]["causes"].as_array().unwrap().len() == 2
        })
        .unwrap();
    let spec = spec_of(&case["spec"]);
    let lay = Layout::new(&spec);
    let mut p = vec![0.0; lay.len];
    for (name, _, off) in &lay.blocks {
        let v = vecf(&case["params"][name]);
        p[*off..off + v.len()].copy_from_slice(&v);
    }
    let mut p2 = p.clone();
    for b in 0..spec.k() * spec.n_bins() {
        p2[lay.a + b] += 1.2;
    }
    let draws = vec![p, p2];
    let c = vec![0.3, -0.5, 0.1, 0.0, 0.2, -0.1, 0.0, 1.0];
    let times = [12.0, 20.0, 30.0];
    let f24 = predict_partition(
        &draws,
        &spec,
        &lay,
        &c,
        0.0,
        10.0,
        &times,
        25.0,
        40.0,
        &quadrature(&spec, 24),
        None,
    );
    let f48 = predict_partition(
        &draws,
        &spec,
        &lay,
        &c,
        0.0,
        10.0,
        &times,
        25.0,
        40.0,
        &quadrature(&spec, 48),
        None,
    );
    let d = (f24.part.f[4] - f48.part.f[4]).abs();
    eprintln!(
        "event probability 24 nodes {:.15} 48 nodes {:.15} (|diff| {d:e}); global weights {:?}",
        f24.part.f[4], f48.part.f[4], f48.global
    );
    assert!(d < 1e-9);
    assert!(
        (f48.global.iter().sum::<f64>() - 1.0).abs() < 1e-14 && (f48.global[0] - 0.5).abs() > 0.05
    );
    assert!((f48.w.iter().sum::<f64>() - 1.0).abs() < 1e-14);
    assert!(f24.part.max_partition_error() < 1e-12 && f48.part.max_partition_error() < 1e-12);
    // known session with τ = entry: no prefix information, global weights stay 1/G
    let q = quadrature(&spec, 12);
    let raw: Vec<f64> = (0..2 * q.n())
        .map(|i| ((i * 37) % 11) as f64 - 5.0)
        .collect();
    let tab = normalise_session_table(&raw, &q);
    let ks = predict_partition(
        &draws,
        &spec,
        &lay,
        &c,
        0.0,
        0.0,
        &[5.0, 20.0],
        f64::INFINITY,
        40.0,
        &q,
        Some(&tab),
    );
    assert!(
        ks.global.iter().all(|g| (g - 0.5).abs() < 1e-12),
        "{:?}",
        ks.global
    );
}
