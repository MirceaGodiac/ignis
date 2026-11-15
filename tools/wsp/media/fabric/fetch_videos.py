"""Download the BASS-II (PSI-25) main-view burn videos and the BASS (PSI-26) burn videos.

Opaque downloads of public PSI files (stable_url from the listings), size-checked and hashed. Run with
`python -I`; nothing is executed from the downloads. Train burns come first so the method can be fixed
while the rest downloads. A manifest line (source key, url, size, sha256, local path) is appended per
file to <out>/../manifest_raw.jsonl.

    python -I tools/wsp/media/fabric/fetch_videos.py --listings runs/dev/data_all/manifests \
        --out runs/dev/media/fabric/raw [--only PSI-25] [--names a.mp4,b.mp4]
"""
import argparse
import hashlib
import json
import sys
from pathlib import Path

import requests

TRAIN_FIRST = ("TEST_T5.mp4", "TEST_T6.mp4", "TEST_T16.mp4", "TEST_T10.mp4")


def main_view_psi25(name: str) -> bool:
    n = name.upper()
    if not n.endswith(".MP4"):
        return False
    if "ND_VIEW" in n or "STILL_SEQUENCE" in n:
        return False
    return True


def wanted(psi: str, rows):
    if psi == "PSI-25":
        sel = [r for r in rows if r["category"] == "Raw Data" and main_view_psi25(r["file_name"])]
        sel.sort(key=lambda r: (0 if r["file_name"].endswith(TRAIN_FIRST) else 1, r["size_bytes"]))
        return sel
    sel = [r for r in rows if r["category"] == "Raw Data" and r["file_name"].lower().endswith(".mp4")]
    sel.sort(key=lambda r: r["file_name"])
    return sel


def sha256(p: Path) -> str:
    h = hashlib.sha256()
    with open(p, "rb") as f:
        while True:
            b = f.read(1 << 22)
            if not b:
                break
            h.update(b)
    return h.hexdigest()


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--listings", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--only", default=None)
    ap.add_argument("--names", default=None, help="comma-separated file names (overrides the selection)")
    args = ap.parse_args()
    out = Path(args.out)
    manifest = out.parent / "manifest_raw.jsonl"
    done = set()
    if manifest.exists():
        for line in manifest.read_text(encoding="utf-8").splitlines():
            done.add(json.loads(line)["file_name"])
    s = requests.Session()
    s.headers.update({"User-Agent": "research-data-client/1.0", "Accept-Encoding": "identity"})
    for psi in ("PSI-25", "PSI-26"):
        if args.only and psi != args.only:
            continue
        rows = [json.loads(l) for l in open(Path(args.listings) / f"{psi}.listing.jsonl", encoding="utf-8")]
        if args.names:
            names = set(args.names.split(","))
            sel = [r for r in rows if r["file_name"] in names]
        else:
            sel = wanted(psi, rows)
        d = out / psi.lower().replace("-", "")
        d.mkdir(parents=True, exist_ok=True)
        for r in sel:
            name = r["file_name"]
            p = d / name
            if name in done and p.exists() and p.stat().st_size == r["size_bytes"]:
                continue
            if not (p.exists() and p.stat().st_size == r["size_bytes"]):
                part = p.with_suffix(".part")
                for attempt in range(4):
                    try:
                        with s.get(r["stable_url"], stream=True, timeout=(30, 300)) as resp:
                            resp.raise_for_status()
                            with open(part, "wb") as f:
                                for chunk in resp.iter_content(1 << 20):
                                    f.write(chunk)
                        if part.stat().st_size == r["size_bytes"]:
                            break
                        print(f"SIZE MISMATCH {name}: {part.stat().st_size} vs {r['size_bytes']} (attempt {attempt})", flush=True)
                    except Exception as e:  # network errors: retry
                        print(f"ERROR {name}: {e!r} (attempt {attempt})", flush=True)
                else:
                    print(f"FAILED {name}", flush=True)
                    continue
                part.replace(p)
            rec = {"investigation": psi, "file_name": name, "key": r["key"], "stable_url": r["stable_url"],
                   "size_bytes": r["size_bytes"], "sha256": sha256(p), "local": str(p.relative_to(out.parent)).replace("\\", "/")}
            with open(manifest, "a", encoding="utf-8") as f:
                f.write(json.dumps(rec) + "\n")
            done.add(name)
            print(f"{psi} {name} {r['size_bytes']}", flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
