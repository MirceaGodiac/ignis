//! `ignis vision`: probe, frames, contours. Decoding commands are guarded (dry-run by default).

use std::path::{Path, PathBuf};

use anyhow::bail;
use clap::Subcommand;
use ignis_core::contracts::EvidenceClass;
use ignis_core::resources::{guard, load_profile, Guard, Job};
use serde_json::json;

use crate::calibration::{calibrate, CalibrationSpec};
use crate::pipeline::{default_out_root, run_contours, RunParams, Segmentation};
use crate::sam2::Sam2Config;
use crate::segment::ClassicalSegmenter;
use crate::video::{probe, FrameCache};

/// Planning rates for the guard (frames per second).
const CLASSICAL_FPS_EST: f64 = 250.0;
const SAM2_FPS_EST: f64 = 1.0;
const DECODE_FPS_EST: f64 = 400.0;

#[derive(Subcommand, Debug)]
pub enum VisionCmd {
    /// Container metadata and every sample PTS (demux only, no decoding).
    Probe { file: PathBuf },
    /// Extract frames as PNG into the cache keyed by source sha256 + frame index (incremental, bounded).
    Frames {
        file: PathBuf,
        /// Keep every N-th frame (presentation order).
        #[arg(long, default_value_t = 30)]
        every: usize,
        #[arg(long, default_value_t = 0)]
        start: usize,
        #[arg(long)]
        stop: Option<usize>,
        /// Cache bound per source; extra frames are refused.
        #[arg(long, default_value_t = 500)]
        max_frames: usize,
        #[arg(long, default_value = "cpu-local")]
        profile: String,
        /// Actually decode and write PNGs.
        #[arg(long)]
        execute: bool,
    },
    /// Causal segmentation + leading-luminous contour on the fixed s grid → a VisualObservationBundle:
    /// metric specs write ContourObservation JSONL (metres), pixel-only specs ImageContourObservation.
    Contours {
        file: PathBuf,
        /// Calibration spec JSON (see configs/vision/).
        #[arg(long)]
        calibration: PathBuf,
        /// classical | sam2
        #[arg(long, default_value = "classical")]
        method: String,
        /// ExperimentRecord id; default: the spec's single `applies_to`, cross-checked with the
        /// clip → run map (else `unmapped:<file stem>`, identity unknown).
        #[arg(long)]
        run_id: Option<String>,
        #[arg(long, default_value_t = 1)]
        every: usize,
        /// First frame (default: the spec's `frames.start`, else 0).
        #[arg(long)]
        start: Option<usize>,
        /// Frame after the last (default: the spec's `frames.stop`, else the end).
        #[arg(long)]
        stop: Option<usize>,
        /// t = 0 on the container PTS axis (default: the spec's `frames.time_origin_s`, else 0).
        #[arg(long)]
        time_origin_s: Option<f64>,
        /// Camera / view id (default: the spec's `camera_id`).
        #[arg(long)]
        camera_id: Option<String>,
        #[arg(long, default_value_t = 32)]
        k: usize,
        /// Mark records as synthetic (generated clip).
        #[arg(long)]
        synthetic: bool,
        #[arg(long, default_value = "cpu-local")]
        profile: String,
        #[arg(long)]
        out_root: Option<PathBuf>,
        #[arg(long)]
        execute: bool,
    },
    /// Visual observation bundles a local research scope (train + validation) may read.
    Bundles,
    /// One visual observation bundle (scope-checked).
    Bundle { bundle_id: String },
    /// Rename pre-0.3.0 `contours.jsonl` files holding pixels under the metric contract (dry-run
    /// unless --execute; reversible: bytes unchanged, file renamed to contours.pixel-legacy.jsonl).
    QuarantineLegacy {
        #[arg(long)]
        out_root: Option<PathBuf>,
        #[arg(long)]
        execute: bool,
    },
}

/// The scope of a local researcher at this terminal (no test partition).
fn local_research_scope() -> ignis_core::contracts::ExecutionScope {
    use ignis_core::contracts::*;
    ExecutionScope {
        scope_id: ScopeId::new("local-cli").expect("id"),
        workspace_id: WorkspaceId::new("local").expect("id"),
        principal: PrincipalId::new("local-researcher").expect("id"),
        role: ScopeRole::Research,
        partitions: [Partition::Train, Partition::Validation].into(),
        capabilities: [Capability::ReadData].into(),
        corpus_snapshot: None,
        parent_scope_id: None,
    }
}

fn config_dir() -> PathBuf {
    ignis_core::resources::configs_dir().join("vision")
}

fn load_toml(name: &str) -> anyhow::Result<toml::Value> {
    let p = config_dir().join(name);
    if !p.exists() {
        return Ok(toml::Value::Table(Default::default()));
    }
    Ok(toml::from_str(&std::fs::read_to_string(p)?)?)
}

fn n_selected(n: usize, start: usize, stop: Option<usize>, every: usize) -> usize {
    let end = stop.unwrap_or(n).min(n);
    if end <= start {
        0
    } else {
        (end - start).div_ceil(every.max(1))
    }
}

pub fn run(cmd: VisionCmd) -> anyhow::Result<()> {
    match cmd {
        VisionCmd::Probe { file } => println!("{}", serde_json::to_string_pretty(&probe(&file)?)?),
        VisionCmd::Frames {
            file,
            every,
            start,
            stop,
            max_frames,
            profile,
            execute,
        } => {
            let p = probe(&file)?;
            let prof = load_profile(&profile)?;
            let n_sel = n_selected(p.n_frames, start, stop, every);
            let detail = json!({"file": file.display().to_string(), "sha256": p.sha256, "frames_total": p.n_frames,
                                "frames_selected": n_sel, "cache_bound": max_frames});
            if guard(
                "vision.frames",
                &prof,
                p.n_frames as f64 / DECODE_FPS_EST / 3600.0,
                execute,
                detail,
            )? == Guard::DryRun
            {
                return Ok(());
            }
            let cache = FrameCache::new(&p.sha256, &FrameCache::default_root(), max_frames);
            let job = Job::start(
                "vision.frames",
                &prof,
                json!({"file": file.display().to_string(), "every": every}),
            )?;
            let res = cache.extract(&file, every, start, stop)?;
            job.finish(res.clone());
            let mut out = json!({"cache": cache.dir.display().to_string()});
            if let (Some(o), Some(r)) = (out.as_object_mut(), res.as_object()) {
                o.extend(r.clone());
            }
            println!("{}", serde_json::to_string_pretty(&out)?);
        }
        VisionCmd::Contours {
            file,
            calibration,
            method,
            run_id,
            every,
            start,
            stop,
            time_origin_s,
            camera_id,
            k,
            synthetic,
            profile,
            out_root,
            execute,
        } => {
            let p = probe(&file)?;
            let mut spec = CalibrationSpec::load(&calibration)?;
            let window = spec.frames.clone();
            let start = start.or(window.as_ref().map(|w| w.start)).unwrap_or(0);
            let stop = stop.or(window.as_ref().map(|w| w.stop));
            let (time_origin_s, time_origin) = match (time_origin_s, &window) {
                (Some(t), _) => (
                    t,
                    format!("container PTS {t} s (declared on the command line; not adjudicated)"),
                ),
                (None, Some(w)) => (
                    w.time_origin_s,
                    format!("container PTS {} s: {}", w.time_origin_s, w.rule),
                ),
                (None, None) => (
                    0.0,
                    "container PTS 0 s (start of the clip; not an event)".to_string(),
                ),
            };
            let camera_id = camera_id
                .or(spec.camera_id.clone())
                .unwrap_or_else(|| "unspecified".into());
            let file_name = file
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            let identity = crate::bundle::resolve_identity(
                &file_name,
                run_id.as_deref(),
                &spec.applies_to,
                &crate::bundle::clip_map()?,
            )?;
            if spec.pixel_only && spec.image_size_px.is_none() {
                spec.image_size_px = Some([p.width, p.height]);
            }
            let cal = calibrate(&spec, Some(&p.sha256), 400, 0)?;
            let seg_cfg = load_toml("segmentation.toml")?;
            let mut classical: ClassicalSegmenter = match seg_cfg.get("classical") {
                Some(t) => t.clone().try_into()?,
                None => ClassicalSegmenter::default(),
            };
            classical.exclude_rois_px = spec.occlusion_rois_px.clone();
            let (seg, fps) = match method.as_str() {
                "classical" => (Segmentation::Classical(classical), CLASSICAL_FPS_EST),
                "sam2" => {
                    let cfg: Sam2Config = load_toml("sam2.toml")?.try_into()?;
                    (
                        Segmentation::Sam2 {
                            cfg,
                            prompt_from: classical,
                        },
                        SAM2_FPS_EST,
                    )
                }
                other => bail!("method must be classical or sam2, got {other:?}"),
            };
            let n_sel = n_selected(p.n_frames, start, stop, every);
            let prof = load_profile(&profile)?;
            let action = format!("vision.contours.{method}");
            let detail = json!({"file": file.display().to_string(), "sha256": p.sha256, "frames_selected": n_sel,
                                "calibration_id": cal.record.id, "units": cal.units(), "calibration_resolution": cal.record.resolution,
                                "run": identity, "camera_id": camera_id, "time_origin": time_origin});
            if guard(&action, &prof, n_sel as f64 / fps / 3600.0, execute, detail)? == Guard::DryRun
            {
                return Ok(());
            }
            let rid = identity
                .run_id
                .as_ref()
                .map(|r| r.as_str().to_string())
                .unwrap_or_else(|| {
                    format!(
                        "unmapped:{}",
                        file.file_stem()
                            .map(|s| s.to_string_lossy())
                            .unwrap_or_default()
                    )
                });
            let params = RunParams {
                run_id: rid,
                run: identity,
                camera_id,
                time_origin,
                every,
                start,
                stop,
                time_origin_s,
                k,
                evidence_class: if synthetic {
                    EvidenceClass::Synthetic
                } else {
                    EvidenceClass::IgnisDerived
                },
                out_root: out_root.unwrap_or_else(default_out_root),
            };
            let job = Job::start(
                &action,
                &prof,
                json!({"file": file.display().to_string(), "every": every, "method": method}),
            )?;
            let res = run_contours(Path::new(&file), &p, &cal, seg, &params, &prof.pool())?;
            job.finish(res.clone());
            println!("{}", serde_json::to_string_pretty(&res)?);
        }
        VisionCmd::Bundles => {
            let store = crate::bundle::VisualBundleStore::open_default()?;
            let scope = local_research_scope();
            let rows: Vec<serde_json::Value> = store
                .all()?
                .into_iter()
                .map(|(dir, b)| match store.admit(&scope, &b) {
                    Ok(()) => json!({"bundle_id": b.bundle_id, "run": b.run, "camera_id": b.camera_id,
                        "units_status": b.units_status, "contract": b.contours.contract, "method": b.method,
                        "dir": dir.display().to_string()}),
                    Err(e) => json!({"bundle_id": b.bundle_id, "refused": e.code, "reason": e.message}),
                })
                .collect();
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({"scope": scope.scope_id, "bundles": rows}))?
            );
        }
        VisionCmd::Bundle { bundle_id } => {
            let store = crate::bundle::VisualBundleStore::open_default()?;
            let id = ignis_core::contracts::VisualBundleId::new(bundle_id)?;
            match store.bundle(&local_research_scope(), &id) {
                Ok(b) => println!("{}", serde_json::to_string_pretty(&b)?),
                Err(e) => {
                    println!("{}", serde_json::to_string_pretty(&json!({"error": e}))?);
                    bail!("{:?}: {}", e.code, e.message);
                }
            }
        }
        VisionCmd::QuarantineLegacy { out_root, execute } => {
            let root = out_root.unwrap_or_else(default_out_root);
            let files = crate::bundle::legacy_pixel_contours(&root)?;
            let list: Vec<String> = files.iter().map(|f| f.display().to_string()).collect();
            if !execute {
                println!(
                    "{}",
                    serde_json::to_string_pretty(
                        &json!({"dry_run": true, "legacy_pixel_contours": list,
                    "action": "rename to contours.pixel-legacy.jsonl (bytes unchanged)", "hint": "re-run with --execute"})
                    )?
                );
                return Ok(());
            }
            let moved = crate::bundle::quarantine_legacy(&files)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({"renamed": moved}))?
            );
        }
    }
    Ok(())
}
