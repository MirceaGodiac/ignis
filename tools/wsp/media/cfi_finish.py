"""Finish CFI (PSI-39) series measured by flex2_measure.py: CFI identity, CFI conditions, the CFI pixel scale.

The runner writes FLEX-2-shaped series. This rewrites, for each CFI series of a tag:
  * run_id "PSI-39:<CIR test point>", investigation "PSI-39", session_id the CFI day group;
  * conditions from the CFI catalogue (runs/dev/data_all/work/transfer_droplet/cfi/conditions.jsonl), the
    same choices as tools/wsp/ext/cfi_cases.py: x_o2, x_he from the campaign sheet, x_co2 = 0, pressure =
    nominal x 101325 Pa, fuel "n-Dodecane";
  * the pinned CFI scale (SCALE_CFI.md): 20.78 px/mm, 1-sigma 1.5 %; every value_si x 20.48 / 20.78;
  * the quality labels of flex2_batch_summary.py (frozen rule, the track screens, the visual check).
Writes cfi_series_v2/<partition>/<test>.json (train) or the held folder for test days. No other value changes.

    python -I tools/wsp/media/cfi_finish.py --tag _cfi_b3 --batch docs/wsp/atlas/splits/PSI-39-batch3.json \
        --visual runs/dev/media/droplet/CFI_B3_VISUAL_CHECK.tsv [--flat --dst cfi_series_test_v2]
"""
import argparse
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from flex2_truncation_screen import screen  # noqa: E402

REPO = Path(__file__).resolve().parents[3]
OUT = REPO / "runs/dev/media/droplet"
CAT = REPO / "runs/dev/data_all/work/transfer_droplet/cfi/conditions.jsonl"
PXMM, SD, ASSUMED = 20.78, 0.31, 20.48
SRC = ("CFI scale 20.78 px/mm (field of view 49.3 mm over 1024 px): deployment-needle arm 33.4 px on CFI against "
       "54.4 px on FLEX H13H101 at the FLEX scale 33.85 px/mm (verified on FLEX against NASA); consistent with the "
       "CFI droplet view of about 5 x 5 cm in the literature; 1-sigma 1.5 %. SCALE_CFI.md")
SEEN = {"ok", "soot", "glare", "outlier", "edge", "static"}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--tag", required=True)
    ap.add_argument("--batch", required=True)
    ap.add_argument("--visual", default=None)
    ap.add_argument("--flat", action="store_true")
    ap.add_argument("--dst", default="cfi_series_v2/{part}")
    a = ap.parse_args()
    cat = {json.loads(l)["cir"]: json.loads(l) for l in CAT.open(encoding="utf-8")}
    tests = {t["test_id"]: t for t in json.loads((REPO / a.batch).read_text(encoding="utf-8"))["tests"]}
    log = {json.loads(l)["test_id"]: json.loads(l) for l in (OUT / f"flex2_measure_log{a.tag}.jsonl").open(encoding="utf-8")}
    vis = {}
    if a.visual and Path(a.visual).exists():
        for line in Path(a.visual).read_text(encoding="utf-8").splitlines():
            p = line.split("\t")
            if len(p) >= 4:
                vis.setdefault(p[0], []).append((p[2], p[1], p[3]))
    out = []
    for tp, t in tests.items():
        r = log.get(tp)
        if r is None:
            out.append((tp, t["partition"], "not measured", []))
            continue
        q, why = r["quality"], list(r.get("reasons") or [])
        fc = r.get("flag_counts") or {}
        post = sum(v for k, v in fc.items() if k != "pre_deploy")
        if q == "not_usable" and post and sum(v for k, v in fc.items() if k in SEEN) / post < 0.05:
            q, why = "failed", ["droplet not detected by the tool in > 95 % of post-deployment frames"] + why
        csvp = OUT / f"flex2_tool{a.tag}" / f"{tp}.csv"
        sc = screen(str(csvp)) if csvp.exists() else dict(truncated=False, n=0)
        if q == "usable" and sc.get("hijacked"):
            q, why = "failed", [f"track jumped to another object ({sc['jumps']} diameter jumps > 30 %)"]
        elif q == "usable" and sc["truncated"]:
            q, why = "failed", [f"track truncated while the droplet was still in view ({sc['n']} lost frames)"]
        vf = [x for x in vis.get(tp, []) if x[0] == "FAIL"]
        if q == "usable" and vf:
            q, why = "failed", [f"visual check {vf[0][1]}: {vf[0][2]}"]
        src = (OUT / f"flex2_series{a.tag}" / f"{tp}.json") if a.flat else (OUT / f"flex2_series{a.tag}" / t["partition"] / f"{tp}.json")
        if src.exists():
            s = json.loads(src.read_text(encoding="utf-8"))
            c = cat[tp]
            src_c = "runs/dev/data_all/work/transfer_droplet/cfi/conditions.jsonl"
            s.update(run_id=f"PSI-39:{tp}", investigation="PSI-39", session_id=c["group"], partition=t["partition"],
                     batch="batch3", quality=q, quality_reasons=why, flex_number=c["flex"])
            s["conditions"] = {
                "x_o2": dict(value=c["x_o2"], unit="1", source=src_c, evidence="original_observation"),
                "x_co2": dict(value=0.0, unit="1", source=src_c, note="no CO2 in CFI", evidence="original_observation"),
                "x_he": dict(value=c["x_he"], unit="1", source=src_c, evidence="original_observation"),
                "pressure": dict(value=c["pressure_pa"], unit="Pa", source=src_c, note="nominal table pressure",
                                 evidence="original_observation"),
                "fuel": dict(value="n-Dodecane", unit=None, source=src_c, evidence="original_observation")}
            for o in s.get("observations", []):
                o["value_si"] = o["value_si"] * ASSUMED / PXMM
            m = s.setdefault("measurement", {})
            m.update(px_per_mm_measured_with=ASSUMED, px_per_mm=PXMM, px_per_mm_sd=SD, scale_source=SRC,
                     rescaled_from_v1_factor=ASSUMED / PXMM)
            s["series_version"] = "v2 (pinned CFI scale, SCALE_CFI.md)"
            dst = OUT / a.dst.format(part=t["partition"]) / f"{tp}.json"
            dst.parent.mkdir(parents=True, exist_ok=True)
            dst.write_text(json.dumps(s, indent=1) + "\n", encoding="utf-8")
        out.append((tp, t["partition"], q, why))
    for x in out:
        print(x[0], x[1], x[2], (x[3] or [""])[0][:80])


if __name__ == "__main__":
    main()
