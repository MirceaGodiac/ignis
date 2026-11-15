"""Recount, on the inverse TRAIN flames, of the claim that the stoichiometric mixture
fraction separates "goes out by itself" from "burning when the flow is stopped" better than the
total burner flow does. Uses the shared records only (conditions and endpoints of train runs).
Prints aggregates and, for the overlap band, the train test names.
"""
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import lib  # noqa: E402

FAM = "acme_inverse"
ids = lib.partition_ids(FAM, "train")
exps = lib.experiments(FAM, ids)
if isinstance(exps, dict):
    exps = list(exps.values())
ends = lib.jsonl_for(FAM, "endpoints.jsonl", ids)
if isinstance(ends, dict):
    flat = []
    for v in ends.values():
        flat += v if isinstance(v, list) else [v]
    ends = flat
end = {e["run_id"]: e for e in ends}
assert all(e["run_id"] in set(ids) for e in ends)

M_O2, M_N2 = 31.998, 28.014
rows = []
for rec in exps:
    rid = rec.get("id") or rec.get("run_id")
    e = end[rid]
    label = e["original_label"]
    t = e.get("first_terminal_s")
    if t is None:
        t = e.get("intervention_s")
    if t is None:
        t = e.get("last_confirmed_active_s")
    xo = float(lib.cond(rec, "x_o2_injected"))
    mo = float(lib.cond(rec, "injected_mass_flow")) * 1e6  # kg/s -> mg/s
    yo = xo * M_O2 / (xo * M_O2 + (1 - xo) * M_N2)
    rows.append(dict(test=rid.split(":")[-1], label=str(label).strip().upper()[:2], t=float(t) if t is not None else None,
                     zst=float(lib.cond(rec, "zst")), m_tot=mo / yo, m_o2=mo, xo=xo,
                     xf=float(lib.cond(rec, "x_fuel_ambient")), fuel=str(lib.cond(rec, "fuel_species"))))

print("train flames:", len(rows), "labels:", sorted({r["label"] for r in rows}))
alive = [r for r in rows if r["t"] is not None and r["t"] >= 5.0]
se = [r for r in alive if r["label"] == "SE"]
ft = [r for r in alive if r["label"] == "FT"]
print("alive at 5 s:", len(alive), "self-extinguished:", len(se), "flow-terminated:", len(ft))


def overlap(key):
    lo = max(min(r[key] for r in se), min(r[key] for r in ft))
    hi = min(max(r[key] for r in se), max(r[key] for r in ft))
    a = [r for r in se if lo <= r[key] <= hi]
    b = [r for r in ft if lo <= r[key] <= hi]
    print(f"{key:6s} SE {min(r[key] for r in se):.3f}..{max(r[key] for r in se):.3f}  FT {min(r[key] for r in ft):.3f}.."
          f"{max(r[key] for r in ft):.3f}  overlap {lo:.3f}..{hi:.3f}: SE {len(a)} FT {len(b)}")
    return a, b


for k in ("zst", "m_tot", "m_o2", "xf", "xo"):
    a, b = overlap(k)
    if k == "zst":
        za, zb = a, b
print()
print("inside the Z_st overlap (train tests):")
for r in sorted(za + zb, key=lambda r: (r["fuel"], r["zst"])):
    print(f"  {r['test']:11s} {r['fuel'][:8]:8s} zst {r['zst']:.3f} O2 {r['m_o2']:.2f} mg/s total {r['m_tot']:.1f} mg/s "
          f"xO2 {r['xo']:.2f} x_fuel {r['xf']:.3f} {r['label']} {r['t']:.0f} s")
print()
print("lean flames (x_fuel <= 0.10) alive at 5 s, by total flow:")
lean = sorted([r for r in alive if r["xf"] <= 0.10], key=lambda r: r["m_tot"])
for lo, hi in ((0, 10.6), (10.6, 16.9), (16.9, 100)):
    g = [r for r in lean if lo < r["m_tot"] <= hi]
    print(f"  total flow {lo}-{hi} mg/s: SE {sum(r['label'] == 'SE' for r in g)}, FT {sum(r['label'] == 'FT' for r in g)};"
          f" Z_st {min([r['zst'] for r in g], default=float('nan')):.2f}..{max([r['zst'] for r in g], default=float('nan')):.2f}")
