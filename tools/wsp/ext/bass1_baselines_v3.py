#!/usr/bin/env python3
"""Constant baselines of the plastics on the third coding (bass1_cases_v3.py; made after the final run, PROTOCOL 13.8),
rebuilt from the version 3 TRAIN file exactly as bass1_cases_v2.py `baselines` builds the version 2 ones: the
constant quantiles of the quench speed (cm/s and display) that minimise the interval pinball loss on the train
burns, and the train incidence of "kept burning". Writes transfer_bass1/baselines_v3.json.

    python -I tools/wsp/ext/bass1_baselines_v3.py
"""
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import bass1_cases_v2 as v2  # noqa: E402

OUT = v2.OUT


def main():
    rd = lambda n: [json.loads(l) for l in (OUT / n).open(encoding="utf-8")]  # noqa: E731
    to = [c["outcome"] for c in rd("cases_train_v3.jsonl")]
    parts = {"validation": rd("cases_validation_outcomes_v3.jsonl"), "test": rd("cases_test_outcomes_v3.jsonl")}
    res = {}
    for target, lo, hi, grid in (("quench", "quench_u_lo_cm_s", "quench_u_hi_cm_s", [i * 0.02 for i in range(1001)]),
                                 ("quench_display", "quench_display_lo", "quench_display_hi", [38 + i * 0.05 for i in range(1700)])):
        qs = v2._fit([o for o in to if v2.quench_scored(o) and o[lo] is not None], lo, hi, grid)
        r = {"quantiles": qs}
        for p, os_ in (("train", to),) + tuple(parts.items()):
            s = v2.score({o["case_id"]: qs for o in os_}, os_, target)
            r[p] = {k: s[k] for k in ("n_cases", "n_days", "case_mean", "day_mean")}
        res[target + "_constant"] = r
    sus = [o for o in to if o["sustain"] is not None]
    p = sum(o["sustain"] for o in sus) / len(sus)
    res["sustain_incidence"] = {"p": round(p, 4)}
    (OUT / "baselines_v3.json").write_text(json.dumps(res, indent=1) + "\n", encoding="utf-8", newline="\n")
    print(json.dumps({k: v.get("quantiles", v.get("p")) for k, v in res.items()}))
    return 0


if __name__ == "__main__":
    sys.exit(main())
