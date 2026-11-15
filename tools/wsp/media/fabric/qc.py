#!/usr/bin/env python3
"""Quality flags for the measured tracks: classes of burns on which the frozen method fails.

Written after the visual check of the first contact sheets, which were all unpartitioned PSI-25
(PMMA slabs and rods) and PSI-26 recordings; no held (validation/test) burn had been viewed. The rules
do not change any measured number: a track that meets one is marked `quality: failed` with the reason
in `qc_flags`, and its outcome is kept only as `outcome_unreliable`.

  R1 no igniter found                                  -> no time origin
  R2 more than one igniter-like burst                  -> t = 0 ambiguous (long records: several bright events
                                                          or several burns; bright flames counted as igniters)
  R3 igniter on for more than 30 s                     -> the "igniter" is a flame, not the coil (train: 5.6-9.4 s)
  R4 largest null-window score above 1000              -> the detection threshold is contaminated by flame or
                                                          lit transitions (train: 25-333)
  R6 1-cm SIBAL samples (added after the pooled agreement with the earlier validation tracks: frame-width tips
     agree, p90 0.011 fw, but widths differ by a median 2.8 w): tip_w/base_w removed, outcome not trusted
  R5 overlay registered in fewer than half of the 1-s strips after igniter off
                                                       -> no registration, no lit / off-scene screening, no fan
                                                          readings (all 640x360 PSI-26 recordings)

    python -I tools/wsp/media/fabric/qc.py --root runs/dev/media/fabric
"""
import argparse
import json
from collections import Counter
from pathlib import Path

import numpy as np

IGN_ON_MAX_S = 30.0
NULL_MAX = 1000.0
OVL_MIN = 0.5


def flags(d, ovl_path):
    out = []
    ig = d.get("igniter")
    if not ig:
        out.append("R1 no igniter found")
        return out
    nb = next((int(n.split()[0]) for n in d.get("notes", []) if "igniter bursts" in n), 1)
    if nb > 1:
        out.append(f"R2 {nb} igniter-like bursts")
    if ig["on_t_s"] is not None and -ig["on_t_s"] > IGN_ON_MAX_S:
        out.append(f"R3 igniter on for {-ig['on_t_s']:.0f} s")
    nm = (d.get("detection") or {}).get("null_max")
    if nm is not None and nm > NULL_MAX:
        out.append(f"R4 null-window score {nm:.0f}")
    if ovl_path.exists():
        z = np.load(ovl_path)
        sel = z["t"] >= z["t_off"][0]
        frac = float(z["has"][sel].mean()) if sel.any() else 0.0
    else:
        frac = 0.0
    if frac < OVL_MIN:
        out.append(f"R5 overlay registered in {frac:.0%} of the record after igniter off")
    mat = (d.get("material") or "").lower()
    if "sibal" in mat and "1 cm" in mat:
        out.append("R6 1-cm SIBAL: the frame-width track agrees with the earlier duct tracks, but the conversion to sample widths "
                   "and the sample end (set on 2-cm train holders) do not; widths removed, outcome not trusted")
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", required=True)
    args = ap.parse_args()
    root = Path(args.root)
    cnt = Counter()
    for f in sorted((root / "tracks").rglob("*.json")):
        d = json.loads(f.read_text(encoding="utf-8"))
        inv = "psi25" if "psi25" in f.parts else "psi26"
        fl = flags(d, root / "ovl" / f"{inv}_{f.stem}.npz")
        d["qc_flags"] = fl
        if any(x.startswith("R6") for x in fl):
            for p_ in d.get("duct") or []:
                p_.pop("tip_w", None)
                p_.pop("base_w", None)
            d["width_scale_w_per_fw"] = None
        if fl:
            d["quality"] = "failed"
            d["outcome_unreliable"] = d.get("outcome_unreliable") or d["outcome"]
            d["outcome"] = {"kind": "cannot_tell", "confidence": "low", "reason": "quality failed: " + "; ".join(fl)}
        f.write_text(json.dumps(d, indent=1), encoding="utf-8")
        for x in fl:
            cnt[(inv, x.split()[0])] += 1
        cnt[(inv, "tracks")] += 1
    print(dict(cnt))


if __name__ == "__main__":
    main()
