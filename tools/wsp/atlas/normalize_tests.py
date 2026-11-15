#!/usr/bin/env python3
"""Test-level atlas of NASA's microgravity combustion investigations (PSI), one common schema.

Reads the experiment tables of the 15 investigations under runs/dev/data_all/ and, for the four
investigations already in the project, the CONDITIONS of train and validation runs through
tools/wsp/analysis/lib.py. Writes, under runs/dev/data_all/work/atlas/ (or --out):

  tests.jsonl           one row per test / test point / test day / matrix row; conditions,
                        identity, hardware, provenance, group id, NAMES of outcome fields only
  tests_outcomes.jsonl  the outcome VALUES of the 15 new investigations, keyed by atlas_id,
                        kept apart until the evaluation groups are frozen
  tests_summary.json    counts, coverage, table defects, input hashes

Schema: docs/wsp/atlas/SCHEMA.md (closed lists in atlas_schema.py). Per-investigation notes:
docs/wsp/atlas/INVESTIGATIONS.md. Nothing is guessed: a value that is not in a source is left
empty with a reason. No network, no randomness, no clock: two runs give identical bytes.

Usage (repository root):  python tools/wsp/atlas/normalize_tests.py [--out DIR]
"""
import argparse
import csv
import datetime
import hashlib
import json
import re
import sys
from collections import Counter, OrderedDict
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
sys.path.insert(0, str(HERE))
import atlas_schema as S  # noqa: E402

NEW = ROOT / "runs/dev/data_all"
SAMPLES = NEW / "raw/samples"
META = SAMPLES / "_meta"
NEW_MANIFESTS = NEW / "manifests"
OLD_MANIFESTS = ROOT / "data/manifests"
TRANSCRIBED = HERE / "transcribed"

NEW_IDS = [10, 20, 21, 22, 23, 39, 47, 68, 98, 99, 100, 101, 102, 106, 107]
OLD_IDS = [25, 26, 69, 159]

ATM = 101325.0
PSI = 6894.757293168
BAR = 1.0e5
SCCM = 1.0e-6 / 60.0  # standard cm3/min -> m3/s (standard conditions of the source)
SLPM = 1.0e-3 / 60.0

# Crew atmospheres (SCHEMA.md, rule CREW). Pressures in psi, oxygen as mole fraction.
CREW_BANDS = OrderedDict(
    [
        ("earth_iss", {"o2": (0.20, 0.22), "p_psi": (14.2, 15.2)}),
        ("moon_base_rover", {"o2": (0.30, 1.0), "p_psi": (0.0, 10.2)}),
        ("spacesuit", {"o2": (0.99, 1.0), "p_psi": (3.8, 4.8)}),
    ]
)

SPECIES = {
    "CH4": "methane",
    "C2H4": "ethylene",
    "C2H6": "ethane",
    "C3H8": "propane",
    "C3H6": "propylene",
    "C4H10": "butane",
    "H2": "hydrogen",
    "N2": "nitrogen",
    "He": "helium",
    "CO2": "carbon_dioxide",
    "O2": "oxygen",
    "C12H26": "dodecane",
}
GAS_FUELS = {"methane", "ethylene", "ethane", "propane", "propylene", "butane", "hydrogen"}
INERTS = {"nitrogen", "helium", "carbon_dioxide"}
MONTHS = {
    "jan": 1, "feb": 2, "mar": 3, "march": 3, "apr": 4, "april": 4, "may": 5, "jun": 6, "june": 6,
    "jul": 7, "july": 7, "aug": 8, "sep": 9, "sept": 9, "oct": 10, "nov": 11, "dec": 12,
}

FLAME_TYPE_OF = {
    "spherical_porous_burner": "spherical_diffusion_flame",
    "flat_porous_burner": "flat_burner_diffusion_flame",
    "gas_jet_burner": "jet_diffusion_flame",
    "coflow_burner": "coflow_jet_diffusion_flame",
    "droplet_free": "droplet_flame",
    "droplet_fiber_supported": "droplet_flame",
    "droplet_unspecified": "droplet_flame",
    "flat_sample": "solid_surface_flame",
    "heated_wire_sample": "overheat_smoke",
    "aerosol_bag": "none",
}
STUDIED_RULES = [
    ("cool_flame", r"cool flame"),
    ("smoke_point", r"smoke point"),
    ("extinction", r"extinguish|suppress"),
    ("electric_field", r"electric"),
    ("liftoff", r"lift-?off"),
    ("burning_rate", r"burning rate"),
    ("aerosol", r"aerosol"),
]


# ----------------------------------------------------------------------------- small helpers
def sig(x):
    """Round to 12 significant digits so unit conversions print the same on every platform."""
    return float(f"{float(x):.12g}")


def num(s):
    if isinstance(s, (int, float)) and not isinstance(s, bool):
        return float(s)
    if not isinstance(s, str):
        return None
    t = s.strip()
    if re.fullmatch(r"[-+]?(\d+\.?\d*|\.\d+)([eE][-+]?\d+)?", t):
        return float(t)
    return None


def src(file, column=None, row=None, sheet=None, page=None, field=None, note=None):
    return {"file": file, "sheet": sheet, "column": column, "row": row, "page": page, "field": field, "note": note}


def _v(status, value=None, unit=None, raw=None, raw_unit=None, basis=None, scope="row", source=None,
       rule=None, flags=(), reason=None, set_=None, alternatives=()):
    if isinstance(value, float):
        value = sig(value)
    if set_ is not None:
        set_ = {"kind": set_["kind"], "values": [sig(x) if isinstance(x, float) else x for x in set_["values"]]}
    return {
        "status": status,
        "value_si": value,
        "unit_si": unit,
        "raw_value": raw,
        "raw_unit": raw_unit,
        "basis": basis,
        "scope": scope,
        "source": source,
        "rule": rule,
        "flags": sorted(set(flags)),
        "reason": reason,
        "set": set_,
        "alternatives": list(alternatives),
    }


def res(value, unit, raw, raw_unit, basis, source, rule=None, flags=(), scope="row", alternatives=()):
    return _v("resolved", value, unit, raw, raw_unit, basis, scope, source, rule, flags, None, None, alternatives)


def multi(kind, values, unit, raw, raw_unit, basis, source, reason, rule=None, flags=(), scope="row", alternatives=()):
    return _v("multiple", None, unit, raw, raw_unit, basis, scope, source, rule, flags, reason,
              {"kind": kind, "values": list(values)}, alternatives)


def unk(reason, raw=None, raw_unit=None, source=None, flags=(), alternatives=(), unit=None):
    return _v("unknown", None, unit, raw, raw_unit, None, "row", source, None, flags, reason, None, alternatives)


def na(reason):
    return _v("not_applicable", None, None, None, None, None, "row", None, None, (), reason)


def varying(raw, source, reason, unit=None, raw_unit=None, flags=()):
    return _v("varying", None, unit, raw, raw_unit, None, "row", source, None, flags, reason)


def withheld(reason, source, unit=None):
    return _v("withheld_outcome", None, unit, None, None, None, "row", source, None, (), reason)


def alt(value, raw, source):
    return {"value_si": sig(value) if isinstance(value, float) else value, "raw_value": raw, "source": source}


def cell(raw, unit, factor, raw_unit, basis, source, flags=(), offset=0.0):
    """A numeric table cell -> value object; labels such as RAMP are kept as what they are."""
    x = num(raw)
    if x is not None:
        f = list(flags)
        if x < 0:
            f.append("negative_value")
        return res(x * factor + offset, unit, x, raw_unit, basis, source, flags=f)
    t = (raw or "").strip() if isinstance(raw, str) else ""
    if t == "":
        return unk("blank cell in the source", source=source, unit=unit)
    if t.upper() in ("RAMP", "RAMPED"):
        return varying(t, source, f"source cell is '{t}': the quantity was ramped during the test, the table gives no value",
                       unit=unit, raw_unit=raw_unit)
    return unk(f"source cell is the label '{t}', not a number", raw=t, raw_unit=raw_unit, source=source, unit=unit)


def text(value, raw, basis, source, rule=None, flags=(), scope="row", alternatives=()):
    return res(value, None, raw, None, basis, source, rule=rule, flags=flags, scope=scope, alternatives=alternatives)


def slug(header):
    h = header.strip().replace("?-g", "ug ")
    return re.sub(r"[^a-z0-9]+", "_", h.lower()).strip("_")


KNOWN_UNITS = {"mm", "mm^2/s", "cm", "cm/s", "atm", "K", "s", "sec", "W", "mg/s", "mg/min", "minutes", "sccm", "percent"}


def header_unit(header):
    """Unit written in parentheses at the end of a column header, when it is a unit."""
    m = re.search(r"\(\s*([^()]+?)\s*\)\s*$", header.strip())
    if m and m.group(1) in KNOWN_UNITS:
        return m.group(1)
    return None


def read_csv(path):
    with open(path, encoding="utf-8-sig", errors="replace", newline="") as f:
        return list(csv.reader(f))


def read_transcription(name):
    with open(TRANSCRIBED / name, encoding="utf-8", newline="") as f:
        lines = [ln for ln in f if not ln.startswith("#")]
    return list(csv.DictReader(lines))


def sha256(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def date_from_yyddd(code):
    """'19115' -> date(2019, 4, 25). Rule D1: YY is the year after 2000, DDD the day of year (GMT)."""
    if not re.fullmatch(r"\d{5}", code):
        return None
    ddd = int(code[2:])
    year = 2000 + int(code[:2])
    if not 1 <= ddd <= 366:
        return None
    d = datetime.date(year, 1, 1) + datetime.timedelta(days=ddd - 1)
    return d if d.year == year else None


def yyddd_of(d):
    return f"{d.year % 100:02d}{d.timetuple().tm_yday:03d}"


def month_number(s):
    t = str(s).strip().rstrip(".").lower()
    if t.isdigit():
        return int(t)
    return MONTHS.get(t)


def species_of(token):
    t = token.strip()
    if t in SPECIES:
        return SPECIES[t]
    return re.sub(r"[^a-z0-9]+", "_", t.lower()).strip("_")


def fill_series(labels):
    """Rule FILL: a spreadsheet fill-series artefact is a run of consecutive cells whose text is the
    first cell's text with its trailing integer increased by 1 per row ('40% CH4', '40% CH5', ...).
    Every cell of such a run is read as the first cell. Returns [(text, was_artefact)]."""
    out = []
    start = None  # (index, prefix, n)
    for i, lab in enumerate(labels):
        t = lab.strip()
        m = re.fullmatch(r"(.*?)(\d+)", t)
        if m and start is not None and m.group(1) == start[1] and int(m.group(2)) == start[2] + (i - start[0]):
            out.append((labels[start[0]].strip(), True))
            continue
        start = (i, m.group(1), int(m.group(2))) if m else None
        out.append((t, False))
    return out


# ----------------------------------------------------------------------------- investigation facts
class Investigation:
    def __init__(self, n):
        self.id = f"PSI-{n}"
        self.new = n in NEW_IDS
        mdir = NEW_MANIFESTS if self.new else OLD_MANIFESTS
        self.manifest_path = mdir / f"{self.id}.investigation.json"
        self.manifest_file = self.manifest_path.name
        m = json.loads(self.manifest_path.read_text(encoding="utf-8"))
        self.acronym = m["acronym"]
        self.title = m["title"]
        self.hardware = m.get("hardware") or ""
        self.meta = {}
        self.meta_file = None
        if self.new:
            p = META / f"{self.id}.json"
            self.meta_file = f"_meta/{p.name}"
            self.meta = json.loads(p.read_text(encoding="utf-8"))
        self.listing_path = mdir / f"{self.id}.listing.jsonl"
        self._days = None

    def listing_days(self):
        """Test days (YYDDD) that appear in raw-data file names of the listing."""
        if self._days is None:
            days = set()
            with open(self.listing_path, encoding="utf-8") as f:
                for line in f:
                    r = json.loads(line)
                    m = re.search(r"(?<!\d)(\d{5})[A-Z]{1,2}\d", r["file_name"])
                    if m:
                        days.add(m.group(1))
            self._days = days
        return self._days

    def listing_names(self):
        """File names of the listing (the catalogue of files, not their content)."""
        if getattr(self, "_names", None) is None:
            with open(self.listing_path, encoding="utf-8") as f:
                self._names = [json.loads(line)["file_name"] for line in f]
        return self._names

    # hardware, rules H1-H3
    def rack(self):
        s = src(self.manifest_file, field="hardware")
        h = self.hardware
        for token, pat in (("CIR", r"Combustion Integrated Rack"), ("MSG", r"Microgravity Science Glovebox"), ("Cygnus", r"Cygnus")):
            if re.search(pat, h):
                return res(token, None, h, None, "nominal", s, rule="H1", scope="investigation")
        if self.id in RACK_OVERRIDE:
            token, s2 = RACK_OVERRIDE[self.id]
            return res(token, None, h or None, None, "nominal", s2, rule="H2", scope="investigation")
        return unk("the hardware string of the investigation metadata names no rack or facility", raw=h or None, source=s)

    def insert(self):
        s = src(self.manifest_file, field="hardware")
        h = self.hardware
        for token, pat in (("MDCA", r"Droplet Combustion App"), ("ACME", r"\bACME\b|Advanced Combustion via Microgravity"),
                           ("Saffire", r"(?i)saffire"), ("CSD_SAME", r"Soot Diagnostic")):
            if re.search(pat, h):
                return res(token, None, h, None, "nominal", s, rule="H1", scope="investigation")
        if h.strip():
            return res("own_apparatus", None, h, None, "derived", s, rule="H3", scope="investigation")
        return unk("the hardware string of the investigation metadata is empty", source=s)

    def gravity(self):
        if self.new:
            pt = self.meta.get("projectType")
            s = src(self.meta_file, field="projectType")
            if pt == "Microgravity Investigation":
                return res("microgravity", None, pt, None, "nominal", s, rule="G1", scope="investigation")
            return unk("projectType of the PSI metadata is not 'Microgravity Investigation'", raw=pt, source=s)
        r = self.rack()
        if r["status"] == "resolved" and r["value_si"] in ("CIR", "MSG"):
            return res("microgravity", None, self.hardware, None, "derived", r["source"], rule="G2", scope="investigation")
        return unk("no gravity statement in the investigation metadata on disk")

    def labels(self):
        studied, need = [], []
        ts = src(self.manifest_file, field="title")
        for token, pat in STUDIED_RULES:
            m = re.search(pat, self.title, re.I)
            if m:
                studied.append({"value": token, "rule": "L2", "matched": m.group(0), "source": ts})
        m = re.search(r"extinguish|suppress", self.title, re.I)
        if m:
            need.append({"value": "suppression", "rule": "L3", "matched": m.group(0), "source": ts})
        m = re.search(r"aerosol", self.title, re.I)
        if m:
            need.append({"value": "detection", "rule": "L3", "matched": m.group(0), "source": ts})
        m = re.search(r"flammab\w*", self.title, re.I)
        s = ts
        for field in ("objective", "approach"):
            if not m and self.new:
                m = re.search(r"flammab\w*", self.meta.get(field) or "", re.I)
                s = src(self.meta_file, field=field)
        if m:
            need.append({"value": "prevention", "rule": "L3", "matched": m.group(0), "source": s})
        order = {t: i for i, t in enumerate(S.FIRE_SAFETY_NEED)}
        need.sort(key=lambda x: order[x["value"]])
        return studied, need


RACK_OVERRIDE = {
    "PSI-101": ("MSG", src("PSI-101_Reports_SAME-R_Final Report.pdf", page=2,
                           note="SAME was run in the Microgravity Science Glovebox; SAME-R used the SAME hardware already on orbit (p. 3)")),
    "PSI-102": ("MSG", src("_meta/PSI-102.json", field="approach", note="the crew installed the hardware in the Glovebox")),
}

R_FINAL_10 = "PSI-10_Reports_FinalRpt_ACMEFlameDesign.pdf"
R_FINAL_23 = "PSI-23_Reports_s_Flame_final_report_20210729.pdf"
R_SPICE = "PSI-107_Reports_SPICE_FinalReport.pdf"
R_SAMER = "PSI-101_Reports_SAME-R_Final Report.pdf"
R_SAME = "PSI-102_Reports_SAME_Final Science Report.pdf"


# ----------------------------------------------------------------------------- row skeleton
def blank(inv, test_id, level):
    studied, need = inv.labels()
    return OrderedDict(
        [
            ("schema", S.SCHEMA_ID),
            ("atlas_id", f"{inv.id}:{test_id}"),
            ("investigation_id", inv.id),
            ("acronym", inv.acronym),
            ("test_id", test_id),
            ("aliases", []),
            ("level", level),
            ("parent_test_id", None),
            ("project_partition", "outside_partitions"),
            ("group_id", None),
            ("group_rule", None),
            ("source_rows", []),
            ("date", unk("no date in the source read", unit="date")),
            ("start_time", unk("no time of day in the source read", unit="time")),
            ("hardware", OrderedDict([("rack", inv.rack()), ("insert", inv.insert()), ("raw", inv.hardware or None)])),
            ("fuel", OrderedDict([("name", unk("not in the source read")), ("components", []), ("phase", unk("not derivable: fuel and configuration unknown"))])),
            ("geometry", OrderedDict([("configuration", unk("not in the source read")), ("flame_orientation", na("not a burner flame")), ("sizes", [])])),
            ("atmosphere", OrderedDict([
                ("x_o2", unk("not in the source read", unit="1")),
                ("pressure", unk("not in the source read", unit="Pa")),
                ("temperature", unk("not in the source read", unit="K")),
                ("diluents", OrderedDict([("status", "unknown"), ("reason", "not in the source read"), ("items", [])])),
                ("x_fuel_ambient", na("fuel is not part of the ambient gas")),
            ])),
            ("oxidizer_stream", OrderedDict([("x_o2", unk("ambient oxygen unknown", unit="1"))])),
            ("flow", OrderedDict([("speed", unk("not in the source read", unit="m/s")), ("direction", unk("not in the source read"))])),
            ("injection", OrderedDict([
                ("x_fuel_injected", na("no gas is injected through a burner")),
                ("fuel_mass_flow", na("no gas is injected through a burner")),
                ("oxidizer_mass_flow", na("no oxidiser is injected through a burner")),
                ("inert_mass_flow", na("no gas is injected through a burner")),
                ("total_mass_flow", na("no gas is injected through a burner")),
            ])),
            ("gravity", inv.gravity()),
            ("extra_conditions", OrderedDict()),
            ("varied", []),
            ("measurements", OrderedDict([("status", "not_filled"), ("items", [])])),
            ("outcome_status", "none_in_source"),
            ("outcome_fields_available", []),
            ("labels", OrderedDict([("flame_type", unk("configuration unknown")), ("studied", studied), ("fire_safety_need", need)])),
            ("map", None),
            ("notes", []),
        ]
    )


def burner_injection_defaults(row, why):
    for k in ("x_fuel_injected", "fuel_mass_flow", "inert_mass_flow", "total_mass_flow"):
        row["injection"][k] = unk(why, unit="1" if k == "x_fuel_injected" else "kg/s")


def set_fuel_gas_phase(row):
    """Rules P1-P4 of SCHEMA.md."""
    cfg = row["geometry"]["configuration"]
    c = cfg["value_si"]
    name = row["fuel"]["name"]
    fuels = [x["species"] for x in row["fuel"]["components"] if x["role"] == "fuel"]
    ins = row["hardware"]["insert"]
    if c == "aerosol_bag":
        row["fuel"]["phase"] = na("no fuel is burned (aerosol instrument test)")
    elif c in ("flat_sample", "heated_wire_sample"):
        row["fuel"]["phase"] = res("solid", None, None, None, "derived", cfg["source"], rule="P3", scope=cfg["scope"])
    elif c and c.startswith("droplet"):
        row["fuel"]["phase"] = res("liquid_droplet", None, None, None, "derived", cfg["source"], rule="P1", scope=cfg["scope"])
    elif fuels and all(f in GAS_FUELS for f in fuels):
        row["fuel"]["phase"] = res("gas", None, None, None, "derived", name["source"], rule="P2")
    elif ins["value_si"] == "MDCA":
        row["fuel"]["phase"] = res("liquid_droplet", None, None, None, "derived", ins["source"], rule="P1", scope="investigation")


def finish(row):
    """Derived fields: flame type (L1), oxidiser side, map and crew atmospheres (CREW)."""
    cfg = row["geometry"]["configuration"]
    if cfg["status"] == "resolved":
        row["labels"]["flame_type"] = res(FLAME_TYPE_OF[cfg["value_si"]], None, None, None, "derived", cfg["source"],
                                          rule="L1", scope=cfg["scope"])
    amb = row["atmosphere"]["x_o2"]
    ox = row["oxidizer_stream"]["x_o2"]
    if amb["status"] != "not_applicable" and ox["status"] == "unknown" and ox["source"] is None:
        # rule OX1: when nothing else is stated the oxidiser side is the ambient
        row["oxidizer_stream"]["x_o2"] = dict(amb, rule="OX1") if amb["status"] in ("resolved", "multiple") else amb
        ox = row["oxidizer_stream"]["x_o2"]
    m = OrderedDict([("x_o2", None), ("x_o2_role", None), ("pressure_pa", None), ("crew_atmospheres", []), ("crew_atmospheres_o2_only", [])])
    if amb["status"] == "resolved":
        m["x_o2"], m["x_o2_role"] = amb["value_si"], "ambient"
    elif amb["status"] == "not_applicable" and ox["status"] == "resolved":
        m["x_o2"], m["x_o2_role"] = ox["value_si"], "injected"
    p = row["atmosphere"]["pressure"]
    if p["status"] == "resolved":
        m["pressure_pa"] = p["value_si"]
    if m["x_o2_role"] == "ambient":
        for name, band in CREW_BANDS.items():
            lo, hi = band["o2"]
            if not (lo - 1e-9 <= m["x_o2"] <= hi + 1e-9):
                continue
            if m["pressure_pa"] is None:
                m["crew_atmospheres_o2_only"].append(name)
                continue
            plo, phi = band["p_psi"]
            if plo - 1e-9 <= m["pressure_pa"] / PSI <= phi + 1e-9:
                m["crew_atmospheres"].append(name)
    row["map"] = m
    return row


def outcome_field(header, file, sheet=None, name=None):
    return {"name": name or slug(header), "unit": header_unit(header), "source": src(file, column=header, sheet=sheet)}


class Out:
    """Collects rows, outcome values and defects of one run."""

    def __init__(self):
        self.rows = []
        self.outcomes = []
        self.defects = OrderedDict()
        self.inputs = OrderedDict()
        self.skipped = OrderedDict()
        self.outcome_cells = OrderedDict()

    def defect(self, inv_id, textline):
        self.defects.setdefault(inv_id, []).append(textline)

    def use(self, path):
        rel = path.resolve().relative_to(ROOT).as_posix()
        if rel not in self.inputs:
            self.inputs[rel] = sha256(path)

    def add(self, row, outcome_items=()):
        self.rows.append(finish(row))
        items = [o for o in outcome_items if o["raw_value"] not in (None, "")]
        for o in items:
            key = (row["investigation_id"], o["name"])
            self.outcome_cells[key] = self.outcome_cells.get(key, 0) + 1
        if items:
            self.outcomes.append(OrderedDict([
                ("schema", "ignis.atlas.test_outcomes/1"),
                ("atlas_id", row["atlas_id"]),
                ("investigation_id", row["investigation_id"]),
                ("test_id", row["test_id"]),
                ("outcomes", items),
            ]))


def outcome_item(header, raw, file, row, sheet=None, name=None):
    if isinstance(raw, str):
        raw = raw.strip()
    if isinstance(raw, float):
        raw = sig(raw)
    return {"name": name or slug(header), "raw_value": raw, "raw_unit": header_unit(header),
            "source": src(file, column=header, row=row, sheet=sheet)}


def table_path(n, pattern):
    hits = sorted((SAMPLES / f"PSI-{n}").glob(pattern))
    if len(hits) != 1:
        raise SystemExit(f"expected exactly one file for PSI-{n} {pattern!r}, found {len(hits)}")
    return hits[0]


def day_group(row, inv, key, rule):
    row["group_id"] = f"{inv.id}:{key}"
    row["group_rule"] = rule


def set_date(row, d, raw, source, basis, rule=None, flags=(), alternatives=()):
    row["date"] = res(d.isoformat(), "date", raw, None, basis, source, rule=rule, flags=flags, alternatives=alternatives)


def burner_size(row, mm, source, scope="investigation", raw=None, basis="nominal", flags=(), alternatives=()):
    row["geometry"]["sizes"].append(OrderedDict([
        ("dimension", "burner_diameter"),
        ("value", res(mm * 1e-3, "m", mm if raw is None else raw, "mm", basis, source, scope=scope, flags=flags, alternatives=alternatives)),
    ]))


def component(species, role, fraction):
    return OrderedDict([("species", species), ("role", role), ("fraction", fraction)])


def balance_fraction(parts, source, note):
    """Rule BAL: a stream of named components whose listed fractions leave a remainder; the last
    named gas is the balance."""
    if all(p["status"] == "resolved" for p in parts):
        x = 1.0 - sum(p["value_si"] for p in parts)
        return res(max(x, 0.0), "1", None, None, "derived", dict(source, note=note), rule="BAL", flags=["balance_gas"])
    st = sorted({p["status"] for p in parts if p["status"] != "resolved"})
    return unk(f"balance of a fraction that is {'/'.join(st)} in the source", unit="1")


# ----------------------------------------------------------------------------- PSI-10 Flame Design
def psi10(out):
    inv = Investigation(10)
    fin = src(R_FINAL_10, page=10, note="the spherical burners were limited to a single diameter of 6.35 mm")
    out_n = ["XO2 (final)", "pfinal (atm)", "pmean (atm)", "mean Zst", "Tad (K)"]
    out_i = ["XC2H4 final", "pfinal (atm)", "pmean (atm)", "mean Zst", "Tad (K)"]
    for kind, pattern, outs in (("normal", "*Normal Flame.csv", out_n), ("inverse", "*Inverse Flame.csv", out_i)):
        path = table_path(10, pattern)
        out.use(path)
        fn = path.name
        rows = read_csv(path)
        H = [h.strip() for h in rows[0]]
        seen = set()
        by_day = {}
        for r in rows[1:]:
            c = dict(zip(H, r))
            by_day.setdefault(c["Day"].strip(), Counter())[c["Test"].strip()[:5]] += 1
        day_code = {k: v.most_common(1)[0][0] for k, v in by_day.items()}
        listed = {m.group(1) for n in inv.listing_names() for m in [re.search(r"(?<!\d)(\d{5}[A-Z]{1,2}\d{1,2})(?!\d)", n)] if m}
        table_names = {r[H.index("Test")].strip() for r in rows[1:]}
        for i, r in enumerate(rows[1:], start=2):
            c = dict(zip(H, r))
            test = c["Test"].strip()
            if test in seen:
                out.defect(inv.id, f"{fn}: test name {test} appears twice (row {i})")
                continue
            seen.add(test)
            s = lambda col: src(fn, column=col, row=i)
            row = blank(inv, test, "test")
            row["source_rows"] = [{"file": fn, "sheet": None, "row": i}]
            row["aliases"].append({"scheme": "campaign_day", "name": f"{kind} day {c['Day'].strip()}"})
            row["extra_conditions"]["campaign_day"] = text(f"{kind} day {c['Day'].strip()}", c["Day"].strip(), "nominal", s("Day"))
            code = day_code[c["Day"].strip()]
            d = date_from_yyddd(code)
            d_name = date_from_yyddd(test[:5])
            if d and test[:5] == code:
                set_date(row, d, test[:5], s("Test"), "derived", rule="D1")
                day_group(row, inv, f"day:{code}", "GR-DAY")
            elif d:
                # rule DAYCOL: the name's day code disagrees with the table's Day column
                set_date(row, d, c["Day"].strip(), dict(s("Day"), note=f"day code {code} of the other tests of this Day index"), "derived",
                         rule="DAYCOL", flags=["conflict"],
                         alternatives=[alt(d_name.isoformat() if d_name else None, test[:5], s("Test"))])
                day_group(row, inv, f"day:{code}", "GR-DAY")
                fixed = code + test[5:]
                in_listing = fixed in listed and test not in listed and fixed not in table_names
                if in_listing:
                    row["aliases"].append({"scheme": "listing_test_name", "name": fixed})
                out.defect(inv.id, f"{fn}: test {test} (row {i}) is in Day {c['Day'].strip()} with the tests of day {code}; no file in the listing "
                                   f"is named {test}" + (f", and files named {fixed} exist while the table has no {fixed}: the name is a typing error" if in_listing else ""))
                row["notes"].append(f"the table's test name {test} disagrees with its Day column (day {code})")
            else:
                out.defect(inv.id, f"{fn}: test name {test} does not start with YYDDD (row {i})")
                day_group(row, inv, f"test:{test}", "GR-ROW")
            row["geometry"]["configuration"] = res("spherical_porous_burner", None, None, None, "nominal", fin, scope="investigation")
            row["geometry"]["flame_orientation"] = text(kind, c["Flame Type"].strip(), "nominal", s("Flame Type"))
            burner_size(row, 6.35, fin)
            row["flow"]["speed"] = na("no ambient flow is imposed in the table; the reactant flow through the burner is under `injection`")
            row["flow"]["direction"] = res("burner_injection", None, None, None, "derived", s("Flame Type"), rule="F1")
            row["atmosphere"]["pressure"] = cell(c["pinit (atm)"], "Pa", ATM, "atm", "unstated", s("pinit (atm)"))
            row["atmosphere"]["diluents"] = OrderedDict([("status", "unknown"), ("reason", "the table does not name the balance gas of the chamber"), ("items", [])])
            inj = row["injection"]
            if kind == "normal":
                sp = species_of(c["Fuel"])
                row["fuel"]["name"] = text(sp, c["Fuel"].strip(), "nominal", s("Fuel"))
                xf = cell(c["Xfuel"], "1", 1.0, "1", "unstated", s("Xfuel"))
                row["fuel"]["components"] = [
                    component(sp, "fuel", xf),
                    component("nitrogen", "diluent", balance_fraction([xf], s("mN2 (mg/s)"), "the only other injected gas of the table is N2")),
                ]
                inj["x_fuel_injected"] = xf
                inj["fuel_mass_flow"] = cell(c["mfuel (mg/s)"], "kg/s", 1e-6, "mg/s", "unstated", s("mfuel (mg/s)"))
                inj["inert_mass_flow"] = cell(c["mN2 (mg/s)"], "kg/s", 1e-6, "mg/s", "unstated", s("mN2 (mg/s)"))
                inj["total_mass_flow"] = cell(c["mtot (mg/s)"], "kg/s", 1e-6, "mg/s", "unstated", s("mtot (mg/s)"))
                row["atmosphere"]["x_o2"] = cell(c["XO2 (init)"], "1", 1.0, "1", "unstated", s("XO2 (init)"))
            else:
                hs = src(fn, column="XC2H4 initial", note="fuel named only by the column header")
                row["fuel"]["name"] = text("ethylene", "C2H4", "nominal", hs)
                xfa = cell(c["XC2H4 initial"], "1", 1.0, "1", "unstated", s("XC2H4 initial"))
                row["fuel"]["components"] = [component("ethylene", "fuel", xfa)]
                row["atmosphere"]["x_fuel_ambient"] = xfa
                row["atmosphere"]["x_o2"] = na("inverse flame: the chamber holds the fuel mixture and oxygen is injected (see oxidizer_stream.x_o2)")
                row["oxidizer_stream"]["x_o2"] = cell(c["XO2"], "1", 1.0, "1", "unstated", s("XO2"))
                inj["x_fuel_injected"] = na("inverse flame: the injected stream is the oxidiser")
                inj["fuel_mass_flow"] = na("inverse flame: the injected stream is the oxidiser")
                inj["oxidizer_mass_flow"] = cell(c["mO2 (mg/s)"], "kg/s", 1e-6, "mg/s", "unstated", s("mO2 (mg/s)"))
                inj["inert_mass_flow"] = cell(c["mN2 (mg/s)"], "kg/s", 1e-6, "mg/s", "unstated", s("mN2 (mg/s)"))
                inj["total_mass_flow"] = cell(c["mtot (mg/s)"], "kg/s", 1e-6, "mg/s", "unstated", s("mtot (mg/s)"))
            set_fuel_gas_phase(row)
            row["outcome_status"] = "listed"
            row["outcome_fields_available"] = [outcome_field(h, fn) for h in outs]
            out.add(row, [outcome_item(h, c[h], fn, i) for h in outs])


# ----------------------------------------------------------------------------- PSI-39 CFI
def clock(raw):
    """'02:03:08 pm est' -> ('14:03:08', 'est'); None when the cell is not a complete time."""
    m = re.fullmatch(r"(\d{1,2}):(\d{2})(?::(\d{2}))?\s*(am|pm)\s*(\w+)?", raw.strip().lower())
    if not m:
        return None
    h, mi, sec, ap, tz = int(m.group(1)), m.group(2), m.group(3), m.group(4), m.group(5)
    if not 1 <= h <= 12:
        return None
    h = (h % 12) + (12 if ap == "pm" else 0)
    return f"{h:02d}:{mi}" + (f":{sec}" if sec else ""), tz


def psi39(out):
    import openpyxl

    inv = Investigation(39)
    xpath = table_path(39, "*CFI_DataSummary_Spreadsheet_Jan2021.xlsx")
    cpath = table_path(39, "*Experimental table_CFI.csv")
    out.use(xpath)
    out.use(cpath)
    xf, cf = xpath.name, cpath.name
    wb = openpyxl.load_workbook(xpath, data_only=True, read_only=True)

    def sheet(name):
        rows = [list(r) for r in wb[name].iter_rows(values_only=True)]
        head = [("" if h is None else str(h)) for h in rows[0]]
        body = OrderedDict()
        for i, r in enumerate(rows[1:], start=2):
            if r[0] is None or str(r[0]).strip() == "":
                continue
            key = str(int(r[0])) if isinstance(r[0], float) and r[0].is_integer() else str(r[0]).strip()
            if key in body:
                out.defect(inv.id, f"{xf} sheet '{name}': Flex Number {key} appears twice (row {i})")
                continue
            body[key] = (i, dict(zip(head, r)))
        return head, body

    mh, master = sheet("Master Sheet")
    dh, data = sheet("DATA")
    ih, ign = sheet("Igniter")
    ch, com = sheet("Comments")

    csv_rows = read_csv(cpath)
    CH = [h.strip() for h in csv_rows[0]]
    csv_by = OrderedDict()
    dup = 0
    seen_rows = set()
    for i, r in enumerate(csv_rows[1:], start=2):
        if tuple(r) in seen_rows:
            dup += 1
        seen_rows.add(tuple(r))
        csv_by.setdefault(r[0].strip(), []).append((i, dict(zip(CH, [x.strip() for x in r]))))
    if dup:
        out.defect(inv.id, f"{cf}: {dup} rows are exact repeats of an earlier row "
                           f"({len(csv_rows) - 1} rows, {len(csv_by)} distinct Flex Numbers)")
    missing = [k for k in csv_by if k not in master]
    if missing:
        out.defect(inv.id, f"{cf}: Flex Numbers absent from the spreadsheet Master Sheet: {', '.join(missing)}")

    data_out = [h for h in dh if h and h.strip() not in ("Flex Number", "D0 ( mm )", "Pressure (atm)")]
    com_col = ch[1]
    fields = [outcome_field(h, xf, sheet="DATA") for h in data_out] + [outcome_field(com_col, xf, sheet="Comments", name="comments")]
    sum_bad, csv_diff = [], []

    for key, (i, c) in master.items():
        g = lambda col: c.get(col)
        s = lambda col: src(xf, column=col.strip(), row=i, sheet="Master Sheet")
        row = blank(inv, key, "test")
        row["source_rows"] = [{"file": xf, "sheet": "Master Sheet", "row": i}]
        cir = str(g("CIR ") or "").strip()
        row["aliases"].append({"scheme": "file_flex_number", "name": f"FLEX-{key}"})
        if cir:
            row["aliases"].append({"scheme": "cir_test_point", "name": cir})
            row["aliases"].append({"scheme": "cir_test_point_upper", "name": cir.upper()})
        td = g("Test Date")
        if isinstance(td, datetime.datetime):
            td = td.date()
        if isinstance(td, datetime.date):
            cm = re.fullmatch(r"[a-z](\d{3})[a-z]\d+", cir)
            cd = (datetime.date(td.year, 1, 1) + datetime.timedelta(days=int(cm.group(1)) - 1)) if cm else None
            if cd and cd != td:
                set_date(row, td, td.isoformat(), s("Test Date"), "nominal", flags=["conflict"],
                         alternatives=[alt(cd.isoformat(), cir, s("CIR "))])
                out.defect(inv.id, f"{xf} Master Sheet: Flex {key} is dated {td.isoformat()} but its CIR test point {cir} carries day {cm.group(1)} "
                                   f"({cd.isoformat()}), the day of the neighbouring tests; the group follows the CIR day")
                day_group(row, inv, f"day:{cd.isoformat()}", "GR-CIRDAY")
            elif cd:
                set_date(row, td, td.isoformat(), s("Test Date"), "nominal")
                day_group(row, inv, f"day:{cd.isoformat()}", "GR-CIRDAY")
            else:
                set_date(row, td, td.isoformat(), s("Test Date"), "nominal")
                day_group(row, inv, f"day:{td.isoformat()}", "GR-DAY")
        else:
            row["date"] = unk("the Test Date cell is not a date", raw=None if td is None else str(td), source=s("Test Date"), unit="date")
            day_group(row, inv, f"test:{key}", "GR-ROW")
        lt = str(g("Local Time") or "").strip()
        ck = clock(lt) if lt else None
        if ck:
            row["start_time"] = res(ck[0], "time", lt, ck[1], "nominal", s("Local Time"), rule="T1")
        elif lt:
            row["start_time"] = unk("the Local Time cell is not a complete time of day", raw=lt, source=s("Local Time"), unit="time")
        # fuel
        fraw = str(g("Fuel ") or "").strip()
        m = re.fullmatch(r"\[(\.\d+)\]/\[(\.\d+)\]\s*(\S+)/(\S+)", fraw)
        if m:
            a, b = species_of(m.group(3)), species_of(m.group(4))
            row["fuel"]["name"] = text(f"{a}+{b}", fraw, "nominal", s("Fuel "))
            row["fuel"]["components"] = [
                component(a, "fuel", res(float(m.group(1)), "1", m.group(1), "1", "nominal", s("Fuel "), flags=["basis_of_fraction_unstated"])),
                component(b, "fuel", res(float(m.group(2)), "1", m.group(2), "1", "nominal", s("Fuel "), flags=["basis_of_fraction_unstated"])),
            ]
        elif fraw:
            sp = species_of(fraw)
            row["fuel"]["name"] = text(sp, fraw, "nominal", s("Fuel "))
            row["fuel"]["components"] = [component(sp, "fuel", res(1.0, "1", None, None, "derived", s("Fuel "), rule="PURE"))]
        # geometry
        fib = g("Fiber")
        fib = "" if fib is None else str(fib).strip()
        fs = s("Fiber")
        if fib.lower() == "yes":
            row["geometry"]["configuration"] = res("droplet_fiber_supported", None, fib, None, "nominal", fs)
        elif fib.lower() == "no":
            row["geometry"]["configuration"] = res("droplet_free", None, fib, None, "nominal", fs)
        else:
            row["geometry"]["configuration"] = res("droplet_unspecified", None, fib or None, None, "derived", fs, rule="CFG-DROPLET",
                                                    flags=["blank_in_table"] if not fib else [])
        if fib:
            row["extra_conditions"]["fiber"] = text(fib, fib, "nominal", fs)
        if key in data:
            di, dc = data[key]
            d0 = dc.get("D0 ( mm )")
            if num(d0) is not None:
                row["geometry"]["sizes"].append(OrderedDict([
                    ("dimension", "initial_droplet_diameter"),
                    ("value", res(num(d0) * 1e-3, "m", num(d0), "mm", "measured", src(xf, column="D0 ( mm )", row=di, sheet="DATA"),
                                  flags=["from_analysis_sheet"])),
                ]))
        # atmosphere
        praw = str(g("Pressure") or "").strip()
        pm = re.fullmatch(r"(\d*\.?\d+)\s*atm", praw)
        if pm:
            pv = res(float(pm.group(1)) * ATM, "Pa", float(pm.group(1)), "atm", "nominal", s("Pressure"))
            if key in data:
                di, dc = data[key]
                p2 = num(dc.get("Pressure (atm)"))
                if p2 is not None and abs(p2 - float(pm.group(1))) > 1e-9:
                    pv["flags"] = sorted(set(pv["flags"]) | {"conflict"})
                    pv["alternatives"] = [alt(p2 * ATM, p2, src(xf, column="Pressure (atm)", row=di, sheet="DATA"))]
                    out.defect(inv.id, f"{xf}: Flex {key} pressure is '{praw}' on the Master Sheet and {p2:g} atm on the DATA sheet")
            row["atmosphere"]["pressure"] = pv
        elif praw:
            row["atmosphere"]["pressure"] = unk("the Pressure cell is not '<number> atm'", raw=praw, source=s("Pressure"), unit="Pa")
        fr = OrderedDict()
        for col, sp in (("Oxygen", "oxygen"), ("Nitrogen", "nitrogen"), ("Helium", "helium")):
            fr[sp] = cell(g(col) if g(col) is not None else "", "1", 1.0, "1", "nominal", s(col))
        if all(v["status"] == "resolved" for v in fr.values()):
            tot = sum(v["value_si"] for v in fr.values())
            if abs(tot - 1.0) > 1e-6:
                sum_bad.append(f"{key} ({tot:.2f})")
                for v in fr.values():
                    v["flags"] = sorted(set(v["flags"]) | {"fractions_sum_not_1"})
        row["atmosphere"]["x_o2"] = fr["oxygen"]
        row["atmosphere"]["diluents"] = OrderedDict([
            ("status", "resolved"), ("reason", None),
            ("items", [OrderedDict([("species", "nitrogen"), ("fraction", fr["nitrogen"])]),
                       OrderedDict([("species", "helium"), ("fraction", fr["helium"])])]),
        ])
        if key in ign:
            gi, gc = ign[key]
            pw = re.fullmatch(r"(\d*\.?\d+)\s*w", str(gc.get(ih[1]) or "").strip().lower())
            tm = re.fullmatch(r"(\d*\.?\d+)\s*ms", str(gc.get(ih[2]) or "").strip().lower())
            if pw:
                row["extra_conditions"]["igniter_power"] = res(float(pw.group(1)), "W", float(pw.group(1)), "W", "nominal",
                                                              src(xf, column=ih[1].strip(), row=gi, sheet="Igniter"))
            if tm:
                row["extra_conditions"]["igniter_time"] = res(float(tm.group(1)) * 1e-3, "s", float(tm.group(1)), "ms", "nominal",
                                                             src(xf, column=ih[2].strip(), row=gi, sheet="Igniter"))
        set_fuel_gas_phase(row)
        # the public CSV table, where it has the test
        for ci, cc in csv_by.get(key, []):
            row["source_rows"].append({"file": cf, "sheet": None, "row": ci})
        if key in csv_by:
            ci, cc = csv_by[key][0]
            pairs = [("Pressure", praw), ("Oxygen", g("Oxygen")), ("Nitrogen", g("Nitrogen")), ("Helium", g("Helium")),
                     ("Fiber", fib), ("Fuel", fraw), ("CIR", cir)]
            for col, xv in pairs:
                cv = cc.get(col, "")
                same = (num(cv) == num(xv)) if num(cv) is not None and num(xv) is not None else (str(cv).strip() == str(xv if xv is not None else "").strip())
                if not same:
                    csv_diff.append(f"{key}.{col}")
        row["outcome_status"] = "listed"
        row["outcome_fields_available"] = fields
        items = []
        if key in data:
            di, dc = data[key]
            for h in data_out:
                val = dc.get(h)
                if val is not None and str(val).strip() != "":
                    items.append(outcome_item(h, val if isinstance(val, (int, float)) else str(val), xf, di, sheet="DATA"))
        if key in com:
            mi, mc = com[key]
            val = mc.get(com_col)
            if val is not None and str(val).strip() != "":
                items.append(outcome_item(com_col, str(val), xf, mi, sheet="Comments", name="comments"))
        out.add(row, items)
    if sum_bad:
        out.defect(inv.id, f"{xf} Master Sheet: oxygen + nitrogen + helium is not 1 for Flex {', '.join(sum_bad)}")
    if csv_diff:
        out.defect(inv.id, f"{cf} differs from the spreadsheet Master Sheet in: {', '.join(csv_diff)}")
    out.skipped[inv.id] = {"csv_rows_not_emitted_separately": len(csv_rows) - 1,
                           "reason": "the CSV is a subset of the spreadsheet Master Sheet; its rows are listed in source_rows"}


# ----------------------------------------------------------------------------- PSI-107 SPICE
def psi107(out):
    inv = Investigation(107)
    path = table_path(107, "*Experimental table_Spice.csv")
    out.use(path)
    fn = path.name
    rows = read_csv(path)
    H = rows[0]
    c_day, c_ign, c_off, c_fuel, c_noz, c_sccm, c_mg, c_hc, c_air = H
    year_src = src(R_SPICE, page=19, note="twelve sessions from February 14, 2009 to June 23, 2009")
    n2_src = src(R_SPICE, page=2, note="propylene/nitrogen mixtures")
    first = OrderedDict()
    for i, r in enumerate(rows[1:], start=2):
        first.setdefault(tuple(r), []).append(i)
    ndup = sum(len(v) - 1 for v in first.values())
    if ndup:
        out.defect(inv.id, f"{fn}: {ndup} rows are exact repeats of another row ({len(rows) - 1} rows, {len(first)} distinct); "
                           "each distinct row is emitted once with all its row numbers in source_rows")
    odd_time = 0
    zero_hc = 0
    for r, where in first.items():
        i = where[0]
        c = dict(zip(H, r))
        s = lambda col: src(fn, column=col.strip(), row=i)
        day = int(c[c_day])
        row = blank(inv, f"d{day:03d}-row{i:04d}", "test_point")
        row["source_rows"] = [{"file": fn, "sheet": None, "row": k} for k in where]
        d = datetime.date(2009, 1, 1) + datetime.timedelta(days=day - 1)
        row["date"] = res(d.isoformat(), "date", c[c_day], "GMT day of year", "derived", dict(s(c_day), note="year 2009 from " + R_SPICE + " p. 19"), rule="D2")
        day_group(row, inv, f"gmt:2009-{day:03d}", "GR-DAY")
        ig = c[c_ign].strip()
        m = re.fullmatch(r"(\d{1,2})[:.](\d{2})[:.](\d{2})", ig)
        if m:
            t = f"{int(m.group(1)):02d}:{m.group(2)}:{m.group(3)}"
            row["start_time"] = res(t, "time", ig, "GMT", "measured", s(c_ign), rule="T1")
            row["parent_test_id"] = f"d{day:03d}-{t}"
        else:
            odd_time += 1
            row["start_time"] = unk("the GMT ignition cell is not HH:MM:SS", raw=ig or None, raw_unit="GMT", source=s(c_ign), unit="time")
        fraw = c[c_fuel].strip()
        m = re.fullmatch(r"(\d+)\s*%\s*(\w+)", fraw)
        if m:
            sp = species_of(m.group(2))
            xfv = res(int(m.group(1)) / 100.0, "1", fraw, "%", "nominal", s(c_fuel), flags=["basis_of_fraction_unstated"])
            row["fuel"]["name"] = text(sp, fraw, "nominal", s(c_fuel))
            row["fuel"]["components"] = [component(sp, "fuel", xfv),
                                         component("nitrogen", "diluent", balance_fraction([xfv], n2_src, "propylene/nitrogen mixtures (final report p. 2)"))]
        else:
            sp = species_of(fraw)
            xfv = res(1.0, "1", None, None, "derived", s(c_fuel), rule="PURE")
            row["fuel"]["name"] = text(sp, fraw, "nominal", s(c_fuel))
            row["fuel"]["components"] = [component(sp, "fuel", xfv)]
        row["geometry"]["configuration"] = res("coflow_burner", None, inv.title, None, "nominal",
                                               src(inv.manifest_file, field="title", note="co-flow; nozzle and air rate columns of the table"),
                                               scope="investigation")
        row["geometry"]["flame_orientation"] = res("normal", None, None, None, "derived", s(c_fuel), rule="OR1")
        nz = cell(c[c_noz], "m", 1e-3, "mm", "nominal", s(c_noz))
        row["geometry"]["sizes"].append(OrderedDict([("dimension", "nozzle_inner_diameter"), ("value", nz)]))
        why = ("not in the table; the final report (p. 19) says the glovebox was flushed with ISS cabin air, "
               "oxygen 'maintained within 19 - 24%', and gives no value per test")
        row["atmosphere"]["x_o2"] = unk(why, unit="1")
        row["atmosphere"]["diluents"] = OrderedDict([("status", "unknown"), ("reason", "cabin air; composition per test not in the table"), ("items", [])])
        row["flow"]["speed"] = cell(c[c_air], "m/s", 1e-2, "cm/s", "measured", s(c_air))
        row["flow"]["direction"] = res("coflow", None, c_air.strip(), None, "nominal", src(fn, column=c_air.strip()), scope="investigation")
        inj = row["injection"]
        inj["x_fuel_injected"] = xfv
        inj["fuel_mass_flow"] = cell(c[c_hc], "kg/s", 1e-6, "mg/s", "measured", s(c_hc))
        if not c[c_mg].strip() and not c[c_sccm].strip() and num(c[c_hc]) == 0:
            zero_hc += 1
            inj["fuel_mass_flow"] = unk("the hydrocarbon rate cell is 0 while both fuel rate cells of the row are blank "
                                        "(a spreadsheet formula over empty cells), so no flow is read",
                                        raw=c[c_hc].strip(), raw_unit="mg/s", source=s(c_hc), unit="kg/s", flags=["malformed_in_table"])
        inj["oxidizer_mass_flow"] = na("no oxidiser is injected through the nozzle")
        inj["inert_mass_flow"] = unk("not tabulated; the total and the hydrocarbon mass flows are given", unit="kg/s")
        inj["total_mass_flow"] = cell(c[c_mg], "kg/s", 1e-6, "mg/s", "measured", s(c_mg))
        row["extra_conditions"]["fuel_volume_flow_std"] = cell(c[c_sccm], "m3/s", SCCM, "sccm", "measured", s(c_sccm))
        set_fuel_gas_phase(row)
        row["outcome_status"] = "listed"
        row["outcome_fields_available"] = [outcome_field(c_off, fn, name="gmt_flame_off")]
        out.add(row, [outcome_item(c_off, c[c_off], fn, i, name="gmt_flame_off")])
    if zero_hc:
        out.defect(inv.id, f"{fn}: {zero_hc} distinct rows have blank fuel rate cells and a hydrocarbon rate of 0; the 0 is not read as a flow")
    if odd_time:
        out.defect(inv.id, f"{fn}: {odd_time} distinct rows have a GMT ignition cell that is not a time "
                           "(for example 'no ku', 'Ku LOS', a four-digit number); two separators ':' and '.' are used for times")


# ----------------------------------------------------------------------------- PSI-101 SAME-R, PSI-102 SAME
MATERIALS = {
    "kapton": "kapton", "lampwick": "lampwick", "pyrell": "pyrell", "silicone": "silicone", "teflon": "teflon",
    "silicone rubber": "silicone_rubber", "dibutyl phthalate": "dibutyl_phthalate",
    "sibal fabric": "sibal_fabric", "cotton-fiberglass (sibal)": "sibal_fabric", "pmma": "pmma",
    "pmma & nomex": "pmma+nomex", "pmma to nomex": "pmma+nomex", "structured pmma": "pmma", "flat pmma": "pmma",
}


def material(raw):
    return MATERIALS.get(raw.strip().lower(), re.sub(r"[^a-z0-9]+", "_", raw.strip().lower()).strip("_"))


def heated_wire(row, source):
    row["geometry"]["configuration"] = res("heated_wire_sample", None, None, None, "nominal", source, scope="investigation")


def psi101(out):
    inv = Investigation(101)
    path = table_path(101, "*Experimental table_SAME-R.csv")
    out.use(path)
    fn = path.name
    rows = read_csv(path)
    H = [h.strip() for h in rows[0]]
    wire = src(R_SAMER, page=26, note="the software controls the sample temperature through the resistance of the heating wire")
    ysrc = "year 2010 from " + R_SAMER + " p. 3 and Appendix A header 'Date (mm/dd/2010)'"
    seen = Counter()
    runs = {}  # run number -> {(day of year, material)} from analysed-data file names
    for n in inv.listing_names():
        m = re.search(r"2010_(\d{3})_\d+_([A-Za-z]+)_run(\d+)", n)
        if m:
            runs.setdefault(m.group(3), set()).add((int(m.group(1)), m.group(2).lower()))
    first_date = {}
    for r in rows[1:]:
        c = dict(zip(H, [x.strip() for x in r]))
        first_date.setdefault(c["SAME Test #"], c["Date"])
    off_day = 0
    for i, r in enumerate(rows[1:], start=2):
        c = dict(zip(H, [x.strip() for x in r]))
        s = lambda col: src(fn, column=col, row=i)
        tid = c["SAME Test #"]
        seen[tid] += 1
        dup_flag = []
        if seen[tid] > 1:
            mats = sorted({m for _, m in runs.get(tid, set())})
            out.defect(inv.id, f"{fn}: SAME Test # {tid} appears {seen[tid]} times (row {i}, sample type {c['Sample Type']}); the later row gets "
                               f"the id '{tid}#{seen[tid]}'" + (f"; analysed-data file names have run {tid} only as {', '.join(mats)}" if mats else ""))
            if mats and not any(c["Sample Type"].lower().startswith(m[:4]) for m in mats):
                dup_flag = ["defect_noted"]
            tid = f"{tid}#{seen[tid]}"
        sub = re.fullmatch(r"(\d+)a(#\d+)?", tid)
        row = blank(inv, tid, "test_point" if sub else "test")
        if sub:
            row["parent_test_id"] = sub.group(1)
            row["notes"].append("secondary aging run of the parent run (final report p. 3: secondary aging runs are listed as separate entries)")
        row["source_rows"] = [{"file": fn, "sheet": None, "row": i}]
        m = re.fullmatch(r"(\d{1,2})-([A-Za-z]{3})", c["Date"])
        if m and month_number(m.group(2)):
            d = datetime.date(2010, month_number(m.group(2)), int(m.group(1)))
            set_date(row, d, c["Date"], dict(s("Date"), note=ysrc), "derived", rule="D3")
            day_group(row, inv, f"day:{d.isoformat()}", "GR-DAY")
            pm = re.fullmatch(r"(\d{1,2})-([A-Za-z]{3})", first_date.get(sub.group(1), "")) if sub else None
            if pm and month_number(pm.group(2)):
                # rule GR-PARENT: a test point stays in the group of its test
                pd = datetime.date(2010, month_number(pm.group(2)), int(pm.group(1)))
                day_group(row, inv, f"day:{pd.isoformat()}", "GR-PARENT")
            doys = {x for x, _ in runs.get(c["SAME Test #"], set())}
            if not sub and seen[c["SAME Test #"]] == 1 and doys and d.timetuple().tm_yday not in doys:
                off_day += 1
        else:
            row["date"] = unk("the Date cell is not day-month", raw=c["Date"] or None, source=s("Date"), unit="date")
            day_group(row, inv, f"test:{tid}", "GR-ROW")
        mat = material(c["Sample Type"])
        row["fuel"]["name"] = text(mat, c["Sample Type"], "nominal", s("Sample Type"), flags=dup_flag)
        row["fuel"]["components"] = [component(mat, "fuel", res(1.0, "1", None, None, "derived", s("Sample Type"), rule="PURE"))]
        heated_wire(row, wire)
        sp = cell(c["Inlet velocity (cm/s)"], "m/s", 1e-2, "cm/s", "nominal", s("Inlet velocity (cm/s)"))
        row["flow"]["speed"] = sp
        if sp["status"] == "resolved":
            row["flow"]["direction"] = res("quiescent" if sp["value_si"] == 0 else "forced", None, None, None, "derived",
                                           s("Inlet velocity (cm/s)"), rule="F2")
        ex = row["extra_conditions"]
        if c["Description"]:
            ex["description"] = text(c["Description"], c["Description"], "nominal", s("Description"))
        for key, col, unit, raw_unit in (("heater_resistance_ratio", "Rt/R0", "1", "1"),
                                         ("heating_duration", "Duration (sec)", "s", "s"),
                                         ("primary_aging_time", "Primary aging time (sec)", "s", "s"),
                                         ("primary_mixing_time", "Primary mixing time (sec)", "s", "s")):
            if c[col] != "":
                ex[key] = cell(c[col], unit, 1.0, raw_unit, "nominal", s(col))
        set_fuel_gas_phase(row)
        oc = "Net aging time (sec)"
        row["outcome_status"] = "listed"
        row["outcome_fields_available"] = [outcome_field(oc, fn)]
        out.add(row, [outcome_item(oc, c[oc], fn, i)])
    if off_day:
        out.defect(inv.id, f"{fn}: for {off_day} runs the Date differs by one day from the GMT day of year in the names of the analysed-data files "
                           "(runs late in a session); the Date column is therefore not uniformly a GMT date")


def psi102(out):
    inv = Investigation(102)
    path = table_path(102, "*Experimental table_SAME.csv")
    out.use(path)
    fn = path.name
    rows = read_csv(path)
    H = [h.strip() for h in rows[0]]
    wire = src(R_SAMER, page=3, note="SAME-R used the SAME hardware; heating wire control described on p. 26")
    period = src(R_SAMER, page=2, note="SAME was conducted from September 9 to October 5, 2007")
    dbp = src(R_SAME, page=3, note="dibutyl-phthalate retained in a cotton wick")
    c_s, c_m, c_air, c_w, c_age = H
    parsed = OrderedDict()
    for i, r in enumerate(rows[1:], start=2):
        parsed[r[0].strip()] = (i, dict(zip(H, [x.strip() for x in r])))
    for sid, (i, c) in parsed.items():
        s = lambda col: src(fn, column=col, row=i)
        row = blank(inv, f"matrix-{sid}", "matrix_row")
        row["source_rows"] = [{"file": fn, "sheet": None, "row": i}]
        day_group(row, inv, "matrix", "GR-MATRIX")
        row["date"] = multi("range", ["2007-09-09", "2007-10-05"], "date", None, None, "nominal", period,
                            "the table has no date; the period of the flight tests is given by a report", scope="investigation")
        heated_wire(row, wire)
        ref = re.fullmatch(r"(.+?): same conditions as (\d+)-(\d+)", c[c_m])
        mraw = ref.group(1) if ref else c[c_m]
        mat = material(mraw)
        row["fuel"]["name"] = text(mat, c[c_m], "nominal", s(c_m))
        row["fuel"]["components"] = [component(mat, "fuel", res(1.0, "1", None, None, "derived", s(c_m), rule="PURE"))]
        if ref:
            refs = [parsed[str(k)] for k in range(int(ref.group(2)), int(ref.group(3)) + 1)]
            why = f"the row says '{c[c_m]}': the values are those of the referenced rows"
            speeds = sorted({num(rc[c_air]) * 1e-2 for _, rc in refs})
            ages = sorted({num(rc[c_age]) * 60.0 for _, rc in refs})
            row["flow"]["speed"] = multi("list", speeds, "m/s", c[c_m], "cm/s", "nominal", s(c_m), why, rule="REF")
            row["flow"]["direction"] = res("forced", None, None, None, "derived", s(c_m), rule="F2")
            row["extra_conditions"]["planned_aging_duration"] = multi("list", ages, "s", c[c_m], "minutes", "nominal", s(c_m), why, rule="REF")
            items = []
        else:
            sp = cell(c[c_air], "m/s", 1e-2, "cm/s", "nominal", s(c_air))
            row["flow"]["speed"] = sp
            if sp["status"] == "resolved":
                row["flow"]["direction"] = res("quiescent" if sp["value_si"] == 0 else "forced", None, None, None, "derived", s(c_air), rule="F2")
            row["extra_conditions"]["planned_aging_duration"] = cell(c[c_age], "s", 60.0, "minutes", "nominal", s(c_age))
            items = [outcome_item(c_w, c[c_w], fn, i)]
        set_fuel_gas_phase(row)
        if mat == "dibutyl_phthalate":
            row["fuel"]["phase"] = res("liquid_other", None, None, None, "nominal", dbp, rule="P5")
        row["outcome_status"] = "listed"
        row["outcome_fields_available"] = [outcome_field(c_w, fn)]
        out.add(row, items)


# ----------------------------------------------------------------------------- PSI-47 DAFT
def psi47(out):
    inv = Investigation(47)
    path = table_path(47, "*Experimental table_DAFT.csv")
    out.use(path)
    fn = path.name
    rows = read_csv(path)
    H = [h.strip() for h in rows[0]]
    bag = src(inv.meta_file, field="approach", note="the dust was aerosolised with GN2 and contained within Mylar bags")
    for i, r in enumerate(rows[1:], start=2):
        c = dict(zip(H, [x.strip() for x in r]))
        s = lambda col: src(fn, column=col, row=i)
        row = blank(inv, c["Session Name"], "test_day")
        row["source_rows"] = [{"file": fn, "sheet": None, "row": i}]
        day_group(row, inv, f"session:{c['Session Name']}", "GR-SESSION")
        m = re.fullmatch(r"GMT (\d+) \((\d{4})\)", c["Actual Day"])
        if m:
            d = datetime.date(int(m.group(2)), 1, 1) + datetime.timedelta(days=int(m.group(1)) - 1)
            set_date(row, d, c["Actual Day"], s("Actual Day"), "derived", rule="D2")
        mraw = c["Sample Material"]
        name = mraw.split(" in combination")[0]
        row["fuel"]["name"] = text(material(re.sub(r"\s*\(.*\)", "", name)), mraw, "nominal", s("Sample Material"))
        row["geometry"]["configuration"] = res("aerosol_bag", None, None, None, "nominal", bag, scope="investigation")
        row["flow"]["speed"] = na("no flame and no flow imposed on a sample")
        row["flow"]["direction"] = na("no flame and no flow imposed on a sample")
        row["extra_conditions"]["hardware_description"] = text(c["Hardware Description"], c["Hardware Description"], "nominal", s("Hardware Description"))
        row["extra_conditions"]["sampled_location"] = text(c["Sampled Location"], c["Sampled Location"], "nominal", s("Sampled Location"))
        set_fuel_gas_phase(row)
        out.add(row)


# ----------------------------------------------------------------------------- PSI-68 FLEX-2 (matrix)
def numbers_in(s):
    return [float(x) for x in re.findall(r"\d*\.?\d+", s)]


def psi68(out):
    inv = Investigation(68)
    path = table_path(68, "*Experimental table_Flex2.csv")
    out.use(path)
    fn = path.name
    rows = read_csv(path)
    H = rows[0]
    c_fuel, c_d0, c_gas, c_o2, c_p, c_t, c_u = H
    mdca = src(inv.manifest_file, field="hardware", note="Multi-user Droplet Combustion Apparatus; the table has an initial droplet diameter")
    for i, r in enumerate(rows[1:], start=2):
        c = dict(zip(H, [x.strip() for x in r]))
        s = lambda col: src(fn, column=col.strip(), row=i)
        row = blank(inv, f"matrix-{i - 1}", "matrix_row")
        row["source_rows"] = [{"file": fn, "sheet": None, "row": i}]
        day_group(row, inv, "matrix", "GR-MATRIX")
        why = "planned test matrix: the row gives the set of values to be covered, not one test"
        fraw = c[c_fuel]
        parts = re.findall(r"([A-Za-z-]+)\s*\((\d+)%\)", fraw)
        if parts:
            comps = [(species_of(n), int(p) / 100.0, p) for n, p in parts]
            row["fuel"]["name"] = text("+".join(n for n, _, _ in comps), fraw, "nominal", s(c_fuel))
            row["fuel"]["components"] = [
                component(n, "fuel", res(x, "1", p, "%", "nominal", s(c_fuel), flags=["basis_of_fraction_unstated"])) for n, x, p in comps
            ]
        else:
            sp = species_of(re.sub(r"\s*\(pure\)", "", fraw))
            row["fuel"]["name"] = text(sp, fraw, "nominal", s(c_fuel))
            row["fuel"]["components"] = [component(sp, "fuel", res(1.0, "1", "pure", None, "nominal", s(c_fuel)))]
        row["geometry"]["configuration"] = res("droplet_unspecified", None, None, None, "derived", mdca, rule="CFG-DROPLET", scope="investigation")
        d = numbers_in(c[c_d0])
        row["geometry"]["sizes"].append(OrderedDict([
            ("dimension", "initial_droplet_diameter"),
            ("value", multi("range", [x * 1e-3 for x in d], "m", c[c_d0], "mm", "nominal", s(c_d0), why)),
        ]))
        row["atmosphere"]["x_o2"] = multi("list", numbers_in(c[c_o2]), "1", c[c_o2], "1", "nominal", s(c_o2), why)
        row["atmosphere"]["pressure"] = multi("list", [x * ATM for x in numbers_in(c[c_p])], "Pa", c[c_p], "atm", "nominal", s(c_p), why)
        row["atmosphere"]["temperature"] = cell(c[c_t], "K", 1.0, "C", "nominal", s(c_t), offset=273.15)
        dil = [species_of(x) for x in re.findall(r"O2/(\w+)", c[c_gas])]
        row["atmosphere"]["diluents"] = OrderedDict([
            ("status", "multiple"),
            ("reason", f"the row says '{c[c_gas]}': one of these gases is the balance of oxygen"),
            ("items", [OrderedDict([("species", x), ("fraction", unk("balance of oxygen, not tabulated", unit="1"))]) for x in dil]),
        ])
        row["flow"]["speed"] = multi("range", [x * 1e-2 for x in numbers_in(c[c_u])], "m/s", c[c_u], "cm/s", "nominal", s(c_u), why)
        row["flow"]["direction"] = unk("the table gives a flow velocity range without a direction")
        set_fuel_gas_phase(row)
        out.add(row)


# ----------------------------------------------------------------------------- PSI-98/99/100 Saffire
LEN_UNITS = {"mm": 1e-3, "cm": 1e-2, "m": 1.0}


def qty(raw, default_unit):
    """'0.27 mm' / '.33 cm' / '20' -> (number, unit) or None. Rule Q1: a number that starts with 0
    followed by a digit ('037 mm') is a damaged cell and is not read."""
    t = raw.strip()
    m = re.fullmatch(r"~?\s*(\d*\.?\d+)\s*([A-Za-z/]+)?", t)
    if not m or re.match(r"0\d", m.group(1)):
        return None
    return float(m.group(1)), (m.group(2) or default_unit)


def length_value(raw, default_unit, source, alt_raw=None, alt_source=None):
    """A length cell of a Saffire table, with the Readme transcription as second source (rule DOC)."""
    q = qty(raw, default_unit)
    aq = qty(alt_raw.split(" (")[0], None) if alt_raw else None
    alts = []
    if q and q[1] in LEN_UNITS:
        v = q[0] * LEN_UNITS[q[1]]
        flags = []
        if aq and abs(aq[0] * LEN_UNITS[aq[1]] - v) > 1e-9:
            flags.append("conflict")
            alts.append(alt(aq[0] * LEN_UNITS[aq[1]], alt_raw, alt_source))
        return res(v, "m", raw.strip(), q[1], "nominal", source, flags=flags, alternatives=alts)
    lst = re.fullmatch(r"(\d*\.?\d+) and (\d*\.?\d+) (mm|cm)", raw.strip())
    if lst:
        f = LEN_UNITS[lst.group(3)]
        flags = []
        if aq:
            flags.append("conflict")
            alts.append(alt(aq[0] * LEN_UNITS[aq[1]], alt_raw, alt_source))
        return multi("list", [float(lst.group(1)) * f, float(lst.group(2)) * f], "m", raw.strip(), lst.group(3), "nominal", source,
                     "the cell gives two values for a two-part sample", flags=flags, alternatives=alts)
    if aq:
        flag = "blank_in_table" if not raw.strip() else "malformed_in_table"
        return res(aq[0] * LEN_UNITS[aq[1]], "m", alt_raw, aq[1], "nominal", alt_source, rule="DOC", flags=[flag],
                   alternatives=[alt(None, raw.strip() or None, source)])
    if not raw.strip():
        return unk("blank cell in the source", source=source, unit="m")
    return unk(f"the cell '{raw.strip()}' is not a length", raw=raw.strip(), source=source, unit="m")


def saffire(out, n):
    inv = Investigation(n)
    path = table_path(n, "*Experimental table_SAFFIRE-*.csv")
    out.use(path)
    fn = path.name
    rows = read_csv(path)
    H = rows[0]

    def col(pat):
        hits = [h for h in H if re.search(pat, h, re.I)]
        return hits[0] if hits else None

    c_id, c_mat = col(r"Sample Number"), col(r"Material")
    c_th, c_len, c_wid = col(r"Thickness"), col(r"Sample Length"), col(r"Width")
    c_air, c_o2 = col(r"Air Flow"), col(r"Percent O2")
    c_pw, c_ti, c_burn = col(r"Ignition Power"), col(r"Ignition Time"), col(r"Burn Time")
    c_dir = col(r"Direction")
    cams = [h for h in H if re.search(r"Camera", h)]
    out_cols = [h for h in H if re.search(r"Burn Time|Burn Length|Spread", h)]
    tr = {}
    tr_file = None
    if n == 99:
        tr_file = "tools/wsp/atlas/transcribed/psi99_saffire2_readme_table1.csv"
        out.use(ROOT / tr_file)
        tr = {t["sample"]: t for t in read_transcription("psi99_saffire2_readme_table1.csv")}
    note1 = None
    for r in rows[1:]:
        if r and r[0].startswith("Note 1"):
            note1 = r[0].strip()
    skipped = 0
    for i, r in enumerate(rows[1:], start=2):
        c = dict(zip(H, r))
        sid = c[c_id].strip()
        if not re.fullmatch(r"S\d+|2-\d+", sid):
            skipped += 1
            continue
        s = lambda cl: src(fn, column=cl.strip(), row=i)
        row = blank(inv, sid, "test")
        row["source_rows"] = [{"file": fn, "sheet": None, "row": i}]
        day_group(row, inv, "flight", "GR-FLIGHT")
        t = tr.get("S" + sid.split("-")[1]) if n == 99 else None
        ts = (lambda column: src(tr_file, column=column, note="hand transcription of Table 1 (a picture) of PSI-99_Science Documents_SAFFIRE-II Readme.docx")) if t else None
        for k in cams:
            v = c[k].strip()
            if v and v.lower() != "n/a":
                row["aliases"].append({"scheme": "psi_record_" + slug(k), "name": v})
        mraw = c[c_mat].strip()
        if mraw:
            mat = material(mraw)
            fl, alts = [], []
            if t and material(t["material"]) != mat:
                fl, alts = ["conflict"], [alt(material(t["material"]), t["material"], ts("material"))]
                out.defect(inv.id, f"{fn}: sample {sid} material '{mraw}' against '{t['material']}' in the Readme table")
            row["fuel"]["name"] = text(mat, mraw, "nominal", s(c_mat), flags=fl, alternatives=alts)
        elif t:
            mat = material(t["material"])
            row["fuel"]["name"] = text(mat, t["material"], "nominal", ts("material"), rule="DOC", flags=["blank_in_table"],
                                       alternatives=[alt(None, None, s(c_mat))])
            out.defect(inv.id, f"{fn}: sample {sid} has a blank Material cell (merged cell lost); filled from the Readme table")
        else:
            mat = None
            row["fuel"]["name"] = unk("blank cell in the source", source=s(c_mat))
        if mat:
            row["fuel"]["components"] = [component(x, "fuel", unk("composition of the sample by part is not tabulated", unit="1")
                                                   if "+" in mat else res(1.0, "1", None, None, "derived", row["fuel"]["name"]["source"], rule="PURE"))
                                         for x in mat.split("+")]
        row["geometry"]["configuration"] = res("flat_sample", None, None, None, "derived",
                                               src(fn, column=f"{c_th.strip()}, {c_len.strip()}, {c_wid.strip()}"), rule="CFG-SHEET", scope="investigation")
        th = length_value(c[c_th], header_unit(c_th), s(c_th), t["thickness"] if t else None, ts("thickness") if t else None)
        if "conflict" in th["flags"] or "malformed_in_table" in th["flags"]:
            out.defect(inv.id, f"{fn}: sample {sid} thickness cell '{c[c_th].strip()}' against '{t['thickness']}' in the Readme table")
        ln = length_value(c[c_len], header_unit(c_len), s(c_len))
        wd = length_value(c[c_wid], header_unit(c_wid), s(c_wid))
        if mat == "sibal_fabric" and th["status"] == "resolved" and th["value_si"] > 1e-3:
            # the same fabric is 0.33 mm thick in the Saffire-II Readme table
            th["flags"] = sorted(set(th["flags"]) | {"defect_noted"})
            out.defect(inv.id, f"{fn}: sample {sid} fabric thickness '{c[c_th].strip()}' in a column or cell of unit cm; the Saffire-II Readme "
                               "table gives 0.33 mm for SIBAL fabric, so the unit is suspect")
        if ln["status"] == "resolved" and wd["status"] == "resolved" and ln["value_si"] < wd["value_si"]:
            for z in (ln, wd):
                z["flags"] = sorted(set(z["flags"]) | {"defect_noted"})
            out.defect(inv.id, f"{fn}: sample {sid} length ({c[c_len].strip()}) is smaller than width ({c[c_wid].strip()}); the PSI metadata "
                               "describes the sample as about 0.94 m long and 0.4 m wide, and the Saffire-III table has the two the other way round")
        row["geometry"]["sizes"] = [
            OrderedDict([("dimension", "sample_thickness"), ("value", th)]),
            OrderedDict([("dimension", "sample_length"), ("value", ln)]),
            OrderedDict([("dimension", "sample_width"), ("value", wd)]),
        ]
        q = qty(c[c_air], header_unit(c_air))
        if q and q[1] == "cm/s":
            fl, alts = [], []
            if t and abs(float(t["air_flow_cm_s"]) - q[0]) > 1e-9:
                fl, alts = ["conflict"], [alt(float(t["air_flow_cm_s"]) * 1e-2, t["air_flow_cm_s"], ts("air_flow_cm_s"))]
                out.defect(inv.id, f"{fn}: sample {sid} air flow {q[0]:g} cm/s against {t['air_flow_cm_s']} cm/s in the Readme table")
            row["flow"]["speed"] = res(q[0] * 1e-2, "m/s", c[c_air].strip(), "cm/s", "nominal", s(c_air), flags=fl, alternatives=alts)
        else:
            row["flow"]["speed"] = unk("the Air Flow cell is not a speed in cm/s", raw=c[c_air].strip() or None, source=s(c_air), unit="m/s")
        dr = c[c_dir].strip().lower()
        if dr in ("concurrent", "opposed"):
            row["flow"]["direction"] = text(dr, c[c_dir].strip(), "nominal", s(c_dir))
        items = [outcome_item(h, c[h], fn, i) for h in out_cols]
        if c_o2:
            oraw = c[c_o2].strip()
            basis = "derived" if note1 else "unstated"
            osrc = dict(s(c_o2), note=note1) if note1 else s(c_o2)
            if re.fullmatch(r"~?\s*\d+\.?\d*", oraw):
                x = float(oraw.lstrip("~ "))
                row["atmosphere"]["x_o2"] = res(x / 100.0, "1", oraw, "%", basis, osrc, flags=["approximate"] if oraw.startswith("~") else [])
            elif oraw:
                row["atmosphere"]["x_o2"] = withheld("the cell gives a range of oxygen over the burn (information from after ignition); "
                                                     "it is kept in tests_outcomes.jsonl until the evaluation groups are frozen", s(c_o2), unit="1")
                items.append(outcome_item(c_o2, oraw, fn, i, name="percent_o2_over_burn"))
        row["atmosphere"]["pressure"] = unk("not in the table; a pressure sensor is listed among the Saffire diagnostics (sensor files in Raw Data)", unit="Pa")
        ex = row["extra_conditions"]
        if c_pw:
            q = qty(c[c_pw], "W")
            if q and q[1] == "W":
                ex["igniter_power"] = res(q[0], "W", c[c_pw].strip(), "W", "nominal", s(c_pw))
            q = qty(c[c_ti], "s")
            if q and q[1] == "s":
                ex["igniter_time"] = res(q[0], "s", c[c_ti].strip(), "s", "nominal", s(c_ti))
        set_fuel_gas_phase(row)
        fields = [outcome_field(h, fn) for h in out_cols]
        if c_o2:
            fields.append(outcome_field(c_o2, fn, name="percent_o2_over_burn"))
        row["outcome_fields_available"] = fields
        row["outcome_status"] = "listed" if fields else "none_in_source"
        out.add(row, items)
    out.skipped[inv.id] = {"table_rows_not_tests": skipped,
                           "reason": "rows that describe flow-visualisation records or are table notes, not samples"}


# ----------------------------------------------------------------------------- PSI-106 SLICE
def psi106(out):
    inv = Investigation(106)
    path = table_path(106, "*Experimental table_Slice.csv")
    out.use(path)
    fn = path.name
    rows = read_csv(path)
    H = [h.strip() for h in rows[0]]
    cof = src(inv.meta_file, field="approach", note="fan-driven flow duct, exchangeable burner tubes, flames observed under controlled coflow air")
    for i, r in enumerate(rows[1:], start=2):
        a, b = r[0].strip(), r[1].strip()
        s = lambda col: src(fn, column=col, row=i)
        m = re.match(r"(\d{4})-([A-Z]{3})-(\d{2})", a)
        d = datetime.date(int(m.group(1)), month_number(m.group(2)), int(m.group(3)))
        row = blank(inv, d.isoformat(), "test_day")
        row["source_rows"] = [{"file": fn, "sheet": None, "row": i}]
        set_date(row, d, m.group(0), s("Test Date"), "nominal")
        day_group(row, inv, f"day:{d.isoformat()}", "GR-DAY")
        rest = a[m.end():].strip()
        both = (rest + " | " + b) if rest else b
        fuels = re.findall(r"(\d+)%\s*(C\d?H\d+)", both)
        fs = src(fn, column="Test Date, Fuel Type" if rest else "Fuel Type", row=i)
        if rest:
            row["extra_conditions"]["description"] = text(both, both, "nominal", fs)
            if "%" in rest:
                out.defect(inv.id, f"{fn}: row {i} has part of the fuel description in the Test Date cell ('{rest}')")
        elif not re.fullmatch(r"\d+%\s*C\d?H\d+(\s*\(balance N2\))?", b):
            row["extra_conditions"]["description"] = text(b, b, "nominal", fs)
        n2 = "balance N2" in both
        if len(fuels) == 1:
            pct, f = fuels[0]
            sp = species_of(f)
            xfv = res(int(pct) / 100.0, "1", f"{pct}% {f}", "%", "nominal", fs)
            row["fuel"]["name"] = text(sp, b, "nominal", fs)
            row["fuel"]["components"] = [component(sp, "fuel", xfv)]
            if n2:
                row["fuel"]["components"].append(component("nitrogen", "diluent", balance_fraction([xfv], fs, "'(balance N2)' in the cell")))
            row["injection"]["x_fuel_injected"] = xfv
        elif fuels:
            names = [species_of(f) for _, f in fuels]
            why = "the row names two fuels used on the same day"
            row["fuel"]["name"] = multi("list", names, None, both, None, "nominal", fs, why)
            row["fuel"]["components"] = [component(species_of(f), "fuel", res(int(p) / 100.0, "1", f"{p}% {f}", "%", "nominal", fs)) for p, f in fuels]
            fracs = sorted({int(p) / 100.0 for p, _ in fuels})
            row["injection"]["x_fuel_injected"] = (res(fracs[0], "1", both, "%", "nominal", fs) if len(fracs) == 1
                                                   else multi("list", fracs, "1", both, "%", "nominal", fs, why))
        row["geometry"]["configuration"] = res("coflow_burner", None, None, None, "nominal", cof, scope="investigation")
        row["geometry"]["flame_orientation"] = res("normal", None, None, None, "derived", fs, rule="OR1")
        row["flow"]["direction"] = res("coflow", None, None, None, "nominal", cof, scope="investigation")
        row["flow"]["speed"] = unk("not in the table (test-day list); fan calibration and coflow files are in Raw Data", unit="m/s")
        why = "not in the table (test-day list)"
        for k in ("fuel_mass_flow", "inert_mass_flow", "total_mass_flow"):
            row["injection"][k] = unk(why, unit="kg/s")
        if row["injection"]["x_fuel_injected"]["status"] == "not_applicable":
            row["injection"]["x_fuel_injected"] = unk("no fuel fraction could be read from the row", unit="1")
        row["atmosphere"]["x_o2"] = unk("not in the table; the metadata says coflow air without a composition", unit="1")
        set_fuel_gas_phase(row)
        out.add(row)


# ----------------------------------------------------------------------------- ACME test-day tables
def check_listing_day(out, inv, row, code):
    if code not in inv.listing_days():
        row["notes"].append(f"no raw-data file name in the listing carries the test day {code}")
        return False
    return True


def bottle(rawtxt):
    """'100%C2H4' / '50% C2H4' / '40% CH4' -> (fraction, formula) or None."""
    m = re.fullmatch(r"(\d+(?:\.\d+)?)\s*%\s*(C\d?H\d+|H2)", rawtxt.strip())
    return (float(m.group(1)) / 100.0, m.group(2)) if m else None


def pct_list(rawtxt):
    return [float(x) / 100.0 for x in re.findall(r"(\d+(?:\.\d+)?)", rawtxt)]


def acme_day_common(row, inv, why_flows="not in the table (test-day list); flows per test are in Raw Data"):
    for k in ("fuel_mass_flow", "inert_mass_flow", "total_mass_flow"):
        row["injection"][k] = unk(why_flows, unit="kg/s")
    row["flow"]["speed"] = na("no ambient flow speed is tabulated; reactant flow through the burner is under `injection`")


def psi20(out):
    inv = Investigation(20)
    path = table_path(20, "*Experimental table_ACME BRE.csv")
    out.use(path)
    fn = path.name
    rows = read_csv(path)
    H = [h.strip() for h in rows[0]]
    tr_file = "tools/wsp/atlas/transcribed/psi20_bre_readme_table1.csv"
    out.use(ROOT / tr_file)
    tr = read_transcription("psi20_bre_readme_table1.csv")
    body = rows[1:]
    if len(tr) != len(body):
        raise SystemExit("PSI-20: the Readme transcription and the CSV table differ in row count")
    tnote = "hand transcription of Table 1 (a picture) of PSI-20_Science Documents_ACME_BRE_Readme.docx"
    flat = src(R_FINAL_10, page=10, note="two flat porous burners for the BRE experiment, inner diameters 25 and 50 mm")
    n2 = src(fn, column="Notes", row=2, note="Oxygen Conc. (as diluted with nitrogen on a volume basis)")
    for i, (r, t) in enumerate(zip(body, tr), start=2):
        c = dict(zip(H, [x.strip() for x in r]))
        s = lambda col: src(fn, column=col, row=i)
        ts = lambda col: src(tr_file, column=col, row=i, note=tnote)
        code = t["test_day"]
        row = blank(inv, code, "test_day")
        row["source_rows"] = [{"file": fn, "sheet": None, "row": i}, {"file": tr_file, "sheet": None, "row": i}]
        row["aliases"].append({"scheme": "test_day_yyddd", "name": code})
        d_tr = date_from_yyddd(code)
        mon = month_number(c["Month"])
        d_csv = datetime.date(int(c["Year"]), mon, int(c["Date"])) if mon else None
        raw_csv = f"{c['Year']} {c['Month']} {c['Date']}"
        if d_csv == d_tr:
            set_date(row, d_tr, raw_csv, s("Year, Month, Date"), "nominal")
        else:
            set_date(row, d_tr, code, ts("test_day"), "derived", rule="D1", flags=["conflict"],
                     alternatives=[alt(d_csv.isoformat() if d_csv else None, raw_csv, s("Year, Month, Date"))])
            out.defect(inv.id, f"{fn}: row {i} is dated '{raw_csv}' but the Readme table gives test day {code} ({d_tr.isoformat()}); "
                               f"the listing has raw files for {code} and "
                               f"{'also' if d_csv and yyddd_of(d_csv) in inv.listing_days() else 'none'} for {yyddd_of(d_csv) if d_csv else '?'}")
        day_group(row, inv, f"day:{code}", "GR-DAY")
        check_listing_day(out, inv, row, code)
        row["geometry"]["configuration"] = res("flat_porous_burner", None, None, None, "nominal", flat, scope="investigation")
        row["geometry"]["flame_orientation"] = res("normal", None, None, None, "derived", s("Fuel Bottle"), rule="OR1")
        bs = num(c["Burner Size (mm)"])
        fl, alts = [], []
        if bs is not None and abs(bs - float(t["burner_size_mm"])) > 1e-9:
            fl, alts = ["conflict"], [alt(float(t["burner_size_mm"]) * 1e-3, t["burner_size_mm"], ts("burner_size_mm"))]
            out.defect(inv.id, f"{fn}: row {i} burner size {bs:g} mm against {t['burner_size_mm']} mm in the Readme table")
        burner_size(row, bs, s("Burner Size (mm)"), scope="row", flags=fl, alternatives=alts)
        bt = bottle(c["Fuel Bottle"])
        if bt:
            sp = species_of(bt[1])
            xfv = res(bt[0], "1", c["Fuel Bottle"], "%", "nominal", s("Fuel Bottle"))
            row["fuel"]["name"] = text(sp, c["Fuel Bottle"], "nominal", s("Fuel Bottle"))
            row["fuel"]["components"] = [component(sp, "fuel", xfv)]
            if bt[0] < 1:
                row["fuel"]["components"].append(component("nitrogen", "diluent", balance_fraction(
                    [xfv], src("PSI-20_Science Documents_ACME_BRE_Readme.docx"), "'50% ethylene (balance nitrogen)' in the Readme text")))
            row["injection"]["x_fuel_injected"] = xfv
            if bottle(t["fuel_bottle"]) != bt:
                out.defect(inv.id, f"{fn}: row {i} fuel bottle '{c['Fuel Bottle']}' against '{t['fuel_bottle']}' in the Readme table")
        acme_day_common(row, inv)
        row["flow"]["direction"] = res("burner_injection", None, None, None, "derived", s("Burner Size (mm)"), rule="F1")
        oraw = c["Oxygen Conc (percent)"]
        ol_csv = pct_list(oraw)
        ol_tr = pct_list(t["oxygen_conc"])
        flags = ["approximate", "conditioned_on_ignition"]
        why = "test-day row: the oxygen concentrations used on that day (table note: approximate, only tests that ignited are listed)"
        if any(x > 1.0 for x in ol_csv):
            out.defect(inv.id, f"{fn}: row {i} oxygen cell '{oraw}' is a damaged percentage; the Readme table gives '{t['oxygen_conc']}'")
            vals, source, raw = ol_tr, ts("oxygen_conc"), t["oxygen_conc"]
            flags.append("malformed_in_table")
            alts, rule = [alt(None, oraw, s("Oxygen Conc (percent)"))], "DOC"
        else:
            vals, source, raw, alts, rule = ol_csv, s("Oxygen Conc (percent)"), oraw, [], None
            if ol_csv != ol_tr:
                flags.append("conflict")
                alts = [alt(None, t["oxygen_conc"], ts("oxygen_conc"))]
                out.defect(inv.id, f"{fn}: row {i} oxygen list '{oraw}' against '{t['oxygen_conc']}' in the Readme table")
        if len(vals) == 1:
            row["atmosphere"]["x_o2"] = res(vals[0], "1", raw, "%", "nominal", source, rule=rule, flags=flags, alternatives=alts)
        else:
            row["atmosphere"]["x_o2"] = multi("list", vals, "1", raw, "%", "nominal", source, why, rule=rule, flags=flags, alternatives=alts)
        row["atmosphere"]["diluents"] = OrderedDict([
            ("status", "resolved"), ("reason", None),
            ("items", [OrderedDict([("species", "nitrogen"), ("fraction", unk("balance of oxygen, not tabulated", source=n2, unit="1"))])]),
        ])
        row["atmosphere"]["pressure"] = unk("table note: the chamber pressures were also varied but are not identified in this summary table",
                                            source=src(fn, column="Notes", row=2), unit="Pa")
        set_fuel_gas_phase(row)
        oc = "Flames Ignited"
        row["outcome_status"] = "listed"
        row["outcome_fields_available"] = [outcome_field(oc, fn)]
        out.add(row, [outcome_item(oc, c[oc], fn, i)])


def psi21(out):
    inv = Investigation(21)
    path = table_path(21, "*Experimental table_CLD Flame.csv")
    out.use(path)
    fn = path.name
    rows = read_csv(path)
    H = [h.strip() for h in rows[0]]
    body = [dict(zip(H, [x.strip() for x in r])) for r in rows[1:]]
    fixed = fill_series([c["Fuel Bottle"] for c in body])
    nfix = sum(1 for _, a in fixed if a)
    if nfix:
        out.defect(inv.id, f"{fn}: {nfix} Fuel Bottle cells are spreadsheet fill-series artefacts "
                           "(for example '40% CH5' ... '40% CH14', '100% C2H5' ... '100% C2H11'); read as the first cell of each run")
    cof = src(inv.manifest_file, field="title", note="Coflow Laminar Diffusion Flame")
    note = src(fn, column="Notes", row=2, note="Fuel conc. (as diluted with nitrogen on a volume basis) was varied between tests and sometimes during tests")
    for i, (c, (btxt, art)) in enumerate(zip(body, fixed), start=2):
        s = lambda col: src(fn, column=col, row=i)
        code = c["Test Day"]
        row = blank(inv, code, "test_day")
        row["source_rows"] = [{"file": fn, "sheet": None, "row": i}]
        row["aliases"].append({"scheme": "test_day_yyddd", "name": code})
        d = date_from_yyddd(code)
        set_date(row, d, code, s("Test Day"), "derived", rule="D1")
        day_group(row, inv, f"day:{code}", "GR-DAY")
        if not check_listing_day(out, inv, row, code):
            out.defect(inv.id, f"{fn}: test day {code} (row {i}) has no raw-data file in the listing")
        row["geometry"]["configuration"] = res("coflow_burner", None, inv.title, None, "nominal", cof, scope="investigation")
        row["geometry"]["flame_orientation"] = res("normal", None, None, None, "derived", s("Fuel Bottle"), rule="OR1")
        bt = bottle(btxt)
        fl = ["fill_series_artifact"] if art else []
        if bt:
            sp = species_of(bt[1])
            row["fuel"]["name"] = text(sp, c["Fuel Bottle"], "nominal", s("Fuel Bottle"), rule="FILL" if art else None, flags=fl)
            kw = re.match(r"(Methane|Ethylene)", c["Test Type"])
            if kw and species_of({"Methane": "CH4", "Ethylene": "C2H4"}[kw.group(1)]) != sp:
                out.defect(inv.id, f"{fn}: row {i} Test Type '{c['Test Type']}' against Fuel Bottle '{btxt}'")
            rng = pct_list(c["Fuel conc. Range"])
            fs = s("Fuel conc. Range")
            if len(rng) == 1:
                xfv = res(rng[0], "1", c["Fuel conc. Range"], "%", "nominal", fs)
            elif len(rng) == 2:
                xfv = multi("range", rng, "1", c["Fuel conc. Range"], "%", "nominal", fs,
                            "test-day row: the fuel concentration was varied between tests and sometimes during tests (table note)")
            else:
                xfv = unk("the Fuel conc. Range cell is not a percentage or a range", raw=c["Fuel conc. Range"] or None, source=fs, unit="1")
            row["fuel"]["components"] = [
                component(sp, "fuel", xfv),
                component("nitrogen", "diluent", balance_fraction([xfv], note, "diluted with nitrogen on a volume basis (table note)")),
            ]
            row["injection"]["x_fuel_injected"] = xfv
        else:
            row["fuel"]["name"] = unk("the Fuel Bottle cell is not '<percent> <fuel>'", raw=c["Fuel Bottle"], source=s("Fuel Bottle"))
        acme_day_common(row, inv)
        row["flow"]["speed"] = unk("coflow speed is not in the table (test-day list)", unit="m/s")
        row["flow"]["direction"] = res("coflow", None, inv.title, None, "nominal", cof, scope="investigation")
        ex = row["extra_conditions"]
        ex["test_type"] = text(c["Test Type"], c["Test Type"], "nominal", s("Test Type"))
        ex["fuel_mfc_full_scale"] = cell(c["Fuel MFC"], "m3/s", SLPM, "slpm (nitrogen basis)", "nominal", s("Fuel MFC"))
        ex["n2_mfc_full_scale"] = cell(c["N2 MFC"], "m3/s", SLPM, "slpm (nitrogen basis)", "nominal", s("N2 MFC"))
        set_fuel_gas_phase(row)
        oc = "Flames ignited"
        row["outcome_status"] = "listed"
        row["outcome_fields_available"] = [outcome_field(oc, fn)]
        out.add(row, [outcome_item(oc, c[oc], fn, i)])


def psi22(out):
    inv = Investigation(22)
    path = table_path(22, "*Experimental table_ACME E-FIELD Flames.csv")
    out.use(path)
    fn = path.name
    rows = read_csv(path)
    H = [h.strip() for h in rows[0]]
    blocks = []
    for i, r in enumerate(rows[1:], start=2):
        c = dict(zip(H, [x.strip() for x in r]))
        if c["Test Day"]:
            blocks.append({"row": i, "c": c, "conc": [(c["Fuel Conc"], i)], "rows": [i]})
        elif c["Fuel Conc"] and blocks:
            blocks[-1]["conc"].append((c["Fuel Conc"], i))
            blocks[-1]["rows"].append(i)
    seen = {}
    for b in blocks:
        c = b["c"]
        i = b["row"]
        sigkey = (c["Test Day"], c["Mesh Polarity"], c["Fuel Bottle"], tuple(x for x, _ in b["conc"]), c["Flames Ignited"])
        if c["Test Day"] in seen:
            if seen[c["Test Day"]][0] == sigkey:
                out.defect(inv.id, f"{fn}: the block of test day {c['Test Day']} (rows {b['rows'][0]}-{b['rows'][-1]}) repeats rows "
                                   f"{seen[c['Test Day']][1][0]}-{seen[c['Test Day']][1][-1]} exactly; emitted once")
                for row in out.rows:
                    if row["atlas_id"] == f"{inv.id}:{c['Test Day']}":
                        row["source_rows"] += [{"file": fn, "sheet": None, "row": k} for k in b["rows"]]
                continue
            raise SystemExit(f"PSI-22: test day {c['Test Day']} appears twice with different content")
        seen[c["Test Day"]] = (sigkey, b["rows"])
        s = lambda col, r=i: src(fn, column=col, row=r)
        code = c["Test Day"]
        row = blank(inv, code, "test_day")
        row["source_rows"] = [{"file": fn, "sheet": None, "row": k} for k in b["rows"]]
        row["aliases"].append({"scheme": "test_day_yyddd", "name": code})
        d = datetime.date(int(c["Year"]), int(c["Month"]), int(c["Date"]))
        raw = f"{c['Year']}-{c['Month']}-{c['Date']}"
        fl, alts = [], []
        if yyddd_of(d) != code:
            d2 = date_from_yyddd(code)
            fl, alts = ["conflict"], [alt(d2.isoformat() if d2 else None, code, s("Test Day"))]
            out.defect(inv.id, f"{fn}: row {i} date {raw} is not the day of year of Test Day {code}")
        set_date(row, d, raw, s("Year, Month, Date"), "nominal", flags=fl, alternatives=alts)
        day_group(row, inv, f"day:{code}", "GR-DAY")
        if not check_listing_day(out, inv, row, code):
            out.defect(inv.id, f"{fn}: test day {code} (row {i}) has no raw-data file in the listing")
        tt = c["Test Type"]
        if "coflow burner" in tt:
            row["geometry"]["configuration"] = res("coflow_burner", None, tt, None, "nominal", s("Test Type"))
            row["flow"]["direction"] = res("coflow", None, tt, None, "nominal", s("Test Type"))
            row["flow"]["speed"] = unk("coflow speed is not in the table (test-day list)", unit="m/s")
        elif "gas-jet burner" in tt:
            row["geometry"]["configuration"] = res("gas_jet_burner", None, tt, None, "nominal", s("Test Type"))
            row["flow"]["direction"] = res("burner_injection", None, tt, None, "derived", s("Test Type"), rule="F1")
        row["geometry"]["flame_orientation"] = res("normal", None, None, None, "derived", s("Fuel Bottle"), rule="OR1")
        acme_day_common(row, inv)
        if "coflow burner" in tt:
            row["flow"]["speed"] = unk("coflow speed is not in the table (test-day list)", unit="m/s")
        bt = bottle(c["Fuel Bottle"])
        note = src(fn, column="Notes", row=4, note="Fuel Conc. (diluted with nitrogen on a volume basis) was varied between tests")
        if bt:
            sp = species_of(bt[1])
            row["fuel"]["name"] = text(sp, c["Fuel Bottle"], "nominal", s("Fuel Bottle"))
            vals = [pct_list(x)[0] for x, _ in b["conc"]]
            fs = src(fn, column="Fuel Conc", row=i, note=f"rows {b['rows'][0]}-{b['rows'][-1]}" if len(b["rows"]) > 1 else None)
            rawc = "; ".join(x for x, _ in b["conc"])
            if len(vals) == 1:
                xfv = res(vals[0], "1", rawc, "%", "nominal", fs)
            else:
                xfv = multi("list", vals, "1", rawc, "%", "nominal", fs,
                            "test-day row: the fuel concentrations of the flames ignited on that day (table note)", flags=["conditioned_on_ignition"])
            if len(vals) == 1:
                xfv["flags"] = sorted(set(xfv["flags"]) | {"conditioned_on_ignition"})
            row["fuel"]["components"] = [
                component(sp, "fuel", xfv),
                component("nitrogen", "diluent", balance_fraction([xfv], note, "diluted with nitrogen on a volume basis (table note)")),
            ]
            row["injection"]["x_fuel_injected"] = xfv
        ex = row["extra_conditions"]
        ex["test_type"] = text(tt, tt, "nominal", s("Test Type"))
        ex["mesh_polarity"] = text(c["Mesh Polarity"], c["Mesh Polarity"], "nominal", s("Mesh Polarity"))
        set_fuel_gas_phase(row)
        oc = "Flames Ignited"
        row["outcome_status"] = "listed"
        row["outcome_fields_available"] = [outcome_field(oc, fn)]
        out.add(row, [outcome_item(oc, c[oc], fn, i)])


def psi23(out):
    inv = Investigation(23)
    path = table_path(23, "*Experimental table_ACME s-Flame.csv")
    out.use(path)
    fn = path.name
    rows = read_csv(path)
    H = [h.strip() for h in rows[0]]
    body = [dict(zip(H, [x.strip() for x in r])) for r in rows[1:]]
    atm_fixed = fill_series([c["Atmosphere*"] for c in body])
    bot_fixed = fill_series([c["Fuel Bottle"] for c in body])
    nfix = sum(1 for _, a in atm_fixed + bot_fixed if a)
    if nfix:
        out.defect(inv.id, f"{fn}: {nfix} Atmosphere and Fuel Bottle cells are spreadsheet fill-series artefacts "
                           "(for example '~21% O2 with N3', '50% C2H4 with N3'); read as the first cell of each run")
    fin = src(R_FINAL_23, page=10, note="the spherical burners were limited to a single diameter of 6.35 mm")
    norm = src(inv.meta_file, field="approach", note="fuel and diluent mixtures issue into an oxidising environment through a spherical porous burner")
    p_row = [k for k, c in enumerate(body, start=2) if "~1 atm" in c["Notes"]]
    p_src = src(fn, column="Notes", row=p_row[0] if p_row else None, note="Atmosphere is ~1 atm but pressure increases and O2% drops with vitiation")
    for i, (c, (atxt, a_art), (btxt, b_art)) in enumerate(zip(body, atm_fixed, bot_fixed), start=2):
        s = lambda col: src(fn, column=col, row=i)
        code = c["Test Day*"]
        row = blank(inv, code, "test_day")
        row["source_rows"] = [{"file": fn, "sheet": None, "row": i}]
        row["aliases"].append({"scheme": "test_day_yyddd", "name": code})
        mon = month_number(c["Month"])
        d = datetime.date(int(c["Year"]), mon, int(c["Date"]))
        raw = f"{c['Year']} {c['Month']} {c['Date']}"
        fl, alts = [], []
        if yyddd_of(d) != code:
            d2 = date_from_yyddd(code)
            fl, alts = ["conflict"], [alt(d2.isoformat() if d2 else None, code, s("Test Day*"))]
            out.defect(inv.id, f"{fn}: row {i} date {raw} is not the day of year of Test Day {code}")
        set_date(row, d, raw, s("Year, Month, Date"), "nominal", flags=fl, alternatives=alts)
        day_group(row, inv, f"day:{code}", "GR-DAY")
        if not check_listing_day(out, inv, row, code):
            out.defect(inv.id, f"{fn}: test day {code} (row {i}) has no raw-data file in the listing")
        row["geometry"]["configuration"] = res("spherical_porous_burner", None, None, None, "nominal", fin, scope="investigation")
        row["geometry"]["flame_orientation"] = res("normal", None, None, None, "nominal", norm, scope="investigation")
        burner_size(row, 6.35, fin)
        acme_day_common(row, inv)
        row["flow"]["direction"] = res("burner_injection", None, None, None, "derived", s("Fuel Bottle"), rule="F1")
        # atmosphere '~21% O2 with N2' / '~30% and ~21% O2 with He'
        m = re.fullmatch(r"~(\d+)%(?: and ~(\d+)%)? O2 with (\w+)", atxt)
        afl = ["approximate"] + (["fill_series_artifact"] if a_art else [])
        if m:
            vals = [int(x) / 100.0 for x in m.groups()[:2] if x]
            if len(vals) == 1:
                row["atmosphere"]["x_o2"] = res(vals[0], "1", c["Atmosphere*"], "%", "nominal", s("Atmosphere*"), flags=afl, rule="FILL" if a_art else None)
            else:
                row["atmosphere"]["x_o2"] = multi("list", vals, "1", c["Atmosphere*"], "%", "nominal", s("Atmosphere*"),
                                                  "test-day row: two oxygen levels were used on that day", flags=afl)
            row["atmosphere"]["diluents"] = OrderedDict([
                ("status", "resolved"), ("reason", None),
                ("items", [OrderedDict([("species", species_of(m.group(3))),
                                        ("fraction", balance_fraction([row["atmosphere"]["x_o2"]], s("Atmosphere*"), "'with' gas of the Atmosphere cell"))])]),
            ])
        row["atmosphere"]["pressure"] = res(ATM, "Pa", "~1 atm", "atm", "nominal", p_src, flags=["approximate"], scope="investigation")
        # fuel bottle '50% C2H4 with N2' / '100% H2' / '25% H2, 20% CH4, with He' / '44.4% H2 with CH4'
        left, _, bal = btxt.partition(" with ")
        left = left.rstrip(", ")
        parts = re.findall(r"(\d+(?:\.\d+)?)%\s*(\w+)", left)
        bfl = ["fill_series_artifact"] if b_art else []
        if parts:
            fs = s("Fuel Bottle")
            comps = [component(species_of(f), "fuel", res(float(p) / 100.0, "1", f"{p}% {f}", "%", "nominal", fs)) for p, f in parts]
            names = [x["species"] for x in comps]
            xfuel = sum(x["fraction"]["value_si"] for x in comps)
            if bal:
                bsp = species_of(bal.strip())
                role = "diluent" if bsp in INERTS else "fuel"
                comps.append(component(bsp, role, balance_fraction([x["fraction"] for x in comps], fs, "'with' gas of the Fuel Bottle cell")))
                names.append(bsp)
                if role == "fuel":
                    xfuel = 1.0
            fuel_names = [x["species"] for x in comps if x["role"] == "fuel"]
            row["fuel"]["name"] = text("+".join(fuel_names), c["Fuel Bottle"], "nominal", fs, flags=bfl, rule="FILL" if b_art else None)
            row["fuel"]["components"] = comps
            row["injection"]["x_fuel_injected"] = res(xfuel, "1", c["Fuel Bottle"], "%", "derived", fs, rule="XF", flags=bfl)
        row["extra_conditions"]["test_type"] = text(c["Test Type"], c["Test Type"], "nominal", s("Test Type"))
        kw = re.search(r"(nitrogen|helium)-diluted", c["Test Type"])
        if kw and m and species_of(m.group(3)) != kw.group(1):
            out.defect(inv.id, f"{fn}: row {i} Test Type says '{kw.group(1)}-diluted' but its Atmosphere cell is '{c['Atmosphere*']}'")
        set_fuel_gas_phase(row)
        oc = "Flames Ignited*"
        row["outcome_status"] = "listed"
        row["outcome_fields_available"] = [outcome_field(oc, fn)]
        out.add(row, [outcome_item(oc, c[oc], fn, i)])


# ----------------------------------------------------------------------------- the four project families
def from_record(cv, family, part, unit=None, basis=None, extra_flags=()):
    """A condition of a project record -> value object. Rule REC of SCHEMA.md."""
    loc = cv["locator"]
    source = src(loc["path"], column=loc.get("column"), row=loc.get("row"),
                 note=f"copied from data/work/records/{family}/experiments.jsonl ({part}), method {cv['derivation']['method_id']}")
    flags = [S.RECORD_FLAG_PREFIX + f for f in cv["flags"]]
    b = basis or ("derived" if cv["evidence_class"] == "source_derived" else "unstated")
    raw = cv["raw_value"]
    if cv["resolution"] == "unresolved_conflict":
        flags.append("conflict")
    if "multiple_settings_untimed" in cv["flags"]:
        return varying(raw, source, "the record holds an untimed sequence of settings used during the test", unit=unit, flags=flags)
    val = cv["value_si"]
    if val is None and isinstance(raw, str) and cv["unit_si"] is None and cv["resolution"] != "unknown":
        return res(raw, None, raw, cv["raw_unit"], b, source, flags=flags)
    if val is None:
        return unk("unresolved in the project record", raw=raw, raw_unit=cv["raw_unit"], source=source, flags=[f for f in flags if f != "conflict"], unit=unit)
    return res(val, cv["unit_si"], raw, cv["raw_unit"], b, source, flags=flags)


def project_families(out):
    sys.path.insert(0, str(ROOT / "tools/wsp/analysis"))
    import lib  # conditions of train and validation runs only

    plan = [("flex", "flex", 69), ("flex_heptane", "flex", 69), ("acme_normal", "acme_normal", 159),
            ("acme_inverse", "acme_inverse", 159), ("bass", "bass", 25)]
    invs = {}
    withheld_counts = OrderedDict()
    for name, family, n in plan:
        inv = invs.setdefault(n, Investigation(n))
        _, run_part = lib.split(name)
        groups = lib.groups(name)
        ids = {p: lib.partition_ids(name, p) for p in ("train", "validation")}
        withheld_counts[name] = {
            "investigation_id": inv.id,
            "runs_in_split": len(run_part),
            "emitted_train": len(ids["train"]),
            "emitted_validation": len(ids["validation"]),
            "test_partition_not_read": len(run_part) - len(ids["train"]) - len(ids["validation"]),
        }
        for part in ("train", "validation"):
            recs = lib.experiments(name, ids[part])
            for rid in sorted(recs):
                rec = recs[rid]
                cond = rec["conditions"]
                tid = rid[len(inv.id) + 1:] if rid.startswith(inv.id + ":") else rid
                row = blank(inv, tid, "test")
                row["project_partition"] = part
                row["group_id"] = groups[rid]
                row["group_rule"] = "GR-SPLIT"
                seen = set()
                for a in rec["aliases"]:
                    if (a["scheme"], a["name"]) not in seen:
                        seen.add((a["scheme"], a["name"]))
                        row["aliases"].append({"scheme": a["scheme"], "name": a["name"]})
                files = []
                for cv in cond.values():
                    p = cv["locator"]["path"]
                    key = (p, cv["locator"].get("row"))
                    if p and key not in files:
                        files.append(key)
                row["source_rows"] = [{"file": p, "sheet": None, "row": r} for p, r in sorted(files, key=lambda x: (x[0], x[1] or 0))]
                rnote = src(f"data/work/records/{family}/experiments.jsonl", field="session_id", note=f"{part} run")
                m = re.search(r"day:(\d{4}-\d{2}-\d{2})", rec.get("session_id") or "")
                if m:
                    row["date"] = res(m.group(1), "date", rec["session_id"], None, "derived", rnote, rule="REC")
                fr = lambda key, **kw: from_record(cond[key], family, part, **kw)
                cfg_src = src(f"data/work/records/{family}/experiments.jsonl", field="configuration", note=f"{part} run")
                if family == "flex":
                    fuel = cond["fuel"]["raw_value"]
                    sp = species_of(fuel)
                    fv = fr("fuel")
                    row["fuel"]["name"] = res(sp, None, fuel, None, "nominal", fv["source"])
                    row["fuel"]["components"] = [component(sp, "fuel", res(1.0, "1", None, None, "derived", fv["source"], rule="PURE"))]
                    fib = cond["fiber"]["raw_value"]
                    fsrc = fr("fiber")["source"]
                    token = {"no": "droplet_free", "yes": "droplet_fiber_supported"}.get(str(fib).strip().lower(), "droplet_unspecified")
                    row["geometry"]["configuration"] = res(token, None, fib, None, "nominal", fsrc)
                    row["extra_conditions"]["fiber"] = fr("fiber", basis="nominal")
                    row["geometry"]["sizes"].append(OrderedDict([("dimension", "initial_droplet_diameter"), ("value", fr("initial_diameter", unit="m"))]))
                    row["atmosphere"]["x_o2"] = fr("x_o2", unit="1")
                    row["atmosphere"]["pressure"] = fr("pressure", unit="Pa")
                    row["atmosphere"]["temperature"] = fr("ambient_temperature", unit="K")
                    row["atmosphere"]["diluents"] = OrderedDict([
                        ("status", "resolved"), ("reason", None),
                        ("items", [OrderedDict([("species", spn), ("fraction", fr(k, unit="1"))])
                                   for spn, k in (("nitrogen", "x_n2"), ("carbon_dioxide", "x_co2"), ("helium", "x_he"))]),
                    ])
                    row["flow"]["speed"] = unk("no flow condition in the project record", unit="m/s")
                    row["flow"]["direction"] = unk("no flow condition in the project record")
                elif family in ("acme_normal", "acme_inverse"):
                    orient = "normal" if family == "acme_normal" else "inverse"
                    sp = species_of(cond["fuel_species"]["raw_value"])
                    fv = fr("fuel_species")
                    row["fuel"]["name"] = res(sp, None, rec["fuel_or_material"], None, "nominal", fv["source"])
                    row["geometry"]["configuration"] = unk("the project record holds only the flame orientation; the burner is not named in it")
                    row["geometry"]["flame_orientation"] = res(orient, None, rec["configuration"], None, "nominal", cfg_src)
                    row["atmosphere"]["pressure"] = fr("pressure", unit="Pa")
                    if "rec:approximate_per_header" in row["atmosphere"]["pressure"]["flags"]:
                        row["atmosphere"]["pressure"]["flags"] = sorted(set(row["atmosphere"]["pressure"]["flags"]) | {"approximate"})
                    row["atmosphere"]["diluents"] = OrderedDict([("status", "unknown"), ("reason", "the project record does not name the balance gas of the chamber"), ("items", [])])
                    row["flow"]["speed"] = na("no ambient flow is imposed in the record; the reactant flow through the burner is under `injection`")
                    row["flow"]["direction"] = res("burner_injection", None, None, None, "derived", cfg_src, rule="F1")
                    inj = row["injection"]
                    burner_injection_defaults(row, "not in the project record")
                    if orient == "normal":
                        xf = fr("x_fuel_injected", unit="1")
                        row["fuel"]["components"] = [component(sp, "fuel", xf)]
                        inj["x_fuel_injected"] = xf
                        inj["fuel_mass_flow"] = fr("injected_mass_flow", unit="kg/s")
                        row["atmosphere"]["x_o2"] = fr("x_o2_ambient", unit="1")
                    else:
                        xfa = fr("x_fuel_ambient", unit="1")
                        row["fuel"]["components"] = [component(sp, "fuel", xfa)]
                        row["atmosphere"]["x_fuel_ambient"] = xfa
                        row["atmosphere"]["x_o2"] = na("inverse flame: the chamber holds the fuel mixture and oxygen is injected (see oxidizer_stream.x_o2)")
                        row["oxidizer_stream"]["x_o2"] = fr("x_o2_injected", unit="1")
                        inj["x_fuel_injected"] = na("inverse flame: the injected stream is the oxidiser")
                        inj["fuel_mass_flow"] = na("inverse flame: the injected stream is the oxidiser")
                        inj["oxidizer_mass_flow"] = fr("injected_mass_flow", unit="kg/s")
                    row["extra_conditions"]["zst"] = fr("zst", unit="1")
                    row["extra_conditions"]["tad"] = fr("tad", unit="K")
                    for a in rec["aliases"]:
                        if a["scheme"] == "campaign_day":
                            row["extra_conditions"]["campaign_day"] = text(f"{orient} {a['name']}", a["name"], "nominal",
                                                                           src(a["locator"]["path"], column=a["locator"].get("column"), row=a["locator"].get("row")))
                else:  # bass
                    mraw = rec["fuel_or_material"]
                    msrc = src(f"data/work/records/{family}/experiments.jsonl", field="fuel_or_material", note=f"{part} run")
                    mat = "sibal_fabric" if re.search(r"(?i)sibal", mraw) else material(mraw)
                    row["fuel"]["name"] = res(mat, None, mraw, None, "nominal", msrc)
                    row["fuel"]["components"] = [component(mat, "fuel", res(1.0, "1", None, None, "derived", msrc, rule="PURE"))]
                    row["geometry"]["configuration"] = res("flat_sample", None, rec["configuration"], None, "nominal", cfg_src)
                    row["atmosphere"]["x_o2"] = fr("x_o2", unit="1", basis="measured")
                    row["atmosphere"]["diluents"] = OrderedDict([
                        ("status", "resolved"), ("reason", None),
                        ("items", [OrderedDict([("species", "carbon_dioxide"), ("fraction", fr("x_co2", unit="1"))])]),
                    ])
                    row["flow"]["speed"] = fr("flow_velocity", unit="m/s", basis="measured")
                    row["flow"]["direction"] = unk("no flow direction among the conditions of the project record")
                    ex = row["extra_conditions"]
                    ex["fan_display"] = fr("fan_display", basis="nominal")
                    ex["flow_restrictor"] = fr("flow_restrictor", basis="nominal")
                    ex["x_co"] = fr("x_co", unit="1")
                set_fuel_gas_phase(row)
                row["outcome_status"] = "not_read_protected"
                out.add(row)
    return withheld_counts


# ----------------------------------------------------------------------------- varied, summary, output
def _sig_of(v):
    if v["status"] == "resolved":
        return ("r", v["value_si"])
    if v["status"] == "multiple":
        return ("m", tuple(v["set"]["values"]))
    if v["status"] == "varying":
        return ("v",)
    return None


def tracked(row):
    """(path, signature) of every condition that can count as 'varied' (rule VAR)."""
    yield "fuel.name", _sig_of(row["fuel"]["name"])
    yield "fuel.phase", _sig_of(row["fuel"]["phase"])
    yield "geometry.configuration", _sig_of(row["geometry"]["configuration"])
    yield "geometry.flame_orientation", _sig_of(row["geometry"]["flame_orientation"])
    for z in row["geometry"]["sizes"]:
        yield f"geometry.sizes.{z['dimension']}", _sig_of(z["value"])
    a = row["atmosphere"]
    for k in ("x_o2", "pressure", "temperature", "x_fuel_ambient"):
        yield f"atmosphere.{k}", _sig_of(a[k])
    dl = a["diluents"]
    if dl["status"] in ("resolved", "multiple"):
        yield "atmosphere.diluents.species", ("r", tuple(sorted(x["species"] for x in dl["items"])), dl["status"])
        for x in dl["items"]:
            yield f"atmosphere.diluents.{x['species']}", _sig_of(x["fraction"])
    if row["atmosphere"]["x_o2"]["status"] == "not_applicable":
        yield "oxidizer_stream.x_o2", _sig_of(row["oxidizer_stream"]["x_o2"])
    yield "flow.speed", _sig_of(row["flow"]["speed"])
    yield "flow.direction", _sig_of(row["flow"]["direction"])
    for k, v in row["injection"].items():
        yield f"injection.{k}", _sig_of(v)
    for k, v in row["extra_conditions"].items():
        if k not in ("description", "campaign_day", "hardware_description"):
            yield f"extra_conditions.{k}", _sig_of(v)


def fill_varied(rows):
    by = OrderedDict()
    for r in rows:
        by.setdefault(r["investigation_id"], []).append(r)
    for inv_id, rs in by.items():
        seen = {}
        for r in rs:
            for path, sg in tracked(r):
                if sg is None:
                    continue
                st = seen.setdefault(path, set())
                st.add(sg)
        varied = sorted(p for p, st in seen.items() if len(st) > 1 or any(x[0] in ("m", "v") for x in st))
        for r in rs:
            r["varied"] = varied


COVERAGE = [
    ("date", lambda r: r["date"]),
    ("start_time", lambda r: r["start_time"]),
    ("hardware.rack", lambda r: r["hardware"]["rack"]),
    ("hardware.insert", lambda r: r["hardware"]["insert"]),
    ("fuel.name", lambda r: r["fuel"]["name"]),
    ("fuel.phase", lambda r: r["fuel"]["phase"]),
    ("geometry.configuration", lambda r: r["geometry"]["configuration"]),
    ("atmosphere.x_o2", lambda r: r["atmosphere"]["x_o2"]),
    ("oxidizer_stream.x_o2", lambda r: r["oxidizer_stream"]["x_o2"]),
    ("atmosphere.pressure", lambda r: r["atmosphere"]["pressure"]),
    ("atmosphere.temperature", lambda r: r["atmosphere"]["temperature"]),
    ("flow.speed", lambda r: r["flow"]["speed"]),
    ("flow.direction", lambda r: r["flow"]["direction"]),
    ("injection.x_fuel_injected", lambda r: r["injection"]["x_fuel_injected"]),
    ("injection.fuel_mass_flow", lambda r: r["injection"]["fuel_mass_flow"]),
    ("gravity", lambda r: r["gravity"]),
    ("labels.flame_type", lambda r: r["labels"]["flame_type"]),
]


def inv_key(inv_id):
    return int(inv_id.split("-")[1])


def summarize(out, withheld_counts):
    rows = out.rows
    invs = sorted({r["investigation_id"] for r in rows}, key=inv_key)
    by_inv = OrderedDict()
    for i in invs:
        rs = [r for r in rows if r["investigation_id"] == i]
        lv = Counter(r["level"] for r in rs)
        by_inv[i] = OrderedDict([
            ("acronym", rs[0]["acronym"]),
            ("rows", len(rs)),
            ("by_level", OrderedDict((k, lv[k]) for k in S.LEVELS if lv[k])),
            ("groups", len({r["group_id"] for r in rs})),
            ("oxygen_single_value", sum(1 for r in rs if r["map"]["x_o2"] is not None)),
            ("pressure_single_value", sum(1 for r in rs if r["map"]["pressure_pa"] is not None)),
            ("both_oxygen_and_pressure", sum(1 for r in rs if r["map"]["x_o2"] is not None and r["map"]["pressure_pa"] is not None)),
            ("outcome_status", OrderedDict(sorted(Counter(r["outcome_status"] for r in rs).items()))),
        ])
    by_inv["PSI-26"] = OrderedDict([
        ("acronym", Investigation(26).acronym),
        ("rows", 0),
        ("reason", "no record of PSI-26 is reachable through tools/wsp/analysis/lib.py (the 'bass' family holds PSI-25 runs only), "
                   "and the brief allows the four project investigations to be read only through that library"),
    ])
    by_inv = OrderedDict(sorted(by_inv.items(), key=lambda kv: inv_key(kv[0])))

    def phase(r):
        v = r["fuel"]["phase"]
        return v["value_si"] if v["status"] == "resolved" else v["status"]

    ph = Counter(phase(r) for r in rows)
    both = [r for r in rows if r["map"]["x_o2"] is not None and r["map"]["pressure_pa"] is not None]
    set_both = sum(1 for r in rows if r["atmosphere"]["x_o2"]["status"] == "multiple" or r["atmosphere"]["pressure"]["status"] == "multiple")
    crew = OrderedDict()
    for name in S.CREW_ATMOSPHERE:
        hit = [r for r in rows if name in r["map"]["crew_atmospheres"]]
        o2only = [r for r in rows if name in r["map"]["crew_atmospheres_o2_only"]]
        lo, hi = CREW_BANDS[name]["o2"]
        in_set = [r for r in rows if r["atmosphere"]["x_o2"]["status"] == "multiple"
                  and any(lo - 1e-9 <= x <= hi + 1e-9 for x in r["atmosphere"]["x_o2"]["set"]["values"])]
        crew[name] = OrderedDict([
            ("rule", CREW_BANDS[name]),
            ("rows_with_oxygen_and_pressure_inside", len(hit)),
            ("by_investigation", OrderedDict(sorted(Counter(r["investigation_id"] for r in hit).items(), key=lambda kv: inv_key(kv[0])))),
            ("by_level", OrderedDict(sorted(Counter(r["level"] for r in hit).items()))),
            ("rows_with_oxygen_inside_and_pressure_unknown", len(o2only)),
            ("pressure_unknown_by_investigation", OrderedDict(sorted(Counter(r["investigation_id"] for r in o2only).items(), key=lambda kv: inv_key(kv[0])))),
            ("day_or_matrix_rows_with_an_oxygen_set_value_inside", len(in_set)),
            ("oxygen_set_by_investigation", OrderedDict(sorted(Counter(r["investigation_id"] for r in in_set).items(), key=lambda kv: inv_key(kv[0])))),
        ])
    cov = OrderedDict()
    for name, get in COVERAGE:
        c = Counter(get(r)["status"] for r in rows)
        cov[name] = OrderedDict((k, c[k]) for k in S.STATUS if c[k])
    cov["geometry.sizes (at least one resolved or multiple)"] = sum(
        1 for r in rows if any(z["value"]["status"] in ("resolved", "multiple") for z in r["geometry"]["sizes"]))
    cov["atmosphere.diluents"] = OrderedDict(sorted(Counter(r["atmosphere"]["diluents"]["status"] for r in rows).items()))
    basis = Counter()
    for r in rows:
        for _, v in S.walk_values(r):
            if v["status"] in ("resolved", "multiple"):
                basis[v["basis"]] += 1
    oc = OrderedDict()
    for (inv_id, name), n in sorted(out.outcome_cells.items(), key=lambda kv: (inv_key(kv[0][0]), kv[0][1])):
        oc.setdefault(inv_id, OrderedDict())[name] = n
    return OrderedDict([
        ("schema", "ignis.atlas.summary/1"),
        ("row_schema", S.SCHEMA_ID),
        ("rows", len(rows)),
        ("investigations_with_rows", len(invs)),
        ("investigations_expected", 19),
        ("by_investigation", by_inv),
        ("by_level", OrderedDict((k, sum(1 for r in rows if r["level"] == k)) for k in S.LEVELS)),
        ("by_fuel_phase", OrderedDict(sorted(ph.items()))),
        ("by_project_partition", OrderedDict(sorted(Counter(r["project_partition"] for r in rows).items()))),
        ("oxygen_and_pressure", OrderedDict([
            ("rows_with_one_oxygen_value", sum(1 for r in rows if r["map"]["x_o2"] is not None)),
            ("of_which_ambient", sum(1 for r in rows if r["map"]["x_o2_role"] == "ambient")),
            ("of_which_injected_oxidiser", sum(1 for r in rows if r["map"]["x_o2_role"] == "injected")),
            ("rows_with_one_pressure_value", sum(1 for r in rows if r["map"]["pressure_pa"] is not None)),
            ("rows_with_both", len(both)),
            ("rows_with_both_by_level", OrderedDict(sorted(Counter(r["level"] for r in both).items()))),
            ("rows_with_both_by_role", OrderedDict(sorted(Counter(r["map"]["x_o2_role"] for r in both).items()))),
            ("rows_where_oxygen_or_pressure_is_a_set", set_both),
        ])),
        ("crew_atmospheres", crew),
        ("coverage_by_status", cov),
        ("values_by_basis", OrderedDict(sorted(basis.items()))),
        ("groups", len({r["group_id"] for r in rows})),
        ("outcome_values_file", OrderedDict([
            ("tests_with_outcome_values", len(out.outcomes)),
            ("non_empty_cells_by_field", oc),
        ])),
        ("project_families", withheld_counts),
        ("table_rows_not_emitted", out.skipped),
        ("defects", OrderedDict(sorted(out.defects.items(), key=lambda kv: inv_key(kv[0])))),
        ("inputs_sha256", OrderedDict(sorted(out.inputs.items()))),
    ])


def dump_jsonl(path, rows):
    with open(path, "w", encoding="utf-8", newline="\n") as f:
        for r in rows:
            f.write(json.dumps(r, ensure_ascii=False, separators=(",", ":")) + "\n")


def build():
    out = Out()
    psi10(out)
    psi20(out)
    psi21(out)
    psi22(out)
    psi23(out)
    psi39(out)
    psi47(out)
    psi68(out)
    saffire(out, 98)
    saffire(out, 99)
    saffire(out, 100)
    psi101(out)
    psi102(out)
    psi106(out)
    psi107(out)
    withheld_counts = project_families(out)
    order = {r["atlas_id"]: k for k, r in enumerate(out.rows)}
    out.rows.sort(key=lambda r: (inv_key(r["investigation_id"]), order[r["atlas_id"]]))
    out.outcomes.sort(key=lambda r: (inv_key(r["investigation_id"]), order[r["atlas_id"]]))
    ids = Counter(r["atlas_id"] for r in out.rows)
    twice = [k for k, n in ids.items() if n > 1]
    if twice:
        raise SystemExit(f"atlas_id not unique: {twice[:5]}")
    fill_varied(out.rows)
    for n in NEW_IDS + OLD_IDS:
        inv = Investigation(n)
        out.use(inv.manifest_path)
    return out, summarize(out, withheld_counts)


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--out", default=str(NEW / "work/atlas"), help="output directory")
    args = ap.parse_args()
    out_dir = Path(args.out)
    out_dir.mkdir(parents=True, exist_ok=True)
    out, summary = build()
    dump_jsonl(out_dir / "tests.jsonl", out.rows)
    dump_jsonl(out_dir / "tests_outcomes.jsonl", out.outcomes)
    with open(out_dir / "tests_summary.json", "w", encoding="utf-8", newline="\n") as f:
        json.dump(summary, f, ensure_ascii=False, indent=1)
        f.write("\n")
    with open(out_dir / "tests.schema.json", "w", encoding="utf-8", newline="\n") as f:
        json.dump(S.row_schema(), f, ensure_ascii=False, indent=1)
        f.write("\n")
    print(f"{summary['rows']} rows from {summary['investigations_with_rows']} investigations -> {out_dir}")
    for k, v in summary["by_investigation"].items():
        print(f"  {k:8s} {v['acronym'][:22]:22s} {v['rows']:5d}")


if __name__ == "__main__":
    main()
