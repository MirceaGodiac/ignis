"""Download the backlit archives (<test>A.zip) of a frozen FLEX-2 image batch, and verify archives on disk.

Opaque, size-checked downloads of public PSI-68 files named in the listing; nothing is opened or
extracted except to verify the zip container. The batch is a file frozen in git before any frame is
fetched (docs/wsp/atlas/splits/PSI-68-batch1.json, written by tools/wsp/atlas/freeze_flex2_days.py).

NASA's archive serves each file from S3 at about 0.2 to 0.3 MB/s per connection (measured 15 November:
8 connections 2.3 MB/s, 32 connections 7.6, 64 connections 11.6), and S3 honours range requests. So a
file is fetched as many byte ranges in parallel: `--connections` in all, over `--files` files at a time,
so that files complete one after another and a measurer can start on the first ones.

Integrity (fixed 15 November after two corrupt archives of the right size were written in batch 2:
A184D03A.zip did not start with a zip header, A186D02A.zip had its bytes shifted by 3 MiB):
  * every range response must be 206 with a `Content-Range: bytes a-b/size` equal to the requested range
    and the file's listed size, and a body of exactly b - a + 1 bytes; anything else is retried with a
    freshly resolved address (a 200 with the whole file, a range of another file, an expired redirect, an
    error page of the right length are all refused);
  * after assembly, before the file is renamed and before its manifest line is written, the zip is
    verified: end-of-central-directory record, central directory, every member's local header signature
    at its offset, and the CRC-32 of every member (`zipfile.testzip`). A file that fails is fetched once
    more from scratch; if it fails again it is recorded as `failed: zip ...` and not kept. NASA's listing
    gives no checksum, so the zip's own CRCs are the check;
  * the manifest records `verified` (the check that passed) next to the sha256 of the verified file.

    python -I tools/wsp/media/fetch_flex2_batch.py --batch docs/wsp/atlas/splits/PSI-68-batch1.json --out runs/dev/media/droplet/flex2_raw
    python -I tools/wsp/media/fetch_flex2_batch.py verify --dir runs/dev/media/droplet/flex2_b2_raw [--quick] [--out report.json]

`verify` is read-only: it opens each finished `*.zip` in the folder (never a `.part`), checks its size
against the listing and the manifest, and runs the same zip check; it writes nothing unless `--out` names
a report file.
"""
import argparse
import concurrent.futures
import hashlib
import json
import re
import shutil
import struct
import sys
import threading
import time
import zipfile
import zlib
from pathlib import Path
from urllib.parse import quote, urlencode

import requests

REPO = Path(__file__).resolve().parents[3]
LISTING = REPO / "runs/dev/data_all/manifests/PSI-68.listing.jsonl"
BASE = "https://psi.nasa.gov/geode-py/ws"
CHUNK = 8 * 1024 * 1024
LOCK = threading.Lock()
LOCAL = threading.local()
CONTENT_RANGE = re.compile(r"bytes\s+(\d+)-(\d+)/(\d+|\*)")


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


def range_ok(r, a, b, size):
    """Why a range response is not the bytes [a, b] of a file of `size` bytes; None when it is."""
    if r.status_code != 206:
        return f"status {r.status_code}"
    m = CONTENT_RANGE.fullmatch((r.headers.get("Content-Range") or "").strip())
    if not m:
        return f"Content-Range {r.headers.get('Content-Range')!r}"
    ra, rb, total = int(m.group(1)), int(m.group(2)), m.group(3)
    if (ra, rb) != (a, b):
        return f"Content-Range {ra}-{rb} for requested {a}-{b}"
    if total != "*" and int(total) != size:
        return f"Content-Range total {total} for listed size {size}"
    if len(r.content) != b - a + 1:
        return f"body {len(r.content)} bytes for {b - a + 1}"
    return None


def fetch_range(remote, fh, fh_lock, a, b, size, retries=8, log=None):
    for attempt in range(retries):
        try:
            r = session().get(remote.get(force=attempt > 0), headers={"Range": f"bytes={a}-{b}"}, timeout=(30, 180))
            why = range_ok(r, a, b, size)
            if why is None:
                with fh_lock:
                    fh.seek(a)
                    fh.write(r.content)
                return True
            if log is not None:
                log.append(why)
        except Exception as e:  # network errors: retry
            if log is not None:
                log.append(type(e).__name__)
        time.sleep(2 * (attempt + 1))
    return False


def sha256(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 22), b""):
            h.update(chunk)
    return h.hexdigest()


def verify_zip(path, quick=False):
    """(ok, detail). Checks the zip container: central directory, each member's local header signature at
    its offset, and (unless quick) the CRC-32 of every member's data."""
    try:
        with open(path, "rb") as f:
            head = f.read(4)
        if head != b"PK\x03\x04":
            return False, f"does not start with a local file header (first bytes {head.hex()})"
        with zipfile.ZipFile(path) as z:
            infos = z.infolist()
            if not infos:
                return False, "central directory lists no member"
            with open(path, "rb") as f:
                for i in infos:
                    f.seek(i.header_offset)
                    sig = f.read(4)
                    if sig != b"PK\x03\x04":
                        return False, f"member {i.filename}: no local header at offset {i.header_offset} ({sig.hex()})"
            if quick:
                return True, f"directory and {len(infos)} local headers"
            bad = z.testzip()
            if bad is not None:
                return False, f"CRC mismatch in member {bad}"
            return True, f"directory, {len(infos)} local headers and CRCs"
    except (zipfile.BadZipFile, zlib.error, OSError, EOFError, struct.error) as e:
        return False, f"{type(e).__name__}: {e}"


def fetch_file(row, dest, pool, min_free_gb, attempts=2):
    size = row["size_bytes"]
    if dest.exists() and dest.stat().st_size == size:
        ok, detail = verify_zip(dest)
        if ok:
            return "present", detail
        dest.unlink()  # a corrupt archive of the right size from an earlier run: fetched again
    if shutil.disk_usage(dest.parent).free < min_free_gb * 1024 ** 3:
        return "stopped: disk floor", None
    part = dest.with_suffix(dest.suffix + ".part")
    detail = None
    for attempt in range(attempts):
        remote, fh_lock, log = Remote(row), threading.Lock(), []
        with open(part, "wb") as fh:
            fh.truncate(size)
            ranges = [(a, min(a + CHUNK, size) - 1) for a in range(0, size, CHUNK)]
            ok = list(pool.map(lambda ab: fetch_range(remote, fh, fh_lock, ab[0], ab[1], size, log=log), ranges))
        if not (all(ok) and part.stat().st_size == size):
            part.unlink(missing_ok=True)
            return f"failed: {ok.count(False)} of {len(ok)} ranges ({'; '.join(sorted(set(log))[:5])})", None
        good, detail = verify_zip(part)
        if good:
            part.replace(dest)
            return "ok", detail
        part.unlink(missing_ok=True)
    return f"failed: zip check ({detail})", None


def listing_rows(names):
    rows = {}
    for line in LISTING.open(encoding="utf-8"):
        r = json.loads(line)
        name = r["file_name"].split("_")[-1]
        if name in names and (name not in rows or r["file_version"] > rows[name]["file_version"]):
            rows[name] = r
    return rows


def cmd_verify(argv):
    ap = argparse.ArgumentParser(prog="fetch_flex2_batch.py verify")
    ap.add_argument("--dir", required=True)
    ap.add_argument("--quick", action="store_true", help="directory and local headers only, no CRC pass")
    ap.add_argument("--out", help="write the report here (JSON); nothing is written otherwise")
    a = ap.parse_args(argv)
    d = Path(a.dir)
    files = sorted(p for p in d.glob("*.zip"))
    rows = listing_rows({p.name for p in files})
    man = {}
    if (d / "MANIFEST.jsonl").exists():
        for line in (d / "MANIFEST.jsonl").open(encoding="utf-8"):
            r = json.loads(line)
            man[r["archive"]] = r
    report = []
    for p in files:
        rec = dict(archive=p.name)
        try:
            size = p.stat().st_size
        except FileNotFoundError:  # removed by the measurer while we looked
            rec.update(ok=None, detail="removed during the check")
            report.append(rec)
            continue
        listed = rows.get(p.name, {}).get("size_bytes")
        rec.update(size=size, listed_size=listed, size_ok=(listed == size) if listed else None,
                   manifest_status=man.get(p.name, {}).get("status"), manifest_verified=man.get(p.name, {}).get("verified"))
        t0 = time.time()
        ok, detail = verify_zip(p, quick=a.quick)
        rec.update(ok=bool(ok and rec["size_ok"] is not False), detail=detail, seconds=round(time.time() - t0, 1))
        report.append(rec)
        print(f"{p.name}: {'OK' if rec['ok'] else 'FAIL'} - {detail}", flush=True)
    bad = [r for r in report if r["ok"] is False]
    print(f"checked {len(report)} archives: {len(report) - len(bad)} ok or gone, {len(bad)} fail: {[r['archive'] for r in bad]}")
    if a.out:
        Path(a.out).write_text(json.dumps(report, indent=1) + "\n", encoding="utf-8")
    return 1 if bad else 0


def main():
    if len(sys.argv) > 1 and sys.argv[1] == "verify":
        return cmd_verify(sys.argv[2:])
    ap = argparse.ArgumentParser()
    ap.add_argument("--batch", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--connections", type=int, default=96)
    ap.add_argument("--files", type=int, default=6, help="files in flight at a time")
    ap.add_argument("--min-free-gb", type=float, default=150.0)
    args = ap.parse_args()
    batch = json.loads(Path(args.batch).read_text(encoding="utf-8"))
    want = {t["archive"]: t for t in batch["tests"]}
    rows = listing_rows(set(want))
    missing = sorted(set(want) - set(rows))
    out = Path(args.out)
    out.mkdir(parents=True, exist_ok=True)
    for stale in out.glob("*.part"):
        stale.unlink()
    manifest = out / "MANIFEST.jsonl"
    known = set()
    if manifest.exists():
        # only verified archives count as known; an unverified "ok" of an earlier run is checked again
        known = {json.loads(l)["archive"] for l in manifest.open(encoding="utf-8") if '"verified"' in l}
    t0, done = time.time(), []
    pool = concurrent.futures.ThreadPoolExecutor(args.connections)

    def one(name):
        row, dest = rows[name], out / name
        status, verified = fetch_file(row, dest, pool, args.min_free_gb)
        rec = {"archive": name, "test_id": want[name]["test_id"], "partition": want[name]["partition"],
               "key": row["key"], "size_bytes": row["size_bytes"], "status": status}
        if status in ("ok", "present"):
            rec["verified"] = verified
            rec["sha256"] = sha256(dest)
        with LOCK:
            done.append(rec)
            if name not in known or status not in ("ok", "present"):
                with manifest.open("a", encoding="utf-8") as f:
                    f.write(json.dumps(rec) + "\n")
            gb = sum(d["size_bytes"] for d in done if d["status"] == "ok") / 1024 ** 3
            dt = time.time() - t0
            print(f"{len(done)}/{len(rows)} {name} {status}; fetched {gb:.1f} GB in {dt:.0f} s ({gb * 1024 / max(dt, 1):.1f} MB/s)", flush=True)

    with concurrent.futures.ThreadPoolExecutor(args.files) as ex:
        list(ex.map(one, sorted(rows, key=lambda n: rows[n]["size_bytes"])))
    pool.shutdown()
    bad = [d for d in done if d["status"] not in ("ok", "present")]
    print(f"done: {len(done) - len(bad)} ok, {len(bad)} not ok, {len(missing)} not in the listing: {missing}")
    return 0 if not bad and not missing else 1


if __name__ == "__main__":
    sys.exit(main())
