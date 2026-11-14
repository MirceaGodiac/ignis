//! Parameter transforms with log-Jacobians and the prior densities on natural coordinates.
//!
//! Convention (fixed in every model manifest): samplers and optimisers work on *unconstrained*
//! coordinates x. A transform maps x -> y (natural coordinate). When a prior is declared on y, the
//! density on x is `prior(y) + log|dy/dx|`. When a prior is declared directly on the unconstrained
//! coordinate (e.g. a Gaussian on log D0, V3.1 §10.2) the transform is `Identity` and no Jacobian
//! is added — adding one would count the change of variables twice.

use serde_json::{json, Value};

use crate::ad::Real;

pub const LOG_2PI: f64 = 1.837_877_066_409_345_5;
pub const LN_2: f64 = std::f64::consts::LN_2;

#[derive(Debug, Clone, PartialEq)]
pub enum Transform {
    Identity,
    /// y = exp(x) > 0.
    Exp,
    /// y = tanh(x) in (-1, 1).
    Tanh,
    /// y = lo + (hi - lo) sigmoid(x).
    Sigmoid {
        lo: f64,
        hi: f64,
    },
}

impl Transform {
    pub fn name(&self) -> &'static str {
        match self {
            Transform::Identity => "identity",
            Transform::Exp => "exp",
            Transform::Tanh => "tanh",
            Transform::Sigmoid { .. } => "sigmoid",
        }
    }
    pub fn forward<R: Real>(&self, x: R) -> R {
        match self {
            Transform::Identity => x,
            Transform::Exp => x.exp(),
            Transform::Tanh => x.tanh(),
            Transform::Sigmoid { lo, hi } => R::cst(1.0) / ((-x).exp() + 1.0) * (hi - lo) + *lo,
        }
    }
    pub fn inverse(&self, y: f64) -> f64 {
        match self {
            Transform::Identity => y,
            Transform::Exp => y.ln(),
            Transform::Tanh => y.atanh(),
            Transform::Sigmoid { lo, hi } => {
                let p = (y - lo) / (hi - lo);
                p.ln() - (-p).ln_1p()
            }
        }
    }
    /// log|dy/dx|.
    pub fn log_det_jacobian<R: Real>(&self, x: R) -> R {
        match self {
            Transform::Identity => R::cst(0.0),
            Transform::Exp => x,
            // log(1 - tanh^2) = log 4 - 2 logaddexp(x, -x)
            Transform::Tanh => R::cst(4f64.ln()) - x.logaddexp(-x) * 2.0,
            Transform::Sigmoid { lo, hi } => {
                R::cst((hi - lo).ln()) - R::cst(0.0).logaddexp(x) - R::cst(0.0).logaddexp(-x)
            }
        }
    }
}

pub fn normal_logpdf<R: Real>(y: R, loc: f64, scale: f64) -> R {
    let z = (y - loc) / scale;
    z * z * (-0.5) - (scale.ln() + 0.5 * LOG_2PI)
}

/// Density on y > 0.
pub fn half_normal_logpdf<R: Real>(y: R, scale: f64) -> R {
    let z = y / scale;
    z * z * (-0.5) + (LN_2 - scale.ln() - 0.5 * LOG_2PI)
}

/// LKJ(eta) on a 2x2 correlation: p(rho) = (1-rho^2)^(eta-1) / B(eta, eta) / 2^(2eta-1).
pub fn lkj_corr_2x2_logpdf<R: Real>(rho: R, eta: f64) -> R {
    let log_norm = (2.0 * eta - 1.0) * LN_2 + 2.0 * libm::lgamma(eta) - libm::lgamma(2.0 * eta);
    (-(rho * rho)).ln_1p() * (eta - 1.0) - log_norm
}

/// A declared prior on the natural coordinate of one parameter block (loc/scale broadcast).
#[derive(Debug, Clone, PartialEq)]
pub enum Prior {
    Normal { loc: Vec<f64>, scale: Vec<f64> },
    HalfNormal { scale: Vec<f64> },
    Lkj2 { eta: f64 },
}

fn bc(v: &[f64], i: usize) -> f64 {
    if v.len() == 1 {
        v[0]
    } else {
        v[i]
    }
}

impl Prior {
    pub fn normal(loc: f64, scale: f64) -> Prior {
        Prior::Normal {
            loc: vec![loc],
            scale: vec![scale],
        }
    }
    pub fn logpdf<R: Real>(&self, y: R, i: usize) -> R {
        match self {
            Prior::Normal { loc, scale } => normal_logpdf(y, bc(loc, i), bc(scale, i)),
            Prior::HalfNormal { scale } => half_normal_logpdf(y, bc(scale, i)),
            Prior::Lkj2 { eta } => lkj_corr_2x2_logpdf(y, *eta),
        }
    }
    pub fn describe(&self) -> Value {
        let l = |v: &[f64]| if v.len() == 1 { json!(v[0]) } else { json!(v) };
        match self {
            Prior::Normal { loc, scale } => {
                json!({"kind": "normal", "loc": l(loc), "scale": l(scale)})
            }
            Prior::HalfNormal { scale } => {
                json!({"kind": "half_normal", "loc": 0.0, "scale": l(scale)})
            }
            Prior::Lkj2 { eta } => json!({"kind": "lkj2", "eta": eta}),
        }
    }
    /// Scale of element i (used for initialisation from the prior).
    pub fn scale(&self, i: usize) -> f64 {
        match self {
            Prior::Normal { scale, .. } | Prior::HalfNormal { scale } => bc(scale, i),
            Prior::Lkj2 { .. } => 1.0,
        }
    }
}

/// One flat parameter block: unconstrained `name` (also its npz key) -> natural value.
#[derive(Debug, Clone, PartialEq)]
pub struct ParamSpec {
    pub name: String,
    pub shape: Vec<usize>,
    pub transform: Transform,
    pub prior: Prior,
    pub natural_name: Option<String>,
    pub unit: String,
}

impl ParamSpec {
    pub fn new(name: &str, shape: &[usize], transform: Transform, prior: Prior) -> Self {
        ParamSpec {
            name: name.into(),
            shape: shape.to_vec(),
            transform,
            prior,
            natural_name: None,
            unit: "1".into(),
        }
    }
    pub fn unit(mut self, u: &str) -> Self {
        self.unit = u.into();
        self
    }
    pub fn natural(mut self, n: &str) -> Self {
        self.natural_name = Some(n.into());
        self
    }
    pub fn size(&self) -> usize {
        self.shape.iter().product::<usize>()
    }
    /// Prior log density of the unconstrained block, Jacobian included.
    pub fn log_density<R: Real>(&self, x: &[R]) -> R {
        let mut acc = R::cst(0.0);
        for (i, &xi) in x.iter().enumerate() {
            let y = self.transform.forward(xi);
            acc += self.prior.logpdf(y, i) + self.transform.log_det_jacobian(xi);
        }
        acc
    }
    pub fn describe(&self) -> Value {
        json!({"shape": self.shape, "transform": self.transform.name(),
               "natural": self.natural_name.clone().unwrap_or_else(|| self.name.clone()),
               "unit": self.unit, "prior": self.prior.describe()})
    }
}

/// Offsets of named blocks inside one flat parameter vector.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Layout {
    pub specs: Vec<ParamSpec>,
    pub offsets: Vec<usize>,
    pub size: usize,
}

impl Layout {
    pub fn new(specs: Vec<ParamSpec>) -> Self {
        let mut offsets = Vec::with_capacity(specs.len());
        let mut size = 0;
        for s in &specs {
            offsets.push(size);
            size += s.size();
        }
        Layout {
            specs,
            offsets,
            size,
        }
    }
    pub fn index(&self, name: &str) -> Option<usize> {
        self.specs.iter().position(|s| s.name == name)
    }
    pub fn range(&self, name: &str) -> Option<std::ops::Range<usize>> {
        let i = self.index(name)?;
        Some(self.offsets[i]..self.offsets[i] + self.specs[i].size())
    }
    pub fn names(&self) -> Vec<String> {
        self.specs.iter().map(|s| s.name.clone()).collect()
    }
    /// Sum of the block prior densities on a flat vector.
    pub fn log_prior<R: Real>(&self, x: &[R]) -> R {
        let mut acc = R::cst(0.0);
        for (s, &o) in self.specs.iter().zip(&self.offsets) {
            acc += s.log_density(&x[o..o + s.size()]);
        }
        acc
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jacobians_match_numeric_derivative() {
        for t in [
            Transform::Exp,
            Transform::Tanh,
            Transform::Sigmoid { lo: -2.0, hi: 3.0 },
        ] {
            for x in [-3.0, -0.2, 0.0, 1.7] {
                let h = 1e-6;
                let d = (t.forward(x + h) - t.forward(x - h)) / (2.0 * h);
                assert!(
                    (t.log_det_jacobian(x) - d.abs().ln()).abs() < 1e-7,
                    "{t:?} {x}"
                );
                assert!((t.inverse(t.forward(x)) - x).abs() < 1e-9);
            }
        }
    }
}
