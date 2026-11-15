#!/usr/bin/env python3
"""Transcribe two member releases into the edits file of a blend model (`acme_*.wsp4`).

The blend forward (`crates/ignis-wsp/src/blend.rs`) holds one complete set of numbers for each of its
two members under the tensor prefixes `a.` and `b.`. Those numbers are not written a second time:
they are the stored values of two existing literal releases, copied statement by statement with the
prefix added. This script makes that copy and nothing else. It chooses no value, computes none and
reads no data. The only numbers of a blend that are new are its two weights, which the author types
in the header file.

    root      python tools/wsp/blend_edits.py --header H --a ID --b ID --out edits.wsp
    revision  ... --parent-a ID --parent-b ID
              only the statements that differ from the parent's members are written; an element the
              parent's member set and this member leaves at zero is written as 0.0

`H` is a hand-written text: the comment header of the edits file and, for a root, the `mix.w` line.
After assembling, `--verify RELEASE.json` checks tensor by tensor (stored sha256) that `a.*` and
`b.*` of the blend release are the tensors of the two member releases.
"""
import argparse
import json
import subprocess
import sys
from pathlib import Path


def statements(exe: str, release: str) -> dict:
    """`name[index]` -> value text, in the order `ignis wsp inspect --edits` prints them."""
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
    return found


def member_lines(prefix: str, this: dict, parent: dict | None) -> list:
    lines = []
    for key, value in this.items():
        if parent is None or key not in parent or float(parent[key]) != float(value):
            lines.append(f"{prefix}{key} = {value}")
    if parent is not None:
        for key in parent:
            if key not in this and float(parent[key]) != 0.0:
                lines.append(f"{prefix}{key} = 0.0")
    return lines


def release_file(exe: str, release: str) -> dict:
    out = subprocess.run([exe, "wsp", "inspect", "--release", release], capture_output=True, text=True)
    if out.returncode != 0:
        sys.exit(f"ignis wsp inspect {release} failed:\n{out.stderr}")
    return json.loads(out.stdout)


def tensor_hashes(root: Path, release: str) -> dict:
    path = root / "models" / release.removeprefix("sha256:") / "release.json"
    record = json.loads(path.read_text(encoding="utf-8"))
    return {t["name"]: (t["shape"], t["sha256"]) for t in record["tensors"]}


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--ignis", default="target/release/ignis")
    ap.add_argument("--a", required=True, help="release id of member a (flame.rs)")
    ap.add_argument("--b", required=True, help="release id of member b (flame3.rs)")
    ap.add_argument("--parent-a")
    ap.add_argument("--parent-b")
    ap.add_argument("--header")
    ap.add_argument("--out")
    ap.add_argument("--verify", help="release.json of the assembled blend: check its tensors against the members")
    ap.add_argument("--artifacts", default="artifacts")
    args = ap.parse_args()
    exe = Path(args.ignis)
    if not exe.exists() and exe.with_suffix(".exe").exists():
        exe = exe.with_suffix(".exe")
    exe = str(exe.resolve())

    if args.verify:
        blend = json.loads(Path(args.verify).read_text(encoding="utf-8"))
        have = {t["name"]: (t["shape"], t["sha256"]) for t in blend["tensors"]}
        bad = 0
        for prefix, member in (("a.", args.a), ("b.", args.b)):
            want = tensor_hashes(Path(args.artifacts), member)
            for name, entry in want.items():
                if have.get(prefix + name) != entry:
                    bad += 1
                    print(f"DIFFERS {prefix}{name}")
            extra = [n for n in have if n.startswith(prefix) and n[len(prefix):] not in want]
            bad += len(extra)
            for n in extra:
                print(f"EXTRA {n}")
            print(f"{prefix}* : {len(want)} tensors of {member}")
        print("identical to the members" if bad == 0 else f"{bad} differences")
        return 1 if bad else 0

    if not (args.header and args.out):
        sys.exit("--header and --out are required")
    if bool(args.parent_a) != bool(args.parent_b):
        sys.exit("give both --parent-a and --parent-b, or neither")
    a, b = statements(exe, args.a), statements(exe, args.b)
    pa = statements(exe, args.parent_a) if args.parent_a else None
    pb = statements(exe, args.parent_b) if args.parent_b else None
    la, lb = member_lines("a.", a, pa), member_lines("b.", b, pb)
    text = Path(args.header).read_text(encoding="utf-8").rstrip("\n") + "\n\n"
    text += f"# ---- member a: the stored values of {args.a}"
    text += f"\n#      ({len(la)} statements" + (f" that differ from {args.parent_a}" if pa is not None else "") + ")\n"
    text += "\n".join(la) + "\n\n"
    text += f"# ---- member b: the stored values of {args.b}"
    text += f"\n#      ({len(lb)} statements" + (f" that differ from {args.parent_b}" if pb is not None else "") + ")\n"
    text += "\n".join(lb) + "\n"
    Path(args.out).write_text(text, encoding="utf-8", newline="\n")
    print(f"{args.out}: a {len(la)} statements, b {len(lb)} statements")
    return 0


if __name__ == "__main__":
    sys.exit(main())
