#!/usr/bin/env python3
"""Transcribe a stored burner-flame release into the edits file of the same law with a fuel table.

The forwards `crates/ignis-wsp/src/flame5.rs`, `flame6.rs` and `flame7.rs` (`acme_*.wsp9`,
`acme_*.wsp10`, `acme_*.wsp11`) are the laws of `flame.rs`, `flame3.rs` and `flame4.rs`
(`acme_*.wsp2`, `acme_*.wsp3`, `acme_*.wsp8`) reading a typed fuel table (`fuel.*`, design:
`docs/wsp/research/phys_spherical_fuels.md`, section 3) in place of the two fuel indicators. Their
specs are the parent specs plus the table; the growth-law and burner-heating specs also lose
`phys.mw_ethane`, `phys.mw_propane`, `phys.mw_butane`, which the table's `fuel.mw_g_mol` replaces
(those are constants of the parent specs and have no statement).

A release whose law numbers already exist in a stored release is not typed again: this script
copies, statement by statement, every stored statement of --source, and appends nothing else. The
fuel table is typed by the author in the header file, which must hold the `fuel.*` lines (all seven
tensors, every element: they are written by the author in a root). It chooses no value, computes none
and reads no data.

    source model           -> model of the edits file
    acme_*.wsp2  (flame.rs)    acme_*.wsp9  (flame5.rs)
    acme_*.wsp3  (flame3.rs)   acme_*.wsp10 (flame6.rs)
    acme_*.wsp8  (flame4.rs)   acme_*.wsp11 (flame7.rs)
    acme_*.wsp9/10/11          the same model (its statements without its fuel.* statements)

    root      python tools/wsp/fuel_member_edits.py --header H --out edits.wsp --source ID
    revision  ... the same, plus --parent ID (the acme_*.wsp9/10/11 release it revises)
              only the copied statements that differ from the parent are written (an element the
              parent set and this one leaves at zero is written as 0.0); the header is kept whole

`H` is a hand-written text: the comment header of the edits file and the `fuel.*` lines. After
assembling, `--verify RELEASE.json` (with --source) checks tensor by tensor (stored sha256) that the
release holds every tensor of the source unchanged and, besides them, exactly the seven fuel tensors.
"""
import argparse
import json
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from committee_edits import member_lines, statements, tensor_hashes  # noqa: E402

FUEL_TENSORS = ["fuel.admit", "fuel.mw_g_mol", "fuel.s", "fuel.se_coord", "fuel.ft_coord",
                "fuel.dosc", "fuel.tad_min_k"]
ALKANE_MW = ["phys.mw_ethane", "phys.mw_propane", "phys.mw_butane"]
# the model of the edits file, by the source's model generation
TARGET = {"wsp2": "wsp9", "wsp3": "wsp10", "wsp8": "wsp11", "wsp9": "wsp9", "wsp10": "wsp10", "wsp11": "wsp11"}


def tensor_of(statement_key: str) -> str:
    return statement_key.partition("[")[0]


def is_fuel(tensor: str) -> bool:
    return tensor.startswith("fuel.")


def model_of(root: Path, rid: str) -> str:
    path = root / "models" / rid.removeprefix("sha256:") / "release.json"
    return json.loads(path.read_text(encoding="utf-8"))["model_name"]


def target_of(model: str) -> str:
    m = re.fullmatch(r"(acme_normal|acme_inverse)\.(wsp\d+)", model)
    if not m or m.group(2) not in TARGET:
        sys.exit(f"{model}: the source must be an acme_*.wsp2, wsp3, wsp8, wsp9, wsp10 or wsp11 release")
    return f"{m.group(1)}.{TARGET[m.group(2)]}"


def header_fuel_tensors(text: str) -> set:
    found = set()
    for line in text.splitlines():
        line = line.split("#", 1)[0].strip()
        if line.startswith("fuel."):
            found.add(tensor_of(line.split("=", 1)[0].strip()))
    return found


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--ignis", default="target/release/ignis")
    ap.add_argument("--source", required=True, help="release id of the stored release whose law numbers are copied")
    ap.add_argument("--parent", help="release id of the acme_*.wsp9/10/11 release this one revises")
    ap.add_argument("--header")
    ap.add_argument("--out")
    ap.add_argument("--verify", help="release.json of the assembled release: check its tensors against the source")
    ap.add_argument("--artifacts", default="artifacts")
    args = ap.parse_args()
    for rid in (args.source, args.parent):
        if rid is not None and not rid.startswith("sha256:"):
            sys.exit(f"{rid!r}: expected sha256:...")
    root = Path(args.artifacts)
    exe = Path(args.ignis)
    if not exe.exists() and exe.with_suffix(".exe").exists():
        exe = exe.with_suffix(".exe")
    exe = str(exe.resolve())
    target = target_of(model_of(root, args.source))

    if args.verify:
        release = json.loads(Path(args.verify).read_text(encoding="utf-8"))
        if release["model_name"] != target:
            print(f"DIFFERS model {release['model_name']}, expected {target}")
            return 1
        source = {k: v for k, v in tensor_hashes(root, args.source).items()
                  if not is_fuel(k) and k not in ALKANE_MW}
        have = {t["name"]: (t["shape"], t["sha256"]) for t in release["tensors"]}
        bad = 0
        for name, entry in source.items():
            if have.get(name) != entry:
                bad += 1
                print(f"DIFFERS {name}")
        extra = sorted(n for n in have if n not in source)
        if extra != sorted(FUEL_TENSORS):
            bad += 1
            print(f"DIFFERS tensors besides the source's: {extra}")
        print(f"{len(source)} tensors of {args.source}")
        print(f"{len(extra)} fuel tensors typed in the header")
        print("identical to the source" if bad == 0 else f"{bad} differences")
        return 1 if bad else 0

    if not (args.header and args.out):
        sys.exit("--header and --out are required")
    header = Path(args.header).read_text(encoding="utf-8").rstrip("\n") + "\n"
    if args.parent is None and header_fuel_tensors(header) != set(FUEL_TENSORS):
        sys.exit(f"{args.header}: a root header writes the seven fuel tensors {FUEL_TENSORS}; "
                 f"found {sorted(header_fuel_tensors(header))}")
    if args.parent is not None and target_of(model_of(root, args.parent)) != target:
        sys.exit(f"{args.parent} is not a release of {target}")
    cache = {}
    this = {k: v for k, v in statements(exe, args.source, cache).items() if not is_fuel(tensor_of(k))}
    if not this:
        sys.exit(f"{args.source} has no statement")
    base = None
    if args.parent is not None:
        base = {k: v for k, v in statements(exe, args.parent, cache).items() if not is_fuel(tensor_of(k))}
    lines = member_lines("", this, base)
    text = header
    text += f"\n# ---- laws of {target}: the stored values of {args.source}"
    if base is not None:
        text += f"\n#      ({len(lines)} statements that differ from the parent, {args.parent})"
    else:
        text += f"\n#      ({len(lines)} statements)"
    text += "\n" + "\n".join(lines) + ("\n" if lines else "")
    Path(args.out).write_text(text, encoding="utf-8", newline="\n")
    print(f"{args.out}: {target}, {len(lines)} copied statements")
    return 0


if __name__ == "__main__":
    sys.exit(main())
