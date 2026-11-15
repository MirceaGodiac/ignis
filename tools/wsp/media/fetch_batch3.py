"""The verified ranged downloader (fetch_flex2_batch.py), with one tolerance for NASA's own archives.

Some NASA image archives carry a Windows thumbnail cache, `Thumbs.db`, whose stored CRC does not match its
bytes (first seen 15 November on A113D07A.zip, refused twice). It is not a frame, the measurer never reads
it, and the fault is in the archive as NASA built it, not in the transfer. This wrapper accepts an archive
whose only CRC mismatches are in `Thumbs.db` members, after it has checked the CRC of every other member
itself; every other check (ranges, Content-Range, headers, central directory) is unchanged. The manifest's
`verified` text says so.

    python -I tools/wsp/media/fetch_batch3.py [--psi PSI-39] --batch <file> --out <dir> [--connections N --files M]
"""
import sys
import zipfile
import zlib
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import fetch_flex2_batch as f  # noqa: E402

_strict = f.verify_zip


def verify_zip(path, quick=False):
    ok, detail = _strict(path, quick)
    if ok or not detail.startswith("CRC mismatch in member") or "thumbs.db" not in detail.lower():
        return ok, detail
    try:
        with zipfile.ZipFile(path) as z:
            skipped = 0
            for i in z.infolist():
                if Path(i.filename).name.lower() == "thumbs.db":
                    skipped += 1
                    continue
                with z.open(i) as m:  # reading to the end checks the member's CRC-32
                    while m.read(1 << 22):
                        pass
        return True, f"directory, local headers and CRCs of every member except {skipped} Thumbs.db (stored CRC wrong in NASA's archive)"
    except (zipfile.BadZipFile, zlib.error, OSError, EOFError) as e:
        return False, f"{type(e).__name__}: {e}"


f.verify_zip = verify_zip

if __name__ == "__main__":
    if "--psi" in sys.argv:
        i = sys.argv.index("--psi")
        psi = sys.argv[i + 1]
        del sys.argv[i:i + 2]
        f.LISTING = f.REPO / f"runs/dev/data_all/manifests/{psi}.listing.jsonl"
    sys.exit(f.main())
