#!/usr/bin/env python3
"""Assemble and score one D1 revision (PROTOCOL 13.3) of a seventh-generation W0 law.

    python -I tools/wsp/ext/law_d1.py assemble --slug cld      # edits from docs/wsp/research/phys_cld_d1.md
    python -I tools/wsp/ext/law_d1.py train    --slug cld      # train score of D1 beside W0 (study data)
    python -I tools/wsp/ext/law_d1.py look     --slug cld      # the one validation look of 13.3, and the result file

`assemble`: the "Edits" block of the dossier, verbatim, as a revision whose parent is the registered W0 release
(changed lines only: every number not written keeps the parent's value), compiled and assembled; then checked
element by element: the D1 release equals W0 except exactly the elements the block writes, which carry the block's
numbers. A line that cannot be applied as written stops the slug.
"""
import argparse
import hashlib
import json
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[2]
sys.path.insert(0, str(HERE))
from law_transcription_check import elements, statements  # noqa: E402

IG = REPO / "target-merged/release/ignis.exe"
LOST = {"bre": "t1: extinction IBS", "cld": "loss: integrated Brier", "slice": "lifted: Brier, mean of day means",
        "bass1": "quench: interval CRPS cm/s, mean of day means", "samer": "d10: CRPS of ln d10, case mean",
        "efield": "t1: CRPS uA, case mean"}


def rid(slug, cand):
    return json.loads((REPO / f"configs/models/law/{slug}/{cand}/release.json").read_text(encoding="utf-8"))["id"]


def ig(*a):
    r = subprocess.run([str(IG), "wsp", *a], cwd=REPO, capture_output=True, text=True)
    if r.returncode != 0:
        sys.exit(f"ignis wsp {' '.join(a[:3])} failed: {(r.stderr or r.stdout)[-1500:]}")
    return r.stdout


def dossier(slug):
    p = REPO / f"docs/wsp/research/phys_{slug}_d1.md"
    t = p.read_text(encoding="utf-8")
    lines = t.split("\n")
    i = next(k for k, l in enumerate(lines) if l.strip().startswith("## Edits"))
    a = next(k for k in range(i, len(lines)) if lines[k].startswith("```"))
    b = next(k for k in range(a + 1, len(lines)) if lines[k].startswith("```"))
    return p, hashlib.sha256(p.read_bytes()).hexdigest(), lines[a + 1:b], (a + 2, b)


def cmd_assemble(a):
    p, sha, block, (l0, l1) = dossier(a.slug)
    parent = rid(a.slug, "w0-fable")
    d = REPO / f"configs/models/law/{a.slug}/d1-fable"
    d.mkdir(parents=True, exist_ok=True)
    hdr = [f"# D1 of {a.slug}.wsp: the Edits block of docs/wsp/research/phys_{a.slug}_d1.md (lines {l0}-{l1}, sha256 {sha}),",
           f"# verbatim, as a revision of the registered W0 release sha256:{parent[7:15]}…{parent[-4:]}; every other number is the parent's."]
    (d / "edits.wsp").write_text("\n".join(hdr + block) + "\n", encoding="utf-8", newline="\n")
    meta = {"patch_id": f"law-{a.slug}-v1-d1-fable", "parent_release_id": parent,
            "author": {"provider": "anthropic", "model_id": "claude-fable-5-2"},
            "exposure": {"sources": [f"repo:docs/wsp/research/phys_{a.slug}_d1.md(sha256:{sha};data-informed-D1-from-train-days)",
                                     f"repo:docs/wsp/feedback/{a.slug}_train_anatomy.md(train-days-only)",
                                     "coordinator-summary-of-the-13.2-blind-look(aggregates-only;no-validation-case)"],
                         "datasets": [{"dataset_id": f"{a.slug}-train-cases(transfer_{a.slug}/cases_train.jsonl)", "partitions": ["train"]}],
                         "development_feedback": [f"PROTOCOL-13.2-validation-aggregates-of-the-W0-parent"],
                         "prior_candidates": [parent]},
            "rationale": f"D1 of {a.slug}.wsp (PROTOCOL 13.3): one damped step typed by the author from the train anatomy of the W0 law; "
                         f"the Edits block of docs/wsp/research/phys_{a.slug}_d1.md (sha256 {sha}) verbatim as a revision of the W0 release. "
                         "Transcribed by the assembly engineer (claude-opus-5-5); no number chosen.",
            "origin": "system"}
    (d / "meta.json").write_text(json.dumps(meta, indent=2) + "\n", encoding="utf-8", newline="\n")
    ig("compile", "--family", a.slug, "--edits", str(d / "edits.wsp"), "--meta", str(d / "meta.json"), "--out", str(d / "patch.json"))
    out = json.loads(ig("assemble", "--family", a.slug, "--patch", str(d / "patch.json"), "--binding",
                        str(REPO / f"configs/models/law/{a.slug}/binding.json"), "--out", str(d / "release.json")))
    new = out["release_id"]
    w0 = elements(statements(ig("inspect", "--release", parent, "--edits")))
    d1 = elements(statements(ig("inspect", "--release", new, "--edits")))
    typed = elements(statements("\n".join(block)))
    bad = {}
    for k in set(w0) | set(d1) | set(typed):
        want = typed[k] if k in typed else w0.get(k, 0.0)
        if d1.get(k, 0.0) != want:
            bad[str(k)] = (w0.get(k), d1.get(k), want)
    changed = {str(k): (w0.get(k, 0.0), v) for k, v in typed.items()}
    res = dict(slug=a.slug, dossier=str(p.relative_to(REPO)), sha256=sha, parent=parent, release=new, edits_lines=len([l for l in block if l.strip()]),
               changed=changed, mismatches=bad)
    (d / "assembly_check.json").write_text(json.dumps(res, indent=1) + "\n", encoding="utf-8", newline="\n")
    print(json.dumps(res, indent=1))
    return 1 if bad else 0


def summary_of(path_md):
    return path_md


def run_look(slug, release, part, look, out):
    cmd = [sys.executable, "-I", str(HERE / "law_score.py"), "look", "--slug", slug, "--release", release, "--partition", part, "--result-out", out]
    if look:
        cmd += ["--look", look]
    r = subprocess.run(cmd, cwd=REPO, capture_output=True, text=True)
    if r.returncode != 0:
        sys.exit(r.stderr[-2000:])
    s = r.stdout
    return json.loads(s[s.index("{"):s.rindex("}") + 1])


def last_summary(slug, release, part):
    for p in ("runs/dev/gen7/look_summaries_validation.jsonl", "runs/dev/gen7/look_summaries.jsonl") if part != "train" else ("runs/dev/gen7/look_summaries.jsonl",):
        rows = [json.loads(l) for l in (REPO / p).read_text(encoding="utf-8").splitlines() if l.strip()]
        rows = [r for r in rows if r["slug"] == slug and r["release"] == release and r["partition"] == part]
        if rows:
            return rows[-1]
    return None


def cmd_train(a):
    w0, d1 = rid(a.slug, "w0-fable"), rid(a.slug, "d1-fable")
    s = run_look(a.slug, d1, "train", None, f"runs/dev/gen7/train_looks/{a.slug}_d1-fable.md")
    p = last_summary(a.slug, w0, "train")
    out = {k: dict(w0=p["readings"][k]["release"] if p else None, d1=v["release"]) for k, v in s["readings"].items()}
    (REPO / f"runs/dev/gen7/train_looks/{a.slug}_d1_vs_w0.json").write_text(json.dumps(out, indent=1), encoding="utf-8")
    print(json.dumps(out, indent=1))
    return 0


def cmd_look(a):
    w0, d1 = rid(a.slug, "w0-fable"), rid(a.slug, "d1-fable")
    chk = json.loads((REPO / f"configs/models/law/{a.slug}/d1-fable/assembly_check.json").read_text(encoding="utf-8"))
    tr = json.loads((REPO / f"runs/dev/gen7/train_looks/{a.slug}_d1_vs_w0.json").read_text(encoding="utf-8"))
    part = "validation"
    vw0 = last_summary(a.slug, w0, part)
    out_md = f"docs/wsp/feedback/{a.slug}_d1_result.md"
    pth = REPO / out_md
    head = [f"# {a.slug}: D1 revision (PROTOCOL 13.3), one look beside its W0 parent", "",
            f"- Dossier: `{chk['dossier']}`, sha256 `{chk['sha256']}`.",
            f"- D1 release: `{d1}` (revision of the W0 release `{w0}`; changed: {json.dumps(chk['changed'])}; assembly check: "
            f"{'every other number equals the parent' if not chk['mismatches'] else 'MISMATCH ' + json.dumps(chk['mismatches'])}).", ""]
    pth.write_text("\n".join(head) + "\n", encoding="utf-8", newline="\n")
    s = run_look(a.slug, d1, part, "13.3", out_md)
    rows = ["", "## Summary: train (study data) and validation, W0 against D1", "",
            "| target, metric | W0 train | D1 train | W0 validation | D1 validation | simple baseline | best baseline (validation) |", "|---|---|---|---|---|---|---|"]
    res = {}
    for k, v in s["readings"].items():
        w = vw0["readings"][k]["release"] if vw0 and k in vw0["readings"] else None
        res[k] = dict(w0_train=tr.get(k, {}).get("w0"), d1_train=tr.get(k, {}).get("d1"), w0_val=w, d1_val=v["release"], simple=v["simple"], best=v["best"])
        rows.append(f"| {k} | {res[k]['w0_train']:.4f} | {res[k]['d1_train']:.4f} | {w:.4f} | {v['release']:.4f} | {v['simple'][1]:.4f} | {v['best'][1]:.4f} ({v['best'][0]}) |")
    lost = LOST[a.slug]
    others = [k for k in res if k != lost and not k.startswith(lost.split(":")[0] + ":")]
    if a.slug == "bre":
        others = ["t1: operator termination IBS"]
    better = res[lost]["d1_val"] < res[lost]["w0_val"]
    ok_other = all(res[k]["d1_val"] <= 1.05 * res[k]["w0_val"] for k in others if k in res)
    verdict = "D1 replaces its parent" if (better and ok_other) else "the parent (W0) stays"
    rows += ["", f"**Reading (13.3):** target that lost: {lost}: W0 {res[lost]['w0_val']:.4f}, D1 {res[lost]['d1_val']:.4f} "
             f"({'lower' if better else 'not lower'}); other scored target(s): "
             + "; ".join(f"{k}: W0 {res[k]['w0_val']:.4f}, D1 {res[k]['d1_val']:.4f} ({'within' if res[k]['d1_val'] <= 1.05 * res[k]['w0_val'] else 'more than'} 5 % of the parent)" for k in others if k in res)
             + f". **{verdict}.**", ""]
    with pth.open("a", encoding="utf-8", newline="\n") as f:
        f.write("\n".join(rows) + "\n")
    summ = dict(slug=a.slug, d1=d1, w0=w0, sha256=chk["sha256"], targets=res, lost=lost, verdict=verdict)
    with (REPO / "runs/dev/gen7/d1_summaries.jsonl").open("a", encoding="utf-8", newline="\n") as f:
        f.write(json.dumps(summ) + "\n")
    print(json.dumps(summ, indent=1))
    return 0


def main():
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    for n, fn in (("assemble", cmd_assemble), ("train", cmd_train), ("look", cmd_look)):
        s = sub.add_parser(n)
        s.add_argument("--slug", required=True)
        s.set_defaults(fn=fn)
    a = ap.parse_args()
    return a.fn(a)


if __name__ == "__main__":
    sys.exit(main())
