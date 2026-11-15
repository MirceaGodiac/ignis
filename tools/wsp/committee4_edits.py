#!/usr/bin/env python3
"""Transcribe member releases and typed tables into the edits file of a model of the fourth committee
(`acme_normal.wsp13`, forward `crates/ignis-wsp/src/committee4.rs`).

Its slots hold `flame8.rs` (a0, a1), `flame9.rs` (b0 to b3) and `flame10.rs` (c0 to c3) members: the
laws of the fuel-table members of `acme_normal.wsp12` with the second fuel table, the diluent table and
the heating law (`docs/wsp/research/phys_sflame_f3.md`, E4). The law numbers are not typed again: each
slot copies, statement by statement, every stored statement of a stored `acme_normal.wsp9` (a slots),
`wsp10` (b slots) or `wsp11` (c slots) release except those of its fuel table, which the typed tables
replace:

    --hinge-table  the fuel table of the hinge members (slots a0, a1, c0 to c3)
    --growth-table the fuel table of the growth members (slots b0 to b3)
    --dil-table    the diluent table of every member

each written into every slot it belongs to, with the slot prefix. The header holds the committee's own
lines and the member lines (`<slot>.bt.kappa_exp`, ...). The script chooses no value, computes none and
reads no data.

    python tools/wsp/committee4_edits.py --header H --hinge-table T1 --growth-table T2 --dil-table T3 \\
        --out edits.wsp --slot a0=ID ... --slot c3=ID
    ... --verify RELEASE.json (with the ten --slot): every non-table tensor of each slot (phys.mw_n2 excepted: dil.mw_g_mol replaces it) is the stored
        tensor of its source (sha256)
"""
import argparse
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from committee_edits import statements, tensor_hashes  # noqa: E402

SLOTS = ["a0", "a1", "b0", "b1", "b2", "b3", "c0", "c1", "c2", "c3"]
KIND = {"a": "wsp9", "b": "wsp10", "c": "wsp11"}


def table_lines(path):
    out = []
    for line in Path(path).read_text(encoding="utf-8").splitlines():
        code = line.split("#", 1)[0].strip()
        if code:
            out.append(code)
    return out


def model_of(root, rid):
    return json.loads((root / "models" / rid.removeprefix("sha256:") / "release.json").read_text(encoding="utf-8"))["model_name"]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--ignis", default="target/release/ignis")
    ap.add_argument("--slot", action="append", required=True)
    ap.add_argument("--header")
    ap.add_argument("--hinge-table")
    ap.add_argument("--growth-table")
    ap.add_argument("--dil-table")
    ap.add_argument("--out")
    ap.add_argument("--verify")
    ap.add_argument("--artifacts", default="artifacts")
    a = ap.parse_args()
    root = Path(a.artifacts)
    exe = Path(a.ignis)
    if not exe.exists() and exe.with_suffix(".exe").exists():
        exe = exe.with_suffix(".exe")
    slots = dict(v.split("=", 1) for v in a.slot)
    if set(slots) != set(SLOTS):
        sys.exit(f"give all ten slots: {SLOTS}")
    for s, rid in slots.items():
        m = model_of(root, rid)
        if m != f"acme_normal.{KIND[s[0]]}":
            sys.exit(f"slot {s}: {rid} is {m}, expected acme_normal.{KIND[s[0]]}")
    if a.verify:
        rel = json.loads(Path(a.verify).read_text(encoding="utf-8"))
        have = {t["name"]: (t["shape"], t["sha256"]) for t in rel["tensors"]}
        bad = 0
        for s in SLOTS:
            want = {k: v for k, v in tensor_hashes(root, slots[s]).items() if not k.startswith("fuel.") and k != "phys.mw_n2"}
            for k, v in want.items():
                if have.get(f"{s}.{k}") != v:
                    bad += 1
                    print(f"DIFFERS {s}.{k}")
            print(f"{s}.* : {len(want)} non-table tensors of {slots[s]}")
        print("identical to the sources" if bad == 0 else f"{bad} differences")
        return 1 if bad else 0
    tables = {"hinge": table_lines(a.hinge_table), "growth": table_lines(a.growth_table), "dil": table_lines(a.dil_table)}
    text = Path(a.header).read_text(encoding="utf-8").rstrip("\n") + "\n"
    cache = {}
    for s in SLOTS:
        st = {k: v for k, v in statements(str(exe.resolve()), slots[s], cache).items() if not k.startswith("fuel.") and k != "phys.mw_n2"}
        fuel = tables["growth" if s[0] == "b" else "hinge"]
        text += f"\n# ---- slot {s}: the stored values of {slots[s]} ({len(st)} statements, its fuel table left out)\n"
        text += "\n".join(f"{s}.{k} = {v}" for k, v in st.items()) + "\n"
        text += f"# ---- slot {s}: the typed {'growth' if s[0] == 'b' else 'hinge'} fuel table and the diluent table\n"
        text += "\n".join(f"{s}.{l}" for l in fuel + tables["dil"]) + "\n"
    Path(a.out).write_text(text, encoding="utf-8", newline="\n")
    print(f"{a.out}: ten slots")
    return 0


if __name__ == "__main__":
    sys.exit(main())
