"""Source-bound condition intake proposals; no atlas or split mutation.

Only this non-modelling custodian may read the source paths used here. Author outputs
contain screened conditions and metadata, never the mixed source bodies. --execute
is required for writes; all published versions are immutable and content addressed.
"""

import argparse
from collections import Counter, defaultdict
from contextlib import contextmanager
import csv
from datetime import datetime
from hashlib import sha256
import io
import json
import math
import os
from pathlib import Path
import re
import tempfile

from screen_conditions import ROOT, encode, hash_file, immutable_write, INVESTIGATIONS

BASE = ROOT / "runs/dev/data_astra"
OUT = BASE / "work/condition_supplements"
FILES = ROOT / "runs/dev/data_all/work/atlas/files.jsonl"
SCHEMA = ROOT / "tools/wsp/atlas/atlas_schema.py"
BASS_CSV = "PSI-26_Experimental table_BASS.csv"
BASS_XLSX = "PSI-26_Science Documents_As Run_Test Matrix.xlsx"
BASS_TRAINING = "PSI-26_Presentations_Crew Training 2011.pdf"
ERD = "PSI-68_Engineering Documents_FLEX-2 Engineering Requirments Document.pdf"
SRD = "PSI-68_Science Documents_FLEX-2 SRD Final _ No Hypertext .pdf"
ACME_DATA_DOC = "PSI-23_Science Documents_ACME non-imaging data file description.pdf"
FUEL_LABELS = {
    "Decane": ("decane",), "Ethanol": ("ethanol",), "Toluene": ("toluene",),
    "IsoOctane": ("isooctane",), "Heptane": ("heptane",), "N_Octane": ("n_octane",),
    "Propylbenzene": ("propylbenzene",), "Propanol80_Glycerol20": ("propanol", "glycerol"),
    "Propanol95_Glycerol5": ("propanol", "glycerol"), "Heptane50_Toluene50": ("heptane", "toluene"),
    "Heptane50_Iso-Octane50": ("heptane", "isooctane"),
    "Decane75_Propylbenzene25": ("decane", "propylbenzene"),
    "Decane25_Propylbenzene75": ("decane", "propylbenzene"),
    "Decane50_Propylbenzene50": ("decane", "propylbenzene"),
    "100% Octane": ("octane",), "95% Propanol, 5% Glycerol": ("propanol", "glycerol"),
}


def source_ref(meta, receipt, **locator):
    return {"identity": meta["identity"], "investigation_id": meta["investigation"],
            "psi_file_id": meta["psi_file_id"], "file_name": meta["file_name"],
            "file_version": meta["file_version"], "sha256": receipt["sha256"],
            "stable_url": meta.get("stable_url"), **locator}


class Sources:
    def __init__(self):
        self.metadata = [json.loads(line) for line in (BASE / "metadata/selected_files.jsonl").open()]
        self.receipts = {}
        with (BASE / "logs/fetch_receipts.jsonl").open() as stream:
            for line in stream:
                try:
                    row = json.loads(line)
                except json.JSONDecodeError:
                    if stream.read():
                        raise ValueError("invalid acquisition ledger") from None
                    break
                self.receipts[row["identity"]] = row
        self.coverage = {}
        self.verified = {}

    def named(self, investigation, name):
        found = [r for r in self.metadata if r["investigation"] == investigation and r["file_name"] == name]
        if len(found) != 1:
            raise ValueError("source identity is ambiguous")
        return found[0]

    def available(self, meta):
        receipt = self.receipts.get(meta["identity"], {})
        return receipt.get("state") in ("verified_download", "verified_existing")

    def verified_path(self, meta):
        """Opaque identity check only; parsers must use open_verified's snapshot."""
        key = meta["identity"]
        if not self.available(meta):
            self.coverage[key] = {"source": meta["file_name"], "state": "not_available_verified"}
            return None
        path = (BASE / meta["local_path"]).resolve()
        if not path.is_relative_to((BASE / "corpus/incoming").resolve()):
            raise ValueError("source path outside incoming corpus")
        receipt = self.receipts[key]
        if path.stat().st_size != receipt["actual_bytes"] or hash_file(path) != receipt["sha256"]:
            raise ValueError("source bytes changed")
        self.verified[key] = path
        self.coverage[key] = {"source": meta["file_name"], "state": "verified_source_opened",
                              "sha256": receipt["sha256"]}
        return path

    @contextmanager
    def open_verified(self, meta):
        """Parse exactly the receipt-verified bytes, never a reopened live path.

        The private temporary stream is fully checked before being yielded. Source
        replacement during parsing fails on exit and again before publication.
        """
        path = self.verified_path(meta)
        if path is None:
            yield None
            return
        receipt = self.receipts[meta["identity"]]
        with path.open("rb") as original, tempfile.TemporaryFile(mode="w+b") as snapshot:
            before = os.fstat(original.fileno())
            h, size = sha256(), 0
            for chunk in iter(lambda: original.read(1 << 20), b""):
                h.update(chunk)
                size += len(chunk)
                snapshot.write(chunk)
            after = os.fstat(original.fileno())
            identity = lambda info: (info.st_dev, info.st_ino, info.st_size, info.st_mtime_ns)
            if (h.hexdigest() != receipt["sha256"] or size != receipt["actual_bytes"]
                    or identity(before) != identity(after) or identity(after) != identity(path.stat())):
                raise ValueError("source changed while snapshotting")
            snapshot.seek(0)
            yield snapshot
        if path.stat().st_size != receipt["actual_bytes"] or hash_file(path) != receipt["sha256"]:
            raise ValueError("used source bytes changed")

    def verify_used_sources(self):
        """A changed used source invalidates the run; no receipt is silently updated."""
        for key, path in self.verified.items():
            receipt = self.receipts[key]
            if path.stat().st_size != receipt["actual_bytes"] or hash_file(path) != receipt["sha256"]:
                raise ValueError("used source bytes changed")

    def ref(self, meta, **locator):
        return source_ref(meta, self.receipts[meta["identity"]], **locator)


def atlas_value(value=None, unit=None, *, basis=None, source=None, status=None, reason=None):
    if status is None:
        status = "resolved" if value is not None else "unknown"
    if isinstance(value, float) and not math.isfinite(value):
        raise ValueError("nonfinite condition")
    locator = None
    if source:
        locator = {"file": source["file_name"], "sheet": source.get("sheet"),
                   "column": source.get("column"), "row": source.get("row"),
                   "page": source.get("page"), "field": source.get("field"), "note": None}
    return {"status": status, "value_si": value, "unit_si": unit, "raw_value": None,
            "raw_unit": None, "basis": basis, "scope": "row", "source": locator,
            "rule": None, "flags": [], "reason": reason if status != "resolved" else None,
            "set": None, "alternatives": []}


def proposal(investigation, key, original_ids, conditions, refs, level="test"):
    return {"source_schema": "ignis.atlas.test/1", "proposal_key": key,
            "investigation_id": investigation, "level": level, "atlas_id": None,
            "group_id": None, "identity_assignment": "pending_atlas_owner",
            "original_identifiers": original_ids, "conditions": conditions,
            "source_evidence": refs, "partial": True,
            "use": "condition_intake_only_no_partition_or_outcome_grant"}


def material_token(text):
    text = text.strip().lower()
    if re.search(r"\bsibal\b", text):
        return "sibal_fabric"
    if re.search(r"\bpmma\b", text):
        return "pmma"
    if re.search(r"\bultem\b", text):
        return "ultem"
    if "nomex" in text:
        return "nomex_blend" if "blend" in text else "nomex_iii_natural"
    if "japan wax" in text:
        return "japan_wax"
    if "paraffin" in text or "parrafin" in text:
        return "paraffin_wax"
    return None


def identifier_text(value):
    if isinstance(value, (int, float)) and not isinstance(value, bool) and int(value) == value:
        return str(int(value))
    return str(value).strip() if value is not None else ""


def bass_workbook_conditions(sources):
    """Only setup/identity/date columns are interpreted, never radiometer/media columns."""
    meta = sources.named("PSI-26", BASS_XLSX)
    found = defaultdict(list)
    import openpyxl
    with sources.open_verified(meta) as snapshot:
        if snapshot is None:
            return found
        book = openpyxl.load_workbook(snapshot, read_only=True, data_only=True)
        try:
            sheet = book["as run matrix"]
            if [sheet.cell(3, col).value for col in (3, 4, 5, 6, 8)] != [
                    "As run test no.", "Test no.", "Sample no.", "material", "Flow restrictor"]:
                raise ValueError("BASS setup layout changed")
            for n, values in enumerate(sheet.iter_rows(min_row=4, max_col=8, values_only=True), 4):
                key = tuple(identifier_text(values[k]) for k in (2, 3, 4))
                if all(key):
                    date = values[0].date().isoformat() if isinstance(values[0], datetime) else None
                    restrictor = identifier_text(values[7])
                    found[key].append({"source": sources.ref(meta, sheet="as run matrix", row=n),
                                       "date": date,
                                       "flow_restrictor": restrictor if re.fullmatch(r"\d+", restrictor) else None})
        finally:
            book.close()
    return found


def bass_intake(sources):
    meta = sources.named("PSI-26", BASS_CSV)
    with sources.open_verified(meta) as snapshot:
        if snapshot is None:
            return [], {"state": "condition_table_not_available_verified"}
        with io.TextIOWrapper(snapshot, encoding="utf-8-sig", newline="") as stream:
            reader = csv.DictReader(stream)
            if reader.fieldnames != ["Date", "GMT", "As Run Test #", "Test #", "Sample #", "Fuel Sample Material"]:
                raise ValueError("BASS condition columns changed")
            rows = [(n, row) for n, row in enumerate(reader, 2)]
    workbook = bass_workbook_conditions(sources)
    training = sources.named("PSI-26", BASS_TRAINING)
    training_ref = None
    if sources.verified_path(training):
        training_ref = sources.ref(training, page=21)
        if training_ref["sha256"] != "11ad76194acf095886b0baf33ac3cb960f266d55dce0ab5b15c69de0db7afb62":
            raise ValueError("reviewed BASS apparatus page changed")
    sample_materials = defaultdict(set)
    sample_sources = defaultdict(list)
    for n, row in rows:
        if row["Test #"] and row["Sample #"]:
            token = material_token(row["Fuel Sample Material"])
            if token:
                sample_materials[row["Sample #"]].add(token)
                sample_sources[row["Sample #"]].append(n)
    output, issues = [], Counter()
    for n, row in rows:
        if not row["Test #"]:
            issues["non_test_source_rows"] += 1
            continue
        ref = sources.ref(meta, row=n)
        workbook_matches = workbook.get((row["As Run Test #"], row["Test #"], row["Sample #"]), [])
        conditions = {}
        try:
            date = datetime.strptime(row["Date"], "%m/%d/%Y").date()
            if date.year not in (2012, 2013) or date.timetuple().tm_yday != int(row["GMT"]):
                raise ValueError("conflicting date")
            conditions["date"] = atlas_value(date.isoformat(), "date", basis="nominal",
                                               source=ref | {"column": "Date,GMT"})
        except (ValueError, TypeError):
            conditions["date"] = atlas_value(unit="date", reason="source_date_requires_resolution")
            issues["date_requires_resolution"] += 1
        material = material_token(row["Fuel Sample Material"])
        material_derived = False
        refs = [ref]
        if len(workbook_matches) == 1:
            matched = workbook_matches[0]
            refs.append(matched["source"])
            if (matched["date"] and conditions["date"]["status"] == "resolved"
                    and matched["date"] != conditions["date"]["value_si"]):
                conditions["date"] = atlas_value(unit="date", reason="conflicting_exact_source_dates")
                issues["date_conflicts_with_exact_workbook_join"] += 1
            if matched["date"] and conditions["date"]["status"] == "unknown":
                conditions["date"]["flags"] = ["conflict"]
                conditions["date"]["alternatives"] = [{"value_si": matched["date"], "raw_value": matched["date"],
                    "source": atlas_value(source=matched["source"] | {"column": "Actual Date"})["source"]}]
                issues["date_with_exact_workbook_alternative"] += 1
            if matched["flow_restrictor"]:
                conditions["extra_conditions.flow_restrictor"] = atlas_value(matched["flow_restrictor"],
                    basis="nominal", source=matched["source"] | {"column": "Flow restrictor"})
                issues["nominal_setup_restrictor_recovered"] += 1
        if material is None and len(sample_materials[row["Sample #"]]) == 1:
            material = next(iter(sample_materials[row["Sample #"]]))
            material_derived = True
            refs.extend(sources.ref(meta, row=k, column="Sample #,Fuel Sample Material")
                        for k in sample_sources[row["Sample #"]])
        material_ref = (sources.ref(meta, row=sample_sources[row["Sample #"]][0],
                        column="Sample #,Fuel Sample Material") if material_derived
                        else ref | {"column": "Fuel Sample Material"})
        conditions["fuel.name"] = atlas_value(material, basis=("derived" if material_derived else "nominal") if material else None,
            source=material_ref, reason="material_identity_not_explicit")
        # The public schema has no spherical-solid/candle configuration or diameter.
        raw_shape = row["Fuel Sample Material"].lower()
        is_flat = any(word in raw_shape for word in ("sheet", "slab"))
        geometry_ref, geometry_basis = ref | {"column": "Fuel Sample Material"}, "nominal"
        if training_ref and material in ("sibal_fabric", "nomex_blend", "nomex_iii_natural", "ultem"):
            is_flat = True
            refs.append(training_ref)
            geometry_ref, geometry_basis = training_ref, "derived"
        conditions["geometry.configuration"] = atlas_value("flat_sample" if is_flat else None,
            basis=geometry_basis if is_flat else None, source=geometry_ref,
            reason="source_shape_not_representable_or_not_explicit_in_current_vocabulary")
        if "opposed" in raw_shape or "oppsed" in raw_shape:
            conditions["flow.direction"] = atlas_value("opposed", basis="nominal",
                source=ref | {"column": "Fuel Sample Material"})
        for field, unit in (("flow.speed", "m/s"), ("atmosphere.x_o2", "1"),
                            ("atmosphere.pressure", "Pa"), ("atmosphere.temperature", "K")):
            conditions[field] = atlas_value(unit=unit, reason="not_in_screened_condition_table")
        output.append(proposal("PSI-26", f"{ref['sha256']}:row:{n}",
            {"test_id": row["Test #"], "as_run_test_id": row["As Run Test #"], "sample_id": row["Sample #"]},
            conditions, refs))
    return output, {"source_rows": len(rows), "proposals": len(output), "issues": dict(issues),
                    "geometry_gap": "solid_sphere_and_candle_not_in_atlas_configuration_enum"}


def parse_folder(folder):
    if not folder.startswith("Reduced Gravity/"):
        return None
    campaign = folder.split("/")[1].strip()
    label = campaign.rsplit(" - ", 1)[-1]
    if label not in FUEL_LABELS:
        return None
    year = re.search(r"\b(20\d{2})\b", campaign)
    return {"species": FUEL_LABELS[label], "campaign_year": int(year[1]) if year else None}


def clock_stamp(text):
    return datetime.fromisoformat(text.strip())


def summary_prefix(stream, expected_point):
    """Whitelisted controller header and last temperature sample before igniter command.

    Do not parse any numeric field from the first row at/after ignition. Never read
    subsequent rows. The bound is the source controller event, not flame detection.
    """
    point, ignition, started = None, None, None
    initial, channels, last_time = None, None, None
    for line_no, raw in enumerate(stream, 1):
        text = raw.decode("utf-8-sig", errors="strict").rstrip("\r\n")
        if text.startswith("Test Point:"):
            point = text.split(":", 1)[1].strip()
        elif text.startswith("AS Started:"):
            value = text.split(":", 1)[1].strip()
            started = clock_stamp(value) if value else None
        elif text.startswith("AS Burn to Completion/Igniter On:"):
            value = text.split(":", 1)[1].strip()
            ignition = clock_stamp(value) if value else None
        elif text.startswith("MDCA MP Time"):
            # Only this explicitly identified trailing pair is supported. Match
            # units and positions, rather than treating any last number as degC.
            labels = re.findall(r"\bAve\s+(\S+)", text)
            tail = re.search(r"\bAve\s+(PSIA|DegC)\s+Ave\s+(PSIA|DegC)\s*$", text)
            channels = ({unit: index - 2 for index, unit in enumerate(tail.groups())}
                        if tail and len(labels) == 2 and set(tail.groups()) == {"PSIA", "DegC"}
                        else None)
        elif re.match(r"\d{4}-\d{2}-\d{2} ", text):
            if point != expected_point or not ignition or not started or not channels:
                return {"state": "clock_or_channel_not_established", "point": expected_point}
            fields = text.split()
            stamp = clock_stamp(" ".join(fields[:2]))
            if stamp >= ignition:
                break
            if last_time and stamp <= last_time:
                return {"state": "nonmonotonic_prefix", "point": expected_point}
            last_time = stamp
            if stamp < started:
                continue
            if len(fields) != 18:
                return {"state": "unknown_telemetry_column_layout", "point": expected_point}
            pressure, temperature = float(fields[channels["PSIA"]]), float(fields[channels["DegC"]])
            if not math.isfinite(pressure) or not math.isfinite(temperature):
                raise ValueError("nonfinite preignition condition")
            initial = {"line": line_no, "line_sha256": sha256(raw).hexdigest(),
                       "timestamp": stamp.isoformat(), "temperature_c": temperature,
                       "pressure_psia": pressure, "seconds_before_igniter": (ignition-stamp).total_seconds()}
    if point != expected_point or not ignition or not started:
        return {"state": "clock_not_established", "point": expected_point}
    return {"state": "preigniter_observation" if initial else "no_preigniter_observation",
            "point": point, "igniter_command_time": ignition.isoformat(),
            "sequence_start_time": started.isoformat(), "initial": initial}


def mixture_timestamp(snapshot):
    with io.TextIOWrapper(snapshot, encoding="utf-8-sig", newline="") as stream:
        reader = csv.DictReader(stream)
        if not reader.fieldnames or reader.fieldnames[0] != "Date/Time":
            return None
        row = next(reader, None)
        if row is None or next(reader, None) is not None:
            return None
        # Composition columns are intentionally never returned by this parser.
        for form in ("%m/%d/%Y %H:%M", "%m/%d/%Y %H:%M:%S"):
            try:
                return datetime.strptime(row["Date/Time"], form).isoformat()
            except ValueError:
                pass
        return None  # Ambiguous 12/24-hour or two-digit-year cells are not repaired.


def flex2_intake(sources, catalogue):
    catalogue_sha = hash_file(FILES)
    rows = [r for r in catalogue if r["investigation"] == "PSI-68"]
    by_point = defaultdict(list)
    for row in rows:
        if row["test_id_scheme"] == "cir_test_point" and row["test_id"]:
            by_point[row["test_id"]].append(row)
    selected = {r["psi_file_id"]: r for r in sources.metadata if r["investigation"] == "PSI-68"}
    mixture = []
    for meta in selected.values():
        if not meta["file_name"].lower().endswith("_mixture.txt"):
            continue
        with sources.open_verified(meta) as snapshot:
            stamp = mixture_timestamp(snapshot) if snapshot else None
            available = snapshot is not None
        mixture.append({"source": sources.ref(meta) if available else {"identity": meta["identity"]},
                        "reading_point": meta["file_name"].rsplit("_", 1)[0],
                        "timestamp": stamp, "time_zone": "unstated",
                        "composition_release": "withheld_pending_exact_fill_and_time_join"})
    proposals, temporal_pairs, diagnostics = [], [], Counter()
    for point, members in sorted(by_point.items()):
        parsed = [(r, parse_folder(r["folder"])) for r in members]
        parsed = [(r, f) for r, f in parsed if f]
        fuels = {f["species"] for _, f in parsed}
        summaries = [selected[r["psi_file_id"]] for r in members
                     if r["psi_file_id"] in selected and re.fullmatch(
                         r"(?:PSI-68_Raw Data_)?" + re.escape(point) + r"\.txt", r["file_name"])]
        observations = []
        for meta in {r["identity"]: r for r in summaries}.values():
            with sources.open_verified(meta) as snapshot:
                if snapshot is None:
                    continue
                obs = summary_prefix(snapshot, point)
                observations.append((meta, obs))
                diagnostics["summary_sources_screened"] += 1
                diagnostics["summary_state_" + obs["state"]] += 1
        conditions, refs = {}, []
        clocks = {obs.get("igniter_command_time") for _, obs in observations if obs.get("igniter_command_time")}
        dates = {clock_stamp(stamp).date().isoformat() for stamp in clocks}
        valid_fuels = len(fuels) == 1 and not any(
            f["campaign_year"] is not None and dates and any(int(d[:4]) != f["campaign_year"] for d in dates)
            for _, f in parsed)
        if valid_fuels:
            species = next(iter(fuels))
            folder_source = {"file_name": "runs/dev/data_all/work/atlas/files.jsonl",
                             "field": "psi_file_id=" + parsed[0][0]["psi_file_id"] + ".folder"}
            conditions["fuel.name"] = atlas_value("+".join(species), basis="nominal", source=folder_source)
            conditions["fuel.components"] = [{"species": token, "role": "fuel",
                "fraction": atlas_value(unit="1", source=folder_source,
                    reason="folder_identity_does_not_establish_fraction_basis_or_purity")}
                for token in species]
            diagnostics["unique_fuel_folder_proposals"] += 1
        else:
            conditions["fuel.name"] = atlas_value(reason="no_unique_source_fuel_folder_join")
        folder_refs = [{"catalogue_sha256": catalogue_sha, "listing_row": r["listing_row"],
                        "psi_file_id": r["psi_file_id"], "file_name": r["file_name"],
                        "folder": r["folder"]} for r, _ in parsed]
        # Deduplicate evidence without silently deciding conflicting fuel assignments.
        folder_refs = list({encode(r): r for r in folder_refs}.values())
        resolved = [(meta, obs) for meta, obs in observations if obs["state"] == "preigniter_observation"]
        agrees = len({encode(obs) for _, obs in resolved}) <= 1
        if resolved and len(clocks) == 1 and agrees:
            meta, obs = resolved[0]
            initial = obs["initial"]
            ref = sources.ref(meta, row=initial["line"], field="Ave DegC",
                              record_sha256=initial["line_sha256"])
            value = atlas_value(initial["temperature_c"] + 273.15, "K", basis="measured", source=ref)
            value.update(raw_value=initial["temperature_c"], raw_unit="degC")
            conditions["atmosphere.temperature"] = value
            refs.append(ref)
            diagnostics["preigniter_temperature_proposals"] += 1
            timing = {k: obs[k] for k in ("igniter_command_time", "sequence_start_time")}
            timing.update(observation_time=initial["timestamp"], seconds_before_igniter=initial["seconds_before_igniter"],
                          clock="same_source_MDCA_controller_clock", method="last_source_sample_before_igniter_command")
        else:
            conditions["atmosphere.temperature"] = atlas_value(unit="K", reason="preigniter_clock_or_channel_not_resolved")
            timing = {"state": "not_resolved"}
        day_matches = bool(len(dates) == 1 and re.fullmatch(r"[A-Z]\d{3}[A-Z]\d+", point)
            and datetime.fromisoformat(next(iter(dates))).timetuple().tm_yday == int(point[1:4]))
        if len(dates) == 1 and day_matches:
            meta = next(meta for meta, obs in observations if obs.get("igniter_command_time"))
            date_ref = sources.ref(meta, field="AS Burn to Completion/Igniter On")
            refs.append(date_ref)
            conditions["date"] = atlas_value(next(iter(dates)), "date", basis="nominal", source=date_ref)
        else:
            conditions["date"] = atlas_value(unit="date", reason="no_unique_source_calendar_day")
        item = proposal("PSI-68", "cir_test_point:" + point, {"cir_test_point": point}, conditions, refs,
                        level="test_point")
        item.update(fuel_folder_evidence=folder_refs, preignition_timing=timing,
                    measurement_qualification="source_observation_only_calibration_not_independently_verified",
                    burn_identity_status="source_test_point_not_independently_classified_as_burn",
                    calendar_day_consistency="matches_source_clock" if day_matches else "not_resolved",
                    split_status="unchanged_held_until_owner_test_list_and_freeze")
        proposals.append(item)
        for mix in mixture:
            if not mix["timestamp"] or mix["reading_point"][:4] != point[:4]:
                continue
            mt = clock_stamp(mix["timestamp"])
            if dates and mt.date().isoformat() not in dates:
                continue
            relation = "unknown"
            if len(clocks) == 1:
                relation = "pre" if mt < clock_stamp(next(iter(clocks))) else "post"
            temporal_pairs.append({"cir_test_point": point, "reading_point": mix["reading_point"],
                "reading_source_identity": mix["source"]["identity"],
                "source_clock_order": relation, "timing_status": "unknown",
                "join_status": "same_day_candidate_not_verified_chamber_fill",
                "reason": "mixture_clock_zone_and_fill_membership_not_proven"})
    return proposals, mixture, temporal_pairs, dict(diagnostics)


def temperature_coverage(sources, flex):
    result = {}
    for number in INVESTIGATIONS:
        inv = f"PSI-{number}"
        metadata = [r for r in sources.metadata if r["investigation"] == inv]
        raw = [r for r in metadata if r["category"] == "Raw Data"]
        candidates = [r for r in raw if re.search(
            r"_Chamber_Readings\.txt$|_data\.csv$|HKPData_|Raw telemetry file\.csv$|Telemetry Data|Sensor Data|Sensors\.xlsx$|_num\.txt$|DTrk|PTrk",
            r["file_name"], re.I)]
        headers, kinds_seen = [], set()
        for meta in candidates:
            name = meta["file_name"]
            kind = ("chamber" if "Chamber_Readings" in name else "acme_data" if name.endswith("_data.csv")
                    else re.sub(r"\d+", "#", name))
            if kind in kinds_seen or not name.lower().endswith((".txt", ".csv")) or not sources.available(meta):
                continue
            kinds_seen.add(kind)
            with sources.open_verified(meta) as snapshot:
                if snapshot is None:
                    continue
                with io.TextIOWrapper(snapshot, encoding="utf-8-sig", errors="replace") as stream:
                    header = stream.readline()
                    if kind == "acme_data":
                        header = stream.readline()
            columns = next(csv.reader([header]))
            channel_names = [c for c in columns if (re.fullmatch(
                r"FCU_MP_TM[1-4]\(Deg C\)|TC[A-E]|TC200[A-B]|TCBurner|TCCJ", c)
                or re.search(r"temp(?:erature)?", c, re.I) and not re.search("pres", c, re.I))]
            headers.append({"source": sources.ref(meta, row=2 if kind == "acme_data" else 1),
                            "temperature_channel_names": channel_names,
                            "interpretation": "header_only_not_ambient_or_preignition_qualification"})
        count = sum(r["conditions"]["atmosphere.temperature"]["status"] == "resolved" for r in flex) if inv == "PSI-68" else 0
        result[inv] = {"atmosphere.temperature": atlas_value(unit="K",
            reason="per_test_initial_temperature_not_supplied_by_this_inventory"),
            "listed_raw_sources": len(raw), "candidate_sources_by_filename": len(candidates),
            "verified_candidates": sum(sources.available(r) for r in candidates),
            "header_sources_screened": headers, "new_per_point_preigniter_observations": count,
            "remaining_requirement": "exact_test_clock_join_and_ambient_sensor_semantics_before_numeric_release"}
    doc = sources.named("PSI-23", ACME_DATA_DOC)
    if sources.verified_path(doc):
        for inv in ("PSI-10", "PSI-20", "PSI-21", "PSI-22", "PSI-23", "PSI-159"):
            result[inv]["screened_instrument_document"] = {
                "source": sources.ref(doc, pages=[2, 6, 7, 9]),
                "applies_to": "common_ACME_software_interface_with_explicit_BRE_variants",
                "TC200A": "igniter_side_Type_K_thermocouple_200_mm_from_axis",
                "TC200B": "relay_side_Type_K_thermocouple_200_mm_from_axis",
                "TCBurner": "burner_temperature_not_ambient;not_valid_for_BRE",
                "TCCJ": "cold_junction_compensation_assembly_temperature_not_ambient",
                "TCA_TCD": "near_or_far_rake_or_BRE_specific_thermocouples",
                "numeric_release": "requires_analog_channel_conversion_cold_junction_and_preigniter_command_log_join",
                "unit_warning": "channel_names_alone_do_not_make_analog_input_values_degrees_C"}
    return result


def validate_values(proposals):
    # Use the authoritative producer metadata, not a parallel value schema.
    namespace = {}
    exec(compile(SCHEMA.read_bytes(), str(SCHEMA), "exec"), namespace)
    from jsonschema import Draft202012Validator
    schema = namespace["row_schema"]()
    for item in proposals:
        if item["level"] not in namespace["LEVELS"]:
            raise ValueError("unknown atlas level")
        for path, value in item["conditions"].items():
            field_schema = schema
            for segment in path.split("."):
                field_schema = field_schema["properties"][segment]
            if not Draft202012Validator(field_schema).is_valid(value):
                raise ValueError("proposal violates authoritative field schema")
            values = [value] if path != "fuel.components" else [component["fraction"] for component in value]
            if any(namespace["value_problems"](path, v) for v in values):
                raise ValueError("invalid atlas value proposal")
            if any(v["status"] not in namespace["STATUS"] for v in values):
                raise ValueError("unknown atlas status")
            if path == "geometry.configuration" and value["status"] == "resolved":
                if value["value_si"] not in namespace["CONFIGURATION"]:
                    raise ValueError("unapproved geometry vocabulary")


def run(execute=False):
    initial_schema_sha = hash_file(SCHEMA)
    initial_code_sha = hash_file(Path(__file__))
    initial_catalogue_sha = hash_file(FILES)
    sources = Sources()
    catalogue = []
    with FILES.open(encoding="utf-8") as stream:
        for line in stream:
            row = json.loads(line)
            if row["investigation"] == "PSI-68":
                catalogue.append(row)
    bass, bass_summary = bass_intake(sources)
    flex, mixture, pairs, flex_summary = flex2_intake(sources, catalogue)
    temperature = temperature_coverage(sources, flex)
    validate_values(bass + flex)
    method_sources = []
    for name in (ERD, SRD):
        meta = sources.named("PSI-68", name)
        if sources.verified_path(meta):
            method_sources.append(sources.ref(meta))
    pin = {"version": "astra-condition-supplements/1", "schema_sha256": initial_schema_sha,
           "file_catalogue_sha256": initial_catalogue_sha, "code_sha256": initial_code_sha,
           "acquisition_metadata_sha256": hash_file(BASE / "metadata/selected_files.jsonl"),
           "used_sources": sorted((k, r["sha256"]) for k, r in sources.coverage.items() if r.get("sha256"))}
    version = sha256(encode(pin)).hexdigest()
    summary = {"version": version, "source_schema": "ignis.atlas.test/1", "partial": True,
               "PSI-26": bass_summary, "PSI-68": {"point_proposals": len(flex), "mixture_sources": len(mixture),
                   "same_day_candidate_pairs": len(pairs), "verified_fill_composition_joins": 0, **flex_summary},
               "temperature_investigations_inventoried": len(temperature),
               "atlas_modified": False, "split_modified": False, "outcome_route_used": False}
    methods = {"method": "same_source_last_preigniter_sample/1", "source_documents": method_sources,
        "ERD_pages": [22, 28, 29], "SRD_pages": [42, 43, 51],
        "interpretation": "Source documents specify chamber ambient thermometry and GMT synchronization. The summary header identifies Ave DegC. Extraction uses the last source sample strictly before the same-file controller igniter command and stops at that boundary.",
        "limits": ["Requirement specifications are not independent calibration evidence.",
                   "No reference-temperature or ISS-temperature default is filled.",
                   "The last source sample is a preigniter observation, not a fitted or averaged t=0 estimate.",
                   "Igniter command time does not establish whether ignition succeeded.",
                   "Mixture timestamp timezone and chamber-fill membership are not inferred from proximity."]}
    sources.verify_used_sources()
    if execute:
        if (hash_file(SCHEMA) != initial_schema_sha or hash_file(Path(__file__)) != initial_code_sha
                or hash_file(FILES) != initial_catalogue_sha):
            raise ValueError("screening inputs changed")
        private = OUT / "private" / version
        public = OUT / "author_safe" / version
        private.mkdir(parents=True, exist_ok=True)
        public.mkdir(parents=True, exist_ok=True)
        exposure = {"role": "non_modelling_condition_custodian", "model_authoring_permitted": False,
            "source_exposures": [
                "One unverified A109D03 mixed-telemetry prefix was exposed during format discovery; no values sent to authors.",
                "BASS workbook header discovery included limited mixed body cells; no body outcomes released.",
                "FLEX-2 comments were scanned for exact gas-reading identifiers; a mixed comment was screened and rejected as a usable composition join.",
                "One FLEX-2 event-log screenshot was inspected; it duplicated controller-command information and did not establish mixture membership."],
            "mixture_join_search": {"comment_files_scanned": 581,
                "usable_explicit_composition_references": 0,
                "eventlog_image_sha256": "d886d0b1dedba320455e655e306c2315bb8751e7f09333a1a7c073c464974d9c"},
            "source_bodies_in_author_output": False}
        immutable_write(private / "exposure_ledger.json", encode(exposure) + b"\n")
        immutable_write(private / "source_coverage.json", encode(sources.coverage) + b"\n")
        # Mixture provenance/timing is metadata only; no composition values are written.
        files = {"psi26_intake.jsonl": b"".join(encode(r) + b"\n" for r in bass),
                 "psi68_intake.jsonl": b"".join(encode(r) + b"\n" for r in flex),
                 "psi68_mixture_sources.json": encode(mixture) + b"\n",
                 "psi68_mixture_join_candidates.jsonl": b"".join(encode(r) + b"\n" for r in pairs),
                 "temperature_coverage.json": encode(temperature) + b"\n",
                 "method.json": encode(methods) + b"\n", "summary.json": encode(summary) + b"\n"}
        for name, raw in files.items():
            immutable_write(public / name, raw)
        immutable_write(public / "manifest.json", encode(pin | {
            "outputs": {name: sha256(raw).hexdigest() for name, raw in files.items()}}) + b"\n")
    print(json.dumps({"mode": "execute" if execute else "dry_run", "summary": summary,
                      "author_directory": str(OUT / "author_safe" / version) if execute else None}))
    return summary


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--execute", action="store_true")
    args = parser.parse_args()
    try:
        run(args.execute)
    except (OSError, ValueError, KeyError, TypeError, UnicodeError):
        raise SystemExit("Condition supplement failed closed; source bodies are not printed.") from None


if __name__ == "__main__":
    main()
