"""Metadata-only recovery audit of unresolved NASA object identities."""
import argparse
import json
import re
import msvcrt
from urllib.parse import urljoin, urlsplit, urlencode, unquote

import requests

from fetch_public import ROOT, BASE, Fetcher, GIB, now, session, write_json


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--execute", action="store_true")
    parser.add_argument("--action", choices=("metadata", "client_scripts", "id_recover", "version_aliases", "historical_listings"), required=True)
    args = parser.parse_args()
    if not args.execute:
        print(json.dumps(dict(execute=False, action=args.action)))
        return 0
    target = ROOT / "metadata/recovery_v2"
    target.mkdir(parents=True, exist_ok=True)
    if args.action == "historical_listings":
        unresolved = [json.loads(line) for line in (ROOT / "logs/acquisition_unresolved.jsonl").read_text().splitlines()]
        unresolved = [r for r in unresolved if r.get("unresolved_reason") == "HTTP_404"]
        output = []
        def leaves(nodes):
            for node in nodes:
                if node.get("files"):
                    yield from leaves(node["files"])
                elif node.get("id"):
                    yield node
        for inv in sorted({r["investigation"] for r in unresolved}):
            selected = [r for r in unresolved if r["investigation"] == inv]
            meta = json.loads((target / (inv + ".version_metadata.json")).read_text())
            for version in range(1, int(meta["version"]) + 1):
                response = session().post(BASE + "/files/v2", json={"folder": "", "fileType": "study",
                                          "studyId": inv, "version": version}, timeout=(20, 60))
                response.raise_for_status()
                nodes = list(leaves(response.json().get("files", [])))
                for row in selected:
                    matches = [{key: n.get(key) for key in ("id", "key", "version", "bytes_size", "status", "visible", "restricted")}
                               for n in nodes if n.get("id") == row["psi_file_id"] or n.get("key") == row["key"]]
                    output.append(dict(identity=row["identity"], investigation=inv, study_version=version,
                                       matches=matches, timestamp=now(), semantic_content_access=False))
                write_json(ROOT / "logs/historical_listing_probes.json", output)
                print(json.dumps(dict(investigation=inv, study_version=version, checked=len(selected))), flush=True)
        return 0
    if args.action == "version_aliases":
        prior = {r["identity"]:r for r in json.loads((ROOT / "logs/id_recovery_probes.json").read_text())}
        rows = [json.loads(line) for line in (ROOT / "metadata/selected_files.jsonl").read_text().splitlines()]
        log = ROOT / "logs/version_alias_probes.json"
        probes = json.loads(log.read_text()) if log.exists() else []
        seen = {(r["identity"],r["variant"]) for r in probes}
        recovered = {r["identity"] for r in probes if r.get("recovery_state") in
                     ("verified_download", "verified_existing")}
        with (ROOT / "logs/active_fetch.lock").open("a+b") as lock:
            lock.seek(0)
            msvcrt.locking(lock.fileno(), msvcrt.LK_NBLCK, 1)
            fetcher = Fetcher(ROOT, 37 * GIB, 10 * GIB)
            for row in rows:
                previous = prior.get(row["identity"], {})
                if previous.get("http_status") != 404 or row["identity"] in recovered:
                    continue
                meta = json.loads((target / (row["investigation"] + ".version_metadata.json")).read_text())
                variants = [("study_version_"+str(v), {"file":row["psi_file_id"],"version":row["file_version"],"studyVersion":v})
                            for v in range(int(meta["version"]),0,-1)]
                variants.extend([("full_key", {"file":row["key"],"version":row["file_version"]}),
                                 ("remote_basename", {"file":unquote(previous["final_path"].rsplit("/",1)[-1]),"version":row["file_version"]})])
                for variant, query in variants:
                    if (row["identity"],variant) in seen:
                        continue
                    url = BASE + "/studies/" + row["investigation"] + "/download?" + urlencode(query)
                    entry = dict(identity=row["identity"], investigation=row["investigation"], variant=variant,
                                 requested_url=url, timestamp=now(), body_bytes_read=0)
                    try:
                        with session().get(url,stream=True,timeout=(20,45)) as response:
                            final=urlsplit(response.url)
                            entry.update(http_status=response.status_code,final_host=final.hostname,final_path=final.path,
                                         response_headers={k:response.headers[k] for k in ("Content-Length","ETag","Content-Type") if k in response.headers})
                    except requests.RequestException as error:
                        entry["error_type"]=type(error).__name__
                    probes.append(entry);write_json(log,probes)
                    eligible=(entry.get("http_status")==200 and (entry.get("final_host") or "").endswith(".amazonaws.com") and
                              entry.get("response_headers",{}).get("Content-Length")==str(row["size_bytes"]))
                    if eligible:
                        entry["recovery_state"]=fetcher.fetch(dict(row,stable_url=url,listing_stable_url=row["stable_url"],acquisition_url_basis=variant))
                        write_json(log,probes)
                    print(json.dumps({k:entry.get(k) for k in ("identity","variant","http_status","error_type","recovery_state")}),flush=True)
                    if entry.get("recovery_state") in ("verified_download","verified_existing"):
                        break
        return 0
    if args.action == "id_recover":
        unresolved = {json.loads(line)["identity"] for line in (ROOT / "logs/acquisition_unresolved.jsonl").read_text().splitlines()}
        rows = [json.loads(line) for line in (ROOT / "metadata/selected_files.jsonl").read_text().splitlines()]
        log = ROOT / "logs/id_recovery_probes.json"
        probes = json.loads(log.read_text()) if log.exists() else []
        with (ROOT / "logs/active_fetch.lock").open("a+b") as lock:
            lock.seek(0)
            msvcrt.locking(lock.fileno(), msvcrt.LK_NBLCK, 1)
            fetcher = Fetcher(ROOT, 37 * GIB, 10 * GIB)
            for row in rows:
                if row["identity"] not in unresolved:
                    continue
                query = {"file": row["psi_file_id"], "version": row["file_version"]}
                url = BASE + "/studies/" + row["investigation"] + "/download?" + urlencode(query)
                entry = dict(identity=row["identity"], investigation=row["investigation"],
                             psi_file_id=row["psi_file_id"], file_version=row["file_version"],
                             requested_url=url, timestamp=now(), declared_bytes=row["size_bytes"], body_bytes_read=0,
                             url_basis="public NASA client download.nodeKey=id; exact selected PSI file id and version")
                try:
                    with session().get(url, stream=True, timeout=(20, 60)) as response:
                        final = urlsplit(response.url)
                        entry.update(http_status=response.status_code, final_host=final.hostname,
                                     final_path=final.path, response_headers={k:response.headers[k] for k in
                                      ("Content-Length", "Content-Type", "ETag", "Last-Modified") if k in response.headers})
                except requests.RequestException as error:
                    entry["error_type"] = type(error).__name__
                probes.append(entry)
                write_json(log, probes)
                eligible = (entry.get("http_status") == 200 and (entry.get("final_host") or "").endswith(".amazonaws.com") and
                            entry.get("response_headers", {}).get("Content-Length") == str(row["size_bytes"]))
                if eligible:
                    entry["recovery_state"] = fetcher.fetch(dict(row, stable_url=url, listing_stable_url=row["stable_url"],
                                                                acquisition_url_basis=entry["url_basis"]))
                    write_json(log, probes)
                print(json.dumps({k:entry.get(k) for k in ("identity", "http_status", "error_type", "recovery_state")}), flush=True)
        return 0
    if args.action == "client_scripts":
        response = session().get("https://psi.nasa.gov/physci/repo", timeout=(20, 60))
        response.raise_for_status()
        scripts = re.findall(r'<script[^>]*src=["\']([^"\']+)', response.text)
        output = []
        for source in scripts:
            url = urljoin(response.url, source)
            if urlsplit(url).hostname != "psi.nasa.gov":
                continue
            body = session().get(url, timeout=(20, 60))
            body.raise_for_status()
            name = urlsplit(url).path.rsplit("/", 1)[-1]
            (target / name).write_bytes(body.content)
            output.append(dict(url=url, local_name=name, actual_bytes=len(body.content)))
        write_json(target / "client_scripts.json", output)
        print(json.dumps(output))
        return 0
    rows = [json.loads(line) for line in (ROOT / "logs/acquisition_unresolved.jsonl").read_text().splitlines()]
    for inv in sorted({row["investigation"] for row in rows}):
        path = target / (inv + ".current_listing.json")
        if path.exists():
            print(json.dumps(dict(investigation=inv, state="retained_metadata_snapshot")), flush=True)
            continue
        response = session().get(BASE + "/repo/investigations/" + inv, timeout=(20, 60))
        response.raise_for_status()
        meta = response.json()
        write_json(target / (inv + ".version_metadata.json"),
                   {key: meta[key] for key in ("version", "versions", "id", "accession", "updated") if key in meta})
        response = session().post(BASE + "/files/v2", json={"folder": "", "fileType": "study",
                                 "studyId": inv, "version": meta["version"]}, timeout=(20, 60))
        response.raise_for_status()
        write_json(path, response.json())
        print(json.dumps(dict(investigation=inv, state="metadata_refreshed", investigation_version=meta["version"])), flush=True)
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except requests.RequestException as error:
        result = dict(timestamp=now(), error_type=type(error).__name__,
                      http_status=getattr(getattr(error, "response", None), "status_code", None))
        write_json(ROOT / "logs/recovery_v2_transport_error.json", result)
        print(json.dumps(result))
        raise SystemExit(1) from None
