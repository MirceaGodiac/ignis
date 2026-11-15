"""Synthetic SIRA artifact invariants; never reads primary pages or outcomes."""
import copy
import json
from pathlib import Path
import tempfile
import unittest

import label_records as labels
import sira_artifacts as sira


def fixture(number, text="Synthetic specimen.", terms=(), dependencies=None):
    raw = text.encode("utf-8")
    document = sira.sha(b"synthetic-document")
    page_id = f"sha256:{document}:page:{number:06d}"
    h = sira.sha(b"synthetic-producer")
    producer = {"kind": "model", "id": "fixture", "model_id": "fixture/model", "model_version": "1",
                "version_limitation": None, "assignment_id": "fixture-assignment", "runtime_session_id": "fixture-session",
                "prompt_sha256": h, "tool_sha256": h, "config_sha256": h}
    locator = {"source_id": "synthetic-source", "source_sha256": document, "page": number,
               "selector": {"kind": "text", "start": 0, "end": len(text), "quote": text,
                            "extraction_sha256": sira.sha(raw)}}
    enrichment = {"status": "filled", "reason": None,
                  "provenance": {"method": "sira-inspired-page-enrichment/1", **{key: producer[key] for key in
                                 ("assignment_id", "prompt_sha256", "tool_sha256", "config_sha256")}},
                  "grounded_terms": [{"id": "g", "kind": "concept", "text": text, "locators": [locator]}],
                  "retrieval_cues": [{"kind": "search_phrasing", "text": term, "grounding_ids": ["g"],
                                      "new_terms": [term]} for term in terms]}
    record = {"envelope_version": "astra.private.labels/3", "origin": "synthetic_fixture", "record_id": page_id,
              "kind": "page", "schema_version": "synthetic/1", "schema_sha256": h, "vocabulary_sha256": h,
              "owner_authority": {"origin": "synthetic_fixture"}, "created_at": "2026-11-14T00:00:00Z",
              "producer": producer, "locators": [locator], "dependencies": dependencies or ["synthetic-source"],
              "test_dependencies": [], "labels": [],
              "body": {"page_id": page_id, "document_sha256": document, "page_number": number,
                       "search_enrichment": enrichment}}
    return sira.VerifiedPage(labels.seal(record), raw, sira.sha(raw))


class ArtifactsTest(unittest.TestCase):
    def setUp(self):
        parent = sira.WORK / "synthetic_tests"
        parent.mkdir(parents=True, exist_ok=True)
        self.tmp = tempfile.TemporaryDirectory(dir=parent)
        self.root = Path(self.tmp.name)
        self.pins = {"fixture_sha256": sira.sha(b"synthetic_fixture")}

    def tearDown(self):
        self.tmp.cleanup()

    def builder(self, name="build", **kwargs):
        return sira.PrivateBuilder(self.root / name, self.pins, synthetic_fixture=True, **kwargs)

    def test_absent_terms_retained_common_filtered_original_bytes_unchanged(self):
        raw = "Synthetic original.\r\nPreserved! "
        with self.builder() as build:
            build.ingest(fixture(1, raw, ["novel cobalt signal", "ubiquitous"] ))
            build.ingest(fixture(2, "ubiquitous across this page"))
            build.ingest(fixture(3, "ubiquitous across another page"))
            build.finalize(3)
            decisions = {r["raw"]: r for r in build._rows("proposals")}
            self.assertEqual(decisions["novel cobalt signal"]["original_df"], 0)
            self.assertEqual(decisions["novel cobalt signal"]["decision"], "retained_absent_original_corpus")
            self.assertEqual(decisions["ubiquitous"]["decision"], "rejected_common_original_phrase")
            self.assertEqual(next(build._rows("pages"))["original_text"], raw)
            self.assertNotIn("ubiquitous", {x["term"] for x in build._rows("enriched_postings")})
            self.assertIn("ubiquitous", {x["term"] for x in build._rows("original_postings")})
            self.assertFalse(decisions["novel cobalt signal"]["evidence"])

    def test_phrase_df_contiguity_and_long_phrase_are_exact(self):
        with self.builder() as build:
            build.ingest(fixture(1, "red and blue red blue"))
            build.ingest(fixture(2, "blue red a b c d gap e"))
            build.ingest(fixture(3, "a b c d e"))
            self.assertEqual(build.original_df("red blue"), 1)
            self.assertEqual(build.original_df("a b c d e"), 1)
            self.assertEqual(build.original_df("blue red"), 2)

    def test_unicode_normalization_dedup_and_atomic_ngrams(self):
        with self.builder() as build:
            build.ingest(fixture(1, terms=["ＣＡＦÉ beam", "cafe\u0301 BEAM", "alpha beta gamma delta epsilon"]))
            build.finalize(1)
            proposals = list(build._rows("proposals"))
            self.assertEqual(proposals[1]["decision"], "rejected_duplicate_normalized_proposal")
            terms = {r["term"] for r in build._rows("enriched_postings")}
            self.assertIn("café beam", terms)
            self.assertIn("beta gamma delta epsilon", terms)
            self.assertNotIn("alpha beta gamma delta epsilon", terms)
            self.assertEqual(sira.tokens("A_B Straẞe ² ﬀ"), ["a", "b", "strasse", "2", "ff"])

    def test_duplicates_conflicts_and_same_text_distinct_physical_pages(self):
        with self.builder() as build:
            page = fixture(1)
            self.assertTrue(build.ingest(page))
            self.assertFalse(build.ingest(page))
            build.ingest(fixture(2))
            self.assertEqual(build.original_df("synthetic specimen"), 2)
            with self.assertRaisesRegex(ValueError, "conflicting_duplicate_page"):
                build.ingest(fixture(1, "Changed original"))
            malformed = copy.deepcopy(page.record)
            malformed["body"]["page_id"] = "different-alias"
            with self.assertRaisesRegex(ValueError, "noncanonical_page_id"):
                build.ingest(sira.VerifiedPage(labels.seal(malformed), page.text, page.text_sha256))
            build.finalize(3)
            self.assertEqual(build.export()["duplicate_page_rows"], 1)

    def test_resume_reproducible_immutable_export_and_changed_pins(self):
        page = fixture(1, terms=["novel cue"])
        with self.builder() as build:
            build.ingest(page)
        with self.builder() as build:
            self.assertFalse(build.ingest(page))
            build.ingest(fixture(2))
            build.finalize(2)
            first = build.export()
            artifact = build.work / "exports" / build.build_id
            (artifact / "proposals.jsonl.partial").write_bytes(b"interrupted write")
            self.assertEqual(first, build.export())
            with self.assertRaisesRegex(ValueError, "immutable_completed_build"):
                build.ingest(fixture(3))
        with self.builder() as build:
            build.finalize(2)
            self.assertEqual(first, build.export())
        with self.assertRaisesRegex(ValueError, "changed_build_pins"):
            self.builder(config=sira.Config(max_df_ratio="0.10"))
        with self.assertRaisesRegex(ValueError, "unreviewed_upstream_pin"):
            self.builder("changed-pin", config=sira.Config(upstream_commit="0" * 40))
        (artifact / "proposals.jsonl").write_bytes(b"tampered")
        with self.assertRaisesRegex(ValueError, "export_checksum_mismatch"):
            sira.verify_export(artifact)

    def test_interrupted_export_resumes_and_is_order_independent(self):
        pages = [fixture(1, terms=["new cue"]), fixture(2)]
        with self.builder("one") as first:
            for page in pages:
                first.ingest(page)
            first.finalize(2)
            summary = first.export()
            export = first.work / "exports" / first.build_id
            (export / "manifest.json").unlink()
            (export / "enriched_postings.jsonl").unlink()
            self.assertEqual(summary, first.export())
        with self.builder("two") as second:
            for page in reversed(pages):
                second.ingest(page)
            second.finalize(2)
            self.assertEqual(summary, second.export())

    def test_concurrent_writer_rejected_and_lock_released_on_close(self):
        with self.builder() as first:
            first.ingest(fixture(1))
            with self.assertRaisesRegex(ValueError, "build_already_has_writer"):
                self.builder()
        with self.builder() as resumed:
            resumed.finalize(1)
            self.assertEqual(resumed.export()["pages"], 1)

    def test_protected_dependencies_never_create_author_projection(self):
        page = fixture(1, terms=["protected cue"], dependencies=["synthetic-source", "protected-record"])
        record = copy.deepcopy(page.record)
        record["test_dependencies"] = ["protected-test-reference"]
        page = sira.VerifiedPage(labels.seal(record), page.text, page.text_sha256)
        with self.builder() as build:
            build.ingest(page)
            build.finalize(1)
            summary = build.export()
            data = list(build._rows("pages"))[0]
            self.assertIn("protected-record", data["record"]["dependencies"])
            self.assertIn("protected-test-reference", data["record"]["test_dependencies"])
            self.assertNotIn("protected", json.dumps(summary))
            self.assertEqual(data["visibility"], "custodian_private")
        with self.assertRaisesRegex(ValueError, "outside_private_root"):
            sira.PrivateBuilder(sira.REPO / "docs/author-output", self.pins)
        with self.assertRaisesRegex(ValueError, "sealed_path"):
            sira.private_path(sira.WORK / "holdout" / "x")

    def test_source_pages_require_independent_host_qualification_not_input_flags(self):
        page = fixture(1)
        record = copy.deepcopy(page.record)
        record.update(origin="source_evidence", qualified=True, audience="author")
        page = sira.VerifiedPage(labels.seal(record), page.text, page.text_sha256)
        with self.assertRaisesRegex(ValueError, "independent_host_qualification_required"):
            sira.PrivateBuilder(self.root / "not-qualified", self.pins)
        with self.builder() as build:
            with self.assertRaisesRegex(ValueError, "source_page_in_fixture_build"):
                build.ingest(page)

    def test_failed_or_unresolved_pages_are_retryable_not_done(self):
        page = fixture(1)
        record = copy.deepcopy(page.record)
        record["body"]["search_enrichment"].update(status="unresolved", reason="fixture failure", grounded_terms=[])
        with self.builder() as build:
            with self.assertRaisesRegex(ValueError, "unresolved_enrichment_not_complete"):
                build.ingest(sira.VerifiedPage(labels.seal(record), page.text, page.text_sha256))
            self.assertTrue(build.ingest(page))
            build.finalize(1)
            self.assertEqual(build.export()["pages"], 1)

    def test_grounding_quote_must_match_pinned_original_text(self):
        page = fixture(1, terms=["new cue"])
        record = copy.deepcopy(page.record)
        record["body"]["search_enrichment"]["grounded_terms"][0]["locators"][0]["selector"]["quote"] = "fabricated"
        with self.builder() as build:
            with self.assertRaisesRegex(ValueError, "source_text_locator_mismatch"):
                build.ingest(sira.VerifiedPage(labels.seal(record), page.text, page.text_sha256))
            self.assertTrue(build.ingest(page))

    def test_exact_threshold_and_no_truncation_or_phrase_cap(self):
        with self.builder() as build:
            build.ingest(fixture(1, "x " * 2500 + "tail original", [f"novel cue {n}" for n in range(15)] + ["rare marker"]))
            build.ingest(fixture(2, "rare marker"))
            for n in range(3, 21):
                build.ingest(fixture(n))
            self.assertEqual(build.original_df("tail original"), 1)
            build.finalize(20)
            proposed = list(build._rows("proposals"))
            self.assertEqual(len(proposed), 16)
            self.assertEqual(proposed[-1]["decision"], "retained_rare_original_phrase")
            enriched = {r["term"]: r for r in build._rows("document_frequencies")}
            self.assertEqual(enriched["rare marker"]["original_df"], 1)
            self.assertEqual(enriched["rare marker"]["enriched_df"], 2)

    def terra_inputs(self):
        h = sira.sha(b"synthetic_definitions")
        defs = labels.Definitions("synthetic/1", h, h, {"page.kind": ["methods"]}, (),
                                  {"origin": "synthetic_fixture"}, labels.VERSION, None, ())
        page = fixture(1, terms=["new retrieval phrase"])
        record = copy.deepcopy(page.record)
        record.update(schema_version=defs.schema_version, schema_sha256=defs.schema_sha256,
                      vocabulary_sha256=defs.vocabulary_sha256, owner_authority=defs.owner_authority)
        record["labels"] = [{"kind": "page.kind", "state": "assigned", "value": ["methods"],
                             "reason": None, "basis": "interpretation", "rule": None,
                             "locators": record["locators"]}]
        record["body"].update(reveals_outcome=False, revealed_test_ids=[], mentioned_test_ids=[],
                              unresolved_test_mentions=[], investigation_ids=[], quantity_evidence=[], condition_evidence=[],
                              review={"confidence": "high", "figure_or_table": False, "label_disagreement": False,
                                      "requires_strong_review": False, "outcome_decision": {
                                          "method": "strong", "assignment_id": "fixture-assignment", "rule": None}})
        record = labels.seal(record)
        text_path = self.root / "original.txt"
        text_path.write_bytes(page.text)
        census = {"page_id": record["body"]["page_id"], "document_sha256": record["body"]["document_sha256"],
                  "page_ordinal": 1, "text_status": "text", "text_path": "original.txt", "text_sha256": page.text_sha256}
        values = {"cards_sha256": [record], "census_sha256": [census],
                  "document_registry_sha256": [{"status": "complete", "receipt_state": "verified_download",
                      "document_sha256": record["body"]["document_sha256"], "source": {
                          "identity": "fixture-source", "file_name": "synthetic-original.pdf", "investigation": "PSI-0",
                          "psi_file_id": 0, "file_version": 1}}],
                  "runtime_receipts_sha256": {"fixture-assignment": record["producer"]},
                  "qualification_review_sha256": {"origin": "synthetic_fixture", "not_an_authorization": True}}
        paths = {}
        pins = {"definition_schema_sha256": defs.schema_sha256, "definition_vocabulary_sha256": defs.vocabulary_sha256}
        for key, value in values.items():
            path = self.root / (key + ".json")
            path.write_bytes(b"".join(sira.canonical(r) + b"\n" for r in value) if isinstance(value, list)
                             else sira.canonical(value))
            paths[key] = path
            pins[key] = sira.file_sha(path)
        return {"work": self.root / "adapter", "cards_path": paths["cards_sha256"],
                "census_path": paths["census_sha256"], "documents_path": paths["document_registry_sha256"], "text_root": self.root,
                "runtime_receipts_path": paths["runtime_receipts_sha256"],
                "qualification_review_path": paths["qualification_review_sha256"],
                "definitions": defs, "pins": pins, "host_check": lambda page, pins: True}

    def test_terra_adapter_streams_and_binds_exact_original_receipts_and_definitions(self):
        kwargs = self.terra_inputs()
        summary = sira.build_terra_private(**kwargs)
        self.assertEqual(summary["pages"], 1)
        export = kwargs["work"] / "exports" / summary["build_id"]
        exported = next(sira.jsonl(export / "pages.jsonl"))
        self.assertEqual(exported["source_aliases"][0]["file_name"], "synthetic-original.pdf")
        self.assertEqual(summary, sira.build_terra_private(**kwargs))
        (self.root / "original.txt").write_bytes(b"tampered text")
        with self.assertRaisesRegex(ValueError, "page_text_hash_mismatch"):
            sira.build_terra_private(**kwargs)

    def test_terra_adapter_does_not_trust_a_review_file_or_changed_pins(self):
        kwargs = self.terra_inputs()
        kwargs["host_check"] = lambda page, pins: False
        with self.assertRaisesRegex(ValueError, "host_qualification_not_accepted"):
            sira.build_terra_private(**kwargs)
        kwargs["host_check"] = None
        with self.assertRaisesRegex(ValueError, "independent_host_qualification_required"):
            sira.build_terra_private(**kwargs)
        kwargs["host_check"] = lambda page, pins: True
        kwargs["qualification_review_path"].write_bytes(b'{"qualified":true}')
        with self.assertRaisesRegex(ValueError, "input_snapshot_pin_mismatch"):
            sira.build_terra_private(**kwargs)


if __name__ == "__main__":
    unittest.main()
