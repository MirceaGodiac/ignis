"""Explicit-network header-only diagnostics for failed PSI corpus downloads."""
import argparse
import json
from urllib.parse import parse_qsl, urlencode, urlsplit, urlunsplit, unquote

import requests

from fetch_public import ROOT, now, session, write_json


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--execute", action="store_true")
    parser.add_argument("--refresh-existing", action="store_true", help="repeat previously recorded identity/version header probes")
    args = parser.parse_args()
    failures = json.loads((ROOT / "logs/acquisition_failures.json").read_text())
    print(json.dumps(dict(failed_files=len(failures), execute=args.execute)))
    if not args.execute:
        return
    output_path = ROOT / "logs/header_probes.json"
    probes = json.loads(output_path.read_text()) if output_path.exists() else []
    seen = {(row["identity"], row["variant"]) for row in probes}
    for row in failures:
        url = row["stable_url"]
        parts = urlsplit(url)
        unversioned = urlunsplit(parts._replace(query=urlencode([(k, v) for k, v in parse_qsl(parts.query) if k != "version"])))
        variants = [("declared_version", url), ("latest_no_version", unversioned)]
        prefix = "psi-data/" + row["investigation"] + "/" + row["category"] + "/"
        if row["key"].startswith(prefix) and row["key"][len(prefix):] != row["file_name"]:
            key_name = row["key"][len(prefix):]
            key_query = [(k, key_name if k == "file" else v) for k, v in parse_qsl(parts.query)]
            variants.append(("metadata_key_filename", urlunsplit(parts._replace(query=urlencode(key_query)))))
        for variant, target in variants:
            if not args.refresh_existing and (row["identity"], variant) in seen:
                continue
            entry = {key: row[key] for key in ("identity", "investigation", "file_name", "key", "file_version", "size_bytes")}
            entry.update(timestamp=now(), variant=variant, requested_url=target, body_bytes_read=0)
            try:
                with session().get(target, stream=True, timeout=(30, 120)) as response:
                    final = urlsplit(response.url)
                    entry.update(http_status=response.status_code, final_host=final.netloc,
                                 final_path=unquote(final.path),
                                 response_headers={key: response.headers[key] for key in
                                     ("Content-Length", "Content-Type", "Content-Disposition", "ETag", "Last-Modified") if key in response.headers})
            except requests.RequestException as error:
                entry["error_type"] = type(error).__name__
            probes.append(entry)
            write_json(output_path, probes)
            print(json.dumps({key: entry.get(key) for key in ("investigation", "file_name", "variant", "http_status", "error_type")}), flush=True)


if __name__ == "__main__":
    main()
