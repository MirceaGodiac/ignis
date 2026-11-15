#!/usr/bin/env python3
"""Flame tip and base against time from BASS-II (PSI-25) and BASS (PSI-26) burn videos.

Blind measurement for the fabric front model. Run with `python -I`; the videos are data, passed as
paths, nothing is executed from them. The method and every constant below were fixed on the four
PSI-25 train burns (T5, T6, T16, T10) before any other burn was processed; `METHOD_VERSION` and the
sha256 of this file are written into every track.

    python -I tools/wsp/media/fabric/measure.py --video V.mp4 --run-id PSI-25:T5 --partition train \
        --material "2 cm SIBAL" --out runs/dev/media/fabric/tracks/psi25/train/T5.json \
        --sheet runs/dev/media/fabric/check/psi25_T5.jpg --ovl runs/dev/media/fabric/ovl/psi25_T5.npz

Method (duct / main view; the camera looks at the fuel edge-on, the flow runs right to left):
  frames     every frame is decoded. Brightness b = R+G+B. The video overlay (FUEL AIR FAN RAD, date,
             clock) sits in recorded rows 25-90; the picture below row TOP_PX is measured.
  register   the analog capture shifts the whole picture sideways (T16: -130 px) and the overlay
             with it, and in the shifted state it is also compressed (T16: about 0.97). Each frame's
             shift d and scale a are found by correlating the column profile of the overlay label row
             (rows 31-47) with the label-box grid (pitch 28.67 px, first box at 155 px; a in 0.95-1.05);
             positions are reported in ref coordinates x_rec = 155 + d + a (x_ref - 155), as a fraction of the 720-px
             frame width (fw). A frame whose label row does not correlate (r < OVL_R_MIN) keeps the
             last shift and is counted as overlay-less.
  windows    frames are binned 2x2 and averaged in WIN_S windows on the ref canvas.
  lit/other  a window is lit (crew light, camera playback lit) when its picture mean exceeds the
             10th percentile of window means by LIT_DELTA; a window is off-scene (camera playback
             screen, other source) when the video has an overlay in most windows and fewer than half
             of this window's frames carry it. Neither is measured.
  background per pixel, the median over dark windows; noise 1.4826 x MAD, floor NOISE_FLOOR.
  luminous   z = (window - background) / noise > Z_PIX; the luminous mask is dilated by DILATE_PX
             (binned px) for connectivity only (the two flame sheets either side of the fuel join);
             the flame is the component with the largest summed z; its score is that sum, its area
             the number of luminous pixels in it. Tip = most downstream (left) ref column holding at
             least COL_MIN luminous pixels of the component, base = most upstream (right) one.
  igniter    per frame, the brightness of 16x16-px blocks above their own median over dark frames;
             igniter on = first frame whose hottest block exceeds IGN_ON; the igniter box is the
             blocks above IGN_ON/2 at the burst peak; off = the steepest drop (3-frame difference) of
             the box brightness between the peak and the end of the first burst (box excess below
             IGN_END x peak excess). t = 0 at igniter off (video time, s).
  detection  threshold = max(DET_K x the largest score in null windows, SCORE_FLOOR) and area >=
             MIN_AREA; null windows = dark windows ending NULL_GAP_S before igniter on, not the first or last window (if fewer than
             NULL_MIN, the NULL_MIN dark windows of lowest score after off + 3 s are added).
             A window counts as flame only from off + START_S and if the component's vertical centre
             lies within PLANE_TOL fh of the igniter box centre (the sample plane).
  scale      the downstream end of the sample is the igniter box's right edge minus WINDOW_FW
             (fixed from train: where the T5, T6 and T16 tips stopped).
             For SIBAL fabric burns of PSI-25 positions are converted to sample widths with
             W_PER_FW (2 cm) x (2 cm / nominal width); origin = igniter box right edge + ORIGIN_FW.
             No metric scale is established (2.0 vs 2.2 cm), so no metres are given.
  outcome    see classify(): spread_to_end / went_out / never_lit / cannot_tell with a confidence.
"""
import argparse
import hashlib
import json
import math
import sys
from pathlib import Path

import cv2
import numpy as np

METHOD_VERSION = "fabric-duct/1"

# ---- fixed constants (set on train burns only) -----------------------------------------------
TOP_PX = 92               # recorded rows above this hold the overlay
LEFT_PX = 10              # recorded left edge band
RIGHT_PX = 686            # recorded encoder band beyond this
OVL_PITCH = 28.67
OVL_FIRST = 155.0
OVL_BOXES = (0, 1, 2, 3, 6, 7, 8, 11, 12, 13, 16, 17, 18)
OVL_R_MIN = 0.55
WIN_S = 1.0
LIT_DELTA = 15.0          # b units (R+G+B), window picture mean above its 10th percentile
NOISE_FLOOR = 0.25
BLUR_SIGMA = 0.0          # binned px, Gaussian on each window mean before the z test
Z_PIX = 4.0
DILATE_PX = 9
COL_MIN = 2
MIN_AREA = 30
DET_K = 1.5
SCORE_FLOOR = 150.0
NULL_MIN = 6
NULL_GAP_S = 2.0          # null windows end this long before igniter on (its weak first glow)
NULL_SKIP = 3             # the first windows of a recording carry start-up artefacts
IGN_ON = 60.0             # b units over the block's dark median (16x16-px block mean)
IGN_OFF_REL = 0.7        # igniter off = last frame whose hottest block is at least this x the burst median
IGN_PERSIST_S = 1.0       # the igniter stays hot at least this long (a capture glitch does not)
IGN_SEP_S = 5.0           # igniter bursts closer than this are one burst
IGN_BOX_REL = 0.5         # igniter box = blocks above this fraction of the hottest block at the peak
LIT_GUARD = 1             # windows next to a lit / off-scene window are not used (partly lit)
START_S = 0.5
PLANE_TOL = 0.12
W_PER_FW = 6.0            # 2-cm sample widths per frame width (train two-point scales 5.83-6.33)
ORIGIN_FW = 0.006         # tip w = 0 lies this far upstream of the igniter box edge (train: +0.017, -0.012, +0.013 fw)
WINDOW_FW = 0.717         # sample end = igniter box edge - this (train: the tips of T5, T6, T16 stop 0.700, 0.727, 0.725 fw from it)
IGN_EDGE_RANGE = (0.89, 0.96)  # igniter box edge (fw) of the train holders: 0.911, 0.933, 0.933, 0.933
X_END_FIXED = 0.208       # ref fw where the train tips stopped (0.211, 0.206, 0.208), used when the igniter was moved
END_TOL_W = 0.35          # tip within this of the sample end counts as reaching it (2-cm widths)
GONE_S = 6.0              # dark record after the last flame needed to call it gone
LATE_S = 2.0              # flame windows are counted for the outcome from this long after igniter off (coil afterglow)
MIN_FLAME_S = 3.0         # less flame than this after igniter off = never lit


def file_sha(p):
    h = hashlib.sha256()
    h.update(Path(p).read_bytes())
    return h.hexdigest()


DX_RANGE = np.arange(-200, 81)
A_RANGE = np.round(np.arange(0.95, 1.0501, 0.005), 3)


def _seg_matrix():
    x = np.arange(720, dtype=np.float32)
    segs, keys = [], []
    for a in A_RANGE:
        for d in DX_RANGE:
            t = np.zeros(720, np.float32)
            for k in OVL_BOXES:
                x0 = OVL_FIRST + d + a * OVL_PITCH * k
                t[(x >= x0) & (x < x0 + 27 * a)] = 1.0
            segs.append(t)
            keys.append((int(d), float(a)))
    segs = np.stack(segs)
    segs -= segs.mean(axis=1, keepdims=True)
    segs /= np.sqrt((segs * segs).sum(axis=1, keepdims=True)) + 1e-9
    return segs, keys


SEGS, SEG_KEYS = _seg_matrix()


def overlay_dx_fast(frame, last=None):
    """Shift d (px) and horizontal scale a of the overlay label row against the label-box grid:
    x_rec = OVL_FIRST + d + a (x_ref - OVL_FIRST). None if it does not correlate."""
    prof = frame[31:48].max(axis=2).mean(axis=0).astype(np.float32)
    prof = prof - prof.mean()
    nrm = float(np.sqrt((prof * prof).sum()))
    if prof.std() < 2.0 or nrm == 0:
        return None, 0.0
    r = SEGS @ (prof / nrm)
    k = int(np.argmax(r))
    return (SEG_KEYS[k] if r[k] >= OVL_R_MIN else None), float(r[k])


def rec_x(fw, d, a):
    """Recorded pixel column of a ref position in frame widths."""
    return OVL_FIRST + d + a * (fw * 720.0 - OVL_FIRST)


def ref_fw(x_rec, d, a):
    return (OVL_FIRST + (x_rec - OVL_FIRST - d) / a) / 720.0


def decode(video, keep_ovl=True):
    cap = cv2.VideoCapture(str(video))
    fps = cap.get(cv2.CAP_PROP_FPS) or 29.97
    if not (5 < fps < 121):
        fps = 29.97
    W, H = int(cap.get(cv2.CAP_PROP_FRAME_WIDTH)), int(cap.get(cv2.CAP_PROP_FRAME_HEIGHT))
    if (W, H) != (720, 480):
        scale_needed = True
    else:
        scale_needed = False
    per = max(1, int(round(WIN_S * fps)))
    hb, wb = (480 - TOP_PX) // 2, 360
    wins, wcnt, wmeta = [], [], []
    acc = np.zeros((hb, wb), np.float32)
    cnt = np.zeros(wb, np.float32)
    n_in, n_ovl, dxs = 0, 0, []
    frame_t, frame_hot, frame_lvl, frame_dx = [], [], [], []
    blocks = []
    last_dx = None
    ovl_strips = []
    idx = 0
    while True:
        ok, f = cap.read()
        if not ok:
            break
        if scale_needed:
            f = cv2.resize(f, (720, 480), interpolation=cv2.INTER_AREA)
        da, r = overlay_dx_fast(f, last_dx)
        has = da is not None
        if has:
            last_dx = da
        d, a = last_dx if last_dx is not None else (0, 1.0)
        b = f[TOP_PX:].astype(np.float32).sum(axis=2)
        bb = cv2.resize(b, (wb, hb), interpolation=cv2.INTER_AREA)
        # ref coords: x_rec = F + d + a (x_ref - F); binned F/2
        F2 = OVL_FIRST / 2.0
        Mx = np.float32([[a, 0, F2 + d / 2.0 - a * F2], [0, 1, 0]])
        sh = cv2.warpAffine(bb, Mx, (wb, hb), flags=cv2.INTER_LINEAR | cv2.WARP_INVERSE_MAP, borderValue=0)
        xr = F2 + d / 2.0 + a * (np.arange(wb) - F2)
        sv = (xr >= LEFT_PX / 2.0) & (xr <= RIGHT_PX / 2.0 - 1)
        acc += sh * sv
        cnt += sv
        n_in += 1
        n_ovl += has
        dxs.append((d, a))
        lvl = float(bb[:, LEFT_PX // 2:RIGHT_PX // 2].mean())
        blk = cv2.resize(sh, (wb // 8, hb // 8), interpolation=cv2.INTER_AREA)
        blk[:, ~sv[::8][:wb // 8]] = 0
        blocks.append(blk.astype(np.float16))
        frame_t.append(idx / fps)
        frame_lvl.append(lvl)
        frame_dx.append(da if has else None)
        if keep_ovl and idx % int(round(fps)) == 0:
            g = f[25:90].max(axis=2)
            ovl_strips.append((idx / fps, d, a, has, g.copy()))
        if n_in == per:
            m = acc / np.maximum(cnt, 1)[None, :]
            bad = cnt < per * 0.5
            if BLUR_SIGMA > 0:
                m = cv2.GaussianBlur(m, (0, 0), BLUR_SIGMA)
                bad = bad | np.convolve(bad, np.ones(5), mode="same").astype(bool)
            m[:, bad] = np.nan
            wins.append(m.astype(np.float16))
            wmeta.append(dict(t0=(idx + 1 - per) / fps, t1=(idx + 1) / fps, ovl_frac=n_ovl / n_in,
                              dx=float(np.median([q[0] for q in dxs])), a=float(np.median([q[1] for q in dxs]))))
            acc[:] = 0
            cnt[:] = 0
            n_in, n_ovl, dxs = 0, 0, []
        idx += 1
    cap.release()
    return dict(fps=fps, n_frames=idx, size=[W, H], wins=wins, wmeta=wmeta, frame_t=np.array(frame_t),
                frame_lvl=np.array(frame_lvl), blocks=np.stack(blocks) if blocks else None,
                frame_dx=frame_dx, ovl=ovl_strips)


def igniter(D, lit_frame):
    B = D["blocks"].astype(np.float32)
    dark = ~lit_frame
    if dark.sum() < 10:
        return None
    sub = B[dark][:: max(1, int(dark.sum() // 2000))]
    med = np.median(sub, axis=0)
    ex = B - med[None]
    ex[lit_frame] = 0
    ex[:, :, :1] = 0
    ex[:, :, -1:] = 0
    hot = ex.reshape(len(ex), -1).max(axis=1)
    above = hot > IGN_ON
    run = int(round(IGN_PERSIST_S * D["fps"]))
    cs = np.concatenate([[0], np.cumsum(above)])
    ok = np.nonzero((cs[run:] - cs[:-run]) >= run)[0] if len(above) > run else np.array([], int)
    if len(ok) == 0:
        return None
    hold = int(round(D["fps"]))
    low = (hot < IGN_ON) | lit_frame
    bursts = []
    first_box = None
    skip_until = -1
    for i_on in ok:
        i_on = int(i_on)
        if i_on < skip_until:
            continue
        end = len(hot) - 1
        for i in range(i_on + 1, len(hot) - hold):
            if low[i:i + hold].all():
                end = i
                break
        med_hot = float(np.median(hot[i_on:end])) if end > i_on else float(hot[i_on])
        strong = np.nonzero(hot[i_on:end] >= IGN_OFF_REL * med_hot)[0]
        i_off = i_on + int(strong[-1]) + 1 if len(strong) else end
        pk = i_on + int(np.argmax(hot[i_on:end])) if end > i_on else i_on
        box = ex[pk] >= IGN_BOX_REL * ex[pk].max()
        skip_until = end + int(IGN_SEP_S * D["fps"])
        if first_box is not None:
            # a later burst counts as an igniter only where the first one was (not a flame elsewhere)
            yy, xx = np.unravel_index(int(np.argmax(ex[pk])), ex[pk].shape)
            fy, fx = np.nonzero(first_box)
            if not (fx.min() - 2 <= xx <= fx.max() + 2 and fy.min() - 2 <= yy <= fy.max() + 2):
                continue
        else:
            first_box = box
        bursts.append(dict(i_on=i_on, i_off=min(i_off, len(hot) - 1), end=end, box=box))
    if not bursts:
        return None
    last = bursts[-1]
    ys, xs = np.nonzero(bursts[0]["box"])
    ft = D["frame_t"]
    return dict(i_on=last["i_on"], i_off=last["i_off"], t_on=float(ft[last["i_on"]]), t_off=float(ft[last["i_off"]]),
                box_x=[float(xs.min() * 16 / 720), float((xs.max() + 1) * 16 / 720)],
                box_y=[float((ys.min() * 16 + TOP_PX) / 480), float(((ys.max() + 1) * 16 + TOP_PX) / 480)],
                bursts=[(float(ft[q["i_on"]]), float(ft[q["i_off"]])) for q in bursts])


def spread_direction(ign):
    """-1: the flame travels toward smaller x (igniter on the right, as in every train burn); +1 otherwise."""
    if ign is None:
        return -1
    return -1 if (ign["box_x"][0] + ign["box_x"][1]) / 2 >= 0.5 else 1


def measure_windows(D, ign):
    direction = spread_direction(ign)
    W = len(D["wins"])
    X = np.stack(D["wins"])  # (W, h, w) float16
    means = np.array([np.nanmean(x.astype(np.float32)) for x in X])
    p10 = np.nanpercentile(means, 10)
    lit = means > p10 + LIT_DELTA
    ovl = np.array([m["ovl_frac"] for m in D["wmeta"]])
    has_overlay = np.mean(ovl > 0.5) > 0.5
    off_scene = (ovl < 0.5) if has_overlay else np.zeros(W, bool)
    bad = lit | off_scene
    guard = bad.copy()
    for g in range(1, LIT_GUARD + 1):
        guard[g:] |= bad[:-g]
        guard[:-g] |= bad[g:]
    off_scene = off_scene | (guard & ~bad)
    dark = ~lit & ~off_scene
    idx = np.nonzero(dark)[0]
    if len(idx) < 4:
        return None
    sel = idx[:: max(1, len(idx) // 400)]
    S = X[sel].astype(np.float32)
    med = np.nanmedian(S, axis=0)
    mad = 1.4826 * np.nanmedian(np.abs(S - med[None]), axis=0)
    noise = np.maximum(np.nan_to_num(mad, nan=1e3), NOISE_FLOOR)
    med = np.nan_to_num(med, nan=0.0)
    k = cv2.getStructuringElement(cv2.MORPH_ELLIPSE, (2 * DILATE_PX + 1, 2 * DILATE_PX + 1))
    rows = []
    for i in range(W):
        m = D["wmeta"][i]
        r = dict(i=i, t0=m["t0"], t1=m["t1"], tm=(m["t0"] + m["t1"]) / 2, dx=m["dx"], a=m["a"], lit=bool(lit[i]),
                 off_scene=bool(off_scene[i]), score=0.0, area=0, tip_fw=None, base_fw=None, y_top=None,
                 y_bot=None, yc=None)
        if dark[i]:
            x = X[i].astype(np.float32)
            valid = ~np.isnan(x)
            z = np.where(valid, (np.nan_to_num(x) - med) / noise, 0.0)
            lum = (z > Z_PIX).astype(np.uint8)
            if lum.any():
                dil = cv2.dilate(lum, k)
                n, lab = cv2.connectedComponents(dil, connectivity=8)
                if n > 1:
                    zl = np.where(lum > 0, z, 0.0)
                    sums = np.bincount(lab.ravel(), weights=zl.ravel(), minlength=n)
                    sums[0] = 0
                    c = int(np.argmax(sums))
                    pix = (lab == c) & (lum > 0)
                    area = int(pix.sum())
                    cols = pix.sum(axis=0)
                    good = np.nonzero(cols >= COL_MIN)[0]
                    if len(good) == 0:
                        good = np.nonzero(cols > 0)[0]
                    ys = np.nonzero(pix.any(axis=1))[0]
                    yw = (pix * zl).sum(axis=1)
                    lo_fw, hi_fw = float(good.min() * 2 / 720), float((good.max() + 1) * 2 / 720)
                    r.update(score=float(sums[c]), area=area,
                             tip_fw=lo_fw if direction < 0 else hi_fw, base_fw=hi_fw if direction < 0 else lo_fw,
                             y_top=float((ys.min() * 2 + TOP_PX) / 480), y_bot=float(((ys.max() + 1) * 2 + TOP_PX) / 480),
                             yc=float((np.average(np.arange(len(yw)), weights=yw) * 2 + TOP_PX) / 480) if yw.sum() > 0 else None)
        rows.append(r)
    return dict(rows=rows, has_overlay=bool(has_overlay), lit=lit, dark=dark, med=med, noise=noise,
                p10=float(p10))


def detect(M, ign):
    rows = M["rows"]
    t_on = ign["t_on"] if ign else None
    t_off = ign["t_off"] if ign else None
    null = [r for r in rows[NULL_SKIP:-1] if not r["lit"] and not r["off_scene"] and t_on is not None and r["t1"] < t_on - NULL_GAP_S]
    null_src = "pre-igniter"
    if len(null) < NULL_MIN:
        later = [r for r in rows if not r["lit"] and not r["off_scene"] and (t_off is None or r["t0"] > t_off + 3.0)]
        later.sort(key=lambda r: r["score"])
        null = null + later[:NULL_MIN - len(null)]
        null_src = "pre-igniter + lowest-score dark windows"
    nmax = max([r["score"] for r in null], default=0.0)
    thr = max(DET_K * nmax, SCORE_FLOOR)
    plane_y = (ign["box_y"][0] + ign["box_y"][1]) / 2 if ign else None
    for r in rows:
        r["det"] = bool(r["score"] > thr and r["area"] >= MIN_AREA)
        r["on_plane"] = bool(r["det"] and (plane_y is None or (r["yc"] is not None and abs(r["yc"] - plane_y) <= PLANE_TOL)))
        r["flame"] = bool(r["on_plane"] and (t_off is None or r["t0"] >= t_off + START_S))
    return dict(threshold=thr, null_max=nmax, null_n=len(null), null_source=null_src, plane_y=plane_y)


def classify(rows, ign, sibal, x_end, direction=-1):
    """Outcome from the window series. Positions are ref fw; `direction` is the way the flame travels
    (-1 toward smaller x). SIBAL fabric burns of PSI-25 have a known sample end (x_end); other burns
    cannot be called spread_to_end or went_out because the end of the sample is not located."""
    if ign is None:
        return dict(kind="cannot_tell", confidence="low", reason="no igniter found in the record"), None, None
    t_off = ign["t_off"]
    after = [r for r in rows if r["t0"] >= t_off + START_S]
    dark_after = [r for r in after if not r["lit"] and not r["off_scene"]]
    fl = [r for r in after if r["flame"]]
    fl_late = [r for r in fl if r["t0"] >= t_off + LATE_S]
    rec_end = dark_after[-1]["t1"] - t_off if dark_after else 0.0
    if not dark_after or rec_end < 2.0:
        return dict(kind="cannot_tell", confidence="low", reason="no dark record after igniter off"), None, rec_end
    flame_s = len(fl_late) * WIN_S
    if flame_s < MIN_FLAME_S:
        if len(fl) == 0 or flame_s == 0:
            conf = "high" if rec_end >= 15 else ("medium" if rec_end >= 6 else "low")
            why = f"no flame window from {LATE_S:.0f} s after igniter off over {rec_end:.1f} s of dark record"
        else:
            conf = "medium"
            why = f"only {flame_s:.0f} s of flame windows from {LATE_S:.0f} s after igniter off (brief flame, not sustained)"
        return dict(kind="never_lit", confidence=conf, reason=why), None, rec_end
    last = fl[-1]
    t_last = last["tm"] - t_off
    d = -direction  # d * x grows downstream... (direction -1: smaller x is downstream)
    tips = np.sort([direction * r["tip_fw"] for r in fl])[::-1]  # largest = furthest downstream
    tip_reach = direction * float(tips[min(1, len(tips) - 1)])   # second furthest: one outlier window does not count
    dark_after_last = [r for r in dark_after if r["t0"] > last["t1"]]
    gone_s = len(dark_after_last) * WIN_S
    if direction < 0:
        edge = ref_fw(LEFT_PX, last["dx"], last["a"])
        leaves = tip_reach <= edge + 0.01
    else:
        edge = ref_fw(RIGHT_PX, last["dx"], last["a"])
        leaves = tip_reach >= edge - 0.01
    end = dict(t_last_flame_s=round(t_last, 2), dark_after_last_s=gone_s, record_end_s=round(rec_end, 2),
               furthest_tip_fw=round(tip_reach, 4))
    short_w = None
    if x_end is not None:
        short_w = direction * (x_end - tip_reach) * W_PER_FW
        end["tip_short_of_end_w2cm"] = round(short_w, 3)
    if leaves and (x_end is None or direction * (x_end - edge) > 0):
        # the sample end lies beyond the captured picture
        if sibal:
            return dict(kind="spread_to_end", confidence="medium",
                        reason="tip leaves the captured picture at its downstream edge, the sample end lies beyond it"), end, rec_end
        return dict(kind="cannot_tell", confidence="low", sub="leaves_picture", reason="tip leaves the captured picture"), end, rec_end
    if short_w is not None and short_w <= END_TOL_W:
        conf = "high" if short_w <= END_TOL_W / 2 else "medium"
        return dict(kind="spread_to_end", confidence=conf,
                    reason=f"furthest tip {short_w:.2f} 2-cm widths from the sample end"), end, rec_end
    if gone_s >= GONE_S:
        if sibal and short_w is not None:
            conf = "high" if (gone_s >= 15 and short_w > 2 * END_TOL_W) else "medium"
            return dict(kind="went_out", confidence=conf,
                        reason=f"no flame for {gone_s:.0f} s of dark record after t={t_last:.1f} s, tip {short_w:.2f} 2-cm widths short of the end"), end, rec_end
        return dict(kind="cannot_tell", confidence="low", sub="flame_disappeared",
                    reason="flame disappears; the sample end is not located in this geometry (burned out or went out)"), end, rec_end
    return dict(kind="cannot_tell", confidence="low", sub="flame_at_record_end",
                reason=f"flame still present {gone_s:.1f} s before the dark record ends"), end, rec_end


def contact_sheet(video, D, rows, ign, x_end, x_origin, path, title, plane_y=None):
    fl = [r for r in rows if r.get("flame")]
    t_off = ign["t_off"] if ign else None
    darks = [r for r in rows if not r["lit"] and not r["off_scene"]]
    pick = []
    if fl:
        idx = np.linspace(0, len(fl) - 1, 5).round().astype(int)
        pick = [fl[i] for i in idx]
        after = [r for r in darks if r["t0"] > fl[-1]["t1"] + 2.0]
        if after:
            pick.append(after[min(len(after) - 1, 4)])
        else:
            pick.append(fl[-1])
    else:
        base = t_off if t_off is not None else (darks[0]["t0"] if darks else 0.0)
        cand = [r for r in darks if r["t0"] >= base]
        if not cand:
            cand = darks or rows
        idx = np.linspace(0, len(cand) - 1, 6).round().astype(int)
        pick = [cand[i] for i in idx]
    cap = cv2.VideoCapture(str(video))
    fps = D["fps"]
    panels = []
    for r in pick:
        cap.set(cv2.CAP_PROP_POS_FRAMES, int(round(r["t0"] * fps)))
        acc = None
        n = 0
        for _ in range(max(1, int(round(WIN_S * fps)))):
            ok, f = cap.read()
            if not ok:
                break
            if f.shape[:2] != (480, 720):
                f = cv2.resize(f, (720, 480), interpolation=cv2.INTER_AREA)
            acc = f.astype(np.float32) if acc is None else acc + f
            n += 1
        if acc is None:
            acc = np.zeros((480, 720, 3), np.float32)
            n = 1
        im = acc / n
        pic = im[TOP_PX:, LEFT_PX:RIGHT_PX]
        lo = np.percentile(pic, 50)
        hi = max(np.percentile(pic, 99.8), lo + 6)
        im = np.clip((im - lo) / (hi - lo), 0, 1) ** 0.6 * 255
        im[:TOP_PX] = acc[:TOP_PX] / n
        im = im.astype(np.uint8)
        def X(fw, r=r):
            return int(round(rec_x(fw, r["dx"], r["a"])))

        if r.get("tip_fw") is not None and r.get("det"):
            col_t = (0, 255, 0) if r.get("flame") else (0, 140, 255)
            yp = int(plane_y * 480) if plane_y else 300
            for xx, col in ((X(r["tip_fw"]), col_t), (X(r["base_fw"]), (255, 0, 255))):
                cv2.line(im, (xx, max(TOP_PX, yp - 110)), (xx, yp - 45), col, 2)
                cv2.line(im, (xx, yp + 45), (xx, min(479, yp + 110)), col, 2)
        if x_end is not None:
            cv2.line(im, (X(x_end), 470), (X(x_end), 479), (0, 255, 255), 3)
        if x_origin is not None:
            cv2.line(im, (X(x_origin), 470), (X(x_origin), 479), (255, 255, 0), 3)
        tt = r["tm"] - t_off if t_off is not None else r["tm"]
        lab = f"t={tt:+.1f}s  {'FLAME' if r.get('flame') else ('det' if r.get('det') else 'none')} sc={r['score']:.0f}"
        cv2.putText(im, lab, (6, 470), cv2.FONT_HERSHEY_SIMPLEX, 0.6, (0, 255, 255), 2)
        panels.append(im)
    cap.release()
    while len(panels) < 6:
        panels.append(np.zeros((480, 720, 3), np.uint8))
    grid = np.vstack([np.hstack(panels[0:3]), np.hstack(panels[3:6])])
    grid = cv2.resize(grid, (grid.shape[1] * 2 // 3, grid.shape[0] * 2 // 3), interpolation=cv2.INTER_AREA)
    bar = np.zeros((36, grid.shape[1], 3), np.uint8)
    cv2.putText(bar, title, (8, 25), cv2.FONT_HERSHEY_SIMPLEX, 0.7, (255, 255, 255), 2)
    Path(path).parent.mkdir(parents=True, exist_ok=True)
    cv2.imwrite(str(path), np.vstack([bar, grid]), [cv2.IMWRITE_JPEG_QUALITY, 85])


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--video", required=True)
    ap.add_argument("--run-id", required=True)
    ap.add_argument("--partition", default=None)
    ap.add_argument("--material", default=None)
    ap.add_argument("--source-key", default=None)
    ap.add_argument("--source-sha256", default=None)
    ap.add_argument("--out", required=True)
    ap.add_argument("--sheet", default=None)
    ap.add_argument("--ovl", default=None)
    ap.add_argument("--title-hidden", action="store_true", help="contact-sheet title without the outcome")
    args = ap.parse_args()
    D = decode(args.video, keep_ovl=args.ovl is not None)
    if not D["wins"]:
        raise SystemExit(f"no frames decoded from {args.video}")
    # frame-level lit flag from window lit flags (computed after measure); first a provisional one
    lvl = D["frame_lvl"]
    lit_frame = lvl > np.percentile(lvl, 10) + LIT_DELTA
    fd = np.array([d is not None for d in D["frame_dx"]])
    if fd.mean() > 0.5:
        lit_frame = lit_frame | ~fd
    ign = igniter(D, lit_frame)
    M = measure_windows(D, ign)
    mat = (args.material or "")
    sibal = "sibal" in mat.lower()
    width_cm = None
    if sibal:
        import re
        mm = re.search(r"(\d+(?:\.\d+)?)\s*cm", mat.lower())
        width_cm = float(mm.group(1)) if mm else None
    out = {"schema": "wsp-bass-front-track/1", "run_id": args.run_id, "partition": args.partition,
           "measurement": {"method_version": METHOD_VERSION, "code_sha256": file_sha(__file__),
                           "view": "duct (main) video", "measured_blind": True},
           "source": {"video": Path(args.video).name, "key": args.source_key, "sha256": args.source_sha256,
                      "fps": D["fps"], "frames": D["n_frames"], "frame_px": D["size"]},
           "material": args.material}
    if M is None:
        out.update(quality="failed", outcome=dict(kind="cannot_tell", confidence="low", reason="fewer than 4 dark windows"),
                   duct=[], notes=["no usable dark record"])
        Path(args.out).parent.mkdir(parents=True, exist_ok=True)
        Path(args.out).write_text(json.dumps(out, indent=1), encoding="utf-8")
        print(json.dumps({"run": args.run_id, "outcome": "cannot_tell", "quality": "failed"}))
        return 0
    det = detect(M, ign)
    rows = M["rows"]
    x_origin = x_end = None
    direction = spread_direction(ign)
    geometry_note = None
    if ign:
        x_origin = (ign["box_x"][1] + ORIGIN_FW) if direction < 0 else (ign["box_x"][0] - ORIGIN_FW)
        x_end = x_origin + direction * WINDOW_FW
        if sibal and not (IGN_EDGE_RANGE[0] <= ign["box_x"][1] <= IGN_EDGE_RANGE[1]):
            # igniter not where the train holders had it (moved igniter, re-burn): the sample end
            # is taken where the train tips stopped, positions stay relative to the igniter
            x_end = X_END_FIXED if direction < 0 else None
            geometry_note = (f"igniter box edge at {ign['box_x'][1]:.3f} fw is outside the train range; sample end "
                             f"{'taken at the fixed train position' if x_end is not None else 'unknown'}; positions are relative to the igniter")
    oc, end, rec_end = classify(rows, ign, sibal, x_end if sibal else None, direction)
    t_off = ign["t_off"] if ign else 0.0
    wscale = None
    if sibal and width_cm:
        wscale = W_PER_FW * (2.0 / width_cm)
    duct = []
    for r in rows:
        if not r["flame"]:
            continue
        e = {"t_s": round(r["tm"] - t_off, 3), "tip_fw": round(r["tip_fw"], 4), "base_fw": round(r["base_fw"], 4),
             "y_top": round(r["y_top"], 4), "y_bottom": round(r["y_bot"], 4), "area_px": r["area"], "score": round(r["score"], 1)}
        if wscale is not None and x_origin is not None:
            e["tip_w"] = round(-direction * (x_origin - r["tip_fw"]) * wscale, 3)
            e["base_w"] = round(-direction * (x_origin - r["base_fw"]) * wscale, 3)
        duct.append(e)
    ignition_t = None
    if ign and oc["kind"] != "never_lit":
        for r in rows:
            if r["tip_fw"] is None:
                continue
            beyond = (r["tip_fw"] < ign["box_x"][0] - 0.02) if direction < 0 else (r["tip_fw"] > ign["box_x"][1] + 0.02)
            if r["t0"] >= ign["t_on"] and r["on_plane"] and r["tip_fw"] is not None and beyond:
                ignition_t = round(r["tm"] - t_off, 2)
                break
    n_dark = int(M["dark"].sum())
    quality = "ok"
    notes = []
    if ign is None:
        quality = "failed"
        notes.append("igniter not found")
    if geometry_note:
        notes.append(geometry_note)
        quality = "suspect_geometry"
    if not M["has_overlay"]:
        notes.append("no overlay found in most windows: positions are recorded coordinates, no fan/air/rad")
    if ign and len(ign["bursts"]) > 1:
        notes.append(f"{len(ign['bursts'])} igniter bursts in this record; t = 0 is the end of the last one")
    out.update({
        "time_origin": "igniter off: the last video frame whose hottest 16-px block is at least 0.7 x its median over the igniter burst; t_s = video seconds after it",
        "position_unit": "fw (fraction of the 720-px frame width, registered to the overlay); tip_w/base_w in sample widths where a scale exists",
        "position_origin_fw": round(x_origin, 4) if x_origin is not None else None,
        "sample_end_fw": round(x_end, 4) if x_end is not None else None,
        "metres": None,
        "metres_note": "no length calibration is established for these videos (sample width 2.0 vs 2.2 cm, holder position); positions are not given in metres",
        "width_scale_w_per_fw": wscale,
        "spread_direction": ("toward smaller x (igniter at the right)" if direction < 0 else "toward larger x (igniter at the left)"),
        "igniter": ({"on_t_s": round(ign["t_on"] - t_off, 3), "off_t_s": 0.0, "off_video_s": round(ign["t_off"], 3),
                     "box_fw": [round(v, 4) for v in ign["box_x"]], "box_fh": [round(v, 4) for v in ign["box_y"]]} if ign else None),
        "detection": {k: (round(v, 2) if isinstance(v, float) else v) for k, v in det.items()},
        "record": {"duration_s": round(D["n_frames"] / D["fps"], 2), "windows": len(rows), "dark_windows": n_dark,
                   "lit_windows": int(M["lit"].sum()), "off_scene_windows": int(sum(r["off_scene"] for r in rows)),
                   "dark_record_end_t_s": round(rec_end, 2) if rec_end is not None else None,
                   "has_overlay": M["has_overlay"]},
        "ignition_t_s": ignition_t,
        "ignition_note": "first window (from igniter on) in which the luminous region on the sample plane reaches 0.02 fw downstream of the igniter box; null when never lit",
        "outcome": oc,
        "end": end,
        "quality": quality,
        "duct": duct,
        "windows": [{"t_s": round(r["tm"] - t_off, 3), "state": ("lit" if r["lit"] else "off_scene" if r["off_scene"] else
                                                                 "flame" if r["flame"] else "detected_off_plane_or_igniter" if r["det"] else "none"),
                     "score": round(r["score"], 1)} for r in rows],
        "notes": notes,
    })
    Path(args.out).parent.mkdir(parents=True, exist_ok=True)
    Path(args.out).write_text(json.dumps(out, indent=1), encoding="utf-8")
    if args.ovl:
        Path(args.ovl).parent.mkdir(parents=True, exist_ok=True)
        if D["ovl"]:
            np.savez_compressed(args.ovl, t=np.array([o[0] for o in D["ovl"]]), dx=np.array([o[1] for o in D["ovl"]]),
                                a=np.array([o[2] for o in D["ovl"]]),
                                has=np.array([o[3] for o in D["ovl"]]), strips=np.stack([o[4] for o in D["ovl"]]),
                                t_off=np.array([t_off]))
        if D["ovl"]:
            import importlib.util
            spec = importlib.util.spec_from_file_location("bass_overlay", Path(__file__).with_name("overlay.py"))
            ovm = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(ovm)
            ser = ovm.series(args.ovl)
            good = [r for r in ser if r["clock_s"] is not None]
            clock0 = None
            if len(good) >= 5:
                a_ = np.array([(r["video_t"], r["clock_s"]) for r in good], float)
                k = np.polyfit(a_[:, 0], a_[:, 1], 1)
                res = a_[:, 1] - np.polyval(k, a_[:, 0])
                keep = np.abs(res) < 2
                if keep.sum() >= 3:
                    k = np.polyfit(a_[keep, 0], a_[keep, 1], 1)
                    c0 = float(np.polyval(k, t_off)) % 86400
                    clock0 = "%02d:%02d:%05.2f" % (c0 // 3600, (c0 % 3600) // 60, c0 % 60)
            out["overlay"] = {"note": "read from the video overlay by glyph matching (templates from train videos); one reading per second; null = cell not read; RAD's last digit is cut by the frame edge in most recordings",
                              "clock_at_t0": clock0, "date_text": (good[0]["date"] if good else None),
                              "series": [{"t_s": round(r["video_t"] - t_off, 2), "fan": r["fan"], "air": r["air"], "rad": r["rad"],
                                          "rad_last_digit_read": r["rad_last_digit_read"], "clock": r["clock"]} for r in ser]}
            Path(args.out).write_text(json.dumps(out, indent=1), encoding="utf-8")
    if args.sheet:
        title = f"{args.run_id}  {Path(args.video).name[:60]}"
        if not args.title_hidden:
            title += f"  -> {oc['kind']} ({oc['confidence']})"
        contact_sheet(args.video, D, rows, ign, x_end, x_origin, args.sheet, title, det.get("plane_y"))
    print(json.dumps({"run": args.run_id, "outcome": oc["kind"], "conf": oc["confidence"], "quality": quality,
                      "n_flame": len(duct)}) if args.partition == "train" else json.dumps({"run": args.run_id, "done": True}))
    return 0


if __name__ == "__main__":
    sys.exit(main())
