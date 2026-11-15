"""Merge per-document result files (results/<parser>-<doc>.json) into results/<parser>.json."""
import json, sys
from pathlib import Path
R = Path(__file__).resolve().parent / "results"
parser = sys.argv[1]
parts = sorted(R.glob(f"{parser}-*.json"))
out = {"parser": parser, "items": [], "perf": {}, "peak_rss_mb": 0}
for p in parts:
    r = json.loads(p.read_text())
    out["items"] += r["items"]; out["perf"].update(r["perf"]); out["peak_rss_mb"] = max(out["peak_rss_mb"], r["peak_rss_mb"])
(R / f"{parser}.json").write_text(json.dumps(out, indent=1) + "\n")
print(parser, len(parts), "documents,", sum(i["found"] for i in out["items"]), "/", len(out["items"]), "found; peak RSS", out["peak_rss_mb"], "MB")
