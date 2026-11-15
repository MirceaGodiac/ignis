"""Build web/public/data/bass5_inputs.json: the bass5.wsp inputs of every BASS run, keyed by IGNIS run id.

The forecast lab fills the bass5 model's conditions from this table when a BASS run is picked. Every value
comes from the WSP lane's own derivation (tools/wsp/ext/law_inputs.py, bass5_row) over the lane's BASS case
*input* files (conditions only: outcome files are never opened). Runs the derivation does not cover are
listed under "not_covered" with the reason, never filled with a guess.

    python tools/web/bass5_inputs.py --lane .
"""
import argparse
import hashlib
import json
import sys
from pathlib import Path

FEATURES = ["x_o2", "flow_speed", "nose_radius", "heat_time", "is_concurrent", "is_rod", "flat_thickness",
            "u_pot1", "is_one_sided", "is_sibal", "is_nomex_ultem", "flat_width"]
FILES = ["cases_train.jsonl", "cases_validation_inputs.jsonl", "cases_test_inputs.jsonl"]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--lane", required=True, help="checkout of the WSP lane (holds tools/wsp/ext and runs/dev)")
    ap.add_argument("--out", default=str(Path(__file__).resolve().parents[2] / "web/public/data/bass5_inputs.json"))
    a = ap.parse_args()
    lane = Path(a.lane)
    sys.path[:0] = [str(lane / "tools/wsp/ext"), str(lane / "tools/wsp")]
    import law_inputs as L  # noqa: E402

    src = lane / "runs/dev/data_all/work/transfer_bass1"
    runs, missing, sources = {}, {}, []
    for name in FILES:
        if "outcome" in name:
            sys.exit("refusing an outcome file")
        p = src / name
        sources.append({"file": f"transfer_bass1/{name}", "sha256": hashlib.sha256(p.read_bytes()).hexdigest()})
        for c in L.read_inputs(p):
            cid = L.case_id(c)
            r, cov = L.bass5_row(c, "sustain")
            lacking = [f for f in FEATURES if r.get(f) is None]
            if cov not in ("nose", "flat") or lacking:
                missing[cid] = cov if cov not in ("nose", "flat") else "not recorded: " + ", ".join(lacking)
                continue
            runs[cid] = {"coverage": cov, "material": c.get("material_raw") or c.get("material_class"),
                         "inputs": {f: round(float(r[f]), 9) for f in FEATURES}}
    out = {"model": "bass5.wsp", "derivation": "tools/wsp/ext/law_inputs.py bass5_row (WSP lane), conditions only",
           "sources": sources, "runs": dict(sorted(runs.items())), "not_covered": dict(sorted(missing.items()))}
    Path(a.out).parent.mkdir(parents=True, exist_ok=True)
    Path(a.out).write_text(json.dumps(out, indent=1, ensure_ascii=False) + "\n", encoding="utf-8", newline="\n")
    print(f"{len(runs)} runs with every bass5 input, {len(missing)} not covered -> {a.out}")


if __name__ == "__main__":
    main()
