//! ACME cause-specific additive neural hazards (V3.1 §8.3–§8.5, §10.1). FP64, analytic gradients.
//!
//! For continuous cause k ∈ {E = extinction, O = operator_termination} and session g:
//!
//!   λ_k(t) = t_ref⁻¹ exp[ a_{k,b(t)} + β_kᵀ c + ½{H_φ(c, x_t)_k − H_φ(0, x_t)_k} + u_{g,k} ],
//!   x_t = log(1 + t/t_ref),   H_φ: 9 → 8 (tanh) → K,   u_g ~ N(0, Σ_u),  Σ_u = diag(σ) Ω diag(σ).
//!
//! Parameters per configuration (K = 2): 12 baseline intercepts (6 fixed physical-time bins per cause),
//! 16 linear coefficients, 98 network weights, 3 frailty-covariance parameters = 129. If every operator
//! termination is scheduled, the continuous O hazard is removed (K = 1, 104 parameters) and only the
//! stop atoms remain (§8.5).
//!
//! Likelihood per run, conditional on being alive at established-phase entry (left truncation):
//!   exact (T, A):        S(T)/S(entry) · Σ_{k∈A} λ_k(T)
//!   interval ((L,R], A): S(L)/S(entry) · [Σ_{k∈A} ∫_L^{min(R,stop)} S λ_k du / S(L) + 1{stop∈(L,R], O∈A} S(stop⁻)/S(L)]
//!   right (C):           S(C)/S(entry)
//!   scheduled stop:      S(stop⁻)/S(entry)            (probability atom, J = O)
//! Session frailties are integrated by tensor Gauss–Hermite quadrature (exact marginal likelihood up
//! to quadrature error), so MAP/NUTS act on the population parameters Θ and the frailty covariance.
//!
//! Implementation: the frailty enters as a per-cause multiplier e^{u_k} of a node-independent base
//! hazard (§10.1), so the network runs once per (row, cell) and every quadrature node reuses it. The
//! gradient is reverse-mode by hand: node-level adjoints of the competing-risk recursions are summed
//! into base-hazard adjoints, then pushed through the 9→8→2 network once; the Cholesky adjoint
//! collects Σ_n ū_n z_nᵀ. Verified against JAX autodiff and finite differences (tests/parity.rs).

use ignis_core::views::{AcmeEventRow, ACME_CAUSES};
use ignis_science::gauss_hermite as gh;
use ignis_science::hazards::{self as hz, Cells, Partition};
use rayon::prelude::*;

pub const OPERATOR: &str = "operator_termination";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Exact,
    Interval,
    Right,
    Stop,
}

impl Kind {
    pub fn from_censoring(s: &str) -> Option<Kind> {
        match s {
            "event" => Some(Kind::Exact),
            "interval" => Some(Kind::Interval),
            "right" => Some(Kind::Right),
            "scheduled_stop" => Some(Kind::Stop),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AcmeSpec {
    /// Continuously modelled causes, subset of ACME_CAUSES in that order.
    pub causes: Vec<String>,
    /// e_0 < … < e_{n-1}; last bin is [e_{n-1}, ∞).
    pub bin_lower_edges_s: Vec<f64>,
    pub t_ref_s: f64,
    pub n_features: usize,
    pub linear: bool,
    pub neural: bool,
    pub frailty: bool,
    pub hidden: usize,
    pub cells_per_bin: usize,
}

impl AcmeSpec {
    pub fn k(&self) -> usize {
        self.causes.len()
    }
    pub fn n_bins(&self) -> usize {
        self.bin_lower_edges_s.len()
    }
    pub fn edges(&self) -> Vec<f64> {
        let mut e = self.bin_lower_edges_s.clone();
        e.push(f64::INFINITY);
        e
    }
}

/// Named parameter blocks in the order of the Python `param_shapes`.
pub fn param_shapes(spec: &AcmeSpec) -> Vec<(String, Vec<usize>)> {
    let (k, f, h) = (spec.k(), spec.n_features, spec.hidden);
    let mut s = vec![("baseline.a".to_string(), vec![k, spec.n_bins()])];
    if spec.linear {
        s.push(("linear.beta".into(), vec![k, f]));
    }
    if spec.neural {
        s.push(("residual.W1".into(), vec![h, f + 1]));
        s.push(("residual.b1".into(), vec![h]));
        s.push(("residual.W2".into(), vec![k, h]));
        s.push(("residual.b2".into(), vec![k]));
    }
    if spec.frailty {
        s.push(("frailty.log_sigma".into(), vec![k]));
        if k == 2 {
            s.push(("frailty.atanh_rho".into(), vec![]));
        }
    }
    s
}

pub fn n_params(spec: &AcmeSpec) -> usize {
    param_shapes(spec)
        .iter()
        .map(|(_, s)| s.iter().product::<usize>())
        .sum()
}

/// Flat parameter layout, blocks in sorted-name order (the order of `ravel_pytree` and of the
/// registry's `TensorMap`).
#[derive(Debug, Clone)]
pub struct Layout {
    pub blocks: Vec<(String, Vec<usize>, usize)>,
    pub len: usize,
    pub a: usize,
    pub beta: Option<usize>,
    pub w1: Option<usize>,
    pub b1: Option<usize>,
    pub w2: Option<usize>,
    pub b2: Option<usize>,
    pub log_sigma: Option<usize>,
    pub atanh_rho: Option<usize>,
}

impl Layout {
    pub fn new(spec: &AcmeSpec) -> Layout {
        let mut shapes = param_shapes(spec);
        shapes.sort_by(|a, b| a.0.cmp(&b.0));
        let mut off = 0;
        let mut blocks = vec![];
        for (n, s) in shapes {
            let len: usize = s.iter().product();
            blocks.push((n, s, off));
            off += len;
        }
        let find = |name: &str| blocks.iter().find(|b| b.0 == name).map(|b| b.2);
        Layout {
            a: find("baseline.a").unwrap(),
            beta: find("linear.beta"),
            w1: find("residual.W1"),
            b1: find("residual.b1"),
            w2: find("residual.W2"),
            b2: find("residual.b2"),
            log_sigma: find("frailty.log_sigma"),
            atanh_rho: find("frailty.atanh_rho"),
            len: off,
            blocks,
        }
    }
    pub fn block(&self, name: &str) -> Option<(usize, usize)> {
        self.blocks
            .iter()
            .find(|b| b.0 == name)
            .map(|b| (b.2, b.1.iter().product()))
    }
    pub fn to_tensors(&self, flat: &[f64]) -> ignis_core::registry::TensorMap {
        self.blocks
            .iter()
            .map(|(n, s, o)| {
                let len: usize = s.iter().product();
                (
                    n.clone(),
                    ignis_core::registry::Tensor::new(s.clone(), flat[*o..o + len].to_vec()),
                )
            })
            .collect()
    }
    pub fn from_tensors(&self, t: &ignis_core::registry::TensorMap) -> anyhow::Result<Vec<f64>> {
        let mut out = vec![0.0; self.len];
        for (n, s, o) in &self.blocks {
            let x = t
                .get(n)
                .ok_or_else(|| anyhow::anyhow!("missing tensor {n}"))?;
            let len: usize = s.iter().product();
            if x.data.len() != len {
                anyhow::bail!("tensor {n} has {} values, expected {len}", x.data.len());
            }
            out[*o..o + len].copy_from_slice(&x.data);
        }
        Ok(out)
    }
    /// Stack posterior draws (`name -> (D, shape...)`) into D flat vectors.
    pub fn draws_from_tensors(
        &self,
        t: &ignis_core::registry::TensorMap,
    ) -> anyhow::Result<Vec<Vec<f64>>> {
        let (n0, s0, _) = &self.blocks[0];
        let x0 = t
            .get(n0)
            .ok_or_else(|| anyhow::anyhow!("missing posterior {n0}"))?;
        let d = x0.data.len() / s0.iter().product::<usize>().max(1);
        let mut out = vec![vec![0.0; self.len]; d];
        for (n, s, o) in &self.blocks {
            let x = t
                .get(n)
                .ok_or_else(|| anyhow::anyhow!("missing posterior {n}"))?;
            let len: usize = s.iter().product();
            for (i, v) in out.iter_mut().enumerate() {
                v[*o..o + len].copy_from_slice(&x.data[i * len..(i + 1) * len]);
            }
        }
        Ok(out)
    }
    pub fn draws_to_tensors(&self, draws: &[Vec<f64>]) -> ignis_core::registry::TensorMap {
        self.blocks
            .iter()
            .map(|(n, s, o)| {
                let len: usize = s.iter().product();
                let mut shape = vec![draws.len()];
                shape.extend(s);
                let data = draws
                    .iter()
                    .flat_map(|d| d[*o..o + len].iter().cloned())
                    .collect();
                (n.clone(), ignis_core::registry::Tensor::new(shape, data))
            })
            .collect()
    }
}

// ---------------------------------------------------------------- quadrature

#[derive(Debug, Clone)]
pub struct Quad {
    /// (N × K) standard-normal nodes.
    pub z: Vec<f64>,
    pub logw: Vec<f64>,
    pub k: usize,
}

impl Quad {
    pub fn n(&self) -> usize {
        self.logw.len()
    }
}

/// Standard-normal GH grid (N, K) and log prior weights, or a single zero node without frailty.
pub fn quadrature(spec: &AcmeSpec, n_nodes: usize) -> Quad {
    if !spec.frailty {
        return Quad {
            z: vec![0.0; spec.k()],
            logw: vec![0.0],
            k: spec.k(),
        };
    }
    let (z, logw) = gh::standard_grid(n_nodes, spec.k());
    Quad {
        z,
        logw,
        k: spec.k(),
    }
}

/// Lower Cholesky factor of the frailty covariance (K × K row-major).
pub fn frailty_chol(p: &[f64], spec: &AcmeSpec, lay: &Layout) -> [f64; 4] {
    let mut l = [0.0; 4];
    if !spec.frailty {
        return l;
    }
    let ls = lay.log_sigma.unwrap();
    if spec.k() == 1 {
        l[0] = p[ls].exp();
        return l;
    }
    let (s0, s1) = (p[ls].exp(), p[ls + 1].exp());
    let rho = p[lay.atanh_rho.unwrap()].tanh();
    l[0] = s0;
    l[2] = s1 * rho;
    l[3] = s1 * (1.0 - rho * rho).sqrt();
    l
}

/// e^{u_n} for every node: (N × K).
pub fn node_multipliers(chol: &[f64; 4], q: &Quad) -> Vec<f64> {
    let k = q.k;
    let n = q.n();
    let mut out = vec![0.0; n * k];
    for i in 0..n {
        let z = &q.z[i * k..(i + 1) * k];
        if k == 1 {
            out[i] = (chol[0] * z[0]).exp();
        } else {
            out[2 * i] = (chol[0] * z[0] + chol[1] * z[1]).exp();
            out[2 * i + 1] = (chol[2] * z[0] + chol[3] * z[1]).exp();
        }
    }
    out
}

// ---------------------------------------------------------------- data encoding

/// One encoded event row with its precomputed cells: points [0, n1) cover (entry, ta], [n1, n1+n2)
/// cover (ta, min(tb, stop)] for interval rows, and the last point is the event time of an exact row.
#[derive(Debug, Clone)]
pub struct Row {
    pub run_id: String,
    pub c: Vec<f64>,
    pub entry: f64,
    pub ta: f64,
    pub tb: f64,
    pub kind: Kind,
    pub cmask: [f64; 2],
    pub atom: f64,
    pub stop: f64,
    pub session: usize,
    pub t: Vec<f64>,
    pub x: Vec<f64>,
    pub bin: Vec<usize>,
    pub dt: Vec<f64>,
    pub n1: usize,
    pub n2: usize,
    pub exact: bool,
}

impl Row {
    pub fn n_points(&self) -> usize {
        self.t.len()
    }
}

#[derive(Debug, Clone)]
pub struct Encoded {
    pub rows: Vec<Row>,
    pub session_ids: Vec<String>,
    /// Row indices per session.
    pub by_session: Vec<Vec<usize>>,
}

impl Encoded {
    pub fn n_sessions(&self) -> usize {
        self.session_ids.len()
    }
}

fn push_cells(row: &mut Row, cells: Cells, t_ref: f64) {
    for i in 0..cells.len() {
        row.t.push(cells.t_mid[i]);
        row.x.push((cells.t_mid[i] / t_ref).ln_1p());
        row.bin.push(cells.bin[i]);
        row.dt.push(cells.dt[i]);
    }
}

/// Validate and encode event rows (censoring "unknown" must be filtered by the caller). `c` are the
/// normalised feature rows; `sessions` fixes the session index order (default: sorted group ids).
pub fn encode_rows(
    rows: &[&AcmeEventRow],
    c: &[Vec<f64>],
    spec: &AcmeSpec,
    sessions: Option<&[String]>,
) -> anyhow::Result<Encoded> {
    let sess: Vec<String> = match sessions {
        Some(s) => s.to_vec(),
        None => {
            let mut s: Vec<String> = rows.iter().map(|r| r.group_id.clone()).collect();
            s.sort();
            s.dedup();
            s
        }
    };
    let edges = spec.edges();
    let m = spec.cells_per_bin;
    let mut out = Vec::with_capacity(rows.len());
    for (i, r) in rows.iter().enumerate() {
        let Some(mut kind) = Kind::from_censoring(&r.censoring) else {
            anyhow::bail!(
                "{}: censoring {:?} is not eligible for the event likelihood",
                r.run_id,
                r.censoring
            );
        };
        if let Some(bad) = r
            .allowed_causes
            .iter()
            .find(|c| !ACME_CAUSES.contains(&c.as_str()))
        {
            anyhow::bail!("{}: unknown cause {bad:?}", r.run_id);
        }
        let st = r.scheduled_stop_s.unwrap_or(f64::INFINITY);
        let mut lo = r.t_lower_s.or(r.t_upper_s);
        let mut hi = r.t_upper_s.or(lo);
        if kind == Kind::Stop {
            if !st.is_finite() {
                anyhow::bail!("{}: scheduled_stop censoring without a stop time", r.run_id);
            }
            lo = Some(st);
            hi = Some(st);
        }
        let (Some(mut lo), Some(mut hi)) = (lo, hi) else {
            anyhow::bail!("{}: no event/censoring time", r.run_id);
        };
        let has_op = r.allowed_causes.iter().any(|c| c == OPERATOR);
        if kind == Kind::Exact && st.is_finite() && (lo - st).abs() < 1e-9 && has_op {
            kind = Kind::Stop; // an operator end exactly at the scheduled stop is the atom
            lo = st;
            hi = st;
        }
        if lo < r.t_entry_s {
            anyhow::bail!(
                "{}: event before established-phase entry (left truncation violated)",
                r.run_id
            );
        }
        if matches!(kind, Kind::Exact | Kind::Right) && lo > st + 1e-9 {
            anyhow::bail!(
                "{}: active/event after the scheduled stop is incompatible with the policy",
                r.run_id
            );
        }
        if kind == Kind::Interval {
            if hi < lo {
                anyhow::bail!("{}: interval upper < lower", r.run_id);
            }
            if st <= lo {
                anyhow::bail!("{}: confirmed active after the scheduled stop", r.run_id);
            }
        }
        let mut cmask = [0.0; 2];
        for (j, cause) in spec.causes.iter().enumerate() {
            cmask[j] = if r.allowed_causes.contains(cause) {
                1.0
            } else {
                0.0
            };
        }
        let has_atom = kind == Kind::Interval && lo < st && st <= hi && has_op;
        if matches!(kind, Kind::Exact | Kind::Interval)
            && cmask.iter().sum::<f64>() == 0.0
            && !has_atom
        {
            anyhow::bail!("{}: admitted cause set has no modelled mechanism", r.run_id);
        }
        let session = sess.iter().position(|s| *s == r.group_id).ok_or_else(|| {
            anyhow::anyhow!(
                "{}: session {} not in the session list",
                r.run_id,
                r.group_id
            )
        })?;
        let mut row = Row {
            run_id: r.run_id.clone(),
            c: c[i].clone(),
            entry: r.t_entry_s,
            ta: lo,
            tb: hi,
            kind,
            cmask,
            atom: if has_atom { 1.0 } else { 0.0 },
            stop: st,
            session,
            t: vec![],
            x: vec![],
            bin: vec![],
            dt: vec![],
            n1: 0,
            n2: 0,
            exact: kind == Kind::Exact,
        };
        let c1 = hz::cell_grid(row.entry, row.ta, &edges, m).nonzero();
        row.n1 = c1.len();
        push_cells(&mut row, c1, spec.t_ref_s);
        let tb_eff = if kind == Kind::Interval {
            row.tb.min(row.stop)
        } else {
            row.ta
        };
        let c2 = hz::cell_grid(row.ta, tb_eff, &edges, m).nonzero();
        row.n2 = c2.len();
        push_cells(&mut row, c2, spec.t_ref_s);
        if row.exact {
            row.t.push(row.ta);
            row.x.push((row.ta / spec.t_ref_s).ln_1p());
            row.bin.push(hz::bin_of(row.ta, &edges));
            row.dt.push(0.0);
        }
        out.push(row);
    }
    let mut by_session = vec![vec![]; sess.len()];
    for (i, r) in out.iter().enumerate() {
        by_session[r.session].push(i);
    }
    Ok(Encoded {
        rows: out,
        session_ids: sess,
        by_session,
    })
}

// ---------------------------------------------------------------- base hazard network

/// Base hazards b_k(t) (without frailty) at `n` points and, if `acts` is given, the hidden tanh
/// activations [h(c,x) ‖ h(0,x)] (n × 2H) needed by the backward pass. `out` is (n × K).
#[allow(clippy::too_many_arguments)]
pub fn base_hazards(
    p: &[f64],
    spec: &AcmeSpec,
    lay: &Layout,
    c: &[f64],
    x: &[f64],
    bin: &[usize],
    out: &mut [f64],
    mut acts: Option<&mut [f64]>,
) {
    let k = spec.k();
    let nb = spec.n_bins();
    let f = spec.n_features;
    let hdim = spec.hidden;
    let log_tref = spec.t_ref_s.ln();
    let mut lin = [0.0f64; 2];
    if let Some(ob) = lay.beta {
        for j in 0..k {
            lin[j] = (0..f).map(|i| c[i] * p[ob + j * f + i]).sum();
        }
    }
    // c-part of the first layer is time independent
    let mut pre_c = [0.0f64; 64];
    assert!(hdim <= 64);
    if spec.neural {
        let (w1, b1) = (lay.w1.unwrap(), lay.b1.unwrap());
        for j in 0..hdim {
            pre_c[j] = (0..f).map(|i| p[w1 + j * (f + 1) + i] * c[i]).sum::<f64>() + p[b1 + j];
        }
    }
    let mut h1 = [0.0f64; 64];
    let mut h0 = [0.0f64; 64];
    for q in 0..x.len() {
        let mut eta = [0.0f64; 2];
        for j in 0..k {
            eta[j] = p[lay.a + j * nb + bin[q]] + lin[j];
        }
        if spec.neural {
            let (w1, b1, w2) = (lay.w1.unwrap(), lay.b1.unwrap(), lay.w2.unwrap());
            for j in 0..hdim {
                let wx = p[w1 + j * (f + 1) + f] * x[q];
                h1[j] = (pre_c[j] + wx).tanh();
                h0[j] = (p[b1 + j] + wx).tanh();
            }
            for kk in 0..k {
                let mut d = 0.0;
                for j in 0..hdim {
                    d += p[w2 + kk * hdim + j] * (h1[j] - h0[j]);
                }
                eta[kk] += 0.5 * d;
            }
            if let Some(a) = acts.as_deref_mut() {
                a[q * 2 * hdim..q * 2 * hdim + hdim].copy_from_slice(&h1[..hdim]);
                a[q * 2 * hdim + hdim..(q + 1) * 2 * hdim].copy_from_slice(&h0[..hdim]);
            }
        }
        for j in 0..k {
            out[q * k + j] = (eta[j] - log_tref).exp();
        }
    }
}

/// Push η̄ (n × K, adjoint of the log base hazard) back into the parameter gradient.
#[allow(clippy::too_many_arguments)]
fn base_backward(
    p: &[f64],
    spec: &AcmeSpec,
    lay: &Layout,
    c: &[f64],
    x: &[f64],
    bin: &[usize],
    acts: &[f64],
    eta_bar: &[f64],
    grad: &mut [f64],
) {
    let k = spec.k();
    let nb = spec.n_bins();
    let f = spec.n_features;
    let hdim = spec.hidden;
    let mut sum_eta = [0.0f64; 2];
    let mut sum_pre1 = [0.0f64; 64];
    for q in 0..x.len() {
        let e = &eta_bar[q * k..(q + 1) * k];
        for j in 0..k {
            grad[lay.a + j * nb + bin[q]] += e[j];
            sum_eta[j] += e[j];
        }
        if spec.neural {
            let (w1, b1, w2) = (lay.w1.unwrap(), lay.b1.unwrap(), lay.w2.unwrap());
            let h1 = &acts[q * 2 * hdim..q * 2 * hdim + hdim];
            let h0 = &acts[q * 2 * hdim + hdim..(q + 1) * 2 * hdim];
            for j in 0..hdim {
                let mut g1 = 0.0;
                for kk in 0..k {
                    let o = 0.5 * e[kk];
                    grad[w2 + kk * hdim + j] += o * (h1[j] - h0[j]);
                    g1 += o * p[w2 + kk * hdim + j];
                }
                let pre1 = g1 * (1.0 - h1[j] * h1[j]);
                let pre0 = -g1 * (1.0 - h0[j] * h0[j]);
                sum_pre1[j] += pre1;
                grad[w1 + j * (f + 1) + f] += (pre1 + pre0) * x[q];
                grad[b1 + j] += pre1 + pre0;
            }
        }
    }
    if let Some(ob) = lay.beta {
        for j in 0..k {
            for i in 0..f {
                grad[ob + j * f + i] += sum_eta[j] * c[i];
            }
        }
    }
    if spec.neural {
        let w1 = lay.w1.unwrap();
        for j in 0..hdim {
            for i in 0..f {
                grad[w1 + j * (f + 1) + i] += sum_pre1[j] * c[i];
            }
        }
    }
}

// ---------------------------------------------------------------- likelihood

/// Per-thread scratch buffers (reused across sessions; nothing is allocated per node or cell).
#[derive(Default)]
pub struct Workspace {
    base: Vec<f64>,
    acts: Vec<f64>,
    bbar: Vec<f64>,
    offs: Vec<usize>,
    b1: Vec<f64>,
    sll: Vec<f64>,
    ubar: Vec<f64>,
    cell: Vec<f64>,
}

#[inline]
fn row_ll_node(row: &Row, base: &[f64], b1: &[f64], e: &[f64], k: usize, cell: &mut [f64]) -> f64 {
    let mut ll = 0.0;
    for j in 0..k {
        ll -= e[j] * b1[j];
    }
    match row.kind {
        Kind::Exact => {
            let q = row.n_points() - 1;
            let mut ex = 0.0;
            for j in 0..k {
                ex += row.cmask[j] * e[j] * base[q * k + j];
            }
            ll += ex.ln();
        }
        Kind::Interval => {
            let (s, _) = interval_forward(row, base, e, k, cell);
            ll += s.ln();
        }
        _ => {}
    }
    ll
}

/// Interval mass I = Σ_c P_c g_c M_c + atom S_end over seg-2 cells; `cell` receives P_c g_c M_c.
#[inline]
fn interval_forward(row: &Row, base: &[f64], e: &[f64], k: usize, cell: &mut [f64]) -> (f64, f64) {
    let mut cum = 0.0;
    let mut tot = 0.0;
    for c in 0..row.n2 {
        let q = row.n1 + c;
        let mut rate = 0.0;
        let mut mm = 0.0;
        for j in 0..k {
            let l = e[j] * base[q * k + j];
            rate += l;
            mm += row.cmask[j] * l;
        }
        let dt = row.dt[q];
        cum += rate * dt;
        let excl = cum - rate * dt;
        let v = (-excl).exp() * hz::one_minus_exp_over_rate(rate, dt) * mm;
        cell[c] = v;
        tot += v;
    }
    let s_end = (-cum).exp();
    (tot + row.atom * s_end, s_end)
}

/// Backward of one (row, node) log-likelihood with weight ω: accumulates b̄ (row points × K) and ū (K).
#[inline]
fn row_backward_node(
    row: &Row,
    base: &[f64],
    b1: &[f64],
    e: &[f64],
    k: usize,
    omega: f64,
    bbar: &mut [f64],
    ubar: &mut [f64],
    cell: &mut [f64],
) {
    for j in 0..k {
        // −Σ_k e_k B1_k
        for q in 0..row.n1 {
            bbar[q * k + j] -= omega * e[j] * row.dt[q];
        }
        ubar[j] -= omega * e[j] * b1[j];
    }
    match row.kind {
        Kind::Exact => {
            let q = row.n_points() - 1;
            let mut ex = 0.0;
            for j in 0..k {
                ex += row.cmask[j] * e[j] * base[q * k + j];
            }
            for j in 0..k {
                let d = omega * row.cmask[j] * e[j] / ex;
                bbar[q * k + j] += d;
                ubar[j] += d * base[q * k + j];
            }
        }
        Kind::Interval => {
            let (tot, s_end) = interval_forward(row, base, e, k, cell);
            let s = omega / tot;
            let mut tail = row.atom * s_end; // Σ_{c'>c} P g M + atom S_end
            for c in (0..row.n2).rev() {
                let q = row.n1 + c;
                let mut rate = 0.0;
                let mut mm = 0.0;
                for j in 0..k {
                    let l = e[j] * base[q * k + j];
                    rate += l;
                    mm += row.cmask[j] * l;
                }
                let dt = row.dt[q];
                let g = hz::one_minus_exp_over_rate(rate, dt);
                let gp = hz::d_one_minus_exp_over_rate(rate, dt);
                // P_c = cell[c] / (g M) is recomputed stably from the stored product when M > 0
                let p_c = if mm > 0.0 && g > 0.0 {
                    cell[c] / (g * mm)
                } else {
                    p_from_tail(row, base, e, k, c)
                };
                for j in 0..k {
                    let d_lam = p_c * (g * row.cmask[j] + gp * mm) - dt * tail;
                    let l = e[j] * base[q * k + j];
                    bbar[q * k + j] += s * e[j] * d_lam;
                    ubar[j] += s * l * d_lam;
                }
                tail += cell[c];
            }
        }
        _ => {}
    }
}

/// Survival factor at the start of seg-2 cell `c` (fallback when the cell carries no admitted mass).
fn p_from_tail(row: &Row, base: &[f64], e: &[f64], k: usize, c: usize) -> f64 {
    let mut cum = 0.0;
    for cc in 0..c {
        let q = row.n1 + cc;
        let rate: f64 = (0..k).map(|j| e[j] * base[q * k + j]).sum();
        cum += rate * row.dt[q];
    }
    (-cum).exp()
}

/// Session-level evaluation. Returns log Σ_n w_n Π_{i∈g} L_i(u_n); with `grad` the gradient is added
/// (parameter part and Cholesky adjoint `lbar`). `sll_out` receives the per-node session log-lik.
#[allow(clippy::too_many_arguments)]
fn session_eval(
    p: &[f64],
    spec: &AcmeSpec,
    lay: &Layout,
    enc: &Encoded,
    rows: &[usize],
    q: &Quad,
    eu: &[f64],
    ws: &mut Workspace,
    grad: Option<(&mut [f64], &mut [f64; 4])>,
    sll_out: Option<&mut [f64]>,
) -> f64 {
    let k = spec.k();
    let n = q.n();
    let h2 = 2 * spec.hidden;
    // stage 1: base hazards per row
    ws.offs.clear();
    let mut tot = 0;
    let mut max_n2 = 0;
    for &r in rows {
        ws.offs.push(tot);
        tot += enc.rows[r].n_points();
        max_n2 = max_n2.max(enc.rows[r].n2);
    }
    ws.base.resize(tot * k, 0.0);
    ws.acts.resize(if spec.neural { tot * h2 } else { 0 }, 0.0);
    ws.b1.resize(rows.len() * k, 0.0);
    ws.cell.resize(max_n2.max(1), 0.0);
    for (i, &r) in rows.iter().enumerate() {
        let row = &enc.rows[r];
        let o = ws.offs[i];
        let np = row.n_points();
        let acts = if spec.neural {
            Some(&mut ws.acts[o * h2..(o + np) * h2])
        } else {
            None
        };
        base_hazards(
            p,
            spec,
            lay,
            &row.c,
            &row.x,
            &row.bin,
            &mut ws.base[o * k..(o + np) * k],
            acts,
        );
        for j in 0..k {
            ws.b1[i * k + j] = (0..row.n1)
                .map(|qq| ws.base[(o + qq) * k + j] * row.dt[qq])
                .sum();
        }
    }
    // stage 2: per-node session log-likelihood
    ws.sll.resize(n, 0.0);
    for nn in 0..n {
        let e = &eu[nn * k..(nn + 1) * k];
        let mut s = 0.0;
        for (i, &r) in rows.iter().enumerate() {
            let row = &enc.rows[r];
            let o = ws.offs[i];
            s += row_ll_node(
                row,
                &ws.base[o * k..(o + row.n_points()) * k],
                &ws.b1[i * k..(i + 1) * k],
                e,
                k,
                &mut ws.cell,
            );
        }
        ws.sll[nn] = s + q.logw[nn];
    }
    let lse = gh::logsumexp(&ws.sll);
    if let Some(out) = sll_out {
        for nn in 0..n {
            out[nn] = ws.sll[nn] - q.logw[nn];
        }
    }
    let Some((g, lbar)) = grad else { return lse };
    // stage 3: adjoints
    ws.bbar.clear();
    ws.bbar.resize(tot * k, 0.0);
    ws.ubar.clear();
    ws.ubar.resize(n * k, 0.0);
    for nn in 0..n {
        let omega = (ws.sll[nn] - lse).exp();
        if omega == 0.0 {
            continue;
        }
        let e = &eu[nn * k..(nn + 1) * k];
        let ub = &mut ws.ubar[nn * k..(nn + 1) * k];
        for (i, &r) in rows.iter().enumerate() {
            let row = &enc.rows[r];
            let o = ws.offs[i];
            let np = row.n_points();
            row_backward_node(
                row,
                &ws.base[o * k..(o + np) * k],
                &ws.b1[i * k..(i + 1) * k],
                e,
                k,
                omega,
                &mut ws.bbar[o * k..(o + np) * k],
                ub,
                &mut ws.cell,
            );
        }
    }
    // b̄ → η̄ = b̄ b → network
    for (i, &r) in rows.iter().enumerate() {
        let row = &enc.rows[r];
        let o = ws.offs[i];
        let np = row.n_points();
        for v in 0..np * k {
            ws.bbar[o * k + v] *= ws.base[o * k + v];
        }
        let acts = if spec.neural {
            &ws.acts[o * h2..(o + np) * h2]
        } else {
            &[][..]
        };
        base_backward(
            p,
            spec,
            lay,
            &row.c,
            &row.x,
            &row.bin,
            acts,
            &ws.bbar[o * k..(o + np) * k],
            g,
        );
    }
    if spec.frailty {
        for nn in 0..n {
            let z = &q.z[nn * k..(nn + 1) * k];
            for i in 0..k {
                // ū_k = Σ e_k-adjoint · e_k  (since ∂e/∂u = e, already folded into ubar via λ = e b)
                for j in 0..k {
                    lbar[i * 2 + j] += ws.ubar[nn * k + i] * z[j];
                }
            }
        }
    }
    lse
}

fn chol_backward(p: &[f64], spec: &AcmeSpec, lay: &Layout, lbar: &[f64; 4], grad: &mut [f64]) {
    if !spec.frailty {
        return;
    }
    let ls = lay.log_sigma.unwrap();
    if spec.k() == 1 {
        grad[ls] += lbar[0] * p[ls].exp();
        return;
    }
    let (s0, s1) = (p[ls].exp(), p[ls + 1].exp());
    let ar = lay.atanh_rho.unwrap();
    let rho = p[ar].tanh();
    let r = (1.0 - rho * rho).sqrt();
    grad[ls] += lbar[0] * s0;
    grad[ls + 1] += lbar[2] * s1 * rho + lbar[3] * s1 * r;
    let d_rho = lbar[2] * s1 - lbar[3] * s1 * rho / r;
    grad[ar] += d_rho * (1.0 - rho * rho);
}

/// Σ_g log Σ_n w_n Π_{i∈g} L_i(u_n) and (optionally) its gradient, parallel over sessions.
pub fn marginal_loglik(
    p: &[f64],
    spec: &AcmeSpec,
    lay: &Layout,
    enc: &Encoded,
    q: &Quad,
    grad: Option<&mut [f64]>,
) -> f64 {
    let chol = frailty_chol(p, spec, lay);
    let eu = node_multipliers(&chol, q);
    let want = grad.is_some();
    let len = lay.len;
    let work = |ws: &mut Workspace, rows: &Vec<usize>| {
        if rows.is_empty() {
            return (0.0, vec![], [0.0; 4]);
        }
        if want {
            let mut g = vec![0.0; len];
            let mut lb = [0.0; 4];
            let v = session_eval(
                p,
                spec,
                lay,
                enc,
                rows,
                q,
                &eu,
                ws,
                Some((&mut g, &mut lb)),
                None,
            );
            (v, g, lb)
        } else {
            (
                session_eval(p, spec, lay, enc, rows, q, &eu, ws, None, None),
                vec![],
                [0.0; 4],
            )
        }
    };
    // small problems (baselines: one node, few rows) are faster without the thread pool
    let results: Vec<(f64, Vec<f64>, [f64; 4])> = if enc.rows.len() * q.n() < 4096 {
        let mut ws = Workspace::default();
        enc.by_session
            .iter()
            .map(|rows| work(&mut ws, rows))
            .collect()
    } else {
        enc.by_session
            .par_iter()
            .map_init(Workspace::default, work)
            .collect()
    };
    let total: f64 = results.iter().map(|r| r.0).sum();
    if let Some(g) = grad {
        let mut lbar = [0.0; 4];
        for (_, gg, lb) in &results {
            if gg.is_empty() {
                continue;
            }
            for i in 0..len {
                g[i] += gg[i];
            }
            for i in 0..4 {
                lbar[i] += lb[i];
            }
        }
        chol_backward(p, spec, lay, &lbar, g);
    }
    total
}

/// Per-session, per-node log-likelihood table (G × N) at one parameter vector.
pub fn session_loglik_nodes(
    p: &[f64],
    spec: &AcmeSpec,
    lay: &Layout,
    enc: &Encoded,
    q: &Quad,
) -> Vec<f64> {
    let chol = frailty_chol(p, spec, lay);
    let eu = node_multipliers(&chol, q);
    let n = q.n();
    let rows: Vec<Vec<f64>> = enc
        .by_session
        .par_iter()
        .map_init(Workspace::default, |ws, rows| {
            let mut out = vec![0.0; n];
            if !rows.is_empty() {
                session_eval(p, spec, lay, enc, rows, q, &eu, ws, None, Some(&mut out));
            }
            out
        })
        .collect();
    rows.concat()
}

/// log L_row(u_n) for every row and node (R × N) — reference/inspection path.
pub fn row_loglik_nodes(
    p: &[f64],
    spec: &AcmeSpec,
    lay: &Layout,
    enc: &Encoded,
    q: &Quad,
) -> Vec<f64> {
    let chol = frailty_chol(p, spec, lay);
    let eu = node_multipliers(&chol, q);
    let n = q.n();
    let rows: Vec<Vec<f64>> = (0..enc.rows.len())
        .into_par_iter()
        .map_init(Workspace::default, |ws, r| {
            let mut out = vec![0.0; n];
            session_eval(p, spec, lay, enc, &[r], q, &eu, ws, None, Some(&mut out));
            out
        })
        .collect();
    rows.concat()
}

// ---------------------------------------------------------------- priors

#[derive(Debug, Clone, serde::Serialize)]
pub struct Priors {
    pub a_mean: f64,
    pub a_sd: f64,
    pub beta_sd: f64,
    pub w1_sd: f64,
    pub b1_sd: f64,
    pub w2_sd: f64,
    pub b2_sd: f64,
    pub sigma_halfnormal: f64,
    pub lkj_eta: f64,
}

pub fn prior_settings(spec: &AcmeSpec, cfg: &serde_json::Value) -> Priors {
    let p = &cfg["priors"];
    let g = |k: &str, d: f64| p.get(k).and_then(|v| v.as_f64()).unwrap_or(d);
    Priors {
        a_mean: (g("rate_ref_per_s", 0.01) * spec.t_ref_s).ln(),
        a_sd: g("a_sd", 1.5),
        beta_sd: g("beta_sd", 0.5),
        w1_sd: g("W1_sd", 1.0 / ((spec.n_features + 1) as f64).sqrt()),
        b1_sd: g("b1_sd", 0.1),
        w2_sd: g("W2_sd", 0.25 / (spec.hidden as f64).sqrt()),
        b2_sd: g("b2_sd", 0.1),
        sigma_halfnormal: g("sigma_halfnormal", 0.5),
        lkj_eta: g("lkj_eta", 2.0),
    }
}

fn normal_block(
    p: &[f64],
    grad: &mut Option<&mut [f64]>,
    off: usize,
    len: usize,
    mu: f64,
    sd: f64,
) -> f64 {
    let c = -sd.ln() - 0.5 * (2.0 * std::f64::consts::PI).ln();
    let mut s = 0.0;
    for i in off..off + len {
        let z = (p[i] - mu) / sd;
        s += -0.5 * z * z + c;
        if let Some(g) = grad.as_deref_mut() {
            g[i] -= z / sd;
        }
    }
    s
}

/// Log prior density in the unconstrained coordinates (Jacobians of log σ and atanh ρ included).
pub fn log_prior(
    p: &[f64],
    spec: &AcmeSpec,
    lay: &Layout,
    pr: &Priors,
    mut grad: Option<&mut [f64]>,
) -> f64 {
    let k = spec.k();
    let mut lp = normal_block(p, &mut grad, lay.a, k * spec.n_bins(), pr.a_mean, pr.a_sd);
    if let Some(o) = lay.beta {
        lp += normal_block(p, &mut grad, o, k * spec.n_features, 0.0, pr.beta_sd);
    }
    if spec.neural {
        let h = spec.hidden;
        lp += normal_block(
            p,
            &mut grad,
            lay.w1.unwrap(),
            h * (spec.n_features + 1),
            0.0,
            pr.w1_sd,
        );
        lp += normal_block(p, &mut grad, lay.b1.unwrap(), h, 0.0, pr.b1_sd);
        lp += normal_block(p, &mut grad, lay.w2.unwrap(), k * h, 0.0, pr.w2_sd);
        lp += normal_block(p, &mut grad, lay.b2.unwrap(), k, 0.0, pr.b2_sd);
    }
    if spec.frailty {
        let ls = lay.log_sigma.unwrap();
        let sc = pr.sigma_halfnormal;
        for j in 0..k {
            let s = p[ls + j].exp();
            lp += 0.5 * (2.0 / std::f64::consts::PI).ln() - sc.ln() - 0.5 * (s / sc).powi(2)
                + p[ls + j];
            if let Some(g) = grad.as_deref_mut() {
                g[ls + j] += -(s / sc).powi(2) + 1.0;
            }
        }
        if k == 2 {
            let ar = lay.atanh_rho.unwrap();
            let eta = pr.lkj_eta;
            let rho = p[ar].tanh();
            let log_beta = 2.0 * libm::lgamma(eta) - libm::lgamma(2.0 * eta);
            let l1 = (-rho * rho).ln_1p();
            lp += (eta - 1.0) * l1 - log_beta - (2.0 * eta - 1.0) * 2f64.ln() + l1;
            if let Some(g) = grad.as_deref_mut() {
                g[ar] += -2.0 * eta * rho;
            }
        }
    }
    lp
}

/// Log posterior and (optionally) its gradient.
pub fn log_posterior(
    p: &[f64],
    spec: &AcmeSpec,
    lay: &Layout,
    pr: &Priors,
    enc: &Encoded,
    q: &Quad,
    mut grad: Option<&mut [f64]>,
) -> f64 {
    if let Some(g) = grad.as_deref_mut() {
        g.iter_mut().for_each(|v| *v = 0.0);
    }
    let lp = log_prior(p, spec, lay, pr, grad.as_deref_mut());
    lp + marginal_loglik(p, spec, lay, enc, q, grad)
}

// ---------------------------------------------------------------- prefix forecast

/// Node-independent base hazards of one draw on the (entry, τ] cells and the output segments.
pub struct DrawBase {
    /// Base cumulative hazard per cause over (entry, τ].
    pub h_tau: [f64; 2],
    pub seg_lam: Vec<Vec<f64>>,
}

pub fn draw_base(
    p: &[f64],
    spec: &AcmeSpec,
    lay: &Layout,
    c: &[f64],
    t_entry: f64,
    tau: f64,
    segs: &[Cells],
) -> DrawBase {
    let k = spec.k();
    let edges = spec.edges();
    let cells = hz::cell_grid(t_entry, tau, &edges, spec.cells_per_bin).nonzero();
    let eval = |cl: &Cells| {
        let x: Vec<f64> = cl
            .t_mid
            .iter()
            .map(|t| (t / spec.t_ref_s).ln_1p())
            .collect();
        let mut out = vec![0.0; cl.len() * k];
        base_hazards(p, spec, lay, c, &x, &cl.bin, &mut out, None);
        out
    };
    let lam = eval(&cells);
    let mut h_tau = [0.0; 2];
    for (i, dt) in cells.dt.iter().enumerate() {
        for j in 0..k {
            h_tau[j] += lam[i * k + j] * dt;
        }
    }
    DrawBase {
        h_tau,
        seg_lam: segs.iter().map(eval).collect(),
    }
}

/// Conditional partition aggregated over draws × nodes with joint prefix weights (§10.1).
pub struct Forecast {
    pub part: Partition,
    pub global: Vec<f64>,
    /// (G × N) joint weights.
    pub w: Vec<f64>,
}

/// Joint (global draw × quadrature node) prefix weighting and the conditional partition. `log_extra`
/// (G × N) adds a known-session log-likelihood table (already normalised per draw by the caller).
#[allow(clippy::too_many_arguments)]
pub fn predict_partition(
    draws: &[Vec<f64>],
    spec: &AcmeSpec,
    lay: &Layout,
    c: &[f64],
    t_entry: f64,
    tau: f64,
    times: &[f64],
    stop: f64,
    horizon: f64,
    q: &Quad,
    log_extra: Option<&[f64]>,
) -> Forecast {
    let k = spec.k();
    let n = q.n();
    let g = draws.len();
    let edges = spec.edges();
    let segs = hz::partition_cells(tau, times, &edges, spec.cells_per_bin, stop, horizon);
    let bases: Vec<(DrawBase, Vec<f64>)> = draws
        .par_iter()
        .map(|p| {
            let b = draw_base(p, spec, lay, c, t_entry, tau, &segs);
            let eu = node_multipliers(&frailty_chol(p, spec, lay), q);
            (b, eu)
        })
        .collect();
    let mut lik = vec![0.0; g * n];
    for (gi, (b, eu)) in bases.iter().enumerate() {
        for nn in 0..n {
            let mut ls = 0.0;
            for j in 0..k {
                ls -= eu[nn * k + j] * b.h_tau[j];
            }
            lik[gi * n + nn] = ls + log_extra.map(|x| x[gi * n + nn]).unwrap_or(0.0);
        }
    }
    let lg = -(g as f64).ln();
    let jw = gh::joint_global_local_weights(&vec![lg; g], &q.logw, &lik);
    let nt = times.len();
    let parts: Vec<Partition> = bases
        .par_iter()
        .enumerate()
        .map(|(gi, (b, eu))| {
            let mut acc = Partition::zeros(nt, k);
            let mut tmp = Partition::zeros(nt, k);
            for nn in 0..n {
                let w = jw.w[gi * n + nn];
                if w == 0.0 {
                    continue;
                }
                hz::partition_from_lam(
                    &segs,
                    &b.seg_lam,
                    &eu[nn * k..(nn + 1) * k],
                    times,
                    stop,
                    horizon,
                    &mut tmp,
                );
                for t in 0..nt {
                    acc.active[t] += w * tmp.active[t];
                    acc.atom[t] += w * tmp.atom[t];
                    acc.unresolved[t] += w * tmp.unresolved[t];
                }
                for v in 0..nt * k {
                    acc.f[v] += w * tmp.f[v];
                }
            }
            acc
        })
        .collect();
    let mut part = Partition::zeros(nt, k);
    for p in &parts {
        for t in 0..nt {
            part.active[t] += p.active[t];
            part.atom[t] += p.atom[t];
            part.unresolved[t] += p.unresolved[t];
        }
        for v in 0..nt * k {
            part.f[v] += p.f[v];
        }
    }
    Forecast {
        part,
        global: jw.global,
        w: jw.w,
    }
}

/// λ_k(t) without frailty at given times (T × K) — lab tools/inspection.
pub fn hazard_curves(
    p: &[f64],
    spec: &AcmeSpec,
    lay: &Layout,
    c: &[f64],
    times: &[f64],
) -> Vec<f64> {
    let edges = spec.edges();
    let bin: Vec<usize> = times.iter().map(|t| hz::bin_of(*t, &edges)).collect();
    let x: Vec<f64> = times.iter().map(|t| (t / spec.t_ref_s).ln_1p()).collect();
    let mut out = vec![0.0; times.len() * spec.k()];
    base_hazards(p, spec, lay, c, &x, &bin, &mut out, None);
    out
}

/// Known-session table normalised per draw (review-B fix 1): the draws Θ_g ~ p(Θ | D) already contain
/// the session marginal Z_s(Θ) = Σ_n w_n L_s(u_n | Θ), so the frailty posterior given Θ_g is
/// w_n L_s(u_n | Θ_g) / Z_s(Θ_g). Without the division the session's data would count twice.
/// `table` is (G × N) raw session log-likelihoods; returns log L_s − log Z_s per draw.
pub fn normalise_session_table(table: &[f64], q: &Quad) -> Vec<f64> {
    let n = q.n();
    let mut out = table.to_vec();
    let mut buf = vec![0.0; n];
    for row in out.chunks_mut(n) {
        for i in 0..n {
            buf[i] = row[i] + q.logw[i];
        }
        let lz = gh::logsumexp(&buf);
        row.iter_mut().for_each(|v| *v -= lz);
    }
    out
}
