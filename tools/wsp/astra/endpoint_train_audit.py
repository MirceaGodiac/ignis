"""TRAIN-only endpoint audit; preserves old measurements and adds conservative diagnostics.

No model, fitting, remote access or Flame Design outcomes. Freeze the executable protocol
with --freeze, then --execute. Exact split/producer pins precede every raw-data route.
"""
import argparse
from collections import Counter
import csv
from datetime import datetime, timezone
import hashlib
import importlib.util
import io
import json
import math
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[3]
OUT = ROOT / "runs/dev/data_astra/work/endpoint_train_audit"
RAW = ROOT / "runs/dev/nasa/PSI-159"
PINS = {
    "acme_normal": ("c77bc5baee98d3e3", "eab7dd3d514d70c3a1a8fd2ea6c65d3625e43f5abce8f77e601551af4f045f02"),
    "acme_inverse": ("747032c921c8c8f2", "f849dc580170ede7b2816bf23509a8dac0f94c2b335278437cba41444acb19d1"),
}
SOURCE_PINS = {
    "tools/wsp/ext/flame_design.py": "4aa060b769ac9d47d1198bf31ce9bbffe0a972fed7d8bab8201576602d47c702",
    "tools/wsp/ext/transfer_acme.py": "70b620a974280abf0851f8ae6d98dd889ab3b9e13f089d9d4a5766801206e48d",
}
OLD_CSV = ROOT / "runs/dev/data_all/work/transfer_acme/cfig/endpoints_check.csv"
OLD_SHA = "63c69b4bfdfeddc57afda7431b519587ed8860d5692c6035dab08fce9ac42e88"
CUTOFFS = (1.0, 3.0, 5.0)
VERSION = "endpoint-train-audit/1"


def sha(data):
    return hashlib.sha256(data).hexdigest()


def pinned(path, digest):
    raw = path.read_bytes()
    if sha(raw) != digest:
        raise ValueError("pinned input changed: " + str(path))
    return raw


def train_scope():
    scope = {}
    for family, (split, digest) in PINS.items():
        data = json.loads(pinned(ROOT / f"data/splits/{family}-{split}.json", digest))["split"]
        if set(data["train"]) & (set(data["validation"]) | set(data["test"])):
            raise ValueError("overlapping split")
        for rid, group in data["run_to_group"].items():
            if group in data["train"]:
                if not re.fullmatch(r"PSI-159:" + family + r":\d{5}[A-Z]\d+(?:_\d+)?", rid):
                    raise ValueError("unexpected identity")
                scope[rid] = {"family": family, "test": rid.rsplit(":", 1)[1], "group": group}
    if len(scope) != 154:
        raise ValueError("scope changed")
    return scope


def top_identity(line, key="run_id"):
    """Read only top-level identity, never decode other scalar/compound payloads."""
    depth, i, found = 0, 0, []
    decoder = json.JSONDecoder()
    while i < len(line):
        char = line[i]
        if char == '"':
            start = i
            i += 1
            while i < len(line):
                if line[i] == "\\":
                    i += 2
                elif line[i] == '"':
                    i += 1
                    break
                else:
                    i += 1
            after = i
            while after < len(line) and line[after].isspace():
                after += 1
            if depth == 1 and after < len(line) and line[after] == ":" and line[start:i] == json.dumps(key):
                value_start = after + 1
                while value_start < len(line) and line[value_start].isspace():
                    value_start += 1
                value, _ = decoder.raw_decode(line, value_start)
                if not isinstance(value, str):
                    raise ValueError("invalid identity")
                found.append(value)
            continue
        if char in "{[":
            depth += 1
        elif char in "}]":
            depth -= 1
        i += 1
    if len(found) != 1:
        raise ValueError("missing or duplicate top-level identity")
    return found[0]


def gated_record(line, scope, decoder=json.loads):
    identity = top_identity(line)
    if identity not in scope:
        return None
    row = decoder(line)
    if row.get("run_id") != identity:
        raise ValueError("identity changed")
    return row


def train_endpoints(scope):
    out = {}
    for family in PINS:
        path = ROOT / "data/work/records" / family / "endpoints.jsonl"
        with path.open(encoding="utf-8") as stream:
            for line in stream:
                row = gated_record(line, scope)
                if row is not None:
                    rid = row["run_id"]
                    if rid in out or scope[rid]["family"] != family:
                        raise ValueError("duplicate or misrouted identity")
                    out[rid] = row
    if set(out) != set(scope):
        raise ValueError("missing train endpoints")
    return out


def old_rows(scope):
    raw = pinned(OLD_CSV, OLD_SHA).decode("utf-8")
    identities = []
    for line in raw.splitlines()[1:]:
        fields = line.split(",", 2)
        rid = "PSI-159:" + fields[1] + ":" + fields[0]
        if rid not in scope:
            raise ValueError("old artifact is not wholly TRAIN")
        identities.append(rid)
    if len(identities) != len(scope) or set(identities) != set(scope):
        raise ValueError("old artifact coverage changed")
    return dict(zip(identities, csv.DictReader(io.StringIO(raw))))


def raw_routes(rid, scope):
    if rid not in scope:
        raise ValueError("run is not authorized TRAIN")
    test = scope[rid]["test"]
    # No wildcard, guessed alias or alternate directory is permitted.
    return {key: RAW / name for key, name in {
        "data": f"PSI-159_Raw Data_{test}_data.csv",
        "tp": f"PSI-159_Raw Data_{test}_TP.txt",
        "setpoints": f"PSI-159_Raw Data_{test}_mfcSetpoints.csv",
        "script": f"PSI-159_Raw Data_as run{test}.txt",
    }.items()}


def observed_sustained(t, mask, start, hold, max_gap):
    """Bracket a threshold transition only after observed confirmation, never EOF/gaps.

    max_gap is an acquisition-resolution parameter in seconds, not a fitted threshold.
    Production diagnostics use twice the measured nominal sample interval: a larger
    interval contains at least one entirely unobserved nominal sample. The result is
    sample-supported, not a proof of continuous physical activity between samples.
    """
    if (len(t) != len(mask) or not all(math.isfinite(float(x)) for x in t)
            or not all(t[i] > t[i-1] for i in range(1, len(t)))
            or not all(math.isfinite(x) for x in (start, hold, max_gap)) or hold <= 0 or max_gap <= 0):
        raise ValueError("invalid seconds/chronology")
    first = None
    for i, (time, on) in enumerate(zip(t, mask)):
        if time < start or not on:
            first = None
            continue
        if first is None or (i and time-t[i-1] > max_gap):
            first = i
        if time-t[first] >= hold:
            # A gap/start-of-file gives only an upper bound on the transition.
            lower = (float(t[first-1]) if first > 0 and t[first]-t[first-1] <= max_gap
                     and not mask[first-1] else None)
            return {"lower_clock_s": lower, "upper_clock_s": float(t[first]),
                    "confirmed_clock_s": float(time)}
    return None


def at_risk_interval(endpoint, cutoff):
    """Shared explicit interval semantics; no fabricated half-second or midpoint."""
    if not math.isfinite(cutoff) or cutoff < 0:
        raise ValueError("invalid cutoff seconds")
    if endpoint is None or endpoint.get("censoring") not in ("event", "interval", "right"):
        return "unknown"
    lo, hi = endpoint.get("last_confirmed_active_s"), endpoint.get("first_terminal_s")
    if any(x is not None and (not math.isfinite(x) or x < 0) for x in (lo, hi)) or (lo is not None and hi is not None and hi < lo):
        raise ValueError("invalid endpoint interval")
    if hi is not None and hi <= cutoff:
        return "not_at_risk"
    if lo is not None and lo >= cutoff:
        return "at_risk"
    return "unknown"


def cause_order(pmt, flow, margin=0.5):
    """Retain old extinction margin but never force a winner inside uncertainty."""
    if pmt is None or flow is None:
        return "unresolved"
    if pmt[1] < flow[0] - margin:
        return "extinction_precedes_flow"
    if flow[1] < pmt[0]:
        return "flow_precedes_darkening"
    return "unresolved"


def stamp_account(stamps, reason):
    # Abort describes script completion, not an extinction cause. Keep both facts.
    return {"aborted": bool(reason or "Test Point Failure" in stamps),
            "reason": reason, "extinction_detected_clock_s": stamps.get("Extinction detected"),
            "mfcs_off_clock_s": stamps.get("MFCs off"),
            "cause": "unresolved"}


def clean(value):
    if isinstance(value, float) and not math.isfinite(value):
        return None
    if isinstance(value, dict):
        return {k: clean(v) for k, v in value.items()}
    if isinstance(value, (list, tuple)):
        return [clean(v) for v in value]
    return value


def write(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(clean(value), indent=2, allow_nan=False) + "\n", encoding="utf-8")


def protocol(scope):
    return {"version": VERSION, "code_sha256": sha(Path(__file__).read_bytes()),
            "split_pins": PINS, "source_pins": SOURCE_PINS, "old_csv_sha256": OLD_SHA,
            "train_ids": sorted(scope), "cutoffs_s": CUTOFFS,
            "policy": "All154; original thresholds retained; no outcome-driven parameter changes. "
                      "Observed 0.5s confirmation, no gap over 2 nominal samples; brackets, not exact events. "
                      "Abort and cause separate. Unordered flow/darkening unresolved. "
                      "Historical table intervals retained; no FlameDesign outcomes or model scoring."}


def audit_one(rid, scope, reference, old, fd):
    import numpy as np
    routes = raw_routes(rid, scope)
    receipt = {k: {"path": p.relative_to(ROOT).as_posix(), "sha256": sha(p.read_bytes())}
               for k, p in routes.items() if p.is_file()}
    test = scope[rid]["test"]
    stamps, tp_name, reason = fd.read_tp(routes["tp"]) if "tp" in receipt else ({}, None, None)
    script = fd.read_script(routes["script"]) if "script" in receipt else {}
    identity_ok = tp_name in (None, test) and script.get("testpointName") in (None, test)
    result = {"run_id": rid, **scope[rid], "sources": receipt,
              "identity": {"tp": tp_name, "script": script.get("testpointName"), "consistent": identity_ok},
              "old_receipt": old, "reference": reference, "stage_stamps_clock_s": stamps,
              "script": script, "script_account": stamp_account(stamps, reason), "classifications": []}
    if not identity_ok:
        result["classifications"].append("identity_conflict_no_measurement")
        return result
    fn = {k: p.name if k in receipt else None for k, p in routes.items()}
    measured = fd.measure(test, fn)
    result["old_method_replay"] = measured
    for key in ("status", "label_sensor", "onset_method"):
        if old.get(key, "") != measured.get(key, ""):
            raise ValueError("old categorical replay changed: " + rid + "/" + key)
    if old.get("end_s") and abs(float(old["end_s"]) - measured["end_s"]) > 0.00051:
        raise ValueError("old numeric replay changed: " + rid)
    classes = result["classifications"]
    if measured["status"] != "ok":
        classes.append("missing_exact_source_no_alias_guess")
        return result
    hdr, times, channels = fd.read_data(routes["data"])
    dt = float(np.median(np.diff(times)))
    invalid = not np.all(np.isfinite(times)) or np.any(np.diff(times) <= 0)
    result["sampling"] = {"header": hdr, "nominal_dt_s": dt, "max_gap_s": float(np.max(np.diff(times))),
                          "gaps_over_two_samples": int(np.sum(np.diff(times) > 2*dt)),
                          "nonfinite_channel_samples": sum(int(np.sum(~np.isfinite(v))) for v in channels.values())}
    if invalid or result["sampling"]["nonfinite_channel_samples"]:
        classes.append("invalid_sample_chronology_or_values")
        return result
    t0 = measured["onset_clock"]
    dark = [fd.dark_level(times, channels[k]) for k in ("PMT1", "PMT2", "PMT3")]
    excess = fd.smooth(times, sum(channels[k]-v for k, v in zip(("PMT1", "PMT2", "PMT3"), dark)), 0.2)
    bracket = observed_sustained(times, excess < fd.PMT_END_V, t0+0.5, fd.PMT_HOLD_S, 2*dt)
    result["diagnostic_v1"] = {"dark_levels_v": dark, "pmt_dark_observation": bracket,
                                "physical_extinction_qualified": False}
    if bracket:
        upper = bracket["upper_clock_s"]
        after = times > bracket["confirmed_clock_s"]
        result["diagnostic_v1"]["pmt_above_lit_after_confirmation"] = bool(np.any(after & (excess > fd.PMT_LIT_V)))
        result["diagnostic_v1"]["old_minus_confirmed_transition_s"] = measured["pmt_end_s"] - (upper-t0)
        if bracket["lower_clock_s"] is not None:
            # Centred PMT smoothing 0.2s and onset smoothing 0.1s are temporal support,
            # not newly calibrated precision. Add one sample each for quantisation.
            origin_half = 0.05 + dt if measured["onset_method"] == "radiometer" else None
            if origin_half is not None:
                bounds = [bracket["lower_clock_s"] - 0.1 - dt - t0 - origin_half,
                          upper + 0.1 + dt - t0 + origin_half]
                result["diagnostic_v1"]["pmt_dark_support_interval_s"] = bounds
                fc = measured["flow_change_s"]
                if math.isfinite(fc) and measured["flow_change_source"] == "setpoints":
                    flow = [fc-0.001-origin_half, fc+0.001+origin_half]
                    result["diagnostic_v1"]["flow_command_support_interval_s"] = flow
                    result["diagnostic_v1"]["ordering"] = cause_order(bounds, flow)
    if math.isfinite(measured["pmt_end_s"]) and bracket is None:
        classes.append("unconfirmed_terminal_candidate")
    if measured["tp_class"] == "abort" and "Extinction detected" in stamps:
        classes.append("abort_is_not_competing_cause")
    if "pmt_end_threshold_sensitive" in measured["flags"]:
        classes.append("instrument_threshold_sensitive_unresolved")
    if measured["onset_method"] != "radiometer":
        classes.append("onset_reference_not_observed_radiometrically")
    if "no_test_flow_level" in measured["flags"]:
        classes.append("constant_stage_missing_flow_reference_unresolved")
    if result["diagnostic_v1"].get("ordering") == "unresolved":
        classes.append("flow_dark_order_unresolved")
    if measured["label_sensor"] not in reference["allowed_causes"]:
        classes.append("sensor_table_cause_conflict_unresolved")
    if old.get("end_minus_table_s") and abs(float(old["end_minus_table_s"])) > 5:
        classes.append("timing_tail_requires_independent_adjudication")
    if result["sampling"]["gaps_over_two_samples"]:
        classes.append("sample_gaps")
    result["at_risk"] = {str(c): {
        "table_interval": at_risk_interval(reference, c),
        "legacy_comparison_midpoint_minus_half": float(old["table_burn_s"])-0.5 >= c,
        "legacy_sensor_point": measured["end_s"] > c,
    } for c in CUTOFFS}
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    choice = parser.add_mutually_exclusive_group(required=True)
    choice.add_argument("--freeze", action="store_true")
    choice.add_argument("--execute", action="store_true")
    args = parser.parse_args()
    scope = train_scope()
    for name, digest in SOURCE_PINS.items():
        pinned(ROOT / name, digest)
    plan = clean(protocol(scope))
    freeze_path = OUT / "protocol_frozen.json"
    if args.freeze:
        if freeze_path.exists() and json.loads(freeze_path.read_text())["protocol"] != plan:
            raise ValueError("frozen protocol exists; preserve it and explicitly version the audit")
        if not freeze_path.exists():
            write(freeze_path, {"frozen_utc": datetime.now(timezone.utc).isoformat(), "protocol": plan})
        print("Frozen all154 TRAIN audit; no raw source opened.")
        return
    if not freeze_path.exists() or json.loads(freeze_path.read_text())["protocol"] != plan:
        raise ValueError("freeze exact method before execution")
    original = old_rows(scope)
    endpoints = train_endpoints(scope)
    spec = importlib.util.spec_from_file_location("original_endpoint_measurement", ROOT / "tools/wsp/ext/flame_design.py")
    fd = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(fd)
    fd.FILES = RAW
    rows = []
    for i, rid in enumerate(sorted(scope), 1):
        row = audit_one(rid, scope, endpoints[rid], original[rid], fd)
        rows.append(row)
        write(OUT / "train" / (scope[rid]["family"] + "_" + scope[rid]["test"] + ".json"), row)
        if i % 20 == 0:
            print(f"Audited {i}/154 TRAIN tests", flush=True)
    summary = {"version": VERSION, "protocol_sha256": sha(freeze_path.read_bytes()), "train_tests": len(rows),
               "classifications": dict(Counter(c for r in rows for c in r["classifications"])),
               "old_replay": dict(Counter(r.get("old_method_replay", {}).get("status", "identity_conflict") for r in rows)),
               "timing_tails": [{"test": r["test"], "family": r["family"],
                                  "delta_s": r["old_receipt"]["end_minus_table_s"],
                                  "classifications": r["classifications"]} for r in rows
                                 if "timing_tail_requires_independent_adjudication" in r["classifications"]],
               "at_risk": {str(c): {k: dict(Counter(str(r["at_risk"][str(c)][k]) for r in rows if "at_risk" in r))
                                     for k in ("table_interval", "legacy_comparison_midpoint_minus_half", "legacy_sensor_point")}
                           for c in CUTOFFS},
               "qualification": "Diagnostics only; no accuracy, blind validation or model comparison."}
    write(OUT / "summary.json", summary)
    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
