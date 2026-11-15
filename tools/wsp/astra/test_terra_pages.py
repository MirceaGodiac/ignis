"""Synthetic protocol, evidence, blind-scope and failure-boundary checks."""
import copy
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import terra_pages as terra


def bind_context(pages):
    full = terra.canonical({"document_sha256": pages[0]["document_sha256"], "pages": [
        {k: p[k] for k in ("page_id", "page_ordinal", "text")} for p in pages]}).decode()
    return [dict(p, document_context=full, document_context_sha256=terra.sha(full.encode()),
                 context_images=[], context_page_count=len(pages)) for p in pages]


class TerraTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.defs = terra.label_records.definitions()
        cls.schema = terra.response_schema(cls.defs)

    def setUp(self):
        parent = terra.WORK / "synthetic_tests"
        parent.mkdir(parents=True, exist_ok=True)
        self.temp = tempfile.TemporaryDirectory(dir=parent)
        self.directory = Path(self.temp.name)
        self.page = {"page_id": "sha256:" + "a" * 64 + ":page:000001", "document_sha256": "a" * 64,
                     "page_ordinal": 1, "text": "SYNTHETIC FIXTURE camera", "text_sha256": terra.sha(b"SYNTHETIC FIXTURE camera"),
                     "render_sha256": None, "image": None, "layout": {"attach_image": False},
                     "investigation_ids": [], "test_catalogue": [], "origin": "synthetic_fixture"}
        self.page = bind_context([self.page])[0]
        self.evidence = {"kind": "text", "quote": "SYNTHETIC FIXTURE", "occurrence": 0,
                         "bbox": None, "figure_id": None, "reason": None}
        self.row = {"page_id": self.page["page_id"], "labels": [{"kind": k, "state": "unresolved", "value": None,
                    "reason": "Synthetic absence of sufficient evidence", "basis": "observed", "rule": None,
                    "evidence": [self.evidence]} for k in sorted(self.defs.vocabularies) if k != "edge.relation"],
                    "mentioned_test_ids": [], "test_evidence": [], "unresolved_test_mentions": [], "investigation_ids": [],
                    "quantity_evidence": [], "condition_evidence": [], "outcome_suggestion": "uncertain",
                    "confidence": "unresolved", "figure_or_table": False,
                    "search_enrichment": {"status": "not_applicable", "reason": "Synthetic fixture", "grounded_terms": [], "retrieval_cues": []}}
        self.receipt = {"provider_response_id": "synthetic_response", "provider_model": terra.MODEL, "received_at": "2026-11-14T00:00:00Z"}

    def tearDown(self):
        self.temp.cleanup()

    def test_exact_model_effort_no_tools_storage_or_truncation(self):
        page = bind_context([dict(self.page, text=self.page["text"] * 20000)])[0]
        request = terra.request_body([page], self.defs, self.schema)
        self.assertEqual(request["model"], "gpt-6.1-terra")
        self.assertEqual(request["reasoning"], {"effort": "xhigh"})
        self.assertEqual(request["tools"], [])
        self.assertEqual(request["tool_choice"], "none")
        self.assertIs(request["store"], False)
        payload = json.loads(request["input"][1]["content"][1]["text"])
        self.assertEqual(payload["pages"][0]["text"], page["text"])
        self.assertNotIn("previous_response_id", request)

    def test_exact_evidence_and_conservative_host_authority(self):
        record = terra.convert_row(self.row, self.page, self.defs, self.receipt, "b" * 64, self.schema)
        self.assertTrue(record["body"]["reveals_outcome"])
        self.assertEqual(record["body"]["review"]["outcome_decision"]["rule"], "terra_non_authorizing/1")
        self.assertEqual(record["locators"][0]["selector"]["start"], 0)
        self.assertEqual(record["body"]["search_enrichment"]["provenance"]["assignment_id"], record["producer"]["assignment_id"])
        evidence = dict(self.evidence, quote="invented quotation")
        with self.assertRaisesRegex(ValueError, "quote_not_found"):
            terra.locator(evidence, self.page)
        with self.assertRaises(ValueError):
            terra.locator(dict(self.evidence, kind="figure", bbox=[0, 0, 1, 1], figure_id="region"), self.page)

    def test_only_failed_rows_are_retry_candidates(self):
        second_page = dict(self.page, page_id=self.page["page_id"].replace("000001", "000002"), page_ordinal=2)
        accepted, failed = terra.validate_response({"rows": [self.row]}, [self.page, second_page], self.defs,
                                                   self.receipt, "b" * 64, self.schema)
        self.assertEqual(set(accepted), {self.page["page_id"]})
        self.assertEqual(set(failed), {second_page["page_id"]})
        accepted, failed = terra.validate_response({"rows": [self.row, self.row]}, [self.page], self.defs,
                                                   self.receipt, "b" * 64, self.schema)
        self.assertFalse(accepted)
        self.assertEqual(failed[self.page["page_id"]], "missing_or_duplicate_page_row")

    def test_unknown_pages_labels_and_test_ids_fail(self):
        altered = copy.deepcopy(self.row)
        altered["page_id"] = "unrequested"
        with self.assertRaisesRegex(ValueError, "unexpected_page"):
            terra.validate_response({"rows": [altered]}, [self.page], self.defs, self.receipt, "b" * 64, self.schema)
        altered = copy.deepcopy(self.row)
        altered["mentioned_test_ids"] = ["invented:test"]
        with self.assertRaisesRegex(ValueError, "unrecognized_test"):
            terra.convert_row(altered, self.page, self.defs, self.receipt, "b" * 64, self.schema)
        altered = copy.deepcopy(self.row)
        altered["labels"].pop()
        with self.assertRaisesRegex(ValueError, "missing_or_duplicate_label"):
            terra.convert_row(altered, self.page, self.defs, self.receipt, "b" * 64, self.schema)

    def test_batches_preserve_document_and_consecutive_page_boundaries(self):
        pages = [dict(self.page, page_ordinal=n) for n in (1, 2, 5, 6)]
        pages.append(dict(self.page, document_sha256="c" * 64))
        self.assertEqual([len(x) for x in terra.batches(iter(pages), 24)], [2, 2, 1])
        self.assertEqual([len(x) for x in terra.batches(iter(pages), 1)], [1] * 5)

    def test_payload_budget_splits_without_truncating_any_page(self):
        first = dict(self.page, text="synthetic full text " * 500)
        second = dict(first, page_id=first["page_id"].replace("000001", "000002"), page_ordinal=2)
        first, second = bind_context([first, second])
        limit = len(terra.canonical(terra.request_body([first], self.defs, self.schema))) + 10
        result = list(terra.adaptive_batches(iter([first, second]), self.defs, self.schema, 24, limit))
        self.assertEqual([len(b) for b in result], [1, 1])
        self.assertEqual(result[0][0]["text"], first["text"])
        self.assertEqual(result[1][0]["text"], second["text"])

    def test_novel_cues_require_source_grounding_and_new_vocabulary(self):
        altered = copy.deepcopy(self.row)
        altered["search_enrichment"] = {"status": "filled", "reason": None,
            "grounded_terms": [{"id": "camera", "kind": "instrument", "text": "camera", "evidence": [dict(self.evidence, quote="camera")]}],
            "retrieval_cues": [{"text": "optical recording instrument", "kind": "concept_expansion", "grounding_ids": ["camera"], "new_terms": ["optical recording"]}]}
        terra.convert_row(altered, self.page, self.defs, self.receipt, "b" * 64, self.schema)
        altered["search_enrichment"]["retrieval_cues"][0]["new_terms"] = ["camera"]
        with self.assertRaisesRegex(ValueError, "already present"):
            terra.convert_row(altered, self.page, self.defs, self.receipt, "b" * 64, self.schema)

    def test_provider_incomplete_effort_fallback_and_tool_calls_rejected(self):
        payload = {"id": "synthetic", "status": "completed", "model": terra.MODEL,
                   "reasoning": {"effort": "xhigh"}, "tools": [], "tool_choice": "none", "store": False,
                   "output": [{"type": "message", "status": "completed", "content": [{"type": "output_text", "text": '{"rows":[]}'}]}]}
        class Response:
            status_code = 200
            headers = {"x-request-id": "synthetic"}
            content = b""
        for update in ({"status": "incomplete"}, {"model": "gpt-other"}, {"reasoning": {"effort": "high"}},
                       {"output": [{"type": "function_call"}]}):
            response = Response()
            response.content = json.dumps(dict(payload, **update)).encode()
            with patch.object(terra.requests, "post", return_value=response), self.assertRaises(ValueError):
                terra.call_provider(terra.request_body([self.page], self.defs, self.schema), self.directory, "synthetic-key-never-store")
            self.assertTrue((self.directory / "receipt.json").exists())


if __name__ == "__main__":
    unittest.main()
