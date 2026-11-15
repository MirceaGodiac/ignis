"""Synthetic custody-boundary and page-completeness tests; no NASA payloads."""
import hashlib
import io
import json
from pathlib import Path
import tempfile
import unittest
import zipfile

import document_pages as pages


class DocumentPagesTests(unittest.TestCase):
    def setUp(self):
        parent = pages.WORK / "synthetic_tests"
        parent.mkdir(parents=True, exist_ok=True)
        self.temporary = tempfile.TemporaryDirectory(dir=parent)
        self.root = Path(self.temporary.name)
        self.work = self.root / "work/document_pages"
        self.work.mkdir(parents=True)

    def tearDown(self):
        self.temporary.cleanup()

    def fixture(self, data=None, suffix=".pdf", category="Reports", identity="synthetic"):
        source = self.root / f"corpus/incoming/PSI-0/{identity}{suffix}"
        source.parent.mkdir(parents=True, exist_ok=True)
        if data is None:
            fitz = pages.pymupdf()
            with fitz.open() as document:
                document.new_page().insert_text((30, 30), "SYNTHETIC FIXTURE: apparatus and methods, no scientific results.")
                document.new_page()  # Intentionally blank physical page must remain.
                page = document.new_page()
                page.draw_rect(fitz.Rect(20, 20, 80, 80), color=(1, 0, 0))
                document.save(source)
        else:
            source.write_bytes(data)
        size, sha = pages.digest(source)
        row = {"identity": identity, "local_path": source.relative_to(self.root).as_posix(),
               "size_bytes": size, "category": category, "investigation": "PSI-0",
               "file_name": "synthetic" + suffix, "key": "synthetic", "file_version": 1,
               "psi_file_id": 0, "origin": "synthetic_fixture"}
        receipt = dict(row, state="verified_download", sha256=sha, actual_bytes=size)
        return source, row, receipt

    def metadata(self, rows, receipts, trailing=b""):
        (self.root / "metadata").mkdir(exist_ok=True)
        (self.root / "logs").mkdir(exist_ok=True)
        (self.root / "metadata/selected_files.jsonl").write_bytes(b"".join(pages.encode(r) for r in rows))
        (self.root / "logs/fetch_receipts.jsonl").write_bytes(b"".join(pages.encode(r) for r in receipts) + trailing)

    def test_blank_vector_and_text_pages_retained_and_reproducible(self):
        source, row, receipt = self.fixture()
        self.metadata([row], [receipt])
        result = pages.census(self.root, self.work)
        before = (self.work / "pages.jsonl").read_bytes()
        again = pages.census(self.root, self.work)
        self.assertEqual(before, (self.work / "pages.jsonl").read_bytes())
        records = [json.loads(line) for line in before.splitlines()]
        self.assertEqual([r["page_ordinal"] for r in records], [1, 2, 3])
        self.assertEqual([r["text_status"] for r in records], ["text", "blank_candidate", "vector_only"])
        self.assertEqual(result["physical_pages"], again["physical_pages"])
        self.assertEqual(len({r["page_id"] for r in records}), 3)
        for record in records:
            self.assertEqual(record["text_sha256"], pages.digest(self.work / record["text_path"])[1])

    def test_hash_size_and_metadata_must_match(self):
        source, row, receipt = self.fixture(b"same bytes")
        self.assertEqual(pages.verify(self.root, row, receipt)[0], source)
        for replacement in ({"sha256": "0" * 64}, {"actual_bytes": 1}, {"category": "Raw Data"},
                            {"state": "readback_verified"}):
            with self.subTest(replacement=replacement), self.assertRaises(ValueError):
                pages.verify(self.root, row, dict(receipt, **replacement))
        source.write_bytes(b"changed!!!")
        with self.assertRaises(ValueError):
            pages.verify(self.root, row, receipt)

    def test_paths_cannot_escape_incoming_or_use_partial_payload(self):
        for path in ("../holdout/a.pdf", "holdout/a.pdf", "corpus/incoming/x.pdf.part"):
            with self.subTest(path=path), self.assertRaises(ValueError):
                pages.safe_source(self.root, {"local_path": path})

    def test_receipt_snapshot_ignores_only_incomplete_final_line(self):
        _, row, receipt = self.fixture(b"fixture")
        self.metadata([row], [receipt], b'{"state":')
        _, latest, identity = pages.snapshot(self.root, self.work)
        self.assertEqual(latest[row["identity"]], receipt)
        self.assertEqual(identity["ignored_incomplete_tail_bytes"], 9)
        self.metadata([row], [receipt], b"invalid complete row\n")
        with self.assertRaises(json.JSONDecodeError):
            pages.snapshot(self.root, self.work)

    def test_corrupt_unsupported_and_nonpaginated_sources_are_accounted(self):
        entries = [self.fixture(b"not pdf", ".pdf", identity="broken"),
                   self.fixture(b"not word", ".doc", identity="word"),
                   self.fixture(b"not sheet", ".xls", identity="sheet"),
                   self.fixture(b"raw secret", ".txt", "Raw Data", "raw")]
        self.metadata([r for _, r, _ in entries], [r for _, _, r in entries])
        result = pages.census(self.root, self.work)
        self.assertEqual(result["selected_documents"], 3)
        self.assertEqual(result["document_states"], {"quarantined_error": 1,
                         "unsupported_paginated_format": 1, "nonpaginated_format": 1})
        self.assertEqual(result["physical_pages"], 0)
        self.assertFalse(result["all_document_page_extraction_complete"])
        self.assertNotIn("raw secret", (self.work / "documents.jsonl").read_text())

    def test_duplicate_bytes_share_page_ids_and_keep_both_sources(self):
        source, row, receipt = self.fixture()
        _, second_row, second_receipt = self.fixture(source.read_bytes(), identity="copy")
        self.metadata([row, second_row], [receipt, second_receipt])
        result = pages.census(self.root, self.work)
        self.assertEqual(result["unique_pdf_documents"], 1)
        self.assertEqual(result["physical_pages"], 3)
        self.assertEqual(len((self.work / "documents.jsonl").read_text().splitlines()), 2)

    def test_render_is_page_bound_and_reverifies_source(self):
        source, row, receipt = self.fixture()
        self.metadata([row], [receipt])
        pages.census(self.root, self.work)
        result = pages.render(self.root, self.work, pages.page_id(receipt["sha256"], 2), 72)
        self.assertEqual(result["rendered_pages"], 1)
        self.assertGreater(result["render_bytes"], 0)
        with self.assertRaises(ValueError):
            pages.render(self.root, self.work, pages.page_id(receipt["sha256"], 4), 72)
        source.write_bytes(b"mutated")
        with self.assertRaises(ValueError):
            pages.render(self.root, self.work, pages.page_id(receipt["sha256"], 2), 72)

    def test_converted_pages_keep_original_hash_and_conversion_locator(self):
        pdf, _, _ = self.fixture()
        source, row, receipt = self.fixture(b"synthetic office placeholder", ".docx", identity="office")
        self.metadata([row], [receipt])
        folder = self.work / "conversions" / receipt["sha256"]
        folder.mkdir(parents=True)
        converted = folder / "converted.pdf"
        converted.write_bytes(pdf.read_bytes())
        size, sha = pages.digest(converted)
        pages.atomic_json(folder / "conversion.json", {"complete": True, "application": "Word",
                          "source_sha256": receipt["sha256"], "pdf_path": converted.relative_to(self.work).as_posix(),
                          "pdf_sha256": sha, "pdf_bytes": size})
        result = pages.census(self.root, self.work)
        records = [json.loads(line) for line in (self.work / "pages.jsonl").read_text().splitlines()]
        self.assertEqual(result["unique_pdf_documents"], 0)
        self.assertEqual(result["unique_converted_documents"], 1)
        self.assertTrue(all(r["document_sha256"] == receipt["sha256"] for r in records))
        self.assertTrue(all(r["page_kind"] == "converted_office_page" for r in records))
        self.assertTrue(all("physical_page" not in r["source_locator"] for r in records))
        self.assertEqual(pages.render(self.root, self.work, records[0]["page_id"], 72)["rendered_pages"], 1)
        converted.write_bytes(b"changed conversion")
        with self.assertRaises(ValueError):
            pages.render(self.root, self.work, records[0]["page_id"], 72)

    def test_presentation_preflight_rejects_external_payloads(self):
        source = self.work / "synthetic.pptx"
        for relation, allowed in (("hyperlink", True), ("image", False), ("oleObject", False)):
            with zipfile.ZipFile(source, "w") as archive:
                archive.writestr("ppt/slides/_rels/slide1.xml.rels",
                                 '<Relationships><Relationship TargetMode="External" '
                                 f'Type="http://example.org/{relation}" Target="https://example.org/payload"/></Relationships>')
            if allowed:
                pages.office_preflight(source)
            else:
                with self.assertRaises(ValueError):
                    pages.office_preflight(source)
        with self.assertRaises(ValueError):
            pages.office_preflight(self.work / "synthetic.ppt")

    def test_zero_page_container_is_quarantined_not_complete(self):
        from pypdf import PdfWriter
        stream = io.BytesIO()
        PdfWriter().write(stream)
        _, row, receipt = self.fixture(stream.getvalue())
        self.metadata([row], [receipt])
        result = pages.census(self.root, self.work)
        self.assertEqual(result["document_states"], {"quarantined_error": 1})
        self.assertEqual(result["physical_pages"], 0)
        record = json.loads((self.work / "quarantine.jsonl").read_text())
        self.assertEqual(record["reason"], "empty_pdf_document")


if __name__ == "__main__":
    unittest.main()
