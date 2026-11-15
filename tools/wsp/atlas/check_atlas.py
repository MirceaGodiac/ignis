#!/usr/bin/env python3
"""Checks of the test-level atlas written by normalize_tests.py.

  1. Determinism: two fresh runs in separate processes give byte-identical tests.jsonl,
     tests_outcomes.jsonl, tests_summary.json and tests.schema.json, and the files on disk equal them.
  2. Every row of tests.jsonl validates against the JSON Schema of atlas_schema.py, and against the
     consistency rules a JSON Schema cannot state (status against value, source and reason).
  3. Identity: atlas_id is unique; every row has a group id; a test point is in the group of its
     parent, and test points of one parent share a group.
  4. Outcome separation: no value object of tests.jsonl takes its value from a table cell that is
     held in tests_outcomes.jsonl; rows of the four project families list no outcome field and have
     no line in tests_outcomes.jsonl; every outcome line belongs to a row and to a listed field.
  5. docs/wsp/atlas/SCHEMA.md names every token of every closed list and every rule id used.
  6. tests_summary.json repeats the row counts of tests.jsonl.

Usage (repository root):  python tools/wsp/atlas/check_atlas.py [--dir runs/dev/data_all/work/atlas]
Exit code 0 when every check passes.
"""
import argparse
import json
import subprocess
import sys
import tempfile
from collections import Counter
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
sys.path.insert(0, str(HERE))
import atlas_schema as S  # noqa: E402

FILES = ["tests.jsonl", "tests_outcomes.jsonl", "tests_summary.json", "tests.schema.json"]
PROTECTED = {"PSI-25", "PSI-26", "PSI-69", "PSI-159"}


def load_jsonl(path):
    with open(path, encoding="utf-8") as f:
        return [json.loads(line) for line in f if line.strip()]


def run_once(out_dir):
    cmd = [sys.executable, str(HERE / "normalize_tests.py"), "--out", str(out_dir)]
    r = subprocess.run(cmd, cwd=str(ROOT), capture_output=True, text=True)
    if r.returncode != 0:
        raise SystemExit("normalize_tests.py failed:\n" + r.stdout + r.stderr)


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--dir", default=str(ROOT / "runs/dev/data_all/work/atlas"))
    ap.add_argument("--schema-doc", default=str(ROOT / "docs/wsp/atlas/SCHEMA.md"))
    args = ap.parse_args()
    disk = Path(args.dir)
    problems = []
    report = []

    # 1. determinism
    with tempfile.TemporaryDirectory() as a, tempfile.TemporaryDirectory() as b:
        run_once(a)
        run_once(b)
        for name in FILES:
            ba, bb = (Path(a) / name).read_bytes(), (Path(b) / name).read_bytes()
            if ba != bb:
                problems.append(f"determinism: two runs differ in {name}")
            if not (disk / name).exists():
                problems.append(f"determinism: {disk / name} is missing; run normalize_tests.py")
            elif (disk / name).read_bytes() != ba:
                problems.append(f"determinism: {name} on disk differs from a fresh run")
    report.append(f"determinism: {len(FILES)} files compared between two fresh runs and the output on disk")

    rows = load_jsonl(disk / "tests.jsonl")
    outcomes = load_jsonl(disk / "tests_outcomes.jsonl")
    summary = json.loads((disk / "tests_summary.json").read_text(encoding="utf-8"))

    # 2. schema
    try:
        import jsonschema
    except ImportError:
        raise SystemExit("the jsonschema package is needed for the row validation (pip install jsonschema)")
    schema = S.row_schema()
    if json.loads((disk / "tests.schema.json").read_text(encoding="utf-8")) != json.loads(json.dumps(schema)):
        problems.append("tests.schema.json on disk is not the schema of atlas_schema.py")
    validator = jsonschema.Draft202012Validator(schema)
    bad = 0
    rules_used = set()
    for r in rows:
        errs = sorted(validator.iter_errors(r), key=lambda e: list(e.absolute_path))
        for e in errs[:3]:
            problems.append(f"schema: {r.get('atlas_id')}: {'/'.join(map(str, e.absolute_path))}: {e.message[:160]}")
        bad += bool(errs)
        for path, v in S.walk_values(r):
            for p in S.value_problems(path, v):
                problems.append(f"value: {r['atlas_id']}: {p}")
            if v["rule"]:
                rules_used.add(v["rule"])
        rules_used.add(r["group_rule"])
        for lab in r["labels"]["studied"] + r["labels"]["fire_safety_need"]:
            rules_used.add(lab["rule"])
        if r["map"]["crew_atmospheres"] or r["map"]["crew_atmospheres_o2_only"]:
            rules_used.add("CREW")
        if r["varied"]:
            rules_used.add("VAR")
    report.append(f"schema: {len(rows)} rows validated, {bad} with schema errors")
    ov = jsonschema.Draft202012Validator(S.OUTCOME_ROW_SCHEMA)
    for o in outcomes:
        for e in list(ov.iter_errors(o))[:3]:
            problems.append(f"outcome schema: {o.get('atlas_id')}: {e.message[:160]}")

    # 3. identity
    ids = Counter(r["atlas_id"] for r in rows)
    for k, n in ids.items():
        if n > 1:
            problems.append(f"identity: atlas_id {k} appears {n} times")
    for r in rows:
        if r["atlas_id"] != f"{r['investigation_id']}:{r['test_id']}":
            problems.append(f"identity: {r['atlas_id']} is not investigation_id:test_id")
        if not r["group_id"].startswith(("session:", r["investigation_id"] + ":")):
            problems.append(f"identity: group id of {r['atlas_id']} does not name its investigation")
    parent_groups = {}
    for r in rows:
        if r["parent_test_id"] is None:
            continue
        key = (r["investigation_id"], r["parent_test_id"])
        parent_groups.setdefault(key, set()).add(r["group_id"])
        parent = f"{r['investigation_id']}:{r['parent_test_id']}"
        if parent in ids:
            parent_groups[key].add(next(x["group_id"] for x in rows if x["atlas_id"] == parent))
    for key, gs in parent_groups.items():
        if len(gs) > 1:
            problems.append(f"identity: the test points of {key[0]}:{key[1]} are spread over groups {sorted(gs)}")
    report.append(f"identity: {len(ids)} distinct atlas ids, {len({r['group_id'] for r in rows})} groups, "
                  f"{len(parent_groups)} parents with test points")

    # 4. outcome separation
    by_id = {r["atlas_id"]: r for r in rows}
    held = set()
    for o in outcomes:
        r = by_id.get(o["atlas_id"])
        if r is None:
            problems.append(f"outcomes: {o['atlas_id']} has no row in tests.jsonl")
            continue
        if r["investigation_id"] in PROTECTED:
            problems.append(f"outcomes: {o['atlas_id']} belongs to a project family")
        listed = {f["name"] for f in r["outcome_fields_available"]}
        for item in o["outcomes"]:
            if item["name"] not in listed:
                problems.append(f"outcomes: {o['atlas_id']} field {item['name']} is not listed on its row")
            s = item["source"]
            held.add((s["file"], s["sheet"], s["column"], s["row"]))
    leaks = 0
    for r in rows:
        if r["investigation_id"] in PROTECTED:
            if r["outcome_status"] != "not_read_protected" or r["outcome_fields_available"]:
                problems.append(f"outcomes: project row {r['atlas_id']} lists outcome fields")
        elif (r["outcome_status"] == "listed") != bool(r["outcome_fields_available"]):
            problems.append(f"outcomes: {r['atlas_id']} outcome_status disagrees with its field list")
        for path, v in S.walk_values(r):
            s = v["source"]
            if s and v["status"] in ("resolved", "multiple") and (s["file"], s["sheet"], s["column"], s["row"]) in held:
                leaks += 1
                problems.append(f"outcomes: {r['atlas_id']} {path} takes its value from a cell held as an outcome")
            for a in v["alternatives"]:
                s2 = a["source"]
                if s2 and (s2["file"], s2["sheet"], s2["column"], s2["row"]) in held:
                    leaks += 1
                    problems.append(f"outcomes: {r['atlas_id']} {path} has an alternative taken from a cell held as an outcome")
    report.append(f"outcome separation: {len(outcomes)} tests with held values, {len(held)} held cells, {leaks} leaks")

    # 5. SCHEMA.md names every token and rule
    doc = Path(args.schema_doc)
    if not doc.exists():
        problems.append(f"schema doc: {doc} is missing")
    else:
        text = doc.read_text(encoding="utf-8")
        missing = [f"{name}:{tok}" for name, toks in S.ALL_LISTS.items() for tok in toks if f"`{tok}`" not in text]
        missing += [f"rule:{x}" for x in sorted(rules_used) if f"`{x}`" not in text]
        missing += [f"field:{k}" for k in S.ROW_KEYS if f"`{k}`" not in text]
        for m in missing:
            problems.append(f"schema doc: SCHEMA.md does not name {m}")
        n = sum(len(t) for t in S.ALL_LISTS.values())
        report.append(f"schema doc: {n} closed-list tokens, {len(rules_used)} rule ids and {len(S.ROW_KEYS)} row fields looked up in SCHEMA.md")

    # 6. summary
    if summary["rows"] != len(rows):
        problems.append("summary: row count differs from tests.jsonl")
    per = Counter(r["investigation_id"] for r in rows)
    for k, v in summary["by_investigation"].items():
        if v["rows"] != per.get(k, 0):
            problems.append(f"summary: {k} has {per.get(k, 0)} rows in tests.jsonl and {v['rows']} in the summary")
    if summary["outcome_values_file"]["tests_with_outcome_values"] != len(outcomes):
        problems.append("summary: outcome line count differs from tests_outcomes.jsonl")
    report.append(f"summary: counts of {len(summary['by_investigation'])} investigations compared")

    for line in report:
        section = line.split(":")[0] + ":"
        print(("FAIL " if any(p.startswith(section) for p in problems) else "ok   ") + line)
    for p in problems[:60]:
        print("  - " + p)
    if len(problems) > 60:
        print(f"  ... and {len(problems) - 60} more")
    print("PASS" if not problems else f"FAIL: {len(problems)} problems")
    sys.exit(0 if not problems else 1)


if __name__ == "__main__":
    main()
