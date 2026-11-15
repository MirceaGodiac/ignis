"""Quality labels and counts of a measured FLEX-2 batch, and its v2 series (pinned scale), for any batch.

    python -I tools/wsp/media/flex2_batch_summary.py --batch docs/wsp/atlas/splits/PSI-68-batch2.json \
        --tag _b2 --name batch2 [--visual runs/dev/media/droplet/FLEX2_B2_VISUAL_CHECK.tsv] [--apply]

The same labels as batch 1 (flex2_summary.py): the frozen acceptance rule's status from the measurement
log; 'failed' when the tool saw no droplet in > 95 % of post-deployment frames, when the track screens of
flex2_truncation_screen.py fire, or when a sheet failed the measurer's visual check. With --apply the label
is written into the series files and the v2 series (pinned scale 33.85 px/mm, SCALE_FLEX2.md) are written
to flex2_series_v2/<partition>/<test>.json with a `batch` field. No value other than the scale changes.
Counts and conditions only: no outcome is computed.
"""
import argparse
import json
import sys
from collections import Counter, defaultdict
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from flex2_truncation_screen import screen  # noqa: E402

REPO = Path(__file__).resolve().parents[3]
OUT = REPO / "runs/dev/media/droplet"
SEEN = {"ok", "soot", "glare", "outlier", "edge", "static"}
PXMM, PXMM_SD, ASSUMED = 33.85, 0.51, 20.48
SCALE_SOURCE = ("FLEX optical scale 33.85 px/mm (HiBMs field of view 30.25 mm over 1024 px), carried to FLEX-2 "
                "by the FLEX-2 requirements document (droplet view 3.0 cm, 30 um), the same camera geometry and the "
                "needle-arm width (53.9 px on FLEX-2, 54.4 px on the last FLEX test); 1-sigma 1.5 %. SCALE_FLEX2.md")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--batch", required=True)
    ap.add_argument("--tag", required=True)
    ap.add_argument("--name", required=True)
    ap.add_argument("--visual", default=None)
    ap.add_argument("--apply", action="store_true")
    ap.add_argument("--flat", action="store_true", help="series of this tag are flat (held batches)")
    ap.add_argument("--v2-dir", default="flex2_series_v2/{part}", help="where the v2 series go")
    a = ap.parse_args()
    batch = json.loads((REPO / a.batch).read_text(encoding="utf-8"))
    tests = {t["test_id"]: t for t in batch["tests"]}
    log = {}
    for line in (OUT / f"flex2_measure_log{a.tag}.jsonl").open(encoding="utf-8"):
        r = json.loads(line)
        log[r["test_id"]] = r
    visual = defaultdict(list)
    if a.visual and Path(a.visual).exists():
        for line in Path(a.visual).read_text(encoding="utf-8").splitlines():
            p = line.split("\t")
            if len(p) >= 4:
                visual[p[0]].append((p[2], p[1], p[3]))
    rows = []
    for tp, t in tests.items():
        r = log.get(tp)
        if r is None:
            rows.append(dict(test_id=tp, fuel=t["fuel"], tier=t.get("tier"), status="not measured", reasons=[]))
            continue
        fc = r.get("flag_counts") or {}
        post = sum(v for k, v in fc.items() if k != "pre_deploy")
        seen = sum(v for k, v in fc.items() if k in SEEN)
        q, why = r.get("quality"), list(r.get("reasons") or [])
        if q == "not_usable" and post and seen / post < 0.05:
            q, why = "failed", ["droplet not detected by the tool in > 95 % of post-deployment frames"] + why
        csvp = OUT / f"flex2_tool{a.tag}" / f"{tp}.csv"
        sc = screen(str(csvp)) if csvp.exists() else dict(truncated=False, n=0)
        if q == "usable" and sc.get("hijacked"):
            q, why = "failed", [f"track jumped to another object ({sc['jumps']} diameter jumps > 30 % between usable frames)"]
        elif q == "usable" and sc["truncated"]:
            q, why = "failed", [f"track truncated while the droplet was still in view ({sc['n']} droplet-sized lost frames)"]
        vf = [x for x in visual.get(tp, []) if x[0] == "FAIL"]
        if q == "usable" and vf:
            q, why = "failed", [f"visual check {vf[0][1]}: {vf[0][2]}"]
        rows.append(dict(test_id=tp, fuel=t["fuel"], tier=t.get("tier"), partition=t["partition"], status=q,
                         reasons=why, visual=[x[0] for x in visual.get(tp, [])], conditions=t.get("conditions"),
                         bytes_freed=r.get("bytes_freed", 0), compute_s=r.get("compute_s")))
        if a.apply:
            src = (OUT / f"flex2_series{a.tag}" / f"{tp}.json") if a.flat else (OUT / f"flex2_series{a.tag}" / t["partition"] / f"{tp}.json")
            if src.exists():
                s = json.loads(src.read_text(encoding="utf-8"))
                if s.get("quality") != q:
                    s["quality"], s["quality_reasons"] = q, why
                    s["quality_rule_note"] = "relabelled by flex2_batch_summary.py (screens / visual check); no value changed"
                s["batch"] = a.name
                src.write_text(json.dumps(s, indent=1) + "\n", encoding="utf-8")
                for o in s.get("observations", []):
                    o["value_si"] = o["value_si"] * ASSUMED / PXMM
                m = s.setdefault("measurement", {})
                m["px_per_mm_measured_with"] = m.get("px_per_mm", ASSUMED)
                m.update(px_per_mm=PXMM, px_per_mm_sd=PXMM_SD, scale_source=SCALE_SOURCE,
                         rescaled_from_v1_factor=ASSUMED / PXMM)
                s["series_version"] = "v2 (pinned scale, SCALE_FLEX2.md)"
                dst = OUT / a.v2_dir.format(part=t["partition"]) / f"{tp}.json"
                dst.parent.mkdir(parents=True, exist_ok=True)
                dst.write_text(json.dumps(s, indent=1) + "\n", encoding="utf-8")
    by = defaultdict(Counter)
    for x in rows:
        by[(x["fuel"], x.get("tier"))][x["status"]] += 1
    summ = dict(batch=a.name, tests=len(rows), status=dict(Counter(x["status"] for x in rows)),
                by_fuel_tier={f"{f} / {t}": dict(c) for (f, t), c in sorted(by.items())},
                reasons=dict(Counter(w.split(" (")[0].split(":")[0] for x in rows for w in x["reasons"])),
                gb_freed=round(sum(x.get("bytes_freed") or 0 for x in rows) / 1e9, 2), per_test=rows)
    (OUT / f"flex2_summary{a.tag}.json").write_text(json.dumps(summ, indent=1) + "\n", encoding="utf-8")
    print(json.dumps({k: v for k, v in summ.items() if k != "per_test"}, indent=1))


if __name__ == "__main__":
    main()
