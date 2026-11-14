//! Minimal n-d helpers over row-major [`Tensor`]s: NumPy-style slice text, selection, broadcasting,
//! nested-JSON arrays and `.npy` files.

use std::path::Path;

use ignis_core::npz::{parse_npy, NpyArray};
use ignis_core::registry::Tensor;
use serde_json::Value;

use crate::util::refused;

#[derive(Debug, Clone, PartialEq)]
pub enum Idx {
    Int(i64),
    Range(Option<i64>, Option<i64>, Option<i64>),
    Ellipsis,
}

/// numpy-style slice text, e.g. `"0:4,:"`, `"-1"`, `"...,0"`, `"::2"`.
pub fn parse_slice(spec: Option<&str>, ndim: usize) -> anyhow::Result<Vec<Idx>> {
    let Some(spec) = spec.filter(|s| !s.trim().is_empty()) else {
        return Ok(vec![]);
    };
    let mut out = vec![];
    for part in spec.split(',') {
        let p = part.trim();
        if p == "..." {
            out.push(Idx::Ellipsis);
        } else if p.contains(':') {
            let bits: Vec<&str> = p.split(':').collect();
            if bits.len() > 3 {
                return Err(refused(format!("bad slice component {p:?}")));
            }
            let num = |s: &str| -> anyhow::Result<Option<i64>> {
                let s = s.trim();
                if s.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(s.parse().map_err(|_| {
                        refused(format!("bad slice component {p:?}"))
                    })?))
                }
            };
            out.push(Idx::Range(
                num(bits[0])?,
                num(bits[1])?,
                if bits.len() == 3 { num(bits[2])? } else { None },
            ));
        } else {
            out.push(Idx::Int(
                p.parse()
                    .map_err(|_| refused(format!("bad slice component {p:?}")))?,
            ));
        }
    }
    if out.iter().filter(|o| **o != Idx::Ellipsis).count() > ndim {
        return Err(refused(format!(
            "slice {spec:?} has more components than the tensor has dimensions ({ndim})"
        )));
    }
    if out.iter().filter(|o| **o == Idx::Ellipsis).count() > 1 {
        return Err(refused("an index can only have a single ellipsis ('...')"));
    }
    Ok(out)
}

/// Python `slice(start, stop, step).indices(len)` expanded.
fn slice_indices(
    start: Option<i64>,
    stop: Option<i64>,
    step: Option<i64>,
    len: usize,
) -> anyhow::Result<Vec<usize>> {
    let n = len as i64;
    let step = step.unwrap_or(1);
    if step == 0 {
        return Err(refused("slice step cannot be zero"));
    }
    let (lower, upper) = if step > 0 { (0, n) } else { (-1, n - 1) };
    let clamp = |v: Option<i64>, default: i64| -> i64 {
        match v {
            None => default,
            Some(mut x) => {
                if x < 0 {
                    x += n;
                    if x < lower {
                        x = lower;
                    }
                } else if x > upper {
                    x = upper;
                }
                x
            }
        }
    };
    let s = clamp(start, if step < 0 { upper } else { lower });
    let e = clamp(stop, if step < 0 { lower } else { upper });
    let mut out = vec![];
    let mut i = s;
    while (step > 0 && i < e) || (step < 0 && i > e) {
        out.push(i as usize);
        i += step;
    }
    Ok(out)
}

#[derive(Debug, Clone)]
pub struct Selection {
    pub shape: Vec<usize>,
    pub flat: Vec<usize>,
}

pub fn select(shape: &[usize], idx: &[Idx]) -> anyhow::Result<Selection> {
    let ndim = shape.len();
    let k = idx.iter().filter(|i| **i != Idx::Ellipsis).count();
    let mut full: Vec<Idx> = vec![];
    let mut had_ellipsis = false;
    for i in idx {
        if *i == Idx::Ellipsis {
            had_ellipsis = true;
            full.extend(std::iter::repeat(Idx::Range(None, None, None)).take(ndim - k));
        } else {
            full.push(i.clone());
        }
    }
    if !had_ellipsis {
        full.extend(std::iter::repeat(Idx::Range(None, None, None)).take(ndim - k));
    }
    let mut per_dim: Vec<Vec<usize>> = vec![];
    let mut out_shape = vec![];
    for (d, ix) in full.iter().enumerate() {
        match ix {
            Idx::Int(i) => {
                let n = shape[d] as i64;
                let j = if *i < 0 { i + n } else { *i };
                if j < 0 || j >= n {
                    return Err(refused(format!(
                        "index {i} is out of bounds for axis {d} with size {n}"
                    )));
                }
                per_dim.push(vec![j as usize]);
            }
            Idx::Range(a, b, c) => {
                let v = slice_indices(*a, *b, *c, shape[d])?;
                out_shape.push(v.len());
                per_dim.push(v);
            }
            Idx::Ellipsis => unreachable!(),
        }
    }
    let mut strides = vec![1usize; ndim];
    for d in (0..ndim.saturating_sub(1)).rev() {
        strides[d] = strides[d + 1] * shape[d + 1];
    }
    let mut flat = vec![0usize];
    for d in 0..ndim {
        let mut next = Vec::with_capacity(flat.len() * per_dim[d].len());
        for base in &flat {
            for &i in &per_dim[d] {
                next.push(base + i * strides[d]);
            }
        }
        flat = next;
    }
    Ok(Selection {
        shape: out_shape,
        flat,
    })
}

pub fn get(t: &Tensor, sel: &Selection) -> Tensor {
    Tensor {
        shape: sel.shape.clone(),
        data: sel.flat.iter().map(|&i| t.data[i]).collect(),
    }
}

/// NumPy broadcasting of `src` onto `dst` shape.
pub fn broadcast(src: &Tensor, dst: &[usize]) -> anyhow::Result<Vec<f64>> {
    let n = dst.len();
    if src.shape.len() > n {
        return Err(refused(format!(
            "could not broadcast shape {:?} into {:?}",
            src.shape, dst
        )));
    }
    let pad = n - src.shape.len();
    let sshape: Vec<usize> = std::iter::repeat(1)
        .take(pad)
        .chain(src.shape.iter().cloned())
        .collect();
    for d in 0..n {
        if sshape[d] != 1 && sshape[d] != dst[d] {
            return Err(refused(format!(
                "could not broadcast shape {:?} into {:?}",
                src.shape, dst
            )));
        }
    }
    let mut sstr = vec![0usize; n];
    let mut acc = 1;
    for d in (0..n).rev() {
        sstr[d] = if sshape[d] == 1 { 0 } else { acc };
        acc *= sshape[d];
    }
    let total: usize = dst.iter().product();
    let mut out = Vec::with_capacity(total);
    let mut idx = vec![0usize; n];
    for _ in 0..total {
        out.push(src.data[(0..n).map(|d| idx[d] * sstr[d]).sum::<usize>()]);
        for d in (0..n).rev() {
            idx[d] += 1;
            if idx[d] < dst[d] {
                break;
            }
            idx[d] = 0;
        }
    }
    Ok(out)
}

/// Write `vals` (already shaped like the selection) into the selected positions.
pub fn put(t: &mut Tensor, sel: &Selection, vals: &[f64]) {
    for (&i, &v) in sel.flat.iter().zip(vals) {
        t.data[i] = v;
    }
}

/// Nested JSON numbers -> tensor (ragged lists refused).
pub fn from_json(v: &Value) -> anyhow::Result<Tensor> {
    fn shape_of(v: &Value) -> anyhow::Result<Vec<usize>> {
        match v {
            Value::Array(a) => {
                if a.is_empty() {
                    return Ok(vec![0]);
                }
                let inner = shape_of(&a[0])?;
                for x in &a[1..] {
                    if shape_of(x)? != inner {
                        return Err(refused("ragged nested list"));
                    }
                }
                Ok(std::iter::once(a.len()).chain(inner).collect())
            }
            Value::Number(_) | Value::Bool(_) => Ok(vec![]),
            Value::Null => Ok(vec![]),
            _ => Err(refused(
                "tensor values must be numbers or nested lists of numbers",
            )),
        }
    }
    fn flatten(v: &Value, out: &mut Vec<f64>) {
        match v {
            Value::Array(a) => a.iter().for_each(|x| flatten(x, out)),
            Value::Bool(b) => out.push(if *b { 1.0 } else { 0.0 }),
            Value::Null => out.push(f64::NAN),
            x => out.push(x.as_f64().unwrap_or(f64::NAN)),
        }
    }
    let shape = shape_of(v)?;
    let mut data = vec![];
    flatten(v, &mut data);
    Ok(Tensor { shape, data })
}

/// Nested JSON lists from a tensor (non-finite -> null).
pub fn to_json(t: &Tensor) -> Value {
    fn rec(shape: &[usize], data: &[f64]) -> Value {
        if shape.is_empty() {
            return serde_json::json!(data[0]);
        }
        let step: usize = shape[1..].iter().product();
        Value::Array(
            (0..shape[0])
                .map(|i| rec(&shape[1..], &data[i * step..(i + 1) * step]))
                .collect(),
        )
    }
    rec(&t.shape, &t.data)
}

pub struct NpyFile {
    pub tensor: Tensor,
    pub dtype: &'static str,
    pub file_sha256: String,
}

pub fn read_npy(path: &Path) -> anyhow::Result<NpyFile> {
    let bytes =
        std::fs::read(path).map_err(|e| refused(format!("cannot read {}: {e}", path.display())))?;
    let a: NpyArray = parse_npy(&path.display().to_string(), &bytes)?;
    Ok(NpyFile {
        tensor: Tensor {
            shape: a.shape.clone(),
            data: a.to_f64(),
        },
        dtype: a.dtype.numpy_name(),
        file_sha256: crate::util::sha256_hex(&bytes),
    })
}

pub fn write_npy(path: &Path, t: &Tensor) -> anyhow::Result<()> {
    if let Some(d) = path.parent() {
        std::fs::create_dir_all(d)?;
    }
    std::fs::write(
        path,
        NpyArray::from_f64(t.shape.clone(), &t.data).to_npy_bytes(),
    )?;
    Ok(())
}

pub fn l2(v: &[f64]) -> f64 {
    v.iter().map(|x| x * x).sum::<f64>().sqrt()
}
