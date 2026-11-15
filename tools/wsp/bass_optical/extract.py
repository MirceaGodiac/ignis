#!/usr/bin/env python3
"""Image-plane flame extent from a BASS-II duct-camera video (prototype, WSP lane).

What this measures, and what it does not. The duct camera of BASS-II looks at the fuel sheet edge-on
through a turning mirror; its 720x480 video carries an overlay (FUEL, AIR, FAN, RAD, date and time).
No physical length scale is recorded with the video, so every position here is a fraction of the
frame width (0 = left edge, 1 = right edge) or of the frame height, and every length is in the same
unit. Nothing is converted to metres, and the luminous region of a camera image is not a pyrolysis
front.

Algorithm (deterministic, the same constants for every video):
  1. frames every `--step` seconds, rows below the overlay (`--top`, default 100 px), 3x3 median;
  2. signal = max(R, G, B) of each pixel minus that pixel's own dark level, the 10th percentile of
     its values over the sampled frames (a flame is in one pixel only a small part of the time);
  3. a pixel is luminous when its signal exceeds `--threshold` (default 12 of 255);
  4. the flame of a frame is the largest 8-connected luminous region of at least `--min-area`
     pixels; isolated hot pixels and overlay bleed never form one;
  5. the rightmost `--right-margin` of the frame is ignored (an encoder band sits there in some
     recordings);
  6. a frame whose mean level exceeds `--lit-level` shows the lit test section (crew light on), not
     a flame: it is flagged `lit` and carries no extent.

Per frame: bounding box of the flame (x_left, x_right, y_top, y_bottom), its area in pixels and its
summed signal.

    python tools/wsp/bass_optical/extract.py VIDEO.mp4 --out series.json

The output is a WSP-lane working product: it is not an IGNIS observation record, carries no
calibration, and must not be written into the shared records.
"""
import argparse
import json
import sys
from pathlib import Path

import cv2
import numpy as np


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("video")
    ap.add_argument("--out", required=True)
    ap.add_argument("--step", type=float, default=0.5)
    ap.add_argument("--top", type=int, default=100)
    ap.add_argument("--threshold", type=float, default=12.0)
    ap.add_argument("--min-area", type=int, default=30)
    ap.add_argument("--lit-level", type=float, default=8.0)
    ap.add_argument("--right-margin", type=float, default=0.05, help="fraction of the frame width ignored at the right edge (a coloured band of the video encoder sits there in some recordings)")
    args = ap.parse_args()
    cap = cv2.VideoCapture(args.video)
    fps = cap.get(cv2.CAP_PROP_FPS)
    count = int(cap.get(cv2.CAP_PROP_FRAME_COUNT))
    width = int(cap.get(cv2.CAP_PROP_FRAME_WIDTH))
    height = int(cap.get(cv2.CAP_PROP_FRAME_HEIGHT))
    stride = max(1, int(round(args.step * fps)))
    times, frames = [], []
    idx = 0
    while True:
        ok, frame = cap.read()
        if not ok:
            break
        if idx % stride == 0:
            keep = int(round(width * (1.0 - args.right_margin)))
            frames.append(cv2.medianBlur(frame[args.top:, :keep, :], 3).max(axis=2).astype(np.uint8))
            times.append(idx / fps)
        idx += 1
    cap.release()
    stack = np.stack(frames)
    dark = np.percentile(stack, 10, axis=0)
    rows = []
    for t, f in zip(times, frames):
        level = float(f.mean())
        row = {"t_s": round(t, 3), "mean_level": round(level, 2), "lit": level > args.lit_level,
               "x_left": None, "x_right": None, "y_top": None, "y_bottom": None, "area_px": 0, "signal": 0.0}
        if not row["lit"]:
            signal = f.astype(np.float32) - dark
            lum = (signal > args.threshold).astype(np.uint8)
            n, labels, stats, _ = cv2.connectedComponentsWithStats(lum, connectivity=8)
            if n > 1:
                k = 1 + int(np.argmax(stats[1:, cv2.CC_STAT_AREA]))
                area = int(stats[k, cv2.CC_STAT_AREA])
                if area >= args.min_area:
                    x, y, w, h = (int(stats[k, c]) for c in (cv2.CC_STAT_LEFT, cv2.CC_STAT_TOP, cv2.CC_STAT_WIDTH, cv2.CC_STAT_HEIGHT))
                    row.update(x_left=round(x / width, 4), x_right=round((x + w) / width, 4),
                               y_top=round((y + args.top) / height, 4), y_bottom=round((y + h + args.top) / height, 4),
                               area_px=area, signal=round(float(signal[labels == k].sum()), 1))
        rows.append(row)
    out = {
        "video": Path(args.video).name,
        "frame_px": [width, height],
        "fps": fps,
        "frames_total": count,
        "units": {"x_left": "fraction of frame width", "x_right": "fraction of frame width",
                  "y_top": "fraction of frame height", "y_bottom": "fraction of frame height",
                  "t_s": "s since the first video frame", "signal": "summed 8-bit levels above the dark level"},
        "algorithm": {"step_s": args.step, "top_px": args.top, "threshold": args.threshold, "min_area_px": args.min_area,
                      "lit_level": args.lit_level, "right_margin": args.right_margin, "dark": "per-pixel 10th percentile over the sampled frames",
                      "flame": "largest 8-connected luminous region"},
        "series": rows,
    }
    Path(args.out).write_text(json.dumps(out), encoding="utf-8")
    seen = [r for r in rows if r["x_left"] is not None]
    print(f"{Path(args.video).name}: {len(rows)} frames sampled, {sum(r['lit'] for r in rows)} lit, flame in {len(seen)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
