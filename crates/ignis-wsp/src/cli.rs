//! `ignis wsp …` — the literal-construction workflow. Every command is deterministic and cheap; none
//! reads observations or outcomes of any run, and none estimates a parameter.
//!
//! ```text
//! ignis wsp spec --family flex --out configs/models/flex/wsp/spec.json
//! ignis wsp compile --family flex --edits w0.edits --meta w0.meta.json --out w0.patch.json
//! ignis wsp check --family flex --patch w0.patch.json
//! ignis wsp assemble --family flex --patch w0.patch.json --binding configs/models/flex/wsp/binding.json
//! ignis wsp rewrite --family flex_methanol6 --patch d1.patch.json --binding binding.json
//! ignis wsp inspect --release sha256:…
//! ignis wsp forward --release sha256:… --prefix prefix.json
//! ```

use std::path::{Path, PathBuf};

use anyhow::{bail, Context};
use clap::Subcommand;
use serde_json::{json, Value};

use ignis_core::contracts::{
    Family, Finite, HandwrittenWeightPatch, LiteralOp, ModelSpec, PrefixBundle, TensorFill,
};
use ignis_core::registry::{self, LoadedRelease};

use crate::assemble::{self, PatchMeta};
use crate::{models, text, Binding};

#[derive(Subcommand)]
pub enum WspCmd {
    /// Print or export the frozen ModelSpec of a model, with its hash. `--family` takes a model key:
    /// flex, acme_normal, acme_inverse, bass, flex_methanol, flex_heptane, acme_normal2, acme_inverse2,
    /// flex_methanol2, flex_heptane2.
    Spec {
        #[arg(long)]
        family: String,
        /// Write the spec JSON here (otherwise print a summary).
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Compile an edits file (literal assignments) and its provenance into a HandwrittenWeightPatch.
    Compile {
        #[arg(long)]
        family: String,
        #[arg(long)]
        edits: PathBuf,
        #[arg(long)]
        meta: PathBuf,
        #[arg(long)]
        out: PathBuf,
    },
    /// Validate a patch against the frozen spec and report parameter counts.
    Check {
        #[arg(long)]
        family: String,
        #[arg(long)]
        patch: PathBuf,
        /// Registry root holding the parent of a revision.
        #[arg(long)]
        root: Option<PathBuf>,
    },
    /// Assemble spec + patch (+ parent) into an immutable release in the registry.
    Assemble {
        #[arg(long)]
        family: String,
        #[arg(long)]
        patch: PathBuf,
        /// Evaluation population the release is declared for (cohort, split, source manifest).
        #[arg(long)]
        binding: PathBuf,
        #[arg(long)]
        root: Option<PathBuf>,
        /// Also write the release record here.
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Build a complete literal rewrite for a new spec, retaining its actual parent as ancestry.
    Rewrite {
        #[arg(long)]
        family: String,
        #[arg(long)]
        patch: PathBuf,
        #[arg(long)]
        binding: PathBuf,
        #[arg(long)]
        root: Option<PathBuf>,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Validate a new-spec rewrite and report counts without writing a release.
    RewriteCheck {
        #[arg(long)]
        family: String,
        #[arg(long)]
        patch: PathBuf,
        #[arg(long)]
        binding: PathBuf,
        #[arg(long)]
        root: Option<PathBuf>,
    },
    /// Show a literal release: manifest, counts and every nonzero authored value as edits.
    Inspect {
        #[arg(long)]
        release: String,
        #[arg(long)]
        root: Option<PathBuf>,
        /// Print the values as an edits file only.
        #[arg(long)]
        edits: bool,
    },
    /// Run the strict forward of a release on a PrefixBundle file.
    Forward {
        #[arg(long)]
        release: String,
        #[arg(long)]
        prefix: PathBuf,
        #[arg(long)]
        settings: Option<String>,
        #[arg(long)]
        root: Option<PathBuf>,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Print the digest of a family's forward sources.
    ForwardHash {
        #[arg(long)]
        family: String,
    },
    /// Write the neutral root of a law slug (`<slug>.wsp`, law.rs) as `edits.wsp` and `meta.json`
    /// in a directory: generated from the spec alone, no claim.
    LawRoot {
        #[arg(long)]
        family: String,
        #[arg(long)]
        out: PathBuf,
    },
}

pub fn family_of(name: &str) -> anyhow::Result<Family> {
    Ok(match name {
        "flex" => Family::Flex,
        "acme_normal" => Family::AcmeNormal,
        "acme_inverse" => Family::AcmeInverse,
        "bass" => Family::Bass,
        other => bail!("unknown family {other:?} (flex, acme_normal, acme_inverse, bass)"),
    })
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> anyhow::Result<T> {
    let text = std::fs::read_to_string(path).with_context(|| path.display().to_string())?;
    serde_json::from_str(&text).with_context(|| path.display().to_string())
}

fn write_json<T: serde::Serialize>(path: &Path, v: &T) -> anyhow::Result<()> {
    if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(path, serde_json::to_string_pretty(v)? + "\n")
        .with_context(|| path.display().to_string())
}

fn root_of(root: Option<PathBuf>) -> PathBuf {
    root.unwrap_or_else(registry::default_root)
}

fn parent_of(patch: &HandwrittenWeightPatch, root: &Path) -> anyhow::Result<Option<LoadedRelease>> {
    match &patch.parent_release_id {
        None => Ok(None),
        Some(id) => Ok(Some(
            registry::load(root, id.as_str()).with_context(|| format!("parent release {id}"))?,
        )),
    }
}

/// Every authored value of a release as single-element statements, in spec order. Zeros of sparse
/// tensors are omitted (they are zero by declaration); everything else is printed.
pub fn edits_of(spec: &ModelSpec, release: &LoadedRelease) -> String {
    let mut out = String::new();
    for t in &spec.tensors {
        if matches!(t.fill, TensorFill::Fixed { .. }) {
            continue;
        }
        let Some(tensor) = release.params.get(&t.name) else {
            continue;
        };
        for (flat, v) in tensor.data.iter().enumerate() {
            if *v == 0.0 && t.fill == TensorFill::StructuralZero {
                continue;
            }
            let mut index = vec![0u32; t.shape.len()];
            let mut rem = flat;
            for (axis, d) in t.shape.iter().enumerate().rev() {
                index[axis] = (rem % *d as usize) as u32;
                rem /= *d as usize;
            }
            let op = LiteralOp::Set {
                tensor: t.name.clone(),
                index,
                value: Finite::new(*v).expect("stored values are finite"),
            };
            out.push_str(&text::render(&op));
            out.push('\n');
        }
    }
    out
}

fn show(v: Value) -> anyhow::Result<()> {
    println!("{}", serde_json::to_string_pretty(&v)?);
    Ok(())
}

pub fn run(cmd: WspCmd) -> anyhow::Result<()> {
    match cmd {
        WspCmd::Spec { family, out } => {
            let model = models::by_key(&family)?;
            let spec = model.spec();
            spec.validate()?;
            if let Some(path) = &out {
                write_json(path, &spec)?;
            }
            let authored: usize = spec
                .tensors
                .iter()
                .filter(|t| !matches!(t.fill, TensorFill::Fixed { .. }))
                .map(|t| t.numel())
                .sum();
            show(json!({
                "spec_id": spec.spec_id,
                "spec_hash": spec.spec_hash(),
                "family": spec.family,
                "model_name": spec.model_name,
                "features": spec.features.len(),
                "tensors": spec.tensors.len(),
                "authored_elements": authored,
                "forward": spec.forward,
                "forward_hash": model.forward_hash(),
                "written_to": out,
            }))
        }
        WspCmd::Compile {
            family,
            edits,
            meta,
            out,
        } => {
            let spec = models::by_key(&family)?.spec();
            let src =
                std::fs::read_to_string(&edits).with_context(|| edits.display().to_string())?;
            let ops = text::compile(&src, &spec)?;
            let meta: PatchMeta = read_json(&meta)?;
            let patch = assemble::make_patch(&spec, meta, ops);
            patch.validate_against(&spec)?;
            write_json(&out, &patch)?;
            show(json!({
                "patch_id": patch.patch_id,
                "patch_hash": patch.patch_hash(),
                "spec_hash": patch.spec_hash,
                "ops": patch.ops.len(),
                "edits_sha256": ignis_core::hashing::sha256_hex(src.replace('\r', "").as_bytes()),
                "written_to": out,
            }))
        }
        WspCmd::Check {
            family,
            patch,
            root,
        } => {
            let spec = models::by_key(&family)?.spec();
            let patch: HandwrittenWeightPatch = read_json(&patch)?;
            let parent = parent_of(&patch, &root_of(root))?;
            let tensors = assemble::materialize(&spec, &patch, parent.as_ref())?;
            show(json!({
                "valid": true,
                "patch_hash": patch.patch_hash(),
                "construction": if patch.parent_release_id.is_some() {
                    "literal_revision"
                } else {
                    "literal_one_shot"
                },
                "counts": assemble::counts(&spec, &tensors),
            }))
        }
        WspCmd::Assemble {
            family,
            patch,
            binding,
            root,
            out,
        } => {
            let model = models::by_key(&family)?;
            let spec = model.spec();
            let patch: HandwrittenWeightPatch = read_json(&patch)?;
            let binding: Binding = read_json(&binding)?;
            let root = root_of(root);
            let parent = parent_of(&patch, &root)?;
            let tensors = assemble::materialize(&spec, &patch, parent.as_ref())?;
            let release = model.build_release(&patch, &tensors, parent.as_ref(), &binding)?;
            let saved = assemble::save(&root, release, &tensors)?;
            if let Some(path) = &out {
                write_json(path, &saved)?;
            }
            let m = saved.construction.as_ref().expect("literal release");
            show(json!({
                "release_id": saved.id,
                "model_name": saved.model_name,
                "state": saved.state,
                "trained_on": saved.trained_on,
                "construction_method": m.construction_method,
                "spec_hash": m.spec_hash,
                "patch_hash": m.patch_hash,
                "weights_hash": m.weights_hash,
                "forward_hash": m.forward_hash,
                "checks": m.checks,
                "counts": assemble::counts(&spec, &tensors),
                "directory": registry::release_dir(&root, &saved.id),
            }))
        }
        WspCmd::RewriteCheck {
            family,
            patch,
            binding,
            root,
        } => {
            let model = models::by_key(&family)?;
            let spec = model.spec();
            let patch: HandwrittenWeightPatch = read_json(&patch)?;
            let binding: Binding = read_json(&binding)?;
            let root = root_of(root);
            let parent =
                parent_of(&patch, &root)?.context("rewrite patch must name a parent release")?;
            let tensors = crate::rewrite::materialize(&spec, &patch, &parent, &binding)?;
            show(json!({
                "valid": true,
                "patch_hash": patch.patch_hash(),
                "construction": "literal_revision_full_rewrite",
                "parent_release_id": parent.release.id,
                "spec_hash": spec.spec_hash(),
                "counts": assemble::counts(&spec, &tensors),
            }))
        }
        WspCmd::Rewrite {
            family,
            patch,
            binding,
            root,
            out,
        } => {
            let model = models::by_key(&family)?;
            let spec = model.spec();
            let patch: HandwrittenWeightPatch = read_json(&patch)?;
            let binding: Binding = read_json(&binding)?;
            let root = root_of(root);
            let parent =
                parent_of(&patch, &root)?.context("rewrite patch must name a parent release")?;
            let tensors = crate::rewrite::materialize(&spec, &patch, &parent, &binding)?;
            let release = model.build_release(&patch, &tensors, Some(&parent), &binding)?;
            crate::rewrite::check_release_parent(&patch, &release, &parent)?;
            let saved = assemble::save(&root, release, &tensors)?;
            if let Some(path) = &out {
                write_json(path, &saved)?;
            }
            let manifest = saved.construction.as_ref().expect("literal release");
            show(json!({
                "release_id": saved.id,
                "parent_release_id": saved.parent_id,
                "model_name": saved.model_name,
                "state": saved.state,
                "trained_on": saved.trained_on,
                "construction_method": manifest.construction_method,
                "rewrite": true,
                "spec_hash": manifest.spec_hash,
                "patch_hash": manifest.patch_hash,
                "weights_hash": manifest.weights_hash,
                "forward_hash": manifest.forward_hash,
                "checks": manifest.checks,
                "counts": assemble::counts(&spec, &tensors),
                "directory": registry::release_dir(&root, &saved.id),
            }))
        }
        WspCmd::Inspect {
            release,
            root,
            edits,
        } => {
            let loaded = registry::load(&root_of(root), &release)?;
            let model = models::by_name(&loaded.release.model_name)
                .with_context(|| format!("{} is not a literal model", loaded.release.model_name))?;
            let spec = model.spec();
            spec.check_release(&loaded.release)?;
            if edits {
                print!("{}", edits_of(&spec, &loaded));
                return Ok(());
            }
            let m = loaded.release.construction.as_ref().expect("checked");
            let now = model.forward_hash();
            show(json!({
                "release_id": loaded.release.id,
                "model_name": loaded.release.model_name,
                "family": loaded.release.family,
                "state": loaded.release.state,
                "trained_on": loaded.release.trained_on,
                "lineage": loaded.release.lineage,
                "parent_id": loaded.release.parent_id,
                "construction": m,
                "counts": assemble::counts(&spec, &loaded.params),
                "forward_hash_now": now,
                "forward_matches": now == m.forward_hash,
            }))
        }
        WspCmd::Forward {
            release,
            prefix,
            settings,
            root,
            out,
        } => {
            let loaded = registry::load(&root_of(root), &release)?;
            let prefix: PrefixBundle = read_json(&prefix)?;
            let settings: Value = match settings {
                Some(s) => serde_json::from_str(&s).context("--settings")?,
                None => Value::Null,
            };
            let artifact = crate::artifact_of(&loaded, &prefix, &settings)
                .map_err(|r| anyhow::anyhow!("{r}"))?;
            artifact.validate().map_err(anyhow::Error::msg)?;
            match out {
                Some(path) => {
                    write_json(&path, &artifact)?;
                    println!("{}", artifact.id);
                    Ok(())
                }
                None => show(serde_json::to_value(&artifact)?),
            }
        }
        WspCmd::ForwardHash { family } => {
            println!("{}", models::by_key(&family)?.forward_hash());
            Ok(())
        }
        WspCmd::LawRoot { family, out } => {
            let model = models::by_key(&family)?;
            if !matches!(model.forward, models::Forward::Law(_)) {
                bail!("{family} is not a law slug (law_slugs::SLUGS)");
            }
            let spec = model.spec();
            let edits = crate::law_tools::root_edits(&spec);
            let meta = crate::law_tools::root_meta(&spec);
            // the generated root must compile against its own spec
            let ops = text::compile(&edits, &spec)?;
            assemble::make_patch(&spec, meta.clone(), ops).validate_against(&spec)?;
            std::fs::create_dir_all(&out).with_context(|| out.display().to_string())?;
            std::fs::write(out.join("edits.wsp"), &edits)
                .with_context(|| out.join("edits.wsp").display().to_string())?;
            write_json(&out.join("meta.json"), &meta)?;
            show(json!({
                "model_name": spec.model_name,
                "spec_id": spec.spec_id,
                "spec_hash": spec.spec_hash(),
                "edits": out.join("edits.wsp"),
                "meta": out.join("meta.json"),
            }))
        }
    }
}
