"""Trusted non-modelling custody pass over the fixed conditions-only atlas.

Dry run by default. No outcome, model, source-document or split payload is opened.
The reviewed policy is bound to the exact atlas producer and catalogue below.
"""

import argparse
from collections import Counter
from hashlib import sha256
import json
from pathlib import Path
import subprocess

from author_gate import (ATLAS_VERSION, AtlasConditions, ConditionsApproval, Pin,
                         CONDITION_PATHS, conditions_approval_bytes, digest)

ROOT = Path(__file__).resolve().parents[3]
AUTHORITY = "5c75f6b91e4acdab8f3bd983981223194faf6148"
CATALOGUE = ROOT / "runs/dev/data_all/work/atlas/tests.jsonl"
CATALOGUE_SHA = "35546e129a160c6ab73c630be82fab98700bb5fd5fb92fa83954eb7af06cdfbd"
PINS = {
    "tools/wsp/atlas/atlas_schema.py": "42574895346e09fff2769b07c95d40be8293d4cc949d0935f691a7c48248be56",
    "docs/wsp/atlas/SCHEMA.md": "b63b28e9763d1ad92892fa7424c093cbeae725b97571092c092a05eac1aeb1fc",
    "tools/wsp/atlas/normalize_tests.py": "61f4d43e12f93eaa3115f7d1b99292429dd79617508929fbb4926fa2ea7e1472",
}
# This is an exposure policy, not a chemistry vocabulary or physical property table.
# All tokens were inspected as material identities in this exact condition snapshot.
SPECIES = frozenset("arizona_road_dust butane decane dibutyl_phthalate dodecane iso_dodecane "
    "ethane ethanol ethylene farnesane heptane isooctane hydrogen kapton lampwick methane "
    "methanol n_octane pmma nomex propane propanol glycerol propylene pyrell sibal_fabric "
    "silicone silicone_rubber teflon nitrogen helium carbon_dioxide oxygen".split())
INVESTIGATIONS = (10, 20, 21, 22, 23, 25, 26, 39, 47, 68, 69, 98, 99, 100, 101, 102, 106, 107, 159)
PHYSICS_FAMILIES = {
    "droplets": (39, 68, 69), "spherical_burners": (10, 23, 159), "flat_burners": (20,),
    "jet_coflow": (21, 22, 106, 107), "solids": (25, 26, 98, 99, 100),
    "heated_smoke": (101, 102), "aerosol": (47,),
}
PROPERTY_REQUIREMENTS = {
    "droplets": ["liquid density", "vapour pressure and validity range", "latent heat", "heat capacity",
                 "liquid viscosity", "surface tension", "gas transport", "mixture fraction basis"],
    "spherical_burners": ["species thermochemistry", "gas transport", "radiative properties", "mixture fraction basis"],
    "flat_burners": ["species thermochemistry", "gas transport", "radiative properties", "mixture fraction basis"],
    "jet_coflow": ["species thermochemistry", "gas transport", "radiative properties", "mixture fraction basis"],
    "solids": ["material grade and composition", "bulk density", "heat capacity", "thermal conductivity",
               "thermal decomposition", "emissivity", "porosity and thickness"],
    "heated_smoke": ["material grade and composition", "density", "heat capacity", "thermal decomposition",
                      "aerosol optical and transport properties"],
    "aerosol": ["particle material composition", "density", "shape and size definition", "optical properties"],
}
INITIAL_COLUMNS = {
    "PSI-10": {"atmosphere.pressure": "pinit (atm)", "atmosphere.x_o2": "XO2 (init)",
               "oxidizer_stream.x_o2": "XO2 (init)", "atmosphere.x_fuel_ambient": "XC2H4 initial"},
    "PSI-159": {"atmosphere.x_o2": " XO2 before test", "oxidizer_stream.x_o2": " XO2 before test",
                "atmosphere.x_fuel_ambient": "Xf  before test"},
}
STATIC_DERIVATIONS = frozenset(("H1", "H2", "H3", "G1", "G2", "P1", "P2", "P3", "P5",
                              "F1", "F2", "OR1", "CFG-DROPLET", "CFG-SHEET"))


def encode(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False).encode()


def hash_file(path):
    h = sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def verify_authority():
    for name, expected in PINS.items():
        if digest((ROOT / name).read_bytes()) != expected:
            raise ValueError("screening authority changed")
        committed = subprocess.run(["git", "show", f"{AUTHORITY}:{name}"], cwd=ROOT,
                                   capture_output=True, check=True).stdout
        if digest(committed) != expected:
            raise ValueError("committed screening authority changed")
    if hash_file(CATALOGUE) != CATALOGUE_SHA:
        raise ValueError("catalogue snapshot changed")


def at(row, path):
    value = row
    for key in path.split("."):
        value = value[int(key)] if isinstance(value, list) else value[key]
    return value


def approved_name(value):
    return isinstance(value, str) and all(part in SPECIES for part in value.split("+"))


def value_reason(row, path, value):
    """A private reason; never exported per row to an author."""
    if value["flags"] or value["alternatives"]:
        return "requires_independent_resolution"
    if value["status"] not in ("resolved", "multiple"):
        return "not_available_as_preignition_value"
    if value["status"] == "multiple" and value["basis"] != "nominal":
        return "multiple_not_established_as_planned"
    if value["source"] is None:
        return "missing_provenance"
    # PSI-21's source explicitly includes concentrations varied during burns.
    # The nominal tag alone does not establish a preignition value or plan.
    if (row["investigation_id"] == "PSI-21"
            and value["source"].get("column") == "Fuel conc. Range"):
        return "timing_not_established_as_preignition"
    if path == "fuel.name":
        names = [value["value_si"]] if value["status"] == "resolved" else value["set"]["values"]
        if not all(approved_name(name) for name in names):
            return "identity_not_reviewed"
    if value["basis"] == "nominal":
        return None
    column = value["source"].get("column")
    if (value["basis"] == "unstated" and path in INITIAL_COLUMNS.get(row["investigation_id"], {})
            and column == INITIAL_COLUMNS[row["investigation_id"]][path]):
        return None
    if value["basis"] == "derived":
        rule = value["rule"]
        if rule in STATIC_DERIVATIONS:
            return None
        if rule == "PURE" and not value_reason(row, "fuel.name", row["fuel"]["name"]):
            return None
        if rule == "OX1" and not value_reason(row, "atmosphere.x_o2", row["atmosphere"]["x_o2"]):
            return None
    return "timing_not_established_as_preignition"


def candidate_values(row):
    for path in sorted(CONDITION_PATHS):
        yield path, at(row, path)
    if "fuel.name" not in CONDITION_PATHS:
        yield "fuel.name", row["fuel"]["name"]
    for index, item in enumerate(row["fuel"]["components"]):
        yield f"fuel.components.{index}.fraction", item["fraction"]
    for index, item in enumerate(row["atmosphere"]["diluents"]["items"]):
        yield f"atmosphere.diluents.items.{index}.fraction", item["fraction"]
    for index, item in enumerate(row["geometry"]["sizes"]):
        yield f"geometry.sizes.{index}.value", item["value"]


def screen_row(row):
    approved, omitted = [], {}
    for path, value in candidate_values(row):
        reason = value_reason(row, path, value)
        if path.startswith("fuel.components.") and value_reason(row, "fuel.name", row["fuel"]["name"]):
            reason = "parent_identity_requires_screening"
        # A day-level list of different fuels is not a single mixture.
        if path.startswith("fuel.components.") and row["fuel"]["name"]["status"] == "multiple":
            reason = "component_context_not_a_single_mixture"
        if path == "fuel.name" and value["status"] == "multiple":
            reason = "identity_list_retained_as_separate_components"
        if reason:
            omitted[path] = reason
        else:
            approved.append(path)
    if not value_reason(row, "fuel.name", row["fuel"]["name"]):
        for index, item in enumerate(row["fuel"]["components"]):
            if item["species"] in SPECIES:
                approved.extend((f"fuel.components.{index}.species", f"fuel.components.{index}.role"))
    diluents = row["atmosphere"]["diluents"]
    if diluents["status"] in ("resolved", "multiple"):
        for index, item in enumerate(diluents["items"]):
            if item["species"] in SPECIES:
                approved.append(f"atmosphere.diluents.items.{index}.species")
    for index, _item in enumerate(row["geometry"]["sizes"]):
        # Dimension is a closed apparatus label, independently of numeric availability.
        approved.append(f"geometry.sizes.{index}.dimension")
    return tuple(approved), omitted


def immutable_write(path, raw):
    if path.exists():
        if path.read_bytes() != raw:
            raise ValueError("immutable output differs")
        return
    with path.open("xb") as stream:
        stream.write(raw)


def run(execute=False):
    verify_authority()
    schema_pin = Pin(ATLAS_VERSION, PINS["tools/wsp/atlas/atlas_schema.py"])
    prose_pin = Pin(ATLAS_VERSION, PINS["docs/wsp/atlas/SCHEMA.md"])
    approvals, omissions = {}, {}
    first_hash = sha256()
    with CATALOGUE.open("rb") as stream:
        for raw in stream:
            first_hash.update(raw)
            row = json.loads(raw)
            key = row["atlas_id"]
            if key in approvals:
                raise ValueError("duplicate catalogue identity")
            fields, omitted = screen_row(row)
            approvals[key] = ConditionsApproval(digest(raw), tuple(row[k] for k in
                ("atlas_id", "investigation_id", "test_id", "level")), fields,
                row["group_id"], CATALOGUE_SHA, approved_species=tuple(sorted(SPECIES)))
            omissions[key] = omitted
    if first_hash.hexdigest() != CATALOGUE_SHA:
        raise ValueError("catalogue changed while screening")
    mapping = conditions_approval_bytes(approvals, schema_pin, prose_pin)
    mapping_pin = Pin("author-conditions-approvals/1", digest(mapping))
    gate = AtlasConditions(schema_pin, prose_pin, approvals, mapping_pin)
    policy = {"version": "astra-condition-screening/1", "authority_commit": AUTHORITY,
              "catalogue_sha256": CATALOGUE_SHA, "source_pins": PINS,
              "screening_code_sha256": hash_file(Path(__file__)),
              "gate_code_sha256": hash_file(ROOT / "tools/wsp/astra/author_gate.py"),
              "approvals_sha256": digest(mapping)}
    version = digest(encode(policy))
    custody = ROOT / "runs/dev/data_astra/work/condition_custody" / version
    destination = ROOT / "runs/dev/data_astra/work/author_conditions" / version
    staging = None
    if execute:
        custody.mkdir(parents=True, exist_ok=True)
        immutable_write(custody / "policy.json", encode(policy) + b"\n")
        immutable_write(custody / "approvals.json", mapping + b"\n")
        immutable_write(custody / "omissions.private.json", encode(omissions) + b"\n")
        staging = (custody / "projection.pending.jsonl").open("wb")
    inventory = {f"PSI-{n}": {"rows": 0, "levels": Counter(), "groups": set(),
        "conditions_present": Counter(), "conditions_multiple": Counter(),
        "species": set(), "geometry_dimensions": set(), "geometry_values_present": Counter()}
        for n in INVESTIGATIONS}
    second_hash, projection_hash = sha256(), sha256()
    count = 0
    try:
        with CATALOGUE.open("rb") as stream:
            for raw in stream:
                second_hash.update(raw)
                # Identity comes from exact pinned row bytes; no alias or source lookup.
                key = json.loads(raw)["atlas_id"]
                result = gate.read(key, lambda _key, raw=raw: raw)
                if result.decision.state != "allowed":
                    raise ValueError("gate rejected screened row")
                projected = result.projection
                projected["group_id"] = approvals[key].group_id
                projected["catalogue_sha256"] = CATALOGUE_SHA
                projected["screening_version"] = version
                projected["component_semantics"] = "source identities; simultaneity is not inferred"
                projected["omitted_fields"] = {path: "not_released_by_preignition_screen"
                    for path in omissions[key]}
                # Hash locators preserve provenance without releasing source text or paths.
                projected["condition_provenance"] = {path: {
                    "catalogue_sha256": CATALOGUE_SHA, "row_sha256": approvals[key].row_sha256,
                    "field_path": path} for path in projected["conditions"]}
                public = encode(projected) + b"\n"
                projection_hash.update(public)
                if staging:
                    staging.write(public)
                count += 1
                inv = inventory[projected["investigation_id"]]
                inv["rows"] += 1
                inv["levels"][projected["level"]] += 1
                inv["groups"].add(projected["group_id"])
                for path, value in projected["conditions"].items():
                    inv["conditions_present"][path] += 1
                    if isinstance(value, dict) and value.get("status") == "multiple":
                        inv["conditions_multiple"][path] += 1
                    if path == "fuel.name":
                        names = [value["value_si"]] if value["status"] == "resolved" else value["set"]["values"]
                        for name in names:
                            inv["species"].update(name.split("+"))
                    if path.endswith(".species"):
                        inv["species"].add(value["value_si"])
                    if path.startswith("geometry.sizes.") and path.endswith(".dimension"):
                        inv["geometry_dimensions"].add(value["value_si"])
                        value_path = path.rsplit(".", 1)[0] + ".value"
                        if value_path in projected["conditions"]:
                            inv["geometry_values_present"][value["value_si"]] += 1
        if second_hash.hexdigest() != CATALOGUE_SHA or count != len(approvals):
            raise ValueError("catalogue changed during projection")
    finally:
        if staging:
            staging.close()
    for investigation, entry in inventory.items():
        entry["group_count"] = len(entry.pop("groups"))
        entry["species"] = sorted(entry["species"])
        entry["geometry_dimensions"] = sorted(entry["geometry_dimensions"])
        entry["coverage_state"] = "condition_rows_present" if entry["rows"] else "no_catalogue_rows"
        entry["physical_condition_requirements"] = {path: {
            "rows_with_released_value": entry["conditions_present"].get(path, 0),
            "rows_without_released_value": entry["rows"] - entry["conditions_present"].get(path, 0),
            "unavailable_reason": "not_released_by_preignition_screen"}
            for path in ("fuel.name", "fuel.phase", "geometry.configuration", "atmosphere.pressure",
                "atmosphere.temperature", "atmosphere.x_o2", "oxidizer_stream.x_o2",
                "flow.speed", "injection.fuel_mass_flow", "injection.x_fuel_injected", "gravity")}
        family = next(name for name, numbers in PHYSICS_FAMILIES.items()
                      if int(investigation.split("-")[1]) in numbers)
        entry["physics_family"] = family
        entry["physical_property_research_requirements"] = PROPERTY_REQUIREMENTS[family]
    report = {"version": "astra-author-condition-inventory/1", "partial": True,
              "source_schema": ATLAS_VERSION, "screening_version": version,
              "catalogue_sha256": CATALOGUE_SHA, "projection_sha256": projection_hash.hexdigest(),
              "rows": count, "investigations_expected": 19, "by_investigation": inventory,
              "meaning": "Condition coverage only; neither number of burns nor model qualification."}
    if (hash_file(Path(__file__)) != policy["screening_code_sha256"]
            or hash_file(ROOT / "tools/wsp/astra/author_gate.py") != policy["gate_code_sha256"]):
        raise ValueError("screening code changed during projection")
    if execute:
        destination.mkdir(parents=True, exist_ok=True)
        target = destination / "conditions.jsonl"
        pending = custody / "projection.pending.jsonl"
        if target.exists():
            if hash_file(target) != projection_hash.hexdigest():
                raise ValueError("immutable projection differs")
            pending.unlink()
        else:
            pending.replace(target)
        immutable_write(destination / "inventory.json", encode(report) + b"\n")
        immutable_write(destination / "manifest.json", encode(policy | {
            "screening_version": version, "projection_sha256": projection_hash.hexdigest(),
            "inventory_sha256": digest(encode(report) + b"\n")}) + b"\n")
    print(json.dumps({"mode": "executed" if execute else "dry_run", "rows": count,
        "investigations_with_rows": sum(bool(v["rows"]) for v in inventory.values()),
        "screening_version": version, "author_directory": str(destination) if execute else None}))
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--execute", action="store_true")
    args = parser.parse_args()
    try:
        run(args.execute)
    except (ValueError, KeyError, TypeError, OSError, subprocess.CalledProcessError):
        raise SystemExit("Conditions screening failed closed; no source values are printed.") from None


if __name__ == "__main__":
    main()
