//! Synthetic BASS front sequences for smoke tests, benchmarks and statistical tasks. Labelled
//! synthetic; never a NASA observation. A smooth single-valued front advancing at a
//! condition-dependent speed with mild curvature, observed with node noise, a few unresolved samples
//! and an occluded frame. Keys are the real view keys of `configs/models/bass/operator.toml`.

use std::collections::BTreeMap;

use ignis_core::views::BassSequence;
use rand::{rngs::StdRng, RngExt, SeedableRng};
use rand_distr::StandardNormal;

#[allow(clippy::too_many_arguments)]
pub fn synthetic_sequence(
    run_id: &str,
    group_id: &str,
    seed: u64,
    n_frames: usize,
    dt_s: f64,
    n_samples: usize,
    width_m: f64,
    length_m: f64,
    noise_m: f64,
    occlude_frame: Option<usize>,
) -> BassSequence {
    let mut rng = StdRng::seed_from_u64(seed);
    let o2: f64 = rng.random_range(0.18..0.30);
    let flow: f64 = rng.random_range(0.05..0.2);
    let speed = 1.0e-3 * (1.0 + 2.0 * (o2 - 0.21)) * (1.0 + 0.5 * (flow - 0.1));
    let s: Vec<f64> = (0..n_samples)
        .map(|i| width_m * i as f64 / (n_samples - 1) as f64)
        .collect();
    let off: f64 = 0.001 * rng.sample::<f64, _>(StandardNormal);
    let t: Vec<f64> = (0..n_frames).map(|i| i as f64 * dt_s).collect();
    let mut f = vec![0.0; n_frames * n_samples];
    let mut valid = vec![true; n_frames * n_samples];
    for i in 0..n_frames {
        for j in 0..n_samples {
            let xi = s[j] / width_m;
            let f0 = 0.02 + 0.002 * (std::f64::consts::PI * xi).cos() + off;
            let shape = 1.0 + 0.1 * (std::f64::consts::PI * xi).cos();
            f[i * n_samples + j] =
                f0 + speed * t[i] * shape + noise_m * rng.sample::<f64, _>(StandardNormal);
            valid[i * n_samples + j] = rng.random::<f64>() > 0.1;
        }
    }
    let mut status = vec!["single_valued".to_string(); n_frames];
    if let Some(o) = occlude_frame.filter(|o| *o < n_frames) {
        status[o] = "occluded".into();
        valid[o * n_samples..(o + 1) * n_samples]
            .iter_mut()
            .for_each(|v| *v = false);
    }
    for i in 0..f.len() {
        if !valid[i] {
            f[i] = f64::NAN;
        }
    }
    BassSequence {
        run_id: run_id.into(),
        group_id: group_id.into(),
        t_s: t,
        s_m: s,
        f_m: f,
        valid,
        status,
        conditions: BTreeMap::from([
            ("x_o2".into(), o2),
            ("pressure".into(), 101325.0),
            ("flow_velocity".into(), flow),
            ("thickness".into(), 1.0e-4),
        ]),
        geometry: BTreeMap::from([("width".into(), width_m), ("length".into(), length_m)]),
        control_changes_s: vec![0.0],
        scale_uncertainty_rel: vec![0.005; n_frames],
        time_origin: "synthetic".into(),
    }
}

pub fn synthetic_sequences(n_groups: usize, runs_per_group: usize, seed: u64) -> Vec<BassSequence> {
    let mut out = vec![];
    for g in 0..n_groups {
        for r in 0..runs_per_group {
            out.push(synthetic_sequence(
                &format!("bass-g{g}-r{r}"),
                &format!("specimen-{g}"),
                seed * 1000 + (g * 10 + r) as u64,
                8,
                1.0,
                40,
                0.04,
                0.1,
                5e-5,
                Some(4),
            ));
        }
    }
    out
}
