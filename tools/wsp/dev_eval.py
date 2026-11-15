#!/usr/bin/env python3
"""Development evaluation of the literal (WSP) releases against the baselines.

Validation goes through the sealed chain of the evaluator (`ignis infer forecast-cohort --seal`,
`ignis eval compare`); train goes through `wsp-dev feedback`, which scores in-process with the
evaluator's own functions. The test partition is never named here and no command below can read it.

    python tools/wsp/dev_eval.py --baselines runs/dev/baselines.json --out docs/wsp/RESULTS.md

`baselines.json` maps a group key -> {label: release id} for fitted releases that exist in the local
registry (`ignis fit run … --execute`). Nothing in this script changes a parameter.

Every table row is one release: its id, how it was made (zero-shot W0, data-informed revision,
reference baseline) and its scores. Each group also records cohort, split and forward identity.
"""
import argparse
import json
import statistics
import subprocess
import sys
from pathlib import Path

FLEX_SETTINGS = '{"horizon_s":10}'
EVENT_TIMES = [6, 8, 10, 12, 15, 20, 25, 30, 40, 50, 60, 80, 100, 120]

# (group key, title, event model?, binding file, [(directory, candidate, label, how)])
GROUPS = [
    ("flex", "FLEX methanol (PSI-69)", False, "configs/models/flex/wsp/binding.json", [
        ("configs/models/flex/wsp2/methanol", "d3-fable", "flex_methanol.wsp2 D3", "data-informed (train)"),
        ("configs/models/flex/wsp2/methanol", "baseline-d2", "flex_methanol.wsp2 D2-law", "hand-written reference"),
        ("configs/models/flex/wsp/methanol", "d2-fable", "flex_methanol.wsp D2", "data-informed (train; spread widened after D1's aggregate validation coverage)"),
        ("configs/models/flex/wsp/methanol", "d1-fable", "flex_methanol.wsp D1", "data-informed (train)"),
        ("configs/models/flex/wsp/methanol", "baseline-d2", "flex_methanol.wsp D2-law", "hand-written reference"),
        ("configs/models/flex/wsp", "w2-fable", "flex.wsp W2", "data-informed (train + validation, phase 1)"),
        ("configs/models/flex/wsp", "w1-fable", "flex.wsp W1", "data-informed (train, phase 1)"),
        ("configs/models/flex/wsp", "w0-fable", "flex.wsp W0", "zero-shot"),
    ]),
    ("flex_heptane", "FLEX n-heptane (PSI-69)", False, "configs/models/flex/wsp/heptane/binding.json", [
        ("configs/models/flex/wsp2/heptane", "d2-fable", "flex_heptane.wsp2 D2", "data-informed (train)"),
        ("configs/models/flex/wsp2/heptane", "baseline-d2", "flex_heptane.wsp2 D2-law", "hand-written reference"),
        ("configs/models/flex/wsp/heptane", "d1-fable", "flex_heptane.wsp D1", "data-informed (train)"),
        ("configs/models/flex/wsp/heptane", "w0-fable", "flex_heptane.wsp W0", "zero-shot on heptane (literature)"),
        ("configs/models/flex/wsp/heptane", "baseline-d2", "flex_heptane.wsp D2-law", "hand-written reference"),
    ]),
    # Fourth-generation droplets are their own groups: their prefix carries the sheet's initial
    # diameter, so `ignis eval compare` cannot pair their cases with older releases. The carry
    # releases hold the third-generation numbers on the new forward (identical forecasts).
    ("flex_gen4", "FLEX methanol, fourth generation (PSI-69)", False, "configs/models/flex/wsp3/methanol/binding.json", [
        ("configs/models/flex/wsp3/methanol", "d4-fable", "flex_methanol.wsp3 D4", "data-informed (train)"),
        ("configs/models/flex/wsp3/methanol", "d5-flags", "flex_methanol.wsp3 D5", "data-informed (train); the third-generation numbers plus the broad component only; written after the first candidate failed on validation"),
        ("configs/models/flex/wsp3/methanol", "carry-d3", "flex_methanol.wsp3 carry of D3", "the third-generation numbers on the fourth forward"),
        ("configs/models/flex/wsp3/methanol", "baseline-d2", "flex_methanol.wsp3 D2-law", "hand-written reference"),
    ]),
    ("flex_heptane_gen4", "FLEX n-heptane, fourth generation (PSI-69)", False, "configs/models/flex/wsp3/heptane/binding.json", [
        ("configs/models/flex/wsp3/heptane", "d3-fable", "flex_heptane.wsp3 D3", "data-informed (train)"),
        # heptane d4-flags (the broad component alone) failed the train rule and is not scored on validation:
        # it is deliberately not a member of this group (PROTOCOL.md section 9)
        ("configs/models/flex/wsp3/heptane", "carry-d2", "flex_heptane.wsp3 carry of D2", "the third-generation numbers on the fourth forward"),
        ("configs/models/flex/wsp3/heptane", "baseline-d2", "flex_heptane.wsp3 D2-law", "hand-written reference"),
    ]),
    ("acme_normal", "ACME normal flames (PSI-159)", True, "configs/models/acme/wsp/normal-binding.json", [
        ("configs/models/acme/wsp4/normal", "d1-fable", "acme_normal.wsp4 D1", "data-informed (train); wsp2 D3 and wsp3 D1 with equal weights; written after both members' aggregate validation scores were seen"),
        ("configs/models/acme/wsp4/normal", "baseline-pair", "acme_normal.wsp4 baseline pair", "the two reference baselines with equal weights, hand-written reference"),
        ("configs/models/acme/wsp3/normal", "d1-fable", "acme_normal.wsp3 D1", "data-informed (train; published theory)"),
        ("configs/models/acme/wsp3/normal", "baseline-theory", "acme_normal.wsp3 theory", "published constants only, hand-written reference"),
        ("configs/models/acme/wsp2/normal", "d3-fable", "acme_normal.wsp2 D3", "data-informed (train)"),
        ("configs/models/acme/wsp2/normal", "d2-fable", "acme_normal.wsp2 D2", "data-informed (train; clocks widened after D1's aggregate validation scores)"),
        ("configs/models/acme/wsp2/normal", "d1-fable", "acme_normal.wsp2 D1", "data-informed (train)"),
        ("configs/models/acme/wsp2/normal", "baseline-clock", "acme_normal.wsp2 baseline clock", "hand-written reference"),
        ("configs/models/acme/wsp", "normal-w0-fable", "acme_normal.wsp W0", "zero-shot"),
        ("configs/models/acme/wsp", "normal-w1-fable", "acme_normal.wsp W1", "data-informed (train, phase 1)"),
        ("configs/models/acme/wsp", "normal-w2-fable", "acme_normal.wsp W2", "data-informed (train, phase 1)"),
    ]),
    ("acme_inverse", "ACME inverse flames (PSI-159)", True, "configs/models/acme/wsp/inverse-binding.json", [
        ("configs/models/acme/wsp4/inverse", "d1-fable", "acme_inverse.wsp4 D1", "data-informed (train); wsp2 D1 and wsp3 D1 with equal weights; written after both members' aggregate validation scores were seen"),
        ("configs/models/acme/wsp4/inverse", "baseline-pair", "acme_inverse.wsp4 baseline pair", "the two reference baselines with equal weights, hand-written reference"),
        ("configs/models/acme/wsp3/inverse", "d1-fable", "acme_inverse.wsp3 D1", "data-informed (train; published theory)"),
        ("configs/models/acme/wsp3/inverse", "baseline-theory", "acme_inverse.wsp3 theory", "published constants only, hand-written reference"),
        ("configs/models/acme/wsp2/inverse", "d3-fable", "acme_inverse.wsp2 D3", "data-informed (train)"),
        ("configs/models/acme/wsp2/inverse", "d2-fable", "acme_inverse.wsp2 D2", "data-informed (train; clocks widened after D1's aggregate validation scores)"),
        ("configs/models/acme/wsp2/inverse", "d1-fable", "acme_inverse.wsp2 D1", "data-informed (train)"),
        ("configs/models/acme/wsp2/inverse", "baseline-clock", "acme_inverse.wsp2 baseline clock", "hand-written reference"),
        ("configs/models/acme/wsp", "inverse-w0-fable", "acme_inverse.wsp W0", "zero-shot"),
        ("configs/models/acme/wsp", "inverse-w1-fable", "acme_inverse.wsp W1", "data-informed (train, phase 1)"),
    ]),
    # Fifth generation. These groups are their own because their prefixes differ from the older specs':
    # a committee prefix may carry the script's planned stop, a droplet6 prefix the flame-camera series.
    # Those inputs exist only in the private copy of the data root: run with IGNIS_DATA=runs/dev/data_ext
    # to score them; on the shared root the same releases answer without them. The carry releases hold
    # the carried numbers on the new spec (identical forecasts) and are the comparison partners.
    # NOT members, on purpose: acme_inverse.wsp5 d1-fable (a superseded, never registered committee).
    # Since the look of 14 November the burner-flame releases of these groups are sealed on BOTH roots, and
    # the evaluator keys a seal by run and prefix: the validation table this script writes for them holds
    # every run twice (once with, once without the planned stop) and is not to be quoted. Split it with
    # tools/wsp/analysis/cmp_split.py (docs/wsp/dossier/09_reproduce.md, 9.7).
    ("acme_normal_gen5", "ACME normal flames, fifth generation (PSI-159)", True, "configs/models/acme/wsp5/normal/binding.json", [
        ("configs/models/acme/wsp5/normal", "p1-fable", "acme_normal.wsp5 P1", "data-informed (train); the carried pair of laws with corrected burner-limit weights; reads the script's planned stop when the prefix carries it"),
        ("configs/models/acme/wsp5/normal", "carry-d1", "acme_normal.wsp5 carry of wsp4 D1", "the carried model on the committee spec"),
        ("configs/models/acme/wsp5/normal", "baseline-set", "acme_normal.wsp5 baseline set", "the reference baselines, hand-written reference"),
    ]),
    # Sixth generation of the normal flames (15 November), on the second committee spec. carry-p1 is the
    # carried acme_normal.wsp5 P1 with the same members and weights, for a paired comparison.
    ("acme_normal_gen6", "ACME normal flames, sixth generation (PSI-159)", True, "configs/models/acme/wsp7/normal/binding.json", [
        ("configs/models/acme/wsp7/normal", "p3-fable", "acme_normal.wsp7 P3", "data-informed (train); P2 with the hinge member answering through the burner-heating law"),
        ("configs/models/acme/wsp7/normal", "p2-fable", "acme_normal.wsp7 P2", "data-informed (train); burner limit by the date of the test, planned stop after the stage sum, three statements corrected"),
        ("configs/models/acme/wsp7/normal", "carry-p1", "acme_normal.wsp7 carry of wsp5 P1", "the carried model on the second committee spec"),
        ("configs/models/acme/wsp7/normal", "baseline-set", "acme_normal.wsp7 baseline set", "the reference baselines, hand-written reference"),
    ]),
    # The look of PROTOCOL 12.10 on the third committee spec. The one-member groups are for the path with the
    # planned stop, where F2 and F1 read different inputs (data_ext2 and data_ext) and cannot be paired.
    ("acme_normal_f2", "ACME normal flames, F2 against F1 (PSI-159)", True, "configs/models/acme/wsp12/normal/binding.json", [
        ("configs/models/acme/wsp12/normal", "f2-fable", "acme_normal.wsp12 F2", "data-informed (train days of Flame Design; CFI-G train)"),
        ("configs/models/acme/wsp12/normal", "f1-fable", "acme_normal.wsp12 F1", "the adopted P3 with the fuel table"),
    ]),
    ("acme_normal_f2_only", "ACME normal flames, F2 alone (PSI-159)", True, "configs/models/acme/wsp12/normal/binding.json", [
        ("configs/models/acme/wsp12/normal", "f2-fable", "acme_normal.wsp12 F2", "data-informed (train days of Flame Design; CFI-G train)"),
    ]),
    ("acme_normal_f1_only", "ACME normal flames, F1 alone (PSI-159)", True, "configs/models/acme/wsp12/normal/binding.json", [
        ("configs/models/acme/wsp12/normal", "f1-fable", "acme_normal.wsp12 F1", "the adopted P3 with the fuel table"),
    ]),
    ("acme_inverse_f2", "ACME inverse flames, F2 against F1 (PSI-159)", True, "configs/models/acme/wsp12/inverse/binding.json", [
        ("configs/models/acme/wsp12/inverse", "f2-fable", "acme_inverse.wsp12 F2", "data-informed (CFI-G and Flame Design train: the planned stop)"),
        ("configs/models/acme/wsp12/inverse", "f1-fable", "acme_inverse.wsp12 F1", "the adopted P1 with the fuel table"),
    ]),
    ("acme_inverse_f2_only", "ACME inverse flames, F2 alone (PSI-159)", True, "configs/models/acme/wsp12/inverse/binding.json", [
        ("configs/models/acme/wsp12/inverse", "f2-fable", "acme_inverse.wsp12 F2", "data-informed (CFI-G and Flame Design train: the planned stop)"),
    ]),
    ("acme_inverse_f1_only", "ACME inverse flames, F1 alone (PSI-159)", True, "configs/models/acme/wsp12/inverse/binding.json", [
        ("configs/models/acme/wsp12/inverse", "f1-fable", "acme_inverse.wsp12 F1", "the adopted P1 with the fuel table"),
    ]),
    ("acme_inverse_gen5", "ACME inverse flames, fifth generation (PSI-159)", True, "configs/models/acme/wsp5/inverse/binding.json", [
        ("configs/models/acme/wsp5/inverse", "p1-fable", "acme_inverse.wsp5 P1", "data-informed (train); the carried law (weight 2) and a limit-cycle scenario (weight 1); reads the script's planned stop when the prefix carries it"),
        ("configs/models/acme/wsp5/inverse", "carry-d1", "acme_inverse.wsp5 carry of wsp3 D1", "the carried model on the committee spec"),
        ("configs/models/acme/wsp5/inverse", "baseline-set", "acme_inverse.wsp5 baseline set", "the reference baselines, hand-written reference"),
    ]),
    ("flex_gen5", "FLEX methanol, fifth generation (PSI-69)", False, "configs/models/flex/wsp5/methanol/binding.json", [
        ("configs/models/flex/wsp5/methanol", "p1-fable", "flex_methanol.wsp5 P1", "data-informed (train); physics-first laws, flame cameras read when the prefix carries them"),
        # methanol p0-fable (the laws without the cameras) failed the train rule on the median and is not scored on
        # validation: it is deliberately not a member of this group (PROTOCOL.md section 10)
        ("configs/models/flex/wsp5/methanol", "carry-d3", "flex_methanol.wsp5 carry of D3", "the carried numbers on the sixth droplet forward"),
        ("configs/models/flex/wsp5/methanol", "baseline-d2", "flex_methanol.wsp5 D2-law", "hand-written reference"),
    ]),
    ("flex_heptane_gen5", "FLEX n-heptane, fifth generation (PSI-69)", False, "configs/models/flex/wsp5/heptane/binding.json", [
        ("configs/models/flex/wsp5/heptane", "p1-fable", "flex_heptane.wsp5 P1", "data-informed (train); physics-first laws, flame cameras read when the prefix carries them"),
        ("configs/models/flex/wsp5/heptane", "p0-fable", "flex_heptane.wsp5 P0", "data-informed (train); the same laws, cameras not read"),
        ("configs/models/flex/wsp5/heptane", "carry-d2", "flex_heptane.wsp5 carry of D2", "the carried numbers on the sixth droplet forward"),
        ("configs/models/flex/wsp5/heptane", "baseline-d2", "flex_heptane.wsp5 D2-law", "hand-written reference"),
    ]),
]


def exe(path: str) -> str:
    p = Path(path)
    if not p.exists() and p.with_suffix(".exe").exists():
        p = p.with_suffix(".exe")
    return str(p.resolve())


def run(cmd, check=True):
    out = subprocess.run(cmd, capture_output=True, text=True)
    if check and out.returncode != 0:
        sys.exit(f"{' '.join(cmd)} failed:\n{out.stderr[-2000:]}{out.stdout[-2000:]}")
    return out.stdout


BASS_LABELS = {
    "sha256:ac15e49f1cb60d4b4679bcdff82a07c18a4a501454428e35ac86e935e168855c": "bass_front.wsp D1 (data-informed (train; published theory))",
    "sha256:0187941810a89347a9ea3953add9aa6058152bb643b009ca1ffa3e9471f216a7": "bass_front.wsp D2 (D1 without the oxygen quench boundary; written after D1's aggregate validation scores, not blind)",
    "sha256:ce53fe1fb3e6300c5e818dd7c9c5ec8e8f4c2cfd19673449d3936de05527de46": "bass_front.wsp persistence (hand-written reference)",
}


def bass_section(work: Path) -> list:
    """The BASS section: the front model's aggregate scores, read from the files `wsp-dev
    front-feedback` wrote (`front_train.json`, `front_validation.json`). Aggregates only."""
    md = ["## BASS (PSI-25)", "",
          "`bass_front.wsp` forecasts the tip of the flame in sample widths, with the fan history as a known "
          "control, on front tracks measured from the public videos (`wsp-dev front-feedback`; rules in "
          "PROTOCOL.md section 9). Case = mean over its target stills, burn = mean over its cases, the table "
          "gives the mean over burns. The validation tracks were measured by agents that knew nothing of the "
          "model; the author has not opened them. `bass.wsp` W0 (first generation) stays runnable on metric "
          "contours and unscored: no run has a length calibration.", ""]
    for name, title in (("front_validation.json", "Validation"), ("front_train.json", "Train")):
        path = work / name
        if not path.exists():
            md += [f"**{title}** — not scored yet (`{path}` does not exist).", ""]
            continue
        d = json.loads(path.read_text())
        c = d["counts"]
        reasons = ", ".join(f"{v} {k.replace('_', ' ')}" for k, v in sorted(c["tracks_unscoreable_by_reason"].items())) or "none"
        md += [f"**{title}** — {c['runs_with_cases']} burns with cases of {c['tracks']} tracks, {c['cases']} cases "
               f"(not scoreable: {reasons}). Sample widths.", "",
               "| release | CRPS | abs. error of median | coverage 50 % | coverage 90 % | track ended (Brier) | spread or not (Brier) |",
               "|---|---|---|---|---|---|---|"]
        for r in d["releases"]:
            a = r["aggregate"]
            o = a["overall"]
            md.append(f"| {BASS_LABELS.get(r['release_id'], r['release_id'])} | {o['crps']['mean_over_runs']:.3f} | "
                      f"{o['ae_median']['mean_over_runs']:.3f} | {o['coverage_50']['mean_over_runs']:.2f} | "
                      f"{o['coverage_90']['mean_over_runs']:.2f} | {a['events']['brier_mean_over_runs']:.3f} | "
                      f"{a['ignition_outcome']['brier_mean_over_tracks']:.3f} |")
        md.append("")
        for pv in d.get("paired_vs_first", []):
            sc = pv["scores"]["crps"]
            lo, hi = sc["ci95_bootstrap_over_runs"]
            md.append(f"Paired difference to {BASS_LABELS.get(pv['first_release_id'], 'the first release')} (other − reference, "
                      f"sample widths, 95 % bootstrap over burns): "
                      f"{BASS_LABELS.get(pv['release_id'], pv['release_id'])}: CRPS {sc['mean_difference_over_runs']:+.3f} "
                      f"[{lo:+.3f}, {hi:+.3f}] ({pv['common_cases']} cases, {sc['runs']} burns; reference better in "
                      f"{sc['runs_where_first_lower']}).")
            md.append("")
        best = d["releases"][0]["aggregate"]
        md.append("By cutoff (CRPS, " + BASS_LABELS.get(d["releases"][0]["release_id"], "first release").split(" (")[0] + "): "
                  + ", ".join(f"{k} s {v['crps']['mean_over_runs']:.3f}" for k, v in sorted(best["by_cutoff_s"].items(), key=lambda kv: float(kv[0])))
                  + ". By lead: " + ", ".join(f"{k} s {v['crps']['mean_over_runs']:.3f}" for k, v in best["by_lead_band_s"].items()) + ".")
        md.append("")
    return md


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--ignis", default="target/release/ignis")
    ap.add_argument("--wsp-dev", default="target/release/wsp-dev")
    ap.add_argument("--baselines", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--work", default="runs/dev")
    ap.add_argument("--only", default="", help="comma-separated group keys (default: all)")
    ap.add_argument("--skip-validation", action="store_true", help="train tables only")
    ap.add_argument("--bass-only", action="store_true",
                    help="rewrite only the BASS section of an existing --out file from the front-feedback aggregates")
    args = ap.parse_args()
    if args.bass_only:
        out = Path(args.out)
        text = out.read_text(encoding="utf-8")
        cut = text.find("## BASS")
        if cut < 0:
            sys.exit(f"{out} has no BASS section")
        out.write_text(text[:cut] + chr(10).join(bass_section(Path(args.work))), encoding="utf-8", newline=chr(10))
        print(f"BASS section of {out} rewritten")
        return 0
    ignis, wspdev = exe(args.ignis), exe(args.wsp_dev)
    baselines = json.loads(Path(args.baselines).read_text())
    work = Path(args.work)
    work.mkdir(parents=True, exist_ok=True)
    only = {k for k in args.only.split(",") if k}
    md = ["# Development results of the literal (WSP) releases", "",
          "Produced by `tools/wsp/dev_eval.py`. Validation: sealed forecasts scored and paired by "
          "`ignis eval compare` (group-equal means, cluster bootstrap; the validation partitions have few "
          "independent clusters, so intervals are wide). Train: `wsp-dev feedback` (case means, evaluator "
          "scoring functions). Cutoffs 1, 3 and 5 s. **The test partition was not read.** A release marked "
          "data-informed was written after its author read development data; only releases marked "
          "zero-shot were written before any outcome of their fuel or family was seen.", ""]
    for key, title, event, binding_path, members in GROUPS:
        if only and key not in only:
            continue
        binding = json.loads(Path(binding_path).read_text())
        split = binding["split_id"]
        rel = {}
        for d, cand, label, how in members:
            path = Path(d) / cand / "release.json"
            if not path.exists():
                print(f"{key}: {label} skipped, {path} does not exist yet")
                continue
            r = json.loads(path.read_text())
            rel[r["id"]] = (label, how, r)
        base = baselines.get(key, {})
        label = {rid: f"{v[0]} ({v[1]})" for rid, v in rel.items()}
        label.update({rid: f"{name} (fitted on train)" for name, rid in base.items()})
        settings = json.dumps({"times_s": EVENT_TIMES}) if event else FLEX_SETTINGS
        ids = list(rel) + list(base.values())
        md += [f"## {title}", "", f"Cohort `{binding['cohort_id']}` · split `{split}`.", "",
               "| release | id | forward | nonzero / authored |", "|---|---|---|---|"]
        for rid, (lab, how, r) in rel.items():
            n = r["architecture"]["parameters"]
            fwd = r["construction"]["forward_hash"]
            md.append(f"| {lab} — {how} | `{rid}` | `{fwd[:15]}…` | {n['authored_nonzero']} / {n['authored_total']} |")
        for name, rid in base.items():
            md.append(f"| {name} — fitted on train | `{rid}` | — | fitted |")
        md.append("")
        if not args.skip_validation:
            for rid in rel:
                run([ignis, "infer", "forecast-cohort", "--release", rid, "--split", split, "--partition", "validation",
                     "--cutoffs", "1,3,5", "--settings", settings, "--execute", "--seal"])
            cmp_path = work / f"cmp_validation_{key}.json"
            cmd = [ignis, "eval", "compare", *ids, "--partition", "validation", "--allow-feature-mismatch",
                   "--allow-unit-mismatch", "--allow-cohort-mismatch", "--intersect", "--out", str(cmp_path)]
            if event:
                cmd += ["--horizons", ",".join(map(str, EVENT_TIMES))]
            run(cmd)
            cmp = json.loads(cmp_path.read_text())
        fb_path = work / f"fb_train_{key}.json"
        cmd = [wspdev, "feedback", *sum((["--release", r] for r in ids), []), "--partition", "train", "--out", str(fb_path)]
        if event:
            cmd += ["--times", ",".join(map(str, EVENT_TIMES))]
        run(cmd)
        fb = json.loads(fb_path.read_text())
        if not event:
            cols = ["crps", "mae_median", "coverage@0.5", "coverage@0.9", "lpd"]
            um = lambda k, v: f"{v * 1e6:.1f}" if k in ("crps", "mae_median") else f"{v:.2f}"
            if not args.skip_validation:
                md += [f"**Validation** — {cmp['n_cases']} common cases; CRPS and MAE in µm.", "",
                       "| release | CRPS | MAE of median | coverage 50 % | coverage 90 % | lpd |", "|---|---|---|---|---|---|"]
                for rid, block in cmp["per_release"].items():
                    m = block["metrics"]
                    md.append(f"| {label[rid]} | " + " | ".join(um(k, m[k]["estimate"]) if k in m else "-" for k in cols) + " |")
                ref = label[cmp["reference"]]
                md += ["", f"Paired difference to {ref} (other − reference, µm, 95 % CI):", ""]
                for rid, block in cmp["paired_vs_reference"].items():
                    d = block["differences"].get("crps")
                    if d:
                        md.append(f"- {label[rid]}: CRPS {d['estimate'] * 1e6:+.1f} [{d['ci'][0] * 1e6:+.1f}, {d['ci'][1] * 1e6:+.1f}] "
                                  f"({d['n_cases']} cases, {d['n_clusters']} clusters)")
                md.append("")
            md += ["**Train** — µm. The mean is dominated by a few artefact runs (soot shell, gas bubble, droplet "
                   "leaving and re-entering the view); the median and the mean without the worst 5 % of cases are "
                   "given beside it.", "",
                   "| release | cases | refused | CRPS mean | CRPS median | CRPS mean, worst 5 % dropped | MAE of median | coverage 50 % | coverage 90 % |",
                   "|---|---|---|---|---|---|---|---|---|"]
            for r in fb["releases"]:
                cs = [c for c in r["cases"] if "crps" in c["metrics"]]
                if not cs:
                    md.append(f"| {label[r['release_id']]} | 0 | {len(r['refused'])} | - | - | - | - | - | - |")
                    continue
                v = sorted(c["metrics"]["crps"] * 1e6 for c in cs)
                trim = v[:max(1, int(len(v) * 0.95))]
                mm = lambda k: statistics.mean(c["metrics"][k] for c in cs)
                md.append(f"| {label[r['release_id']]} | {len(cs)} | {len(r['refused'])} | {statistics.mean(v):.1f} | "
                          f"{statistics.median(v):.1f} | {statistics.mean(trim):.1f} | {mm('mae_median') * 1e6:.1f} | "
                          f"{mm('coverage@0.5'):.2f} | {mm('coverage@0.9'):.2f} |")
        else:
            ibs = lambda block, c: block.get("events", {}).get("causes", {}).get(c, {}).get("integrated_brier")
            if not args.skip_validation:
                md += [f"**Validation** — {cmp['n_cases']} common cases; integrated Brier score over "
                       f"{EVENT_TIMES[0]}–{EVENT_TIMES[-1]} s (lower is better).", "",
                       "| release | extinction | operator termination |", "|---|---|---|"]
                for rid, block in cmp["per_release"].items():
                    md.append(f"| {label[rid]} | {ibs(block, 'extinction'):.4f} | {ibs(block, 'operator_termination'):.4f} |")
                md.append("")
            md += ["**Train** — integrated Brier score.", "", "| release | cases | refused | extinction | operator termination |",
                   "|---|---|---|---|---|"]
            for r in fb["releases"]:
                md.append(f"| {label[r['release_id']]} | {len(r['cases'])} | {len(r['refused'])} | "
                          f"{ibs(r['aggregate'], 'extinction'):.4f} | {ibs(r['aggregate'], 'operator_termination'):.4f} |")
        md.append("")
    md += bass_section(work)
    Path(args.out).parent.mkdir(parents=True, exist_ok=True)
    Path(args.out).write_text("\n".join(md), encoding="utf-8", newline="\n")
    print(f"written {args.out}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
