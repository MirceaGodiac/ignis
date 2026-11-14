//! OU/Kalman Student-t log likelihood equals the dense reference on an irregular-time case;
//! the fixture release tensors read from npz reproduce the Python tensor hashes.

use ignis_core::contracts::ModelRelease;
use ignis_core::kernels::{ou_kalman_stats, student_t_logpdf, student_t_ou_loglik, OuParams, NU};
use ignis_core::npz::{read_npz, tensor_sha256};

/// Dense reference: build Σ, Cholesky, log|Σ| and eᵀΣ⁻¹e directly.
fn dense(t: &[f64], e: &[f64], p: OuParams) -> (f64, f64) {
    let n = t.len();
    let mut a = vec![vec![0.0; n]; n];
    for i in 0..n {
        for j in 0..n {
            let nug = if i == j { p.sigma_n.powi(2) } else { 0.0 };
            a[i][j] = p.sigma_c.powi(2) * (-(t[i] - t[j]).abs() / p.ell).exp() + nug;
        }
    }
    let mut l = vec![vec![0.0; n]; n];
    for i in 0..n {
        for j in 0..=i {
            let s: f64 = (0..j).map(|k| l[i][k] * l[j][k]).sum();
            if i == j {
                l[i][i] = (a[i][i] - s).sqrt();
            } else {
                l[i][j] = (a[i][j] - s) / l[j][j];
            }
        }
    }
    let logdet: f64 = (0..n).map(|i| 2.0 * l[i][i].ln()).sum();
    let mut z = vec![0.0; n];
    for i in 0..n {
        let s: f64 = (0..i).map(|k| l[i][k] * z[k]).sum();
        z[i] = (e[i] - s) / l[i][i];
    }
    (logdet, z.iter().map(|x| x * x).sum())
}

#[test]
fn kalman_equals_dense_on_irregular_times() {
    let t = [0.0, 0.013, 0.05, 0.051, 0.2, 0.47, 0.48, 1.3, 2.0];
    let e = [0.3, -0.1, 0.25, 0.2, -0.4, 0.05, 0.1, -0.2, 0.6];
    for p in [
        OuParams {
            sigma_c: 0.7,
            ell: 0.3,
            sigma_n: 0.1,
        },
        OuParams {
            sigma_c: 1.3,
            ell: 2.5,
            sigma_n: 0.02,
        },
        OuParams {
            sigma_c: 0.2,
            ell: 0.01,
            sigma_n: 0.5,
        },
    ] {
        let (ld, dl) = ou_kalman_stats(&t, &e, p).unwrap();
        let (ld_ref, dl_ref) = dense(&t, &e, p);
        assert!((ld - ld_ref).abs() < 1e-9, "logdet {ld} vs {ld_ref}");
        assert!(
            (dl - dl_ref).abs() < 1e-9 * dl_ref.max(1.0),
            "delta {dl} vs {dl_ref}"
        );
        let ll = student_t_ou_loglik(&t, &e, p).unwrap();
        let ll_ref = student_t_logpdf(t.len(), ld_ref, dl_ref, NU);
        assert!((ll - ll_ref).abs() < 1e-9, "loglik {ll} vs {ll_ref}");
    }
}

#[test]
fn univariate_case_matches_closed_form() {
    // n = 1, nu = 4: density = Γ(5/2) / (Γ(2) sqrt(4π) s) * (1 + x²/(4 s²))^(-5/2); Γ(5/2) = 3√π/4.
    let (s, x) = (0.8_f64, 0.37_f64);
    let p = OuParams {
        sigma_c: s,
        ell: 1.0,
        sigma_n: 0.0,
    };
    let ll = student_t_ou_loglik(&[0.0], &[x], p).unwrap();
    let g52 = 0.75 * std::f64::consts::PI.sqrt();
    let reference = (g52 / ((4.0 * std::f64::consts::PI).sqrt() * s)).ln()
        - 2.5 * (1.0 + x * x / (4.0 * s * s)).ln();
    assert!((ll - reference).abs() < 1e-12, "{ll} vs {reference}");
}

#[test]
fn duplicate_timestamps_are_rejected() {
    let p = OuParams {
        sigma_c: 1.0,
        ell: 1.0,
        sigma_n: 0.1,
    };
    assert!(student_t_ou_loglik(&[0.0, 1.0, 1.0], &[0.0, 0.1, 0.2], p).is_err());
}

#[test]
fn fixture_release_tensors_verify() {
    let models = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../ignis-server/fixtures/store/models");
    let mut n = 0;
    for dir in std::fs::read_dir(&models).unwrap() {
        let dir = dir.unwrap().path();
        let rel: ModelRelease =
            serde_json::from_str(&std::fs::read_to_string(dir.join("release.json")).unwrap())
                .unwrap();
        let tensors = read_npz(&dir.join("params.npz")).unwrap();
        for e in &rel.tensors {
            let a = &tensors[&e.name];
            assert_eq!(a.dtype.numpy_name(), e.dtype);
            assert_eq!(
                a.shape.iter().map(|&d| d as i64).collect::<Vec<_>>(),
                e.shape
            );
            assert_eq!(tensor_sha256(a), e.sha256, "{}", e.name);
            n += 1;
        }
    }
    assert!(n > 0);
}
