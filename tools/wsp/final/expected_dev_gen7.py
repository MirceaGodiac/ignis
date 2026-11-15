#!/usr/bin/env python3
"""The validation-partition numbers of every release newer than 13.3, as its own development result file states them,
for the rehearsal of run_final_gen7.py (PROTOCOL 13.6; development, in-sample). Reads only the producers of those
result files (runs/dev/gen7/*_dev.json, the JSON each docs/wsp/feedback/*_result.md was written from), never a case
file. Each entry names the release by its label in the release list, the coding, the metric as the final path names
it ("target: metric"), the expected number, the releases whose common cases it was computed on (`common_with`, the
rehearsal restricts to the same cases), and its source.

    python -I tools/wsp/final/expected_dev_gen7.py      # writes tools/wsp/final/expected_dev_gen7.json
"""
import json
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
G = REPO / "runs/dev/gen7"
LIST = REPO / "tools/wsp/final/releases_final_gen7.json"
OUT = REPO / "tools/wsp/final/expected_dev_gen7.json"


def main():
    lst = json.loads(LIST.read_text(encoding="utf-8"))
    by_id = {}
    for key, inv in lst["investigations"].items():
        for e in [inv["model"]] + inv["comparators"]:
            by_id.setdefault((e["release"], e["rule"]), (key, e["label"]))
    lab = lambda key, rid, rule="lane": by_id[(rid, rule)][1]  # noqa: E731
    out = []

    def add(key, coding, label, metric, value, common, source):
        out.append(dict(investigation=key, coding=coding, label=label, metric=metric, expected=value, common_with=common, source=source))

    # second engineer's revisions (tools/wsp/ext/law_revision.py dev): parent and candidate on the cases both answer
    for f, key, coding, res in [("cld_c1-fable_dev.json", "cld", "v1", "cld_c1_result.md"), ("efield_e1-fable_dev.json", "efield", "v1", "efield_e1_result.md"),
                                ("spice_p2-fable_dev.json", "spice", "v1", "spice_p2_result.md"), ("spice_p3-fable_dev.json", "spice", "v1", "spice_p3_result.md"),
                                ("slice2_s2-fable_dev.json", "slice", "v1", "slice2_s2_result.md"), ("samer_s1-fable_dev.json", "samer", "v1", "samer_s1_result.md"),
                                ("bass3_q1-fable_v1_dev.json", "bass", "v1", "bass3_s1_q1_result.md"), ("bass3_q1-fable_v2_dev.json", "bass", "v2", "bass3_s1_q1_result.md"),
                                ("bass3_os1-fable_v1_dev.json", "bass", "v1", "bass3_s1_q1_result.md"), ("bass3_os1-fable_v2_dev.json", "bass", "v2", "bass3_s1_q1_result.md")]:
        d = json.loads((G / f).read_text(encoding="utf-8"))
        pair = [lab(key, d["ids"]["parent"]), lab(key, d["ids"]["candidate"])]
        for role in ("parent", "candidate"):
            for t, ms in d["scores"][role].items():
                for m, v in ms.items():
                    if isinstance(v, dict) and v.get("validation") is not None:
                        add(key, coding, lab(key, d["ids"][role]), f"{t}: {m}", v["validation"], pair, f"{res} (runs/dev/gen7/{f})")
    # bre d3, d3m, W0 by lasts: on the cases W0 answers (bre_d3_result.md, first table)
    d = json.loads((G / "bre_d3_dev.json").read_text(encoding="utf-8"))
    w0 = lab("bre", lst["investigations"]["bre"]["comparators"][0]["release"], "flame_end")
    for col, rid_lab in [("d3 lasts", "bre d3 by the lasts rule"), ("d3m lasts", "bre d3m by the lasts rule"),
                         ("w0 lasts", "bre W0 by the lasts rule (phys_bre2.md section 2)")]:
        v = d["table"][f"common|validation|{col}"]
        for i, m in enumerate(("t1: extinction IBS", "t1: operator termination IBS")):
            add("bre", "v1", rid_lab, m, v[i], [w0], "bre_d3_result.md (runs/dev/gen7/bre_d3_dev.json, cases W0 answers)")
    d = json.loads((G / "bre_d2_dev.json").read_text(encoding="utf-8"))
    v = d["table"]["d2_lasts|validation"]
    for i, m in enumerate(("t1: extinction IBS", "t1: operator termination IBS")):
        add("bre", "v1", "bre d2 by the lasts rule", m, v[i], [w0], "bre_second_law_result.md (runs/dev/gen7/bre_d2_dev.json)")
    # cld3, C1, cld2 and efield3, E1, efield2: third_laws_dev.json (cases all four releases of the table answer)
    d = json.loads((G / "third_laws_dev.json").read_text(encoding="utf-8"))
    for key, names in (("cld", {"cld3": "cld3 (third law)", "C1": "cld C1", "cld2 s1": "cld2 (second law)", "D1": "cld D1"}),
                       ("efield", {"efield3": "efield3 (third law)", "E1": "efield E1", "efield2 s1": "efield2 (second law)", "W0": "efield W0"})):
        common = list(names.values())
        for col, label in names.items():
            if col in ("D1", "W0"):
                continue
            for m, v in d[key][col]["validation"]["metrics"].items():
                add(key, "v1", label, m, v, common, f"{'cld3' if key == 'cld' else 'efield3'}_result.md (runs/dev/gen7/third_laws_dev.json)")
    # plastics bass2 s1, K1, bass3 on both codings (bass3_result.md, cases the three answer)
    d = json.loads((G / "bass_codings_dev_three.json").read_text(encoding="utf-8"))
    names = {"bass2 s1": "bass2 s1", "bass2 K1": "bass2 K1", "bass3": "bass3"}
    mlab = {"quench": "interval CRPS cm/s, mean of day means", "sustain": "Brier, mean of day means",
            "quench_display": "interval CRPS, fan display units, mean of day means"}
    for coding, tab in d.items():
        for t in mlab:
            k = f"{t}|validation|all"
            if k not in tab:
                continue
            for col, label in names.items():
                dm, cm = tab[k][col]
                add("bass", coding, label, f"{t}: {mlab[t]}", dm, list(names.values()), "bass3_result.md (runs/dev/gen7/bass_codings_dev_three.json)")
                add("bass", coding, label, f"{t}: case mean", cm, list(names.values()), "bass3_result.md (runs/dev/gen7/bass_codings_dev_three.json)")
    # bass4: own answered cases (bass4_result.md, by-partition table)
    d = json.loads((G / "bass4_dev.json").read_text(encoding="utf-8"))
    for coding, x in d.items():
        for m, v in x["by_partition"]["bass4|validation"].items():
            if m != "answered":
                add("bass", coding, "bass4 (fabrics in kept burning)", m, v, [], "bass4_result.md (runs/dev/gen7/bass4_dev.json)")
    # SLICE case version 2: slice2 s1 and S2 (slice_cases_v2_result.md, lines the three answer)
    d = json.loads((G / "slice_cases_versions_dev.json").read_text(encoding="utf-8"))
    names = {"slice W0": "slice W0", "slice2 s1": "slice2 s1 (second law)", "slice2 S2": "slice2 S2"}
    for col in ("slice2 s1", "slice2 S2"):
        for m, v in d["v2"][col]["validation"]["metrics"].items():
            if not m.endswith("lines"):
                add("slice", "v2", names[col], m, v, list(names.values()), "slice_cases_v2_result.md (runs/dev/gen7/slice_cases_versions_dev.json)")
    OUT.write_text(json.dumps(out, indent=1) + "\n", encoding="utf-8", newline="\n")
    print(f"{len(out)} expected numbers -> {OUT.relative_to(REPO)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
