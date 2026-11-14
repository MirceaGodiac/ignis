//! Shared plumbing: roots, refusals, JSON emission, split loading, a seeded RNG and NumPy-exact
//! numerical helpers (`interp`, `quantile`, `linspace`, `%g`).

use std::path::{Path, PathBuf};

use anyhow::Context;
use ignis_core::contracts::Split;
use ignis_core::hashing::hex_part;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

/// An evaluation or lab rule forbids the requested operation. The message names the rule.
#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct Refused(pub String);

pub fn refused(msg: impl Into<String>) -> anyhow::Error {
    anyhow::Error::new(Refused(msg.into()))
}

fn env_path(var: &str, default: &str) -> PathBuf {
    std::env::var_os(var)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(default))
}

pub fn artifacts_root() -> PathBuf {
    env_path("IGNIS_ARTIFACTS", "artifacts")
}

pub fn holdout_root() -> PathBuf {
    env_path("IGNIS_HOLDOUT", "holdout")
}

pub fn eval_root() -> PathBuf {
    env_path("IGNIS_RUNS", "runs").join("evaluation")
}

pub fn splits_dir() -> PathBuf {
    env_path("IGNIS_SPLITS", "data/work/splits")
}

/// A split JSON file path, or a split id resolved under `IGNIS_SPLITS` or the committed `data/splits/`
/// (`<family>-<hex16>.json`). Accepts both a bare `Split` and the frozen-split file written by
/// `ignis data split freeze` (`{"split": {...}, ...}`).
pub fn load_split_file(reference: &str) -> anyhow::Result<Option<Split>> {
    let hex = hex_part(reference).to_string();
    let mut candidates = vec![
        PathBuf::from(reference),
        splits_dir().join(format!("{hex}.json")),
    ];
    if hex.len() >= 16 {
        if let Ok(rd) = std::fs::read_dir(env_path("IGNIS_FROZEN_SPLITS", "data/splits")) {
            candidates.extend(rd.flatten().map(|e| e.path()).filter(|p| {
                p.file_stem()
                    .and_then(|s| s.to_str())
                    .is_some_and(|s| s.ends_with(&hex[..16]))
            }));
        }
    }
    let Some(p) = candidates.into_iter().find(|p| p.is_file()) else {
        return Ok(None);
    };
    let text = std::fs::read_to_string(&p)?;
    let mut v: Value =
        serde_json::from_str(&text).with_context(|| format!("split {}", p.display()))?;
    if let Some(inner) = v.get_mut("split").map(Value::take) {
        v = inner;
    }
    let split: Split =
        serde_json::from_value(v).with_context(|| format!("split {}", p.display()))?;
    if hex.len() == 64 && hex_part(&split.id) != hex {
        anyhow::bail!(
            "split file {} holds {}, not {reference}",
            p.display(),
            split.id
        );
    }
    Ok(Some(split))
}

pub fn load_split(reference: &str) -> anyhow::Result<Split> {
    load_split_file(reference)?.ok_or_else(|| {
        anyhow::anyhow!(
            "split {reference:?} not found (path or {}/<id>.json)",
            splits_dir().display()
        )
    })
}

/// Pretty JSON (NaN/inf already become `null` through serde_json).
pub fn dumps(v: &Value) -> String {
    serde_json::to_string_pretty(v).expect("json values serialize")
}

/// Print the result (or a refusal object) and return the process exit code (0 or 2).
pub fn emit(res: anyhow::Result<Value>, out: Option<&Path>) -> i32 {
    match res {
        Ok(v) => {
            if let Some(o) = out {
                if let Some(d) = o.parent() {
                    let _ = std::fs::create_dir_all(d);
                }
                if let Err(e) = std::fs::write(o, dumps(&v) + "\n") {
                    return emit(Err(e.into()), None);
                }
                println!("{}", dumps(&json!({"written": o.display().to_string()})));
            } else {
                println!("{}", dumps(&v));
            }
            0
        }
        Err(e) => {
            let is_refusal = e.downcast_ref::<Refused>().is_some();
            let kind = if is_refusal { "Refused" } else { "Error" };
            let msg = format!("{e:#}");
            println!(
                "{}",
                dumps(&json!({"refused": is_refusal, "error": format!("{kind}: {msg}")}))
            );
            2
        }
    }
}

pub fn write_json_file(path: &Path, v: &Value) -> anyhow::Result<()> {
    if let Some(d) = path.parent() {
        std::fs::create_dir_all(d)?;
    }
    std::fs::write(path, dumps(v) + "\n")?;
    Ok(())
}

/// Comma-separated finite non-negative cutoffs in seconds.
pub fn parse_cutoffs(text: &str) -> anyhow::Result<Vec<f64>> {
    let v: Vec<f64> = text
        .split(',')
        .filter(|x| !x.trim().is_empty())
        .map(|x| x.trim().parse::<f64>())
        .collect::<Result<_, _>>()
        .map_err(|e| refused(format!("--cutoffs: {e}")))?;
    if v.is_empty() || v.iter().any(|x| !x.is_finite() || *x < 0.0) {
        return Err(refused(
            "--cutoffs: give non-negative finite seconds, e.g. 1,3,5",
        ));
    }
    Ok(v)
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

pub fn now_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true)
}

/// Write-once file: identical content is a no-op, different content is refused.
pub fn write_once(path: &Path, text: &str) -> anyhow::Result<()> {
    if path.exists() {
        if std::fs::read_to_string(path)? != text {
            return Err(refused(format!(
                "{} already exists with different content (write-once)",
                path.display()
            )));
        }
        return Ok(());
    }
    if let Some(d) = path.parent() {
        std::fs::create_dir_all(d)?;
    }
    let tmp = path.with_file_name(format!(
        "{}.tmp",
        path.file_name().unwrap().to_string_lossy()
    ));
    std::fs::write(&tmp, text)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

// ------------------------------------------------------------------------------------- RNG

/// xoshiro256++ seeded through SplitMix64. Deterministic across platforms and thread counts.
#[derive(Clone)]
pub struct Rng {
    s: [u64; 4],
    spare: Option<f64>,
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        let mut x = seed;
        let mut next = || {
            x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
            let mut z = x;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            z ^ (z >> 31)
        };
        Rng {
            s: [next(), next(), next(), next()],
            spare: None,
        }
    }
    /// Stream keyed by (seed, label): e.g. one stream per tensor name or per bootstrap replicate.
    pub fn keyed(seed: u64, label: &str) -> Self {
        let d = Sha256::digest(format!("{seed}|{label}").as_bytes());
        Rng::new(u64::from_le_bytes(d[..8].try_into().unwrap()))
    }
    pub fn next_u64(&mut self) -> u64 {
        let s = &mut self.s;
        let r = s[0].wrapping_add(s[3]).rotate_left(23).wrapping_add(s[0]);
        let t = s[1] << 17;
        s[2] ^= s[0];
        s[3] ^= s[1];
        s[1] ^= s[2];
        s[0] ^= s[3];
        s[2] ^= t;
        s[3] = s[3].rotate_left(45);
        r
    }
    /// Uniform in [0, 1).
    pub fn uniform(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
    }
    /// Uniform integer in [0, n) (Lemire, unbiased).
    pub fn below(&mut self, n: usize) -> usize {
        let n = n as u64;
        loop {
            let m = (self.next_u64() as u128) * (n as u128);
            let lo = m as u64;
            if lo >= n.wrapping_neg() % n {
                return (m >> 64) as usize;
            }
        }
    }
    pub fn normal(&mut self) -> f64 {
        if let Some(z) = self.spare.take() {
            return z;
        }
        loop {
            let u = 2.0 * self.uniform() - 1.0;
            let v = 2.0 * self.uniform() - 1.0;
            let s = u * u + v * v;
            if s > 0.0 && s < 1.0 {
                let f = (-2.0 * s.ln() / s).sqrt();
                self.spare = Some(v * f);
                return u * f;
            }
        }
    }
}

// ------------------------------------------------------------------------------ numerics

/// `np.interp(x, xp, fp)` for increasing `xp` (clamped at the ends), same arithmetic as NumPy.
pub fn interp(x: f64, xp: &[f64], fp: &[f64]) -> f64 {
    let n = xp.len();
    if n == 0 {
        return f64::NAN;
    }
    if x.is_nan() {
        return f64::NAN;
    }
    if n == 1 {
        return fp[0];
    }
    if x <= xp[0] {
        return fp[0];
    }
    if x >= xp[n - 1] {
        return fp[n - 1];
    }
    // j such that xp[j] <= x < xp[j+1]
    let j = xp.partition_point(|&v| v <= x) - 1;
    if x == xp[j] {
        return fp[j];
    }
    let slope = (fp[j + 1] - fp[j]) / (xp[j + 1] - xp[j]);
    let r = slope * (x - xp[j]) + fp[j];
    if r.is_nan() {
        let r2 = slope * (x - xp[j + 1]) + fp[j + 1];
        if r2.is_nan() && fp[j] == fp[j + 1] {
            return fp[j];
        }
        return r2;
    }
    r
}

/// `np.linspace(a, b, n)` (endpoint included, last value exactly `b`).
pub fn linspace(a: f64, b: f64, n: usize) -> Vec<f64> {
    match n {
        0 => vec![],
        1 => vec![a],
        _ => {
            let div = (n - 1) as f64;
            let step = (b - a) / div;
            let mut v: Vec<f64> = (0..n).map(|i| a + i as f64 * step).collect();
            if step == 0.0 {
                v = (0..n).map(|i| a + (i as f64 / div) * (b - a)).collect();
            }
            v[n - 1] = b;
            v
        }
    }
}

/// `np.quantile(sorted, q)` with the default linear method; `sorted` must be ascending.
pub fn quantile_sorted(sorted: &[f64], q: f64) -> f64 {
    let n = sorted.len();
    if n == 0 {
        return f64::NAN;
    }
    let pos = q * (n - 1) as f64;
    let lo = pos.floor() as usize;
    let hi = (lo + 1).min(n - 1);
    let g = pos - lo as f64;
    // NumPy `_lerp`: a + (b - a) * t, switching form for t >= 0.5.
    let (a, b) = (sorted[lo], sorted[hi]);
    let d = b - a;
    let r = a + d * g;
    if g >= 0.5 {
        b - d * (1.0 - g)
    } else {
        r
    }
}

pub fn mean(v: &[f64]) -> f64 {
    if v.is_empty() {
        return f64::NAN;
    }
    pairwise_sum(v) / v.len() as f64
}

/// NumPy pairwise summation (blocks of 8, unrolled, recursion above 128) so means agree to ~1 ulp.
pub fn pairwise_sum(a: &[f64]) -> f64 {
    let n = a.len();
    if n < 8 {
        let mut s = 0.0; // NumPy starts from -0.0 only for empty; 0.0 is fine here
        for &x in a {
            s += x;
        }
        s
    } else if n <= 128 {
        let mut r = [a[0], a[1], a[2], a[3], a[4], a[5], a[6], a[7]];
        let mut i = 8;
        while i + 8 <= n {
            for k in 0..8 {
                r[k] += a[i + k];
            }
            i += 8;
        }
        let mut res = ((r[0] + r[1]) + (r[2] + r[3])) + ((r[4] + r[5]) + (r[6] + r[7]));
        while i < n {
            res += a[i];
            i += 1;
        }
        res
    } else {
        let mut n2 = n / 2;
        n2 -= n2 % 8;
        pairwise_sum(&a[..n2]) + pairwise_sum(&a[n2..])
    }
}

/// Python `f"{x:g}"`.
pub fn fmt_g(x: f64) -> String {
    fmt_g_prec(x, 6)
}

pub fn fmt_g_prec(x: f64, p: usize) -> String {
    if x.is_nan() {
        return "nan".into();
    }
    if x.is_infinite() {
        return if x > 0.0 { "inf".into() } else { "-inf".into() };
    }
    if x == 0.0 {
        return if x.is_sign_negative() {
            "-0".into()
        } else {
            "0".into()
        };
    }
    let p = p.max(1);
    let sci = format!("{:.*e}", p - 1, x);
    let (mant, exp) = sci.split_once('e').unwrap();
    let exp: i32 = exp.parse().unwrap();
    if exp < -4 || exp >= p as i32 {
        let m = strip_zeros(mant);
        format!("{m}e{}{:02}", if exp < 0 { '-' } else { '+' }, exp.abs())
    } else {
        let decimals = (p as i32 - 1 - exp).max(0) as usize;
        strip_zeros(&format!("{:.*}", decimals, x))
    }
}

fn strip_zeros(s: &str) -> String {
    if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        s.to_string()
    }
}

/// Python `str(float)`-like rendering for grouping keys.
pub fn py_float(x: f64) -> String {
    if x.is_finite() && x == x.trunc() && x.abs() < 1e16 {
        format!("{x:.1}")
    } else if x.is_nan() {
        "nan".into()
    } else {
        format!("{x}")
    }
}

pub fn f64_of(v: &Value) -> Option<f64> {
    v.as_f64()
}

/// JSON array of numbers (null -> NaN).
pub fn vec_f64(v: &Value) -> Vec<f64> {
    v.as_array()
        .map(|a| a.iter().map(|x| x.as_f64().unwrap_or(f64::NAN)).collect())
        .unwrap_or_default()
}

pub fn mat_f64(v: &Value) -> Vec<Vec<f64>> {
    v.as_array()
        .map(|a| a.iter().map(vec_f64).collect())
        .unwrap_or_default()
}
