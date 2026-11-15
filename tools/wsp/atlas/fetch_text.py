"""Download every text, table and document file of the 19 PSI flight combustion investigations.

Opaque byte transfer: no downloaded file is opened, parsed or executed. The selection comes from the
file catalogue (runs/dev/data_all/work/atlas/files.jsonl, written by classify_files.py): rows whose
`medium` is "text, tables and documents" (so no image archive and no video, whatever the kind),
latest version only, repeated listing rows dropped.

Layout:   runs/dev/data_all/raw/<PSI-id>/<listing key without "psi-data/<PSI-id>/">
          (a name that Windows cannot hold, or that differs from another only in case, gets a
          suffix; the manifest's `local_path` says where each file is)
Manifest: runs/dev/data_all/raw/MANIFEST.jsonl, one row per selected file (rewritten at the end and
          every few minutes); every attempt is also appended to raw/_log/receipts.jsonl.

Resumable: a file whose manifest row is `ok` and whose size on disk matches is skipped; a `.part`
is resumed with an HTTP Range request when the server honours it. Retries with exponential backoff.
Stops when the drive has less than --min-free-gb free.

Order: investigations in PRIORITY order, then by key. A server that answers 429/503 or drops the
connection makes every worker pause; a server that serves the same other size twice is taken at that
size and recorded as `size_differs`. When PSI-10 and PSI-23 are complete, DONE_PSI-10_PSI-23.txt is
written beside the manifest.

Usage (from the repository root):
  python -I tools/wsp/atlas/fetch_text.py --dry-run
  python -I tools/wsp/atlas/fetch_text.py --workers 40
"""
import argparse
import concurrent.futures
import hashlib
import json
import random
import re
import shutil
import sys
import threading
import time
from collections import Counter, defaultdict
from datetime import datetime, timezone
from pathlib import Path
from urllib.parse import quote, urlencode

import requests

ROOT = Path(__file__).resolve().parents[3]
DATA = ROOT / "runs/dev/data_all"
LISTINGS = (DATA / "manifests", ROOT / "data/manifests")
REUSE_DIRS = (ROOT / "runs/dev/data_astra/corpus", ROOT / "runs/dev/data_ext", ROOT / "runs/dev/data_docs")
TEXT = "text, tables and documents"
BASE = "https://psi.nasa.gov/geode-py/ws"
LOCAL = threading.local()
LOCK = threading.Lock()
STOP = threading.Event()
PRIORITY = ("PSI-10", "PSI-23", "PSI-159", "PSI-39", "PSI-20", "PSI-21", "PSI-22", "PSI-101", "PSI-102",
            "PSI-47", "PSI-107", "PSI-25", "PSI-26", "PSI-98", "PSI-99", "PSI-100", "PSI-69", "PSI-68",
            "PSI-106")
FINAL = ("ok", "reused", "size_differs", "failed")
MARKERS = {"DONE_PSI-10_PSI-23.txt": ("PSI-10", "PSI-23")}


def now():
    return datetime.now(timezone.utc).isoformat().replace("+00:00", "Z")


def session():
    if not hasattr(LOCAL, "s"):
        s = requests.Session()
        s.headers.update({"User-Agent": "research-data-client/1.0", "Accept-Encoding": "identity"})
        a = requests.adapters.HTTPAdapter(pool_connections=2, pool_maxsize=2)
        s.mount("https://", a)
        LOCAL.s = s
    return LOCAL.s


def sha256_of(path):
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def safe_component(c):
    bad = '<>:"|?*\\'
    out = "".join("_" if ch in bad or ord(ch) < 32 else ch for ch in c)
    if out.rstrip(" .") != out:
        out = out.rstrip(" .") + "_"
    return out or "_"


def select(rows_path):
    keys = {}
    for d in LISTINGS:
        for p in sorted(d.glob("PSI-*.listing.jsonl")):
            for line in p.open(encoding="utf-8"):
                r = json.loads(line)
                keys.setdefault((r["investigation"], r["psi_file_id"], int(r["file_version"])), r["key"])
    sel = []
    for line in rows_path.open(encoding="utf-8"):
        r = json.loads(line)
        if r["medium"] != TEXT or not r["latest_version"] or r.get("repeat_of_row") is not None:
            continue
        inv = r["investigation"]
        key = keys[(inv, r["psi_file_id"], int(r["file_version"]))]
        rel = key.split("/", 2)[2] if key.startswith("psi-data/%s/" % inv) else key
        sel.append(dict(investigation=inv, key=key, rel=rel, psi_file_id=r["psi_file_id"],
                        file_version=int(r["file_version"]), file_name=r["file_name"],
                        size_bytes=int(r["size_bytes"]), kind=r["kind"], url=r["url"]))
    seen = set()
    for s in sorted(sel, key=lambda s: (s["investigation"], s["key"], s["psi_file_id"])):
        parts = [safe_component(c) for c in s["rel"].split("/")]
        local = "/".join([s["investigation"]] + parts)
        if local.lower() in seen:
            stem, dot, ext = parts[-1].rpartition(".")
            parts[-1] = (stem + "__" + s["psi_file_id"] + dot + ext) if dot else parts[-1] + "__" + s["psi_file_id"]
            local = "/".join([s["investigation"]] + parts)
        seen.add(local.lower())
        s["local_path"] = local
    return sel


def aliases(s):
    """Other spellings of the download URL for names the endpoint cannot resolve as listed: the name
    without trailing blanks, and the whole key tail when the name holds a '/'."""
    tail = s["key"].split("/", 3)[3] if s["key"].count("/") >= 3 else s["file_name"]
    out = []
    for name in (s["file_name"].strip(), tail):
        if name != s["file_name"]:
            q = {"file": name, "version": s["file_version"]}
            u = "%s/studies/%s/download?%s" % (BASE, quote(s["investigation"], safe=""), urlencode(q, quote_via=quote))
            if u not in out:
                out.append(u)
    return out


def reuse_index():
    idx = defaultdict(list)
    for d in REUSE_DIRS:
        if not d.exists():
            continue
        for p in d.rglob("*"):
            if "holdout" in str(p).lower():
                continue
            if p.is_file():
                idx[(p.name, p.stat().st_size)].append(p)
    return idx


class Run:
    def __init__(self, a, sel):
        self.a, self.sel = a, sel
        self.raw = DATA / "raw"
        self.log = self.raw / "_log" / "receipts.jsonl"
        self.log.parent.mkdir(parents=True, exist_ok=True)
        self.manifest = self.raw / "MANIFEST.jsonl"
        self.state = {}
        for p in (self.manifest, self.log):
            if p.exists():
                for line in p.open(encoding="utf-8"):
                    if line.strip():
                        r = json.loads(line)
                        self.state[r["local_path"]] = r
        # earlier failures are tried again in this run; only finished downloads carry over
        self.state = {k: v for k, v in self.state.items() if v.get("status") in ("ok", "reused", "size_differs")}
        self.min_free = a.min_free_gb * 1024 ** 3
        self.bytes_done = 0
        self.files_done = Counter()
        self.t0 = time.time()
        self.since_flush = 0
        self.throttle_until = 0.0
        self.errors = Counter()

    def record(self, s, status, **kw):
        row = dict(investigation=s["investigation"], key=s["key"], local_path=s["local_path"],
                   psi_file_id=s["psi_file_id"], file_version=s["file_version"], kind=s["kind"],
                   size=s["size_bytes"], sha256=kw.pop("sha256", None), status=status, time=now(), **kw)
        with LOCK:
            self.state[s["local_path"]] = row
            with self.log.open("a", encoding="utf-8") as f:
                f.write(json.dumps(row, sort_keys=True) + "\n")
            self.files_done[status] += 1
            self.since_flush += 1
            flush = self.since_flush >= self.a.flush_every
            if flush:
                self.since_flush = 0
        if flush:
            self.write_manifest()
        self.check_markers()
        return status

    def check_markers(self):
        for name, invs in MARKERS.items():
            path = self.raw / name
            if path.exists():
                continue
            with LOCK:
                rows = [self.state.get(x["local_path"]) for x in self.sel if x["investigation"] in invs]
            if all(r and r["status"] in FINAL for r in rows):
                c = Counter(r["status"] for r in rows)
                per = {i: dict(Counter(r["status"] for r in rows if r["investigation"] == i)) for i in invs}
                path.write_text("%s %s complete: %d files, %d bytes, statuses %s, per investigation %s, failed %s\n" % (
                    now(), " and ".join(invs), len(rows), sum((r.get("served_size") or r["size"]) for r in rows
                                                           if r["status"] != "failed"),
                    json.dumps(dict(c)), json.dumps(per),
                    json.dumps([r["key"] for r in rows if r["status"] == "failed"])), encoding="utf-8")

    def write_manifest(self):
        with LOCK:
            rows = [self.state.get(s["local_path"]) or dict(
                investigation=s["investigation"], key=s["key"], local_path=s["local_path"],
                psi_file_id=s["psi_file_id"], file_version=s["file_version"], kind=s["kind"],
                size=s["size_bytes"], sha256=None, status="pending") for s in self.sel]
        tmp = self.manifest.with_suffix(".tmp")
        with tmp.open("w", encoding="utf-8", newline="\n") as f:
            for r in rows:
                f.write(json.dumps(r, sort_keys=True) + "\n")
        tmp.replace(self.manifest)
        return Counter(r["status"] for r in rows)

    def free(self):
        return shutil.disk_usage(self.raw).free

    def one(self, s, reuse):
        if STOP.is_set():
            return "stopped"
        dest = self.raw / s["local_path"]
        prev = self.state.get(s["local_path"])
        if prev and dest.exists() and (
                (prev.get("status") in ("ok", "reused") and dest.stat().st_size == s["size_bytes"]) or
                (prev.get("status") == "size_differs" and dest.stat().st_size == prev.get("served_size"))):
            with LOCK:
                self.files_done["already"] += 1
            return "already"
        if self.free() < self.min_free:
            STOP.set()
            return self.record(s, "stopped_low_disk", free_bytes=self.free())
        dest.parent.mkdir(parents=True, exist_ok=True)
        for src in reuse.get((s["file_name"], s["size_bytes"]), []):
            shutil.copyfile(src, dest)
            if dest.stat().st_size == s["size_bytes"]:
                return self.record(s, "reused", sha256=sha256_of(dest), source=str(src.relative_to(ROOT)))
        part = dest.with_name(dest.name + ".part")
        last = None
        expected = s["size_bytes"]
        mismatches = []
        urls = [s["url"]] + aliases(s)
        ui = 0
        for attempt in range(1, self.a.retries + 1):
            if STOP.is_set():
                return "stopped"
            try:
                offset = part.stat().st_size if part.exists() else 0
                if offset > expected:
                    part.unlink()
                    offset = 0
                headers = {"Range": "bytes=%d-" % offset} if 0 < offset < expected else {}
                if offset == expected and expected > 0:
                    headers = {}
                    offset = 0
                wait = self.throttle_until - time.time()
                if wait > 0:
                    time.sleep(wait)
                with session().get(urls[ui], headers=headers, stream=True, timeout=(30, 180)) as r:
                    r.raise_for_status()
                    append = False
                    if headers and r.status_code == 206:
                        m = re.fullmatch(r"bytes (\d+)-(\d+)/(\d+)", r.headers.get("Content-Range", ""))
                        if m and int(m[1]) == offset and int(m[3]) == expected:
                            append = True
                        else:
                            raise ValueError("bad Content-Range")
                    if not append:
                        offset = 0
                    cl = r.headers.get("Content-Length")
                    if cl is not None and int(cl) != expected - offset:
                        if append:
                            raise ValueError("Content-Length %s differs from the rest %d" % (cl, expected - offset))
                        mismatches.append(int(cl))
                        # the server serves the same other size twice: take it, and say so
                        if mismatches.count(int(cl)) < 2:
                            raise ValueError("Content-Length %s differs from listing %d" % (cl, expected))
                        expected = int(cl)
                    got = offset
                    with part.open("ab" if append else "wb") as f:
                        for chunk in r.iter_content(1 << 20):
                            got += len(chunk)
                            if got > expected:
                                raise ValueError("more bytes than expected")
                            f.write(chunk)
                            with LOCK:
                                self.bytes_done += len(chunk)
                if got != expected:
                    raise ValueError("got %d of %d bytes" % (got, expected))
                digest = sha256_of(part)
                part.replace(dest)
                if expected != s["size_bytes"]:
                    return self.record(s, "size_differs", sha256=digest, attempts=attempt, served_size=expected,
                                       url_used=urls[ui] if ui else None,
                                       error="served %d bytes, listing says %d" % (expected, s["size_bytes"]))
                return self.record(s, "ok", sha256=digest, attempts=attempt, resumed=append,
                                   **({"url_used": urls[ui]} if ui else {}))
            except (requests.RequestException, ValueError, OSError) as e:
                last = "%s: %s" % (type(e).__name__, str(e)[:200])
                if isinstance(e, requests.HTTPError) and e.response is not None and e.response.status_code in (401, 403):
                    return self.record(s, "failed", error=last, attempts=attempt)
                code = getattr(getattr(e, "response", None), "status_code", None)
                if code == 404 and len(urls) > 1:
                    ui = (ui + 1) % len(urls)
                    if part.exists():
                        part.unlink()
                with LOCK:
                    self.errors[str(code) if code else type(e).__name__] += 1
                    if code in (429, 503) or isinstance(e, requests.ConnectionError):
                        # the server pushes back: every worker pauses a little
                        self.throttle_until = max(self.throttle_until, time.time() + 5 * attempt)
                time.sleep(min(120, 2 ** attempt) * (0.5 + random.random()))
        return self.record(s, "failed", error=last, attempts=self.a.retries)

    def progress(self):
        last_m = time.time()
        while not self.done.wait(30):
            el = time.time() - self.t0
            with LOCK:
                fd, bd = dict(self.files_done), self.bytes_done
            new = sum(v for k, v in fd.items() if k != "already")
            with LOCK:
                errs = dict(self.errors)
            print(json.dumps(dict(t=now(), elapsed_s=round(el), files=fd, files_s=round(new / max(el, 1), 2),
                                  mb=round(bd / 1e6, 1), mb_s=round(bd / 1e6 / max(el, 1), 2),
                                  retried_errors=errs, free_gb=round(self.free() / 1e9, 1))),
                  flush=True)
            if time.time() - last_m > 300:
                self.write_manifest()
                last_m = time.time()

    def go(self):
        reuse = reuse_index() if self.a.reuse else {}
        self.done = threading.Event()
        t = threading.Thread(target=self.progress, daemon=True)
        t.start()
        # large files first so that the long transfers overlap the many small ones
        rank = {inv: k for k, inv in enumerate(PRIORITY)}
        order = sorted(self.sel, key=lambda s: (rank.get(s["investigation"], len(rank)), s["key"]))
        with concurrent.futures.ThreadPoolExecutor(self.a.workers) as ex:
            list(ex.map(lambda s: self.one(s, reuse), order))
        self.done.set()
        el = time.time() - self.t0
        counts = self.write_manifest()
        summary = dict(finished=now(), elapsed_s=round(el, 1), bytes_this_run=self.bytes_done,
                       mb_s_this_run=round(self.bytes_done / 1e6 / max(el, 1), 2),
                       statuses=dict(counts), stopped_low_disk=STOP.is_set(), workers=self.a.workers)
        (self.raw / "_log" / "last_run.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
        print(json.dumps(summary), flush=True)
        return 0 if counts.get("failed", 0) == 0 and not STOP.is_set() else 1


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--files", default=str(DATA / "work/atlas/files.jsonl"))
    ap.add_argument("--workers", type=int, default=40)
    ap.add_argument("--flush-every", type=int, default=300, help="rewrite MANIFEST.jsonl every N files")
    ap.add_argument("--retries", type=int, default=6)
    ap.add_argument("--min-free-gb", type=float, default=150)
    ap.add_argument("--no-reuse", dest="reuse", action="store_false")
    ap.add_argument("--dry-run", action="store_true")
    a = ap.parse_args(argv)
    sel = select(Path(a.files))
    by = Counter(s["investigation"] for s in sel)
    print(json.dumps(dict(selected=len(sel), bytes=sum(s["size_bytes"] for s in sel), per_investigation=by)), flush=True)
    if a.dry_run:
        return 0
    return Run(a, sel).go()


if __name__ == "__main__":
    sys.exit(main())
