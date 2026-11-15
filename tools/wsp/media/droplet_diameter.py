"""Droplet diameter against time from the backlit HiBMs frames of a FLEX / FLEX-2 test point.

Input: the camera-A archive `<tp>A.zip` (16-bit TIFF frames, droplet dark on a bright backlight) and, when
present beside it, the frame index `<tp>A.txt`, the timeline `<tp>.txt` and the HiBMs camera settings.
No per-test tuning: every threshold below is either relative (fractions of the local backlight level) or
expressed in millimetres and converted with the pixel scale.

Output per frame: time on the test clock (s, t = 0 at "AS Burn to Completion/Igniter On" of the timeline),
area-equivalent diameter (m), centroid (px), quality flag. Plus a JSON header with the scale and its
source, the time origin, and a check of the camera clock against the needle retraction seen in the frames.

    python -I tools/wsp/media/droplet_diameter.py <dir-with-files-or-zip> --out out_prefix [--sheet dir]
        [--investigation FLEX|FLEX-2] [--fov-mm F | --scale-px-per-mm S --scale-source "..."] [--step 1]

Method (per frame):
 1. backlight = grey closing (6 mm disc) of the frame: dark objects up to ~6 mm removed, flame halos and
    illumination gradients kept; normalised image n = frame / backlight;
 2. dark mask n < 0.5; enclosed holes filled; opening with a 0.3 mm disc cuts needles, liquid bridges,
    igniter wire and fibres from the droplet;
 3. candidates = components with equivalent diameter >= 0.25 mm and size <= 8 mm; "good" if solidity
    >= 0.90 and ellipse axis ratio >= 0.75; the candidate nearest the previous droplet (position and size)
    or, without a track, the largest one is the droplet;
 4. diameter: edge = steepest intensity rise along 360 rays from the centroid (sub-pixel, searched
    within +-max(3 px, 0.1 mm) of a threshold outline), area of that outline -> area-equivalent D.
    The threshold diameter (half level between local backlight and droplet rim, sub-pixel soft area)
    is kept as a diagnostic column and as the fallback when fewer than 80 % of the rays find an edge;
 5. flags: pre_deploy (before needle retraction), no_droplet, edge (touches the frame border: partly out
    of view), shape (no candidate passes the shape test: on needles, igniter contact, merged soot,
    deformation), glare (rim contrast < 70 % of the test's typical contrast: igniter or flame light on
    the droplet), soot (mean grey attenuation >= 5 % in an annulus 1.25-2.5 R: soot shell; informative),
    outlier (> 6 % off the running median of its accepted neighbours), lost (not reachable from the
    confirmed track at 10 mm/s + 2 D, larger than the droplet after a gap, mid-field after leaving through
    the border, or a run of fewer than 5 consistent frames), static (centroid fixed within 0.1 px for 2 s:
    dust or a sensor defect), unreadable, ok.  Usable for a diameter series: ok and soot.
 Frames are smoothed before all of this (Gaussian 0.7 px, more when the frame carries laser speckle).
 The track (which object is the droplet) starts at needle retraction; nothing on the needles carries over.
"""
from __future__ import annotations

import argparse
import json
import math
import re
import sys
import time
import zipfile
from datetime import datetime
from pathlib import Path

import cv2
import numpy as np

# Documented HiBMs field of view over the full sensor (TP15 §3.1.2 error section: 30.25 mm).
FOV_DOC_MM = {"FLEX": 30.25}
# FLEX-2 / CFI: "about 5 x 5 cm" in the literature (F19, S4). Not a measurement; flagged as assumed.
FOV_ASSUMED_MM = {"FLEX-2": 50.0}
USABLE = ("ok", "soot")


# ----------------------------------------------------------------------------------------- metadata
def _lines(text: str):
    return text.replace("\r\n", "\n").replace("\r", "\n").split("\n")


def parse_frame_index(text: str):
    """ImgNum -> (file stem, datetime of the camera time stamp)."""
    out = {}
    for line in _lines(text):
        p = [x.strip() for x in line.split(",")]
        if len(p) < 9 or not p[0].isdigit():
            continue
        try:
            ts = datetime.strptime(p[6] + " " + p[7], "%m/%d/%Y %H:%M:%S.%f")
        except ValueError:
            continue
        out[int(p[0])] = (p[8], ts)
    return out


def parse_timeline(text: str):
    ev = {}
    for line in _lines(text):
        m = re.match(r"^(AS [^:]+|Igniter On Time[^:]*|Burn to Completion[^:]*):\s*(.*)$", line.strip())
        if m:
            v = m.group(2).strip()
            try:
                ev[m.group(1)] = datetime.strptime(v, "%Y-%m-%d %H:%M:%S.%f")
            except ValueError:
                ev[m.group(1)] = v
    return ev


def parse_settings(text: str):
    rows = [r for r in _lines(text) if r.strip()]
    if len(rows) < 2:
        return {}
    head = [h.strip('"').split("(")[0] for h in rows[0].split('","')]
    vals = rows[1].split(",")
    return {h.strip('"'): v.strip().strip('"') for h, v in zip(head, vals)}


# ----------------------------------------------------------------------------------------- inputs
class Inputs:
    def __init__(self, src: Path):
        src = Path(src)
        self.dir = src if src.is_dir() else src.parent
        zips = [src] if src.suffix.lower() == ".zip" else sorted(self.dir.glob("*A.zip"))
        if not zips:
            raise SystemExit(f"no <test>A.zip in {src}")
        self.zip_path = zips[0]
        self.tp = re.sub(r"A$", "", self.zip_path.stem.split("_")[-1])
        self.z = zipfile.ZipFile(self.zip_path)
        self.members = {Path(n).stem: n for n in self.z.namelist() if n.lower().endswith((".tif", ".tiff"))}

        def sibling(pattern):
            c = sorted(self.dir.glob(pattern))
            return c[0].read_text(encoding="utf-8", errors="replace") if c else None

        idx = sibling(f"*{self.tp}A.txt")
        if idx is None:  # the archive itself usually carries the index
            inner = [n for n in self.z.namelist() if n.lower().endswith("a.txt")]
            idx = self.z.read(inner[0]).decode("utf-8", "replace") if inner else None
        self.index = parse_frame_index(idx) if idx else {}
        tl = sibling(f"*_{self.tp}.txt")
        self.events = parse_timeline(tl) if tl else {}
        st = sibling(f"*{self.tp}_HI*MS*_Camera_Settings.txt")
        self.settings = parse_settings(st) if st else {}
        self.frames = []  # (imgnum, member)
        for n, member in self.members.items():
            m = re.search(r"_(\d+)$", n)
            if m:
                self.frames.append((int(m.group(1)), member))
        self.frames.sort()

    def read(self, member):
        b = self.z.read(member)
        return cv2.imdecode(np.frombuffer(b, np.uint8), cv2.IMREAD_UNCHANGED)


# ----------------------------------------------------------------------------------------- scale and clock
def pixel_scale(inp: Inputs, width_px: int, investigation: str, fov_mm=None, scale=None, scale_source=None):
    """Return px/mm of the archived image and a plain statement of where it came from."""
    if scale:
        return scale, scale_source or "given on the command line"
    binning = inp.settings.get("DCM_MP_CAM_BINNING", "")
    xres = inp.settings.get("IPSU_TP_XRES", "")
    if fov_mm:
        return width_px / fov_mm, f"field of view {fov_mm} mm given on the command line over {width_px} px"
    if investigation in FOV_DOC_MM:
        f = FOV_DOC_MM[investigation]
        return width_px / f, (f"documented HiBMs field of view {f} mm (TP15 §3.1.2) over the image width "
                              f"{width_px} px (settings XRES={xres or '?'}, binning={binning or '?'}); "
                              "not measured in the image")
    f = FOV_ASSUMED_MM.get(investigation, 50.0)
    return width_px / f, (f"ASSUMED field of view {f} mm (literature: about 5 x 5 cm for FLEX-2/CFI) over "
                          f"{width_px} px; no calibration applied - diameters carry the full scale error")


def clock(inp: Inputs):
    """Camera frame -> test time (s from igniter on, rack clock).

    The camera stamps of `<tp>A.txt` are used directly against the timeline's igniter-on stamp; run()
    checks this against the needle-retraction step seen in the frames (`clock_check`). The timeline's
    "Start Image" note is not used: on FLEX it disagrees with the frame events by seconds.
    """
    ign = inp.events.get("AS Burn to Completion/Igniter On")
    if not isinstance(ign, datetime) or not inp.index:
        return None, "no timeline or frame index: time is frame number / 30 s, origin unknown"

    def fn(img):
        e = inp.index.get(img)
        return None if e is None else (e[1] - ign).total_seconds()
    return fn, "camera time stamp (<tp>A.txt) minus timeline 'AS Burn to Completion/Igniter On'"


# ----------------------------------------------------------------------------------------- per frame
def background(f, pxmm):
    """Backlight under dark objects up to ~6 mm: grey closing on a 4x reduced image, smoothed."""
    h, w = f.shape
    q = 4
    small = cv2.resize(f, (w // q, h // q), interpolation=cv2.INTER_AREA)
    r = max(3, int(round(3.0 * pxmm / q)))
    k = cv2.getStructuringElement(cv2.MORPH_ELLIPSE, (2 * r + 1, 2 * r + 1))
    bg = cv2.morphologyEx(small, cv2.MORPH_CLOSE, k)
    bg = cv2.GaussianBlur(bg, (0, 0), r / 2)
    return cv2.resize(bg, (w, h), interpolation=cv2.INTER_LINEAR)


def edge_diameter(f, cx, cy, r0, half_window, n_rays=360, step=0.2):
    """Area-equivalent diameter (px) of the steepest-rise outline around (cx, cy), or None."""
    if r0 < 2:
        return None, 0.0
    rs = np.arange(max(0.5, r0 - half_window), r0 + half_window + step, step, dtype=np.float32)
    th = np.linspace(0, 2 * np.pi, n_rays, endpoint=False, dtype=np.float32)
    X = cx + rs[:, None] * np.cos(th)[None, :]
    Y = cy + rs[:, None] * np.sin(th)[None, :]
    prof = cv2.remap(f, X.astype(np.float32), Y.astype(np.float32), cv2.INTER_LINEAR,
                     borderMode=cv2.BORDER_CONSTANT, borderValue=float("nan"))
    s = 0.6 / step  # smooth along each ray (sigma 0.6 px) before differentiating
    kx = np.exp(-0.5 * (np.arange(-3 * int(s), 3 * int(s) + 1) / s) ** 2)
    kx = (kx / kx.sum()).astype(np.float32)
    prof = cv2.sepFilter2D(prof, -1, np.ones(1, np.float32), kx, borderType=cv2.BORDER_REPLICATE)
    g = np.diff(prof, axis=0)
    valid = np.isfinite(g).all(axis=0)
    g = np.where(np.isfinite(g), g, -np.inf)
    k = np.argmax(g, axis=0)
    inside = (k > 0) & (k < g.shape[0] - 1)
    kk = np.clip(k, 1, g.shape[0] - 2)
    cols = np.arange(n_rays)
    gm, g0, gp = g[kk - 1, cols], g[kk, cols], g[kk + 1, cols]
    den = gm - 2 * g0 + gp
    off = np.where(np.abs(den) > 1e-9, 0.5 * (gm - gp) / den, 0.0)
    r = rs[0] + (kk + 0.5 + np.clip(off, -0.5, 0.5)) * step
    ok = valid & inside & (g0 > 0)
    if ok.mean() < 0.8:
        return None, float(ok.mean())
    r = np.where(ok, r, np.nan)
    # suppress single-ray outliers (soot particles, specks) with a circular median over 9 rays
    rr = np.concatenate([r[-4:], r, r[:4]])
    med = np.array([np.nanmedian(rr[i:i + 9]) for i in range(n_rays)])
    bad = ~np.isfinite(r) | (np.abs(r - med) > 1.0)
    r = np.where(bad, med, r)
    if not np.isfinite(r).all():
        return None, float(ok.mean())
    area = 0.5 * float(np.sum(r ** 2)) * (2 * np.pi / n_rays)
    return 2.0 * math.sqrt(area / math.pi), float(ok.mean())


def measure_frame(im, pxmm, prev, sat_level):
    h, w = im.shape
    raw = im.astype(np.float32)
    sat = raw >= sat_level
    # noise-adaptive smoothing: some tests are lit above the laser threshold and carry coherent speckle
    # (grain of a few pixels, +-40 %); the smoothing grows with the measured noise, capped at 0.1 mm
    f0 = cv2.GaussianBlur(raw, (0, 0), 0.7)
    sub = (slice(None, None, 4), slice(None, None, 4))
    resid = (raw[sub] - f0[sub]) / np.maximum(f0[sub], 1.0)
    noise = float(1.4826 * np.median(np.abs(resid - np.median(resid))))
    sigma = float(np.clip(0.7 * noise / 0.02, 0.7, max(0.7, 0.1 * pxmm)))
    f = f0 if sigma <= 0.71 else cv2.GaussianBlur(raw, (0, 0), sigma)
    bgimg = background(f, pxmm)
    n = f / np.maximum(bgimg, 1.0)
    mask = (n < 0.5).astype(np.uint8)
    dark_frac = float(mask.mean())
    # fill enclosed holes (bright spot at the centre of a lens-like droplet)
    nl, lb, st, _ = cv2.connectedComponentsWithStats(1 - mask, 4)
    x_, y_, bw_, bh_, a_ = st[:, 0], st[:, 1], st[:, 2], st[:, 3], st[:, 4]
    inside = (x_ > 0) & (y_ > 0) & (x_ + bw_ < w) & (y_ + bh_ < h) & (a_ < 0.02 * h * w)
    inside[0] = False
    mask[inside[lb]] = 1
    r_open = max(1, int(round(0.15 * pxmm)))
    k = cv2.getStructuringElement(cv2.MORPH_ELLIPSE, (2 * r_open + 1, 2 * r_open + 1))
    op = cv2.morphologyEx(mask, cv2.MORPH_OPEN, k)
    nlab, lab, stats, cents = cv2.connectedComponentsWithStats(op, 8)
    dmin_px = 0.25 * pxmm
    cands = []
    for i in range(1, nlab):
        a = stats[i, cv2.CC_STAT_AREA]
        if a < math.pi * (dmin_px / 2) ** 2:
            continue
        x, y, bw, bh = stats[i, :4]
        if max(bw, bh) > 8.0 * pxmm:  # larger than any droplet: holders, needles
            continue
        sub = (lab[y:y + bh, x:x + bw] == i).astype(np.uint8)
        cs, _ = cv2.findContours(sub, cv2.RETR_EXTERNAL, cv2.CHAIN_APPROX_NONE)
        c = max(cs, key=cv2.contourArea)
        hull = cv2.contourArea(cv2.convexHull(c))
        sol = a / hull if hull > 0 else 0
        ar = 0.0
        if len(c) >= 5:
            (_, _), (ma, mi), _ = cv2.fitEllipse(c)
            ar = min(ma, mi) / max(ma, mi) if max(ma, mi) > 0 else 0
        edge = x <= 1 or y <= 1 or x + bw >= w - 1 or y + bh >= h - 1
        cands.append(dict(i=i, area=a, cx=cents[i][0], cy=cents[i][1], sol=sol, ar=ar, edge=edge,
                          box=(x, y, bw, bh), contour=c + np.array([[x, y]])))
    good = [c for c in cands if c["sol"] >= 0.90 and c["ar"] >= 0.75]
    pick, flag = None, "ok"
    pool = good if good else cands
    if pool:
        if prev is not None:
            def cost(c):
                d = math.hypot(c["cx"] - prev[0], c["cy"] - prev[1])
                s = abs(math.sqrt(c["area"]) - math.sqrt(prev[2])) / max(math.sqrt(prev[2]), 1)
                return d / max(math.sqrt(prev[2]), 1) + 3 * s
            pick = min(pool, key=cost)
        else:
            pick = max(pool, key=lambda c: c["area"])
        if not good:
            flag = "shape"
    if pick is None:
        return dict(flag="no_droplet", dark_frac=dark_frac, noise=noise, smooth_px=sigma)
    # sub-pixel area: soft step centred on the half level between local backlight and droplet core
    x, y, bw, bh = pick["box"]
    rr = int(math.ceil(0.75 * max(bw, bh))) + 4
    x0, y0, x1, y1 = max(0, x - rr), max(0, y - rr), min(w, x + bw + rr), min(h, y + bh + rr)
    froi = f[y0:y1, x0:x1]
    comp = (lab[y0:y1, x0:x1] == pick["i"]).astype(np.uint8)
    rd = max(2.0, 0.08 * pxmm)
    dout = cv2.distanceTransform(1 - comp, cv2.DIST_L2, 3)
    din = cv2.distanceTransform(comp, cv2.DIST_L2, 3)
    near = dout <= rd
    ring = (dout > rd + 1) & (dout <= max(rd + 4, 0.5 * max(bw, bh)))
    other_dark = (mask[y0:y1, x0:x1] > 0) & ~near
    ring_vals = froi[ring & ~other_dark]
    bg_roi = float(np.median(bgimg[y0:y1, x0:x1]))
    local_bg = float(np.median(ring_vals)) if ring_vals.size > 20 else bg_roi
    inner = din > rd
    core = float(np.percentile(froi[inner], 10)) if inner.sum() > 4 else float(froi[comp > 0].min())
    # threshold diameter (diagnostic and fallback): half level between local backlight and droplet rim
    level = 0.5 * (local_bg + core)
    width = max(1.0, 0.12 * (local_bg - core))
    soft = np.clip((level - froi) / width + 0.5, 0.0, 1.0)
    area = float(soft[near].sum())
    d_thr = 2.0 * math.sqrt(area / math.pi)
    # primary diameter: edge = steepest rise of intensity along 360 rays from the centroid (sub-pixel,
    # searched within +-max(3 px, 0.1 mm) of the threshold radius); area of that outline -> D.
    # The steepest-rise edge does not move when the rim darkens or the interior transmission changes.
    d_grad, ok_rays = edge_diameter(f, pick["cx"], pick["cy"], d_thr / 2, max(3.0, 0.1 * pxmm))
    d_px = d_grad if d_grad is not None else d_thr
    contrast = (local_bg - core) / max(local_bg, 1.0)
    ring_sat = float(sat[y0:y1, x0:x1][ring].mean()) if ring.any() else 0.0
    # soot shell: the annulus 1.25-2.5 R around the droplet is on average >= 5 % darker than the clear
    # backlight of the frame (speckle and noise average out; a soot shell or cloud does not)
    R = d_px / 2
    ann = (dout > 0.25 * R + rd) & (dout < 1.5 * R) & ~other_dark
    soot = False
    if ann.sum() > 20:
        nsub = n[::4, ::4]
        clear = float(np.median(nsub[nsub > 0.7])) if np.any(nsub > 0.7) else 1.0
        ann_n = float(np.mean(froi[ann] / np.maximum(bgimg[y0:y1, x0:x1][ann], 1.0)))
        soot = ann_n < clear - 0.05
    if pick["edge"]:
        flag = "edge"
    elif soot and flag == "ok":
        flag = "soot"
    return dict(flag=flag, d_px=d_px, d_thr_px=d_thr, rays_ok=ok_rays, cx=pick["cx"], cy=pick["cy"], area_px=area,
                sol=pick["sol"], ar=pick["ar"],
                local_bg=local_bg, core=core, contrast=contrast, ring_sat=ring_sat, soot=soot,
                contour=pick["contour"], n_cands=len(cands), dark_frac=dark_frac, noise=noise, smooth_px=sigma)


# ----------------------------------------------------------------------------------------- ruler check
# Width of the straight dark rods (needle arms) in the first frame, measured on FLEX TRAIN tests with the
# documented FLEX scale (PILOT_REPORT.md): 1.606-1.725 mm on six tests, a 7 % spread (more than one rod type in view),
# so it is a sanity check of the scale (catches a wrong field of view), not a calibration.
ROD_FLEX_MM = (1.60, 1.73)


def rod_width_px(im, pxmm_guess):
    """Median width (px) of straight dark rods 0.6-2.4 mm wide, from sub-pixel edge pairs; or None."""
    f = im.astype(np.float32)
    n = f / np.maximum(background(f, pxmm_guess), 1.0)
    mask = (n < 0.5).astype(np.uint8)
    dt = cv2.distanceTransform(mask, cv2.DIST_L2, 5)
    dil = cv2.dilate(dt, np.ones((3, 3), np.uint8))
    ys, xs = np.nonzero((dt >= dil - 1e-3) & (dt > 0.3 * pxmm_guess) & (dt < 1.2 * pxmm_guess))
    if len(xs) < 20:
        return None
    pts = np.stack([xs, ys], 1).astype(np.float32)
    sel = np.random.default_rng(0).choice(len(pts), size=min(600, len(pts)), replace=False)
    widths = []
    for k in sel:
        p = pts[k]
        hw = float(dt[int(p[1]), int(p[0])])
        nb = pts[np.sum((pts - p) ** 2, 1) < (1.5 * hw) ** 2]
        if len(nb) < 8:
            continue
        _, s, vt = np.linalg.svd(nb - nb.mean(0), full_matrices=False)
        if s[1] > 0.15 * s[0]:
            continue  # not a straight ridge
        axis = vt[0]
        perp = np.array([-axis[1], axis[0]], np.float32)
        ts = np.arange(-2.0 * hw, 2.0 * hw, 0.1, dtype=np.float32)
        prof = np.zeros_like(ts)
        for a in np.linspace(-0.5 * hw, 0.5 * hw, 5):
            q = p + a * axis
            X = (q[0] + ts * perp[0]).reshape(1, -1).astype(np.float32)
            Y = (q[1] + ts * perp[1]).reshape(1, -1).astype(np.float32)
            prof += cv2.remap(n, X, Y, cv2.INTER_LINEAR, borderMode=cv2.BORDER_CONSTANT, borderValue=float("nan"))[0]
        if not np.isfinite(prof).all():
            continue
        kx = np.exp(-0.5 * (np.arange(-15, 16) / 5.0) ** 2)
        g = np.gradient(np.convolve(prof, kx / kx.sum(), mode="same"))
        half = len(g) // 2

        def refine(i):
            if 0 < i < len(g) - 1:
                den = g[i - 1] - 2 * g[i] + g[i + 1]
                return i + (0.5 * (g[i - 1] - g[i + 1]) / den if den != 0 else 0.0)
            return float(i)
        widths.append((refine(half + int(np.argmax(g[half:]))) - refine(int(np.argmin(g[:half])))) * 0.1)
    if len(widths) < 10:
        return None
    wv = np.array(widths)
    wv = wv[np.abs(wv - np.median(wv)) < 0.1 * np.median(wv)]
    return float(np.median(wv)), len(wv)


# ----------------------------------------------------------------------------------------- run
def run(src, investigation="FLEX", fov_mm=None, scale=None, scale_source=None, step=1, sheet_dir=None,
        sheet_every=None, limit=None):
    t0 = time.time()
    inp = Inputs(Path(src))
    frames = inp.frames[::step]
    if limit:
        frames = frames[:limit]
    first = inp.read(frames[0][1])
    h, w = first.shape[:2]
    pxmm, scale_src = pixel_scale(inp, w, investigation, fov_mm, scale, scale_source)
    rod = rod_width_px(first if first.ndim == 2 else cv2.cvtColor(first, cv2.COLOR_BGR2GRAY), pxmm)
    ruler = None
    if rod:
        mm = rod[0] / pxmm
        ruler = dict(rod_width_px=round(rod[0], 3), rod_sections=rod[1], rod_width_mm_at_scale=round(mm, 4),
                     flex_reference_mm=list(ROD_FLEX_MM),
                     verdict="consistent" if ROD_FLEX_MM[0] * 0.97 <= mm <= ROD_FLEX_MM[1] * 1.03 else
                     "INCONSISTENT: the scale or the rods differ from FLEX; check the field of view")
    tfn, clock_desc = clock(inp)
    ign = inp.events.get("AS Burn to Completion/Igniter On")

    def rel(key):
        e = inp.events.get(key)
        return (e - ign).total_seconds() if isinstance(e, datetime) and isinstance(ign, datetime) else None
    t_needles = rel("AS Needles Retract")
    sat = 0.995 * (65535 if first.dtype == np.uint16 else 255)
    rows, prev, miss = [], None, 0
    contours = {}
    for img, member in frames:
        t = tfn(img) if tfn else img / 30.0
        try:
            im = inp.read(member)
        except Exception:
            im = None
        if im is None:
            rows.append(dict(frame=img, t_s=t, flag="unreadable"))
            continue
        if im.ndim == 3:
            im = cv2.cvtColor(im, cv2.COLOR_BGR2GRAY)
        r = measure_frame(im, pxmm, prev, sat)
        before_deploy = t is not None and t_needles is not None and t < t_needles + 0.15
        if before_deploy:
            prev = None  # the track starts at deployment: nothing seen on the needles is carried over
        elif "d_px" in r and r["flag"] in ("ok", "soot"):
            prev = (r["cx"], r["cy"], r["area_px"])
            miss = 0
        else:
            miss += 1
            if miss > 15:
                prev = None
        c = r.pop("contour", None)
        if c is not None:
            contours[img] = c
        rows.append(dict(frame=img, t_s=t, **r))
    # clock check: the needle-retraction step (largest one-frame change of the dark area within 1.5 s of
    # the stamped retraction) seen in the frames, against the timeline stamp
    clock_check = None
    if t_needles is not None and step == 1:
        cand = [(abs(rows[i]["dark_frac"] - rows[i - 1]["dark_frac"]), rows[i]["t_s"]) for i in range(1, len(rows))
                if rows[i].get("t_s") is not None and "dark_frac" in rows[i] and "dark_frac" in rows[i - 1]
                and abs(rows[i]["t_s"] - t_needles) <= 1.5]
        if cand:
            jump, t_seen = max(cand)
            clock_check = dict(t_needles_retract_stamp_s=t_needles, t_needles_retract_seen_s=t_seen,
                               seen_minus_stamp_s=round(t_seen - t_needles, 3), dark_area_step=round(jump, 4))
    t_dep = t_needles
    if clock_check and abs(clock_check["seen_minus_stamp_s"]) < 1.0:
        t_dep = clock_check["t_needles_retract_seen_s"]
    for r in rows:
        # 0.1 s margin: the needle tips take a few frames to leave the droplet
        if t_dep is not None and r.get("t_s") is not None and r["t_s"] < t_dep + 0.1 and r["flag"] != "unreadable":
            r["flag"] = "pre_deploy"
    # glare: light added on the droplet (igniter glow, flame luminosity with a long shutter) lifts its
    # rim; judged against this test's own typical rim contrast so no absolute level is needed
    cs = [r["contrast"] for r in rows if r.get("flag") in ("ok", "soot") and "contrast" in r]
    c_typ = float(np.median(cs)) if cs else None
    for r in rows:
        if c_typ and r.get("flag") in ("ok", "soot") and r["contrast"] < 0.7 * c_typ:
            r["flag"] = "glare"
    # lost track: in every frame the object must be within reach of the last droplet position (10 mm/s,
    # well above FLEX drift of 1-8 mm/s, plus 2 D); after a gap > 0.5 s (burnt out, drifted out, hidden) it
    # must also not be larger than the last droplet (+5 %).
    # A droplet last seen leaving through the border must come back through it (first seen within one
    # diameter of the border). Otherwise the object is flagged 'lost' (soot agglomerates and debris left
    # after burnout or drift-out look like small droplets).
    # After any interruption of more than 0.2 s (and at the start) the track is only (re)confirmed by five
    # consecutive mutually consistent frames; a fleeting object (igniter fragment, flash) is flagged lost.
    def state(r):
        return (r["cx"], r["cy"], r["d_px"], r["t_s"])

    def fits(r, ref, check_border):
        gap = r["t_s"] - ref[3]
        # drift allowance capped at 1 s of 10 mm/s: a droplet hidden longer is taken back only through the
        # border it left by (check_border), never from mid-field (debris and fixtures sit there)
        reach = (gap if check_border else min(gap, 1.0)) * 10.0 * pxmm + 2 * ref[2]
        border = min(r["cx"], r["cy"], w - r["cx"], h - r["cy"]) - r["d_px"] / 2
        far = math.hypot(r["cx"] - ref[0], r["cy"] - ref[1]) > reach
        bigger = gap > 0.5 and r["d_px"] > 1.05 * ref[2]
        return not (far or bigger or (check_border and border > r["d_px"]))

    last, left_view, pend = None, False, []
    for r in rows:
        if r.get("t_s") is None or r["flag"] in ("pre_deploy", "unreadable"):
            continue
        tracking = last is not None and not pend and r["t_s"] - last[3] <= 0.2
        if r["flag"] in USABLE:
            if tracking:
                if fits(r, last, False):
                    last, left_view = state(r), False
                else:
                    r["flag"] = "lost"
                continue
            ref = state(pend[-1]) if pend else last
            if ref is not None and not fits(r, ref, left_view and not pend):
                if pend:  # the pending run broke: it was not the droplet
                    for p in pend:
                        p["flag"] = "lost"
                    pend = []
                    if last is not None and not fits(r, last, left_view):
                        r["flag"] = "lost"
                        continue
                else:
                    r["flag"] = "lost"
                    continue
            pend.append(r)
            if len(pend) >= 5:
                last, left_view, pend = state(pend[-1]), False, []
        elif r["flag"] == "edge" and "d_px" in r and last is not None and not pend:
            # only the droplet itself crossing the border counts (not an igniter swinging through it)
            if fits(r, last, False):
                last, left_view = (r["cx"], r["cy"], last[2], r["t_s"]), True
    for p in pend:
        p["flag"] = "lost"
    # fixtures: an object whose centroid comes back to within 0.1 px in 30 frames of a 2 s window is dust or a sensor defect, not a
    # free droplet (the slowest FLEX droplet seen, H08M101, still moves ~0.5 px/s)
    us = [r for r in rows if r.get("flag") in USABLE and r.get("t_s") is not None]
    ts = np.array([r["t_s"] for r in us])
    xs = np.array([r["cx"] for r in us])
    ys = np.array([r["cy"] for r in us])
    static = np.zeros(len(us), bool)
    for i in range(len(us)):  # >= 30 frames within +-1 s sit within 0.1 px of this frame's centroid
        j0, j1 = np.searchsorted(ts, ts[i] - 1.0), np.searchsorted(ts, ts[i] + 1.0)
        same = (np.abs(xs[j0:j1] - xs[i]) < 0.1) & (np.abs(ys[j0:j1] - ys[i]) < 0.1)
        static[i] = int(same.sum()) >= 30
    for r, s_ in zip(us, static):
        if s_:
            r["flag"] = "static"
    # outlier pass: running median of accepted diameters (+-5 accepted frames)
    acc = [i for i, r in enumerate(rows) if r.get("flag") in USABLE]
    d = np.array([rows[i]["d_px"] for i in acc])
    for j, i in enumerate(acc):
        lo, hi = max(0, j - 5), min(len(acc), j + 6)
        med = float(np.median(d[lo:hi]))
        if abs(d[j] - med) > 0.06 * med:
            rows[i]["flag"] = "outlier"
    for r in rows:
        if "d_px" in r:
            r["d_m"] = r["d_px"] / pxmm * 1e-3
            r["d_thr_m"] = r["d_thr_px"] / pxmm * 1e-3
    meta = dict(test_point=inp.tp, investigation=investigation, archive=inp.zip_path.name,
                frames_in_archive=len(inp.frames), frames_measured=len(rows), image_px=[w, h],
                px_per_mm=pxmm, m_per_px=1e-3 / pxmm, scale_source=scale_src,
                time_origin="timeline 'AS Burn to Completion/Igniter On' (igniter energised), rack MDCA clock",
                clock=clock_desc, clock_check=clock_check, t_deploy_used_s=t_dep, usable_flags=list(USABLE),
                typical_rim_contrast=c_typ, ruler_check=ruler,
                t_igniter_off_s=rel("AS Igniter Off/Retract Start"),
                settings={k: inp.settings.get(k) for k in ("IPSU_TP_XRES", "IPSU_TP_YRES", "DCM_MP_CAM_BINNING",
                                                          "DCM_MP_CAM_SHUTTER_SPEED", "IPSU_TP_FPS")},
                compute_s=round(time.time() - t0, 1), method_version="droplet_diameter 1")
    if sheet_dir:
        contact_sheets(inp, rows, contours, Path(sheet_dir), every=sheet_every)
        meta["compute_s_with_sheets"] = round(time.time() - t0, 1)
    return meta, rows


def contact_sheets(inp, rows, contours, out: Path, every=None, per_sheet=12, t_min=-1.0):
    """Sheets of frames from t_min on: full frame (outline green = usable, red = flagged; blue circle =
    area-equivalent diameter) with a zoomed inset of the droplet in the lower-right quarter."""
    out.mkdir(parents=True, exist_ok=True)
    last = max((r["t_s"] for r in rows if r.get("flag") in USABLE and r.get("t_s") is not None), default=None)
    t_max = last + 2.0 if last is not None else float("inf")
    pick = [r for r in rows if r.get("t_s") is not None and t_min <= r["t_s"] <= t_max]
    if every is None:
        every = max(1, len(pick) // (per_sheet * 4))
    pick = pick[::every]
    tile = 256
    members = dict(inp.frames)
    for s in range(0, len(pick), per_sheet):
        tiles = []
        for r in pick[s:s + per_sheet]:
            im = inp.read(members[r["frame"]])
            if im is None:
                continue
            if im.ndim == 3:
                im = cv2.cvtColor(im, cv2.COLOR_BGR2GRAY)
            v = cv2.normalize(im.astype(np.float32), None, 0, 255, cv2.NORM_MINMAX).astype(np.uint8)
            v = cv2.cvtColor(v, cv2.COLOR_GRAY2BGR)
            c = contours.get(r["frame"])
            col = (0, 200, 0) if r["flag"] in USABLE else (0, 0, 255)
            crop = None
            if c is not None:
                cv2.drawContours(v, [c], -1, col, 1)
            if "d_px" in r:
                cv2.circle(v, (int(round(r["cx"])), int(round(r["cy"]))), int(round(r["d_px"] / 2)), (255, 128, 0), 1)
                R = int(max(r["d_px"], 20))
                x0, y0 = max(0, int(r["cx"]) - R), max(0, int(r["cy"]) - R)
                crop = v[y0:y0 + 2 * R, x0:x0 + 2 * R]
                if crop.size:
                    crop = cv2.resize(crop, (tile // 2, tile // 2), interpolation=cv2.INTER_NEAREST)
                else:
                    crop = None
            hh, ww = v.shape[:2]
            v = cv2.resize(v, (tile, tile), interpolation=cv2.INTER_AREA)
            if crop is not None:
                # inset in the quarter farthest from the droplet, so the droplet stays visible
                ox = 0 if r["cx"] > ww / 2 else tile // 2
                oy = 16 if r["cy"] > hh / 2 else tile // 2
                v[oy:oy + tile // 2, ox:ox + tile // 2] = crop[: tile - oy, :]
                cv2.rectangle(v, (ox, oy), (ox + tile // 2 - 1, oy + tile // 2 - 1), (255, 255, 0), 1)
            d = f"{r['d_m'] * 1e3:.3f}mm" if "d_m" in r else "-"
            txt = f"#{r['frame']} t={r['t_s']:.2f} {d} {r['flag']}"
            cv2.rectangle(v, (0, 0), (tile, 16), (0, 0, 0), -1)
            cv2.putText(v, txt, (2, 12), cv2.FONT_HERSHEY_SIMPLEX, 0.38, (255, 255, 255), 1, cv2.LINE_AA)
            tiles.append(v)
        if not tiles:
            continue
        while len(tiles) % 4:
            tiles.append(np.zeros_like(tiles[0]))
        grid = np.vstack([np.hstack(tiles[i:i + 4]) for i in range(0, len(tiles), 4)])
        cv2.imwrite(str(out / f"{inp.tp}_sheet_{s // per_sheet:02d}.png"), grid)


def write(meta, rows, prefix: Path):
    prefix.parent.mkdir(parents=True, exist_ok=True)
    with open(str(prefix) + ".csv", "w", encoding="utf-8") as f:
        f.write("frame,t_s,d_m,x_px,y_px,flag,solidity,axis_ratio,contrast,ring_saturated,soot_annulus,"
                "d_threshold_m,rays_ok\n")
        for r in rows:
            def g(k, fmt):
                v = r.get(k)
                return "" if v is None else fmt.format(v)
            f.write(f"{r['frame']},{g('t_s', '{:.5f}')},{g('d_m', '{:.5e}')},{g('cx', '{:.2f}')},{g('cy', '{:.2f}')},"
                    f"{r['flag']},{g('sol', '{:.3f}')},{g('ar', '{:.3f}')},{g('contrast', '{:.3f}')},"
                    f"{g('ring_sat', '{:.3f}')},{int(bool(r.get('soot')))},{g('d_thr_m', '{:.5e}')},"
                    f"{g('rays_ok', '{:.3f}')}\n")
    flags = {}
    for r in rows:
        flags[r["flag"]] = flags.get(r["flag"], 0) + 1
    meta["flag_counts"] = flags
    Path(str(prefix) + ".json").write_text(json.dumps(meta, indent=2, default=str) + "\n", encoding="utf-8")


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("src", help="directory holding <tp>A.zip (+ <tp>A.txt, <tp>.txt, HiBMs settings) or the zip")
    ap.add_argument("--out", required=True, help="output prefix (writes .csv and .json)")
    ap.add_argument("--investigation", default="FLEX", choices=["FLEX", "FLEX-2"])
    ap.add_argument("--fov-mm", type=float, default=None, help="full-frame field of view (mm), overrides documentation")
    ap.add_argument("--scale-px-per-mm", type=float, default=None, help="pixel scale of the archived images")
    ap.add_argument("--scale-source", default=None, help="say where --scale-px-per-mm came from")
    ap.add_argument("--step", type=int, default=1, help="measure every n-th frame (disables the clock check)")
    ap.add_argument("--sheet", default=None, help="directory for contact sheets")
    ap.add_argument("--sheet-every", type=int, default=None)
    a = ap.parse_args()
    meta, rows = run(a.src, a.investigation, a.fov_mm, a.scale_px_per_mm, a.scale_source, a.step, a.sheet,
                     a.sheet_every)
    write(meta, rows, Path(a.out))
    print(json.dumps({k: meta.get(k) for k in ("test_point", "px_per_mm", "scale_source", "clock", "clock_check",
                                               "compute_s", "flag_counts")}, indent=1, default=str))
    return 0


if __name__ == "__main__":
    sys.exit(main())
