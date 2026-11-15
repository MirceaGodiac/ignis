#!/usr/bin/env python3
"""Writes the machine-made registers of the review dossier (docs/wsp/dossier/registers/).

Nothing here judges or computes a model number: it lists what the repository already records, so
that a reviewer can check the prose of the dossier against it.

  candidates.md / candidates.json   every candidate under configs/models: model, release id, parent,
                                    how it was built, statements typed, the exposure its meta.json declares
  agents.md                         every delegated agent of the workstream: task, model asked for, model
                                    that answered, whether it finished, where its report was saved
                                    (needs --transcripts; the transcripts are local and not in git)
  local_folders.md                  the local, git-ignored working folders the dossier cites: file counts
                                    and bytes only, never a file's content

    python tools/wsp/dossier_registers.py [--transcripts DIR [DIR ...]]
"""
import argparse
import json
import re
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
OUT = REPO / "docs/wsp/dossier/registers"

LOCAL = [
    ("data", "shared data root (raw samples, manifests, records, splits); the test partition's files are in it and are never opened by this lane"),
    ("runs/dev/analysis", "train-only tables of tools/wsp/analysis (profiles, per-run finite differences)"),
    ("runs/dev/research5/shift", "where validation inputs sit relative to train (inputs only)"),
    ("runs/dev/research5/flex_flame", "flame-camera measurement of the FLEX train runs"),
    ("runs/dev/research5/acme_raw", "raw-channel measurement of the ACME train tests"),
    ("runs/dev/research5/bass1", "tracks of the earlier BASS campaign (PSI-26, outside every partition)"),
    ("runs/dev/research6", "physics dossiers of the fifth generation (working files)"),
    ("runs/dev/nasa/PSI-69", "downloads: flame-camera files of FLEX train runs"),
    ("runs/dev/nasa/PSI-159", "downloads: raw channels, timelines and script parameters of ACME tests"),
    ("runs/dev/nasa/PSI-26", "downloads: videos and documents of the earlier BASS campaign"),
    ("runs/dev/data_ext", "PRIVATE copy of the data root with flame-camera series and programmed stops added; holds validation series: read only through the prefix builder"),
    ("runs/dev/bass_front/val_sealed", "SEALED validation front tracks: never opened by the author of the model"),
    ("runs/dev/ext", "train feedback of the fifth-generation candidates on the private data root"),
    ("runs/dev/gen5", "fifth generation: train feedback, single-change variants, the aggregate files of the registered validation look and its split comparison (the comparison files hold per-case validation scores: aggregates only are read)"),
    ("runs/dev/holdout_ext", "PRIVATE vault: validation continuations of the private data root, written by the evaluator's deposit command; never opened"),
    ("artifacts/models", "local release registry (rebuilt by tools/wsp/reassemble.py)"),
]


# Reports that were saved after editing or reformatting (the text match below cannot find them), and
# helpers whose product is code or sealed files, not a report. Typed by hand from the session record.
BY_TASK = {
    "FLEX experiment science dossier": "docs/wsp/research/exp_flex.md (reformatted)",
    "BASS experiment science dossier": "docs/wsp/research/exp_bass.md (reformatted)",
    "ACME CFI-G experiment dossier": "docs/wsp/research/exp_acme.md (reformatted)",
    "Methanol train residual audit": "docs/wsp/feedback/flex_methanol_d2_train_audit.md (reformatted)",
    "BASS train video measurement": "docs/wsp/feedback/bass_train_measurement.md (reformatted)",
    "BASS validation measurement A": "sealed track files (runs/dev/bass_front/val_sealed, not in git); no report",
    "BASS validation measurement B": "sealed track files (runs/dev/bass_front/val_sealed, not in git); no report",
    "Map ACME expert and data": "code map used in the session only",
    "Map evaluation and registry machinery": "code map used in the session only",
    "Map BASS expert and observables": "code map used in the session only",
    "Wire third-generation droplet models": "code (crates/ignis-wsp), reviewed by the author",
    "Tests for droplet3 forward": "code (crates/ignis-wsp/tests), reviewed by the author",
    "Droplet fourth-generation forward": "code (crates/ignis-wsp), finished by the author",
    "Growth-clock forward tests": "code (crates/ignis-wsp/tests), reviewed by the author",
    "BASS front scoring tool and tests": "code (crates/ignis-wsp-dev), reviewed by the author",
    "Implement the WSP ModelService": "code (crates/ignis-wsp/src/service.rs), reviewed by the author",
}


def git_tracked(path: Path) -> bool:
    r = subprocess.run(["git", "ls-files", "--error-unmatch", str(path.relative_to(REPO)).replace("\\", "/")],
                       cwd=REPO, capture_output=True, text=True)
    return r.returncode == 0


def statements(edits: Path) -> int:
    if not edits.exists():
        return 0
    n = 0
    for line in edits.read_text(encoding="utf-8", errors="replace").splitlines():
        s = line.split("#", 1)[0].strip()
        if re.match(r"^[A-Za-z_][\w.]*(\[[^\]]*\])?\s*=", s):
            n += 1
    return n


def candidates():
    rows = []
    for meta_path in sorted((REPO / "configs/models").rglob("meta.json")):
        d = meta_path.parent
        meta = json.loads(meta_path.read_text(encoding="utf-8"))
        rel = {}
        if (d / "release.json").exists():
            rel = json.loads((d / "release.json").read_text(encoding="utf-8"))
        con = rel.get("construction") or {}
        params = ((rel.get("architecture") or {}).get("parameters")) or {}
        exp = meta.get("exposure") or {}
        parts = sorted({p for ds in exp.get("datasets", []) for p in ds.get("partitions", [])})
        rows.append({
            "path": str(d.relative_to(REPO)).replace("\\", "/"),
            "in_git": git_tracked(meta_path),
            "patch_id": meta.get("patch_id"),
            "model_name": rel.get("model_name"),
            "release_id": rel.get("id"),
            "parent_release_id": meta.get("parent_release_id"),
            "construction_method": con.get("construction_method"),
            "forward": ((con.get("forward") or {}).get("entry_point")),
            "forward_hash": con.get("forward_hash"),
            "weights_hash": con.get("weights_hash"),
            "author": (meta.get("author") or {}).get("model_id"),
            "statements_typed": statements(d / "edits.wsp"),
            "authored_total": params.get("authored_total"),
            "authored_nonzero": params.get("authored_nonzero"),
            "partitions_declared": parts,
            "exposure": exp,
            "rationale": meta.get("rationale"),
        })
    OUT.mkdir(parents=True, exist_ok=True)
    (OUT / "candidates.json").write_text(json.dumps(rows, indent=1, ensure_ascii=False) + "\n", encoding="utf-8", newline="\n")
    lines = ["# Register: every candidate under `configs/models`", "",
             "Made by `tools/wsp/dossier_registers.py` from each candidate's `meta.json`, `release.json` and `edits.wsp`.",
             "Full exposure records and rationales: `candidates.json` beside this file. \"Statements\" counts the",
             "assignment lines of `edits.wsp` (for a blend or committee release most of them are copied member",
             "values, see the file's header). \"In git\" is the state when the register was made; a release id shown",
             "for a folder that is not in git may be stale (see `08_in_progress.md`).", "",
             "| Candidate folder | Model | Built as | Release id | Parent | Statements | Elements (non-zero) | Partitions declared | In git |",
             "|---|---|---|---|---|---|---|---|---|"]
    for r in rows:
        rid = (r["release_id"] or "—").replace("sha256:", "")[:12]
        par = (r["parent_release_id"] or "—").replace("sha256:", "")[:12]
        el = f'{r["authored_total"]} ({r["authored_nonzero"]})' if r["authored_total"] is not None else "—"
        lines.append(f'| `{r["path"].replace("configs/models/", "")}` | `{r["model_name"] or "—"}` | {r["construction_method"] or "—"} | '
                     f'`{rid}` | `{par}` | {r["statements_typed"]} | {el} | {", ".join(r["partitions_declared"]) or "none"} | '
                     f'{"yes" if r["in_git"] else "**no**"} |')
    lines += ["", f"{len(rows)} candidates. `literal_one_shot` = written with no outcome exposure (W0 and reference baselines);",
              "`literal_revision` = typed over a parent after development data were read.", ""]
    (OUT / "candidates.md").write_text("\n".join(lines), encoding="utf-8", newline="\n")
    return len(rows)


def agent_rows(folder: Path, docs_text):
    rows = []
    for meta_path in sorted(folder.glob("agent-*.meta.json")):
        meta = json.loads(meta_path.read_text(encoding="utf-8"))
        jl = meta_path.with_name(meta_path.name.replace(".meta.json", ".jsonl"))
        models, first, last, final = {}, None, None, None
        if jl.exists():
            with open(jl, encoding="utf-8", errors="replace") as fh:
                for line in fh:
                    try:
                        o = json.loads(line)
                    except ValueError:
                        continue
                    ts = o.get("timestamp")
                    if ts:
                        first = first or ts
                        last = ts
                    msg = o.get("message") or {}
                    if msg.get("role") != "assistant":
                        continue
                    if msg.get("model"):
                        models[msg["model"]] = models.get(msg["model"], 0) + 1
                    for block in msg.get("content") or []:
                        if isinstance(block, dict) and block.get("type") == "text" and len(block.get("text", "")) >= 4000:
                            final = block["text"]
        saved = "—"
        if final:
            body = final.replace("\r\n", "\n")
            probe = body[len(body) // 2: len(body) // 2 + 160]
            for name, text in docs_text.items():
                if probe and probe in text:
                    saved = name
                    break
            else:
                saved = "not saved in the repository"
        task = meta.get("description") or ""
        if not saved.startswith("docs/") and task in BY_TASK:
            saved = BY_TASK[task]
        rows.append({
            "started": (first or "")[:16].replace("T", " "),
            "by": "author" if int(meta.get("spawnDepth") or 1) <= 1 else "a delegate",
            "task": task,
            "asked_model": meta.get("model") or "(default)",
            "asked_effort": meta.get("effort") or "(default)",
            "answered": ", ".join(sorted(models, key=models.get, reverse=True)) or "—",
            "finished": bool(meta.get("finished")),
            "report": saved,
        })
    return rows


def agents(folders):
    docs_text = {}
    for sub in ("research", "feedback"):
        for p in sorted((REPO / "docs/wsp" / sub).glob("*.md")):
            docs_text[f"docs/wsp/{sub}/{p.name}"] = p.read_text(encoding="utf-8", errors="replace")
    rows = []
    for f in folders:
        rows += agent_rows(Path(f), docs_text)
    rows.sort(key=lambda r: r["started"])
    lines = ["# Register: delegated agents of the workstream", "",
             "Made by `tools/wsp/dossier_registers.py --transcripts …` from the transcripts of the author's sessions",
             "(local files, not in git). \"Asked\" is what the launch requested; \"answered\" is the model id recorded on the",
             "agent's own messages. A report is matched to a saved file by its text. An agent with no saved report either",
             "did not finish, returned a short answer that was used in the session only, or was a measurement or",
             "review helper whose result went into another file; the dossier names the ones that matter.", "",
             "| Started (UTC) | Launched by | Task | Asked: model / effort | Answered by | Finished | Product |",
             "|---|---|---|---|---|---|---|"]
    for r in rows:
        lines.append(f'| {r["started"]} | {r["by"]} | {r["task"]} | {r["asked_model"]} / {r["asked_effort"]} | {r["answered"]} | '
                     f'{"yes" if r["finished"] else "no"} | {r["report"]} |')
    lines += ["", f"{len(rows)} agents.", ""]
    (OUT / "agents.md").write_text("\n".join(lines), encoding="utf-8", newline="\n")
    return len(rows)


def local_folders():
    lines = ["# Register: local working folders the dossier cites", "",
             "Made by `tools/wsp/dossier_registers.py`. These folders are git-ignored; counts and bytes only. A reviewer",
             "rebuilds what is needed with the commands of `09_reproduce.md`.", "",
             "| Folder | What it is | Files | Bytes |", "|---|---|---|---|"]
    for rel, what in LOCAL:
        p = REPO / rel
        if not p.exists():
            lines.append(f"| `{rel}` | {what} | absent | — |")
            continue
        n = size = 0
        for f in p.rglob("*"):
            if f.is_file():
                n += 1
                try:
                    size += f.stat().st_size
                except OSError:
                    pass
        lines.append(f"| `{rel}` | {what} | {n} | {size:,} |")
    lines.append("")
    (OUT / "local_folders.md").write_text("\n".join(lines), encoding="utf-8", newline="\n")


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--transcripts", nargs="*", default=[], help="folders holding agent-*.jsonl and agent-*.meta.json")
    args = ap.parse_args()
    n = candidates()
    print("candidates:", n)
    if args.transcripts:
        print("agents:", agents(args.transcripts))
    local_folders()
    print("local folders: written")
    return 0


if __name__ == "__main__":
    sys.exit(main())
