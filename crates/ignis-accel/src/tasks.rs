//! Statistical proposal tasks generated from a FITTED expert (V3.1 §10.6). Model-generated, not NASA.
//!
//! Each task draws Θ from the release (posterior draws or the MAP point / ensemble member), local
//! effects from their prior, admitted conditions from the train condition support, an observation
//! schedule, and the event/observations from the model itself. The cutoff is drawn by a rule that is
//! independent of the final duration; tasks are kept only when the run is still active and in support
//! at the cutoff, which is exactly the conditioning the runtime target uses.

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{bail, Context};
use ignis_acme::model as am;
use ignis_bass::dynamics as dy;
use ignis_bass::model as bm;
use ignis_bass::operator::{self as bop, Grid};
use ignis_core::npz::{read_npz, write_npz, NpyArray};
use ignis_core::registry::LoadedRelease;
use ignis_science::hazards as hz;
use rand::{rngs::StdRng, RngExt, SeedableRng};
use rand_distr::{Exp1, StandardNormal};
use serde_json::{json, Value};

use crate::nn::{METADATA_FIELDS, META_DIM, TOKEN_DIM, TOKEN_FIELDS};

pub const LABEL: &str =
    "model-generated statistical proposal tasks; synthetic, not NASA observations";

/// Named arrays (shape, values) — written as an uncompressed npz.
pub type Arrays = BTreeMap<String, (Vec<usize>, Vec<f64>)>;

/// PCA (rank ≤ 16) of standardised global parameter draws, fitted on training draws only (block
/// subspace iteration). A proposal-side summary; the likelihood always uses the complete Θ.
pub struct GlobalSummary {
    center: Vec<f64>,
    scale: Vec<f64>,
    loadings: Vec<Vec<f64>>,
    rank: usize,
}

impl GlobalSummary {
    pub fn fit(draws: &[Vec<f64>], rank: usize) -> GlobalSummary {
        let (n, p) = (draws.len(), draws[0].len());
        let center: Vec<f64> = (0..p)
            .map(|j| draws.iter().map(|d| d[j]).sum::<f64>() / n as f64)
            .collect();
        let scale: Vec<f64> = (0..p)
            .map(|j| {
                ((draws
                    .iter()
                    .map(|d| (d[j] - center[j]).powi(2))
                    .sum::<f64>()
                    / n as f64)
                    .sqrt())
                .max(1e-8)
            })
            .collect();
        let z: Vec<Vec<f64>> = draws
            .iter()
            .map(|d| (0..p).map(|j| (d[j] - center[j]) / scale[j]).collect())
            .collect();
        let mut loadings = vec![];
        if n > 1 {
            let k = rank.min(n).min(p);
            let mut rng = StdRng::seed_from_u64(1);
            let mut v: Vec<Vec<f64>> = (0..k)
                .map(|_| {
                    (0..p)
                        .map(|_| rng.sample::<f64, _>(StandardNormal))
                        .collect()
                })
                .collect();
            let apply = |x: &[f64]| -> Vec<f64> {
                // ZᵀZ x
                let zx: Vec<f64> = z
                    .iter()
                    .map(|r| r.iter().zip(x).map(|(a, b)| a * b).sum())
                    .collect();
                (0..p)
                    .map(|j| z.iter().zip(&zx).map(|(r, s)| r[j] * s).sum())
                    .collect()
            };
            let mut norms = vec![0.0; k];
            for _ in 0..60 {
                v = v.iter().map(|x| apply(x)).collect();
                for i in 0..k {
                    for jj in 0..i {
                        let d: f64 = v[i].iter().zip(&v[jj]).map(|(a, b)| a * b).sum();
                        let vj = v[jj].clone();
                        v[i].iter_mut().zip(&vj).for_each(|(a, b)| *a -= d * b);
                    }
                    let nn = v[i].iter().map(|a| a * a).sum::<f64>().sqrt();
                    norms[i] = nn;
                    if nn > 0.0 {
                        v[i].iter_mut().for_each(|a| *a /= nn);
                    }
                }
            }
            let top = norms.iter().cloned().fold(0.0, f64::max);
            loadings = v
                .into_iter()
                .zip(&norms)
                .filter(|(_, nn)| **nn > 1e-20 * top.max(1e-300) && **nn > 1e-20)
                .map(|(x, _)| x)
                .collect();
        }
        GlobalSummary {
            center,
            scale,
            loadings,
            rank,
        }
    }
    pub fn apply(&self, d: &[f64]) -> Vec<f64> {
        let mut out = vec![0.0; self.rank];
        for (i, l) in self.loadings.iter().enumerate() {
            out[i] = d
                .iter()
                .enumerate()
                .map(|(j, x)| (x - self.center[j]) / self.scale[j] * l[j])
                .sum();
        }
        out
    }
}

fn push(a: &mut Arrays, name: &str, row: &[f64]) {
    let e = a
        .entry(name.to_string())
        .or_insert((vec![0, row.len()], vec![]));
    e.0[0] += 1;
    e.1.extend_from_slice(row);
}

/// ACME tasks: the latent is the standardised session frailty z (u = L z).
pub fn acme_tasks(
    r: &LoadedRelease,
    n_tasks: usize,
    seed: u64,
    cutoff_fraction: f64,
    stop_choices: &[Option<f64>],
) -> anyhow::Result<Arrays> {
    let b = ignis_acme::expert::bind(r)?;
    if !b.spec.frailty {
        bail!("ACME tasks need a frailty model (the latent is the session frailty)");
    }
    let gsum = GlobalSummary::fit(&b.draws, 16);
    let gs: Vec<Vec<f64>> = b.draws.iter().map(|d| gsum.apply(d)).collect();
    let c = r
        .params
        .get("support.train_features")
        .context("support.train_features")?;
    let f = c.shape[1];
    let horizon = r.release.applicability["horizon_s"]
        .as_f64()
        .context("horizon_s")?;
    let entry = 0.0;
    let k = b.spec.k();
    let cells = hz::cell_grid(entry, horizon, &b.spec.edges(), b.spec.cells_per_bin).nonzero();
    let x: Vec<f64> = cells
        .t_mid
        .iter()
        .map(|t| (t / b.spec.t_ref_s).ln_1p())
        .collect();
    let mut rng = StdRng::seed_from_u64(seed);
    let mut out = Arrays::new();
    let mut lam = vec![0.0; cells.len() * k];
    let mut kept = 0;
    let t_ref = b.spec.t_ref_s;
    while kept < n_tasks {
        let g = rng.random_range(0..b.draws.len());
        let p = &b.draws[g];
        let z: Vec<f64> = (0..k)
            .map(|_| rng.sample::<f64, _>(StandardNormal))
            .collect();
        let ci = rng.random_range(0..c.shape[0]);
        let cv = &c.data[ci * f..(ci + 1) * f];
        am::base_hazards(p, &b.spec, &b.lay, cv, &x, &cells.bin, &mut lam, None);
        let l = am::frailty_chol(p, &b.spec, &b.lay);
        let u: Vec<f64> = if k == 1 {
            vec![l[0] * z[0]]
        } else {
            vec![l[0] * z[0], l[2] * z[0] + l[3] * z[1]]
        };
        let e: f64 = rng.sample(Exp1);
        let mut h = 0.0;
        let mut t_ev = f64::INFINITY;
        for i in 0..cells.len() {
            let rate: f64 = (0..k).map(|j| lam[i * k + j] * u[j].exp()).sum();
            let hn = h + rate * cells.dt[i];
            if hn >= e {
                t_ev = cells.t_mid[i] - 0.5 * cells.dt[i] + (e - h) / rate;
                break;
            }
            h = hn;
        }
        let stop = stop_choices[rng.random_range(0..stop_choices.len())].unwrap_or(f64::INFINITY);
        let tau = rng.random_range(entry..entry + cutoff_fraction * (horizon - entry)); // independent of T
        if !(t_ev > tau && tau < stop) {
            continue; // condition on being active (and fed) at the cutoff, as the runtime does
        }
        let mut tok = vec![0.0; TOKEN_DIM];
        tok[0] = (tau / t_ref).ln_1p();
        tok[1] = ((tau - entry) / t_ref).ln_1p();
        tok[2] = 1.0; // still active at the cutoff (event-only expert: no continuous measurement)
        tok[5] = 1.0;
        tok[6] = 1.0;
        tok[7] = if stop.is_finite() { 1.0 } else { 0.0 };
        tok[8] = 1.0; // channel code 0 = activity/status
        for i in 0..8.min(f) {
            tok[16 + i] = cv[i];
        }
        let mut meta = vec![0.0; META_DIM];
        for i in 0..8.min(f) {
            meta[i] = cv[i];
        }
        meta[8] = (tau / t_ref).ln_1p();
        meta[9] = 1f64.ln_1p();
        meta[10] = (entry / t_ref).ln_1p();
        meta[11] = (tau / t_ref).ln_1p();
        meta[13] = 0.0;
        meta[14] = 1.0;
        meta[15] = 1.0;
        push(&mut out, "tokens", &tok);
        push(&mut out, "mask", &[1.0]);
        push(&mut out, "meta", &meta);
        push(&mut out, "gsum", &gs[g]);
        push(&mut out, "latent", &z);
        push(&mut out, "draw", &[g as f64]);
        push(&mut out, "c", cv);
        push(&mut out, "tau", &[tau]);
        push(
            &mut out,
            "stop",
            &[if stop.is_finite() { stop } else { -1.0 }],
        );
        kept += 1;
    }
    // tokens are (n, 1, 24)
    if let Some(t) = out.get_mut("tokens") {
        t.0 = vec![t.0[0], 1, TOKEN_DIM];
    }
    Ok(out)
}

struct ZeroCond<'a> {
    c_raw: &'a [f64],
    na: &'a ignis_bass::data::NormArrays,
}
impl dy::Conditions for ZeroCond<'_> {
    fn at(&self, t: f64, out: &mut [f64]) {
        for j in 0..out.len() {
            let raw = if self.na.is_time[j] {
                (t.max(0.0) / self.na.reference[j]).ln_1p()
            } else {
                self.c_raw[j]
            };
            out[j] = (raw - self.na.center[j]) / self.na.scale[j];
        }
    }
}

/// BASS tasks: the latent is the standardised 12-dim local state x / σ_prior.
pub fn bass_tasks(
    r: &LoadedRelease,
    n_tasks: usize,
    seed: u64,
    frame_dt_s: f64,
    max_frames: usize,
    width_m: f64,
    length_m: f64,
) -> anyhow::Result<Arrays> {
    let b = ignis_bass::expert::bind_operator(r)?;
    let lay = bm::GlobalLayout::new();
    let gsum = GlobalSummary::fit(
        &b.members.iter().map(|m| m.g.clone()).collect::<Vec<_>>(),
        16,
    );
    let gs: Vec<Vec<f64>> = b.members.iter().map(|m| gsum.apply(&m.g)).collect();
    let grid = Grid::new(bop::N_NODES);
    let num = bm::numerics_from(&b.cfg, true);
    let sd = bm::local_sd();
    let times: Vec<f64> = (0..max_frames).map(|i| i as f64 * frame_dt_s).collect();
    let g8 = ignis_bass::observation::projector(&grid.xi, &vec![true; grid.k]);
    let mut rng = StdRng::seed_from_u64(seed);
    let mut out = Arrays::new();
    let mut work = dy::SimWork::default();
    let mut tr = dy::Trajectory::default();
    let mut kept = 0;
    const MAXT: usize = 64;
    while kept < n_tasks {
        let k = rng.random_range(0..b.members.len());
        let z: Vec<f64> = (0..bm::N_LOCAL)
            .map(|_| rng.sample::<f64, _>(StandardNormal))
            .collect();
        let x: Vec<f64> = z.iter().zip(&sd).map(|(a, s)| a * s).collect();
        let c_raw: Vec<f64> = b
            .feats
            .iter()
            .map(|f| {
                if f.transform == "time_since" {
                    0.0
                } else {
                    let n = &b.norm[&f.name];
                    rng.random_range(n["min"]..=n["max"])
                }
            })
            .collect();
        let cx = dy::Ctx {
            p: &b.members[k].g,
            net: &lay.net,
            phys: &lay.phys,
            grid: &grid,
            num,
            l: length_m,
            w: width_m,
        };
        dy::simulate(
            &cx,
            &x[..8],
            &x[8..],
            &times,
            &ZeroCond {
                c_raw: &c_raw,
                na: &b.na,
            },
            false,
            &mut work,
            &mut tr,
        );
        if tr.unreached > 0.0 || tr.frames.iter().any(|v| *v <= 0.0 || *v >= length_m) {
            continue;
        }
        let n_obs = rng.random_range(1..=max_frames); // cutoff after n_obs frames, independent of the outcome
        let sn = b.members[k].g[lay.lsn].exp();
        let c_norm: Vec<f64> = (0..c_raw.len())
            .map(|j| (c_raw[j] - b.na.center[j]) / b.na.scale[j])
            .collect();
        let mut tok = vec![0.0; MAXT * TOKEN_DIM];
        let mut mask = vec![0.0; MAXT];
        for j in 0..n_obs {
            let noisy: Vec<f64> = (0..grid.k)
                .map(|i| tr.frames[j * grid.k + i] + sn * rng.sample::<f64, _>(StandardNormal))
                .collect();
            for mm in 0..8 {
                let coef: f64 = (0..grid.k)
                    .map(|i| g8[mm * grid.k + i] * noisy[i])
                    .sum::<f64>()
                    / length_m;
                let r = j * 8 + mm;
                if r >= MAXT {
                    break;
                }
                let t = &mut tok[r * TOKEN_DIM..(r + 1) * TOKEN_DIM];
                t[0] = (times[j] / 10.0).ln_1p();
                t[1] = if j > 0 {
                    (frame_dt_s / 10.0).ln_1p()
                } else {
                    0.0
                };
                t[2] = coef;
                t[3] = (sn / length_m).ln();
                t[4] = 1.0;
                t[5] = 1.0;
                t[6] = 1.0;
                t[8 + mm] = 1.0;
                t[16..24].copy_from_slice(&c_norm[..8]);
                mask[r] = 1.0;
            }
        }
        let mut meta = vec![0.0; META_DIM];
        meta[..8].copy_from_slice(&c_norm[..8]);
        let tl = (times[n_obs - 1] / 10.0).ln_1p();
        meta[8..16].copy_from_slice(&[tl, (n_obs as f64).ln_1p(), 0.0, tl, 0.0, 1.0, 1.0, 1.0]);
        push(&mut out, "tokens", &tok);
        push(&mut out, "mask", &mask);
        push(&mut out, "meta", &meta);
        push(&mut out, "gsum", &gs[k]);
        push(&mut out, "latent", &z);
        push(&mut out, "draw", &[k as f64]);
        push(&mut out, "n_frames", &[n_obs as f64]);
        kept += 1;
    }
    if let Some(t) = out.get_mut("tokens") {
        t.0 = vec![t.0[0], MAXT, TOKEN_DIM];
    }
    Ok(out)
}

pub fn write_tasks(path: &Path, arr: &Arrays, manifest: &Value) -> anyhow::Result<()> {
    if let Some(d) = path.parent() {
        std::fs::create_dir_all(d)?;
    }
    let npy: BTreeMap<String, NpyArray> = arr
        .iter()
        .map(|(k, (s, v))| (k.clone(), NpyArray::from_f64(s.clone(), v)))
        .collect();
    write_npz(path, &npy)?;
    std::fs::write(
        path.with_extension("json"),
        serde_json::to_vec_pretty(manifest)?,
    )?;
    Ok(())
}

pub fn load_tasks(path: &Path) -> anyhow::Result<(Arrays, Value)> {
    let arr = read_npz(path)?
        .into_iter()
        .map(|(k, a)| (k, (a.shape.clone(), a.to_f64())))
        .collect();
    let manifest = serde_json::from_slice(
        &std::fs::read(path.with_extension("json")).context("task manifest")?,
    )?;
    Ok((arr, manifest))
}

/// Generate tasks for a release and write `out` (+ `.json` manifest).
pub fn generate(
    r: &LoadedRelease,
    n_tasks: usize,
    out: &Path,
    seed: u64,
    options: &Value,
) -> anyhow::Result<Value> {
    let fam = r.release.family.as_str();
    let arr = if fam.starts_with("acme") {
        let stops: Vec<Option<f64>> = options["stop_choices_s"]
            .as_array()
            .map(|a| a.iter().map(|v| v.as_f64()).collect())
            .unwrap_or(vec![None]);
        acme_tasks(
            r,
            n_tasks,
            seed,
            options["cutoff_fraction"].as_f64().unwrap_or(0.8),
            &stops,
        )?
    } else if fam == "bass" {
        let g = |k: &str, d: f64| options[k].as_f64().unwrap_or(d);
        bass_tasks(
            r,
            n_tasks,
            seed,
            g("frame_dt_s", 1.0),
            g("max_frames", 8.0) as usize,
            g("width_m", 0.04),
            g("length_m", 0.1),
        )?
    } else {
        bail!("no task generator for {fam}");
    };
    let latent_dim = arr["latent"].0[1];
    let manifest = json!({"label": LABEL, "evidence_class": "model_output", "family": fam, "release_id": r.release.id,
        "n_tasks": arr["latent"].0[0], "latent_dim": latent_dim, "seed": seed,
        "cutoff_rule": "uniform, independent of the final duration; kept only if active/in support at cutoff",
        "token_fields": TOKEN_FIELDS, "metadata_fields": METADATA_FIELDS, "options": options});
    write_tasks(out, &arr, &manifest)?;
    Ok(manifest)
}
