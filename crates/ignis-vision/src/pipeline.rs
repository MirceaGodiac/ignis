//! End-to-end causal contour run: decode → segment → contour → records + dependency manifest.
//!
//! Output directory (unique per run, write-once): `artifacts/vision/<sha16>/<method>-<runhash>/`
//!   calibration.json   the `Calibration` record
//!   contours.jsonl     one `ContourObservation` per processed frame (presentation order)
//!   manifest.json      probe, segmenter manifest (prompts, checkpoint), parameters, status counts,
//!                      timing, and the causal check (every dependency ≤ own frame)
//!
//! Classical: decoding runs on its own thread; frames are segmented in parallel batches on the
//! profile's bounded pool (a mask depends on its own frame only) and written in presentation order.
//! SAM2: strictly sequential through the sidecar.

use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::bail;
use ignis_core::contracts::{ContourObservation, EvidenceClass};
use rayon::prelude::*;
use serde_json::{json, Value};

use crate::calibration::PlaneCalibration;
use crate::contour::{make_observation, ContourExtractor, ObservationContext};
use crate::sam2::{Sam2Config, Sam2Sidecar, SAM2_METHOD_ID};
use crate::segment::{ClassicalSegmenter, MaskResult, CLASSICAL_METHOD_ID};
use crate::video::{iter_frames, Frame, VideoProbe};

pub enum Segmentation {
    Classical(ClassicalSegmenter),
    Sam2 {
        cfg: Sam2Config,
        prompt_from: ClassicalSegmenter,
    },
}

impl Segmentation {
    pub fn method_id(&self) -> &'static str {
        match self {
            Segmentation::Classical(_) => CLASSICAL_METHOD_ID,
            Segmentation::Sam2 { .. } => SAM2_METHOD_ID,
        }
    }
    fn plan_manifest(&self) -> Value {
        match self {
            Segmentation::Classical(c) => c.manifest(),
            Segmentation::Sam2 { cfg, prompt_from } => {
                json!({"method_id": SAM2_METHOD_ID, "config": cfg, "prompt_from": prompt_from.manifest()})
            }
        }
    }
}

pub struct RunParams {
    pub run_id: String,
    pub every: usize,
    pub start: usize,
    pub stop: Option<usize>,
    pub time_origin_s: f64,
    pub k: usize,
    pub evidence_class: EvidenceClass,
    pub out_root: PathBuf,
}

pub fn default_out_root() -> PathBuf {
    std::env::var_os("IGNIS_ARTIFACTS")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("artifacts"))
        .join("vision")
}

pub fn run_contours(
    video: &Path,
    probe: &VideoProbe,
    cal: &PlaneCalibration,
    seg: Segmentation,
    p: &RunParams,
    pool: &rayon::ThreadPool,
) -> anyhow::Result<Value> {
    let params = json!({
        "every": p.every, "start": p.start, "stop": p.stop, "time_origin_s": p.time_origin_s, "k": p.k,
        "run_id": p.run_id, "calibration_id": cal.record.id,
    });
    let key = ignis_core::hashing::key_of(
        &json!({"sha": probe.sha256, "seg": seg.plan_manifest(), "params": params}),
    );
    let run_key = ignis_core::hashing::hex_part(&key)[..12].to_string();
    let short = seg.method_id().rsplit('.').next().unwrap_or("run");
    let out = p
        .out_root
        .join(&probe.sha256[..16])
        .join(format!("{short}-{run_key}"));
    std::fs::create_dir_all(&out)?;
    std::fs::write(
        out.join("calibration.json"),
        serde_json::to_vec_pretty(&cal.record)?,
    )?;
    let (w, h) = (probe.width as usize, probe.height as usize);
    let ext = ContourExtractor::new(cal.clone(), p.k, w, h)?;
    let asset_path = video.file_name().map(|s| s.to_string_lossy().to_string());
    let ctx = ObservationContext {
        cal,
        run_id: &p.run_id,
        asset_sha256: &probe.sha256,
        asset_path: asset_path.as_deref(),
        time_origin_s: p.time_origin_s,
        method_id: seg.method_id(),
        evidence_class: p.evidence_class,
    };
    let mut fh = std::io::BufWriter::new(std::fs::File::create(out.join("contours.jsonl"))?);
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut n = 0usize;
    let t0 = Instant::now();
    let finish = |m: &MaskResult, f: &Frame| -> anyhow::Result<ContourObservation> {
        if m.width != w || m.height != h {
            bail!(
                "frame {} is {}x{}, the probe says {w}x{h}",
                f.index,
                m.width,
                m.height
            );
        }
        let geom = ext.extract(&m.mask);
        let obs = make_observation(
            &geom,
            &ctx,
            f.index,
            f.pts_s,
            &m.dependency_frames,
            &[format!("seg={}", m.info)],
        );
        ignis_core::prefix::validate_contour(&obs)
            .map_err(|e| anyhow::anyhow!("leakage at frame {}: {e}", f.index))?;
        Ok(obs)
    };
    let mut emit = |obs: &ContourObservation,
                    fh: &mut std::io::BufWriter<std::fs::File>|
     -> anyhow::Result<()> {
        serde_json::to_writer(&mut *fh, obs)?;
        fh.write_all(b"\n")?;
        *counts
            .entry(
                serde_json::to_value(obs.status)?
                    .as_str()
                    .unwrap_or("?")
                    .to_string(),
            )
            .or_default() += 1;
        n += 1;
        Ok(())
    };
    let seg_manifest;
    match seg {
        Segmentation::Classical(c) => {
            let batch = (pool.current_num_threads() * 4).max(8);
            let (tx, rx) = std::sync::mpsc::sync_channel::<anyhow::Result<Frame>>(batch * 2);
            let (vp, every, start, stop) = (video.to_path_buf(), p.every, p.start, p.stop);
            let dec = std::thread::spawn(move || -> anyhow::Result<()> {
                for f in iter_frames(&vp, every, start, stop, true)? {
                    if tx.send(f).is_err() {
                        break;
                    }
                }
                Ok(())
            });
            let mut buf: Vec<Frame> = Vec::with_capacity(batch);
            let mut rx_iter = rx.into_iter();
            loop {
                buf.clear();
                for f in rx_iter.by_ref().take(batch) {
                    buf.push(f?);
                }
                if buf.is_empty() {
                    break;
                }
                let done: Vec<anyhow::Result<ContourObservation>> = pool.install(|| {
                    buf.par_iter()
                        .map(|f| finish(&c.segment(f.index, f.width, f.height, &f.rgb)?, f))
                        .collect()
                });
                for o in done {
                    emit(&o?, &mut fh)?;
                }
            }
            dec.join()
                .map_err(|_| anyhow::anyhow!("decoder thread panicked"))??;
            seg_manifest = c.manifest();
        }
        Segmentation::Sam2 { cfg, prompt_from } => {
            let mut sc = Sam2Sidecar::start(&cfg, &out.join("sam2_work"), w, h)?;
            for f in iter_frames(video, p.every, p.start, p.stop, true)? {
                let f = f?;
                let m = sc.segment(&f, &prompt_from)?;
                emit(&finish(&m, &f)?, &mut fh)?;
            }
            seg_manifest = sc.close()?;
        }
    }
    fh.flush()?;
    let secs = t0.elapsed().as_secs_f64();
    let manifest = json!({
        "run_key": run_key, "video": probe, "params": params, "segmenter": seg_manifest,
        "decoder": {"demux": "mp4 (Rust) sample table: pts = dts + ctts - elst", "codec": "openh264 (bundled source)", "rgb": "BT.601 limited, FFmpeg x86 yuv2rgb arithmetic"},
        "calibration_id": cal.record.id, "units": cal.units(), "frames_processed": n, "status_counts": counts,
        "causal_check": {"future_dependencies": 0, "passed": true}, "evidence_class": p.evidence_class,
        "wall_s": secs, "frames_per_s": if secs > 0.0 { n as f64 / secs } else { 0.0 },
        "method_version": crate::METHOD_VERSION,
    });
    std::fs::write(
        out.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest)?,
    )?;
    Ok(json!({
        "out": out.display().to_string(), "frames_processed": n, "status_counts": manifest["status_counts"],
        "causal_check": manifest["causal_check"], "units": cal.units(), "frames_per_s": manifest["frames_per_s"],
    }))
}
