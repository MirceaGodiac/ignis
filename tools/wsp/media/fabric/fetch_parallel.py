"""Parallel byte-range download of the remaining burn videos (PSI-26 or PSI-25), from the archive's S3
redirect, 8 MB ranges over many connections. Remote / fetch_range / fetch_file copied from
tools/wsp/media/fetch_flex2_batch.py. Appends to <root>/manifest_raw.jsonl like fetch_videos.py.
Run with `python -I`.

    python -I tools/wsp/media/fabric/fetch_parallel.py --root runs/dev/media/fabric --only PSI-26 --connections 64
"""
import argparse
import concurrent.futures
import hashlib
import json
import shutil
import threading
import time
from pathlib import Path
from urllib.parse import quote, urlencode

import requests

REPO = Path(__file__).resolve().parents[4]
BASE = "https://psi.nasa.gov/geode-py/ws"
CHUNK = 8 * 1024 * 1024
LOCAL = threading.local()
def session():
    if not hasattr(LOCAL, "s"):
        LOCAL.s = requests.Session()
    return LOCAL.s


def psi_url(row):
    q = {"file": row["file_name"], "version": row["file_version"]}
    return f"{BASE}/studies/{quote(row['investigation'], safe='')}/download?{urlencode(q, quote_via=quote)}"


class Remote:
    """The S3 address of one file, resolved from the archive's redirect and renewed when it expires."""

    def __init__(self, row):
        self.row, self.lock, self.url, self.at = row, threading.Lock(), None, 0.0

    def get(self, force=False):
        with self.lock:
            if force or self.url is None or time.time() - self.at > 600:
                r = session().get(psi_url(self.row), allow_redirects=False, timeout=60)
                self.url = r.headers.get("Location") or psi_url(self.row)
                self.at = time.time()
            return self.url


def fetch_range(remote, fh, fh_lock, a, b, retries=8):
    for attempt in range(retries):
        try:
            r = session().get(remote.get(force=attempt > 0), headers={"Range": f"bytes={a}-{b}"}, timeout=(30, 180))
            if r.status_code == 206 and len(r.content) == b - a + 1:
                with fh_lock:
                    fh.seek(a)
                    fh.write(r.content)
                return True
        except Exception:
            pass
        time.sleep(2 * (attempt + 1))
    return False


def sha256(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 22), b""):
            h.update(chunk)
    return h.hexdigest()


def fetch_file(row, dest, pool, min_free_gb):
    size = row["size_bytes"]
    if dest.exists() and dest.stat().st_size == size:
        return "present"
    if shutil.disk_usage(dest.parent).free < min_free_gb * 1024 ** 3:
        return "stopped: disk floor"
    part = dest.with_suffix(dest.suffix + ".part")
    remote, fh_lock = Remote(row), threading.Lock()
    with open(part, "wb") as fh:
        fh.truncate(size)
        ranges = [(a, min(a + CHUNK, size) - 1) for a in range(0, size, CHUNK)]
        ok = list(pool.map(lambda ab: fetch_range(remote, fh, fh_lock, ab[0], ab[1]), ranges))
    if all(ok) and part.stat().st_size == size:
        part.replace(dest)
        return "ok"
    part.unlink(missing_ok=True)
    return f"failed: {ok.count(False)} of {len(ok)} ranges"



def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", required=True)
    ap.add_argument("--only", required=True)
    ap.add_argument("--connections", type=int, default=64)
    ap.add_argument("--min-free-gb", type=float, default=150)
    args = ap.parse_args()
    root = Path(args.root)
    imp = __import__("importlib.util").util
    spec = imp.spec_from_file_location("fv", Path(__file__).with_name("fetch_videos.py"))
    fv = imp.module_from_spec(spec)
    spec.loader.exec_module(fv)
    rows = [json.loads(l) for l in open(REPO / "runs/dev/data_all/manifests" / f"{args.only}.listing.jsonl", encoding="utf-8")]
    want = fv.wanted(args.only, rows)
    man = root / "manifest_raw.jsonl"
    done = {json.loads(l)["file_name"] for l in man.read_text(encoding="utf-8").splitlines()} if man.exists() else set()
    d = root / "raw" / args.only.lower().replace("-", "")
    d.mkdir(parents=True, exist_ok=True)
    with concurrent.futures.ThreadPoolExecutor(args.connections) as pool:
        for r in want:
            if r["file_name"] in done:
                continue
            dest = d / r["file_name"]
            st = fetch_file(r, dest, pool, args.min_free_gb)
            print(r["file_name"], st, flush=True)
            if st.startswith("stopped"):
                break
            if st in ("ok", "present"):
                rec = {"investigation": args.only, "file_name": r["file_name"], "key": r["key"], "stable_url": r["stable_url"],
                       "size_bytes": r["size_bytes"], "sha256": sha256(dest), "local": str(dest.relative_to(root)).replace("\\", "/")}
                with open(man, "a", encoding="utf-8") as f:
                    f.write(json.dumps(rec) + "\n")


if __name__ == "__main__":
    main()
