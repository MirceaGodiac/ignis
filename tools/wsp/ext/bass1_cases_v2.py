"""Second coding of the BASS (PSI-26) and BASS-II solids (PSI-25) outcomes, beside the first.

The first version (`bass1_cases.py`, files `cases_*.jsonl`) is left untouched: registered looks were scored on
it. This script reuses its readers, splits, inputs and fan curves, and changes only the rules listed in
docs/wsp/feedback/bass1_coding_v2.md (R1 to R10), each stated generally so that it can be applied unchanged to
the test days in the final run. It also records the fan display (volts x 10) of every flow step, so the outcome
can be stated on the display scale (`quench_display_lo/hi`) beside the speed scale (`quench_u_lo/hi_cm_s`).

    python tools/wsp/ext/bass1_cases_v2.py build      # writes cases_*_v2.jsonl, prints counts only
    python tools/wsp/ext/bass1_cases_v2.py diff       # every train/validation case whose coding changed
    python tools/wsp/ext/bass1_cases_v2.py spread     # day-to-day spread of the cm/s value of fixed settings
    python tools/wsp/ext/bass1_cases_v2.py baselines  # constant baselines on both scales, train and validation

Run-record fields (the display at every step, the last held and first failed step) are written into the
outcome part of a case, never into an inputs file: they are the operators' response to the flame.
"""
import argparse
import json
import math
import re
import statistics
import sys
from collections import Counter, defaultdict
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import bass1_cases as v1  # noqa: E402

OUT = v1.OUT
FAN_OFF = v1.FAN_OFF
NUM = v1.NUM

# ----------------------------------------------------------------------------------------------- text
def normalise(text):
    """R4: join words the pdf split at a hyphen ('self- extinguished'), and drop durations ('after 15 min',
    '29-min burn', '15 s') so that a duration is never read as a flow setting."""
    t = text.lower().replace("pots", "pot")
    t = re.sub(r"(\w)-\s+(\w)", r"\1-\2", t)
    t = re.sub(r"(?<![\d.])\d+(?:\.\d+)?\s*-?\s*(?:min|mins|minutes?|sec|seconds?|s)\b", " ", t)
    return t


# ----------------------------------------------------------------------------------------------- steps
def _cell_numbers(cell):
    """Numbers of a cell. R1: a bracket holding a single number right after a value restates that value
    ('116 (pot 9.3)', 'pot 1.2 (48)', '44 (pot 0.68)') and is dropped as a separate step. Clock times dropped."""
    s = cell.lower().replace("pots", "pot")
    s = re.sub(r"(\d)\s*\(\s*(?:pot\s*)?(\d*\.\d+|\d+)\s*\)", r"\1", s)
    s = re.sub(r"(\d)\s*(?=ext\b)", r"\1 ", s)  # R6: '0.3ext' is the setting 0.3 and the marker 'ext'
    return v1._numbers(s)


def fan_cell_steps(cell, fc, curve):
    """As v1, with R1; each step is (label, U cm/s, fan display, display derived?)."""
    s, nums = _cell_numbers(cell)
    small_pots = any(n[0] < 1 and n[4] for n in nums)
    out = []
    for v, before, after, inside, dec, lt in nums:
        if v >= 36:
            out.append(("fan %.4g" % v, v1.fan_to_u(v, curve), v, False))
        elif v == 0:
            out.append(("fan off", 0.0, FAN_OFF, False))
        elif v <= 12:
            pot_cell = re.match(r"\s*(fan\s+)?pot", s)  # R1: a cell that opens with 'pots' lists pots only
            if not dec and small_pots and "pot" not in before and not after.strip().startswith("pot") and not pot_cell:
                out.append(("air %.3g cm/s" % v, 1.0066 * v, v1.u_to_fan(1.0066 * v, curve), True))
            else:
                d = v1.pot_to_fan(v, fc)
                out.append(("pot %.3g" % v, v1.fan_to_u(d, curve), d, False))
    return out


def air_cell_steps(cell, fc, curve):
    s, nums = _cell_numbers(cell)
    out = []
    for v, before, after, inside, dec, lt in nums:
        if "pot" in before or after.strip().startswith("pot") or inside or s.strip().startswith("pot"):
            if v <= 12:
                d = v1.pot_to_fan(v, fc)
                out.append(("pot %.3g" % v, v1.fan_to_u(d, curve), d, False))
        elif v <= 60:
            u = 1.0066 * (v / 2 if lt else v)
            out.append(("air %s%.3g cm/s" % ("<" if lt else "", v), u, v1.u_to_fan(u, curve), True))
    return out


def steps_of(rec, fc, curve, prefer_air=False):
    """As v1 (with R1). When fan and air cells list the same number of steps they are paired: the display is the
    fan's, the speed the anemometer's when it reads at least 2 cm/s. An air reading with no fan value beside
    it gets its display from the burn's fan curve, marked derived."""
    f = fan_cell_steps(rec["fan"], fc, curve)
    a = air_cell_steps(rec["air"], fc, curve)
    a_plain = [x for x in a if x[0].startswith("air")]
    a_pot = [x for x in a if x[0].startswith("pot")]
    paired = bool(f) and len(f) == len(a_plain)
    if paired:
        pairs = [(fa[0] + " / " + aa[0], aa[1] if aa[1] >= 2 else fa[1], fa[2], fa[3]) for fa, aa in zip(f, a_plain)]
        return pairs + a_pot
    if prefer_air and a_plain:
        return a_plain + a_pot
    if f:
        return f + a_pot
    return a


# ----------------------------------------------------------------------------------------------- rules
NOT_A_TEST = re.compile(v1.NOT_A_TEST.pattern)
IGNITER_FAILED = re.compile(r"(broke|broken|failed)\s+(\w+\s+){0,2}igniters?|igniters?\s+(\w+\s+){0,2}(broke|broken|failed)")
NOT_SUSTAINED = re.compile(v1.NOT_SUSTAINED.pattern +
                           r"|(did not|didn't|could not|couldn't|no luck)\s+(keep\w*\s+)?(the\s+)?(flame\s+)?sustain\w*")
BRIEF = re.compile(r"brief flames?\b")
GREW = re.compile(r"grew|wrap|spread|burn")


def not_sustained(obs):
    """R2: v1's words, plus 'did not sustain' / 'no luck keeping the flame sustained', plus 'brief flame' when no
    later words say the flame grew, wrapped, spread or burned (brief flames during repeated ignition attempts
    followed by a burn are a sustained burn)."""
    if NOT_SUSTAINED.search(obs):
        return True
    m = BRIEF.search(obs)
    return bool(m) and not GREW.search(obs[m.end():])
POSITIVE_FLAME = re.compile(r"ignited|lit\b|flame (was|grew|spread|wrapped)|spread|burned|burning|sooty|wrapped")
EXT = re.compile(v1.EXT.pattern + r"|\bdied\b")
OFF = re.compile(v1.OFF.pattern + r"|(fan )?pot (down )?to 0(?![.\d])|fan to 0(?![.\d])")
COMPLETE = re.compile(v1.COMPLETE.pattern + r"|reached (the )?end of (the )?sample")


def extinction_value(text, fc, curve, cell=False):
    """v1's rule with R4 (normalised text), R5 (a value with an explicit unit, 'extinguished 8 cm/s', needs no
    key word; 'died' is an extinction word) and R6 (cells: 'ext' glued to a number)."""
    t = normalise(text)
    if cell:
        t = re.sub(r"(\d)(ext\b)", r"\1 \2", t)
    for m in EXT.finditer(t):
        word = m.group(0)
        if not cell and word in ("out", "ext") and not re.match(r"\s*(at|when)\b", t[m.end():m.end() + 6]):
            continue
        head = t[max(0, m.start() - 14):m.start()]
        tail = t[m.end():m.end() + 40]
        if re.search(r"(not|n't)\s+(\w+\s+)?$", head) and not tail.strip().startswith("until"):
            continue
        mm = re.match(r"([^\d:;.]{0,34}?)(\d*\.\d+|\d+)(?!\.?\d)(?:\s*-\s*(\d*\.\d+|\d+)(?!\.?\d))?\s*(cm/s|pot)?", tail)
        if mm and (cell or v1.KEY.search(mm.group(1)) or (mm.group(4) and not mm.group(1).strip())) and \
                not re.search(r"increas|\bup\b", mm.group(1)):
            v = float(mm.group(3) or mm.group(2))
            got = v1._value(v, mm.group(4), mm.group(1), fc, curve)
            if got and mm.group(3):
                held = v1._value(float(mm.group(2)), mm.group(4), mm.group(1), fc, curve)
                return got[0], got[1] + " (from %s)" % held[1], held[0]
            if got:
                return got
        hb = t[max(0, m.start() - 32):m.start()]
        mh = (re.search(r"(?<![\d.:])(\d*\.\d+|\d+)\s*,?\s*$", hb) if cell else
              re.search(r"(?:down to|to)\s*~?(\d*\.\d+|\d+)\s*(cm/s)?\s*(?:ish)?\s*\(?\s*(?:flame\s+)?$", hb))
        if mh:
            got = v1._value(float(mh.group(1)), (mh.group(2) if not cell and mh.lastindex and mh.lastindex >= 2 else None),
                            "", fc, curve)
            if got:
                return got
        if not cell:
            # the clause names the setting just before the extinction word: 'increased flow to 14 cm/s, but it
            # self-extinguished' (R5)
            clause = re.split(r"[.;]", t[:m.start()])[-1]
            mc = list(re.finditer(r"(?:to|at)\s+(\d*\.\d+|\d+)\s*(cm/s|pot)", clause))
            if mc and re.search(r"\bbut\b|\band\b|,", clause[mc[-1].end():]):
                got = v1._value(float(mc[-1].group(1)), mc[-1].group(2), "", fc, curve)
                if got:
                    return got
    return None


def _display_of(u, steps, curve):
    """Display of a setting given as a speed: the paired fan value of a step with that speed, else derived."""
    for _, su, d, der in steps:
        if abs(su - u) < 1e-6:
            return d, der
    return v1.u_to_fan(u, curve), True


def outcome_of(rec, inp, fc):
    o = _outcome_of(rec, inp, fc)
    # the display interval through one campaign curve (no day calibration), for comparison with quench_u
    camp = fc["campaign"][str(inp["flow_restrictors"] if inp["flow_restrictors"] in (0, 1, 2) else 2)]
    for end in ("lo", "hi"):
        d = o.get("quench_display_" + end)
        o["quench_u_%s_campaign_cm_s" % end] = round(v1.fan_to_u(d, camp), 3) if d is not None else None
    return o


def _outcome_of(rec, inp, fc):
    text = " ".join((rec["obs"], "| fan:", rec["fan"], "| air:", rec["air"])).lower()
    obs = normalise(rec["obs"])
    curve = inp["fan_curve"]
    o = {"case_id": rec["case_id"], "day_group": rec["date"], "coding": "v2", "sustain": None, "sustain_rule": None,
         "quench_class": None, "quench_u_lo_cm_s": None, "quench_u_hi_cm_s": None, "quench_hi_open": False,
         "quench_display_lo": None, "quench_display_hi": None, "quench_display_derived": False,
         "quench_rule": None, "n2_applied": bool(v1.N2.search(obs)) and "vitiat" not in obs,
         "x_o2_final_post_burn": rec.get("x_o2_final")}
    steps = steps_of(rec, fc, curve)
    o["run_record"] = {"steps": [{"setting": lab, "u_cm_s": round(u, 3), "fan_display": round(d, 2),
                                  "display_derived": der} for lab, u, d, der in steps]}
    # R3: an igniter that broke or failed, and no flame at all
    if (NOT_A_TEST.search(obs) and not re.search(r"ignited (at|on|after|within|quickly|nicely|very)", obs)) or \
            (IGNITER_FAILED.search(obs) and re.search(r"no ignition|did not ignite", obs) and not POSITIVE_FLAME.search(obs)):
        o["sustain_rule"] = "not a test: igniter failed before any flame"
        return o
    if not_sustained(obs):  # R2
        o.update(sustain=0, sustain_rule="words: no sustained flame", quench_class="not_sustained")
        return o
    if not v1.BURNING.search(text):
        o["sustain_rule"] = "no words about the flame"
        return o
    o["sustain"], o["sustain_rule"] = 1, "words: burning"
    if v1.BLOWOFF.search(text) and not v1.NO_BLOWOFF.search(text):
        o["quench_class"], o["quench_rule"] = "blowoff", "high-flow blowoff: not the low-speed branch"
        return o
    off = OFF.search(obs) and re.search(r"extingu|out|flame", obs)
    # R7: an explicit stop of the fan in the observations outranks an extinction marker in the cells
    ev = extinction_value(rec["obs"], fc, curve)
    if ev is None and not off:
        ev = extinction_value(rec["fan"], fc, curve, cell=True) or extinction_value(rec["air"], fc, curve, cell=True)
    if ev is not None:
        steps = steps_of(rec, fc, curve, prefer_air=ev[1].startswith("air"))
    if "didn't complete" in obs or "did not complete" in obs:
        steps = steps[:1]
    us = [s[1] for s in steps]

    def held_step(u):
        return next(s for s in steps if abs(s[1] - u) < 1e-9)

    if ev is not None:
        lo, why = ev[0], ev[1]
        if len(ev) > 2:
            steps = [s for s in steps if s[1] > ev[2] * 1.0001] + [(why.split("(from ")[-1].rstrip(")"), ev[2],
                                                                     _display_of(ev[2], steps, curve)[0], False)]
            us = [s[1] for s in steps]
        d_lo, der_lo = _display_of(lo, steps, curve) if why.startswith("air") else (
            float(why.split()[1]) if why.startswith("fan") else v1.pot_to_fan(float(why.split()[1]), fc), False)
        above = [u for u in us if u > lo * 1.0001 + 1e-9]
        if not above and len(us) >= 2:
            # R8: out at a stated setting after the flow had been changed, with no higher setting held: the limit
            # is at least that setting. With a single setting the end is not attributed to the flow.
            o.update(quench_class="out_no_upper", quench_u_lo_cm_s=round(lo, 3), quench_hi_open=True,
                     quench_display_lo=round(d_lo, 2), quench_display_derived=der_lo,
                     quench_rule="out at %s; no higher setting held before: [lo, open)" % why)
            o["run_record"].update(first_failed=round(d_lo, 2))
            return o
        if not above:
            o["quench_class"], o["quench_rule"] = "out_no_upper", "out at %s at the only setting written" % why
            return o
        hi, history = min(above), "monotone"
        if us and lo > min(us) + 1e-9:
            at = [k for k, u in enumerate(us) if abs(u - lo) < 1e-6]
            prev = [u for u in us[:at[-1]] if u > lo * 1.0001 + 1e-9] if at else above
            hi, history = (prev[-1] if prev else min(above)), "a lower setting was held earlier"
        hs = held_step(hi)
        if der_lo and d_lo >= hs[2] - 1e-9:
            # a failed setting given only as an air reading, whose display through the curve is not below the held
            # display: take the display of the next written step after the held one, else leave the display open
            k = steps.index(hs)
            nxt = [s for s in steps[k + 1:] if s[2] < hs[2]]
            d_lo = nxt[0][2] if nxt else None
        o.update(quench_class="out_at_finite_flow", quench_u_lo_cm_s=round(lo, 3), quench_u_hi_cm_s=round(hi, 3),
                 quench_display_lo=round(d_lo, 2) if d_lo is not None else None,
                 quench_display_hi=round(hs[2], 2) if d_lo is not None else None,
                 quench_display_derived=bool(der_lo or hs[3]), quench_history=history,
                 quench_rule="out at %s; held %s before" % (why, hs[0]))
        o["run_record"].update(last_held=round(hs[2], 2), first_failed=round(d_lo, 2) if d_lo is not None else None)
        return o
    for cls, hit, words in (("out_at_fan_off", off, "burned at every setting down to %s, out when the fan was stopped"),
                            ("burned_to_completion", COMPLETE.search(obs), "burned to completion; lowest setting %s")):
        if hit:
            held = [s for s in steps if s[1] > 0]
            if not held:
                o["quench_class"], o["quench_rule"] = cls + "_no_step", "no step read"
                return o
            hs = min(held, key=lambda s: s[1])
            o.update(quench_class=cls, quench_u_lo_cm_s=0.0, quench_u_hi_cm_s=round(hs[1], 3),
                     quench_display_lo=FAN_OFF, quench_display_hi=round(hs[2], 2), quench_display_derived=hs[3],
                     quench_rule=words % hs[0])
            o["run_record"].update(last_held=round(hs[2], 2), first_failed=FAN_OFF if cls == "out_at_fan_off" else None)
            return o
    if v1.SELF.search(obs):
        o["quench_class"], o["quench_rule"] = "self_extinguished", "went out by itself at a held flow; no value read"
    else:
        o["quench_class"], o["quench_rule"] = "end_not_stated", "the words do not say how the flame ended"
    return o


def quench_scored(o):
    return (o.get("quench_u_hi_cm_s") is not None or o.get("quench_hi_open")) and not o.get("n2_applied")


def inputs_of(rec, fc, prior):
    """v1 inputs, with R10: the speed at ignition is the burn's own anemometer reading when it is given for the
    first step (>= 2 cm/s), through the v2 steps; and the display at ignition."""
    inp = v1.inputs_of(rec, fc, prior)
    st = steps_of(rec, fc, inp["fan_curve"])
    inp["ignition_setting"] = st[0][0] if st else None
    inp["u_cm_s_at_ignition"] = round(st[0][1], 3) if st else None
    inp["fan_display_at_ignition"] = round(st[0][2], 2) if st else None
    inp["fan_display_at_ignition_derived"] = st[0][3] if st else None
    inp["coding"] = "v2"
    return inp


# ----------------------------------------------------------------------------------------------- build
def load_all():
    fc = v1.calibrations()
    recs = v1.read_psi26() + v1.read_psi25()
    finals = final_o2()
    prior = Counter()
    out = []
    for rec in recs:
        if rec["partition"] in ("test", "held"):
            out.append((rec, None, None, None, None))
            continue
        rec["x_o2_final"] = finals.get(rec["case_id"])
        key = (rec["investigation"], rec["sample"])
        i1 = v1.inputs_of(rec, fc, prior[key])
        o1 = v1.outcome_of(rec, i1, fc)
        i2 = inputs_of(rec, fc, prior[key])
        o2 = outcome_of(rec, i2, fc)
        prior[key] += 1
        out.append((rec, i1, o1, i2, o2))
    return fc, out


def final_o2():
    """PSI-25 calibrated final O2 (vol %, / 100) by case id; measured after the burn (not a forecast input)."""
    import openpyxl
    wb = openpyxl.load_workbook(v1.MATRIX_25, data_only=True, read_only=True)
    res = {}
    for ws in wb.worksheets:
        for r in list(ws.iter_rows(values_only=True))[1:]:
            if r and r[2] is not None and v1.partition_of("PSI-25", v1.norm_date(r[0])) in ("train", "validation"):
                if isinstance(r[11], (int, float)):
                    res["PSI-25:%s" % str(r[2]).strip()] = round(r[11] / 100.0, 4)
    return res


def cmd_build(_):
    fc, cases = load_all()
    tr = [dict(i2, outcome=o2) for rec, i1, o1, i2, o2 in cases if rec["partition"] == "train"]
    vi = [i2 for rec, i1, o1, i2, o2 in cases if rec["partition"] == "validation"]
    vo = [o2 for rec, i1, o1, i2, o2 in cases if rec["partition"] == "validation"]
    v1.write_jsonl(OUT / "cases_train_v2.jsonl", tr)
    v1.write_jsonl(OUT / "cases_validation_inputs_v2.jsonl", vi)
    v1.write_jsonl(OUT / "cases_validation_outcomes_v2.jsonl", vo)
    cnt = defaultdict(Counter)
    for rec, i1, o1, i2, o2 in cases:
        p = rec["partition"]
        cnt[p]["rows"] += 1
        if o2 is not None:
            cnt[p]["sustain v1"] += o1["sustain"] is not None
            cnt[p]["sustain v2"] += o2["sustain"] is not None
            cnt[p]["quench v1"] += v1.quench_scored(o1)
            cnt[p]["quench v2"] += quench_scored(o2)
            cnt[p]["quench v2 open"] += quench_scored(o2) and o2["quench_hi_open"]
            cnt[p]["quench v2 display derived"] += quench_scored(o2) and o2["quench_display_derived"]
    print(json.dumps({k: dict(v) for k, v in cnt.items()}, indent=1))
    return 0


def _brief(o, scale):
    if o is None:
        return "-"
    if scale == "u":
        lo, hi = o.get("quench_u_lo_cm_s"), o.get("quench_u_hi_cm_s")
    else:
        lo, hi = o.get("quench_display_lo"), o.get("quench_display_hi")
    q = "" if lo is None and not o.get("quench_hi_open") else "[%s, %s]" % (lo, "open" if o.get("quench_hi_open") else hi)
    return "%s %s %s%s" % (o["sustain"], o["quench_class"], q, " n2" if o["n2_applied"] else "")


def cmd_diff(_):
    fc, cases = load_all()
    rows = []
    for rec, i1, o1, i2, o2 in cases:
        if o2 is None:
            continue
        a = (o1["sustain"], o1["quench_class"], o1["quench_u_lo_cm_s"], o1["quench_u_hi_cm_s"], v1.quench_scored(o1))
        b = (o2["sustain"], o2["quench_class"], o2["quench_u_lo_cm_s"], o2["quench_u_hi_cm_s"], quench_scored(o2))
        ign = (i1["u_cm_s_at_ignition"], i2["u_cm_s_at_ignition"])
        if a != b or abs((ign[0] or 0) - (ign[1] or 0)) > 1e-6:
            rows.append((rec["partition"], rec["case_id"], _brief(o1, "u"), _brief(o2, "u"), o2["quench_rule"] or o2["sustain_rule"],
                         ign))
    for r in rows:
        print(" | ".join(str(x) for x in r))
    print(len(rows), "cases changed")
    return 0


def cmd_spread(_):
    fc = v1.calibrations()
    import openpyxl
    ws = openpyxl.load_workbook(v1.FANCAL_25, data_only=True).worksheets[0]
    by = defaultdict(lambda: defaultdict(list))
    for c in range(2, ws.max_column + 1):
        if ws.cell(6, c).value != "Pot":
            continue
        day = v1._day_label(ws.cell(3, c + 1).value)
        block, last = 0, None
        for r in range(7, ws.max_row + 1):
            pot, fan, air = (ws.cell(r, c + k).value for k in range(3))
            if not isinstance(pot, (int, float)):
                continue
            if last is not None and pot > last:
                block += 1
            last = pot
            if isinstance(air, (int, float)) and block <= 2 and pot > 0:
                by[block][float(pot)].append((day, 1.0066 * float(air)))
    res = {}
    for b in sorted(by):
        for pot in sorted(by[b]):
            us = [u for _, u in by[b][pot]]
            if len(us) < 3:
                continue
            pos = [u for u in us if u > 0]
            res["%d restrictor(s), pot %g (display %.4g)" % (b, pot, v1.pot_to_fan(pot, fc))] = {
                "days": len(us), "min": round(min(us), 2), "median": round(statistics.median(us), 2),
                "max": round(max(us), 2), "zeros": len(us) - len(pos),
                "max_over_min_nonzero": round(max(pos) / min(pos), 2) if pos else None,
                "cv": round(statistics.pstdev(us) / statistics.mean(us), 2) if statistics.mean(us) > 0 else None}
    # the burns' own fan curves at fixed displays
    cur = defaultdict(list)
    for rec, i1, o1, i2, o2 in load_all()[1]:
        if i2 is not None and i2["flow_restrictors"] == 2:
            for d in (40.6, 42.0, 46.5, 55.0):
                cur[d].append(v1.fan_to_u(d, i2["fan_curve"]))
    res["case curves, 2 restrictors"] = {"display %.4g" % d: {"min": round(min(v), 2), "median": round(statistics.median(v), 2),
                                                             "max": round(max(v), 2)} for d, v in cur.items()}
    (OUT / "fan_spread_v2.json").write_text(json.dumps(res, indent=1) + "\n", encoding="utf-8", newline="\n")
    print(json.dumps(res, indent=1))
    return 0


# ----------------------------------------------------------------------------------------------- scores
def score(forecasts, outcomes, target):
    """As v1.score; targets 'quench' (cm/s), 'quench_display' (fan display) and 'sustain'. An open upper end
    (R8) costs nothing above the lower end."""
    per_day, refused, n = defaultdict(list), 0, 0
    for o in outcomes:
        if target in ("quench", "quench_display"):
            if not quench_scored(o):
                continue
            if target == "quench":
                lo, hi = o["quench_u_lo_cm_s"], o["quench_u_hi_cm_s"]
            else:
                lo, hi = o["quench_display_lo"], o["quench_display_hi"]
                if lo is None:
                    continue
            y = (lo, math.inf if o["quench_hi_open"] else hi)
        else:
            if o["sustain"] is None:
                continue
            y = o["sustain"]
        f = forecasts.get(o["case_id"])
        if f is None:
            refused += 1
            continue
        s = v1.crps_interval(sorted(f), *y) if target != "sustain" else (float(f) - y) ** 2
        per_day[o["day_group"]].append(s)
        n += 1
    days = {d: sum(v) / len(v) for d, v in sorted(per_day.items())}
    return {"target": target, "n_cases": n, "n_days": len(days), "refused": refused,
            "case_mean": sum(sum(v) for v in per_day.values()) / n if n else None,
            "day_mean": sum(days.values()) / len(days) if days else None, "per_day": days}


def _fit(outs, lo_key, hi_key, grid):
    qs = []
    for t in v1.QUANTILES:
        qs.append(min(grid, key=lambda q: sum(v1.interval_pinball(q, t, o[lo_key], math.inf if o["quench_hi_open"] else o[hi_key])
                                              for o in outs)))
    return [round(q, 3) for q in qs]


def cmd_baselines(_):
    tr = [json.loads(line) for line in (OUT / "cases_train_v2.jsonl").open(encoding="utf-8")]
    vo = [json.loads(line) for line in (OUT / "cases_validation_outcomes_v2.jsonl").open(encoding="utf-8")]
    to = [c["outcome"] for c in tr]
    res = {}
    for target, lo, hi, grid in (("quench", "quench_u_lo_cm_s", "quench_u_hi_cm_s", [i * 0.02 for i in range(1001)]),
                                 ("quench_display", "quench_display_lo", "quench_display_hi", [38 + i * 0.05 for i in range(1700)])):
        qs = _fit([o for o in to if quench_scored(o) and o[lo] is not None], lo, hi, grid)
        fc = {o["case_id"]: qs for o in to + vo}
        a, b = score(fc, to, target), score(fc, vo, target)
        res[target + "_constant"] = {"quantiles": qs, "train": {k: a[k] for k in ("n_cases", "n_days", "case_mean", "day_mean")},
                                     "validation": {k: b[k] for k in ("n_cases", "n_days", "case_mean", "day_mean")}}
    sus = [o for o in to if o["sustain"] is not None]
    p = sum(o["sustain"] for o in sus) / len(sus)
    fc = {o["case_id"]: p for o in to + vo}
    a, b = score(fc, to, "sustain"), score(fc, vo, "sustain")
    res["sustain_incidence"] = {"p": round(p, 4), "train": {k: a[k] for k in ("n_cases", "n_days", "case_mean", "day_mean")},
                                "validation": {k: b[k] for k in ("n_cases", "n_days", "case_mean", "day_mean")}}
    (OUT / "baselines_v2.json").write_text(json.dumps(res, indent=1) + "\n", encoding="utf-8", newline="\n")
    print(json.dumps(res, indent=1))
    return 0


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("cmd", choices=["build", "diff", "spread", "baselines"])
    a = ap.parse_args()
    return {"build": cmd_build, "diff": cmd_diff, "spread": cmd_spread, "baselines": cmd_baselines}[a.cmd](a)


if __name__ == "__main__":
    sys.exit(main())
