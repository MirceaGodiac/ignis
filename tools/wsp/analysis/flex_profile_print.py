"""Print the profile table written by p2_flex_profile.py."""
import json
import sys

rows = json.load(open(sys.argv[1], encoding="utf-8"))
mode = sys.argv[2] if len(sys.argv) > 2 else "main"


def f(v, fmt="%.2f"):
    return "   ?" if v is None else fmt % v


rows.sort(key=lambda r: (r.get("co2") or 0, r.get("he") or 0, -(r.get("o2") or 0), r.get("p_atm") or 0, r.get("d0") or 0))
if mode == "main":
    print("run            O2   CO2  He   P    D0   kavg Dext | n    tlast d_last | K:0.2-1 1-2  2-3  3-5  5-8  8-12 12-18 18-26 | k1_5 tknee dknee kpost | Dext/D0 dknee/D0 drift")
    for r in rows:
        K = r.get("K", {})
        ks = " ".join(f(K.get(w)) for w in ["0.2-1", "1-2", "2-3", "3-5", "5-8", "8-12", "12-18", "18-26"])
        d0 = r.get("d0")
        ratio = (r["dext"] / d0) if r.get("dext") and d0 else None
        ratio2 = (r["d_knee"] / d0) if r.get("d_knee") and d0 else None
        print("%-14s %.2f %.2f %.2f %.2f %s %s %s | %4d %5s %5s | %s | %s %5s %5s %6s | %s %s %s" % (
            r["run"].split(":")[1], r.get("o2") or 0, r.get("co2") or 0, r.get("he") or 0, r.get("p_atm") or 0,
            f(d0), f(r.get("kavg")), f(r.get("dext")), r["n"], f(r.get("t_last"), "%.1f"), f(r.get("d_last")), ks,
            f(r.get("k_1_5")), f(r.get("t_knee"), "%.1f"), f(r.get("d_knee")), f(r.get("k_post"), "%.3f"),
            f(ratio), f(ratio2), f(r.get("drift_mm_s"), "%.1f")))
else:
    print("run            D0   d_pre d_min_ign d.15-.35 max_jump      held (t0,t1,D)            gaps")
    for r in rows:
        print("%-14s %s %s %s %s %-14s %-40s %s" % (
            r["run"].split(":")[1], f(r.get("d0")), f(r.get("d_pre")), f(r.get("d_min_ign")), f(r.get("d_015_035")),
            str(r.get("max_jump")), str(r.get("held"))[:40], str(r.get("gaps"))[:60]))
