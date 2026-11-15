"""Protected, script-only NASA cross-checks. Dry run; --freeze --execute pins sources.

No source payload or discrepancy values are printed. No model input is replaced.
"""
from __future__ import annotations

import argparse
import csv
from collections import Counter
from datetime import datetime
from decimal import Decimal, InvalidOperation
import hashlib
import io
import json
from pathlib import Path
import re

import endpoint_train_audit as audit
import endpoint_trace_v2 as trace

ROOT = Path(__file__).resolve().parents[3]
BASE = ROOT / "runs/dev/data_astra/work"
PRIVATE = BASE / "discrepancies"
LABELS = BASE / "labels"
VERSION = "astra.discrepancies/1"
INVESTIGATIONS = (10,20,21,22,23,25,26,39,47,68,69,98,99,100,101,102,106,107,159)
SAFE = BASE / "author_conditions/84fa8febc84acc04768c6f9ce31896b08b8a6eb7acbce3978547b6fdb1f026f8/conditions.jsonl"
SAFE_SHA = "b001d711ab32109a3b1563436fc80174baba5a35f48056bac0d2704a3f59a9f9"
DOC = ROOT / "runs/dev/data_astra/corpus/incoming/PSI-23/44a988b935c2bfd186c95a33ae163a7bc8fe66541f6a29bddcb84077182f8fd6.pdf"
DOC_SHA = "684d321a26e81472f3c9ca7d320f531217b06cf863075a60ebb004a931bba06b"
TABLES = {
    "acme_inverse": ("PSI-159_Experimental table_CFI-G_Inverse Flames.csv", "b1229c4137a5c5603dd2cb07108d36544f198291ae700a94ec680f377cff8432", "d4d44f8b9896936728748e34d1b6ad0812032a92fa5289599b708c138e3a9b39"),
    "acme_normal": ("PSI-159_Experimental table_CFI-G_Normal Flames.csv", "4db6151fe05e08ba0f5d55f21b22cfd0b2fdbced8e759d6072a15d9a73ee9e1b", "d95a621c3fbd27d204dd32ed29fa17d68f2b779d6a523cde8479ed7ebaa5251a"),
}
QUESTIONS = [
    {"id":"identity", "question":"Do exact routed filenames, sensor/command header Test Point, and as-run testpointName agree? TP aliases require unique matching pretest calendar Script start, never outcomes."},
    {"id":"endpoint", "question":"Do table endpoint support and frozen v2 first/terminal PMT dark diagnostics overlap under their respective explicit time origins? Do not call PMT dark physical extinction."},
    {"id":"stages", "question":"Does explicit stabilizationTime1Sec equal elapsed time from Flame stabilization #1 to Flame stabilization #2, when script identity and TP clock are bound? Missing fields stay unknown; execution interruptions are not a table error."},
    {"id":"command_readback", "question":"At exactly shared, unique acquisition timestamps, do MFCA1/2/3 command voltages and corresponding readback voltages differ? No interpolation or transient exclusion, no volumetric flow inference without qualified calibration."},
    {"id":"conditions", "question":"Across all 19 investigations, do approved scalar fractions lie in [0,1], do complete resolved component fractions sum to one, and do explicit source-unit/SI conversions agree? No missing fraction is zero and multiple values are not collapsed."},
]
CAUSE_CHECKS = (
    "same_method_agreement_controls", "independent_third_NASA_source",
    "physical_conservation", "difference_shape", "same_quantity_and_definition")
VERDICTS = {"table_value_wrong", "table_records_plan_not_execution", "two_definitions",
            "our_reading_wrong", "unresolved"}


def encode(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False).encode()


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def digest_file(path):
    h = hashlib.sha256()
    with path.open("rb") as source:
        for block in iter(lambda: source.read(1 << 20), b""):
            h.update(block)
    return h.hexdigest()


def relative(path):
    return path.relative_to(ROOT).as_posix()


def checked_path(name):
    path = Path(name)
    if path.is_absolute() or ".." in path.parts:
        raise ValueError("nonrelative source route")
    lowered = name.replace("\\", "/").lower()
    if any(part in lowered.split("/") for part in ("holdout", "holdout_ext", "validation", "sealedvalidation", "models", "results")) or "tests_outcomes" in lowered:
        raise ValueError("forbidden source route")
    return ROOT / path


def pinned_bytes(name, expected):
    raw = checked_path(name).read_bytes()
    if sha(raw) != expected:
        raise ValueError("source bytes changed")
    return raw


def immutable(path, raw):
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.exists():
        if path.read_bytes() != raw:
            raise ValueError("immutable output collision")
    else:
        with path.open("xb") as stream:
            stream.write(raw)


def gate_artifact(raw, scope, expected):
    text = raw.decode("utf-8")
    if expected not in scope or audit.top_identity(text) != expected:
        raise ValueError("TRAIN identity denied before payload decode")
    return audit.gated_record(text, scope)


def source_plan():
    scope = audit.train_scope()
    if digest_file(Path(audit.__file__)) != trace.AUDIT_SHA:
        raise ValueError("TRAIN gate code changed")
    files = {relative(SAFE): SAFE_SHA, relative(DOC): DOC_SHA}
    tables = {}
    for family,(name,identity,digest) in TABLES.items():
        path = "runs/dev/data_astra/corpus/incoming/PSI-159/"+identity+".csv"
        files[path] = digest
        tables[family] = {"path":path,"sha256":digest,"original_filename":name,
                          "identity_column":"Test","quantity_column":"burn time (sec)","format":"CSV"}
    records = []
    for rid, identity in sorted(scope.items()):
        stem = identity["family"] + "_" + identity["test"]
        v1 = audit.OUT / "train" / (stem + ".json")
        v2 = audit.OUT / "trace_v2/train" / stem / "inventory.json"
        raw = v1.read_bytes()
        old = gate_artifact(raw, scope, rid)
        files[relative(v1)] = sha(raw)
        files[relative(v2)] = digest_file(v2)
        routes = audit.raw_routes(rid, scope)
        sources = {}
        for key, value in old["sources"].items():
            if value["path"] != relative(routes[key]):
                raise ValueError("exact TRAIN source routing changed")
            files[value["path"]] = value["sha256"]
            sources[key] = value
        records.append({"run_id": rid, **identity, "v1": relative(v1), "v2": relative(v2), "sources": sources})
    return {"version":VERSION,"questions":QUESTIONS,"cause_checks_in_order":list(CAUSE_CHECKS),
            "verdicts":sorted(VERDICTS),"source_files":files,"train_sources":records,
            "train_split_pins":audit.PINS,"investigations":list(INVESTIGATIONS),"tables":tables,
            "named_stage_fields":["stabilizationTime1Sec","Flame stabilization #1","Flame stabilization #2"],
            "unit_document":{"path":relative(DOC),"sha256":DOC_SHA,"pages":[2,4,5,9]},
            "boundaries":{"numerical_payload":"script_only_protected", "images":False,
                          "author_export":"none; conditions projection separate", "models":"no reads or writes",
                          "sensor_not_applicable":[25,26,107]},
            "uncertainty":"Unknown method accuracy remains unknown. No normalized residual without qualified independent uncertainty.",
            "independence":"A repeat of the same code is not independent confirmation; receipts bind question, source hashes, reader and different implementation."}


def freeze():
    plan = source_plan()
    raw = encode(plan) + b"\n"
    key = sha(raw)
    path = PRIVATE / "protocols" / (key + ".json")
    immutable(path, raw)
    return path, key


def number(value):
    if isinstance(value, bool):
        raise ValueError("boolean is not a measurement")
    try:
        parsed = Decimal(str(value))
    except InvalidOperation as error:
        raise ValueError("invalid numeric token") from error
    if not parsed.is_finite():
        raise ValueError("nonfinite measurement")
    return parsed


def measure(raw, unit, source, *, si=None, si_unit=None, clock=None, quantity=None):
    return {"raw_value":raw,"raw_unit":unit,"value_si":si if si is not None else raw,
            "unit_si":si_unit or unit,"locator":source,"clock":clock,
            "quantity":quantity,"uncertainty":{"status":"unknown","value_si":None,
            "reason":"source precision is not calibrated method accuracy"}}


def comparison(rid, investigation, field, left, right, method, *, status="unsettled", delta=None, explanation=None):
    if status not in {"seen_once","unsettled"}:
        raise ValueError("independent confirmation requires a separate receipt")
    if left["unit_si"] != right["unit_si"]:
        raise ValueError("incommensurate quantities")
    row = {"schema":VERSION,"kind":"discrepancy","run_id":rid,"investigation_id":investigation,
        "field":field,"table_value":left,"sensor_value":right,"difference_si":delta,
        "normalized_difference":None,"method_uncertainty":{"status":"unknown","value_si":None},
        "method":{"id":method,"version":VERSION},"status":status,"verdict":"unresolved",
        "deciding_check":None,"cause_established":False,"independent_reader_receipt":None,
        "checks":[{"order":i+1,"id":c,"status":"unavailable","evidence":[]} for i,c in enumerate(CAUSE_CHECKS)],
        "explanation":explanation,"protection":"protected","author_release":False,
        "written_by":{"role":"non_modelling_condition_custodian","agent":"Codex"}}
    row["id"] = "discrepancy:" + sha(encode({"run":rid,"field":field,"left":left,"right":right,"method":method}))
    return row


def independent_confirmation(primary, receipt):
    required = {"reader_id","implementation_sha256","protocol_sha256","source_sha256s","discrepancy_id","verdict","independence_attestation"}
    if not required.issubset(receipt) or receipt["discrepancy_id"] != primary["id"]:
        raise ValueError("incomplete independent receipt")
    if receipt["implementation_sha256"] == primary["method"]["code_sha256"] or receipt["reader_id"] == primary["written_by"]["agent"]:
        raise ValueError("same method/reader rerun is not independent")
    if receipt["verdict"] not in VERDICTS or receipt["independence_attestation"] is not True:
        raise ValueError("unqualified independent verdict")
    # Integration requires an explicit separately reviewed adjudication, never just a boolean.
    raise ValueError("receipt requires host adjudication; this producer never self-confirms")


def loc(path, file_hash, **extra):
    return {"file":path,"sha256":file_hash,**extra}


def read_plan(path):
    raw = path.read_bytes()
    if path.stem != sha(raw):
        raise ValueError("protocol content-address mismatch")
    plan = json.loads(raw)
    if plan["questions"] != QUESTIONS or plan["version"] != VERSION:
        raise ValueError("question protocol drift")
    scope = audit.train_scope()
    if set(scope) != {r["run_id"] for r in plan["train_sources"]}:
        raise ValueError("TRAIN scope drift")
    return plan, scope


def payload(plan, name):
    return pinned_bytes(name, plan["source_files"][name])


def header_identity(record, old, new, sources):
    rid, test = record["run_id"], record["test"]
    out = []
    for kind, value, pointer in [("script",old.get("identity",{}).get("script"),"identity.script"),
                                  ("tp",old.get("identity",{}).get("tp"),"identity.tp")]:
        if value is None or value == test:
            continue
        source = record["sources"].get(kind)
        if not source:
            continue
        left = measure(test,"identifier",loc(source["path"],source["sha256"],field="filename exact test token"),quantity="source identity")
        right = measure(value,"identifier",loc(source["path"],source["sha256"],field="testpointName" if kind=="script" else "Test Point"),quantity="source identity")
        row = comparison(rid,159,"identity."+kind,left,right,"exact_identity_comparison")
        link = new.get("tp_clock_link",{})
        row["checks"][1].update(status="documentary_clock_binding" if kind=="tp" and link.get("aligned") else "unavailable",
                                   evidence=[{"artifact":record["v2"],"field":"tp_clock_link","sha256":sources[record["v2"]]}])
        row["checks"][4].update(status="checked",evidence=["source identity tokens are compared exactly; clock alias does not rewrite them"])
        out.append(row)
    return out


def table_burn(record, plan, scope):
    if record["run_id"] not in scope:
        raise ValueError("table access outside TRAIN")
    source = plan["tables"][record["family"]]
    reader = csv.DictReader(io.StringIO(payload(plan,source["path"]).decode("utf-8-sig")))
    matches = []
    for row_number,row in enumerate(reader,2):
        # Only the exact authorized identity is interpreted numerically.
        if row.get("Test", "").strip() == record["test"]:
            matches.append((row_number,row["burn time (sec)"]))
    if len(matches) != 1:
        return None
    row_number,raw = matches[0]
    try:
        number(raw)
    except ValueError:
        return None
    return raw,loc(source["path"],source["sha256"],row=row_number,column="burn time (sec)",
                   original_filename=source["original_filename"],identity_column="Test",identity=record["test"])


def endpoint_comparisons(record, old, new, plan, scope):
    # Imported diagnostics are measurements by the pinned method, never NASA outcome truth.
    reference = old["reference"]
    optical = new.get("optical",{})
    onset = optical.get("onset_clock_s")
    if onset is None or optical.get("status") != "diagnostic_only":
        return [], "unavailable_unqualified_optical_or_origin"
    table = [reference.get("last_confirmed_active_s"),reference.get("first_terminal_s")]
    if any(x is None for x in table):
        return [], "unavailable_censored_table_interval"
    original = table_burn(record,plan,scope)
    if original is None:
        return [], "unavailable_exact_original_table_row"
    hashes = plan["source_files"]
    results = []
    for label in ("first_confirmed_dark_interval","last_observed_loss_candidate"):
        candidate = optical.get(label)
        if not candidate or candidate.get("transition_lower_clock_s") is None:
            continue
        interval = [candidate["transition_lower_clock_s"]-onset,candidate["transition_upper_clock_s"]-onset]
        gap = max(table[0]-interval[1],interval[0]-table[1],0.0)
        if not gap:
            continue
        original_raw, original_locator = original
        original_locator.update(interval_artifact=record["v1"],interval_artifact_sha256=hashes[record["v1"]],field="reference",source_evidence=reference["evidence"])
        left = measure(original_raw,"s",original_locator,si=table,
                       clock={"origin":"table endpoint producer origin; not newly requalified"},quantity="table endpoint support")
        source = record["sources"]["data"]
        right = measure(interval,"s",loc(source["path"],source["sha256"],channels=["PMT1","PMT2","PMT3"],
            rows=[candidate["first_line"]-1,candidate["first_line"]],artifact=record["v2"],artifact_sha256=hashes[record["v2"]],field="optical."+label),
            clock={"origin":"pinned v1 radiometer onset", "onset_clock_s":onset,"timezone":"unspecified"},quantity="PMT dark diagnostic support")
        row = comparison(record["run_id"],159,"endpoint."+label,left,right,"existing_frozen_v2_interval_separation",delta=gap,
            explanation="Nonoverlapping numerical supports under inherited origins; neither origin equivalence nor physical extinction is qualified.")
        row["checks"][4].update(status="unsettled_distinct_observables",evidence=["PMT dark is not qualified physical extinction; table time origin requires independent qualification"])
        row["checks"][3].update(status="checked",evidence=[{"kind":"minimum_interval_separation","unit":"s","value":gap}])
        results.append(row)
    return results, "compared"


def script_fields(raw):
    result = {}
    for line_no,line in enumerate(raw.decode("utf-8").splitlines(),1):
        key,sep,value = line.strip().rstrip(",").partition(":")
        if sep:
            key = key.strip()
            if key in result:
                raise ValueError("duplicate script field")
            result[key] = (value.strip(),line_no)
    return result


def stage_comparison(record, old, new, plan):
    if new.get("script_binding",{}).get("status") != "exact_identity" or not new.get("tp_clock_link",{}).get("aligned"):
        return [],"unavailable_identity_or_clock"
    stamps = new.get("tp_stamps",{})
    start,end = (stamps.get(n) for n in ("Flame stabilization #1","Flame stabilization #2"))
    source = record["sources"].get("script")
    if not source or not start or not end:
        return [],"unavailable_named_stage"
    fields = script_fields(payload(plan,source["path"]))
    item = fields.get("stabilizationTime1Sec")
    if item is None:
        return [],"unavailable_planned_duration"
    planned = number(item[0])
    observed = Decimal(str((datetime.fromisoformat(end["raw"])-datetime.fromisoformat(start["raw"])).total_seconds()))
    if planned < 0 or observed < 0:
        return [],"unavailable_invalid_chronology"
    if planned == observed:
        return [],"agreement"
    left = measure(item[0],"s",loc(source["path"],source["sha256"],row=item[1],field="stabilizationTime1Sec"),si=float(planned),quantity="planned stage duration")
    tp = record["sources"]["tp"]
    right = measure([start["raw"],end["raw"]],"calendar timestamps",loc(tp["path"],tp["sha256"],rows=[start["line"],end["line"]],
        fields=["Flame stabilization #1","Flame stabilization #2"]),si=float(observed),si_unit="s",quantity="observed stage elapsed time",
        clock={"domain":"bound TP acquisition calendar", "timezone":"unspecified"})
    row = comparison(record["run_id"],159,"stages.stabilization1",left,right,"explicit_named_stage_elapsed",delta=float(observed-planned))
    row["checks"][4].update(status="checked_plan_and_elapsed_distinct",evidence=["runtime latency, pause or early stop remain possible; difference alone establishes no cause"])
    return [row],"difference"


def exact_rows(raw, test, kind):
    lines = raw.splitlines(keepends=True)
    meta = trace.metadata(lines[0])
    if meta["fields"]["Test Point"] != test:
        raise ValueError("trace identity before body")
    columns = lines[1].decode("utf-8").strip().split(",")
    if len(set(columns)) != len(columns) or columns[0] != "Time":
        raise ValueError("unknown trace schema")
    if kind == "commands" and tuple(columns) != trace.COMMAND_COLUMNS:
        raise ValueError("command channel schema changed")
    values = {}
    previous = None
    for n,line in enumerate(lines[2:],3):
        clock, parsed, errors,_ = trace.parse_numeric_row(line,columns)
        if errors or clock is None or previous is not None and clock <= previous:
            raise ValueError("invalid or nonunique chronology; no repair")
        previous = clock
        values[clock] = (n,line.decode("utf-8").strip().split(","),parsed)
    return meta,columns,values


def command_comparisons(record, plan):
    sources = record["sources"]
    if "data" not in sources or "setpoints" not in sources:
        return [],"unavailable_missing_source",0
    try:
        sm,sc,sr = exact_rows(payload(plan,sources["data"]["path"]),record["test"],"sensor")
        cm,cc,cr = exact_rows(payload(plan,sources["setpoints"]["path"]),record["test"],"commands")
    except ValueError:
        return [],"unavailable_schema_or_chronology",0
    if sm["date"] != cm["date"] or sm["clock_ms"] != cm["clock_ms"]:
        return [],"unavailable_clock_origin",0
    matched = sorted(sr.keys() & cr.keys())
    results = []
    for index,channel in enumerate(trace.SENSOR_REQUIRED[3:],1):
        if channel not in sc:
            return [],"unavailable_channel",0
        # Retain every exact-time pair in protected evidence, including agreement controls.
        pairs = []
        for clock in matched:
            sn,st,_ = sr[clock]; cn,ct,_ = cr[clock]
            actual, command = number(st[sc.index(channel)]),number(ct[index])
            pairs.append({"clock_ms":clock,"sensor_row":sn,"command_row":cn,
                          "sensor_raw":st[sc.index(channel)],"command_raw":ct[index],
                          "difference_v":str(actual-command)})
        different = [p for p in pairs if number(p["difference_v"]) != 0]
        if not different:
            continue
        left = measure([p["command_raw"] for p in pairs],"V",loc(sources["setpoints"]["path"],sources["setpoints"]["sha256"],
            column=cc[index],rows=[p["command_row"] for p in pairs]),si=[float(number(p["command_raw"])) for p in pairs],quantity="command voltage")
        right = measure([p["sensor_raw"] for p in pairs],"V",loc(sources["data"]["path"],sources["data"]["sha256"],
            channel=channel,rows=[p["sensor_row"] for p in pairs]),si=[float(number(p["sensor_raw"])) for p in pairs],quantity="readback voltage")
        clock = {"domain":"identical acquisition date and start, exact millisecond timestamp join", "date":sm["date"],
                 "clock_ms":[p["clock_ms"] for p in pairs],"timezone":"unspecified"}
        left["clock"] = right["clock"] = clock
        row = comparison(record["run_id"],159,"command_readback.MFCA"+str(index),left,right,"exact_timestamp_voltage_pairs",
            delta=[float(number(p["difference_v"])) for p in pairs],explanation="Voltage comparison only; no calibrated physical flow or table-flow verdict. Transients retained.")
        row["unit_evidence"] = {"path":relative(DOC),"sha256":DOC_SHA,"page":4,"table":1}
        row["checks"][0].update(status="executed",evidence=[{"pairs":len(pairs),"exact_agreements":len(pairs)-len(different),
            "selection":"all exact unique shared timestamps; no value-based selection"}])
        row["checks"][4].update(status="checked_different_observables",evidence=["requested voltage and measured voltage; neither guarantees actual volumetric flow"])
        results.append(row)
    return results,"compared",len(matched)*3


def condition_checks(plan):
    raw = payload(plan,relative(SAFE))
    coverage = Counter(); rows = []
    for line_no,line in enumerate(raw.splitlines(),1):
        data = json.loads(line)
        inv = int(data["investigation_id"].split("-")[-1])
        coverage[inv] += 1
        for field,value in data["conditions"].items():
            if value.get("status") != "resolved" or value.get("unit_si") != "1" or not any(k in field for k in ("fraction","x_o2","x_fuel","x_co2")):
                continue
            actual = number(value["value_si"])
            if 0 <= actual <= 1:
                continue
            source = loc(relative(SAFE),SAFE_SHA,row=line_no,field=field,original_row_sha256=data["row_sha256"])
            left = measure(value["value_si"],"1",source,quantity="approved fraction")
            right = measure([0,1],"1",{"definition":"dimensionless fraction closed range"},quantity="fraction domain")
            row = comparison(data["atlas_id"],inv,"conditions."+field,left,right,"fraction_domain",delta=None)
            row["checks"][2].update(status="executed",evidence=["fraction outside closed unit interval; source semantics still require independent reading"])
            rows.append(row)
    return rows,coverage


def execute(protocol_path):
    plan,scope = read_plan(protocol_path)
    # All exact pins validated before payload comparisons; each payload rehashes same parsed bytes.
    for name,expected in plan["source_files"].items():
        if digest_file(checked_path(name)) != expected:
            raise ValueError("frozen source drift")
    code_sha = digest_file(Path(__file__))
    version = sha(encode({"protocol":protocol_path.stem,"code":code_sha}))
    destination = PRIVATE / version
    counts = Counter(); gaps = Counter(); output = []
    for entry in plan["train_sources"]:
        rid = entry["run_id"]
        old = gate_artifact(payload(plan,entry["v1"]),scope,rid)
        new = gate_artifact(payload(plan,entry["v2"]),scope,rid)
        if new.get("prior_receipt_sha256") != plan["source_files"][entry["v1"]]:
            raise ValueError("v2 dependency pin changed")
        found = header_identity(entry,old,new,plan["source_files"])
        output.extend(found); counts["identity.records"] += len(found)
        found,state = endpoint_comparisons(entry,old,new,plan,scope)
        output.extend(found); counts["endpoint.records"] += len(found); gaps["endpoint."+state] += 1
        found,state = stage_comparison(entry,old,new,plan)
        output.extend(found); counts["stages.records"] += len(found); gaps["stages."+state] += 1
        found,state,pairs = command_comparisons(entry,plan)
        output.extend(found); counts["command_readback.records"] += len(found)
        counts["command_readback.exact_time_pairs"] += pairs; gaps["command_readback."+state] += 1
    conditions,condition_coverage = condition_checks(plan)
    output.extend(conditions); counts["conditions.records"] = len(conditions)
    for row in output:
        row["method"].update(code_sha256=code_sha,protocol_sha256=protocol_path.stem)
    raw = b"".join(encode(row)+b"\n" for row in output)
    immutable(destination/"discrepancies.jsonl",raw)
    links = [{"schema":"astra.discrepancy.link/1","relation":"a table value to the sensor value that contradicts it",
              "discrepancy_id":row["id"],"run_id":row["run_id"],"from":row["table_value"]["locator"],
              "to":row["sensor_value"]["locator"],"protection":"protected","assertion":"observed difference; cause unresolved"} for row in output]
    immutable(destination/"links.jsonl",b"".join(encode(row)+b"\n" for row in links))
    coverage = [{"investigation_id":"PSI-"+str(inv),"approved_condition_rows_screened":condition_coverage[inv],
                 "sensor_comparison":"executed_exact_TRAIN" if inv==159 else "not_applicable" if inv in (25,26,107) else "unavailable_no_frozen_exact_source_quantity_clock_join",
                 "protected_outcome_counts":"not_disclosed","pressure_composition_chamber":"unavailable_exact_initial_calibration_join",
                 "fraction_sums":"unavailable_complete_component_basis_not_established","unit_conversion":"unavailable_original_raw_unit_pair_not_in_safe_projection"} for inv in INVESTIGATIONS]
    summary = {"version":VERSION,"protocol_sha256":protocol_path.stem,"code_sha256":code_sha,"train_identities":len(scope),
               "N_discrepancies":len(output),"M_causes_established":0,"counts":dict(counts),"coverage_states":dict(gaps),
               "investigation_coverage":coverage,"independent_confirmation":"pending_separate_reader",
               "discrepancies_sha256":sha(raw),"author_payload_rows":0,"physical_flow_qualification":"unavailable"}
    immutable(destination/"operational_summary.json",encode(summary)+b"\n")
    immutable(LABELS/"discrepancies.jsonl",raw)
    immutable(LABELS/"discrepancies_links.jsonl",(destination/"links.jsonl").read_bytes())
    immutable(LABELS/("discrepancies_operational_"+version+".json"),encode(summary)+b"\n")
    print(json.dumps({"version":version,"N":len(output),"M":0,"authorized_train_counts":dict(counts),"coverage_states":dict(gaps),"protected_output":relative(destination),"author_payload_rows":0},sort_keys=True))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--freeze",action="store_true")
    parser.add_argument("--execute",action="store_true")
    parser.add_argument("--protocol",type=Path)
    args = parser.parse_args()
    if not args.execute:
        print("Dry run: no source payloads read or outputs written. Use --freeze --execute, then --protocol PATH --execute.")
    elif args.freeze:
        path,digest = freeze(); print(json.dumps({"protocol":relative(path),"sha256":digest,"questions":len(QUESTIONS),"train_identities":154}))
    elif args.protocol:
        execute(args.protocol)
    else:
        parser.error("--execute requires --freeze or --protocol")


if __name__ == "__main__":
    main()
