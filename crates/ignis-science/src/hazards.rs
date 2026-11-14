//! Cause-specific competing-risk accounting over piecewise cells (V3.1 §6.5, §8.4, §8.5). FP64.
//!
//! * Time is cut into *cells*. Each baseline bin [e_j, e_{j+1}) that intersects a segment [t0, t1] is
//!   split into `m` equal cells, so a cell never straddles a bin edge (the piecewise baseline is exact).
//! * Within a cell the hazards are evaluated once at the cell midpoint and held constant. For constant
//!   hazards on a cell of length Δt with Λ = Σ_k λ_k (V3.1 §8.5):
//!   S_new = S_old · exp(−Λ Δt),  ΔF_k = S_old · (λ_k / Λ) · (1 − exp(−Λ Δt)),
//!   evaluated with `expm1` and the stable Λ→0 limit ΔF_k → S_old λ_k Δt. Hence S + Σ_k F_k = 1
//!   exactly for the discretised model; refinement (m vs 2m) controls only the neural time variation.
//! * A *known cause set* A on (L, R] contributes Σ_{k∈A} ∫_L^R S λ_k du, which is NOT the all-cause
//!   probability S(L) − S(R) unless A contains every cause (§6.5).
//! * A deterministic scheduled operator stop at t_stop is a probability ATOM, never a hazard:
//!   P(T = t_stop, J = O | alive at τ) = S(t_stop⁻)/S(τ); fed-phase survival after the stop is 0.
//!
//! Cells of zero width (bins outside the segment) contribute exactly nothing, so [`Cells::nonzero`]
//! drops them; values and gradients are unchanged.

/// Cells covering a segment: midpoints, widths and baseline bin index.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Cells {
    pub t_mid: Vec<f64>,
    pub dt: Vec<f64>,
    pub bin: Vec<usize>,
}

impl Cells {
    pub fn len(&self) -> usize {
        self.dt.len()
    }
    pub fn is_empty(&self) -> bool {
        self.dt.is_empty()
    }
    /// Drop zero-width cells (exactly zero contribution).
    pub fn nonzero(self) -> Cells {
        let mut out = Cells::default();
        for i in 0..self.dt.len() {
            if self.dt[i] > 0.0 {
                out.t_mid.push(self.t_mid[i]);
                out.dt.push(self.dt[i]);
                out.bin.push(self.bin[i]);
            }
        }
        out
    }
}

fn clip(x: f64, lo: f64, hi: f64) -> f64 {
    // jnp.clip(x, lo, hi) = min(max(x, lo), hi)
    x.max(lo).min(hi)
}

/// Cells covering [t0, t1] (t1 ≥ t0) aligned with baseline bin `edges` (last edge may be +∞);
/// `n_bins · m` cells in Python order, zero width outside [t0, t1].
pub fn cell_grid(t0: f64, t1: f64, edges: &[f64], m: usize) -> Cells {
    let nb = edges.len() - 1;
    let mut c = Cells {
        t_mid: Vec::with_capacity(nb * m),
        dt: Vec::with_capacity(nb * m),
        bin: Vec::with_capacity(nb * m),
    };
    for j in 0..nb {
        let lo = clip(edges[j], t0, t1);
        let hi = clip(edges[j + 1], t0, t1);
        let width = (hi - lo) / m as f64;
        for k in 0..m {
            let left = lo + width * k as f64;
            c.t_mid.push(left + 0.5 * width);
            c.dt.push(width);
            c.bin.push(j);
        }
    }
    c
}

/// Bin index of a time: clip(searchsorted(edges, t, 'right') − 1, 0, n_bins − 1).
pub fn bin_of(t: f64, edges: &[f64]) -> usize {
    let nb = edges.len() - 1;
    let pos = edges.partition_point(|e| *e <= t);
    pos.saturating_sub(1).min(nb - 1)
}

/// (1 − exp(−Λ Δt)) / Λ with the stable Λ→0 limit Δt (1 − ΛΔt/2).
#[inline]
pub fn one_minus_exp_over_rate(rate: f64, dt: f64) -> f64 {
    let x = rate * dt;
    if x < 1e-8 {
        dt * (1.0 - 0.5 * x)
    } else {
        -(-x).exp_m1() / rate
    }
}

/// d/dΛ of [`one_minus_exp_over_rate`] (stable series for small ΛΔt; the Λ→0 branch matches the
/// derivative of the limit form exactly).
#[inline]
pub fn d_one_minus_exp_over_rate(rate: f64, dt: f64) -> f64 {
    let x = rate * dt;
    if x < 1e-8 {
        -0.5 * dt * dt
    } else if x < 1e-2 {
        // g'(Λ) = Δt² h(x), h(x) = −Σ_{n≥0} (−x)^n (n+1)/(n+2)!
        let mut term = 0.5; // (n+1)/(n+2)! at n = 0
        let mut s = 0.0;
        let mut p = 1.0;
        let mut fact = 2.0; // (n+2)!
        for n in 0..12 {
            s += p * term;
            p *= -x;
            fact *= (n + 3) as f64;
            term = (n + 2) as f64 / fact;
        }
        -dt * dt * s
    } else {
        let e = (-x).exp();
        (dt * e - (-(-x).exp_m1()) / rate) / rate
    }
}

/// Accumulate over consecutive cells starting from S = 1. `lam` (C × K row-major) cause-specific
/// hazards, `dt` (C). Writes the cause-specific incidence F (K) over the cells relative to survival at
/// the start of the first cell and returns log S_end.
pub fn competing_increments(lam: &[f64], dt: &[f64], k: usize, f_out: &mut [f64]) -> f64 {
    f_out.iter_mut().for_each(|v| *v = 0.0);
    let mut cum = 0.0;
    for c in 0..dt.len() {
        let l = &lam[c * k..(c + 1) * k];
        let rate: f64 = l.iter().sum();
        cum += rate * dt[c];
        let excl = cum - rate * dt[c];
        let factor = (-excl).exp() * one_minus_exp_over_rate(rate, dt[c]);
        for j in 0..k {
            f_out[j] += factor * l[j];
        }
    }
    -cum
}

/// Σ_cells Σ_k λ_k Δt (all-cause cumulative hazard over the cells).
pub fn cumulative_hazard(lam: &[f64], dt: &[f64], k: usize) -> f64 {
    (0..dt.len())
        .map(|c| lam[c * k..(c + 1) * k].iter().sum::<f64>() * dt[c])
        .sum()
}

/// Hazard function: (t_mid (C), bin (C)) → writes (C × K) hazards.
pub trait HazardFn {
    fn k(&self) -> usize;
    fn eval(&self, t_mid: &[f64], bin: &[usize], out: &mut [f64]);
}

/// Constant cause-specific hazards (tests and analytic references).
pub struct ConstantHazard(pub Vec<f64>);

impl HazardFn for ConstantHazard {
    fn k(&self) -> usize {
        self.0.len()
    }
    fn eval(&self, t_mid: &[f64], _bin: &[usize], out: &mut [f64]) {
        let k = self.0.len();
        for c in 0..t_mid.len() {
            out[c * k..(c + 1) * k].copy_from_slice(&self.0);
        }
    }
}

/// (log S(t1)/S(t0), F_k over (t0, t1] relative to S(t0)).
pub fn segment(haz: &dyn HazardFn, t0: f64, t1: f64, edges: &[f64], m: usize) -> (f64, Vec<f64>) {
    let k = haz.k();
    let cells = cell_grid(t0, t1, edges, m).nonzero();
    let mut lam = vec![0.0; cells.len() * k];
    haz.eval(&cells.t_mid, &cells.bin, &mut lam);
    let mut f = vec![0.0; k];
    let ls = competing_increments(&lam, &cells.dt, k, &mut f);
    (ls, f)
}

/// P(L < T ≤ R, J ∈ A | T > t_entry) = Σ_{k∈A} ∫_L^R S λ_k du / S(t_entry).
pub fn known_cause_interval(
    haz: &dyn HazardFn,
    t_entry: f64,
    l: f64,
    r: f64,
    cause_mask: &[f64],
    edges: &[f64],
    m: usize,
) -> f64 {
    let (log_s_l, _) = segment(haz, t_entry, l, edges, m);
    let (_, f) = segment(haz, l, r, edges, m);
    log_s_l.exp() * f.iter().zip(cause_mask).map(|(a, b)| a * b).sum::<f64>()
}

/// S(L)/S(entry) − S(R)/S(entry): probability that *any* cause occurs in (L, R].
pub fn all_cause_interval(
    haz: &dyn HazardFn,
    t_entry: f64,
    l: f64,
    r: f64,
    edges: &[f64],
    m: usize,
) -> f64 {
    let (log_s_l, _) = segment(haz, t_entry, l, edges, m);
    let (log_s_lr, _) = segment(haz, l, r, edges, m);
    log_s_l.exp() * -log_s_lr.exp_m1()
}

/// Conditional partition at cutoff τ for output times (sorted, ≥ τ) — V3.1 §6.6 + §8.5.
/// A + U + Σ_k F_k + atom = 1 for every t (to rounding).
#[derive(Debug, Clone, PartialEq)]
pub struct Partition {
    pub k: usize,
    /// A(t) = 1_{t<min(stop,B)} S(t)/S(τ)
    pub active: Vec<f64>,
    /// (T × K) continuous cause-specific incidence up to min(t, stop, B)
    pub f: Vec<f64>,
    /// operator-stop atom mass S(stop⁻)/S(τ) once t ≥ stop (if stop ≤ B)
    pub atom: Vec<f64>,
    /// U(t) = S(B)/S(τ) once t ≥ B (if B < stop)
    pub unresolved: Vec<f64>,
}

impl Partition {
    pub fn zeros(t: usize, k: usize) -> Partition {
        Partition {
            k,
            active: vec![0.0; t],
            f: vec![0.0; t * k],
            atom: vec![0.0; t],
            unresolved: vec![0.0; t],
        }
    }
    pub fn max_partition_error(&self) -> f64 {
        (0..self.active.len())
            .map(|i| {
                let s = self.active[i]
                    + self.atom[i]
                    + self.unresolved[i]
                    + self.f[i * self.k..(i + 1) * self.k].iter().sum::<f64>();
                (s - 1.0).abs()
            })
            .fold(0.0, f64::max)
    }
}

/// Output-time segments [lo_i, hi_i] of a forecast: hi = min(t_i, min(stop, B)), lo = previous hi
/// (τ first), each as nonzero cells.
pub fn partition_cells(
    tau: f64,
    times: &[f64],
    edges: &[f64],
    m: usize,
    t_stop: f64,
    horizon: f64,
) -> Vec<Cells> {
    let end = t_stop.min(horizon);
    let mut out = Vec::with_capacity(times.len());
    let mut prev = tau;
    for &t in times {
        let hi = t.min(end);
        let lo = prev.min(hi);
        out.push(cell_grid(lo, hi, edges, m).nonzero());
        prev = hi;
    }
    out
}

/// Partition from per-segment hazards `seg_lam[i]` (C_i × K, already evaluated on `segs[i]`) scaled
/// per cause by `scale` (frailty multipliers e^{u_k}; pass ones for none). Allocation-free.
pub fn partition_from_lam(
    segs: &[Cells],
    seg_lam: &[Vec<f64>],
    scale: &[f64],
    times: &[f64],
    t_stop: f64,
    horizon: f64,
    out: &mut Partition,
) {
    let k = scale.len();
    let end = t_stop.min(horizon);
    let stop_first = t_stop <= horizon;
    let mut log_s_cum = 0.0;
    let mut fcum = [0.0f64; 8];
    assert!(k <= 8);
    fcum[..k].iter_mut().for_each(|v| *v = 0.0);
    for (i, cells) in segs.iter().enumerate() {
        let lam = &seg_lam[i];
        // competing increments of the scaled hazards over this segment
        let mut cum = 0.0;
        let mut fseg = [0.0f64; 8];
        for c in 0..cells.dt.len() {
            let mut rate = 0.0;
            for j in 0..k {
                rate += lam[c * k + j] * scale[j];
            }
            let dt = cells.dt[c];
            cum += rate * dt;
            let excl = cum - rate * dt;
            let factor = (-excl).exp() * one_minus_exp_over_rate(rate, dt);
            for j in 0..k {
                fseg[j] += factor * lam[c * k + j] * scale[j];
            }
        }
        let log_s_seg = -cum;
        let log_s_start = log_s_cum;
        log_s_cum += log_s_seg;
        let w = log_s_start.exp();
        for j in 0..k {
            fcum[j] += w * fseg[j];
            out.f[i * k + j] = fcum[j];
        }
        let s_end = log_s_cum.exp();
        let alive = times[i] < end;
        out.active[i] = if alive { s_end } else { 0.0 };
        out.atom[i] = if !alive && stop_first { s_end } else { 0.0 };
        out.unresolved[i] = if !alive && !stop_first { s_end } else { 0.0 };
    }
}

/// Conditional partition at cutoff τ for a hazard function (`times` sorted, ≥ τ).
pub fn forecast_partition(
    haz: &dyn HazardFn,
    tau: f64,
    times: &[f64],
    edges: &[f64],
    m: usize,
    t_stop: f64,
    horizon: f64,
) -> Partition {
    let k = haz.k();
    let segs = partition_cells(tau, times, edges, m, t_stop, horizon);
    let lam: Vec<Vec<f64>> = segs
        .iter()
        .map(|c| {
            let mut l = vec![0.0; c.len() * k];
            haz.eval(&c.t_mid, &c.bin, &mut l);
            l
        })
        .collect();
    let mut out = Partition::zeros(times.len(), k);
    partition_from_lam(&segs, &lam, &vec![1.0; k], times, t_stop, horizon, &mut out);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const EDGES: [f64; 4] = [0.0, 5.0, 10.0, f64::INFINITY];

    #[test]
    fn known_cause_interval_is_not_all_cause() {
        let (le, lo, l, r) = (0.03, 0.02, 4.0, 12.0);
        let h = ConstantHazard(vec![le, lo]);
        let known = known_cause_interval(&h, 0.0, l, r, &[1.0, 0.0], &EDGES, 4);
        let every = all_cause_interval(&h, 0.0, l, r, &EDGES, 4);
        let lam = le + lo;
        let s_l = (-lam * l).exp();
        assert!((known - s_l * le / lam * -(-lam * (r - l)).exp_m1()).abs() < 1e-14);
        assert!((every - s_l * -(-lam * (r - l)).exp_m1()).abs() < 1e-14);
        assert!(known < every - 0.05);
        let both = known_cause_interval(&h, 0.0, l, r, &[1.0, 1.0], &EDGES, 4);
        assert!((both - every).abs() < 1e-14);
    }

    #[test]
    fn scheduled_stop_atom_reference_case() {
        // V3.1 §26: λ_E = 2 λ_O, Λ Δt = 0.6 before the stop.
        let h = ConstantHazard(vec![0.04, 0.02]);
        let p = forecast_partition(
            &h,
            0.0,
            &[5.0, 9.999, 10.0, 25.0],
            &EDGES,
            8,
            10.0,
            f64::INFINITY,
        );
        assert!(p.max_partition_error() < 1e-14);
        assert!((p.f[3 * 2] - 0.3007922426039824).abs() < 1e-12);
        assert!((p.atom[3] - (-0.6f64).exp()).abs() < 1e-12);
        assert!((p.f[3 * 2 + 1] + p.atom[3] - 0.6992077573960176).abs() < 1e-12);
        assert_eq!(p.active[2], 0.0);
        assert_eq!(p.active[3], 0.0);
        assert_eq!(p.atom[1], 0.0);
    }

    #[test]
    fn derivative_of_increment_factor() {
        assert_eq!(d_one_minus_exp_over_rate(1e-12, 0.3), -0.5 * 0.09);
        for &(r, dt) in &[(1e-4, 0.5), (0.02, 0.7), (0.5, 2.0), (3.0, 1.5)] {
            let h = r * 1e-4;
            let fd = (one_minus_exp_over_rate(r + h, dt) - one_minus_exp_over_rate(r - h, dt))
                / (2.0 * h);
            let an = d_one_minus_exp_over_rate(r, dt);
            assert!(
                (fd - an).abs() <= 1e-7 * an.abs().max(1e-12),
                "{r} {dt} {fd} {an}"
            );
        }
    }
}
