"""IGNIS SAM2.1 sidecar: strictly causal video tracking, lock-stepped by the Rust side.

Protocol (JSON lines on stdin, one reply per request on stdout; see crates/ignis-vision/src/sam2.rs):

  {"op":"init","checkpoint","model_cfg","device","width","height","work_dir"}
      -> {"ok":true,"sam2_version","checkpoint_sha256"}
  {"op":"frame","frame":t,"image":<png>,"prompt_box_px":[x0,y0,x1,y1]|null}
      -> {"ok":true,"frame":t,"mask":<png>,"dependency_frames":[...],"state":"prompted|tracked"}
  {"op":"close"}
      -> {"ok":true,"manifest":{...}}   (also <work_dir>/sam2_manifest.json)
  any failure -> {"ok":false,"error":"..."} and the Rust side stops the run.

Causality: frames arrive one at a time, in increasing order, and are appended to the predictor's
frame store only when they arrive; propagation is forward only (reverse=False) from the prompt
frame, so the memory bank holds only frames in [p0, t]. Because propagation is recurrent, the mask
at t is reported as depending on every fed frame in [p0, t]. A prompt is accepted only on the frame
being sent, and only once.

Setup: `uv sync --project sidecar/sam2`; checkpoint e.g.
https://dl.fbaipublicfiles.com/segment_anything_2/092824/sam2.1_hiera_small.pt
"""

from __future__ import annotations

import hashlib
import json
import os
import sys
import tempfile
from typing import Any

import numpy as np
from PIL import Image

MEAN = np.array([0.485, 0.456, 0.406], np.float32)
STD = np.array([0.229, 0.224, 0.225], np.float32)


def _sha256(path: str) -> str:
    h = hashlib.sha256()
    with open(path, "rb") as f:
        while chunk := f.read(1 << 20):
            h.update(chunk)
    return h.hexdigest()


class AppendableFrames:
    """List-like tensor store matching what SAM2's predictor indexes (``images[i]``)."""

    def __init__(self, size: int):
        self.size = size
        self.frames: list[Any] = []

    def append(self, rgb: np.ndarray) -> None:
        import torch

        x = np.asarray(Image.fromarray(rgb).resize((self.size, self.size), Image.BILINEAR), np.float32) / 255.0
        self.frames.append(torch.from_numpy(((x - MEAN) / STD).transpose(2, 0, 1).copy()))

    def __getitem__(self, i: int) -> Any:
        return self.frames[i]

    def __len__(self) -> int:
        return len(self.frames)


class Tracker:
    def __init__(self, req: dict[str, Any]):
        import sam2
        import torch
        from sam2.build_sam import build_sam2_video_predictor

        ckpt = req["checkpoint"]
        if not ckpt or not os.path.exists(ckpt):
            raise FileNotFoundError(f"SAM2.1 checkpoint not found: {ckpt!r}")
        torch.set_grad_enabled(False)
        self.req = req
        self.predictor = build_sam2_video_predictor(req["model_cfg"], ckpt, device=req.get("device", "cpu"))
        self.version = getattr(sam2, "__version__", "unknown")
        self.ckpt_sha = _sha256(ckpt)
        self.work = req["work_dir"]
        os.makedirs(os.path.join(self.work, "masks"), exist_ok=True)
        self.state: Any = None
        self.seen: list[int] = []
        self.prompts: list[dict[str, Any]] = []

    def _init_state(self, first: np.ndarray) -> None:
        """Official ``init_state`` on a one-frame directory (the current frame only), then swap in an
        appendable store: no frame after the current one is ever loaded."""
        with tempfile.TemporaryDirectory() as d:
            Image.fromarray(first).save(os.path.join(d, "00000.jpg"), quality=95)
            self.state = self.predictor.init_state(video_path=d, offload_video_to_cpu=True)
        self.state["images"] = AppendableFrames(self.predictor.image_size)
        self.state["num_frames"] = 0

    def frame(self, req: dict[str, Any]) -> dict[str, Any]:
        import torch

        t = int(req["frame"])
        if self.seen and t <= self.seen[-1]:
            raise ValueError(f"frame {t} after {self.seen[-1]}: frames must arrive in increasing order")
        box = req.get("prompt_box_px")
        if box is not None and self.prompts:
            raise ValueError("a second prompt is not accepted (single forward track)")
        if box is None and not self.prompts:
            raise ValueError("the first frame sent must carry the prompt box")
        rgb = np.asarray(Image.open(req["image"]).convert("RGB"))
        if self.state is None:
            self._init_state(rgb)
        local = len(self.seen)
        self.seen.append(t)
        self.state["images"].append(rgb)
        self.state["num_frames"] = local + 1
        p = self.predictor
        with torch.inference_mode():
            if box is not None:
                self.prompts.append({"frame": t, "local_index": local, "box_px": box, "obj_id": 1})
                _, _, logits = p.add_new_points_or_box(self.state, frame_idx=local, obj_id=1, box=np.asarray(box, np.float32))
                state = "prompted"
            else:
                logits = None
                # exactly one step forward; the memory holds only frames already fed
                for fidx, _, lg in p.propagate_in_video(self.state, start_frame_idx=local, max_frame_num_to_track=0, reverse=False):
                    if fidx == local:
                        logits = lg
                if logits is None:
                    raise RuntimeError("SAM2 propagation produced no output for the current frame")
                state = "tracked"
        mask = (logits[0, 0] > 0).cpu().numpy()
        p0 = min(q["local_index"] for q in self.prompts)
        deps = self.seen[p0 : local + 1]
        if max(deps) > t:
            raise RuntimeError("leakage: dependency after the current frame")
        out = os.path.join(self.work, "masks", f"m{t:07d}.png")
        Image.fromarray(mask.astype(np.uint8) * 255).save(out)
        return {"ok": True, "frame": t, "mask": out, "dependency_frames": deps, "state": state}

    def manifest(self) -> dict[str, Any]:
        m = {
            "causal": True,
            "dependency": "recurrent forward propagation: mask(t) depends on all fed frames in [p0, t]",
            "prompts": self.prompts, "frames_fed": len(self.seen), "model_cfg": self.req["model_cfg"],
            "checkpoint": self.req["checkpoint"], "checkpoint_sha256": self.ckpt_sha,
            "sam2_version": self.version, "device": self.req.get("device", "cpu"),
        }
        with open(os.path.join(self.work, "sam2_manifest.json"), "w", encoding="utf-8") as f:
            json.dump(m, f, indent=2)
        return m


def main() -> None:
    tracker: Tracker | None = None
    for line in sys.stdin:
        if not line.strip():
            continue
        req: dict[str, Any] = {}
        try:
            req = json.loads(line)
            op = req.get("op")
            if op == "init":
                tracker = Tracker(req)
                rep = {"ok": True, "sam2_version": tracker.version, "checkpoint_sha256": tracker.ckpt_sha}
            elif tracker is None:
                raise RuntimeError("init first")
            elif op == "frame":
                rep = tracker.frame(req)
            elif op == "close":
                rep = {"ok": True, "manifest": tracker.manifest()}
            else:
                raise ValueError(f"unknown op {op!r}")
        except Exception as e:  # reported to the Rust side, which stops the run
            rep = {"ok": False, "error": f"{type(e).__name__}: {e}"}
        sys.stdout.write(json.dumps(rep) + "\n")
        sys.stdout.flush()
        if req.get("op") == "close" or not rep["ok"]:
            break


if __name__ == "__main__":
    main()
