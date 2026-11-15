"""Synthetic-only boundary and reproducibility tests; no experiment files read."""
import copy
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parent))
import author_gate
import label_records as labels


class LabelsTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.defs = labels.definitions()

    def setUp(self):
        self.source = b"Synthetic page: specimen is solid."
        self.h = labels.digest(self.source)
        self.sources = {"source": self.h, "target": self.h}
        self.producer = {"kind": "model", "id": "synthetic-runtime", "model_id": "fixture/provider-model",
                         "model_version": None, "version_limitation": "Synthetic runtime exposes no build version.",
                         "assignment_id": "A", "runtime_session_id": "session-A", "prompt_sha256": self.h,
                         "tool_sha256": self.h, "config_sha256": self.h}
        self.receipts = {"A": copy.deepcopy(self.producer)}
        self.locator = {"source_id": "source", "source_sha256": self.h, "page": 1,
                        "selector": {"kind": "text", "start": 16, "end": 33, "quote": "specimen is solid",
                                     "extraction_sha256": self.h}}

    def record(self, key="page", kind="page"):
        label = {"kind": "fuel.phase", "state": "assigned", "value": ["solid"], "reason": None,
                 "basis": "observed", "rule": None, "locators": [copy.deepcopy(self.locator)]}
        body = {"page_id": key, "document_sha256": self.h, "page_number": 1, "reveals_outcome": False,
                "revealed_test_ids": [], "mentioned_test_ids": ["train"], "unresolved_test_mentions": [],
                "investigation_ids": [], "quantity_evidence": [], "condition_evidence": [],
                "search_enrichment": {"status": "unresolved", "reason": "Synthetic evidence only.",
                                      "grounded_terms": [], "retrieval_cues": [],
                                      "provenance": {"method": "sira-inspired-page-enrichment/1",
                                                     **{k: self.producer[k] for k in
                                                        ("assignment_id", "prompt_sha256", "tool_sha256", "config_sha256")}}},
                "review": {"confidence": "high", "figure_or_table": False, "label_disagreement": False,
                           "requires_strong_review": False, "outcome_decision": {
                               "method": "strong", "assignment_id": "A", "rule": None}}}
        record = {"envelope_version": labels.VERSION, "origin": "synthetic_fixture", "record_id": key,
                  "kind": kind, "schema_version": self.defs.schema_version, "schema_sha256": self.defs.schema_sha256,
                  "vocabulary_sha256": self.defs.vocabulary_sha256, "producer": copy.deepcopy(self.producer),
                  "owner_authority": self.defs.owner_authority,
                  "created_at": "2026-11-14T12:00:00Z", "locators": [copy.deepcopy(self.locator)],
                  "dependencies": ["source"], "test_dependencies": ["train"], "labels": [label], "body": body}
        if kind == "test_label":
            record["body"] = {"test_id": "train"}
        if kind == "page":
            record["labels"].append({"kind": "page.kind", "state": "assigned", "value": ["methods"],
                                     "reason": None, "basis": "interpretation",
                                     "rule": None, "locators": [copy.deepcopy(self.locator)]})
        return labels.seal(record)

    def gate(self, records, partitions=None):
        partitions = partitions or {"train": "train", "val": "validation"}
        schema = {"origin": "synthetic_fixture", "version": "schema/1"}
        schema_raw = labels.canonical(schema)
        split = {"origin": "synthetic_fixture", "version": "split/1", "frozen_by": "Fable",
                 "schema_sha256": labels.digest(schema_raw), "runs": {
                     key: {"group": key, "partition": value} for key, value in partitions.items()}}
        split_raw = labels.canonical(split)
        payloads = {key: {"sha256": value, "kind": "public_method", "screened": True,
                          "dependencies_complete": True, "runs": [], "pages": []} for key, value in self.sources.items()}
        for record in records:
            payloads[record["record_id"]] = {"sha256": labels.digest(labels.canonical(record)),
                                             "kind": "outcome_page", "screened": True,
                                             "dependencies_complete": True,
                                             "runs": record["test_dependencies"],
                                             "pages": record["dependencies"]}
        mapping = {"origin": "synthetic_fixture", "version": "mapping/1", "schema_sha256": labels.digest(schema_raw),
                   "split_sha256": labels.digest(split_raw), "payloads": payloads}
        metadata = {"schema": schema_raw, "split": split_raw, "mapping": labels.canonical(mapping)}
        pins = {key: author_gate.Pin(json.loads(raw)["version"], labels.digest(raw)) for key, raw in metadata.items()}
        return author_gate.AuthorGate.synthetic_fixture(metadata, pins)

    def validate(self, records):
        return labels.validate_bundle(records, self.sources, self.defs, self.receipts)

    def units(self, n, kind="fuel.phase"):
        return [{"unit_id": str(i), "kind": kind, "source_id": "source", "source_sha256": self.h,
                 "dependency_group": "document-" + str(i // 5)} for i in range(n)]

    def rule(self):
        return {"id": "solid-material", "version": "1", "schema_version": self.defs.schema_version,
                "vocabulary_sha256": self.defs.vocabulary_sha256, "kind": "fuel.phase", "operator": "contains_any",
                "values": ["solid"], "missing": "unresolved", "overlap": True, "unit_conversion": None}

    def test_pins_exact_locators_and_content_are_required(self):
        record = self.record()
        self.validate([record])
        for field in ("schema_sha256", "vocabulary_sha256"):
            bad = copy.deepcopy(record)
            bad[field] = "0" * 64
            with self.assertRaises(ValueError):
                self.validate([labels.seal(bad)])
        bad = copy.deepcopy(record)
        bad["labels"][0]["value"] = ["gas"]
        with self.assertRaisesRegex(ValueError, "content hash"):
            self.validate([bad])
        bad = copy.deepcopy(record)
        bad["locators"][0]["selector"] = {"kind": "caption"}
        with self.assertRaises(ValueError):
            self.validate([labels.seal(bad)])

    def test_missing_vocab_is_unresolved_not_topic_as_event(self):
        record = self.record()
        label = record["labels"][0]
        label.update(kind="observed.phenomenon", value=None, state="unresolved",
                     reason="Vocabulary absent in authoritative schema.")
        self.validate([labels.seal(record)])
        label.update(value=["extinction"], state="assigned")
        with self.assertRaisesRegex(ValueError, "vocabulary extension"):
            self.validate([labels.seal(record)])
        label.update(kind="labels.studied", rule="L2")
        with self.assertRaisesRegex(ValueError, "rule-derived"):
            self.validate([labels.seal(record)])

    def test_actual_identity_and_rule_impersonation_rejected(self):
        record = self.record()
        record["producer"]["model_id"] = "Astra"
        with self.assertRaisesRegex(ValueError, "runtime identity"):
            self.validate([labels.seal(record)])
        record = self.record()
        record["producer"]["model_id"] = "invented/provider"
        with self.assertRaisesRegex(ValueError, "runtime receipt"):
            self.validate([labels.seal(record)])
        record["producer"]["kind"] = "rule"
        with self.assertRaisesRegex(ValueError, "impersonate"):
            self.validate([labels.seal(record)])

    def test_owner_page_kinds_multilabel_and_additional_kind_unresolved(self):
        record = self.record()
        page_kind = record["labels"][1]
        page_kind["value"] = ["methods", "apparatus", "figure"]
        record["body"]["review"]["figure_or_table"] = True
        self.validate([labels.seal(record)])
        page_kind["value"] = ["reference"]
        with self.assertRaisesRegex(ValueError, "unapproved vocabulary"):
            self.validate([labels.seal(record)])
        page_kind.update(state="unresolved", value=None, reason="Reference-only page; no owner-listed kind applies.")
        self.validate([labels.seal(record)])
        record["owner_authority"] = {**self.defs.owner_authority, "sha256": "0" * 64}
        with self.assertRaisesRegex(ValueError, "owner authority"):
            self.validate([labels.seal(record)])

    def test_only_owner_requested_edges_are_assigned(self):
        edge = self.record("edge", "edge")
        edge["dependencies"] = ["source", "target"]
        edge["body"] = {"source": "source", "target": "target", "relation": None,
                        "state": "assigned", "reason": None}
        for relation in labels.EDGE_RELATIONS:
            edge["body"]["relation"] = relation
            self.validate([labels.seal(edge)])
        edge["body"]["relation"] = "similar to"
        with self.assertRaisesRegex(ValueError, "not requested"):
            self.validate([labels.seal(edge)])
        edge["body"].update(state="unresolved", relation=None, reason="Source identity not resolved.")
        self.validate([labels.seal(edge)])

    def test_current_authority_drift_and_bad_heading_fail_clearly(self):
        original = labels.git_authority_bytes
        def changed(commit, path, newline="lf"):
            raw = original(commit, path, newline)
            return raw + b"\n# drift\n" if path == labels.SCHEMA_SOURCE else raw
        with patch.object(labels, "git_authority_bytes", changed):
            with self.assertRaisesRegex(ValueError, "authority/schema drift"):
                labels.definitions()
        schema = original(labels.CURRENT_AUTHORITY_COMMIT, labels.SCHEMA_SOURCE)
        prose = original(labels.CURRENT_AUTHORITY_COMMIT, labels.SCHEMA_PROSE)
        owner = original(labels.CURRENT_AUTHORITY_COMMIT, labels.OWNER_SOURCE)
        broken = owner.replace(b"### 2. Page cards", b"### 20. Page cards")
        with self.assertRaisesRegex(ValueError, "heading changed"):
            labels._definitions_from_bytes(schema, prose, broken)

    def test_qualification_snapshot_never_reads_mutable_worktree_metadata(self):
        with patch.object(Path, "read_bytes", side_effect=AssertionError("mutable metadata was read")):
            pinned = labels.definitions()
        self.assertEqual(pinned.replay_commit["owner_commit"], labels.CURRENT_AUTHORITY_COMMIT)
        self.assertEqual(pinned.owner_authority["version"], "astra.owner.whole-goal/2")
        self.assertIn("rules", pinned.owner_authority["section_sha256"])
        self.assertIn("9", pinned.owner_authority["section_sha256"])

    def legacy_defs(self):
        # Independently recorded metadata-only pins; no private label payload.
        pins = {"schema_sha256": "42574895346e09fff2769b07c95d40be8293d4cc949d0935f691a7c48248be56",
                "vocabulary_sha256": "292b641f105a8a5737bb6f81ef75199139afc06ed54ff2e6fce63f0bc15a31c0",
                "owner_authority": {"source": labels.OWNER_SOURCE, "section": "2b", "version": "astra.owner.section2b/1",
                                    "sha256": "16ac9ea1ea96f9859ab91af82acf4560c9ebace811682c6c18c9ba3e8cdba50a",
                                    "section_sha256": "2ce97f352df2b4d17cbca38b82dbc67a9cceb1d64a47c948d73e1b38761a45b3"}}
        return labels.definitions(legacy_commit="1ca40c0e5fa23159137c331f8729b037c61e4186",
                                  schema_commit=labels.CURRENT_AUTHORITY_COMMIT, expected_pins=pins, newline="lf")

    def test_explicit_legacy_replay_preserves_envelope_and_rejects_wrong_pins(self):
        legacy = self.legacy_defs()
        record = self.record()
        record.update(envelope_version=legacy.envelope_version, schema_sha256=legacy.schema_sha256,
                      vocabulary_sha256=legacy.vocabulary_sha256, owner_authority=legacy.owner_authority)
        del record["body"]["search_enrichment"]
        del record["body"]["review"]
        record = labels.seal(record)
        before = labels.canonical(record)
        labels.validate_bundle([record], self.sources, legacy, self.receipts)
        self.assertEqual(labels.canonical(record), before)
        with self.assertRaisesRegex(ValueError, "version mismatch"):
            self.validate([record])
        wrong = labels.definition_pins(legacy)
        wrong["vocabulary_sha256"] = "0" * 64
        with self.assertRaisesRegex(ValueError, "vocabulary or section pin"):
            labels.definitions(legacy_commit=legacy.replay_commit["owner_commit"],
                               schema_commit=labels.CURRENT_AUTHORITY_COMMIT, expected_pins=wrong)
        with self.assertRaisesRegex(ValueError, "pin mismatch"):
            labels.definitions(legacy_commit=legacy.replay_commit["owner_commit"],
                               schema_commit=labels.CURRENT_AUTHORITY_COMMIT,
                               expected_pins=labels.definition_pins(legacy), newline="crlf")

    def test_previous_consolidated_authority_replay_and_script_discrepancy_edge(self):
        pins = {"schema_sha256": "42574895346e09fff2769b07c95d40be8293d4cc949d0935f691a7c48248be56",
                "vocabulary_sha256": "99a733052b1ea801bbdf07a63c69ea53e925e3cdfa1cbe93caf0a900a741fa37",
                "owner_authority": {"source": labels.OWNER_SOURCE, "sections": ["2", "4"],
                                    "version": "astra.owner.consolidated/1",
                                    "sha256": "6b1750856d3149d66c352b0ff770e83a9a7bbf02472dbe275f80434d8e77762a",
                                    "section_sha256": {
                                        "2": "aa4debdf0aef512bd4999b18e3d11c59d806d618d88af957a85236384752c2fc",
                                        "4": "67ecdb66481e80abc09f435e4c786cd2d3be296ae88c08637c16d35cadec57a4"}}}
        old = labels.definitions(authority_commit=labels.PREVIOUS_CONSOLIDATED_COMMIT, expected_pins=pins)
        self.assertEqual(labels.definition_pins(old), pins)
        edge = self.record("discrepancy-edge", "edge")
        edge["producer"].update(kind="rule", id="synthetic-comparison/1", model_id=None,
                                model_version=None, version_limitation=None)
        edge["labels"] = []
        edge["dependencies"] = ["source", "target"]
        edge["body"] = {"source": "source", "target": "target", "state": "assigned", "reason": None,
                        "relation": "table value contradicted by sensor value"}
        self.validate([labels.seal(edge)])
        edge.update(owner_authority=old.owner_authority, vocabulary_sha256=old.vocabulary_sha256)
        with self.assertRaisesRegex(ValueError, "not requested"):
            labels.validate_bundle([labels.seal(edge)], self.sources, old, self.receipts)

    def test_enrichment_grounding_novelty_and_same_pass_provenance(self):
        record = self.record()
        enrichment = record["body"]["search_enrichment"]
        enrichment.update(status="filled", reason=None,
                          grounded_terms=[{"id": "t1", "kind": "material", "text": "solid",
                                           "locators": [self.locator]}],
                          retrieval_cues=[{"kind": "search_phrasing", "text": "solid specimen morphology",
                                           "grounding_ids": ["t1"], "new_terms": ["morphology"]}])
        self.validate([labels.seal(record)])
        labels.validate_enrichment_novelty(enrichment, self.source.decode())
        enrichment["retrieval_cues"][0]["new_terms"] = ["solid"]
        with self.assertRaisesRegex(ValueError, "already present"):
            labels.validate_enrichment_novelty(enrichment, self.source.decode())
        enrichment["retrieval_cues"][0]["grounding_ids"] = ["invented"]
        with self.assertRaisesRegex(ValueError, "source grounding"):
            self.validate([labels.seal(record)])
        enrichment["retrieval_cues"] = []
        enrichment["provenance"]["assignment_id"] = "second-pass"
        with self.assertRaisesRegex(ValueError, "same page-reading"):
            self.validate([labels.seal(record)])

    def test_terra_never_authorizes_and_low_confidence_requires_review(self):
        record = self.record()
        record["producer"]["model_id"] = "gpt-6.1-terra"
        self.receipts["A"] = copy.deepcopy(record["producer"])
        with self.assertRaisesRegex(ValueError, "Terra cannot authorize"):
            self.validate([labels.seal(record)])
        record["body"]["review"]["outcome_decision"] = {
            "method": "conservative", "assignment_id": None, "rule": "terra_non_authorizing/1"}
        with self.assertRaisesRegex(ValueError, "fail closed"):
            self.validate([labels.seal(record)])
        record["body"]["reveals_outcome"] = True
        record = labels.seal(record)
        self.validate([record])
        self.assertEqual(labels.author_projection([record], self.sources, self.defs, self.receipts, self.gate([record])), [])
        record["body"]["review"].update(confidence="low", figure_or_table=True)
        with self.assertRaisesRegex(ValueError, "strong review required"):
            self.validate([labels.seal(record)])
        record["body"]["review"]["requires_strong_review"] = True
        self.validate([labels.seal(record)])

    def test_all_200_qualification_and_no_pass_from_matching_unknowns(self):
        universe = labels.freeze_universe(self.units(200, "page.kind"), self.defs)
        audit = labels.qualification_audit(universe, self.defs, "xhigh")
        self.assertEqual(audit["per_kind"]["page.kind"]["sample_size"], 200)
        first = {(u["unit_id"], u["kind"]): {"state": "unresolved", "value": None, "assignment_id": "A"}
                 for u in audit["selected"]}
        packet = labels.blind_packet(audit, universe, self.defs, labels.assignment_ledger_hash(first))
        self.receipts["B"] = {**self.producer, "model_id": "gpt-6.1-terra", "assignment_id": "B", "runtime_session_id": "session-B"}
        second = {alias: {"state": "unresolved", "value": None, "assignment_id": "B"}
                  for alias in packet["host_routes"]["aliases"]}
        request_receipts = {"B": {"assignment_id": "B", "request_id": "synthetic-request", "model_id": "gpt-6.1-terra",
                                   "reasoning_effort": "xhigh", "config_sha256": self.h, "wall_time_seconds": 2.5,
                                   "usage": {"input_tokens": 10, "output_tokens": 5, "total_tokens": 15}}}
        full_report = labels.compare_audit(audit, universe, packet, first, second, self.defs, self.receipts, request_receipts)
        self.assertEqual(len(full_report["request_metrics"]), 1)
        self.assertEqual(full_report["reasoning_effort"], "xhigh")
        report = full_report["per_kind"]["page.kind"]
        self.assertEqual(report["agreement"], 1)
        self.assertEqual(report["resolved_agreement"], 0)
        self.assertEqual(report["terra_qualification"], "not_qualified_repair_and_repeat")
        bad_universe = labels.freeze_universe(self.units(199, "page.kind"), self.defs)
        with self.assertRaisesRegex(ValueError, "200 pages"):
            labels.qualification_audit(bad_universe, self.defs, "xhigh")
        with self.assertRaisesRegex(ValueError, "explicit separately frozen"):
            labels.qualification_audit(universe, self.defs)
        high_audit = labels.qualification_audit(universe, self.defs, "high")
        self.assertNotEqual(high_audit["sha256"], audit["sha256"])
        self.assertEqual(high_audit["selected"], audit["selected"])
        request_receipts["B"]["reasoning_effort"] = "high"
        with self.assertRaisesRegex(ValueError, "mixed reasoning efforts"):
            labels.compare_audit(audit, universe, packet, first, second, self.defs, self.receipts, request_receipts)

    def test_effort_selection_never_blends_rows_or_different_samples(self):
        row = {"sample_size": 200, "resolved_agreement": .9, "terra_qualification": "eligible_for_this_kind"}
        high = {"scope": "protected", "reasoning_effort": "high", "universe_sha256": self.h,
                "audit_sha256": self.h, "per_kind": {"page.kind": dict(row)}}
        xhigh = {**high, "reasoning_effort": "xhigh", "per_kind": {"page.kind": dict(row)}}
        self.assertEqual(labels.select_qualified_effort(high, xhigh)["selected_effort"], "high")
        high["per_kind"]["page.kind"].update(resolved_agreement=.89, terra_qualification="not_qualified_repair_and_repeat")
        self.assertEqual(labels.select_qualified_effort(high, xhigh)["selected_effort"], "xhigh")
        xhigh["universe_sha256"] = "0" * 64
        with self.assertRaisesRegex(ValueError, "different qualification universes"):
            labels.select_qualified_effort(high, xhigh)

    def test_legacy_current_qualification_bridge_preserves_pins(self):
        legacy = self.legacy_defs()
        bridge = labels.qualification_compatibility(legacy, self.defs, ["page.kind"])
        self.assertEqual(bridge["first"], labels.definition_pins(legacy))
        self.assertEqual(bridge["second"], labels.definition_pins(self.defs))
        self.assertEqual(bridge["scope"], "closed_labels_only_no_enrichment_equivalence")
        with self.assertRaisesRegex(ValueError, "identical closed"):
            labels.qualification_compatibility(legacy, self.defs, ["generated_enrichment"])

    def test_dependency_dangling_cycle_and_omissions(self):
        record = self.record()
        record["dependencies"] = []
        with self.assertRaisesRegex(ValueError, "omitted"):
            self.validate([labels.seal(record)])
        record = self.record()
        record["dependencies"].append("missing")
        with self.assertRaisesRegex(ValueError, "dangling"):
            self.validate([labels.seal(record)])
        record["dependencies"] = ["source", "page"]
        with self.assertRaisesRegex(ValueError, "cycle"):
            self.validate([labels.seal(record)])

    def test_mixed_unknown_pages_and_protected_edges_do_not_project(self):
        record = self.record()
        self.assertEqual(labels.author_projection([record], self.sources, self.defs, self.receipts,
                                                  self.gate([record])), [record])
        for unknown in (True, False):
            bad = self.record()
            if unknown:
                bad["body"]["reveals_outcome"] = None
            else:
                bad["body"]["reveals_outcome"] = True
                bad["body"]["revealed_test_ids"] = ["val"]
                bad["test_dependencies"].append("val")
            bad = labels.seal(bad)
            edge = self.record("edge", "edge")
            edge["body"] = {"source": "page", "target": "target", "relation": "page to quantity",
                            "state": "assigned", "reason": None}
            edge["dependencies"] = ["source", "page", "target"]
            edge = labels.seal(edge)
            self.assertEqual(labels.author_projection([bad, edge], self.sources, self.defs, self.receipts,
                                                      self.gate([bad, edge])), [])

    def test_gate_binds_full_bytes_and_live_is_closed(self):
        record = self.record()
        gate = self.gate([record])
        changed = copy.deepcopy(record)
        changed["labels"][0]["value"] = ["gas"]
        changed = labels.seal(changed)
        self.assertEqual(labels.author_projection([changed], self.sources, self.defs, self.receipts, gate), [])
        self.assertEqual(labels.author_projection([record], self.sources, self.defs, self.receipts,
                                                  author_gate.AuthorGate.live()), [])

    def test_rule_clusters_preserve_missing_and_are_rebuilt(self):
        positive = self.record("positive", "test_label")
        negative = self.record("negative", "test_label")
        negative["labels"][0].update(state="absent", value=[])
        negative = labels.seal(negative)
        missing = self.record("missing", "test_label")
        missing["labels"][0].update(state="unresolved", value=None, reason="Unreadable.")
        missing = labels.seal(missing)
        members = labels.cluster_members(self.rule(), [missing, positive, negative], self.defs)
        self.assertEqual(members, {"member_ids": ["positive"], "unresolved_ids": ["missing"]})
        cluster = self.record("cluster", "cluster")
        cluster["producer"].update(kind="rule", id="label_records.cluster_members/1", model_id=None,
                                    model_version=None, version_limitation=None)
        cluster["labels"] = []
        cluster["body"] = {"rule": self.rule(), "input_ids": ["positive", "negative", "missing"], **members}
        cluster["dependencies"] += cluster["body"]["input_ids"]
        self.validate([positive, negative, missing, labels.seal(cluster)])
        exclusive = copy.deepcopy(cluster)
        exclusive["record_id"] = "exclusive"
        exclusive["body"]["rule"]["overlap"] = False
        with self.assertRaisesRegex(ValueError, "overlap forbidden"):
            self.validate([positive, negative, missing, labels.seal(cluster), labels.seal(exclusive)])
        cluster["body"]["member_ids"] = ["negative"]
        with self.assertRaisesRegex(ValueError, "not reproducible"):
            self.validate([positive, negative, missing, labels.seal(cluster)])

    def test_finding_is_interpretation_and_unknown_page_is_retained(self):
        record = self.record("finding", "finding")
        record["body"] = {"assertion_kind": "observed", "subject": "synthetic investigation", "claim": "Synthetic claim.",
                          "domain": "one synthetic source", "disagreements": [], "unknowns": ["No real evidence."]}
        with self.assertRaisesRegex(ValueError, "interpretation"):
            self.validate([labels.seal(record)])
        record["body"]["assertion_kind"] = "interpretation"
        self.validate([labels.seal(record)])
        page = self.record()
        page["body"]["reveals_outcome"] = None
        page = labels.seal(page)
        self.assertTrue(labels.page_coverage([page], {self.h: 1})["complete"])
        self.assertEqual(labels.page_coverage([page], {self.h: 2})["missing"], [(self.h, 2)])

    def test_sample_boundaries_and_negatives_do_not_change_selection(self):
        for population, expected in ((1, 1), (99, 99), (100, 100), (101, 100), (1999, 100), (2000, 100), (2001, 101), (4001, 201)):
            universe = labels.freeze_universe(self.units(population), self.defs)
            audit = labels.sample_audit(universe, self.defs, "frozen-before-A")
            summary = audit["per_kind"]["fuel.phase"]
            self.assertEqual(summary["sample_size"], expected)
            self.assertEqual(summary["below_minimum_population"], population < 100)
            self.assertEqual(audit, labels.sample_audit(universe, self.defs, "frozen-before-A"))
        bad = self.units(1)
        bad[0]["first_label"] = "solid"
        with self.assertRaises(ValueError):
            labels.freeze_universe(bad, self.defs)

    def audit_fixture(self, n=100):
        universe = labels.freeze_universe(self.units(n), self.defs)
        audit = labels.sample_audit(universe, self.defs, "precommitted")
        first, second = {}, {}
        self.receipts["B"] = {**self.producer, "assignment_id": "B", "runtime_session_id": "session-B"}
        for unit in audit["selected"]:
            # Every negative/absent label is included in the independently frozen census.
            first[(unit["unit_id"], unit["kind"])] = {"state": "absent", "value": [], "assignment_id": "A"}
        packet = labels.blind_packet(audit, universe, self.defs, labels.assignment_ledger_hash(first))
        for alias in packet["host_routes"]["aliases"]:
            second[alias] = {"state": "absent", "value": [], "assignment_id": "B"}
        return universe, audit, packet, first, second

    def test_packet_is_aliased_blind_and_selection_cannot_be_repaired(self):
        universe, audit, packet, _, _ = self.audit_fixture()
        manifest = packet["packet"]
        self.assertEqual(set(manifest), {"task_id", "definitions_sha256", "entries"})
        for entry in manifest["entries"]:
            self.assertEqual(set(entry), {"alias", "kind", "path", "sha256"})
        self.assertNotEqual(manifest, labels.blind_packet(audit, universe, self.defs, self.h)["packet"])
        changed = copy.deepcopy(audit)
        changed["selected"] = changed["selected"][:-1]
        with self.assertRaises(ValueError):
            labels.blind_packet(changed, universe, self.defs, self.h)

    def test_independent_audit_missing_denominator_and_whole_kind_relabel(self):
        universe, audit, packet, first, second = self.audit_fixture()
        aliases = list(second)
        for alias in aliases[:11]:
            del second[alias]
        report = labels.compare_audit(audit, universe, packet, first, second, self.defs, self.receipts)
        result = report["per_kind"]["fuel.phase"]
        self.assertEqual(result["agreement"], .89)
        self.assertEqual(result["effective_sample_size"], 100)
        self.assertEqual(result["action"], "redefine_and_relabel_entire_kind_domain")
        self.assertEqual(len(result["disagreements"]), 11)
        self.receipts["B"]["runtime_session_id"] = "session-A"
        with self.assertRaisesRegex(ValueError, "not independent"):
            labels.compare_audit(audit, universe, packet, first, second, self.defs, self.receipts)

    def test_set_agreement_and_small_population_disclosure(self):
        universe, audit, packet, first, second = self.audit_fixture(1)
        first[("0", "fuel.phase")].update(state="assigned", value=["solid", "gas"])
        packet = labels.blind_packet(audit, universe, self.defs, labels.assignment_ledger_hash(first))
        second = {alias: {"state": "assigned", "value": ["gas", "solid"], "assignment_id": "B"}
                  for alias in packet["host_routes"]["aliases"]}
        next(iter(second.values())).update(state="assigned", value=["gas", "solid"])
        result = labels.compare_audit(audit, universe, packet, first, second, self.defs, self.receipts)["per_kind"]["fuel.phase"]
        self.assertEqual(result["agreement"], 1)
        self.assertTrue(result["below_minimum_population"])
        self.assertIsNone(result["cluster_standard_error"])

    def test_original_assignments_sealed_before_blind_dispatch(self):
        universe, audit, packet, first, second = self.audit_fixture(1)
        first[("0", "fuel.phase")].update(state="assigned", value=["solid"])
        with self.assertRaisesRegex(ValueError, "changed after dispatch"):
            labels.compare_audit(audit, universe, packet, first, second, self.defs, self.receipts)

    def test_agreement_is_per_kind_and_threshold_is_strictly_below_point_nine(self):
        universe = labels.freeze_universe(self.units(100) + self.units(100, "flow.direction"), self.defs)
        audit = labels.sample_audit(universe, self.defs, "frozen-multiple-kinds")
        first = {(u["unit_id"], u["kind"]): {"state": "absent", "value": [], "assignment_id": "A"}
                 for u in audit["selected"]}
        packet = labels.blind_packet(audit, universe, self.defs, labels.assignment_ledger_hash(first))
        self.receipts["B"] = {**self.producer, "assignment_id": "B", "runtime_session_id": "session-B"}
        second, seen = {}, {"fuel.phase": 0, "flow.direction": 0}
        for alias, unit in packet["host_routes"]["aliases"].items():
            kind = unit["kind"]
            seen[kind] += 1
            errors = 10 if kind == "fuel.phase" else 11
            second[alias] = {"state": "absent", "value": [], "assignment_id": "B"}
            if seen[kind] <= errors:
                second[alias].update(state="assigned", value=["solid" if kind == "fuel.phase" else "forced"])
        report = labels.compare_audit(audit, universe, packet, first, second, self.defs, self.receipts)["per_kind"]
        self.assertEqual(report["fuel.phase"]["agreement"], .9)
        self.assertEqual(report["fuel.phase"]["action"], "retain_and_review_rare_classes")
        self.assertEqual(report["flow.direction"]["action"], "redefine_and_relabel_entire_kind_domain")

    def test_dry_run_and_immutable_hash_bound_export(self):
        with tempfile.TemporaryDirectory() as directory:
            destination = Path(directory) / "outputs"
            record = self.record()
            plan = labels.immutable_export([record], self.sources, self.defs, self.receipts, destination)
            self.assertFalse(destination.exists())
            output = labels.immutable_export([record], self.sources, self.defs, self.receipts, destination, True)
            self.assertEqual(plan["sha256"], output["sha256"])
            page_path = Path(output["directory"]) / "pages.jsonl"
            self.assertEqual(json.loads(page_path.read_text(encoding="utf-8")), record)
            page_path.write_text("tampered", encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "immutable"):
                labels.immutable_export([record], self.sources, self.defs, self.receipts, destination, True)

    def test_cli_freeze_default_does_not_write(self):
        with tempfile.TemporaryDirectory() as directory:
            source = Path(directory) / "fixture.json"
            source.write_bytes(labels.canonical({"units": self.units(3)}))
            output = Path(directory) / "output"
            result = subprocess.run([sys.executable, str(Path(labels.__file__)), "freeze", "--bundle", str(source),
                                     "--output", str(output)], text=True, capture_output=True, check=True)
            self.assertFalse(json.loads(result.stdout)["execute"])
            self.assertFalse(output.exists())


if __name__ == "__main__":
    unittest.main()
