#!/usr/bin/env python3
"""What-if runs for the physicist of the fabric flames (PROTOCOL 13.8): a child of a stored `bass_front.wsp5`
release with a few changed lines of its ignition and quench numbers, scored beside the base, D1, the carry of D1
and persistence on every fabric track that exists in the scorer's format.

    python tools/wsp/ext/front_whatif.py --base configs/models/bass/front5/q0-fable \\
        --edits changed.wsp --note "ign.dx 0.004 -> 0.002: more samples light at 17-18 % O2"

Labelling (13.8): every partition here, test included, is study data after the final run of 15 November. The
scores are descriptions of fit on data the author has seen, never held-out results.

What it does, in order:

1. Builds the child: the lines of `--edits` (the edit language with the spec's names, e.g. `ign.dx[0:2] = [0.003,
   0.006]`) compiled as a literal revision whose parent is the base release, assembled into a private registry
   under `runs/dev/gen7/front_whatif/<run>/registry`. The base, D1, the carry and persistence are copied there;
   the lane registry `artifacts/` is only read.
2. Scores base, child, D1 (`bass_front.wsp`), the carry of D1 and persistence with
   `target-merged/release/wsp-dev.exe front-feedback` (the protocol of `crates/ignis-wsp-dev/src/front.rs`,
   unchanged) on:
   * train: `tools/wsp/bass_optical/tracks/train` (the registered train tracks, 4 burns);
   * validation: `runs/dev/bass_front/val_sealed` (the 17 tracks measured blind on 14 November, the set of 12.9);
   * test: `runs/dev/media/fabric/tracks_scorer/psi25/test` (the converted tracks the final run scored). The
     scorer refuses a test track outside `--final-test`, whose reveal lines belong to the final run only, so the
     files are copied into this run's folder and read through a `--partition-map` that names them
     `validation` (the scorer's label; they are reported as test here);
   * pooled: the three folders together, the same way.
   Saffire I-III (PSI-98/99/100) have sensor tables and images only, no front track: shown as a row with no score.
3. Prints, per partition and pooled, with the count of burns: tip CRPS (sample widths, mean over burns), the
   track-ended Brier (mean over burns) and the ignition-outcome Brier (mean over tracks), for every release and for
   two constants fitted on the train tracks only: the share of train burns that did not spread (ignition) and the
   share of train event pairs observed ended at each horizon (track ended).
4. Prints once per run (and writes `burns.md` and `burns.json` in the run folder) the per-burn table: conditions
   (oxygen, flow at igniter-off, sample label and window length, material), whether it ignited and spread, when
   its track ended, and what base, child and D1 said (probability of no spread at 20 s; probability that the track
   has ended 30 and 60 s after the 10 s cutoff, or the first cutoff the burn has).
5. Appends one line per run to `runs/dev/gen7/whatif_log.jsonl`: time, family, base id, child id, sha256 of the
   edits, the note, the binary's sha256, the scores per partition.
"""
import argparse
import datetime as dt
import hashlib
import json
import os
import shutil
import statistics
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
LOG = REPO / "runs/dev/gen7/whatif_log.jsonl"
WORK = REPO / "runs/dev/gen7/front_whatif"
EXE = REPO / ("target-merged/release/wsp-dev.exe" if os.name == "nt" else "target-merged/release/wsp-dev")
IGNIS = REPO / ("target-merged/release/ignis.exe" if os.name == "nt" else "target-merged/release/ignis")
FRONT5 = REPO / "configs/models/bass/front5"
D1 = "sha256:ac15e49f1cb60d4b4679bcdff82a07c18a4a501454428e35ac86e935e168855c"
CARRY = "sha256:4da9967f32a8935890dbe07c700f7b89cfd589e177800fc2116ffcf5069b8ed4"
PERSIST = "sha256:95a4d6ec04e138d9f8787ecec1220abb3b173bea6d3dbfea7c91d3ef24fd3e62"
FOLDERS = {
    "train": REPO / "tools/wsp/bass_optical/tracks/train",
    "validation": REPO / "runs/dev/bass_front/val_sealed",
    "test": REPO / "runs/dev/media/fabric/tracks_scorer/psi25/test",
}
SAFFIRE = "Saffire I-III (PSI-98/99/100): sensor tables and images only, no front track in the scorer format"
LABEL = "typed after the final run with every partition seen; in-sample on all data; no held-out score exists"


def sha256(p):
    return hashlib.sha256(Path(p).read_bytes()).hexdigest()


def sh(*a, env=None):
    r = subprocess.run([str(x) for x in a], capture_output=True, text=True, env=env, cwd=REPO)
    if r.returncode != 0:
        sys.exit(f"{' '.join(map(str, a[:3]))} failed:\n{r.stderr}{r.stdout}")
    return r.stdout


def copy_release(rid, reg):
    """Copy a release (and its parent chain) from the lane registry into the private one."""
    while rid:
        src = REPO / "artifacts/models" / rid[7:]
        dst = reg / "models" / rid[7:]
        if not src.exists():
            sys.exit(f"release {rid} is not in artifacts/models")
        if not dst.exists():
            shutil.copytree(src, dst)
        rid = json.loads((src / "release.json").read_text(encoding="utf-8")).get("parent_release_id")


def build_child(base_dir, edits, work, reg):
    base = json.loads((base_dir / "release.json").read_text(encoding="utf-8"))
    if base["model_name"] != "bass_front.wsp5":
        sys.exit("the base must be a bass_front.wsp5 release")
    for rid in (base["id"], D1, CARRY, PERSIST):
        copy_release(rid, reg)
    meta = dict(patch_id=f"front-whatif-{work.name}", parent_release_id=base["id"],
                author=dict(provider="anthropic", model_id="whatif"),
                exposure=dict(sources=["repo:tools/wsp/ext/front_whatif.py(what-if-child;not-a-candidate)"],
                              datasets=[], development_feedback=[], prior_candidates=[]),
                rationale="What-if child, not a candidate.", origin="system")
    (work / "meta.json").write_text(json.dumps(meta, indent=2), encoding="utf-8")
    shutil.copyfile(edits, work / "edits.wsp")
    env = dict(os.environ, IGNIS_ARTIFACTS=str(reg))
    sh(IGNIS, "wsp", "compile", "--family", "bass_front5", "--edits", work / "edits.wsp", "--meta", work / "meta.json",
       "--out", work / "patch.json", env=env)
    out = json.loads(sh(IGNIS, "wsp", "assemble", "--family", "bass_front5", "--patch", work / "patch.json",
                        "--binding", base_dir.parent / "binding.json", "--root", reg, env=env))
    return base["id"], out["release_id"]


def stage(work, name, parts):
    """A folder of copies of the tracks of `parts` and a map naming every file `validation` (the scorer's label)."""
    dest = work / "tracks" / name
    dest.mkdir(parents=True, exist_ok=True)
    pmap, origin = {}, {}
    for part in parts:
        for f in sorted(FOLDERS[part].glob("*.json")):
            d = json.loads(f.read_text(encoding="utf-8"))
            if d.get("schema") != "wsp-bass-front-track/1":
                continue
            stem = f"{part}__{f.stem}"
            shutil.copyfile(f, dest / f"{stem}.json")
            pmap[stem] = "validation"
            origin[d.get("run_id") or stem] = part
    mp = work / "tracks" / f"{name}.map.json"
    mp.write_text(json.dumps(pmap, indent=1), encoding="utf-8")
    return dest, mp, origin


def score(ids, folder, label, out, det, reg, pmap=None):
    env = dict(os.environ, IGNIS_ARTIFACTS=str(reg))
    env.pop("IGNIS_RUNS", None)
    cmd = [EXE, "front-feedback", *sum((["--release", i] for i in ids), []), "--tracks", folder, "--label", label,
           "--quiet-cases", "--out", out, "--detail", det]
    if pmap:
        cmd += ["--partition-map", pmap]
    sh(*cmd, env=env)
    return json.loads(Path(out).read_text(encoding="utf-8")), json.loads(Path(det).read_text(encoding="utf-8"))


def mean(v):
    return sum(v) / len(v) if v else None


def constants(train_detail, rid):
    """Train-only constants: share of tracks that did not spread; share of event pairs ended per horizon."""
    rows = [r for r in train_detail["rows"] if r.get("release_id") == rid]
    ign = [r["ignition_outcome"]["observed_no_spread"] for r in rows
           if "ignition_outcome" in r and "observed_no_spread" in r["ignition_outcome"]]
    by_h = {}
    for r in rows:
        for e in r.get("events") or []:
            by_h.setdefault(e["h_s"], []).append(e["observed"])
    return mean(ign), {h: mean(v) for h, v in by_h.items()}


def constant_scores(detail, rid, c_ign, c_ev):
    """The scorer's aggregation (case mean over pairs, burn mean over cases, mean over burns) for the constants."""
    rows = [r for r in detail["rows"] if r.get("release_id") == rid]
    ign = [(c_ign - r["ignition_outcome"]["observed_no_spread"]) ** 2 for r in rows
           if "ignition_outcome" in r and "observed_no_spread" in r["ignition_outcome"]]
    by_run = {}
    for r in rows:
        ev = r.get("events")
        if ev:
            by_run.setdefault(r["run_id"], []).append(mean([(c_ev[e["h_s"]] - e["observed"]) ** 2 for e in ev]))
    return dict(tip_crps=None, track_ended=mean([mean(v) for v in by_run.values()]), ignition=mean(ign),
                burns_ended=len(by_run), tracks_ignition=len(ign))


def release_scores(report):
    out = {}
    for x in report["releases"]:
        a = x["aggregate"]
        out[x["release_id"]] = dict(tip_crps=a["overall"]["crps"]["mean_over_runs"], burns_tip=a["overall"]["runs"],
                                    track_ended=a["events"]["brier_mean_over_runs"], burns_ended=a["events"]["runs"],
                                    ignition=a["ignition_outcome"]["brier_mean_over_tracks"],
                                    tracks_ignition=a["ignition_outcome"]["tracks"],
                                    ignition_refused=a["ignition_outcome"]["tracks_refused"])
    return out


def f4(x):
    return "     -" if x is None else f"{x:.4f}"


def burn_table(detail, origin, names, track_files):
    """One row per burn: conditions, outcome, and what the releases said."""
    rows = []
    by_run = {}
    for r in detail["rows"]:
        by_run.setdefault(r.get("run_id"), []).append(r)
    for run_id, part in origin.items():
        t = track_files.get(run_id) or {}
        c = t.get("conditions") or {}
        fan = [f for f in t.get("fan") or [] if f.get("t_s") is not None]
        at0 = [f for f in fan if f["t_s"] <= 0] or fan[:1]
        u0 = at0[-1].get("u_cm_s") if at0 else None
        fan0 = at0[-1].get("fan") if at0 else None
        if u0 is None and fan0 is not None and t.get("fan_calibration"):
            cal = sorted([tuple(x) for x in t["fan_calibration"] if x and x[1] is not None] + [(38, 0.0)])
            for (a, ua), (b, ub) in zip(cal, cal[1:]):
                if a <= fan0 <= b and b > a:
                    u0 = ua + (ub - ua) * (fan0 - a) / (b - a)
                    break
        end = t.get("end") or {}
        row = dict(burn=run_id, partition=part, x_o2=c.get("x_o2_initial"), flow_dir=c.get("flow") or "concurrent",
                   fan_at_igniter_off=fan0, u_cm_s_at_igniter_off=None if u0 is None else round(u0, 2),
                   sample=c.get("sample_label"), window_w=(t.get("geometry") or {}).get("window_length_w"),
                   material="SIBAL fabric" if "SIBAL" in (c.get("sample_label") or "") else c.get("material"),
                   spread=None if not end else end.get("kind") != "no_spread", end_kind=end.get("kind"),
                   end_t_s=end.get("t_s"), said={})
        for name, rid in names.items():
            rr = [r for r in by_run.get(run_id, []) if r.get("release_id") == rid]
            ig = [r["ignition_outcome"] for r in rr if "ignition_outcome" in r]
            cases = sorted([r for r in rr if r.get("events")], key=lambda r: (r["cutoff_s"] != 10.0, r["cutoff_s"]))
            ev = {e["h_s"]: e["p_ended"] for e in cases[0]["events"]} if cases else {}
            row["said"][name] = dict(p_no_spread=ig[0].get("p") if ig else None,
                                     refused=ig[0].get("refusal") if ig else "not read",
                                     cutoff_s=cases[0]["cutoff_s"] if cases else None,
                                     p_ended_30s=ev.get(30.0), p_ended_60s=ev.get(60.0))
        rows.append(row)
    return rows


def burns_md(rows, names):
    head = ["burn", "part", "O2", "flow cm/s (fan)", "sample, window w", "material", "spread?", "track ended"]
    for n in names:
        head += [f"{n} P(no spread)", f"{n} P(ended +30/+60 s)"]
    md = ["| " + " | ".join(head) + " |", "|" + "---|" * len(head)]
    for r in rows:
        u = "-" if r["u_cm_s_at_igniter_off"] is None else f"{r['u_cm_s_at_igniter_off']:.1f}"
        end_t = "" if r["end_t_s"] is None else f" {r['end_t_s']:.1f} s"
        cells = [r["burn"], r["partition"], f"{r['x_o2']:.3f}" if r["x_o2"] is not None else "-",
                 f"{u} ({r['fan_at_igniter_off']}) {r['flow_dir']}", f"{r['sample']}, {r['window_w']}", r["material"] or "-",
                 {True: "yes", False: "no", None: "-"}[r["spread"]], f"{r['end_kind']}{end_t}"]
        for n in names:
            s = r["said"][n]
            cells.append(("refused" if s["refused"] else "-") if s["p_no_spread"] is None else f"{s['p_no_spread']:.3f}")
            cells.append("-" if s["cutoff_s"] is None else
                         f"{s['p_ended_30s']:.2f} / {s['p_ended_60s']:.2f} (cut {s['cutoff_s']:g})")
        md.append("| " + " | ".join(cells) + " |")
    return "\n".join(md)


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("--base", required=True, type=Path)
    ap.add_argument("--edits", required=True, type=Path)
    ap.add_argument("--note", required=True)
    ap.add_argument("--run", default=None, help="name of the run folder (default: a timestamp)")
    a = ap.parse_args()
    base_dir = (REPO / a.base) if not a.base.is_absolute() else a.base
    run = a.run or dt.datetime.now(dt.timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    work = WORK / run
    if work.exists():
        sys.exit(f"{work} exists: name another --run")
    work.mkdir(parents=True)
    reg = work / "registry"
    base, child = build_child(base_dir, a.edits, work, reg)
    names = {"base": base, "child": child, "D1": D1, "carry": CARRY, "persistence": PERSIST}
    ids = [D1, base, child, CARRY, PERSIST]
    print(f"front what-if {run}: base {base}  child {child}")
    print(f"({LABEL})")
    print(f"note: {a.note}")

    results, details, origin_all = {}, {}, {}
    for part in ("train", "validation"):
        rep, det = score(ids, FOLDERS[part], f"front what-if {run}: {part}", work / f"{part}.json",
                         work / f"{part}_detail.json", reg)
        results[part], details[part] = rep, det
    for name, parts in (("test", ["test"]), ("pooled", ["train", "validation", "test"])):
        folder, pmap, origin = stage(work, name, parts)
        if name == "pooled":
            origin_all = origin
        rep, det = score(ids, folder, f"front what-if {run}: {name} (study data, 13.8)", work / f"{name}.json",
                         work / f"{name}_detail.json", reg, pmap)
        results[name], details[name] = rep, det

    c_ign, c_ev = constants(details["train"], D1)
    table = {}
    for part, rep in results.items():
        s = release_scores(rep)
        row = {n: s[rid] for n, rid in names.items()}
        row["constant (train)"] = constant_scores(details[part], D1, c_ign, c_ev)
        row["_tracks"] = rep["counts"]["tracks"]
        table[part] = row
    print(f"\nconstants from the train tracks: P(no spread) {c_ign:.3f}; P(ended) by horizon "
          + ", ".join(f"{h:g} s {v:.3f}" for h, v in sorted(c_ev.items())))
    cols = list(names) + ["constant (train)"]
    for part in ("train", "validation", "test", "pooled"):
        row = table[part]
        print(f"\n== {part}: {row['_tracks']} track files ({'study data after the final run' if part in ('test', 'pooled') else part})")
        print(f"  {'':<18}{'tip CRPS':>9} {'burns':>5}  {'ended':>7} {'burns':>5}  {'ignition':>8} {'tracks':>6}")
        for n in cols:
            r = row[n]
            print(f"  {n:<18}{f4(r['tip_crps']):>9} {r.get('burns_tip', '-'):>5}  {f4(r['track_ended']):>7} "
                  f"{r['burns_ended']:>5}  {f4(r['ignition']):>8} {r['tracks_ignition']:>6}")
    print(f"\n== {SAFFIRE}: not scored")

    track_files = {}
    for part in FOLDERS:
        for f in FOLDERS[part].glob("*.json"):
            d = json.loads(f.read_text(encoding="utf-8"))
            if d.get("schema") == "wsp-bass-front-track/1":
                track_files[d.get("run_id") or f.stem] = d
    shown = {"Q0/base": base, "child": child, "D1": D1}
    burns = burn_table(details["pooled"], origin_all, shown, track_files)
    (work / "burns.json").write_text(json.dumps(burns, indent=1), encoding="utf-8")
    md = burns_md(burns, shown)
    (work / "burns.md").write_text(f"# Fabric burns, front what-if {run}\n\n{LABEL}. Base {base}, child {child}.\n"
                                   f"P(no spread) is the forecast at 20 s from cutoff 0 with no stills; P(ended) "
                                   f"is from the 10 s cutoff (or the first the burn has).\n\n{md}\n", encoding="utf-8")
    print("\nper burn (also " + str((work / "burns.md").relative_to(REPO)) + "):\n" + md)

    LOG.parent.mkdir(parents=True, exist_ok=True)
    line = dict(at=dt.datetime.now(dt.timezone.utc).isoformat(), tool="front_whatif", family="bass_front5", run=run,
                base=base, child=child, edits_sha256=sha256(a.edits), edits=a.edits.read_text(encoding="utf-8"),
                note=a.note, binary_sha256=sha256(EXE), label=LABEL,
                scores={p: {n: {k: v for k, v in r.items()} for n, r in row.items() if n != "_tracks"}
                        for p, row in table.items()})
    with LOG.open("a", encoding="utf-8", newline="\n") as f:
        f.write(json.dumps(line) + "\n")
    print(f"\nlogged to {LOG.relative_to(REPO)}")


if __name__ == "__main__":
    main()
