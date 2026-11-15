"""Write runs/dev/heptane_fresh/README.md from the freeze file, the measurement log and the series.

Counts, conditions and quality only: no burn time, rate or extinction is computed. The initial diameter
d0 is reported as the first written observation (a measured quantity a forecaster conditions on).

    python -I tools/wsp/media/heptane_fresh_readme.py
"""
import json
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
FREEZE = REPO / "docs/wsp/atlas/splits/heptane-fresh-blind.json"
OUT = REPO / "runs/dev/heptane_fresh"


def main():
    b = json.loads(FREEZE.read_text(encoding="utf-8"))
    log = {}
    if (OUT / "measure_log.jsonl").exists():
        for line in (OUT / "measure_log.jsonl").open(encoding="utf-8"):
            r = json.loads(line)
            log[r["test_id"]] = r
    man = {}
    if (OUT / "raw/MANIFEST.jsonl").exists():
        for line in (OUT / "raw/MANIFEST.jsonl").open(encoding="utf-8"):
            r = json.loads(line)
            man[r["test_id"]] = r
    rows, n_use = [], 0
    for t in b["tests"]:
        a = t["ambient"]
        co2 = a["x_co2"] if a["x_co2"] is not None else "/".join(str(x) for x in a["x_co2_alternatives"]) + " (unresolved)"
        sp = OUT / "series" / f"{t['test_id']}.json"
        if not t["in_measured_batch"]:
            status, d0, n, end, why = "not measured", "", "", "", "fibre-supported (configuration, frozen before any frame)"
        elif sp.exists():
            s = json.loads(sp.read_text(encoding="utf-8"))
            status = s["quality"]
            obs = s.get("observations") or []
            d0 = f"{obs[0]['value_si'] * 1e3:.2f} mm at t={obs[0]['time_s']:.2f} s" if obs else ""
            n = len(obs)
            vc = s.get("visual_check") or {}
            end = vc.get("end_of_series", "visual check pending")
            if vc.get("verdict") == "PASS" and status == "usable":
                status = "usable (visual PASS)"
            why = "; ".join(s.get("quality_reasons") or []) or vc.get("note", "")
            n_use += s["quality"] == "usable"
        elif t["test_id"] in man and man[t["test_id"]]["status"] not in ("ok", "present"):
            status, d0, n, end, why = "download failed", "", "", "", man[t["test_id"]]["status"]
        else:
            status, d0, n, end, why = "pending (not yet downloaded or measured)", "", "", "", ""
        rows.append(f"| {t['test_id']} | {t['investigation']} | {t['date']} | {t['configuration']} | "
                    f"{t['pressure']['value_pa'] / 1000:.1f} | {a['x_o2']} | {co2} | {a['grade']} | {status} | "
                    f"{d0} | {n} | {end} | {why} |")
    m = b["method"]
    text = f"""# Fresh blind n-heptane droplet series

Measured blind for scoring a frozen n-heptane candidate against the adopted release. **Model agents do
not read this folder.** No model was run, scored or opened by the measurer.

Freeze (committed before any frame was fetched): `docs/wsp/atlas/splits/heptane-fresh-blind.json`,
written by `tools/wsp/media/heptane_fresh_freeze.py`. Measurement: `tools/wsp/media/heptane_fresh_measure.py`
(fetch, measure, relabel). This page: `tools/wsp/media/heptane_fresh_readme.py`.

## Series format

`series/<test>.json`: `observations` = liquid_diameter in metres (`value_si`), `time_s` on the FLEX
processed time base (t = 0 at the timeline stamp 'AS Igniter Off/Retract Start'; `time_base.rack_igniter_on_s`
gives igniter-on on the same axis), one row per usable frame (tool flags ok and soot), camera frame number
in `row`. `conditions` as frozen. `quality` = usable / not_usable / failed with reasons; `visual_check` and
`endpoint_note.end_of_series` say why a series stops (left_view: the droplet drifted out of the field of
view, so the last diameter is NOT the end of the burn; vanished_in_view: it shrank or went out in view;
track_lost). Per-frame tool output with all flags: `tool/<test>.csv`; contact sheets: `check/<test>/`.

## Scale

{m['pixel_scale']['rule']}. Source: {m['pixel_scale']['source']} Uncertainty: {m['pixel_scale']['uncertainty']}.
Independent check per test, never used to rescale: the needle-arm width (`measurement.ruler_check`;
FLEX reference 1.60 to 1.73 mm).

## Tests ({sum(1 for t in b['tests'] if t['in_measured_batch'])} in the measured batch, {n_use} usable by the frozen rule so far)

Pressure in kPa. d0 = first written diameter (after needle retraction + 0.1 s).

| Test | Inv. | Date | Config | P kPa | x_O2 | x_CO2 | Ambient grade | Status | d0 | frames | End of series | Reasons / notes |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
""" + "\n".join(rows) + """

## Conditions: what is measured and what is assumed

- **2013-03-28 (E087, FLEX-2): the ambient is an assumption.** No gas reading that day; air (O2 0.2113,
  N2 0.7887) carried from E073R01 of 2013-03-14. No test or reading point of PSI-68 or PSI-69 lies between;
  the chamber pressure is continuous with the last 2013-03-14 test by the ideal gas law. A refill to the
  same pressure with another gas cannot be excluded. Pressure is measured (chamber PT16).
- **Day 021 and day 049 (PSI-69): bracketed** by gas readings before and after the session that agree.
- **Day 039: one reading**, taken at the reading point 20 min before the first test (GC line dated a
  week earlier); no reading after.
- **Day 030: conflicting.** The pre-session point gives CO2 0.50, the GC line dated two minutes after the
  last test gives CO2 0.75; the pressure shows no vent. The CO2 fraction is unresolved; both values are
  in the series. Decided before any frame: this day is a secondary set.
- d0 is not a condition of the files (no dispensed volume per droplet); it is the first measured diameter.
- PSI-69 2013 points have no NASA test sheet or processed series; fuel is from the test-point name
  (H = n-heptane), configuration from the MDCA settings fibre flag.

## State and how to finish

The download runs over a link of 1 to 5 MB/s, so the batch is measured as archives arrive. A test whose
status is `usable` without "(visual PASS)" has not had its contact sheets checked yet. To finish:
`python -I tools/wsp/media/heptane_fresh_measure.py fetch` and `... measure` (both resume), add a line per
new test to `VISUAL_CHECK.tsv` (test, PASS/FAIL, end of series, note), then `... relabel --visual
runs/dev/heptane_fresh/VISUAL_CHECK.tsv` and `python -I tools/wsp/media/heptane_fresh_readme.py`.
A scorer should use only series with `quality == "usable"` and a `visual_check.verdict == "PASS"`.

## Exclusions

By configuration, frozen before any frame: the 7 fibre-supported points (C039H03, H04, H07, H08;
C049H01, H02, H05). By measurement quality only: the series marked not_usable or failed above, with the
tool's or the screen's reason. No exclusion used any model output.
"""
    (OUT / "README.md").write_text(text, encoding="utf-8")
    print("README written;", n_use, "usable")


if __name__ == "__main__":
    main()
