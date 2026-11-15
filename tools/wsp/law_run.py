#!/usr/bin/env python3
"""Run a release of the generic law forward (`<slug>.wsp`, crates/ignis-wsp/src/law.rs) on a case
file of named inputs, through `ignis wsp forward`. Fits nothing, changes no number, scores nothing.

    python tools/wsp/law_run.py --release sha256:... --cases runs/dev/data_all/work/transfer_<slug>/cases_train.jsonl \
        --out runs/dev/law/<slug>/train_forecasts.jsonl [--targets a,b] [--times 10,30,60] [--root artifacts]

A case is one JSON object per line, as the data engineers write them. The value of a feature (and of
an event target's declared stop condition) is read by its declared name from `case["inputs"]` when
the case has such a dict, else from the case itself:

* a number is the SI value (bools are not numbers);
* a dict is read as `value`, with `unit` checked against the law's declared unit (a different unit
  makes the input unusable: the forward refuses it, naming the feature; nothing is converted), and
  `known: false` meaning the input is not known before the test (treated as missing);
* anything else (text, a list, null) is handed to the forward without an SI value, which refuses it.

Per case: `case_id` (or `test`, `run_id`, else the line number), optional `cutoff_s` (default 0),
optional `times_s` (event targets; else `--times`). Output, one line per case:

    {"case_id", "release_id", "slug", "targets": {<name>: {"kind", "label", "admitted", "reasons",
     "quantiles": {"0.05": ...}, "mean", "probability", "times_s", "cdf": {<category>: [...]}}}}

Prefix files and artifacts are kept next to the output (`<out>.work/`). Never point it at holdout/.
"""
import argparse
import hashlib
import json
import math
import os
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor
from decimal import Decimal
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
LEVELS = [0.05, 0.25, 0.5, 0.75, 0.95]


# canonical JSON and content hash, as `ignis_core::hashing` (serde_json writes floats with ryu);
# the same functions as tools/wsp/ext/transfer_acme.py
def ryu(x):
    if x == 0.0:
        return "-0.0" if math.copysign(1.0, x) < 0 else "0.0"
    sign, digits, exp = Decimal(repr(abs(x))).as_tuple()
    ds = "".join(map(str, digits)).lstrip("0")
    k = exp + (len(ds) - len(ds.rstrip("0")))
    ds = ds.rstrip("0")
    n, kk = len(ds), len(ds) + k
    if 0 <= k and kk <= 16:
        body = ds + "0" * k + ".0"
    elif 0 < kk <= 16:
        body = ds[:kk] + "." + ds[kk:]
    elif -5 < kk <= 0:
        body = "0." + "0" * (-kk) + ds
    elif n == 1:
        body = f"{ds}e{kk - 1}"
    else:
        body = f"{ds[0]}.{ds[1:]}e{kk - 1}"
    return ("-" if x < 0 else "") + body


def canonical(v):
    if isinstance(v, dict):
        return "{" + ",".join(json.dumps(k, ensure_ascii=False) + ":" + canonical(v[k]) for k in sorted(v)) + "}"
    if isinstance(v, (list, tuple)):
        return "[" + ",".join(canonical(x) for x in v) + "]"
    if isinstance(v, bool):
        return "true" if v else "false"
    if v is None:
        return "null"
    if isinstance(v, float):
        return ryu(v) if math.isfinite(v) else "null"
    if isinstance(v, int):
        return str(v)
    return json.dumps(v, ensure_ascii=False)


def content_hash(obj, exclude=()):
    v = {k: x for k, x in obj.items() if k not in exclude}
    return "sha256:" + hashlib.sha256(canonical(v).encode("utf-8")).hexdigest()


def find_exe(path):
    exe = Path(path)
    if not exe.is_absolute():
        exe = REPO / exe
    if not exe.exists() and exe.with_suffix(".exe").exists():
        exe = exe.with_suffix(".exe")
    if not exe.exists():
        sys.exit(f"no ignis binary at {exe} (build: CARGO_TARGET_DIR=target-merged cargo build --release -p ignis-cli)")
    return exe


def release_record(root, release_id):
    hexpart = release_id.split(":", 1)[-1]
    path = root / "models" / hexpart / "release.json"
    if not path.exists():
        sys.exit(f"release {release_id} is not in the registry {root} ({path})")
    rec = json.loads(path.read_text(encoding="utf-8"))
    decl = (rec.get("applicability") or {}).get("declaration")
    if not decl:
        sys.exit(f"{release_id} ({rec.get('model_name')}) is not a release of the law forward")
    return rec, decl


def lookup(case, name):
    """(value, unit, known): the case's entry for a declared name."""
    src = case["inputs"] if isinstance(case.get("inputs"), dict) else case
    if name not in src:
        return None, None, False
    v = src[name]
    if isinstance(v, dict):
        known = v.get("known", True) is not False
        return v.get("value"), v.get("unit"), known
    return v, None, True


def measured(value, unit_case, unit_law, where):
    number = isinstance(value, (int, float)) and not isinstance(value, bool) and math.isfinite(float(value))
    flags, resolution = [], "resolved"
    if unit_case is not None and unit_case != unit_law:
        flags.append(f"unit_mismatch: the case gives {unit_case}, the law declares {unit_law}; not converted")
        resolution, number = "unknown", False
    raw = value if isinstance(value, (int, float, str)) and not isinstance(value, bool) else None
    if isinstance(raw, (int,)) and not isinstance(raw, bool):
        raw = float(raw)
    return dict(
        raw_value=raw, raw_unit=unit_case if unit_case is not None else unit_law,
        value_si=float(value) if number else None, unit_si=unit_law,
        uncertainty_si=None, uncertainty_reason="not reported by the case file",
        evidence_class="source_derived", locator=None, derivation=None,
        resolution=resolution, flags=flags + [f"case_file:{where}"])


def build_prefix(case, decl, family, slug, case_id, where):
    cond = {}
    for f in decl["features"]:
        value, unit, known = lookup(case, f["name"])
        if known and value is not None:
            cond[f["name"]] = measured(value, unit, f["unit"], where)
    for t in decl["targets"]:
        stop = (t.get("scheduled_stop") or {}).get("condition")
        if stop and stop not in cond:
            value, unit, known = lookup(case, stop)
            if known and value is not None:
                cond[stop] = measured(value, unit, "s", where)
    bundle = dict(
        family=family, run_id=f"law:{slug}:{case_id}", session_id=None,
        cutoff_s=float(case.get("cutoff_s", 0.0) or 0.0),
        time_origin=str(case.get("time_origin", "the time origin declared by the case file")),
        known_conditions=cond, known_controls=[], observations=[], contours=[],
        phase_established_at_s=None, support_evidence=[f"case file {where}"], transform_versions={})
    bundle["input_hash"] = content_hash(bundle, ("input_hash",))
    return bundle


def forward(exe, root, release_id, prefix_path, settings, out_path):
    cmd = [str(exe), "wsp", "forward", "--release", release_id, "--prefix", str(prefix_path),
           "--settings", json.dumps(settings, separators=(",", ":")), "--root", str(root), "--out", str(out_path)]
    r = subprocess.run(cmd, cwd=REPO, capture_output=True, text=True)
    if r.returncode != 0:
        raise RuntimeError(f"ignis wsp forward failed on {prefix_path.name}: {(r.stderr or r.stdout).strip()[-800:]}")
    return json.loads(out_path.read_text(encoding="utf-8"))


def summary(a, kind):
    law = a["diagnostics"].get("law", {})
    out = dict(kind=kind, label=law.get("label"), unit=law.get("unit"),
               shared_quantity=law.get("shared_quantity"),
               admitted=bool(a["support"]["admitted"]), reasons=a["support"]["reasons"])
    if not out["admitted"]:
        return out
    if kind == "positive":
        out["quantiles"] = {str(l): row[0] for l, row in zip(a["quantile_levels"], a["quantiles"])}
        out["mean"] = a["mean"][0] if a["mean"] else None
    elif kind == "binary":
        out["probability"] = a["event_cdf"][a["event_categories"].index("yes")][0]
    else:
        out["times_s"] = a["times_s"]
        out["cdf"] = {c: row for c, row in zip(a["event_categories"], a["event_cdf"])}
    return out


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("--release", required=True)
    ap.add_argument("--cases", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--targets", default="", help="comma-separated target names (default: every declared target)")
    ap.add_argument("--times", default="", help="comma-separated times (s) for event targets, when a case gives none")
    ap.add_argument("--levels", default="", help="comma-separated quantile levels (default 0.05,0.25,0.5,0.75,0.95)")
    ap.add_argument("--root", default=os.environ.get("IGNIS_ARTIFACTS", "artifacts"))
    ap.add_argument("--ignis", default="target-merged/release/ignis")
    ap.add_argument("--workers", type=int, default=8)
    args = ap.parse_args()
    for p in (args.cases, args.out):
        parts = Path(p).resolve().parts
        if "holdout" in parts or "holdout_ext" in parts:
            sys.exit("refusing a path under holdout/")
    exe = find_exe(args.ignis)
    root = Path(args.root)
    if not root.is_absolute():
        root = REPO / root
    rec, decl = release_record(root, args.release)
    slug, family = decl["slug"], rec["family"]
    targets = [t for t in decl["targets"] if not args.targets or t["name"] in args.targets.split(",")]
    unknown = set(x for x in args.targets.split(",") if x) - {t["name"] for t in targets}
    if unknown:
        sys.exit(f"targets {sorted(unknown)} are not declared by {slug}")
    default_times = [float(x) for x in args.times.split(",") if x]
    levels = [float(x) for x in args.levels.split(",") if x] or LEVELS
    out = Path(args.out)
    work = out.with_name(out.name + ".work")
    (work / "prefix").mkdir(parents=True, exist_ok=True)
    (work / "artifact").mkdir(parents=True, exist_ok=True)

    cases, jobs = [], []
    with open(args.cases, encoding="utf-8") as f:
        for n, line in enumerate(f, 1):
            if not line.strip():
                continue
            case = json.loads(line)
            inner = case.get("inputs") if isinstance(case.get("inputs"), dict) else {}
            cid = str(case.get("case_id") or case.get("test") or case.get("run_id")
                      or inner.get("case_id") or inner.get("test") or f"line{n}")
            safe = "".join(c if c.isalnum() or c in "-_." else "_" for c in cid)
            where = f"{Path(args.cases).name}:{n}"
            prefix = build_prefix(case, decl, family, slug, cid, where)
            pp = work / "prefix" / f"{n:06d}_{safe}.json"
            pp.write_text(json.dumps(prefix, indent=1, ensure_ascii=False) + "\n", encoding="utf-8", newline="\n")
            cases.append((cid, n))
            for t in targets:
                settings = {"target": t["name"]}
                if t["kind"] == "positive":
                    settings["quantile_levels"] = levels
                elif t["kind"] == "event":
                    settings["times_s"] = case.get("times_s") or default_times
                    if not settings["times_s"]:
                        sys.exit(f"event target {t['name']}: give --times or times_s per case")
                ap_ = work / "artifact" / f"{n:06d}_{safe}.{t['name']}.json"
                jobs.append(((n, t["name"], t["kind"]), pp, settings, ap_))

    def one(job):
        key, pp, settings, ap_ = job
        return key, forward(exe, root, args.release, pp, settings, ap_)

    with ThreadPoolExecutor(max_workers=max(1, args.workers)) as ex:
        results = dict(ex.map(one, jobs))
    out.parent.mkdir(parents=True, exist_ok=True)
    admitted = 0
    with out.open("w", encoding="utf-8", newline="\n") as f:
        for cid, n in cases:
            row = dict(case_id=cid, release_id=args.release, slug=slug, targets={})
            for t in targets:
                s = summary(results[(n, t["name"], t["kind"])], t["kind"])
                admitted += s["admitted"]
                row["targets"][t["name"]] = s
            f.write(json.dumps(row, ensure_ascii=False) + "\n")
    print(json.dumps(dict(release=args.release, slug=slug, cases=len(cases), forecasts=len(jobs),
                          admitted=admitted, out=str(out), work=str(work))))


if __name__ == "__main__":
    main()
