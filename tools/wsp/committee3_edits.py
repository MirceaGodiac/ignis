#!/usr/bin/env python3
"""Transcribe member releases into the edits file of a model of the third committee (`acme_*.wsp12`).

The forward (`crates/ignis-wsp/src/committee3.rs`) is the committee of `committee2.rs` with members
that read a typed fuel table: `a0.`, `a1.` are `flame5.rs` releases (`acme_*.wsp9`), `b0.` ... `b3.`
are `flame6.rs` releases (`acme_*.wsp10`), `c0.` ... `c3.` are `flame7.rs` releases
(`acme_*.wsp11`). Each slot holds one complete set of numbers, its fuel table included. Those numbers
are not written a second time: they are the stored values of existing literal releases, copied
statement by statement with the slot prefix added. This script makes that copy and nothing else. It
chooses no value, computes none and reads no data. The numbers of a committee that are new (its
weight vectors, its era rule and its offsets of the planned stop) are typed by the author in the
header file.

It is the sibling of `committee2_edits.py` (`acme_*.wsp7`), which it leaves as it is. A member of
another generation is first carried to its fuel-table model with `tools/wsp/fuel_member_edits.py`.

A slot is filled from

    <slot>=sha256:...          a stored member release of the slot's own model (a: acme_*.wsp9,
                               b: acme_*.wsp10, c: acme_*.wsp11), or
    <slot>=sha256:...@<slot'>  slot <slot'> of a stored acme_*.wsp12 release (the same kind of slot)

    root      python tools/wsp/committee3_edits.py --header H --out edits.wsp \\
                  --slot a0=SRC --slot a1=SRC --slot b0=SRC ... --slot c3=SRC      (all ten)
    revision  ... the same ten --slot, plus --parent a0=SRC ... for all ten
              only the statements that differ from the parent's slot are written; an element the
              parent's slot set and this one leaves at zero is written as 0.0

`H` is a hand-written text: the comment header of the edits file and the lines of the committee's own
tensors. After assembling, `--verify RELEASE.json` (with the ten --slot) checks tensor by tensor
(stored sha256) that every slot of the committee release is the tensors of the source named for it.
"""
import argparse
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from committee_edits import member_lines, statements, tensor_hashes  # noqa: E402

SLOTS = ["a0", "a1", "b0", "b1", "b2", "b3", "c0", "c1", "c2", "c3"]
# the model generation of a member of each kind of slot
KIND = {"a": "wsp9", "b": "wsp10", "c": "wsp11"}
COMMITTEE = "wsp12"


def source(value: str, what: str):
    """`sha256:...` or `sha256:...@slot` -> (release id, slot of that release or None)."""
    rid, _, inner = value.partition("@")
    if not rid.startswith("sha256:") or (inner and inner not in SLOTS):
        sys.exit(f"--{what} {value!r}: expected sha256:... or sha256:...@<slot> with slot in {SLOTS}")
    return rid, inner or None


def pairs(values, what):
    out = {}
    for v in values or []:
        slot, _, src = v.partition("=")
        if slot not in SLOTS:
            sys.exit(f"--{what} {v!r}: expected <slot>=<source> with slot in {SLOTS}")
        out[slot] = source(src, what)
    return out


def model_of(root: Path, rid: str) -> str:
    path = root / "models" / rid.removeprefix("sha256:") / "release.json"
    return json.loads(path.read_text(encoding="utf-8"))["model_name"]


def check_kind(root: Path, slot: str, src, family: str | None) -> str:
    """The family (`acme_normal` or `acme_inverse`) of a source that fits the slot."""
    rid, inner = src
    model = model_of(root, rid)
    fam, _, gen = model.partition(".")
    if inner is None:
        want = KIND[slot[0]]
        if gen != want:
            sys.exit(f"slot {slot}: {rid} is a {model} release; this slot takes an {fam}.{want} release")
    else:
        if gen != COMMITTEE or inner[0] != slot[0]:
            sys.exit(f"slot {slot}: {rid}@{inner} is slot {inner} of a {model} release; this slot "
                     f"takes slot {slot[0]}* of an {fam}.{COMMITTEE} release")
    if family is not None and fam != family:
        sys.exit(f"slot {slot}: {rid} is a {model} release, the other slots are {family}")
    return fam


def slot_statements(exe: str, src, cache: dict) -> dict:
    """The statements of a source, with the names a member release of its own gives them."""
    rid, inner = src
    found = statements(exe, rid, cache)
    if inner is None:
        return found
    prefix = inner + "."
    part = {k[len(prefix):]: v for k, v in found.items() if k.startswith(prefix)}
    if not part:
        sys.exit(f"{rid} has no statement of slot {inner}")
    return part


def slot_hashes(root: Path, src) -> dict:
    rid, inner = src
    found = tensor_hashes(root, rid)
    if inner is None:
        return found
    prefix = inner + "."
    part = {k[len(prefix):]: v for k, v in found.items() if k.startswith(prefix)}
    if not part:
        sys.exit(f"{rid} has no tensor of slot {inner}")
    return part


def name(src) -> str:
    rid, inner = src
    return rid if inner is None else f"slot {inner} of {rid}"


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--ignis", default="target/release/ignis")
    ap.add_argument("--slot", action="append", help="<slot>=<source>; all ten are required")
    ap.add_argument("--parent", action="append", help="<slot>=<source> of the parent committee's slot")
    ap.add_argument("--header")
    ap.add_argument("--out")
    ap.add_argument("--verify", help="release.json of the assembled committee: check its tensors against the sources")
    ap.add_argument("--artifacts", default="artifacts")
    args = ap.parse_args()
    root = Path(args.artifacts)
    exe = Path(args.ignis)
    if not exe.exists() and exe.with_suffix(".exe").exists():
        exe = exe.with_suffix(".exe")
    exe = str(exe.resolve())
    slots, parents = pairs(args.slot, "slot"), pairs(args.parent, "parent")
    if set(slots) != set(SLOTS):
        sys.exit(f"give all ten slots: {SLOTS}")
    family = None
    for slot in SLOTS:
        family = check_kind(root, slot, slots[slot], family)
    for slot, src in parents.items():
        check_kind(root, slot, src, family)

    if args.verify:
        committee = json.loads(Path(args.verify).read_text(encoding="utf-8"))
        if committee["model_name"] != f"{family}.{COMMITTEE}":
            print(f"DIFFERS model {committee['model_name']}, expected {family}.{COMMITTEE}")
            return 1
        have = {t["name"]: (t["shape"], t["sha256"]) for t in committee["tensors"]}
        bad = 0
        for slot in SLOTS:
            want = slot_hashes(root, slots[slot])
            prefix = slot + "."
            for tensor, entry in want.items():
                if have.get(prefix + tensor) != entry:
                    bad += 1
                    print(f"DIFFERS {prefix}{tensor}")
            extra = [n for n in have if n.startswith(prefix) and n[len(prefix):] not in want]
            bad += len(extra)
            fuel = sum(1 for t in want if t.startswith("fuel."))
            print(f"{prefix}* : {len(want)} tensors ({fuel} of the fuel table) of {name(slots[slot])}")
        print("identical to the sources" if bad == 0 else f"{bad} differences")
        return 1 if bad else 0

    if not (args.header and args.out):
        sys.exit("--header and --out are required")
    if parents and set(parents) != set(SLOTS):
        sys.exit("a revision names the parent of all ten slots")
    cache = {}
    text = Path(args.header).read_text(encoding="utf-8").rstrip("\n") + "\n"
    counts = []
    for slot in SLOTS:
        this = slot_statements(exe, slots[slot], cache)
        if not any(k.startswith("fuel.") for k in this):
            sys.exit(f"slot {slot}: {name(slots[slot])} has no statement of a fuel table")
        parent = slot_statements(exe, parents[slot], cache) if parents else None
        lines = member_lines(slot + ".", this, parent)
        counts.append(f"{slot} {len(lines)}")
        text += f"\n# ---- slot {slot}: the stored values of {name(slots[slot])}"
        if parent is not None:
            text += f"\n#      ({len(lines)} statements that differ from the parent's slot, {name(parents[slot])})"
        else:
            text += f"\n#      ({len(lines)} statements)"
        text += "\n" + "\n".join(lines) + ("\n" if lines else "")
    Path(args.out).write_text(text, encoding="utf-8", newline="\n")
    print(f"{args.out}: " + ", ".join(counts))
    return 0


if __name__ == "__main__":
    sys.exit(main())
