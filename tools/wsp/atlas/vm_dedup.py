"""Compare the NASA files of this lane's download with what the application's VM already stores, by content.

Inputs: `runs/dev/data_all/raw/MANIFEST.jsonl` (this lane: one row per downloaded file with sha256) and a
listing of the VM's corpus as `sha256sum` lines (`<sha256>  ./PSI-<id>/<file name>`), taken read-only from
`/srv/ignis/data/raw/samples`. Output, under `runs/dev/vm_upload/`:
  missing.jsonl   files of this lane whose content is not on the VM (to upload)
  present.jsonl   files of this lane whose content is already on the VM, with the VM path
  summary.json    counts and bytes per investigation
Nothing is read from inside any file: names, sizes and hashes only.

    python tools/wsp/atlas/vm_dedup.py --vm runs/dev/vm_upload/vm_corpus_sha256.txt
"""
import argparse
import collections
import json
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
RAW = REPO / "runs/dev/data_all/raw"
OUT = REPO / "runs/dev/vm_upload"


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--vm", required=True)
    args = ap.parse_args()
    vm = {}
    vm_names = collections.defaultdict(set)
    for line in Path(args.vm).read_text(encoding="utf-8", errors="replace").splitlines():
        if len(line) < 67:
            continue
        sha, path = line[:64], line[66:]
        vm.setdefault(sha, path)
        vm_names[path.split("/")[-1]].add(sha)
    rows = []
    for line in (RAW / "MANIFEST.jsonl").open(encoding="utf-8"):
        r = json.loads(line)
        if r.get("status") in ("ok", "reused", "size_differs", "present") and r.get("sha256"):
            rows.append(r)
    missing, present = [], []
    per = collections.defaultdict(lambda: collections.Counter())
    same_name_other_content = 0
    for r in rows:
        inv = r["key"].split("/")[1]
        name = r["key"].split("/")[-1]
        size = r.get("served_size") or r.get("size") or r.get("size_bytes") or 0
        rec = {"investigation": inv, "key": r["key"], "sha256": r["sha256"], "size": size, "file_version": r.get("file_version", 1),
               "local": r.get("local") or r.get("local_path") or r.get("path")}
        if r["sha256"] in vm:
            rec["vm_path"] = "/srv/ignis/data/raw/samples/" + vm[r["sha256"]][2:]
            present.append(rec)
            per[inv]["present_files"] += 1
            per[inv]["present_bytes"] += size
        else:
            if name in vm_names:
                same_name_other_content += 1
                rec["vm_has_same_name_with_other_content"] = True
            missing.append(rec)
            per[inv]["missing_files"] += 1
            per[inv]["missing_bytes"] += size
    local_sha = {r["sha256"] for r in rows}
    vm_only = sum(1 for s in vm if s not in local_sha)
    OUT.mkdir(parents=True, exist_ok=True)
    for name, data in (("missing.jsonl", missing), ("present.jsonl", present)):
        with (OUT / name).open("w", encoding="utf-8", newline="\n") as f:
            for rec in data:
                f.write(json.dumps(rec, ensure_ascii=False) + "\n")
    summary = {
        "local_files": len(rows), "vm_files": sum(len(v) for v in vm_names.values()), "vm_distinct_contents": len(vm),
        "already_on_vm": {"files": len(present), "bytes": sum(r["size"] for r in present)},
        "to_upload": {"files": len(missing), "bytes": sum(r["size"] for r in missing)},
        "same_name_other_content": same_name_other_content, "on_vm_only": vm_only,
        "per_investigation": {k: dict(v) for k, v in sorted(per.items(), key=lambda kv: int(kv[0].split("-")[1]))},
    }
    (OUT / "summary.json").write_text(json.dumps(summary, indent=1) + "\n", encoding="utf-8")
    gb = lambda b: "%.2f GB" % (b / 1024 ** 3)
    print("local files:", len(rows), "| already on the VM:", len(present), gb(summary["already_on_vm"]["bytes"]),
          "| to upload:", len(missing), gb(summary["to_upload"]["bytes"]))
    print("same name on the VM with other content:", same_name_other_content, "| on the VM only:", vm_only)
    for k, v in summary["per_investigation"].items():
        print("  %-8s present %5d %9s   missing %5d %9s" % (k, v.get("present_files", 0), gb(v.get("present_bytes", 0)),
                                                             v.get("missing_files", 0), gb(v.get("missing_bytes", 0))))


if __name__ == "__main__":
    main()
