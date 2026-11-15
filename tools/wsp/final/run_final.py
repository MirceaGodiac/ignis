#!/usr/bin/env python3
"""The one final run of PROTOCOL 12.5: the project's sealed test partitions and the test days of the other
investigations, scored once, on frozen releases, returning aggregate tables only.

Two modes.

* **Rehearsal** (the default). Every step runs on the VALIDATION partition or the validation days and must
  reproduce numbers that are already known (each step names them). It never names the test partition, never
  passes `--final-test`, never seals, never deposits, and opens no file of a test day or of `holdout/`.
* **Final**: only with `--execute-final --i-am-the-final-run --releases <frozen list>`. The sha256 of the list
  is printed and written into every result. Each step runs at most once: a marker
  `runs/final/20261115/markers/<step>.started` is created (exclusively) before anything is read, and a step
  whose marker exists refuses. A crashed step is not rerun by this script; the lead decides.

    python tools/wsp/final/run_final.py preflight [--releases tools/wsp/final/releases_proposed.json]
    python tools/wsp/final/run_final.py rehearse sealed|flame-design|cfi|fabric|all
    python tools/wsp/final/run_final.py final sealed|flame-design|cfi|fabric --releases LIST \
        --execute-final --i-am-the-final-run

Steps: `sealed` (FLEX droplets and ACME burner flames through the evaluator, 1), `flame-design` (2), `cfi`
(3), `fabric` (4). A FLEX-2 test-day step does not exist (its frames are not downloaded; README).

Printed and written to `runs/final/20261115/<step>.json`: aggregates only. Per-case rows (the evaluator's
comparison files, forecast-cohort listings, the chains' case files) go to `runs/final/20261115/local/`,
which is never printed. In final mode `docs/wsp/RESULTS_FINAL.md` is assembled from the step files.

Reveal log. Step 1 reveals through `ignis eval compare`, which writes its own line per reveal to
`runs/evaluation/reveal_log.jsonl`; this script adds nothing for it and counts the evaluator's lines after.
Steps 2 to 4 read test-day outcomes outside the vault; for each forecast artifact scored against a test-day
outcome this script appends one line with exactly the keys the evaluator writes (`artifact_id`, `at`,
`final_test`, `granted`, `host`, `partition`, `purpose`, `run_id`, `split_id`, `user`).
"""
import argparse
import csv
import datetime
import hashlib
import json
import os
import shutil
import subprocess
import sys
from collections import Counter
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(REPO / "tools/wsp/ext"))
sys.path.insert(0, str(REPO / "tools/wsp/analysis"))

DAY = "20261115"
OUT = REPO / "runs/final" / DAY
LOCAL = OUT / "local"            # per-case rows: never printed
MARKERS = OUT / "markers"
REVEAL_LOG = REPO / "runs/evaluation/reveal_log.jsonl"
SEALS = REPO / "runs/evaluation/seals"
ANALYSIS = REPO / "artifacts/analysis"
IGNIS_REL = "target-merged/release/ignis.exe" if os.name == "nt" else "target-merged/release/ignis"
WSPDEV_REL = "target-merged/release/wsp-dev.exe" if os.name == "nt" else "target-merged/release/wsp-dev"
IGNIS, WSPDEV = REPO / IGNIS_REL, REPO / WSPDEV_REL
RESULTS_MD = REPO / "docs/wsp/RESULTS_FINAL.md"

EVENT_TIMES = [6, 8, 10, 12, 15, 20, 25, 30, 40, 50, 60, 80, 100, 120]
SETTINGS = {"droplet": json.dumps({"horizon_s": 10}), "event": json.dumps({"times_s": EVENT_TIMES})}
INPUT_ROOT = {"shared": None, "data_ext2": "runs/dev/data_ext2"}
FAMILY_KIND = {"flex_methanol": "droplet", "flex_heptane": "droplet", "acme_normal": "event", "acme_inverse": "event"}
SEALED_FAMILIES = tuple(FAMILY_KIND)
STEPS = ("sealed", "flame-design", "cfi", "fabric", "flex2")

# ------------------------------------------------------------------------------------------------
# Rehearsal: what each step must reproduce on validation (sources named beside each number)
# ------------------------------------------------------------------------------------------------
EXPECT_SEALED = {
    # docs/wsp/README.md: CRPS in micrometres, the evaluator's estimate on the registered group's common cases
    "flex_methanol": {"shared": {"crps_um": 40.0}},
    "flex_heptane": {"shared": {"crps_um": 46.1}},
    # docs/wsp/RESULTS_GEN6.md: F2 extinction / operator termination, case means
    "acme_normal": {"shared": {"extinction": 0.1537, "operator_termination": 0.0961},
                    "data_ext2": {"extinction": 0.1473, "operator_termination": 0.0798}},
    "acme_inverse": {"shared": {"extinction": 0.1288, "operator_termination": 0.0880},
                     "data_ext2": {"extinction": 0.1349, "operator_termination": 0.0480}},
}
# the comparisons of the registered looks (their release sets, read back from the comparison files they wrote)
REGISTERED_LOOKS = {
    "flex_methanol": [("shared", "runs/dev/cmp_validation_flex.json")],
    "flex_heptane": [("shared", "runs/dev/cmp_validation_flex_heptane.json")],
    "acme_normal": [("shared", "runs/dev/gen6/f2look/b/cmp_validation_acme_normal_f2.json"),
                    ("data_ext2", "runs/dev/gen6/f2look/cmp_ext2_normal.json")],
    "acme_inverse": [("shared", "runs/dev/gen6/f2look/b/cmp_validation_acme_inverse_f2.json"),
                     ("data_ext2", "runs/dev/gen6/f2look/cmp_ext2_inverse.json")],
}
# a doubled comparison (both roots sealed) and the half sealed alone: the split must give the evaluator's numbers
SPLIT_CHECK = ("runs/dev/gen5/cmp_validation_acme_normal_gen5_mixed.json",
               "runs/dev/gen5/cmp_validation_acme_normal_gen5_script.json")
# docs/wsp/feedback/flame_design_f2_result.md, "Decision inputs for 12.10" (F2 / F1, ext / term / sum)
EXPECT_FD = {
    "acme_normal/nostop": {"f2": (0.0317, 0.3933, 0.4250), "f1": (0.0558, 0.4028, 0.4586), "cases": 86},
    "acme_normal/stop": {"f2": (0.0068, 0.0724, 0.0792), "f1": (0.0141, 0.0826, 0.0966), "cases": 86},
    "acme_inverse/nostop": {"f2": (0.0871, 0.1959, 0.2831), "f1": (0.0871, 0.1959, 0.2831), "cases": 75},
    "acme_inverse/stop": {"f2": (0.0920, 0.0018, 0.0938), "f1": (0.0921, 0.0240, 0.1160), "cases": 75},
}
FD_REPORT = REPO / "runs/dev/data_all/work/transfer_acme/fd_f2/report.json"
# docs/wsp/feedback/cfi_transfer_result.md, validation-day line (micrometres)
EXPECT_CFI = {"cases": 6, "tests": 2, "days": 1,
              "candidate": dict(crps=130.1, mae_median=155.3, cov50=0.33, cov90=0.54),
              "root": dict(crps=212.0, mae_median=256.6, cov50=0.24, cov90=0.38),
              "persistence": 313.6, "d2_law": 1315.6, "paired_candidate_minus_root": -81.9}
CFI_AGGREGATE = REPO / "runs/dev/data_all/work/transfer_droplet/cfi/aggregate.json"
CFI_DEV_TESTS = {"1206", "1239", "1240", "1250", "1264", "1221", "1223"}   # tools/wsp/ext/cfi_cases.py
# PROTOCOL 12.9: on the four train tracks
EXPECT_FABRIC = {"sha256:ac15e49f1cb60d4b4679bcdff82a07c18a4a501454428e35ac86e935e168855c": (0.0761, 0.0033),
                 "sha256:00a8bc356d8e4e02a58b87d4786feeafd25b55b90a74b45e29c797054b9c5e72": (0.0761, 0.1469)}
FABRIC_TRAIN = REPO / "tools/wsp/bass_optical/tracks/train"
FABRIC_TEST = REPO / "runs/dev/media/fabric/tracks/psi25/held_test"
PSI26_TRACKS = REPO / "runs/dev/media/fabric/tracks/psi26"
PSI26_CONDITIONS_DONE = REPO / "runs/dev/media/fabric/DONE_PSI-26_CONDITIONS.txt"
FABRIC_MEASURER_TRAIN = REPO / "runs/dev/media/fabric/tracks/psi25/train"   # the measurer's train folder (quality field)
FABRIC_SCORER = REPO / "runs/dev/media/fabric/tracks_scorer/psi25"   # converted tracks (CONVERSION.md)


def conversion_left_out(partition):
    """Counts of the tracks the conversion left out, by reason category; names and reasons only."""
    d = json.loads((FABRIC_SCORER / "left_out.json").read_text(encoding="utf-8"))
    rows = d.get(partition, [])
    return dict(source=rel(FABRIC_SCORER / "left_out.json"), partition=partition, left_out=len(rows),
                by_reason=dict(Counter(r["reason"] for r in rows)), files=sorted(r["file"] for r in rows))


def quality_ok_folder(src, dest, final):
    """A folder of the measurer's tracks whose own top-level verdict `quality` is "ok", files copied unchanged.
    The decision reads that one field and nothing else of a track (front-feedback does not read it)."""
    if dest.exists():
        if final:
            sys.exit(f"{rel(dest)} exists: the quality-ok folder is built once, by this step")
        shutil.rmtree(dest)
    dest.mkdir(parents=True)
    kept, left = 0, Counter()
    for f in sorted(src.glob("*.json")):
        try:
            q = json.loads(f.read_text(encoding="utf-8")).get("quality")
        except (json.JSONDecodeError, AttributeError):
            q = "unreadable"
        if q == "ok":
            shutil.copyfile(f, dest / f.name)
            kept += 1
        else:
            left[str(q)] += 1
    return dict(source=rel(src), tracks=kept + sum(left.values()), kept_quality_ok=kept,
                left_out_for_quality=sum(left.values()), left_out_by_quality=dict(left))
PSI26_SPLIT_GLOB = "PSI-26*.json"
BASS_SPLIT = REPO / "data/splits/bass-4083156d5b18dab4.json"   # the project split the BASS-II folders follow


# ------------------------------------------------------------------------------------------------
# common
# ------------------------------------------------------------------------------------------------
def sha256_file(p):
    h = hashlib.sha256()
    with Path(p).open("rb") as f:
        for b in iter(lambda: f.read(1 << 20), b""):
            h.update(b)
    return h.hexdigest()


def write_json(path, obj):
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(obj, indent=1, ensure_ascii=False, default=str) + "\n", encoding="utf-8", newline="\n")


def rel(p):
    return Path(p).resolve().relative_to(REPO).as_posix()


def binaries():
    for b in (IGNIS, WSPDEV):
        if not b.exists():
            sys.exit(f"{b} does not exist (this script never builds a binary)")
    return {rel(b): sha256_file(b) for b in (IGNIS, WSPDEV)}


def ignis(args, data=None, check=True):
    """The evaluator CLI with the default vault and run roots; IGNIS_DATA only when an input names one."""
    env = dict(os.environ)
    for k in ("IGNIS_DATA", "IGNIS_HOLDOUT", "IGNIS_RUNS"):
        env.pop(k, None)
    if data:
        env["IGNIS_DATA"] = data
    r = subprocess.run([str(IGNIS)] + args, capture_output=True, text=True, cwd=REPO, env=env)
    if check and r.returncode != 0:
        # the evaluator prints a refusal object, not outcomes; keep only its tail
        sys.exit(f"ignis {' '.join(args[:3])} failed (exit {r.returncode}): {(r.stdout or r.stderr).strip()[-1500:]}")
    return r


def release_json(rid):
    p = REPO / "artifacts/models" / rid.split(":")[-1] / "release.json"
    if not p.exists():
        sys.exit(f"release {rid} is not in the local registry ({rel(p)})")
    return json.loads(p.read_text(encoding="utf-8"))


def seal_index():
    """(release id, partition) -> number of seal records. Seal records hold no outcome."""
    n = Counter()
    for p in SEALS.glob("*.json"):
        s = json.loads(p.read_text(encoding="utf-8"))
        n[(s["model_release_id"], s["sealed"]["split_partition"])] += 1
    return n


def split_ref(path):
    return "sha256:" + sha256_file(path)


def reveal_line(artifact_id, run_id, split_id, purpose):
    """One line in the evaluator's format (crates/ignis-lab/src/eval/vault.rs, `log`)."""
    e = {"artifact_id": artifact_id, "at": datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%S.%fZ"),
         "final_test": True, "granted": True,
         "host": os.environ.get("COMPUTERNAME") or os.environ.get("HOSTNAME") or "unknown",
         "partition": "test", "purpose": purpose, "run_id": run_id, "split_id": split_id,
         "user": os.environ.get("USERNAME") or os.environ.get("USER") or "unknown"}
    return json.dumps(dict(sorted(e.items())), separators=(",", ":"))


def append_reveals(lines):
    REVEAL_LOG.parent.mkdir(parents=True, exist_ok=True)
    with REVEAL_LOG.open("a", encoding="utf-8", newline="\n") as f:
        for line in lines:
            f.write(line + "\n")
            f.flush()
    return len(lines)


def evaluator_final_reveals():
    n = Counter()
    with REVEAL_LOG.open(encoding="utf-8") as f:
        for line in f:
            try:
                e = json.loads(line)
            except json.JSONDecodeError:
                continue
            if e.get("final_test") is True and e.get("partition") == "test":
                n[(e.get("purpose"), e.get("granted"))] += 1
    return {f"{k[0]}|granted={k[1]}": v for k, v in n.items()}


def close(a, b, tol):
    return a is not None and b is not None and abs(a - b) <= tol


# ------------------------------------------------------------------------------------------------
# the release list
# ------------------------------------------------------------------------------------------------
def load_list(path):
    p = Path(path)
    text = p.read_bytes()
    d = json.loads(text)
    if d.get("schema") != "ignis.wsp.final-release-list/1":
        sys.exit(f"{path}: schema must be ignis.wsp.final-release-list/1")
    fams = d.get("families", {})
    for name, f in fams.items():
        if not isinstance(f.get("release"), str) or not f["release"].startswith("sha256:"):
            sys.exit(f"{path}: family {name} has no release id")
        inputs = f.get("input", "shared")
        inputs = [inputs] if isinstance(inputs, str) else inputs
        if any(i not in INPUT_ROOT for i in inputs):
            sys.exit(f"{path}: family {name}: input must be among {list(INPUT_ROOT)}")
        f["input"] = inputs
        base = f.setdefault("baselines", {})
        # a comparator is an id (scored on every input of the family) or {"id", "input"}
        f["baseline_inputs"] = {}
        for label, v in list(base.items()):
            if isinstance(v, dict):
                vi = v.get("input", inputs)
                vi = [vi] if isinstance(vi, str) else vi
                if any(i not in inputs for i in vi):
                    sys.exit(f"{path}: {name}/{label}: input {vi} is not among the family's inputs {inputs}")
                base[label], f["baseline_inputs"][label] = v["id"], vi
            else:
                f["baseline_inputs"][label] = inputs
    return d, hashlib.sha256(text).hexdigest()


class Mode:
    def __init__(self, a, step):
        flags = (a.execute_final, a.i_am_the_final_run)
        self.final = a.cmd == "final"
        if self.final and not all(flags):
            sys.exit("final mode needs both --execute-final and --i-am-the-final-run")
        if not self.final and any(flags):
            sys.exit("--execute-final / --i-am-the-final-run belong to `final` only")
        if self.final and not a.releases:
            sys.exit("final mode needs --releases <frozen release list>")
        self.step = step
        self.list, self.list_sha = (load_list(a.releases) if a.releases else (None, None))
        if self.list_sha:
            print(f"release list {a.releases}: sha256 {self.list_sha}", flush=True)
        self.partition = "test" if self.final else "validation"
        self.tag = "final" if self.final else "rehearsal"
        self.workers = a.workers

    def begin(self):
        """Final mode: the marker, created exclusively before any test data is touched."""
        if not self.final:
            return
        MARKERS.mkdir(parents=True, exist_ok=True)
        m = MARKERS / f"{self.step}.started"
        try:
            fd = os.open(m, os.O_CREAT | os.O_EXCL | os.O_WRONLY)
        except FileExistsError:
            sys.exit(f"step {self.step} has already been started ({rel(m)}): a final step runs once")
        with os.fdopen(fd, "w", encoding="utf-8") as f:
            f.write(json.dumps({"step": self.step, "started_at": datetime.datetime.now(datetime.timezone.utc).isoformat(),
                                "release_list_sha256": self.list_sha}) + "\n")

    def result_path(self):
        return OUT / (f"{self.step}.json" if self.final else f"rehearsal/{self.step}.json")

    def local(self, *parts):
        p = LOCAL / self.tag / self.step
        for x in parts:
            p = p / x
        return p

    def family(self, name):
        return (self.list or {}).get("families", {}).get(name)


# ------------------------------------------------------------------------------------------------
# step 1: sealed project partitions through the evaluator
# ------------------------------------------------------------------------------------------------
def prefix_info(artifact_id):
    """(run id, prefix hash, carries the planned stop) of a stored forecast (no outcome is in these files)."""
    d = ANALYSIS / artifact_id.split(":")[-1]
    c = json.loads((d / "context.json").read_text(encoding="utf-8"))
    return c["run_id"], c["prefix_hash"], "programmed_stop" in c.get("known_condition_keys", [])


def compare(mode, ids, kind, out):
    args = ["eval", "compare", *ids, "--partition", mode.partition, "--allow-feature-mismatch", "--allow-unit-mismatch",
            "--allow-cohort-mismatch", "--intersect", "--out", str(out)]
    if mode.final:
        args.append("--final-test")
    if kind == "event":
        args += ["--horizons", ",".join(map(str, EVENT_TIMES))]
    ignis(args)
    return json.loads(Path(out).read_text(encoding="utf-8"))


def block_metrics(kind, rows, block=None):
    """The family's judged metrics. From the evaluator's own block when the comparison is one input; else
    recomputed from the rows of that input with the functions of tools/wsp/analysis/cmp_split.py."""
    import cmp_split as cs
    if kind == "droplet":
        out = {}
        for m, key in (("crps", "crps_um"), ("mae_median", "mae_median_um"), ("coverage@0.5", "coverage_50"), ("coverage@0.9", "coverage_90")):
            if block is not None:
                v = block["metrics"].get(m, {}).get("estimate")
            else:
                v = cs.cluster_mean(rows, m)[0]
            out[key] = None if v is None else (v * 1e6 if key.endswith("_um") else v)
        return out
    out = {}
    for cause in ("extinction", "operator_termination"):
        if block is not None:
            out[cause] = block["events"]["causes"][cause]["integrated_brier"]
        else:
            out[cause] = cs.case_mean(rows, f"ibs_contrib[{cause}]")
    out["sum"] = None if None in out.values() else out["extinction"] + out["operator_termination"]
    return out


def paired_metrics(kind, ref_rows, oth_rows, evaluator=None):
    """other minus reference; the evaluator's cluster bootstrap when one input, cmp_split.paired otherwise."""
    import cmp_split as cs
    names = (["crps", "mae_median", "coverage@0.5", "coverage@0.9"] if kind == "droplet"
             else ["ibs_contrib[extinction]", "ibs_contrib[operator_termination]"])
    out = {}
    for m in names:
        if evaluator is not None:
            d = evaluator["differences"].get(m)
            v = None if d is None else dict(estimate=d["estimate"], case_mean=d.get("case_mean"), ci=d["ci"],
                                            n_cases=d["n_cases"], n_clusters=d["n_clusters"], source="evaluator")
        else:
            d = cs.paired(ref_rows, oth_rows, m, 2000, 0)
            v = None if d is None else dict(estimate=d["estimate"], ci=d["ci"], n_cases=d.get("n_cases"),
                                            n_clusters=d["n_clusters"], source="cmp_split.paired (cluster bootstrap)")
            if v is not None:  # case mean of the paired difference (the event tables quote case means)
                ref = {(r["run_id"], r["prefix_hash"]): r for r in ref_rows}
                diffs = [r["metrics"][m] - ref[(r["run_id"], r["prefix_hash"])]["metrics"][m] for r in oth_rows
                         if (r["run_id"], r["prefix_hash"]) in ref and r["scored"] and ref[(r["run_id"], r["prefix_hash"])]["scored"]
                         and r["metrics"].get(m) is not None and ref[(r["run_id"], r["prefix_hash"])]["metrics"].get(m) is not None]
                v["case_mean"] = sum(diffs) / len(diffs) if diffs else None
        if v is not None and kind == "droplet" and m in ("crps", "mae_median"):
            for k in ("estimate", "case_mean"):
                if v.get(k) is not None:
                    v[k] *= 1e6
            v["ci"] = None if v["ci"] is None else [x * 1e6 for x in v["ci"]]
            v["unit"] = "um"
        out[m] = v
    return out


def summarise_compare(kind, cmp, release, keysets):
    """Aggregates of one comparison, per input. keysets: input -> set of (run, prefix) of the release on it."""
    ids = cmp["releases"]
    rows = cmp["cases"]
    if keysets is None:   # one input only: every case of the comparison
        keysets = {"shared": {(r["run_id"], r["prefix_hash"]) for r in rows[release]}}
    present = {inp: [r for r in rows[release] if (r["run_id"], r["prefix_hash"]) in ks] for inp, ks in keysets.items()}
    one_input = sum(1 for v in present.values() if v) <= 1 and sum(len(v) for v in present.values()) == len(rows[release])
    out = {"releases": ids, "reference": cmp["reference"], "n_cases_total": cmp["n_cases"],
           "excluded_incomplete_cases": len(cmp["excluded_incomplete_cases"]), "per_input": {}}
    for inp, ks in keysets.items():
        sel = {rid: [r for r in rows[rid] if (r["run_id"], r["prefix_hash"]) in ks] for rid in ids}
        if not sel[release]:
            continue
        b = {"cases": len(sel[release]), "runs": len({r["run_id"] for r in sel[release]}),
             "sessions": len({r["cluster"] for r in sel[release]}),
             "scored": sum(1 for r in sel[release] if r["scored"]), "per_release": {}, "paired_vs_reference": {}}
        for rid in ids:
            blk = cmp["per_release"][rid] if one_input else None
            b["per_release"][rid] = block_metrics(kind, sel[rid], blk)
        for rid in ids:
            if rid == cmp["reference"]:
                continue
            ev = cmp["paired_vs_reference"][rid] if one_input else None
            b["paired_vs_reference"][rid] = paired_metrics(kind, sel[cmp["reference"]], sel[rid], ev)
        b["source"] = "evaluator aggregate" if one_input else "split by input from the evaluator's rows"
        out["per_input"][inp] = b
    return out


def stop_index(release):
    """(run, prefix) -> carries the planned stop, for every sealed forecast of a release (rehearsal)."""
    out = {}
    for p in SEALS.glob("*.json"):
        s = json.loads(p.read_text(encoding="utf-8"))
        if s["model_release_id"] != release:
            continue
        run, ph, stop = prefix_info(s["sealed"]["artifact_id"])
        out[(run, ph)] = stop
    return out


def check_split_tool():
    """The split by input reproduces the evaluator on a doubled comparison of 14 November (validation, local)."""
    import cmp_split as cs
    mixed = json.loads((REPO / SPLIT_CHECK[0]).read_text(encoding="utf-8"))
    priv = json.loads((REPO / SPLIT_CHECK[1]).read_text(encoding="utf-8"))
    rels = mixed["releases"]
    keys = {(c["run_id"], c["prefix_hash"]) for c in priv["cases"][rels[0]]}
    worst = 0.0
    for rid in rels:
        half = [c for c in mixed["cases"][rid] if (c["run_id"], c["prefix_hash"]) in keys]
        for cause in ("extinction", "operator_termination"):
            mine = cs.case_mean(half, f"ibs_contrib[{cause}]")
            tool = priv["per_release"][rid]["events"]["causes"][cause]["integrated_brier"]
            worst = max(worst, abs(mine - tool))
    return {"mixed": SPLIT_CHECK[0], "alone": SPLIT_CHECK[1], "releases": len(rels),
            "doubled_cases": mixed["n_cases"], "half_cases": priv["n_cases"],
            "max_abs_difference_to_evaluator": worst, "exact": worst <= 1e-12}


def rehearsal_families():
    """The releases of the step as the rehearsal runs them: the proposed list if given, else that file."""
    d, _ = load_list(HERE / "releases_proposed.json")
    return d["families"]


def step_sealed(mode):
    mode.begin()
    fams = (mode.list["families"] if mode.final else (mode.list or {"families": rehearsal_families()})["families"])
    fams = {k: v for k, v in fams.items() if k in SEALED_FAMILIES}
    res = {"step": "sealed", "mode": mode.tag, "partition": mode.partition, "binaries": binaries(),
           "release_list_sha256": mode.list_sha, "families": {}}
    seals = seal_index()
    if mode.final:
        # 1. no test seal may exist for a listed release (a seal after a reveal would be refused anyway)
        for name, f in fams.items():
            for rid in [f["release"], *f["baselines"].values()]:
                if seals[(rid, "test")]:
                    sys.exit(f"{name}: {rid} already has {seals[(rid, 'test')]} test seals; the final run seals once")
        # 2. deposits, 3. every seal of every family and input, before any comparison reveals a test run
        plan = {}
        for name, f in fams.items():
            kind = FAMILY_KIND[name]
            ids = [f["release"], *f["baselines"].values()]
            meta = {rid: release_json(rid) for rid in ids}
            fam = res["families"][name] = {"kind": kind, "release": f["release"], "baselines": f["baselines"],
                                           "inputs": f["input"], "deposits": [], "forecasts": {}}
            # the continuations come from the shared records into the shared vault, as for validation: the
            # forecasts of runs/dev/data_ext2 were scored on validation against exactly those continuations
            for cohort, split in sorted({(m["cohort_id"], m["split_id"]) for m in meta.values()}):
                r = ignis(["eval", "deposit-cohort", "--cohort", cohort, "--split", split, "--partition", "test"])
                fam["deposits"].append(dict(cohort=cohort, split=split, result=json.loads(r.stdout)))
            for inp in f["input"]:
                on_inp = [f["release"]] + [b for lab, b in f["baselines"].items() if inp in f["baseline_inputs"][lab]]
                for rid in on_inp:
                    m = meta[rid]
                    r = ignis(["infer", "forecast-cohort", "--release", rid, "--split", m["split_id"], "--partition", "test",
                               "--final-test", "--cutoffs", "1,3,5", "--settings", SETTINGS[kind], "--execute", "--seal"],
                              data=INPUT_ROOT[inp], check=False)
                    lst = mode.local(name, f"forecast_{inp}_{rid.split(':')[-1][:16]}.json")
                    lst.parent.mkdir(parents=True, exist_ok=True)
                    lst.write_text(r.stdout, encoding="utf-8")        # per-case listing: local only
                    if r.returncode != 0:
                        if rid == f["release"]:
                            sys.exit(f"{name}/{inp}: forecast-cohort of the release failed (exit {r.returncode}); see {rel(lst)}")
                        try:
                            err = json.loads(r.stdout).get("error")
                        except json.JSONDecodeError:
                            err = (r.stderr or r.stdout)[-300:]
                        fam["forecasts"].setdefault(inp, {})[rid] = dict(failed=True, error=err)
                        continue
                    d = json.loads(r.stdout)
                    fam["forecasts"].setdefault(inp, {})[rid] = dict(
                        artifacts=d["artifacts"], sealed=d["sealed"], refused=d["refused"],
                        refusal_counts=d["refusal_counts"], not_at_risk=d["not_at_risk"],
                        runs=len({c["run_id"] for c in d["cases"]}))
                    plan.setdefault(name, {}).setdefault(inp, {})[rid] = [c["artifact_id"] for c in d["cases"] if c["status"] == "artifact"]
        # 4. comparisons, one per comparator, split by the input each case was sealed on
        for name, f in fams.items():
            kind = FAMILY_KIND[name]
            fam = res["families"][name]
            keysets = None if f["input"] == ["shared"] else {
                inp: {prefix_info(a)[:2] for a in plan[name][inp][f["release"]]} for inp in f["input"]}
            fam["comparisons"] = {}
            for label, b in f["baselines"].items():
                try:
                    cmp = compare(mode, [b, f["release"]], kind, mode.local(name, f"cmp_{label_slug(label)}.json"))
                except SystemExit as e:
                    fam["comparisons"][label] = {"error": str(e)[-600:]}
                    continue
                fam["comparisons"][label] = summarise_compare(kind, cmp, f["release"], keysets)
                write_json(OUT / "sealed.partial.json", res)
        res["evaluator_reveal_lines_final"] = evaluator_final_reveals()
    else:
        res["split_tool_check"] = check_split_tool()
        for name, f in fams.items():
            kind = FAMILY_KIND[name]
            ids = [f["release"], *f["baselines"].values()]
            fam = res["families"][name] = {"kind": kind, "release": f["release"], "baselines": f["baselines"],
                                           "inputs": f["input"], "validation_seals": {}, "plan": {},
                                           "not_rehearsed": [], "comparisons": {}, "registered_looks": {}}
            for rid in ids:
                fam["validation_seals"][rid] = seals[(rid, "validation")]
            # forecast-cohort as a plan only (no --execute, no --seal): counts, never a new seal
            for inp in f["input"]:
                for rid in [f["release"]] + [b for lab, b in f["baselines"].items() if inp in f["baseline_inputs"][lab]]:
                    m = release_json(rid)
                    r = ignis(["infer", "forecast-cohort", "--release", rid, "--split", m["split_id"], "--partition",
                               "validation", "--cutoffs", "1,3,5", "--settings", SETTINGS[kind]], data=INPUT_ROOT[inp])
                    d = json.loads(r.stdout)
                    fam["plan"].setdefault(inp, {})[rid] = dict(cases=d["cases"], runs=d["runs"], not_at_risk=d["not_at_risk_cases"])
            keysets = None
            if f["input"] != ["shared"]:
                stops = stop_index(f["release"])
                keysets = {"shared": {k for k, s in stops.items() if not s}, "data_ext2": {k for k, s in stops.items() if s}}
                keysets = {k: v for k, v in keysets.items() if k in f["input"]}
            for label, b in f["baselines"].items():
                if not seals[(b, "validation")]:
                    fam["not_rehearsed"].append(f"{label}: no validation seal (a rehearsal never seals)")
                    continue
                cmp = compare(mode, [b, f["release"]], kind, mode.local(name, f"cmp_{label_slug(label)}.json"))
                fam["comparisons"][label] = summarise_compare(kind, cmp, f["release"], keysets)
            # the registered looks, release sets as they were run, must give the quoted numbers
            for inp, src in REGISTERED_LOOKS[name]:
                ids_reg = json.loads((REPO / src).read_text(encoding="utf-8"))["releases"]
                cmp = compare(mode, ids_reg, kind, mode.local(name, f"registered_{inp}.json"))
                got = block_metrics(kind, None, cmp["per_release"][f["release"]])
                exp = EXPECT_SEALED[name][inp]
                tol = 0.05 if kind == "droplet" else 0.00005
                fam["registered_looks"][inp] = dict(source=src, releases=len(ids_reg), cases=cmp["n_cases"], got=got,
                                                    expected=exp, reproduced=all(close(round(got[k], 1 if kind == "droplet" else 4), v, tol)
                                                                                 for k, v in exp.items()))
        res["reproduced"] = res["split_tool_check"]["exact"] and all(
            lk["reproduced"] for fam in res["families"].values() for lk in fam["registered_looks"].values())
    write_json(mode.result_path(), res)
    return res


def label_slug(s):
    return "".join(ch if ch.isalnum() else "_" for ch in s).strip("_")[:60]


# ------------------------------------------------------------------------------------------------
# step 2: Flame Design test days (chain of tools/wsp/ext/transfer_f2.py)
# ------------------------------------------------------------------------------------------------
def fd_modules():
    import transfer_f2 as tf2  # noqa: E402  (sets the fd_fuel layout of transfer_acme and flame_design)
    return tf2, tf2.ta, tf2.g6, tf2.tf, tf2.tf.fdm


def fd_check_days(ta):
    s = json.loads((REPO / "docs/wsp/atlas/splits/PSI-10.json").read_text(encoding="utf-8"))
    for fam, stratum in (("acme_normal", "normal_flames"), ("acme_inverse", "inverse_flames")):
        g = s["strata"][stratum]["groups"]
        days = {f"{p}_days": {x.split(":")[-1] for x in g[p]} for p in ("train", "validation", "test")}
        assert days == ta.FD_DAYS[fam], (fam, days, ta.FD_DAYS[fam])


def fd_roles(mode, tf2):
    roles = {}
    for fam in ("acme_normal", "acme_inverse"):
        f = mode.family(f"flame_design_{fam.split('_')[1]}")
        if f is None:
            roles[fam] = dict(tf2.ROLES[fam])
            continue
        b = f["baselines"]
        roles[fam] = dict(f2=f["release"], f1=b["F1"], surrogate=b["ethylene as ethane"])
    return roles


def fd_test_day_tests(ta, fdm, cond_path, end_path):
    """The fd_tests() rows of transfer_acme for the TEST days, endpoints from this run's own file."""
    nan = float("nan")
    with cond_path.open(encoding="utf-8", newline="") as f:
        cond = [dict(r, row=i + 2) for i, r in enumerate(csv.DictReader(f)) if r["include_transfer"] == "1"]
    for r in cond:
        r["part"] = ta.fd_part(r["family"], r["test"])
    cond = [r for r in cond if r["part"] == "test_days"]
    with end_path.open(encoding="utf-8", newline="") as f:
        ends = {r["test"]: dict(r, erow=i + 2) for i, r in enumerate(csv.DictReader(f))}
    tests = []
    for r in cond:
        e = ends.get(r["test"])
        t = dict(r, erow=e["erow"] if e else None, endpoint=None, why="not measured", programmed_stop=nan,
                 onset_method="", eflags="")
        if e:
            num = lambda s: float(s) if s not in ("", None) else nan  # noqa: E731
            t.update(programmed_stop=num(e["script_programmed_stop_s"]), onset_method=e["onset_method"], eflags=e["flags"])
            end, label = num(e["end_s"]), e["label_sensor"]
            if e["status"] != "ok":
                t["why"] = f"status {e['status']}"
            elif label in ta.CAUSES and end == end:
                t["endpoint"] = dict(censoring="event", last_confirmed_active_s=end, first_terminal_s=end, allowed_causes=[label])
                t["why"] = ""
            elif label == "burning_at_file_end" and end == end:
                t["endpoint"] = dict(censoring="right", last_confirmed_active_s=end, first_terminal_s=None, allowed_causes=[])
                t["why"] = ""
            else:
                t["why"] = f"label {label or 'none'}"
        tests.append(t)
    return tests


def fd_missing_files(ta, fdm, cond_path, part):
    """Raw files the tests of a partition need that are not on disk (file names and existence only)."""
    names = fdm.listing()
    with cond_path.open(encoding="utf-8", newline="") as f:
        tests = [r["test"] for r in csv.DictReader(f) if r["include_transfer"] == "1" and ta.fd_part(r["family"], r["test"]) == part]
    miss = {t: [n for n in fdm.file_names(t, names).values() if n and not (fdm.FILES / n).exists()] for t in tests}
    return tests, {t: v for t, v in miss.items() if v}


def step_flame_design(mode):
    tf2, ta, g6, tf, fdm = fd_modules()
    mode.begin()
    binary = ta.use_binary(argparse.Namespace(ignis=IGNIS_REL))
    chain = json.loads((tf2.WORK / "cfig/chain_check.json").read_text(encoding="utf-8"))
    if not (chain["reproduced"] and chain["ignis"]["sha256"] == binary["sha256"]):
        sys.exit("the CFI-G train chain check of transfer_f2.py is not reproduced with this binary")
    fd_check_days(ta)
    tf2.ROLES = fd_roles(mode, tf2)
    baseline = {fam: v["cutoffs"] for fam, v in json.loads(ta.BASELINE.read_text(encoding="utf-8"))["families"].items()}
    res = {"step": "flame-design", "mode": mode.tag, "binaries": binaries(), "release_list_sha256": mode.list_sha,
           "releases": tf2.ROLES, "baseline_file": rel(ta.BASELINE), "baseline_sha256": sha256_file(ta.BASELINE),
           "bootstrap": dict(resamples=ta.N_BOOT, seed=ta.BOOT_SEED, unit="session"), "tests": {}, "blocks": {}}
    tf2.WORK = mode.local("forward")
    if mode.final:
        cond_src = ta.FD_WORK / "conditions.csv"
        here = mode.local("flame_design")
        here.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(cond_src, here / "conditions.csv")            # byte copy; the prefixes name this run's files
        g6.FD_WORK = here
        tests_all, missing = fd_missing_files(ta, fdm, here / "conditions.csv", "test_days")
        if missing:   # the data lane's command, public NASA files of the included tests
            ta.cmd_fd_fetch(argparse.Namespace(ignis=IGNIS_REL, execute=True, batch=25, retries=2))
            _, missing = fd_missing_files(ta, fdm, here / "conditions.csv", "test_days")
            if missing:
                sys.exit(f"{len(missing)} test-day tests still lack raw files after the fetch; nothing scored")
        names = fdm.listing()
        rows = []
        for test in tests_all:       # flame_design.py `endpoints`, thresholds unchanged, written to this run's folder
            try:
                row = fdm.measure(test, fdm.file_names(test, names))
            except Exception as e:  # noqa: BLE001  (as cmd_endpoints: one unreadable file must not hide the others)
                row = dict(test=test, status=f"error:{type(e).__name__}", flags=[])
            row["flags"] = "|".join(row.get("flags", []))
            rows.append({k: fdm.fmt(row.get(k, "")) for k in fdm.FIELDS})
        with (here / "endpoints.csv").open("w", encoding="utf-8", newline="") as f:
            w = csv.DictWriter(f, fieldnames=fdm.FIELDS)
            w.writeheader()
            w.writerows(rows)
        res["endpoint_status"] = dict(Counter(r["status"] for r in rows))
        sha_c, sha_e = sha256_file(here / "conditions.csv"), sha256_file(here / "endpoints.csv")
        tests = fd_test_day_tests(ta, fdm, here / "conditions.csv", here / "endpoints.csv")
        part = "test_days"
    else:
        tests, sha_c, sha_e, _ = ta.fd_tests()
        tests = [t for t in tests if t["part"] == "validation_days"]
        part = "validation_days"
    assert all(t["test"][:5] in ta.FD_DAYS[t["family"]][part] for t in tests)
    res["inputs"] = dict(conditions_csv_sha256=sha_c, endpoints_csv_sha256=sha_e, partition=part)
    reveals = []
    split_id = split_ref(REPO / "docs/wsp/atlas/splits/PSI-10.json")
    for fam in tf2.ROLES:
        tt = [t for t in tests if t["family"] == fam]
        res["tests"][fam] = dict(tests=len(tt), sessions=sorted({t["session"] for t in tt}),
                                 by_fuel=dict(Counter(t["fuel_species"] for t in tt)),
                                 without_endpoint=sum(t["endpoint"] is None for t in tt),
                                 why_without=dict(Counter(t["why"] for t in tt if t["endpoint"] is None)))
        for stop in (False, True):
            cases, br = tf2.build(fam, tt, stop, sha_c, sha_e, mode.workers)
            b = tf2.summarise(cases, baseline)
            b["f2_new_stop_branches"] = br
            if fam == "acme_normal":
                b["by_fuel"] = {fu: tf2.summarise([k for k in cases if k["fuel"] == fu], baseline)
                                for fu in sorted({k["fuel"] for k in cases})}
            res["blocks"][f"{fam}/{'stop' if stop else 'nostop'}"] = b
            if mode.final:
                for k in cases:
                    for role, a in k["art"].items():
                        reveals.append(reveal_line(a["id"], f"PSI-10:{fam}:{k['test']}", split_id, "final:flame_design"))
    if mode.final:
        res["reveal_lines_appended"] = append_reveals(reveals)
    else:
        ref = json.loads(FD_REPORT.read_text(encoding="utf-8"))
        worst, table = 0.0, {}
        for key, b in res["blocks"].items():
            rb = ref["blocks"][key]
            s = b["all_answer"]["score"]
            for role in ("f2", "f1", "surrogate", "incidence_baseline"):
                for x in tf2.METRICS:
                    worst = max(worst, abs(s[role][x]["case_mean"] - rb["all_answer"]["score"][role][x]["case_mean"]))
            got = {r: tuple(round(s[r][x]["case_mean"], 4) for x in tf2.METRICS) for r in ("f2", "f1")}
            e = EXPECT_FD[key]
            table[key] = dict(cases=b["cases"], got=got, expected={r: e[r] for r in ("f2", "f1")},
                              reproduced=b["cases"] == e["cases"] and all(close(got[r][i], e[r][i], 5e-5) for r in ("f2", "f1") for i in range(3)))
        res["decision_inputs"] = table
        res["max_abs_difference_to_report_json"] = worst
        res["reproduced"] = worst <= 1e-12 and all(v["reproduced"] for v in table.values())
    write_json(mode.result_path(), res)
    return res


# ------------------------------------------------------------------------------------------------
# step 3: CFI test days (chain of tools/wsp/ext/transfer_droplet_arms.py, inputs of cfi_cases.py)
# ------------------------------------------------------------------------------------------------
def cfi_rows(partition):
    import cfi_cases as cc
    cond = [json.loads(x) for x in (cc.WORK / "conditions.jsonl").read_text(encoding="utf-8").splitlines() if x.strip()]
    # the selection rule of the registered set (PROTOCOL 12.7): n-dodecane, free droplet, a processed series,
    # inside the domain of the dodecane release; on train/validation it gives exactly cfi_cases.TESTS
    sel = [c for c in cond if c["fuel"] == "n-dodecane" and c["fibre"] == "free" and c["droplet_series"]
           and c["gates"].get("wsp5/dodecane/w0-fable") == []]
    dev = {c["flex"] for c in sel if c["partition"] in ("train", "validation")}
    assert dev == CFI_DEV_TESTS, f"the selection rule does not give the registered set: {sorted(dev)}"
    want = [c for c in sel if c["partition"] == partition]
    rows = []
    src = "PSI-39_Science Documents_CFI_DataSummary_Spreadsheet_Jan2021.xlsx (Master Sheet)"
    for c in want:
        t = c["flex"]
        conditions = dict(
            x_o2=dict(value=c["x_o2"], unit="1", source=src, column="Oxygen"),
            x_co2=dict(value=0.0, unit="1", source=src, column="(no CO2 column; CFI used none)", evidence="ignis_derived"),
            x_he=dict(value=c["x_he"], unit="1", source=src, column="Helium"),
            pressure=dict(value=c["pressure_pa"], unit="Pa", source=src, column="Pressure (nominal atm x 101325)"),
            fuel=dict(value="n-Dodecane", unit=None, source=src, column="Fuel (C12H26)"))
        path = cc.RAW / f"PSI-39_Raw Data_FLEX-{t}_DropletData.txt"
        if not path.exists():
            sys.exit(f"FLEX-{t}: {rel(path)} is not on disk; nothing scored")
        obs = cc.series(path, f"PSI-39:FLEX-{t}")
        rows.append(dict(run_id=f"PSI-39:FLEX-{t}", session_id=c["group"], investigation="PSI-39", partition=c["partition"],
                         test=f"FLEX-{t}", fuel="n-dodecane", conditions=conditions, observations=obs, endpoint=None))
    return rows


def step_cfi(mode):
    import transfer_droplet as td
    import transfer_droplet_arms as arms
    mode.begin()
    f = mode.family("cfi_dodecane") or {"release": "sha256:70042961d2a45df07a93a0cc3ed828dd8b022de5dbebbe9b97d8b18f7f8d3e74",
                                        "baselines": {"root": "configs/models/flex/wsp7/dodecane/baseline-d2"}}
    rel_ids = {"candidate": arms.release_id(f["release"]), "root": arms.release_id(f["baselines"]["root"])}
    bins = td.use_binaries(argparse.Namespace(ignis=IGNIS_REL, wsp_dev=WSPDEV_REL))
    rows_in = cfi_rows("test" if mode.final else "validation")
    meta = {r["run_id"]: r for r in rows_in}
    rows, nar, pre_ref = td.run_cases([td.ext_case_inputs(r) for r in rows_in], rel_ids, mode.local("chain"), mode.workers)
    spec = td.score_rows(rows, rel_ids)
    for r in rows:
        r["partition"] = meta[r["run_id"]]["partition"]
        r["d2_law"] = arms.d2_law(json.loads(r["prefix_path"].read_text(encoding="utf-8")), r["cont"])
    roles = list(rel_ids)
    own = arms.summarise(rows, roles, lambda r: True)
    com, cs = arms.common_block(rows, roles, lambda r: True)
    pairs = [arms.paired(cs, a, b) for a, b in (("candidate", "root"), ("candidate", "persistence"), ("candidate", "d2_law"),
                                                 ("root", "d2_law"))]
    res = {"step": "cfi", "mode": mode.tag, "binaries": bins, "release_list_sha256": mode.list_sha, "releases": rel_ids,
           "partition": "test days" if mode.final else "validation days", "tests": len(rows_in), "cases": len(rows),
           "not_at_risk": nar, "prefix_refusals": len(pre_ref),
           "refusals": {role: dict(Counter(r["metrics"][role]["refusal"] for r in rows if not r["metrics"][role]["admitted"])) for role in roles},
           "own_cases": own, "common_cases": com, "paired": pairs, "units": "metres (CRPS, MAE); x1e6 for micrometres"}
    write_json(mode.local("cases.json"), dict(note="per case: scoring agent only", cases=[
        dict(run_id=r["run_id"], session_id=r["session_id"], cutoff_s=r["cutoff_s"], persistence=r["persistence"],
             d2_law=r["d2_law"], releases=r["metrics"]) for r in rows]))
    if mode.final:
        split_id = split_ref(REPO / "docs/wsp/atlas/splits/PSI-39.json")
        res["reveal_lines_appended"] = append_reveals(
            [reveal_line(r["artifacts"][role]["id"], r["run_id"], split_id, "final:cfi") for r in rows for role in roles])
    else:
        um = lambda v: None if v is None else round(v * 1e6, 1)  # noqa: E731
        got = {"cases": com["cases"], "tests": com["tests"], "days": com["days"],
               "candidate": dict(crps=um(com["candidate"]["crps"]), mae_median=um(com["candidate"]["mae_median"]),
                                 cov50=round(com["candidate"]["coverage@0.5"], 2), cov90=round(com["candidate"]["coverage@0.9"], 2)),
               "root": dict(crps=um(com["root"]["crps"]), mae_median=um(com["root"]["mae_median"]),
                            cov50=round(com["root"]["coverage@0.5"], 2), cov90=round(com["root"]["coverage@0.9"], 2)),
               "persistence": um(com["persistence"]["crps"]), "d2_law": um(com["d2_law"]["crps"]),
               "paired_candidate_minus_root": um(pairs[0]["mean_difference"])}
        ref = json.loads(CFI_AGGREGATE.read_text(encoding="utf-8"))["scopes"]["validation"]["common_cases"]
        worst = max(abs(com[r][k] - ref[r][k]) for r in roles for k in arms.KEYS)
        res.update(got=got, expected=EXPECT_CFI, max_abs_difference_to_aggregate_json=worst,
                   reproduced=got == EXPECT_CFI and worst <= 1e-15)
    write_json(mode.result_path(), res)
    return res


# ------------------------------------------------------------------------------------------------
# step 4: fabric flames (wsp-dev front-feedback)
# ------------------------------------------------------------------------------------------------
def front_feedback_has_final_mode():
    r = subprocess.run([str(WSPDEV), "front-feedback", "--help"], capture_output=True, text=True, cwd=REPO)
    return "--final-test" in r.stdout


def psi26_test_folder(mode, map_path):
    """Final mode: copy the PSI-26 tracks that the frozen partition map puts on test days into a local folder
    (bytes unchanged, nothing parsed), because `front-feedback --final-test` refuses a mixed folder. The map
    names a track by run id (`PSI-26:<stem>`) or by file stem, as front-feedback reads it."""
    m = json.loads(Path(map_path).read_text(encoding="utf-8"))
    if not isinstance(m, dict) or any(v not in ("train", "validation", "test") for v in m.values()):
        sys.exit(f"{map_path}: a partition map is a JSON object of run id or file stem -> train | validation | test")
    test_keys = {k for k, v in m.items() if v == "test"}
    if not test_keys:
        return None, 0
    dest = mode.local("psi26_test_tracks")
    if dest.exists():
        sys.exit(f"{rel(dest)} exists: the PSI-26 test folder is built once, by this step")
    dest.mkdir(parents=True)
    n = 0
    for f in sorted(PSI26_TRACKS.glob("*.json")):
        if f.stem in test_keys or f"PSI-26:{f.stem}" in test_keys:
            shutil.copyfile(f, dest / f.name)
            n += 1
    return dest, n


def wspdev_env():
    """The default run root, so that the reveal lines land in runs/evaluation/reveal_log.jsonl."""
    env = dict(os.environ)
    env.pop("IGNIS_RUNS", None)
    return env


def step_fabric(mode):
    f = mode.family("fabric") or {"release": "sha256:00a8bc356d8e4e02a58b87d4786feeafd25b55b90a74b45e29c797054b9c5e72",
                                  "baselines": {"D1": "sha256:ac15e49f1cb60d4b4679bcdff82a07c18a4a501454428e35ac86e935e168855c",
                                                "carry of D1": "sha256:4da9967f32a8935890dbe07c700f7b89cfd589e177800fc2116ffcf5069b8ed4"}}
    ids = [f["baselines"].get("D1"), f["release"], *[v for k, v in f["baselines"].items() if k != "D1"]]
    ids = [i for i in ids if i]
    psi26_map = f.get("psi26_partition_map")
    psi26_skipped = None
    if psi26_map and not PSI26_CONDITIONS_DONE.exists():
        # PSI-26 tracks carry no conditions yet (oxygen, flow, material, width null): no model can run on them
        psi26_skipped = f"{rel(PSI26_CONDITIONS_DONE)} does not exist: PSI-26 is not scored and its test days stay unread"
        print(psi26_skipped, flush=True)
        psi26_map = None
    if mode.final and psi26_map and f.get("psi26_map_from_frozen_split"):
        # the map of the tracks that exist at the final run, from the frozen day split (file names only)
        mp = REPO / psi26_map
        if mp.exists():
            mp.unlink()
        r = subprocess.run([sys.executable, "-I", str(REPO / "tools/wsp/atlas/freeze_psi26_days.py"), "--map", str(mp)],
                           capture_output=True, text=True, cwd=REPO)
        if r.returncode != 0:
            sys.exit(f"freeze_psi26_days.py --map failed: {(r.stderr or r.stdout).strip()[-600:]}")
        print(r.stdout.strip(), flush=True)
    if mode.final:
        # `front-feedback --final-test` scores test tracks only (a mixed folder is refused), never prints a
        # per-case line, and appends one reveal line per scored burn (purpose final:bass_front) itself.
        if not front_feedback_has_final_mode():
            sys.exit("wsp-dev front-feedback in target-merged has no --final-test mode: rebuild it; nothing was read.")
        if psi26_map and not (REPO / psi26_map).is_file():
            sys.exit(f"the release list names the PSI-26 partition map {psi26_map}, which does not exist; nothing was read.")
        mode.begin()
        bass_split = json.loads(BASS_SPLIT.read_text(encoding="utf-8"))["split"]["id"]
        # the converted test tracks named by the list (the conversion already left out failed and incomplete tracks)
        tracks = REPO / f.get("tracks", rel(FABRIC_SCORER / "test"))
        qinfo = {"BASS-II test partition": conversion_left_out("test")}
        folders = [("BASS-II test partition", tracks, ["--final-test", "--split-id", bass_split])]
        if psi26_map:
            dest, n = psi26_test_folder(mode, REPO / psi26_map)
            if dest is None:
                print(f"the PSI-26 map {psi26_map} names no test day: PSI-26 is not scored here", flush=True)
            elif n == 0:
                sys.exit(f"the PSI-26 map {psi26_map} names test tracks, but none is among {rel(PSI26_TRACKS)}")
            else:
                folders.append(("PSI-26 test days", dest, ["--final-test", "--partition-map", str(REPO / psi26_map)]))
        else:
            print("the release list names no PSI-26 partition map: PSI-26 test days are not scored", flush=True)
    else:
        # the reproduction (the four train tracks of 12.9), and the quality filter rehearsed on the measurer's train folder
        # the reproduction (the four train tracks of 12.9), and the final code path on the converted train tracks
        qinfo = {"BASS-II converted train tracks": conversion_left_out("train")}
        folders = [("BASS-II train tracks", FABRIC_TRAIN, []),
                   ("BASS-II converted train tracks", FABRIC_SCORER / "train", [])]
    res = {"step": "fabric", "mode": mode.tag, "binaries": binaries(), "release_list_sha256": mode.list_sha,
           "releases": ids, "folders": {}}
    for label, folder, extra in folders:
        out = mode.local(f"front_{label_slug(label)}.json")
        det = mode.local(f"front_{label_slug(label)}_detail.json")
        out.parent.mkdir(parents=True, exist_ok=True)
        cmd = [str(WSPDEV), "front-feedback", *sum((["--release", i] for i in ids), []), "--tracks", str(folder),
               "--label", f"final run {DAY} {mode.tag}: {label}", "--quiet-cases", "--out", str(out), "--detail", str(det), *extra]
        r = subprocess.run(cmd, capture_output=True, text=True, cwd=REPO, env=wspdev_env())
        if r.returncode != 0:
            sys.exit(f"front-feedback failed (exit {r.returncode}): {(r.stderr or r.stdout).strip()[-800:]}")
        d = json.loads(out.read_text(encoding="utf-8"))
        blk = {"counts": d["counts"], "releases": {}}
        for x in d["releases"]:
            a = x["aggregate"]
            blk["releases"][x["release_id"]] = dict(
                tip_crps=a["overall"]["crps"]["mean_over_runs"], ae_median=a["overall"]["ae_median"]["mean_over_runs"],
                coverage_50=a["overall"]["coverage_50"]["mean_over_runs"], coverage_90=a["overall"]["coverage_90"]["mean_over_runs"],
                track_ended_brier=a["events"]["brier_mean_over_runs"], ignition_brier=a["ignition_outcome"]["brier_mean_over_tracks"],
                ignition_tracks=a["ignition_outcome"]["tracks"], ignition_refused=a["ignition_outcome"]["tracks_refused"],
                cases_scored=a["cases_scored"], cases_refused=a["cases_refused"])
        blk["paired_vs_first"] = [dict(release=p["release_id"], minus=p["first_release_id"], common_cases=p["common_cases"],
                                       crps=p["scores"]["crps"]) for p in d.get("paired_vs_first", [])]
        if label in qinfo:
            blk["quality_filter"] = qinfo[label]
        res["folders"][label] = blk
    res["psi26_skipped"] = psi26_skipped
    if mode.final:
        res["psi26_partition_map"] = psi26_map and dict(path=psi26_map, sha256=sha256_file(REPO / psi26_map))
        res["reveal_lines_final"] = evaluator_final_reveals()
    if not mode.final:
        blk = res["folders"]["BASS-II train tracks"]["releases"]
        res["checks"] = {rid: dict(got=(round(blk[rid]["tip_crps"], 4), round(blk[rid]["ignition_brier"], 4)), expected=e,
                                   reproduced=(round(blk[rid]["tip_crps"], 4), round(blk[rid]["ignition_brier"], 4)) == e)
                         for rid, e in EXPECT_FABRIC.items()}
        res["reproduced"] = all(c["reproduced"] for c in res["checks"].values())
    write_json(mode.result_path(), res)
    return res


# ------------------------------------------------------------------------------------------------
# step 5: FLEX-2 (PSI-68) test days, arm F per fuel (chain of tools/wsp/ext/transfer_droplet_arms.py)
# ------------------------------------------------------------------------------------------------
DROPLET = REPO / "runs/dev/media/droplet"
SCALE_MARKER = DROPLET / "SCALE_PINNED.txt"
FLEX2_FUEL = {"n-octane": "octane", "iso-octane": "isooctane", "n-decane": "decane"}   # arm F fuels (heptane is arm H)
FLEX2_BATCH_TEST = REPO / "docs/wsp/atlas/splits/PSI-68-batch-test.json"
FLEX2_DAYS = REPO / "docs/wsp/atlas/splits/PSI-68-days.json"
# docs/wsp/feedback/flex2_transfer_result.md, n-decane validation days (unpinned scale; plumbing check only)
EXPECT_FLEX2_UNPINNED = {"decane": dict(cases=6, tests=2, days=2, candidate=113.3, root=56.1, persistence=175.5,
                                        d2_law=84.7, paired_candidate_minus_root=57.2)}
FLEX2_UNPINNED_AGG = REPO / "runs/dev/data_all/work/transfer_droplet/flex2_F_decane/aggregate.json"
FLEX2_DEFAULT_FUELS = {
    "octane": {"candidate": "sha256:2c7a77162fa7f71975ae36c4e6928ed7c96c2b2241274000ca5425b86c968297",
               "root": "configs/models/flex/wsp7/octane/baseline-d2"},
    "isooctane": {"candidate": "sha256:b2e57d80a8a76c37448e71456ae7231b9a13c8e9e108f92a2149084dce73f7e2",
                  "root": "configs/models/flex/wsp7/isooctane/baseline-d2"},
    "decane": {"candidate": "sha256:62e406617af4a791bd4706088f788d386a6c44305a81ce8f30bfffd6ed51e5bf",
               "root": "configs/models/flex/wsp7/decane/baseline-d2"}}


def flex2_folder(mode):
    """The series the step reads. Final: the held test-day series under the pinned scale rule. Rehearsal: the
    validation-day series rescaled under it when they exist, else the first (unpinned) series, as plumbing."""
    pinned = SCALE_MARKER.exists()
    f = mode.family("flex2") or {}
    if mode.final:
        if not pinned:
            sys.exit(f"{rel(SCALE_MARKER)} does not exist: the FLEX-2 scale is not pinned (PROTOCOL 12.7, third "
                     "amendment); no test-day series is read")
        # the list's note: the rescaled held series the marker names (the list's own key predates the pinning)
        if "flex2_series_test_v2" in SCALE_MARKER.read_text(encoding="utf-8") and (DROPLET / "flex2_series_test_v2").is_dir():
            return DROPLET / "flex2_series_test_v2", True
        return REPO / f.get("test_series", "runs/dev/media/droplet/flex2_series_test"), True
    v2 = DROPLET / "flex2_series_v2/validation"
    if pinned and v2.is_dir():
        return v2, True
    return DROPLET / "flex2_series/validation", False


def flex2_series(mode, folder):
    """Series files of the partition, accepted by the measurer's own rule (quality = usable), by fuel."""
    want = None
    if mode.final:
        b = json.loads(FLEX2_BATCH_TEST.read_text(encoding="utf-8"))
        want = {t["test_id"] for t in b["tests"]}
        files = [q for q in folder.rglob("*.json") if q.stem in want]
    else:
        files = sorted(folder.glob("*.json"))
    rows, counts = {}, Counter()
    part = "test" if mode.final else "validation"
    for q in sorted(files):
        r = json.loads(q.read_text(encoding="utf-8"))
        if r.get("partition") != part:
            sys.exit(f"{rel(q)} is of partition {r.get('partition')!r}, not {part}: nothing scored")
        fuel_text = r["conditions"]["fuel"]["value"]
        counts[(fuel_text, r.get("quality"))] += 1
        fuel = FLEX2_FUEL.get(fuel_text)
        if fuel is None or r.get("quality") != "usable":
            continue
        rows.setdefault(fuel, []).append((r, q))
    found = {"files": len(files), "expected_tests": len(want) if want else None,
             "by_fuel_and_quality": {f"{k[0]}|{k[1]}": v for k, v in sorted(counts.items())}}
    return rows, found


def step_flex2(mode):
    import transfer_droplet as td
    import transfer_droplet_arms as arms
    folder, pinned = flex2_folder(mode)
    mode.begin()
    bins = td.use_binaries(argparse.Namespace(ignis=IGNIS_REL, wsp_dev=WSPDEV_REL))
    roles_of = (mode.family("flex2") or {}).get("fuels") or FLEX2_DEFAULT_FUELS
    rows_by_fuel, found = flex2_series(mode, folder)
    res = {"step": "flex2", "mode": mode.tag, "binaries": bins, "release_list_sha256": mode.list_sha,
           "series_folder": rel(folder), "scale_pinned": pinned,
           "scale_marker_sha256": sha256_file(SCALE_MARKER) if SCALE_MARKER.exists() else None,
           "acceptance": "the measurer's rule: quality = usable", "found": found, "fuels": {}}
    reveals = []
    split_id = split_ref(FLEX2_DAYS)
    for fuel, rr in sorted(rows_by_fuel.items()):
        rel_ids = {role: arms.release_id(v) for role, v in roles_of[fuel].items()}
        cases_in = [r for r, _ in rr]
        rows, nar, pre_ref = td.run_cases([td.ext_case_inputs(r) for r in cases_in], rel_ids, mode.local(fuel), mode.workers)
        td.score_rows(rows, rel_ids)
        for r in rows:
            r["d2_law"] = arms.d2_law(json.loads(r["prefix_path"].read_text(encoding="utf-8")), r["cont"])
        roles = list(rel_ids)
        com, cs = arms.common_block(rows, roles, lambda r: True)
        pairs = [arms.paired(cs, a, b) for a, b in (("candidate", "root"), ("candidate", "persistence"),
                                                     ("candidate", "d2_law"), ("root", "persistence"))]
        res["fuels"][fuel] = {
            "releases": rel_ids, "tests_accepted": len(cases_in), "cases": len(rows), "not_at_risk": nar,
            "prefix_refusals": len(pre_ref),
            "refusals": {role: dict(Counter(r["metrics"][role]["refusal"] for r in rows if not r["metrics"][role]["admitted"]))
                         for role in roles},
            "own_cases": arms.summarise(rows, roles, lambda r: True), "common_cases": com, "paired": pairs,
            "units": "metres (CRPS, MAE); x1e6 for micrometres"}
        write_json(mode.local(fuel, "cases.json"), dict(note="per case: scoring agent only", cases=[
            dict(run_id=r["run_id"], session_id=r["session_id"], cutoff_s=r["cutoff_s"], persistence=r["persistence"],
                 d2_law=r["d2_law"], releases=r["metrics"]) for r in rows]))
        if mode.final:   # one line per scored test; the revealed object is the held series file
            scored = {r["run_id"] for r in rows}
            reveals += [reveal_line("sha256:" + sha256_file(q), r["run_id"], split_id, "final:flex2")
                        for r, q in rr if r["run_id"] in scored]
    if mode.final:
        res["reveal_lines_appended"] = append_reveals(reveals)
    elif pinned:
        um = lambda v: None if v is None else round(v * 1e6, 1)  # noqa: E731
        com = res["fuels"]["decane"]["common_cases"]
        got = dict(cases=com["cases"], tests=com["tests"], days=com["days"], candidate=um(com["candidate"]["crps"]),
                   root=um(com["root"]["crps"]), persistence=um(com["persistence"]["crps"]), d2_law=um(com["d2_law"]["crps"]),
                   paired_candidate_minus_root=um(res["fuels"]["decane"]["paired"][0]["mean_difference"]))
        exp = dict(cases=6, tests=2, days=2, candidate=72.4, root=57.3, persistence=106.2, d2_law=51.3,
                   paired_candidate_minus_root=15.1)
        res.update(got=got, expected=exp, expected_source="docs/wsp/feedback/flex2_transfer_result_v2.md, n-decane validation days",
                   reproduced=got == exp)
    elif not pinned:
        um = lambda v: None if v is None else round(v * 1e6, 1)  # noqa: E731
        d = res["fuels"]["decane"]
        ref = json.loads(FLEX2_UNPINNED_AGG.read_text(encoding="utf-8"))["scopes"]["validation"]
        com = d["common_cases"]
        got = dict(cases=com["cases"], tests=com["tests"], days=com["days"], candidate=um(com["candidate"]["crps"]),
                   root=um(com["root"]["crps"]), persistence=um(com["persistence"]["crps"]), d2_law=um(com["d2_law"]["crps"]),
                   paired_candidate_minus_root=um(d["paired"][0]["mean_difference"]))
        worst = max(abs(com[r][k] - ref["common_cases"][r][k]) for r in ("candidate", "root") for k in arms.KEYS)
        worst = max(worst, abs(d["paired"][0]["mean_difference"] - ref["paired"][0]["mean_difference"]))
        res.update(plumbing_only="the scale is not pinned: the first series, scored to prove the chain, not a look",
                   got=got, expected=EXPECT_FLEX2_UNPINNED["decane"], max_abs_difference_to_aggregate_json=worst,
                   reproduced=got == EXPECT_FLEX2_UNPINNED["decane"] and worst <= 1e-15)
    write_json(mode.result_path(), res)
    return res


# ------------------------------------------------------------------------------------------------
# preflight (safe in any mode: no test file is opened, file existence and seal counts only)
# ------------------------------------------------------------------------------------------------
def preflight(a):
    fams = (load_list(a.releases)[0] if a.releases else load_list(HERE / "releases_proposed.json")[0])["families"]
    seals = seal_index()
    rep = {"binaries": binaries(), "front_feedback_final_mode": front_feedback_has_final_mode(),
           "markers": sorted(p.name for p in MARKERS.glob("*.started")) if MARKERS.exists() else [],
           "test_seals_of_listed_releases": {}, "releases_in_registry": {}}
    for name, f in fams.items():
        for rid in [f["release"], *f["baselines"].values()]:
            if not rid.startswith("sha256:"):
                continue
            rep["releases_in_registry"][rid] = (REPO / "artifacts/models" / rid.split(":")[-1] / "release.json").exists()
            rep["test_seals_of_listed_releases"][rid] = seals[(rid, "test")]
    tf2, ta, g6, tf, fdm = fd_modules()
    tests, missing = fd_missing_files(ta, fdm, ta.FD_WORK / "conditions.csv", "test_days")
    rep["flame_design_test_days"] = dict(included_tests=len(tests), tests_missing_raw_files=len(missing),
                                         files_missing=sum(len(v) for v in missing.values()))
    import cfi_cases as cc
    cond = [json.loads(x) for x in (cc.WORK / "conditions.jsonl").read_text(encoding="utf-8").splitlines() if x.strip()]
    sel = [c for c in cond if c["fuel"] == "n-dodecane" and c["fibre"] == "free" and c["droplet_series"]
           and c["gates"].get("wsp5/dodecane/w0-fable") == [] and c["partition"] == "test"]
    rep["cfi_test_days"] = dict(tests=len(sel), series_on_disk=sum((cc.RAW / f"PSI-39_Raw Data_FLEX-{c['flex']}_DropletData.txt").exists() for c in sel))
    pm = fams.get("fabric", {}).get("psi26_partition_map")
    rep["fabric"] = dict(bass2_held_test_folder_exists=FABRIC_TEST.is_dir(),
                         psi26_day_split_committed=any((REPO / "docs/wsp/atlas/splits").glob(PSI26_SPLIT_GLOB)),
                         psi26_partition_map=pm, psi26_partition_map_exists=bool(pm) and (REPO / pm).is_file(),
                         psi26_conditions_done=PSI26_CONDITIONS_DONE.exists())
    b = json.loads(FLEX2_BATCH_TEST.read_text(encoding="utf-8"))
    held = DROPLET / "flex2_series_test"
    names = {q.stem for q in held.rglob("*.json")} if held.is_dir() else set()   # file names only, nothing opened
    rep["flex2_test_days"] = dict(batch_tests=len(b["tests"]), by_fuel=dict(Counter(t["fuel"] for t in b["tests"])),
                                  held_series_files_present=sum(t["test_id"] in names for t in b["tests"]),
                                  scale_pinned=SCALE_MARKER.exists(),
                                  rescaled_validation_folder=(DROPLET / "flex2_series_v2/validation").is_dir())
    rep["validation_seals_of_listed_releases"] = {rid: seals[(rid, "validation")] for rid in rep["test_seals_of_listed_releases"]}
    print(json.dumps(rep, indent=1))
    return 0


# ------------------------------------------------------------------------------------------------
# results page (final mode only)
# ------------------------------------------------------------------------------------------------
def write_results_md():
    md = ["# Final run of 15 November (PROTOCOL 12.5)", "",
          "Produced by `tools/wsp/final/run_final.py` in final mode. Aggregates only; per-case rows are local "
          "(`runs/final/20261115/local/`). Nothing is revised after this page in this generation.", ""]
    for step in STEPS:
        p = OUT / f"{step}.json"
        if not p.exists():
            md += [f"## {step}", "", "Not run.", ""]
            continue
        d = json.loads(p.read_text(encoding="utf-8"))
        md += [f"## {step}", "", f"Release list sha256 `{d.get('release_list_sha256')}`.", ""]
        if step == "sealed":
            for name, fam in d["families"].items():
                md += [f"### {name}", ""]
                for inp, fc in fam["forecasts"].items():
                    r = fc[fam["release"]]
                    md.append(f"- {inp}: release forecasts {r['artifacts']} cases of {r['runs']} runs; refused {r['refused']} "
                              f"{r['refusal_counts']}; not at risk {r['not_at_risk']}.")
                md.append("")
                for label, c in fam["comparisons"].items():
                    for inp, b in c["per_input"].items():
                        pr = b["per_release"]
                        md.append(f"- against {label}, input {inp}: {b['cases']} cases, {b['runs']} runs, {b['sessions']} sessions; "
                                  + "; ".join(f"`{rid[:15]}` " + ", ".join(f"{k} {v:.4f}" for k, v in m.items() if v is not None)
                                              for rid, m in pr.items()))
                        for rid, pv in b["paired_vs_reference"].items():
                            md.append("  - paired, release minus comparator: " + "; ".join(
                                f"{m} {v['estimate']:+.4f}" + (f" [{v['ci'][0]:+.4f}, {v['ci'][1]:+.4f}]" if v["ci"] else "")
                                + f" ({v['n_clusters']} clusters)" for m, v in pv.items() if v))
                md.append("")
        else:
            md += ["```json", json.dumps({k: v for k, v in d.items() if k not in ("binaries",)}, indent=1, default=str)[:20000], "```", ""]
    RESULTS_MD.write_text("\n".join(md), encoding="utf-8", newline="\n")


# ------------------------------------------------------------------------------------------------
def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("cmd", choices=["preflight", "rehearse", "final"])
    ap.add_argument("step", nargs="?", choices=[*STEPS, "all"])
    ap.add_argument("--releases", help="the frozen release list (JSON)")
    ap.add_argument("--execute-final", action="store_true")
    ap.add_argument("--i-am-the-final-run", action="store_true")
    ap.add_argument("--workers", type=int, default=12)
    a = ap.parse_args()
    if a.cmd == "preflight":
        return preflight(a)
    if not a.step:
        sys.exit("name a step")
    if a.cmd == "final" and a.step == "all":
        sys.exit("final mode runs one named step per call")
    steps = STEPS if a.step == "all" else (a.step,)
    fn = {"sealed": step_sealed, "flame-design": step_flame_design, "cfi": step_cfi, "fabric": step_fabric, "flex2": step_flex2}
    summary = {}
    for s in steps:
        mode = Mode(a, s)
        r = fn[s](mode)
        summary[s] = {"result": rel(mode.result_path()), "reproduced": r.get("reproduced")} if not mode.final else {"result": rel(mode.result_path())}
        print(json.dumps({s: summary[s]}), flush=True)
    if a.cmd == "final":
        try:
            write_results_md()
            print(f"written {rel(RESULTS_MD)}")
        except Exception as e:  # noqa: BLE001  (the page is assembled again by hand; the step result is written)
            print(f"RESULTS_FINAL.md not assembled automatically: {type(e).__name__}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
