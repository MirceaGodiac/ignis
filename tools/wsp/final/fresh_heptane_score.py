"""The registered blind look of PROTOCOL 13.9 (with its amendment): D4 against D2 on fresh n-heptane tests.

For each listed test and cutoff 1, 3, 5 s: a case enters if the series has at least five valid points
after the cutoff. Both releases are run by `ignis wsp forward` on the same PrefixBundle
(`fresh_heptane_prefix.py`, checked equal to `infer forecast` on a train run).
  (1) MAE of the median over the observations in (cutoff, cutoff + 10 s], per case;
  (2) nRMSE of the median (RMSE / mean measured diameter) over (cutoff, min(series end, 45 s)].
Session-weighted = mean of session means (sessions = test days). Prints aggregates and per-case rows.

Usage: python tools/wsp/final/fresh_heptane_score.py --stage A --tests E087C01 E087C03 ... [--out FILE]
"""
import argparse
import bisect
import json
import pathlib
import subprocess
import sys
import tempfile

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import fresh_heptane_prefix as F  # noqa: E402

ROOT = F.ROOT
EXE = ROOT / "target-d4/release/ignis.exe"
SERIES = ROOT / "runs/dev/heptane_fresh/series"
RELEASES = {"D4": "sha256:f3ccc122172b9d680cd17af5183bc640bf643d1175ed576b684ec9b6f324b164",
            "D2": "sha256:43f7871f778040550291789ca97e319729675a9dd0df9c9529525c1805ad588d"}
T_MAX = 45.0


def at(xs, ys, t):
    if t < xs[0] or t > xs[-1]:
        return None
    i = bisect.bisect_left(xs, t)
    if xs[i] == t:
        return ys[i]
    w = (t - xs[i - 1]) / (xs[i] - xs[i - 1])
    return ys[i - 1] * (1 - w) + ys[i] * w


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--stage", required=True)
    ap.add_argument("--tests", nargs="+", required=True)
    ap.add_argument("--out")
    a = ap.parse_args()
    rows = []
    with tempfile.TemporaryDirectory() as td:
        for test in a.tests:
            s = json.loads((SERIES / f"{test}.json").read_text(encoding="utf8"))
            assert s.get("quality") == "usable" and s["visual_check"]["verdict"] == "PASS", test
            pts = sorted((o["time_s"], o["value_si"]) for o in s["observations"]
                         if o.get("valid", True) and o.get("value_si") is not None)
            end = pts[-1][0]
            for cut in (1.0, 3.0, 5.0):
                after = [p for p in pts if p[0] > cut]
                if len(after) < 5:
                    continue
                pf = pathlib.Path(td) / f"{test}_{cut:g}.json"
                pf.write_text(json.dumps(F.from_series(SERIES / f"{test}.json", cut)), encoding="utf8")
                row = {"test": test, "session": s["session_id"], "cutoff": cut, "series_end_s": end}
                for name, rid in RELEASES.items():
                    of = pathlib.Path(td) / f"{test}_{cut:g}_{name}.json"
                    r = subprocess.run([str(EXE), "wsp", "forward", "--release", rid, "--prefix", str(pf),
                                        "--settings", json.dumps({"horizon_s": T_MAX - cut}), "--out", str(of)],
                                       cwd=ROOT, capture_output=True, text=True)
                    if r.returncode != 0:
                        row[name] = {"refused": (r.stderr or r.stdout).strip()[-300:]}
                        continue
                    art = json.loads(of.read_text(encoding="utf8"))
                    art = art.get("artifact", art)
                    lv = art["quantile_levels"]
                    xs, ys = art["times_s"], art["quantiles"][lv.index(0.5)]
                    e10 = [at(xs, ys, t) - d for t, d in after if t <= cut + 10 and at(xs, ys, t) is not None]
                    eb = [(at(xs, ys, t) - d, d) for t, d in after if t <= min(end, T_MAX) and at(xs, ys, t) is not None]
                    rmse = (sum(e * e for e, _ in eb) / len(eb)) ** 0.5
                    row[name] = {"mae10_um": 1e6 * sum(abs(e) for e in e10) / len(e10), "n10": len(e10),
                                 "nrmse_burn_pct": 100 * rmse / (sum(d for _, d in eb) / len(eb)), "n_burn": len(eb)}
                rows.append(row)
    ok = [r for r in rows if all("mae10_um" in r[n] for n in RELEASES)]

    def sw(name, key):
        by = {}
        for r in ok:
            by.setdefault(r["session"], []).append(r[name][key])
        return sum(sum(v) / len(v) for v in by.values()) / len(by)

    summ = {"stage": a.stage, "tests": a.tests, "n_cases": len(ok), "n_refused_or_missing": len(rows) - len(ok),
            "sessions": sorted({r["session"] for r in ok}),
            **{f"{n}_{k}": sw(n, k) for n in RELEASES for k in ("mae10_um", "nrmse_burn_pct")}}
    summ["criterion1_D4_lower_mae10"] = summ["D4_mae10_um"] < summ["D2_mae10_um"]
    summ["criterion2_D4_lower_nrmse_burn"] = summ["D4_nrmse_burn_pct"] < summ["D2_nrmse_burn_pct"]
    summ["both_hold"] = summ["criterion1_D4_lower_mae10"] and summ["criterion2_D4_lower_nrmse_burn"]
    out = {"summary": summ, "cases": rows}
    if a.out:
        pathlib.Path(a.out).write_text(json.dumps(out, indent=1), encoding="utf8")
    print(json.dumps(out, indent=1))


if __name__ == "__main__":
    main()
