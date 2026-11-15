"""Reconcile authoritative server CSV metadata with acquisition; never read NASA payloads."""
import collections
import csv
import gzip
import hashlib
import io
import json
from pathlib import Path

from fetch_public import ROOT, REPO, IDS, CATEGORIES, EXTENSION, now, stable_url, write_json


def key(row):
    return (row["investigation"], str(row.get("psi_file_id", "")), str(row["file_version"]),
            row["file_name"], int(row["size_bytes"]))


def read_jsonl(path):
    return [json.loads(line) for line in path.read_text(encoding="utf-8").splitlines()]


def main():
    metadata = ROOT / "metadata"
    current = read_jsonl(metadata / "selected_files.jsonl")
    current_keys = {key(row) for row in current}
    fresh, historical = {}, {}
    fresh_rows = []
    for number in IDS:
        inv = f"PSI-{number}"
        records = read_jsonl(metadata / f"{inv}.listing.jsonl")
        fresh_rows.extend(records)
        fresh.update((key(row), row) for row in records)
        base = "data/manifests" if number in (69, 159, 25, 26) else "runs/dev/data_all/manifests"
        historical.update((key(row), row) for row in read_jsonl(REPO / base / f"{inv}.listing.jsonl"))
    base_keys = {key(row) for row in fresh_rows if row["category"] in CATEGORIES and EXTENSION.search(row["file_name"])}
    fresh_by_name = collections.defaultdict(list)
    for row in fresh.values():
        fresh_by_name[(row["investigation"], row["file_name"])].append(row)
    server = []
    inputs = []
    for label, relative in (("documents", "server_documents.csv"), ("sensor_text", "server_sensor_text.csv.gz")):
        path = REPO / "docs/wsp/atlas" / relative
        raw = path.read_bytes()  # Named catalogue metadata, never a scientific payload.
        decoded = (gzip.decompress(raw) if path.suffix == ".gz" else raw).decode("utf-8-sig")
        rows = list(csv.DictReader(io.StringIO(decoded)))
        for row in rows:
            row["size_bytes"], row["file_version"] = int(row["size_bytes"]), int(row["file_version"])
            row["server_set"] = label
        server.extend(rows)
        length, sha = len(raw), hashlib.sha256(raw).hexdigest()
        inputs.append(dict(path=str(path.relative_to(REPO)), bytes=length, sha256=sha,
                           rows=len(rows), unique_identities=len({key(row) for row in rows}),
                           declared_bytes=sum(row["size_bytes"] for row in rows)))
    server_map = {}
    for row in server:
        if key(row) in server_map and row != server_map[key(row)]:
            raise ValueError("conflicting authoritative CSV metadata")
        server_map[key(row)] = row
    additions, reasons, deferred = [], [], []
    for identity, row in server_map.items():
        if identity in base_keys:
            continue
        if identity in fresh:
            canonical = dict(fresh[identity])
            reason = "category_filter" if row["category"] not in CATEGORIES else "extension_filter"
        elif identity in historical:
            canonical = dict(historical[identity])
            reason = "authoritative_identity_absent_from_fresh_listing"
        else:
            deferred.append(dict(row, reason="identity_unresolved_against_known_listings"))
            continue
        ext = Path(row["file_name"]).suffix.lower()
        safe_extension = bool(EXTENSION.search(row["file_name"])) or ext == ".log"
        safe_category = row["category"] in CATEGORIES | {"Calibration"}
        if canonical.get("restricted") or not safe_category or not safe_extension:
            deferred.append(dict(row, reason="restricted_or_sequential_archive_media_review_required"))
            continue
        canonical["stable_url"] = stable_url(canonical)
        canonical["identity"] = hashlib.sha256(json.dumps({field: canonical.get(field) for field in
            ("investigation", "key", "file_version", "size_bytes", "psi_file_id")}, sort_keys=True).encode()).hexdigest()
        canonical["local_path"] = f"corpus/incoming/{canonical['investigation']}/{canonical['identity']}{ext}"
        canonical["reconciliation_reason"] = reason
        canonical["authoritative_server_set"] = row["server_set"]
        additions.append(canonical)
        reasons.append(dict(canonical, present_in_fresh_listing=identity in fresh,
                            present_in_historical_listing=identity in historical,
                            alternate_current_versions=[dict(file_version=x["file_version"], size_bytes=x["size_bytes"], psi_file_id=x.get("psi_file_id"))
                                for x in fresh_by_name[(row["investigation"], row["file_name"])] if key(x) != identity]))
    additions.sort(key=lambda row: (row["investigation"], row["key"], row["file_version"]))
    extra_path = metadata / "server_extra_files.jsonl"
    extra_path.write_text("".join(json.dumps(row, sort_keys=True) + "\n" for row in additions), encoding="utf-8")
    write_json(metadata / "server_reconciliation_missing.json", reasons)
    write_json(metadata / "server_reconciliation_deferred.json", deferred)
    write_json(metadata / "current_not_in_server_lists.json", [row for row in current if key(row) not in server_map])
    grouped = collections.defaultdict(lambda: dict(files=0, declared_bytes=0))
    for row in additions:
        group = (row["reconciliation_reason"], row["category"], Path(row["file_name"]).suffix.lower())
        grouped[group]["files"] += 1
        grouped[group]["declared_bytes"] += row["size_bytes"]
    plan = json.loads((metadata / "fetch_plan.json").read_text())
    summary = dict(timestamp=now(), input_metadata=inputs,
                   current_selected_files=len(current_keys), current_selected_bytes=sum(row["size_bytes"] for row in current),
                   original_filter_unique_files=len(base_keys),
                   server_rows=len(server), server_unique_files=len(server_map),
                   server_identity_sha256=hashlib.sha256(json.dumps(sorted(server_map), separators=(",", ":")).encode()).hexdigest(),
                   server_unique_bytes=sum(row["size_bytes"] for row in server_map.values()),
                   overlap_files=len(current_keys & server_map.keys()),
                   current_only_files=len(current_keys - server_map.keys()),
                   server_only_files=len(server_map.keys() - current_keys),
                   safe_additional_files=len(additions), safe_additional_bytes=sum(row["size_bytes"] for row in additions),
                   additions_by_reason_category_extension=[dict(reason=reason, category=cat, extension=ext, **values)
                       for (reason, cat, ext), values in sorted(grouped.items())],
                   deferred_server_files=len(deferred),
                   original_filter_duplicate_listing_rows=plan.get("duplicate_listing_rows"),
                   fresh_listing_rows=len(fresh_rows), fresh_unique_identity_rows=len(fresh),
                   archive_media_category_exclusions=len(read_jsonl(metadata / "excluded_document_category_files.jsonl")),
                   archive_media_accounting="separate archive_triage.jsonl, outside both authoritative CSV sets; pending sequential workflow",
                   semantic_content_access=False, complete_corpus_claim=False)
    write_json(metadata / "server_reconciliation.json", summary)
    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
