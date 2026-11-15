"""nRMSE of the frozen FLEX releases under the protocol of Xi, Torero & Jahn (2021): observe the first 5 s,
forecast 5-10 s, RMSE / mean measured diameter. Post-hoc, 15 November 2026; nothing is adopted or refused.
Uses only the cutoff-5 forecasts sealed by the final run and the continuations already cached by
posthoc_point_metrics.py (no new reveal). Usage: python tools/wsp/final/posthoc_xi_window.py [flex_heptane|flex_methanol]
"""
import json, sys, statistics as st
import pathlib
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import posthoc_point_metrics as P
fam = "flex_heptane" if len(sys.argv) < 2 else sys.argv[1]
spec = P.LIST["families"][fam]; adopted = spec["release"]
labels = {adopted: spec["label"], **{(v if isinstance(v, str) else v["id"]): k for k, v in spec["baselines"].items()}}
for rid, lab in labels.items():
    fc = P.SEALED / fam / f"forecast_shared_{P.short(rid)}.json"
    if not fc.exists(): continue
    rows = []
    for c in json.loads(fc.read_text(encoding="utf8"))["cases"]:
        if c.get("status") != "artifact" or c["cutoff_s"] != 5: continue
        key = f"{c['run_id'].replace(':','_')}__5.json"
        cont = json.loads((P.CONT / key).read_text(encoding="utf8"))["continuation"]  # cache only, no reveal
        cont = dict(cont); cont["observations"] = [o for o in cont["observations"] if o.get("time_s") is not None and o["time_s"] <= 10.0]
        art = json.loads((P.ART / f"{c['artifact_id'].split(':')[1]}.json").read_text(encoding="utf8"))
        m = P.case_metrics(art, cont)
        if m: rows.append({"run": c["run_id"], "session": c["run_id"].split(":")[-1], **m})
    v = sorted(r["nrmse_mean_pct"] for r in rows)
    by = {}
    for r in rows: by.setdefault(r["session"], []).append(r["nrmse_mean_pct"])
    sw = st.mean(st.mean(x) for x in by.values())
    print(f"{lab:28s} runs={len(rows):2d} sessions={len(by)} nRMSE case-mean={st.mean(v):.2f}% session-w={sw:.2f}% median={st.median(v):.2f}% min={v[0]:.2f}% <=2.46%: {sum(x<=2.46 for x in v)}/{len(v)}")
    if rid == adopted:
        for r in sorted(rows, key=lambda r: r["nrmse_mean_pct"]): print("   ", r["run"], f"{r['nrmse_mean_pct']:.2f}%", r["n_points"], "pts")
