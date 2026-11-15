"""Astra's scoped, deterministic literal-copy/assembly receipts; never selects a weight.

Only the five explicitly named immutable releases and the main author's addition.wsp
and joint edits are read. No data, feedback, partitions, optimizer or numerical search.
Run prepare after the new CLI is built; assemble creates only the new Astra releases.
Existing releases and old configuration files are never rewritten.
"""
from __future__ import annotations

import argparse
import copy
import json
from pathlib import Path
import tempfile

from binary_snapshot import add_snapshot_arguments, capture_binary, select_snapshot

ROOT = Path(__file__).resolve().parents[3]
IGNIS = ROOT / "runs/tooling/astra-target/release/ignis.exe"
EXECUTION_BINARY = None
CASES = [
    ("flex_methanol6", "flex/wsp5/methanol/carry-d3", "flex/wsp6/methanol"),
    ("flex_heptane6", "flex/wsp5/heptane/carry-d2", "flex/wsp6/heptane"),
    ("acme_normal6", "acme/wsp5/normal/p1-fable", "acme/wsp6/normal"),
    ("acme_inverse6", "acme/wsp5/inverse/p1-fable", "acme/wsp6/inverse"),
    ("bass_front3", "bass/front2/carry-d1", "bass/front3"),
]
SOURCES = [
    "repo:docs/wsp/dossier(README,01,02,03,05,07,10;train measurements and historical aggregate validation, not per-run validation)",
    "repo:docs/wsp/dossier/ASTRA_DESIGN.md",
    "repo:docs/wsp/dossier/ASTRA_ERROR_BUDGET.md(historical train reports; not independently remeasured)",
    "repo:docs/wsp/dossier/review/F-001-programmed-stop.md",
    "repo:docs/wsp/dossier/review/F-002-gen5-validation-execution.md",
    "repo:docs/wsp/dossier/review/F-003-methanol-self-cooling.md",
    "repo:docs/wsp/dossier/review/F-004-methanol-alias-exposure.md(X23 possible delegated raw-file identity ambiguity; all uncertain raw material excluded; no protected numerical value sent to author)",
    "repo:docs/wsp/dossier/review/F-005-ensemble-and-equivalence.md",
    "repo:docs/wsp/dossier/review/F-006-generalization-and-freedom.md",
    "repo:docs/wsp/dossier/review/F-007-front-event-cdf-semantics.md",
    "repo:docs/wsp/dossier/review/F-008-train-feedback-late-partition-filter.md",
    "repo:docs/wsp/dossier/review/F-009-feedback-pair-key-collision.md(static scorer audit before candidate feedback)",
    "repo:docs/wsp/research/astra_weight_space_research.md",
    "repo:docs/wsp/research/astra_uncertainty_and_freedom.md",
    "repo:docs/wsp/research/astra_physical_properties_and_transport.md(X24 class-C abstract aggregate exposure by collector, excluded from model reasoning)",
    "repo:docs/wsp/research/astra_research_analysis.md(explicit gpt-6-astra high effort)",
    "repo:docs/wsp/research/astra_physics_analysis.md(explicit gpt-6-astra high effort)",
    "repo:docs/wsp/research/astra_forward_physics_review.md",
    "repo:docs/wsp/dossier/ASTRA_PROPERTIES.md(primary references and exact reading extents)",
    "repo:docs/wsp/research/astra_data_and_input_audit.md",
    "repo:docs/wsp/feedback/astra_property_crosscheck.json(independent property-library evaluation after literals typed; no candidate number emitted or changed)",
    "repo:docs/wsp/research/properties_verified.md(reference-property cross-check; no modelled-series outcomes)",
    "repo:docs/wsp/research/phys_bass.md(existing train-informed steady-law proposal)",
    "author:pretraining-knowledge(unverifiable)",
    "author:main-thread turn_context model=gpt-6-astra effort=max; directly verified metadata only on 2026-11-14",
]


def read(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def write_same(path: Path, value):
    text = value if isinstance(value, str) else json.dumps(value, indent=2, ensure_ascii=False) + "\n"
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.exists():
        if path.read_text(encoding="utf-8") != text:
            raise RuntimeError(f"refuse to overwrite different existing artifact: {path}")
    else:
        path.write_text(text, encoding="utf-8", newline="\n")


def cli(*args: str):
    global EXECUTION_BINARY
    if EXECUTION_BINARY is None:
        EXECUTION_BINARY = capture_binary(IGNIS)
    result = EXECUTION_BINARY.run(["wsp", *map(str, args)], cwd=ROOT,
                                  capture_output=True, text=True)
    if result.returncode:
        raise RuntimeError(f"wsp {args[0]} failed ({result.returncode}): {result.stderr}")
    return result.stdout


def cli_output(command: str, destination: Path, *args: str):
    """Stage a CLI output before comparing it with an immutable candidate file."""
    with tempfile.TemporaryDirectory(prefix="ignis-astra-") as directory:
        staged = Path(directory) / destination.name
        response = cli(command, *args, "--out", staged)
        write_same(destination, read(staged))
        return response


def assemble_once(command: str, key: str, candidate: Path, binding: Path):
    release_path = candidate / "release.json"
    receipt_path = candidate / "assembly.json"
    if release_path.exists() or receipt_path.exists():
        if not (release_path.exists() and receipt_path.exists()):
            raise RuntimeError(f"incomplete existing assembly requires inspection: {candidate}")
        release, receipt = read(release_path), read(receipt_path)
        inspection = json.loads(cli("inspect", "--release", release["id"]))
        if receipt["release_id"] != release["id"] or not inspection["forward_matches"]:
            raise RuntimeError(f"existing assembly identity changed: {candidate}")
        return receipt
    receipt = json.loads(cli_output(command, release_path, "--family", key,
                                    "--patch", candidate / "patch.json", "--binding", binding))
    receipt["execution_binary"] = EXECUTION_BINARY.receipt()
    EXECUTION_BINARY.verify()
    write_same(receipt_path, receipt)
    return receipt


def exposure():
    e = {"sources": [], "datasets": [], "development_feedback": [], "prior_candidates": []}
    for _, old, _ in CASES:
        parent_dir = ROOT / "configs/models" / old
        source = read(parent_dir / "meta.json")["exposure"]
        for key in e:
            for item in source.get(key, []):
                if item not in e[key]:
                    e[key].append(item)
        rid = read(parent_dir / "release.json")["id"]
        if rid not in e["prior_candidates"]:
            e["prior_candidates"].append(rid)
    e["sources"].extend(s for s in SOURCES if s not in e["sources"])
    # The parent sources describe inherited exposure, not a claim that Astra personally
    # reopened every historical source. No uncertain raw values are supplied to this tool.
    e["sources"].append("provenance:inherited source entries above retain Fable's exposure; Astra's direct readings are the explicitly named Astra records")
    # SourceId is a whitespace-free identifier; the readable ledgers remain in the
    # referenced documents. This is syntax normalization, never exposure removal.
    e["sources"] = ["-".join(source.split()) for source in e["sources"]]
    return e


def meta(key, label, parent, exp, rationale):
    return {"patch_id": f"{key}-{label}-astra", "parent_release_id": parent,
            "author": {"provider": "openai", "model_id": "gpt-6-astra", "effort": "max"},
            "exposure": exp, "rationale": rationale, "origin": "system"}


def prepare():
    exp = exposure()
    for key, old, new in CASES:
        source = ROOT / "configs/models" / old
        dest = ROOT / "configs/models" / new
        rid = read(source / "release.json")["id"]
        # `inspect --edits` transcribes stored values exactly. It chooses no number.
        inherited = cli("inspect", "--release", rid, "--edits")
        header = (f"# DATA-INFORMED Astra R1; not zero-shot.\n"
                  f"# Exact deterministic transcription of Fable's stored release {rid}.\n"
                  f"# Source configuration: configs/models/{old}. No value is estimated.\n"
                  "# New main-authored literals follow the inherited block.\n")
        write_same(dest / "r1-astra/inherited.edits.wsp", header + inherited)
        addition = (dest / "r1-astra/addition.wsp").read_text(encoding="utf-8")
        write_same(dest / "r1-astra/edits.wsp", header + inherited + "\n" + addition)
        write_same(dest / "binding.json", read(source.parent / "binding.json"))
        spec_path = dest / "spec.json"
        exported = json.loads(cli_output("spec", spec_path, "--family", key))
        rationale = ("Data-informed new-spec full literal rewrite; parent records ancestry, not tensor inheritance. "
                     "Astra R1 is a new experiment. Every destination assignment is explicit; inherited values are "
                     "credited to Fable. Added values are typed once by the main author before candidate feedback; "
                     "see addition.wsp, ASTRA_DESIGN, ASTRA_ERROR_BUDGET and ASTRA_PROPERTIES. No fitting or search.")
        write_same(dest / "r1-astra/meta.json", meta(key, "r1", rid, copy.deepcopy(exp), rationale))
        print(json.dumps({"model": key, "parent": rid, "spec_hash": exported["spec_hash"]}))


def assemble_all():
    for key, old, new in CASES:
        dest = ROOT / "configs/models" / new
        first = dest / "r1-astra"
        cli_output("compile", first / "patch.json", "--family", key, "--edits", first / "edits.wsp", "--meta", first / "meta.json")
        cli("rewrite-check", "--family", key, "--patch", first / "patch.json", "--binding", dest / "binding.json")
        receipt = assemble_once("rewrite", key, first, dest / "binding.json")
        joint = dest / "joint-astra"
        e = read(first / "meta.json")["exposure"]
        e["prior_candidates"].append(receipt["release_id"])
        joint_rationale = ("Data-informed equal Fable/Astra distribution statement, fixed before train scores. "
                           "No validation look. The BASS joint reinterprets both speed members with the new coherent "
                           "first-event mechanism; it does not average the predecessor's incoherent event rows.")
        write_same(joint / "meta.json", meta(key, "joint", receipt["release_id"], e, joint_rationale))
        cli_output("compile", joint / "patch.json", "--family", key, "--edits", joint / "edits.wsp", "--meta", joint / "meta.json")
        j = assemble_once("assemble", key, joint, dest / "binding.json")
        counts = {name: value for name, value in receipt["counts"].items() if name != "per_tensor"}
        print(json.dumps({"model": key, "r1": receipt["release_id"], "joint": j["release_id"], "counts": counts}))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=["prepare", "assemble"])
    add_snapshot_arguments(parser)
    args = parser.parse_args()
    EXECUTION_BINARY = select_snapshot(args, IGNIS)
    print(json.dumps({"execution_binary": EXECUTION_BINARY.receipt()}), flush=True)
    {"prepare": prepare, "assemble": assemble_all}[args.action]()
    EXECUTION_BINARY.verify()
