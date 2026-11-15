#!/usr/bin/env python3
"""Train-only loss diagnostics of a literal (WSP) release: where the loss sits, and which numbers can move it.

Diagnostic only. A perturbation is a question about the model, not a proposed value. This script never
writes a weight into a model, never proposes a value, never touches `configs/` or `artifacts/models/`.

    python tools/wsp/attrib/attrib.py decompose   --family acme_normal7 --candidate configs/models/acme/wsp7/normal/p3-fable
    python tools/wsp/attrib/attrib.py sensitivity --family flex_methanol2 --candidate configs/models/flex/wsp2/methanol/d3-fable \
        --blocks tools/wsp/attrib/examples/d3.blocks

Both subcommands read the TRAIN partition only (`wsp-dev feedback --partition train`, or `wsp-dev front-feedback`
on the train track folder for `bass_front*`). `--final-test` is never passed; `holdout/` and
`runs/dev/holdout_ext/` are never opened. Every forward run happens against a scratch registry
(`IGNIS_ARTIFACTS` pointed at a copy of the candidate under the output folder), so even the feedback runs
cannot write into the real registry. Usage and column meanings: tools/wsp/attrib/README.md.
"""
import argparse
import concurrent.futures as cf
import hashlib
import json
import math
import os
import re
import shutil
import statistics
import subprocess
import sys
import time
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
sys.dont_write_bytecode = True  # write nothing outside tools/wsp/attrib and runs/dev/attrib
sys.path.insert(0, str(ROOT / "tools" / "wsp" / "analysis"))
import lib  # noqa: E402  (train-only accessors of the records)

IGNIS = ROOT / "target" / "release" / ("ignis.exe" if os.name == "nt" else "ignis")
WSPDEV = ROOT / "target" / "release" / ("wsp-dev.exe" if os.name == "nt" else "wsp-dev")
REGISTRY = ROOT / "artifacts" / "models"
OUT_ROOT = ROOT / "runs" / "dev" / "attrib"
BASS_TRAIN_TRACKS = ROOT / "tools" / "wsp" / "bass_optical" / "tracks" / "train"
EVENT_TIMES = [6, 8, 10, 12, 15, 20, 25, 30, 40, 50, 60, 80, 100, 120]  # as tools/wsp/dev_eval.py
BANNER = "Diagnostic only. A perturbation is a question about the model, not a proposed value."
FORBIDDEN_PARTS = {"holdout", "holdout_ext"}


# ------------------------------------------------------------------------------------------ guards

def guard_path(p: Path, what: str) -> Path:
    p = Path(p).resolve()
    if any(part.lower() in FORBIDDEN_PARTS for part in p.parts):
        sys.exit(f"refused: {what} {p} is inside a held-out folder")
    return p


def guard_out(p: Path) -> Path:
    p = guard_path(p, "output")
    if OUT_ROOT.resolve() not in (p, *p.parents):
        sys.exit(f"refused: outputs go under {OUT_ROOT}, not {p}")
    return p


def kind_of(family: str) -> str:
    if family.startswith("bass_front"):
        return "front"
    if family.startswith("acme_"):
        return "event"
    if family.startswith("flex_"):
        return "droplet"
    sys.exit(f"family {family}: not one of flex_*, acme_*, bass_front*")


def records_name(family: str) -> str:
    for k in ("acme_normal", "acme_inverse"):
        if family.startswith(k):
            return k
    return "flex" if family.startswith("flex") else "bass"


def registry_snapshot() -> dict:
    """What the isolation check compares before and after: the registry listing and the candidate files."""
    names = sorted(os.listdir(REGISTRY)) if REGISTRY.is_dir() else []
    return {"artifacts_models_entries": len(names),
            "artifacts_models_listing_sha256": hashlib.sha256("\n".join(names).encode()).hexdigest()}


def tree_hash(d: Path) -> str:
    h = hashlib.sha256()
    for f in sorted(p for p in d.rglob("*") if p.is_file()):
        h.update(str(f.relative_to(d)).encode())
        h.update(f.read_bytes())
    return h.hexdigest()


# ------------------------------------------------------------------------------------------ candidate

class Candidate:
    def __init__(self, family: str, cand_dir: str):
        self.family = family
        self.kind = kind_of(family)
        self.dir = guard_path(ROOT / cand_dir, "candidate")
        rel = self.dir / "release.json"
        if not rel.exists():
            sys.exit(f"{rel} does not exist (assemble the candidate first: python tools/wsp/reassemble.py)")
        self.release = json.loads(rel.read_text(encoding="utf-8"))
        self.id = self.release["id"]
        self.model_name = self.release["model_name"]
        if self.model_name.replace(".wsp", "") != family:
            sys.exit(f"--family {family} does not match the candidate's model {self.model_name}")
        self.binding = self.dir.parent / "binding.json"
        if not self.binding.exists():
            sys.exit(f"no binding next to the candidate: {self.binding}")
        self.meta = json.loads((self.dir / "meta.json").read_text(encoding="utf-8"))
        self.reg_dir = REGISTRY / self.id.split(":", 1)[1]
        if not (self.reg_dir / "params.npz").exists():
            sys.exit(f"{self.id} is not in the local registry (python tools/wsp/reassemble.py)")
        stored = json.loads((self.reg_dir / "release.json").read_text(encoding="utf-8"))
        if stored["construction"]["weights_hash"] != self.release["construction"]["weights_hash"]:
            sys.exit("the registry copy of the candidate differs from its release.json")
        self.label = f"{self.model_name} {self.dir.name}"

    def tensors(self):
        import numpy as np
        with np.load(self.reg_dir / "params.npz") as z:
            return {k: z[k].copy() for k in z.files}


def scratch_registry(where: Path, cand: Candidate) -> Path:
    """An isolated IGNIS_ARTIFACTS holding a copy of the candidate only."""
    models = where / "models"
    models.mkdir(parents=True, exist_ok=True)
    dst = models / cand.reg_dir.name
    if not dst.exists():
        shutil.copytree(cand.reg_dir, dst)
    return where


# ------------------------------------------------------------------------------------------ feedback

def run(cmd, env_artifacts: Path, data_root=None):
    env = dict(os.environ)
    env["IGNIS_ARTIFACTS"] = str(env_artifacts)
    if data_root:
        env["IGNIS_DATA"] = str(data_root)
    assert "--final-test" not in cmd
    out = subprocess.run([str(c) for c in cmd], capture_output=True, text=True, env=env, cwd=ROOT)
    if out.returncode != 0:
        msg = out.stderr.strip() or out.stdout.strip()
        msg = msg[msg.find("Error:"):] if "Error:" in msg else msg
        raise RuntimeError(f"{Path(str(cmd[0])).stem} {cmd[1]}: {msg[-1500:]}")
    return out.stdout


def feedback(cand: Candidate, release_id: str, artifacts: Path, out: Path, data_root, n_boot: int):
    """Train feedback of one release; returns (raw report, normalised cases, reported totals)."""
    if cand.kind == "front":
        tracks = guard_path(BASS_TRAIN_TRACKS, "tracks")
        for f in sorted(tracks.glob("T*.json")):
            part = json.loads(f.read_text(encoding="utf-8")).get("partition")
            if part != "train":
                sys.exit(f"refused: track {f} is partition {part!r}, not train")
        detail = out.with_name(out.stem + "_detail.json")
        run([WSPDEV, "front-feedback", "--release", release_id, "--tracks", tracks, "--label", "attrib-train",
             "--out", out, "--detail", detail], artifacts)
        agg = json.loads(out.read_text(encoding="utf-8"))
        det = json.loads(detail.read_text(encoding="utf-8"))
        return {"aggregate": agg, "detail": det}, front_cases(det), front_totals(agg)
    cmd = [WSPDEV, "feedback", "--release", release_id, "--partition", "train", "--n-boot", str(n_boot), "--out", out]
    if cand.kind == "event":
        cmd += ["--times", ",".join(map(str, EVENT_TIMES))]
    run(cmd, artifacts, data_root)
    fb = json.loads(out.read_text(encoding="utf-8"))
    if fb.get("partition") != "train":
        sys.exit("feedback did not report the train partition")
    r = fb["releases"][0]
    if cand.kind == "event":
        causes = fb["spec"]["causes"]
        return fb, event_cases(r, causes), event_totals(r, causes)
    return fb, droplet_cases(r, fb["spec"]), droplet_totals(r)


def centred_coefficients(levels):
    """CRPS of the evaluator's quantile score when the observation sits at the median, as a linear function
    of the central band widths: crps_quantiles at y = q50 equals sum over symmetric pairs (tau, 1-tau) of
    2 * w_tau * tau * width, w_tau the midpoint weight of crps_quantiles (ignis_lab::eval::scores)."""
    t = sorted(levels)
    ext = [0.0, *t, 1.0]
    w = {t[k]: (ext[k + 2] - ext[k]) / 2 for k in range(len(t))}
    coef = {}
    for tau in t:
        if tau < 0.5 and any(abs(u - (1 - tau)) < 1e-12 for u in t):
            nominal = round(1 - 2 * tau, 10)
            coef[f"width@{nominal:g}"] = 2 * w[tau] * tau
    return coef


def droplet_cases(r, spec):
    coef = centred_coefficients(spec.get("levels") or [])
    cases = []
    for c in r["cases"]:
        m = c["metrics"]
        if "crps" not in m:
            continue
        b = c.get("bias") or {}
        n = b.get("n") or [0, 0, 0]
        vals = [b.get("lead_0_2s_um"), b.get("lead_2_5s_um"), b.get("lead_5_10s_um")]
        num = sum(v * k for v, k in zip(vals, n) if v is not None)
        den = sum(k for v, k in zip(vals, n) if v is not None)
        centred = sum(a * m[k] for k, a in coef.items() if k in m) if coef and all(k in m for k in coef) else None
        cases.append({
            "run_id": c["run_id"], "cutoff_s": c["cutoff_s"], "session": c.get("cluster") or c.get("group_id"),
            "conditions": c.get("conditions") or {}, "loss": m["crps"], "parts": {"crps": m["crps"]},
            "mae_median": m.get("mae_median"), "cov50": m.get("coverage@0.5"), "cov90": m.get("coverage@0.9"),
            "bias_um": num / den if den else None, "crps_centred": centred,
        })
    return cases


def droplet_totals(r):
    a = r["aggregate"]["metrics"]
    return {"loss": a["crps"]["case_mean"], "crps_case_mean": a["crps"]["case_mean"],
            "mae_median_case_mean": a["mae_median"]["case_mean"], "cases": a["crps"]["n_cases"],
            "refused": len(r["refused"])}


def event_cases(r, causes):
    cases = []
    for c in r["cases"]:
        if not c.get("scored"):
            continue
        parts = {k: c["metrics"].get(f"ibs_contrib[{k}]", 0.0) for k in causes}
        cases.append({"run_id": c["run_id"], "cutoff_s": c["cutoff_s"], "session": c.get("cluster") or c.get("group_id"),
                      "conditions": c.get("conditions") or {}, "loss": sum(parts.values()), "parts": parts,
                      "endpoint": c.get("endpoint")})
    return cases


def event_totals(r, causes):
    ev = r["aggregate"]["events"]["causes"]
    t = {f"ibs[{k}]": ev[k]["integrated_brier"] for k in causes}
    t["loss"] = sum(t.values())
    t["cases"] = sum(1 for c in r["cases"] if c.get("scored"))
    t["refused"] = len(r["refused"])
    return t


def front_cases(det):
    cases = []
    for c in det["rows"]:
        if "cutoff_s" not in c:  # the per-track spread-or-not rows (ignition outcome) are not position cases
            continue
        cases.append({"run_id": c["run_id"], "cutoff_s": c["cutoff_s"], "session": c["run_id"],
                      "conditions": {}, "loss": c["crps"], "parts": {"crps": c["crps"]},
                      "mae_median": c.get("ae_median"), "cov50": c.get("coverage_50"), "cov90": c.get("coverage_90"),
                      "events": c.get("events"), "by_lead_band_s": c.get("by_lead_band_s")})
    return cases


def front_totals(agg):
    o = agg["releases"][0]["aggregate"]["overall"]
    return {"loss": o["crps"]["mean_over_runs"], "crps_mean_over_runs": o["crps"]["mean_over_runs"],
            "ae_median_mean_over_runs": o["ae_median"]["mean_over_runs"], "cases": agg["counts"]["cases"]}


def front_loss(cases):
    """The front protocol's aggregate: case = mean over targets, run = mean over its cases, mean over runs."""
    by = defaultdict(list)
    for c in cases:
        by[c["run_id"]].append(c["loss"])
    return statistics.mean(statistics.mean(v) for v in by.values()) if by else float("nan")


def mean_loss(kind, cases):
    if not cases:
        return float("nan")
    return front_loss(cases) if kind == "front" else statistics.mean(c["loss"] for c in cases)


# ------------------------------------------------------------------------------------------ regimes

BIN_ORDER = {}


def bin_of(v, edges, labels):
    for i, lab in enumerate(labels):
        BIN_ORDER.setdefault(lab, i)
    if v is None:
        return "unknown"
    for e, lab in zip(edges, labels):
        if v < e:
            return lab
    return labels[-1]


def session_date(session: str):
    m = re.search(r"(\d{4}-\d{2}-\d{2})", session or "")
    return m.group(1) if m else None


def attach_regimes(cand: Candidate, cases, data_root):
    """Adds `regimes` {dimension: bin} to every case. Conditions come from the feedback rows; fuel names from
    the train records (tools/wsp/analysis/lib.py), read for the case ids only, which are all train runs."""
    fam = cand.kind
    ids = sorted({c["run_id"] for c in cases})
    recs = {}
    if fam != "front":
        name = records_name(cand.family)
        if data_root is None:
            recs = lib.experiments(name, ids)
        else:
            p = guard_path(Path(data_root) / "work" / "records" / name / "experiments.jsonl", "records")
            want = set(ids)
            with p.open(encoding="utf-8") as f:
                for line in f:
                    r = json.loads(line)
                    if r["id"] in want:
                        recs[r["id"]] = r
    tracks = {}
    if fam == "front":
        for f in BASS_TRAIN_TRACKS.glob("T*.json"):
            t = json.loads(f.read_text(encoding="utf-8"))
            tracks[t["run_id"]] = t.get("conditions", {})
    for c in cases:
        k = c["conditions"]
        rec = recs.get(c["run_id"], {})
        g = {}
        if fam == "droplet":
            g["fuel"] = rec.get("fuel_or_material") or "unknown"
            g["oxygen"] = bin_of(k.get("x_o2"), [0.16, 0.20, 0.25], ["x_O2 < 0.16", "0.16-0.20", "0.20-0.25", ">= 0.25"])
            g["pressure"] = bin_of((k.get("pressure") or 0) / 1e3 if "pressure" in k else None, [80, 150],
                                   ["< 80 kPa", "80-150 kPa", ">= 150 kPa"])
            he, co2 = k.get("x_he") or 0.0, k.get("x_co2") or 0.0
            g["diluent"] = "He" if he > 0 else ("CO2" if co2 > 0 else "N2 only")
            d0 = k.get("initial_diameter")
            g["initial diameter"] = bin_of(None if d0 is None else d0 * 1e3, [2.5, 3.5],
                                           ["d0 < 2.5 mm", "2.5-3.5 mm", ">= 3.5 mm"])
        elif fam == "event":
            g["fuel"] = rec.get("fuel_or_material") or "unknown"
            g["pressure"] = bin_of(None if "pressure" not in k else k["pressure"] / 1e3, [75, 150, 250],
                                   ["< 75 kPa", "75-150 kPa", "150-250 kPa", ">= 250 kPa"])
            g["oxygen"] = bin_of(k.get("x_o2_ambient"), [0.25, 0.38], ["x_O2 < 0.25", "0.25-0.38", ">= 0.38"])
            d = session_date(c["session"])
            g["era"] = ("unknown" if d is None else "early (<= 2021-08-30)" if d <= "2021-08-30"
                        else "late (>= 2021-09-27)" if d >= "2021-09-27" else "between")
        else:
            t = tracks.get(c["run_id"], {})
            c["conditions"] = t
            g["oxygen"] = bin_of(t.get("x_o2_initial"), [0.17, 0.19], ["x_O2 < 0.17", "0.17-0.19", ">= 0.19"])
            g["restrictor"] = str(t.get("restrictor", "unknown"))
        c["regimes"] = g


# ------------------------------------------------------------------------------------------ decompose

def droplet_summary(cs, total_sum):
    def mean(key):
        v = [c[key] for c in cs if c.get(key) is not None]
        return statistics.mean(v) if v else None
    crps = statistics.mean(c["loss"] for c in cs)
    centred = mean("crps_centred")
    return {"cases": len(cs), "crps": crps, "share_of_loss": sum(c["loss"] for c in cs) / total_sum if total_sum else None,
            "bias_median_um": mean("bias_um"), "mae_median": mean("mae_median"), "cov50": mean("cov50"),
            "cov90": mean("cov90"), "crps_centred": centred,
            "centred_share": (centred / crps) if centred is not None and crps > 0 else None}


def event_summary(cs, total_sum, causes):
    out = {"cases": len(cs), "loss": statistics.mean(c["loss"] for c in cs),
           "share_of_loss": sum(c["loss"] for c in cs) / total_sum if total_sum else None}
    for k in causes:
        out[f"ibs[{k}]"] = statistics.mean(c["parts"][k] for c in cs)
    return out


def front_summary(cs, total_sum):
    def mean(key):
        v = [c[key] for c in cs if c.get(key) is not None]
        return statistics.mean(v) if v else None
    return {"cases": len(cs), "runs": len({c["run_id"] for c in cs}), "crps": front_loss(cs),
            "share_of_loss": sum(c["loss"] for c in cs) / total_sum if total_sum else None,
            "ae_median": mean("mae_median"), "cov50": mean("cov50"), "cov90": mean("cov90")}


def group_by(cases, keyf):
    g = defaultdict(list)
    for c in cases:
        g[keyf(c)].append(c)
    def order(k):
        try:
            return (0, float(k), "")
        except ValueError:
            return (1, BIN_ORDER.get(k, 99), str(k))
    return dict(sorted(g.items(), key=lambda kv: order(kv[0])))


def fmt(v, kind="f4"):
    if v is None or (isinstance(v, float) and math.isnan(v)):
        return "-"
    if kind == "um":
        return f"{v * 1e6:.2f}"
    if kind == "pct":
        return f"{100 * v:.1f} %"
    if kind == "f2":
        return f"{v:.2f}"
    if kind == "f1":
        return f"{v:+.1f}"
    if kind == "e":
        return f"{v:+.3e}"
    return f"{v:.4f}"


def table(rows, cand_kind, causes, first="group"):
    if cand_kind == "droplet":
        head = [first, "cases", "CRPS µm", "share of loss", "bias of median µm", "MAE of median µm", "cov 50 %",
                "cov 90 %", "CRPS if centred µm", "centred share"]
        body = [[k, str(s["cases"]), fmt(s["crps"], "um"), fmt(s["share_of_loss"], "pct"), fmt(s["bias_median_um"], "f1"),
                 fmt(s["mae_median"], "um"), fmt(s["cov50"], "f2"), fmt(s["cov90"], "f2"), fmt(s["crps_centred"], "um"),
                 fmt(s["centred_share"], "pct")] for k, s in rows]
    elif cand_kind == "event":
        head = [first, "cases", "loss (sum)", *[f"IBS {k}" for k in causes], "share of loss"]
        body = [[k, str(s["cases"]), fmt(s["loss"]), *[fmt(s[f"ibs[{c}]"]) for c in causes], fmt(s["share_of_loss"], "pct")]
                for k, s in rows]
    else:
        head = [first, "cases", "runs", "CRPS (w)", "share of loss", "AE of median (w)", "cov 50 %", "cov 90 %"]
        body = [[k, str(s["cases"]), str(s["runs"]), fmt(s["crps"]), fmt(s["share_of_loss"], "pct"), fmt(s["ae_median"]),
                 fmt(s["cov50"], "f2"), fmt(s["cov90"], "f2")] for k, s in rows]
    return ["| " + " | ".join(head) + " |", "|" + "---|" * len(head)] + ["| " + " | ".join(r) + " |" for r in body]


def summarise(kind, cs, total_sum, causes):
    if kind == "droplet":
        return droplet_summary(cs, total_sum)
    if kind == "event":
        return event_summary(cs, total_sum, causes)
    return front_summary(cs, total_sum)


def decompose(args):
    cand = Candidate(args.family, args.candidate)
    out_dir = guard_out(Path(args.out) if args.out else OUT_ROOT / f"{cand.family}_{cand.dir.name}")
    out_dir.mkdir(parents=True, exist_ok=True)
    data_root = guard_path(ROOT / args.data_root, "data root") if args.data_root else None
    before = registry_snapshot()
    artifacts = scratch_registry(out_dir / "_registry", cand)
    t0 = time.time()
    fb, cases, totals = feedback(cand, cand.id, artifacts, out_dir / "feedback_train.json", data_root, args.n_boot)
    secs = time.time() - t0
    attach_regimes(cand, cases, data_root)
    causes = fb["spec"]["causes"] if cand.kind == "event" else []
    total_sum = sum(c["loss"] for c in cases)
    recomputed = mean_loss(cand.kind, cases)
    report = {"banner": BANNER, "command": "decompose", "family": cand.family, "candidate": str(cand.dir.relative_to(ROOT)),
              "release_id": cand.id, "partition": "train", "data_root": str(data_root) if data_root else "data",
              "feedback_seconds": round(secs, 2), "totals_reported_by_feedback": totals,
              "totals_recomputed_from_cases": {"loss": recomputed, "cases": len(cases)}}
    if cand.kind == "event":
        report["totals_recomputed_from_cases"].update(
            {f"ibs[{k}]": statistics.mean(c["parts"][k] for c in cases) for k in causes})
    report["totals_match"] = abs(recomputed - totals["loss"]) <= 1e-12 * max(1.0, abs(totals["loss"]))
    md = [f"# Train loss decomposition: {cand.label}", "", f"**{BANNER}**", "",
          f"Release `{cand.id}` · family `{cand.family}` · TRAIN partition only · data root "
          f"`{report['data_root']}` · feedback run {secs:.1f} s.", ""]
    if cand.kind == "droplet":
        md += [f"Loss = CRPS of the diameter forecast, case mean (µm). Total: **{totals['loss'] * 1e6:.3f} µm** over "
               f"{totals['cases']} cases ({totals['refused']} refused) as reported by `wsp-dev feedback`; recomputed "
               f"from the case rows: {recomputed * 1e6:.3f} µm ({'equal' if report['totals_match'] else 'DIFFERENT'}).", ""]
    elif cand.kind == "event":
        md += [f"Loss = sum over causes of the per-case integrated Brier contributions (IPCW, {EVENT_TIMES[0]}-"
               f"{EVENT_TIMES[-1]} s). Totals reported by `wsp-dev feedback`: "
               + ", ".join(f"{k} **{totals[f'ibs[{k}]']:.4f}**" for k in causes)
               + f", sum {totals['loss']:.4f}, on {totals['cases']} cases ({totals['refused']} refused). Recomputed from "
               f"the case rows: " + ", ".join(f"{k} {report['totals_recomputed_from_cases'][f'ibs[{k}]']:.4f}" for k in causes)
               + f" ({'equal' if report['totals_match'] else 'DIFFERENT'}).", ""]
    else:
        md += [f"Loss = CRPS of the flame-tip position in sample widths (case = mean over targets, run = mean over its "
               f"cases, mean over runs). Total: **{totals['loss']:.4f}** over {totals['cases']} cases; recomputed: "
               f"{recomputed:.4f} ({'equal' if report['totals_match'] else 'DIFFERENT'}).", ""]

    sections = [("Session (evaluator's cluster)", lambda c: c["session"]), ("Cutoff (s)", lambda c: f"{c['cutoff_s']:g}")]
    dims = sorted({d for c in cases for d in c["regimes"]})
    sections += [(f"Regime: {d}", (lambda dd: lambda c: c["regimes"][dd])(d)) for d in dims]
    report["groups"] = {}
    for title, keyf in sections:
        rows = [(k, summarise(cand.kind, cs, total_sum, causes)) for k, cs in group_by(cases, keyf).items()]
        report["groups"][title] = dict(rows)
        md += [f"## By {title.lower()}", ""] + table(rows, cand.kind, causes, title.split(":")[-1].strip()) + [""]
    allrow = [("all", summarise(cand.kind, cases, total_sum, causes))]
    report["groups"]["all"] = dict(allrow)
    if cand.kind == "droplet":
        md += ["## Centre against spread (all cases)", ""] + table(allrow, cand.kind, causes, "") + [
            "", "*CRPS if centred* is the CRPS the same quantile bands would score if their median sat exactly on every "
            "observation (exact for the evaluator's quantile CRPS, from the band widths). *Centred share* = that over the "
            "actual CRPS: near 100 % means the loss is the width of the band; low means the median is off.", ""]
    if cand.kind == "event":
        agg = fb["releases"][0]["aggregate"]["events"]
        hz = agg["horizons"]
        report["by_horizon"] = {k: [b["brier"] for b in agg["causes"][k]["brier"]] for k in causes}
        md += ["## Brier score by horizon", "", "| horizon s | " + " | ".join(causes) + " |", "|---|" + "---|" * len(causes)]
        for i, h in enumerate(hz):
            md.append(f"| {h:g} | " + " | ".join(f"{report['by_horizon'][k][i]:.4f}" for k in causes) + " |")
        md += ["", "## Reliability (evaluator's calibration bins: four bins of forecast probability per horizon, "
               "observed frequency IPCW-weighted)", ""]
        report["reliability"] = {k: agg["causes"][k]["calibration"] for k in causes}
        show_h = [h for h in (10.0, 20.0, 30.0, 60.0, 120.0) if h in hz]
        for k in causes:
            if all(b["predicted"] == 0 for cal in agg["causes"][k]["calibration"] for b in cal["bins"]):
                md += [f"**{k}**: never forecast, never observed.", ""]
                continue
            md += [f"**{k}**", "", "| horizon s | bin: n, mean forecast → observed |", "|---|---|"]
            for cal in agg["causes"][k]["calibration"]:
                if cal["horizon"] in show_h:
                    md.append(f"| {cal['horizon']:g} | " + " · ".join(
                        f"{b['n']}: {b['predicted']:.3f} → {b['observed_ipcw']:.3f}" for b in cal["bins"]) + " |")
            md.append("")
    if cand.kind == "front":
        agg = fb["aggregate"]["releases"][0]["aggregate"]
        report["events"] = agg.get("events")
        report["ignition_outcome"] = agg.get("ignition_outcome")
        md += [f"Track-ended events: Brier {agg['events']['brier_mean_over_runs']:.4f} (mean over runs); spread-or-not: "
               f"Brier {agg['ignition_outcome']['brier_mean_over_tracks']:.4f} (mean over tracks).", ""]
    top = sorted(cases, key=lambda c: -c["loss"])[:15]
    md += ["## The 15 cases with the largest loss", "",
           "| run | session | cutoff s | loss | " + ("parts | " if cand.kind == "event" else "")
           + "regimes | conditions |", "|---|---|---|---|" + ("---|" if cand.kind == "event" else "") + "---|---|"]
    for c in top:
        lv = fmt(c["loss"], "um") + " µm" if cand.kind == "droplet" else fmt(c["loss"])
        parts = ("| " + ", ".join(f"{k} {v:.4f}" for k, v in c["parts"].items()) + " ") if cand.kind == "event" else ""
        cond = ", ".join(f"{k}={v:.4g}" if isinstance(v, (int, float)) else f"{k}={v}" for k, v in c["conditions"].items())
        md.append(f"| {c['run_id']} | {c['session']} | {c['cutoff_s']:g} | {lv} {parts}| "
                  f"{'; '.join(c['regimes'].values())} | {cond} |")
    report["top15"] = top
    report["cases"] = cases
    after = registry_snapshot()
    report["isolation"] = {"before": before, "after": after, "registry_unchanged": before == after}
    md += ["", f"Real registry `artifacts/models/`: {before['artifacts_models_entries']} entries before, "
           f"{after['artifacts_models_entries']} after ({'unchanged' if before == after else 'CHANGED'})."]
    (out_dir / "decompose.md").write_text("\n".join(md) + "\n", encoding="utf-8", newline="\n")
    (out_dir / "decompose.json").write_text(json.dumps(report, indent=1) + "\n", encoding="utf-8", newline="\n")
    if not args.keep_scratch:
        shutil.rmtree(out_dir / "_registry", ignore_errors=True)
    print(f"{BANNER}\nwritten {out_dir / 'decompose.md'} and decompose.json; totals "
          f"{'match' if report['totals_match'] else 'DIFFER'}: {json.dumps(totals)}")
    return 0 if report["totals_match"] else 1


# ------------------------------------------------------------------------------------------ sensitivity

STMT = re.compile(r"^\s*([A-Za-z_][A-Za-z0-9_.]*)\s*(\[[^\]]*\])?\s+(\S+)\s*$")


def parse_blocks(path: Path):
    out = []
    for n, raw in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        line = raw.split("#", 1)[0].strip()
        if not line:
            continue
        m = STMT.match(line)
        if not m:
            sys.exit(f"{path}:{n}: expected `<tensor>[selectors] <step>`, got {raw!r}")
        name, sel, step = m.group(1), m.group(2), m.group(3)
        if step[0] in "xX*":
            mode, val = "rel", float(step[1:])
        elif step[0] in "+-":
            mode, val = "abs", float(step)
        else:
            sys.exit(f"{path}:{n}: step {step!r} is neither +v / -v nor xF")
        out.append({"line": n, "target": name + (sel or ""), "name": name, "sel": sel, "step": step, "mode": mode, "value": val})
    return out


def selection(arr_shape, sel):
    """Selector text -> list of (start, stop) per axis (ints become one-element ranges)."""
    if sel is None:
        return [(0, d) for d in arr_shape]
    items = [s.strip() for s in sel[1:-1].split(",")]
    if len(items) != len(arr_shape):
        raise ValueError(f"{len(items)} selectors for a tensor of rank {len(arr_shape)}")
    rng = []
    for s, d in zip(items, arr_shape):
        if s == ":":
            rng.append((0, d))
        elif ":" in s:
            a, b = s.split(":")
            rng.append((int(a), int(b)))
        else:
            rng.append((int(s), int(s) + 1))
        if not (0 <= rng[-1][0] < rng[-1][1] <= d):
            raise ValueError(f"selector {s} out of range for axis of length {d}")
    return rng


def literal(v: float, old=None) -> str:
    """Shortest exact literal of an unchanged value; 12 significant digits for a stepped one (no float noise)."""
    v = float(v)
    s = repr(v) if old is not None and v == float(old) else f"{v:.12g}"
    return s if any(ch in s for ch in ".eE") else s + ".0"


def statement(name, shape, rng, new, old):
    """One literal statement of the edit language writing `new` (array of the box shape) at the box."""
    if len(shape) == 0:
        return f"{name} = {literal(new, old)}"
    if all(b - a == 1 for a, b in rng):
        return f"{name}[{', '.join(str(a) for a, _ in rng)}] = {literal(new.reshape(-1)[0], old.reshape(-1)[0])}"

    def nest(x, o):
        return "[" + ", ".join(nest(y, p) for y, p in zip(x, o)) + "]" if getattr(x, "ndim", 0) else literal(x, o)
    return f"{name}[{', '.join(f'{a}:{b}' for a, b in rng)}] = {nest(new, old)}"


def probe_job(job):
    """Builds, assembles and scores one perturbed child release in its own scratch registry (worker process)."""
    t0 = time.time()
    d = Path(job["dir"])
    d.mkdir(parents=True, exist_ok=True)
    art = d / "registry"
    (art / "models").mkdir(parents=True, exist_ok=True)
    dst = art / "models" / Path(job["cand_reg"]).name
    if not dst.exists():
        shutil.copytree(job["cand_reg"], dst)
    res = {"index": job["index"], "target": job["target"], "step": job["step"], "statement": job["statement"]}
    try:
        (d / "edits.wsp").write_text(
            f"# tools/wsp/attrib probe {job['index']}: {BANNER}\n# child of {job['cand_id']}\n{job['statement']}\n",
            encoding="utf-8", newline="\n")
        (d / "meta.json").write_text(json.dumps(job["meta"], indent=2) + "\n", encoding="utf-8", newline="\n")
        run([IGNIS, "wsp", "compile", "--family", job["family"], "--edits", d / "edits.wsp", "--meta", d / "meta.json",
             "--out", d / "patch.json"], art)
        built = json.loads(run([IGNIS, "wsp", "assemble", "--family", job["family"], "--patch", d / "patch.json",
                                "--binding", job["binding"], "--root", art, "--out", d / "release.json"], art))
        res["release_id"] = built["release_id"]
        res["weights_hash"] = built["weights_hash"]
        t1 = time.time()
        cand = job["cand_obj"]
        fb, cases, totals = feedback(cand, built["release_id"], art, d / "feedback_train.json", job["data_root"], job["n_boot"])
        res.update({"cases": cases, "totals": totals, "assemble_s": round(t1 - t0, 2), "feedback_s": round(time.time() - t1, 2)})
    except (RuntimeError, SystemExit) as e:
        res["error"] = str(e)[-1500:]
    res["seconds"] = round(time.time() - t0, 2)
    if not job["keep"]:
        shutil.rmtree(art, ignore_errors=True)
    return res


def compare(kind, base_cases, child_cases, causes):
    key = lambda c: (c["run_id"], c["cutoff_s"])
    b = {key(c): c for c in base_cases}
    ch = {key(c): c for c in child_cases}
    common = [k for k in b if k in ch]
    deltas = []
    for k in common:
        e = {"run_id": k[0], "cutoff_s": k[1], "session": b[k]["session"], "regimes": b[k]["regimes"],
             "base": b[k]["loss"], "child": ch[k]["loss"], "delta": ch[k]["loss"] - b[k]["loss"]}
        if kind == "event":
            e["delta_parts"] = {c: ch[k]["parts"][c] - b[k]["parts"][c] for c in causes}
        deltas.append(e)
    base_mean, child_mean = mean_loss(kind, base_cases), mean_loss(kind, child_cases)
    out = {"base_loss": base_mean, "child_loss": child_mean, "delta_loss": child_mean - base_mean,
           "cases_common": len(common), "cases_only_base": len(b) - len(common), "cases_only_child": len(ch) - len(common)}
    if kind == "event":
        out["delta_parts"] = {c: statistics.mean(x["parts"][c] for x in child_cases) - statistics.mean(x["parts"][c] for x in base_cases)
                              for c in causes}
    tol = 1e-9 * abs(base_mean) if base_mean else 1e-15
    sess = defaultdict(list)
    for e in deltas:
        sess[e["session"]].append(e["delta"])
    out["by_session"] = {s: {"cases": len(v), "delta_mean": statistics.mean(v)} for s, v in sorted(sess.items())}
    out["sessions_improve"] = sum(1 for v in out["by_session"].values() if v["delta_mean"] < -tol)
    out["sessions_worsen"] = sum(1 for v in out["by_session"].values() if v["delta_mean"] > tol)
    out["sessions_total"] = len(sess)
    reg = defaultdict(list)
    for e in deltas:
        for dname, binname in e["regimes"].items():
            reg[(dname, binname)].append(e["delta"])
    out["by_regime"] = {f"{d}: {bn}": {"cases": len(v), "delta_mean": statistics.mean(v)}
                        for (d, bn), v in sorted(reg.items(), key=lambda kv: (kv[0][0], BIN_ORDER.get(kv[0][1], 99), kv[0][1]))}
    srt = sorted(deltas, key=lambda e: e["delta"])
    out["cases_improve_most"] = [e for e in srt[:5] if e["delta"] < -tol]
    out["cases_worsen_most"] = [e for e in reversed(srt[-5:]) if e["delta"] > tol]
    out["cases_moved"] = sum(1 for e in deltas if abs(e["delta"]) > tol)
    out["max_abs_case_delta"] = max((abs(e["delta"]) for e in deltas), default=0.0)
    return out


def sensitivity(args):
    import numpy as np
    cand = Candidate(args.family, args.candidate)
    blocks = parse_blocks(guard_path(ROOT / args.blocks, "blocks"))
    out_dir = guard_out(Path(args.out) if args.out else OUT_ROOT / f"{cand.family}_{cand.dir.name}_sensitivity")
    out_dir.mkdir(parents=True, exist_ok=True)
    data_root = guard_path(ROOT / args.data_root, "data root") if args.data_root else None
    before = registry_snapshot()
    configs_before = tree_hash(cand.dir.parent)
    tensors = cand.tensors()
    t0 = time.time()
    base_art = scratch_registry(out_dir / "_registry_base", cand)
    fb, base_cases, base_totals = feedback(cand, cand.id, base_art, out_dir / "feedback_train_base.json", data_root, args.n_boot)
    base_s = time.time() - t0
    attach_regimes(cand, base_cases, data_root)
    causes = fb["spec"]["causes"] if cand.kind == "event" else []
    jobs, results = [], []
    for i, b in enumerate(blocks):
        entry = dict(b)
        if b["name"] not in tensors:
            entry["error"] = f"unknown tensor {b['name']}"
            results.append(entry)
            continue
        arr = tensors[b["name"]]
        try:
            rng = selection(arr.shape, b["sel"])
        except ValueError as e:
            entry["error"] = str(e)
            results.append(entry)
            continue
        box = arr[tuple(slice(a, c) for a, c in rng)] if arr.ndim else arr
        new = box * b["value"] if b["mode"] == "rel" else box + b["value"]
        entry["old"] = np.asarray(box).reshape(-1).tolist()
        entry["new"] = np.asarray(new).reshape(-1).tolist()
        stmt = statement(b["name"], arr.shape, rng, np.asarray(new), np.asarray(box))
        meta = json.loads(json.dumps(cand.meta))
        meta["patch_id"] = f"{cand.meta.get('patch_id', cand.dir.name)}-attrib-probe-{i}"
        meta["parent_release_id"] = cand.id
        meta["rationale"] = (f"tools/wsp/attrib diagnostic probe, never a candidate: {b['target']} {b['step']} over "
                             f"{cand.id}. {BANNER}")
        jobs.append({"index": i, "target": b["target"], "step": b["step"], "statement": stmt, "meta": meta,
                     "dir": str(out_dir / f"probe_{i:02d}"), "cand_reg": str(cand.reg_dir), "cand_id": cand.id,
                     "family": cand.family, "binding": str(cand.binding), "data_root": data_root, "n_boot": args.n_boot,
                     "keep": args.keep_scratch, "cand_obj": cand, "entry": entry})
    with cf.ProcessPoolExecutor(max_workers=max(1, min(args.workers, len(jobs) or 1))) as pool:
        futs = {pool.submit(probe_job, {k: v for k, v in j.items() if k != "entry"}): j for j in jobs}
        for f in cf.as_completed(futs):
            j = futs[f]
            r = f.result()
            e = j["entry"]
            e.update({k: r[k] for k in ("statement", "release_id", "weights_hash", "seconds", "assemble_s", "feedback_s", "error") if k in r})
            e["weights_equal_candidate"] = r.get("weights_hash") == cand.release["construction"]["weights_hash"]
            if "cases" in r:
                attach_regimes(cand, r["cases"], data_root)
                e["child_totals"] = r["totals"]
                e["comparison"] = compare(cand.kind, base_cases, r["cases"], causes)
            results.append(e)
    results.sort(key=lambda e: -abs(e.get("comparison", {}).get("delta_loss", 0.0)))
    after = registry_snapshot()
    configs_after = tree_hash(cand.dir.parent)
    isolation = {"registry_before": before, "registry_after": after, "registry_unchanged": before == after,
                 "candidate_config_tree_sha256_before": configs_before, "candidate_config_tree_sha256_after": configs_after,
                 "configs_unchanged": configs_before == configs_after}
    secs = [e["seconds"] for e in results if "seconds" in e and "error" not in e]
    report = {"banner": BANNER, "command": "sensitivity", "family": cand.family, "candidate": str(cand.dir.relative_to(ROOT)),
              "release_id": cand.id, "partition": "train", "base_totals": base_totals,
              "base_loss_recomputed": mean_loss(cand.kind, base_cases), "base_feedback_seconds": round(base_s, 2),
              "workers": args.workers, "seconds_per_perturbation": secs, "isolation": isolation,
              "perturbations": [{k: v for k, v in e.items()} for e in results]}
    unit = (lambda v: f"{v * 1e6:+.4f} µm") if cand.kind == "droplet" else (lambda v: f"{v:+.5f}")
    md = [f"# Train sensitivity: {cand.label}", "", f"**{BANNER}**", "",
          f"Release `{cand.id}` · family `{cand.family}` · TRAIN partition only. Each row is a child release "
          f"(parent = the candidate, one statement) assembled in a scratch registry and scored with the same train "
          f"feedback. Base loss {unit(report['base_loss_recomputed']).lstrip('+')}"
          + (" (sum of " + " + ".join(f"{k} {base_totals[f'ibs[{k}]']:.4f}" for k in causes) + ")" if causes else "")
          + f"; Δ = child − base, negative = lower loss. Sessions: evaluator clusters.", "",
          "| line | perturbation | old → new | Δ loss | Δ % | " + ("Δ by cause | " if causes else "")
          + "sessions better / worse / all | cases moved | largest session Δ |",
          "|---|---|---|---|---|" + ("---|" if causes else "") + "---|---|---|"]
    for e in results:
        tgt = f"`{e['target']}` {e['step']}"
        if "comparison" not in e:
            md.append(f"| {e['line']} | {tgt} | - | refused: {e.get('error', '?')[-220:].replace(chr(10), ' ').replace('|', '/')} | | "
                      + ("| " if causes else "") + "| | |")
            continue
        c = e["comparison"]
        old_new = (f"{e['old'][0]:.6g} → {e['new'][0]:.6g}" if len(e["old"]) == 1 else f"{len(e['old'])} elements")
        big = max(c["by_session"].items(), key=lambda kv: abs(kv[1]["delta_mean"]), default=(None, None))
        bigs = f"{big[0]} {unit(big[1]['delta_mean'])}" if big[0] and big[1]["delta_mean"] != 0 else "-"
        if e["weights_equal_candidate"]:
            old_new_note = " (weights = candidate's)"
        else:
            old_new_note = ""
        pct = 100 * c["delta_loss"] / c["base_loss"] if c["base_loss"] else float("nan")
        md.append(f"| {e['line']} | {tgt} | {old_new}{old_new_note} | {unit(c['delta_loss'])} | {pct:+.3f} % | "
                  + ((", ".join(f"{k} {v:+.5f}" for k, v in c["delta_parts"].items()) + " | ") if causes else "")
                  + f"{c['sessions_improve']} / {c['sessions_worsen']} / {c['sessions_total']} | {c['cases_moved']} | {bigs} |")
    md += ["", f"Runtime: base feedback {base_s:.1f} s; per perturbation (compile + assemble + feedback, "
           f"{args.workers} workers) " + (f"median {statistics.median(secs):.1f} s, max {max(secs):.1f} s." if secs else "-"),
           "", f"Isolation: real registry `artifacts/models/` {before['artifacts_models_entries']} → "
           f"{after['artifacts_models_entries']} entries ({'unchanged' if isolation['registry_unchanged'] else 'CHANGED'}); "
           f"`{cand.dir.parent.relative_to(ROOT)}` tree {'unchanged' if isolation['configs_unchanged'] else 'CHANGED'}.", ""]
    for e in results:
        if "comparison" not in e:
            continue
        c = e["comparison"]
        md += [f"## {e['target']} {e['step']}", "", f"Statement: `{e['statement'][:300]}` · child `{e['release_id']}` · "
               f"{e['seconds']:.1f} s · common cases {c['cases_common']} (only base {c['cases_only_base']}, only child "
               f"{c['cases_only_child']}).", "", "| regime | cases | Δ mean |", "|---|---|---|"]
        md += [f"| {k} | {v['cases']} | {unit(v['delta_mean'])} |" for k, v in c["by_regime"].items()]
        md += ["", "| session | cases | Δ mean |", "|---|---|---|"]
        md += [f"| {k} | {v['cases']} | {unit(v['delta_mean'])} |" for k, v in c["by_session"].items()]
        for title, lst in (("improve most", c["cases_improve_most"]), ("worsen most", c["cases_worsen_most"])):
            md += ["", f"Cases that {title}: " + ("; ".join(f"{x['run_id']} @{x['cutoff_s']:g} s {unit(x['delta'])}" for x in lst) or "none")]
        md.append("")
    (out_dir / "sensitivity.md").write_text("\n".join(md) + "\n", encoding="utf-8", newline="\n")
    (out_dir / "sensitivity.json").write_text(json.dumps(report, indent=1, default=str) + "\n", encoding="utf-8", newline="\n")
    if not args.keep_scratch:
        shutil.rmtree(out_dir / "_registry_base", ignore_errors=True)
    print(f"{BANNER}\nwritten {out_dir / 'sensitivity.md'} and sensitivity.json; isolation: registry "
          f"{'unchanged' if isolation['registry_unchanged'] else 'CHANGED'}, configs "
          f"{'unchanged' if isolation['configs_unchanged'] else 'CHANGED'}")
    return 0 if isolation["registry_unchanged"] and isolation["configs_unchanged"] else 2


def main() -> int:
    ap = argparse.ArgumentParser(description=BANNER)
    sub = ap.add_subparsers(dest="cmd", required=True)
    for name in ("decompose", "sensitivity"):
        p = sub.add_parser(name)
        p.add_argument("--family", required=True, help="model key as `ignis wsp ... --family` (e.g. acme_normal7)")
        p.add_argument("--candidate", required=True, help="config dir of a release (holds release.json, meta.json)")
        p.add_argument("--partition", default="train", choices=["train"], help="train only; anything else is refused")
        p.add_argument("--data-root", default=None, help="IGNIS_DATA for the forwards (default: data)")
        p.add_argument("--out", default=None, help=f"output folder under {OUT_ROOT.relative_to(ROOT)}")
        p.add_argument("--n-boot", type=int, default=50, help="bootstrap draws of feedback (aggregates only; case rows do not depend on it)")
        p.add_argument("--keep-scratch", action="store_true", help="keep the scratch registries")
        if name == "sensitivity":
            p.add_argument("--blocks", required=True, help="perturbations, one `<tensor>[sel] <step>` per line")
            p.add_argument("--workers", type=int, default=4)
    args = ap.parse_args()
    if args.partition != "train":
        sys.exit("refused: train only")
    for exe in (IGNIS, WSPDEV):
        if not exe.exists():
            sys.exit(f"{exe} missing: cargo build --release -p ignis-cli -p ignis-wsp-dev")
    return decompose(args) if args.cmd == "decompose" else sensitivity(args)


if __name__ == "__main__":
    sys.exit(main())
