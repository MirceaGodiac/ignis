"""PSI metadata-only planning and opaque, verified downloads into Astra's incoming corpus.

No document, table, trace, image or archive is semantically opened by this tool.
Network access requires --execute. This is corpus acquisition, not author access.
"""
from __future__ import annotations

import argparse
import collections
import concurrent.futures
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import threading
import time
from datetime import datetime, timezone
from urllib.parse import quote, urlencode

import requests

REPO = Path(__file__).resolve().parents[3]
ROOT = REPO / "runs/dev/data_astra"
IDS = (69, 159, 25, 26, 68, 39, 10, 23, 20, 21, 22, 107, 106, 98, 99, 100, 102, 101, 47)
CATEGORIES = {"Science Documents", "Reports", "Presentations", "Engineering Documents",
              "Analyzed Data", "Reference Data", "Experimental table", "Raw Data"}
EXTENSION = re.compile(r"\.(pdf|docx?|pptx?|xlsx?|csv|txt|rtf|pxp)$", re.I)
BASE = "https://psi.nasa.gov/geode-py/ws"
GIB = 1024 ** 3
LOCK = threading.RLock()
LOCAL = threading.local()


def now():
    return datetime.now(timezone.utc).isoformat().replace("+00:00", "Z")


def digest(path):
    h = hashlib.sha256()
    size = 0
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            size += len(chunk)
            h.update(chunk)
    return size, h.hexdigest()


def write_json(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_suffix(path.suffix + ".tmp")
    temporary.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    temporary.replace(path)


def session():
    if not hasattr(LOCAL, "session"):
        LOCAL.session = requests.Session()
        LOCAL.session.headers.update({"User-Agent": "IGNIS-Astra-corpus/1.0 (read-only NASA research)",
                                      "Accept-Encoding": "identity"})
    return LOCAL.session


def stable_url(row):
    query = {"file": row["file_name"]}
    if row.get("file_version") is not None:
        query["version"] = row["file_version"]
    return f"{BASE}/studies/{quote(row['investigation'], safe='')}/download?{urlencode(query, quote_via=quote)}"


def flatten(investigation, payload):
    def walk(node):
        for leaf in node.get("files", []):
            if leaf.get("directory") is False:
                yield leaf
            else:
                yield from walk(leaf)
    result = []
    for category in payload["files"]:
        for leaf in walk(category):
            key = leaf["key"]
            result.append(dict(investigation=investigation, category=category["displayName"],
                               key=key, file_name=key.rsplit("/", 1)[-1],
                               size_bytes=int(leaf["bytes_size"]), file_version=int(leaf["version"]),
                               psi_file_id=leaf.get("id"), updated=leaf.get("updated"),
                               restricted=bool(leaf.get("restricted") or leaf.get("restrictedAccess"))))
    return sorted(result, key=lambda row: row["key"])


def listings(root, refresh):
    rows, snapshots = [], []
    for number in IDS:
        inv = f"PSI-{number}"
        target = root / "metadata" / f"{inv}.listing.jsonl"
        if refresh:
            for attempt in range(1, 4):
                try:
                    response = session().get(f"{BASE}/repo/investigations/{inv}", timeout=(30, 120))
                    response.raise_for_status()
                    version = int(response.json()["version"])
                    response = session().post(f"{BASE}/files/v2", json={"folder": "", "fileType": "study",
                                              "studyId": inv, "version": version}, timeout=(30, 120))
                    response.raise_for_status()
                    records = flatten(inv, response.json())
                    break
                except (requests.RequestException, ValueError, KeyError) as error:
                    write_json(root / "metadata" / f"{inv}.refresh_status.json", dict(timestamp=now(),
                               state="failed", attempt=attempt, error_type=type(error).__name__))
                    if attempt == 3:
                        raise RuntimeError(f"metadata refresh failed for {inv}: {type(error).__name__}") from None
                    time.sleep(2 ** attempt)
            source = f"{BASE}/files/v2"
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text("".join(json.dumps(row, sort_keys=True) + "\n" for row in records), encoding="utf-8")
            write_json(root / "metadata" / f"{inv}.refresh_status.json", dict(timestamp=now(),
                       state="complete", investigation_version=version, files=len(records), listing_sha256=digest(target)[1]))
            print(json.dumps(dict(investigation=inv, metadata_files=len(records), state="metadata_refreshed")), flush=True)
        else:
            source_path = target if target.exists() else REPO / (
                "data/manifests" if number in (69, 159, 25, 26) else "runs/dev/data_all/manifests"
            ) / f"{inv}.listing.jsonl"
            records = [json.loads(line) for line in source_path.read_text(encoding="utf-8").splitlines()]
            source = str(source_path.relative_to(REPO))
            if source_path != target:
                target.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(source_path, target)
            version = None
        length, sha = digest(target)
        snapshots.append(dict(investigation=inv, source=source, listing_bytes=length,
                              listing_sha256=sha, investigation_version=version, files=len(records)))
        rows.extend(records)
    return rows, snapshots


class Fetcher:
    def __init__(self, root, peak_bytes, minimum_free):
        self.root, self.peak, self.minimum_free = root, peak_bytes, minimum_free
        self.log = root / "logs" / "fetch_receipts.jsonl"
        self.log.parent.mkdir(parents=True, exist_ok=True)
        self.latest = {}
        self.verified = {}
        if self.log.exists():
            lines = self.log.read_text(encoding="utf-8").splitlines()
            for line in lines:
                item = json.loads(line)  # A damaged audit log fails closed; never silently discard it.
                self.latest[item["identity"]] = item
                if item.get("state") in ("verified_download", "verified_existing", "readback_verified"):
                    self.verified[item["identity"]] = item
        self.used = sum(path.stat().st_size for path in root.rglob("*") if path.is_file())
        self.reserved = 0

    def record(self, row, **fields):
        receipt = dict(row, timestamp=now(), **fields)
        with LOCK:
            with self.log.open("a", encoding="utf-8") as out:
                out.write(json.dumps(receipt, sort_keys=True) + "\n")
                out.flush()
            self.latest[row["identity"]] = receipt
            if receipt.get("state") in ("verified_download", "verified_existing", "readback_verified"):
                self.verified[row["identity"]] = receipt
        return receipt

    def fetch(self, row):
        dest = self.root / row["local_path"]
        dest.parent.mkdir(parents=True, exist_ok=True)
        partial = dest.with_suffix(dest.suffix + ".part")
        previous = self.verified.get(row["identity"], {})
        if dest.exists():
            size, sha = digest(dest)
            if size == row["size_bytes"] and previous.get("sha256") == sha:
                self.record(row, state="verified_existing", actual_bytes=size, sha256=sha,
                            disposition="kept", response_headers=previous.get("response_headers", {}))
                return "verified_existing"
            self.record(row, state="existing_verification_failed", actual_bytes=size, sha256=sha,
                        disposition="kept_for_review")
            return "failed"
        expected = row["size_bytes"]
        initial = partial.stat().st_size if partial.exists() else 0
        if initial > expected:
            size, sha = digest(partial)
            # This is a failed transfer owned by this acquisition, never a corpus final file.
            partial.resolve().relative_to((self.root / "corpus/incoming").resolve())
            partial.unlink()
            self.record(row, state="oversized_partial_discarded", actual_bytes=size, sha256=sha,
                        disposition="deleted_invalid_partial")
            with LOCK:
                self.used -= size
            initial = 0
        reservation = max(0, expected - initial)
        with LOCK:
            free = shutil.disk_usage(self.root).free
            if self.used + self.reserved + reservation > self.peak or free - self.reserved - reservation < self.minimum_free:
                self.record(row, state="budget_blocked", actual_bytes=initial, sha256=None,
                            disposition="kept_partial" if initial else "not_downloaded", disk_free_bytes=free)
                return "failed"
            self.reserved += reservation
        try:
            for attempt in range(1, 4):
                remote_headers = self.latest.get(row["identity"], {}).get("response_headers", {})
                try:
                    offset = partial.stat().st_size if partial.exists() else 0
                    prior = self.latest.get(row["identity"], {})
                    validator = prior.get("response_headers", {}).get("ETag")
                    headers = {"Range": f"bytes={offset}-", "If-Range": validator} if 0 < offset < expected and validator else {}
                    if offset == expected and prior.get("sha256"):
                        size, sha = digest(partial)
                        if sha == prior["sha256"]:
                            partial.replace(dest)
                            self.record(row, state="verified_download", actual_bytes=size, sha256=sha, disposition="kept")
                            return "verified_download"
                    with session().get(row["stable_url"], headers=headers, stream=True, timeout=(30, 120)) as response:
                        response.raise_for_status()
                        remote_headers = {key: response.headers[key] for key in
                                          ("Content-Length", "Content-Type", "ETag", "Last-Modified", "Content-Range")
                                          if key in response.headers}
                        append = response.status_code == 206 and bool(headers)
                        if response.status_code == 206 and not append:
                            raise ValueError("unexpected partial response")
                        if append:
                            match = re.fullmatch(r"bytes (\d+)-(\d+)/(\d+)", response.headers.get("Content-Range", ""))
                            if not match or int(match[1]) != offset or int(match[2]) != expected - 1 or int(match[3]) != expected:
                                raise ValueError("invalid resume Content-Range")
                        else:
                            offset = 0
                        if response.headers.get("Content-Length") and int(response.headers["Content-Length"]) != expected - offset:
                            raise ValueError("HTTP Content-Length differs from listing")
                        self.record(row, state="downloading", attempt=attempt, actual_bytes=offset, sha256=None,
                                    disposition="kept_partial", response_headers=remote_headers)
                        streamed = offset
                        stream_hash = hashlib.sha256()
                        if append:
                            with partial.open("rb") as existing:
                                for chunk in iter(lambda: existing.read(1024 * 1024), b""):
                                    stream_hash.update(chunk)
                        with partial.open("ab" if append else "wb") as output:
                            for chunk in response.iter_content(chunk_size=1024 * 1024):
                                if streamed + len(chunk) > expected:
                                    raise ValueError("response exceeds declared size")
                                if shutil.disk_usage(self.root).free - len(chunk) < self.minimum_free:
                                    raise OSError("free-space reserve reached")
                                output.write(chunk)
                                stream_hash.update(chunk)
                                streamed += len(chunk)
                            output.flush()
                            os.fsync(output.fileno())
                    size, sha = digest(partial)
                    if size != expected:
                        raise ValueError("readback length differs from listing")
                    if sha != stream_hash.hexdigest():
                        raise ValueError("readback hash differs from streamed bytes")
                    self.record(row, state="readback_verified", attempt=attempt, actual_bytes=size, sha256=sha,
                                disposition="kept_partial", response_headers=remote_headers)
                    partial.replace(dest)
                    self.record(row, state="verified_download", actual_bytes=size, sha256=sha,
                                disposition="kept", response_headers=remote_headers)
                    return "verified_download"
                except (requests.RequestException, OSError, ValueError) as error:
                    # Exception text can contain signed redirects; log type/status, never response bodies.
                    status = getattr(getattr(error, "response", None), "status_code", None)
                    self.record(row, state="failed_attempt", attempt=attempt,
                                actual_bytes=partial.stat().st_size if partial.exists() else 0, sha256=None,
                                disposition="kept_partial" if partial.exists() else "not_downloaded",
                                error_type=type(error).__name__, http_status=status,
                                error_reason=str(error) if isinstance(error, ValueError) else None,
                                response_headers=remote_headers)
                    if isinstance(error, requests.ConnectionError) and hasattr(LOCAL, "session"):
                        LOCAL.session.close()
                        del LOCAL.session
                    if attempt < 3:
                        time.sleep(2 ** attempt)
            return "failed"
        finally:
            with LOCK:
                self.reserved -= reservation
                current = dest.stat().st_size if dest.exists() else partial.stat().st_size if partial.exists() else 0
                self.used += current - initial


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--execute", action="store_true", help="permit network requests and payload download")
    parser.add_argument("--refresh", action="store_true", help="refresh metadata listings; requires --execute")
    parser.add_argument("--metadata-only", action="store_true", help="plan and optionally refresh without payload downloads")
    parser.add_argument("--include-server-additions", action="store_true", help="include reconciled authoritative CSV document/text additions")
    parser.add_argument("--workers", type=int, default=8)
    parser.add_argument("--peak-gib", type=float, default=37)
    parser.add_argument("--minimum-free-gib", type=float, default=10)
    args = parser.parse_args()
    if args.refresh and not args.execute:
        parser.error("--refresh requires --execute")
    if args.workers < 1 or not 0 < args.peak_gib * GIB <= 40_000_000_000 or args.minimum_free_gib < 1:
        parser.error("positive workers, peak <=40 decimal GB and minimum free >=1 GiB required")
    ROOT.mkdir(parents=True, exist_ok=True)
    rows, snapshots = listings(ROOT, args.refresh)
    selected, restricted, excluded = [], [], []
    for row in rows:
        if row["category"] not in CATEGORIES or not EXTENSION.search(row["file_name"]):
            if row["category"] in CATEGORIES - {"Raw Data"}:
                excluded.append(row)
            continue
        if row.get("restricted"):
            restricted.append(row)
            continue
        if type(row.get("size_bytes")) is not int or row["size_bytes"] < 0:
            raise ValueError("selected listing has unknown or invalid size")
        row["stable_url"] = stable_url(row)
        row["identity"] = hashlib.sha256(json.dumps({key: row.get(key) for key in
            ("investigation", "key", "file_version", "size_bytes", "psi_file_id")}, sort_keys=True).encode()).hexdigest()
        row["local_path"] = f"corpus/incoming/{row['investigation']}/{row['identity']}{Path(row['file_name']).suffix.lower()}"
        selected.append(row)
    if args.include_server_additions:
        for line in (ROOT / "metadata/server_extra_files.jsonl").read_text(encoding="utf-8").splitlines():
            row = json.loads(line)
            if row["investigation"] not in {f"PSI-{number}" for number in IDS} or row.get("restricted"):
                raise ValueError("unapproved server addition investigation or restricted file")
            if row["category"] not in CATEGORIES | {"Calibration"} or not (EXTENSION.search(row["file_name"]) or row["file_name"].lower().endswith(".log")):
                raise ValueError("server addition requires separate archive/media lifecycle")
            if type(row.get("size_bytes")) is not int or row["size_bytes"] < 0:
                raise ValueError("server addition has invalid declared size")
            row["stable_url"] = stable_url(row)
            row["identity"] = hashlib.sha256(json.dumps({key: row.get(key) for key in
                ("investigation", "key", "file_version", "size_bytes", "psi_file_id")}, sort_keys=True).encode()).hexdigest()
            row["local_path"] = f"corpus/incoming/{row['investigation']}/{row['identity']}{Path(row['file_name']).suffix.lower()}"
            selected.append(row)
    unique = {}
    for row in selected:
        if row["identity"] in unique and row != unique[row["identity"]]:
            raise ValueError("conflicting metadata for identical file identity")
        unique[row["identity"]] = row
    duplicate_rows = len(selected) - len(unique)
    selected = list(unique.values())
    if args.include_server_additions:
        reconciliation = json.loads((ROOT / "metadata/server_reconciliation.json").read_text())
        identities = sorted((row["investigation"], str(row.get("psi_file_id", "")), str(row["file_version"]),
                             row["file_name"], row["size_bytes"]) for row in selected)
        identity_sha = hashlib.sha256(json.dumps(identities, separators=(",", ":")).encode()).hexdigest()
        if identity_sha != reconciliation["server_identity_sha256"]:
            raise ValueError("expanded selection does not exactly match the authoritative server identity set")
    selected.sort(key=lambda row: (row["category"] == "Raw Data", IDS.index(int(row["investigation"][4:])), row["key"]))
    totals = collections.defaultdict(lambda: {"files": 0, "declared_bytes": 0})
    for row in selected:
        totals[row["investigation"]]["files"] += 1
        totals[row["investigation"]]["declared_bytes"] += row["size_bytes"]
    plan = dict(created_at=now(), selection=("original filter plus reconciled authoritative server document/text additions"
                if args.include_server_additions else "eight categories plus case-insensitive approved document/text extensions"),
                files=len(selected), declared_bytes=sum(row["size_bytes"] for row in selected),
                duplicate_listing_rows=duplicate_rows,
                include_server_additions=args.include_server_additions,
                restricted_files=len(restricted), per_investigation=dict(totals), metadata_snapshots=snapshots,
                execute=args.execute, peak_bytes=int(args.peak_gib * GIB),
                disk_free_bytes=shutil.disk_usage(ROOT).free, semantic_content_access=False)
    write_json(ROOT / "metadata" / "fetch_plan.json", plan)
    excluded_counts = collections.Counter((row["category"], Path(row["file_name"]).suffix.lower()) for row in excluded)
    write_json(ROOT / "metadata" / "excluded_summary.json", dict(files=len(excluded),
        declared_bytes=sum(row["size_bytes"] or 0 for row in excluded),
        by_category_extension=[dict(category=category, extension=ext, files=count)
            for (category, ext), count in sorted(excluded_counts.items())],
        reason="Outside explicit document/text extension filter; archive/media lifecycle pending"))
    (ROOT / "metadata" / "excluded_document_category_files.jsonl").write_text(
        "".join(json.dumps(row, sort_keys=True) + "\n" for row in excluded), encoding="utf-8")
    (ROOT / "metadata" / "selected_files.jsonl").write_text(
        "".join(json.dumps(row, sort_keys=True) + "\n" for row in selected), encoding="utf-8")
    print(json.dumps({key: plan[key] for key in ("files", "declared_bytes", "restricted_files", "disk_free_bytes", "execute")}), flush=True)
    if not args.execute or args.metadata_only:
        return 0
    # Windows advisory byte lock: a second download process must never touch the same partials.
    import msvcrt
    lock_path = ROOT / "logs" / "active_fetch.lock"
    lock_path.parent.mkdir(parents=True, exist_ok=True)
    lock_handle = lock_path.open("a+b")
    if lock_path.stat().st_size == 0:
        lock_handle.write(b"0")
        lock_handle.flush()
    lock_handle.seek(0)
    try:
        msvcrt.locking(lock_handle.fileno(), msvcrt.LK_NBLCK, 1)
    except OSError:
        lock_handle.close()
        raise RuntimeError("another acquisition process owns this incoming corpus") from None
    write_json(ROOT / "logs/fetch_run_status.json", dict(state="running", pid=os.getpid(), started_at=now(),
               selected_files=len(selected), include_server_additions=args.include_server_additions))
    fetcher = Fetcher(ROOT, int(args.peak_gib * GIB), int(args.minimum_free_gib * GIB))
    counts = collections.Counter()
    start, last = time.monotonic(), time.monotonic()
    with concurrent.futures.ThreadPoolExecutor(max_workers=args.workers) as pool:
        futures = {pool.submit(fetcher.fetch, row): row["identity"] for row in selected}
        for future in concurrent.futures.as_completed(futures):
            counts[future.result()] += 1
            if time.monotonic() - last >= 30:
                print(json.dumps(dict(completed=sum(counts.values()), total=len(selected), states=dict(counts),
                                      disk_free_bytes=shutil.disk_usage(ROOT).free)), flush=True)
                last = time.monotonic()
    result = dict(finished_at=now(), selected_files=len(selected), states=dict(counts),
                  elapsed_seconds=round(time.monotonic() - start, 2),
                  disk_free_bytes=shutil.disk_usage(ROOT).free, semantic_content_access=False)
    write_json(ROOT / "logs" / "fetch_summary.json", result)
    write_json(ROOT / "logs/fetch_run_status.json", dict(state="finished_with_failures" if counts["failed"] else "finished",
               pid=os.getpid(), **result))
    print(json.dumps(result), flush=True)
    return 1 if counts["failed"] or restricted else 0


if __name__ == "__main__":
    raise SystemExit(main())
