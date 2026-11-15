"""Acquire exact metadata-key filename aliases after successful zero-body probes."""
import argparse
import json
import msvcrt
from urllib.parse import parse_qsl, urlsplit

from fetch_public import ROOT, Fetcher, GIB, now, stable_url, write_json


def eligible_aliases(rows, probes):
    latest = {probe["identity"]: probe for probe in probes if probe["variant"] == "metadata_key_filename"}
    selected = []
    for identity, probe in latest.items():
        row = rows[identity]
        prefix = "psi-data/" + row["investigation"] + "/" + row["category"] + "/"
        if not row["key"].startswith(prefix):
            raise ValueError("alias is not derived from the selected metadata key")
        target = stable_url(dict(row, file_name=row["key"][len(prefix):]))
        actual, expected = urlsplit(probe["requested_url"]), urlsplit(target)
        if (actual.scheme, actual.netloc, actual.path, sorted(parse_qsl(actual.query))) != (
                expected.scheme, expected.netloc, expected.path, sorted(parse_qsl(expected.query))):
            raise ValueError("probe URL differs from exact versioned metadata-key alias")
        if (probe.get("http_status") != 200 or not probe.get("final_host", "").endswith(".amazonaws.com") or
                probe.get("response_headers", {}).get("Content-Length") != str(row["size_bytes"])):
            continue
        selected.append(dict(row, listing_stable_url=row["stable_url"], stable_url=target,
                             acquisition_url_basis="exact metadata key filename; same selected identity and version"))
    return selected


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--execute", action="store_true")
    args = parser.parse_args()
    rows = {row["identity"]: row for row in
            (json.loads(line) for line in (ROOT / "metadata/selected_files.jsonl").read_text().splitlines())}
    probes = json.loads((ROOT / "logs/header_probes.json").read_text())
    selected = eligible_aliases(rows, probes)
    print(json.dumps(dict(eligible_alias_files=len(selected), execute=args.execute)), flush=True)
    if not args.execute:
        return 0
    with (ROOT / "logs/active_fetch.lock").open("a+b") as lock:
        if lock.tell() == 0:
            lock.write(b"0")
            lock.flush()
        lock.seek(0)
        msvcrt.locking(lock.fileno(), msvcrt.LK_NBLCK, 1)
        fetcher = Fetcher(ROOT, 37 * GIB, 10 * GIB)
        results = [dict(identity=row["identity"], state=fetcher.fetch(row)) for row in selected]
    write_json(ROOT / "logs/alias_recovery.json", dict(timestamp=now(), results=results,
                                                      semantic_content_access=False))
    print(json.dumps(results), flush=True)
    return int(any(result["state"] == "failed" for result in results))


if __name__ == "__main__":
    raise SystemExit(main())
