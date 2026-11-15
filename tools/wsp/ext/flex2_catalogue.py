#!/usr/bin/env python3
"""A per-test catalogue of FLEX-2 (PSI-68), conditions only. Writes
`runs/dev/data_all/work/atlas/flex2_tests.jsonl` (one row per rack test point) and
`runs/dev/data_all/work/atlas/flex2_tests_summary.json`. No image or video is downloaded or opened.

Sources (all text files of the test points, already downloaded, and the file catalogue):
  * `runs/dev/data_all/work/atlas/files.jsonl`: the test points (`test_id_scheme` cir_test_point;
    gas reading points cir_gas_reading_point), the PSI data-record folder each file is filed under
    ("Reduced Gravity/<test matrix> <year> - <fuel>"), and the size of each image archive.
  * `<test>.txt`, the rack timeline: its header block (igniter, planned burn-to-completion time,
    sequence stamps) and, of its table, only the rows stamped in the 120 s before the igniter came on
    (chamber pressure before ignition). Reading stops at the first row stamped at or after igniter-on.
  * `<test>_MDCA_Testpoint_Settings.txt`: the test point's parameters (fibre usage flag, flow nozzle
    speeds, which dispenser and needle profiles are active, needle stretch, igniter).
  * `<R>_Mixture.txt` of the gas reading points: date and ambient composition (with fuel vapour);
    a test point takes the composition of the gas reading point of the same day taken last before
    its igniter came on (else the first one after it on the same day, else none).
  * `<test>_Comments.txt`: image-downlink notes. Only deployment remarks (manual deployment) and
    record-length remarks are taken; remarks on what the flame or droplet did, or on ignition, are
    not recorded.

How the fuel is established (field `fuel_how`), in this order:
  `psi_data_record`      the test point's files are filed by NASA under a data record named for a fuel
  `joined_by_date`       not filed; every filed test point of the same day has one and the same fuel
  `joined_by_sequence`   not filed; the filed test points just before and just after it that day agree
  `inferred_vapour`      not filed and not joined; the gas readings of that day show exactly one fuel vapour
  `inferred_campaign_neighbours`  the filed test points of the same rack series within 30 days carry one fuel
  `unknown`

    python -I tools/wsp/ext/flex2_catalogue.py
"""
import csv
import datetime
import json
import re
import sys
from collections import Counter, defaultdict
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
RAW = REPO / "runs/dev/data_all/raw"
FILES = REPO / "runs/dev/data_all/work/atlas/files.jsonl"
MANIFEST = RAW / "MANIFEST.jsonl"
OUT = REPO / "runs/dev/data_all/work/atlas/flex2_tests.jsonl"
SUMMARY = REPO / "runs/dev/data_all/work/atlas/flex2_tests_summary.json"
PSIA = 6894.757293168

# data-record fuel label -> (fuel, pure?)
FUEL_LABEL = {
    "Decane": ("n-decane", True), "N_Octane": ("n-octane", True), "100% Octane": ("n-octane", True),
    "Heptane": ("n-heptane", True), "IsoOctane": ("iso-octane", True), "Ethanol": ("ethanol", True),
    "Toluene": ("toluene", True), "Propylbenzene": ("propylbenzene", True),
    "Heptane50_Iso-Octane50": ("n-heptane 50 / iso-octane 50", False),
    "Heptane50_Toluene50": ("n-heptane 50 / toluene 50", False),
    "Decane50_Propylbenzene50": ("n-decane 50 / propylbenzene 50", False),
    "Decane75_Propylbenzene25": ("n-decane 75 / propylbenzene 25", False),
    "Decane25_Propylbenzene75": ("n-decane 25 / propylbenzene 75", False),
    "Propanol80_Glycerol20": ("1-propanol 80 / glycerol 20", False),
    "Propanol95_Glycerol5": ("1-propanol 95 / glycerol 5", False),
    "95% Propanol, 5% Glycerol": ("1-propanol 95 / glycerol 5", False),
    "Background Images": (None, None),
}
VAPOUR = {"Decane": "n-decane", "Ethanol": "ethanol", "Iso-octane": "iso-octane", "Heptane": "n-heptane",
          "Octane": "n-octane", "Toluene": "toluene", "Propylbenzene": "propylbenzene", "Methanol": "methanol",
          "Hexadecane": "n-hexadecane", "Propanol": "1-propanol", "Glycerol": "glycerol", "Hexanol": "1-hexanol"}
# candidate droplet models (docs/wsp/research/phys_droplet_fuels.md section 0) and their fuel
CANDIDATE_FUELS = {"n-heptane": "flex_heptane (frozen: wsp2 D2, wsp5)", "methanol": "flex_methanol (frozen: wsp2 D3, wsp5)",
                   "n-octane": "wsp5/octane/w0-fable", "iso-octane": "wsp5/isooctane/w0-fable",
                   "n-decane": "wsp5/decane/w0-fable", "n-dodecane": "wsp5/dodecane/w0-fable",
                   "farnesane": "wsp5/farnesane/w0-fable", "ethanol": "wsp5/ethanol/w0-fable"}
DOMAIN = dict(c_o2=(-2.2, 4.8), c_co2=(0.0, 4.0), c_he=(0.0, 3.0), c_logp=(-1.0, 1.5))


def folder_record(folder):
    """'Reduced Gravity/SurrogateFuel 2014 - IsoOctane[/...]' -> (matrix, year, label)."""
    if not folder.startswith("Reduced Gravity/"):
        return None
    top = folder.split("/")[1].strip()
    m = re.fullmatch(r"(.+?) (\d{4}) - (.+)", top)
    if m:
        return m.group(1).strip(), int(m.group(2)), m.group(3).strip()
    return None, None, top  # "100% Octane", "95% Propanol, 5% Glycerol": no matrix or year in the name


def timeline(path):
    head, psia, other = {}, [], 0
    ign = None
    with path.open(encoding="utf-8", errors="replace") as f:
        in_table = False
        for line in f:
            if not in_table:
                if line.startswith("MDCA MP Time"):
                    in_table = True
                    s = head.get("AS Burn to Completion/Igniter On") or ""
                    try:
                        ign = datetime.datetime.fromisoformat(s) if s else None
                    except ValueError:
                        ign = None
                    continue
                if ":" in line and not line[:1].isspace():
                    k, v = line.split(":", 1)
                    head[k.strip()] = v.strip()
                continue
            ts = line[:23].strip()
            if not re.match(r"\d{4}-\d{2}-\d{2} \d", ts):
                continue
            t = datetime.datetime.fromisoformat(ts)
            if ign is None or t >= ign:
                break
            if t < ign - datetime.timedelta(seconds=120):
                other += 1
                continue
            try:
                psia.append(float(line.split()[-2]))
            except (ValueError, IndexError):
                pass

    def num(k):
        try:
            return float(head[k].replace(",", "")) if head.get(k) else None
        except ValueError:
            return None

    def stamp(k):
        try:
            return datetime.datetime.fromisoformat(head[k]) if head.get(k) else None
        except ValueError:
            return None
    start = stamp("AS Started") or stamp("CC Setup Started") or stamp("AS Setup Completed")
    return dict(igniter_on_ms=num("Igniter On Time (milliseconds)"), igniter_w=num("Igniter Wattage"),
                igniter_positions=[num("Igniter 1 Position"), num("Igniter 2 Position")],
                planned_burn_ms=num("Burn to Completion (milliseconds)"),
                dispensed=bool(head.get("AS Dispense Start")), stretched=bool(head.get("AS Stretch Start")),
                igniter_on=ign.isoformat(sep=" ") if ign else None, started=start.isoformat(sep=" ") if start else None,
                pre_ignition_psia=sum(psia) / len(psia) if psia else None, pre_ignition_rows=len(psia),
                rows_skipped_before_window=other)


def chamber_pressure(path, ign):
    """Mean of FCU_MP_PT16 (psia) over the rows of `<test>_Chamber_Readings.txt` stamped in the 120 s
    before igniter on; reading stops at the first row at or after it."""
    if ign is None:
        return None
    ign = datetime.datetime.fromisoformat(ign)
    v = []
    with path.open(encoding="utf-8", errors="replace") as f:
        head = next(csv.reader([f.readline()]))
        try:
            i = [h.strip() for h in head].index("FCU_MP_PT16(PSIA)")
        except ValueError:
            return None
        for line in f:
            ts = line.split(",", 1)[0].strip().strip('"')
            try:
                t = datetime.datetime.fromisoformat(ts)
            except ValueError:
                continue
            if t >= ign:
                break
            if t >= ign - datetime.timedelta(seconds=120):
                try:
                    v.append(float(line.split(",")[i]))
                except (ValueError, IndexError):
                    pass
    return sum(v) / len(v) if v else None


def settings(path):
    rows = list(csv.reader(path.open(encoding="utf-8", errors="replace")))
    if len(rows) < 2:
        return None
    h = [x.strip() for x in rows[0]]
    vals = [dict(zip(h, [x.strip() for x in r])) for r in rows[1:] if len(r) >= len(h) // 2]
    if not vals:
        return None
    v = vals[-1]
    g = lambda k: v.get(f"TP_PARAMETER_{k}(N/A)", v.get(f"TP_PARAMETER_{k}"))  # noqa: E731
    fl = lambda x: float(x) if x not in (None, "") and re.fullmatch(r"-?\d+(\.\d+)?", x) else x  # noqa: E731
    return dict(rows=len(vals), computer_time=v.get("MAIN_COMPUTER_TIME(GMT)"), fiber=g("FIBER_USAGE_FLAG"), flow_nozzle_speeds=[fl(g("FLOW_NOZZLE_1_SPEED")), fl(g("FLOW_NOZZLE_2_SPEED"))],
                dispenser_profiles=[v.get("DISPENSER_1_PROFILE(N/A)"), v.get("DISPENSER_2_PROFILE(N/A)")],
                needle_stretch_moves=fl(g("NEEDLE_STRETCH_MOVES")), soot_sampler=g("SOOT_SAMPLER_USAGE_FLAG"),
                igniter_on_ms=fl(g("IGNITER_ON_TIME")), igniter_w=fl(v.get("TP_PARAMETER_IGNITER_WATTAGE(Watts)")),
                burn_to_completion_ms=fl(g("BURN_TO_COMPLETION_TIME")),
                settings_change_within_file=len({json.dumps(x, sort_keys=True) for x in
                                                 [{k: y for k, y in r.items() if k.startswith("TP_PARAMETER")} for r in vals]}) > 1)


def mixture(path):
    rows = list(csv.reader(path.open(encoding="utf-8", errors="replace")))
    if len(rows) < 2:
        return None
    h = [x.strip() for x in rows[0]]
    r = dict(zip(h, [x.strip() for x in rows[1]]))
    pct = {}
    for k, v in r.items():
        if k == "Date/Time":
            continue
        try:
            pct[k] = float(v.rstrip("%")) / 100.0
        except ValueError:
            pct[k] = None
    return dict(date_time=r.get("Date/Time"), fractions=pct)


def comment_flags(path):
    flags = []
    for line in path.open(encoding="utf-8", errors="replace").read().splitlines():
        low = line.lower()
        if "manual" in low and "deploy" in low:
            flags.append("manual droplet deployment (operator note)")
        if "burn to" in low and "completion" in low and "incorrect" in low:
            flags.append("burn-to-completion time set incorrectly; images end before the test (operator note)")
        if "1 image per second" in low:
            flags.append("images at 1 per second only (operator note)")
    return sorted(set(flags))


def main():
    local = {}
    with MANIFEST.open(encoding="utf-8") as f:
        for line in f:
            r = json.loads(line)
            if r["investigation"] == "PSI-68" and r.get("status") in ("ok", "reused"):
                local[r["key"].split("/")[-1]] = RAW / r["local_path"]
    files = [json.loads(l) for l in FILES.open(encoding="utf-8") if '"PSI-68"' in l]
    files = [r for r in files if r["investigation"] == "PSI-68"]
    by_tp = defaultdict(list)
    for r in files:
        if r.get("test_id"):
            by_tp[(r["test_id"], r["test_id_scheme"])].append(r)

    def path_of(tp, suffix):
        for r in by_tp.get(tp, []):
            if r["file_name"].endswith(f"{tp[0]}{suffix}") and r["file_name"] in local:
                return local[r["file_name"]]
        return None

    # gas reading points: date and composition
    gas = []
    for tp in [k for k in by_tp if k[1] == "cir_gas_reading_point"]:
        mx, tl = path_of(tp, "_Mixture.txt"), path_of(tp, ".txt")
        m = mixture(mx) if mx else None
        t = timeline(tl) if tl else None
        when = (t or {}).get("igniter_on") or (t or {}).get("started")
        if m is None:
            continue
        if when is None and m.get("date_time"):
            try:
                when = datetime.datetime.strptime(m["date_time"], "%m/%d/%Y %H:%M").isoformat(sep=" ")
            except ValueError:
                when = None
        fr_ = m["fractions"]
        tot = sum(fr_.get(k) or 0.0 for k in ("O2", "N2", "He", "CO2", "Xe", "H20"))
        plausible = 0.97 <= tot <= 1.03 and (fr_.get("O2") or 0.0) >= 0.05
        mdate = None
        try:
            mdate = datetime.datetime.strptime(m["date_time"], "%m/%d/%Y %H:%M").date().isoformat()
        except (ValueError, TypeError):
            pass
        gas.append(dict(id=tp[0], when=when, mixture=m, plausible=plausible, gas_sum=round(tot, 4),
                        mixture_date=mdate, date_agrees=(mdate == when[:10]) if (mdate and when) else None,
                        timeline_pressure_psia=(t or {}).get("pre_ignition_psia")))
    gas_by_day = defaultdict(list)
    for g in gas:
        if g["when"]:
            gas_by_day[g["when"][:10]].append(g)

    rows = []
    for tp in sorted(k for k in by_tp if k[1] == "cir_test_point"):
        tid = tp[0]
        fr = by_tp[tp]
        recs = sorted({folder_record(r["folder"]) for r in fr if folder_record(r["folder"])}, key=str)
        names = {r["file_name"] for r in fr}
        tl_path, st_path, cm_path = path_of(tp, ".txt"), path_of(tp, "_MDCA_Testpoint_Settings.txt"), path_of(tp, "_Comments.txt")
        t = timeline(tl_path) if tl_path else None
        s = settings(st_path) if st_path else None
        arch = {cam: next((r["size_bytes"] for r in fr if r["file_name"].endswith(f"{tid}{cam}.zip")), None) for cam in "ABCD"}
        when = (t or {}).get("igniter_on") or (t or {}).get("started")
        s_ = settings(st_path) if st_path else None
        date_how = "timeline stamp" if when else None
        if not when and s_ and s_.get("computer_time"):
            when, date_how = s_["computer_time"], "test-point settings log (MAIN_COMPUTER_TIME)"
        row = dict(test_id=tid, kind="calibration" if tid.startswith("LCTF") else ("background" if tid.startswith("BKGD") else "test point"),
                   date=when[:10] if when else None, date_how=date_how or "unknown", group=f"PSI-68:day:{when[:10]}" if when else None,
                   igniter_on_gmt=(t or {}).get("igniter_on"), data_records=[dict(matrix=a, year=b, label=c) for a, b, c in recs],
                   has_timeline=tl_path is not None, has_settings=st_path is not None, has_comments=cm_path is not None,
                   has_cool_flame_log=f"{tid}_Cool_Flames.txt" in names,
                   backlit_archive_bytes=arch["A"], archives_bytes=arch)
        cp_path = path_of(tp, "_Chamber_Readings.txt")
        cp = chamber_pressure(cp_path, (t or {}).get("igniter_on")) if cp_path else None
        row["chamber_pt16_pre_ignition_pa"] = cp * PSIA if cp else None
        if t:
            row["timeline"] = {k: t[k] for k in ("igniter_on_ms", "igniter_w", "igniter_positions", "planned_burn_ms", "dispensed",
                                                 "stretched", "pre_ignition_rows", "rows_skipped_before_window")}
            row["pressure_pa"] = t["pre_ignition_psia"] * PSIA if t["pre_ignition_psia"] else None
            row["pressure_how"] = "mean chamber pressure (timeline table) in the 120 s before igniter on" if t["pre_ignition_psia"] else "unknown"
        else:
            row["pressure_pa"], row["pressure_how"] = None, "unknown"
        if row["pressure_pa"] is None and cp:
            row["pressure_pa"], row["pressure_how"] = cp * PSIA, "mean FCU_MP_PT16 (chamber readings) in the 120 s before igniter on"
        row["settings"] = s
        row["comment_flags"] = comment_flags(cm_path) if cm_path else []
        # ambient composition from the gas reading of the day
        comp, how = None, "unknown"
        if when:
            allday = sorted(gas_by_day.get(when[:10], []), key=lambda g: g["when"])
            row["gas_readings_rejected"] = [dict(id=g["id"], sum=g["gas_sum"], o2=g["mixture"]["fractions"].get("O2"))
                                            for g in allday if not g["plausible"]]
            day = [g for g in allday if g["plausible"]]
            before = [g for g in day if g["when"] <= when]
            after = [g for g in day if g["when"] > when]
            g = before[-1] if before else (after[0] if after else None)
            if g:
                fr_ = g["mixture"]["fractions"]
                comp = {k: fr_.get(k) for k in ("O2", "N2", "He", "CO2", "Xe", "H20") if k in fr_}
                vap = {VAPOUR[k]: v for k, v in fr_.items() if k in VAPOUR and v}
                how = f"joined: gas reading point {g['id']} of the same day, {'before' if before else 'after'} the test point"
                if g["date_agrees"] is False:
                    how += f" (its Mixture file is dated {g['mixture_date']})"
                row["vapour_at_gas_reading"] = vap
                row["gas_reading_point"] = g["id"]
        row["ambient"], row["ambient_how"] = comp, how
        rows.append(row)

    # the fuel
    for r in rows:
        labels = sorted({d["label"] for d in r["data_records"]})
        fuels = sorted({FUEL_LABEL.get(l, (l, None))[0] for l in labels if FUEL_LABEL.get(l, (l, None))[0]})
        r["fuel"], r["fuel_how"], r["fuel_pure"] = None, "unknown", None
        if len(fuels) == 1:
            r["fuel"], r["fuel_how"] = fuels[0], "psi_data_record"
        elif len(fuels) > 1:
            yr = int(r["date"][:4]) if r["date"] else None
            fit = sorted({FUEL_LABEL.get(d["label"], (d["label"], None))[0] for d in r["data_records"] if d["year"] in (yr, None)})
            if len(fit) == 1:
                r["fuel"], r["fuel_how"] = fit[0], "psi_data_record (two records; the other names another year)"
            else:
                r["fuel"], r["fuel_how"] = " | ".join(fuels), "psi_data_record (conflicting records)"
                r["fuel_candidates"] = fuels
        r["matrix"] = sorted({d["matrix"] for d in r["data_records"] if d["matrix"]}) or None
    by_day = defaultdict(list)
    for r in rows:
        if r["date"]:
            by_day[r["date"]].append(r)
    for day, rs in by_day.items():
        rs.sort(key=lambda r: r["igniter_on_gmt"] or "")
        filed = [r for r in rs if r["fuel_how"] == "psi_data_record"]
        for i, r in enumerate(rs):
            if r["fuel_how"] != "unknown" or r["kind"] != "test point":
                continue
            fs = {x["fuel"] for x in filed}
            if len(fs) == 1:
                r["fuel"], r["fuel_how"] = fs.pop(), "joined_by_date"
                continue
            prev = next((x for x in reversed(rs[:i]) if x["fuel_how"] == "psi_data_record"), None)
            nxt = next((x for x in rs[i + 1:] if x["fuel_how"] == "psi_data_record"), None)
            if prev and nxt and prev["fuel"] == nxt["fuel"]:
                r["fuel"], r["fuel_how"] = prev["fuel"], "joined_by_sequence"
                continue
            vap = set()
            for g in gas_by_day.get(day, []):
                vap |= {VAPOUR[k] for k, v in g["mixture"]["fractions"].items() if k in VAPOUR and v}
            if len(vap) == 1:
                r["fuel"], r["fuel_how"] = vap.pop(), "inferred_vapour"
    # still unknown: the filed test points of the same rack series (first letter of the name) within
    # 30 days on either side all carry one fuel
    for r in rows:
        if r["fuel_how"] != "unknown" or not r["date"] or r["kind"] != "test point":
            continue
        d0 = datetime.date.fromisoformat(r["date"])
        near = {x["fuel"] for x in rows if x["fuel_how"] == "psi_data_record" and x["date"] and x["test_id"][0] == r["test_id"][0]
                and abs((datetime.date.fromisoformat(x["date"]) - d0).days) <= 30}
        if len(near) == 1:
            r["fuel"], r["fuel_how"] = near.pop(), "inferred_campaign_neighbours"
    for r in rows:  # a conflict left by the records, resolved by the fuel vapour of the day's gas readings
        if r.get("fuel_candidates") and r["date"]:
            vap = set()
            for g in gas_by_day.get(r["date"], []):
                vap |= {VAPOUR[k] for k, v in g["mixture"]["fractions"].items() if k in VAPOUR and v}
            hit = [f for f in r["fuel_candidates"] if f in vap]
            r["vapour_of_day"] = sorted(vap)
            if len(hit) == 1:
                r["fuel"], r["fuel_how"] = hit[0], "psi_data_record (conflicting records; resolved by the fuel vapour of the day)"
    pure = {v[0]: v[1] for v in FUEL_LABEL.values() if v[0]}
    for r in rows:
        r["fuel_pure"] = pure.get(r["fuel"]) if r["fuel"] in pure else (True if r["fuel"] in VAPOUR.values() else None)
        st = r["settings"] or {}
        flow = st.get("flow_nozzle_speeds") or []
        moving = any(isinstance(x, float) and x != 0 for x in flow)
        both = (st.get("dispenser_profiles") or [None, None])
        two = all(p not in (None, "", "None") for p in both)
        m = r["matrix"] or []
        if any("Binary Array" in x for x in m):
            cfg = "droplet array (data record 'Binary Array')"
        elif any("Convective Flow" in x for x in m) or moving:
            cfg = "imposed flow (data record or flow nozzle speed)"
        elif st.get("fiber") == "Yes":
            cfg = "fibre-supported"
        elif two:
            cfg = "two dispensers active"
        elif st.get("fiber") == "No":
            cfg = "single free droplet (settings: no fibre, no flow, one dispenser)"
        else:
            cfg = "unknown"
        r["configuration"] = cfg
        a = r["ambient"] or {}
        p = r["pressure_pa"]
        import math
        c = dict(c_o2=(a["O2"] - 0.21) / 0.05 if a.get("O2") is not None else None,
                 c_co2=(a.get("CO2") or 0.0) / 0.20 if a else None, c_he=(a.get("He") or 0.0) / 0.20 if a else None,
                 c_logp=math.log(p / 1e5) if p else None)
        r["features"] = {k: None if v is None else round(v, 4) for k, v in c.items()}
        out = [k for k, (lo, hi) in DOMAIN.items() if c[k] is not None and not (lo <= c[k] <= hi)]
        r["unknown_features"] = [k for k in DOMAIN if c[k] is None]
        r["candidate"] = CANDIDATE_FUELS.get(r["fuel"]) if r["fuel_pure"] else None
        r["single_pure_candidate"] = bool(r["candidate"] and r["configuration"].startswith("single free droplet"))
        r["outside_domain"] = out
        r["inside_domain"] = not out and not r["unknown_features"]

    OUT.parent.mkdir(parents=True, exist_ok=True)
    with OUT.open("w", encoding="utf-8", newline="\n") as f:
        for r in rows:
            f.write(json.dumps(r, ensure_ascii=False) + "\n")

    tps = [r for r in rows if r["kind"] == "test point"]
    sp = [r for r in tps if r["single_pure_candidate"]]
    s = dict(
        rows=len(rows), by_kind=dict(Counter(r["kind"] for r in rows)), test_points=len(tps),
        days=len({r["date"] for r in tps if r["date"]}), test_points_without_date=sum(1 for r in tps if not r["date"]),
        by_year=dict(sorted(Counter((r["date"] or "none")[:4] for r in tps).items())),
        by_date=dict(sorted(Counter(r["date"] or "none" for r in tps).items())),
        by_fuel=dict(Counter(r["fuel"] or "unknown" for r in tps).most_common()),
        by_fuel_how=dict(Counter(r["fuel_how"] for r in tps)),
        by_fuel_and_how={f"{k[0]} :: {k[1]}": v for k, v in Counter((r["fuel"] or "unknown", r["fuel_how"]) for r in tps).most_common()},
        by_configuration=dict(Counter(r["configuration"] for r in tps)),
        by_matrix=dict(Counter(" + ".join(r["matrix"] or ["none"]) for r in tps)),
        ambient_how=dict(Counter(r["ambient_how"].split(":")[0] for r in tps)),
        pressure_how=dict(Counter(r["pressure_how"].split(" (")[0] for r in tps)),
        atmospheres=dict(Counter(f"O2 {round((r['ambient'] or {}).get('O2') or -1, 2)} He {round((r['ambient'] or {}).get('He') or 0, 2)}"
                                 for r in tps).most_common(20)),
        pressure_atm=dict(Counter(round(r["pressure_pa"] / 101325, 1) if r["pressure_pa"] else None for r in tps).most_common()),
        with_backlit_archive=sum(1 for r in tps if r["backlit_archive_bytes"]),
        backlit_archive_gb=sum(r["backlit_archive_bytes"] or 0 for r in tps) / 1e9,
        single_pure_candidate=dict(n=len(sp), by_fuel=dict(Counter(r["fuel"] for r in sp)),
                                   by_fuel_how=dict(Counter(r["fuel_how"] for r in sp)),
                                   with_backlit_archive=sum(1 for r in sp if r["backlit_archive_bytes"]),
                                   backlit_gb=sum(r["backlit_archive_bytes"] or 0 for r in sp) / 1e9,
                                   by_fuel_backlit_gb={f: sum(r["backlit_archive_bytes"] or 0 for r in sp if r["fuel"] == f) / 1e9
                                                       for f in sorted({r["fuel"] for r in sp})},
                                   inside_domain=sum(1 for r in sp if r["inside_domain"]),
                                   conditions_unknown=sum(1 for r in sp if r["unknown_features"] and not r["outside_domain"]),
                                   unknown_by_feature=dict(Counter(k for r in sp for k in r["unknown_features"])),
                                   outside_domain_by_feature=dict(Counter(k for r in sp for k in r["outside_domain"])),
                                   inside_domain_with_backlit=sum(1 for r in sp if r["inside_domain"] and r["backlit_archive_bytes"]),
                                   inside_domain_backlit_gb=sum(r["backlit_archive_bytes"] or 0 for r in sp if r["inside_domain"]) / 1e9,
                                   inside_domain_by_fuel=dict(Counter(r["fuel"] for r in sp if r["inside_domain"])),
                                   outside_by_fuel=dict(Counter(r["fuel"] for r in sp if r["outside_domain"])),
                                   by_day=len({r["date"] for r in sp})),
        comment_flags=dict(Counter(f for r in tps for f in r["comment_flags"])),
        settings_missing=sum(1 for r in tps if not r["settings"]), timeline_missing=sum(1 for r in tps if not r["has_timeline"]),
        settings_change_within_file=sum(1 for r in tps if (r["settings"] or {}).get("settings_change_within_file")),
        gas_reading_points=len(gas), gas_readings_implausible=[(g["id"], g["gas_sum"]) for g in gas if not g["plausible"]],
        gas_mixture_date_disagrees=[(g["id"], g["when"], g["mixture_date"]) for g in gas if g["date_agrees"] is False],
        date_how=dict(Counter(r["date_how"] for r in tps)),
        pressure_timeline_vs_pt16=(lambda pairs: dict(n=len(pairs), max_rel=max(abs(a / b - 1) for a, b in pairs) if pairs else None))(
            [(r["pressure_pa"], r["chamber_pt16_pre_ignition_pa"]) for r in tps if r["pressure_how"].startswith("mean chamber pressure (timeline")
             and r["chamber_pt16_pre_ignition_pa"]]))
    SUMMARY.write_text(json.dumps(s, indent=1, ensure_ascii=False) + "\n", encoding="utf-8")
    print(json.dumps({k: v for k, v in s.items() if k != "by_date"}, indent=1, ensure_ascii=False))
    return 0


if __name__ == "__main__":
    sys.exit(main())
