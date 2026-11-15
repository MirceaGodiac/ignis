#!/usr/bin/env python3
"""Flame radius against time, and flame end, from the image sources of an ACME burner test (CFI-G,
s-Flame, Flame Design). A measurement tool: it fits no model parameter and writes no release.

Sources (one test each; files as named in the PSI listing without the "PSI-<n>_Raw Data_" prefix):

  colour  the colour camera as downlinked: `Images_<test>.zip`, `<test>_Images.zip` or `<test>_ACME.zip`.
          HALCON .hobj frames, 1360 x 1024 BayerRG12 stored big-endian after a run-length region header;
          the per-frame index `ACME_<test>.mdipsug` (inside the zip and beside it) gives exposure and gain.
  intns   the intensified camera: `<test>_INTNS_ALL.zip`, 16-bit TIFF frames, index `INTNS_<test>.mdipsug`.
  video   the recording of the downlinked operations video: `vlc-record-...<test>.ts` (or .mp4).

Time. Every time is in seconds on the test clock: seconds since the flame onset that
`tools/wsp/ext/flame_design.py` measures from the radiometers in `<test>_data.csv` (the sensor clock,
seconds of day). Colour and intensified frames carry a time stamp in their file names (camera unit
clock, seconds of day); it is put on the sensor clock unchanged (checked in the pilot: the ignition
flash frame falls within one frame of the radiometer onset). The video carries no time stamp and no
overlay: its frame brightness (log, differentiated) is cross-correlated with the summed
photomultiplier signal of the data file, and the lag of the correlation peak puts frame 0 on the
sensor clock (`align_*` fields).

Scale. Colour camera: the burner (a 6.35 mm porous sphere on a vertical supply tube) is located in the
illuminated 1 s pre-ignition frames (silhouette circle fit), and its diameter in pixels gives mm per
pixel; the burner centre is the origin of the radial rays. When those frames are saturated (camera
gain 24-33) the CFI-G campaign constant `CFIG_COLOUR_SCALE` is used (or `--scale-from` a same-day
test). The dark disc that a sooty flame shows around the burner is NOT the burner (it is the soot-free
core, up to twice the burner) and is never used. Video and intensified camera show no usable burner:
they report radius in pixels; `compare_sources.py` transfers the colour scale to them per test.

Radius. Each frame is background-subtracted (per exposure setting), median-filtered against hot pixels,
smoothed, and resampled on 90 rays from the centre. Colour and video use G + B - 0.5 R, which keeps
the blue-green flame sheet and cancels most soot glow. On every ray each hump above 6 profile-noise
sigma and 10 % of the ray maximum gives a candidate edge (its outer half maximum); the outermost
hump is taken unless another hump of the ray lies nearer the median edge of all rays (reflections,
igniter). Rays along the supply tube and rays leaving the frame are dropped. A circle is fitted to
the edge points with outlier trimming (algebraic least squares on the rim: a measurement, not a
model); when the points cover less than half the circumference (flame larger than the field, or a
partial flame) the radius is the median distance from the burner centre instead (`partial_arc`).
Asymmetry: (p90 - p10) / (2 x radius) of the point distances; centre offset / radius.

Flame end. Rim criterion: a frame shows the flame when a rim of >= 12 points and median peak >= 10
sigma is found and it is not much smaller than the rims just before (a glowing burner after a
burner-limit stop is not the flame); the end lies between the last such frame and the first of three
consecutive frames without one. Light criterion: background-subtracted light per second of exposure
(averaged over 0.3 s for video) against the dark level of unlit pre-onset frames + 8 spreads. The
reported end (`end_s`) is the later of the two (neither fails late). With bracketed exposures the
end is read on the longest exposure. Error bar: half the frame gap (plus alignment, for video).

    python -I tools/wsp/media/flame_radius.py run --test 21088F1 --source colour \
        --raw runs/dev/media/acme/raw --out runs/dev/media/acme
    python -I tools/wsp/media/flame_radius.py run --test 21088F1 --source video --raw ... --out ...
    python -I tools/wsp/media/flame_radius.py reend --test 21088F1 --source video --out ...
    python -I tools/wsp/media/flame_radius.py scale-probe --test 21162B1 --out ...   # HTTP ranges, ~40 MB

Outputs under --out: `series/<test>_<source>.csv` (one row per frame), `series/<test>_<source>.json`
(alignment, scale, end, cost), `check/<test>_<source>.jpg` (contact sheet, detected rim drawn),
`check/<test>_colour_burner.png` (burner fit). Frames are decoded on access (LazyFrame), so a test
of thousands of frames runs in about 1 GB of memory. The raw folder needs the test's text files
(`_data.csv`, `_mfcSetpoints.csv`, `_TP.txt`, `as run<test>.txt`) for the sensor reference.
"""
import argparse
import csv
import io
import json
import math
import re
import sys
import time
import zipfile
from pathlib import Path

import cv2
import numpy as np

REPO = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(REPO / "tools/wsp/ext"))
import flame_design as fd  # noqa: E402  (sensor onset/end: the reference of this tool)

BURNER_D_MM = 6.35
# CFI-G (PSI-159) colour camera, measured on the lit pre-ignition frames of six train tests on five days
# and all three zoom settings of the camera configuration scripts (25000, 35000, 0): burner centre at
# (753, 532) full-resolution px in every one, 0.0798-0.0858 mm/px (median 0.0817). The fit misses the
# faint shadowed limb on some, so the spread is mostly fit error; the field did not change with the
# configured zoom. Used when a test has no clean lit frame (gain 24-33 saturates them).
CFIG_COLOUR_SCALE = dict(mm_per_px_full=0.0817, burner_cx_px=753.0, burner_cy_px=532.0,
                         rel_err=0.04, note="CFI-G campaign constant (6 lit-frame probes, 0.0798-0.0858 mm/px)")
N_RAYS = 90
RAY_SNR_MIN = 6.0
END_SNR = 8.0
TUBE_HALF_ANGLE_DEG = 14.0
NAN = float("nan")


# ------------------------------------------------------------------------------------------------
# sensor reference
# ------------------------------------------------------------------------------------------------
def sensor_reference(test, raw):
    """flame_design.measure on the raw folder; returns its row and the raw data channels."""
    fd.FILES = Path(raw)
    fn = dict(data=f"{test}_data.csv", setpoints=f"{test}_mfcSetpoints.csv", tp=f"{test}_TP.txt",
              script=f"as run{test}.txt")
    fn = {k: (v if (Path(raw) / v).exists() else None) for k, v in fn.items()}
    row = fd.measure(test, fn)
    hdr, t, d = fd.read_data(Path(raw) / fn["data"])
    return row, t, d


def stamp_seconds(name):
    """`20210329_174810.566_...` -> seconds of day."""
    m = re.search(r"(\d{8})_(\d\d)(\d\d)(\d\d)\.(\d{3})", name)
    if not m:
        return NAN
    return int(m.group(2)) * 3600 + int(m.group(3)) * 60 + int(m.group(4)) + int(m.group(5)) / 1000.0


def read_index(text):
    rows = list(csv.DictReader(io.StringIO(text)))
    out = {}
    for r in rows:
        key = r["Filename"].rsplit("/", 1)[-1]
        out[key] = r
    return out


# ------------------------------------------------------------------------------------------------
# remote archives: the PSI files are served from S3, which honours HTTP range requests, so a zip can
# be opened in place and only the frames needed are transferred
# ------------------------------------------------------------------------------------------------
PSI_DOWNLOAD = "https://psi.nasa.gov/geode-py/ws/studies/{inv}/download?{q}"
LISTING = REPO / "runs/dev/data_astra/metadata/PSI-159.listing.jsonl"


class HttpRangeFile(io.RawIOBase):
    """Read-only, seekable view of a remote file through HTTP range requests (1 MiB blocks cached)."""

    def __init__(self, url, size, block=1 << 20):
        import requests
        self.s = requests.Session()
        self.s.headers.update({"User-Agent": "research-data-client/1.0", "Accept-Encoding": "identity"})
        r = self.s.get(url, headers={"Range": "bytes=0-0"}, timeout=60, allow_redirects=True)
        if r.status_code != 206:
            raise IOError(f"range requests refused ({r.status_code})")
        self.url, self.size, self.pos, self.block, self.cache = r.url, size, 0, block, {}
        self.bytes_fetched = 0

    def readable(self):
        return True

    def seekable(self):
        return True

    def tell(self):
        return self.pos

    def seek(self, off, whence=0):
        self.pos = off if whence == 0 else self.pos + off if whence == 1 else self.size + off
        return self.pos

    def _blk(self, k):
        if k not in self.cache:
            a, b = k * self.block, min(self.size, (k + 1) * self.block) - 1
            for attempt in range(4):
                try:
                    r = self.s.get(self.url, headers={"Range": f"bytes={a}-{b}"}, timeout=120)
                    r.raise_for_status()
                    self.cache[k] = r.content
                    break
                except Exception:
                    if attempt == 3:
                        raise
                    time.sleep(3)
            self.bytes_fetched += len(self.cache[k])
            if len(self.cache) > 64:
                self.cache.pop(next(iter(self.cache)))
        return self.cache[k]

    def read(self, n=-1):
        if n is None or n < 0:
            n = self.size - self.pos
        n = max(0, min(n, self.size - self.pos))
        out = bytearray()
        while n > 0:
            k, o = divmod(self.pos, self.block)
            chunk = self._blk(k)[o:o + n]
            out += chunk
            self.pos += len(chunk)
            n -= len(chunk)
        return bytes(out)

    def readinto(self, b):
        d = self.read(len(b))
        b[:len(d)] = d
        return len(d)


def remote_file(test, source):
    """(HttpRangeFile, listing row) of the colour or intensified archive of a test."""
    pats = {"colour": [f"Images_{test}.zip", f"Images_ {test}.zip", f"{test}_Images.zip", f"{test}_ACME.zip"],
            "intns": [f"{test}_INTNS_ALL.zip"]}[source]
    rows = {}
    for line in LISTING.open(encoding="utf-8"):
        r = json.loads(line)
        rows[r["file_name"].replace("PSI-159_Raw Data_", "")] = r
    for p in pats:
        if p in rows:
            r = rows[p]
            from urllib.parse import quote, urlencode
            url = PSI_DOWNLOAD.format(inv=r["investigation"], q=urlencode({"file": r["file_name"], "version": r["file_version"]}, quote_via=quote))
            return HttpRangeFile(url, r["size_bytes"]), r
    raise SystemExit(f"{test}: no {source} archive in the listing")


# ------------------------------------------------------------------------------------------------
# frame sources: each yields dicts {name, t_clock, exposure_s, img (float32, linear), sat (bool mask or None)}
# ------------------------------------------------------------------------------------------------
HOBJ_MARK = b"\x00\x00\x05P\x00\x00\x04\x00"  # width 1360, height 1024 at the end of the region header


def decode_hobj(data, w=1360, h=1024):
    off = data.rfind(HOBJ_MARK)
    if off < 0:
        raise ValueError("hobj: image header not found")
    off += len(HOBJ_MARK)
    a = np.frombuffer(data[off:off + w * h * 2], dtype=">u2").reshape(h, w).astype(np.uint16)
    return a


def rim_channel(r, g, b):
    """The flame sheet of a blue (CH*, C2) burner flame against soot: G + B - 0.5 R. Glowing soot
    (R > G > B) is largely cancelled; the blue-green sheet is kept."""
    return g.astype(np.float32) + b.astype(np.float32) - 0.5 * r.astype(np.float32)


def bayer_bin(a):
    """2 x 2 Bayer cell -> one pixel, sum of R + 2G + B (half resolution, no demosaic fringes)."""
    a = a.astype(np.float32)
    return a[0::2, 0::2] + a[0::2, 1::2] + a[1::2, 0::2] + a[1::2, 1::2]


class LazyFrame(dict):
    """A frame whose arrays ('img', 'rim', 'sat') are decoded on access from a small source (a zip
    member, a PNG of a video frame) and not kept: a test of thousands of frames fits in memory. The
    last decoded frame is cached, so the three arrays of one frame cost one decode."""
    _last = (None, None)

    def __missing__(self, key):
        if key not in ("img", "rim", "sat"):
            raise KeyError(key)
        if LazyFrame._last[0] is not self:
            LazyFrame._last = (self, self["_decode"]())
        return LazyFrame._last[1][key]


def colour_frames(zpath, select=None):
    """select: optional function(name, index_row) -> bool; frames it rejects are not read."""
    z = zipfile.ZipFile(zpath)
    names = sorted(n for n in z.namelist() if n.lower().endswith(".hobj"))
    idx_name = [n for n in z.namelist() if n.lower().endswith(".mdipsug")]
    index = read_index(z.read(idx_name[0]).decode("utf-8", "replace")) if idx_name else {}

    def decoder(member):
        def dec():
            a = decode_hobj(z.read(member))
            sat = (a[0::2, 0::2] >= 4095) | (a[0::2, 1::2] >= 4095) | (a[1::2, 0::2] >= 4095) | (a[1::2, 1::2] >= 4095)
            return dict(img=bayer_bin(a), sat=sat,
                        rim=rim_channel(a[0::2, 0::2], 0.5 * (a[0::2, 1::2].astype(np.float32) + a[1::2, 0::2]), a[1::2, 1::2]))
        return dec
    for n in names:
        base = n.rsplit("/", 1)[-1]
        r = index.get(base, {})
        if select is not None and not select(base, r):
            continue
        exp = float(r.get("ExposureManual", "nan")) * 1e-6
        f = LazyFrame(name=base, t_clock=stamp_seconds(base), exposure_s=exp, gain=r.get("GainManual", ""),
                      full_scale=4 * 4095.0, px_bin=2, _decode=decoder(n))
        f["mean"] = float(np.mean(f["img"]))
        yield f


def intns_frames(zpath, every=1):
    z = zipfile.ZipFile(zpath)
    names = sorted(n for n in z.namelist() if n.lower().endswith((".tif", ".tiff")))
    idx_name = [n for n in z.namelist() if n.lower().endswith(".mdipsug")]
    index = read_index(z.read(idx_name[0]).decode("utf-8", "replace")) if idx_name else {}

    def decoder(member):
        def dec():
            a = cv2.imdecode(np.frombuffer(z.read(member), np.uint8), cv2.IMREAD_UNCHANGED)
            if a.ndim == 3:
                a = a[..., 0]
            top = float(np.iinfo(a.dtype).max) if a.dtype.kind == "u" else float(a.max())
            img = a.astype(np.float32)
            img = img[0::2, 0::2] + img[0::2, 1::2] + img[1::2, 0::2] + img[1::2, 1::2]
            # an intensifier near full scale (65492 of 65535 seen) counts as saturated
            return dict(img=img, rim=img, sat=a[0::2, 0::2] >= 0.995 * top)
        return dec
    for i, n in enumerate(names):
        if i % every:
            continue
        base = n.rsplit("/", 1)[-1]
        r = index.get(base, {})
        exp = float(r.get("ExposureManual", "nan")) * 1e-6
        f = LazyFrame(name=base, t_clock=stamp_seconds(base), exposure_s=exp, gain=r.get("GainManual", ""),
                      full_scale=4 * 65535.0, px_bin=2, _decode=decoder(n))
        f["mean"] = float(np.mean(f["img"]))
        yield f


def video_frames(path):
    cap = cv2.VideoCapture(str(path))
    i = 0

    def decoder(png):
        def dec():
            f = cv2.imdecode(png, cv2.IMREAD_COLOR)
            return dict(img=f.astype(np.float32).sum(axis=2), rim=rim_channel(f[..., 2], f[..., 1], f[..., 0]),
                        sat=(f.max(axis=2) >= 255))
        return dec
    while True:
        ok, f = cap.read()
        if not ok:
            break
        ok2, png = cv2.imencode(".png", f, [cv2.IMWRITE_PNG_COMPRESSION, 1])
        fr = LazyFrame(name=f"frame{i:06d}", index=i, pts_s=cap.get(cv2.CAP_PROP_POS_MSEC) / 1000.0, t_clock=NAN,
                       exposure_s=NAN, gain="", full_scale=3 * 255.0, px_bin=1, _decode=decoder(png),
                       mean=float(f.mean()) * 3)
        yield fr
        i += 1


# ------------------------------------------------------------------------------------------------
# geometry
# ------------------------------------------------------------------------------------------------
def find_burner(lit, near, search=80, all_r=False):
    """Burner sphere in an illuminated pre-ignition frame (binned image), searched within `search`
    binned pixels of `near` (the centroid of the flame light: the flame surrounds the burner).
    Matched search: for each trial radius r the contrast (disc of 0.8 r) - (annulus 1.2 r .. 1.6 r)
    is computed at every pixel by box filtering; the best (x, y, r) is kept. Returns (cx, cy, r) or None."""
    if near is None:
        return None
    g = cv2.medianBlur(lit.astype(np.float32), 5)
    h, w = g.shape
    x0, x1 = int(max(0, near[0] - search)), int(min(w, near[0] + search))
    y0, y1 = int(max(0, near[1] - search)), int(min(h, near[1] + search))
    best, best_s, per_r = None, -1e18, []
    for r in range(6, 40, 2):
        kd = np.zeros((4 * r + 1, 4 * r + 1), np.float32)
        cv2.circle(kd, (2 * r, 2 * r), int(0.8 * r), 1, -1)
        ka = np.zeros_like(kd)
        cv2.circle(ka, (2 * r, 2 * r), int(1.6 * r), 1, -1)
        cv2.circle(ka, (2 * r, 2 * r), int(1.2 * r), 0, -1)
        ka[: 2 * r] = 0  # the tube enters from above: leave the upper half of the annulus out
        disc = cv2.filter2D(g, -1, kd / kd.sum(), borderType=cv2.BORDER_REFLECT)
        ann = cv2.filter2D(g, -1, ka / ka.sum(), borderType=cv2.BORDER_REFLECT)
        c = (disc - ann)[y0:y1, x0:x1]
        k = int(np.argmax(c))
        yy, xx = divmod(k, c.shape[1])
        per_r.append((float(x0 + xx), float(y0 + yy), float(r)))
        if c.flat[k] > best_s * 1.02:
            best, best_s = (float(x0 + xx), float(y0 + yy), float(r)), float(c.flat[k])
    return per_r if all_r else best


def flame_centroid(frames, lit, meta):
    """Centroid of the brightest flame light summed over unlit frames between onset and sensor end."""
    end = meta["sensor_end_s"] if np.isfinite(meta["sensor_end_s"]) else 20.0
    use = [f for i, f in enumerate(frames) if not lit[i] and 1.0 < f["t"] < end - 0.5]
    if not use:
        return None
    acc = np.zeros(use[0]["img"].shape, np.float32)
    for f in use[:: max(1, len(use) // 40)]:
        x = f["img"] - np.median(f["img"])
        acc += cv2.medianBlur(x.astype(np.float32), 5) / max(1.0, float(np.percentile(x, 99.9)))
    acc = cv2.GaussianBlur(acc, (0, 0), 3)
    m = acc > 0.3 * acc.max()
    yy, xx = np.nonzero(m)
    return (float(xx.mean()), float(yy.mean())) if len(xx) else None


def silhouette_burner(lit, b):
    """Outline of the sphere around a first guess `b` = (x, y, r). The sphere is lit from one side:
    one hemisphere is brighter than the chamber background, the other darker. Both differ from it, so
    the silhouette is the set of pixels whose |value - background| exceeds 5 noise sigma (background and
    sigma from an annulus at 3-4.5 r), closed morphologically, the component holding the guess. On 120
    rays (the tube sector above left out) the outermost silhouette pixel within 2.2 r is an outline
    point; a circle is fitted to the points, refitted twice without outliers.
    Returns (cx, cy, r, rms, n_points) or None."""
    x, y, r = b
    g = cv2.GaussianBlur(cv2.medianBlur(lit.astype(np.float32), 5), (0, 0), 1.0)
    h, w = g.shape
    yy, xx = np.ogrid[:h, :w]
    d2 = (xx - x) ** 2 + (yy - y) ** 2
    ann = (d2 > (3 * r) ** 2) & (d2 < (4.5 * r) ** 2)
    bgv = float(np.median(g[ann]))
    sig = 1.4826 * float(np.median(np.abs(g[ann] - bgv))) + 1e-6
    m = (np.abs(g - bgv) > 5 * sig).astype(np.uint8)
    m = cv2.morphologyEx(m, cv2.MORPH_CLOSE, np.ones((5, 5), np.uint8))
    m = cv2.morphologyEx(m, cv2.MORPH_OPEN, np.ones((3, 3), np.uint8))
    n, lab = cv2.connectedComponents(m)
    l0 = lab[int(round(y)), int(round(x))]
    if l0 == 0:
        return None
    m = (lab == l0).astype(np.float32)
    rmax = int(2.2 * r) + 2
    pol = cv2.warpPolar(m, (rmax, 120), (float(x), float(y)), float(rmax), cv2.WARP_POLAR_LINEAR + cv2.INTER_NEAREST + cv2.WARP_FILL_OUTLIERS)
    ang = (np.arange(120) + 0.5) * 3.0
    pts = []
    for i in range(120):
        if abs(((ang[i] - 270 + 180) % 360) - 180) < 35:
            continue
        on = np.where(pol[i] > 0.5)[0]
        if not len(on) or on[-1] >= rmax - 2:
            continue
        k = on[-1] + 0.5
        pts.append((x + k * math.cos(math.radians(ang[i])), y + k * math.sin(math.radians(ang[i]))))
    pts = np.array(pts)
    if len(pts) < 20:
        return None
    keep = np.ones(len(pts), bool)
    for _ in range(3):
        cx, cy, rr = circle_fit(pts[keep, 0], pts[keep, 1])
        res = np.abs(np.hypot(pts[:, 0] - cx, pts[:, 1] - cy) - rr)
        keep = res < max(1.5, 2.5 * float(np.median(res[keep])))
    cx, cy, rr = circle_fit(pts[keep, 0], pts[keep, 1])
    res = np.hypot(pts[keep, 0] - cx, pts[keep, 1] - cy) - rr
    return float(cx), float(cy), float(rr), float(np.sqrt(np.mean(res ** 2))), int(keep.sum())


def refine_burner_radius(lit, cx, cy, r0):
    """Radius of the sphere from the azimuthal median profile around its centre (the tube sector above
    left out): the outer edge, where the profile has fallen to background + 10 % of the disc contrast,
    and the half-contrast radius. The lit sphere darkens towards its limb, so the half-contrast radius
    underestimates and the 10 % radius (slightly blurred outward) is taken; both are returned."""
    pol = polar(cv2.medianBlur(lit.astype(np.float32), 5), cx, cy, int(3 * r0) + 4)
    ang = np.linspace(0, 360, pol.shape[0], endpoint=False)
    keep = np.abs(((ang - 270 + 180) % 360) - 180) > 30  # 270 deg = up in image coordinates
    prof = np.median(pol[keep], axis=0)
    inner = float(np.median(prof[: max(2, int(0.5 * r0))]))
    bgv = float(np.median(prof[int(2.0 * r0):]))
    out = {}
    for name, frac in (("r10", 0.1), ("r50", 0.5)):
        lvl = bgv + frac * (inner - bgv)
        k = np.where(prof[int(0.5 * r0):] < lvl)[0] if inner > bgv else np.where(prof[int(0.5 * r0):] > lvl)[0]
        if not len(k):
            out[name] = NAN
            continue
        j = int(0.5 * r0) + int(k[0])
        y0, y1 = prof[j - 1], prof[j]
        out[name] = (j - 1) + (y0 - lvl) / (y0 - y1) if y0 != y1 else float(j)
    out["contrast"] = inner - bgv
    return out


def polar(img, cx, cy, rmax, n_ang=N_RAYS * 4):
    """Rows = angle (0..360 deg, 0 = +x, 90 = +y down), columns = radius 0..rmax-1 px."""
    out = cv2.warpPolar(img.astype(np.float32), (int(rmax), n_ang), (float(cx), float(cy)), float(rmax),
                        cv2.WARP_POLAR_LINEAR + cv2.INTER_LINEAR + cv2.WARP_FILL_OUTLIERS)
    return out


def circle_fit(x, y):
    a = np.c_[2 * x, 2 * y, np.ones(len(x))]
    b = x * x + y * y
    sol, *_ = np.linalg.lstsq(a, b, rcond=None)
    cx, cy, c = sol
    return float(cx), float(cy), float(math.sqrt(max(c + cx * cx + cy * cy, 0.0)))


def ray_edges(img, cx, cy, rmin, noise, block_deg):
    """Per-ray outer half-maximum of the outermost significant hump. Returns angles (deg), edge radii
    (px, nan if invalid), peak/noise, and the radius at which each ray leaves the frame."""
    h, w = img.shape
    rmax = int(math.hypot(max(cx, w - cx), max(cy, h - cy))) + 2
    pol = polar(img, cx, cy, rmax)
    n = pol.shape[0] // N_RAYS
    pol = pol[: n * N_RAYS].reshape(N_RAYS, n, -1).mean(axis=1)           # 4 deg sectors
    pol = cv2.GaussianBlur(pol, (5, 1), 1.0)                               # along radius
    if noise is None:
        # per-sample noise of the profiles: differences two samples apart (the smoothing correlates
        # neighbours), robust over all rays and radii, divided by sqrt 2
        dd = (pol[:, 4:] - pol[:, :-4]).ravel()
        dd = dd[dd != 0]
        noise = 1.4826 * float(np.median(np.abs(dd - np.median(dd)))) / math.sqrt(2) + 1e-6 if dd.size else 1.0
    ray_edges.last_noise = noise
    ang = (np.arange(N_RAYS) + 0.5) * 360.0 / N_RAYS
    edges = np.full(N_RAYS, NAN)
    snr = np.zeros(N_RAYS)
    rlim = np.zeros(N_RAYS)
    alts = {}
    for i in range(N_RAYS):
        c, s_ = math.cos(math.radians(ang[i])), math.sin(math.radians(ang[i]))
        lims = []
        for v, lo, hi in ((c, -cx, w - 1 - cx), (s_, -cy, h - 1 - cy)):
            if v > 1e-9:
                lims.append(hi / v)
            elif v < -1e-9:
                lims.append(lo / v)
        rlim[i] = min(lims) - 3 * 1.0
        if any(abs(((ang[i] - b + 180) % 360) - 180) < TUBE_HALF_ANGLE_DEG for b in block_deg):
            continue
        p = pol[i]
        r0, r1 = int(rmin), int(max(rmin + 4, min(rlim[i], len(p) - 1)))
        seg = p[r0:r1]
        if seg.size < 4:
            continue
        gmax = float(seg.max())
        snr[i] = gmax / noise
        if snr[i] < RAY_SNR_MIN:
            continue
        # a faint flame sheet outside a saturated soot zone or a hot burner is still the rim: the
        # outermost hump counts when it is significant and at least a tenth of the ray maximum
        thr = max(RAY_SNR_MIN * noise, 0.1 * gmax)
        above = np.where(seg >= thr)[0]
        if not np.isfinite(gmax) or not len(above):
            continue
        # every hump above the threshold gives a candidate edge (its outer half maximum); the outermost
        # is the default rim, the caller may pick another hump of the same ray (see measure_frame)
        runs_, start = [], None
        for j in range(seg.size + 1):
            on = j < seg.size and seg[j] >= thr
            if on and start is None:
                start = j
            elif not on and start is not None:
                runs_.append((start, j))
                start = None
        cands = []
        for a0, a1 in runs_:
            k = r0 + a0 + int(np.argmax(seg[a0:a1]))
            pk = float(p[k])
            half = 0.5 * pk
            below = np.where(p[k:r1] < half)[0]
            if not len(below):
                continue                  # the profile leaves the frame before falling: no edge here
            jj = k + int(below[0])
            y0, y1 = float(p[jj - 1]), float(p[jj])
            cands.append(((jj - 1) + (y0 - half) / (y0 - y1) if y0 != y1 else float(jj), pk / noise))
        if not cands:
            continue
        alts[i] = cands
        edges[i], snr[i] = cands[-1]
    ray_edges.last_alts = alts
    return ang, edges, snr, rlim


def robust_circle(px, py, ox, oy):
    """Circle through rim points; points far from the median distance to the ray origin (ox, oy) are
    left out before the first fit, and points off the fitted circle after each fit."""
    d = np.hypot(px - ox, py - oy)
    md = float(np.median(d))
    keep = np.abs(d - md) < max(3.0, 3.0 * 1.4826 * float(np.median(np.abs(d - md))), 0.15 * md)
    for _ in range(4):
        if keep.sum() < 6:
            break
        cx, cy, r = circle_fit(px[keep], py[keep])
        res = np.abs(np.hypot(px - cx, py - cy) - r)
        mad = 1.4826 * float(np.median(res[keep])) + 1e-6
        keep = res < max(2.0, 3.0 * mad)
    cx, cy, r = circle_fit(px[keep], py[keep])
    return cx, cy, r, keep


def measure_frame(rim_img, sat, bg, cx, cy, r_burner, block_deg):
    """One frame: rim radius (px), asymmetry, coverage, from the rim channel minus its background."""
    x = cv2.medianBlur((rim_img - bg).astype(np.float32), 3)
    x = x - float(np.median(x))
    # the rays average 4-degree sectors and are smoothed along the radius; their noise is measured on
    # the ray profiles themselves (sample-to-sample differences, robust), not assumed from pixel noise
    xs = cv2.GaussianBlur(x, (0, 0), 1.5)
    noise = None
    rmin = 1.15 * r_burner
    ang, e, snr, rlim = ray_edges(xs, cx, cy, rmin, noise, block_deg)
    noise = ray_edges.last_noise
    alts = ray_edges.last_alts
    if np.isfinite(e).sum() >= 8:
        # a ray whose outermost hump is an artefact (a reflection, an igniter, noise) still has the rim
        # among its humps: take the hump edge nearest the median outermost edge of all rays
        r0m = float(np.nanmedian(e))
        for i, cands in alts.items():
            best = min(cands, key=lambda c: abs(c[0] - r0m))
            if abs(best[0] - r0m) < abs(e[i] - r0m):
                e[i], snr[i] = best
    open_rays = N_RAYS - sum(1 for a in ang if any(abs(((a - b + 180) % 360) - 180) < TUBE_HALF_ANGLE_DEG for b in block_deg))
    ok = np.isfinite(e)
    res = dict(noise=noise, n_rays=int(ok.sum()), ray_frac=ok.sum() / float(open_rays),
               signal_sum=float(np.clip(x, 0, None).sum()))
    if ok.sum() < 8:
        return res, x, None
    th = np.radians(ang[ok])
    px, py = cx + e[ok] * np.cos(th), cy + e[ok] * np.sin(th)
    fcx, fcy, fr, keep = robust_circle(px, py, cx, cy)
    px, py = px[keep], py[keep]
    # coverage: share of 10-degree sectors around the rim centre that hold a point
    phi = np.degrees(np.arctan2(py - fcy, px - fcx)) % 360
    coverage = len(set((phi // 10).astype(int))) / 36.0
    if coverage < 0.5 or not (0 < fr < 4 * max(x.shape)):
        # a short arc does not fix a centre: the burner centre is used instead
        fcx, fcy = cx, cy
        dist = np.hypot(px - cx, py - cy)
        rr = float(np.median(dist))
        centre = "burner"
    else:
        dist = np.hypot(px - fcx, py - fcy)
        rr = fr
        centre = "rim"
    p10, p90 = np.percentile(dist, [10, 90])
    ring = np.zeros(x.shape, np.uint8)
    cv2.circle(ring, (int(round(fcx)), int(round(fcy))), int(round(rr * 1.05)), 1, -1)
    cv2.circle(ring, (int(round(fcx)), int(round(fcy))), int(round(rr * 0.7)), 0, -1)
    res.update(r_px=rr, r_p10_px=float(p10), r_p90_px=float(p90), asym=float((p90 - p10) / (2 * rr)),
               rim_cx=fcx, rim_cy=fcy, centre_offset=math.hypot(fcx - cx, fcy - cy) / rr, centre_used=centre,
               coverage=coverage, n_rays=int(len(px)), ray_frac=len(px) / float(open_rays),
               peak_snr=float(np.median(snr[ok][keep])),
               sat_frac=float(sat[ring > 0].mean()) if sat is not None and ring.any() else 0.0)
    return res, x, np.c_[px, py]


# ------------------------------------------------------------------------------------------------
# video alignment
# ------------------------------------------------------------------------------------------------
def align_video(pts, bright, t, d, onset_clock):
    """Lag (s) such that t_clock = pts + lag, by cross-correlating the video's frame brightness with
    the summed photomultiplier signal (both differentiated in time, 30 Hz)."""
    pmt = sum(d[c] for c in ("PMT1", "PMT2", "PMT3"))
    fs = 30.0
    tv = np.arange(pts[0], pts[-1], 1 / fs)
    bv = np.interp(tv, pts, bright)
    tp = np.arange(t[0], t[-1], 1 / fs)
    bp = np.interp(tp, t, pmt)
    # the video starts on another view; only the dark part is comparable. Use log-brightness changes.
    def prep(y):
        # changes of log-brightness: the ignition flash and the extinction are sharp in both records
        y = np.log1p(np.maximum(y - np.percentile(y, 5), 0) / (np.percentile(y, 99) - np.percentile(y, 5) + 1e-9) * 100)
        y = np.convolve(y, np.ones(3) / 3, mode="same")
        d = np.gradient(y)
        return d - d.mean()
    a, b = prep(bv), prep(bp)
    # search lags: video time + lag = clock time
    best, best_c, cs = NAN, -1e9, []
    lags = np.arange(tp[0] - tv[-1], tp[-1] - tv[0], 1 / fs)
    # coarse-to-fine: compute correlation via FFT
    n = len(a) + len(b)
    nfft = 1 << (n - 1).bit_length()
    cc = np.fft.irfft(np.fft.rfft(b, nfft) * np.conj(np.fft.rfft(a, nfft)), nfft)
    # cc[k] = sum_i b[i+k] a[i]; lag in samples k (can be negative via wrap)
    ks = np.r_[np.arange(0, len(b)), np.arange(-len(a) + 1, 0)]
    vals = np.r_[cc[: len(b)], cc[nfft - len(a) + 1:]]
    # normalise by overlap length
    overlap = np.array([min(len(a), len(b) - k) if k >= 0 else min(len(a) + k, len(b)) for k in ks], float)
    vals = vals / np.maximum(overlap, 1) / (a.std() * b.std() + 1e-12)
    good = overlap > 0.3 * min(len(a), len(b))
    vals[~good] = -1e9
    k = int(np.argmax(vals))
    lag = tp[0] + ks[k] / fs - tv[0]
    srt = np.sort(vals[good])[::-1]
    # second best lag more than 2 s away
    far = np.abs(ks - ks[k]) > 2 * fs
    second = float(np.max(vals[good & far])) if (good & far).any() else NAN
    # width: lags within 0.9 of the peak
    near = np.where(vals > 0.9 * vals[k])[0]
    width = (ks[near].max() - ks[near].min()) / fs if len(near) else NAN
    return dict(align_lag_s=float(lag), align_corr=float(vals[k]), align_second_corr=second,
                align_peak_width_s=float(width))


# ------------------------------------------------------------------------------------------------
# run one source of one test
# ------------------------------------------------------------------------------------------------
def find_source(raw, test, source):
    raw = Path(raw)
    pats = {
        "colour": [f"Images_{test}.zip", f"Images_ {test}.zip", f"{test}_Images.zip", f"{test}_ACME.zip"],
        "intns": [f"{test}_INTNS_ALL.zip", f"{test}_INTNS_ALLzip.zip"],
    }
    if source == "video":
        c = sorted(list(raw.glob(f"vlc-record-*{test}.ts")) + list(raw.glob(f"vlc-record-*{test}.mp4")))
        return c[0] if c else None
    for p in pats[source]:
        if (raw / p).exists():
            return raw / p
    return None


def run(test, source, raw, out, every=1, scale_from=None):
    """scale_from: the series JSON of another test of the same day and camera (the zoom is set once per
    day by the camera configuration script), used when this test has no clean lit frame."""
    t_start = time.time()
    path = find_source(raw, test, source)
    if path is None:
        raise SystemExit(f"{test}: no {source} file in {raw}")
    ref, t, d = sensor_reference(test, raw)
    onset = ref.get("onset_clock", NAN)
    meta = dict(test=test, source=source, file=path.name, file_bytes=path.stat().st_size,
                time_origin="sensor flame onset (flame_design.py radiometer onset), seconds",
                sensor_onset_clock_s=onset, sensor_end_s=ref.get("end_s", NAN), sensor_pmt_end_s=ref.get("pmt_end_s", NAN),
                sensor_label=ref.get("label_sensor", ""), sensor_tp_class=ref.get("tp_class", ""))
    if source == "colour":
        frames = list(colour_frames(path))
    elif source == "intns":
        frames = list(intns_frames(path, every=every))
    else:
        frames = list(video_frames(path))
    meta["n_frames"] = len(frames)
    if not frames:
        raise SystemExit("no frames")
    imgs_mean = np.array([f["mean"] for f in frames])

    # --- time ----------------------------------------------------------------------------------
    if source == "video":
        pts = np.array([f["pts_s"] for f in frames])
        # brightness of the central region only (the view is mostly dark when the flame camera is shown)
        al = align_video(pts, imgs_mean, t, d, onset)
        meta.update(al)
        meta["time_alignment"] = "cross-correlation of frame brightness with summed PMT signal"
        for f in frames:
            f["t_clock"] = f["pts_s"] + al["align_lag_s"]
    else:
        meta["time_alignment"] = "frame file-name stamp (camera unit clock) taken as the sensor clock"
        # midnight
        tc = np.array([f["t_clock"] for f in frames])
        if np.isfinite(onset) and np.nanmedian(tc) < onset - 40000:
            for f in frames:
                f["t_clock"] += 86400
    for f in frames:
        f["t"] = f["t_clock"] - onset

    # --- lit (illuminated) frames: the burner, and which frames are not flame-view -------------------
    rel = imgs_mean / np.median(imgs_mean)
    # colour/intensified: the chamber lamp lights the view before ignition only; bright frames after
    # the onset are flame light (saturated sooty phase), not another view
    lit = (rel > 5.0) & (np.array([f["t"] for f in frames]) < -0.2) if source != "video" else np.zeros(len(frames), bool)
    if source == "video":
        # the video starts on a lit chamber view; the flame view is dark. Lit = mean far above the dark mode.
        dark_level = np.percentile(imgs_mean, 20)
        lit = imgs_mean > dark_level + 0.25 * (np.percentile(imgs_mean, 99) - dark_level)
    burner = None
    pre_lit = [i for i, f in enumerate(frames) if lit[i] and f["t"] < 0]
    if source in ("colour", "intns"):
        near = flame_centroid(frames, lit, meta)
        tries = []
        if pre_lit:
            stack = np.median(np.stack([frames[i]["img"] for i in pre_lit[-5:]]), axis=0)
            satf = float(np.mean(np.stack([frames[i]["sat"] for i in pre_lit[-5:]])))
            if satf < 0.3:
                tries.append(("lit pre-ignition frames", stack, False))
            else:
                meta["burner_lit_note"] = f"lit frames saturated ({satf:.0%}); not used"
        # Not used: the dark disc that a sooty or large flame shows around the burner. It looks like the
        # burner but is the soot-free inner zone, up to twice the burner size (seen on 21088L1 against
        # the lit frames of the same day).
        fits = []
        for label, img, invert in tries:
            cands = find_burner(-img if invert else img, near=near, all_r=True) or []
            best = None
            for b in cands:
                hc = silhouette_burner(img, b)
                if hc is None or hc[4] < 40 or hc[3] > 2.5:
                    continue
                score = hc[4] / (hc[3] + 0.3)
                if best is None or score > best[0]:
                    best = (score, hc, b)
            if best is None:
                continue
            fits.append((label, img, best[1], best[2]))
            if not invert:
                break               # a lit frame is the reference; the flame-backlit frames are the fallback
        if fits:
            if len(fits) > 1:
                arr = np.array([f[2][:3] for f in fits])
                med = np.median(arr, axis=0)
                k = int(np.argmin(np.abs(arr[:, 2] - med[2])))
                meta.update(burner_n_frames=len(fits), burner_r_spread_px=float(np.std(arr[:, 2])) * frames[0]["px_bin"])
                label, img, hc, b = fits[k]
                hc = (float(med[0]), float(med[1]), float(med[2]), hc[3], hc[4])
            else:
                label, img, hc, b = fits[0]
            burner = hc[:3]
            meta.update(burner_method="circle fitted to the sphere silhouette", burner_image=label,
                        burner_fit_rms_px=hc[3] * frames[0]["px_bin"], burner_fit_points=hc[4],
                        burner_match_r_px=b[2] * frames[0]["px_bin"])
            bx, by, br = burner
            ox, oy = max(0, int(bx - 4 * br)), max(0, int(by - 4 * br))
            cr = img[oy:int(by + 4 * br), ox:int(bx + 4 * br)]
            lo, hi = np.percentile(cr, [0.5, 99.5])
            v = np.clip((cr - lo) / max(1e-6, hi - lo) * 255, 0, 255).astype(np.uint8)
            v = cv2.resize(cv2.cvtColor(v, cv2.COLOR_GRAY2BGR), None, fx=4, fy=4, interpolation=cv2.INTER_NEAREST)
            cv2.circle(v, (int((bx - ox) * 4), int((by - oy) * 4)), int(br * 4), (0, 255, 0), 1)
            Path(out, "check").mkdir(parents=True, exist_ok=True)
            cv2.imwrite(str(Path(out, "check", f"{test}_{source}_burner.png")), v)
    px_bin = frames[0]["px_bin"]
    if burner is None and scale_from:
        ref_meta = json.loads(Path(scale_from).read_text(encoding="utf-8"))
        if np.isfinite(ref_meta.get("burner_r_px", NAN)):
            burner = (ref_meta["burner_cx_px"] / px_bin, ref_meta["burner_cy_px"] / px_bin, ref_meta["burner_r_px"] / px_bin)
            meta["burner_image"] = f"transferred from {ref_meta['test']} (same day, same camera configuration)"
    if burner is None and source == "colour" and "PSI-159" in str(LISTING):
        c = CFIG_COLOUR_SCALE
        rb_full = BURNER_D_MM / 2 / c["mm_per_px_full"]
        burner = (c["burner_cx_px"] / px_bin, c["burner_cy_px"] / px_bin, rb_full / px_bin)
        meta["burner_image"] = c["note"]
    if burner is not None:
        cx, cy, rb = burner
        mm_per_px = BURNER_D_MM / (2 * rb)
        meta.update(burner_cx_px=cx * px_bin, burner_cy_px=cy * px_bin, burner_r_px=rb * px_bin,
                    scale_source="burner sphere (6.35 mm), " + meta.get("burner_image", ""), mm_per_px_binned=mm_per_px,
                    mm_per_px_full=mm_per_px / px_bin)
    else:
        cx = cy = rb = None
        mm_per_px = NAN
        meta["scale_source"] = "none (no burner found)"

    # --- dark background ----------------------------------------------------------------------------
    dark_idx = [i for i, f in enumerate(frames) if not lit[i]]
    pre_dark = [i for i in dark_idx if frames[i]["t"] < -0.3]
    post = [i for i in dark_idx if np.isfinite(meta["sensor_end_s"]) and frames[i]["t"] > meta["sensor_end_s"] + 3]
    # one background per exposure setting (the colour camera brackets 50/100/200/400 ms in some tests)
    bgs, rim_bgs, notes = {}, {}, []
    exps = sorted({round(frames[i]["exposure_s"], 4) for i in dark_idx if np.isfinite(frames[i]["exposure_s"])}) or [NAN]
    for e in exps:
        same = (lambda i: round(frames[i]["exposure_s"], 4) == e) if np.isfinite(e) else (lambda i: True)
        cand, where = [i for i in pre_dark if same(i)][-5:], "pre-onset"
        if len(cand) < 2:
            cand, where = [i for i in post if same(i)][-5:], "post-end"
        if len(cand) < 2:
            cand, where = (pre_dark[-5:] or post[-5:]), "other exposure"
        if cand:
            bgs[e] = np.median(np.stack([frames[i]["img"] for i in cand]), axis=0).astype(np.float32)
            rim_bgs[e] = np.median(np.stack([frames[i]["rim"] for i in cand]), axis=0).astype(np.float32)
            notes.append(f"{e}s: {len(cand)} {where}")
    meta["background"] = "; ".join(notes) if notes else "none"
    zero = np.zeros_like(frames[0]["img"], dtype=np.float32)

    def bg_for(f):
        e = round(f["exposure_s"], 4) if np.isfinite(f["exposure_s"]) else NAN
        if e in bgs:
            return bgs[e]
        return next(iter(bgs.values())) if bgs else zero
    bg = next(iter(bgs.values())) if bgs else zero

    def rim_bg_for(f):
        e = round(f["exposure_s"], 4) if np.isfinite(f["exposure_s"]) else NAN
        if e in rim_bgs:
            return rim_bgs[e]
        return next(iter(rim_bgs.values())) if rim_bgs else zero

    h, w = frames[0]["img"].shape
    # centre for frames without burner: brightest-blob centroid of the dark-view frames (video)
    if cx is None:
        # how often each pixel is lit (above 5 noise sigma) over the flame frames: the flame disc is lit
        # in nearly every frame, a small saturated blob (igniter, soot) covers little area, so the
        # centroid of the often-lit area is the flame centre rather than the brightest blob
        acc = np.zeros((h, w), np.float32)
        tend = meta["sensor_end_s"] if np.isfinite(meta["sensor_end_s"]) else 30
        use = [i for i in dark_idx if 0.5 < frames[i]["t"] < tend]
        for i in use[:: max(1, len(use) // 80)]:
            x = cv2.GaussianBlur((frames[i]["rim"] - rim_bg_for(frames[i])).astype(np.float32), (0, 0), 2)
            x -= float(np.median(x))
            sd = 1.4826 * float(np.median(np.abs(x))) + 1e-6
            acc += (x > 5 * sd).astype(np.float32)
        if acc.max() > 0:
            m = acc >= 0.5 * acc.max()
            n_lab, lab, stats, cents = cv2.connectedComponentsWithStats(m.astype(np.uint8))
            k = 1 + int(np.argmax(stats[1:, cv2.CC_STAT_AREA])) if n_lab > 1 else 0
            cx, cy = float(cents[k][0]), float(cents[k][1])
        else:
            cx, cy = w / 2.0, h / 2.0
        rb = 3.0
    block = [270.0] if burner is not None else []  # the supply tube, straight up

    if burner is None:
        # no burner in view (video, intensified camera): a first pass from the brightest blob, then the
        # median centre of the well-covered rims of that pass is the ray origin of the second pass
        centres = []
        flame_frames = [f for i, f in enumerate(frames) if not lit[i] and 0.5 < f["t"] < (meta["sensor_end_s"] if np.isfinite(meta["sensor_end_s"]) else 30)]
        for f in flame_frames[:: max(1, len(flame_frames) // 60)]:
            res, x, pts = measure_frame(f["rim"], f["sat"], rim_bg_for(f), cx, cy, rb, block)
            if pts is not None and res.get("coverage", 0) >= 0.6 and res["n_rays"] >= 30 and res["centre_used"] == "rim":
                centres.append((res["rim_cx"], res["rim_cy"]))
        if len(centres) >= 3:
            cx, cy = float(np.median([c[0] for c in centres])), float(np.median([c[1] for c in centres]))
            meta["ray_origin"] = f"median rim centre of {len(centres)} first-pass frames"
        else:
            meta["ray_origin"] = "centroid of the most often lit area of the flame frames"

    rows = []
    sheet = []
    for i, f in enumerate(frames):
        row = dict(test=test, source=source, frame=f["name"], t_s=f["t"], t_clock_s=f["t_clock"],
                   exposure_s=f["exposure_s"], gain=f["gain"], lit=int(lit[i]))
        if lit[i]:
            row["quality"] = "lit_view"
            rows.append(row)
            continue
        cxx, cyy = cx, cy
        res, x, pts = measure_frame(f["rim"], f["sat"], rim_bg_for(f), cxx, cyy, rb, block)
        exp = f["exposure_s"] if np.isfinite(f["exposure_s"]) and f["exposure_s"] > 0 else 1.0
        lum = cv2.medianBlur((f["img"] - bg_for(f)).astype(np.float32), 3)
        row.update(noise=res["noise"], signal_per_s=float(np.clip(lum - np.median(lum), 0, None).sum()) / exp,
                   n_rays=res["n_rays"], ray_frac=res["ray_frac"])
        if pts is not None:
            row.update(r_px=res["r_px"] * px_bin, r_m=res["r_px"] * mm_per_px * 1e-3 if np.isfinite(mm_per_px) else NAN,
                       r_p10_m=res["r_p10_px"] * mm_per_px * 1e-3 if np.isfinite(mm_per_px) else NAN,
                       r_p90_m=res["r_p90_px"] * mm_per_px * 1e-3 if np.isfinite(mm_per_px) else NAN,
                       asym=res["asym"], centre_offset=res["centre_offset"], peak_snr=res["peak_snr"],
                       sat_frac=res["sat_frac"], rim_cx_px=res["rim_cx"] * px_bin, rim_cy_px=res["rim_cy"] * px_bin,
                       coverage=res["coverage"], centre_used=res["centre_used"])
        q = []
        if pts is None:
            q.append("no_rim")
        else:
            if res["n_rays"] < 12:
                q.append("few_rays")
            if res["centre_used"] == "burner":
                q.append("partial_arc")
            if res["sat_frac"] > 0.02:
                q.append("saturated")
            if res["peak_snr"] < 10:
                q.append("low_snr")
            if res["asym"] > 0.25:
                q.append("asymmetric")
        row["quality"] = "|".join(q) if q else "good"
        rows.append(row)
        sheet.append((f, None, pts, row))

    detect_end(rows, source, meta)

    # --- rate and cost ----------------------------------------------------------------------------
    tt = np.array([r["t_s"] for r in rows if not r["lit"]])
    if len(tt) > 2:
        dt = np.diff(tt)
        meta["frame_interval_median_s"] = float(np.median(dt))
        meta["frame_interval_max_s"] = float(np.max(dt))
    meta["compute_s"] = time.time() - t_start

    # --- write ----------------------------------------------------------------------------------
    out = Path(out)
    (out / "series").mkdir(parents=True, exist_ok=True)
    (out / "check").mkdir(parents=True, exist_ok=True)
    keys = ["test", "source", "frame", "t_s", "t_clock_s", "exposure_s", "gain", "lit", "quality", "r_m", "r_p10_m",
            "r_p90_m", "r_px", "asym", "centre_offset", "coverage", "centre_used", "peak_snr", "sat_frac", "n_rays", "ray_frac", "signal_per_s",
            "noise", "rim_cx_px", "rim_cy_px"]
    with open(out / "series" / f"{test}_{source}.csv", "w", newline="", encoding="utf-8") as fh:
        wr = csv.DictWriter(fh, fieldnames=keys, extrasaction="ignore")
        wr.writeheader()
        for r in rows:
            wr.writerow({k: (f"{v:.6g}" if isinstance(v, float) else v) for k, v in r.items()})
    (out / "series" / f"{test}_{source}.json").write_text(json.dumps(meta, indent=1, default=float), encoding="utf-8")
    meta["_prep"] = lambda f: (lambda x: x - float(np.median(x)))(cv2.medianBlur((f["rim"] - rim_bg_for(f)).astype(np.float32), 3))
    write_sheet(out / "check" / f"{test}_{source}.jpg", sheet, meta, (cx, cy, rb), burner is not None,
                frames, lit, px_bin)
    meta.pop("_prep", None)
    return meta, rows


def detect_end(rows, source, meta):
    """Flame onset and end of one source from its per-frame rows (rim and light criteria), on the
    test clock. Updates and returns meta. Also used by `reend` on a written series."""
    # --- flame onset and end from images -----------------------------------------------------------------
    # A frame "shows the flame" when a rim of 12 points or more with a median peak of 10 sigma or more
    # is found. The end lies between the last such frame and the first of three consecutive frames
    # without one; the error bar is half that gap.
    unlit = [r for r in rows if not r["lit"]]
    # bracketed exposures: the shortest ones lose a faint flame, so the end is read on the longest
    exps_all = sorted({round(r["exposure_s"], 4) for r in unlit if np.isfinite(r["exposure_s"])})
    if len(exps_all) > 1:
        unlit = [r for r in unlit if np.isfinite(r["exposure_s"]) and round(r["exposure_s"], 4) == exps_all[-1]]
        end_note = f"end read on the {exps_all[-1]:g} s exposures of the bracket"
    else:
        end_note = ""
    shows = np.array([np.isfinite(r.get("r_px", NAN)) and r.get("n_rays", 0) >= 12 and r.get("peak_snr", 0) >= 10
                      for r in unlit])
    # a rim far smaller than the flame just before it is not the flame: after a burner-limit stop the
    # glowing burner (500 C) shows as a small disc for seconds
    hist = []
    for k, r in enumerate(unlit):
        if shows[k]:
            if len(hist) >= 5 and r["r_px"] < 0.6 * float(np.median(hist[-30:])):
                shows[k] = False
                r["quality"] = (r["quality"] + "|small_rim").strip("|")
            else:
                hist.append(r["r_px"])
    ts = np.array([r["t_s"] for r in unlit])

    def gap_n(k):
        # the flame is out when it stays out for 3 frames and at least 0.5 s (video flickers per frame)
        j = int(np.searchsorted(ts, ts[k] + 0.5))
        return max(3, j - k)
    end = dict(image_onset_s=NAN, image_end_s=NAN, image_end_err_s=NAN, end_exposure_note=end_note)
    on = np.where(shows & (ts > 0.0))[0]
    if len(on):
        k_on = int(on[0])
        end["image_onset_s"] = float(ts[k_on])
        k_end = None
        for k in range(k_on, len(unlit)):
            if not shows[k] and shows[k:k + gap_n(k)].sum() == 0:
                k_end = k
                break
        if k_end is not None:
            t_last, t_next = ts[k_end - 1], ts[k_end]
            end.update(image_end_s=0.5 * (t_last + t_next), image_end_err_s=0.5 * (t_next - t_last),
                       image_last_flame_s=float(t_last), image_first_dark_s=float(t_next))
        else:
            end["image_end_note"] = "flame seen in the last frame: the record ends before the flame"
            end["image_last_frame_s"] = float(ts[-1])
    # Light-based end: the exposure-normalised light of the frame (luminance channel, background and
    # offset removed) against the dark level. Dark level and spread from unlit frames before the
    # igniter heats (t < -1.5 s) with the same exposure, else from the last frames of the record when
    # they are dark. The flame is on while the light exceeds dark + 8 spreads; the end lies between the
    # last such frame and the first of three consecutive frames below.
    sig = np.array([r.get("signal_per_s", NAN) for r in unlit])
    if len(ts) > 20 and float(np.median(np.diff(ts))) < 0.05:
        # video rate: compression noise dominates single frames; the light is averaged over 9 frames
        # (0.3 s) before it is compared with the dark level, which is averaged the same way
        k9 = np.ones(9) / 9.0
        good = np.isfinite(sig)
        sm = np.convolve(np.where(good, sig, 0.0), k9, mode="same") / np.maximum(np.convolve(good.astype(float), k9, mode="same"), 1e-9)
        sig = np.where(good, sm, NAN)
    exps_u = np.array([round(r["exposure_s"], 4) if np.isfinite(r["exposure_s"]) else -1 for r in unlit])
    lvl = np.full(len(unlit), NAN)
    spr = np.full(len(unlit), NAN)
    for e in set(exps_u.tolist()):
        same = exps_u == e
        base = same & (ts < -1.5)
        src = "pre"
        if base.sum() < 2:
            base = same & (ts > ts.max() - 3.0) if ts.max() - (meta.get("sensor_end_s") or 0) > 3 else np.zeros_like(same)
            src = "tail"
        if base.sum() >= 2:
            b = sig[base]
            lvl[same] = float(np.median(b))
            spr[same] = max(1.4826 * float(np.median(np.abs(b - np.median(b)))), 1e-3 * abs(float(np.median(b))) + 1e-9)
    on_l = sig > lvl + 8 * spr
    end.update(light_onset_s=NAN, light_end_s=NAN, light_end_err_s=NAN)
    cand_on = np.where(on_l & (ts > 0.0))[0]
    if np.isfinite(lvl).any() and len(cand_on):
        k_on = int(cand_on[0])
        end["light_onset_s"] = float(ts[k_on])
        k_end = None
        for k in range(k_on, len(unlit)):
            if np.isfinite(lvl[k]) and not on_l[k] and on_l[k:k + gap_n(k)].sum() == 0:
                k_end = k
                break
        if k_end is not None:
            end.update(light_end_s=0.5 * (ts[k_end - 1] + ts[k_end]), light_end_err_s=0.5 * (ts[k_end] - ts[k_end - 1]))
        else:
            end["light_end_note"] = "light above the dark level to the last frame"
    elif not np.isfinite(lvl).any():
        end["light_end_note"] = "no dark frames to set the dark level"
    # best end: the later of the rim end and the light end. A rim lost at low signal or a flame larger
    # than the field ends the rim series early; light that never returns to the dark level leaves the
    # light end undefined. Neither fails late, so the later finite one is taken.
    cands = [(end.get("image_end_s", NAN), end.get("image_end_err_s", NAN), "rim"),
             (end.get("light_end_s", NAN), end.get("light_end_err_s", NAN), "light")]
    cands = [c for c in cands if np.isfinite(c[0])]
    if cands:
        c = max(cands, key=lambda c: c[0])
        end.update(end_s=c[0], end_err_s=c[1], end_from=c[2])
    else:
        end.update(end_s=NAN, end_err_s=NAN, end_from="none")
    if source == "video":
        for k in ("image_end_err_s", "light_end_err_s", "end_err_s"):
            if np.isfinite(end.get(k, NAN)):
                end[k] = math.hypot(end[k], meta.get("align_peak_width_s", 0) / 2)
    meta.update(end)
    return meta


def write_sheet(path, sheet, meta, centre, have_burner, frames, lit, px_bin, n=24):
    """Contact sheet: n frames spread over the flame life, auto-stretched, rim (green) drawn, burner
    (cyan) drawn, sensor end marked in the label."""
    if not sheet:
        return
    t_end = np.nanmax([meta.get("image_end_s", NAN), meta.get("light_end_s", NAN), meta.get("sensor_pmt_end_s", NAN), -1e9])
    t_hi = t_end + 1.5 if t_end > -1e8 else max(s[3]["t_s"] for s in sheet)
    cand = [s for s in sheet if -1.0 <= s[3]["t_s"] <= t_hi]
    if not cand:
        cand = sheet
    pick = [cand[int(round(k))] for k in np.linspace(0, len(cand) - 1, min(n, len(cand)))]
    cx, cy, rb = centre
    # crop window around the centre, sized to the largest rim
    good = [s[3]["r_px"] / px_bin for s in sheet if s[3].get("quality") == "good"]
    rmax_px = float(np.percentile(good, 95)) if good else 40.0
    half = int(min(1.6 * rmax_px + 20, max(sheet[0][0]["img"].shape) / 2))
    tiles = []
    for f, x, pts, row in pick:
        x = meta["_prep"](f)
        h, w = x.shape
        x0, y0 = int(cx) - half, int(cy) - half
        crop = np.zeros((2 * half, 2 * half), np.float32)
        xs0, ys0 = max(0, x0), max(0, y0)
        xs1, ys1 = min(w, x0 + 2 * half), min(h, y0 + 2 * half)
        crop[ys0 - y0:ys1 - y0, xs0 - x0:xs1 - x0] = x[ys0:ys1, xs0:xs1]
        hi = np.percentile(crop, 99.7)
        v = np.clip(crop / (hi if hi > 0 else 1) * 255, 0, 255).astype(np.uint8)
        v = cv2.cvtColor(v, cv2.COLOR_GRAY2BGR)
        if pts is not None:
            for p in pts:
                cv2.circle(v, (int(p[0] - x0), int(p[1] - y0)), 1, (0, 255, 0), -1)
            if np.isfinite(row.get("r_px", NAN)):
                cv2.circle(v, (int(row["rim_cx_px"] / px_bin - x0), int(row["rim_cy_px"] / px_bin - y0)),
                           int(row["r_px"] / px_bin), (0, 160, 0), 1)
        if have_burner:
            cv2.circle(v, (int(cx - x0), int(cy - y0)), int(rb), (255, 255, 0), 1)
        v = cv2.resize(v, (256, 256), interpolation=cv2.INTER_AREA)
        r_mm = row.get("r_m", NAN) * 1e3
        cv2.putText(v, f"t={row['t_s']:.2f}s r={r_mm:.1f}mm", (3, 13), 0, 0.4, (0, 255, 255), 1)
        cv2.putText(v, f"{row['quality'][:28]}", (3, 250), 0, 0.35, (0, 200, 255), 1)
        tiles.append(v)
    while len(tiles) % 6:
        tiles.append(np.zeros_like(tiles[0]))
    img = np.vstack([np.hstack(tiles[k:k + 6]) for k in range(0, len(tiles), 6)])
    head = np.zeros((40, img.shape[1], 3), np.uint8)
    cv2.putText(head, f"{meta['test']} {meta['source']}  sensor end {meta.get('sensor_pmt_end_s', NAN):.2f}s  "
                f"rim end {meta.get('image_end_s', NAN):.2f}  light end {meta.get('light_end_s', NAN):.2f}+-{meta.get('light_end_err_s', NAN):.2f}s  "
                f"scale {meta.get('mm_per_px_full', NAN):.4f} mm/px", (5, 25), 0, 0.5, (255, 255, 255), 1)
    cv2.imwrite(str(path), np.vstack([head, img]), [cv2.IMWRITE_JPEG_QUALITY, 85])


def reend(test, source, out):
    base = Path(out, "series", f"{test}_{source}")
    meta = json.loads(base.with_suffix(".json").read_text(encoding="utf-8"))
    rows = list(csv.DictReader(base.with_suffix(".csv").open(encoding="utf-8")))
    for r in rows:
        for k, v in list(r.items()):
            if k in ("frame", "quality", "test", "source", "gain", "centre_used"):
                continue
            try:
                r[k] = float(v)
            except (TypeError, ValueError):
                r[k] = NAN
        r["lit"] = int(r["lit"]) if np.isfinite(r["lit"]) else 0
        r["quality"] = (r.get("quality") or "").replace("|small_rim", "").replace("small_rim", "")
    for k in [k for k in meta if k.startswith(("image_", "light_", "end_"))]:
        meta.pop(k)
    detect_end(rows, source, meta)
    base.with_suffix(".json").write_text(json.dumps(meta, indent=1, default=float), encoding="utf-8")
    return {k: meta.get(k) for k in ("test", "source", "image_end_s", "light_end_s", "end_s", "end_err_s", "end_from")}


def scale_probe(test, out):
    """Burner scale of a test from its lit pre-ignition colour frames only, read in place from the
    archive (HTTP ranges): the 1 s exposures before ignition. Writes series/<test>_scale.json."""
    t0 = time.time()
    f, row = remote_file(test, "colour")
    frames = list(colour_frames(f, select=lambda name, r: float(r.get("ExposureManual", 0) or 0) >= 5e5))
    frames = frames[-5:]
    meta = dict(test=test, source="colour scale probe", file=row["file_name"], file_bytes=row["size_bytes"])
    if not frames:
        meta["note"] = "no 1 s pre-ignition frames"
    else:
        stack = np.median(np.stack([fr["img"] for fr in frames]), axis=0)
        satf = float(np.mean(np.stack([fr["sat"] for fr in frames])))
        meta.update(lit_frames=len(frames), lit_saturated_fraction=satf, gain=frames[-1]["gain"])
        h, w = stack.shape
        cands = find_burner(stack, near=(w / 2, h / 2), search=int(0.1 * w), all_r=True) or []
        best = None
        for b in cands:
            hc = silhouette_burner(stack, b)
            if hc is None or hc[4] < 40 or hc[3] > 2.5:
                continue
            sc = hc[4] / (hc[3] + 0.3)
            if best is None or sc > best[0]:
                best = (sc, hc)
        if best is not None and satf < 0.3:
            cx, cy, r, rms, npts = best[1]
            meta.update(burner_cx_px=cx * 2, burner_cy_px=cy * 2, burner_r_px=r * 2, burner_fit_rms_px=rms * 2,
                        burner_fit_points=npts, mm_per_px_full=BURNER_D_MM / (4 * r),
                        burner_image="lit pre-ignition frames (scale probe)")
            ox, oy = max(0, int(cx - 4 * r)), max(0, int(cy - 4 * r))
            cr = stack[oy:int(cy + 4 * r), ox:int(cx + 4 * r)]
            lo, hi = np.percentile(cr, [0.5, 99.5])
            v = np.clip((cr - lo) / max(1e-6, hi - lo) * 255, 0, 255).astype(np.uint8)
            v = cv2.resize(cv2.cvtColor(v, cv2.COLOR_GRAY2BGR), None, fx=4, fy=4, interpolation=cv2.INTER_NEAREST)
            cv2.circle(v, (int((cx - ox) * 4), int((cy - oy) * 4)), int(r * 4), (0, 255, 0), 1)
            Path(out, "check").mkdir(parents=True, exist_ok=True)
            cv2.imwrite(str(Path(out, "check", f"{test}_scale_burner.png")), v)
    meta.update(bytes_fetched=f.bytes_fetched, compute_s=time.time() - t0)
    Path(out, "series").mkdir(parents=True, exist_ok=True)
    Path(out, "series", f"{test}_scale.json").write_text(json.dumps(meta, indent=1, default=float), encoding="utf-8")
    return meta


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    sub = ap.add_subparsers(dest="cmd", required=True)
    p = sub.add_parser("run")
    p.add_argument("--test", required=True)
    p.add_argument("--source", required=True, choices=["colour", "intns", "video"])
    p.add_argument("--raw", required=True)
    p.add_argument("--out", required=True)
    p.add_argument("--every", type=int, default=1, help="intensified camera: use every n-th frame")
    p.add_argument("--scale-from", help="series JSON of a same-day test whose lit frames gave the burner")
    e = sub.add_parser("reend", help="recompute onset and end from a written series (no image work)")
    e.add_argument("--test", required=True)
    e.add_argument("--source", required=True, choices=["colour", "intns", "video"])
    e.add_argument("--out", required=True)
    q = sub.add_parser("scale-probe", help="burner scale from the lit frames only, read remotely by HTTP ranges")
    q.add_argument("--test", required=True)
    q.add_argument("--out", required=True)
    a = ap.parse_args()
    if a.cmd == "reend":
        print(json.dumps(reend(a.test, a.source, a.out), default=float))
        return 0
    if a.cmd == "scale-probe":
        print(json.dumps(scale_probe(a.test, a.out), default=float))
        return 0
    meta, rows = run(a.test, a.source, a.raw, a.out, every=a.every, scale_from=a.scale_from)
    keep = {k: meta.get(k) for k in ("test", "source", "n_frames", "scale_source", "mm_per_px_full", "time_alignment",
                                     "align_lag_s", "align_corr", "image_onset_s", "image_end_s", "image_end_err_s", "light_onset_s", "light_end_s", "light_end_err_s", "end_s", "end_err_s", "end_from",
                                     "sensor_pmt_end_s", "sensor_end_s", "frame_interval_median_s", "compute_s")}
    print(json.dumps(keep, default=float))


if __name__ == "__main__":
    sys.exit(main())
