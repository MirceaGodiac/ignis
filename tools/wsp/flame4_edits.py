#!/usr/bin/env python3
"""Transcribe two stored releases into the edits file of a hinge-law model with the burner-heating
law (`acme_*.wsp8`, forward `crates/ignis-wsp/src/flame4.rs`).

The spec of that model is the hinge-law spec (`flame.rs`, `acme_*.wsp2`) without its burner-limit
clock, plus the tensors the burner-limit clock of the growth-clock spec (`flame3.rs`, `acme_*.wsp3`)
reads. A release whose numbers already exist in two stored releases is not typed again: this script
copies, statement by statement,

    from --hinge   every stored statement except those of `ft.tau.*` and `ft.shape` (the clock this
                   model does not have) and except the burner tensors below, and
    from --burner  the stored statements of the burner tensors (`flame4::is_burner_tensor`: `bt.*`,
                   `se.rho_d_g_m_s`, `norm.r_ref_mm`, `norm.mtot_ref_kg_s`, `phys.mw_*`,
                   `num.bt_nodes`; those the spec fixes have no statement).

It chooses no value, computes none and reads no data. Either source may also be an `acme_*.wsp8`
release (its hinge part, or its burner part).

    root      python tools/wsp/flame4_edits.py --header H --out edits.wsp --hinge ID --burner ID
    revision  ... the same, plus --parent ID (the `acme_*.wsp8` release it revises)
              only the statements that differ from the parent are written; an element the parent set
              and this one leaves at zero is written as 0.0

`H` is a hand-written text: the comment header of the edits file. After assembling,
`--verify RELEASE.json` (with --hinge and --burner) checks tensor by tensor (stored sha256) that the
release holds the tensors of the two sources.
"""
import argparse
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from committee_edits import member_lines, statements, tensor_hashes  # noqa: E402


def tensor_of(statement_key: str) -> str:
    return statement_key.partition("[")[0]


def is_old_limit_clock(tensor: str) -> bool:
    """`flame4::is_old_limit_clock`."""
    return tensor.startswith("ft.tau.") or tensor == "ft.shape"


def is_burner_tensor(tensor: str) -> bool:
    """`flame4::is_burner_tensor`."""
    return (tensor.startswith("bt.") or tensor.startswith("phys.mw_")
            or tensor in ("se.rho_d_g_m_s", "norm.r_ref_mm", "norm.mtot_ref_kg_s", "num.bt_nodes"))


def is_hinge_tensor(tensor: str) -> bool:
    return not is_old_limit_clock(tensor) and not is_burner_tensor(tensor)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--ignis", default="target/release/ignis")
    ap.add_argument("--hinge", required=True, help="release id of the source of the hinge-law tensors")
    ap.add_argument("--burner", required=True, help="release id of the source of the burner tensors")
    ap.add_argument("--parent", help="release id of the acme_*.wsp8 release this one revises")
    ap.add_argument("--header")
    ap.add_argument("--out")
    ap.add_argument("--verify", help="release.json of the assembled release: check its tensors against the sources")
    ap.add_argument("--artifacts", default="artifacts")
    args = ap.parse_args()
    for rid in (args.hinge, args.burner, args.parent):
        if rid is not None and not rid.startswith("sha256:"):
            sys.exit(f"{rid!r}: expected sha256:...")
    exe = Path(args.ignis)
    if not exe.exists() and exe.with_suffix(".exe").exists():
        exe = exe.with_suffix(".exe")
    exe = str(exe.resolve())

    if args.verify:
        release = json.loads(Path(args.verify).read_text(encoding="utf-8"))
        hinge = tensor_hashes(Path(args.artifacts), args.hinge)
        burner = tensor_hashes(Path(args.artifacts), args.burner)
        bad = 0
        for t in release["tensors"]:
            want = (burner if is_burner_tensor(t["name"]) else hinge).get(t["name"])
            if want != (t["shape"], t["sha256"]):
                bad += 1
                print(f"DIFFERS {t['name']}")
        n_burner = sum(is_burner_tensor(t["name"]) for t in release["tensors"])
        print(f"{len(release['tensors']) - n_burner} tensors of {args.hinge}")
        print(f"{n_burner} tensors of {args.burner}")
        print("identical to the sources" if bad == 0 else f"{bad} differences")
        return 1 if bad else 0

    if not (args.header and args.out):
        sys.exit("--header and --out are required")
    cache = {}
    parent = statements(exe, args.parent, cache) if args.parent else None
    text = Path(args.header).read_text(encoding="utf-8").rstrip("\n") + "\n"
    counts = []
    for what, rid, keep in (("hinge laws", args.hinge, is_hinge_tensor), ("burner limit", args.burner, is_burner_tensor)):
        this = {k: v for k, v in statements(exe, rid, cache).items() if keep(tensor_of(k))}
        if not this:
            sys.exit(f"{rid} has no statement of the {what}")
        base = None if parent is None else {k: v for k, v in parent.items() if keep(tensor_of(k))}
        lines = member_lines("", this, base)
        counts.append(f"{what} {len(lines)}")
        text += f"\n# ---- {what}: the stored values of {rid}"
        if base is not None:
            text += f"\n#      ({len(lines)} statements that differ from the parent, {args.parent})"
        else:
            text += f"\n#      ({len(lines)} statements)"
        text += "\n" + "\n".join(lines) + ("\n" if lines else "")
    Path(args.out).write_text(text, encoding="utf-8", newline="\n")
    print(f"{args.out}: " + ", ".join(counts))
    return 0


if __name__ == "__main__":
    sys.exit(main())
