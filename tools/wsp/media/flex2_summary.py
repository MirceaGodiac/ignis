"""Counts and quality of a measured FLEX-2 batch (no outcome: no burn time, rate or extinction).

    python -I tools/wsp/media/flex2_summary.py   # prints JSON; writes runs/dev/media/droplet/flex2_summary.json

A not-usable series is labelled 'failed' (the tool could not see the droplet) when, after deployment,
fewer than 5 % of the frames gave any droplet candidate (flags ok, soot, glare, outlier, edge, static);
this label changes no series' usability, it only names the reason. A series the acceptance rule passed
is also labelled 'failed' when a sheet of it failed the visual check (FLEX2_VISUAL_CHECK.tsv), and when flex2_truncation_screen.py finds its track truncated while the droplet was still
in view (the frozen tool's track-hijack failure, seen first on FLEX-2). With --apply the label and the
reason are written into flex2_series/<partition>/<test>.json ('quality'); no value is changed.
"""
import sys
import json
from collections import Counter, defaultdict
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from flex2_truncation_screen import screen  # noqa: E402

REPO = Path(__file__).resolve().parents[3]
OUT = REPO / "runs/dev/media/droplet"
SEEN = {"ok", "soot", "glare", "outlier", "edge", "static"}


APPLY = "--apply" in sys.argv
VISUAL = {}  # test -> [(PASS|FAIL, sheet, note)] from FLEX2_VISUAL_CHECK.tsv (the measurer's own look)
_vt = Path(__file__).resolve().parents[3] / "runs/dev/media/droplet/FLEX2_VISUAL_CHECK.tsv"
if _vt.exists():
    for _l in _vt.read_text(encoding="utf-8").splitlines():
        _p = _l.split("	")
        if len(_p) >= 4:
            VISUAL.setdefault(_p[0], []).append((_p[2], _p[1], _p[3]))


def main():
    batch = json.loads((REPO / "docs/wsp/atlas/splits/PSI-68-batch1.json").read_text(encoding="utf-8"))
    tests = {t["test_id"]: t for t in batch["tests"]}
    log = {}
    for line in (OUT / "flex2_measure_log.jsonl").open(encoding="utf-8"):
        r = json.loads(line)
        log[r["test_id"]] = r
    rows = []
    for tp, t in tests.items():
        r = log.get(tp)
        if r is None:
            rows.append(dict(test_id=tp, fuel=t["fuel"], partition=t["partition"], status="not measured"))
            continue
        fc = r.get("flag_counts") or {}
        post = sum(v for k, v in fc.items() if k != "pre_deploy")
        seen = sum(v for k, v in fc.items() if k in SEEN)
        q = r.get("quality")
        why = list(r.get("reasons") or [])
        if q == "not_usable" and post and seen / post < 0.05:
            q = "failed"
            why = ["droplet not detected by the tool in > 95 % of post-deployment frames"] + why
        csvp = OUT / "flex2_tool" / f"{tp}.csv"
        trunc = screen(str(csvp)) if csvp.exists() else dict(truncated=False, n=0)
        if q == "usable" and trunc.get("hijacked"):
            q = "failed"
            why = [f"track jumped to another object ({trunc['jumps']} diameter jumps > 30 % between usable frames)"]
        elif q == "usable" and trunc["truncated"]:
            q = "failed"
            why = [f"track truncated while the droplet was still in view ({trunc['n']} droplet-sized lost frames)"]
        vfail = [x for x in VISUAL.get(tp, []) if x[0] == "FAIL"]
        if q == "usable" and vfail:
            q = "failed"
            why = [f"visual check {vfail[0][1]}: {vfail[0][2]}"]
        if APPLY and q != r.get("quality") and q == "failed":
            sp = OUT / "flex2_series" / t["partition"] / f"{tp}.json"
            if sp.exists():
                ser = json.loads(sp.read_text(encoding="utf-8"))
                ser["quality"], ser["quality_reasons"] = "failed", why
                ser["quality_rule_note"] = "relabelled by tools/wsp/media/flex2_summary.py --apply (no value changed)"
                sp.write_text(json.dumps(ser, indent=1) + "\n", encoding="utf-8")
        ruler = (r.get("ruler") or {}).get("rod_width_mm_at_scale")
        rows.append(dict(test_id=tp, fuel=t["fuel"], partition=t["partition"], status=q, reasons=why,
                         usable_frames=r.get("observations"), image_px=r.get("image_px"), px_per_mm=r.get("px_per_mm"),
                         ruler_mm=ruler, clock_check_s=(r.get("clock_check") or {}).get("seen_minus_stamp_s"),
                         compute_s=r.get("compute_s"), bytes_freed=r.get("bytes_freed", 0),
                         archive_bytes=r.get("archive_bytes")))
    by = defaultdict(Counter)
    for x in rows:
        by[(x["fuel"], x["partition"])][x["status"]] += 1
    rul = sorted(x["ruler_mm"] for x in rows if x.get("ruler_mm"))
    summ = dict(
        tests=len(rows), status=dict(Counter(x["status"] for x in rows)),
        by_fuel_partition={f"{f} / {p}": dict(c) for (f, p), c in sorted(by.items())},
        reasons=dict(Counter(w.split(":")[0] for x in rows for w in x.get("reasons") or [])),
        ruler_mm=dict(n=len(rul), min=rul[0] if rul else None, median=rul[len(rul) // 2] if rul else None,
                      max=rul[-1] if rul else None),
        image_px=dict(Counter(str(x.get("image_px")) for x in rows if x.get("image_px"))),
        gb_freed=round(sum(x.get("bytes_freed") or 0 for x in rows) / 1e9, 2),
        compute_s_total=round(sum(x.get("compute_s") or 0 for x in rows), 0), per_test=rows)
    (OUT / "flex2_summary.json").write_text(json.dumps(summ, indent=1) + "\n", encoding="utf-8")
    print(json.dumps({k: v for k, v in summ.items() if k != "per_test"}, indent=1))


if __name__ == "__main__":
    main()
