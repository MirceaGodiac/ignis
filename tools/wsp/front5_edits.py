#!/usr/bin/env python3
"""Transcribe a stored front release and a typed header into the edits file of a fabric-flame model
with the boundary law of ignition and quench (`bass_front.wsp5`, forward
`crates/ignis-wsp/src/front5.rs`).

The spec of that model is the spec of the first front forward (`front.rs`, `bass_front.wsp`) without
its quench hazard (`quench.*`), plus the tensors of the boundary law (section 6.2 of
`docs/wsp/research/phys_quench_w0.md`: `sus.*`, `ign.dx`, `ign.sd_a`, `ign.t_req`, `ign.sd_lnt`,
`ext.*`). The tip law of a release already exists in a stored release and is not typed again: this
script copies, statement by statement,

    from --tip     every stored statement except those of `quench.*` (the hazard this model does
                   not have) and except the boundary tensors; the source is a `bass_front.wsp`
                   release, or a `bass_front.wsp5` release (its tip part)
    from --header  the hand-written text, as it is: the comment header of the edits file and the
                   statements of the boundary tensors. The header may state nothing else; the
                   boundary tensors come from it alone.

It chooses no value, computes none and reads no data.

    root      python tools/wsp/front5_edits.py --tip ID --header H --out edits.wsp
    revision  ... the same, plus --parent ID (the `bass_front.wsp5` release it revises): only the
              tip statements that differ from the parent are written (an element the parent set and
              this one leaves at zero is written as 0.0); the header is written as typed

After assembling, `--verify RELEASE.json` (with --tip) checks tensor by tensor (stored sha256) that
the release holds the tip tensors of the source and names every tensor it took from the header.
"""
import argparse
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from committee_edits import member_lines, statements, tensor_hashes  # noqa: E402

BOUNDARY_SCALARS = ("ign.dx", "ign.sd_a", "ign.t_req", "ign.sd_lnt")


def tensor_of(statement_key: str) -> str:
    return statement_key.partition("[")[0].strip()


def is_quench(tensor: str) -> bool:
    """The hazard network of `front.rs` that `front5.rs` replaces."""
    return tensor.startswith("quench.")


def is_boundary(tensor: str) -> bool:
    """The authored tensors of the boundary law (`front5::spec`, section 6.2 of the dossier)."""
    return tensor.startswith("sus.") or tensor.startswith("ext.") or tensor in BOUNDARY_SCALARS


def is_quadrature(tensor: str) -> bool:
    """Constants of the fifth spec that no earlier spec has."""
    return tensor.startswith("quad.")


def is_tip(tensor: str) -> bool:
    return not is_quench(tensor) and not is_boundary(tensor) and not is_quadrature(tensor)


def header_tensors(text: str) -> list:
    """Tensor names of the statements of a header (comments and blank lines skipped)."""
    names = []
    for line in text.splitlines():
        code = line.split("#", 1)[0].strip()
        if not code or code.startswith("[") or code.startswith("]") or "=" not in code:
            continue  # comments, blank lines, continuation lines of a nested list
        names.append(tensor_of(code.partition("=")[0]))
    return names


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--ignis", default="target/release/ignis")
    ap.add_argument("--tip", required=True, help="release id of the source of the tip-law tensors")
    ap.add_argument("--parent", help="release id of the bass_front.wsp5 release this one revises")
    ap.add_argument("--header", help="hand-written header: comments and the boundary-law statements")
    ap.add_argument("--out")
    ap.add_argument("--verify", help="release.json of the assembled release: check its tensors against the source")
    ap.add_argument("--artifacts", default="artifacts")
    args = ap.parse_args()
    for rid in (args.tip, args.parent):
        if rid is not None and not rid.startswith("sha256:"):
            sys.exit(f"{rid!r}: expected sha256:...")
    exe = Path(args.ignis)
    if not exe.exists() and exe.with_suffix(".exe").exists():
        exe = exe.with_suffix(".exe")
    exe = str(exe.resolve())

    if args.verify:
        release = json.loads(Path(args.verify).read_text(encoding="utf-8"))
        if release.get("model_name") != "bass_front.wsp5":
            sys.exit(f"{args.verify} is {release.get('model_name')}, not bass_front.wsp5")
        tip = tensor_hashes(Path(args.artifacts), args.tip)
        bad, n_tip, from_header = 0, 0, []
        for t in release["tensors"]:
            name = t["name"]
            if is_boundary(name):
                from_header.append(name)
            elif is_quadrature(name):
                continue
            else:
                n_tip += 1
                if tip.get(name) != (t["shape"], t["sha256"]):
                    bad += 1
                    print(f"DIFFERS {name}")
        missing = [n for n in tip if is_tip(n) and n not in {t["name"] for t in release["tensors"]}]
        for n in missing:
            bad += 1
            print(f"MISSING {n}")
        print(f"{n_tip} tip tensors of {args.tip}")
        print(f"{len(from_header)} boundary tensors from the header: {', '.join(sorted(from_header))}")
        print("identical to the source" if bad == 0 else f"{bad} differences")
        return 1 if bad else 0

    if not (args.header and args.out):
        sys.exit("--header and --out are required")
    header = Path(args.header).read_text(encoding="utf-8").rstrip("\n") + "\n"
    stray = [n for n in header_tensors(header) if not is_boundary(n)]
    if stray:
        sys.exit(f"{args.header} states tensors that are not of the boundary law: {sorted(set(stray))}")
    if not header_tensors(header):
        sys.exit(f"{args.header} states no boundary-law tensor")
    cache = {}
    this = {k: v for k, v in statements(exe, args.tip, cache).items() if is_tip(tensor_of(k))}
    if not this:
        sys.exit(f"{args.tip} has no statement of the tip law")
    base = None
    if args.parent:
        base = {k: v for k, v in statements(exe, args.parent, cache).items() if is_tip(tensor_of(k))}
    lines = member_lines("", this, base)
    text = header
    text += f"\n# ---- tip law: the stored values of {args.tip}"
    if base is not None:
        text += f"\n#      ({len(lines)} statements that differ from the parent, {args.parent})"
    else:
        text += f"\n#      ({len(lines)} statements)"
    text += "\n" + "\n".join(lines) + ("\n" if lines else "")
    Path(args.out).write_text(text, encoding="utf-8", newline="\n")
    print(f"{args.out}: tip law {len(lines)}, boundary law {len(header_tensors(header))} (typed)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
