"""Private, receipt-verified document page census. No model or outcome interpretation.

Only --execute opens source payloads. All generated payloads remain under the
custodian work directory. stdout contains aggregate operational metadata only.
"""
from __future__ import annotations

import argparse
import collections
import hashlib
import json
import math
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import zipfile
import xml.etree.ElementTree as ET
from datetime import datetime, timezone

REPO = Path(__file__).resolve().parents[3]
ROOT = REPO / "runs/dev/data_astra"
WORK = ROOT / "work/document_pages"
SCHEMA = "ignis.private.document-page/1"
VERSION = "document-pages/1"
DOC_CATEGORIES = {"Science Documents", "Reports", "Presentations", "Engineering Documents",
                  "Analyzed Data", "Reference Data", "Experimental table"}
VERIFIED = {"verified_download", "verified_existing"}
NONPAGINATED = {".csv", ".xlsx", ".xls", ".txt", ".pxp"}
PEAK_BYTES = 40_000_000_000


def now():
    return datetime.now(timezone.utc).isoformat().replace("+00:00", "Z")


def sha_bytes(data):
    return hashlib.sha256(data).hexdigest()


def digest(path):
    sha, size = hashlib.sha256(), 0
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            sha.update(chunk)
            size += len(chunk)
    return size, sha.hexdigest()


def encode(value):
    return (json.dumps(value, ensure_ascii=False, sort_keys=True) + "\n").encode("utf-8")


def atomic_bytes(path, data, budget=None):
    if budget:
        budget.check(len(data))
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_name(path.name + ".tmp")
    temporary.write_bytes(data)
    temporary.replace(path)


def atomic_json(path, value, budget=None):
    atomic_bytes(path, encode(value), budget)


def safe_source(root, row):
    """Only opaque incoming paths; reject traversal, symlinks and sealed locations."""
    relative = Path(row["local_path"])
    if relative.is_absolute() or ".." in relative.parts:
        raise ValueError("unsafe_source_path")
    source = root / relative
    incoming = (root / "corpus/incoming").resolve()
    source.resolve().relative_to(incoming)
    if source.suffix == ".part":
        raise ValueError("partial_payload")
    return source


def snapshot(root, work):
    """Snapshot only complete receipt lines while acquisition appends concurrently."""
    selected_bytes = (root / "metadata/selected_files.jsonl").read_bytes()
    selected = [json.loads(line) for line in selected_bytes.splitlines() if line.strip()]
    receipts, latest, tail = bytearray(), {}, 0
    with (root / "logs/fetch_receipts.jsonl").open("rb") as stream:
        for line in stream:
            if not line.endswith(b"\n"):
                tail += len(line)
                break
            item = json.loads(line)
            receipts.extend(line)
            latest[item["identity"]] = item
    # These snapshots contain document names and are private to custodians.
    atomic_bytes(work / "inputs/selected_files.jsonl", selected_bytes)
    atomic_bytes(work / "inputs/fetch_receipts.jsonl", bytes(receipts))
    identity = {"selected_sha256": sha_bytes(selected_bytes),
                "receipts_sha256": sha_bytes(receipts), "ignored_incomplete_tail_bytes": tail}
    atomic_json(work / "inputs/snapshot.json", identity)
    return selected, latest, identity


def verify(root, row, receipt):
    if receipt is None or receipt.get("state") not in VERIFIED:
        raise ValueError("awaiting_verified_receipt")
    for key in ("identity", "local_path", "size_bytes", "category", "investigation",
                "file_name", "key", "file_version", "psi_file_id"):
        if receipt.get(key) != row.get(key):
            raise ValueError("receipt_metadata_mismatch")
    sha = receipt.get("sha256")
    if not isinstance(sha, str) or not re.fullmatch(r"[0-9a-f]{64}", sha):
        raise ValueError("invalid_receipt_hash")
    source = safe_source(root, row)
    if not source.is_file():
        raise ValueError("verified_source_missing")
    size, actual_sha = digest(source)
    if size != row["size_bytes"] or size != receipt.get("actual_bytes") or actual_sha != sha:
        raise ValueError("source_verification_failed")
    return source, sha


def page_id(sha, ordinal):
    if not re.fullmatch(r"[0-9a-f]{64}", sha) or type(ordinal) is not int or ordinal < 1:
        raise ValueError("invalid_page_identity")
    return f"sha256:{sha}:page:{ordinal:06d}"


def pymupdf():
    sys.path.insert(0, str(WORK / "dependencies"))
    import pymupdf as fitz
    # MuPDF diagnostics can include source text; persist failures privately instead.
    fitz.TOOLS.mupdf_display_errors(False)
    fitz.TOOLS.mupdf_display_warnings(False)
    return fitz


class Budget:
    def __init__(self, root, selected):
        self.root = root
        current = sum(p.stat().st_size for p in root.rglob("*") if p.is_file())
        remaining = 0
        for row in selected:
            path = safe_source(root, row)
            partial = path.with_suffix(path.suffix + ".part")
            existing = path.stat().st_size if path.exists() else partial.stat().st_size if partial.exists() else 0
            remaining += max(0, row["size_bytes"] - existing)
        # Reserve the entire selected acquisition, including concurrent transfers.
        self.available = PEAK_BYTES - current - remaining

    def check(self, amount):
        if amount > self.available or shutil.disk_usage(self.root).free - amount < 1024 ** 3:
            raise OSError("shared_storage_budget_exhausted")
        self.available -= amount  # Conservative: overwritten bytes stay charged for this run.


def extract_pdf(source, sha, work, budget, identity, conversion=None):
    fitz = pymupdf()
    folder = work / "documents" / sha
    receipt_path = folder / "extraction.json"
    cache_key = {"source_sha256": sha, "extractor": "PyMuPDF", "extractor_version": fitz.VersionBind,
                 "mupdf_version": fitz.VersionFitz, "pipeline": VERSION, "code_sha256": identity,
                 "conversion": conversion}
    if receipt_path.exists():
        existing = json.loads(receipt_path.read_text(encoding="utf-8"))
        if existing.get("cache_key") == cache_key and existing.get("status") == "complete":
            manifest = folder / "pages.jsonl"
            if manifest.exists() and digest(manifest)[1] == existing["manifest_sha256"]:
                rows = [json.loads(line) for line in manifest.read_text(encoding="utf-8").splitlines()]
                if all(digest(work / row["text_path"])[1] == row["text_sha256"] for row in rows):
                    return existing
    folder.mkdir(parents=True, exist_ok=True)
    manifest_tmp = folder / "pages.jsonl.tmp"
    counts, errors = collections.Counter(), 0
    with fitz.open(source) as document, manifest_tmp.open("wb") as manifest:
        if not document.is_pdf or document.needs_pass:
            raise ValueError("not_readable_pdf")
        total = document.page_count
        if total == 0:
            raise ValueError("empty_pdf_document")
        for index in range(total):
            ordinal = index + 1
            text_path = folder / f"{ordinal:06d}.txt"
            row = {"schema": SCHEMA, "page_id": page_id(sha, ordinal), "document_sha256": sha,
                   "page_ordinal": ordinal, "page_kind": "physical_pdf_page", "source_locator":
                   {"sha256": sha, "physical_page": ordinal}, "extractor": cache_key,
                   "text_path": text_path.relative_to(work).as_posix(),
                   "requires_visual_review": True, "semantic_labels_status": "not_assigned"}
            if conversion:
                row.update(page_kind="converted_office_page", conversion=conversion)
                row["source_locator"] = {"sha256": sha, "conversion_page": ordinal,
                                         "converted_pdf_sha256": conversion["pdf_sha256"]}
                if conversion["application"] == "PowerPoint":
                    row["source_locator"]["slide_ordinal"] = ordinal
            try:
                page = document.load_page(index)
                text = page.get_text("text", sort=False)
                images = len(page.get_images(full=True))
                drawings = len(page.get_drawings())
                chars = len("".join(text.split()))
                status = ("text" if chars >= 40 else "low_text" if chars else
                          "image_only" if images else "vector_only" if drawings else "blank_candidate")
                row.update(text_status=status, text_characters=chars, image_objects=images,
                           vector_paths=drawings, width_points=page.rect.width,
                           height_points=page.rect.height, printed_page_label=page.get_label())
                data = text.encode("utf-8")
            except Exception as error:
                errors += 1
                row.update(text_status="extraction_error", error_type=type(error).__name__)
                data = b""
            atomic_bytes(text_path, data, budget)
            row["text_sha256"] = sha_bytes(data)
            counts[row["text_status"]] += 1
            encoded = encode(row)
            budget.check(len(encoded))
            manifest.write(encoded)
    manifest_tmp.replace(folder / "pages.jsonl")
    receipt = {"cache_key": cache_key, "status": "complete" if errors == 0 else "partial",
               "page_count": total, "pages_written": sum(counts.values()), "page_errors": errors,
               "text_states": dict(counts), "manifest_sha256": digest(folder / "pages.jsonl")[1]}
    atomic_json(receipt_path, receipt, budget)
    return receipt


def census(root, work):
    work.mkdir(parents=True, exist_ok=True)
    selected, latest, snapshot_identity = snapshot(root, work)
    budget = Budget(root, selected)
    code_sha = digest(Path(__file__))[1]
    statuses, extraction_states = collections.Counter(), collections.Counter()
    unique_docs, successful_bytes, page_count, page_errors = {}, 0, 0, 0
    docs_tmp = work / "documents.jsonl.tmp"
    quarantine_tmp = work / "quarantine.jsonl.tmp"
    with docs_tmp.open("wb") as docs, quarantine_tmp.open("wb") as quarantine:
        for row in selected:
            if row["category"] not in DOC_CATEGORIES:
                continue
            record = {"schema": "ignis.private.document-source/1", "source": row,
                      "receipt_state": latest.get(row["identity"], {}).get("state")}
            try:
                source, sha = verify(root, row, latest.get(row["identity"]))
                record.update(document_sha256=sha, verified_bytes=row["size_bytes"])
                successful_bytes += row["size_bytes"]
                extension = source.suffix.lower()
                conversion_path = work / "conversions" / sha / "conversion.json"
                conversion = None
                if extension in {".doc", ".docx", ".ppt", ".pptx", ".rtf"} and conversion_path.exists():
                    conversion = json.loads(conversion_path.read_text(encoding="utf-8"))
                    converted = work / conversion["pdf_path"]
                    if (conversion["source_sha256"] != sha or not conversion["complete"] or
                            digest(converted) != (conversion["pdf_bytes"], conversion["pdf_sha256"])):
                        raise ValueError("invalid_conversion_receipt")
                    source = converted
                if extension != ".pdf" and conversion is None:
                    record["status"] = "nonpaginated_format" if extension in NONPAGINATED else "unsupported_paginated_format"
                    record["reason"] = "No physical PDF page claim; requires format-specific custodian or faithful conversion"
                else:
                    if sha not in unique_docs:
                        extraction = extract_pdf(source, sha, work, budget, code_sha, conversion)
                        unique_docs[sha] = extraction
                        page_count += extraction["pages_written"]
                        page_errors += extraction["page_errors"]
                        extraction_states.update(extraction["text_states"])
                    record.update(status=unique_docs[sha]["status"], extraction=unique_docs[sha])
            except Exception as error:
                # Exception messages can contain payloads. Only enumerated reasons are emitted.
                reason = str(error) if isinstance(error, ValueError) and str(error) in {
                    "awaiting_verified_receipt", "receipt_metadata_mismatch", "invalid_receipt_hash",
                    "verified_source_missing", "source_verification_failed", "unsafe_source_path",
                    "partial_payload", "not_readable_pdf", "empty_pdf_document"} else "document_processing_failed"
                record.update(status="awaiting_verified" if reason == "awaiting_verified_receipt" else "quarantined_error",
                              reason=reason, error_type=type(error).__name__)
            statuses[record["status"]] += 1
            data = encode(record)
            budget.check(len(data))
            docs.write(data)
            if record["status"] not in {"complete", "partial"}:
                budget.check(len(data))
                quarantine.write(data)
    docs_tmp.replace(work / "documents.jsonl")
    quarantine_tmp.replace(work / "quarantine.jsonl")
    pages_tmp = work / "pages.jsonl.tmp"
    with pages_tmp.open("wb") as target:
        for sha in sorted(unique_docs):
            with (work / "documents" / sha / "pages.jsonl").open("rb") as source:
                for chunk in iter(lambda: source.read(1024 * 1024), b""):
                    budget.check(len(chunk))
                    target.write(chunk)
    pages_tmp.replace(work / "pages.jsonl")
    summary = {"schema": "ignis.private.document-census-summary/1", "created_at": now(),
               "selected_documents": sum(statuses.values()), "document_states": dict(statuses),
               "verified_source_bytes": successful_bytes,
               "unique_pdf_documents": sum(not r["cache_key"]["conversion"] for r in unique_docs.values()),
               "unique_converted_documents": sum(bool(r["cache_key"]["conversion"]) for r in unique_docs.values()),
               "original_pdf_pages": sum(r["pages_written"] for r in unique_docs.values() if not r["cache_key"]["conversion"]),
               "converted_office_pages": sum(r["pages_written"] for r in unique_docs.values() if r["cache_key"]["conversion"]),
               "physical_pages": page_count, "page_extraction_errors": page_errors,
               "text_states": dict(extraction_states), "snapshot": snapshot_identity,
               "software": {"pipeline": VERSION, "code_sha256": code_sha,
                            "pymupdf": pymupdf().VersionBind, "mupdf": pymupdf().VersionFitz},
               "manifest_sha256": digest(work / "pages.jsonl")[1],
               "complete_selected_corpus": statuses["awaiting_verified"] == 0,
               "all_document_page_extraction_complete": not any(statuses[k] for k in
                   ("awaiting_verified", "quarantined_error", "partial", "unsupported_paginated_format")),
               "semantic_labelling_complete": False}
    atomic_json(work / "summary.json", summary, budget)
    return summary


def render(root, work, requested_id, dpi):
    match = re.fullmatch(r"sha256:([0-9a-f]{64}):page:([0-9]+)", requested_id)
    if not match or not math.isfinite(dpi) or dpi <= 0:
        raise ValueError("invalid_render_request")
    sha, ordinal = match[1], int(match[2])
    # Lookup may contain protected identities; never print it.
    record = None
    with (work / "documents.jsonl").open(encoding="utf-8") as stream:
        for line in stream:
            candidate = json.loads(line)
            if candidate.get("document_sha256") == sha and candidate["status"] in {"complete", "partial"}:
                record = candidate
                break
    if record is None:
        raise ValueError("page_not_in_verified_manifest")
    row = record["source"]
    source = safe_source(root, row)
    if digest(source) != (row["size_bytes"], sha):
        raise ValueError("source_changed_since_census")
    conversion = record["extraction"]["cache_key"].get("conversion")
    if conversion:
        source = work / conversion["pdf_path"]
        if digest(source) != (conversion["pdf_bytes"], conversion["pdf_sha256"]):
            raise ValueError("converted_pdf_changed_since_census")
    selected = [json.loads(line) for line in (root / "metadata/selected_files.jsonl").read_text(encoding="utf-8").splitlines()]
    budget = Budget(root, selected)
    fitz = pymupdf()
    with fitz.open(source) as document:
        if not 1 <= ordinal <= document.page_count:
            raise ValueError("page_out_of_range")
        page = document.load_page(ordinal - 1)
        # Raw allocation is bounded by the same actual storage budget, before rasterizing.
        budget.check(math.ceil(page.rect.width * dpi / 72) * math.ceil(page.rect.height * dpi / 72) * 4)
        pixmap = page.get_pixmap(matrix=fitz.Matrix(dpi / 72, dpi / 72), alpha=False)
        data = pixmap.tobytes("png")
    path = work / "renders" / sha / f"{ordinal:06d}-{dpi:g}dpi.png"
    atomic_bytes(path, data, budget)
    atomic_json(path.with_suffix(".json"), {"page_id": requested_id, "source_sha256": sha,
                "dpi": dpi, "renderer": "PyMuPDF", "version": fitz.VersionBind,
                "image_sha256": sha_bytes(data), "bytes": len(data)}, budget)
    return {"rendered_pages": 1, "render_bytes": len(data)}


OFFICE_SCRIPT = r'''
$ErrorActionPreference = 'Stop'
$app = $null
$document = $null
$oldLinks = $null
$oldFields = $null
$oldPrintLinks = $null
$ownedProcess = $null
$sourcePath = $env:IGNIS_CONVERSION_SOURCE
$targetPath = $env:IGNIS_CONVERSION_TARGET
$kind = $env:IGNIS_CONVERSION_KIND
$processName = if ($kind -eq 'Word') { 'WINWORD' } else { 'POWERPNT' }
if (Get-Process -Name $processName -ErrorAction SilentlyContinue) { throw 'existing_office_instance' }
try {
    $app = New-Object -ComObject ($kind + '.Application')
    $createdProcesses = @(Get-Process -Name $processName -ErrorAction SilentlyContinue)
    if ($createdProcesses.Count -eq 1) { $ownedProcess = $createdProcesses[0] }
    $app.AutomationSecurity = 3
    if ($kind -eq 'Word') {
        if ($app.Documents.Count -ne 0) { $app = $null; throw 'nonisolated_office_instance' }
        $app.Visible = $false
        $app.DisplayAlerts = 0
        $oldLinks = $app.Options.UpdateLinksAtOpen
        $oldFields = $app.Options.UpdateFieldsAtPrint
        $oldPrintLinks = $app.Options.UpdateLinksAtPrint
        $app.Options.UpdateLinksAtOpen = $false
        $app.Options.UpdateFieldsAtPrint = $false
        $app.Options.UpdateLinksAtPrint = $false
        $document = $app.Documents.Open($sourcePath, $false, $true, $false)
        $document.ExportAsFixedFormat($targetPath, 17, $false)
        $sourcePages = $document.ComputeStatistics(2)
    } else {
        if ($app.Presentations.Count -ne 0) { $app = $null; throw 'nonisolated_office_instance' }
        $app.DisplayAlerts = 1
        # pptx preflight rejected all external payload relationships (hyperlinks excepted).
        # Untitled=True opens a copy, without giving the original a recent-file identity.
        $document = $app.Presentations.Open($sourcePath, -1, -1, 0)
        $sourcePages = $document.Slides.Count
        $printRange = $document.PrintOptions.Ranges.Add(1, $sourcePages)
        # All slides, including hidden slides, as one PDF page per slide.
        $document.ExportAsFixedFormat($targetPath, 2, 2, 0, 1, 1, -1, $printRange, 4,
                                     '', $false, $true, $true, $true, $false)
    }
    @{application=$kind; version=[string]$app.Version; build=[string]$app.Build;
      source_page_count=$sourcePages} | ConvertTo-Json -Compress
} finally {
    if ($null -ne $document) {
        if ($kind -eq 'Word') { $document.Close(0) } else { $document.Saved = -1; $document.Close() }
        [System.Runtime.InteropServices.Marshal]::FinalReleaseComObject($document) | Out-Null
    }
    if ($null -ne $app) {
        if ($kind -eq 'Word') {
            if ($null -ne $oldLinks) { $app.Options.UpdateLinksAtOpen = $oldLinks }
            if ($null -ne $oldFields) { $app.Options.UpdateFieldsAtPrint = $oldFields }
            if ($null -ne $oldPrintLinks) { $app.Options.UpdateLinksAtPrint = $oldPrintLinks }
            if ($app.Documents.Count -eq 0) { $app.Quit() }
        } elseif ($app.Presentations.Count -eq 0) { $app.Quit() }
        [System.Runtime.InteropServices.Marshal]::FinalReleaseComObject($app) | Out-Null
        if ($null -ne $ownedProcess) { $ownedProcess.WaitForExit(30000) | Out-Null }
    }
}
'''


def office_preflight(source):
    if source.suffix.lower() == ".ppt":
        # Binary PPT can contain automatic linked OLE payloads. No safe parser here.
        raise ValueError("legacy_ppt_external_link_preflight_unavailable")
    if source.suffix.lower() == ".pptx":
        with zipfile.ZipFile(source) as archive:
            for name in archive.namelist():
                if name.endswith(".rels"):
                    with archive.open(name) as stream:
                        for _, element in ET.iterparse(stream, events=("end",)):
                            if (element.attrib.get("TargetMode") == "External" and
                                    not element.attrib.get("Type", "").endswith("/hyperlink")):
                                raise ValueError("external_payload_link_in_presentation")
                            element.clear()


def convert_office(root, work, timeout):
    selected, latest, _ = snapshot(root, work)
    budget = Budget(root, selected)
    states = collections.Counter()
    script = work / "conversion_worker.ps1"
    atomic_bytes(script, OFFICE_SCRIPT.encode("utf-8"), budget)
    log = work / "conversion_receipts.jsonl"
    for row in selected:
        if row["category"] not in DOC_CATEGORIES or Path(row["local_path"]).suffix.lower() not in {".doc", ".docx", ".ppt", ".pptx", ".rtf"}:
            continue
        record = {"source_identity": row["identity"], "status": "conversion_error"}
        try:
            source, sha = verify(root, row, latest.get(row["identity"]))
            folder = work / "conversions" / sha
            target = folder / "converted.pdf"
            receipt_path = folder / "conversion.json"
            if receipt_path.exists():
                prior = json.loads(receipt_path.read_text(encoding="utf-8"))
                if prior.get("complete") and prior["source_sha256"] == sha and digest(target) == (prior["pdf_bytes"], prior["pdf_sha256"]):
                    states["reused"] += 1
                    continue
            office_preflight(source)
            budget.check(0)
            folder.mkdir(parents=True, exist_ok=True)
            environment = dict(os.environ, IGNIS_CONVERSION_SOURCE=str(source.resolve()),
                               IGNIS_CONVERSION_TARGET=str(target.resolve()),
                               IGNIS_CONVERSION_KIND="PowerPoint" if source.suffix.lower().startswith(".ppt") else "Word")
            completed = subprocess.run(["powershell.exe", "-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass",
                                        "-File", str(script.resolve())], env=environment,
                                       stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=timeout,
                                       creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0))
            atomic_bytes(folder / "worker.stdout", completed.stdout, budget)
            atomic_bytes(folder / "worker.stderr", completed.stderr, budget)
            if completed.returncode:
                raise ValueError("office_worker_failed")
            software = json.loads(completed.stdout.decode("utf-8-sig"))
            size, pdf_sha = digest(target)
            budget.check(size)
            if digest(source) != (row["size_bytes"], sha):
                raise ValueError("source_changed_during_conversion")
            with pymupdf().open(target) as pdf:
                count = pdf.page_count
            if count != software["source_page_count"]:
                raise ValueError("conversion_page_count_mismatch")
            receipt = dict(software, complete=True, source_sha256=sha, pdf_sha256=pdf_sha,
                           pdf_bytes=size, pdf_path=target.relative_to(work).as_posix(),
                           converted_pages=count, conversion_script_sha256=sha_bytes(OFFICE_SCRIPT.encode()),
                           original_unchanged=True, macros_disabled=True, external_links_not_refreshed=True,
                           page_fidelity="conversion_dependent_not_original_pdf_pagination",
                           font_substitution_status="not_independently_verified", visual_review_required=True)
            atomic_json(receipt_path, receipt, budget)
            record.update(status="converted", source_sha256=sha, converted_pages=count)
        except Exception as error:
            record.update(error_type=type(error).__name__, reason=str(error) if isinstance(error, ValueError)
                          and str(error) in {"legacy_ppt_external_link_preflight_unavailable",
                          "external_payload_link_in_presentation", "office_worker_failed", "conversion_page_count_mismatch",
                          "source_changed_during_conversion", "awaiting_verified_receipt"} else "conversion_failed")
        states[record["status"]] += 1
        with log.open("ab") as stream:
            stream.write(encode(record))
        print(json.dumps({"office_progress": dict(states)}), flush=True)
    return {"office_conversion_states": dict(states)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--execute", action="store_true")
    parser.add_argument("--render-page", help="private page identifier from pages.jsonl")
    parser.add_argument("--dpi", type=float, default=150)
    parser.add_argument("--convert-office", action="store_true")
    parser.add_argument("--conversion-timeout", type=float, default=300,
                        help="per-document Office process wall-clock resource budget in seconds")
    args = parser.parse_args()
    if not args.execute:
        print(json.dumps({"execute": False, "payloads_opened": 0, "operation": "render" if args.render_page else "census"}))
        return 0
    if args.convert_office:
        result = convert_office(ROOT, WORK, args.conversion_timeout)
    else:
        result = render(ROOT, WORK, args.render_page, args.dpi) if args.render_page else census(ROOT, WORK)
    print(json.dumps(result, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
