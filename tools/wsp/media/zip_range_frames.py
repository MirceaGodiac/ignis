"""Fetch single frames from a remote PSI image archive with HTTP range requests (no full download).

Reads the zip's central directory from the end of the file, then the local entries of the requested
members. Frames are written as files into --out; nothing is executed.

    python -I tools/wsp/media/zip_range_frames.py --psi PSI-68 --file "PSI-68_Raw Data_E073C02A.zip" \
        --version 1 --frames 0 30 --out <dir>
"""
import argparse
import io
import json
import struct
import sys
import zlib
from pathlib import Path
from urllib.parse import quote, urlencode

import requests

BASE = "https://psi.nasa.gov/geode-py/ws"


def url(psi, name, version):
    q = {"file": name, "version": version}
    return f"{BASE}/studies/{quote(psi, safe='')}/download?{urlencode(q, quote_via=quote)}"


def rng(s, u, a, b):
    r = s.get(u, headers={"Range": f"bytes={a}-{b}"}, timeout=(30, 300))
    r.raise_for_status()
    if r.status_code != 206:
        raise RuntimeError(f"server ignored the range request ({r.status_code})")
    return r.content


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--psi", required=True)
    ap.add_argument("--file", required=True)
    ap.add_argument("--version", default="1")
    ap.add_argument("--size", type=int, required=True, help="archive size in bytes (from the listing)")
    ap.add_argument("--frames", type=int, nargs="+", default=[0])
    ap.add_argument("--out", required=True)
    a = ap.parse_args()
    s = requests.Session()
    s.headers.update({"User-Agent": "research-data-client/1.0", "Accept-Encoding": "identity"})
    u = url(a.psi, a.file, a.version)
    tail = rng(s, u, max(0, a.size - 65536 - 22), a.size - 1)
    i = tail.rfind(b"PK\x05\x06")
    cd_size, cd_off = struct.unpack("<II", tail[i + 12:i + 20])
    if cd_off == 0xFFFFFFFF or cd_size == 0xFFFFFFFF:  # zip64
        j = tail.rfind(b"PK\x06\x06")
        cd_size, cd_off = struct.unpack("<QQ", tail[j + 40:j + 56])
    cd = rng(s, u, cd_off, cd_off + cd_size - 1)
    entries, p = {}, 0
    while p < len(cd) and cd[p:p + 4] == b"PK\x01\x02":
        method, = struct.unpack("<H", cd[p + 10:p + 12])
        csize, usize = struct.unpack("<II", cd[p + 20:p + 28])
        nlen, elen, clen = struct.unpack("<HHH", cd[p + 28:p + 34])
        off, = struct.unpack("<I", cd[p + 42:p + 46])
        name = cd[p + 46:p + 46 + nlen].decode("utf-8", "replace")
        extra = cd[p + 46 + nlen:p + 46 + nlen + elen]
        q = 0
        while q + 4 <= len(extra):
            hid, hl = struct.unpack("<HH", extra[q:q + 4])
            if hid == 1:
                vals = list(struct.unpack("<" + "Q" * (hl // 8), extra[q + 4:q + 4 + hl]))
                if usize == 0xFFFFFFFF:
                    usize = vals.pop(0)
                if csize == 0xFFFFFFFF:
                    csize = vals.pop(0)
                if off == 0xFFFFFFFF:
                    off = vals.pop(0)
            q += 4 + hl
        entries[name] = (method, csize, off)
        p += 46 + nlen + elen + clen
    tifs = sorted(n for n in entries if n.lower().endswith((".tif", ".tiff")))
    out = Path(a.out)
    out.mkdir(parents=True, exist_ok=True)
    for k in a.frames:
        name = tifs[k]
        method, csize, off = entries[name]
        head = rng(s, u, off, off + 29)
        nlen, elen = struct.unpack("<HH", head[26:30])
        data = rng(s, u, off + 30 + nlen + elen, off + 30 + nlen + elen + csize - 1)
        raw = data if method == 0 else zlib.decompress(data, -15)
        (out / Path(name).name).write_bytes(raw)
        print(json.dumps(dict(member=name, bytes=len(raw))))
    return 0


if __name__ == "__main__":
    sys.exit(main())
