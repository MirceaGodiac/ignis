#!/usr/bin/env python3
"""Conditions of the CFI tests (PSI-39) for the feasibility of a droplet transfer test. Reads
conditions only, and writes only under `runs/dev/data_all/work/transfer_droplet/cfi/`.

Sources, all of them condition-bearing:
  * the campaign spreadsheet `CFI_DataSummary_Spreadsheet_Jan2021.xlsx`: sheet `Master Sheet`
    (date, local time, CIR test point, pressure, O2/N2/He, fibre, fuel) and sheet `Igniter` (power,
    time), for every row; from sheet `DATA` the column `D0 ( mm )` alone (the initial diameter, a
    condition in the project's test catalogue, `tools/wsp/atlas/normalize_tests.py`), and only for
    rows outside the test days. No other column of `DATA`, and no cell of `Comments`, is read.
  * the PSI experiment table `PSI-39_Experimental table_CFI.csv` (the same condition columns, 87
    rows) for a cross-check.
  * the timeline `<CIR>.txt` of each rack test point outside the test days: its header block (igniter
    on time and power, igniter positions, the planned "Burn to Completion" time, sequence stamps) and,
    of its table, only the rows stamped in the 120 s before the igniter came on (chamber pressure
    before ignition); rows stamped earlier (some tables begin with rows of another session) are
    counted and skipped. Reading stops at the first row stamped at or after the igniter-on stamp.
  * the file catalogue `runs/dev/data_all/work/atlas/files.jsonl`: which processed files exist.
No processed series, test sheet, comment file, image or video is opened here.

The day group of a test is the CIR day of year (in the year of the table's date), as the project's
catalogue assigns it; a test whose group is a TEST day of docs/wsp/atlas/splits/PSI-39.json is
carried with its Master-Sheet conditions only.

    python -I tools/wsp/ext/cfi_conditions.py
"""
import csv
import datetime
import json
import math
import re
import sys
from collections import Counter
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
RAW = REPO / "runs/dev/data_all/raw/PSI-39"
XLSX = RAW / "Science Documents/PSI-39_Science Documents_CFI_DataSummary_Spreadsheet_Jan2021.xlsx"
CSV = RAW / "Experimental table/PSI-39_Experimental table_CFI.csv"
FILES = REPO / "runs/dev/data_all/work/atlas/files.jsonl"
SPLIT = REPO / "docs/wsp/atlas/splits/PSI-39.json"
OUT = REPO / "runs/dev/data_all/work/transfer_droplet/cfi"
ATM = 101325.0

# the declared domain of every droplet release on droplet3 / droplet6 (applicability.domain), with the
# feature transforms of the spec: c_o2 = (x_o2 - 0.21)/0.05, c_co2 = x_co2/0.20, c_he = x_he/0.20,
# c_logp = ln(p / 1e5 Pa), c_logd0 = ln(D0 / 2 mm) (D0 estimated by the forward from the prefix)
DOMAIN = dict(c_o2=(-2.2, 4.8), c_co2=(0.0, 4.0), c_he=(0.0, 3.0), c_logp=(-1.0, 1.5), c_logd0=(-0.9, 1.26))
# candidate models (docs/wsp/research/phys_droplet_fuels.md section 0) -> the fuel each one serves
CANDIDATES = {
    "flex_heptane (wsp2 D2 / wsp5)": "n-heptane", "flex_methanol (wsp2 D3 / wsp5)": "methanol",
    "wsp5/octane/w0-fable": "n-octane", "wsp5/isooctane/w0-fable": "iso-octane", "wsp5/decane/w0-fable": "n-decane",
    "wsp5/dodecane/w0-fable": "n-dodecane", "wsp5/farnesane/w0-fable": "farnesane", "wsp5/ethanol/w0-fable": "ethanol",
}


def fuel_of(raw):
    s = (raw or "").strip()
    if s.upper() == "C12H26":
        return "n-dodecane", None
    if s.lower() == "farnesane":
        return "farnesane", None
    m = re.fullmatch(r"\[(\.\d+)\]/\[(\.\d+)\]\s*C12H26/iso-dodecane", s)
    if m:
        return "n-dodecane+iso-dodecane", f"{float(m.group(1)):.2f}/{float(m.group(2)):.2f}"
    return "unknown", None


def atm_of(raw):
    m = re.fullmatch(r"\s*(\d*\.?\d+)\s*atm\s*", str(raw or ""))
    return float(m.group(1)) if m else None


def timeline_header(path):
    """Header fields of a CIR timeline and the mean chamber pressure (psia) over the table rows stamped
    before the igniter-on stamp. Stops reading at the first later row."""
    head, psia, n, other = {}, [], 0, 0
    ign = None
    with path.open(encoding="utf-8", errors="replace") as f:
        in_table = False
        for line in f:
            if not in_table:
                if line.startswith("MDCA MP Time"):
                    in_table = True
                    continue
                if ":" in line and not line.startswith("\t") and not line.startswith(" "):
                    k, v = line.split(":", 1)
                    head[k.strip()] = v.strip()
                continue
            if line.startswith("="):
                continue
            parts = line.split("\t")
            ts = parts[0].strip()
            if not re.match(r"\d{4}-\d{2}-\d{2} ", ts):
                continue
            if ign is None:
                stamp = head.get("AS Burn to Completion/Igniter On") or ""
                ign = datetime.datetime.fromisoformat(stamp) if stamp else None
            t = datetime.datetime.fromisoformat(ts)
            if ign is None or t >= ign:
                break
            if not (ign - datetime.timedelta(seconds=120) <= t):
                other += 1  # a row stamped long before this test point's ignition (another session)
                continue
            cols = line.split()
            try:
                psia.append(float(cols[-2]))
                n += 1
            except (ValueError, IndexError):
                pass

    def num(k):
        v = head.get(k)
        try:
            return float(v.replace(",", "")) if v else None
        except ValueError:
            return None
    return dict(igniter_on_ms=num("Igniter On Time (milliseconds)"), igniter_w=num("Igniter Wattage"),
                planned_burn_ms=num("Burn to Completion (milliseconds)"),
                dispensed=bool(head.get("AS Dispense Start")), stretched=bool(head.get("AS Stretch Start")),
                pre_ignition_psia=(sum(psia) / len(psia)) if psia else None, pre_ignition_rows=n,
                rows_stamped_over_120_s_before_ignition=other,
                igniter_on_stamp=head.get("AS Burn to Completion/Igniter On") or None)


def main():
    import openpyxl
    split = json.loads(SPLIT.read_text(encoding="utf-8"))["strata"]["all"]["groups"]
    part = {g: p for p, gs in split.items() for g in gs}
    wb = openpyxl.load_workbook(XLSX, read_only=True, data_only=True)

    def sheet(name, cols=None):
        it = wb[name].iter_rows(values_only=True)
        head = [("" if h is None else str(h)) for h in next(it)]
        idx = [head.index(c) for c in cols] if cols else range(len(head))
        out = {}
        for r in it:
            if r[0] is None or str(r[0]).strip() == "":
                continue
            key = str(int(r[0])) if isinstance(r[0], (int, float)) else str(r[0]).strip()
            out.setdefault(key, {head[i]: r[i] for i in idx})
        return out
    master = sheet("Master Sheet")
    igniter = sheet("Igniter", ["Flex Number", "Ignition Power ( W )", "Ignition Time ( ms )"])
    d0col = sheet("DATA", ["Flex Number", "D0 ( mm )"])  # the D0 column alone; read below only off test days
    csv_rows = list(csv.reader(CSV.open(encoding="utf-8-sig", errors="replace")))
    csv_by = {r[0].strip(): r for r in csv_rows[1:] if r and r[0].strip()}

    files = {}
    with FILES.open(encoding="utf-8") as f:
        for line in f:
            if '"PSI-39"' not in line:
                continue
            r = json.loads(line)
            if r["investigation"] == "PSI-39" and r.get("test_id"):
                files.setdefault(r["test_id"], []).append((r["file_name"], r["size_bytes"], r["kind"]))
    raw_dir = RAW / "Raw Data"
    have = {p.name for p in raw_dir.iterdir()}

    rows = []
    for key, m in master.items():
        cir = str(m.get("CIR ") or "").strip().lower()
        td = m["Test Date"].date() if isinstance(m["Test Date"], datetime.datetime) else None
        cm = re.fullmatch(r"[a-z](\d{3})[a-z]\d+", cir)
        cd = datetime.date(td.year, 1, 1) + datetime.timedelta(days=int(cm.group(1)) - 1) if (td and cm) else td
        group = f"PSI-39:day:{cd.isoformat()}" if cd else f"PSI-39:test:{key}"
        p = part.get(group, "held")
        fuel, blend = fuel_of(m.get("Fuel "))
        fib = (str(m.get("Fiber")).strip() if m.get("Fiber") is not None else None)
        x_o2, x_n2, x_he = (float(m[k]) if m.get(k) is not None else None for k in ("Oxygen", "Nitrogen", "Helium"))
        p_atm = atm_of(m.get("Pressure"))
        flexid = f"FLEX-{key}"
        fl = files.get(flexid, []) + files.get(cir.upper(), [])
        names = {n for n, _, _ in fl}
        row = dict(flex=key, cir=cir.upper(), table_date=td.isoformat() if td else None,
                   cir_day=cd.isoformat() if cd else None, date_conflict=bool(td and cd and td != cd),
                   group=group, partition=p, fuel=fuel, blend_mass_or_volume_fractions=blend,
                   fibre={"Yes": "fibre", "No": "free"}.get(fib, "not stated"), pressure_atm=p_atm,
                   pressure_pa=p_atm * ATM if p_atm is not None else None, x_o2=x_o2, x_n2=x_n2, x_he=x_he, x_co2=0.0,
                   composition_sum=None if None in (x_o2, x_n2, x_he) else round(x_o2 + x_n2 + x_he, 6),
                   in_psi_csv=key in csv_by,
                   droplet_series=f"PSI-39_Raw Data_{flexid}_DropletData.txt" in names,
                   droplet_series_downloaded=f"PSI-39_Raw Data_{flexid}_DropletData.txt" in have,
                   igor_pxp=sorted(n for n in names if n.endswith(".pxp")),
                   test_sheet=f"PSI-39_Raw Data_{flexid}_TestSheet.txt" in names,
                   timeline=f"PSI-39_Raw Data_{cir.upper()}.txt" in names,
                   cool_flame_series=f"PSI-39_Raw Data_{flexid}_CoolFlameData.txt" in names)
        if key in csv_by:
            c = csv_by[key]
            same = (atm_of(c[4]) == p_atm and float(c[5]) == x_o2 and float(c[6]) == x_n2 and float(c[7]) == x_he
                    and c[3].strip().lower() == cir)
            row["psi_csv_agrees"] = same
        if p != "test":
            ig = igniter.get(key, {})
            row["igniter_table"] = dict(power=ig.get("Ignition Power ( W )"), time=ig.get("Ignition Time ( ms )"))
            d0 = d0col.get(key, {}).get("D0 ( mm )")
            row["d0_mm_table"] = float(d0) if isinstance(d0, (int, float)) else None
            tl = raw_dir / f"PSI-39_Raw Data_{cir.upper()}.txt"
            if cir and tl.exists():
                row["timeline_header"] = timeline_header(tl)
        rows.append(row)

    # gates of each candidate model, from the conditions (the forward also refuses on its own measurement
    # and horizon gates, which need the series and are not evaluated here)
    for r in rows:
        r["gates"] = {}
        c = dict(c_o2=(r["x_o2"] - 0.21) / 0.05 if r["x_o2"] is not None else None,
                 c_co2=r["x_co2"] / 0.20, c_he=r["x_he"] / 0.20 if r["x_he"] is not None else None,
                 c_logp=math.log(r["pressure_pa"] / 1e5) if r.get("pressure_pa") else None,
                 c_logd0=math.log(r["d0_mm_table"] / 2.0) if r.get("d0_mm_table") else None)
        r["features"] = {k: None if v is None else round(v, 4) for k, v in c.items()}
        for model, fuel in CANDIDATES.items():
            why = []
            if r["fuel"] != fuel:
                why.append("fuel" if r["fuel"] != "n-dodecane+iso-dodecane" else "fuel (mixture)")
            for k, (lo, hi) in DOMAIN.items():
                v = c[k]
                if v is None:
                    if k != "c_logd0":
                        why.append(f"{k} unknown")
                elif not (lo <= v <= hi):
                    why.append(f"{k} outside [{lo}, {hi}]")
            if r["fibre"] == "fibre":
                why.append("fibre-supported (applicability: single free-floating droplet)")
            elif r["fibre"] == "not stated":
                why.append("support not stated")
            r["gates"][model] = why
    OUT.mkdir(parents=True, exist_ok=True)
    with (OUT / "conditions.jsonl").open("w", encoding="utf-8", newline="\n") as f:
        for r in rows:
            f.write(json.dumps(r, ensure_ascii=False) + "\n")

    # summaries
    def cnt(sel, key):
        return dict(Counter(key(r) for r in sel))
    s = dict(rows=len(rows), by_partition=cnt(rows, lambda r: r["partition"]),
             by_fuel=cnt(rows, lambda r: r["fuel"]), by_support=cnt(rows, lambda r: r["fibre"]),
             date_conflicts=[(r["flex"], r["table_date"], r["cir_day"]) for r in rows if r["date_conflict"]],
             composition_sums_not_1=[(r["flex"], r["composition_sum"]) for r in rows if r["composition_sum"] not in (1.0, None)],
             psi_csv_rows=sum(r["in_psi_csv"] for r in rows), psi_csv_disagrees=[r["flex"] for r in rows if r.get("psi_csv_agrees") is False],
             pressures=cnt(rows, lambda r: r["pressure_atm"]), atmospheres=cnt(rows, lambda r: f"O2 {r['x_o2']} N2 {r['x_n2']} He {r['x_he']}"),
             series=dict(by_partition=cnt([r for r in rows if r["droplet_series"]], lambda r: r["partition"]),
                         tests=[(r["flex"], r["cir"], r["partition"], r["fuel"], r["fibre"], r["pressure_atm"], r["x_o2"], r["x_he"])
                                for r in rows if r["droplet_series"]],
                         downloaded=sum(r["droplet_series_downloaded"] for r in rows),
                         pxp_tests=sum(bool(r["igor_pxp"]) for r in rows),
                         pxp_without_txt=[r["flex"] for r in rows if r["igor_pxp"] and not r["droplet_series"]]),
             timelines_read=sum("timeline_header" in r for r in rows),
             d0_known_off_test_days=sum(r.get("d0_mm_table") is not None for r in rows))
    off = [r for r in rows if r["partition"] != "test"]
    d0 = [r["d0_mm_table"] for r in off if r.get("d0_mm_table")]
    s["d0_mm_off_test_days"] = dict(n=len(d0), min=min(d0) if d0 else None, max=max(d0) if d0 else None)
    pb = [r["timeline_header"]["planned_burn_ms"] for r in off if r.get("timeline_header") and r["timeline_header"]["planned_burn_ms"]]
    s["planned_record_ms_off_test_days"] = dict(n=len(pb), values=dict(Counter(pb)))
    pp = [(r["flex"], r["pressure_atm"], r["timeline_header"]["pre_ignition_psia"]) for r in off
          if r.get("timeline_header") and r["timeline_header"]["pre_ignition_psia"] and r["pressure_atm"]]
    s["pre_ignition_pressure_vs_nominal"] = dict(n=len(pp), max_rel_diff=max(abs(p * 0.0689476 * 1e5 / (a * ATM) - 1) for _, a, p in pp) if pp else None,
                                               worst=sorted(pp, key=lambda x: -abs(x[2] * 0.0689476e5 / (x[1] * ATM) - 1))[:5])
    s["dispensed_or_stretched"] = cnt([r for r in off if r.get("timeline_header")],
                                      lambda r: ("dispense" if r["timeline_header"]["dispensed"] else "-") + "/" + ("stretch" if r["timeline_header"]["stretched"] else "-"))
    gates = {}
    for model in CANDIDATES:
        for scope, sel in (("all_227", rows), ("train_validation", off), ("with_series_train_validation", [r for r in off if r["droplet_series"]])):
            g = Counter()
            for r in sel:
                w = r["gates"][model]
                g["answers" if not w else "refused"] += 1
                for x in w:
                    g["reason: " + x.split(" outside")[0].split(" (")[0]] += 1
            gates.setdefault(model, {})[scope] = dict(g)
    s["gates"] = gates
    (OUT / "summary.json").write_text(json.dumps(s, indent=1, ensure_ascii=False, default=str) + "\n", encoding="utf-8")
    print(json.dumps(s, indent=1, default=str))
    return 0


if __name__ == "__main__":
    sys.exit(main())
