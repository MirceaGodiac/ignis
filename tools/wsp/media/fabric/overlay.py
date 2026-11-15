"""Read the BASS SPICE video overlay (FUEL, AIR, FAN, RAD, date, clock) from strips saved by measure.py.

The overlay is a fixed character grid (pitch 28.67 px, first label box at 155 px + the frame's shift dx).
Each cell is matched by normalised cross-correlation (shift search +-2 px, +-1 row) against glyph
templates in overlay_glyphs.npz. The digit templates were built from the four PSI-25 train videos only:
the seconds digit of the clock was labelled by its own count (the clock advances one second per
second), using strips at least 0.15 s from a tick. A cell whose best correlation is below MIN_R is
unread ('?'). A blank cell (low contrast) is a space.
"""
import re
from pathlib import Path

import cv2
import numpy as np

PITCH = 28.67
FIRST = 155.0
ROWS = {"val": (23, 41), "date": (42, 60)}
MIN_R = 0.75
VPAD = 3                  # the shifted capture state also moves the overlay down by about 2 rows
BLANK_STD = 9.0
_NAMES = {"sp": " ", "sl": "/", "mi": "-", "co": ":", "dt": "."}


def _norm(c):
    c = c.astype(np.float32)
    c = c - c.mean()
    return c / (np.sqrt((c * c).sum()) + 1e-6)


def load_templates(path=None):
    z = np.load(path or Path(__file__).with_name("overlay_glyphs.npz"))
    out = {}
    for k in z.files:
        ch = _NAMES.get(k, k[1:] if k.startswith("d") else k)
        if ch == " ":
            continue
        out[ch] = z[k].astype(np.float32)
    return out


TEMPL = load_templates()


def cell(strip, dx, row, k, a=1.0):
    y0, y1 = ROWS[row]
    xf = FIRST + dx + a * PITCH * k
    x = int(round(xf))
    w = int(round(28 * a))
    if x < 0 or x >= strip.shape[1] - 4:
        return None, 0
    c = strip[max(0, y0 - VPAD):y1 + VPAD, x:x + w].astype(np.float32)
    if c.shape[0] != 18 + 2 * VPAD:
        return None, 0
    vis = c.shape[1]
    if abs(a - 1.0) > 0.004:
        c = cv2.resize(c, (max(1, int(round(vis / a))), c.shape[0]), interpolation=cv2.INTER_LINEAR)
    return c, min(28, c.shape[1])


GLYPH_SX = (1.0, 0.93, 0.86)   # glyph widths: the shifted capture state draws narrower glyphs (T16)


def _variants():
    out = []
    for ch, t in TEMPL.items():
        for sx in GLYPH_SX:
            w = int(round(28 * sx))
            tt = t if w == 28 else cv2.resize(t, (w, 18), interpolation=cv2.INTER_AREA)
            out.append((ch, sx, tt))
    return out


VARIANTS = _variants()


def read_cell(c, width):
    if c is None or width < 6:
        return "?", 0.0
    if c[VPAD + 3:VPAD + 15, 5:min(width, 24)].std() < BLANK_STD:
        return " ", 1.0
    cp = cv2.copyMakeBorder(c[:, :width], 0, 0, 3, 3, cv2.BORDER_REPLICATE).astype(np.float32)
    best, bk = -1.0, "?"
    for ch, sx, t in VARIANTS:
        tw = min(t.shape[1], width) if sx == 1.0 else t.shape[1]
        if tw > width:
            continue
        r = float(cv2.matchTemplate(cp, np.ascontiguousarray(t[:, :tw]), cv2.TM_CCOEFF_NORMED).max())
        if r > best:
            best, bk = r, ch
    return (bk if best >= MIN_R else "?"), best


def read_strip(strip, dx, a=1.0):
    def txt(row, ks):
        out, rs = "", []
        for k in ks:
            c, w = cell(strip, dx, row, k, a)
            ch, r = read_cell(c, w)
            out += ch
            rs.append(r)
        return out, min(rs) if rs else 0.0
    fuel, _ = txt("val", range(0, 4))
    air, _ = txt("val", range(6, 9))
    fan, _ = txt("val", range(11, 14))
    rad, _ = txt("val", range(16, 20))
    clock, _ = txt("date", range(9, 17))
    date, _ = txt("date", range(0, 8))
    return dict(fuel=fuel, air=air, fan=fan, rad=rad, clock=clock, date=date)


def _num(s, kind):
    s = s.strip()
    if not s or "?" in s:
        return None
    try:
        return int(s) if kind == "int" else float(s)
    except ValueError:
        return None


def series(npz_path):
    """Overlay readings, one per strip (about 1 s), with numbers parsed where the cells read cleanly."""
    z = np.load(npz_path)
    out = []
    aa = z["a"] if "a" in z.files else np.ones(len(z["t"]))
    for t, dx, a, has, s in zip(z["t"], z["dx"], aa, z["has"], z["strips"]):
        if not has:
            continue
        r = read_strip(s, float(dx), float(a))
        clk = r["clock"].replace(" ", "")
        m = re.fullmatch(r"(\d\d):(\d\d):(\d\d)", clk)
        rt = r["rad"]
        rad_v, rad_last = None, None
        if len(rt) == 4 and rt[0].isdigit() and rt[2].isdigit() and rt[1] in ". ":
            rad_v = float(rt[0] + "." + rt[2] + (rt[3] if rt[3].isdigit() else ""))
            rad_last = rt[3].isdigit()
        out.append(dict(video_t=float(t), fan=_num(r["fan"], "int"), air=_num(r["air"], "int"), rad=rad_v, rad_last_digit_read=rad_last,
                        rad_text=r["rad"], fuel=r["fuel"].strip(), clock=clk if m else None, date=r["date"],
                        clock_s=(int(m.group(1)) * 3600 + int(m.group(2)) * 60 + int(m.group(3))) if m else None))
    return out
