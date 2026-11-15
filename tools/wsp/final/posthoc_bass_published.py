"""Head-to-head of the BASS-II fabric tip forecast with the published SIBAL spread rates, rebuilt.

Post-hoc, 15 November 2026, after the final run. Nothing is adopted, refused, fitted or tuned; no IGNIS
weight is touched. DRY RUN ONLY: train and validation tracks. The test tracks (2 burns) are never opened
here; scoring them needs the evaluator's final path and is not implemented.

Published sources (transcribed with provenance in docs/wsp/research/bass_published_burns.json):
  TM   NASA/TM-20210011385 sec. 7.1 (Zhao, T'ien, Ferkul, Olson), Table 7.1 and Fig. 7.4: the steady
       spread rate of each BASS/BASS-II SIBAL burn, "constant spread rate for more than 20 s".
  POS12 Ferkul, Olson, Johnston, T'ien, ASGSR 2012 (NTRS 20130012832), BASS-I fits at ~21 % O2:
       2 cm: Vf[mm/s] = 0.20 U + 0.56;  1 cm: Vf[mm/s] = 0.15 U + 0.07   (U in cm/s)
  OLSON Olson et al., UCL Discovery 10164340, Table 1: two BASS-I rows (Vf 0.27 and 0.51 cm/s).

None of these papers gives a model that maps (flow, O2, width) to a front position in time with
published constants for O2 below 21 %. The only published predictor with constants is the POS12
linear-in-U fit (21 % O2). The BASS team's own description of the front is kinematic: after a growth
phase the tip moves at a constant rate Vf (TM p. 95-96). The comparators are therefore:

  steady_prefix  constant-rate front, Vf = least-squares slope of the tip over the prefix stills with
                 cutoff - 10 s <= t <= cutoff (at least 2; otherwise all prefix stills), floored at 0.
                 Their measurement rule applied to what is known at the cutoff. No constant.
  pos12          rate(t) = POS12 fit at the flow U(t) of the track (the same flow history the
                 forward gets), width class from the label; no O2 correction (published at 21 %).
  pos12_o2       pos12 times X/0.21 (Vf linear in O2, the form of Ferkul & T'ien 1994 eq. 84 and
                 Olson 2008 that the TM cites); the scaling is a stated assumption, not a published fit.
  fig74_oracle   constant rate = the burn's own published steady Vf (TM Fig. 7.4). NOT a forecast:
                 it reads the burn's future. A ceiling for any constant-rate forecaster.

Every comparator is anchored at the cutoff on the same point: the value at the cutoff of the
least-squares line of the steady_prefix fit (so they differ only by the rate). Positions are clipped
at the window length (sample end), which the forward is given as well. mm -> sample widths:
100 mm exposed length / window_length_w (23.25 mm/w for the 4.302 w of the converted tracks);
--mm-per-w 22 gives the investigators' 2.2 cm.

Cases, targets and aggregation are those of wsp-dev front-feedback (crates/ignis-wsp-dev/src/front.rs):
cutoffs 5, 10, 20, 40 s after igniter-off; >= 2 prefix flame stills; >= 3 target flame stills in
(cutoff, cutoff + 60 s]; flame stills = state flame, quality != edge, t >= 0, tip finite; track needs
>= 6. A point forecast's CRPS equals its absolute error, so comparators are set against D1's CRPS and
against the absolute error of D1's median. Run score = mean over its cases (the shifted T16 video is
a second file of run T16, as in the scorer); reported = mean over runs, paired by run.

Reproduction (before any score): steady tip rate of each matched burn from our tracks against TM
Fig. 7.4 (and OLSON for two BASS-I burns), with the windows stated in reproduce().

Usage: python -I tools/wsp/final/posthoc_bass_published.py [--partition train|validation|both]
       [--mm-per-w 22] [--out runs/final/20261115/posthoc_bass_published.json]
       [--tracks DIR --no-reproduce]   (a batch: DIR holds <partition>/*.json; --no-reproduce skips the
                                        reproduction, so docs/wsp/research/bass_published_burns.json is not read)
"""
import argparse
import glob
import json
import os
import pathlib
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[3]
PUB = ROOT / "docs/wsp/research/bass_published_burns.json"
TRACKS = ROOT / "runs/dev/media/fabric/tracks_scorer/psi25"
EXTERNAL = ROOT / "tools/wsp/bass_optical/tracks/external_psi26"
WSP_DEV = ROOT / "target-merged/release/wsp-dev.exe"
D1 = "sha256:ac15e49f1cb60d4b4679bcdff82a07c18a4a501454428e35ac86e935e168855c"
OUT = ROOT / "runs/final/20261115/posthoc_bass_published.json"

CUTOFFS = (5.0, 10.0, 20.0, 40.0)
MIN_FLAME, MIN_PREFIX, MIN_TARGETS, WINDOW_S = 6, 2, 3, 60.0
FAN_ZERO = 38.0
FIT_S = 10.0
EXPOSED_MM = 100.0
POS12 = {"2 cm": (0.20, 0.56), "1 cm": (0.15, 0.07)}
FIG74 = {"PSI-25:T5": 0.784, "PSI-25:T6": 0.838, "PSI-25:T16": 0.825}  # mm/s, TM Fig. 7.4


def load(partition):
    out = []
    for f in sorted(glob.glob(str(TRACKS / partition / "*.json"))):
        t = json.loads(pathlib.Path(f).read_text(encoding="utf8"))
        if t.get("schema") != "wsp-bass-front-track/1":
            continue
        if t.get("partition") not in ("train", "validation"):
            raise SystemExit(f"refused: {f} is partition {t.get('partition')!r}")
        t["_file"] = os.path.basename(f)
        out.append(t)
    return out


def flow_speed(cal, fan, calibrated):
    """front_tools::flow_speed, line for line."""
    if calibrated is not None and calibrated >= 0:
        return calibrated
    rows = sorted((a, b) for a, b in cal)
    if not rows:
        return None
    if not any(r[1] == 0.0 for r in rows) and rows[0][0] > FAN_ZERO:
        rows.insert(0, (FAN_ZERO, 0.0))
    start = max(i for i, r in enumerate(rows) if r[1] == 0.0) if any(r[1] == 0.0 for r in rows) else 0
    rows = rows[start:]
    if fan <= rows[0][0]:
        return max(rows[0][1], 0.0)
    if len(rows) == 1:
        return None
    i = min(max(sum(1 for r in rows if r[0] <= fan), 1), len(rows) - 1)
    a, b = rows[i - 1], rows[i]
    if not b[0] > a[0]:
        return a[1]
    return max(a[1] + (fan - a[0]) * (b[1] - a[1]) / (b[0] - a[0]), 0.0)


def flow_of(t):
    steps = []
    for e in t.get("fan", []):
        u = flow_speed(t.get("fan_calibration", []), e["fan"], e.get("u_cm_s"))
        if u is None:
            raise ValueError("no flow speed")
        if not steps or steps[-1][1] != u:
            steps.append((e["t_s"], u))
    return steps


def u_at(steps, s):
    u = steps[0][1]
    for ts, us in steps:
        if ts <= s:
            u = us
        else:
            break
    return u


def flame_stills(t):
    v = [(s["t_s"], s["tip_w"]) for s in t["stills"]
         if s.get("state") == "flame" and (s.get("quality") or "") != "edge" and s["t_s"] >= 0
         and s.get("tip_w") is not None]
    return sorted(v)


def label_class(t):
    lab = (t["conditions"].get("sample_label") or "").lower().strip()
    return "1 cm" if lab.startswith("1 cm") or lab.startswith("1cm") else "2 cm"


def lsq(pts):
    n = len(pts)
    mt = sum(p[0] for p in pts) / n
    mx = sum(p[1] for p in pts) / n
    stt = sum((p[0] - mt) ** 2 for p in pts)
    k = sum((p[0] - mt) * (p[1] - mx) for p in pts) / stt if stt > 0 else 0.0
    return k, mx - k * mt


def cases_of(t):
    st = flame_stills(t)
    if len(st) < MIN_FLAME:
        return []
    out = []
    for c in CUTOFFS:
        pre = [p for p in st if p[0] <= c]
        tg = [p for p in st if c < p[0] <= c + WINDOW_S]
        if len(pre) >= MIN_PREFIX and len(tg) >= MIN_TARGETS:
            out.append((c, pre, tg))
    return out


def forecast(rate_fn, x0, cutoff, times, steps, window, dt=0.05):
    """x(t) = x0 + integral of rate (w/s) from the cutoff, clipped at the window length."""
    out, x, s = [], x0, cutoff
    for tt in times:
        while s < tt - 1e-12:
            h = min(dt, tt - s)
            x += rate_fn(s, steps) * h
            s += h
        out.append(min(x, window) if window else x)
    return out


def comparators(t, cutoff, pre, mm_per_w):
    recent = [p for p in pre if p[0] >= cutoff - FIT_S]
    fit = recent if len(recent) >= 2 else pre
    k, b = lsq(fit)
    x0 = k * cutoff + b
    v_prefix = max(k, 0.0)
    a, c0 = POS12[label_class(t)]
    xo2 = t["conditions"]["x_o2_initial"]
    xo2 = xo2 / 100 if xo2 > 1 else xo2
    comps = {
        "steady_prefix": lambda s, st: v_prefix,
        "pos12": lambda s, st: (a * u_at(st, s) + c0) / mm_per_w,
        "pos12_o2": lambda s, st: (a * u_at(st, s) + c0) * (xo2 / 0.21) / mm_per_w,
    }
    if t["run_id"] in FIG74:
        v = FIG74[t["run_id"]] / mm_per_w
        comps["fig74_oracle"] = lambda s, st: v
    return x0, comps, {"v_prefix_w_s": v_prefix, "x0_w": x0}


def ignis_detail(partition, workdir):
    det = workdir / f"ignis_detail_{partition}.json"
    subprocess.run([str(WSP_DEV), "front-feedback", "--release", D1, "--tracks", str(TRACKS / partition),
                    "--label", f"posthoc_bass_published dry run {partition}", "--out",
                    str(workdir / f"ignis_{partition}.json"), "--detail", str(det),
                    "--only-partition", partition], check=True, capture_output=True)
    rows = json.loads(det.read_text(encoding="utf8"))["rows"]
    by = {}
    for r in rows:  # in the scorer's file order; a run id with two files has its rows twice
        if r.get("cutoff_s") is None:
            continue
        by.setdefault((r["run_id"], r["cutoff_s"]), []).append(r)
    return by


def reproduce(mm_per_w_ii):
    """Steady tip rate from our tracks against the published numbers. Windows stated before reading
    any score: BASS-II train, flame stills with 20 s <= t <= end - 10 s (TM: steady > 20 s; end
    acceleration excluded); BASS-I external, 2 s <= t <= end - 3 s (short burns); mm per w =
    exposed length / window for BASS-II, the investigators' width (12 or 22 mm) for BASS-I."""
    pub = json.loads(PUB.read_text(encoding="utf8"))
    rows = []
    for name, rid in [("T5.json", "PSI-25:T5"), ("T6.json", "PSI-25:T6"), ("T16.json", "PSI-25:T16"),
                      ("T16___GMT100_sample139_test_T16_shifted_video.json", "PSI-25:T16")]:
        t = json.loads((TRACKS / "train" / name).read_text(encoding="utf8"))
        st = flame_stills(t)
        end = t["end"]["t_s"]
        k, _ = lsq([p for p in st if 20 <= p[0] <= end - 10])
        steps = flow_of(t)
        mmw = mm_per_w_ii or EXPOSED_MM / t["geometry"]["window_length_w"]
        rows.append({"file": name, "burn": rid, "ours_w_s": k, "mm_per_w": mmw, "ours_mm_s": k * mmw,
                     "published_mm_s": FIG74[rid], "error_pct": 100 * (k * mmw / FIG74[rid] - 1),
                     "scorer_flow_cm_s_in_window": sorted({round(u_at(steps, s), 2) for s, _ in st if 20 <= s <= end - 10}),
                     "published_U0_cm_s": {"PSI-25:T5": 2.40, "PSI-25:T6": 2.60, "PSI-25:T16": 2.99}[rid]})
    ext = {"BASS1_GMT131_T10_1cm.json": (12.0, 2.025, "TM Fig. 7.4"),
           "BASS1_GMT222_T11_1cm.json": (12.0, 2.766, "TM Fig. 7.4 (OLSON 2.7)"),
           "BASS1_GMT131_T4.json": (22.0, 5.1, "OLSON Table 1 (as-run: not steady)")}
    for name, (mmw, v, src) in ext.items():
        t = json.loads((EXTERNAL / name).read_text(encoding="utf8"))
        st = flame_stills(t)
        end = t["end"]["t_s"]
        k, _ = lsq([p for p in st if 2 <= p[0] <= end - 3])
        rows.append({"file": name, "burn": name, "ours_w_s": k, "mm_per_w": mmw, "ours_mm_s": k * mmw,
                     "published_mm_s": v, "source": src, "error_pct": 100 * (k * mmw / v - 1)})
    return rows


def agg(case_rows, key):
    """Run = track file, as the scorer weighs files (the shifted T16 video counts as its own run)."""
    runs = {}
    for r in case_rows:
        runs.setdefault(r["file"], []).append(r[key])
    rs = {k: sum(v) / len(v) for k, v in runs.items()}
    return {"mean_over_runs": sum(rs.values()) / len(rs) if rs else None, "by_run": rs,
            "n_runs": len(rs), "n_cases": len(case_rows)}


def main(argv=None):
    ap = argparse.ArgumentParser()
    ap.add_argument("--partition", choices=["train", "validation", "both"], default="both")
    ap.add_argument("--mm-per-w", type=float, default=None)
    ap.add_argument("--out", default=str(OUT))
    ap.add_argument("--tracks", default=None, help="folder holding <partition>/*.json (default: the converted PSI-25 tracks)")
    ap.add_argument("--no-reproduce", action="store_true", help="skip reproduce() (it reads the published-burns file)")
    a = ap.parse_args(argv)
    if a.tracks:
        global TRACKS
        TRACKS = pathlib.Path(a.tracks).resolve()
    out_path = pathlib.Path(a.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    work = out_path.parent / "posthoc_bass_published_work"
    work.mkdir(exist_ok=True)
    summary = {"what": __doc__.strip().splitlines()[0], "dry_run": True, "release_D1": D1,
               "mm_per_w": a.mm_per_w or "exposed 100 mm / window_length_w",
               "tracks": str(TRACKS), "reproduction": None if a.no_reproduce else reproduce(a.mm_per_w), "partitions": {}}
    parts = ["train", "validation"] if a.partition == "both" else [a.partition]
    for part in parts:
        ign = ignis_detail(part, work)
        rows = []
        for t in load(part):
            try:
                steps = flow_of(t)
            except ValueError:
                continue
            window = (t.get("geometry") or {}).get("window_length_w")
            mmw = a.mm_per_w or EXPOSED_MM / window
            for c, pre, tg in cases_of(t):
                x0, comps, info = comparators(t, c, pre, mmw)
                times = [p[0] for p in tg]
                obs = [p[1] for p in tg]
                row = {"run_id": t["run_id"], "file": t["_file"], "cutoff": c, "targets": len(tg), **info}
                for name, fn in comps.items():
                    pred = forecast(fn, x0, c, times, steps, window)
                    row[f"ae_{name}"] = sum(abs(p - o) for p, o in zip(pred, obs)) / len(obs)
                cand = ign.get((t["run_id"], c), [])
                if cand and cand[0]["targets"] == len(tg):  # files are read in the scorer's order
                    r0 = cand.pop(0)
                    row["ae_D1_median"] = r0["ae_median"]
                    row["crps_D1"] = r0["crps"]
                else:
                    row["ae_D1_median"] = None
                rows.append(row)
        names = ["steady_prefix", "pos12", "pos12_o2", "fig74_oracle"]
        res = {"cases": rows, "models": {}}
        ok = [r for r in rows if r.get("ae_D1_median") is not None]
        res["models"]["D1 crps"] = agg(ok, "crps_D1")
        res["models"]["D1 ae_median"] = agg(ok, "ae_D1_median")
        for n in names:
            sub = [r for r in ok if f"ae_{n}" in r]
            if sub:
                res["models"][n] = agg(sub, f"ae_{n}")
                res["models"][f"D1 crps on {n} cases"] = agg(sub, "crps_D1")
        res["by_cutoff"] = {f"{c:g}": {m: agg([r for r in ok if r["cutoff"] == c and k in r], k)["mean_over_runs"]
                                       for m, k in [("D1 crps", "crps_D1"), ("D1 ae_median", "ae_D1_median")]
                                       + [(n, f"ae_{n}") for n in names]
                                       if any(r["cutoff"] == c and k in r for r in ok)} for c in CUTOFFS}
        res["paired_D1crps_minus"] = {}
        for n in names:
            sub = [r for r in ok if f"ae_{n}" in r]
            if sub:
                d = agg([{**r, "d": r["crps_D1"] - r[f"ae_{n}"]} for r in sub], "d")
                res["paired_D1crps_minus"][n] = {"mean_over_runs": d["mean_over_runs"],
                                                 "runs_D1_lower": sum(v < 0 for v in d["by_run"].values()),
                                                 "n_runs": d["n_runs"]}
        res["unmatched_ignis_cases"] = len(rows) - len(ok)
        summary["partitions"][part] = res
    out_path.write_text(json.dumps(summary, indent=1), encoding="utf8")
    brief = {"reproduction": None if summary["reproduction"] is None else
             [{k: (round(v, 4) if isinstance(v, float) else v) for k, v in r.items()} for r in summary["reproduction"]]}
    for part, res in summary["partitions"].items():
        brief[part] = {m: {"mean": round(v["mean_over_runs"], 4), "runs": v["n_runs"], "cases": v["n_cases"],
                           "by_run": {k: round(x, 4) for k, x in v["by_run"].items()}}
                       for m, v in res["models"].items()}
        brief[part]["unmatched"] = res["unmatched_ignis_cases"]
        brief[part]["by_cutoff"] = {c: {m: round(v, 4) for m, v in d.items() if v is not None} for c, d in res["by_cutoff"].items()}
        brief[part]["paired_D1crps_minus"] = res["paired_D1crps_minus"]
    json.dump(brief, sys.stdout, indent=1)


if __name__ == "__main__":
    main()
