"""Fetch the PSI file listings and raw listing trees of the 19 flight combustion investigations.

Metadata only: one GET of the investigation record (for its version) and one POST of the file tree
per investigation. No file is downloaded and nothing is opened.

Writes (defaults):
  runs/dev/data_all/raw/samples/_meta/files_PSI-<id>.json   the raw listing tree (folders)
  runs/dev/data_all/manifests/PSI-<id>.listing.jsonl         the flat listing, sorted by key, with stable_url
  runs/dev/data_all/raw/samples/_meta/PSI-<id>.json         the investigation record, bytes as served
  runs/dev/data_all/manifests/PSI-<id>.investigation.json    title, acronym, version, hardware, ... of it

Usage (from the repository root):
  python -I tools/wsp/atlas/fetch_listings.py
"""
import argparse
import hashlib
import json
import sys
import time
from datetime import datetime, timezone
from pathlib import Path
from urllib.parse import quote, urlencode

import requests

ROOT = Path(__file__).resolve().parents[3]
BASE = "https://psi.nasa.gov/geode-py/ws"
IDS = (10, 20, 21, 22, 23, 25, 26, 39, 47, 68, 69, 98, 99, 100, 101, 102, 106, 107, 159)


def stable_url(inv, name, version):
    q = {"file": name, "version": version}
    return f"{BASE}/studies/{quote(inv, safe='')}/download?{urlencode(q, quote_via=quote)}"


def flatten(inv, tree):
    out = []

    def walk(node, category):
        for leaf in node.get("files", []):
            if leaf.get("directory") is False:
                key = leaf["key"]
                name = key.rsplit("/", 1)[-1]
                v = int(leaf["version"])
                out.append(dict(investigation=inv, category=category, key=key, file_name=name,
                                size_bytes=int(leaf["bytes_size"]), file_version=v,
                                psi_file_id=leaf.get("id"), updated=leaf.get("updated"),
                                restricted=bool(leaf.get("restricted") or leaf.get("restrictedAccess")),
                                stable_url=stable_url(inv, name, v)))
            else:
                walk(leaf, category)

    for cat in tree["files"]:
        walk(cat, cat["displayName"])
    return sorted(out, key=lambda r: r["key"])


def get(s, method, url, **kw):
    for attempt in range(1, 6):
        try:
            r = s.request(method, url, timeout=(30, 300), **kw)
            r.raise_for_status()
            return r.json()
        except (requests.RequestException, ValueError):
            if attempt == 5:
                raise
            time.sleep(2 ** attempt)


def get_bytes(s, url):
    for attempt in range(1, 6):
        try:
            r = s.get(url, timeout=(30, 300))
            r.raise_for_status()
            json.loads(r.content)
            return r.content
        except (requests.RequestException, ValueError):
            if attempt == 5:
                raise
            time.sleep(2 ** attempt)


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--manifests", default=str(ROOT / "runs/dev/data_all/manifests"))
    ap.add_argument("--meta", default=str(ROOT / "runs/dev/data_all/raw/samples/_meta"))
    a = ap.parse_args(argv)
    man, meta = Path(a.manifests), Path(a.meta)
    man.mkdir(parents=True, exist_ok=True)
    meta.mkdir(parents=True, exist_ok=True)
    s = requests.Session()
    s.headers.update({"User-Agent": "research-data-client/1.0"})
    for n in IDS:
        inv = f"PSI-{n}"
        rec = get_bytes(s, f"{BASE}/repo/investigations/{inv}")
        (meta / f"{inv}.json").write_bytes(rec)
        m = json.loads(rec)
        version = int(m["version"])
        tree = get(s, "POST", f"{BASE}/files/v2",
                   json={"folder": "", "fileType": "study", "studyId": inv, "version": version})
        (meta / f"files_{inv}.json").write_text(json.dumps(tree), encoding="utf-8")
        rows = flatten(inv, tree)
        with (man / f"{inv}.listing.jsonl").open("w", encoding="utf-8", newline="\n") as f:
            for r in rows:
                f.write(json.dumps(r, sort_keys=True) + "\n")
        info = dict(investigation=inv, title=m.get("title"), acronym=m.get("investigationAcronym"),
                    investigation_version=str(m["version"]), version_reason=m.get("versionReason"),
                    modified=m.get("modifiedDate"), doi=m.get("doi"), license=m.get("licenseIdentifier"),
                    license_link=m.get("licenseLink"), hardware=m.get("experimentHardware"),
                    total_file_size_bytes=m.get("totalFileSize"),
                    metadata_url_sha256=hashlib.sha256(rec).hexdigest(),
                    retrieved_at=datetime.now(timezone.utc).isoformat())
        (man / f"{inv}.investigation.json").write_text(json.dumps(info, indent=1, ensure_ascii=False) + "\n",
                                                       encoding="utf-8")
        print(json.dumps(dict(investigation=inv, version=version, files=len(rows),
                              bytes=sum(r["size_bytes"] for r in rows))), flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
