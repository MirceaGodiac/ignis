"""Summarize acquisition receipts and filename metadata; never open corpus payloads."""
import collections
import json
from pathlib import Path
import re

from fetch_public import ROOT, now, write_json


def main():
    metadata = ROOT / "metadata"
    selected = [json.loads(line) for line in (metadata / "selected_files.jsonl").read_text().splitlines()]
    excluded = [json.loads(line) for line in (metadata / "excluded_document_category_files.jsonl").read_text().splitlines()]
    counts = collections.defaultdict(lambda: {"files": 0, "declared_bytes": 0})
    for row in selected:
        key = (row["category"], Path(row["file_name"]).suffix.lower())
        counts[key]["files"] += 1
        counts[key]["declared_bytes"] += row["size_bytes"]
    archive_rows = []
    for row in excluded:
        name = row["file_name"].lower()
        if not name.endswith(".zip"):
            classification = "media_file"
        elif re.search(r"images|photos", name):
            classification = "likely_media_archive"
        elif re.search(r"drawings|data interpretation|gas flow|instruments|resistive heater", name):
            classification = "likely_document_bundle"
        else:
            classification = "uncertain_archive"
        archive_rows.append(dict(row, filename_only_classification=classification,
                                 classification_verified=False, disposition="pending_sequential_workflow"))
    (metadata / "archive_triage.jsonl").write_text(
        "".join(json.dumps(row, sort_keys=True) + "\n" for row in archive_rows), encoding="utf-8")
    latest = {}
    log = ROOT / "logs" / "fetch_receipts.jsonl"
    if log.exists():
        with log.open(encoding="utf-8") as source:
            for line in source:
                try:
                    row = json.loads(line)
                except json.JSONDecodeError:
                    # A concurrently written last receipt is absent from this snapshot only.
                    if source.read():
                        raise
                    break
                latest[row["identity"]] = row
    states = collections.Counter()
    verified_bytes = 0
    per_investigation = collections.defaultdict(lambda: collections.Counter())
    failures = []
    retrying = []
    object_states = []
    quarantine_path = ROOT / "logs/size_mismatch_quarantine.json"
    quarantined = {row["identity"]: row for row in json.loads(quarantine_path.read_text())} if quarantine_path.exists() else {}
    for row in selected:
        receipt = latest.get(row["identity"], {})
        state = receipt.get("state", "pending")
        verified = state in ("verified_download", "verified_existing")
        if verified:
            reason = "length_and_sha256_verified"
        elif not receipt:
            reason = "not_yet_attempted"
        elif receipt.get("http_status"):
            reason = "HTTP_" + str(receipt["http_status"])
        else:
            reason = receipt.get("error_reason") or receipt.get("error_type") or state
        object_states.append(dict(row, acquisition_state=state, verified=verified,
                                  unresolved_reason=None if verified else reason,
                                  latest_receipt_timestamp=receipt.get("timestamp"),
                                  actual_bytes=receipt.get("actual_bytes"),
                                  sha256=receipt.get("sha256"),
                                  attempt=receipt.get("attempt"),
                                  disposition=receipt.get("disposition", "not_downloaded")))
        if row["identity"] in quarantined:
            kept = quarantined[row["identity"]]
            object_states[-1]["quarantine"] = {key: kept[key] for key in
                ("state", "disposition", "local_path", "actual_bytes", "sha256", "eligible_as_verified_listing_file")}
        states[state] += 1
        per_investigation[row["investigation"]][state] += 1
        if state in ("verified_download", "verified_existing"):
            verified_bytes += receipt["actual_bytes"]
        elif state in ("failed_attempt", "existing_verification_failed", "budget_blocked"):
            if state == "failed_attempt" and receipt.get("attempt", 0) < 3:
                retrying.append(receipt)
            else:
                failures.append(receipt)
    summary = dict(timestamp=now(), selected_files=len(selected),
                   selected_bytes=sum(row["size_bytes"] for row in selected),
                   selected_by_category_extension=[dict(category=cat, extension=ext, **values)
                       for (cat, ext), values in sorted(counts.items())],
                   excluded_files=len(excluded),
                   archive_triage_counts=dict(collections.Counter(row["filename_only_classification"] for row in archive_rows)),
                   latest_states=dict(states), verified_bytes=verified_bytes,
                   per_investigation=dict(per_investigation),
                   terminal_failed_files=len(failures), retrying_files=len(retrying),
                   semantic_content_access=False)
    run_status = ROOT / "logs/fetch_run_status.json"
    summary["run_state"] = json.loads(run_status.read_text()).get("state") if run_status.exists() else "unknown"
    if summary["run_state"] == "interrupted_external_service_unavailable":
        summary["interrupted_retry_files"] = summary["retrying_files"]
        summary["retrying_files"] = 0
    write_json(ROOT / "logs" / "acquisition_failures.json", failures)
    write_json(ROOT / "logs" / "acquisition_retrying.json", retrying)
    for filename, records in (("acquisition_objects.jsonl", object_states),
                              ("acquisition_unresolved.jsonl", [row for row in object_states if not row["verified"]])):
        path = ROOT / "logs" / filename
        temporary = path.with_suffix(path.suffix + ".tmp")
        temporary.write_text("".join(json.dumps(row, sort_keys=True) + "\n" for row in records), encoding="utf-8")
        temporary.replace(path)
    write_json(ROOT / "logs" / "acquisition_inventory.json", summary)
    print(json.dumps({key: summary[key] for key in ("selected_files", "selected_bytes", "excluded_files",
                       "archive_triage_counts", "latest_states", "verified_bytes", "terminal_failed_files", "retrying_files", "run_state")}))


if __name__ == "__main__":
    main()
