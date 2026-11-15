#!/usr/bin/env python3
"""Builds docs/wsp/overview.html: a self-contained visual summary of the literal models.

Everything on the page is read at build time from the committed releases and the development runs:
forecasts of three TRAIN runs (`ignis infer forecast`), the aggregate validation tables written by
`tools/wsp/dev_eval.py` (runs/dev/cmp_validation_*.json), and the train feedback
(runs/dev/fb_train_*.json). No test-partition run is read, and nothing here writes a parameter.

    python tools/wsp/overview.py --out docs/wsp/overview.html
"""
import argparse
import html
import json
import math
import re
import subprocess
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent / "analysis"))
import lib  # noqa: E402

ROOT = lib.ROOT
RAW = ROOT / "data/raw/samples/PSI-69"
LEVELS = [0.05, 0.25, 0.5, 0.75, 0.95]


def exe(path):
    p = ROOT / path
    if not p.exists() and p.with_suffix(".exe").exists():
        p = p.with_suffix(".exe")
    return str(p)


def rid(d):
    return json.loads((ROOT / d / "release.json").read_text())["id"]


def series(split, run):
    """Raw droplet series (t s, D mm) of a TRAIN run."""
    ids = lib.partition_ids(split, "train")
    assert run in ids, f"{run} is not a train run of {split}"
    rec = lib.experiments(split, [run])[run]
    keys = {a["name"] for a in rec["aliases"]}
    name = next(p.name for p in sorted(RAW.iterdir())
                if p.name.endswith("_DropletData.txt") and any(("_" + k + "_") in p.name for k in keys))
    out = []
    for line in (RAW / name).read_text(encoding="utf-8", errors="replace").splitlines()[1:]:
        p = line.split("\t")
        try:
            t, d = float(p[0]), float(p[1])
        except (ValueError, IndexError):
            continue
        if d > 0:
            out.append((t, d))
    return out


def forecast(release, run, cutoff, t_end, work):
    times = [round(cutoff + 0.25 * i, 2) for i in range(int((t_end - cutoff) / 0.25) + 1)]
    out = work / f"{release[7:15]}_{run.split(':')[1]}_{cutoff:g}.json"
    settings = json.dumps({"times_s": times, "quantile_levels": LEVELS})
    r = subprocess.run([exe("target/release/ignis"), "infer", "forecast", "--release", release, "--run", run,
                        "--cutoff", str(cutoff), "--settings", settings, "--out", str(out)],
                       capture_output=True, text=True, cwd=ROOT)
    if r.returncode != 0:
        sys.exit(f"forecast failed: {r.stderr[-800:]}{r.stdout[-400:]}")
    a = json.loads(out.read_text())
    q = {lv: [v * 1e3 for v in a["quantiles"][i]] for i, lv in enumerate(a["quantile_levels"])}
    sp = a["diagnostics"].get("state_probability")
    return {"t": a["times_s"], "q": q, "state": sp, "laws": (a["diagnostics"].get("encoder") or {}).get("laws")}


# ---------------------------------------------------------------------------------------------------
# SVG helpers. Marks follow the fixed specs: 2 px lines, area washes, hairline grid, bars <= 24 px.
# ---------------------------------------------------------------------------------------------------
def esc(s):
    return html.escape(str(s), quote=True)


def ticks(lo, hi, n=5):
    span = hi - lo
    step = 10 ** math.floor(math.log10(span / n))
    for m in (1, 2, 2.5, 5, 10):
        if span / (step * m) <= n:
            step *= m
            break
    first = math.ceil(lo / step - 1e-9) * step
    out = []
    v = first
    while v <= hi + 1e-9:
        out.append(round(v, 10))
        v += step
    return out


def fmt(v):
    return ("%g" % round(v, 3))


class Plot:
    def __init__(self, w, h, x0, x1, y0, y1, left=46, right=96, top=30, bottom=34):
        self.w, self.h = w, h
        self.x0, self.x1, self.y0, self.y1 = x0, x1, y0, y1
        self.l, self.r, self.t, self.b = left, right, top, bottom
        self.parts = []

    def px(self, x):
        return self.l + (x - self.x0) / (self.x1 - self.x0) * (self.w - self.l - self.r)

    def py(self, y):
        return self.h - self.b - (y - self.y0) / (self.y1 - self.y0) * (self.h - self.t - self.b)

    def axes(self, xlabel, ylabel, xt=None, yt=None):
        xt = xt or ticks(self.x0, self.x1, 6)
        yt = yt or ticks(self.y0, self.y1, 5)
        for v in yt:
            y = self.py(v)
            self.parts.append(f'<line class="grid" x1="{self.l}" x2="{self.w - self.r}" y1="{y:.1f}" y2="{y:.1f}"/>')
            self.parts.append(f'<text class="tick" x="{self.l - 6}" y="{y + 3.5:.1f}" text-anchor="end">{fmt(v)}</text>')
        for v in xt:
            x = self.px(v)
            self.parts.append(f'<text class="tick" x="{x:.1f}" y="{self.h - self.b + 14}" text-anchor="middle">{fmt(v)}</text>')
        self.parts.append(f'<line class="axis" x1="{self.l}" x2="{self.w - self.r}" y1="{self.h - self.b}" y2="{self.h - self.b}"/>')
        self.parts.append(f'<text class="axlabel" x="{(self.l + self.w - self.r) / 2:.0f}" y="{self.h - 2}" text-anchor="middle">{esc(xlabel)}</text>')
        self.parts.append(f'<text class="axlabel" x="{self.l - 38}" y="11" text-anchor="start">{esc(ylabel)}</text>')

    def path(self, pts):
        return "M" + " L".join(f"{self.px(x):.1f},{self.py(y):.1f}" for x, y in pts)

    def line(self, pts, cls):
        self.parts.append(f'<path class="{cls}" d="{self.path(pts)}"/>')

    def band(self, t, lo, hi, cls):
        up = [(a, b) for a, b in zip(t, hi)]
        dn = [(a, b) for a, b in zip(t, lo)][::-1]
        self.parts.append(f'<path class="{cls}" d="{self.path(up + dn)} Z"/>')

    def label(self, x, y, text, dy=4, cls="direct"):
        self.parts.append(f'<text class="{cls}" x="{self.px(x) + 8:.1f}" y="{self.py(y) + dy:.1f}">{esc(text)}</text>')

    def vline(self, x, text):
        self.parts.append(f'<line class="mark" x1="{self.px(x):.1f}" x2="{self.px(x):.1f}" y1="{self.t}" y2="{self.h - self.b}"/>')
        self.parts.append(f'<text class="note" x="{self.px(x) + 5:.1f}" y="{self.t - 5}">{esc(text)}</text>')

    def svg(self, title, data=None, extra=""):
        d = f" data-chart='{esc(json.dumps(data))}'" if data else ""
        return (f'<svg viewBox="0 0 {self.w} {self.h}" role="img" aria-label="{esc(title)}"{d}{extra}>'
                + "".join(self.parts) + "</svg>")


def forecast_figure(obs, cutoff, t_end, fc, base, title, y_range, labels):
    """Observed diameter, forecast median with 50 % and 90 % bands, and the D2-law median."""
    y0, y1 = y_range
    p = Plot(760, 320, 0, t_end, y0, y1)
    p.axes("time since igniter-on, s", "droplet diameter, mm")
    t = fc["t"]
    p.band(t, fc["q"][0.05], fc["q"][0.95], "band90")
    p.band(t, fc["q"][0.25], fc["q"][0.75], "band50")
    seen = [(a, b) for a, b in obs if 0 <= a <= cutoff]
    later = [(a, b) for a, b in obs if cutoff < a <= t_end]
    p.line(seen, "obs seen")
    p.line(later, "obs")
    if base:
        p.line(list(zip(base["t"], base["q"][0.5])), "s2")
    p.line(list(zip(t, fc["q"][0.5])), "s1")
    p.vline(cutoff, f"cutoff {cutoff:g} s: the model has seen only what is left of this line")
    ends = [(later[-1][1] if later else seen[-1][1], labels[0]), (fc["q"][0.5][-1], labels[1])]
    if base:
        ends.append((base["q"][0.5][-1], labels[2]))
    # direct labels at the right edge, spread so they never overlap
    ends.sort(key=lambda e: -e[0])
    ys = [p.py(e[0]) for e in ends]
    for i in range(1, len(ys)):
        ys[i] = max(ys[i], ys[i - 1] + 13)
    for (v, text), y in zip(ends, ys):
        p.parts.append(f'<text class="direct" x="{p.px(t_end) + 8:.1f}" y="{y + 4:.1f}">{esc(text)}</text>')

    def at(tt):
        best = min(range(len(later)), key=lambda i: abs(later[i][0] - tt)) if later else None
        return later[best][1] if best is not None and abs(later[best][0] - tt) < 0.3 else None

    data = {"kind": "x", "l": p.l, "r": p.w - p.r, "x0": 0, "x1": t_end, "t": t, "unit": " mm", "xunit": " s",
            "rows": [{"name": labels[1], "key": "s1", "v": [round(v, 3) for v in fc["q"][0.5]]},
                     {"name": "90 % band", "key": "b", "v": [f"{a:.3f} – {b:.3f}" for a, b in zip(fc["q"][0.05], fc["q"][0.95])]},
                     {"name": labels[0], "key": "o", "v": [None if at(tt) is None else round(at(tt), 3) for tt in t]}]
            + ([{"name": labels[2], "key": "s2", "v": [round(v, 3) for v in base["q"][0.5]]}] if base else [])}
    return p.svg(title, data)


def front_forecast(release, track, cutoff, until, work):
    """One forecast of a bass_front.wsp release on a TRAIN track (wsp-dev refuses any other partition)."""
    out = work / f"front_{release[7:15]}_{Path(track).stem}_{cutoff:g}.json"
    r = subprocess.run([exe("target/release/wsp-dev"), "front-forecast", "--release", release, "--track", track,
                        "--cutoff", str(cutoff), "--until", str(until), "--step", "1", "--out", str(out)],
                       capture_output=True, text=True, cwd=ROOT)
    if r.returncode != 0:
        sys.exit(f"front-forecast failed: {r.stderr[-800:]}{r.stdout[-400:]}")
    return json.loads(out.read_text())


def front_figure(fc, base, t_end, title, labels, step_at, step_text):
    """Measured tip of the flame, forecast median with 50 % and 90 % bands, persistence median."""
    cutoff = fc["cutoff_s"]
    p = Plot(760, 320, 0, t_end, 0.5, 4.75)
    p.axes("time since the igniter was switched off, s", "tip of the flame, sample widths from the igniter")
    t, q = fc["times_s"], fc["tip_quantiles_w"]
    p.band(t, q[0], q[4], "band90")
    p.band(t, q[1], q[3], "band50")
    stills = [(a, b) for a, b, _ in fc["stills_t_tip_base"] if a <= t_end]
    seen = [x for x in stills if x[0] <= cutoff]
    later = [x for x in stills if x[0] > cutoff]
    p.line(seen, "obs seen")
    p.line(later, "obs")
    p.line(list(zip(base["times_s"], base["tip_quantiles_w"][2])), "s2")
    p.line(list(zip(t, q[2])), "s1")
    p.vline(cutoff, f"cutoff {cutoff:g} s")
    x = p.px(step_at)
    p.parts.append(f'<line class="mark" x1="{x:.1f}" x2="{x:.1f}" y1="{p.t}" y2="{p.h - p.b}"/>')
    p.parts.append(f'<text class="note" x="{x + 5:.1f}" y="{p.h - p.b - 8}">{esc(step_text)}</text>')
    ends = [(later[-1][1], labels[0]), (q[2][-1], labels[1]), (min(base["tip_quantiles_w"][2][-1], 4.75), labels[2])]
    ends.sort(key=lambda e: -e[0])
    ys = [p.py(e[0]) for e in ends]
    for i in range(1, len(ys)):
        ys[i] = max(ys[i], ys[i - 1] + 13)
    for (v, text), y in zip(ends, ys):
        p.parts.append(f'<text class="direct" x="{p.px(t_end) + 8:.1f}" y="{y + 4:.1f}">{esc(text)}</text>')

    def at(tt):
        best = min(later, key=lambda s: abs(s[0] - tt)) if later else None
        return round(best[1], 2) if best and abs(best[0] - tt) < 0.6 else None

    data = {"kind": "x", "l": p.l, "r": p.w - p.r, "x0": 0, "x1": t_end, "t": t, "unit": " w", "xunit": " s",
            "rows": [{"name": labels[1], "key": "s1", "v": [round(v, 2) for v in q[2]]},
                     {"name": "90 % band", "key": "b", "v": [f"{a:.2f} – {b:.2f}" for a, b in zip(q[0], q[4])]},
                     {"name": labels[0], "key": "o", "v": [at(tt) for tt in t]},
                     {"name": labels[2], "key": "s2", "v": [round(v, 2) for v in base["tip_quantiles_w"][2]]}]}
    return p.svg(title, data)


def grouped_bars(groups, names, unit, title, digits):
    """Two bars per group, in group order. groups: (label, value of names[0], value of names[1])."""
    top = max(max(g[1], g[2]) for g in groups)
    w, bar_h, left = 760, 14, 230
    group_h = 2 * bar_h + 2 + 12
    h = 10 + group_h * len(groups)
    scale = (w - left - 70) / top
    out = []
    for i, (label, a, b) in enumerate(groups):
        y = 10 + i * group_h
        out.append(f'<text class="rowlabel" x="{left - 10}" y="{y + bar_h + 5}" text-anchor="end">{esc(label)}</text>')
        for j, (value, kind) in enumerate(((a, 1), (b, 3))):
            yy = y + j * (bar_h + 2)
            bw = max(6.0, value * scale)
            tip = f"{label}, {names[j]}: {value:.{digits}f}{unit}"
            out.append(f'<path class="bar k{kind}" tabindex="0" data-tip="{esc(tip)}" '
                       f'd="M{left},{yy} h{bw - 4:.1f} a4,4 0 0 1 4,4 v{bar_h - 8} a4,4 0 0 1 -4,4 h-{bw - 4:.1f} Z"/>')
            out.append(f'<text class="value" x="{left + bw + 6:.1f}" y="{yy + bar_h - 3}">{value:.{digits}f}</text>')
    out.append(f'<line class="axis" x1="{left}" x2="{left}" y1="6" y2="{h - 8}"/>')
    return f'<svg viewBox="0 0 {w} {h}" role="img" aria-label="{esc(title)}">' + "".join(out) + "</svg>"


def state_figure(fc, t_end, cutoff, title):
    p = Plot(760, 150, 0, t_end, 0, 1, top=24, bottom=34)
    p.axes("time since igniter-on, s", "probability", yt=[0, 0.5, 1])
    t = fc["t"]
    hot, cool, none = fc["state"]["hot"], fc["state"]["cool"], fc["state"]["no_flame"]
    lower = [0.0] * len(t)
    for vals, cls in ((hot, "f1"), (cool, "f2"), (none, "f3")):
        upper = [a + b for a, b in zip(lower, vals)]
        p.band(t, lower, upper, cls)
        lower = upper
    # the 2 px surface gap between stacked fills
    acc = [0.0] * len(t)
    for vals in (hot, cool):
        acc = [a + b for a, b in zip(acc, vals)]
        p.line(list(zip(t, acc)), "gap")
    p.vline(cutoff, "")
    data = {"kind": "x", "l": p.l, "r": p.w - p.r, "x0": 0, "x1": t_end, "t": t, "unit": "", "xunit": " s",
            "rows": [{"name": "hot flame", "key": "s1", "v": [round(v, 2) for v in hot]},
                     {"name": "cool flame", "key": "s2", "v": [round(v, 2) for v in cool]},
                     {"name": "no flame", "key": "s3", "v": [round(v, 2) for v in none]}]}
    return p.svg(title, data)


def bars(rows, unit, title, digits):
    """Horizontal bars, best at the top. rows: (label, value, kind 1..3, note)."""
    rows = sorted(rows, key=lambda r: r[1])
    top = max(r[1] for r in rows)
    w, row_h, left = 760, 30, 330
    h = 12 + row_h * len(rows) + 6
    scale = (w - left - 70) / top
    out = []
    for i, (label, value, kind, note) in enumerate(rows):
        y = 12 + i * row_h
        bw = max(2.0, value * scale)
        tip = f"{label}: {value:.{digits}f}{unit}" + (f" — {note}" if note else "")
        out.append(f'<text class="rowlabel" x="{left - 10}" y="{y + 15}" text-anchor="end">{esc(label)}</text>')
        out.append(f'<path class="bar k{kind}" tabindex="0" data-tip="{esc(tip)}" '
                   f'd="M{left},{y + 3} h{bw - 4:.1f} a4,4 0 0 1 4,4 v10 a4,4 0 0 1 -4,4 h-{bw - 4:.1f} Z"/>')
        out.append(f'<text class="value" x="{left + bw + 6:.1f}" y="{y + 15}">{value:.{digits}f}</text>')
    out.append(f'<line class="axis" x1="{left}" x2="{left}" y1="8" y2="{h - 4}"/>')
    return f'<svg viewBox="0 0 {w} {h}" role="img" aria-label="{esc(title)}">' + "".join(out) + "</svg>"


def scatter(points, title, xlabel, ylabel):
    """points: (x, y, tip). One series: no legend; the title names it."""
    p = Plot(370, 280, 0.4, 2.0, 0, 1.05, right=16)
    p.axes(xlabel, ylabel, xt=[0.5, 1.0, 1.5, 2.0], yt=[0, 0.25, 0.5, 0.75, 1.0])
    p.vline(1.0, "critical diameter")
    for x, y, tip in points:
        cx, cy = p.px(min(max(x, 0.4), 2.0)), p.py(y)
        p.parts.append(f'<circle class="dot" cx="{cx:.1f}" cy="{cy:.1f}" r="4.5"/>')
        p.parts.append(f'<circle class="hit" tabindex="0" data-tip="{esc(tip)}" cx="{cx:.1f}" cy="{cy:.1f}" r="12"/>')
    return p.svg(title)


def table(headers, rows):
    head = "".join(f"<th>{esc(h)}</th>" for h in headers)
    body = "".join("<tr>" + "".join(f"<td>{esc(c)}</td>" for c in r) + "</tr>" for r in rows)
    return f'<details><summary>Table view</summary><table><thead><tr>{head}</tr></thead><tbody>{body}</tbody></table></details>'


CSS = """
:root{color-scheme:light dark}
.viz-root{--plane:#f9f9f7;--surface-1:#fcfcfb;--ink:#0b0b0b;--ink2:#52514e;--muted:#8a8980;--hair:#e4e3df;
--series-1:#2a78d6;--series-2:#eb6834;--series-3:#1baf7a;--obs:#0b0b0b}
@media (prefers-color-scheme: dark){:root:where(:not([data-theme="light"])) .viz-root{--plane:#0d0d0d;--surface-1:#1a1a19;
--ink:#f0efec;--ink2:#c3c2b7;--muted:#8f8e86;--hair:#30302d;--series-1:#3987e5;--series-2:#d95926;--series-3:#199e70;--obs:#f0efec}}
:root[data-theme="dark"] .viz-root{--plane:#0d0d0d;--surface-1:#1a1a19;--ink:#f0efec;--ink2:#c3c2b7;--muted:#8f8e86;
--hair:#30302d;--series-1:#3987e5;--series-2:#d95926;--series-3:#199e70;--obs:#f0efec}
html,body{margin:0}
.viz-root{background:var(--plane);color:var(--ink);font:15px/1.55 system-ui,-apple-system,"Segoe UI",Roboto,sans-serif;padding:32px 16px 64px}
.wrap{max-width:820px;margin:0 auto}
h1{font-size:30px;line-height:1.2;margin:0 0 10px;font-weight:650;letter-spacing:-.01em;text-wrap:balance}
h2{font-size:20px;margin:44px 0 6px;font-weight:650;text-wrap:balance}
p{margin:8px 0;color:var(--ink2);max-width:68ch}
p.lead{color:var(--ink);font-size:17px}
.card{background:var(--surface-1);border:1px solid var(--hair);border-radius:10px;padding:16px 16px 10px;margin:14px 0}
.card h3{margin:0 0 2px;font-size:15px;font-weight:650}
.card .sub{margin:0 0 8px;font-size:13px;color:var(--ink2)}
.tiles{display:grid;grid-template-columns:repeat(auto-fit,minmax(138px,1fr));gap:12px;margin:18px 0}
.tile{background:var(--surface-1);border:1px solid var(--hair);border-radius:10px;padding:14px}
.tile .label{font-size:13px;color:var(--ink2)}
.tile .value{font-size:26px;font-weight:650;line-height:1.2;margin:2px 0;white-space:nowrap}
.tile .delta{font-size:13px;color:var(--ink2)}
svg{width:100%;height:auto;display:block;overflow:visible}
.grid{stroke:var(--hair);stroke-width:1}.axis{stroke:var(--muted);stroke-width:1}
.tick,.axlabel,.note{font-size:11px;fill:var(--ink2)}.note{fill:var(--muted)}
.direct,.rowlabel,.value{font-size:12px;fill:var(--ink)}.rowlabel{fill:var(--ink2)}
.mark{stroke:var(--muted);stroke-width:1}
.s1,.s2,.obs,.gap{fill:none;stroke-width:2;stroke-linejoin:round;stroke-linecap:round}
.s1{stroke:var(--series-1)}.s2{stroke:var(--series-2)}.obs{stroke:var(--obs)}.obs.seen{stroke:var(--muted)}
.gap{stroke:var(--surface-1)}
.band90{fill:var(--series-1);opacity:.10}.band50{fill:var(--series-1);opacity:.14}
.f1{fill:var(--series-1)}.f2{fill:var(--series-2)}.f3{fill:var(--series-3)}
.bar.k1{fill:var(--series-1)}.bar.k2{fill:var(--series-2)}.bar.k3{fill:var(--series-3)}
.bar:hover,.bar:focus{filter:brightness(1.15);outline:none}
.dot{fill:var(--series-1);stroke:var(--surface-1);stroke-width:2}.hit{fill:transparent;outline:none}
.legend{display:flex;flex-wrap:wrap;gap:6px 18px;font-size:13px;color:var(--ink2);margin:2px 0 8px}
.legend i{display:inline-block;width:14px;height:10px;border-radius:2px;margin-right:6px;vertical-align:-1px}
.legend i.ln{height:2px;vertical-align:3px;border-radius:1px}
.duo{display:grid;grid-template-columns:repeat(auto-fit,minmax(300px,1fr));gap:12px}
details{margin:6px 0 4px;font-size:13px;color:var(--ink2)}summary{cursor:pointer}
table{border-collapse:collapse;margin:8px 0;font-size:12.5px;font-variant-numeric:tabular-nums;display:block;overflow-x:auto}
th,td{text-align:left;padding:3px 12px 3px 0;border-bottom:1px solid var(--hair);white-space:nowrap}
#tip{position:fixed;pointer-events:none;background:var(--surface-1);color:var(--ink);border:1px solid var(--hair);border-radius:8px;
padding:8px 10px;font-size:12.5px;box-shadow:0 4px 18px rgba(0,0,0,.14);display:none;max-width:320px;z-index:5}
#tip b{font-weight:650}#tip .k{display:inline-block;width:12px;height:2px;margin-right:6px;vertical-align:3px;background:var(--muted)}
#tip .k.s1{background:var(--series-1)}#tip .k.s2{background:var(--series-2)}#tip .k.s3{background:var(--series-3)}#tip .k.o{background:var(--obs)}
.cross{stroke:var(--muted);stroke-width:1;pointer-events:none}
.fine{font-size:13px;color:var(--ink2)}code{font-size:12.5px}
"""

JS = """
const tip=document.getElementById('tip');
function place(e){const w=tip.offsetWidth,h=tip.offsetHeight;let x=e.clientX+14,y=e.clientY+14;
 if(x+w>innerWidth-8)x=e.clientX-w-14;if(y+h>innerHeight-8)y=e.clientY-h-14;tip.style.left=x+'px';tip.style.top=y+'px';}
function row(key,name,val){const d=document.createElement('div');const k=document.createElement('span');k.className='k '+key;
 const b=document.createElement('b');b.textContent=val;d.append(k,b,document.createTextNode(' '+name));return d;}
document.querySelectorAll('[data-tip]').forEach(el=>{
 const show=e=>{tip.replaceChildren(document.createTextNode(el.dataset.tip));tip.style.display='block';
  if(e.clientX!==undefined)place(e);else{const r=el.getBoundingClientRect();place({clientX:r.right,clientY:r.top});}};
 el.addEventListener('pointermove',show);el.addEventListener('focus',show);
 el.addEventListener('pointerleave',()=>tip.style.display='none');el.addEventListener('blur',()=>tip.style.display='none');});
document.querySelectorAll('svg[data-chart]').forEach(svg=>{
 const c=JSON.parse(svg.dataset.chart);const vb=svg.viewBox.baseVal;
 const line=document.createElementNS('http://www.w3.org/2000/svg','line');line.setAttribute('class','cross');
 line.setAttribute('y1',30);line.setAttribute('y2',vb.height-34);line.style.display='none';svg.append(line);
 svg.addEventListener('pointermove',e=>{const r=svg.getBoundingClientRect();const x=(e.clientX-r.left)/r.width*vb.width;
  if(x<c.l||x>c.r){line.style.display='none';tip.style.display='none';return;}
  const tx=c.x0+(x-c.l)/(c.r-c.l)*(c.x1-c.x0);let i=0,best=1e9;c.t.forEach((t,j)=>{const d=Math.abs(t-tx);if(d<best){best=d;i=j;}});
  if(best>0.6){line.style.display='none';tip.style.display='none';return;}
  const px=c.l+(c.t[i]-c.x0)/(c.x1-c.x0)*(c.r-c.l);line.setAttribute('x1',px);line.setAttribute('x2',px);line.style.display='';
  const head=document.createElement('div');head.textContent='t = '+c.t[i]+c.xunit;head.style.marginBottom='4px';
  const rows=c.rows.filter(s=>s.v[i]!==null&&s.v[i]!==undefined).map(s=>row(s.key,s.name,s.v[i]+(typeof s.v[i]==='number'?c.unit:'')));
  tip.replaceChildren(head,...rows);tip.style.display='block';place(e);});
 svg.addEventListener('pointerleave',()=>{line.style.display='none';tip.style.display='none';});});
"""


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", default="docs/wsp/overview.html")
    ap.add_argument("--methanol", default="configs/models/flex/wsp2/methanol/d3-fable")
    ap.add_argument("--heptane", default="configs/models/flex/wsp2/heptane/d2-fable")
    ap.add_argument("--normal", default="configs/models/acme/wsp4/normal/d1-fable")
    ap.add_argument("--inverse", default="configs/models/acme/wsp3/inverse/d1-fable")
    ap.add_argument("--bass", default="configs/models/bass/front/d1-fable")
    args = ap.parse_args()
    work = ROOT / "runs/dev/overview"
    work.mkdir(parents=True, exist_ok=True)
    meth, hept = rid(args.methanol), rid(args.heptane)
    hept_w0 = rid("configs/models/flex/wsp/heptane/w0-fable")
    hept_base = rid(str(Path(args.heptane).parent / "baseline-d2"))
    meth_base = rid(str(Path(args.methanol).parent / "baseline-d2"))
    name = lambda d: Path(d).name.split("-")[0].upper()

    # ---- figures from train runs
    h_run, h_cut, h_end = "PSI-69:N02H101:2011-08-29", 3.0, 30.0
    h_obs = series("flex_heptane", h_run)
    h_fc = forecast(hept, h_run, h_cut, h_end, work)
    h_w0 = forecast(hept_w0, h_run, h_cut, h_end, work)
    h_bl = forecast(hept_base, h_run, h_cut, h_end, work)
    m1_run, m1_cut, m1_end = "PSI-69:175R001:2010-03-29", 3.0, 20.0
    m1_obs = series("flex", m1_run)
    m1_fc = forecast(meth, m1_run, m1_cut, m1_end, work)
    m1_bl = forecast(meth_base, m1_run, m1_cut, m1_end, work)
    m2_run, m2_cut, m2_end = "PSI-69:N01M101:2011-08-24", 5.0, 24.0
    m2_obs = series("flex", m2_run)
    m2_fc = forecast(meth, m2_run, m2_cut, m2_end, work)
    m2_bl = forecast(meth_base, m2_run, m2_cut, m2_end, work)

    # ---- validation aggregates
    def cmp(key):
        return json.loads((ROOT / f"runs/dev/cmp_validation_{key}.json").read_text())

    base = json.loads((ROOT / "runs/dev/baselines.json").read_text())
    flex_cmp, hep_cmp, an_cmp, ai_cmp = cmp("flex"), cmp("flex_heptane"), cmp("acme_normal"), cmp("acme_inverse")
    crps = lambda c, r: c["per_release"][r]["metrics"]["crps"]["estimate"] * 1e6
    ibs = lambda c, r, cause: c["per_release"][r]["events"]["causes"][cause]["integrated_brier"]
    w0, w1, w2 = (rid(f"configs/models/flex/wsp/{k}-fable") for k in ("w0", "w1", "w2"))
    meth_d1 = rid("configs/models/flex/wsp/methanol/d1-fable")
    meth_d2 = rid("configs/models/flex/wsp/methanol/d2-fable")
    flex_rows = [(f"flex_methanol.wsp2 {name(args.methanol)} — third generation", crps(flex_cmp, meth), 1, "data-informed; chosen on train before this score existed"),
                 ("flex_methanol.wsp D2 — second generation", crps(flex_cmp, meth_d2), 1, "data-informed; spread widened after one aggregate validation look"),
                 ("flex.wsp W2 — phase 1, tuned on validation itself", crps(flex_cmp, w2), 2, "its author read validation run by run"),
                 ("flex.wsp W0 — phase 1, zero-shot", crps(flex_cmp, w0), 2, "written before any run"),
                 ("flex.v3 — fitted physics-neural model", crps(flex_cmp, base["flex"]["flex.v3 (MAP)"]), 3, "fitted on train"),
                 ("flex.d2_prefix — fitted D²-law", crps(flex_cmp, base["flex"]["flex.d2_prefix"]), 3, "fitted on train"),
                 ("D²-law — hand-written reference", crps(flex_cmp, meth_base), 3, "knows nothing about extinction")]
    hept_d1 = rid("configs/models/flex/wsp/heptane/d1-fable")
    hep_rows = [(f"flex_heptane.wsp2 {name(args.heptane)} — third generation", crps(hep_cmp, hept), 1, "data-informed; chosen on train before this score existed"),
                ("flex_heptane.wsp D1 — second generation", crps(hep_cmp, hept_d1), 1, "data-informed"),
                ("flex_heptane.wsp W0 — literature only, zero-shot on heptane", crps(hep_cmp, hept_w0), 2, "frozen before any heptane outcome was opened"),
                ("D²-law — hand-written reference", crps(hep_cmp, hept_base), 3, "knows nothing about extinction or cool flames")]
    an, ai = rid(args.normal), rid(args.inverse)
    A = "configs/models/acme/"
    how_carried = lambda r: "carried" if r in (an, ai) else "not adopted"
    an_blend, an_d3, an_g = rid(A + "wsp4/normal/d1-fable"), rid(A + "wsp2/normal/d3-fable"), rid(A + "wsp3/normal/d1-fable")
    an_rows = [("acme_normal.wsp4 D1 — both laws together", ibs(an_cmp, an_blend, "extinction"), 1, f"equal weights, registered before this score existed; {how_carried(an_blend)}"),
               ("acme_normal.wsp2 D3 — hinge law", ibs(an_cmp, an_d3, "extinction"), 1, f"data-informed; third generation; {how_carried(an_d3)}"),
               ("acme_normal.wsp3 D1 — growth clock", ibs(an_cmp, an_g, "extinction"), 1, f"data-informed; published physics with corrections from train; {how_carried(an_g)}"),
               ("published theory alone", ibs(an_cmp, rid(A + "wsp3/normal/baseline-theory"), "extinction"), 3, "hand-written reference: no number from this campaign"),
               ("linear hazard — fitted", ibs(an_cmp, base["acme_normal"]["acme_normal.linear_baseline"], "extinction"), 3, "fitted on train"),
               ("acme_normal.hazard — fitted", ibs(an_cmp, base["acme_normal"]["acme_normal.hazard"], "extinction"), 3, "fitted on train"),
               ("constant hazard — fitted", ibs(an_cmp, base["acme_normal"]["acme_normal.constant_baseline"], "extinction"), 3, "fitted on train"),
               ("acme_normal.wsp W0 — phase 1, zero-shot", ibs(an_cmp, rid(A + "wsp/normal-w0-fable"), "extinction"), 2, "written before any run"),
               ("acme_normal.wsp W2 — phase 1", ibs(an_cmp, rid(A + "wsp/normal-w2-fable"), "extinction"), 2, "data-informed")]
    ai_blend, ai_d1, ai_g = rid(A + "wsp4/inverse/d1-fable"), rid(A + "wsp2/inverse/d1-fable"), rid(A + "wsp3/inverse/d1-fable")
    ai_rows = [("acme_inverse.wsp4 D1 — both laws together", ibs(ai_cmp, ai_blend, "extinction"), 1, f"equal weights, registered before this score existed; {how_carried(ai_blend)}"),
               ("acme_inverse.wsp3 D1 — heating law", ibs(ai_cmp, ai_g, "extinction"), 1, f"data-informed; fourth generation; {how_carried(ai_g)}"),
               ("acme_inverse.wsp2 D1 — physical clocks", ibs(ai_cmp, ai_d1, "extinction"), 1, "data-informed; second generation; carried until 14 November"),
               ("published theory alone", ibs(ai_cmp, rid(A + "wsp3/inverse/baseline-theory"), "extinction"), 3, "hand-written reference: it does not describe lean inverse flames"),
               ("acme_inverse.hazard — fitted", ibs(ai_cmp, base["acme_inverse"]["acme_inverse.hazard"], "extinction"), 3, "fitted on train"),
               ("linear hazard — fitted", ibs(ai_cmp, base["acme_inverse"]["acme_inverse.linear_baseline"], "extinction"), 3, "fitted on train"),
               ("constant hazard — fitted", ibs(ai_cmp, base["acme_inverse"]["acme_inverse.constant_baseline"], "extinction"), 3, "fitted on train"),
               ("acme_inverse.wsp W1 — phase 1", ibs(ai_cmp, rid(A + "wsp/inverse-w1-fable"), "extinction"), 2, "data-informed"),
               ("acme_inverse.wsp W0 — phase 1, zero-shot", ibs(ai_cmp, rid(A + "wsp/inverse-w0-fable"), "extinction"), 2, "written before any run")]
    an_val, ai_val = ibs(an_cmp, an, "extinction"), ibs(ai_cmp, ai, "extinction")

    # ---- fourth-generation droplet candidates against the carried numbers, on their own common cases
    F = "configs/models/flex/wsp3/"
    m4, h4 = cmp("flex_gen4"), cmp("flex_heptane_gen4")
    m4_rows = [("flex_methanol.wsp2 D3 — carried", crps(m4, rid(F + "methanol/carry-d3")), 1, "flex_methanol.wsp2 D3 on the fourth forward: identical forecasts"),
               ("flex_methanol.wsp3 D5 — tracker flags alone", crps(m4, rid(F + "methanol/d5-flags")), 2, "passed on train in 25 of 26 sessions; not adopted"),
               ("flex_methanol.wsp3 D4 — sharper, narrower, flags", crps(m4, rid(F + "methanol/d4-fable")), 2, "passed on train in 24 of 26 sessions; not adopted")]
    h4_rows = [("flex_heptane.wsp2 D2 — carried", crps(h4, rid(F + "heptane/carry-d2")), 1, "flex_heptane.wsp2 D2 on the fourth forward: identical forecasts"),
               ("flex_heptane.wsp3 D3 — helium law, narrower, flags", crps(h4, rid(F + "heptane/d3-fable")), 2, "passed on train in 18 of 20 sessions; not adopted")]

    # ---- BASS: aggregates of the blind validation, and one train burn
    bass, bass_p = rid(args.bass), rid("configs/models/bass/front/baseline-persistence")
    fv = json.loads((ROOT / "runs/dev/front_validation.json").read_text())
    fagg = {r["release_id"]: r["aggregate"] for r in fv["releases"]}
    b_over = lambda r, k: fagg[r]["overall"][k]["mean_over_runs"]
    b_cut = [(f"{float(k):g} s of flame seen", fagg[bass]["by_cutoff_s"][k]["crps"]["mean_over_runs"],
              fagg[bass_p]["by_cutoff_s"][k]["crps"]["mean_over_runs"])
             for k in sorted(fagg[bass]["by_cutoff_s"], key=float)]
    b_counts = fv["counts"]
    T5 = "tools/wsp/bass_optical/tracks/train/T5.json"
    b_fc = front_forecast(bass, T5, 5.0, 65.0, work)
    b_bl = front_forecast(bass_p, T5, 5.0, 65.0, work)
    b_step = next(t for t, u in b_fc["flow_cm_s"] if t > 0)
    b_u0 = b_fc["flow_cm_s"][0][1]
    b_u1 = [u for t, u in b_fc["flow_cm_s"] if t <= 30.0][-1]  # the flow after the ramp has settled

    # ---- regime scatter from train feedback (laws) and NASA's extinction diameters
    def regime(fb_path, prof_path, release, skip):
        fb = json.loads((ROOT / fb_path).read_text())
        prof = {r["run"]: r for r in json.loads((ROOT / prof_path).read_text())}
        rel = next(r for r in fb["releases"] if r["release_id"] == release)
        seen, pts = set(), []
        for c in rel["cases"]:
            run = c["run_id"]
            p = prof.get(run, {})
            if run in seen or not c.get("encoder") or not p.get("dext") or not p.get("d0") or any(s in run for s in skip):
                continue
            seen.add(run)
            ratio = math.exp(c["encoder"]["laws"]["x_exc"])
            y = min(p["dext"] / p["d0"], 1.05)
            pts.append((ratio, y, f"{run.split(':')[1]}: D0 {p['d0']:.2f} mm, {100 * p['o2']:.0f} % O2 — visible flame out at {p['dext']:.2f} mm ({y:.2f} of D0); D0 is {ratio:.2f} × the model's critical diameter"))
        return pts

    m_pts = regime("runs/dev/fb_train_flex.json", "runs/dev/analysis/flex_methanol_profile.json", meth,
                   ["240R003", "C12M401", "40CAL01", "H12M101", "256F002", "256R001", "192F001"])
    h_pts = regime("runs/dev/fb_train_flex_heptane.json", "runs/dev/analysis/flex_heptane_profile.json", hept,
                   ["195F00", "182F00", "196F001", "CAL", "248F001", "C01H101"])

    best_fit = lambda rows: min(r[1] for r in rows if "fitted" in r[0])
    tiles = [("FLEX methanol, validation CRPS", f"{crps(flex_cmp, meth):.1f} µm", f"best fitted model {best_fit(flex_rows):.1f} µm"),
             ("FLEX n-heptane, validation CRPS", f"{crps(hep_cmp, hept):.1f} µm", f"zero-shot W0 {crps(hep_cmp, hept_w0):.1f} µm · D²-law {crps(hep_cmp, hept_base):.1f} µm"),
             ("ACME normal, extinction Brier", f"{an_val:.3f}", f"best fitted baseline {best_fit(an_rows):.3f}"),
             ("ACME inverse, extinction Brier", f"{ai_val:.3f}", f"best fitted baseline {best_fit(ai_rows):.3f}"),
             ("BASS flame tip, validation CRPS", f"{b_over(bass, 'crps'):.3f} w", f"persistence {b_over(bass_p, 'crps'):.3f} sample widths")]

    legend_fc = ('<div class="legend"><span><i class="ln" style="background:var(--obs)"></i>measured by NASA (train run)</span>'
                 '<span><i class="ln" style="background:var(--series-1)"></i>hand-written model, median</span>'
                 '<span><i style="background:var(--series-1);opacity:.25"></i>its 50 % and 90 % bands</span>'
                 '<span><i class="ln" style="background:var(--series-2)"></i>D²-law reference</span></div>')
    legend_state = ('<div class="legend"><span><i style="background:var(--series-1)"></i>hot flame</span>'
                    '<span><i style="background:var(--series-2)"></i>cool flame</span>'
                    '<span><i style="background:var(--series-3)"></i>no flame</span></div>')
    legend_try = ('<div class="legend"><span><i style="background:var(--series-1)"></i>carried model</span>'
                  '<span><i style="background:var(--series-2)"></i>fourth-generation candidate, not adopted</span></div>')
    legend_pair = ('<div class="legend"><span><i style="background:var(--series-1)"></i>hand-written front model</span>'
                   '<span><i style="background:var(--series-3)"></i>persistence: the tip keeps the speed it showed</span></div>')
    legend_front = ('<div class="legend"><span><i class="ln" style="background:var(--obs)"></i>measured from the video stills (train burn)</span>'
                    '<span><i class="ln" style="background:var(--series-1)"></i>front model, median</span>'
                    '<span><i style="background:var(--series-1);opacity:.25"></i>its 50 % and 90 % bands</span>'
                    '<span><i class="ln" style="background:var(--series-2)"></i>persistence</span></div>')
    legend_bar = ('<div class="legend"><span><i style="background:var(--series-1)"></i>hand-written, phase 2 and later</span>'
                  '<span><i style="background:var(--series-2)"></i>hand-written, phase 1</span>'
                  '<span><i style="background:var(--series-3)"></i>fitted on train, or reference</span></div>')

    def fc_table(fc, obs):
        rows = []
        for i in range(0, len(fc["t"]), 8):
            t = fc["t"][i]
            o = min(obs, key=lambda p: abs(p[0] - t))
            rows.append((f"{t:g}", f"{fc['q'][0.5][i]:.3f}", f"{fc['q'][0.05][i]:.3f}", f"{fc['q'][0.95][i]:.3f}",
                         f"{o[1]:.3f}" if abs(o[0] - t) < 0.3 else "—"))
        return table(["t, s", "median, mm", "5 %", "95 %", "measured, mm"], rows)

    bar_table = lambda rows, unit: table(["release", "score" + unit, "how made"], [(r[0], f"{r[1]:.3f}" if r[1] < 5 else f"{r[1]:.1f}", r[3]) for r in sorted(rows, key=lambda r: r[1])])
    d_ce = h_fc["laws"]["d_ce_m"] * 1e3 if h_fc["laws"] else float("nan")
    page = f"""<!doctype html>
<html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>Hand-written combustion models</title><style>{CSS}</style></head>
<body class="viz-root"><div class="wrap">
<h1>Models of space-station flames whose every weight was typed by hand</h1>
<p class="lead">No training, no fitting, no search: an author model studied NASA's FLEX, ACME and BASS experiments, read the
train runs, and wrote each number with its reason. These are the development results, the refused candidates included.
The final test partition has not been read.</p>
<div class="tiles">{''.join(f'<div class="tile"><div class="label">{esc(a)}</div><div class="value">{esc(b)}</div><div class="delta">{esc(c)}</div></div>' for a, b, c in tiles)}</div>
<p class="fine">Lower is better everywhere. Validation runs were never used to write a law. Every model on this page was
committed before its validation score existed, and the score then decided only whether it replaced the one before it.</p>

<h2>A heptane droplet has three lives, and only the diameter shows the second</h2>
<p>A 3.3 mm n-heptane droplet in 16 % oxygen loses its visible flame to radiation within seconds. It then keeps burning
for twenty seconds in a cool flame that no camera on the experiment could see, until the curve plateaus near
{d_ce:.1f} mm. The model is given three seconds of data and the atmosphere.</p>
<div class="card"><h3>Train run N02H101: forecast from a 3 s cutoff</h3>
<p class="sub">Droplet diameter against time. The D²-law reference burns the droplet away; the three-state model slows it and stops it.</p>
{legend_fc}{forecast_figure(h_obs, h_cut, h_end, h_fc, h_bl, "Heptane forecast against a train run", (0.8, 3.6), ["measured", f"{name(args.heptane)} median", "D²-law"])}
{fc_table(h_fc, h_obs)}</div>
<div class="card"><h3>What the model believes the droplet is doing</h3>
<p class="sub">Probability of each state for the same forecast. Nothing in the data labels these stages.</p>
{legend_state}{state_figure(h_fc, h_end, h_cut, "State probabilities of the heptane forecast")}
{table(["t, s", "hot", "cool", "no flame"], [(f"{h_fc['t'][i]:g}", f"{h_fc['state']['hot'][i]:.2f}", f"{h_fc['state']['cool'][i]:.2f}", f"{h_fc['state']['no_flame'][i]:.2f}") for i in range(0, len(h_fc['t']), 8)])}</div>

<h2>Methanol flames die in two different ways, split by a critical diameter</h2>
<p>Below a critical size a methanol droplet burns until the water it has absorbed puts the flame out, at about half its
initial diameter whatever its size. Above it the flame is lost to radiation early. The first-generation model saw this
as scatter; the train data show a threshold.</p>
<div class="duo">
<div class="card"><h3>Methanol: where the visible flame went out</h3><p class="sub">Each dot is a train run. Horizontal: initial diameter over the model's critical diameter. Vertical: NASA's extinction diameter over the initial diameter.</p>
{scatter(m_pts, "Methanol extinction diameter against supercriticality", "D0 / critical diameter", "Dext / D0")}</div>
<div class="card"><h3>Heptane: the same threshold for the hot flame</h3><p class="sub">Above the line the visible flame is lost early and a cool flame takes over.</p>
{scatter(h_pts, "Heptane hot-flame extinction diameter against supercriticality", "D0 / critical diameter", "Dext / D0")}</div>
</div>
{table(["fuel", "run", "D0 / D_crit", "Dext / D0"], [("methanol", t.split(":")[0], f"{x:.2f}", f"{y:.2f}") for x, y, t in sorted(m_pts)] + [("heptane", t.split(":")[0], f"{x:.2f}", f"{y:.2f}") for x, y, t in sorted(h_pts)])}
<div class="card"><h3>Train run 175R001: a 3.3 mm droplet in 14 % oxygen, just above the critical diameter</h3>
<p class="sub">The flame is lost near 10 s and the droplet goes on evaporating slowly.</p>
{legend_fc}{forecast_figure(m1_obs, m1_cut, m1_end, m1_fc, m1_bl, "Methanol forecast, radiative branch", (2.2, 3.5), ["measured", f"{name(args.methanol)} median", "D²-law"])}
{fc_table(m1_fc, m1_obs)}</div>
<div class="card"><h3>Train run N01M101: a 3.3 mm droplet in air, far below the critical diameter</h3>
<p class="sub">It burns to a diffusive extinction near 19 s.</p>
{legend_fc}{forecast_figure(m2_obs, m2_cut, m2_end, m2_fc, m2_bl, "Methanol forecast, diffusive branch", (0.6, 3.5), ["measured", f"{name(args.methanol)} median", "D²-law"])}
{fc_table(m2_fc, m2_obs)}</div>

<h2>On runs the laws were not written from</h2>
<p>Validation partitions, scored by the repository's own evaluator. Most validation atmospheres and fuel–pressure
combinations do not occur in train.</p>
<div class="card"><h3>FLEX methanol — CRPS of the droplet diameter, µm</h3><p class="sub">56 forecast cases in 9 sessions.</p>{legend_bar}{bars(flex_rows, " µm", "Methanol validation CRPS", 1)}{bar_table(flex_rows, ", µm")}</div>
<div class="card"><h3>FLEX n-heptane — CRPS of the droplet diameter, µm</h3><p class="sub">39 forecast cases in 7 sessions. The fitted models of the repository refuse heptane.</p>{legend_bar}{bars(hep_rows, " µm", "Heptane validation CRPS", 1)}{bar_table(hep_rows, ", µm")}</div>
<div class="card"><h3>ACME normal flames — integrated Brier score of self-extinction</h3><p class="sub">{an_cmp['n_cases']} cases in 2 sessions; the margins between the top rows are within noise.</p>{legend_bar}{bars(an_rows, "", "ACME normal validation Brier", 3)}{bar_table(an_rows, "")}</div>
<div class="card"><h3>ACME inverse flames — integrated Brier score of self-extinction</h3><p class="sub">{ai_cmp['n_cases']} cases in 2 sessions.</p>{legend_bar}{bars(ai_rows, "", "ACME inverse validation Brier", 3)}{bar_table(ai_rows, "")}</div>

<h2>A flame on a strip of fabric, forecast under a fan schedule</h2>
<p>BASS burns a thin cotton and fibreglass fabric in a small wind tunnel. The flame is a band that travels with the
flow: a front that advances at a rate set by flow and oxygen, a visible tip ahead of it, and a base that follows one
burn-through time later. No run has a length calibration, so the model works in sample widths, on tracks measured from
the public videos in this project. The validation burns were measured by agents that had never seen the model.</p>
<div class="card"><h3>Train burn T5: forecast from 5 s after the igniter is switched off</h3>
<p class="sub">Tip of the flame along the sample. The operator steps the fan down at {b_step:.0f} s; the model is told the fan
schedule, not the outcome, and says the tip will fall back before it advances again. Persistence carries on at the old speed.</p>
{legend_front}{front_figure(b_fc, b_bl, 65.0, "BASS flame-tip forecast against a train burn", ["measured", "front model median", "persistence"], b_step, f"fan stepped down: {b_u0:.0f} to {b_u1:.1f} cm/s")}
{table(["t, s", "median, w", "5 %", "95 %", "persistence, w"], [(f"{b_fc['times_s'][i]:g}", f"{b_fc['tip_quantiles_w'][2][i]:.2f}", f"{b_fc['tip_quantiles_w'][0][i]:.2f}", f"{b_fc['tip_quantiles_w'][4][i]:.2f}", f"{b_bl['tip_quantiles_w'][2][i]:.2f}") for i in range(4, len(b_fc['times_s']), 5)])}</div>
<div class="card"><h3>BASS validation — CRPS of the flame tip, sample widths, by how much of the burn was seen</h3>
<p class="sub">{b_counts['cases']} forecast cases on {b_counts['runs_with_cases']} burns measured blind, at flows, oxygen levels and a sample width the train burns do not hold.
Over all cases: {b_over(bass, 'crps'):.3f} against {b_over(bass_p, 'crps'):.3f}; the 90 % band covers {b_over(bass, 'coverage_90'):.2f}.</p>
{legend_pair}{grouped_bars(b_cut, ["front model", "persistence"], " w", "BASS validation CRPS by cutoff", 3)}
{table(["prefix", "front model, w", "persistence, w"], [(g[0], f"{g[1]:.3f}", f"{g[2]:.3f}") for g in b_cut])}
<p class="sub">What failed: the quench boundary the model places in oxygen. On "will this sample carry a flame at all" it scores
{fagg[bass]['ignition_outcome']['brier_mean_over_tracks']:.3f} where answering "yes" every time scores {fagg[bass_p]['ignition_outcome']['brier_mean_over_tracks']:.3f}.</p></div>

<h2>What the fourth generation tried, and what validation refused</h2>
<p>On 14 November three more candidates for the droplet models passed the selection rule on the train partition. They
made a class boundary steeper and the bands narrower because the train runs looked that way, and added a wider band for
prefixes whose tracker looked unstable. None improved on unseen sessions, so the third-generation numbers stay.</p>
<div class="card"><h3>Methanol — CRPS, µm, same {m4['n_cases']} cases</h3>{legend_try}{bars(m4_rows, " µm", "Methanol fourth-generation candidates", 1)}{bar_table(m4_rows, ", µm")}</div>
<div class="card"><h3>n-Heptane — CRPS, µm, same {h4['n_cases']} cases</h3>{legend_try}{bars(h4_rows, " µm", "Heptane fourth-generation candidates", 1)}{bar_table(h4_rows, ", µm")}</div>
<p>For the burner flames the fourth generation wrote the published physics of a growing spherical flame and a heating
law for the burner. It replaced the inverse-flame model and tied the normal-flame one; the charts above show both next
to a release that lets the two laws answer together.</p>

<h2>What is zero-shot and what is not</h2>
<p>Only candidates named W0 were written before any outcome of their fuel or family was seen. The heptane W0 was
committed after the heptane evaluation groups were frozen from metadata and before any heptane series was opened; its
author had read the methanol train runs and the published literature. Every candidate named D1, D2 or D3 was typed after
its author read train data, and so was the BASS front model. Its numbers were typed from four train burns and the
published theory; no run has a length calibration, and the lowest fan settings are outside the fan calibration.</p>
<p class="fine">Carried releases: methanol <code>{meth[:23]}…</code>, heptane <code>{hept[:23]}…</code>, ACME normal
<code>{an[:23]}…</code>, ACME inverse <code>{ai[:23]}…</code>, BASS <code>{bass[:23]}…</code>. Built by <code>tools/wsp/overview.py</code>; details in
<code>docs/wsp/README.md</code>.</p>
</div><div id="tip" role="tooltip"></div><script>{JS}</script></body></html>"""
    out = ROOT / args.out
    out.write_text(page, encoding="utf-8", newline="\n")
    print("written", out, len(page))
    return 0


if __name__ == "__main__":
    sys.exit(main())
