"""Private, protected label staging; not a replacement atlas/application schema.

No semantic labelling occurs here. Author export requires a trusted host gate;
the CLI only validates or writes immutable protected artifacts. See LABELS.md.
"""
from __future__ import annotations

import argparse
from collections import Counter, defaultdict
from dataclasses import dataclass
from datetime import datetime
import hashlib
import json
import math
from pathlib import Path
import random
import re
import secrets
import subprocess


ROOT = Path(__file__).resolve().parents[3]
VERSION = "astra.private.labels/3"
LEGACY_VERSION = "astra.private.labels/2"
OWNER_SOURCE = "docs/wsp/dossier/ASTRA_PROMPT_ALL_NASA.md"
PREVIOUS_CONSOLIDATED_COMMIT = "3577073e7112a5f2ecebb3d0e5951d2ea9e4a5fa"
CURRENT_AUTHORITY_COMMIT = "76fbb19244fb148ef94eef508896df5b81825c75"
CONSOLIDATED_REVISIONS = {
    PREVIOUS_CONSOLIDATED_COMMIT: 1,
    "1f0eef6ffb4408775d955bba5d9bd85c273fac86": 2,
    "ccb0432eee7f45dd1127157f6767422c6dd97c41": 3,
    "0427549a8bf36c918176cb5fc36d6b6fb57219c9": 4,
    CURRENT_AUTHORITY_COMMIT: 5,
}
SCHEMA_SOURCE = "tools/wsp/atlas/atlas_schema.py"
SCHEMA_PROSE = "docs/wsp/atlas/SCHEMA.md"
CURRENT_METADATA_HASHES = {
    SCHEMA_SOURCE: "42574895346e09fff2769b07c95d40be8293d4cc949d0935f691a7c48248be56",
    SCHEMA_PROSE: "b63b28e9763d1ad92892fa7424c093cbeae725b97571092c092a05eac1aeb1fc",
    OWNER_SOURCE: "e07606a85b65f4a2c12e3412f1281a1fe8df2b1a3479cf667f4f9799f35ca1fb",
}
PAGE_KINDS = ("methods", "apparatus", "requirements", "results table", "figure", "discussion", "drawing")
# These relations are precisely the owner-requested section-2b edges. English
# strings retain the instruction's meanings without inventing a graph ontology.
LEGACY_EDGE_RELATIONS = (
    "test to investigation", "test to file", "test to page", "page to quantity",
    "investigation to hardware", "investigation to fire-safety need",
    "investigation to the crew atmospheres its tests cover",
    "model parameter to the source that justifies it",
    "model to the tests it was scored on, with the score",
    "a table value to the sensor value that contradicts it",
)
EDGE_RELATIONS = LEGACY_EDGE_RELATIONS + ("table value contradicted by sensor value",)
UNSUPPORTED = frozenset(("observed.phenomenon", "document.topic",
                         "flow.regime", "material.class", "physical.observable"))


def canonical(value):
    return json.dumps(value, ensure_ascii=False, sort_keys=True,
                      separators=(",", ":"), allow_nan=False).encode("utf-8")


def digest(value):
    return hashlib.sha256(value).hexdigest()


def _require(condition, message):
    if not condition:
        raise ValueError(message)


def _keys(value, expected):
    _require(isinstance(value, dict) and set(value) == set(expected), "unexpected fields")


def _text(value):
    _require(isinstance(value, str) and bool(value.strip()), "missing text")


def _hash(value):
    _require(isinstance(value, str) and re.fullmatch("[0-9a-f]{64}", value), "invalid hash")


def _strings(value):
    _require(isinstance(value, list), "expected list")
    for entry in value:
        _text(entry)
    _require(len(set(value)) == len(value), "duplicate entries")


def load_json(path):
    def unique(pairs):
        result = {}
        for key, value in pairs:
            _require(key not in result, "duplicate JSON key")
            result[key] = value
        return result
    def bad_constant(_):
        raise ValueError("nonfinite JSON number")
    result = json.loads(Path(path).read_bytes(), object_pairs_hook=unique,
                        parse_constant=bad_constant)
    canonical(result)  # Also rejects overflowing numeric exponents.
    return result


@dataclass(frozen=True)
class Definitions:
    schema_version: str
    schema_sha256: str
    vocabulary_sha256: str
    vocabularies: dict
    rule_ids: tuple
    owner_authority: dict
    envelope_version: str
    replay_commit: dict | None
    edge_relations: tuple


def git_authority_bytes(commit, path, newline="lf"):
    """Read one of three fixed metadata paths at an explicit immutable Git commit."""
    _require(isinstance(commit, str) and re.fullmatch(r"[0-9a-f]{40}", commit), "full Git commit required")
    _require(path in (OWNER_SOURCE, SCHEMA_SOURCE, SCHEMA_PROSE), "not an authority metadata path")
    _require(newline in ("lf", "crlf"), "explicit LF or CRLF representation required")
    result = subprocess.run(["git", "show", f"{commit}:{path}"], cwd=ROOT, capture_output=True, check=False)
    _require(result.returncode == 0, "pinned Git authority unavailable")
    raw = result.stdout
    if newline == "crlf":
        raw = raw.replace(b"\r\n", b"\n").replace(b"\n", b"\r\n")
    return raw


def _section(text, heading, following):
    _require(text.count(heading) == 1 and text.count(following) == 1, "authority heading changed; versioned migration required")
    section = text.split(heading, 1)[1]
    _require(following in section, "authority section order changed")
    return section.split(following, 1)[0]


def _definitions_from_bytes(schema_raw, prose, owner_raw, legacy=False, replay_commit=None, authority_revision=5):
    """Only call after the fixed metadata bytes have passed independent pin checks."""
    namespace = {}
    exec(compile(schema_raw, SCHEMA_SOURCE, "exec"), namespace)
    names = {
        "hardware.rack": "RACK", "hardware.insert": "INSERT", "fuel.phase": "PHASE",
        "geometry.configuration": "CONFIGURATION", "geometry.flame_orientation": "ORIENTATION",
        "flow.direction": "DIRECTION", "gravity": "GRAVITY", "labels.flame_type": "FLAME_TYPE",
        "labels.studied": "STUDIED", "labels.fire_safety_need": "FIRE_SAFETY_NEED",
        "map.crew_atmospheres": "CREW_ATMOSPHERE", "measurements.items.kind": "MEASUREMENT_KIND",
    }
    vocab = {key: list(namespace[name]) for key, name in names.items()}
    text = owner_raw.decode("utf-8")
    if legacy:
        section = _section(text, "## 2b.", "## 3.")
        owner_authority = {"source": OWNER_SOURCE, "section": "2b", "version": "astra.owner.section2b/1",
                           "sha256": digest(owner_raw), "section_sha256": digest(section.encode("utf-8"))}
    else:
        pages = _section(text, "### 2. Page cards and search enrichment: a cheap model reads every page", "### 3.")
        _require(authority_revision in (1, 2, 3, 4, 5), "unsupported consolidated authority revision")
        links = _section(text, "### 4. Test labels, clusters, links, findings",
                         "### 4b." if authority_revision >= 2 else "### 5.")
        section = pages
        authority_version = (f"astra.owner.whole-goal/{authority_revision - 3}" if authority_revision >= 4
                             else f"astra.owner.consolidated/{authority_revision}")
        owner_authority = {"source": OWNER_SOURCE, "sections": ["2", "4"], "version": authority_version,
                           "sha256": digest(owner_raw), "section_sha256": {
                               "2": digest(pages.encode("utf-8")), "4": digest(links.encode("utf-8"))}}
        if authority_revision >= 2:
            discrepancies = _section(text, "### 4b. Cross-check NASA's tables against NASA's own sensor files", "### 5.")
            _require("a script does it, not a model" in discrepancies and
                     "table value contradicted by sensor value" in discrepancies,
                     "discrepancy authority changed; versioned migration required")
            owner_authority["sections"].append("4b")
            owner_authority["section_sha256"]["4b"] = digest(discrepancies.encode("utf-8"))
            if authority_revision >= 3:
                _require("Then decide the cause" in discrepancies and "Verdicts:" in discrepancies,
                         "discrepancy cause/verdict requirements changed")
        if authority_revision >= 4:
            additional = {
                "roles": _section(text, "## Which model does what", "## How research and findings work"),
                "research_findings": _section(text, "## How research and findings work", "## Where things stand"),
                "rules": _section(text, "## Rules", "## What to do, in this order"),
                "9": _section(text, "### 9. Hand over what the application shows", "## The first instruction, what still stands"),
            }
            _require("Conditions of consecutive tests" in additional["rules"] and
                     "not differenced" in additional["rules"] and
                     "Before a finding is filed, a refuter" in additional["research_findings"] and
                     "not by an author" in additional["9"], "whole-goal role/export safeguards changed")
            for name, content in additional.items():
                owner_authority["sections"].append(name)
                owner_authority["section_sha256"][name] = digest(content.encode("utf-8"))
        if authority_revision == 5:
            _require("whole document" in pages and "start at 24" in pages and
                     "Qualify two efforts on the same 200 pages" in pages,
                     "whole-document/two-effort qualification requirements changed")
        _require("same pass writes the search enrichment" in pages and "200" in pages and
                 "nine pages in ten" in pages and "strong agent or a conservative rule" in pages,
                 "authority requirements changed; versioned migration required")
    _require(all(value in section for value in PAGE_KINDS), "owner page-kind instruction changed")
    vocab["page.kind"] = list(PAGE_KINDS)
    edge_relations = LEGACY_EDGE_RELATIONS if legacy or authority_revision == 1 else EDGE_RELATIONS
    vocab["edge.relation"] = list(edge_relations)
    # Both executable lists and prose rules are identity-bound; no copied enums.
    schema_hash = digest(schema_raw)
    rule_section = _section(prose.decode("utf-8"), "## 5. Rules", "## 6.")
    rule_ids = tuple(re.findall(r"(?m)^\| `([^`]+)` \|", rule_section))
    _require(bool(rule_ids), "authoritative rules unavailable")
    vocab_hash = digest(canonical({"lists": vocab, "schema_sha256": schema_hash,
                                  "prose_sha256": digest(prose), "owner_authority": owner_authority,
                                  "edge_relations": edge_relations}))
    return Definitions(namespace["SCHEMA_ID"], schema_hash, vocab_hash, vocab, rule_ids, owner_authority,
                       LEGACY_VERSION if legacy else VERSION, replay_commit, edge_relations)


def definition_pins(defs):
    return {"schema_sha256": defs.schema_sha256, "vocabulary_sha256": defs.vocabulary_sha256,
            "owner_authority": defs.owner_authority}


def definitions(*, legacy_commit=None, expected_pins=None, newline="lf", schema_commit=None, authority_commit=None):
    """Immutable reviewed authority snapshot or explicit hash-verified replay.

    Legacy byte representation is explicit. Neither record nor tool hashes are
    rewritten; replay_commit records the matched Git identity separately.
    """
    if legacy_commit is not None or authority_commit is not None:
        _require(not (legacy_commit and authority_commit), "choose one replay authority")
        if authority_commit is not None:
            _require(authority_commit in CONSOLIDATED_REVISIONS,
                     "unrecognized consolidated authority revision")
        source_ref = legacy_commit or authority_commit
        _require(isinstance(expected_pins, dict), "legacy replay requires independently sealed expected pins")
        _keys(expected_pins, ("schema_sha256", "vocabulary_sha256", "owner_authority"))
        schema_ref = schema_commit or source_ref
        schema_raw = git_authority_bytes(schema_ref, SCHEMA_SOURCE, newline)
        owner_raw = git_authority_bytes(source_ref, OWNER_SOURCE, newline)
        prose = git_authority_bytes(schema_ref, SCHEMA_PROSE, newline)
        _require(digest(schema_raw) == expected_pins["schema_sha256"]
                 and digest(owner_raw) == expected_pins["owner_authority"]["sha256"], "legacy authority pin mismatch")
        result = _definitions_from_bytes(schema_raw, prose, owner_raw, legacy_commit is not None,
                                         {"owner_commit": source_ref, "schema_commit": schema_ref, "newline": newline},
                                         authority_revision=CONSOLIDATED_REVISIONS.get(authority_commit, 5))
        _require(definition_pins(result) == expected_pins, "legacy vocabulary or section pin mismatch")
        return result
    _require(expected_pins is None and newline == "lf" and schema_commit is None,
             "legacy pins/representation require explicit commit")
    metadata = {}
    for path in (SCHEMA_SOURCE, SCHEMA_PROSE, OWNER_SOURCE):
        # Qualification consumes this immutable reviewed snapshot. Later worktree
        # or HEAD edits cannot mutate its definitions or interrupt a sealed run.
        raw = git_authority_bytes(CURRENT_AUTHORITY_COMMIT, path)
        _require(digest(raw) == CURRENT_METADATA_HASHES[path],
                 "pinned authority/schema drift; versioned migration required")
        metadata[path] = raw
    return _definitions_from_bytes(metadata[SCHEMA_SOURCE], metadata[SCHEMA_PROSE], metadata[OWNER_SOURCE],
                                    replay_commit={"owner_commit": CURRENT_AUTHORITY_COMMIT,
                                                   "schema_commit": CURRENT_AUTHORITY_COMMIT, "newline": "lf"})


def validate_locator(locator):
    _keys(locator, ("source_id", "source_sha256", "page", "selector"))
    _text(locator["source_id"])
    _hash(locator["source_sha256"])
    _require(locator["page"] is None or type(locator["page"]) is int and locator["page"] > 0,
             "invalid page")
    selector = locator["selector"]
    _require(isinstance(selector, dict), "missing exact selector")
    kind = selector.get("kind")
    if kind == "text":
        _keys(selector, ("kind", "start", "end", "quote", "extraction_sha256"))
        _require(type(selector["start"]) is int and type(selector["end"]) is int
                 and 0 <= selector["start"] < selector["end"], "invalid text span")
        _text(selector["quote"])
        _hash(selector["extraction_sha256"])
    elif kind == "cell":
        _keys(selector, ("kind", "sheet", "row", "column"))
        _text(selector["sheet"])
        _text(selector["column"])
        _require(type(selector["row"]) is int and selector["row"] > 0, "invalid cell")
    elif kind == "figure":
        _keys(selector, ("kind", "figure_id", "bbox", "render_sha256"))
        _text(selector["figure_id"])
        _hash(selector["render_sha256"])
        box = selector["bbox"]
        _require(isinstance(box, list) and len(box) == 4 and all(type(n) in (int, float)
                 and math.isfinite(n) and 0 <= n <= 1 for n in box), "invalid normalized bbox")
        _require(box[0] < box[2] and box[1] < box[3], "empty figure region")
    elif kind == "field":
        _keys(selector, ("kind", "pointer"))
        _text(selector["pointer"])
        _require(selector["pointer"].startswith("/"), "expected JSON pointer")
    elif kind == "whole_page":
        _keys(selector, ("kind", "reason", "render_sha256"))
        _require(locator["page"] is not None, "page required")
        _require(selector["reason"] in ("blank", "unreadable", "unclassified"), "invalid page failure")
        _hash(selector["render_sha256"])
    else:
        raise ValueError("unsupported selector")


def validate_label(label, defs):
    _keys(label, ("kind", "state", "value", "reason", "basis", "rule", "locators"))
    kind = label["kind"]
    _require(kind in defs.vocabularies or kind in UNSUPPORTED, "unknown label kind")
    _require(label["basis"] in ("observed", "derived", "interpretation"), "invalid assertion basis")
    _require(label["state"] in ("assigned", "absent", "unresolved"), "invalid label state")
    _require(isinstance(label["locators"], list) and label["locators"], "label needs exact evidence")
    for locator in label["locators"]:
        validate_locator(locator)
    if label["state"] == "unresolved":
        _require(label["value"] is None, "unresolved value must stay null")
        _text(label["reason"])
        _require(label["rule"] is None, "unresolved label cannot claim derivation")
        return
    _require(kind not in UNSUPPORTED, "vocabulary extension required")
    _strings(label["value"])
    _require(set(label["value"]) <= set(defs.vocabularies[kind]), "unapproved vocabulary value")
    _require(bool(label["value"]) == (label["state"] == "assigned"), "absence must be empty set")
    _require(label["reason"] is None or isinstance(label["reason"], str), "invalid reason")
    expected_rule = {"labels.flame_type": "L1", "labels.studied": "L2",
                     "labels.fire_safety_need": "L3", "map.crew_atmospheres": "CREW"}.get(kind)
    if expected_rule:
        _require(label["basis"] == "derived" and label["rule"] == expected_rule,
                 "atlas topic/geometry/keyword labels are rule-derived, not observed outcomes")
    elif label["basis"] == "derived":
        _require(label["rule"] in defs.rule_ids, "unknown atlas derivation rule")
    else:
        _require(label["rule"] is None, "non-derived assertion cannot name a derivation rule")
    _require(all(loc["selector"]["kind"] != "whole_page" for loc in label["locators"]),
             "whole-page failure locator cannot support an assigned label")


def content_hash(record):
    return digest(canonical({k: v for k, v in record.items() if k != "content_sha256"}))


def seal(record):
    """Hash an envelope; this is integrity, never assignment or authorization."""
    result = dict(record)
    result["content_sha256"] = content_hash(result)
    return result


def validate_enrichment(enrichment, producer=None):
    _keys(enrichment, ("status", "reason", "provenance", "grounded_terms", "retrieval_cues"))
    provenance = enrichment["provenance"]
    _keys(provenance, ("method", "assignment_id", "prompt_sha256", "tool_sha256", "config_sha256"))
    _require(provenance["method"] == "sira-inspired-page-enrichment/1", "unsupported enrichment method")
    _text(provenance["assignment_id"])
    for key in ("prompt_sha256", "tool_sha256", "config_sha256"):
        _hash(provenance[key])
    if producer is not None:
        _require(all(provenance[key] == producer[key] for key in
                     ("assignment_id", "prompt_sha256", "tool_sha256", "config_sha256")),
                 "enrichment must be attributed to the same page-reading assignment")
    _require(enrichment["status"] in ("filled", "unresolved", "not_applicable"), "invalid enrichment status")
    _require(isinstance(enrichment["grounded_terms"], list) and isinstance(enrichment["retrieval_cues"], list),
             "invalid enrichment lists")
    if enrichment["status"] != "filled":
        _text(enrichment["reason"])
        _require(not enrichment["grounded_terms"] and not enrichment["retrieval_cues"],
                 "unresolved enrichment cannot imply generated content")
    else:
        _require(enrichment["reason"] is None, "filled enrichment cannot have missingness reason")
    ids, locators = set(), []
    for term in enrichment["grounded_terms"]:
        _keys(term, ("id", "kind", "text", "locators"))
        _text(term["id"])
        _text(term["text"])
        _require(term["id"] not in ids, "duplicate grounded term")
        ids.add(term["id"])
        _require(term["kind"] in ("concept", "synonym", "abbreviation", "instrument", "material", "test_id"),
                 "unknown enrichment source-term role")
        _require(isinstance(term["locators"], list) and term["locators"], "grounded term needs evidence")
        for locator in term["locators"]:
            validate_locator(locator)
            _require(locator["selector"]["kind"] != "whole_page", "grounded terms need exact evidence")
            locators.append(locator)
    for cue in enrichment["retrieval_cues"]:
        _keys(cue, ("kind", "text", "grounding_ids", "new_terms"))
        _text(cue["text"])
        _strings(cue["grounding_ids"])
        _strings(cue["new_terms"])
        _require(cue["new_terms"], "SIRA-inspired cue must identify new retrieval vocabulary")
        _require(cue["grounding_ids"] and set(cue["grounding_ids"]) <= ids, "generated cue lacks source grounding")
        _require(cue["kind"] in ("search_phrasing", "synonym_expansion", "abbreviation_expansion", "concept_expansion"),
                 "unknown retrieval-cue role")
    return locators


def validate_enrichment_novelty(enrichment, page_text):
    """Host uses already supplied text; this never opens or rereads source files."""
    validate_enrichment(enrichment)
    _require(isinstance(page_text, str), "page text required for novelty check")
    text = page_text.casefold()
    for cue in enrichment["retrieval_cues"]:
        for term in cue["new_terms"]:
            pattern = r"(?<!\w)" + re.escape(term.casefold()) + r"(?!\w)"
            _require(re.search(pattern, text) is None, "retrieval term is already present in source")
            _require(re.search(pattern, cue["text"].casefold()) is not None, "new retrieval term missing from cue")


def validate_page_review(review, record, runtime_receipts):
    _keys(review, ("confidence", "figure_or_table", "label_disagreement", "requires_strong_review", "outcome_decision"))
    _require(review["confidence"] in ("low", "medium", "high", "unresolved"), "invalid review confidence")
    for field in ("figure_or_table", "label_disagreement", "requires_strong_review"):
        _require(type(review[field]) is bool, "review decisions must be explicit booleans")
    page_kinds = {value for label in record["labels"] if label["kind"] == "page.kind"
                  for value in (label["value"] or [])}
    _require(not page_kinds.intersection(("figure", "results table")) or review["figure_or_table"],
             "figure/table review flag contradicts page label")
    needed = review["label_disagreement"] or (review["figure_or_table"] and review["confidence"] in ("low", "unresolved"))
    _require(not needed or review["requires_strong_review"], "strong review required")
    decision = review["outcome_decision"]
    _keys(decision, ("method", "assignment_id", "rule"))
    if decision["method"] == "conservative":
        _require(decision["assignment_id"] is None and decision["rule"] in
                 ("terra_non_authorizing/1", "uncertain_outcome_is_protected/1"), "invalid conservative protection rule")
        _require(record["body"]["reveals_outcome"] is True, "conservative outcome protection must fail closed")
    elif decision["method"] == "strong":
        _require(decision["rule"] is None and decision["assignment_id"] in runtime_receipts,
                 "strong outcome decision needs captured runtime receipt")
        receipt = runtime_receipts[decision["assignment_id"]]
        _validate_runtime_receipt(receipt, decision["assignment_id"])
        _require(receipt.get("kind") == "model" and "terra" not in str(receipt.get("model_id", "")).lower(),
                 "Terra cannot authorize outcome-bearing status")
    else:
        raise ValueError("outcome decision requires strong agent or conservative rule")


def validate_record(record, defs, runtime_receipts):
    _keys(record, ("envelope_version", "origin", "record_id", "kind", "schema_version",
                   "schema_sha256", "vocabulary_sha256", "content_sha256", "producer",
                   "owner_authority", "created_at", "locators", "dependencies", "test_dependencies", "labels", "body"))
    _require(record["envelope_version"] == defs.envelope_version, "envelope/authority version mismatch")
    _require(record["origin"] in ("synthetic_fixture", "source_evidence"), "invalid origin")
    _text(record["record_id"])
    _require(record["schema_version"] == defs.schema_version
             and record["schema_sha256"] == defs.schema_sha256
             and record["vocabulary_sha256"] == defs.vocabulary_sha256, "definition identity mismatch")
    _require(record["owner_authority"] == defs.owner_authority, "owner authority identity mismatch")
    _require(content_hash(record) == record["content_sha256"], "content hash mismatch")
    _require(isinstance(record["created_at"], str) and record["created_at"].endswith("Z"), "UTC timestamp required")
    datetime.fromisoformat(record["created_at"].replace("Z", "+00:00"))
    producer = record["producer"]
    _keys(producer, ("kind", "id", "model_id", "model_version", "version_limitation",
                     "assignment_id", "runtime_session_id", "prompt_sha256", "tool_sha256", "config_sha256"))
    for key in ("id", "assignment_id", "runtime_session_id"):
        _text(producer[key])
    for key in ("prompt_sha256", "tool_sha256", "config_sha256"):
        _hash(producer[key])
    _require(producer["kind"] in ("model", "rule"), "invalid producer")
    if producer["kind"] == "model":
        _text(producer["model_id"])
        _require(producer["model_id"] not in ("Astra", "Fable"), "friendly name is not runtime identity")
        if producer["model_version"] is None:
            _text(producer["version_limitation"])
        else:
            _text(producer["model_version"])
        _require(runtime_receipts.get(producer["assignment_id"]) == producer,
                 "assignment lacks matching independently captured runtime receipt")
    else:
        _require(producer["model_id"] is None and producer["model_version"] is None
                 and producer["version_limitation"] is None, "rule output must not impersonate model")
        _require(record["kind"] == "cluster" or (record["kind"] == "edge" and
                 record["body"].get("relation") == "table value contradicted by sensor value"),
                 "semantic labels require an actual assigning model")
        _require(record["labels"] == [], "rule output cannot invent semantic labels")
    _strings(record["dependencies"])
    _strings(record["test_dependencies"])
    _require(isinstance(record["locators"], list) and record["locators"], "record needs evidence")
    _require(isinstance(record["labels"], list), "invalid labels")
    _require(len({x["kind"] for x in record["labels"]}) == len(record["labels"]), "duplicate label kinds")
    for label in record["labels"]:
        validate_label(label, defs)
    locators = record["locators"] + [loc for label in record["labels"] for loc in label["locators"]]
    for locator in locators:
        validate_locator(locator)
        _require(locator["source_id"] in record["dependencies"], "evidence omitted from dependencies")
    body = record["body"]
    kind = record["kind"]
    if kind == "page":
        expected_body = ["page_id", "document_sha256", "page_number", "reveals_outcome",
                     "revealed_test_ids", "mentioned_test_ids", "unresolved_test_mentions",
                     "investigation_ids", "quantity_evidence", "condition_evidence"]
        if defs.envelope_version == VERSION:
            expected_body += ["search_enrichment", "review"]
        _keys(body, expected_body)
        if defs.envelope_version == VERSION:
            for locator in validate_enrichment(body["search_enrichment"], record["producer"]):
                _require(locator["source_id"] in record["dependencies"], "enrichment dependency omitted")
            validate_page_review(body["review"], record, runtime_receipts)
        _text(body["page_id"])
        _hash(body["document_sha256"])
        _require(type(body["page_number"]) is int and body["page_number"] > 0, "invalid page")
        _require(body["reveals_outcome"] is None or type(body["reveals_outcome"]) is bool, "invalid reveal state")
        for key in ("revealed_test_ids", "mentioned_test_ids", "unresolved_test_mentions"):
            _strings(body[key])
        _strings(body["investigation_ids"])
        _require(all(re.fullmatch(r"PSI-\d+", value) for value in body["investigation_ids"]), "invalid investigation ID")
        _require(any(label["kind"] == "page.kind" for label in record["labels"]), "page kind must be explicit")
        for evidence in body["quantity_evidence"]:
            _keys(evidence, ("raw_name", "raw_unit", "unit_reason", "locator"))
            _text(evidence["raw_name"])
            if evidence["raw_unit"] is None:
                _text(evidence["unit_reason"])
            else:
                _text(evidence["raw_unit"])
                _require(evidence["unit_reason"] is None, "resolved raw unit cannot have missingness reason")
        for evidence in body["condition_evidence"]:
            _keys(evidence, ("raw_text", "locator"))
            _text(evidence["raw_text"])
        for evidence in body["quantity_evidence"] + body["condition_evidence"]:
            validate_locator(evidence["locator"])
            _require(evidence["locator"]["source_id"] in record["dependencies"], "page evidence dependency omitted")
        _require(set(body["revealed_test_ids"] + body["mentioned_test_ids"]) <= set(record["test_dependencies"]),
                 "page test dependency omitted")
        _require(body["reveals_outcome"] is not False or not body["revealed_test_ids"], "inconsistent reveal state")
        _require(any(loc["page"] == body["page_number"] and loc["source_sha256"] == body["document_sha256"]
                     for loc in record["locators"]), "page locator mismatch")
    elif kind == "test_label":
        _keys(body, ("test_id",))
        _require(body["test_id"] in record["test_dependencies"], "test dependency omitted")
    elif kind == "edge":
        _keys(body, ("source", "target", "relation", "state", "reason"))
        _require(body["source"] in record["dependencies"] and body["target"] in record["dependencies"],
                 "edge endpoints omitted from dependencies")
        _require(body["state"] in ("assigned", "unresolved"), "invalid edge state")
        if body["state"] == "unresolved":
            _require(body["relation"] is None, "unresolved relation must be null")
            _text(body["reason"])
        else:
            _require(body["relation"] in defs.edge_relations, "relation not requested in pinned owner authority")
            _require(body["reason"] is None, "assigned edge cannot have unresolved reason")
    elif kind == "finding":
        _keys(body, ("assertion_kind", "subject", "claim", "domain", "disagreements", "unknowns"))
        _require(body["assertion_kind"] == "interpretation", "findings are interpretation")
        for key in ("subject", "claim", "domain"):
            _text(body[key])
        for key in ("disagreements", "unknowns"):
            _strings(body[key])
    elif kind == "cluster":
        _keys(body, ("rule", "input_ids", "member_ids", "unresolved_ids"))
        validate_rule(body["rule"], defs)
        for key in ("input_ids", "member_ids", "unresolved_ids"):
            _strings(body[key])
        _require(set(body["input_ids"]) <= set(record["dependencies"]), "cluster must depend on entire input universe")
        _require(set(body["member_ids"] + body["unresolved_ids"]) <= set(body["input_ids"]), "unknown cluster member")
        _require(not set(body["member_ids"]) & set(body["unresolved_ids"]), "unresolved member")
    else:
        raise ValueError("unsupported private record kind")


def validate_bundle(records, sources, defs, receipts):
    """Sources: opaque ID -> content digest. No paths or grants are accepted."""
    _require(isinstance(sources, dict), "invalid source inventory")
    for key, value in sources.items():
        _text(key)
        _hash(value)
    by_id = {}
    for record in records:
        validate_record(record, defs, receipts)
        key = record["record_id"]
        _require(key not in by_id and key not in sources, "duplicate identity")
        by_id[key] = record
    identities = {**sources, **{key: r["content_sha256"] for key, r in by_id.items()}}
    for record in records:
        all_locators = record["locators"] + [x for label in record["labels"] for x in label["locators"]]
        if record["kind"] == "page":
            all_locators += [entry["locator"] for name in ("quantity_evidence", "condition_evidence")
                             for entry in record["body"][name]]
            if defs.envelope_version == VERSION:
                all_locators += validate_enrichment(record["body"]["search_enrichment"])
        for loc in all_locators:
            _require(identities.get(loc["source_id"]) == loc["source_sha256"], "evidence identity mismatch")
        dependency_closure(record["record_id"], by_id, sources)
        if record["kind"] == "cluster":
            _require(all(key in by_id for key in record["body"]["input_ids"]), "cluster inputs must be label records")
            expected = cluster_members(record["body"]["rule"],
                                       [by_id[key] for key in record["body"]["input_ids"]], defs)
            _require(expected == {key: record["body"][key] for key in expected}, "cluster is not reproducible")
    pages = [(r["body"]["document_sha256"], r["body"]["page_number"]) for r in records if r["kind"] == "page"]
    _require(len(set(pages)) == len(pages), "duplicate page card")
    memberships = defaultdict(list)
    for record in records:
        if record["kind"] == "cluster":
            rule = record["body"]["rule"]
            for member in record["body"]["member_ids"]:
                memberships[(rule["kind"], member)].append(rule["overlap"])
    _require(all(len(flags) == 1 or all(flags) for flags in memberships.values()), "cluster overlap forbidden")
    return by_id


def dependency_closure(key, records, sources):
    resolved, active = set(), set()
    def visit(current):
        _require(current not in active, "dependency cycle")
        if current in resolved:
            return
        _require(current in records or current in sources, "dangling dependency")
        active.add(current)
        if current in records:
            for dep in records[current]["dependencies"]:
                visit(dep)
        active.remove(current)
        resolved.add(current)
    visit(key)
    return sorted(resolved)


def dependency_manifest(key, records, sources):
    closure = dependency_closure(key, records, sources)
    return {"records": {name: records[name]["content_sha256"] for name in closure if name in records},
            "sources": {name: sources[name] for name in closure if name in sources},
            "tests": sorted({test for name in closure if name in records
                             for test in records[name]["test_dependencies"]})}


def author_projection(records, sources, defs, receipts, gate):
    """Host-only: receipt-bound closure is checked BEFORE any aggregate is made.

    Gate must bind complete record bytes independently, including its dependency
    metadata. AuthorGate.live currently denies all live outcomes. This function
    does not create permissions and CLI exposes no gate/partition argument.
    """
    by_id = validate_bundle(records, sources, defs, receipts)
    visible = []
    for record in records:
        closure = dependency_closure(record["record_id"], by_id, sources)
        safe = True
        for key in closure:
            if key not in by_id:
                safe = safe and gate.decide(key).state == "allowed"
                continue
            candidate = by_id[key]
            if candidate["kind"] == "page":
                body = candidate["body"]
                if body["reveals_outcome"] is None or body["unresolved_test_mentions"]:
                    safe = False
                if "review" in body and (body["review"]["requires_strong_review"] or
                                         body["review"]["outcome_decision"]["method"] == "conservative"):
                    safe = False
            if gate.read_author(key, lambda _: canonical(candidate)).decision.state != "allowed":
                safe = False
        if safe:
            visible.append(record)
    # No counts or denied identities are returned; callers aggregate this list only.
    return visible


def validate_rule(rule, defs):
    _keys(rule, ("id", "version", "schema_version", "vocabulary_sha256", "kind", "operator",
                 "values", "missing", "overlap", "unit_conversion"))
    _text(rule["id"])
    _text(rule["version"])
    _require(rule["schema_version"] == defs.schema_version and rule["vocabulary_sha256"] == defs.vocabulary_sha256,
             "rule definitions changed")
    _require(rule["kind"] in defs.vocabularies, "cluster vocabulary approval required")
    _require(rule["operator"] in ("set_equals", "contains_any"), "only deterministic token predicates supported")
    _strings(rule["values"])
    _require(set(rule["values"]) <= set(defs.vocabularies[rule["kind"]]), "unknown rule value")
    _require(rule["missing"] == "unresolved" and type(rule["overlap"]) is bool
             and rule["unit_conversion"] is None, "unsupported missingness/unit rule")


def cluster_members(rule, records, defs):
    validate_rule(rule, defs)
    members, unresolved = [], []
    for record in sorted(records, key=lambda r: r["record_id"]):
        labels = [label for label in record["labels"] if label["kind"] == rule["kind"]]
        if not labels or labels[0]["state"] == "unresolved":
            unresolved.append(record["record_id"])
            continue
        label = labels[0]
        validate_label(label, defs)
        values, expected = set(label["value"]), set(rule["values"])
        if values == expected if rule["operator"] == "set_equals" else bool(values & expected):
            members.append(record["record_id"])
    return {"member_ids": members, "unresolved_ids": unresolved}


def freeze_universe(units, defs):
    """Independent census, not derived from positive labels; missing labels included."""
    seen, source_identity = set(), {}
    for unit in units:
        _keys(unit, ("unit_id", "kind", "source_id", "source_sha256", "dependency_group"))
        for key in ("unit_id", "source_id", "dependency_group"):
            _text(unit[key])
        _hash(unit["source_sha256"])
        _require(unit["kind"] in defs.vocabularies or unit["kind"] in UNSUPPORTED, "unknown audit kind")
        key = (unit["unit_id"], unit["kind"])
        _require(key not in seen, "duplicate audit unit")
        seen.add(key)
        identity = (unit["source_id"], unit["source_sha256"], unit["dependency_group"])
        _require(source_identity.setdefault(unit["unit_id"], identity) == identity,
                 "one audit unit refers to different sources across kinds")
    ordered = sorted(units, key=lambda unit: (unit["kind"], unit["unit_id"]))
    body = {"version": "astra.private.label-universe/1", "schema_sha256": defs.schema_sha256,
            "vocabulary_sha256": defs.vocabulary_sha256, "units": ordered}
    return {**body, "sha256": digest(canonical(body))}


def sample_audit(universe, defs, seed=None):
    _require(freeze_universe(universe["units"], defs) == universe, "universe changed")
    if seed is None:
        seed = secrets.token_hex(32)
    _text(seed)
    groups = defaultdict(list)
    for unit in universe["units"]:
        groups[unit["kind"]].append(unit)
    selected, summary = [], {}
    for kind, units in sorted(groups.items()):
        n = min(len(units), max(100, (len(units) + 19) // 20))
        rng = random.Random(int(digest(canonical([seed, kind, universe["sha256"]])), 16))
        selected.extend(rng.sample(units, n))
        summary[kind] = {"population": len(units), "sample_size": n,
                         "inclusion_probability": n / len(units), "below_minimum_population": len(units) < 100}
    body = {"version": "astra.private.label-audit/1", "universe_sha256": universe["sha256"],
            "seed": seed, "selected": selected, "per_kind": summary}
    return {**body, "sha256": digest(canonical(body))}


def qualification_audit(universe, defs, effort=None):
    """The owner-prescribed 200-page Terra/strong qualification, not a 1/20 audit."""
    _require(freeze_universe(universe["units"], defs) == universe, "universe changed")
    groups = defaultdict(set)
    for unit in universe["units"]:
        groups[unit["kind"]].add(unit["unit_id"])
    pages = {unit["unit_id"] for unit in universe["units"]}
    _require(len(pages) == 200 and groups and all(values == pages for values in groups.values()),
             "qualification requires the same frozen 200 pages for every kind, including missing labels")
    dual_effort = defs.owner_authority["version"] == "astra.owner.whole-goal/2"
    _require(effort in ("high", "xhigh") if dual_effort else effort is None,
             "qualification requires explicit separately frozen high or xhigh effort")
    body = {"version": "astra.private.page-qualification/2" if dual_effort else "astra.private.page-qualification/1",
            "universe_sha256": universe["sha256"],
            "seed": "all-200-no-subsampling", "selected": universe["units"],
            "per_kind": {kind: {"population": 200, "sample_size": 200, "inclusion_probability": 1.0,
                                "below_minimum_population": False} for kind in sorted(groups)}}
    if dual_effort:
        body["reasoning_effort"] = effort
    return {**body, "sha256": digest(canonical(body))}


def qualification_compatibility(first_defs, second_defs, kinds):
    """Explicit metadata bridge: no rewriting legacy labels, receipts or tool pins."""
    _require(first_defs.schema_sha256 == second_defs.schema_sha256, "qualification schema meaning changed")
    for kind in kinds:
        _require(kind in first_defs.vocabularies and kind in second_defs.vocabularies
                 and first_defs.vocabularies[kind] == second_defs.vocabularies[kind],
                 "qualification requires identical closed kind definitions")
    return {"first": definition_pins(first_defs), "second": definition_pins(second_defs),
            "first_replay": first_defs.replay_commit, "kinds": sorted(kinds),
            "scope": "closed_labels_only_no_enrichment_equivalence"}


def validate_audit(audit, universe, defs):
    expected = (qualification_audit(universe, defs, audit.get("reasoning_effort"))
                if audit.get("version", "").startswith("astra.private.page-qualification/")
                else sample_audit(universe, defs, audit["seed"]))
    _require(expected == audit, "audit is not the frozen selection")


def assignment_ledger_hash(assignments):
    return digest(canonical([{"unit_id": key[0], "kind": key[1], "assignment": value}
                             for key, value in sorted(assignments.items())]))


def blind_packet(audit, universe, defs, first_assignments_sha256):
    """Only `packet` goes to B. Custodian stages screened bytes at opaque paths.

    No first labels, model identities, partitions, source IDs or routing metadata
    are accepted here. Sources still need host screening; renaming is not redaction.
    """
    validate_audit(audit, universe, defs)
    _hash(first_assignments_sha256)
    task_id = secrets.token_hex(16)
    entries, routes = [], {}
    for unit in audit["selected"]:
        alias = secrets.token_hex(16)
        entries.append({"alias": alias, "kind": unit["kind"], "path": f"inputs/{alias}.bin",
                        "sha256": unit["source_sha256"]})
        routes[alias] = unit
    return {"packet": {"task_id": task_id, "definitions_sha256": defs.vocabulary_sha256, "entries": entries},
            "host_routes": {"task_id": task_id, "audit_sha256": audit["sha256"],
                            "first_assignments_sha256": first_assignments_sha256, "aliases": routes}}


def _validate_runtime_receipt(receipt, assignment_id):
    _keys(receipt, ("kind", "id", "model_id", "model_version", "version_limitation",
                    "assignment_id", "runtime_session_id", "prompt_sha256", "tool_sha256", "config_sha256"))
    _require(receipt.get("kind") == "model" and receipt.get("assignment_id") == assignment_id,
             "audit needs actual model assignment")
    _text(receipt["id"])
    _text(receipt.get("runtime_session_id"))
    _text(receipt.get("model_id"))
    _require(receipt["model_id"] not in ("Astra", "Fable"), "friendly name is not runtime identity")
    for key in ("prompt_sha256", "tool_sha256", "config_sha256"):
        _hash(receipt[key])
    if receipt["model_version"] is None:
        _text(receipt["version_limitation"])
    else:
        _text(receipt["model_version"])


def _audit_assignment(value, defs, receipts):
    _keys(value, ("state", "value", "assignment_id"))
    _require(value["state"] in ("assigned", "absent", "unresolved", "failed"), "invalid audit state")
    _require(value["assignment_id"] in receipts, "missing audit runtime receipt")
    _validate_runtime_receipt(receipts[value["assignment_id"]], value["assignment_id"])
    if value["state"] in ("failed", "unresolved"):
        _require(value["value"] is None, "unresolved audit value must be null")
    else:
        _strings(value["value"])
        _require(bool(value["value"]) == (value["state"] == "assigned"), "invalid audit absence")
    return (value["state"], tuple(sorted(value["value"])) if value["value"] is not None else None)


def _request_metrics(metadata, producer, expected_effort):
    _keys(metadata, ("assignment_id", "request_id", "model_id", "reasoning_effort", "config_sha256",
                     "wall_time_seconds", "usage"))
    _text(metadata["request_id"])
    _require(metadata["assignment_id"] == producer["assignment_id"] and
             metadata["model_id"] == producer["model_id"] and
             metadata["config_sha256"] == producer["config_sha256"], "request metadata identity mismatch")
    _require(metadata["reasoning_effort"] == expected_effort, "mixed reasoning efforts in qualification")
    elapsed = metadata["wall_time_seconds"]
    _require(type(elapsed) in (int, float) and math.isfinite(elapsed) and elapsed >= 0, "invalid request wall time")
    usage = metadata["usage"]
    _require(isinstance(usage, dict) and {"input_tokens", "output_tokens", "total_tokens"} <= set(usage)
             and set(usage) <= {"input_tokens", "output_tokens", "total_tokens", "input_tokens_details", "output_tokens_details"},
             "request token usage required")
    for key in ("input_tokens", "output_tokens", "total_tokens"):
        _require(type(usage[key]) is int and usage[key] >= 0, "invalid token count")
    _require(usage["total_tokens"] == usage["input_tokens"] + usage["output_tokens"], "inconsistent token usage")
    for key in ("input_tokens_details", "output_tokens_details"):
        if key in usage and usage[key] is not None:
            _require(isinstance(usage[key], dict) and all(isinstance(k, str) and type(v) is int and v >= 0
                     for k, v in usage[key].items()), "invalid detailed usage")
    return metadata


def compare_audit(audit, universe, packet, first, second, defs, receipts, request_receipts=None):
    """A keys are (unit_id, kind); B keys are fresh aliases. No averaging away.

    Failed/unreturned B assignments count as disagreements, even when A failed.
    Distinct runtime sessions/assignment IDs are mandatory, not distinct brands.
    """
    validate_audit(audit, universe, defs)
    routes = packet["host_routes"]
    _require(routes["audit_sha256"] == audit["sha256"], "wrong audit routing")
    _require(assignment_ledger_hash(first) == routes["first_assignments_sha256"], "first assignments changed after dispatch")
    _require(packet["packet"]["task_id"] == routes["task_id"]
             and packet["packet"]["definitions_sha256"] == defs.vocabulary_sha256, "packet identity mismatch")
    entries = packet["packet"]["entries"]
    _require(len(entries) == len(routes["aliases"]) and {e["alias"] for e in entries} == set(routes["aliases"]),
             "packet aliases changed")
    for entry in entries:
        _keys(entry, ("alias", "kind", "path", "sha256"))
        unit = routes["aliases"][entry["alias"]]
        _require(entry["kind"] == unit["kind"] and entry["sha256"] == unit["source_sha256"]
                 and entry["path"] == f"inputs/{entry['alias']}.bin", "packet source identity changed")
    _require(sorted(canonical(x) for x in routes["aliases"].values()) ==
             sorted(canonical(x) for x in audit["selected"]), "audit routing changed")
    _require(set(second) <= set(routes["aliases"]), "unknown second-label alias")
    result, metrics = {}, {}
    for kind, summary in audit["per_kind"].items():
        disagreements, confusion, groups = [], Counter(), defaultdict(list)
        matched, resolved_matched = 0, 0
        for alias, unit in routes["aliases"].items():
            if unit["kind"] != kind:
                continue
            key = (unit["unit_id"], kind)
            a, b = first.get(key), second.get(alias)
            av = _audit_assignment(a, defs, receipts) if a else ("missing", None)
            bv = _audit_assignment(b, defs, receipts) if b else ("missing", None)
            if b and audit["version"] == "astra.private.page-qualification/2":
                _require(isinstance(request_receipts, dict) and b["assignment_id"] in request_receipts,
                         "qualification needs independently captured effort/time/usage receipt")
                metadata = _request_metrics(request_receipts[b["assignment_id"]], receipts[b["assignment_id"]],
                                            audit["reasoning_effort"])
                key = metadata["request_id"]
                _require(metrics.setdefault(key, metadata) == metadata, "conflicting request metadata")
            for assignment in (a, b):
                if assignment and assignment["state"] in ("assigned", "absent"):
                    _require(kind in defs.vocabularies and set(assignment["value"]) <= set(defs.vocabularies[kind]),
                             "unapproved audit label")
            if a and b:
                ar, br = receipts[a["assignment_id"]], receipts[b["assignment_id"]]
                _require(a["assignment_id"] != b["assignment_id"] and
                         ar["runtime_session_id"] != br["runtime_session_id"], "audit is not independent")
                if audit["version"].startswith("astra.private.page-qualification/"):
                    _require("terra" not in ar["model_id"].lower() and "terra" in br["model_id"].lower(),
                             "qualification requires strong A and runtime-identified Terra B")
            agrees = av == bv and av[0] not in ("failed", "missing")
            matched += agrees
            resolved_matched += agrees and av[0] in ("assigned", "absent")
            groups[unit["dependency_group"]].append(int(agrees))
            confusion[canonical([av, bv]).decode()] += 1
            if not agrees:
                disagreements.append({"unit_id": unit["unit_id"], "first": av, "second": bv})
        n = summary["sample_size"]
        _require(sum(map(len, groups.values())) == n, "audit denominator changed")
        rate = matched / n
        # Cluster-robust variance of the mean, grouped by source document/test.
        g = len(groups)
        se = (math.sqrt(g / (g - 1) * sum((sum(v) - len(v) * rate) ** 2 for v in groups.values())) / n
              if g > 1 else None)
        result[kind] = {**summary, "agreement": rate, "agreements": matched,
                        "resolved_agreement": resolved_matched / n,
                        "effective_sample_size": n, "dependency_groups": g,
                        "cluster_standard_error": se, "uncertainty_limitation":
                        "Descriptive cluster-robust standard error; no truth-accuracy claim."
                        if se is not None else "Only one dependency group; uncertainty not estimable.",
                        "confusion": dict(confusion), "disagreements": disagreements,
                        "action": "redefine_and_relabel_entire_kind_domain" if rate < .9 else "retain_and_review_rare_classes"}
        if audit["version"].startswith("astra.private.page-qualification/"):
            qualifies = kind in defs.vocabularies and resolved_matched / n >= .9
            result[kind]["terra_qualification"] = "eligible_for_this_kind" if qualifies else "not_qualified_repair_and_repeat"
    report = {"audit_sha256": audit["sha256"], "scope": "protected", "per_kind": result}
    if audit["version"] == "astra.private.page-qualification/2":
        report["reasoning_effort"] = audit["reasoning_effort"]
        report["universe_sha256"] = audit["universe_sha256"]
        report["request_metrics"] = [metrics[key] for key in sorted(metrics)]
    return report


def select_qualified_effort(high_report, xhigh_report):
    """Host-side decision over separately sealed reports from the SAME 200 pages."""
    _require(high_report.get("reasoning_effort") == "high" and xhigh_report.get("reasoning_effort") == "xhigh",
             "separate high and xhigh reports required")
    _require(high_report.get("universe_sha256") == xhigh_report.get("universe_sha256")
             and high_report.get("universe_sha256") is not None, "efforts used different qualification universes")
    kinds = set(high_report["per_kind"])
    _require(kinds and kinds == set(xhigh_report["per_kind"]), "efforts assessed different label kinds")
    for report in (high_report, xhigh_report):
        _require(report.get("scope") == "protected" and all(row["sample_size"] == 200
                 for row in report["per_kind"].values()), "qualification needs complete protected 200-page reports")
    def passed(report):
        return all(row.get("terra_qualification") == "eligible_for_this_kind" and
                   row["resolved_agreement"] >= .9 for row in report["per_kind"].values())
    selected = "high" if passed(high_report) else "xhigh" if passed(xhigh_report) else None
    return {"universe_sha256": high_report["universe_sha256"], "selected_effort": selected,
            "action": "use_qualified_effort_with_exception_review" if selected else "repair_definitions_and_repeat",
            "report_audit_hashes": {"high": high_report["audit_sha256"], "xhigh": xhigh_report["audit_sha256"]}}


def page_coverage(records, expected_pages):
    """Protected census check; expected_pages maps document hash to physical count.

    This checks cards, including failure cards, not the truth of their semantics.
    The caller must supply an independently captured document-page inventory.
    """
    expected = set()
    for document, count in expected_pages.items():
        _hash(document)
        _require(type(count) is int and count >= 0, "invalid physical page count")
        expected.update((document, page) for page in range(1, count + 1))
    actual = [(r["body"]["document_sha256"], r["body"]["page_number"])
              for r in records if r["kind"] == "page"]
    _require(len(set(actual)) == len(actual), "duplicate page card")
    return {"complete": set(actual) == expected, "missing": sorted(expected - set(actual)),
            "unexpected": sorted(set(actual) - expected)}


def write_private_artifact(value, destination, execute=False):
    raw = canonical(value) + b"\n"
    identity = digest(raw)
    path = Path(destination) / (identity + ".json")
    if execute:
        path.parent.mkdir(parents=True, exist_ok=True)
        if path.exists():
            _require(path.read_bytes() == raw, "immutable artifact collision")
        else:
            with path.open("xb") as output:
                output.write(raw)
    return {"sha256": identity, "execute": execute, "path": str(path)}


def immutable_export(records, sources, defs, receipts, destination, execute=False):
    by_id = validate_bundle(records, sources, defs, receipts)
    names = {"page": "pages.jsonl", "test_label": "test_labels.jsonl", "edge": "edges.jsonl", "finding": "findings.jsonl"}
    files = {filename: b"".join(canonical(r) + b"\n" for r in sorted(records, key=lambda r: r["record_id"])
                               if r["kind"] == kind) for kind, filename in names.items()}
    files["clusters.json"] = canonical([r for r in sorted(records, key=lambda r: r["record_id"])
                                        if r["kind"] == "cluster"]) + b"\n"
    manifest = {"version": defs.envelope_version, "scope": "protected", "schema_sha256": defs.schema_sha256,
                "vocabulary_sha256": defs.vocabulary_sha256,
                "owner_authority": defs.owner_authority,
                "authority_replay_commit": defs.replay_commit,
                "producer_code_sha256": digest(Path(__file__).read_bytes()),
                "runtime_receipts_sha256": digest(canonical(receipts)),
                "dependency_closure": {key: dependency_manifest(key, by_id, sources) for key in by_id},
                "files": {name: digest(raw) for name, raw in files.items()}}
    identity = digest(canonical(manifest))
    target = Path(destination) / identity
    if execute:
        target.mkdir(parents=True, exist_ok=True)
        for name, raw in {**files, "manifest.json": canonical(manifest) + b"\n"}.items():
            path = target / name
            if path.exists():
                _require(path.read_bytes() == raw, "immutable artifact collision")
            else:
                with path.open("xb") as output:
                    output.write(raw)
    return {"sha256": identity, "execute": execute, "directory": str(target)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("operation", choices=("export", "freeze", "sample", "qualify", "packet", "compare"))
    parser.add_argument("--bundle", type=Path, required=True, help="protected private bundle JSON")
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--execute", action="store_true", help="write immutable protected outputs")
    parser.add_argument("--legacy-commit", help="full explicit legacy owner commit (export replay only)")
    parser.add_argument("--legacy-schema-commit", help="full explicit schema/prose commit for legacy replay")
    parser.add_argument("--legacy-pins", type=Path, help="independently sealed definition pins JSON")
    parser.add_argument("--legacy-newline", choices=("lf", "crlf"), default="lf")
    args = parser.parse_args()
    bundle = load_json(args.bundle)
    if args.legacy_commit:
        _require(args.operation == "export" and args.legacy_pins is not None, "legacy replay is explicit export validation only")
        defs = definitions(legacy_commit=args.legacy_commit, schema_commit=args.legacy_schema_commit,
                           expected_pins=load_json(args.legacy_pins), newline=args.legacy_newline)
    else:
        _require(args.legacy_pins is None and args.legacy_schema_commit is None and args.legacy_newline == "lf",
                 "legacy options need explicit legacy commit")
        defs = definitions()
    if args.operation == "export":
        _keys(bundle, ("records", "sources", "runtime_receipts"))
        result = immutable_export(bundle["records"], bundle["sources"], defs,
                                  bundle["runtime_receipts"], args.output, args.execute)
    else:
        if args.operation == "freeze":
            _keys(bundle, ("units",))
            value = freeze_universe(bundle["units"], defs)
        elif args.operation == "sample":
            _keys(bundle, ("universe", "seed"))
            value = sample_audit(bundle["universe"], defs, bundle["seed"])
        elif args.operation == "qualify":
            _keys(bundle, ("universe", "reasoning_effort"))
            value = qualification_audit(bundle["universe"], defs, bundle["reasoning_effort"])
        elif args.operation == "packet":
            _keys(bundle, ("universe", "audit", "first_assignments_sha256"))
            value = blind_packet(bundle["audit"], bundle["universe"], defs, bundle["first_assignments_sha256"])
        else:
            fields = ["universe", "audit", "packet", "first", "second", "runtime_receipts"]
            if bundle.get("audit", {}).get("version") == "astra.private.page-qualification/2":
                fields.append("request_receipts")
            _keys(bundle, fields)
            first = {}
            for row in bundle["first"]:
                _keys(row, ("unit_id", "kind", "assignment"))
                key = (row["unit_id"], row["kind"])
                _require(key not in first, "duplicate first assignment")
                first[key] = row["assignment"]
            value = compare_audit(bundle["audit"], bundle["universe"], bundle["packet"], first,
                                  bundle["second"], defs, bundle["runtime_receipts"], bundle.get("request_receipts"))
        result = write_private_artifact(value, args.output, args.execute)
    print(json.dumps(result, sort_keys=True))


if __name__ == "__main__":
    main()
