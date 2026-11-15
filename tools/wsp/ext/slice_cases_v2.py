#!/usr/bin/env python3
"""SLICE cases, version 2: the burner tube of each flame taken from the investigators' picture table where it
has the flame, the summary log otherwise. Version 1 (`slice_cases.py`, `cases_*.jsonl`) is left untouched.

Why: the summary log's "Burner tube" and "Fuel (bottle)" cells were found copied down from earlier lines on two
train days (docs/wsp/feedback/slice_tube_check.md). `Raw Data/pic_info_ISS.txt.docx` (one row per still: test
point, burner, fuel-type code, folder, FUEL/AIR/FAN displays, time) records the same flames independently; its
fuel code agrees with the bottle table on every train and validation day, GMT 061 included, where the log does not.

Rule, the same on every day (no outcome is read):
  1. Picture rows of the day are grouped into runs: same test-point label, no gap above 90 s.
  2. A log flame takes the run with its own label; a label with two runs takes the earliest unmatched one.
  3. A log flame without a run of its label takes the unmatched run of the day that lies in time between the runs
     of its matched neighbours in the log, has the flame's fuel code, and whose pictures show, within 3 units, every logged (FUEL, FAN) setting of the flame (and, when the log line has a readable time,
     starts within 180 s of it), if exactly one such run exists.
  4. The run's burner, if all its pictures agree, replaces the log's; otherwise the log's stays.
Then `burner_id_mm` and `u_jet_m_s` are recomputed. Only train and validation days exist in version 1, so only
they are rebuilt; picture-table folders of test days are skipped by name.

Writes cases_train_v2.jsonl, cases_validation_inputs_v2.jsonl, cases_validation_outcomes_v2.jsonl (copied
unchanged: outcomes do not depend on the tube) and tube_changes_v2.json, under transfer_slice/.
"""
import json
import math
import re
import sys
import zipfile
from collections import defaultdict
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from slice_cases import BURNER_ID, OUT, RAW  # noqa: E402

PIC = RAW / "pic_info_ISS.txt.docx"
FOLDER_DAY = {"040_splice_forHSV_UT040": "040", "UT048_Splice_images_DSC957": "048",
              "059_UT058_forHSV_splice_images_DSC957": "058", "061_UT061_spice_images_forHSV": "061",
              "065_UT065_for_HSV_Splice_1_DSC": "065", "069_ForHSV_SPLICE": "069",
              "072_UT072_forHSV_Splice_DSC957": "072", "075_forHSV_Splice": "075",
              "UT079_forHSV_SPLICE_DSC957": "079"}  # train and validation folders
# Test-day folders, used ONLY with final_run=True (the final run of PROTOCOL 12.5, tools/wsp/final/run_final_gen7.py).
# Names from the file catalogue's folder list; "084_..._ut083c" is the second picture folder of GMT 083 ("UT083c").
TEST_FOLDER_DAY = {"00_UT044_ForHSV_Splice_DSC957": "044", "UT051_Splice_DSC957": "051",
                   "083_forHSV_splice_ut083_DSC957": "083", "084_ForHSV_splice_ut083c_DSC957": "083"}
TEST_DAYS = ("044", "051", "083")
# The ONLY columns of the picture table this builder reads (index: header name), on every day. Never read: file name
# prefix, AIR display, file number, exposure time, f-number, Rmax, Gmax, Bmax, radiometer (Rad), frame number,
# saturation. The colour maxima, the radiometer and the saturation flag are observations of the flame.
COLUMNS = {0: "Test Point", 1: "Burner", 2: "Fuel type", 3: "folder", 5: "Fuel", 7: "Fan", 9: "Time"}


# fuel-type code of the picture table, read off the days whose bottle is not in doubt (Table 6 of the 2017 report)
FUEL_CODE = {"1": ("methane", 1.0), "2": ("methane", 0.7), "3": ("methane", 0.4), "4": ("ethylene", 1.0),
             "5": ("ethylene", 0.2), "6": ("ethane", 1.0), "7": ("propane", 1.0)}


def _table_lines():
    x = zipfile.ZipFile(PIC).read("word/document.xml").decode("utf8")
    for q in re.findall(r"<w:p[ >].*?</w:p>", x, flags=re.S):
        q = re.sub(r"<w:tab/>", "\t", q)
        yield [c.strip() for c in "".join(a or b for a, b in re.findall(r"<w:t(?: [^>]*)?>(.*?)</w:t>|(\t)", q, flags=re.S)).split("\t")]


def folder_map(final_run):
    return {**FOLDER_DAY, **(TEST_FOLDER_DAY if final_run else {})}


def check_coverage(days, final_run):
    """Fail closed. Test-day cases without the flag are refused (version 2 would otherwise keep the log's tube on
    them without saying so); with the flag, every test day asked for must have a folder in the picture table.
    Reads only the folder column."""
    test_asked = sorted(set(days) & set(TEST_DAYS))
    if test_asked and not final_run:
        raise SystemExit(f"refused: cases of test days {test_asked} given without final_run=True (--final-run); "
                         "version 2 applies the picture-table rule to test days only in the final run")
    if final_run and test_asked:
        present = {TEST_FOLDER_DAY[f[3]] for f in _table_lines() if len(f) > 3 and f[3] in TEST_FOLDER_DAY}
        missing = [d for d in test_asked if d not in present]
        if missing:
            raise SystemExit(f"refused: the picture table {PIC.name} has no folder for test days {missing} "
                             f"(expected {sorted(k for k, v in TEST_FOLDER_DAY.items() if v in missing)})")


def pic_runs(days, final_run=False):
    """Picture runs of the given days. A row is kept only if its folder (column 3) maps to one of `days`; only the
    columns of COLUMNS are then read from it."""
    fmap = folder_map(final_run)
    rows = []
    for n, f in enumerate(_table_lines()):
        if n == 0:
            assert all(f[i] == name for i, name in COLUMNS.items()), f"picture table header changed: {f}"
            continue
        if len(f) < 10 or f[3] not in fmap or fmap[f[3]] not in days:
            continue
        t = re.match(r"(\d+):(\d+):(\d+)", f[9])
        if not t:
            continue
        rows.append(dict(day=fmap[f[3]], label=f[0].upper(), burner=f[1], fuel_code=f[2],
                         fuel=float(f[5]) if re.match(r"^[\d.]+$", f[5]) else None,
                         fan=float(f[7]) if re.match(r"^[\d.]+$", f[7]) else None,
                         t=int(t[1]) * 3600 + int(t[2]) * 60 + int(t[3])))
    runs = defaultdict(list)
    for r in sorted(rows, key=lambda r: (r["day"], r["t"])):
        lst = runs[r["day"]]
        if lst and lst[-1]["label"] == r["label"] and r["t"] - lst[-1]["rows"][-1]["t"] <= 90:
            lst[-1]["rows"].append(r)
        else:
            lst.append(dict(label=r["label"], rows=[r], used=False))
    return runs


def log_time(text):
    m = re.search(r"(\d{1,2}):(\d{2})(?::(\d{2}))?$", text or "")
    return int(m[1]) * 3600 + int(m[2]) * 60 + int(m[3] or 0) if m else None


def covers(run, lines):
    """Every logged (FUEL, FAN) setting of the flame appears among the run's pictures, within 3 display units."""
    pts = [(r["fuel"], r["fan"]) for r in run["rows"] if r["fuel"] is not None and r["fan"] is not None]
    for c in lines:
        f, a = c["inputs"]["fuel_display"], c["inputs"]["fan_display"]
        if f is None or a is None:
            continue
        if not any(abs(f - pf) <= 3 and abs(a - pa) <= 3 for pf, pa in pts):
            return False
    return True


def apply_tubes(cases, runs, changes):
    """The rule, in place on `cases` (one file's cases); appends every changed line to `changes`."""
    # flames in log order; a flame continued on the next page keeps the label, so merge by (day, test) order
    order, seen = [], {}
    for c in cases:
        k = (c["day"], c["test"])
        if k not in seen:
            seen[k] = []
            order.append(k)
        seen[k].append(c)
    assign = {}
    for k in order:  # pass 1: by label
        day, test = k
        cand = [r for r in runs.get(day, []) if r["label"] == str(test).upper() and not r["used"]]
        if cand:
            cand[0]["used"] = True
            assign[k] = (cand[0], "label")
    for n, k in enumerate(order):  # pass 2: by first settings, between the neighbours matched in pass 1
        if k in assign:
            continue
        day, test = k
        first = seen[k][0]["inputs"]
        before = [assign[o][0]["rows"][0]["t"] for o in order[:n] if o in assign and o[0] == day]
        after = [assign[o][0]["rows"][0]["t"] for o in order[n + 1:] if o in assign and o[0] == day]
        t0, t1 = (max(before) if before else -1), (min(after) if after else 10 ** 6)
        tlog = log_time(seen[k][0].get("gmt_time"))
        cand = [r for r in runs.get(day, []) if not r["used"] and t0 < r["rows"][0]["t"] < t1
                and FUEL_CODE.get(r["rows"][0]["fuel_code"]) == (first["fuel_hydrocarbon"], first["x_fuel"])
                and (tlog is None or abs(r["rows"][0]["t"] - tlog) <= 180)
                and covers(r, seen[k])]
        if len(cand) == 1:
            cand[0]["used"] = True
            assign[k] = (cand[0], "logged settings, fuel code and time order")
    for k, lst in seen.items():
        if k not in assign:
            continue
        run, how = assign[k]
        burners = {r["burner"] for r in run["rows"]}
        if len(burners) != 1 or next(iter(burners)) not in BURNER_ID:
            continue
        b = next(iter(burners))
        for c in lst:
            i = c["inputs"]
            old = i["burner_nominal_mm"]
            c.setdefault("v2", {})["tube_source"] = f"pic_info_ISS run '{run['label']}' ({how})"
            if float(b) != old:
                lo, hi = BURNER_ID[b]
                d = (lo + hi) / 2
                i["burner_nominal_mm"], i["burner_id_mm"] = float(b), d
                if i["q_jet_sccm"]:
                    i["u_jet_m_s"] = (i["q_jet_sccm"] * 1e-6 / 60) / (math.pi * (d * 1e-3) ** 2 / 4)
                c["v2"]["tube_log_mm"] = old
                changes.append(dict(case_id=c["case_id"], partition=c["partition"], log_tube_mm=old,
                                    v2_tube_mm=float(b), matched_by=how, picture_label=run["label"]))
    for c in cases:
        c.setdefault("v2", {}).setdefault("tube_source", "summary log (no matching picture run)")
    return cases


FILES = {"cases_train.jsonl": "cases_train_v2.jsonl", "cases_validation_inputs.jsonl": "cases_validation_inputs_v2.jsonl"}


def load(src):
    return [json.loads(l) for l in (OUT / src).read_text(encoding="utf-8").splitlines()]


def main(final_run=False):
    """final_run=True is passed by the final run only: it extends the rule to the test days' picture folders."""
    data = {src: load(src) for src in FILES}
    days = {c["day"] for cs in data.values() for c in cs}
    check_coverage(days, final_run)
    runs = pic_runs(days, final_run)
    changes = []
    for src, dst in FILES.items():
        cases = apply_tubes(data[src], runs, changes)
        with (OUT / dst).open("w", encoding="utf-8", newline="\n") as fh:
            for c in cases:
                fh.write(json.dumps(c, ensure_ascii=False) + "\n")
    (OUT / "cases_validation_outcomes_v2.jsonl").write_bytes((OUT / "cases_validation_outcomes.jsonl").read_bytes())
    (OUT / "tube_changes_v2.json").write_text(json.dumps(changes, indent=1), encoding="utf-8")
    if final_run:
        print("final run: tube rule applied;", len(changes), "lines changed (not itemised)")
        return 0
    by = defaultdict(int)
    for ch in changes:
        by[(ch["partition"], ch["case_id"].split(":")[1], ch["case_id"].split(":")[3], ch["log_tube_mm"], ch["v2_tube_mm"], ch["matched_by"])] += 1
    for k, v in sorted(by.items()):
        print(k, v, "lines")
    print("lines changed:", len(changes))
    return 0


def check_flag_noop():
    """Dry check on the train and validation files: the rule with and without final_run gives identical cases.
    Writes nothing; reads no test-day row of the picture table (rows are filtered by the days of the input)."""
    out = {}
    for flag in (False, True):
        data = {src: load(src) for src in FILES}
        days = {c["day"] for cs in data.values() for c in cs}
        assert not days & set(TEST_DAYS), "the dry check runs on train and validation days only"
        check_coverage(days, flag)
        runs = pic_runs(days, flag)
        ch = []
        out[flag] = ({src: apply_tubes(data[src], runs, ch) for src in FILES}, ch)
    same = out[False] == out[True]
    print("flag changes nothing on train and validation:", same, "| lines changed either way:", len(out[False][1]))
    return 0 if same else 1


if __name__ == "__main__":
    import argparse
    ap = argparse.ArgumentParser()
    ap.add_argument("--final-run", action="store_true", help="final run only: apply the rule to the test days too")
    ap.add_argument("--check-flag-noop", action="store_true", help="dry check on train and validation days; writes nothing")
    a = ap.parse_args()
    sys.exit(check_flag_noop() if a.check_flag_noop else main(final_run=a.final_run))
