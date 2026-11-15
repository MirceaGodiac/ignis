#!/usr/bin/env python3
"""Inputs of the generic law forward (`law.rs`) from the data lanes' case files: the seventh-generation W0
laws of docs/wsp/research/phys_<slug>_w0.md, read by tools/wsp/law_run.py.

Conditions only. This step reads the `inputs` of a case (never an outcome, a prefix or a prior) and writes,
per case, the features its slug declares (configs/models/law/<slug>/decl.json) under their declared names
and in their declared SI units. Every derived input is a deterministic function of the conditions, the
same for every partition; each is listed below with the dossier line that asks for it. Nothing is fitted,
tuned or chosen per case; a condition that is missing, or a text value outside the declared set, is left
out of the row, so that the forward refuses the case and names the feature.

  bre     burner_diameter = burner_diameter_m; x_o2 = x_o2_ambient; pressure = pressure_pa;
          x_fuel = x_fuel_burner_stream (renamed, unchanged);
          burner_mass_flux = burner_exit_velocity_stp_m_s * N_STP * (x_fuel MW_fuel + (1 - x_fuel) MW_N2):
            "mass flow of the burner gas (fuel plus its nitrogen) at the first setpoint, per unit face area"
            (phys_bre_w0.md 7.1). N_STP and MW are the constants of bre_cases.py that defined the velocity.
            For a pure fuel it equals fuel_mass_flux_kg_m2_s.
          is_methane = 1 if fuel_species == methane, 0 if ethylene ("1 for methane, 0 for ethylene").
          stated_time: the time asked by `lasts` ("any admissible value for the other targets, which give it
            no weight"); not a condition of the test. Set by the caller (--stated-time); the admission rows
            carry 60 s, one of the feasibility horizons.
  spice   nozzle_diameter = nozzle_id_mm * 1e-3; coflow_speed = u_air_cm_s * 1e-2 (SI conversions);
          x_fuel unchanged; is_propane / is_propylene / is_methane from `hydrocarbon` (ethylene: all 0;
            "indicator (ethylene is the reference fuel)", phys_spice_w0.md 7.1);
          fuel_mass_flow = inputs_given_flow.mdot_jet_mg_s * 1e-6 when the row gives it (T2 rows). T1 rows
            (smoke point not given) carry the declared centre 1e-6 kg/s: "The two smoke-point targets carry
            weight zero on fuel_mass_flow (typed below), so the flow of a row cannot inform them; the forward
            still needs the condition to be present" (phys_spice_w0.md section 7). Only the smoke-point
            targets of a T1 row are used.
  efield  carbon_flow = carbon_flow_atoms_s; x_fuel; is_ethylene (fuel == ethylene); mesh_positive
            (polarity == positive); window = window_s; sweep_v_max = max |v_cmd| over inputs.levels
            ("largest magnitude of v_cmd over the programme", phys_efield_w0.md 7.1).
          One row per planned level for ion_current: voltage = |v_cmd| of the level ("magnitude of the level's
            v_cmd"); one row per test for flame_end and the peaks: voltage = sweep_v_max ("for the peak targets
            give the highest voltage of the sweep as voltage", section 7).
  samer   is_kapton / is_teflon / is_silicone / is_pyrell from `material` (lamp wick: all 0);
          no_flow = 1 when inlet_velocity == 0 ("derived by the engineer from material and inlet_velocity",
            phys_samer_w0.md 7.1); sample_temperature, inlet_velocity, net_aging_time, heating_duration are
            the case inputs, passed with their units.
  same    is_kapton / is_teflon / is_silicone / is_dbp from `material` (lamp wick: all 0; phys_same_w0.md 7.1);
          inlet_velocity, net_aging_time passed with their units.
  daft    is_cabin / is_velcro / is_bag from `air` (hepa: all 0);
          dusttrak_mass_floored = max(dusttrak_mass_concentration, 1e-9 kg/m3): "`dusttrak_mass_floored` is
            `max(dusttrak_mass_concentration, 1e-9 kg/m3)`" (phys_daft_w0.md 7.1).
  bass1   (case inputs are at the top level of the bass1 rows.) x_o2 = x_o2_initial; flow_speed =
            u_cm_s_at_ignition * 1e-2 (the setting chosen before the burn; quench_speed and flat_quench_speed give
            the speed weight zero). Geometry covered by the dossier: "rod tips, spheres, sphere sections" and
            "films and slabs" (phys_bass1_w0.md section 2):
            nose_radius = diameter_m / 2 for pmma_rod, pmma_sphere, pmma_spherical_section in configuration
              opposed, stagnation or forward stagnation (the nose faces the flow); "for a flat sample supply 0.01"
              for pmma_film and pmma_slab (7.1);
            is_concurrent = 1 if configuration == concurrent, 0 if opposed, for films and slabs; 0 for rods and
              spheres ("0 for rods and spheres", 7.1);
            heat_time = 200 s for every case: "If it is not recorded supply 200" (7.1); no bass1 case records it.
          Not covered, no feature written beyond x_o2 (refused by the forward, naming nose_radius):
            candles, wax, fabrics, Ultem ("Not covered at all: wax candles ..., wake flames behind a sphere or
            hemisphere, ..., Nomex and Ultem, fabric", section 2), spheres in the wake configuration, rods in a
            concurrent or unstated configuration, and flat samples with no stated direction (`configuration` null).
            Each row carries `coverage` saying which rule applied.
  slice   u_coflow = u_coflow_cm_s / 100; u_jet = u_jet_m_s; burner_id = burner_id_mm / 1000; x_fuel; x_o2 = x_o2_cabin
            (left out where the log has none: "Where the log has no cabin oxygen ..., the forward refuses the case",
            phys_slice_w0.md 7.1); is_ethane / is_propane from fuel_hydrocarbon (methane: both 0; another fuel: none
            written); mdot_jet = mdot_jet_mg_s / 1e6. One row per logged line, as the case files.
  cld     (phys_cld_w0.md 7.1) x_fuel_final = the lowest x_fuel and v_coflow_final = the highest v_coflow_m_s over the
            segments of schedule_from_ignition_command (segment start and end values): "If the lowest fuel fraction or
            the highest coflow speed of the schedule is not the final one, give the lowest fraction and the highest
            speed in features 0 and 1"; window = at_cutoff.planned_fuel_off_from_onset_s, which is also the declared
            stop condition planned_fuel_off_from_onset; is_ethylene from fuel; x_fuel_stage, v_fuel_stage = the
            x_fuel and v_fuel_m_s of the first segment whose stage is constant_flow (at its start; no such segment:
            left out, refused). Reads `at_cutoff` (known at flame onset, the forecast time), not outcomes.

    python -I tools/wsp/ext/law_inputs.py build [--slug bre,...]

writes runs/dev/data_all/work/transfer_<slug>/law_<part>.jsonl for the input files of each slug
(train inputs and validation inputs, or the held inputs of same and daft). Outcome files are never opened.
"""
import argparse
import json
import math
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[2]
WORK = REPO / "runs/dev/data_all/work"

# bre_cases.py constants, unchanged (the MFC standard and molar masses that defined burner_exit_velocity_stp_m_s)
N_STP = 101325.0 / (8.314462618 * 273.15)
MW_KG = {"ethylene": 0.0280532, "methane": 0.0160425, "nitrogen": 0.0280134}

# input files per slug: (partition name, file). Outcome files are not listed and never read here.
INPUTS = {
    "bre": [("train", "cases_train.jsonl"), ("validation", "cases_validation_inputs.jsonl")],
    "spice": [("train", "cases_train.jsonl"), ("validation", "cases_validation_inputs.jsonl"),
              ("validation_given_flow", "cases_validation_inputs_given_flow.jsonl")],
    "efield": [("train", "cases_train.jsonl"), ("validation", "cases_validation_inputs.jsonl")],
    "samer": [("train", "cases_train.jsonl"), ("validation", "cases_validation_inputs.jsonl")],
    "same": [("held", "cases_held_inputs.jsonl")],
    "daft": [("held", "cases_held_inputs.jsonl")],
    "bass1": [("train", "cases_train.jsonl"), ("validation", "cases_validation_inputs.jsonl")],
    "slice": [("train", "cases_train.jsonl"), ("validation", "cases_validation_inputs.jsonl")],
    "cld": [("train", "cases_train.jsonl"), ("validation", "cases_validation_inputs.jsonl")],
}
FORBIDDEN = ("outcome", "outcomes", "prior", "prefix_3s", "pre_field")   # at_cutoff (cld) is the onset record, kept


def val(x):
    """The value of a case input (a number, or the value of a {value, unit, ...} dict)."""
    return x.get("value") if isinstance(x, dict) else x


def num(x):
    v = val(x)
    if isinstance(v, bool) or not isinstance(v, (int, float)) or not math.isfinite(float(v)):
        return None
    return float(v)


def put(row, name, v):
    """Set a derived feature; a missing value is left out (the forward then refuses, naming it)."""
    if v is not None:
        row[name] = v


def indicators(row, value, names):
    """0/1 indicators of a text condition; an unknown or missing value sets none (refused, not defaulted)."""
    if value not in names:
        return
    for k, n in names.items():
        if n is not None:
            row[n] = 1.0 if value == k else 0.0


def bre_row(i, stated_time):
    r = {}
    put(r, "burner_diameter", num(i.get("burner_diameter_m")))
    put(r, "x_o2", num(i.get("x_o2_ambient")))
    put(r, "pressure", num(i.get("pressure_pa")))
    x, v, fuel = num(i.get("x_fuel_burner_stream")), num(i.get("burner_exit_velocity_stp_m_s")), i.get("fuel_species")
    put(r, "x_fuel", x)
    if x is not None and v is not None and fuel in ("ethylene", "methane"):
        r["burner_mass_flux"] = v * N_STP * (x * MW_KG[fuel] + (1.0 - x) * MW_KG["nitrogen"])
    indicators(r, fuel, {"ethylene": "is_methane", "methane": "is_methane"})
    if fuel == "ethylene":
        r["is_methane"] = 0.0
    r["stated_time"] = stated_time
    return r


SPICE_FUELS = {"ethylene": None, "propane": "is_propane", "propylene": "is_propylene", "methane": "is_methane"}


def spice_row(c):
    i = c["inputs"]
    r = {}
    d, u = num(i.get("nozzle_id_mm")), num(i.get("u_air_cm_s"))
    put(r, "nozzle_diameter", d * 1e-3 if d is not None else None)
    put(r, "coflow_speed", u * 1e-2 if u is not None else None)
    put(r, "x_fuel", num(i.get("x_fuel")))
    hc = i.get("hydrocarbon")
    if hc in SPICE_FUELS:
        for n in ("is_propane", "is_propylene", "is_methane"):
            r[n] = 1.0 if SPICE_FUELS[hc] == n else 0.0
    g = c.get("inputs_given_flow")
    m = num(g.get("mdot_jet_mg_s")) if isinstance(g, dict) else None
    if m is not None:
        r["fuel_mass_flow"] = m * 1e-6
        given = True
    else:
        r["fuel_mass_flow"] = 1e-6   # declared centre; weight zero in the smoke-point targets (see docstring)
        given = False
    return r, given


def efield_rows(c):
    i = c["inputs"]
    base = {}
    put(base, "carbon_flow", num(i.get("carbon_flow_atoms_s")))
    put(base, "x_fuel", num(i.get("x_fuel")))
    if i.get("fuel") in ("methane", "ethylene"):
        base["is_ethylene"] = 1.0 if i["fuel"] == "ethylene" else 0.0
    if i.get("polarity") in ("negative", "positive"):
        base["mesh_positive"] = 1.0 if i["polarity"] == "positive" else 0.0
    put(base, "window", num(i.get("window_s")))
    vs = [abs(num(l.get("v_cmd"))) for l in (i.get("levels") or []) if num(l.get("v_cmd")) is not None]
    vmax = max(vs) if vs else None
    put(base, "sweep_v_max", vmax)
    test_row = dict(base)
    put(test_row, "voltage", vmax)
    level_rows = []
    for l in i.get("levels") or []:
        lr = dict(base)
        v = num(l.get("v_cmd"))
        put(lr, "voltage", abs(v) if v is not None else None)
        level_rows.append((l.get("level_id"), lr))
    return test_row, level_rows


SAMER_MATERIALS = {"lampwick": None, "kapton": "is_kapton", "teflon": "is_teflon", "silicone": "is_silicone",
                   "pyrell": "is_pyrell"}
SAME_MATERIALS = {"lampwick": None, "kapton": "is_kapton", "teflon": "is_teflon", "silicone": "is_silicone",
                  "dibutyl_phthalate": "is_dbp"}


def material_rows(i, materials, passthrough):
    r = {}
    m = val(i.get("material"))
    if m in materials:
        for n in materials.values():
            if n is not None:
                r[n] = 1.0 if materials[m] == n else 0.0
    for name in passthrough:
        x = i.get(name)
        if isinstance(x, dict) and x.get("value") is not None:
            r[name] = {"value": x["value"], "unit": x.get("unit")}
        elif num(x) is not None:
            r[name] = num(x)
    return r


def samer_row(i):
    r = material_rows(i, SAMER_MATERIALS, ("sample_temperature", "inlet_velocity", "net_aging_time", "heating_duration"))
    u = num(i.get("inlet_velocity"))
    if u is not None:
        r["no_flow"] = 1.0 if u == 0.0 else 0.0
    return r


def same_row(i):
    return material_rows(i, SAME_MATERIALS, ("inlet_velocity", "net_aging_time"))


DAFT_AIR = {"hepa": None, "cabin": "is_cabin", "velcro": "is_velcro", "bag": "is_bag"}


def daft_row(i):
    r = {}
    a = val(i.get("air"))
    if a in DAFT_AIR:
        for n in ("is_cabin", "is_velcro", "is_bag"):
            r[n] = 1.0 if DAFT_AIR[a] == n else 0.0
    x = num(i.get("dusttrak_mass_concentration"))
    if x is not None:
        r["dusttrak_mass_floored"] = max(x, 1e-9)
    return r


NOSE = {"pmma_rod", "pmma_sphere", "pmma_spherical_section"}
FLAT = {"pmma_film", "pmma_slab"}
NOSE_CONFIG = {"opposed", "stagnation", "forward stagnation"}
HEAT_TIME_DEFAULT_S = 200.0   # phys_bass1_w0.md 7.1: "If it is not recorded supply 200"
# D1 input convention (docs/wsp/research/phys_bass1_d1.md, Edits block): "flat PMMA thicker than 0.5 mm is supplied
# nose_radius = 0.02 (films stay at 0.01)". Used only for the D1 release (law_score.py sets it from the release id).
BASS1_D1_CONVENTION = False
D1_THICK_FLAT_M, D1_THICK_FLAT_RADIUS_M = 0.0005, 0.02


def flat_radius(c):
    th = num(c.get("thickness_m"))
    if BASS1_D1_CONVENTION and th is not None and th > D1_THICK_FLAT_M + 1e-12:
        return D1_THICK_FLAT_RADIUS_M
    return 0.01


def bass1_row(c):
    r, mat, conf = {}, c.get("material_class"), c.get("configuration")
    put(r, "x_o2", num(c.get("x_o2_initial")))
    u = num(c.get("u_cm_s_at_ignition"))
    if mat in NOSE and conf in NOSE_CONFIG and num(c.get("diameter_m")):
        coverage = "nose"
        r["nose_radius"] = num(c["diameter_m"]) / 2.0
        r["is_concurrent"] = 0.0
    elif mat in FLAT and conf in ("opposed", "concurrent"):
        coverage = "flat"
        r["nose_radius"] = flat_radius(c)
        r["is_concurrent"] = 1.0 if conf == "concurrent" else 0.0
    else:
        coverage = f"not_covered:{mat}|{conf}"
        return r, coverage
    put(r, "flow_speed", u * 1e-2 if u is not None else None)
    r["heat_time"] = HEAT_TIME_DEFAULT_S
    return r, coverage


def slice_row(i):
    r = {}
    u, d, m = num(i.get("u_coflow_cm_s")), num(i.get("burner_id_mm")), num(i.get("mdot_jet_mg_s"))
    put(r, "u_coflow", u / 100.0 if u is not None else None)
    put(r, "u_jet", num(i.get("u_jet_m_s")))
    put(r, "burner_id", d / 1000.0 if d is not None else None)
    put(r, "x_fuel", num(i.get("x_fuel")))
    put(r, "x_o2", num(i.get("x_o2_cabin")))
    hc = i.get("fuel_hydrocarbon")
    if hc in ("methane", "ethane", "propane"):
        r["is_ethane"] = 1.0 if hc == "ethane" else 0.0
        r["is_propane"] = 1.0 if hc == "propane" else 0.0
    put(r, "mdot_jet", m * 1e-6 if m is not None else None)
    return r


def cld_row(c):
    i, at = c.get("inputs") or {}, c.get("at_cutoff") or {}
    r = {}
    seg = i.get("schedule_from_ignition_command") or []
    xs = [num(v) for s in seg for v in (s.get("x_fuel") or [])]
    vs = [num(v) for s in seg for v in (s.get("v_coflow_m_s") or [])]
    if seg and None not in xs and None not in vs:
        r["x_fuel_final"] = min(xs)
        r["v_coflow_final"] = max(vs)
    w = num(at.get("planned_fuel_off_from_onset_s"))
    if w is not None:
        r["window"] = w
        r["planned_fuel_off_from_onset"] = w
    if i.get("fuel") in ("methane", "ethylene"):
        r["is_ethylene"] = 1.0 if i["fuel"] == "ethylene" else 0.0
    cf = [s for s in seg if s.get("stage") == "constant_flow"]
    if cf:
        put(r, "x_fuel_stage", num((cf[0].get("x_fuel") or [None])[0]))
        put(r, "v_fuel_stage", num((cf[0].get("v_fuel_m_s") or [None])[0]))
    return r


SUSTAIN_NOSE_CONFIG = {"stagnation", "forward stagnation", "concurrent"}


def bass1_sustain_row(c):
    """Inputs of the kept-burning target (phys_bass1_w0.md section 9; PROTOCOL 13.2 amendment). Same features as
    bass1_row, with these differences of coverage: rods "lit at the downstream end with the flame spreading against
    the flow along it" (configuration opposed) are refused, rods lit at the upstream end (stagnation, concurrent)
    are rod tips; flat PMMA thicker than 5 mm is refused. heat_time = planned igniter on-time + 60 s where a planned
    on-time is among the pre-ignition inputs; no bass1 case carries one, so every covered case gets 200 s."""
    r, mat, conf = {}, c.get("material_class"), c.get("configuration")
    put(r, "x_o2", num(c.get("x_o2_initial")))
    u = num(c.get("u_cm_s_at_ignition"))
    th = num(c.get("thickness_m"))
    if mat in NOSE and num(c.get("diameter_m")) and (conf in SUSTAIN_NOSE_CONFIG if mat == "pmma_rod" else conf in NOSE_CONFIG):
        coverage = "nose"
        r["nose_radius"] = num(c["diameter_m"]) / 2.0
        r["is_concurrent"] = 0.0
    elif mat in FLAT and conf in ("opposed", "concurrent") and (th is None or th <= 0.005 + 1e-12):
        coverage = "flat"
        r["nose_radius"] = flat_radius(c)
        r["is_concurrent"] = 1.0 if conf == "concurrent" else 0.0
    elif mat in FLAT and th is not None and th > 0.005 + 1e-12:
        return r, f"not_covered:flat thicker than 5 mm|{conf}"
    else:
        return r, f"not_covered:{mat}|{conf}"
    put(r, "flow_speed", u * 1e-2 if u is not None else None)
    r["heat_time"] = HEAT_TIME_DEFAULT_S
    return r, coverage


def cld2_row(c):
    """cld2 (docs/wsp/research/phys_cld2.md, "For the engineer"): the cld features plus
    `dilutes`: "1 if `x_fuel_final` < `x_fuel_initial`, else 0" and
    `velocity_ramp`: "1 if `schedule_from_ignition_command` has a stage `ramp_1`, `ramp_2` or `ramp_3`, else 0".
    Conditions of the script only (known before ignition)."""
    i = c.get("inputs") or {}
    r = cld_row(c)
    xf, xi = num(i.get("x_fuel_final")), num(i.get("x_fuel_initial"))
    if xf is not None and xi is not None:
        r["dilutes"] = 1.0 if xf < xi else 0.0
    seg = i.get("schedule_from_ignition_command")
    if isinstance(seg, list) and seg:
        r["velocity_ramp"] = 1.0 if any(s.get("stage") in ("ramp_1", "ramp_2", "ramp_3") for s in seg) else 0.0
    return r


def cld3_row(c):
    """cld3 (docs/wsp/research/phys_cld_study.md section 5, "For the engineer"): the cld features plus `t_severe`:
    "seconds from flame onset at which the schedule first holds both its lowest `x_fuel` and its highest
    `v_coflow_m_s` (the later of the two times, less `at_cutoff.onset_from_ignition_command_s`), and not less than 2 s".
    The schedule is linear within a segment between its start and end values, so a value is first held at a segment
    boundary: the start of the first segment, or the end of a segment. Conditions of the script and the onset only."""
    i, at = c.get("inputs") or {}, c.get("at_cutoff") or {}
    r = cld_row(c)
    seg = i.get("schedule_from_ignition_command") or []
    on = num(at.get("onset_from_ignition_command_s"))
    if not seg or on is None or "x_fuel_final" not in r or "v_coflow_final" not in r:
        return r
    pts_x = [(num(seg[0]["t0_s"]), num(seg[0]["x_fuel"][0]))] + [(num(s["t1_s"]), num(s["x_fuel"][1])) for s in seg]
    pts_v = [(num(seg[0]["t0_s"]), num(seg[0]["v_coflow_m_s"][0]))] + [(num(s["t1_s"]), num(s["v_coflow_m_s"][1])) for s in seg]
    if any(a is None or b is None for a, b in pts_x + pts_v):
        return r
    tx = min(t_ for t_, x in pts_x if x == r["x_fuel_final"])
    tv = min(t_ for t_, v in pts_v if v == r["v_coflow_final"])
    r["t_severe"] = max(max(tx, tv) - on, 2.0)
    return r


def efield3_extra(i):
    """efield3 (docs/wsp/research/phys_efield_study.md section 3, "For the engineer"): `is_gas_jet` "`burner` =
    `gas_jet_2.1mm`"; `still_coflow_burner` "`burner` = `coflow` and `u_coflow_m_s` = 0"; `strong_coflow`
    "`u_coflow_m_s` >= 0.1". Fields known before the test (docs/wsp/feedback/efield_planned_fields.md). A missing
    burner or coflow leaves the indicator out (refused, naming it); a gas-jet burner is not the coflow burner whatever
    the coflow field says."""
    r = {}
    b, u = i.get("burner"), num(i.get("u_coflow_m_s"))
    if b in ("gas_jet_2.1mm", "coflow"):
        r["is_gas_jet"] = 1.0 if b == "gas_jet_2.1mm" else 0.0
        if b == "gas_jet_2.1mm":
            r["still_coflow_burner"] = 0.0
        elif u is not None:
            r["still_coflow_burner"] = 1.0 if u == 0 else 0.0
    if u is not None:
        r["strong_coflow"] = 1.0 if u >= 0.1 else 0.0
    return r


def bass2_row(c, target="quench"):
    """bass2 (docs/wsp/research/phys_bass2.md, "For the engineer"): the five bass1 features ("every flat sample
    `nose_radius` = 0.01 (no flag)") plus `is_rod`: "1 if the sample is a rod (any flow configuration: stagnation,
    concurrent, opposed); 0 for spheres, spherical sections and flat samples" and `flat_thickness`: "flat samples: the
    thickness condition ... in metres, as installed; rods, spheres and sections: 0.0005". Coverage: rods in every stated
    configuration, spheres and sections with the nose to the flow, films and slabs with a stated direction; still refused:
    wake-lit spheres, candles and wax, fabric, Nomex, Ultem, fan-off ignition (outside the domain)."""
    r, mat, conf = {}, c.get("material_class"), c.get("configuration")
    put(r, "x_o2", num(c.get("x_o2_initial")))
    u, th = num(c.get("u_cm_s_at_ignition")), num(c.get("thickness_m"))
    if mat == "pmma_rod" and conf in ("stagnation", "concurrent", "opposed") and num(c.get("diameter_m")):
        cov, r["nose_radius"], r["is_concurrent"], r["is_rod"], r["flat_thickness"] = "nose", num(c["diameter_m"]) / 2.0, 0.0, 1.0, 0.0005
    elif mat in ("pmma_sphere", "pmma_spherical_section") and conf in NOSE_CONFIG and num(c.get("diameter_m")):
        cov, r["nose_radius"], r["is_concurrent"], r["is_rod"], r["flat_thickness"] = "nose", num(c["diameter_m"]) / 2.0, 0.0, 0.0, 0.0005
    elif mat in FLAT and conf in ("opposed", "concurrent") and th is not None:
        cov, r["nose_radius"], r["is_rod"], r["flat_thickness"] = "flat", 0.01, 0.0, th
        r["is_concurrent"] = 1.0 if conf == "concurrent" else 0.0
    else:
        return r, f"not_covered:{mat}|{conf}"
    put(r, "flow_speed", u * 1e-2 if u is not None else None)
    r["heat_time"] = HEAT_TIME_DEFAULT_S
    return r, cov


def bass3_row(c, target="quench"):
    """bass3 (docs/wsp/research/phys_bass_study.md section 4.3): the seven bass2 features, unchanged, plus
    `u_pot1`: "the case's `fan_curve` evaluated at fan display 46.5 (linear interpolation, as `fan_to_u(46.5,
    fan_curve)` in `bass1_cases.py`), divided by 100. For every sample kind" and
    `is_one_sided`: "flat samples: 1 if `sides_burning` is 1, else 0; rods, spheres, sections: 0".
    Condition fields only (the fan calibration of the burn and the sample's geometry)."""
    import bass1_cases as B
    r, cov = bass2_row(c, target)
    if cov in ("nose", "flat"):
        fc = c.get("fan_curve")
        if isinstance(fc, list) and fc:
            r["u_pot1"] = B.fan_to_u(46.5, fc) / 100.0
        if cov == "flat":
            sb = c.get("sides_burning")
            r["is_one_sided"] = 1.0 if sb == 1 else 0.0
        else:
            r["is_one_sided"] = 0.0
    return r, cov


FABRIC_THICKNESS = {"fabric_sibal": 0.00032, "fabric_nomex": 0.0003, "ultem_sheet": 0.00025}


def bass4_row(c, target="quench"):
    """bass4 (docs/wsp/research/phys_bass_study2.md section E, "For the engineer"): the nine bass3 features plus
    `is_sibal` "1 if `material_class` is `fabric_sibal`, else 0" and `is_nomex_ultem` "1 if `material_class` is
    `fabric_nomex` (any Nomex) or `ultem_sheet`, else 0". PMMA rows are bass3's, with both indicators 0.
    "Rows for fabrics and Ultem (kept burning only; `quench_u` stays refused for them): `x_o2` initial oxygen;
    `flow_speed` the speed at ignition; `nose_radius` 0.01; `heat_time` 200; `is_concurrent` 1 concurrent, 0 opposed,
    0.5 where the records state no direction; `is_rod` 0; `flat_thickness` 0.00032 (SIBAL), 0.0003 (Nomex), 0.00025
    (Ultem); `u_pot1` as for any case; `is_one_sided` 0. PMMA flat samples with no stated direction stay refused, as in
    bass3." Condition fields only."""
    import bass1_cases as B
    mat = c.get("material_class")
    if mat in FABRIC_THICKNESS:
        if target != "sustain":
            return {}, f"not_covered:{mat}|{c.get('configuration')}"
        r = {}
        put(r, "x_o2", num(c.get("x_o2_initial")))
        u = num(c.get("u_cm_s_at_ignition"))
        put(r, "flow_speed", u * 1e-2 if u is not None else None)
        conf = c.get("configuration")
        r.update(nose_radius=0.01, heat_time=HEAT_TIME_DEFAULT_S, is_rod=0.0, flat_thickness=FABRIC_THICKNESS[mat], is_one_sided=0.0,
                 is_concurrent=1.0 if conf == "concurrent" else 0.0 if conf == "opposed" else 0.5)
        fc = c.get("fan_curve")
        if isinstance(fc, list) and fc:
            r["u_pot1"] = B.fan_to_u(46.5, fc) / 100.0
        r["is_sibal"] = 1.0 if mat == "fabric_sibal" else 0.0
        r["is_nomex_ultem"] = 0.0 if mat == "fabric_sibal" else 1.0
        return r, "flat"
    r, cov = bass3_row(c, target)
    if cov in ("nose", "flat"):
        r["is_sibal"], r["is_nomex_ultem"] = 0.0, 0.0
    return r, cov


def bass5_row(c, target="quench"):
    """bass5 (docs/wsp/research/phys_bass_post_final.md, "For the engineer"; PROTOCOL 13.8): the eleven bass4 features
    plus `flat_width`: "PMMA films and slabs: `width_m` as installed. Everything else (rods, spheres, sections,
    fabrics, Ultem), and a flat PMMA sample with no width recorded: 0.02". Condition fields only."""
    r, cov = bass4_row(c, target)
    if cov in ("nose", "flat"):
        w = num(c.get("width_m"))
        r["flat_width"] = w if (c.get("material_class") in ("pmma_film", "pmma_slab") and w is not None) else 0.02
    return r, cov


def case_id(c):
    return str(c.get("case_id") or c.get("test") or (c.get("inputs") or {}).get("test"))


def law_rows(slug, cases, stated_time=60.0):
    """Rows for law_run.py: {case_id, test, group, kind, inputs: {feature: value}} (no outcome)."""
    out = []
    for c in cases:
        cid = case_id(c)
        i = c.get("inputs") or {}
        if slug == "bre":
            out.append(dict(case_id=cid, test=c.get("test"), group=c.get("day"), kind="test", inputs=bre_row(i, stated_time)))
        elif slug == "spice":
            r, given = spice_row(c)
            out.append(dict(case_id=cid, group=c.get("day"), kind="given_flow" if given else "smoke_point", inputs=r))
        elif slug == "efield2":   # phys_efield2.md: has_coflow "1 if `u_coflow_m_s` > 0, else 0"
            tr, levels = efield_rows(c)
            u = num(i.get("u_coflow_m_s"))
            extra = {} if u is None else {"has_coflow": 1.0 if u > 0 else 0.0}
            out.append(dict(case_id=f"{i.get('test')}", test=i.get("test"), group=i.get("day"), kind="test", inputs=dict(tr, **extra)))
            for lid, lr in levels:
                out.append(dict(case_id=f"{i.get('test')}|{lid}", test=i.get("test"), level_id=lid, group=i.get("day"), kind="level", inputs=dict(lr, **extra)))
        elif slug == "efield3":
            tr, levels = efield_rows(c)
            extra = efield3_extra(i)
            out.append(dict(case_id=f"{i.get('test')}", test=i.get("test"), group=i.get("day"), kind="test", inputs=dict(tr, **extra)))
            for lid, lr in levels:
                out.append(dict(case_id=f"{i.get('test')}|{lid}", test=i.get("test"), level_id=lid, group=i.get("day"), kind="level", inputs=dict(lr, **extra)))
        elif slug == "efield":
            t, levels = efield_rows(c)
            out.append(dict(case_id=f"{i.get('test')}", test=i.get("test"), group=i.get("day"), kind="test", inputs=t))
            for lid, lr in levels:
                out.append(dict(case_id=f"{i.get('test')}|{lid}", test=i.get("test"), level_id=lid, group=i.get("day"),
                                kind="level", inputs=lr))
        elif slug == "samer":
            out.append(dict(case_id=cid, group=c.get("group_id"), kind="row", inputs=samer_row(i)))
        elif slug == "same":
            out.append(dict(case_id=cid, group=c.get("group_id"), kind="row", inputs=same_row(i)))
        elif slug == "daft":
            out.append(dict(case_id=cid, group=c.get("group_id"), kind="row", inputs=daft_row(i)))
        elif slug == "slice":
            out.append(dict(case_id=cid, test=c.get("test"), group=c.get("day"), kind="line", inputs=slice_row(i)))
        elif slug == "cld3":
            out.append(dict(case_id=cid, test=c.get("test"), group=c.get("day"), kind="test", ignited=(c.get("at_cutoff") or {}).get("ignited"),
                            inputs=cld3_row(c)))
        elif slug == "cld2":
            out.append(dict(case_id=cid, test=c.get("test"), group=c.get("day"), kind="test", ignited=(c.get("at_cutoff") or {}).get("ignited"),
                            inputs=cld2_row(c)))
        elif slug == "cld":
            out.append(dict(case_id=cid, test=c.get("test"), group=c.get("day"), kind="test", ignited=(c.get("at_cutoff") or {}).get("ignited"),
                            inputs=cld_row(c)))
        elif slug == "bass1":
            r, cov = bass1_row(c)
            out.append(dict(case_id=cid, group=c.get("day_group"), kind="burn", coverage=cov,
                            heat_time_default=cov in ("nose", "flat"), inputs=r))
        else:
            raise ValueError(slug)
    return out


def read_inputs(path):
    rows = []
    for line in Path(path).read_text(encoding="utf-8").splitlines():
        if line.strip():
            c = json.loads(line)
            # conditions only: drop everything a forecast may not see before handing the row on
            rows.append({k: v for k, v in c.items() if k not in FORBIDDEN})
    return rows


def write(path, rows):
    with Path(path).open("w", encoding="utf-8", newline="\n") as f:
        for r in rows:
            f.write(json.dumps(r, ensure_ascii=False) + "\n")


def cmd_build(args):
    slugs = [s for s in args.slug.split(",") if s] or list(INPUTS)
    for slug in slugs:
        d = WORK / f"transfer_{slug}"
        for part, name in INPUTS[slug]:
            p = d / name
            if "outcome" in name:
                sys.exit("refusing an outcome file")
            rows = law_rows(slug, read_inputs(p))
            write(d / f"law_{part}.jsonl", rows)
            print(f"{slug:7s} {part:22s} {len(rows):5d} rows -> {d / f'law_{part}.jsonl'}")
    return 0


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    sub = ap.add_subparsers(dest="cmd", required=True)
    p = sub.add_parser("build")
    p.add_argument("--slug", default="")
    p.set_defaults(fn=cmd_build)
    a = ap.parse_args()
    return a.fn(a)


if __name__ == "__main__":
    sys.exit(main())
