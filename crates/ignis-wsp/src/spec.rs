//! Small constructors for [`ModelSpec`]s and typed read access to an assembled release's tensors.
//!
//! A family's spec is defined once, in code, next to the forward that implements it; the JSON under
//! `configs/models/<family>/wsp/` is its export (checked by a drift test) and is what an author is
//! handed. A forward only runs a release whose manifest names exactly the spec it was compiled with.

use anyhow::{anyhow, ensure};

use ignis_core::contracts::{
    Finite, ModelRelease, ModelSpec, TensorFill, TensorRole, TensorSpec, ValueBounds,
};
use ignis_core::expert::Refusal;
use ignis_core::registry::TensorMap;

fn fin(v: f64) -> Finite {
    Finite::new(v).expect("spec literals are finite")
}

fn bounds(b: Option<(f64, f64)>) -> Option<ValueBounds> {
    b.map(|(lo, hi)| ValueBounds {
        min: fin(lo),
        max: fin(hi),
    })
}

/// A tensor whose every element the root patch must write.
pub fn authored(
    name: &str,
    shape: &[u32],
    role: TensorRole,
    unit: Option<&str>,
    range: Option<(f64, f64)>,
    description: &str,
) -> TensorSpec {
    TensorSpec {
        name: name.into(),
        shape: shape.to_vec(),
        role,
        fill: TensorFill::AuthorWritesAll,
        unit: unit.map(str::to_string),
        bounds: bounds(range),
        description: description.into(),
    }
}

/// A sparse tensor: elements the patch does not write are zero by declaration.
pub fn sparse(
    name: &str,
    shape: &[u32],
    role: TensorRole,
    range: Option<(f64, f64)>,
    description: &str,
) -> TensorSpec {
    TensorSpec {
        name: name.into(),
        shape: shape.to_vec(),
        role,
        fill: TensorFill::StructuralZero,
        unit: None,
        bounds: bounds(range),
        description: description.into(),
    }
}

/// A scalar fixed by the spec (normalisation references, numerical settings, support limits).
pub fn constant(name: &str, value: f64, unit: &str, description: &str) -> TensorSpec {
    TensorSpec {
        name: name.into(),
        shape: vec![],
        role: TensorRole::Constant,
        fill: TensorFill::Fixed {
            values: vec![fin(value)],
        },
        unit: Some(unit.into()),
        bounds: None,
        description: description.into(),
    }
}

/// Borrowed, shape-checked view of a release's tensors.
pub struct Params<'a> {
    spec: &'a ModelSpec,
    tensors: &'a TensorMap,
}

impl<'a> Params<'a> {
    /// Binds tensors to the spec the forward was compiled with. A release built from any other spec
    /// (another architecture, another constant) is refused instead of being reinterpreted.
    pub fn bind(
        spec: &'a ModelSpec,
        release: &ModelRelease,
        tensors: &'a TensorMap,
    ) -> Result<Self, Refusal> {
        spec.check_release(release).map_err(|e| {
            Refusal::unsupported(
                "spec_mismatch",
                format!(
                    "release {} was not built from spec {} ({}): {e}",
                    release.id,
                    spec.spec_id,
                    spec.spec_hash()
                ),
            )
        })?;
        let p = Params { spec, tensors };
        for t in &spec.tensors {
            p.slice(&t.name)
                .map_err(|e| Refusal::failed("release_tensors", e.to_string()))?;
        }
        Ok(p)
    }

    pub fn slice(&self, name: &str) -> anyhow::Result<&'a [f64]> {
        let spec = self
            .spec
            .tensor(name)
            .ok_or_else(|| anyhow!("tensor {name} is not in the spec"))?;
        let t = self
            .tensors
            .get(name)
            .ok_or_else(|| anyhow!("release lacks tensor {name}"))?;
        ensure!(
            t.shape
                .iter()
                .map(|&d| d as u32)
                .eq(spec.shape.iter().copied()),
            "tensor {name} has shape {:?}, spec says {:?}",
            t.shape,
            spec.shape
        );
        ensure!(
            t.data.iter().all(|v| v.is_finite()),
            "tensor {name} holds a non-finite value"
        );
        Ok(&t.data)
    }

    /// A tensor known to exist (checked by [`Params::bind`]).
    pub fn v(&self, name: &str) -> &'a [f64] {
        self.slice(name).expect("bound tensors are present")
    }

    pub fn s(&self, name: &str) -> f64 {
        self.v(name)[0]
    }
}
