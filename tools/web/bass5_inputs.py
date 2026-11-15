"""Build web/public/data/bass5_inputs.json: the bass5.wsp inputs of every BASS run, keyed by IGNIS run id.

The forecast lab fills the bass5 model's conditions from this table when a BASS run is picked. Every run of the
IGNIS registry's BASS family (the rows of NASA's experimental tables, PSI-25 BASS-II and PSI-26 BASS) gets one of:

  runs[id]      coverage "complete": every bass5 input derived from NASA's records;
                coverage "partial": the inputs NASA records, the others listed under "missing" (the lab asks the
                user to type them; nothing is guessed);
  excluded[id]  a short reason: the burn is outside what bass5 is written for (candles, wax, spheres lit in
                their wake), or the row is not a burn.

The derivation is the WSP lane's own, imported, not copied: the readers, material geometry, configuration words,
flow steps and fan curves of tools/wsp/ext/bass1_cases.py with the third coding of the flow cells
(bass1_cases_v3.py, R13: fan displays written in the air cell are not speeds), and the features of
tools/wsp/ext/law_inputs.py bass5_row(c, "sustain"). Two differences from the lane's case files, both about which
rows are read, none about how a value is derived:

  * every dated row is read: the lane wrote inputs only for days of its frozen splits (a row whose date no split
    names, such as PSI-25 O20 to O22 typed 2016, was counted and left unread) because its files fed blind scoring;
    the web table holds conditions only, never an outcome, and the test days are study data since PROTOCOL 13.8;
  * the PSI-25 SIBAL rows are read: the lane's plastics files left them to its fabric split, and bass5 covers
    SIBAL for kept burning (is_sibal) with bass4's fabric rule.

Conditions only: the observation columns of the sheets are never written here, and no outcome file is opened.

    python tools/web/bass5_inputs.py --raw D:/dev/SpaceApps-Ardealu/data/raw/samples \
        --raw D:/dev/SpaceApps-Ardealu/runs/dev/data_all/raw
"""
import argparse
import csv
import hashlib
import io
import json
import re
import sys
from collections import Counter
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
sys.path[:0] = [str(REPO / "tools/wsp/ext"), str(REPO / "tools/wsp")]
import bass1_cases as v1  # noqa: E402
import bass1_cases_v2 as v2  # noqa: E402
import bass1_cases_v3 as v3  # noqa: E402
import law_inputs as L  # noqa: E402

FEATURES = ["x_o2", "flow_speed", "nose_radius", "heat_time", "is_concurrent", "is_rod", "flat_thickness",
            "u_pot1", "is_one_sided", "is_sibal", "is_nomex_ultem", "flat_width"]

# NASA files read (by file name; found under any --raw directory). The experimental tables are the IGNIS
# registry's own source of the run ids; the others are the lane's sources of the conditions.
T25 = "PSI-25_Experimental table_BASS-II.csv"
T26 = "PSI-26_Experimental table_BASS.csv"
FILES = {"AS_RUN_26": "PSI-26_Science Documents_As Run_Test Matrix.xlsx",
         "MATRIX_25": "PSI-25_Science Documents_BASS-II_Test.Matrices.xlsx",
         "FANCAL_25": "PSI-25_Reference Data_Fan calibration data.xlsx",
         "REPORT_25": "PSI-25_Reports_BASS-II Summary Report.pdf"}

# What each input is and where it comes from (unit SI, as the served card declares it).
INPUT_DOC = {
    "x_o2": ("1", "PSI-26: as-run matrix 'O2 % by volume' (written as a fraction); PSI-25: test matrix 'Calibrated "
                  "initial O2 % by vol' / 100. Volume fraction taken as mole fraction"),
    "flow_speed": ("m/s", "duct speed of the first flow step: the burn's anemometer reading ('Air display', x 1.0066) "
                          "when it is >= 2 cm/s and pairs with the fan cell, else the fan display or pot setting of the "
                          "'Fan display' cell through the burn's fan curve; cm/s / 100"),
    "nose_radius": ("m", "rods, spheres, sections: diameter from the material text (rod inch fractions, sphere cm; "
                         "spherical section 4 cm) / 2; flat samples 0.01"),
    "heat_time": ("s", "200: no planned igniter on-time is recorded for any BASS burn (bass1_sustain_row)"),
    "is_concurrent": ("1", "flat samples: 1 concurrent, 0 opposed, from the configuration words (material, igniter, "
                           "distance cells; PSI-25: Summary Report Appendix A); fabrics with no stated direction 0.5; "
                           "rods, spheres, sections 0"),
    "is_rod": ("1", "1 if the material text names a rod"),
    "flat_thickness": ("m", "flat PMMA: thickness from the material text (micron or mm; PSI-26 slabs 2 mm per the "
                            "SRD); SIBAL 0.00032, Nomex 0.0003, Ultem 0.00025; rods, spheres, sections 0.0005"),
    "u_pot1": ("m/s", "the burn's fan curve (BASS-II Reference Data fan calibrations: the day's own block, else the "
                      "campaign median for the restrictor count) at fan display 46.5, / 100"),
    "is_one_sided": ("1", "flat PMMA: 1 if the material text says one side burns (PSI-26 slabs one-sided per the "
                          "SRD); fabrics, rods, spheres, sections 0"),
    "is_sibal": ("1", "1 if the material text names SIBAL"),
    "is_nomex_ultem": ("1", "1 if the material text names Nomex or Ultem"),
    "flat_width": ("m", "PMMA films and slabs: width from the material text ('2 cm wide'; PSI-26 slabs 1 cm per "
                        "the SRD); everything else, and a flat PMMA sample with no width recorded, 0.02"),
}

# Outside what bass5 is written for (docs/wsp/models/bass5.wsp.md: PMMA rods, spheres and sections with the nose
# to the flow, films, slabs, SIBAL, Nomex, Ultem). Keyed by the lane's material class.
OUT_OF_SCOPE = {"candle": "wax candle: bass5 covers PMMA and fabrics only",
                "wax_slab": "paraffin wax slab: bass5 covers PMMA and fabrics only"}
WAKE = "sphere lit in its wake: bass5 forecasts spheres lit with the nose to the flow only"


def find(raws, name):
    for d in raws:
        hit = sorted(Path(d).rglob(name))
        if hit:
            return hit[0]
    sys.exit(f"{name}: not under any --raw directory")


def table(path):
    """Rows of an IGNIS experimental table, with the registry's reading (UTF-8 with BOM, cells stripped, blank rows
    skipped)."""
    text = path.read_bytes().decode("utf-8-sig", errors="replace")
    rows = list(csv.reader(io.StringIO(text)))
    head = [h.strip() for h in rows[0]]
    return [dict(zip(head, (c.strip() for c in r))) for r in rows[1:] if any(c.strip() for c in r)]


def registry_runs(raws):
    """IGNIS run ids of the BASS family, as crates/ignis-data/src/families/bass.rs builds them."""
    out, seen = [], set()
    for r in table(find(raws, T25)):
        out.append({"id": f"PSI-25:{r['Test #']}", "inv": "PSI-25", "test": r["Test #"], "material": r["Fuel Sample Material"]})
    for k, r in enumerate(table(find(raws, T26))):
        rid = f"PSI-26:asrun{r['As Run Test #']}:test{r['Test #']}"
        if rid in seen:
            sys.exit(f"{rid}: duplicate run id; the registry suffixes '#row<n>', not handled here")
        seen.add(rid)
        out.append({"id": rid, "inv": "PSI-26", "asrun": r["As Run Test #"], "test": r["Test #"],
                    "material": r["Fuel Sample Material"]})
    return out


def read_psi25_all():
    """bass1_cases.read_psi25 without its SIBAL skip: every row of the BASS-II test matrices. Same cells; the
    observation text of the report's appendix is not kept (conditions only)."""
    import openpyxl
    wb = openpyxl.load_workbook(v1.MATRIX_25, data_only=True, read_only=True)
    app, out = v1.appendix_records(), []
    for ws in wb.worksheets:
        for r in list(ws.iter_rows(values_only=True))[1:]:
            if not r or r[2] is None:
                continue
            tid = str(r[2]).strip()
            cfg, _obs = app.get(tid, ("", ""))
            out.append({
                "investigation": "PSI-25", "date": v1.norm_date(r[0]), "partition": "all", "row": ws.title,
                "case_id": "PSI-25:%s" % tid, "as_run": "", "test": tid, "sample": str(r[3]).strip(),
                "material_raw": str(r[4]).strip(), "igniter": "", "restrictors_raw": r[5], "distance": "",
                "fan": str(r[6] if r[6] is not None else ""), "air": str(r[7] if r[7] is not None else ""), "gn2": "",
                "x_o2": (r[10] / 100.0) if isinstance(r[10], (int, float)) else None, "p_atm": None,
                "co2_mmhg": (r[12] * 7.6) if isinstance(r[12], (int, float)) else None,
                "config_raw": cfg, "obs": ""})
    return out


def as_run(v):
    s = str(v if v is not None else "").strip()
    return s[:-2] if re.fullmatch(r"\d+\.0", s) else s


def lane_cases():
    """The lane's inputs (bass1_cases_v3 coding) of every burn row, keyed by IGNIS-style key."""
    v1.partition_of = lambda inv, date: "all"  # read every dated row (see the module docstring)
    fc = v1.calibrations()
    recs = v1.read_psi26() + read_psi25_all()
    prior, out = Counter(), {}
    v3.install()
    try:
        for rec in recs:
            if "case_id" not in rec:
                continue  # no date: bass1_cases.read_psi26 keeps the date only
            rec["obs"] = ""  # conditions only
            key = (rec["investigation"], rec["sample"])
            inp = v2.inputs_of(rec, fc, prior[key])
            prior[key] += 1
            k = (f"PSI-26:asrun{as_run(rec['as_run'])}:test{rec['test']}" if rec["investigation"] == "PSI-26"
                 else f"PSI-25:{rec['test']}")
            if k in out:
                sys.exit(f"{k}: two lane rows")
            out[k] = inp
    finally:
        v3.restore()
    return fc, out


def sources_of(c, r):
    """Per input: where this run's value came from."""
    s = {}
    s["x_o2"] = c.get("x_o2_source")
    if r.get("flow_speed") is not None:
        s["flow_speed"] = f"first flow step '{c.get('ignition_setting')}' = {c.get('u_cm_s_at_ignition')} cm/s"
    s["u_pot1"] = c.get("fan_curve_source") + f"; {c.get('flow_restrictors')} restrictor(s), {c.get('flow_restrictors_source')}"
    s["heat_time"] = "200 s: no planned igniter on-time recorded"
    mat = c.get("material_class")
    if mat in ("pmma_film", "pmma_slab"):
        s["flat_width"] = ("material text" if c.get("width_m") is not None else "0.02: no width recorded")
    else:
        s["flat_width"] = "0.02: not a PMMA film or slab"
    if c.get("configuration"):
        s["is_concurrent"] = f"configuration '{c['configuration']}'"
    elif mat in L.FABRIC_THICKNESS:
        s["is_concurrent"] = "0.5: the records state no direction (fabric rule)"
    s["material"] = f"material class {mat} from '{c.get('material_raw')}'"
    return s


def classify(c, top):
    """('complete'|'partial', inputs, missing, note) or ('excluded', reason). `top` is the highest fan display
    of the calibrations (pot 10)."""
    mat, conf = c.get("material_class"), c.get("configuration")
    if mat in OUT_OF_SCOPE:
        return ("excluded", OUT_OF_SCOPE[mat])
    if mat == "pmma_sphere" and conf == "wake":
        return ("excluded", WAKE)
    r, cov = L.bass5_row(c, "sustain")
    missing, note = [], None
    if cov not in ("nose", "flat"):
        if mat in L.FLAT and conf is None:
            # A flat PMMA sample whose direction the records do not state: every bass5 input but is_concurrent
            # follows from the records (bass2_row: only is_concurrent reads the configuration of a flat sample).
            r, cov = L.bass5_row(dict(c, configuration="opposed"), "sustain")
            r.pop("is_concurrent", None)
            missing.append("is_concurrent")
            note = "flow direction (opposed or concurrent) not recorded"
        elif mat == "pmma_rod" and conf is None and L.num(c.get("diameter_m")):
            # A rod's bass5 inputs do not read its configuration (bass2_row: nose radius, is_concurrent 0 and
            # is_rod 1 in every configuration), so they are complete; the configuration is reported, not filled.
            r, cov = L.bass5_row(dict(c, configuration="stagnation"), "sustain")
            note = "flow configuration not recorded; no bass5 input of a rod depends on it"
        else:
            return ("excluded", f"not a sample bass5 covers ({mat or 'material not read'}, {conf or 'no configuration'})")
    m = re.fullmatch(r"fan ([\d.e+]+)", c.get("ignition_setting") or "")
    if m and float(m.group(1)) > top:
        # A fan display above pot 10 is not a setting the fan has (PSI-25 T15: '80122', two displays run
        # together): the cell is not read, the speed is left for the user.
        r.pop("flow_speed", None)
        note = "flow speed at ignition: the fan display cell is not a readable setting"
    missing += [f for f in FEATURES if r.get(f) is None and f not in missing]
    inputs = {f: round(float(r[f]), 9) for f in FEATURES if r.get(f) is not None}
    # kind: which targets apply. "nose" rod tips, spheres, sections; "flat" PMMA films and slabs; "fabric" SIBAL,
    # Nomex, Ultem: the flat kept-burning pair only (bass4_row: "quench_u stays refused for them").
    kind = "fabric" if mat in L.FABRIC_THICKNESS else cov
    return ("partial" if missing else "complete", kind, inputs, missing, note)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--raw", action="append", required=True, help="directory holding the NASA files (searched by name)")
    ap.add_argument("--out", default=str(REPO / "web/public/data/bass5_inputs.json"))
    a = ap.parse_args()
    for attr, name in FILES.items():
        setattr(v1, attr, find(a.raw, name))
    v1.REPO = v1.FANCAL_25.parent  # calibrations() names its file relative to REPO (not written here)
    runs = registry_runs(a.raw)
    fc, cases = lane_cases()
    top = max(d for _, d in fc["fan_display_by_pot"])
    rows, excluded = {}, {}
    for run in runs:
        c = cases.get(run["id"])
        if c is None:
            excluded[run["id"]] = ("not a burn: " + run["material"].lower()) if not run["test"] else \
                "no row of NASA's as-run matrix for this test"
            continue
        got = classify(c, top)
        if got[0] == "excluded":
            excluded[run["id"]] = got[1]
            continue
        coverage, kind, inputs, missing, note = got
        src = sources_of(c, inputs)
        row = {"coverage": coverage, "kind": kind, "material": c.get("material_raw"), "case_id": c["case_id"],
               "inputs": inputs, "sources": {k: v for k, v in src.items() if k in inputs or k == "material"}}
        if missing:
            row["missing"] = missing
        if note:
            row["note"] = note
        rows[run["id"]] = row
    used = [v1.AS_RUN_26, v1.MATRIX_25, v1.FANCAL_25, v1.REPORT_25, find(a.raw, T25), find(a.raw, T26)]
    out = {
        "model": "bass5.wsp",
        "derivation": "tools/web/bass5_inputs.py: tools/wsp/ext/bass1_cases.py readers and geometry with the "
                      "bass1_cases_v3.py flow coding, law_inputs.py bass5_row(c, 'sustain'); conditions only",
        "sources": [{"file": p.name, "sha256": hashlib.sha256(p.read_bytes()).hexdigest()} for p in used],
        "inputs": {k: {"unit": u, "source": s} for k, (u, s) in INPUT_DOC.items()},
        "counts": dict(Counter(r["coverage"] for r in rows.values()), excluded=len(excluded), registry=len(runs)),
        "runs": dict(sorted(rows.items())),
        "excluded": dict(sorted(excluded.items())),
    }
    Path(a.out).parent.mkdir(parents=True, exist_ok=True)
    Path(a.out).write_text(json.dumps(out, indent=1, ensure_ascii=False) + "\n", encoding="utf-8", newline="\n")
    print(json.dumps(out["counts"]), "->", a.out)


if __name__ == "__main__":
    main()
