"""Download and extract the Saffire mapped image sets used by measure_tracks.py.

Opaque downloads of public PSI files (stable_url from the listings), size-checked, each zip extracted
into its own new directory. Run with `python -I`. Paths are arguments; nothing is executed from the
downloads.

    python -I tools/wsp/saffire/fetch_saffire.py --listings runs/dev/data_all/manifests --out runs/dev/saffire/raw
"""
import argparse
import json
import sys
import zipfile
from pathlib import Path

import requests

WANTED = {
    "PSI-98": [
        "PSI-98_Raw Data_SAFFIRE_I_S1C1_JPEG_Mapped_part1.zip",
        "PSI-98_Raw Data_SAFFIRE_I_S1C1_JPEG_Mapped_part2.zip",
        "PSI-98_Raw Data_SAFFIRE_I_S1C1_JPEG_Mapped_part3.zip",
        "PSI-98_Raw Data_SAFFIRE_I_S1C2_JPEG_Mapped_Part1.zip",
        "PSI-98_Raw Data_SAFFIRE_I_S1C2_JPEG_Mapped_part2.zip",
        "PSI-98_Raw Data_SAFFIRE_I_S1C2_JPEG_Mapped_part3.zip",
        "PSI-98_Raw Data_SAFFIRE_I_S1C2_JPEG_Mapped_part4.zip",
    ],
    "PSI-100": [
        "PSI-100_Raw Data_SAFFIRE_III_S1C1_JPEG_Mapped_part1.zip",
        "PSI-100_Raw Data_SAFFIRE_III_S1C1_JPEG_Mapped_part2.zip",
        "PSI-100_Raw Data_SAFFIRE_III_S1C1_JPEG_Mapped_part3.zip",
        "PSI-100_Raw Data_SAFFIRE_III_S1C1_JPEG_Mapped_part4.zip",
        "PSI-100_Raw Data_SAFFIRE_III_S1C2_JPEG_Mapped_part1.zip",
        "PSI-100_Raw Data_SAFFIRE_III_S1C2_JPEG_Mapped_Part2.zip",
        "PSI-100_Raw Data_SAFFIRE_III_S1C2_JPEG_Mapped_Part3.zip",
        "PSI-100_Raw Data_SAFFIRE_III_S1C2_JPEG_Mapped_Part4.zip",
    ],
    "PSI-99": [
        "PSI-99_Raw Data_SAFFIRE-II_S5C1_Image_Data_All_JPEGs_Mapped_part1.zip",
        "PSI-99_Raw Data_SAFFIRE-II_S5C1_Image_Data_All_JPEGs_Mapped_part2.zip",
        "PSI-99_Raw Data_SAFFIRE-II_S5C2_Image_Data_All_JPEGs_Mapped_part1.zip",
        "PSI-99_Raw Data_SAFFIRE-II_S5C2_Image_Data_All-JPEGs_Mapped_part2.zip",
        "PSI-99_Raw Data_SAFFIRE-II_S6C1_Image_Data_All-JPEGs_Mapped_part1.zip",
        "PSI-99_Raw Data_SAFFIRE-II-S6C1_Image_Data_All_JPEGs_Mapped_part2.zip",
        "PSI-99_Raw Data_SAFFIRE-II_S6C2_Image_Data_All_JPEGs_Mapped_part1.zip",
        "PSI-99_Raw Data_SAFFIRE-II_S6C2_Image_Data_All_JPEGs_Mapped_part2.zip",
        "PSI-99_Raw Data_SAFFIRE-II_S5C1_DistortionGrids_Sample_ROI_Mapped JPEGS.zip",
        "PSI-99_Raw Data_SAFFIRE-II_S5C2_DistortionGrids_Sample_ROI_Mapped JPEGS.zip",
        "PSI-99_Raw Data_SAFFIRE-II_S6C1_DistortionGrids_Sample_ROI_Mapped JPEGS.zip",
        "PSI-99_Raw Data_SAFFIRE-II_S6C2_DistortionGrids_Sample_ROI_Mapped JPEGS.zip",
    ],
}


def safe_extract(zpath: Path, dest: Path) -> int:
    dest.mkdir(parents=True, exist_ok=False)
    n = 0
    with zipfile.ZipFile(zpath) as z:
        root = dest.resolve()
        for info in z.infolist():
            target = (dest / info.filename).resolve()
            if root not in target.parents and target != root:
                raise RuntimeError(f"unsafe member {info.filename!r} in {zpath.name}")
            if info.is_dir():
                target.mkdir(parents=True, exist_ok=True)
                continue
            target.parent.mkdir(parents=True, exist_ok=True)
            with z.open(info) as src, open(target, "wb") as dst:
                while True:
                    b = src.read(1 << 20)
                    if not b:
                        break
                    dst.write(b)
            n += 1
    return n


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--listings", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--only", default=None, help="PSI id to restrict to")
    args = ap.parse_args()
    s = requests.Session()
    s.headers.update({"User-Agent": "research-data-client/1.0", "Accept-Encoding": "identity"})
    for psi, names in WANTED.items():
        if args.only and psi != args.only:
            continue
        rows = {}
        for line in open(Path(args.listings) / f"{psi}.listing.jsonl", encoding="utf-8"):
            r = json.loads(line)
            rows[r["file_name"]] = r
        zdir = Path(args.out) / psi / "zips"
        zdir.mkdir(parents=True, exist_ok=True)
        for name in names:
            r = rows[name]
            zp = zdir / name
            if not (zp.exists() and zp.stat().st_size == r["size_bytes"]):
                part = zp.with_suffix(".part")
                with s.get(r["stable_url"], stream=True, timeout=(30, 300)) as resp:
                    resp.raise_for_status()
                    with open(part, "wb") as f:
                        for chunk in resp.iter_content(1 << 20):
                            f.write(chunk)
                if part.stat().st_size != r["size_bytes"]:
                    print(f"SIZE MISMATCH {name}: {part.stat().st_size} vs {r['size_bytes']}", flush=True)
                    continue
                part.replace(zp)
            ex = Path(args.out) / psi / "x" / zp.stem
            if not ex.exists():
                n = safe_extract(zp, ex)
                print(f"{psi} {name}: {r['size_bytes']} bytes, {n} files", flush=True)
            else:
                print(f"{psi} {name}: already extracted", flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
