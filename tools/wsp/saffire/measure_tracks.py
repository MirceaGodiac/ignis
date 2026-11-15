#!/usr/bin/env python3
"""Flame tracks of the Saffire I/II/III concurrent burns from NASA's geometrically mapped frames.

Deterministic measurement for docs/wsp/SAFFIRE_TRANSFER.md sections 3-5. Run with `python -I`;
the downloads are data, passed as paths. Output: wsp-bass-front-track/1 files (git-ignored).

    python -I tools/wsp/saffire/measure_tracks.py --raw runs/dev/saffire/raw --out runs/dev/saffire --stage geom --diag DIR
    python -I tools/wsp/saffire/measure_tracks.py --raw runs/dev/saffire/raw --out runs/dev/saffire --stage track

Method (fixed before any track was computed; every constant is named below):
  frames    Mapped_<rec>-<unix ms>.jpeg; record time from the file name (both cameras share the
            clock). Summary images (`__Max`, `__Mean` ...) are never read. A repeated time stamp
            (Saffire I S1C2 part3 repeats part2) is read once. The backlight flashes (about 0.45 s
            of every 2 s); a frame whose mean green level exceeds LIT_LEVEL is lit and is never
            measured for a flame, nor are the LIT_GUARD_FRAMES frames on each side of a lit frame
            (partly lit transition frames).
  geometry  from the lit frames before the igniter (those with a 95th-percentile green below 250,
            i.e. unsaturated; all of them if none is): the backlit sample is the largest 4-connected
            area brighter than half of the frame's 95th percentile; straight-line fits of its top
            and bottom edges over the columns where it is at least 80 % of its full height give the
            sample width in pixels (ppw) at the mid column; the columns where its height falls to
            half give the backlit ends. If within WINDOW_SEARCH_W beyond a backlit end a thin bright
            line (the window outline, >= OUTLINE_REL x the median level between) is found, the
            sample end is that line and the band between is opaque (an obstruction). An end within
            END_VALID_PX of the unmapped area is out of view. Flow axis = image x; the upstream
            (igniter) end is on the right in every set used (checked against the igniter columns).
  positions camera 1: (x_up - x)/ppw from the upstream end. Camera 2 (downstream part of the
            sheet; part of the strip): L_ref - (x - x_down)/ppw with x_down its downstream sample
            end and L_ref the sample length in widths (sheets: 94/40.6 nominal, because camera 1
            does not see the downstream end; strips: camera 1's measured end-to-end length).
  dark      per-pixel median of up to DARK_N dark frames ending 1 s before the first igniter glow.
  signal    3x3 median, Gaussian BLUR_SIGMA_PX, max(R,G,B) minus the pixel's dark level, minus the
            frame's median of that difference over the mapped area (gain shifts of the whole frame).
  threshold per camera, max(THRESH_FLOOR, ceil(THRESH_K x the 99.99th percentile of the signal over
            the pre-ignition dark frames inside the mapped area)); fixed before tracking.
  region    luminous mask dilated by JOIN_W sample widths for connectivity only; the flame is the
            component holding the most luminous pixels, if they are at least MIN_AREA_W2 w^2.
  profile   per column, the fraction of the sample width (rows between the fitted edges) covered
            by region pixels; tip = most downstream position with fraction >= FRAC_MIN, base = most
            upstream.
  quality   edge when a region pixel is within EDGE_PX of the frame border, the unmapped area or a
            static obstruction (in-sample area darker than OBSTRUCT_REL x the sample's median
            backlit level, component >= OBSTRUCT_MIN_W2 w^2, not within IGNITER_ZONE_W of the
            upstream end); else ok.
  igniter   hot pixel = red >= IGN_RED. First glow = first dark camera-1 frame with >= IGN_MIN_PX
            hot pixels. Saffire I and II: t0 = first glow + tabulated igniter time (the coil columns
            re-brighten with flame and glowing char after the coil drops, so the last coil-glow
            frame cannot be told apart). Saffire III (no tabulated time): t0 = the last dark frame
            before the coil-column hot count first falls below IGN_DROP x the median count since
            first glow + 1 s. on_t_s = -(tabulated) or first glow - t0.
  stills    one per second: at t0 + k s (integer k from the igniter-on second), the nearest dark
            frame within STILL_TOL_S; none if there is none. Sheets: per still, the camera whose
            tip is further downstream (camera 1 unless camera 2's tip is more than CAM_PREF_W
            further downstream, or camera 1 has no region). Strips: camera 1 only.
  states    t < 0: igniter. t >= 0: flame if a region with a tip, else none.
  end       left_view at the first still (t >= 0, flame) whose tip >= downstream sample end -
            END_TOL_W, time = midpoint of it and the still before; otherwise extinction if flame
            stills are followed only by none stills, time = midpoint of the last flame still and the
            next; record_end if the last still has a flame, time = that still; lost if no flame
            still after t = 0.
"""
import argparse
import hashlib
import json
import math
import os
import re
import sys
from pathlib import Path

import cv2
import numpy as np

# ---- fixed constants -------------------------------------------------------------------------
LIT_LEVEL = 8.0           # mean green (8-bit) above which a frame is backlit (revised from 20 after the first full pass, see report)
LIT_GUARD_FRAMES = 2      # added after the first full pass: partly lit transition frames (see report)
SAT_P95 = 250.0           # lit frame counted as saturated above this 95th percentile
VALID_LEVEL = 4           # a pixel is mapped if its max over the lit frames exceeds this
DARK_N = 40
BLUR_SIGMA_PX = 4.0       # revised from 1.5 after the first strip pass (see report): halo lost in noise
THRESH_FLOOR = 6.0
THRESH_K = 1.5
JOIN_W = 0.02
MIN_AREA_W2 = 0.002
FRAC_MIN = 0.05
EDGE_PX = 3
END_VALID_PX = 6
OBSTRUCT_REL = 0.35
OBSTRUCT_MIN_W2 = 0.01
IGNITER_ZONE_W = 0.10
WINDOW_SEARCH_W = 0.30
OUTLINE_REL = 2.5
OUTLINE_WIDTH_PX = 8
IGN_RED = 200
IGN_MIN_PX = 10
IGN_DROP = 0.30
STILL_TOL_S = 0.5
CAM_PREF_W = 0.05
END_TOL_W = 0.02
CHECK_GAIN = 4.0
CHECK_MIN = 12

PAT = re.compile(r'^Mapped_[^_]*?-(\d{13})\.jpe?g$', re.I)

BURNS = [
    dict(key='PSI-98_S1', run_id='PSI-98:S1', group='PSI-98:flight', stratum='wide', psi='PSI-98',
         c1='*SAFFIRE_I_S1C1_JPEG_Mapped*', c2='*SAFFIRE_I_S1C2_JPEG_Mapped*', kind='sheet',
         width_m=0.406, length_m=0.94, u=20.0, x_o2=0.21, ign_tab=8.0,
         label='SIBAL cotton-fibreglass sheet 40.6 cm x 94 cm (Saffire I sample 1)'),
    dict(key='PSI-100_S1', run_id='PSI-100:S1', group='PSI-100:flight', stratum='wide', psi='PSI-100',
         c1='*SAFFIRE_III_S1C1_JPEG_Mapped*', c2='*SAFFIRE_III_S1C2_JPEG_Mapped*', kind='sheet',
         width_m=0.406, length_m=0.94, u=30.0, x_o2=0.21, ign_tab=None,
         label='SIBAL cotton-fibreglass sheet 40.6 cm x 94 cm (Saffire III sample 1)'),
    dict(key='PSI-99_2-5', run_id='PSI-99:2-5', group='PSI-99:flight', stratum='strip', psi='PSI-99',
         c1='*SAFFIRE-II_S5C1_Image_Data_All_JPEGs_Mapped*', c2='*SAFFIRE-II_S5C2_Image_Data_All*JPEGs_Mapped*',
         kind='strip', width_m=0.05, length_m=0.29, u=20.0, x_o2=0.221, ign_tab=9.2,
         label='SIBAL cotton-fibreglass strip 5 cm x 29 cm (Saffire II sample 2-5)'),
    dict(key='PSI-99_2-6', run_id='PSI-99:2-6', group='PSI-99:flight', stratum='strip', psi='PSI-99',
         c1='*SAFFIRE-II*S6C1_Image_Data_All*JPEGs_Mapped*', c2='*SAFFIRE-II_S6C2_Image_Data_All_JPEGs_Mapped*',
         kind='strip', width_m=0.05, length_m=0.29, u=30.0, x_o2=0.221, ign_tab=9.2,
         label='SIBAL cotton-fibreglass strip 5 cm x 29 cm (Saffire II sample 2-6)'),
]
OPPOSED = [
    dict(key='PSI-98_S2', run_id='PSI-98:S2', group='PSI-98:flight', stratum='wide', u=20.0, x_o2=0.21,
         width_m=0.406, length_m=0.94, label='SIBAL cotton-fibreglass sheet (Saffire I sample 2, opposed flow)'),
    dict(key='PSI-100_S2', run_id='PSI-100:S2', group='PSI-100:flight', stratum='wide', u=30.0, x_o2=0.21,
         width_m=0.406, length_m=0.94, label='SIBAL cotton-fibreglass sheet (Saffire III sample 2, opposed flow)'),
]


# ---- frames ------------------------------------------------------------------------------------
def list_frames(dirs):
    out = {}
    for d in sorted(dirs):
        for root, _, files in os.walk(d):
            for name in sorted(files):
                m = PAT.match(name)
                if m:
                    out.setdefault(int(m.group(1)), os.path.join(root, name))
    return [(t / 1000.0, out[t]) for t in sorted(out)]


def read(f):
    im = cv2.imread(f, cv2.IMREAD_COLOR)
    if im is None:
        raise RuntimeError(f"unreadable frame {f}")
    return im


class Camera:
    def __init__(self, name, dirs):
        self.name = name
        self.dirs = [os.path.basename(d) for d in sorted(dirs)]
        self.frames = list_frames(dirs)
        if not self.frames:
            raise RuntimeError(f"no frames for {name}")
        self.t = np.array([t for t, _ in self.frames])
        self._lit = {}

    def raw_lit(self, i):
        if i not in self._lit:
            im = cv2.imread(self.frames[i][1], cv2.IMREAD_REDUCED_COLOR_4)
            self._lit[i] = float(im[:, :, 1].mean()) > LIT_LEVEL
        return self._lit[i]

    def lit(self, i):
        # a frame next to a backlit frame can be partly lit (transition): guard LIT_GUARD_FRAMES each side
        lo, hi = max(0, i - LIT_GUARD_FRAMES), min(len(self.t) - 1, i + LIT_GUARD_FRAMES)
        return any(self.raw_lit(k) for k in range(lo, hi + 1))

    def nearest_dark(self, t, tol=STILL_TOL_S):
        j = int(np.searchsorted(self.t, t))
        cand = [k for k in range(max(0, j - 40), min(len(self.t), j + 40)) if abs(self.t[k] - t) <= tol]
        cand.sort(key=lambda k: (abs(self.t[k] - t), k))
        for k in cand:
            if not self.lit(k):
                return k
        return None

    def timing(self):
        d = np.diff(self.t)
        gaps = [(round(float(self.t[i] - self.t[0]), 2), round(float(d[i]), 2)) for i in np.where(d > 0.2)[0]]
        return dict(frames=len(self.t), span_s=round(float(self.t[-1] - self.t[0]), 2),
                    median_dt_s=round(float(np.median(d)), 4), gaps_over_0p2_s=gaps)


# ---- igniter -----------------------------------------------------------------------------------
def first_glow(cam, horizon_s=120.0):
    for i in range(len(cam.frames)):
        if cam.t[i] - cam.t[0] > horizon_s:
            break
        if cam.lit(i):
            continue
        im = read(cam.frames[i][1])
        if int((im[:, :, 2] >= IGN_RED).sum()) >= IGN_MIN_PX:
            return i
    raise RuntimeError(f"no igniter glow in {cam.name}")


def igniter_drop(cam, i0, horizon_s=30.0):
    """Coil columns from the first 3 s of glow; off = last dark frame before the hot count first
    falls below IGN_DROP x the median count since first glow + 1 s."""
    cols = set()
    series = []
    for i in range(i0, len(cam.frames)):
        dt = cam.t[i] - cam.t[i0]
        if dt > horizon_s:
            break
        if cam.lit(i):
            continue
        hot = read(cam.frames[i][1])[:, :, 2] >= IGN_RED
        if dt <= 3.0:
            cols.update(np.where(hot.any(axis=0))[0].tolist())
        series.append((i, hot))
    lo, hi = min(cols), max(cols)
    counts = [(i, int(h[:, lo:hi + 1].sum())) for i, h in series]
    prev = None
    for k, (i, c) in enumerate(counts):
        dt = cam.t[i] - cam.t[i0]
        if dt > 1.0:
            ref = [cc for ii, cc in counts[:k] if cam.t[ii] - cam.t[i0] >= 1.0]
            if ref and c < IGN_DROP * float(np.median(ref)):
                return prev, (lo, hi)
        prev = i
    raise RuntimeError("no igniter drop found")


# ---- geometry ----------------------------------------------------------------------------------
def geometry_from_lit(ims):
    res = []
    for im in ims:
        g = cv2.GaussianBlur(im[:, :, 1], (0, 0), 2.0).astype(np.float32)
        p95 = float(np.percentile(g[g > VALID_LEVEL], 95))
        bright = (g > 0.5 * p95).astype(np.uint8)
        n, lab, st, _ = cv2.connectedComponentsWithStats(bright, connectivity=4)
        k = 1 + int(np.argmax(st[1:, cv2.CC_STAT_AREA]))
        comp = lab == k
        cols = np.where(comp.any(axis=0))[0]
        top = np.array([np.argmax(comp[:, c]) for c in cols])
        bot = np.array([comp.shape[0] - 1 - np.argmax(comp[::-1, c]) for c in cols])
        h = bot - top + 1
        full = np.percentile(h, 90)
        good = h >= 0.8 * full
        a_t = np.polyfit(cols[good], top[good], 1)
        a_b = np.polyfit(cols[good], bot[good], 1)
        xc = float(cols[good].mean())
        ppw = float(np.polyval(a_b, xc) - np.polyval(a_t, xc))
        half = h >= 0.5 * full
        res.append(dict(ppw=ppw, top=a_t, bot=a_b, x_left=int(cols[half].min()), x_right=int(cols[half].max()),
                        xc=xc, p95=p95, col_width_sd=float(np.std(h[good]))))
    return res


def outline_beyond(level, top, bot, x_end, ppw, direction, valid):
    """Thin bright outline within WINDOW_SEARCH_W beyond a backlit end; direction -1 = leftwards."""
    W = level.shape[1]
    span = int(WINDOW_SEARCH_W * ppw)
    xs = [x_end + direction * d for d in range(3, span) if 0 <= x_end + direction * d < W]
    if not xs:
        return None
    prof = []
    for x in xs:
        y0, y1 = np.polyval(top, x), np.polyval(bot, x)
        ya, yb = int(y0 + 0.2 * (y1 - y0)), int(y0 + 0.8 * (y1 - y0))
        col = level[ya:yb, x]
        ok = valid[ya:yb, x]
        prof.append(float(np.median(col[ok])) if ok.any() else np.nan)
    prof = np.array(prof)
    for j, x in enumerate(xs):
        if np.isnan(prof[j]):
            break
        between = prof[:j]
        if j >= 3 and np.nanmedian(between) > 0 and prof[j] >= OUTLINE_REL * np.nanmedian(between):
            # a thin line: the level falls back below half the peak within OUTLINE_WIDTH_PX further out
            ahead = prof[j + 1:j + 1 + OUTLINE_WIDTH_PX]
            if len(ahead) and np.nanmin(ahead) < 0.5 * prof[j]:
                return int(x)
    return None


class View:
    """Everything fixed about one camera before tracking."""

    def __init__(self, cam, t_glow_abs, role):
        self.cam = cam
        self.role = role
        cut = t_glow_abs - 1.0
        pre = [i for i in range(len(cam.t)) if cam.t[i] < cut]
        lit_all = [i for i in pre if cam.raw_lit(i)]
        dark = [i for i in pre if not cam.lit(i)]
        if len(lit_all) < 1 or len(dark) < 5:
            raise RuntimeError(f"{cam.name}: too few pre-ignition frames ({len(lit_all)} lit, {len(dark)} dark)")
        lit_ims = [read(cam.frames[i][1]) for i in lit_all]
        self.valid = np.max([im.max(axis=2) for im in lit_ims], axis=0) > VALID_LEVEL
        p95s = [float(np.percentile(cv2.GaussianBlur(im[:, :, 1], (0, 0), 2.0)[self.valid], 95)) for im in lit_ims]
        uns = [im for im, p in zip(lit_ims, p95s) if p < SAT_P95]
        self.geo_frames_saturated = not uns
        use = uns or lit_ims
        geo = geometry_from_lit(use)
        self.n_geo_frames = len(use)
        self.ppw_frames = [g['ppw'] for g in geo]
        self.ppw = float(np.median(self.ppw_frames))
        self.top = np.median([g['top'] for g in geo], axis=0)
        self.bot = np.median([g['bot'] for g in geo], axis=0)
        self.col_width_sd = float(np.median([g['col_width_sd'] for g in geo]))
        self.tilt = float(np.degrees(np.arctan(0.5 * (self.top[0] + self.bot[0]))))
        backlit_left = int(np.median([g['x_left'] for g in geo]))
        backlit_right = int(np.median([g['x_right'] for g in geo]))
        level = np.median([cv2.GaussianBlur(im[:, :, 1], (0, 0), 2.0).astype(np.float32) for im in use], axis=0)
        self.level = level
        H, W = level.shape
        inv = cv2.dilate((~self.valid).astype(np.uint8), np.ones((3, 3), np.uint8), iterations=END_VALID_PX)

        def end_state(x, direction):
            yc = int(0.5 * (np.polyval(self.top, x) + np.polyval(self.bot, x)))
            xo = x + direction * 2
            if xo < 0 or xo >= W or inv[yc, xo]:
                return x, 'out_of_view', None
            o = outline_beyond(level, self.top, self.bot, x, self.ppw, direction, self.valid)
            if o is not None:
                return o, 'window_outline', (min(o, x), max(o, x))
            return x, 'backlit_end', None

        self.x_down, self.down_kind, band_d = end_state(backlit_left, -1)
        self.x_up, self.up_kind, band_u = end_state(backlit_right, +1)
        self.backlit = (backlit_left, backlit_right)
        # sample rows mask and obstruction mask
        yy, xx = np.mgrid[0:H, 0:W]
        ytop = np.polyval(self.top, xx)
        ybot = np.polyval(self.bot, xx)
        self.rows = (yy >= ytop) & (yy <= ybot)
        self.col_h = np.polyval(self.bot, np.arange(W)) - np.polyval(self.top, np.arange(W))
        inside = self.rows & (xx >= self.x_down) & (xx <= self.x_up) & self.valid
        med = float(np.median(level[inside & (xx >= backlit_left) & (xx <= backlit_right)]))
        dark_in = (level < OBSTRUCT_REL * med) & inside
        dark_in &= xx < (self.x_up - IGNITER_ZONE_W * self.ppw)
        n, lab, st, _ = cv2.connectedComponentsWithStats(dark_in.astype(np.uint8), connectivity=8)
        obst = np.zeros_like(dark_in)
        self.obstructions = []
        for k in range(1, n):
            if st[k, cv2.CC_STAT_AREA] >= OBSTRUCT_MIN_W2 * self.ppw ** 2:
                obst |= lab == k
                self.obstructions.append(dict(x=int(st[k, 0]), y=int(st[k, 1]), w=int(st[k, 2]), h=int(st[k, 3]),
                                              area_w2=round(st[k, cv2.CC_STAT_AREA] / self.ppw ** 2, 4)))
        self.obst = obst
        border = np.zeros((H, W), np.uint8)
        border[:EDGE_PX, :] = 1
        border[-EDGE_PX:, :] = 1
        border[:, :EDGE_PX] = 1
        border[:, -EDGE_PX:] = 1
        border |= cv2.dilate((~self.valid).astype(np.uint8), np.ones((3, 3), np.uint8), iterations=EDGE_PX)
        self.border = border.astype(bool)
        self.obst_zone = cv2.dilate(obst.astype(np.uint8), np.ones((3, 3), np.uint8), iterations=EDGE_PX).astype(bool)
        # dark level and threshold
        sel = dark[:: max(1, len(dark) // DARK_N)][:DARK_N]
        stack = np.stack([self._smooth(read(cam.frames[i][1])) for i in sel])
        self.dark = np.median(stack, axis=0)
        sig = np.stack([self._offset(x - self.dark) for x in stack])
        self.noise9999 = float(np.percentile(sig[:, self.valid], 99.99))
        self.thresh = max(THRESH_FLOOR, math.ceil(THRESH_K * self.noise9999))
        self.n_dark = len(sel)
        self.n_lit_pre = len(lit_all)
        r = max(1, int(round(JOIN_W * self.ppw)))
        self.join_kernel = cv2.getStructuringElement(cv2.MORPH_ELLIPSE, (2 * r + 1, 2 * r + 1))
        self.min_area = MIN_AREA_W2 * self.ppw ** 2
        self.xmid = 0.5 * (self.x_down + self.x_up)
        self.L_ref = None   # set by the burn

    @staticmethod
    def _smooth(im):
        im = cv2.medianBlur(im, 3)
        im = cv2.GaussianBlur(im, (0, 0), BLUR_SIGMA_PX)
        return im.max(axis=2).astype(np.float32)

    def _offset(self, sig):
        # per-frame background offset (camera gain shifts the whole dark frame): median over the mapped area
        return sig - float(np.median(sig[self.valid]))

    def pos(self, x):
        if self.role == 'c2':
            return self.L_ref - (x - self.x_down) / self.ppw
        return (self.x_up - x) / self.ppw

    def measure(self, idx):
        im = read(self.cam.frames[idx][1])
        sig = self._offset(self._smooth(im) - self.dark)
        lum = (sig > self.thresh) & self.valid
        out = dict(frame=os.path.basename(self.cam.frames[idx][1]), t_abs=float(self.cam.t[idx]))
        if not lum.any():
            return out
        grown = cv2.dilate(lum.astype(np.uint8), self.join_kernel)
        n, lab = cv2.connectedComponents(grown, connectivity=8)
        cnt = np.bincount(lab[lum], minlength=n)
        cnt[0] = 0
        k = int(np.argmax(cnt))
        if cnt[k] < self.min_area:
            return out
        reg = lum & (lab == k)
        colcount = (reg & self.rows).sum(axis=0)
        frac = colcount / np.maximum(self.col_h, 1)
        cols = np.where(frac >= FRAC_MIN)[0]
        out['area_px'] = int(cnt[k])
        out['sat_px'] = int((im.max(axis=2)[reg] >= 255).sum())
        bz = reg & self.border
        out['touch_border_down'] = bool((bz & (np.arange(reg.shape[1]) < self.xmid)[None, :]).any())
        out['touch_border_up'] = bool((bz & (np.arange(reg.shape[1]) >= self.xmid)[None, :]).any())
        out['touch_obstruction'] = bool((reg & self.obst_zone).any())
        if len(cols) == 0:
            return out
        xt, xb = int(cols.min()), int(cols.max())
        out['tip_px'], out['base_px'] = xt, xb
        out['tip_w'] = round(float(self.pos(xt)), 4)
        out['base_w'] = round(float(self.pos(xb)), 4)
        # tip clear of the border for the camera cross-check: tip column band free of border contact
        band = reg[:, max(0, xt - 3):xt + 4] & self.border[:, max(0, xt - 3):xt + 4]
        out['tip_clear'] = not bool(band.any())
        out['edge'] = bool(bz.any() or out['touch_obstruction'])
        return out

    def describe(self):
        return dict(camera=self.cam.name, zips_dirs=self.cam.dirs, timing=self.cam.timing(),
                    ppw_px=round(self.ppw, 2), ppw_frames_px=[round(p, 2) for p in self.ppw_frames],
                    ppw_sd_px=round(float(np.std(self.ppw_frames)), 2),
                    ppw_range_px=round(float(np.ptp(self.ppw_frames)), 2),
                    column_width_sd_px=round(self.col_width_sd, 2), tilt_deg=round(self.tilt, 3),
                    geometry_frames=self.n_geo_frames, geometry_frames_saturated=self.geo_frames_saturated,
                    x_down_px=self.x_down, down_end=self.down_kind, x_up_px=self.x_up, up_end=self.up_kind,
                    backlit_px=list(self.backlit), obstructions=self.obstructions,
                    dark_frames=self.n_dark, noise_p9999=round(self.noise9999, 2), threshold=self.thresh,
                    join_radius_px=int(round(JOIN_W * self.ppw)), min_area_px=round(self.min_area, 1))


# ---- one burn ----------------------------------------------------------------------------------
def setup_burn(b, raw):
    root = Path(raw) / b['psi'] / 'x'
    d1 = sorted(str(p) for p in root.glob(b['c1']))
    d2 = sorted(str(p) for p in root.glob(b['c2']))
    c1 = Camera('camera 1', d1)
    c2 = Camera('camera 2', d2) if d2 else None
    i_glow = first_glow(c1)
    t_glow = float(c1.t[i_glow])
    v1 = View(c1, t_glow, 'c1')
    v2 = View(c2, t_glow, 'c2') if c2 else None
    if b['ign_tab'] is not None:
        t0 = t_glow + b['ign_tab']
        on = -b['ign_tab']
        rule = (f"first frame of igniter glow in camera 1 + tabulated igniter time {b['ign_tab']} s "
                "(coil glow and flame/char glow overlap in the coil columns after the coil drops, so the last "
                "coil-glow frame cannot be told apart)")
        i_drop, coil = igniter_drop(c1, i_glow)
        drop_info = round(float(c1.t[i_drop] - t_glow), 2)
    else:
        i_drop, coil = igniter_drop(c1, i_glow)
        t0 = float(c1.t[i_drop])
        on = t_glow - t0
        rule = (f"last camera-1 dark frame before the igniter-coil hot-pixel count (red >= {IGN_RED}) first falls "
                f"below {IGN_DROP} x its median since first glow + 1 s; no tabulated igniter time")
        drop_info = round(float(t0 - t_glow), 2)
    coil_mid = 0.5 * (coil[0] + coil[1])
    if coil_mid < v1.xmid:
        raise RuntimeError(f"{b['key']}: igniter not on the right of camera 1")
    if b['kind'] == 'strip':
        L_ref = (v1.x_up - v1.x_down) / v1.ppw
        if v1.down_kind == 'out_of_view' or v1.up_kind == 'out_of_view':
            raise RuntimeError(f"{b['key']}: strip end out of camera-1 view")
    else:
        L_ref = b['length_m'] / b['width_m']
    v1.L_ref = L_ref
    if v2:
        v2.L_ref = L_ref
        if v2.down_kind == 'out_of_view':
            raise RuntimeError(f"{b['key']}: camera 2 does not see the downstream end")
    return dict(c1=c1, c2=c2, v1=v1, v2=v2, t_glow=t_glow, t0=t0, on=on, rule=rule, drop_s=drop_info,
                coil=coil, L_ref=L_ref)


def track_burn(b, s):
    v1, v2, t0, on = s['v1'], s['v2'], s['t0'], s['on']
    c1 = s['c1']
    t_last = float(c1.t[-1]) if b['kind'] == 'strip' else max(float(c1.t[-1]), float(s['c2'].t[-1]) if s['c2'] else 0)
    k0 = math.ceil(on)
    k1 = int(math.floor(t_last - t0))
    stills, pairs = [], []
    for k in range(k0, k1 + 1):
        ta = t0 + k
        i1 = c1.nearest_dark(ta)
        m1 = v1.measure(i1) if i1 is not None else None
        m2 = None
        if v2 is not None:
            i2 = s['c2'].nearest_dark(ta)
            m2 = v2.measure(i2) if i2 is not None else None
        if m1 is None and (m2 is None or b['kind'] == 'strip'):
            continue
        if m1 is None and 'tip_w' not in m2:
            # camera 1 has no frame here and camera 2 sees only the downstream part: no observation
            continue
        if b['kind'] == 'sheet':
            choose = m1
            if m1 is None or 'tip_w' not in m1:
                if m2 is not None and 'tip_w' in m2:
                    choose = m2
            elif m2 is not None and 'tip_w' in m2 and m2['tip_w'] > m1['tip_w'] + CAM_PREF_W:
                choose = m2
            if choose is None:
                choose = m2
            cam = 1 if choose is m1 else 2
        else:
            choose, cam = m1, 1
        t = round(choose['t_abs'] - t0, 3)
        st = dict(t_s=t, tip_w=None, base_w=None)
        if t < 0:
            st.update(state='igniter', quality=None)
        elif 'tip_w' in choose:
            st.update(tip_w=choose['tip_w'], base_w=choose['base_w'], state='flame',
                      quality='edge' if choose['edge'] else 'ok')
        else:
            st.update(state='none', quality=None)
        st['camera'] = cam
        st['frame'] = choose['frame']
        for f in ('area_px', 'sat_px', 'touch_border_down', 'touch_border_up', 'touch_obstruction', 'tip_px', 'base_px'):
            if f in choose:
                st[f] = choose[f]
        stills.append(st)
        if t >= 0 and m1 and m2 and 'tip_w' in m1 and 'tip_w' in m2 and m1.get('tip_clear') and m2.get('tip_clear'):
            pairs.append(abs(m1['tip_w'] - m2['tip_w']))
    return stills, pairs


def end_of(stills, L_end):
    post = [s for s in stills if s['t_s'] >= 0]
    prev = None
    for s in post:
        if s['state'] == 'flame' and s['tip_w'] is not None and s['tip_w'] >= L_end - END_TOL_W:
            t = s['t_s'] if prev is None else 0.5 * (prev['t_s'] + s['t_s'])
            return dict(kind='left_view', t_s=round(t, 3))
        prev = s
    flames = [k for k, s in enumerate(post) if s['state'] == 'flame']
    if not flames:
        return dict(kind='lost', t_s=None)
    j = flames[-1]
    if j == len(post) - 1:
        return dict(kind='record_end', t_s=post[j]['t_s'])
    return dict(kind='extinction', t_s=round(0.5 * (post[j]['t_s'] + post[j + 1]['t_s']), 3))


def flame_stills(stills):
    return [s for s in stills if s['state'] == 'flame' and s['quality'] != 'edge' and s['t_s'] >= 0 and s['tip_w'] is not None]


def check_pngs(b, s, stills, outdir):
    fl = flame_stills(stills)
    pool = fl if len(fl) >= CHECK_MIN else [x for x in stills if x['state'] == 'flame' and x['t_s'] >= 0]
    if not pool:
        return []
    n_target = max(CHECK_MIN, 14)
    stride = max(1, len(pool) // n_target)
    pick = pool[::stride]
    d = Path(outdir) / b['run_id'].replace(':', '_')
    d.mkdir(parents=True, exist_ok=True)
    files = []
    for x in pick:
        v = s['v1'] if x['camera'] == 1 else s['v2']
        f = [p for t, p in v.cam.frames if os.path.basename(p) == x['frame']][0]
        im = read(f).astype(np.float32) * CHECK_GAIN
        im = np.clip(im, 0, 255).astype(np.uint8)
        H, W = im.shape[:2]
        for xp, col in ((x['tip_px'], (0, 0, 255)), (x['base_px'], (255, 255, 0))):
            y0 = int(np.polyval(v.top, xp)); y1 = int(np.polyval(v.bot, xp))
            cv2.line(im, (xp, y0), (xp, y1), col, max(1, W // 600))
        for poly in (v.top, v.bot):
            cv2.line(im, (0, int(np.polyval(poly, 0))), (W - 1, int(np.polyval(poly, W - 1))), (128, 128, 128), 1)
        scale = 1.0 if W <= 1000 else 1000.0 / W
        if scale != 1.0:
            im = cv2.resize(im, (int(W * scale), int(H * scale)), interpolation=cv2.INTER_AREA)
        fs = 0.4
        cv2.putText(im, f"t={x['t_s']:.1f}s cam{x['camera']} q={x['quality']}", (4, 14), cv2.FONT_HERSHEY_SIMPLEX, fs,
                    (255, 255, 255), 1)
        name = f"t{x['t_s']:+08.2f}_cam{x['camera']}.png".replace('+', 'p').replace('-', 'm')
        cv2.imwrite(str(d / name), im)
        files.append(dict(file=f"{d.name}/{name}", t_s=x['t_s'], camera=x['camera'], quality=x['quality']))
    return files


def write_json(path, obj):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(obj, indent=1) + '\n', encoding='utf-8')


def common_header(b):
    return {
        'schema': 'wsp-bass-front-track/1',
        'run_id': b['run_id'],
        'partition': 'validation',
        'evaluation_group': b['group'],
        'group_status': 'held',
    }


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--raw', required=True)
    ap.add_argument('--out', required=True)
    ap.add_argument('--stage', choices=['geom', 'track'], required=True)
    ap.add_argument('--diag', default=None)
    ap.add_argument('--only', default=None)
    ap.add_argument('--summary', default=None, help='working summary JSON (outside the outputs)')
    args = ap.parse_args()
    out = Path(args.out)
    if args.stage == 'geom':
        for b in BURNS:
            if args.only and b['key'] != args.only:
                continue
            s = setup_burn(b, args.raw)
            info = dict(t_glow_rel_record=round(s['t_glow'] - float(s['c1'].t[0]), 2), drop_s=s['drop_s'],
                        coil=s['coil'], L_ref=round(s['L_ref'], 4),
                        c1=s['v1'].describe(), c2=s['v2'].describe() if s['v2'] else None)
            print(b['key'], json.dumps(info, default=str))
            if args.diag:
                for v, nm in ((s['v1'], 'c1'), (s['v2'], 'c2')):
                    if v is None:
                        continue
                    lv = cv2.cvtColor(np.clip(v.level, 0, 255).astype(np.uint8), cv2.COLOR_GRAY2BGR)
                    lv[v.obst] = (0, 0, 255)
                    lv[v.border] = (255, 0, 0)
                    H, W = lv.shape[:2]
                    for poly in (v.top, v.bot):
                        cv2.line(lv, (0, int(np.polyval(poly, 0))), (W - 1, int(np.polyval(poly, W - 1))), (0, 255, 255), 2)
                    for xx, c in ((v.x_down, (255, 0, 255)), (v.x_up, (0, 255, 0))):
                        cv2.line(lv, (xx, 0), (xx, H - 1), c, 2)
                    cv2.imwrite(os.path.join(args.diag, f"geom_{b['key']}_{nm}.png"), lv)
        return 0
    # ---- track
    held = out / 'tracks_held'
    check = out / 'check'
    summary, index = {}, {}
    for b in BURNS:
        if args.only and b['key'] != args.only:
            continue
        s = setup_burn(b, args.raw)
        stills, pairs = track_burn(b, s)
        L_end = s['L_ref']
        end = end_of(stills, L_end)
        v1, v2 = s['v1'], s['v2']
        window_nominal = round(b['length_m'] / b['width_m'], 4)
        geometry = dict(sample_width_m=b['width_m'], window_length_w=window_nominal,
                        source_view='mapped camera-1 frames' + (' (camera 2 for the downstream part)' if b['kind'] == 'sheet' else ''),
                        ppw_px_camera1=round(v1.ppw, 2), ppw_sd_px_camera1=round(float(np.std(v1.ppw_frames)), 2))
        if b['kind'] == 'strip':
            geometry['measured_window_w'] = round(L_ref_val(s), 4)
        if v2:
            geometry['ppw_px_camera2'] = round(v2.ppw, 2)
            geometry['ppw_sd_px_camera2'] = round(float(np.std(v2.ppw_frames)), 2)
        n_fl = len(flame_stills(stills))
        n_edge = sum(1 for x in stills if x.get('quality') == 'edge')
        notes = [
            "positions: sample widths from the upstream (igniter-side) sample end along the flow; camera 1 "
            f"x_w = (x_up - x)/ppw; camera 2 x_w = {round(L_end, 4)} - (x - x_down)/ppw (downstream sample end at "
            f"{round(L_end, 4)} w: {'nominal 94/40.6' if b['kind'] == 'sheet' else 'camera-1 measured end-to-end length'}).",
            "luminous region includes any self-luminous source (flame and glowing char/embers); no colour separation.",
            f"stills: one per second on the t0 + k s grid, nearest unlit frame within {STILL_TOL_S} s; 'camera' says which.",
            f"end rule: left_view when tip >= downstream end - {END_TOL_W} w; else extinction/record_end/lost; "
            "time = midpoint of the deciding still and its neighbour (record_end: last still).",
        ]
        if v2 and v2.down_kind == 'window_outline':
            notes.append("camera 2: an opaque band lies between the backlit fabric and the window outline at the downstream "
                         "end; the sample end is taken at the outline and the band is a static obstruction (stills touching it are edge).")
        track = common_header(b)
        track.update({
            'time_origin': f"igniter off; {s['rule']}; t_s = record seconds after it (file-name clock of the mapped frames)",
            'position_unit': 'sample_width',
            'position_origin': 'upstream (igniter-side) end of the sample in the mapped camera-1 frame; positive downstream, along the flow',
            'conditions': dict(x_o2_initial=b['x_o2'], sample_label=b['label'], flow='concurrent', u_nominal_cm_s=b['u']),
            'geometry': geometry,
            'igniter': dict(on_t_s=round(s['on'], 3), off_t_s=0.0, rule=s['rule']),
            'fan': [dict(t_s=round(s['on'], 3), fan=0, u_cm_s=b['u'])],
            'fan_calibration': [],
            'stills': stills,
            'end': end,
            'notes': notes,
            'measurement': dict(script='tools/wsp/saffire/measure_tracks.py', camera1=v1.describe(),
                                camera2=v2.describe() if v2 else None, glow_to_coil_drop_s=s['drop_s'],
                                constants={k: v for k, v in globals().items() if k.isupper() and isinstance(v, (int, float))}),
        })
        path = held / b['stratum'] / f"T_{b['key']}.json"
        write_json(path, track)
        files = check_pngs(b, s, stills, check)
        index[b['run_id']] = files
        summary[b['run_id']] = dict(
            stills=len(stills), flame_stills=n_fl, edge_stills=n_edge,
            camera2_stills=sum(1 for x in stills if x.get('camera') == 2),
            cross_pairs=len(pairs), cross_median_abs_tip_diff_w=round(float(np.median(pairs)), 4) if pairs else None,
            sat_stills=sum(1 for x in stills if x.get('sat_px', 0) > 0),
            at_least_6=n_fl >= 6, check_pngs=len(files),
            c1=v1.describe(), c2=v2.describe() if v2 else None, rule=s['rule'], on_t_s=round(s['on'], 3),
            drop_s=s['drop_s'], L_ref=round(s['L_ref'], 4))
        print(b['key'], 'stills', len(stills), 'flame', n_fl, 'edge', n_edge, flush=True)
    for o in OPPOSED:
        if args.only and o['key'] != args.only:
            continue
        track = common_header(o)
        track.update({
            'time_origin': 'igniter off (not measured: opposed-flow burn, frames not opened)',
            'position_unit': 'sample_width',
            'position_origin': 'upstream end of the sample (not measured)',
            'conditions': dict(x_o2_initial=o['x_o2'], sample_label=o['label'], flow='opposed', u_nominal_cm_s=o['u']),
            'geometry': dict(sample_width_m=o['width_m'], window_length_w=round(o['length_m'] / o['width_m'], 4),
                             source_view='none (condition-only track)'),
            'fan': [dict(t_s=0.0, fan=0, u_cm_s=o['u'])],
            'fan_calibration': [],
            'stills': [],
            'notes': ['condition-only track: opposed flow; the model is for concurrent spread and the reader refuses it '
                      'at the flow condition; frames were not downloaded or measured.'],
        })
        write_json(held / o['stratum'] / f"T_{o['key']}.json", track)
    if summary:
        if args.summary:
            write_json(Path(args.summary), summary)
        idx_path = check / 'INDEX.json'
        old = json.loads(idx_path.read_text(encoding='utf-8')) if idx_path.exists() else {}
        old.update(index)
        write_json(idx_path, dict(sorted(old.items())))
    return 0


def L_ref_val(s):
    return s['L_ref']


if __name__ == '__main__':
    sys.exit(main())
