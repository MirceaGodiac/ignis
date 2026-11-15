#!/usr/bin/env python3
"""Transcription check of the seventh-generation W0 law releases at the condition points of each dossier's
"registered expectations" (section 5 of docs/wsp/research/phys_<slug>_w0.md). No data: the condition points
are those of the dossier's tables, read from the dossier file itself; the forward is run on them through
tools/wsp/law_run.py; the forward value is put beside the dossier's hand-computed value. Nothing is changed
on a difference: it is reported.

    python -I tools/wsp/ext/law_expectations.py [--slug bre,...] [--out runs/dev/gen7/transcription_checks.md]
"""
import argparse
import json
import math
import re
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[2]
RES = REPO / "docs/wsp/research"
WORKDIR = REPO / "runs/dev/gen7/transcription_work"
ATM = 101325.0


CANDIDATE = {"spice": "lit-fable"}   # the section 5 tables of phys_spice_w0.md are the main law's


def release_of(slug, cand=None):
    cand = cand or CANDIDATE.get(slug, "w0-fable")
    return json.loads((REPO / f"configs/models/law/{slug}/{cand}/release.json").read_text(encoding="utf-8"))["id"]


def md_table(slug, header_start, occurrence=0):
    """Rows (lists of cell strings) of the n-th markdown table whose header line starts with header_start."""
    lines = (RES / f"phys_{slug}_w0.md").read_text(encoding="utf-8").splitlines()
    i = [k for k, l in enumerate(lines) if l.startswith(header_start)][occurrence]
    rows = []
    for l in lines[i + 2:]:
        if not l.startswith("|"):
            break
        rows.append([c.strip() for c in l.strip().strip("|").split("|")])
    return rows


def run(slug, cases, targets, times=None, cand=None):
    """Run law_run.py on synthetic condition points; returns {case_id: {target: summary + diagnostics}}."""
    WORKDIR.mkdir(parents=True, exist_ok=True)
    tag = f"{slug}_{cand or ''}_{targets.replace(',', '_')}"
    cpath, out = WORKDIR / f"{tag}_points.jsonl", WORKDIR / f"{tag}_forward.jsonl"
    with cpath.open("w", encoding="utf-8", newline="\n") as f:
        for c in cases:
            f.write(json.dumps(c) + "\n")
    cmd = [sys.executable, str(REPO / "tools/wsp/law_run.py"), "--release", release_of(slug, cand), "--cases", str(cpath),
           "--out", str(out), "--targets", targets]
    if times:
        cmd += ["--times", ",".join(str(t) for t in times)]
    r = subprocess.run(cmd, cwd=REPO, capture_output=True, text=True)
    if r.returncode != 0:
        sys.exit(r.stderr or r.stdout)
    res = {}
    work = out.with_name(out.name + ".work") / "artifact"
    for n, line in enumerate(out.read_text(encoding="utf-8").splitlines(), 1):
        row = json.loads(line)
        for t, s in row["targets"].items():
            safe = "".join(ch if ch.isalnum() or ch in "-_." else "_" for ch in row["case_id"])
            a = json.loads((work / f"{n:06d}_{safe}.{t}.json").read_text(encoding="utf-8"))
            s["diag"] = a["diagnostics"].get("outcome", {})
        res[row["case_id"]] = row["targets"]
    return res


def f(x, nd=3):
    if x is None:
        return "-"
    if isinstance(x, str):
        return x
    if x == 0:
        return "0"
    mag = math.floor(math.log10(abs(x)))
    return f"{x:.{max(0, nd - 1 - mag)}f}" if -3 < mag < 6 else f"{x:.{nd}g}"


def rel(a, b):
    """Relative difference of the forward value a from the dossier value b."""
    try:
        b = float(b)
    except (TypeError, ValueError):
        return None
    if a is None:
        return None
    if b == 0:
        return abs(a)
    return a / b - 1.0


def cell_num(s):
    m = re.match(r"^\s*([-+]?\d[\d\s]*\.?\d*(?:e[-+]?\d+)?)", s.replace(" ", ""))
    return float(m.group(1).replace(" ", "")) if m else None


# ------------------------------------------------------------------------------------------------- bre
def check_bre():
    rows = md_table("bre", "| d mm | fuel | X_O2 |")
    cases, meta = [], []
    for k, r in enumerate(rows):
        d, fuel, x, p, flux = r[0], r[1], float(r[2]), float(r[3]), float(r[4])
        base = dict(burner_diameter=float(d) / 1000.0, burner_mass_flux=flux / 1000.0, x_o2=x, pressure=p * ATM,
                    x_fuel=0.5 if "50 %" in fuel else 1.0, is_methane=1.0 if fuel.startswith("CH4") else 0.0)
        for st in (10, 30, 60, 120, 3000):
            cases.append(dict(case_id=f"p{k}_t{st}", inputs=dict(base, stated_time=float(st))))
        meta.append((k, r))
    lasts = run("bre", cases, "lasts")
    pts = [dict(case_id=f"p{k}", inputs=dict(c["inputs"])) for k, c in zip(range(len(rows)), cases[::5])]
    other = run("bre", pts, "flame_end,height_at_end", times=[30.0])
    out = ["| d mm | fuel | X_O2 | p atm | flux | tau dossier / forward | P10 | P30 | P60 | P120 | P_inf (dossier) / P at 3000 s (forward) | h mm |",
           "|---|---|---|---|---|---|---|---|---|---|---|---|"]
    worst = {}
    for k, r in meta:
        tau = math.exp(other[f"p{k}"]["flame_end"]["diag"]["clocks"][0]["ln_tau"]) if "clocks" in other[f"p{k}"]["flame_end"]["diag"] else None
        h = other[f"p{k}"]["height_at_end"]["quantiles"]["0.5"] * 1000.0
        ps = [lasts[f"p{k}_t{st}"]["lasts"]["probability"] for st in (10, 30, 60, 120, 3000)]
        vals = [(tau, r[8], "tau"), (ps[0], r[9], "P10"), (ps[1], r[10], "P30"), (ps[2], r[11], "P60"),
                (ps[3], r[12], "P120"), (ps[4], r[13], "P_inf"), (h, r[14], "h")]
        cells = []
        for a, b, name in vals:
            cells.append(f"{b} / {f(a)}")
            if name in ("tau", "h"):
                e = rel(a, b)
                worst[name] = max(worst.get(name, 0.0), abs(e) if e is not None else 0.0)
            else:
                e = abs(a - float(b))
                worst[name] = max(worst.get(name, 0.0), e)
        out.append("| " + " | ".join(r[:5] + cells) + " |")
    return out, worst


# ------------------------------------------------------------------------------------------------- spice
SPICE_FUEL = {"Ethylene": (1.0, {}), "Propane": (1.0, {"is_propane": 1.0}), "Propylene": (1.0, {"is_propylene": 1.0}),
              "75 % propylene": (0.75, {"is_propylene": 1.0}), "50 % propylene": (0.5, {"is_propylene": 1.0})}


def spice_inputs(fuel, d_mm, u_cm_s, mdot_mg_s=1.0, methane=False):
    x, ind = SPICE_FUEL.get(fuel, (1.0, {}))
    r = dict(nozzle_diameter=d_mm * 1e-3, coflow_speed=u_cm_s * 1e-2, fuel_mass_flow=mdot_mg_s * 1e-6, x_fuel=x,
             is_propane=0.0, is_propylene=0.0, is_methane=1.0 if methane else 0.0)
    r.update(ind)
    return r


def check_spice():
    rows = md_table("spice", "| Fuel | d (mm) |")
    speeds = [5, 10, 20, 40, 65]
    cases, keys = [], []
    for r in rows:
        if r[0] == "Methane":
            continue
        for j, u in enumerate(speeds):
            cid = f"{r[0]}|{r[1]}|{u}"
            cases.append(dict(case_id=cid, inputs=spice_inputs(r[0], float(r[1]), u)))
            keys.append((cid, r[0], r[1], u, r[2 + j]))
    # flow invariance of the smoke-point targets (weight zero on fuel_mass_flow): the same points at 5 mg/s
    cases5 = [dict(case_id=c["case_id"] + "|m5", inputs=dict(c["inputs"], fuel_mass_flow=5e-6)) for c in cases]
    res = run("spice", cases + cases5, "smoke_point_length,smoke_point_fuel_flow")
    out = ["| Fuel | d mm | u cm/s | dossier L_sp mm / flow mg/s | forward L_sp mm / flow mg/s | rel. diff L / flow |", "|---|---|---|---|---|---|"]
    worst = {"L_sp": 0.0, "mdot_sp": 0.0, "flow_invariance": 0.0}
    for cid, fuel, d, u, cell in keys:
        L = res[cid]["smoke_point_length"]["quantiles"]["0.5"] * 1000
        m = res[cid]["smoke_point_fuel_flow"]["quantiles"]["0.5"] * 1e6
        L5 = res[cid + "|m5"]["smoke_point_length"]["quantiles"]["0.5"] * 1000
        m5 = res[cid + "|m5"]["smoke_point_fuel_flow"]["quantiles"]["0.5"] * 1e6
        worst["flow_invariance"] = max(worst["flow_invariance"], abs(L5 / L - 1), abs(m5 / m - 1))
        parts = [p.strip() for p in cell.replace("(floor)", "").split("/")]
        eL, em = rel(L, parts[0]), rel(m, parts[1])
        worst["L_sp"] = max(worst["L_sp"], abs(eL))
        worst["mdot_sp"] = max(worst["mdot_sp"], abs(em))
        out.append(f"| {fuel} | {d} | {u} | {cell} | {f(L)} / {f(m)} | {eL:+.3f} / {em:+.3f} |")
    # statements 4 and 5 of section 5
    pts = [("eth_1", "Ethylene", 1.6, 20, 1.0, False, None, 0.01), ("eth_2", "Ethylene", 1.6, 20, 2.0, False, None, 0.43),
           ("eth_3", "Ethylene", 1.6, 20, 3.0, False, None, 0.93), ("pro_04", "Propylene", 1.6, 20, 0.4, False, None, 0.04),
           ("pro_08", "Propylene", 1.6, 20, 0.8, False, None, 0.75), ("me_1", "Ethylene", 1.6, 20, 1.0, True, 22.0, None),
           ("me_3", "Ethylene", 1.6, 20, 3.0, True, 78.0, None)]
    res2 = run("spice", [dict(case_id=p[0], inputs=spice_inputs(p[1], p[2], p[3], p[4], p[5])) for p in pts],
               "flame_length,emits_soot")
    out += ["", "Section 5, statements 4 and 5 (1.6 mm nozzle, 20 cm/s):", "",
            "| point | flow mg/s | dossier | forward |", "|---|---|---|---|"]
    for p in pts:
        name = ("methane" if p[5] else p[1])
        if p[6] is not None:
            v = res2[p[0]]["flame_length"]["quantiles"]["0.5"] * 1000
            out.append(f"| {name} flame length mm | {p[4]} | {p[6]} | {f(v)} |")
        else:
            v = res2[p[0]]["emits_soot"]["probability"]
            out.append(f"| {name} P(emits soot) | {p[4]} | {p[7]} | {f(v, 2)} |")
    return out, worst


# ------------------------------------------------------------------------------------------------- efield
def ef(cf, x, eth, pos, v, vmax, win=100.0):
    return dict(carbon_flow=cf, x_fuel=x, is_ethylene=eth, mesh_positive=pos, voltage=v, sweep_v_max=vmax, window=win)


def check_efield():
    out, worst = [], {}
    # saturation table: peak_ion_current, peak_voltage (neg / pos)
    rows = md_table("efield", "| Fuel fraction | 0.5e19 |")
    cfs = [0.5e19, 1e19, 2e19, 4e19]
    cases = []
    for r in rows:
        for j, cf in enumerate(cfs):
            for pos in (0.0, 1.0):
                cases.append(dict(case_id=f"{r[0]}|{cf}|{pos}", inputs=ef(cf, float(r[0]), 0.0, pos, 5000.0, 5000.0)))
    res = run("efield", cases, "peak_ion_current,peak_voltage")
    out += ["Saturation current and its voltage (mesh negative / positive):", "",
            "| x_fuel | carbon flow | dossier | forward I uA at V neg / V pos kV |", "|---|---|---|---|"]
    for r in rows:
        for j, cf in enumerate(cfs):
            i0 = res[f"{r[0]}|{cf}|0.0"]["peak_ion_current"]["quantiles"]["0.5"] * 1e6
            v0 = res[f"{r[0]}|{cf}|0.0"]["peak_voltage"]["quantiles"]["0.5"] / 1000
            v1 = res[f"{r[0]}|{cf}|1.0"]["peak_voltage"]["quantiles"]["0.5"] / 1000
            i1 = res[f"{r[0]}|{cf}|1.0"]["peak_ion_current"]["quantiles"]["0.5"] * 1e6
            m = re.match(r"([\d.]+) uA at ([\d.]+) / ([\d.]+) kV", r[1 + j])
            di, dv0, dv1 = float(m.group(1)), float(m.group(2)), float(m.group(3))
            worst["peak_I"] = max(worst.get("peak_I", 0), abs(i0 / di - 1), abs(i1 / di - 1))
            worst["peak_V"] = max(worst.get("peak_V", 0), abs(v0 / dv0 - 1), abs(v1 / dv1 - 1))
            out.append(f"| {r[0]} | {cf:.1e} | {r[1 + j]} | {f(i0)} (pos {f(i1)}) at {f(v0)} / {f(v1)} |")
    # T1 rows
    volts = [500, 1000, 2000, 3000, 4000, 5000, 6000, 8000, 10000]
    t1a = md_table("efield", "| 0.5 kV | 1 kV |")
    lines = (RES / "phys_efield_w0.md").read_text(encoding="utf-8").splitlines()
    hdr = [k for k, l in enumerate(lines) if l.startswith("| 0.5 kV | 1 kV |")]
    t1b = [[c.strip() for c in lines[hdr[1] + 2].strip().strip("|").split("|")]]
    cases = []
    for v in volts:
        for pos in (0.0, 1.0):
            cases.append(dict(case_id=f"a|{v}|{pos}", inputs=ef(2e19, 1.0, 0.0, pos, float(v), 10000.0)))
        cases.append(dict(case_id=f"b|{v}", inputs=ef(1e19, 0.4, 0.0, 0.0, float(v), 10000.0)))
    res = run("efield", cases, "ion_current")
    out += ["", "T1 median current uA, pure fuel at 2e19 atoms/s (neg / pos), and 40 % fuel at 1e19, neg (ln-sd):", "",
            "| kV | dossier pure | forward pure | dossier 40 % | forward 40 % (ln-sd) |", "|---|---|---|---|---|"]
    for j, v in enumerate(volts):
        a0 = res[f"a|{v}|0.0"]["ion_current"]["quantiles"]["0.5"] * 1e6
        a1 = res[f"a|{v}|1.0"]["ion_current"]["quantiles"]["0.5"] * 1e6
        b = res[f"b|{v}"]["ion_current"]
        bm, bs = b["quantiles"]["0.5"] * 1e6, b["diag"].get("ln_sd")
        d0, d1 = [float(x) for x in t1a[0][j].split("/")]
        db, dbs = float(t1b[0][j].split("(")[0]), float(t1b[0][j].split("(")[1].rstrip(")"))
        worst["T1"] = max(worst.get("T1", 0), abs(a0 / d0 - 1), abs(a1 / d1 - 1), abs(bm / db - 1))
        worst["T1_lnsd_abs"] = max(worst.get("T1_lnsd_abs", 0), abs(bs - dbs))
        out.append(f"| {v / 1000:g} | {t1a[0][j]} | {f(a0, 2)} / {f(a1, 2)} | {t1b[0][j]} | {f(bm, 2)} ({f(bs, 3)}) |")
    # T2 table, P(out before 100 s)
    rows = md_table("efield", "| Fuel | Fuel fraction | 1e19 |")
    cases = []
    for r in rows:
        eth = 1.0 if r[0] == "Ethylene" else 0.0
        x = float(r[1])
        for tag, cf, vmax in (("s", 1e19, 3000.0), ("s10", 1e19, 10000.0), ("l3", 4e19, 3000.0), ("l10", 4e19, 10000.0)):
            for pos in (0.0, 1.0):
                cases.append(dict(case_id=f"{r[0]}|{r[1]}|{tag}|{pos}", inputs=ef(cf, x, eth, pos, vmax, vmax, 100.0)))
    res = run("efield", cases, "flame_end", times=[100.0])
    cdf_end = lambda s: 1.0 - s["flame_end"]["cdf"]["active"][0]  # noqa: E731
    out += ["", "T2, P(out before the end of a 100 s programme), neg / pos; small flame at 3 kV and at 10 kV (dossier: any from 3 kV):", "",
            "| Fuel | x | dossier 1e19 | forward 1e19 to 3 kV | forward 1e19 to 10 kV | dossier 4e19 to 3 kV | forward | dossier 4e19 to 10 kV | forward |",
            "|---|---|---|---|---|---|---|---|---|"]
    for r in rows:
        g = lambda tag: [cdf_end(res[f"{r[0]}|{r[1]}|{tag}|{p}"]) for p in (0.0, 1.0)]  # noqa: E731
        vals = {t: g(t) for t in ("s", "s10", "l3", "l10")}
        for t, cell in (("s", r[2]), ("s10", r[2]), ("l3", r[3]), ("l10", r[4])):
            dn, dp = [float(z) for z in cell.split("/")]
            worst["T2"] = max(worst.get("T2", 0), abs(vals[t][0] - dn), abs(vals[t][1] - dp))
        fm = lambda v: f"{v[0]:.2f} / {v[1]:.2f}"  # noqa: E731
        out.append(f"| {r[0]} | {r[1]} | {r[2]} | {fm(vals['s'])} | {fm(vals['s10'])} | {r[3]} | {fm(vals['l3'])} | {r[4]} | {fm(vals['l10'])} |")
    # time series
    ts1 = [5, 10, 20, 30, 45, 60, 90]
    d1 = [0.00, 0.01, 0.04, 0.09, 0.22, 0.39, 0.68]
    ts2 = [10, 30, 60, 90]
    d2 = [0.002, 0.011, 0.032, 0.060]
    res = run("efield", [dict(case_id="m30", inputs=ef(4e19, 0.3, 0.0, 0.0, 10000.0, 10000.0)),
                         dict(case_id="m100", inputs=ef(1e19, 1.0, 0.0, 0.0, 10000.0, 10000.0))],
              "flame_end", times=sorted(set(ts1 + ts2)))
    out += ["", "T2 against time, methane 30 % at 4e19 to 10 kV in 100 s, neg; pure methane at 1e19 (taken to 10 kV):", "",
            "| case | t s | dossier | forward |", "|---|---|---|---|"]
    for cid, ts, ds in (("m30", ts1, d1), ("m100", ts2, d2)):
        s = res[cid]["flame_end"]
        for t, dv in zip(ts, ds):
            v = 1.0 - s["cdf"]["active"][s["times_s"].index(float(t))]
            worst["T2_time"] = max(worst.get("T2_time", 0), abs(v - dv))
            out.append(f"| {cid} | {t} | {dv} | {v:.3f} |")
    return out, worst


# ------------------------------------------------------------------------------------------------- samer / same
SAMER_MAT = {"Lamp wick": ({}, 560.0), "Kapton": ({"is_kapton": 1.0}, 830.0), "Teflon": ({"is_teflon": 1.0}, 790.0),
             "Silicone": ({"is_silicone": 1.0}, 670.0), "Pyrell": ({"is_pyrell": 1.0}, 570.0)}


def samer_inputs(mat, u_cm_s, t, T=None, th=60.0):
    ind, tref = SAMER_MAT[mat]
    r = dict(is_kapton=0.0, is_teflon=0.0, is_silicone=0.0, is_pyrell=0.0, sample_temperature=T or tref,
             inlet_velocity=u_cm_s / 100.0, no_flow=1.0 if u_cm_s == 0 else 0.0, net_aging_time=t, heating_duration=th)
    r.update(ind)
    return r


def check_samer():
    rows = md_table("samer", "| Material | Flow cm/s | d10 at 40 s |")
    cases, keys = [], []
    for r in rows:
        flows = [2.0, 1.0] if r[1] == "2 or 1" else [float(r[1])]
        for u in flows:
            for t in (40.0, 750.0, 1900.0):
                cid = f"{r[0]}|{u}|{t}"
                cases.append(dict(case_id=cid, inputs=samer_inputs(r[0], u, t)))
            keys.append((r, u))
    res = run("samer", cases, "d10,d30,iss_full_scale")
    out = ["| Material | flow | d10 40 / 750 / 1900 s nm (dossier ; forward) | D30 40 / 1900 (dossier ; forward) | ln-sd 40 / 1900 (dossier ; forward) | P 40 / 1900 (dossier ; forward) |",
           "|---|---|---|---|---|---|"]
    worst = {}
    for r, u in keys:
        g = lambda t, tg: res[f"{r[0]}|{u}|{t}"][tg]  # noqa: E731
        d10 = [g(t, "d10")["quantiles"]["0.5"] * 1e9 for t in (40.0, 750.0, 1900.0)]
        d30 = [g(t, "d30")["quantiles"]["0.5"] * 1e9 for t in (40.0, 1900.0)]
        sd = [g(t, "d10")["diag"]["ln_sd"] for t in (40.0, 1900.0)]
        p = [g(t, "iss_full_scale")["probability"] for t in (40.0, 1900.0)]
        dd10 = [float(r[2]), float(r[3]), float(r[4])]
        dd30 = [float(r[5]), float(r[6])]
        dsd = [float(z) for z in r[7].split("/")]
        dp = [float(r[8]), float(r[9])]
        worst["d10"] = max([worst.get("d10", 0)] + [abs(a / b - 1) for a, b in zip(d10, dd10)])
        worst["d30"] = max([worst.get("d30", 0)] + [abs(a / b - 1) for a, b in zip(d30, dd30)])
        worst["ln_sd_abs"] = max([worst.get("ln_sd_abs", 0)] + [abs(a - b) for a, b in zip(sd, dsd)])
        worst["P_abs"] = max([worst.get("P_abs", 0)] + [abs(a - b) for a, b in zip(p, dp)])
        fl = r[1] if r[1] != "2 or 1" else f"{u:g} (row '2 or 1')"
        out.append(f"| {r[0]} | {fl} | {' / '.join(r[2:5])} ; {' / '.join(f(x) for x in d10)} | {r[5]} / {r[6]} ; {' / '.join(f(x) for x in d30)} "
                   f"| {r[7]} ; {' / '.join(f'{x:.2f}' for x in sd)} | {r[8]} / {r[9]} ; {' / '.join(f'{x:.2f}' for x in p)} |")
    # Kapton and Teflon at 1 cm/s: P 0.51 and 0.39 (40 s), 0.65 and 0.54 (1900 s); core quantiles; temperature lines; checks
    extra = [dict(case_id="k1_40", inputs=samer_inputs("Kapton", 1.0, 40.0)), dict(case_id="t1_40", inputs=samer_inputs("Teflon", 1.0, 40.0)),
             dict(case_id="k1_1900", inputs=samer_inputs("Kapton", 1.0, 1900.0)), dict(case_id="t1_1900", inputs=samer_inputs("Teflon", 1.0, 1900.0)),
             dict(case_id="kcore", inputs=samer_inputs("Kapton", 8.1, 40.0)),
             dict(case_id="chk_lw", inputs=samer_inputs("Lamp wick", 8.1, 40.0)),
             dict(case_id="chk_k", inputs=samer_inputs("Kapton", 0.0, 1900.0, T=830.0)),
             dict(case_id="heat120", inputs=samer_inputs("Lamp wick", 8.1, 40.0, th=120.0))]
    temps = [("Lamp wick", "141 / 0.65, 166 / 0.75, 194 / 0.83"), ("Kapton", "79 / 0.10, 94 / 0.15, 114 / 0.22"),
             ("Teflon", "89 / 0.06, 104 / 0.10, 126 / 0.15"), ("Silicone", "175 / 0.59, 205 / 0.70, 241 / 0.79"),
             ("Pyrell", "163 / 0.57, 191 / 0.68, 224 / 0.77")]
    for m, _ in temps:
        for dT in (-40.0, 0.0, 40.0):
            extra.append(dict(case_id=f"T|{m}|{dT}", inputs=samer_inputs(m, 8.1, 40.0, T=SAMER_MAT[m][1] + dT)))
    rx = run("samer", extra, "d10,iss_full_scale")
    P = lambda c: rx[c]["iss_full_scale"]["probability"]  # noqa: E731
    D = lambda c: rx[c]["d10"]["quantiles"]["0.5"] * 1e9  # noqa: E731
    out += ["", "Other registered points:", "", "| point | dossier | forward |", "|---|---|---|",
            f"| Kapton, Teflon at 1 cm/s, P at 40 s | 0.51, 0.39 | {P('k1_40'):.2f}, {P('t1_40'):.2f} |",
            f"| Kapton, Teflon at 1 cm/s, P at 1900 s | 0.65, 0.54 | {P('k1_1900'):.2f}, {P('t1_1900'):.2f} |"]
    d = rx["kcore"]["d10"]["diag"]
    core = [math.exp(d["ln_median"] + d["ln_sd"] * z) * 1e9 for z in (-1.6449, -0.6745, 0.0, 0.6745, 1.6449)]
    mix = [rx["kcore"]["d10"]["quantiles"][k] * 1e9 for k in ("0.05", "0.25", "0.5", "0.75", "0.95")]
    out.append(f"| Kapton 8.1 cm/s 40 s, core quantiles 5..95 % nm | 49, 72, 94, 123, 181 | {', '.join(f(x) for x in core)} (with the tail component: {', '.join(f(x) for x in mix)}) |")
    for m, txt in temps:
        vals = ", ".join(f"{D(f'T|{m}|{dT}'):.0f} / {P(f'T|{m}|{dT}'):.2f}" for dT in (-40.0, 0.0, 40.0))
        out.append(f"| {m}, 40 K below / at / above T_ref, d10 nm / P | {txt} | {vals} |")
    out.append(f"| heating 120 s over 60 s, d10 ratio | 1.07 | {D('heat120') / D('chk_lw'):.3f} |")
    out.append(f"| check 7.4: lamp wick 560 K 8.1 cm/s 40 s: d10, logit | 166 nm, 1.10 | {D('chk_lw'):.1f} nm, {math.log(P('chk_lw') / (1 - P('chk_lw'))):.3f} |")
    out.append(f"| check 7.4: Kapton 830 K 0 cm/s 1900 s: d10, P | 315 nm, 0.74 | {D('chk_k'):.1f} nm, {P('chk_k'):.3f} |")
    return out, worst


SAME_MAT = {"Lamp wick": {}, "Kapton": {"is_kapton": 1.0}, "Teflon": {"is_teflon": 1.0}, "Silicone": {"is_silicone": 1.0},
            "DBP": {"is_dbp": 1.0}}


def same_inputs(mat, u_cm_s, t):
    r = dict(is_kapton=0.0, is_teflon=0.0, is_silicone=0.0, is_dbp=0.0, inlet_velocity=u_cm_s / 100.0, net_aging_time=t)
    r.update(SAME_MAT[mat])
    return r


def check_same():
    rows = md_table("same", "| Material | Flow cm/s | d10 at 40 s |")
    cases = []
    for r in rows:
        for t in (40.0, 750.0, 1800.0):
            cases.append(dict(case_id=f"{r[0]}|{r[1]}|{t}", inputs=same_inputs(r[0], float(r[1]), t)))
    cases += [dict(case_id="chk_lw", inputs=same_inputs("Lamp wick", 8.1, 40.0)), dict(case_id="chk_dbp", inputs=same_inputs("DBP", 5.0, 1800.0))]
    res = run("same", cases, "d10,d30,iss_full_scale")
    out = ["| Material | flow cm/s | d10 40 / 750 / 1800 s nm (dossier ; forward) | D30 40 / 1800 (dossier ; forward) | P 40 / 1800 (dossier ; forward) |",
           "|---|---|---|---|---|"]
    worst = {}
    for r in rows:
        g = lambda t, tg: res[f"{r[0]}|{r[1]}|{t}"][tg]  # noqa: E731
        d10 = [g(t, "d10")["quantiles"]["0.5"] * 1e9 for t in (40.0, 750.0, 1800.0)]
        d30 = [g(t, "d30")["quantiles"]["0.5"] * 1e9 for t in (40.0, 1800.0)]
        p = [g(t, "iss_full_scale")["probability"] for t in (40.0, 1800.0)]
        worst["d10"] = max([worst.get("d10", 0)] + [abs(a / float(b) - 1) for a, b in zip(d10, r[2:5])])
        worst["d30"] = max([worst.get("d30", 0)] + [abs(a / float(b) - 1) for a, b in zip(d30, r[5:7])])
        worst["P_abs"] = max([worst.get("P_abs", 0)] + [abs(a - float(b)) for a, b in zip(p, r[7:9])])
        out.append(f"| {r[0]} | {r[1]} | {' / '.join(r[2:5])} ; {' / '.join(f(x) for x in d10)} | {r[5]} / {r[6]} ; {' / '.join(f(x) for x in d30)} "
                   f"| {r[7]} / {r[8]} ; {' / '.join(f'{x:.2f}' for x in p)} |")
    P = lambda c: res[c]["iss_full_scale"]["probability"]  # noqa: E731
    D = lambda c: res[c]["d10"]["quantiles"]["0.5"] * 1e9  # noqa: E731
    out += ["", "| check of 7.2 | dossier | forward |", "|---|---|---|",
            f"| lamp wick 8.1 cm/s 40 s: d10, logit | 166 nm, 1.10 | {D('chk_lw'):.1f} nm, {math.log(P('chk_lw') / (1 - P('chk_lw'))):.3f} |",
            f"| DBP 5 cm/s 1800 s: d10, P | 492 nm, 0.92 | {D('chk_dbp'):.1f} nm, {P('chk_dbp'):.3f} |"]
    return out, worst


# ------------------------------------------------------------------------------------------------- daft
def check_daft():
    rows = md_table("daft", "| Air | DustTrak reading, mg/m3 |")
    cases = []
    for k, r in enumerate(rows):
        air = r[0]
        x = 1e-6 if r[1] == "any" else cell_num(r[1]) * 1e-6
        ind = dict(is_cabin=1.0 if air == "cabin" else 0.0, is_velcro=1.0 if air == "velcro" else 0.0, is_bag=1.0 if air == "bag" else 0.0)
        cases.append(dict(case_id=f"r{k}", inputs=dict(ind, dusttrak_mass_floored=max(x, 1e-9))))
    res = run("daft", cases, "number_concentration")
    out = ["| Air | reading mg/m3 | dossier 5 / 25 / 50 / 75 / 95 % per cm3 | forward core (from ln_median, ln_sd) | forward quantiles with the 15 % tail |",
           "|---|---|---|---|---|"]
    worst = {"median": 0.0, "core": 0.0}
    for k, r in enumerate(rows):
        s = res[f"r{k}"]["number_concentration"]
        d = s["diag"]
        core = [math.exp(d["ln_median"] + d["ln_sd"] * z) / 1e6 for z in (-1.6449, -0.6745, 0.0, 0.6745, 1.6449)]
        mix = [s["quantiles"][q] / 1e6 for q in ("0.05", "0.25", "0.5", "0.75", "0.95")]
        dv = [cell_num(c) for c in r[2:7]]
        worst["median"] = max(worst["median"], abs(core[2] / dv[2] - 1))
        worst["core"] = max([worst["core"]] + [abs(a / b - 1) for a, b in zip(core, dv)])
        out.append(f"| {r[0]} | {r[1]} | {' / '.join(r[2:7])} | {' / '.join(f(x) for x in core)} | {' / '.join(f(x) for x in mix)} |")
    return out, worst


# ------------------------------------------------------------------------------------------------- bass1
def b1(x, u_cm_s, r_cm=1.0, t=200.0, conc=0.0):
    return dict(x_o2=x, flow_speed=u_cm_s / 100.0, nose_radius=r_cm / 100.0, heat_time=t, is_concurrent=conc)


def body(label):
    m = re.search(r"R = ([\d.]+)", label)
    tm = re.search(r"([\d]+) s", label)
    return float(m.group(1)), (float(tm.group(1)) if tm else 200.0)


def check_bass1():
    out, worst = [], {}

    def grid(title, header, occ, target, rowfn, colvals, rel_=False, scale=1.0, kind="prob"):
        rows = md_table("bass1", header, occ)
        cases, keys = [], []
        for k, r in enumerate(rows):
            spec = rowfn(r[0])
            if spec is None:
                continue
            for j, u in enumerate(colvals):
                cid = f"{k}|{j}"
                cases.append(dict(case_id=cid, inputs=spec(u)))
                keys.append((k, j, r))
        res = run("bass1", cases, target)
        out.extend(["", title, "", "| row | " + " | ".join(str(u) for u in colvals) + " |", "|---" * (len(colvals) + 1) + "|"])
        cells = {}
        for k, j, r in keys:
            s = res[f"{k}|{j}"][target]
            v = s["probability"] if kind == "prob" else s["quantiles"]["0.5"] * scale
            d = cell_num(r[1 + j]) if r[1 + j] and not r[1 + j].startswith("below") else None
            if d is not None and " to " not in r[1 + j].split("(")[0]:
                e = abs(v - d) if kind == "prob" else abs(v / d - 1)
                worst[target] = max(worst.get(target, 0.0), e)
            cells.setdefault(k, [r[0]]).append(f"{r[1 + j]} ; {f(v, 2 if kind == 'prob' else 3)}")
        for k in sorted(cells):
            out.append("| " + " | ".join(cells[k]) + " |")

    def e1(label):
        if "rod" not in label and "sphere" not in label:
            return None
        rr, tt = body(label)
        return lambda u: b1(0.18, u, rr, tt)
    grid("E1 oxygen limit %, dossier ; forward median", "| Sample, time since ignition |", 0, "oxygen_limit", e1,
         [0.5, 1, 2, 3, 5, 10, 20, 40], kind="pos", scale=100.0)
    for occ, (rr, tt), name in ((0, (1.0, 120.0), "2 cm sphere, 120 s"), (1, (1.0, 600.0), "2 cm sphere, 600 s")):
        grid(f"E2 holds_low_flow, {name}", "| X_O2 \\ U (cm/s) | 0.3 | 0.5 | 1 | 2 | 3 |", occ, "holds_low_flow",
             lambda lab, rr=rr, tt=tt: (lambda u, x=float(lab): b1(x, u, rr, tt)), [0.3, 0.5, 1, 2, 3, 5, 10])
    grid("E2 holds_low_flow, 1/4 inch rod, 300 s (first row is a range; not compared)", "| X_O2 \\ U (cm/s) | 0.3 | 0.5 | 1 | 2 | 5 | 10 |", 0,
         "holds_low_flow", lambda lab: None if "to" in lab else (lambda u, x=float(lab): b1(x, u, 0.318, 300.0)), [0.3, 0.5, 1, 2, 5, 10])

    state = {"r": None}

    def e3(label):
        m = re.match(r"(rod|sphere) R = ([\d.]+): ([\d.]+)", label)
        if m:
            state["r"] = float(m.group(2))
            x = float(m.group(3))
        else:
            x = float(label)
        return lambda u, x=x, rr=state["r"]: b1(x, u, rr, 200.0)
    grid("E3 holds_high_flow", "| X_O2 \\ U (cm/s) | 5 | 10 | 15 |", 0, "holds_high_flow", e3, [5, 10, 15, 20, 30, 40, 55])

    # E4 quench speed and E5 blow-off speed: columns are oxygen
    def oxy_grid(title, header, target, xs, rowfn):
        rows = md_table("bass1", header)
        cases = []
        for k, r in enumerate(rows):
            for j, x in enumerate(xs):
                cases.append(dict(case_id=f"{k}|{j}", inputs=rowfn(r[0], x)))
        res = run("bass1", cases, target)
        out.extend(["", title, "", "| row | " + " | ".join(f"{x:g}" for x in xs) + " |", "|---" * (len(xs) + 1) + "|"])
        for k, r in enumerate(rows):
            cells = [r[0]]
            for j, x in enumerate(xs):
                s = res[f"{k}|{j}"][target]
                q = s["quantiles"]
                d = s["diag"]
                core = [math.exp(d["ln_median"] + d["ln_sd"] * z) * 100 for z in (-1.6449, 1.6449)]
                v = q["0.5"] * 100
                c = r[1 + j]
                if not c.startswith("below"):
                    dv = cell_num(c)
                    worst[target] = max(worst.get(target, 0.0), abs(v / dv - 1))
                cells.append(f"{c} ; {f(v, 2)} ({f(core[0], 2)} to {f(core[1], 2)})")
            out.append("| " + " | ".join(cells) + " |")
    oxy_grid("E4 quench speed cm/s, dossier ; forward median (core 5 to 95 %)", "| Sphere, time since ignition | 21 % |",
             "quench_speed", [0.21, 0.19, 0.17, 0.15], lambda lab, x: b1(x, 1.0, *body(lab)))
    oxy_grid("E5 blow-off speed cm/s, dossier ; forward median (core 5 to 95 %)", "| Body | 14.5 % |",
             "blowoff_speed", [0.145, 0.15, 0.16, 0.17, 0.19, 0.21], lambda lab, x: b1(x, 10.0, body(lab)[0], 200.0))
    # E6 flat
    grid("E6 flat_holds_low_flow, opposed", "| X_O2 \\ U (cm/s) | 0.3 | 0.5 | 1 | 1.5 |", 0, "flat_holds_low_flow",
         lambda lab: (lambda u, x=float(lab): b1(x, u, 1.0, 200.0, 0.0)), [0.3, 0.5, 1, 1.5, 2, 3, 5])
    grid("E6 flat_holds_high_flow, opposed", "| X_O2 \\ U (cm/s) | 5 | 10 | 20 | 30 |", 0, "flat_holds_high_flow",
         lambda lab: (lambda u, x=float(lab): b1(x, u, 1.0, 200.0, 0.0)), [5, 10, 20, 30, 40, 55])
    pts = [("op21", 0.21, 0.0, 0.83), ("op19", 0.19, 0.0, 0.94), ("op17", 0.17, 0.0, 1.08), ("op15", 0.15, 0.0, 1.25),
           ("co21", 0.21, 1.0, 1.25), ("co19", 0.19, 1.0, 1.41), ("co17", 0.17, 1.0, 1.61), ("co15", 0.15, 1.0, 1.87)]
    conc = [dict(case_id="c_lo_air1", inputs=b1(0.21, 1.0, 1.0, 200.0, 1.0)), dict(case_id="c_lo_air2", inputs=b1(0.21, 2.0, 1.0, 200.0, 1.0))]
    res = run("bass1", [dict(case_id=p[0], inputs=b1(p[1], 1.0, 1.0, 200.0, p[2])) for p in pts] + conc, "flat_quench_speed,flat_holds_low_flow")
    out += ["", "E6 flat quench speed cm/s, and the concurrent low-flow statement", "", "| point | dossier | forward |", "|---|---|---|"]
    for p in pts:
        v = res[p[0]]["flat_quench_speed"]["quantiles"]["0.5"] * 100
        worst["flat_quench_speed"] = max(worst.get("flat_quench_speed", 0.0), abs(v / p[3] - 1))
        out.append(f"| {'concurrent' if p[2] else 'opposed'} {p[1]:.2f} | {p[3]} | {v:.3f} |")
    out.append(f"| concurrent, 21 %, flat_holds_low_flow at 1 / 2 cm/s | 0.30 / 0.85 | "
               f"{res['c_lo_air1']['flat_holds_low_flow']['probability']:.2f} / {res['c_lo_air2']['flat_holds_low_flow']['probability']:.2f} |")
    return out, worst


# ------------------------------------------------------------------------------------------------- spice variant B
def check_spice_b():
    pts = [("Ethylene", 52.7), ("Propane", 63.9), ("Propylene", 19.3)]
    res = run("spice", [dict(case_id=p[0], inputs=spice_inputs(p[0], 1.6, 25.0)) for p in pts], "smoke_point_length", cand="w0-fable")
    out = ["Variant B (candidate w0-fable), section 5.1: smoke-point length at 1.6 mm and 25 cm/s.", "",
           "| fuel | dossier mm | forward mm |", "|---|---|---|"]
    worst = {"L_sp": 0.0}
    for name, d in pts:
        v = res[name]["smoke_point_length"]["quantiles"]["0.5"] * 1000
        worst["L_sp"] = max(worst["L_sp"], abs(v / d - 1))
        out.append(f"| {name} | {d} | {v:.1f} |")
    s = res["Ethylene"]["smoke_point_length"]["diag"]["ln_sd"]
    out.append(f"| flat width (ln-sd) | 0.45 | {s:.3f} |")
    return out, worst


# ------------------------------------------------------------------------------------------------- slice
SL_FUEL = {"Methane 100 %": (1.0, 0, 0), "Methane 70 %": (0.7, 0, 0), "Methane 40 %": (0.4, 0, 0),
           "Ethane": (1.0, 1, 0), "Propane": (1.0, 0, 1)}


def sl(fuel, uco, uj, d_mm, xo2=0.21, mdot=1e-6):
    x, e2, e3 = SL_FUEL[fuel]
    return dict(u_coflow=uco, u_jet=uj, burner_id=d_mm / 1000.0, x_fuel=x, x_o2=xo2, is_ethane=float(e2),
                is_propane=float(e3), mdot_jet=mdot)


def check_slice():
    out, worst = [], {"lifted": 0.0, "blow_off": 0.0}
    rows = md_table("slice", "| Fuel | 10 cm/s |")
    cos = [10, 20, 30, 45, 60, 80]
    cases = []
    for r in rows:
        fuels = ["Ethane", "Propane"] if r[0].startswith("Ethane") else [r[0]]
        for fu in fuels:
            for u in cos:
                cases.append(dict(case_id=f"c|{fu}|{u}", inputs=sl(fu, u / 100.0, 0.5, 3.2)))
    rj = md_table("slice", "| Fuel | Tube |")
    js = [2, 5, 10, 20, 40]
    for r in rj:
        for u in js:
            cases.append(dict(case_id=f"j|{r[0]}|{r[1]}|{u}", inputs=sl(r[0], 0.2, float(u), float(r[1].split()[0]))))
    for xo in (0.20, 0.21, 0.22, 0.23):
        cases.append(dict(case_id=f"o|{xo}", inputs=sl("Methane 40 %", 0.45, 0.5, 3.2, xo2=xo)))
    res = run("slice", cases, "lifted,blow_off")

    def P(cid):
        return res[cid]["lifted"]["probability"], res[cid]["blow_off"]["probability"]

    def cmp(cell, v):
        dl, db = [float(z) for z in cell.split("/")]
        worst["lifted"] = max(worst["lifted"], abs(v[0] - dl))
        worst["blow_off"] = max(worst["blow_off"], abs(v[1] - db))
        return f"{cell} ; {v[0]:.2f} / {v[1]:.2f}"
    out += ["Coflow-driven, 3.2 mm tube, jet 0.5 m/s: P(lifted) / P(blow-off on the line), dossier ; forward", "",
            "| Fuel | " + " | ".join(f"{u} cm/s" for u in cos) + " |", "|---" * 7 + "|"]
    for r in rows:
        fuels = ["Ethane", "Propane"] if r[0].startswith("Ethane") else [r[0]]
        for fu in fuels:
            out.append(f"| {fu} | " + " | ".join(cmp(r[1 + j], P(f"c|{fu}|{u}")) for j, u in enumerate(cos)) + " |")
    out += ["", "Jet-driven, coflow 20 cm/s", "", "| Fuel | Tube | " + " | ".join(f"{u} m/s" for u in js) + " |", "|---" * 7 + "|"]
    for r in rj:
        out.append(f"| {r[0]} | {r[1]} | " + " | ".join(cmp(r[2 + j], P(f"j|{r[0]}|{r[1]}|{u}")) for j, u in enumerate(js)) + " |")
    dv = [0.65, 0.53, 0.41, 0.31]
    vals = [P(f"o|{xo}")[0] for xo in (0.20, 0.21, 0.22, 0.23)]
    worst["oxygen"] = max(abs(a - b) for a, b in zip(vals, dv))
    out += ["", "Oxygen, 40 % methane at 45 cm/s, large tube (3.2 mm, jet 0.5 m/s), P(lifted) at 20 / 21 / 22 / 23 %: "
            f"dossier 0.65 / 0.53 / 0.41 / 0.31 ; forward {' / '.join(f'{v:.2f}' for v in vals)}", ""]
    # length: the jet speed of the check points from the mass flow and the SPICE dossier's methane density at 294 K
    # (a condition of the check point only, not a model number)
    rho = 0.665
    a32 = math.pi * 0.0032 ** 2 / 4
    lp = [(1.0, 23.0), (4.0, 103.0)]
    rl = run("slice", [dict(case_id=f"l{k}", inputs=sl("Methane 100 %", 0.2, m * 1e-6 / (rho * a32), 3.2, mdot=m * 1e-6))
                       for k, (m, _) in enumerate(lp)], "flame_length")
    for k, (m, d) in enumerate(lp):
        v = rl[f"l{k}"]["flame_length"]["quantiles"]["0.5"] * 1000
        worst["length"] = max(worst.get("length", 0.0), abs(v / d - 1))
        out.append(f"Length, methane on 3.2 mm at 20 cm/s, {m} mg/s (jet speed from rho 0.665 kg/m3): dossier {d} mm ; forward {v:.1f} mm")
    return out, worst


# ------------------------------------------------------------------------------------------------- cld
def cl(x_final, v_co, window=60.0, eth=0.0, x_stage=1.0, v_stage=0.2):
    return dict(x_fuel_final=x_final, v_coflow_final=v_co, window=window, is_ethylene=eth, x_fuel_stage=x_stage,
                v_fuel_stage=v_stage, planned_fuel_off_from_onset=window)


def check_cld():
    out, worst = [], {}
    rows = md_table("cld", "| Fuel | Final fuel fraction |")
    cos = [10, 20, 30, 40, 60]
    cases = [dict(case_id=f"{r[0]}|{r[1]}|{u}", inputs=cl(float(r[1]), u / 100.0, eth=1.0 if r[0] == "Ethylene" else 0.0))
             for r in rows for u in cos]
    res = run("cld", cases, "loss", times=[60.0])

    def lost(s, k=0):
        return sum(s["loss"]["cdf"][c][k] for c in ("blowoff", "other"))
    out += ["P(lost before a planned fuel-off 60 s after onset), dossier ; forward (sum of the two cause rows at 60 s)", "",
            "| Fuel | x final | " + " | ".join(f"{u} cm/s" for u in cos) + " |", "|---" * 7 + "|"]
    for r in rows:
        cells = []
        for j, u in enumerate(cos):
            v = lost(res[f"{r[0]}|{r[1]}|{u}"])
            worst["P_loss"] = max(worst.get("P_loss", 0.0), abs(v - float(r[2 + j])))
            cells.append(f"{r[2 + j]} ; {v:.2f}")
        out.append(f"| {r[0]} | {r[1]} | " + " | ".join(cells) + " |")
    s1 = [(5, 0.01), (10, 0.02), (20, 0.11), (30, 0.32), (45, 0.64), (60, 0.83)]
    s2 = [(3, 0.007), (10, 0.012), (30, 0.021), (60, 0.033), (90, 0.054)]
    for k, (name, inp, ser) in enumerate((("methane diluted to 15 % at 30 cm/s, window 60 s", cl(0.15, 0.3, 60.0), s1),
                                          ("methane held at 35 % at 30 cm/s, window 90 s", cl(0.35, 0.3, 90.0, x_stage=0.35), s2))):
        rr = run("cld", [dict(case_id=f"t{k}", inputs=inp)], "loss", times=[float(t) for t, _ in ser], cand=None)[f"t{k}"]
        vals = [lost(rr, i) for i in range(len(ser))]
        worst["P_loss_time"] = max([worst.get("P_loss_time", 0.0)] + [abs(v - d) for v, (_, d) in zip(vals, ser)])
        out.append("")
        out.append(f"Against time, {name}: " + ", ".join(f"{t} s dossier {d} ; forward {v:.3f}" for v, (t, d) in zip(vals, ser)))
    rows = md_table("cld", "| Fuel | Fuel fraction | 10 cm/s |")
    vs = [10, 20, 30, 50]
    cases = [dict(case_id=f"{r[0]}|{r[1]}|{v}", inputs=cl(1.0, 0.2, eth=1.0 if r[0] == "Ethylene" else 0.0,
                                                          x_stage=float(r[1]), v_stage=v / 100.0)) for r in rows for v in vs]
    res = run("cld", cases, "radiometer")
    out += ["", "Radiometer excess, V (median), dossier ; forward", "",
            "| Fuel | x stage | " + " | ".join(f"{v} cm/s" for v in vs) + " |", "|---" * 6 + "|"]
    for r in rows:
        cells = []
        for j, v in enumerate(vs):
            q = res[f"{r[0]}|{r[1]}|{v}"]["radiometer"]["quantiles"]["0.5"]
            worst["radiometer"] = max(worst.get("radiometer", 0.0), abs(q / float(r[2 + j]) - 1))
            cells.append(f"{r[2 + j]} ; {f(q, 2)}")
        out.append(f"| {r[0]} | {r[1]} | " + " | ".join(cells) + " |")
    return out, worst


CHECKS = {"spice_b": check_spice_b, "slice": check_slice, "cld": check_cld, "bass1": check_bass1, "bre": check_bre, "spice": check_spice, "efield": check_efield, "samer": check_samer, "same": check_same, "daft": check_daft}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--slug", default="")
    ap.add_argument("--out", default="runs/dev/gen7/transcription_checks.md")
    a = ap.parse_args()
    slugs = [s for s in a.slug.split(",") if s] or list(CHECKS)
    text = ["# Transcription checks of the seventh-generation W0 law releases", "",
            "Forward values at the condition points of each dossier's registered expectations, beside the dossier's "
            "hand-computed values. No data involved. Written by `tools/wsp/ext/law_expectations.py`.", ""]
    summary = {}
    for s in slugs:
        lines, worst = CHECKS[s]()
        summary[s] = worst
        text += [f"## {s} (release {release_of(s.split('_')[0], 'w0-fable' if s == 'spice_b' else None)})", "", "Largest differences: " + ", ".join(f"{k} {v:.3f}" for k, v in worst.items())
                 + " (relative for sizes, times, currents and voltages; absolute for probabilities and ln-sd)", ""] + lines + [""]
    p = REPO / a.out
    p.write_text("\n".join(text) + "\n", encoding="utf-8", newline="\n")
    print(json.dumps(summary, indent=1))
    return 0


if __name__ == "__main__":
    sys.exit(main())
