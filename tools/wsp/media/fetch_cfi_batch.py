"""The verified ranged downloader of fetch_flex2_batch.py, pointed at the CFI (PSI-39) listing.

    python -I tools/wsp/media/fetch_cfi_batch.py --batch <file> --out <dir> [--connections N --files M]
    python -I tools/wsp/media/fetch_cfi_batch.py verify --dir <dir>
"""
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import fetch_flex2_batch as f  # noqa: E402

f.LISTING = f.REPO / "runs/dev/data_all/manifests/PSI-39.listing.jsonl"

if __name__ == "__main__":
    sys.exit(f.main())
