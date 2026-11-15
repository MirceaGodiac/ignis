#!/usr/bin/env python3
"""Anatomy of F1's loss on the TRAIN DAYS of Flame Design (PROTOCOL 12.3 step 2, decompose). Reads only
runs/dev/data_all/work/transfer_acme/fd_fuel/train_days_cases.jsonl (train-day rows only; it asserts so)
and prints aggregate tables. Nothing is fitted.

  * scores (IBS contributions as stored) by session and by regime, F1 against the surrogate;
  * Brier by horizon and cause (forecasts interpolated from the dense grid at the 14 horizons);
  * centre against spread, one row per test (cutoff 1 s): ln(t50 / t_end), t50 the time at which the
    forecast's total event probability (extinction + operator termination) reaches 0.5, from the dense
    grid 1.5..600 s; the width ln(t90 / t10); whether t_end lies inside [t10, t90];
  * the probability of extinction by 120 s, forecast against the observed fraction.

    python tools/wsp/ext/fd_train_anatomy.py [--path nostop|stop]
"""
import argparse
import json
import math
import statistics as st
import sys
from collections import defaultdict
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import transfer_acme as ta  # noqa: E402

REPO = ta.REPO
FILE = REPO / "runs/dev/data_all/work/transfer_acme/fd_fuel/train_days_cases.jsonl"
TRAIN = {"acme_normal": {"19115", "19137", "19150", "19156", "19171", "19175", "19182", "19189"},
         "acme_inverse": {"21328", "21349", "22024", "22027", "22035"}}


def qt(times, total, q):
    pt, pp = None, None
    for t, p in zip(times, total):
        if p >= q:
            if pt is None or p == pp:
                return t
            return math.exp(math.log(pt) + (q - pp) / (p - pp) * (math.log(t) - math.log(pt)))
        pt, pp = t, p
    return None


def fc(table):
    cats = table["event_categories"]
    e, o = table["event_cdf"][cats.index("extinction")], table["event_cdf"][cats.index("operator_termination")]
    return table["times_s"], e, o


def regime(fam, c):
    p = c["pressure"] / 101325.0
    o2 = c["x_o2_role"]
    if fam == "acme_normal":
        ro = "air (0.20-0.21)" if o2 < 0.25 else ("29-30 %" if o2 < 0.33 else "36-40 %")
        rp = "1.0 atm" if p < 1.15 else "1.25-1.29 atm"
        return ro, rp, None
    ro = "O2 <= 0.30" if o2 <= 0.30 else ("O2 0.31-0.65" if o2 <= 0.65 else "O2 0.85")
    rp = f"{p:.1f} atm".replace("0.5 atm", "0.5 atm")
    xf = c["x_fuel_role"]
    rf = "C2H4 0.08-0.10 (lean)" if xf < 0.12 else ("C2H4 0.17-0.19" if xf < 0.22 else "C2H4 0.25-0.28")
    return ro, rp, rf


def med(v):
    return st.median(v) if v else float("nan")


def iqr(v):
    if len(v) < 4:
        return (float("nan"), float("nan"))
    q = st.quantiles(v, n=4)
    return q[0], q[2]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--path", default="nostop")
    a = ap.parse_args()
    rows = [json.loads(l) for l in FILE.read_text(encoding="utf-8").splitlines() if l.strip()]
    assert all(r["days"] == "train" and r["session"][-5:] in TRAIN[r["family"]] for r in rows), "non-train row"
    rows = [r for r in rows if r["path"] == a.path]
    f4 = lambda x: "  n/a " if x != x else f"{x:+.3f}" if abs(x) < 10 else f"{x:.0f}"  # noqa: E731
    for fam in ("acme_normal", "acme_inverse"):
        R = [r for r in rows if r["family"] == fam]
        ans = [r for r in R if r["f1_refused"] is None]
        print(f"\n######## {fam} ({a.path}): {len(R)} cases, {len({r['test'] for r in R})} tests; F1 answers {len(ans)}")
        # ---- scores by group
        def group_table(title, key):
            g = defaultdict(list)
            for r in ans:
                g[key(r)].append(r)
            print(f"\n{title}: cases tests | ext obs | F1 ext  term  sum | surr ext  term  sum | F1-surr ext | P(ext<=120) F1 / surr")
            for k in sorted(g):
                v = g[k]
                m = lambda who, x: st.fmean(r["ibs_contrib"][who][x] for r in v)  # noqa: E731
                pe = {who: st.fmean(fc(r[f"{'f1' if who == 'f1' else 'surrogate'}_forecast_dense"])[1][-1] if False else
                                    ta.interp_row(*fc(r["f1_forecast_dense" if who == "f1" else "surrogate_forecast_dense"])[:2], [120.0])[0]
                                    for r in v) for who in ("f1", "surrogate")}
                obs = st.fmean(1.0 if r["endpoint"]["cause"] == "extinction" and r["endpoint"]["time_s"] <= 120 else 0.0 for r in v)
                print(f"  {str(k):<40} {len(v):>4} {len({r['test'] for r in v}):>4} | {obs:.2f} | "
                      f"{m('f1','extinction'):.3f} {m('f1','operator_termination'):.3f} {m('f1','sum'):.3f} | "
                      f"{m('surrogate','extinction'):.3f} {m('surrogate','operator_termination'):.3f} {m('surrogate','sum'):.3f} | "
                      f"{m('f1','extinction') - m('surrogate','extinction'):+.3f} | {pe['f1']:.2f} / {pe['surrogate']:.2f}")
        group_table("by session", lambda r: r["session"][-5:])
        group_table("by oxygen", lambda r: regime(fam, r["conditions"])[0])
        group_table("by pressure", lambda r: regime(fam, r["conditions"])[1])
        if fam == "acme_inverse":
            group_table("by ambient ethylene", lambda r: regime(fam, r["conditions"])[2])
        else:
            group_table("by fuel mole fraction", lambda r: "undiluted" if r["conditions"]["x_fuel_role"] > 0.9 else
                        ("0.25-0.57" if r["conditions"]["x_fuel_role"] >= 0.25 else "0.05-0.22"))
        group_table("by end cause", lambda r: r["endpoint"]["cause"])
        group_table("by cutoff", lambda r: r["cutoff_s"])
        # ---- Brier by horizon and cause
        print("\nBrier by horizon (all answered cases), F1 / surrogate:")
        for cause, ci in (("extinction", 1), ("operator_termination", 2)):
            line = []
            for h in ta.HORIZONS:
                b = {}
                for who in ("f1", "surrogate"):
                    s = 0.0
                    for r in ans:
                        t, e, o = fc(r["f1_forecast_dense" if who == "f1" else "surrogate_forecast_dense"])
                        p = ta.interp_row(t, e if ci == 1 else o, [h])[0]
                        ev = r["endpoint"]["time_s"] <= h and r["endpoint"]["cause"] == cause
                        s += ((1.0 if ev else 0.0) - p) ** 2
                    b[who] = s / len(ans)
                line.append(f"{h:g}:{b['f1']:.3f}/{b['surrogate']:.3f}")
            print(f"  {cause}: " + "  ".join(line))
        # ---- centre against spread, one row per test (cutoff 1 s)
        print("\ncentre and spread, cutoff 1 s, one row per test: ln(t50/t_end) median [IQR]; width ln(t90/t10) median; "
              "t_end in [t10,t90]; t50 beyond 600 s")
        one = [r for r in ans if r["cutoff_s"] == 1.0]
        def centre(sel, label):
            out = {}
            for who in ("f1", "surrogate"):
                lr, w, inside, beyond = [], [], 0, 0
                for r in sel:
                    t, e, o = fc(r["f1_forecast_dense" if who == "f1" else "surrogate_forecast_dense"])
                    tot = [x + y for x, y in zip(e, o)]
                    t10, t50, t90 = qt(t, tot, 0.1), qt(t, tot, 0.5), qt(t, tot, 0.9)
                    te = r["endpoint"]["time_s"]
                    if t50 is None:
                        beyond += 1
                        continue
                    lr.append(math.log(t50 / te))
                    if t90 is not None and t10 is not None:
                        w.append(math.log(t90 / t10))
                        inside += t10 <= te <= t90
                out[who] = f"{med(lr):+.2f} [{iqr(lr)[0]:+.2f}, {iqr(lr)[1]:+.2f}] w {med(w):.2f} in {inside}/{len(sel)} >600 {beyond}"
            print(f"  {label:<40} n {len(sel):>3} | F1 {out['f1']} | surr {out['surrogate']}")
        g = defaultdict(list)
        for r in one:
            ro, rp, rf = regime(fam, r["conditions"])
            g[("cause", r["endpoint"]["cause"])].append(r)
            g[("oxygen", ro, r["endpoint"]["cause"])].append(r)
            g[("pressure", rp, r["endpoint"]["cause"])].append(r)
            if rf:
                g[("ambient", rf, r["endpoint"]["cause"])].append(r)
            g[("session", r["session"][-5:])].append(r)
        centre(one, "all")
        for k in sorted(g):
            centre(g[k], " ".join(k))
        # ---- model ratio F1 / surrogate life (t50), registered expectation check
        print("\nmodel life ratio t50(F1)/t50(surrogate), cutoff 1 s, median by oxygen (and ambient for inverse):")
        g = defaultdict(list)
        for r in one:
            tt = {}
            for who in ("f1", "surrogate"):
                t, e, o = fc(r["f1_forecast_dense" if who == "f1" else "surrogate_forecast_dense"])
                tt[who] = qt(t, [x + y for x, y in zip(e, o)], 0.5)
            if tt["f1"] and tt["surrogate"]:
                ro, rp, rf = regime(fam, r["conditions"])
                g[ro if fam == "acme_normal" else rf].append(tt["f1"] / tt["surrogate"])
        for k in sorted(g):
            print(f"  {k:<30} n {len(g[k]):>3} median {med(g[k]):.2f} range {min(g[k]):.2f}-{max(g[k]):.2f}")
        # refused by F1
        rf = [r for r in R if r["f1_refused"] is not None]
        if rf:
            print(f"\nrefused by F1: {len(rf)} cases, {len({r['test'] for r in rf})} tests, sessions "
                  f"{sorted({r['session'][-5:] for r in rf})}, Tad {sorted({r['conditions']['tad'] for r in rf})}; surrogate "
                  f"ext {st.fmean(r['ibs_contrib']['surrogate']['extinction'] for r in rf):.3f} term "
                  f"{st.fmean(r['ibs_contrib']['surrogate']['operator_termination'] for r in rf):.3f}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
