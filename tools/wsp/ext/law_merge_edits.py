#!/usr/bin/env python3
"""Merge a dossier's changed lines into a parent edits block for a NEW slug, verbatim in numbers.

The edit language refuses an element written twice in one file. When a dossier says "all lines of the parent
unchanged, except the lines below", an element the parent writes inside a list line (`skip[0:5] = [...]`,
`W1[0, 0:5] = [...]`, `W1[0:4, 0:5] = [[...]]`) cannot be written again on its own line. This tool writes the new
number INTO the parent's list line instead (form only: the same element, the dossier's number) and keeps every
other line as it is; a changed element the parent does not write is appended as the dossier wrote it.

    python -I tools/wsp/ext/law_merge_edits.py --parent configs/models/law/bass2/s1-fable/edits.wsp --changes block.wsp --out edits.wsp
"""
import argparse
import ast
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from law_transcription_check import elements, index_list, statements  # noqa: E402


def merge(parent_text, change_text):
    changes = elements(statements(change_text))
    used = set()
    out = []
    buf, depth = [], 0
    lines = parent_text.split("\n")
    i = 0
    while i < len(lines):
        line = lines[i]
        code = line.split("#", 1)[0].strip()
        if not code or "=" not in code:
            out.append(line)
            i += 1
            continue
        stmt_lines = [line]
        st = code
        while st.count("[") > st.count("]"):
            i += 1
            stmt_lines.append(lines[i])
            st += " " + lines[i].split("#", 1)[0].strip()
        el = elements([st])
        hit = {k for k in el if k in changes}
        if not hit:
            out.extend(stmt_lines)
        else:
            lhs, rhs = st.split("=", 1)
            lhs = lhs.strip()
            m = re.fullmatch(r"([A-Za-z0-9_.]+)(?:\[(.*)\])?", lhs)
            name, idx = m.group(1), m.group(2)
            val = ast.literal_eval(rhs.strip())
            if idx is None:
                val = changes[(name, ())]
            else:
                dims = index_list(idx)
                if not isinstance(val, list) and all(len(d) == 1 for d in dims):
                    val = changes[(name, tuple(d[0] for d in dims))]
                elif not isinstance(val, list) and all(k in changes for k in el) and len({changes[k] for k in el}) == 1:
                    val = changes[next(iter(el))]   # the whole slice changed to one number: the line keeps its form
                elif not isinstance(val, list):
                    raise SystemExit(f"a parent line writes one number to a slice and the dossier changes part of it: {st}")
                elif len(dims) == 1:
                    val = [changes.get((name, (j,)), v) for j, v in zip(dims[0], val)]
                elif len(dims[0]) == 1:
                    val = [changes.get((name, (dims[0][0], j)), v) for j, v in zip(dims[1], val)]
                else:
                    val = [[changes.get((name, (a, b)), v) for b, v in zip(dims[1], row)] for a, row in zip(dims[0], val)]
            out.append(f"{lhs} = {val!r}".replace("'", ""))
            out.append(f"# (form only: {', '.join(n + str(list(ix)) for n, ix in sorted(hit))} written inside the parent's line above, the dossier's number)")
            used |= hit
        i += 1
    rest = [s for s in change_text.split("\n")]
    keep = []
    for s in rest:
        code = s.split("#", 1)[0].strip()
        if code and "=" in code:
            el = elements([code])
            if el and all(k in used for k in el):
                continue
            if el and any(k in used for k in el):
                # part of the dossier's list line is written inside the parent's line above; the rest, element by element
                rest_el = [(k, v) for k, v in el.items() if k not in used]
                for (n, ix), v in rest_el:
                    keep.append(f"{n}[{', '.join(str(j) for j in ix)}] = {v!r}")
                keep.append(f"# (form only: the elements of the dossier's line `{code.split('=', 1)[0].strip()}` that the parent does not write, one per line)")
                continue
        keep.append(s)
    return "\n".join(out + keep) + "\n", used


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--parent", required=True)
    ap.add_argument("--changes", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--skip-header", type=int, default=0)
    a = ap.parse_args()
    pt = Path(a.parent).read_text(encoding="utf-8").split("\n")[a.skip_header:]
    text, used = merge("\n".join(pt), Path(a.changes).read_text(encoding="utf-8"))
    Path(a.out).write_text(text, encoding="utf-8", newline="\n")
    print(f"{len(used)} changed elements written inside parent lines")
    return 0


if __name__ == "__main__":
    sys.exit(main())
