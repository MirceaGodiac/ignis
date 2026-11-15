#!/usr/bin/env python3
"""Transcribe member releases into the edits file of a committee model (`acme_*.wsp5`).

The committee forward (`crates/ignis-wsp/src/committee.rs`) holds one complete set of numbers per
slot: `a0.`, `a1.` are `flame.rs` releases, `b0.` ... `b3.` are `flame3.rs` releases. Those numbers
are not written a second time: they are the stored values of existing literal releases, copied
statement by statement with the slot prefix added. This script makes that copy and nothing else. It
chooses no value, computes none and reads no data. The numbers of a committee that are new are its
six weights, which the author types in the header file.

    root      python tools/wsp/committee_edits.py --header H --out edits.wsp \\
                  --slot a0=ID --slot a1=ID --slot b0=ID --slot b1=ID --slot b2=ID --slot b3=ID
    revision  ... the same six --slot, plus --parent a0=ID ... for all six
              only the statements that differ from the parent's slot are written; an element the
              parent's slot set and this one leaves at zero is written as 0.0

`H` is a hand-written text: the comment header of the edits file and the `mix.w` line. After
assembling, `--verify RELEASE.json` (with the six --slot) checks tensor by tensor (stored sha256)
that every slot of the committee release is the tensors of the member release named for it.
"""
import argparse
import json
import subprocess
import sys
from pathlib import Path

SLOTS = ["a0", "a1", "b0", "b1", "b2", "b3"]


def statements(exe: str, release: str, cache: dict) -> dict:
    """`name[index]` -> value text, in the order `ignis wsp inspect --edits` prints them."""
    if release in cache:
        return cache[release]
    out = subprocess.run([exe, "wsp", "inspect", "--release", release, "--edits"], capture_output=True, text=True)
    if out.returncode != 0:
        sys.exit(f"ignis wsp inspect {release} failed:\n{out.stderr}")
    found = {}
    for line in out.stdout.splitlines():
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        key, _, value = line.partition("=")
        key, value = key.strip(), value.strip()
        if not key or not value or key in found:
            sys.exit(f"cannot read statement {line!r} of {release}")
        found[key] = value
    cache[release] = found
    return found


def member_lines(prefix: str, this: dict, parent) -> list:
    lines = []
    for key, value in this.items():
        if parent is None or key not in parent or float(parent[key]) != float(value):
            lines.append(f"{prefix}{key} = {value}")
    if parent is not None:
        for key in parent:
            if key not in this and float(parent[key]) != 0.0:
                lines.append(f"{prefix}{key} = 0.0")
    return lines


def tensor_hashes(root: Path, release: str) -> dict:
    path = root / "models" / release.removeprefix("sha256:") / "release.json"
    record = json.loads(path.read_text(encoding="utf-8"))
    return {t["name"]: (t["shape"], t["sha256"]) for t in record["tensors"]}


def pairs(values, what):
    out = {}
    for v in values or []:
        slot, _, rid = v.partition("=")
        if slot not in SLOTS or not rid.startswith("sha256:"):
            sys.exit(f"--{what} {v!r}: expected <slot>=sha256:... with slot in {SLOTS}")
        out[slot] = rid
    return out


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--ignis", default="target/release/ignis")
    ap.add_argument("--slot", action="append", help="<slot>=<release id>; all six are required")
    ap.add_argument("--parent", action="append", help="<slot>=<release id> of the parent committee's slot")
    ap.add_argument("--header")
    ap.add_argument("--out")
    ap.add_argument("--verify", help="release.json of the assembled committee: check its tensors against the members")
    ap.add_argument("--artifacts", default="artifacts")
    args = ap.parse_args()
    exe = Path(args.ignis)
    if not exe.exists() and exe.with_suffix(".exe").exists():
        exe = exe.with_suffix(".exe")
    exe = str(exe.resolve())
    slots, parents = pairs(args.slot, "slot"), pairs(args.parent, "parent")
    if set(slots) != set(SLOTS):
        sys.exit(f"give all six slots: {SLOTS}")

    if args.verify:
        blend = json.loads(Path(args.verify).read_text(encoding="utf-8"))
        have = {t["name"]: (t["shape"], t["sha256"]) for t in blend["tensors"]}
        bad = 0
        for slot in SLOTS:
            want = tensor_hashes(Path(args.artifacts), slots[slot])
            prefix = slot + "."
            for name, entry in want.items():
                if have.get(prefix + name) != entry:
                    bad += 1
                    print(f"DIFFERS {prefix}{name}")
            extra = [n for n in have if n.startswith(prefix) and n[len(prefix):] not in want]
            bad += len(extra)
            print(f"{prefix}* : {len(want)} tensors of {slots[slot]}")
        print("identical to the members" if bad == 0 else f"{bad} differences")
        return 1 if bad else 0

    if not (args.header and args.out):
        sys.exit("--header and --out are required")
    if parents and set(parents) != set(SLOTS):
        sys.exit("a revision names the parent of all six slots")
    cache = {}
    text = Path(args.header).read_text(encoding="utf-8").rstrip("\n") + "\n"
    counts = []
    for slot in SLOTS:
        this = statements(exe, slots[slot], cache)
        parent = statements(exe, parents[slot], cache) if parents else None
        lines = member_lines(slot + ".", this, parent)
        counts.append(f"{slot} {len(lines)}")
        text += f"\n# ---- slot {slot}: the stored values of {slots[slot]}"
        if parent is not None:
            text += f"\n#      ({len(lines)} statements that differ from the parent's slot, {parents[slot]})"
        else:
            text += f"\n#      ({len(lines)} statements)"
        text += "\n" + "\n".join(lines) + ("\n" if lines else "")
    Path(args.out).write_text(text, encoding="utf-8", newline="\n")
    print(f"{args.out}: " + ", ".join(counts))
    return 0


if __name__ == "__main__":
    sys.exit(main())
