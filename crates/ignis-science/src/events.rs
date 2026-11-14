//! Event and support accounting after a cutoff (V3.1 §6.5–6.6).
//!
//! For a particle (global draw + locals) with exit-from-support time B, survival S and cause hazards
//! lambda_k, conditional on being active and in support at the cutoff tau, for t >= tau:
//!
//! ```text
//! A_p(t)  = 1[t < B] S(t)/S(tau)                         active and supported
//! U_p(t)  = 1[t >= B] S(B)/S(tau)                        unresolved: model limit reached first
//! F_kp(t) = int_tau^{min(t,B)} S(u) lambda_k(u) du / S(tau)
//! ```
//!
//! A_p + U_p + sum_k F_kp = 1 identically. Averaging with normalised particle weights keeps the
//! identity. U is not a physical cause; particles that leave support are never dropped.

pub const ACTIVE: &str = "active_supported";
pub const UNRESOLVED: &str = "unresolved";

pub fn categories(causes: &[&str]) -> Vec<String> {
    let mut v = vec![ACTIVE.to_string()];
    v.extend(causes.iter().map(|s| s.to_string()));
    v.push(UNRESOLVED.into());
    v
}

/// One particle's forecast quantities on a time grid (T points, K causes).
#[derive(Debug, Clone, Default)]
pub struct ParticleEvents {
    pub before_b: Vec<bool>,
    pub s_rel: Vec<f64>,
    pub s_b_rel: f64,
    /// (K, T) row-major.
    pub f_rel: Vec<Vec<f64>>,
}

/// (K+2, T) table [active, causes..., unresolved] of one particle.
pub fn per_particle_partition(p: &ParticleEvents) -> Vec<Vec<f64>> {
    let nt = p.s_rel.len();
    let mut out = Vec::with_capacity(p.f_rel.len() + 2);
    out.push(
        (0..nt)
            .map(|j| if p.before_b[j] { p.s_rel[j] } else { 0.0 })
            .collect(),
    );
    out.extend(p.f_rel.iter().cloned());
    out.push(
        (0..nt)
            .map(|j| if p.before_b[j] { 0.0 } else { p.s_b_rel })
            .collect(),
    );
    out
}

/// Weighted (K+2, T) table; weights must sum to one.
pub fn support_partition(weights: &[f64], parts: &[ParticleEvents]) -> Vec<Vec<f64>> {
    let Some(first) = parts.first() else {
        return vec![];
    };
    let (k, nt) = (first.f_rel.len() + 2, first.s_rel.len());
    let mut table = vec![vec![0.0; nt]; k];
    for (w, p) in weights.iter().zip(parts) {
        if *w == 0.0 {
            continue;
        }
        for (row, prow) in table.iter_mut().zip(per_particle_partition(p)) {
            for (a, b) in row.iter_mut().zip(prow) {
                *a += w * b;
            }
        }
    }
    table
}

/// max_t |sum_k table[k, t] - 1| — reported in artifact diagnostics.
pub fn partition_error(table: &[Vec<f64>]) -> f64 {
    let Some(r0) = table.first() else { return 0.0 };
    (0..r0.len())
        .map(|j| (table.iter().map(|r| r[j]).sum::<f64>() - 1.0).abs())
        .fold(0.0, f64::max)
}

/// Weights for the diameter conditioned on 'still active and in support at t' (§6.6), (P, T)
/// column-normalised, plus the conditioned mass per column. A column with zero mass stays zero and
/// the caller reports it as unresolved instead of drawing a band.
pub fn conditioned_trajectory_weights(
    weights: &[f64],
    parts: &[ParticleEvents],
) -> (Vec<Vec<f64>>, Vec<f64>) {
    let nt = parts.first().map_or(0, |p| p.s_rel.len());
    let mut raw: Vec<Vec<f64>> = weights
        .iter()
        .zip(parts)
        .map(|(w, p)| {
            (0..nt)
                .map(|j| if p.before_b[j] { w * p.s_rel[j] } else { 0.0 })
                .collect()
        })
        .collect();
    let tot: Vec<f64> = (0..nt).map(|j| raw.iter().map(|r| r[j]).sum()).collect();
    for r in raw.iter_mut() {
        for j in 0..nt {
            r[j] = if tot[j] > 0.0 { r[j] / tot[j] } else { 0.0 };
        }
    }
    (raw, tot)
}
