//! SAM2.1 causal tracking through the Python sidecar (`sidecar/sam2/`).
//!
//! Interface (JSON lines; requests on the sidecar's stdin, one reply per request on its stdout;
//! frames and masks are PNG files in a per-run work directory written by the side that produces
//! them):
//!
//! ```text
//! → {"op":"init","checkpoint":…,"model_cfg":…,"device":…,"width":W,"height":H,"work_dir":…}
//! ← {"ok":true,"sam2_version":…,"checkpoint_sha256":…}
//! → {"op":"frame","frame":t,"image":"<work>/frames/f0000123.png","prompt_box_px":[x0,y0,x1,y1]|null}
//! ← {"ok":true,"frame":t,"mask":"<work>/masks/m0000123.png","dependency_frames":[…],"state":"prompted|tracked"}
//! → {"op":"close"}
//! ← {"ok":true,"manifest":{…}}           (also written to <work>/sam2_manifest.json)
//! ```
//!
//! Causality is structural: the sidecar is lock-stepped, so frame `t+1` is not decoded, written or
//! sent before the mask for `t` has come back. Frames must arrive in increasing order; a prompt is
//! only accepted on the frame being sent. The prompt box comes from the classical comparator on the
//! first frame where it finds a luminous region (so it is derived from that frame only); before it,
//! the result is the (empty) classical mask depending on the current frame. Every reply is checked
//! here: `dependency_frames ⊆ frames fed so far` and `max ≤ t`, else the run stops as a leak.

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use anyhow::{anyhow, bail, Context};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::segment::{ClassicalSegmenter, MaskResult};
use crate::video::Frame;

pub const SAM2_METHOD_ID: &str = "vision.segment.sam2_causal";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Sam2Config {
    pub checkpoint: Option<String>,
    pub model_cfg: String,
    pub device: String,
    /// argv that starts the sidecar (stdin/stdout protocol above).
    pub sidecar_cmd: Vec<String>,
}

impl Default for Sam2Config {
    fn default() -> Self {
        Sam2Config {
            checkpoint: None,
            model_cfg: "configs/sam2.1/sam2.1_hiera_s.yaml".into(),
            device: "cpu".into(),
            sidecar_cmd: [
                "uv",
                "run",
                "--project",
                "sidecar/sam2",
                "python",
                "sidecar/sam2/ignis_sam2.py",
            ]
            .map(String::from)
            .to_vec(),
        }
    }
}

pub struct Sam2Sidecar {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    work: PathBuf,
    prompts: Vec<Value>,
    fed: Vec<i64>,
    last: Option<i64>,
    init: Value,
    cfg: Sam2Config,
}

impl Sam2Sidecar {
    pub fn start(
        cfg: &Sam2Config,
        work: &Path,
        width: usize,
        height: usize,
    ) -> anyhow::Result<Self> {
        let checkpoint = cfg
            .checkpoint
            .clone()
            .or_else(|| std::env::var("IGNIS_SAM2_CHECKPOINT").ok());
        let ckpt = checkpoint.as_deref().ok_or_else(|| anyhow!("SAM2.1 checkpoint not configured (configs/vision/sam2.toml or IGNIS_SAM2_CHECKPOINT)"))?;
        if !Path::new(ckpt).exists() {
            bail!("SAM2.1 checkpoint not found: {ckpt}");
        }
        std::fs::create_dir_all(work.join("frames"))?;
        std::fs::create_dir_all(work.join("masks"))?;
        let (prog, args) = cfg
            .sidecar_cmd
            .split_first()
            .ok_or_else(|| anyhow!("empty sidecar_cmd"))?;
        let mut child = Command::new(prog)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .with_context(|| format!("spawn SAM2 sidecar {:?}", cfg.sidecar_cmd))?;
        let stdin = child.stdin.take().ok_or_else(|| anyhow!("sidecar stdin"))?;
        let stdout = BufReader::new(
            child
                .stdout
                .take()
                .ok_or_else(|| anyhow!("sidecar stdout"))?,
        );
        let mut s = Sam2Sidecar {
            child,
            stdin,
            stdout,
            work: work.to_path_buf(),
            prompts: vec![],
            fed: vec![],
            last: None,
            init: Value::Null,
            cfg: Sam2Config {
                checkpoint: Some(ckpt.to_string()),
                ..cfg.clone()
            },
        };
        let abs = |p: &Path| {
            std::path::absolute(p)
                .unwrap_or_else(|_| p.to_path_buf())
                .display()
                .to_string()
        };
        s.init = s.call(json!({
            "op": "init", "checkpoint": ckpt, "model_cfg": cfg.model_cfg, "device": cfg.device,
            "width": width, "height": height, "work_dir": abs(work),
        }))?;
        Ok(s)
    }

    fn call(&mut self, req: Value) -> anyhow::Result<Value> {
        writeln!(self.stdin, "{req}")?;
        self.stdin.flush()?;
        let mut line = String::new();
        if self.stdout.read_line(&mut line)? == 0 {
            bail!("SAM2 sidecar exited without replying to {}", req["op"]);
        }
        let v: Value =
            serde_json::from_str(line.trim()).with_context(|| format!("sidecar reply {line:?}"))?;
        if v["ok"] != json!(true) {
            bail!("SAM2 sidecar error: {}", v["error"]);
        }
        Ok(v)
    }

    pub fn segment(
        &mut self,
        f: &Frame,
        classical: &ClassicalSegmenter,
    ) -> anyhow::Result<MaskResult> {
        let t = f.index as i64;
        if matches!(self.last, Some(l) if t <= l) {
            bail!("frames must be fed in increasing presentation order");
        }
        self.last = Some(t);
        let mut prompt: Option<[f64; 4]> = None;
        if self.prompts.is_empty() {
            let r = classical.segment(f.index, f.width, f.height, &f.rgb)?;
            let (mut x0, mut y0, mut x1, mut y1) = (usize::MAX, usize::MAX, 0, 0);
            for (i, _) in r.mask.iter().enumerate().filter(|(_, &m)| m) {
                let (x, y) = (i % f.width, i / f.width);
                (x0, y0, x1, y1) = (x0.min(x), y0.min(y), x1.max(x), y1.max(y));
            }
            if x0 == usize::MAX {
                return Ok(MaskResult {
                    info: json!({"state": "awaiting_prompt"}),
                    ..r
                });
            }
            prompt = Some([x0 as f64, y0 as f64, x1 as f64, y1 as f64]);
        }
        let img = self
            .work
            .join("frames")
            .join(format!("f{:07}.png", f.index));
        crate::imageio::write_rgb_png(&img, f.width, f.height, &f.rgb)?;
        self.fed.push(t);
        let rep = self.call(json!({"op": "frame", "frame": t, "image": img.display().to_string(), "prompt_box_px": prompt}))?;
        let _ = std::fs::remove_file(&img);
        if rep["frame"] != json!(t) {
            bail!("sidecar answered frame {} for frame {t}", rep["frame"]);
        }
        if let Some(b) = prompt {
            self.prompts
                .push(json!({"frame": t, "box_px": b, "obj_id": 1}));
        }
        let deps: Vec<i64> = serde_json::from_value(rep["dependency_frames"].clone())?;
        if deps.iter().any(|&d| d > t || !self.fed.contains(&d)) {
            bail!("leakage: SAM2 mask for frame {t} depends on frames not fed before it: {deps:?}");
        }
        let mpath = PathBuf::from(
            rep["mask"]
                .as_str()
                .ok_or_else(|| anyhow!("sidecar reply without mask"))?,
        );
        let (w, h, mask) = crate::imageio::read_mask_png(&mpath)?;
        if (w, h) != (f.width, f.height) {
            bail!("sidecar mask is {w}x{h}, frame is {}x{}", f.width, f.height);
        }
        let _ = std::fs::remove_file(&mpath);
        Ok(MaskResult {
            frame: f.index,
            width: w,
            height: h,
            mask,
            dependency_frames: deps,
            info: json!({"state": rep["state"]}),
        })
    }

    /// Close the sidecar and return the prompt/dependency manifest.
    pub fn close(mut self) -> anyhow::Result<Value> {
        let rep = if self.fed.is_empty() {
            json!({"manifest": {}})
        } else {
            self.call(json!({"op": "close"}))?
        };
        drop(self.stdin);
        let _ = self.child.wait();
        Ok(json!({
            "method_id": SAM2_METHOD_ID, "causal": true,
            "dependency": "recurrent forward propagation: mask(t) depends on all fed frames in [p0, t]",
            "prompts": self.prompts, "frames_fed": self.fed.len(), "model_cfg": self.cfg.model_cfg,
            "checkpoint": self.cfg.checkpoint, "device": self.cfg.device, "sidecar": self.init, "sidecar_manifest": rep["manifest"],
        }))
    }
}
