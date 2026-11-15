"""Rewrite FLEX-2 series with the pinned pixel scale (SCALE_FLEX2.md); no frame is measured again.

Diameters were measured in pixels and converted with the assumed 20.48 px/mm (50 mm field of view);
here every value_si is multiplied by 20.48 / PXMM. The acceptance rule of FREEZE.json is scale-free
(clock check, frame counts, coverage) and is re-applied unchanged to the tool's per-frame flags; the
screen and visual labels are carried over. Nothing else changes.

    python -I tools/wsp/media/flex2_rescale.py --pxmm 33.85 --src flex2_series --dst flex2_series_v2 [--flat]
"""
import argparse
import json
from pathlib import Path

OUT = Path(__file__).resolve().parents[3] / "runs/dev/media/droplet"
ASSUMED = 20.48


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--pxmm", type=float, required=True)
    ap.add_argument("--pxmm-sd", type=float, required=True)
    ap.add_argument("--source", required=True)
    ap.add_argument("--src", required=True)
    ap.add_argument("--dst", required=True)
    a = ap.parse_args()
    f = ASSUMED / a.pxmm
    n = 0
    status = {}
    for p in sorted((OUT / a.src).rglob("*.json")):
        s = json.loads(p.read_text(encoding="utf-8"))
        for o in s.get("observations", []):
            o["value_si"] = o["value_si"] * f
        m = s.setdefault("measurement", {})
        m["px_per_mm_measured_with"] = m.get("px_per_mm", ASSUMED)
        m["px_per_mm"] = a.pxmm
        m["px_per_mm_sd"] = a.pxmm_sd
        m["scale_source"] = a.source
        m["rescaled_from_v1_factor"] = f
        s["series_version"] = "v2 (pinned scale, SCALE_FLEX2.md)"
        dst = OUT / a.dst / p.relative_to(OUT / a.src)
        dst.parent.mkdir(parents=True, exist_ok=True)
        dst.write_text(json.dumps(s, indent=1) + "\n", encoding="utf-8")
        n += 1
        status[s["quality"]] = status.get(s["quality"], 0) + 1
    print(json.dumps(dict(series=n, factor_on_diameter=f, status=status)))


if __name__ == "__main__":
    main()
