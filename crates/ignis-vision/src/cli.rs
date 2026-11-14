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
    /// Causal segmentation + leading-luminous contour on the fixed s grid → ContourObservation JSONL.
    Contours {
        file: PathBuf,
        /// Calibration spec JSON (see configs/vision/).
        #[arg(long)]
        calibration: PathBuf,
        /// classical | sam2
        #[arg(long, default_value = "classical")]
        method: String,
        /// ExperimentRecord id; default `unmapped:<file stem>`.
        #[arg(long)]
        run_id: Option<String>,
        #[arg(long, default_value_t = 1)]
        every: usize,
        #[arg(long, default_value_t = 0)]
        start: usize,
        #[arg(long)]
        stop: Option<usize>,
        /// Adjudicated t=0 on the container PTS axis.
        #[arg(long, default_value_t = 0.0)]
        time_origin_s: f64,
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
            k,
            synthetic,
            profile,
            out_root,
            execute,
        } => {
            let p = probe(&file)?;
            let mut spec = CalibrationSpec::load(&calibration)?;
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
                                "calibration_id": cal.record.id, "units": cal.units(), "calibration_resolution": cal.record.resolution});
            if guard(&action, &prof, n_sel as f64 / fps / 3600.0, execute, detail)? == Guard::DryRun
            {
                return Ok(());
            }
            let rid = run_id.unwrap_or_else(|| {
                format!(
                    "unmapped:{}",
                    file.file_stem()
                        .map(|s| s.to_string_lossy())
                        .unwrap_or_default()
                )
            });
            let params = RunParams {
                run_id: rid,
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
    }
    Ok(())
}
