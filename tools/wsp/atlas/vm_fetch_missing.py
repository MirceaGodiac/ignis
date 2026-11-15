#!/usr/bin/env python3
"""Runs ON THE VM. Fetches, straight from NASA's public PSI archive, the files of a list that the VM does not
hold yet, into a staging folder, and checks each against the sha256 this lane recorded when it downloaded
the same file. Nothing is written outside the staging folder; nothing of the application is touched.

    python3 vm_fetch_missing.py --list missing.jsonl --dest /srv/ignis/staging/wsp-20261115/raw --workers 48

Standard library only. Resumable: a file already in place with the right hash is skipped.
"""
import argparse
import concurrent.futures
import hashlib
import json
import os
import sys
import threading
import time
import urllib.parse
import urllib.request

BASE = "https://psi.nasa.gov/geode-py/ws"
LOCK = threading.Lock()


def url(inv, name, version):
    q = urllib.parse.urlencode({"file": name, "version": version}, quote_via=urllib.parse.quote)
    return "%s/studies/%s/download?%s" % (BASE, urllib.parse.quote(inv, safe=""), q)


def sha256(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for b in iter(lambda: f.read(1 << 20), b""):
            h.update(b)
    return h.hexdigest()


def fetch(row, dest, retries=5):
    inv, name = row["investigation"], row["key"].split("/")[-1]
    d = os.path.join(dest, inv)
    os.makedirs(d, exist_ok=True)
    out = os.path.join(d, name)
    if os.path.exists(out) and sha256(out) == row["sha256"]:
        return "present", 0
    err = ""
    for attempt in range(retries):
        tmp = out + ".part"
        try:
            req = urllib.request.Request(url(inv, name, row.get("file_version", 1)), headers={"User-Agent": "ignis-corpus-fetch"})
            n = 0
            with urllib.request.urlopen(req, timeout=120) as r, open(tmp, "wb") as f:
                for b in iter(lambda: r.read(1 << 20), b""):
                    f.write(b)
                    n += len(b)
            if sha256(tmp) == row["sha256"]:
                os.replace(tmp, out)
                return "ok", n
            err = "sha256 differs from this lane's copy (%d bytes)" % n
        except Exception as e:  # network errors are retried
            err = repr(e)[:160]
        time.sleep(3 * (attempt + 1))
    try:
        os.remove(out + ".part")
    except OSError:
        pass
    return "failed: " + err, 0


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--list", required=True)
    ap.add_argument("--dest", required=True)
    ap.add_argument("--workers", type=int, default=48)
    a = ap.parse_args()
    rows = [json.loads(l) for l in open(a.list, encoding="utf-8")]
    os.makedirs(a.dest, exist_ok=True)
    log = open(os.path.join(a.dest, "FETCH_LOG.jsonl"), "a", encoding="utf-8")
    t0, done, got = time.time(), [0, 0, 0], [0]

    def one(row):
        status, n = fetch(row, a.dest)
        with LOCK:
            done[0] += 1
            done[1] += status in ("ok", "present")
            done[2] += status.startswith("failed")
            got[0] += n
            log.write(json.dumps({"key": row["key"], "sha256": row["sha256"], "status": status}) + "\n")
            if done[0] % 500 == 0 or done[0] == len(rows):
                log.flush()
                print("%d/%d ok %d failed %d  %.2f GB in %.0f s" % (done[0], len(rows), done[1], done[2], got[0] / 1024 ** 3, time.time() - t0), flush=True)

    with concurrent.futures.ThreadPoolExecutor(a.workers) as ex:
        list(ex.map(one, sorted(rows, key=lambda r: r["size"])))
    log.close()
    print("DONE ok %d failed %d of %d" % (done[1], done[2], len(rows)), flush=True)
    return 0 if done[2] == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
