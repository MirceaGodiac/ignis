"""Lossless CFI-G TRAIN trace inventory and optical-episode diagnostics, never labels.

Run --freeze before --execute. Raw bytes are preserved per physical row in deterministic
gzip JSONL ledgers, alongside offsets, parse errors and clock evidence. No row is sorted,
deduplicated, interpolated or silently dropped. Existing registered artifacts are read-only.
"""
import argparse
import base64
from collections import Counter
from datetime import date, datetime, timedelta, timezone
import gzip
import hashlib
import importlib.util
import json
import math
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[3]
OUT = ROOT / "runs/dev/data_astra/work/endpoint_train_audit/trace_v2"
V1 = ROOT / "runs/dev/data_astra/work/endpoint_train_audit"
AUDIT_SHA = "c4646847ceeae9ed49cb30938c01d1313cd926bb8ce4ce9f29349deeaef83680"
FD_SHA = "4aa060b769ac9d47d1198bf31ce9bbffe0a972fed7d8bab8201576602d47c702"
VERSION = "endpoint-trace-diagnostics/2"
DAY_MS = 86_400_000
SENSOR_REQUIRED = ("PMT1", "PMT2", "PMT3", "MFCA1StaticMixerFlow", "MFCA2BypassFlow", "MFCA3FuelFlow")
COMMAND_COLUMNS = ("Time", "MFCA1Setpoint(V)", "MFCA2Setpoint(V)", "MFCA3Setpoint(V)")


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def load_pinned_module(name, relative, expected):
    path = ROOT / relative
    raw = path.read_bytes()
    if digest(raw) != expected:
        raise ValueError("source pin mismatch")
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def write(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, allow_nan=False) + "\n", encoding="utf-8")


def clock_ms(token):
    match = re.fullmatch(r"(\d\d):(\d\d):(\d\d):(\d{3})", token.strip())
    if not match:
        raise ValueError("clock_not_hh_mm_ss_millisecond")
    hour, minute, second, millisecond = map(int, match.groups())
    if hour > 23 or minute > 59 or second > 59:
        raise ValueError("clock_out_of_range")
    return ((hour*60+minute)*60+second)*1000+millisecond


def metadata(raw):
    text = raw.decode("utf-8", errors="strict").strip()
    pieces = text.split(",")
    match = re.fullmatch(r"(\d{4}-\d\d-\d\d) (\d{3}) (\S+)\s*", pieces[0])
    if not match:
        raise ValueError("metadata_clock_format")
    day = date.fromisoformat(match[1])
    if day.timetuple().tm_yday != int(match[2]):
        raise ValueError("metadata_date_doy_conflict")
    result = {"date": day.isoformat(), "clock_ms": clock_ms(match[3]),
              "timezone": "not_declared_by_source", "fields": {}}
    for piece in pieces[1:]:
        key, sep, value = piece.partition(":")
        key = key.strip()
        if not sep or not key or key in result["fields"]:
            raise ValueError("metadata_duplicate_or_missing_field")
        result["fields"][key] = value.strip()
    if not result["fields"].get("Test Point"):
        raise ValueError("metadata_missing_test_identity")
    return result


def stamp_text(raw):
    """Preserve all TP lines; parse complete calendar stamps without discarding dates."""
    rows, stamps, fields = [], {}, {}
    for number, line in enumerate(raw.decode("utf-8", errors="strict").splitlines(), 1):
        rows.append({"line": number, "text": line})
        key, sep, value = line.strip().partition(":")
        if not sep:
            continue
        value = value.strip()
        if re.fullmatch(r"\d{4}-\d\d-\d\d \d\d:\d\d:\d\d", value):
            parsed = datetime.strptime(value, "%Y-%m-%d %H:%M:%S")
            if key in stamps:
                raise ValueError("duplicate_stage_stamp")
            stamps[key] = {"date": parsed.date().isoformat(),
                           "clock_ms": (parsed.hour*3600+parsed.minute*60+parsed.second)*1000,
                           "line": number, "raw": value, "precision_s": 1}
        elif key in ("Test Point", "Reason"):
            if key in fields:
                raise ValueError("duplicate_tp_identity_or_reason")
            fields[key] = {"value": value, "line": number}
    return {"rows": rows, "stamps": stamps, "fields": fields}


def start_key(meta):
    return (meta["date"], meta["clock_ms"]//1000)


def clock_link(test, meta, tp, unique_starts):
    """Link clocks/TP alias only by exact pre-test calendar start, never outcomes."""
    start = tp["stamps"].get("Script start") if tp else None
    tp_name = tp["fields"].get("Test Point", {}).get("value") if tp else None
    evidence = {"sensor_header_test": meta["fields"]["Test Point"], "tp_test": tp_name,
                "sensor_start": start_key(meta), "tp_start": start_key(start) if start else None,
                "basis": "header and TP Script start only; one-second TP precision"}
    matched = (start is not None and start_key(meta) == start_key(start)
               and unique_starts.get(start_key(meta)) == [test])
    status = "same_named_clock" if matched and tp_name == test else (
        "tp_alias_bound_by_unique_pretest_start" if matched else "unresolved_clock_link")
    evidence.update(status=status, aligned=matched,
                    timezone="unspecified acquisition/script calendar clock; not asserted UTC")
    return evidence


def parse_numeric_row(raw, columns):
    """The acquisition schema is one unquoted, numeric CSV record per physical line."""
    errors, values, raw_clock = [], None, None
    try:
        line = raw.decode("utf-8", errors="strict").rstrip("\r\n")
    except UnicodeDecodeError:
        return None, None, ["invalid_utf8"], None
    fields = line.split(",")
    if len(fields) != len(columns):
        errors.append("column_count_mismatch")
    if not fields or not fields[0]:
        errors.append("missing_clock")
    else:
        try:
            raw_clock = clock_ms(fields[0])
        except ValueError as error:
            errors.append(str(error))
    parsed = []
    for index, token in enumerate(fields[1:], 1):
        try:
            value = float(token)
            if not math.isfinite(value):
                raise ValueError("nonfinite")
            parsed.append(value)
        except ValueError:
            errors.append("nonfinite_or_invalid_number:" + str(index))
    if not errors:
        values = parsed
    return raw_clock, values, errors, len(fields)


def read_trace(path, expected_sha, test, kind, ledger, corroborated_dates=()):
    """Stream all rows to a lossless ledger and preserve order in a compact numeric view.

    Unknown header identities stop before the data body. Row bytes are kept even when
    parsing fails. Day rollover requires a date present in the already-bound TP log;
    other backward timestamps remain backward. No chronology repair is performed.
    """
    import numpy as np
    ledger.parent.mkdir(parents=True, exist_ok=True)
    summary = {"path": str(path.relative_to(ROOT)) if path.is_relative_to(ROOT) else str(path),
               "expected_sha256": expected_sha, "kind": kind, "rows": 0, "data_rows": 0,
               "valid_rows": 0, "parse_failures": [], "clock_anomalies": [],
               "calendar_rollovers": [], "missing_intervals": [], "schema_errors": []}
    times, arrays, line_numbers, valid = [], [], [], []
    source_hash = hashlib.sha256()
    offset, previous_raw, previous_clock, day_offset, previous_day = 0, None, None, 0, None
    header, columns = None, []
    with path.open("rb") as stream, ledger.open("wb") as destination:
        with gzip.GzipFile(filename="", mode="wb", fileobj=destination, mtime=0) as journal:
            for number, raw in enumerate(stream, 1):
                record = {"line": number, "byte_offset": offset, "byte_length": len(raw),
                          "raw_base64": base64.b64encode(raw).decode("ascii")}
                source_hash.update(raw)
                offset += len(raw)
                summary["rows"] += 1
                if number == 1:
                    try:
                        header = metadata(raw)
                    except (ValueError, UnicodeError) as error:
                        record.update(kind="metadata", errors=[str(error)])
                        journal.write((json.dumps(record)+"\n").encode())
                        summary.update(status="refused_metadata", header_error=str(error))
                        return summary, None
                    if header["fields"]["Test Point"] != test:
                        record.update(kind="metadata", errors=["header_identity_mismatch"])
                        journal.write((json.dumps(record)+"\n").encode())
                        summary.update(status="refused_identity", header=header)
                        return summary, None
                    record.update(kind="metadata", parsed=header)
                    previous_day = date.fromisoformat(header["date"])
                    # The acquisition can begin after midnight even when the header
                    # describes a script started the previous day.
                    previous_raw = header["clock_ms"]
                elif number == 2:
                    try:
                        columns = [c.strip() for c in raw.decode("utf-8", errors="strict").rstrip("\r\n").split(",")]
                    except UnicodeError:
                        summary["schema_errors"].append("invalid_utf8_header")
                    if not columns or columns[0] != "Time" or len(set(columns)) != len(columns):
                        summary["schema_errors"].append("missing_time_or_duplicate_columns")
                    if kind == "sensor" and any(k not in columns for k in SENSOR_REQUIRED):
                        summary["schema_errors"].append("required_sensor_columns_missing")
                    if kind == "commands" and tuple(columns) != COMMAND_COLUMNS:
                        summary["schema_errors"].append("unknown_command_schema")
                    record.update(kind="schema", columns=columns, errors=list(summary["schema_errors"]))
                else:
                    summary["data_rows"] += 1
                    raw_clock, values, errors, count = parse_numeric_row(raw, columns)
                    if summary["schema_errors"]:
                        errors.append("schema_not_accepted")
                    clock = None
                    if raw_clock is not None:
                        if previous_raw is not None and raw_clock-previous_raw < -DAY_MS//2:
                            following_day = previous_day+timedelta(days=1)
                            if following_day.isoformat() in corroborated_dates:
                                day_offset += DAY_MS
                                previous_day = following_day
                                summary["calendar_rollovers"].append({"line": number, "date": str(following_day),
                                    "basis": "negative half-day wrap and date in bound TP log"})
                            else:
                                summary["clock_anomalies"].append({"line": number, "kind": "uncorroborated_day_wrap"})
                                errors.append("uncorroborated_day_wrap")
                        clock = raw_clock+day_offset
                        if previous_clock is not None and clock <= previous_clock:
                            summary["clock_anomalies"].append({"line": number,
                                "kind": "duplicate_clock" if clock == previous_clock else "backward_clock",
                                "previous_clock_ms": previous_clock, "clock_ms": clock})
                        previous_raw, previous_clock = raw_clock, clock
                    record.update(kind="sample", source_clock_ms=raw_clock, calendar_relative_ms=clock,
                                  columns_seen=count, errors=errors)
                    if errors:
                        summary["parse_failures"].append({"line": number, "errors": errors})
                    else:
                        summary["valid_rows"] += 1
                    times.append(clock if clock is not None else math.nan)
                    arrays.append(values if values is not None and not errors else [math.nan]*max(0,len(columns)-1))
                    line_numbers.append(number)
                    valid.append(not errors)
                journal.write((json.dumps(record, separators=(",", ":"))+"\n").encode())
    if source_hash.hexdigest() != expected_sha:
        raise ValueError("raw source hash changed: " + path.name)
    summary.update(status="recorded", sha256=source_hash.hexdigest(), bytes=offset, header=header, columns=columns,
                   ledger=str(ledger.relative_to(OUT)) if ledger.is_relative_to(OUT) else str(ledger))
    if len(columns) < 2 or not times:
        summary["status"] = "refused_empty_or_unknown_schema"
        return summary, None
    times = np.array(times, float)/1000.0
    valid = np.array(valid, bool)
    delta = np.diff(times)
    positive = delta[np.isfinite(delta) & (delta > 0)]
    nominal = float(np.median(positive)) if len(positive) else None
    summary["nominal_dt_s"] = nominal
    # Commands are irregular events, not sampled at a fixed cadence.
    if kind == "sensor" and nominal is not None:
        for i in np.where(delta > 2*nominal)[0]:
            summary["missing_intervals"].append({"before_line": line_numbers[i], "after_line": line_numbers[i+1],
                "lower_clock_s": float(times[i]), "upper_clock_s": float(times[i+1]),
                "elapsed_s": float(delta[i]), "physical_state": "unobserved"})
    return summary, {"times": times, "values": np.array(arrays, float), "valid": valid,
                     "lines": line_numbers, "columns": columns[1:]}


def segments(trace, max_gap):
    """Return contiguous row intervals, never join across invalid or unordered samples."""
    start, out = None, []
    for i, (t, valid) in enumerate(zip(trace["times"], trace["valid"])):
        continuous = (i > 0 and trace["valid"][i-1] and 0 < t-trace["times"][i-1] <= max_gap)
        if not valid or not math.isfinite(t):
            if start is not None:
                out.append((start, i))
            start = None
        elif start is None:
            start = i
        elif not continuous:
            out.append((start, i))
            start = i
    if start is not None:
        out.append((start, len(trace["times"])))
    return out


def intervals(times, mask, lines, hold, segment, kind):
    """All observed threshold spans; confirmation needs actual samples spanning hold."""
    out, begin = [], None
    for i in range(len(times)+1):
        on = i < len(times) and bool(mask[i])
        if on and begin is None:
            begin = i
        if begin is not None and not on:
            end = i-1
            duration = float(times[end]-times[begin])
            confirmation = next((j for j in range(begin,end+1) if times[j]-times[begin] >= hold), None)
            out.append({"kind": kind, "segment": segment, "first_line": int(lines[begin]), "last_line": int(lines[end]),
                        "first_clock_s": float(times[begin]), "last_clock_s": float(times[end]),
                        "transition_lower_clock_s": float(times[begin-1]) if begin > 0 else None,
                        "transition_upper_clock_s": float(times[begin]), "observed_span_s": duration,
                        "confirmed": confirmation is not None,
                        "confirmation_clock_s": float(times[confirmation]) if confirmation is not None else None,
                        "left_censored": begin == 0, "right_censored": end == len(times)-1,
                        "confirmation_right_censored": end == len(times)-1 and confirmation is None})
            begin = None
    return out


def optical_diagnostics(trace, nominal, onset, fd):
    """Frozen old PMT thresholds/smoothers; sequence diagnostics, not extinction labels."""
    import numpy as np
    spans = segments(trace, 2*nominal)
    pmts = [trace["columns"].index(c) for c in ("PMT1", "PMT2", "PMT3")]
    complete = trace["valid"]
    # Preserve the old global dark estimator, including its scientific limitation.
    # It is refused when chronology is not strictly increasing; segmented episodes
    # alone cannot repair the estimator's fixed-time block interpretation.
    if not complete.all() or np.any(np.diff(trace["times"]) <= 0):
        return {"status": "refused_global_dark_estimator_input", "segments": spans,
                "reason": "old block estimator requires unbroken valid ordered samples; no sorting/repair",
                "qualified_physical_extinction": False}
    dark = [fd.dark_level(trace["times"],trace["values"][:,index]) for index in pmts]
    episodes = []
    for segment, (lo, hi) in enumerate(spans):
        t = trace["times"][lo:hi]
        if len(t) < 2:
            continue
        signal = sum(trace["values"][lo:hi,index]-baseline for index,baseline in zip(pmts,dark))
        # Same centred mean, segmented so no missing span is used as evidence.
        if len(t) < max(1,int(round(.2/nominal))):
            continue
        signal = fd.smooth(t,signal,.2)
        eligible = t >= onset+.5
        if not eligible.any():
            continue
        selected = np.where(eligible)[0]
        first = int(selected[0])
        t, signal = t[first:], signal[first:]
        lines = trace["lines"][lo+first:hi]
        episodes.extend(intervals(t, signal < fd.PMT_END_V, lines, fd.PMT_HOLD_S, segment,"dark"))
        episodes.extend(intervals(t, signal > fd.PMT_LIT_V, lines, .2, segment,"lit"))
    episodes.sort(key=lambda e:(e["first_line"],e["kind"]))  # presentation only; no raw samples reordered
    observed_dark = [e for e in episodes if e["kind"] == "dark"]
    confirmed_dark = [e for e in observed_dark if e["confirmed"]]
    relit = []
    for e in episodes:
        if e["kind"] == "lit" and e["confirmed"] and any(
                d["segment"] == e["segment"] and d["last_line"] < e["first_line"] for d in confirmed_dark):
            relit.append(e)
    final = next((e for e in reversed(observed_dark) if e["last_line"] == trace["lines"][-1]),None)
    terminal = final if final and final["confirmed"] else None
    return {"status": "diagnostic_only", "qualified_physical_extinction": False,
            "dark_baselines_v": dark, "onset_clock_s": onset, "onset_source": "pinned unchanged v1 radiometer receipt",
            "segments": spans, "first_observed_dark_interval": observed_dark[0] if observed_dark else None,
            "first_confirmed_dark_interval": confirmed_dark[0] if confirmed_dark else None,
            "confirmed_later_relighting": relit, "last_observed_loss_candidate": terminal,
            "unconfirmed_eof_dark_span": final if final and not final["confirmed"] else None,
            "record_end_censoring": {"at_clock_s": float(trace["times"][-1]),
                "state_beyond_record": "unobserved", "permanent_extinction": "not_identified"},
            "all_threshold_intervals": episodes}


def command_actions(trace):
    actions = []
    previous = None
    for i, valid in enumerate(trace["valid"]):
        if not valid:
            previous = None
            continue
        values = [float(x) for x in trace["values"][i]]
        if previous is not None and values != previous:
            actions.append({"line": trace["lines"][i], "clock_s": float(trace["times"][i]),
                            "before_setpoint_v": previous, "after_setpoint_v": values,
                            "all_zero_command": all(v == 0 for v in values),
                            "actor": "human_vs_automatic_not_identified"})
        previous = values
    return actions


def diagnostic_order(dark, action, source_clock_aligned):
    """Only ordering with temporal support is identified; cause remains unresolved."""
    if not dark or not action or not source_clock_aligned or dark["transition_lower_clock_s"] is None:
        return {"temporal_order": "unresolved", "physical_cause": "unresolved"}
    lower, upper = dark["transition_lower_clock_s"]-.1, dark["transition_upper_clock_s"]+.1
    before, after = action["clock_s"]-.001, action["clock_s"]+.001
    order = "dark_before_command" if upper < before-.5 else (
        "command_before_dark" if after < lower else "unresolved")
    return {"temporal_order": order, "physical_cause": "unresolved",
            "dark_processing_support_s": [lower,upper], "command_clock_precision_s": [before,after]}


def v1_rows(scope, audit):
    out = {}
    for rid, identity in scope.items():
        path = V1/"train"/(identity["family"]+"_"+identity["test"]+".json")
        raw = path.read_bytes()
        row = json.loads(raw)
        if row["run_id"] != rid:
            raise ValueError("TRAIN receipt identity mismatch")
        routes = audit.raw_routes(rid,scope)
        for key, receipt in row["sources"].items():
            if receipt["path"] != routes[key].relative_to(ROOT).as_posix():
                raise ValueError("source route differs from exact TRAIN identity")
        out[rid] = {"receipt": row, "sha256": digest(raw)}
    return out


def protocol(scope, rows):
    return {"version":VERSION,"code_sha256":digest(Path(__file__).read_bytes()),
            "audit_source_sha256":AUDIT_SHA,"original_measurement_sha256":FD_SHA,
            "train_ids":sorted(scope),"v1_receipts":{rid:v["sha256"] for rid,v in rows.items()},
            "raw_sources":{rid:v["receipt"]["sources"] for rid,v in rows.items()},
            "method":"Preserve every physical row, dates, byte offsets and raw bytes; no repairs. "
                     "Header identity gate; pretest unique Script start links TP clock and TP alias only. "
                     "As-run identity mismatch refuses script. Original radiometer onset is provenance-bound. "
                     "PMT dark0.05V/hold0.5s; lit0.05V/hold0.2s; original0.2s mean and global dark estimator. "
                     "Gap above2 median sample periods splits support; invalid/nonincreasing clock refuses estimator. "
                     "First dark, later confirmed lit, last observed dark and actions remain distinct; EOF censored. "
                     "No physical labels, scoring, model changes or extra exposure."}


def run(scope, rows, audit, fd):
    metas, unique = {}, {}
    # Metadata-only inventory first: aliases cannot be selected by measured outcome similarity.
    for rid, identity in scope.items():
        row = rows[rid]["receipt"]
        if "data" not in row["sources"]:
            continue
        path = audit.raw_routes(rid,scope)["data"]
        with path.open("rb") as stream:
            meta = metadata(stream.readline())
        if meta["fields"]["Test Point"] != identity["test"]:
            continue
        metas[rid] = meta
        unique.setdefault(start_key(meta),[]).append(identity["test"])
    results = []
    for number, (rid, identity) in enumerate(sorted(scope.items()),1):
        prior = rows[rid]["receipt"]
        routes = audit.raw_routes(rid,scope)
        result = {"run_id":rid,**identity,"version":VERSION,"prior_receipt_sha256":rows[rid]["sha256"],
                  "qualified_physical_extinction":False,"clock_domain":"acquisition calendar; timezone unspecified"}
        result_dir = OUT/"train"/(identity["family"]+"_"+identity["test"])
        result_dir.mkdir(parents=True,exist_ok=True)
        if rid not in metas:
            result["status"]="missing_or_mismatched_sensor_header"
            write(result_dir/"inventory.json",result)
            results.append(result)
            continue
        tp = None
        if "tp" in prior["sources"]:
            tp_raw = audit.pinned(routes["tp"],prior["sources"]["tp"]["sha256"])
            tp = stamp_text(tp_raw)
        link = clock_link(identity["test"],metas[rid],tp,unique)
        result["tp_clock_link"] = link
        result["tp_stamps"] = tp["stamps"] if tp else {}
        dates = {s["date"] for s in tp["stamps"].values()} if tp and link["aligned"] else set()
        script_name = prior["identity"]["script"]
        result["script_binding"] = {"source_testpointName":script_name,
                                    "status":"exact_identity" if script_name==identity["test"] else "unresolved_refused",
                                    "reason":"a matching filename or similar conditions cannot override testpointName"}
        sensor, trace = read_trace(routes["data"],prior["sources"]["data"]["sha256"],identity["test"],
                                   "sensor",result_dir/"sensor_rows.jsonl.gz",dates)
        result["sensor"] = sensor
        actions, command_summary = [], None
        if "setpoints" in prior["sources"]:
            command_summary, commands = read_trace(routes["setpoints"],prior["sources"]["setpoints"]["sha256"],
                identity["test"],"commands",result_dir/"command_rows.jsonl.gz",dates)
            if commands is not None and not command_summary["schema_errors"]:
                actions = command_actions(commands)
        result["commands"] = command_summary
        result["operator_actions"] = actions
        old = prior.get("old_method_replay",{})
        if trace is not None and not sensor["schema_errors"] and sensor["nominal_dt_s"] is not None and old.get("onset_method")=="radiometer":
            result["optical"] = optical_diagnostics(trace,sensor["nominal_dt_s"],old["onset_clock"],fd)
        else:
            result["optical"] = {"status":"unresolved_no_qualified_onset_or_schema", "qualified_physical_extinction":False}
        optical = result["optical"]
        # Command observations stay plural. The old selected command is only an audit comparison.
        selected = None
        if old.get("flow_change_source")=="setpoints" and old.get("flow_change_s") is not None:
            selected_clock = old["onset_clock"]+old["flow_change_s"]
            selected = next((a for a in actions if abs(a["clock_s"]-selected_clock)<1e-6),None)
        command_header = (command_summary or {}).get("header")
        sensor_header = sensor.get("header")
        aligned = bool(command_header and sensor_header
                       and command_header["date"] == sensor_header["date"]
                       and abs(command_header["clock_ms"]-sensor_header["clock_ms"]) <= 1)
        result["ordering_of_old_selected_command"] = diagnostic_order(optical.get("first_confirmed_dark_interval"),selected,aligned)
        result["status"]="inventoried"
        write(result_dir/"inventory.json",result)
        results.append(result)
        if number%10==0:
            print(f"Lossless TRAIN inventory {number}/154",flush=True)
    summary={"version":VERSION,"train_identities":len(results),
             "sensor_files":sum("sensor" in r for r in results),
             "sensor_rows":sum(r.get("sensor",{}).get("data_rows",0) for r in results),
             "sensor_parse_failures":sum(len(r.get("sensor",{}).get("parse_failures",[])) for r in results),
             "sensor_clock_anomalies":sum(len(r.get("sensor",{}).get("clock_anomalies",[])) for r in results),
             "sensor_missing_intervals":sum(len(r.get("sensor",{}).get("missing_intervals",[])) for r in results),
             "command_files":sum(r.get("commands") is not None for r in results),
             "command_parse_failures":sum(len((r.get("commands")or{}).get("parse_failures",[])) for r in results),
             "optical_status":dict(Counter(r.get("optical",{}).get("status","no_sensor") for r in results)),
             "tp_clock_links":dict(Counter(r.get("tp_clock_link",{}).get("status","no_sensor") for r in results)),
             "confirmed_relighting_tests":[r["test"] for r in results if r.get("optical",{}).get("confirmed_later_relighting")],
             "unconfirmed_eof_tests":[r["test"] for r in results if r.get("optical",{}).get("unconfirmed_eof_dark_span")],
             "last_observed_loss_candidates":sum(r.get("optical",{}).get("last_observed_loss_candidate") is not None for r in results),
             "qualification":"Measurement diagnostics only, no physical endpoint labels or accuracy claim",
             "protocol_sha256":digest((OUT/"protocol_frozen.json").read_bytes())}
    write(OUT/"summary.json",summary)
    print(json.dumps(summary,indent=2))


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    mode=parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--freeze",action="store_true")
    mode.add_argument("--execute",action="store_true")
    args=parser.parse_args()
    audit=load_pinned_module("endpoint_audit_frozen_v1","tools/wsp/astra/endpoint_train_audit.py",AUDIT_SHA)
    scope=audit.train_scope()
    rows=v1_rows(scope,audit)
    plan=protocol(scope,rows)
    path=OUT/"protocol_frozen.json"
    if args.freeze:
        if path.exists() and json.loads(path.read_text())["protocol"]!=plan:
            raise ValueError("existing freeze changed; explicitly version instead of overwrite")
        if not path.exists():
            write(path,{"frozen_utc":datetime.now(timezone.utc).isoformat(),"protocol":plan})
        print("Trace v2 frozen for all154 TRAIN IDs; no raw replay.")
        return
    if not path.exists() or json.loads(path.read_text())["protocol"]!=plan:
        raise ValueError("freeze exact v2 protocol before replay")
    fd=load_pinned_module("original_measurement","tools/wsp/ext/flame_design.py",FD_SHA)
    run(scope,rows,audit,fd)


if __name__=="__main__":
    main()
