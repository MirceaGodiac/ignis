"""Third coding of the BASS (PSI-26) and BASS-II solids (PSI-25) outcomes, beside versions 1 and 2.

Made on 15 November 2026 AFTER the final run, with the test-day outcomes seen (PROTOCOL 13.8 opened the test
days as study data), after the coordinator found test burn PSI-25:O13a coded with a reversed display interval.
Versions 1 and 2 and their files are untouched. Rules R11 to R14 are listed in
docs/wsp/feedback/bass1_coding_v2.md, addendum; everything else is version 2.

    python tools/wsp/ext/bass1_cases_v3.py build   # cases_{train,validation,test}_*_v3.jsonl, counts only
    python tools/wsp/ext/bass1_cases_v3.py diff    # every case, any partition, whose coding differs from v2
    python tools/wsp/ext/bass1_cases_v3.py audit   # v2 cases whose extinction setting was reached by a flow increase
    python tools/wsp/ext/bass1_cases_v3.py swapped # every row, any partition, with fan displays in the air cell (R13)
"""
import argparse
import json
import re
import sys
from collections import Counter, defaultdict
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import bass1_cases as v1  # noqa: E402
import bass1_cases_v2 as v2  # noqa: E402

OUT = v1.OUT
_orig_partition = v1.partition_of


def _open_partition(inv, date):
    p = _orig_partition(inv, date)
    return "testopen" if p == "test" else p  # the readers skip 'test'; the test days are study data since 13.8


# --------------------------------------------------------------------------------------- v2 rules kept for the diff
V2 = {k: getattr(v2, k) for k in ("normalise", "steps_of", "extinction_value", "_outcome_of")}


# --------------------------------------------------------------------------------------- R12: durations
def normalise(text):
    """v2's R4, and durations written with a qualifier ('3+ min', '~3 min', 'about 3 min') are dropped too."""
    t = re.sub(r"(?<![\d.])\d+(?:\.\d+)?\s*\+?\s*(?=-?\s*(?:min|mins|minutes?|sec|seconds?|s)\b)", "", text.lower())
    return V2["normalise"](t)


# --------------------------------------------------------------------------------------- R13: displays in the air cell
DISPLAY_IN_AIR = re.compile(r"(?<![\d.])(\d+(?:\.\d+)?)(?!\s*cm/s)(?![\d.])")


def displays_in_air_cell(rec, fc):
    """Fan displays written in the air cell. A number between 38 and 122 in the air cell, not followed by 'cm/s',
    cannot be an anemometer reading when the row's highest setting is below pot 6 (display 88.5): the calibrations
    give at most 39 cm/s at pot 6 even without a restrictor, and 23 cm/s with two. Such a number is the fan
    display of a step (rows of 2014-07-21: fan cell '1.3, 0.7, 0.45, 0.36, 0.32, 0.31', air cell
    '49,44,42,41,41,40.5 (5, 2, 0)'). Returns the displays found, or [] when the test does not apply."""
    f = v2.fan_cell_steps(rec["fan"], fc, [(38.0, 0.0), (122.0, 40.0)])
    top = max([d for _, _, d, _ in f], default=None)
    if re.search(r"\bmax", rec["fan"].lower()):
        top = 122.0  # 'max' is pot 10
    if top is None or top >= v1.pot_to_fan(6.0, fc):
        return []
    return [float(m.group(1)) for m in DISPLAY_IN_AIR.finditer(rec["air"].lower()) if 38 <= float(m.group(1)) <= 122]


def steps_of(rec, fc, curve, prefer_air=False):
    """v2's steps, after removing from the air cell the fan displays found by R13 (they are never read as cm/s;
    the pots of the fan cell carry the same steps)."""
    if displays_in_air_cell(rec, fc):
        air = re.sub(r"(?<![\d.])(\d+(?:\.\d+)?)(?!\s*cm/s)(?![\d.])",
                     lambda m: "" if 38 <= float(m.group(1)) <= 122 else m.group(0), rec["air"])
        air = re.sub(r"\(\s*[, ?]*\)", "", air)
        rec = dict(rec, air=air)
    return V2["steps_of"](rec, fc, curve, prefer_air)


# --------------------------------------------------------------------------------------- R14: counts are not settings
def extinction_value(text, fc, curve, cell=False):
    """v2's rule, with 'N pot settings' / 'N settings' / 'N steps' read as a count, not a setting."""
    t = re.sub(r"(?<![\d.])\d+\s+(?=(?:pot\s+|fan\s+)?(?:settings|steps|increments|changes)\b)", "", text.lower())
    if not cell:
        t = inherit_units(t)
    return V2["extinction_value"](t, fc, curve, cell)


def inherit_units(t):
    """R15: within one clause (between '.' or ';'), a setting written 'to N' or 'at N' without a unit takes the
    unit of the last setting written with one, for a whole number only ( to 20 cm/s ..., turned down flow to 15 and got
    ... extinction there' is 15 cm/s). Without an earlier unit in the clause the number is left as it is."""
    out = []
    for clause in re.split(r"((?<!\d)\.|\.(?!\d)|;)", t):  # a point between digits is a decimal point
        unit = None

        def fix(m):
            nonlocal unit
            if m.group(3):
                unit = m.group(3)
                return m.group(0)
            if "." in m.group(2):
                return m.group(0)  # a bare decimal ('at 0.35') is a pot setting, never an inherited cm/s
            return m.group(0) + (" " + unit if unit else "")
        out.append(re.sub(r"\b(to|at)\s+(\d*\.\d+|\d+)(?![\d.:])(?:\s*(cm/s|pot))?", fix, clause))
    return "".join(out)


# --------------------------------------------------------------------------------------- R11: lowering only
def _outcome_of(rec, inp, fc):
    """v2's coding, then R11: a low-flow quench needs the flame to have gone out at a setting LOWER than the one
    it held just before, on the display (or, for a step written only in cm/s, the speed). If the setting it went
    out at was reached by raising the flow, the burn is `out_after_increase` and leaves the low-flow target."""
    o = V2["_outcome_of"](rec, inp, fc)
    if o.get("quench_class") in ("out_at_finite_flow", "out_no_upper"):
        lo_d, hi_d = o.get("quench_display_lo"), o.get("quench_display_hi")
        lo_u = o.get("quench_u_lo_cm_s")
        steps = o["run_record"]["steps"]
        raised = False
        if lo_d is not None and hi_d is not None and lo_d >= hi_d - 1e-9:
            raised = True
        elif (o["quench_class"] == "out_no_upper" and lo_u is not None) or o.get("quench_history", "").startswith("a lower"):
            # the setting written just before the failed one, in the operators' order
            us = [s["u_cm_s"] for s in steps]
            at = [k for k, u in enumerate(us) if lo_u is not None and abs(u - lo_u) < 1e-3]
            if at and at[-1] > 0 and us[at[-1] - 1] < lo_u - 1e-9:
                raised = True
            if o["quench_class"] == "out_no_upper" and lo_u is not None:
                # v2's R8: out at a setting after the flow had been changed, with no higher setting held before:
                # the failed setting was the highest of the burn, so it was reached by raising the flow
                raised = True
        if raised:
            for k in ("quench_u_lo_cm_s", "quench_u_hi_cm_s", "quench_display_lo", "quench_display_hi"):
                o[k] = None
            o.update(quench_class="out_after_increase", quench_hi_open=False,
                     quench_rule="went out after the flow was raised to the failed setting: not the low-flow branch "
                                 "(R11); v2 had: " + (o.get("quench_rule") or ""))
    o["coding"] = "v3"
    return o


def install():
    v2.normalise = normalise
    v2.steps_of = steps_of
    v2.extinction_value = extinction_value
    v2._outcome_of = _outcome_of


def restore():
    for k, f in V2.items():
        setattr(v2, k, f)


def build_all():
    """(rec, inputs, v2 outcome, v3 outcome) for every train, validation and test burn."""
    v1.partition_of = _open_partition
    try:
        fc = v1.calibrations()
        recs = v1.read_psi26() + v1.read_psi25()
        finals = v2.final_o2()
        prior = Counter()
        res = []
        for rec in recs:
            if rec["partition"] == "held":
                continue
            rec["x_o2_final"] = finals.get(rec["case_id"]) if rec["partition"] != "testopen" else _final_o2_test(rec)
            key = (rec["investigation"], rec["sample"])
            restore()
            i2 = v2.inputs_of(rec, fc, prior[key])
            o2 = v2.outcome_of(rec, i2, fc)
            o2["_ign_v2"] = i2["u_cm_s_at_ignition"]
            install()
            i3 = v2.inputs_of(rec, fc, prior[key])
            i3["coding"] = "v3"
            o3 = v2.outcome_of(rec, i3, fc)
            restore()
            prior[key] += 1
            part = "test" if rec["partition"] == "testopen" else rec["partition"]
            res.append((part, rec, i3, o2, o3))
        return res
    finally:
        v1.partition_of = _orig_partition
        restore()


_FINAL_TEST = None


def _final_o2_test(rec):
    global _FINAL_TEST
    if _FINAL_TEST is None:
        import openpyxl
        wb = openpyxl.load_workbook(v1.MATRIX_25, data_only=True, read_only=True)
        _FINAL_TEST = {}
        for ws in wb.worksheets:
            for r in list(ws.iter_rows(values_only=True))[1:]:
                if r and r[2] is not None and isinstance(r[11], (int, float)):
                    _FINAL_TEST["PSI-25:%s" % str(r[2]).strip()] = round(r[11] / 100.0, 4)
    return _FINAL_TEST.get(rec["case_id"])


def cmd_build(_):
    res = build_all()
    by = defaultdict(list)
    for part, rec, i3, o2, o3 in res:
        by[part].append((i3, o3))
    v1.write_jsonl(OUT / "cases_train_v3.jsonl", [dict(i, outcome=o) for i, o in by["train"]])
    for part in ("validation", "test"):
        v1.write_jsonl(OUT / ("cases_%s_inputs_v3.jsonl" % part), [i for i, o in by[part]])
        v1.write_jsonl(OUT / ("cases_%s_outcomes_v3.jsonl" % part), [o for i, o in by[part]])
    cnt = {p: {"burns": len(v), "sustain": sum(o["sustain"] is not None for _, o in v),
               "quench v3": sum(v2.quench_scored(o) for _, o in v),
               "quench v2": sum(v2.quench_scored(o2) for pp, r, i, o2, o3 in res if pp == p)} for p, v in by.items()}
    print(json.dumps(cnt, indent=1))
    return 0


def _brief(o):
    lo, hi = o.get("quench_u_lo_cm_s"), o.get("quench_u_hi_cm_s")
    dl, dh = o.get("quench_display_lo"), o.get("quench_display_hi")
    return "%s %s u[%s, %s] d[%s, %s]%s" % (o["sustain"], o["quench_class"], lo, "open" if o.get("quench_hi_open") else hi,
                                          dl, dh, " n2" if o["n2_applied"] else "")


def cmd_diff(_):
    n = 0
    res = build_all()
    o2_ign = {r["case_id"]: o2["_ign_v2"] for _, r, _, o2, _ in res}
    for part, rec, i3, o2, o3 in res:
        k = ("sustain", "quench_class", "quench_u_lo_cm_s", "quench_u_hi_cm_s", "quench_display_lo", "quench_display_hi")
        ign2 = o2_ign.get(rec["case_id"])
        if any(o2.get(x) != o3.get(x) for x in k) or ign2 != i3["u_cm_s_at_ignition"]:
            n += 1
            print(" | ".join((part, rec["case_id"], _brief(o2), _brief(o3), o3["quench_rule"] or o3["sustain_rule"] or "",
                              "ignition %s -> %s cm/s" % (ign2, i3["u_cm_s_at_ignition"]))))
    print(n, "cases changed")
    return 0


def cmd_audit(_):
    """v2 scored cases whose failed setting is at or above the setting written just before it."""
    for part, rec, i3, o2, o3 in build_all():
        if not v2.quench_scored(o2):
            continue
        steps = o2["run_record"]["steps"]
        dl, dh = o2.get("quench_display_lo"), o2.get("quench_display_hi")
        us = [s["u_cm_s"] for s in steps]
        lo = o2["quench_u_lo_cm_s"]
        at = [k for k, u in enumerate(us) if lo is not None and abs(u - lo) < 1e-3]
        prev_lower = bool(at and at[-1] > 0 and us[at[-1] - 1] < lo - 1e-9)
        if (dl is not None and dh is not None and dl >= dh) or prev_lower:
            print(" | ".join((part, rec["case_id"], o2["quench_class"], "display [%s, %s]" % (dl, dh),
                              "steps " + ", ".join(s["setting"] for s in steps), "obs: " + rec["obs"][:160])))
    return 0


def cmd_swapped(_):
    fc = v1.calibrations()
    for part, rec, i3, o2, o3 in build_all():
        d = displays_in_air_cell(rec, fc)
        if d:
            print(" | ".join((part, rec["case_id"], "fan: " + rec["fan"], "air: " + rec["air"], "displays %s" % d)))
    return 0


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("cmd", choices=["build", "diff", "audit", "swapped"])
    a = ap.parse_args()
    return {"build": cmd_build, "diff": cmd_diff, "audit": cmd_audit, "swapped": cmd_swapped}[a.cmd](a)


if __name__ == "__main__":
    sys.exit(main())
