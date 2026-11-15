"""Closed lists and the JSON Schema of one atlas row (docs/wsp/atlas/SCHEMA.md is the prose form).

Single source of the allowed values: `normalize_tests.py` writes only these tokens and
`check_atlas.py` validates every row against `row_schema()` and checks that SCHEMA.md
names every token.
"""

SCHEMA_ID = "ignis.atlas.test/1"

LEVELS = ["test", "test_point", "test_day", "matrix_row"]
PARTITIONS = ["train", "validation", "outside_partitions"]
STATUS = ["resolved", "multiple", "varying", "unknown", "not_applicable", "withheld_outcome"]
BASIS = ["nominal", "measured", "derived", "unstated"]
SCOPE = ["row", "investigation"]
SET_KIND = ["list", "range"]
UNITS = ["Pa", "1", "m", "m/s", "kg/s", "m3/s", "K", "s", "W", "date", "time"]

RACK = ["CIR", "MSG", "Cygnus"]
INSERT = ["MDCA", "ACME", "Saffire", "CSD_SAME", "own_apparatus"]
PHASE = ["gas", "liquid_droplet", "liquid_other", "solid"]
CONFIGURATION = [
    "spherical_porous_burner",
    "flat_porous_burner",
    "gas_jet_burner",
    "coflow_burner",
    "droplet_free",
    "droplet_fiber_supported",
    "droplet_unspecified",
    "flat_sample",
    "heated_wire_sample",
    "aerosol_bag",
]
ORIENTATION = ["normal", "inverse"]
DIMENSION = [
    "burner_diameter",
    "nozzle_inner_diameter",
    "initial_droplet_diameter",
    "sample_thickness",
    "sample_length",
    "sample_width",
]
DIRECTION = ["quiescent", "concurrent", "opposed", "coflow", "forced", "burner_injection"]
GRAVITY = ["microgravity", "normal_gravity", "partial_gravity"]

FLAME_TYPE = [
    "spherical_diffusion_flame",
    "flat_burner_diffusion_flame",
    "jet_diffusion_flame",
    "coflow_jet_diffusion_flame",
    "droplet_flame",
    "solid_surface_flame",
    "overheat_smoke",
    "none",
]
STUDIED = ["cool_flame", "smoke_point", "extinction", "electric_field", "liftoff", "burning_rate", "aerosol"]
FIRE_SAFETY_NEED = ["prevention", "detection", "suppression"]

CREW_ATMOSPHERE = ["earth_iss", "moon_base_rover", "spacesuit"]
O2_ROLE = ["ambient", "injected"]

OUTCOME_STATUS = ["listed", "none_in_source", "not_read_protected"]
MEASUREMENT_STATUS = ["not_filled", "filled"]
MEASUREMENT_KIND = [
    "color_imaging",
    "backlit_imaging",
    "uv_imaging",
    "intensified_imaging",
    "still_images",
    "radiometer",
    "photomultiplier",
    "thermocouple",
    "pressure",
    "gas_flow",
    "gas_composition",
    "air_velocity",
    "particle_instrument",
    "electrical",
    "test_log",
]

FLAGS = [
    "approximate",
    "conflict",
    "malformed_in_table",
    "blank_in_table",
    "fill_series_artifact",
    "fractions_sum_not_1",
    "negative_value",
    "conditioned_on_ignition",
    "defect_noted",
    "balance_gas",
    "from_analysis_sheet",
    "basis_of_fraction_unstated",
]
# flags copied from a project record keep their text behind this prefix
RECORD_FLAG_PREFIX = "rec:"

EXTRA_CONDITIONS = [
    "campaign_day",
    "description",
    "test_type",
    "fiber",
    "igniter_power",
    "igniter_time",
    "heater_resistance_ratio",
    "heating_duration",
    "primary_aging_time",
    "primary_mixing_time",
    "planned_aging_duration",
    "mesh_polarity",
    "fuel_mfc_full_scale",
    "n2_mfc_full_scale",
    "fuel_volume_flow_std",
    "hardware_description",
    "sampled_location",
    "fan_display",
    "flow_restrictor",
    "x_co",
    "zst",
    "tad",
]

VALUE_KEYS = [
    "status",
    "value_si",
    "unit_si",
    "raw_value",
    "raw_unit",
    "basis",
    "scope",
    "source",
    "rule",
    "flags",
    "reason",
    "set",
    "alternatives",
]
SOURCE_KEYS = ["file", "sheet", "column", "row", "page", "field", "note"]

ROW_KEYS = [
    "schema",
    "atlas_id",
    "investigation_id",
    "acronym",
    "test_id",
    "aliases",
    "level",
    "parent_test_id",
    "project_partition",
    "group_id",
    "group_rule",
    "source_rows",
    "date",
    "start_time",
    "hardware",
    "fuel",
    "geometry",
    "atmosphere",
    "oxidizer_stream",
    "flow",
    "injection",
    "gravity",
    "extra_conditions",
    "varied",
    "measurements",
    "outcome_status",
    "outcome_fields_available",
    "labels",
    "map",
    "notes",
]

ALL_LISTS = {
    "level": LEVELS,
    "project_partition": PARTITIONS,
    "status": STATUS,
    "basis": BASIS,
    "scope": SCOPE,
    "set.kind": SET_KIND,
    "unit_si": UNITS,
    "hardware.rack": RACK,
    "hardware.insert": INSERT,
    "fuel.phase": PHASE,
    "geometry.configuration": CONFIGURATION,
    "geometry.flame_orientation": ORIENTATION,
    "geometry.sizes.dimension": DIMENSION,
    "flow.direction": DIRECTION,
    "gravity": GRAVITY,
    "labels.flame_type": FLAME_TYPE,
    "labels.studied": STUDIED,
    "labels.fire_safety_need": FIRE_SAFETY_NEED,
    "map.crew_atmospheres": CREW_ATMOSPHERE,
    "map.x_o2_role": O2_ROLE,
    "outcome_status": OUTCOME_STATUS,
    "measurements.status": MEASUREMENT_STATUS,
    "measurements.items.kind": MEASUREMENT_KIND,
    "flags": FLAGS,
    "extra_conditions": EXTRA_CONDITIONS,
}


def _source_schema():
    return {
        "type": ["object", "null"],
        "additionalProperties": False,
        "required": SOURCE_KEYS,
        "properties": {
            "file": {"type": "string", "minLength": 1},
            "sheet": {"type": ["string", "null"]},
            "column": {"type": ["string", "null"]},
            "row": {"type": ["integer", "null"]},
            "page": {"type": ["integer", "null"]},
            "field": {"type": ["string", "null"]},
            "note": {"type": ["string", "null"]},
        },
    }


def _value_schema(enum=None, kind=None, units=None):
    """One value object. `enum`: closed list for value_si; `kind`: 'number' or 'string'."""
    if enum is not None:
        value = {"enum": list(enum) + [None]}
    elif kind == "number":
        value = {"type": ["number", "null"]}
    elif kind == "string":
        value = {"type": ["string", "null"]}
    else:
        value = {"type": ["number", "string", "null"]}
    flag = {
        "anyOf": [
            {"enum": FLAGS},
            {"type": "string", "pattern": "^" + RECORD_FLAG_PREFIX + ".+"},
        ]
    }
    return {
        "type": "object",
        "additionalProperties": False,
        "required": VALUE_KEYS,
        "properties": {
            "status": {"enum": STATUS},
            "value_si": value,
            "unit_si": {"enum": (units if units is not None else UNITS) + [None]},
            "raw_value": {"type": ["string", "number", "null"]},
            "raw_unit": {"type": ["string", "null"]},
            "basis": {"enum": BASIS + [None]},
            "scope": {"enum": SCOPE},
            "source": _source_schema(),
            "rule": {"type": ["string", "null"]},
            "flags": {"type": "array", "items": flag},
            "reason": {"type": ["string", "null"]},
            "set": {
                "type": ["object", "null"],
                "additionalProperties": False,
                "required": ["kind", "values"],
                "properties": {
                    "kind": {"enum": SET_KIND},
                    "values": {"type": "array", "minItems": 1, "items": {"type": ["number", "string"]}},
                },
            },
            "alternatives": {
                "type": "array",
                "items": {
                    "type": "object",
                    "additionalProperties": False,
                    "required": ["value_si", "raw_value", "source"],
                    "properties": {
                        "value_si": {"type": ["number", "string", "null"]},
                        "raw_value": {"type": ["string", "number", "null"]},
                        "source": _source_schema(),
                    },
                },
            },
        },
    }


def _labelled(enum):
    return {
        "type": "array",
        "items": {
            "type": "object",
            "additionalProperties": False,
            "required": ["value", "rule", "matched", "source"],
            "properties": {
                "value": {"enum": enum},
                "rule": {"type": "string"},
                "matched": {"type": "string"},
                "source": _source_schema(),
            },
        },
    }


def row_schema():
    num = lambda units: _value_schema(kind="number", units=units)
    frac = num(["1"])
    obj = lambda props: {
        "type": "object",
        "additionalProperties": False,
        "required": list(props),
        "properties": props,
    }
    component = obj(
        {
            "species": {"type": "string", "minLength": 1},
            "role": {"enum": ["fuel", "diluent"]},
            "fraction": frac,
        }
    )
    props = {
        "schema": {"const": SCHEMA_ID},
        "atlas_id": {"type": "string", "pattern": "^PSI-[0-9]+:.+"},
        "investigation_id": {"type": "string", "pattern": "^PSI-[0-9]+$"},
        "acronym": {"type": "string", "minLength": 1},
        "test_id": {"type": "string", "minLength": 1},
        "aliases": {
            "type": "array",
            "items": obj({"scheme": {"type": "string"}, "name": {"type": "string"}}),
        },
        "level": {"enum": LEVELS},
        "parent_test_id": {"type": ["string", "null"]},
        "project_partition": {"enum": PARTITIONS},
        "group_id": {"type": "string", "minLength": 1},
        "group_rule": {"type": "string", "minLength": 1},
        "source_rows": {
            "type": "array",
            "minItems": 1,
            "items": obj(
                {
                    "file": {"type": "string"},
                    "sheet": {"type": ["string", "null"]},
                    "row": {"type": ["integer", "null"]},
                }
            ),
        },
        "date": _value_schema(kind="string", units=["date"]),
        "start_time": _value_schema(kind="string", units=["time"]),
        "hardware": obj(
            {
                "rack": _value_schema(enum=RACK, units=[]),
                "insert": _value_schema(enum=INSERT, units=[]),
                "raw": {"type": ["string", "null"]},
            }
        ),
        "fuel": obj(
            {
                "name": _value_schema(kind="string", units=[]),
                "components": {"type": "array", "items": component},
                "phase": _value_schema(enum=PHASE, units=[]),
            }
        ),
        "geometry": obj(
            {
                "configuration": _value_schema(enum=CONFIGURATION, units=[]),
                "flame_orientation": _value_schema(enum=ORIENTATION, units=[]),
                "sizes": {
                    "type": "array",
                    "items": obj({"dimension": {"enum": DIMENSION}, "value": num(["m"])}),
                },
            }
        ),
        "atmosphere": obj(
            {
                "x_o2": frac,
                "pressure": num(["Pa"]),
                "temperature": num(["K"]),
                "diluents": obj(
                    {
                        "status": {"enum": ["resolved", "multiple", "unknown", "not_applicable"]},
                        "reason": {"type": ["string", "null"]},
                        "items": {
                            "type": "array",
                            "items": obj({"species": {"type": "string", "minLength": 1}, "fraction": frac}),
                        },
                    }
                ),
                "x_fuel_ambient": frac,
            }
        ),
        "oxidizer_stream": obj({"x_o2": frac}),
        "flow": obj({"speed": num(["m/s"]), "direction": _value_schema(enum=DIRECTION, units=[])}),
        "injection": obj(
            {
                "x_fuel_injected": frac,
                "fuel_mass_flow": num(["kg/s"]),
                "oxidizer_mass_flow": num(["kg/s"]),
                "inert_mass_flow": num(["kg/s"]),
                "total_mass_flow": num(["kg/s"]),
            }
        ),
        "gravity": _value_schema(enum=GRAVITY, units=[]),
        "extra_conditions": {
            "type": "object",
            "additionalProperties": False,
            "properties": {k: _value_schema() for k in EXTRA_CONDITIONS},
        },
        "varied": {"type": "array", "items": {"type": "string"}},
        "measurements": obj(
            {
                "status": {"enum": MEASUREMENT_STATUS},
                "items": {
                    "type": "array",
                    "items": obj(
                        {
                            "kind": {"enum": MEASUREMENT_KIND},
                            "category": {"type": "string"},
                            "n_files": {"type": "integer"},
                            "bytes": {"type": "integer"},
                        }
                    ),
                },
            }
        ),
        "outcome_status": {"enum": OUTCOME_STATUS},
        "outcome_fields_available": {
            "type": "array",
            "items": obj(
                {
                    "name": {"type": "string", "pattern": "^[a-z0-9_]+$"},
                    "unit": {"type": ["string", "null"]},
                    "source": _source_schema(),
                }
            ),
        },
        "labels": obj(
            {
                "flame_type": _value_schema(enum=FLAME_TYPE, units=[]),
                "studied": _labelled(STUDIED),
                "fire_safety_need": _labelled(FIRE_SAFETY_NEED),
            }
        ),
        "map": obj(
            {
                "x_o2": {"type": ["number", "null"]},
                "x_o2_role": {"enum": O2_ROLE + [None]},
                "pressure_pa": {"type": ["number", "null"]},
                "crew_atmospheres": {"type": "array", "items": {"enum": CREW_ATMOSPHERE}},
                "crew_atmospheres_o2_only": {"type": "array", "items": {"enum": CREW_ATMOSPHERE}},
            }
        ),
        "notes": {"type": "array", "items": {"type": "string"}},
    }
    assert list(props) == ROW_KEYS
    return {
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": SCHEMA_ID,
        "title": "One row of the test-level atlas of NASA PSI combustion investigations",
        "type": "object",
        "additionalProperties": False,
        "required": ROW_KEYS,
        "properties": props,
    }


OUTCOME_ROW_SCHEMA = {
    "$schema": "https://json-schema.org/draft/2020-12/schema",
    "$id": "ignis.atlas.test_outcomes/1",
    "type": "object",
    "additionalProperties": False,
    "required": ["schema", "atlas_id", "investigation_id", "test_id", "outcomes"],
    "properties": {
        "schema": {"const": "ignis.atlas.test_outcomes/1"},
        "atlas_id": {"type": "string"},
        "investigation_id": {"type": "string"},
        "test_id": {"type": "string"},
        "outcomes": {
            "type": "array",
            "minItems": 1,
            "items": {
                "type": "object",
                "additionalProperties": False,
                "required": ["name", "raw_value", "raw_unit", "source"],
                "properties": {
                    "name": {"type": "string", "pattern": "^[a-z0-9_]+$"},
                    "raw_value": {"type": ["string", "number"]},
                    "raw_unit": {"type": ["string", "null"]},
                    "source": _source_schema(),
                },
            },
        },
    },
}


def value_problems(path, v):
    """Consistency rules a JSON Schema cannot express compactly."""
    out = []
    st = v["status"]
    if st == "resolved":
        if v["value_si"] is None:
            out.append(f"{path}: resolved without value_si")
        if v["source"] is None:
            out.append(f"{path}: resolved without source")
        if v["basis"] is None:
            out.append(f"{path}: resolved without basis")
        if v["set"] is not None:
            out.append(f"{path}: resolved with a set")
    else:
        if v["value_si"] is not None:
            out.append(f"{path}: {st} with a value_si")
        if not v["reason"]:
            out.append(f"{path}: {st} without a reason")
    if st == "multiple" and v["set"] is None:
        out.append(f"{path}: multiple without a set")
    if st != "multiple" and v["set"] is not None:
        out.append(f"{path}: set on status {st}")
    if st == "multiple" and (v["source"] is None or v["basis"] is None):
        out.append(f"{path}: multiple without source or basis")
    if isinstance(v["value_si"], float) and v["value_si"] != v["value_si"]:
        out.append(f"{path}: NaN")
    if v["alternatives"] and "conflict" not in v["flags"] and "malformed_in_table" not in v["flags"] and "blank_in_table" not in v["flags"]:
        out.append(f"{path}: alternatives without a conflict, malformed or blank flag")
    return out


def walk_values(row):
    """Yield (path, value object) for every value object of a row."""
    def is_value(x):
        return isinstance(x, dict) and set(x.keys()) == set(VALUE_KEYS)

    def rec(path, x):
        if is_value(x):
            yield path, x
        elif isinstance(x, dict):
            for k in x:
                yield from rec(f"{path}.{k}" if path else k, x[k])
        elif isinstance(x, list):
            for i, y in enumerate(x):
                yield from rec(f"{path}[{i}]", y)

    yield from rec("", row)
