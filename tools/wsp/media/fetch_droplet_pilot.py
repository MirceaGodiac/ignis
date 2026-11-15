"""Download the backlit (HiBMs, camera A) inputs of FLEX TRAIN test points for the droplet pilot.

Opaque, size-checked downloads of public PSI-69 files named in the listing. Archives are NOT
extracted: droplet_diameter.py reads frames straight from the zip. Run with `python -I`.

    python -I tools/wsp/media/fetch_droplet_pilot.py --out runs/dev/media/droplet/raw 20CAL08 053F001 ...

Per test point it fetches `<tp>A.zip`, `<tp>A.txt`, `<tp>.txt` (timeline), the HiBMs camera settings,
the MDCA test point settings and the comments file when they exist.
"""
import argparse
import json
import shutil
import sys
import time
from pathlib import Path
from urllib.parse import quote, urlencode

import requests

REPO = Path(__file__).resolve().parents[3]
LISTING = REPO / "runs/dev/data_astra/metadata/PSI-69.listing.jsonl"
BASE = "https://psi.nasa.gov/geode-py/ws"
SUFFIXES = ("A.zip", "A.txt", ".txt", "_HIBMS_Camera_Settings.txt", "_HIMBS_Camera_Settings.txt",
            "_MDCA_Testpoint_Settings.txt", "_Testpoint_Settings.txt", "_Comments.txt")


def url(row):
    q = {"file": row["file_name"], "version": row["file_version"]}
    return f"{BASE}/studies/{quote(row['investigation'], safe='')}/download?{urlencode(q, quote_via=quote)}"


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", required=True)
    ap.add_argument("--min-free-gb", type=float, default=150.0)
    ap.add_argument("points", nargs="+")
    a = ap.parse_args()
    rows = {}
    for line in LISTING.open(encoding="utf-8"):
        r = json.loads(line)
        rows.setdefault(r["file_name"], r)
    s = requests.Session()
    s.headers.update({"User-Agent": "research-data-client/1.0", "Accept-Encoding": "identity"})
    out = Path(a.out)
    log = []
    for tp in a.points:
        d = out / tp
        d.mkdir(parents=True, exist_ok=True)
        for name, r in rows.items():
            ok = any(name.endswith("_" + tp + suf) for suf in SUFFIXES)
            if not ok:
                continue
            free = shutil.disk_usage(out).free / 1e9
            if free - r["size_bytes"] / 1e9 < a.min_free_gb:
                print(f"STOP: {free:.0f} GB free", flush=True)
                return 2
            target = d / name
            if target.exists() and target.stat().st_size == r["size_bytes"]:
                continue
            t0 = time.time()
            part = target.with_suffix(target.suffix + ".part")
            for attempt in range(3):
                try:
                    with s.get(url(r), stream=True, timeout=(30, 300)) as resp:
                        resp.raise_for_status()
                        with open(part, "wb") as f:
                            for chunk in resp.iter_content(1 << 20):
                                f.write(chunk)
                    break
                except requests.RequestException as e:
                    print(f"retry {name}: {type(e).__name__}", flush=True)
                    time.sleep(5 * (attempt + 1))
            if part.stat().st_size != r["size_bytes"]:
                print(f"SIZE MISMATCH {name}", flush=True)
                continue
            part.replace(target)
            dt = time.time() - t0
            log.append(dict(test_point=tp, file=name, bytes=r["size_bytes"], seconds=round(dt, 1)))
            print(f"{tp} {name} {r['size_bytes']/1e9:.3f} GB {dt:.0f} s", flush=True)
    (out / "download_log.jsonl").open("a", encoding="utf-8").write("".join(json.dumps(x) + "\n" for x in log))
    return 0


if __name__ == "__main__":
    sys.exit(main())
