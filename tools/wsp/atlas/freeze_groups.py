"""Freezes the evaluation groups of the investigations that are outside the project's partitions.

The owner's rule: evaluation groups are frozen before anyone who types a number views outcome data.
This script reads the test catalogue (conditions and group ids only; outcome values are in another
file, which it does not open) and applies the project's own split rule, the one of
`ignis data split freeze` (crates/ignis-data/src/cohort.rs), with the project's seed:

    60/20/20 of groups (round half to even), groups shuffled by numpy default_rng(seed)

An investigation whose catalogue has fewer than three groups cannot be split three ways. All of it
is then HELD: nobody who types a number reads an outcome of it per test, and a model is scored on it
only as an aggregate, in a look registered beforehand. An investigation whose table is only a test
matrix is held until a per-test list exists.

A frozen file is never rewritten: the script refuses to overwrite one.

    python tools/wsp/atlas/freeze_groups.py            # writes docs/wsp/atlas/splits/
    python tools/wsp/atlas/freeze_groups.py --check    # recomputes and compares, writes nothing
"""
import argparse
import hashlib
import io
import json
import sys
from collections import Counter, defaultdict
from pathlib import Path

import numpy as np

REPO = Path(__file__).resolve().parents[3]
TESTS = REPO / "runs/dev/data_all/work/atlas/tests.jsonl"
OUT = REPO / "docs/wsp/atlas/splits"
SEED = 20261107  # the seed of every split in data/splits/
RULE = "60/20/20 of groups (round half to even), groups shuffled by numpy default_rng(seed)"
FROZEN_ON = "2026-11-14"
# Investigations that already have a project split (data/splits/) or are declared external: not frozen here.
SKIP = {"PSI-69": "project split flex-660b7d9ede629b30 / flex-c43bbe3580ffce2a",
        "PSI-159": "project splits acme_normal-c77bc5baee98d3e3 / acme_inverse-747032c921c8c8f2",
        "PSI-25": "project split bass-4083156d5b18dab4",
        "PSI-26": "declared outside every partition before this script existed; no row in the catalogue"}


def split_groups(gids, seed=SEED):
    """The rule of crates/ignis-data/src/cohort.rs::freeze, on a sorted list of distinct group ids."""
    gids = sorted(set(gids))
    n = len(gids)
    assert n >= 3
    order = [gids[int(i)] for i in np.random.default_rng(seed).permutation(n)]
    n_tr = max(1, round(0.6 * n))  # Python rounds half to even, as the rule says
    n_va = max(1, round(0.2 * n))
    if n_tr + n_va >= n:
        n_tr = n - n_va - 1
    return sorted(order[:n_tr]), sorted(order[n_tr:n_tr + n_va]), sorted(order[n_tr + n_va:])


def stratum_of(row):
    """Flame Design is split as two cohorts, normal and inverse flames, as the project splits CFI-G."""
    if row["investigation_id"] == "PSI-10":
        # the burner feeds fuel in a normal flame and oxidiser in an inverse one; the catalogue marks the
        # stream that does not flow as not applicable
        flow = (row.get("injection") or {}).get("oxidizer_mass_flow") or {}
        return "normal_flames" if flow.get("status") == "not_applicable" else "inverse_flames"
    return "all"


def build():
    rows = [json.loads(line) for line in io.open(TESTS, encoding="utf-8")]
    by = defaultdict(lambda: defaultdict(list))
    meta = {}
    for r in rows:
        inv = r["investigation_id"]
        if inv in SKIP:
            continue
        by[inv][stratum_of(r)].append(r)
        meta[inv] = r["acronym"]
    out = {}
    for inv in sorted(by, key=lambda s: int(s.split("-")[1])):
        strata = {}
        levels = Counter(r["level"] for s in by[inv].values() for r in s)
        for name, srows in sorted(by[inv].items()):
            g2rows = Counter(r["group_id"] for r in srows)
            gids = sorted(g2rows)
            # a day must belong to one stratum only, or the stratified split would cut it
            for other, orows in by[inv].items():
                if other != name:
                    assert not (set(gids) & {r["group_id"] for r in orows}), (inv, name, other)
            if set(levels) == {"matrix_row"}:
                status, parts = "held_until_test_list", {"held": gids}
            elif len(gids) < 3:
                status, parts = "held_too_few_groups", {"held": gids}
            else:
                tr, va, te = split_groups(gids)
                status, parts = "split", {"train": tr, "validation": va, "test": te}
            strata[name] = {
                "status": status,
                "groups": parts,
                "rows_in_catalogue": {p: sum(g2rows[g] for g in gs) for p, gs in parts.items()},
            }
        out[inv] = {
            "schema": "ignis.atlas.split/1",
            "investigation_id": inv,
            "acronym": meta[inv],
            "rule": RULE,
            "seed": SEED,
            "group_key": "group_id of the test catalogue (docs/wsp/atlas/SCHEMA.md): the test day or session",
            "catalogue_level": dict(levels),
            "strata": strata,
            "frozen_on": FROZEN_ON,
            "use": {
                "train": "outcomes may be read by a model's author",
                "validation": "read only as an aggregate table from a look registered in docs/wsp/PROTOCOL.md beforehand",
                "test": "never read by an author or by an agent working for one; scored once by the evaluation lane",
                "held": "as validation: no outcome is read per test; scored only as an aggregate in a registered look",
            },
            "note": "A test that is not in the catalogue yet belongs to the partition of its group id. A group id that is "
                    "not listed here is held until this file's rule has been applied to it in a new file; this file is not rewritten.",
        }
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--check", action="store_true")
    a = ap.parse_args()
    out = build()
    digest = hashlib.sha256(io.open(TESTS, "rb").read()).hexdigest()
    lines = ["# Evaluation groups of the investigations outside the project's partitions", "",
             "Frozen on 14 November 2026 by `tools/wsp/atlas/freeze_groups.py`, from the group ids of the test catalogue",
             "(conditions only; the file of outcome values was not opened), with the project's split rule and seed",
             f"(`{RULE}`, seed {SEED}). One JSON file per investigation lists every group and its partition.", "",
             "| Investigation | Stratum | Status | Groups: train / validation / test | Rows of the catalogue: train / validation / test | Held |",
             "|---|---|---|---|---|---|"]
    for inv, d in out.items():
        text = json.dumps(d, indent=1, ensure_ascii=False) + "\n"
        path = OUT / f"{inv}.json"
        if a.check:
            assert path.exists() and io.open(path, encoding="utf-8").read() == text, f"{inv}: differs from the frozen file"
        else:
            if path.exists():
                if io.open(path, encoding="utf-8").read() != text:
                    sys.exit(f"{path} is frozen and differs from what the rule gives now: not overwritten")
            else:
                OUT.mkdir(parents=True, exist_ok=True)
                io.open(path, "w", encoding="utf-8", newline="\n").write(text)
        for name, s in d["strata"].items():
            g, n = s["groups"], s["rows_in_catalogue"]
            if s["status"] == "split":
                lines.append(f"| {inv} {d['acronym']} | {name} | split | {len(g['train'])} / {len(g['validation'])} / {len(g['test'])} | "
                             f"{n['train']} / {n['validation']} / {n['test']} | |")
            else:
                why = "fewer than three groups" if s["status"] == "held_too_few_groups" else "the table is a test matrix; no per-test list yet"
                lines.append(f"| {inv} {d['acronym']} | {name} | held: {why} | | | {len(g['held'])} groups, {n['held']} rows |")
    lines += ["", "Not frozen here: " + "; ".join(f"{k} ({v})" for k, v in SKIP.items()) + ".", "",
              f"Catalogue used: `runs/dev/data_all/work/atlas/tests.jsonl`, sha256 `{digest}` (local, not in git; rebuilt by",
              "`tools/wsp/atlas/normalize_tests.py`).", ""]
    if not a.check:
        io.open(OUT / "README.md", "w", encoding="utf-8", newline="\n").write("\n".join(lines))
    print("\n".join(lines[6:-4]))
    print("check passed" if a.check else f"written {len(out)} files under {OUT.relative_to(REPO)}")


if __name__ == "__main__":
    main()
