#!/usr/bin/env python3
"""Burning-rate constants of the TRAIN-DAY FLEX-2 series (v2, pinned scale) for the comparison with the
registered expectations of docs/wsp/research/phys_droplet_fuels.md section 4. Train days only; usable
series only; never reads a validation or test folder. Written after the score of PROTOCOL 12.7 (third
amendment).

Per test (usable frames only, D in mm, t in s on the series' time base):
  D0      median D over 0.5 to 1.0 s (after deployment, before most of the burn)
  K_hot   -slope of D^2 by least squares over [1.5 s, 1.5 s + 0.4 (t_last - 1.5 s)]
  K_late  -slope of D^2 over the last 40 % of the usable span
  K_10s   -slope of D^2 over [1.5, 10] s (the horizon the forecasts are scored on)
  D_last / D0, t_last
A cool flame after the hot one shows as K_late clearly below K_hot with the droplet still shrinking.

    python -I tools/wsp/ext/flex2_rates.py
"""
import glob
import json
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
SERIES = REPO / "runs/dev/media/droplet/flex2_series_v2/train"
CAT = REPO / "runs/dev/data_all/work/atlas/flex2_tests.jsonl"
OUT = REPO / "runs/dev/data_all/work/transfer_droplet/flex2_v2/rates_train.json"


def slope(pts):
    if len(pts) < 10:
        return None
    n = len(pts)
    mt = sum(t for t, _ in pts) / n
    mq = sum(q for _, q in pts) / n
    sxx = sum((t - mt) ** 2 for t, _ in pts)
    return -sum((t - mt) * (q - mq) for t, q in pts) / sxx if sxx > 0 else None


def main():
    cat = {json.loads(l)["test_id"]: json.loads(l) for l in CAT.read_text(encoding="utf-8").splitlines()}
    rows = {}
    for p in sorted(glob.glob(str(SERIES / "*.json"))):
        d = json.loads(Path(p).read_text(encoding="utf-8"))
        if d.get("quality") != "usable" or d["partition"] != "train":
            continue
        tid = d["run_id"].split(":")[1]
        pts = sorted((o["time_s"], o["value_si"] * 1e3) for o in d["observations"]
                     if o.get("value_si") and not o.get("quality_flags"))
        t_last = pts[-1][0]
        early = sorted(x for t, x in pts if 0.5 <= t <= 1.0)
        d0 = early[len(early) // 2] if early else None
        a, b = 1.5, 1.5 + 0.4 * (t_last - 1.5)
        rows[tid] = dict(fuel=cat[tid]["fuel"], day=d["session_id"], x_o2=d["conditions"]["x_o2"]["value"],
                         pressure_atm=round(d["conditions"]["pressure"]["value"] / 101325, 3),
                         D0_mm=d0, t_last_s=t_last, D_last_over_D0=pts[-1][1] / d0 if d0 else None,
                         K_hot=slope([(t, x * x) for t, x in pts if a <= t <= b]),
                         K_late=slope([(t, x * x) for t, x in pts if t >= t_last - 0.4 * (t_last - 1.5)]),
                         K_10s=slope([(t, x * x) for t, x in pts if 1.5 <= t <= 10.0]))
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(rows, indent=1) + "\n", encoding="utf-8")
    for k, v in rows.items():
        print(k, {x: (round(y, 3) if isinstance(y, float) else y) for x, y in v.items()})
    return 0


if __name__ == "__main__":
    sys.exit(main())
