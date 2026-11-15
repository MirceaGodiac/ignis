"""Per-run view of a droplet release on a development partition, next to what the run did.

usage: p2_droplet_compare.py <feedback.json> <profile.json> [release-index=0] [sort=crps|cond]
Model side: laws and state filter from the forecast diagnostics. Data side: NASA test sheet and the
windowed finite differences of the profile. Nothing here writes a number into a model.
"""
import json
import math
import sys
from collections import defaultdict

fb = json.load(open(sys.argv[1], encoding="utf-8"))
prof = {r["run"]: r for r in json.load(open(sys.argv[2], encoding="utf-8"))}
ri = int(sys.argv[3]) if len(sys.argv) > 3 else 0
sort = sys.argv[4] if len(sys.argv) > 4 else "crps"
rel = fb["releases"][ri]
print("release", rel.get("model_name"), rel["release_id"][:18], "cases", len(rel["cases"]), "refused", len(rel["refused"]))
by_run = defaultdict(dict)
for c in rel["cases"]:
    if c.get("scored") and "crps" in c["metrics"]:
        by_run[c["run_id"]][c["cutoff_s"]] = c
others = []
for j, r in enumerate(fb["releases"]):
    if j != ri:
        others.append({(c["run_id"], c["cutoff_s"]): c["metrics"]["crps"] for c in r["cases"] if "crps" in c["metrics"]})


def um(x):
    return "   ." if x is None or (isinstance(x, float) and math.isnan(x)) else "%4.0f" % (x * 1e6)


rows = []
for run, cases in by_run.items():
    any_c = next(iter(cases.values()))
    enc = any_c["encoder"]
    laws = enc["laws"]
    p = prof.get(run, {})
    d0 = math.sqrt(enc["q0_m2"]) * 1e3
    mean_crps = sum(c["metrics"]["crps"] for c in cases.values()) / len(cases)
    rows.append((mean_crps, run, cases, enc, laws, p, d0))
if sort == "crps":
    rows.sort(key=lambda r: -r[0])
else:
    rows.sort(key=lambda r: (r[5].get("co2") or 0, r[5].get("he") or 0, -(r[5].get("o2") or 0), r[6]))
print("run            O2  CO2  He   P   D0  | Dcr  piR  De_d  tR   pc  Kc   Dce | NASA: k   Dext | knee  dk   | crps@1 @3 @5 (others) | state@1      state@3      state@5      | bias5-10@1,@3,@5 | r_late@3 gate")
for mean_crps, run, cases, enc, laws, p, d0 in rows:
    de_d = d0 * math.sqrt(max(0.0, 1 - laws["w_d"]))
    t_r = laws["s_r"] * d0 * d0
    cr = []
    for cut in (1.0, 3.0, 5.0):
        c = cases.get(cut)
        if c is None:
            cr.append("   -")
        else:
            oth = "/".join(um(o.get((run, cut))).strip() for o in others)
            cr.append(um(c["metrics"]["crps"]) + ("(" + oth + ")" if others else ""))
    st = []
    for cut in (1.0, 3.0, 5.0):
        c = cases.get(cut)
        st.append("  -  " if c is None else "%.2f/%.2f/%.2f" % tuple(c["encoder"]["state"]))
    bias = []
    for cut in (1.0, 3.0, 5.0):
        c = cases.get(cut)
        b = None if c is None else (c.get("bias") or {}).get("lead_5_10s_um")
        b2 = None if c is None else (c.get("bias") or {}).get("lead_2_5s_um")
        bias.append(("%4.0f" % b) if b is not None else (("%4.0f*" % b2) if b2 is not None else "   ."))
    c3 = cases.get(3.0) or next(iter(cases.values()))
    print("%-14s %.2f %.2f %.2f %.1f %.2f | %4.2f %.2f %4.2f %4.1f %.2f %.2f %4.2f | %4s %4s | %4s %4s | %s | %s | %s | %5.2f %d" % (
        run.split(":")[1], p.get("o2") or 0, p.get("co2") or 0, p.get("he") or 0, p.get("p_atm") or 0, d0,
        laws["d_crit_m"] * 1e3, laws["pi_r"], de_d, t_r, laws["p_cool"], laws["k_cool_m2_s"] * 1e6, laws["d_ce_m"] * 1e3,
        "%.2f" % p["kavg"] if p.get("kavg") else "  ? ", "%.2f" % p["dext"] if p.get("dext") else "  ? ",
        "%.1f" % p["t_knee"] if p.get("t_knee") else "  . ", "%.2f" % p["d_knee"] if p.get("d_knee") else "  . ",
        " ".join(cr), " ".join(st), ",".join(bias), c3["encoder"].get("r_late") or float("nan"), c3["encoder"].get("n_gated", 0)))
# by session
sess = defaultdict(list)
for mean_crps, run, cases, enc, laws, p, d0 in rows:
    for cut, c in cases.items():
        sess[p.get("group", "?")].append(c["metrics"]["crps"])
print("\nby session (case-mean CRPS um):")
for g in sorted(sess):
    v = sess[g]
    print("  %-26s n=%3d  %5.1f" % (g, len(v), 1e6 * sum(v) / len(v)))
