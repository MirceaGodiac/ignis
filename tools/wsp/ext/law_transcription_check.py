#!/usr/bin/env python3
"""Transcription check of a law release against its dossier block, with no data involved.

Parses the literal assignments of an edits block (scalar `a = v`, `a[i] = v`, `a[i:j] = [..]`,
`a[i, j] = v`, `a[i, j:k] = [..]`, `a[i:j, k:l] = [[..], ..]`) into single elements and compares them with
`ignis wsp inspect --release <id> --edits` (every number of the release, one element per line). Reports:
  * an element of the dossier whose released value differs (must be none);
  * an element of the release that the dossier did not write and that is not zero (a value inherited from
    the neutral root: the dossier left that number out).

    python -I tools/wsp/ext/law_transcription_check.py --release sha256:... --edits configs/models/law/<slug>/w0-fable/edits.wsp
"""
import argparse
import ast
import json
import re
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]


def statements(text):
    """Join continuation lines (an open bracket) into single statements; drop comments."""
    out, buf, depth = [], "", 0
    for line in text.splitlines():
        line = line.split("#", 1)[0].rstrip()
        if not line.strip():
            continue
        buf = (buf + " " + line.strip()) if buf else line.strip()
        depth = buf.count("[") - buf.count("]")
        if depth <= 0 and "=" in buf:
            out.append(buf)
            buf = ""
    if buf:
        raise ValueError(f"unterminated statement: {buf}")
    return out


def index_list(s):
    """'0:5, 0:7' -> [range(0,5), range(0,7)]; '3' -> [range(3,4)]"""
    res = []
    for part in s.split(","):
        part = part.strip()
        if ":" in part:
            a, b = part.split(":")
            res.append(list(range(int(a), int(b))))
        else:
            res.append([int(part)])
    return res


def elements(stmts):
    el = {}
    for st in stmts:
        lhs, rhs = st.split("=", 1)
        lhs, rhs = lhs.strip(), ast.literal_eval(rhs.strip())
        m = re.fullmatch(r"([A-Za-z0-9_.]+)(?:\[(.*)\])?", lhs)
        name, idx = m.group(1), m.group(2)
        if idx is None:
            el[(name, ())] = float(rhs)
            continue
        dims = index_list(idx)
        if not isinstance(rhs, list):   # one number written to every element of a slice
            import itertools
            for ij in itertools.product(*dims):
                el[(name, tuple(ij))] = float(rhs)
            continue
        if len(dims) == 1:
            vals = rhs if isinstance(rhs, list) else [rhs]
            assert len(vals) == len(dims[0]), st
            for i, v in zip(dims[0], vals):
                el[(name, (i,))] = float(v)
        else:
            rows = rhs if isinstance(rhs, list) else [[rhs]]
            if len(dims[0]) == 1 and rows and not isinstance(rows[0], list):
                rows = [rows]
            if len(dims[1]) == 1 and rows and not isinstance(rows[0], list):
                rows = [[v] for v in rows]
            assert len(rows) == len(dims[0]), st
            for i, row in zip(dims[0], rows):
                row = row if isinstance(row, list) else [row]
                assert len(row) == len(dims[1]), st
                for j, v in zip(dims[1], row):
                    el[(name, (i, j))] = float(v)
    return el


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--release", required=True)
    ap.add_argument("--edits", required=True)
    ap.add_argument("--ignis", default="target-merged/release/ignis.exe")
    a = ap.parse_args()
    dossier = elements(statements(Path(a.edits).read_text(encoding="utf-8")))
    r = subprocess.run([str(REPO / a.ignis), "wsp", "inspect", "--release", a.release, "--edits"], cwd=REPO,
                       capture_output=True, text=True, check=True)
    released = elements(statements(r.stdout))
    differ = {k: (v, released.get(k)) for k, v in dossier.items() if released.get(k, 0.0) != v}
    inherited = {k: v for k, v in released.items() if k not in dossier and v != 0.0}
    fmt = lambda k: k[0] + (("[" + ", ".join(map(str, k[1])) + "]") if k[1] else "")  # noqa: E731
    print(json.dumps(dict(release=a.release, dossier_elements=len(dossier), released_elements_listed=len(released),
                          differ={fmt(k): v for k, v in differ.items()},
                          not_written_by_dossier_nonzero={fmt(k): v for k, v in inherited.items()}), indent=1))
    return 1 if differ else 0


if __name__ == "__main__":
    sys.exit(main())
