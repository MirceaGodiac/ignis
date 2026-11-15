#!/usr/bin/env python3
"""Builds the private data root runs/dev/data_ext2 as a copy of runs/dev/data_ext in which the condition
`programmed_stop` of every CFI-G (PSI-159) run that has a script parameter file is the value of
tools/wsp/ext/acme_script2.py (the planned stop on the forecast clock, v2 rule), in every partition.

It is an input computed from the pre-ignition parameter file alone. Only the `conditions.programmed_stop`
field of `work/records/acme_{normal,inverse}/experiments.jsonl` is changed; no other record and nothing of
data_ext is modified. A run whose file gives no v2 value (incomplete file, no constant-flow stage) has the
condition removed, so it is run without a planned stop (as acme_script.py did for a file without a value).
Prints counts only.

    python tools/wsp/ext/make_data_ext2.py
"""
import collections
import hashlib
import json
import shutil
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import acme_script2  # noqa: E402

REPO = Path(__file__).resolve().parents[3]
SRC, DST = REPO / "runs/dev/data_ext", REPO / "runs/dev/data_ext2"
CONDITION = "programmed_stop"


def partitions(family):
    """run id -> partition, from the split file of data_ext (counting only)."""
    p = next((SRC / "splits").glob(f"{family}-*.json"))
    s = json.loads(p.read_text(encoding="utf-8"))["split"]
    g = {x: part for part in ("train", "validation", "test") for x in s[part]}
    return {r: g.get(grp, "test") for r, grp in s["run_to_group"].items()}


def main():
    if DST.exists():
        sys.exit(f"{DST} exists; remove it first (this tool never overwrites)")
    shutil.copytree(SRC, DST)
    report = {}
    for family in ("acme_normal", "acme_inverse"):
        part = partitions(family)
        path = DST / "work/records" / family / "experiments.jsonl"
        out, n = [], collections.Counter()
        for line in path.read_text(encoding="utf-8").splitlines():
            if not line.strip():
                continue
            rec = json.loads(line)
            if rec["id"].startswith("PSI-159:"):
                test = rec["id"].split(":")[-1]
                p = part.get(rec["id"], "unknown")
                r = acme_script2.for_test("PSI-159", test)
                old = rec["conditions"].get(CONDITION)
                v = r.get("planned_stop_s")
                if r["branch"] == "no_script_file":
                    n[(p, "no script file (unchanged)")] += 1
                elif v is not None and v > 0:
                    sp = acme_script2.script_path("PSI-159", test)
                    rec["conditions"][CONDITION] = {
                        "derivation": {"agent": None, "inputs": [], "method": "deterministic",
                                       "method_id": acme_script2.METHOD_ID, "method_version": acme_script2.METHOD_VERSION,
                                       "notes": f"planned flow stop on the forecast clock (D + S + F - G, branch {r['branch']}) "
                                                "from the test's script parameter file alone (tools/wsp/ext/acme_script2.py); "
                                                "set before ignition"},
                        "evidence_class": "source_derived", "flags": list(r.get("flags", [])),
                        "locator": {"asset_sha256": hashlib.sha256(sp.read_bytes()).hexdigest(), "char_end": None,
                                    "char_start": None, "column": "script parameters", "frame": None, "page": None,
                                    "path": sp.name, "pts_s": None, "row": None},
                        "raw_unit": "s", "raw_value": v, "resolution": "resolved",
                        "uncertainty_reason": "not reported by source", "uncertainty_si": None, "unit_si": "s", "value_si": v}
                    n[(p, "new value" + (" (had old)" if old else " (had none)"))] += 1
                else:
                    if old:
                        rec["conditions"].pop(CONDITION)
                    n[(p, f"no v2 value ({r['branch']}), condition {'removed' if old else 'absent'}")] += 1
            out.append(json.dumps(rec, ensure_ascii=False))
        path.write_text("\n".join(out) + "\n", encoding="utf-8", newline="\n")
        report[family] = {f"{a} / {b}": c for (a, b), c in sorted(n.items())}
    print(json.dumps(report, indent=1))
    return 0


if __name__ == "__main__":
    sys.exit(main())
