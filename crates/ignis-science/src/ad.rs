//! Reverse-mode automatic differentiation for small FP64 models.
//!
//! Decision (measured, see `benches`/report): the FLEX likelihood is a long chain of *scalar*
//! operations (Kalman recursions, bracketed Newton inversions, Gauss–Legendre sums over a 16-unit
//! tanh network). Tensor frameworks (candle) pay a per-op dispatch/allocation cost of ~1 µs, which
//! is 100× the arithmetic here. We therefore use:
//!
//! * a thread-local Wengert tape of `f64` nodes ([`Var`]): every op pushes one node with its local
//!   partials; constants never touch the tape; buffers are reused across evaluations (no
//!   allocation in the inner loop once warm);
//! * hand-fused custom nodes ([`Real::custom`]) for hot kernels whose partials are cheap to derive
//!   analytically (the FLEX integrand sums), verified against finite differences;
//! * one generic code path: model code is written once over `R: Real` and instantiated with `f64`
//!   (values only, zero overhead) or [`Var`] (value + gradient).
//!
//! A tape belongs to one thread. A computation that records on a tape must not call into Rayon in
//! the middle (work stealing could interleave another tape user); parallelise *around* tape
//! sessions (one session per run / chain), never inside one.

use std::cell::RefCell;
use std::fmt::Debug;
use std::ops::{Add, AddAssign, Div, Mul, MulAssign, Neg, Sub, SubAssign};

/// Scalar type of the generic model code.
pub trait Real:
    Copy
    + Debug
    + Send
    + Sync
    + 'static
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + Neg<Output = Self>
    + Add<f64, Output = Self>
    + Sub<f64, Output = Self>
    + Mul<f64, Output = Self>
    + Div<f64, Output = Self>
    + AddAssign
    + SubAssign
    + MulAssign
{
    /// True when partials are recorded (skip computing them otherwise).
    const GRAD: bool;
    fn val(self) -> f64;
    fn cst(v: f64) -> Self;
    fn exp(self) -> Self;
    fn ln(self) -> Self;
    fn ln_1p(self) -> Self;
    fn expm1(self) -> Self;
    fn tanh(self) -> Self;
    fn sqrt(self) -> Self;
    fn recip(self) -> Self {
        Self::cst(1.0) / self
    }
    fn sq(self) -> Self {
        self * self
    }
    /// A node with value `v` whose partials w.r.t. `parents` are `partials`.
    fn custom(v: f64, parents: &[Self], partials: &[f64]) -> Self;
    /// `log(e^a + e^b)` (stable).
    fn logaddexp(self, b: Self) -> Self {
        let (x, y) = (self.val(), b.val());
        let m = x.max(y);
        if m == f64::NEG_INFINITY {
            return Self::cst(f64::NEG_INFINITY);
        }
        let (ea, eb) = ((x - m).exp(), (y - m).exp());
        let s = ea + eb;
        Self::custom(m + s.ln(), &[self, b], &[ea / s, eb / s])
    }
    /// Value-carrying copy whose derivative is zero (JAX `stop_gradient`).
    fn detach(self) -> Self {
        Self::cst(self.val())
    }
}

impl Real for f64 {
    const GRAD: bool = false;
    #[inline(always)]
    fn val(self) -> f64 {
        self
    }
    #[inline(always)]
    fn cst(v: f64) -> Self {
        v
    }
    #[inline(always)]
    fn exp(self) -> Self {
        f64::exp(self)
    }
    #[inline(always)]
    fn ln(self) -> Self {
        f64::ln(self)
    }
    #[inline(always)]
    fn ln_1p(self) -> Self {
        f64::ln_1p(self)
    }
    #[inline(always)]
    fn expm1(self) -> Self {
        f64::exp_m1(self)
    }
    #[inline(always)]
    fn tanh(self) -> Self {
        f64::tanh(self)
    }
    #[inline(always)]
    fn sqrt(self) -> Self {
        f64::sqrt(self)
    }
    #[inline(always)]
    fn custom(v: f64, _: &[Self], _: &[f64]) -> Self {
        v
    }
}

// ------------------------------------------------------------------------------------- tape

const CONST: u32 = u32::MAX;

#[derive(Default)]
struct Tape {
    /// CSR: node i has parents par[off[i]..off[i+1]].
    off: Vec<u32>,
    par: Vec<u32>,
    der: Vec<f64>,
    adj: Vec<f64>,
}

impl Tape {
    #[inline(always)]
    fn len(&self) -> u32 {
        (self.off.len() - 1) as u32
    }
    #[inline(always)]
    fn push(&mut self, ps: &[(u32, f64)]) -> u32 {
        for &(p, d) in ps {
            if p != CONST {
                self.par.push(p);
                self.der.push(d);
            }
        }
        let i = self.len();
        self.off.push(self.par.len() as u32);
        i
    }
}

thread_local! {
    static TAPE: RefCell<Tape> = RefCell::new(Tape { off: vec![0], ..Default::default() });
}

/// A recorded scalar: tape index + value. `Copy`, 16 bytes.
#[derive(Clone, Copy, Debug)]
pub struct Var {
    i: u32,
    v: f64,
}

impl Var {
    #[inline(always)]
    fn rec1(v: f64, a: Var, da: f64) -> Var {
        if a.i == CONST {
            return Var { i: CONST, v };
        }
        let i = TAPE.with(|t| t.borrow_mut().push(&[(a.i, da)]));
        Var { i, v }
    }
    #[inline(always)]
    fn rec2(v: f64, a: Var, da: f64, b: Var, db: f64) -> Var {
        if a.i == CONST && b.i == CONST {
            return Var { i: CONST, v };
        }
        let i = TAPE.with(|t| t.borrow_mut().push(&[(a.i, da), (b.i, db)]));
        Var { i, v }
    }
    pub fn is_const(self) -> bool {
        self.i == CONST
    }
}

/// Clear this thread's tape (keeps capacity).
pub fn reset() {
    TAPE.with(|t| {
        let mut t = t.borrow_mut();
        t.off.clear();
        t.off.push(0);
        t.par.clear();
        t.der.clear();
    });
}

/// New independent leaves (contiguous tape indices).
pub fn leaves(x: &[f64]) -> Vec<Var> {
    TAPE.with(|t| {
        let mut t = t.borrow_mut();
        x.iter().map(|&v| Var { i: t.push(&[]), v }).collect()
    })
}

/// Number of nodes on this thread's tape.
pub fn tape_len() -> usize {
    TAPE.with(|t| t.borrow().len() as usize)
}

/// Back-propagate from `out` and *add* d out / d leaf into `g` (leaves in the order given).
pub fn accumulate_grad(out: Var, leaves: &[Var], g: &mut [f64]) {
    if out.i == CONST {
        return;
    }
    TAPE.with(|t| {
        let mut guard = t.borrow_mut();
        let t = &mut *guard;
        let n = (out.i + 1) as usize;
        t.adj.clear();
        t.adj.resize(n, 0.0);
        t.adj[out.i as usize] = 1.0;
        for i in (0..n).rev() {
            let a = t.adj[i];
            if a == 0.0 {
                continue;
            }
            let (s, e) = (t.off[i] as usize, t.off[i + 1] as usize);
            for k in s..e {
                let p = t.par[k] as usize;
                t.adj[p] += a * t.der[k];
            }
        }
        for (gi, l) in g.iter_mut().zip(leaves) {
            if l.i != CONST && (l.i as usize) < n {
                *gi += t.adj[l.i as usize];
            }
        }
    });
}

/// Value and gradient of `f` at `x` on this thread's tape (resets it first and after).
pub fn value_and_grad(x: &[f64], f: impl FnOnce(&[Var]) -> Var) -> (f64, Vec<f64>) {
    reset();
    let xs = leaves(x);
    let y = f(&xs);
    let mut g = vec![0.0; x.len()];
    accumulate_grad(y, &xs, &mut g);
    reset();
    (y.v, g)
}

impl Real for Var {
    const GRAD: bool = true;
    #[inline(always)]
    fn val(self) -> f64 {
        self.v
    }
    #[inline(always)]
    fn cst(v: f64) -> Self {
        Var { i: CONST, v }
    }
    #[inline(always)]
    fn exp(self) -> Self {
        let e = self.v.exp();
        Var::rec1(e, self, e)
    }
    #[inline(always)]
    fn ln(self) -> Self {
        Var::rec1(self.v.ln(), self, 1.0 / self.v)
    }
    #[inline(always)]
    fn ln_1p(self) -> Self {
        Var::rec1(self.v.ln_1p(), self, 1.0 / (1.0 + self.v))
    }
    #[inline(always)]
    fn expm1(self) -> Self {
        Var::rec1(self.v.exp_m1(), self, self.v.exp())
    }
    #[inline(always)]
    fn tanh(self) -> Self {
        let t = self.v.tanh();
        Var::rec1(t, self, 1.0 - t * t)
    }
    #[inline(always)]
    fn sqrt(self) -> Self {
        let s = self.v.sqrt();
        Var::rec1(s, self, 0.5 / s)
    }
    fn custom(v: f64, parents: &[Self], partials: &[f64]) -> Self {
        if parents.iter().all(|p| p.i == CONST) {
            return Var { i: CONST, v };
        }
        let i = TAPE.with(|t| {
            let mut t = t.borrow_mut();
            for (p, &d) in parents.iter().zip(partials) {
                if p.i != CONST && d != 0.0 {
                    t.par.push(p.i);
                    t.der.push(d);
                }
            }
            let i = t.len();
            let n = t.par.len() as u32;
            t.off.push(n);
            i
        });
        Var { i, v }
    }
}

macro_rules! binops {
    () => {
        impl Add for Var {
            type Output = Var;
            #[inline(always)]
            fn add(self, b: Var) -> Var {
                Var::rec2(self.v + b.v, self, 1.0, b, 1.0)
            }
        }
        impl Sub for Var {
            type Output = Var;
            #[inline(always)]
            fn sub(self, b: Var) -> Var {
                Var::rec2(self.v - b.v, self, 1.0, b, -1.0)
            }
        }
        impl Mul for Var {
            type Output = Var;
            #[inline(always)]
            fn mul(self, b: Var) -> Var {
                Var::rec2(self.v * b.v, self, b.v, b, self.v)
            }
        }
        impl Div for Var {
            type Output = Var;
            #[inline(always)]
            fn div(self, b: Var) -> Var {
                let q = self.v / b.v;
                Var::rec2(q, self, 1.0 / b.v, b, -q / b.v)
            }
        }
        impl Neg for Var {
            type Output = Var;
            #[inline(always)]
            fn neg(self) -> Var {
                Var::rec1(-self.v, self, -1.0)
            }
        }
        impl Add<f64> for Var {
            type Output = Var;
            #[inline(always)]
            fn add(self, b: f64) -> Var {
                Var::rec1(self.v + b, self, 1.0)
            }
        }
        impl Sub<f64> for Var {
            type Output = Var;
            #[inline(always)]
            fn sub(self, b: f64) -> Var {
                Var::rec1(self.v - b, self, 1.0)
            }
        }
        impl Mul<f64> for Var {
            type Output = Var;
            #[inline(always)]
            fn mul(self, b: f64) -> Var {
                Var::rec1(self.v * b, self, b)
            }
        }
        impl Div<f64> for Var {
            type Output = Var;
            #[inline(always)]
            fn div(self, b: f64) -> Var {
                Var::rec1(self.v / b, self, 1.0 / b)
            }
        }
        impl Add<Var> for f64 {
            type Output = Var;
            #[inline(always)]
            fn add(self, b: Var) -> Var {
                b + self
            }
        }
        impl Sub<Var> for f64 {
            type Output = Var;
            #[inline(always)]
            fn sub(self, b: Var) -> Var {
                Var::rec1(self - b.v, b, -1.0)
            }
        }
        impl Mul<Var> for f64 {
            type Output = Var;
            #[inline(always)]
            fn mul(self, b: Var) -> Var {
                b * self
            }
        }
        impl Div<Var> for f64 {
            type Output = Var;
            #[inline(always)]
            fn div(self, b: Var) -> Var {
                let q = self / b.v;
                Var::rec1(q, b, -q / b.v)
            }
        }
        impl AddAssign for Var {
            #[inline(always)]
            fn add_assign(&mut self, b: Var) {
                *self = *self + b;
            }
        }
        impl SubAssign for Var {
            #[inline(always)]
            fn sub_assign(&mut self, b: Var) {
                *self = *self - b;
            }
        }
        impl MulAssign for Var {
            #[inline(always)]
            fn mul_assign(&mut self, b: Var) {
                *self = *self * b;
            }
        }
    };
}
binops!();

/// Dot product of a generic vector with an `f64` vector (one fused node).
pub fn dot_cst<R: Real>(a: &[R], b: &[f64]) -> R {
    let v: f64 = a.iter().zip(b).map(|(x, y)| x.val() * y).sum();
    if R::GRAD {
        R::custom(v, a, b)
    } else {
        R::cst(v)
    }
}

/// Sum of a generic vector (one fused node).
pub fn sum<R: Real>(a: &[R]) -> R {
    let v: f64 = a.iter().map(|x| x.val()).sum();
    if R::GRAD {
        let ones = vec![1.0; a.len()];
        R::custom(v, a, &ones)
    } else {
        R::cst(v)
    }
}

/// Central finite-difference gradient (checks only).
pub fn fd_grad(x: &[f64], h: f64, f: impl Fn(&[f64]) -> f64) -> Vec<f64> {
    let mut xp = x.to_vec();
    (0..x.len())
        .map(|i| {
            let hi = h * x[i].abs().max(1.0);
            xp[i] = x[i] + hi;
            let a = f(&xp);
            xp[i] = x[i] - hi;
            let b = f(&xp);
            xp[i] = x[i];
            (a - b) / (2.0 * hi)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn f<R: Real>(x: &[R]) -> R {
        let a = (x[0] * x[1]).tanh() + (x[2] / x[0]).exp().ln_1p();
        a * a.sqrt() - (x[1] - 0.3).expm1() * 2.0 + x[0].logaddexp(x[2])
    }

    #[test]
    fn tape_matches_finite_differences() {
        let x = [0.7, -0.4, 1.3];
        let (v, g) = value_and_grad(&x, |xs| f(xs));
        assert!((v - f(&x[..])).abs() < 1e-15);
        let fd = fd_grad(&x, 1e-6, |y| f(y));
        for (a, b) in g.iter().zip(&fd) {
            assert!((a - b).abs() < 1e-8 * b.abs().max(1.0), "{a} vs {b}");
        }
    }
}
