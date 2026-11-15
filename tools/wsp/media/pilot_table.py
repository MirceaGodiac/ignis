"""Collect the droplet pilot results (tool JSON + comparison JSON per test) into one table.

    python -I tools/wsp/media/pilot_table.py runs/dev/media/droplet  # writes comparison.csv / comparison.md
"""
import json
import sys
from pathlib import Path

import numpy as np

TESTS = [  # test point, FLEX number, fuel, NASA D0 (mm, first sample), note
    ("20CAL08", "FLEX-001", "methanol", "small, 2009, 512 px, 1/30 s shutter"),
    ("30CAL06", "FLEX-008", "n-heptane", "2009, 512 px, 1/30 s shutter, drifts out of view"),
    ("053F001", "FLEX-089", "methanol", "large, 2010, 1024 px, speckle"),
    ("H08M101", "FLEX-254", "methanol", "large, 2011, 1024 px, barely burns (K 0.02 mm2/s)"),
    ("H13H101", "FLEX-266", "n-heptane", "large, 2011, 1024 px"),
    ("C03H701", "FLEX-174", "n-heptane", "smallest, 2011, 1024 px, strong laser speckle"),
]


def main(root):
    root = Path(root)
    rows = []
    for tp, fx, fuel, note in TESTS:
        m = json.loads((root / "out" / f"{tp}.json").read_text(encoding="utf-8"))
        c = json.loads((root / "out" / f"{tp}_vs_nasa.json").read_text(encoding="utf-8"))
        rows.append(dict(test_point=tp, flex=fx, fuel=fuel, note=note, image_px=m["image_px"][0],
                         px_per_mm=round(m["px_per_mm"], 3), nasa_d0_mm=round(c["nasa_d0_mm"], 3),
                         offset_s=round(c["time_offset_s"], 3), offset_determined=c["time_offset_determined"],
                         offset_range_s=[round(x, 3) for x in c["time_offset_range_s"]],
                         clock_check_s=(m.get("clock_check") or {}).get("seen_minus_stamp_s"),
                         t_igniter_off_s=m.get("t_igniter_off_s"),
                         n_nasa=c["n_nasa"], n_matched=c["n_matched"],
                         bias_um=round(c["bias_um"], 1), rms_um=round(c["rms_um"], 1),
                         robust_sd_um=round(c["robust_sd_um"], 1), n_after_1s=c["n_matched_after_1s"],
                         bias_after_1s_um=round(c["bias_um_after_1s"], 1),
                         rms_after_1s_um=round(c["rms_um_after_1s"], 1),
                         k_nasa=round(c["k_nasa_mm2_s"], 4), k_tool=round(c["k_tool_mm2_s"], 4),
                         k_ratio=round(c["k_ratio"], 3), k_window_s=[round(x, 2) for x in c["k_window_s"]],
                         d_ratio=round(c["ratio_d_median"], 4), bad_share=round(c["bad_share_in_nasa_span"], 3),
                         flags_in_span=c["flags_in_span"], flag_counts=m["flag_counts"],
                         rod_mm=(m.get("ruler_check") or {}).get("rod_width_mm_at_scale"),
                         compute_s=m["compute_s"], compute_s_with_sheets=m.get("compute_s_with_sheets")))
    # pooled over tests: frames after 1 s (weights = matched frames)
    w = np.array([r["n_after_1s"] for r in rows], float)
    b = np.array([r["bias_after_1s_um"] for r in rows])
    s = np.array([r["rms_after_1s_um"] for r in rows])
    pooled = dict(bias_after_1s_um=float(np.sum(w * b) / w.sum()),
                  rms_after_1s_um=float(np.sqrt(np.sum(w * s ** 2) / w.sum())),
                  test_bias_sd_um=float(np.std(b, ddof=1)),
                  k_ratio_mean_burning=float(np.mean([r["k_ratio"] for r in rows if r["k_nasa"] > 0.1])),
                  k_ratio_sd_burning=float(np.std([r["k_ratio"] for r in rows if r["k_nasa"] > 0.1], ddof=1)))
    (root / "comparison.json").write_text(json.dumps(dict(tests=rows, pooled=pooled), indent=2) + "\n", encoding="utf-8")
    cols = ["test_point", "flex", "fuel", "image_px", "nasa_d0_mm", "n_nasa", "n_matched", "bias_um", "rms_um",
            "robust_sd_um", "bias_after_1s_um", "rms_after_1s_um", "k_nasa", "k_tool", "k_ratio", "offset_s",
            "offset_determined", "clock_check_s", "bad_share", "d_ratio", "rod_mm", "compute_s"]
    with open(root / "comparison.csv", "w", encoding="utf-8") as f:
        f.write(",".join(cols) + "\n")
        for r in rows:
            f.write(",".join(str(r[c]) for c in cols) + "\n")
    print(json.dumps(pooled, indent=1))
    for r in rows:
        print({c: r[c] for c in cols})


if __name__ == "__main__":
    main(sys.argv[1] if len(sys.argv) > 1 else "runs/dev/media/droplet")
