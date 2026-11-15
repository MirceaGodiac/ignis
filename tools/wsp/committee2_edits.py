#!/usr/bin/env python3
"""Transcribe member releases into the edits file of a model of the second committee (`acme_*.wsp7`).

The forward (`crates/ignis-wsp/src/committee2.rs`) holds one complete set of numbers per slot:
`a0.`, `a1.` are `flame.rs` releases (`acme_*.wsp2`), `b0.` ... `b3.` are `flame3.rs` releases
(`acme_*.wsp3`), `c0.` ... `c3.` are `flame4.rs` releases (`acme_*.wsp8`). Those numbers are not
written a second time: they are the stored values of existing literal releases, copied statement by
statement with the slot prefix added. This script makes that copy and nothing else. It chooses no
value, computes none and reads no data. The numbers of a committee that are new (its weight vectors,
its era rule and its offsets of the planned stop) are typed by the author in the header file.

It is the sibling of `committee_edits.py` (six slots, `acme_*.wsp5`), which it leaves as it is and
whose copying functions it uses.

A slot is filled from

    <slot>=sha256:...          a stored member release of the slot's own forward, or
    <slot>=sha256:...@<slot'>  slot <slot'> of a stored committee release (`acme_*.wsp5` or
                               `acme_*.wsp7`): the statements `<slot'>.*` of that release

    root      python tools/wsp/committee2_edits.py --header H --out edits.wsp \\
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
# slots a committee release of either kind may be read from
SOURCE_SLOTS = SLOTS


def source(value: str, what: str):
    """`sha256:...` or `sha256:...@slot` -> (release id, slot of that release or None)."""
    rid, _, inner = value.partition("@")
    if not rid.startswith("sha256:") or (inner and inner not in SOURCE_SLOTS):
        sys.exit(f"--{what} {value!r}: expected sha256:... or sha256:...@<slot> with slot in {SOURCE_SLOTS}")
    return rid, inner or None


def pairs(values, what):
    out = {}
    for v in values or []:
        slot, _, src = v.partition("=")
        if slot not in SLOTS:
            sys.exit(f"--{what} {v!r}: expected <slot>=<source> with slot in {SLOTS}")
        out[slot] = source(src, what)
    return out


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
    exe = Path(args.ignis)
    if not exe.exists() and exe.with_suffix(".exe").exists():
        exe = exe.with_suffix(".exe")
    exe = str(exe.resolve())
    slots, parents = pairs(args.slot, "slot"), pairs(args.parent, "parent")
    if set(slots) != set(SLOTS):
        sys.exit(f"give all ten slots: {SLOTS}")

    if args.verify:
        committee = json.loads(Path(args.verify).read_text(encoding="utf-8"))
        have = {t["name"]: (t["shape"], t["sha256"]) for t in committee["tensors"]}
        bad = 0
        for slot in SLOTS:
            want = slot_hashes(Path(args.artifacts), slots[slot])
            prefix = slot + "."
            for tensor, entry in want.items():
                if have.get(prefix + tensor) != entry:
                    bad += 1
                    print(f"DIFFERS {prefix}{tensor}")
            extra = [n for n in have if n.startswith(prefix) and n[len(prefix):] not in want]
            bad += len(extra)
            print(f"{prefix}* : {len(want)} tensors of {name(slots[slot])}")
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
