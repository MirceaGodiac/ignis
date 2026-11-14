//! Codex/Astra laboratory: inspect checkpoints, probe them on train/validation data, edit them
//! reproducibly into new releases. JSON output everywhere; tensors are bounded unless written to a file.
//!
//! Final-test outcomes are unreachable from here: views pass [`guard`], every entry point holds a
//! [`crate::access::LabSession`] (the vault refuses while one is active) and lab code cannot mint the
//! vault's access token.

pub mod edits;
pub mod guard;
pub mod inspect;
pub mod nd;
pub mod probe;
pub mod resume;

use std::path::PathBuf;

use clap::Subcommand;
use ignis_core::expert::Expert;
use ignis_core::views::ViewSource;
use serde_json::{json, Map, Value};

use crate::access::LabSession;
use crate::util::{emit, refused};

#[derive(Debug, Subcommand)]
pub enum LabCmd {
    /// Architecture, config, priors, numerics, lineage chain and tensor table with statistics.
    Show {
        release: String,
        #[arg(long)]
        no_posterior: bool,
    },
    /// Statistics, bounded slice values, histogram; full access only via --out.
    Tensor {
        release: String,
        name: String,
        /// e.g. "0:4,:"
        #[arg(long, allow_hyphen_values = true)]
        slice: Option<String>,
        /// Histogram bins.
        #[arg(long)]
        hist: Option<usize>,
        /// Write the (sliced) tensor to .npy.
        #[arg(long)]
        out: Option<PathBuf>,
        /// Read from posterior draws.
        #[arg(long)]
        posterior: bool,
        #[arg(long, default_value_t = inspect::MAX_INLINE)]
        max_elements: usize,
    },
    /// Parent chain with lineage kinds and edit manifests.
    Lineage { release: String },
    /// Tensor-wise deltas, field diffs and the lineage kinds between two releases.
    Diff { a: String, b: String },
    /// Activation statistics from the expert's activations(params, view) hook.
    Activations {
        release: String,
        #[arg(long)]
        cohort: String,
        #[arg(long)]
        split: String,
        #[arg(long, default_value = "validation")]
        partition: String,
        #[arg(long, default_value_t = 8)]
        limit: usize,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Objective value and gradient norms per tensor on train or validation views.
    Grad {
        release: String,
        #[arg(long)]
        cohort: String,
        #[arg(long)]
        split: String,
        #[arg(long, default_value = "validation")]
        partition: String,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// 1-D (or 2-D with --direction2) loss slice along random/grad/file directions.
    Sensitivity {
        release: String,
        #[arg(long)]
        cohort: String,
        #[arg(long)]
        split: String,
        #[arg(long, default_value = "validation")]
        partition: String,
        #[arg(long, default_value = "random")]
        direction: String,
        #[arg(long)]
        direction2: Option<String>,
        #[arg(long, default_value_t = 1.0)]
        span: f64,
        #[arg(long, default_value_t = 21)]
        points: usize,
        #[arg(long, default_value_t = 0)]
        seed: u64,
        #[arg(long, default_value = "filter")]
        normalize: String,
        /// Regex of tensors; default all trainable.
        #[arg(long)]
        tensors: Option<String>,
    },
    /// Per-run objective grouped by condition, VALIDATION partition only.
    Residuals {
        release: String,
        #[arg(long)]
        cohort: String,
        #[arg(long)]
        split: String,
        /// Condition keys to group by, comma-separated.
        #[arg(long)]
        by: Option<String>,
    },
    /// One op from flags: edit manifest, then a NEW release (parent unchanged).
    Edit(Box<EditArgs>),
    /// Apply an edit JSON written by Codex/Astra (ops list; see `ignis_lab::lab::edits`).
    Apply {
        release: String,
        edit_file: PathBuf,
        #[arg(long)]
        dry_run: bool,
    },
    /// Continue training via `ignis fit run ... --resume <release>` (dry-run unless --execute). Extra args pass through.
    Resume {
        release: String,
        /// configs/resources/<name>
        #[arg(long)]
        profile: String,
        #[arg(long)]
        execute: bool,
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        extra: Vec<String>,
    },
}

#[derive(Debug, clap::Args)]
pub struct EditArgs {
    pub release: String,
    /// set|add|scale|clip|low_rank_add|reinit|reparameterize|freeze|unfreeze|set_field|from_npy|create|remove|rename|reshape|pad|select
    #[arg(long)]
    pub op: Option<String>,
    /// Apply a tensor file (op from_npy): `--from-npy delta.npy --tensor net.w [--slice ..] [--mode add]`.
    #[arg(long)]
    pub from_npy: Option<String>,
    #[arg(long)]
    pub tensor: Option<String>,
    /// Comma-separated.
    #[arg(long)]
    pub tensors: Option<String>,
    #[arg(long)]
    pub pattern: Option<String>,
    #[arg(long, allow_hyphen_values = true)]
    pub slice: Option<String>,
    /// JSON value.
    #[arg(long, allow_hyphen_values = true)]
    pub value: Option<String>,
    #[arg(long, allow_hyphen_values = true)]
    pub factor: Option<f64>,
    #[arg(long)]
    pub rank: Option<u64>,
    #[arg(long)]
    pub seed: Option<u64>,
    #[arg(long)]
    pub std: Option<f64>,
    #[arg(long, allow_hyphen_values = true)]
    pub alpha: Option<f64>,
    /// Factor A (.npy) or affine a.
    #[arg(long, allow_hyphen_values = true)]
    pub a: Option<String>,
    /// Factor B (.npy) or affine b.
    #[arg(long, allow_hyphen_values = true)]
    pub b: Option<String>,
    #[arg(long)]
    pub init: Option<String>,
    #[arg(long)]
    pub map: Option<String>,
    #[arg(long)]
    pub field: Option<String>,
    #[arg(long)]
    pub key: Option<String>,
    #[arg(long)]
    pub kind: Option<String>,
    /// from_npy: set|add.
    #[arg(long)]
    pub mode: Option<String>,
    /// rename target.
    #[arg(long)]
    pub to: Option<String>,
    /// create/reshape shape as JSON, e.g. "[4,2]".
    #[arg(long)]
    pub shape: Option<String>,
    /// pad widths as JSON, e.g. "[[0,0],[0,1]]".
    #[arg(long)]
    pub pad: Option<String>,
    #[arg(long, allow_hyphen_values = true)]
    pub min: Option<f64>,
    #[arg(long, allow_hyphen_values = true)]
    pub max: Option<f64>,
    #[arg(long)]
    pub note: Option<String>,
    #[arg(long)]
    pub dry_run: bool,
}

fn edit_spec(a: &EditArgs) -> anyhow::Result<Value> {
    let op = match (&a.op, &a.from_npy) {
        (Some(o), _) => o.clone(),
        (None, Some(_)) => "from_npy".into(),
        (None, None) => return Err(refused("give --op <op> or --from-npy <file.npy>")),
    };
    let mut s = Map::new();
    s.insert("op".into(), json!(op));
    let strs = [
        ("tensor", &a.tensor),
        ("pattern", &a.pattern),
        ("slice", &a.slice),
        ("init", &a.init),
        ("map", &a.map),
        ("field", &a.field),
        ("key", &a.key),
        ("kind", &a.kind),
        ("mode", &a.mode),
        ("to", &a.to),
        ("file", &a.from_npy),
    ];
    for (k, v) in strs {
        if let Some(v) = v {
            s.insert(k.into(), json!(v));
        }
    }
    for (k, v) in [
        ("factor", a.factor),
        ("std", a.std),
        ("alpha", a.alpha),
        ("min", a.min),
        ("max", a.max),
    ] {
        if let Some(v) = v {
            s.insert(k.into(), json!(v));
        }
    }
    for (k, v) in [("rank", a.rank), ("seed", a.seed)] {
        if let Some(v) = v {
            s.insert(k.into(), json!(v));
        }
    }
    if let Some(t) = &a.tensors {
        s.insert("tensors".into(), json!(t.split(',').collect::<Vec<_>>()));
    }
    for (k, v) in [("value", &a.value), ("shape", &a.shape), ("pad", &a.pad)] {
        if let Some(v) = v {
            s.insert(
                k.into(),
                serde_json::from_str(v).map_err(|e| refused(format!("--{k} is not JSON: {e}")))?,
            );
        }
    }
    if op == "reparameterize" && a.map.as_deref() == Some("affine") {
        let num = |x: &Option<String>, d: f64| -> anyhow::Result<f64> {
            x.as_deref().map_or(Ok(d), |v| {
                v.parse()
                    .map_err(|_| refused("affine --a/--b must be numbers"))
            })
        };
        s.insert("a".into(), json!(num(&a.a, 1.0)?));
        s.insert("b".into(), json!(num(&a.b, 0.0)?));
    } else if a.a.is_some() || a.b.is_some() {
        s.insert("A".into(), json!(a.a));
        s.insert("B".into(), json!(a.b));
    }
    Ok(json!({"ops": [Value::Object(s)], "note": a.note}))
}

/// `ignis lab ...`. Prints JSON; returns the process exit code (2 on refusal/error).
pub fn run(cmd: LabCmd, experts: &[Box<dyn Expert>], views: &dyn ViewSource) -> i32 {
    let _lab = LabSession::enter();
    let ctx = probe::Ctx {
        experts,
        source: views,
    };
    let res = match cmd {
        LabCmd::Show {
            release,
            no_posterior,
        } => inspect::show(&release, !no_posterior),
        LabCmd::Tensor {
            release,
            name,
            slice,
            hist,
            out,
            posterior,
            max_elements,
        } => inspect::tensor(
            &release,
            &name,
            slice.as_deref(),
            hist,
            out.as_deref(),
            posterior,
            max_elements,
        ),
        LabCmd::Lineage { release } => inspect::lineage(&release),
        LabCmd::Diff { a, b } => inspect::diff(&a, &b),
        LabCmd::Activations {
            release,
            cohort,
            split,
            partition,
            limit,
            out,
        } => probe::activations(
            &ctx,
            &release,
            &cohort,
            &split,
            &partition,
            limit,
            out.as_deref(),
        ),
        LabCmd::Grad {
            release,
            cohort,
            split,
            partition,
            out,
        } => probe::grad(&ctx, &release, &cohort, &split, &partition, out.as_deref()),
        LabCmd::Sensitivity {
            release,
            cohort,
            split,
            partition,
            direction,
            direction2,
            span,
            points,
            seed,
            normalize,
            tensors,
        } => {
            let o = probe::SensitivityOpts {
                partition: &partition,
                direction: &direction,
                direction2: direction2.as_deref(),
                span,
                points,
                seed,
                normalize: &normalize,
                tensors: tensors.as_deref(),
            };
            probe::sensitivity(&ctx, &release, &cohort, &split, &o)
        }
        LabCmd::Residuals {
            release,
            cohort,
            split,
            by,
        } => probe::residuals(
            &ctx,
            &release,
            &cohort,
            &split,
            by.map(|b| b.split(',').map(String::from).collect()),
        ),
        LabCmd::Edit(a) => {
            edit_spec(&a).and_then(|s| edits::apply_edit(&a.release, s, None, a.dry_run))
        }
        LabCmd::Apply {
            release,
            edit_file,
            dry_run,
        } => (|| {
            let spec: Value = serde_json::from_str(&std::fs::read_to_string(&edit_file)?)?;
            edits::apply_edit(&release, spec, edit_file.parent(), dry_run)
        })(),
        LabCmd::Resume {
            release,
            profile,
            execute,
            extra,
        } => resume::resume(&release, &profile, &extra, execute),
    };
    emit(res, None)
}
