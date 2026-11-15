#!/usr/bin/env python3
"""Planned flow stop of an ACME burner test (PSI-159 CFI-G, PSI-10 Flame Design), from the script
parameter file alone.

The script parameter file (`<inv>_Raw Data_as run<test>.txt`, or `<test>.txt` on the first Flame Design
days) is written before ignition: key:value lines of script, igniter, fiber and camera settings. This tool
reads nothing else: no TP stage stamps, no command log, no sensor channel, no table. It replaces the bare
stage sum of `acme_script.py` (which it leaves untouched) by the time the script itself spends between its
"Ignition and retraction" step and the end of the last constant-flow stage, and places that on the
forecast clock (seconds since the flame onset).

Two script builds exist (`generation`):
  v1  Flame Design normal flames, 2019: one constant-flow stage `constantFlowTimeSec`, igniter brought to
      `igniterIgnitionPosition` during the preheat step that precedes the ignition step.
  v2  CFI-G (2021) and Flame Design inverse flames (2021-22): up to three constant-flow stages
      (`constantFlowTime1/2/3Sec` with ramps between), igniter driven from `igniterStartPosition` to
      `igniterEndPosition` at `igniterVelocityCountsPerSec` within the ignition step.

Rule (all times in s; every term is a parameter of the file or a stated script behaviour):

  S  stage sum = stabilizationTime1Sec + stabilizationTime2Sec + ignitionToTestRampTimeSec
                 + constantFlowTimeSec                                   (v1)
                 + constantFlowTime1Sec + testFlowRamp1To2TimeSec + constantFlowTime2Sec
                 + testFlowRamp2To3TimeSec + constantFlowTime3Sec       (v2)
                 (= the old `programmed_stop`)
  D  ignition step: stabilization #1 starts igniterDelayBeforeRetractMs / 1000 after the ignition step
     starts (the igniter is retracted then; both builds)
  F  fiber step before the constant-flow stage:
       v2: |fiberTestPosition - fiberIgnitionPosition| / fiberVelocity (the TFP fiber array is driven
           from its ignition to its test position at that velocity; 0 when the positions are equal)
       v1: FIBER_WAIT_V1_S = 8 s when fiberMoveDuringTest is true and fiberIgnitionPosition differs from
           fiberTestPosition, else 0 (v3, 15 Nov 2026: with equal positions there is no traverse; found on
           the spent fresh-batch tests 19123L4-L7, whose flow changed about 6 s before the v2 stop; the
           only tests it changes are 19123L3-L8, all on a spent test day; no train or validation test). This is a behaviour of the
           2019 build read from its own stage stamps (constant-flow start minus end of stabilization,
           8 s in 34 of 40 train tests, independent of distance and velocity); it is NOT a parameter.
  G  igniter travel inside the ignition step, after which the flame lights:
       v2: (igniterEndPosition - igniterStartPosition) / igniterVelocityCountsPerSec
       v1: 0 (the igniter is already in position when the ignition step starts)

  planned stop, from the start of the ignition step      = D + S + F
  planned stop, forecast clock (from the flame onset)    = D + S + F - G

The forecast clock places the onset at the end of the igniter travel. The physical ignition delay
after that point is not in the file and is a remainder (docs/wsp/feedback/acme_planned_stop_v2.md).
Ramps are taken at their nominal length; the script's ramp loop runs about 5 % slow (remainder).

Also reported: burner_limit (burnerTemperatureLimitC, burnerTemperatureLimitDurationSecs when the file has them).
Also reported:
  planned_first_flow_change_s  end of constant stage 1 on the forecast clock (v2 with stage 2 or 3: the
                               first scripted departure from test flow 1); equals the stop otherwise
  planned_flow_zero_s          v1 with useRampFlowToZero: end of the ramp to zero, capped by
                               flowTimeoutSec counted from the ignition step; else equals the stop

    python tools/wsp/ext/acme_script2.py PSI-10 19150A3 21349M1
    python tools/wsp/ext/acme_script2.py --file "<path to as-run file>"
"""
import argparse
import hashlib
import json
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
RAW = REPO / "runs/dev/data_all/raw"
METHOD_ID, METHOD_VERSION = "wsp.ext.acme.cond.programmed_stop", "3"
FIBER_WAIT_V1_S = 8.0   # stated script behaviour of the 2019 build (see module docstring)

STAGES_V1 = ["stabilizationTime1Sec", "stabilizationTime2Sec", "ignitionToTestRampTimeSec", "constantFlowTimeSec"]
STAGES_V2 = ["stabilizationTime1Sec", "stabilizationTime2Sec", "ignitionToTestRampTimeSec", "constantFlowTime1Sec",
             "testFlowRamp1To2TimeSec", "constantFlowTime2Sec", "testFlowRamp2To3TimeSec", "constantFlowTime3Sec"]


def script_path(inv, test, raw=RAW):
    d = raw / inv / "Raw Data"
    for name in (f"{inv}_Raw Data_as run{test}.txt", f"{inv}_Raw Data_{test}.txt"):
        if (d / name).exists():
            return d / name
    return None


def parse(path):
    out = {}
    for line in Path(path).read_text(encoding="utf-8", errors="replace").replace("\r", "\n").split("\n"):
        key, sep, value = line.strip().rstrip(",").partition(":")
        if sep:
            out[key.strip()] = value.strip()
    return out


def _num(s, k):
    v = s.get(k)
    if v is None:
        return None
    try:
        return float(v)
    except ValueError:
        return None


def planned(s):
    """The rule on a parsed parameter file. Returns a dict; values are None when the branch cannot be computed."""
    r = dict(generation=None, branch=None, flags=[], terms={}, stage_sum_s=None, planned_stop_from_ignition_step_s=None,
             planned_stop_s=None, planned_first_flow_change_s=None, planned_flow_zero_s=None)
    if "constantFlowTimeSec" in s:
        gen, stages = "v1", STAGES_V1
    elif "constantFlowTime1Sec" in s:
        gen, stages = "v2", STAGES_V2
    else:
        r["branch"] = "no_constant_flow_stage"
        return r
    r["generation"] = gen
    vals = {k: _num(s, k) for k in stages}
    bad = [k for k in ("constantFlowTimeSec" if gen == "v1" else "constantFlowTime1Sec",) if vals[k] is None]
    if bad or any(s.get(k) is not None and vals[k] is None for k in stages):
        r["branch"] = "unreadable_stage_value"
        return r
    S = sum(v or 0.0 for v in vals.values())
    r["stage_sum_s"] = S
    missing = []
    D = _num(s, "igniterDelayBeforeRetractMs")
    if D is None:
        missing.append("igniterDelayBeforeRetractMs")
    else:
        D /= 1000.0
    if gen == "v2":
        a, b, v = (_num(s, k) for k in ("igniterStartPosition", "igniterEndPosition", "igniterVelocityCountsPerSec"))
        if None in (a, b, v) or v <= 0:
            missing.append("igniter start/end position and velocity")
            G = None
        else:
            G = abs(b - a) / v
        fi, ft, fv = (_num(s, k) for k in ("fiberIgnitionPosition", "fiberTestPosition", "fiberVelocity"))
        if None in (fi, ft):
            missing.append("fiberIgnitionPosition/fiberTestPosition")
            F = None
        elif fi == ft:
            F, fb = 0.0, "no_fiber_traverse"
        elif fv is None or fv <= 0:
            missing.append("fiberVelocity")
            F = None
        else:
            F, fb = abs(ft - fi) / fv, "fiber_traverse"
        if s.get("fiberMoveDuringTest") == "true":
            r["flags"].append("fiber_moves_during_test_unverified_in_v2")
    else:
        G = 0.0
        fm = s.get("fiberMoveDuringTest")
        if fm is None:
            missing.append("fiberMoveDuringTest")
            F = None
        elif fm == "true" and _num(s, "fiberIgnitionPosition") is not None                 and _num(s, "fiberIgnitionPosition") == _num(s, "fiberTestPosition"):
            # v3: the wait is the traverse from the ignition to the test position (as F of v2); with equal
            # positions there is no traverse, so no wait. Every train test with the 8 s gap has unequal positions.
            F, fb = 0.0, "fiber_move_no_traverse"
        elif fm == "true":
            F, fb = FIBER_WAIT_V1_S, "fiber_move_wait"
        else:
            F, fb = 0.0, "no_fiber_move"
            if _num(s, "fiberIgnitionPosition") not in (None, _num(s, "fiberTestPosition")):
                r["flags"].append("fiber_positions_differ_without_move_unverified_in_v1")
    r["terms"] = dict(stage_sum_s=S, igniter_delay_before_retract_s=D, fiber_step_s=F, igniter_travel_s=G)
    if missing:
        r["branch"] = f"{gen}|incomplete_file"
        r["flags"].append("missing:" + ",".join(missing))
        return r
    r["branch"] = f"{gen}|{fb}"
    r["planned_stop_from_ignition_step_s"] = D + S + F
    r["planned_stop_s"] = D + S + F - G
    first = r["planned_stop_s"]
    if gen == "v2" and ((vals["constantFlowTime2Sec"] or 0) > 0 or (vals["constantFlowTime3Sec"] or 0) > 0):
        first = D + F - G + sum(vals[k] or 0.0 for k in STAGES_V2[:4])
        r["flags"].append("multi_stage_flow")
    r["planned_first_flow_change_s"] = first
    zero = r["planned_stop_s"]
    if gen == "v1" and s.get("useRampFlowToZero") == "true":
        ramp = _num(s, "rampFlowToZeroSec") or 0.0
        zero = r["planned_stop_s"] + ramp
        to = _num(s, "flowTimeoutSec")
        if to is not None and to - G < zero:
            zero = to - G
            r["flags"].append("flow_timeout_cuts_ramp_to_zero")
        r["flags"].append("ramp_flow_to_zero")
    r["planned_flow_zero_s"] = zero
    to = _num(s, "flowTimeoutSec")
    if to is not None and to < D + S + F:
        r["flags"].append("flow_timeout_before_planned_stop")
    return r


def burner_limit(s):
    """Burner temperature limit set in the parameter file: burnerTemperatureLimitC (deg C) and
    burnerTemperatureLimitDurationSecs (how long the burner thermocouple must stay above it before the
    script stops the flow). Present in Flame Design files from day 19156 on (except the 2021 normal day
    21301) and in s-Flame. The CFI-G files never have it: there the limit is a constant of the CFI-G flight
    script, printed per test in the session log ("temperature limit of N C is ... mV"), see
    burner_limit_from_log; its dwell is not printed anywhere."""
    c, d = _num(s, "burnerTemperatureLimitC"), _num(s, "burnerTemperatureLimitDurationSecs")
    if c is None and d is None:
        return dict(burner_limit_c=None, burner_limit_duration_s=None, status="not_in_parameter_file")
    return dict(burner_limit_c=c, burner_limit_duration_s=d,
                status="parameter_file" if c is not None and d is not None else "parameter_file_partial")


def burner_limit_from_log(inv, test, raw=RAW):
    """The limit the flight script prints when the test point starts, read from the session log(s).
    Only the 'test point <test> STARTED' line and the limit line that follows it are matched; reading stops
    there, so nothing the log records after the start of the test is read. Not a parameter-file value."""
    import re
    for p in sorted((raw / inv / "Raw Data").glob("*ACME.log*")):
        started = False
        with p.open(encoding="utf-8", errors="replace") as fh:
            for line in fh:
                if not started:
                    started = f"test point {test} STARTED" in line
                    continue
                m = re.search(r"temperature limit of ([0-9.]+) C is", line)
                if m:
                    return dict(burner_limit_c=float(m.group(1)), log=p.name, status="session_log_at_test_start")
                if "STARTED" in line or "Received command: SystemOpenTestPoint" in line:
                    break
    return dict(burner_limit_c=None, status="no_log_line")


def for_file(path):
    path = Path(path)
    out = dict(file=path.name, sha256=hashlib.sha256(path.read_bytes()).hexdigest(),
               method_id=METHOD_ID, method_version=METHOD_VERSION)
    s = parse(path)
    out.update(planned(s))
    out["burner_limit"] = burner_limit(s)
    return out


def for_test(inv, test, raw=RAW):
    p = script_path(inv, test, raw)
    if p is None:
        return dict(investigation=inv, test=test, branch="no_script_file", planned_stop_s=None,
                    burner_limit=dict(burner_limit_c=None, burner_limit_duration_s=None, status="no_script_file"))
    return dict(investigation=inv, test=test, **for_file(p))


def burner_limit_for_test(inv, test, raw=RAW):
    p = script_path(inv, test, raw)
    if p is None:
        return dict(burner_limit_c=None, burner_limit_duration_s=None, status="no_script_file")
    return burner_limit(parse(p))


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("investigation", nargs="?", choices=["PSI-10", "PSI-159", "PSI-23"])
    ap.add_argument("tests", nargs="*")
    ap.add_argument("--file", help="a script parameter file")
    ap.add_argument("--session-log", action="store_true",
                    help="when the file has no burner limit, also read the limit line printed at test start in the session log")
    a = ap.parse_args()
    if a.file:
        print(json.dumps(for_file(a.file), indent=1))
    elif a.investigation and a.tests:
        out = [for_test(a.investigation, t) for t in a.tests]
        if a.session_log:
            for o in out:
                if o["burner_limit"]["burner_limit_c"] is None:
                    o["burner_limit_session_log"] = burner_limit_from_log(a.investigation, o["test"])
        print(json.dumps(out, indent=1))
    else:
        ap.error("give an investigation and test names, or --file")
    return 0


if __name__ == "__main__":
    sys.exit(main())
