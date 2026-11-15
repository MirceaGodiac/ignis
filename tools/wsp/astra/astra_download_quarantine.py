"""Keep an opaque size-disagreeing response separately; never qualify it as a verified listing file."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil

import requests

from fetch_public import ROOT, GIB, digest, now, session, write_json


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--execute", action="store_true")
    args = parser.parse_args()
    probes = json.loads((ROOT / "logs/header_probes.json").read_text())
    latest = {row["identity"]: row for row in probes if row["variant"] == "declared_version"}
    selected = [row for row in latest.values() if
                row.get("http_status") == 200 and row.get("response_headers", {}).get("Content-Length") and
                int(row["response_headers"]["Content-Length"]) != row["size_bytes"]]
    print(json.dumps(dict(size_disagreeing_files=len(selected), execute=args.execute)), flush=True)
    if not args.execute:
        return
    receipts = []
    receipt_path = ROOT / "logs/size_mismatch_quarantine.json"
    if receipt_path.exists():
        receipts = json.loads(receipt_path.read_text())
    for row in selected:
        dest = ROOT / "corpus/quarantine" / row["investigation"] / (row["identity"] + Path(row["file_name"]).suffix.lower())
        prior = next((receipt for receipt in reversed(receipts) if receipt["identity"] == row["identity"]), None)
        if dest.exists():
            size, sha = digest(dest)
            if prior and (size, sha) == (prior["actual_bytes"], prior["sha256"]):
                print(json.dumps(dict(identity=row["identity"], state="quarantine_readback_verified")), flush=True)
                continue
            raise RuntimeError("existing quarantine file has no matching verified receipt")
        expected_http = int(row["response_headers"]["Content-Length"])
        used = sum(path.stat().st_size for path in ROOT.rglob("*") if path.is_file())
        # Reserve the entire selected corpus while its independent bounded downloader is active.
        plan = json.loads((ROOT / "metadata/fetch_plan.json").read_text())
        if used + plan["declared_bytes"] + expected_http >= 37 * GIB:
            raise RuntimeError("quarantine peak-space reservation refused")
        if shutil.disk_usage(ROOT).free - plan["declared_bytes"] - expected_http < 10 * GIB:
            raise RuntimeError("quarantine free-space reservation refused")
        dest.parent.mkdir(parents=True, exist_ok=True)
        partial = dest.with_suffix(dest.suffix + ".part")
        streamed, stream_hash = 0, hashlib.sha256()
        with session().get(row["requested_url"], stream=True, timeout=(30, 120)) as response:
            response.raise_for_status()
            if int(response.headers.get("Content-Length", -1)) != expected_http:
                raise RuntimeError("HTTP length changed since metadata-only probe")
            with partial.open("wb") as output:
                for chunk in response.iter_content(1024 * 1024):
                    streamed += len(chunk)
                    if streamed > expected_http or shutil.disk_usage(ROOT).free < 10 * GIB:
                        raise RuntimeError("quarantine stream exceeded its resource reservation")
                    output.write(chunk)
                    stream_hash.update(chunk)
                output.flush()
                os.fsync(output.fileno())
        size, sha = digest(partial)
        if size != expected_http or sha != stream_hash.hexdigest():
            raise RuntimeError("quarantine stream/readback verification failed")
        partial.replace(dest)
        receipt = dict(row, timestamp=now(), local_path=str(dest.relative_to(ROOT)),
                       state="transport_verified_listing_size_mismatch", disposition="kept_quarantined",
                       actual_bytes=size, body_bytes_read=size, sha256=sha, semantic_content_access=False,
                       eligible_as_verified_listing_file=False)
        receipts.append(receipt)
        write_json(receipt_path, receipts)
        print(json.dumps({key: receipt[key] for key in ("investigation", "file_name", "state", "size_bytes", "actual_bytes", "sha256")}), flush=True)


if __name__ == "__main__":
    try:
        main()
    except requests.RequestException as error:
        # Requests' exception string can contain the expiring signed redirect URL.
        failure = dict(timestamp=now(), state="quarantine_transport_failed", error_type=type(error).__name__,
                       http_status=getattr(getattr(error, "response", None), "status_code", None),
                       partials=[dict(local_path=str(path.relative_to(ROOT)), actual_bytes=path.stat().st_size,
                                      sha256=None, disposition="kept_partial")
                                 for path in (ROOT / "corpus/quarantine").rglob("*.part")],
                       semantic_content_access=False)
        write_json(ROOT / "logs/quarantine_transport_error.json", failure)
        print(json.dumps(failure), flush=True)
        raise SystemExit(1) from None
