"""Screen a droplet_diameter.py output for a truncated track (the frozen tool's track-hijack failure).

Found on the first FLEX-2 tests: after a short interruption (flash, soot burst) the frozen tool can hand
its track to a small nearby object, then rejects the real droplet as 'lost' for the rest of the test
(its size rule forbids taking back an object larger than the last tracked one). The written series is
then the droplet's but stops early, which a scorer would read as the end of the run.

Screen (classification only; it changes no value and no usable frame): after the last usable frame,
if >= 10 'lost' frames within 2 s have a diameter between 0.8 and 1.1 of the last usable diameter and lie
within reach of its last position (2 D + 10 mm/s x elapsed time), the droplet was still in view and the
series is TRUNCATED.

A second screen catches the other face of the same failure, a HIJACKED series: the track jumps to
another object (a soot clump, an igniter fragment) that is then reported as usable. It flags a series
in which two consecutive usable frames less than 3 s apart differ in diameter by more than 30 %.

    python -I tools/wsp/media/flex2_truncation_screen.py <tool.csv> [...]
"""
import csv
import json
import math
import sys


def screen(path):
    rows = [r for r in csv.DictReader(open(path, encoding="utf-8")) if r["t_s"]]
    good = [r for r in rows if r["flag"] in ("ok", "soot")]
    if not good:
        return dict(truncated=False, n=0)
    pxmm = json.load(open(path[:-4] + ".json", encoding="utf-8"))["px_per_mm"]
    last = good[-1]
    t1, d1 = float(last["t_s"]), float(last["d_m"])
    x1, y1 = float(last["x_px"]), float(last["y_px"])

    def near(r):
        dt = float(r["t_s"]) - t1
        return math.hypot(float(r["x_px"]) - x1, float(r["y_px"]) - y1) <= 2 * d1 * 1e3 * pxmm + dt * 10 * pxmm
    n = sum(1 for r in rows if r["flag"] == "lost" and r["d_m"] and 0 < float(r["t_s"]) - t1 <= 2.0
            and 0.8 * d1 <= float(r["d_m"]) <= 1.1 * d1 and near(r))
    jumps = 0
    for a, b in zip(good, good[1:]):
        da, db = float(a["d_m"]), float(b["d_m"])
        if float(b["t_s"]) - float(a["t_s"]) < 3.0 and abs(db - da) > 0.3 * max(da, db):
            jumps += 1
    return dict(truncated=n >= 10, n=n, hijacked=jumps > 0, jumps=jumps)


if __name__ == "__main__":
    for p in sys.argv[1:]:
        print(p, screen(p))
