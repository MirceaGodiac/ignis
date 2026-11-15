"""Shared helpers for Phase-2 train-data analysis.

Safety: every accessor of outcome-bearing data goes through `train_ids(...)`; runs of the test
partition are never returned, and validation is returned only when `allow_validation=True`.
"""
import json
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
SPLITS = {
    "flex": "data/splits/flex-660b7d9ede629b30.json",
    "flex_heptane": "data/splits/flex-c43bbe3580ffce2a.json",
    "acme_normal": "data/splits/acme_normal-c77bc5baee98d3e3.json",
    "acme_inverse": "data/splits/acme_inverse-747032c921c8c8f2.json",
    "bass": "data/splits/bass-4083156d5b18dab4.json",
}
RECORDS = {"flex": "flex", "flex_heptane": "flex", "acme_normal": "acme_normal", "acme_inverse": "acme_inverse", "bass": "bass"}


def split(name):
    d = json.loads((ROOT / SPLITS[name]).read_text(encoding="utf-8"))
    s = d["split"]
    part = {}
    for p in ("train", "validation"):
        for g in s[p]:
            part[g] = p
    run_part = {}
    for r, g in s["run_to_group"].items():
        run_part[r] = part.get(g, "test")
    return s, run_part


def partition_ids(name, partition="train"):
    assert partition in ("train", "validation"), "the test partition is not readable here"
    s, rp = split(name)
    return sorted(r for r, p in rp.items() if p == partition)


def groups(name):
    s, rp = split(name)
    return {r: g for r, g in s["run_to_group"].items() if rp[r] != "test"}


def experiments(name, ids):
    ids = set(ids)
    out = {}
    with (ROOT / "data/work/records" / RECORDS[name] / "experiments.jsonl").open(encoding="utf-8") as f:
        for line in f:
            r = json.loads(line)
            if r["id"] in ids:
                out[r["id"]] = r
    return out


def jsonl_for(name, file, ids, key="run_id"):
    """Rows of a records file restricted to the given (train/validation) run ids."""
    ids = set(ids)
    out = defaultdict(list)
    p = ROOT / "data/work/records" / RECORDS[name] / file
    with p.open(encoding="utf-8") as f:
        for line in f:
            # cheap prefilter: skip lines that mention none of the ids
            r = json.loads(line)
            k = r.get(key)
            if k in ids:
                out[k].append(r)
    return out


def cond(rec, key, default=None):
    c = rec["conditions"].get(key)
    if c is None:
        return default
    v = c.get("value_si")
    return v if v is not None else c.get("raw_value", default)
