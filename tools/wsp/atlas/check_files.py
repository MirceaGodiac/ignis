"""Checks of the file catalogue written by classify_files.py.

  1. Every listed file gets exactly one row: the rows of each investigation are the lines of its
     listing, in order, with the same name, size, URL and file id.
  2. The bytes add up to the listings' totals, per investigation and overall, and the summary
     repeats the same totals.
  3. Every row carries a kind and a confidence from the closed lists, and a rule.
  4. Two fresh runs give byte-identical output, and the output on disk equals them.

Usage (from the repository root):
  python tools/wsp/atlas/check_files.py
Exit code 0 when every check passes.
"""
import hashlib
import json
import shutil
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import classify_files as cf  # noqa: E402

OUT = cf.ROOT / "runs/dev/data_all/work/atlas"


def sha(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for block in iter(lambda: f.read(1 << 20), b""):
            h.update(block)
    return h.hexdigest()


def main():
    failures = []

    def check(ok, text):
        print(("ok    " if ok else "FAIL  ") + text)
        if not ok:
            failures.append(text)

    files = OUT / "files.jsonl"
    summary_path = OUT / "files_summary.json"
    check(files.exists() and summary_path.exists(), "outputs exist under %s" % OUT.relative_to(cf.ROOT))
    if failures:
        return 1
    rows = [json.loads(line) for line in files.open(encoding="utf-8")]
    summary = json.loads(summary_path.read_text(encoding="utf-8"))

    # 1 and 2: one row per listed file, bytes add up
    total_files = total_bytes = 0
    by_inv = {}
    for r in rows:
        by_inv.setdefault(r["investigation"], []).append(r)
    check(sorted(by_inv) == sorted(cf.ORDER), "the %d investigations are all present, and no other" % len(cf.ORDER))
    for inv in cf.ORDER:
        listing = cf.read_listing(inv)
        mine = by_inv.get(inv, [])
        same = len(listing) == len(mine) and all(
            m["listing_row"] == l["_row"] and m["file_name"] == l["file_name"] and m["url"] == l["stable_url"]
            and m["psi_file_id"] == l["psi_file_id"] and m["size_bytes"] == int(l["size_bytes"] or 0)
            and m["category"] == l["category"]
            for m, l in zip(mine, listing))
        lb = sum(int(l["size_bytes"] or 0) for l in listing)
        mb = sum(m["size_bytes"] for m in mine)
        s = summary["investigations"].get(inv, {})
        ok = same and lb == mb and s.get("files") == len(listing) and s.get("bytes") == lb \
            and len({m["listing_row"] for m in mine}) == len(mine)
        check(ok, "%-8s %6d listed files = %6d rows, %16d bytes = %16d bytes" % (inv, len(listing), len(mine), lb, mb))
        total_files += len(listing)
        total_bytes += lb
    check(len(rows) == total_files and sum(r["size_bytes"] for r in rows) == total_bytes,
          "overall: %d rows and %d bytes equal the listings' totals" % (len(rows), sum(r["size_bytes"] for r in rows)))
    check(summary["total"]["files"] == total_files and summary["total"]["bytes"] == total_bytes,
          "the summary repeats the totals (%d files, %d bytes)" % (summary["total"]["files"], summary["total"]["bytes"]))
    kinds_sum = sum(v["files"] for v in summary["total"]["by_kind"].values())
    kinds_bytes = sum(v["bytes"] for v in summary["total"]["by_kind"].values())
    check(kinds_sum == total_files and kinds_bytes == total_bytes, "files and bytes by kind add up to the totals")

    # repeats: a repeated listing row points at an earlier row with the same file id
    index = {(r["investigation"], r["listing_row"]): r for r in rows}
    bad = [r for r in rows if r["repeat_of_row"] is not None and (
        r["repeat_of_row"] >= r["listing_row"]
        or index[(r["investigation"], r["repeat_of_row"])]["psi_file_id"] != r["psi_file_id"])]
    distinct = sum(1 for r in rows if r["repeat_of_row"] is None)
    check(not bad and distinct == len({(r["investigation"], r["psi_file_id"]) for r in rows}),
          "repeated listing rows point at the first row of the same file id (%d distinct files)" % distinct)

    # 3: closed lists
    check(all(r["kind"] in cf.KINDS for r in rows), "every row has a kind from the closed list")
    check(all(r["confidence"] in cf.CONFIDENCES for r in rows), "every row has a confidence from the closed list")
    check(all(r["rule"] for r in rows), "every row names the rule that classified it")
    check(all((r["test_id"] is None) == (r["test_id_scheme"] is None) for r in rows),
          "a test id always comes with its naming scheme")
    unclassified = [r for r in rows if r["rule"] == "unclassified"]
    check(all(r["confidence"] == cf.UNK and r["kind"] == "other" for r in unclassified),
          "unclassified rows are kind 'other' with confidence 'unknown' (%d rows)" % len(unclassified))

    # 4: determinism
    tmp = OUT / "_determinism_check"
    hashes = []
    try:
        for n in (1, 2):
            d = tmp / ("run%d" % n)
            d.mkdir(parents=True, exist_ok=True)
            r = cf.build_rows()
            cf.write(r, cf.summarize(r), d)
            hashes.append((sha(d / "files.jsonl"), sha(d / "files_summary.json")))
    finally:
        shutil.rmtree(tmp, ignore_errors=True)
    check(hashes[0] == hashes[1], "two fresh runs give identical output")
    check(hashes[0] == (sha(files), sha(summary_path)), "the output on disk equals a fresh run")
    print("files.jsonl          sha256 %s" % hashes[0][0])
    print("files_summary.json   sha256 %s" % hashes[0][1])
    print("%d checks failed" % len(failures) if failures else "all checks passed")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
