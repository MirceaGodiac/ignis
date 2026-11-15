# SAME (PSI-102, held whole): what can be forecast, from what, and how it is scored

> Written by a delegated data agent (claude-opus-5-5) on 15 November 2026 for the seventh generation, from
> `runs/dev/gen7/BRIEF_DATA.md`. SAME is held whole (`docs/wsp/atlas/splits/PSI-102.json`, group `PSI-102:matrix`):
> its inputs were read, its outcomes were written by script to a separate file and never printed, counted only for
> presence. Its blind aggregate score in a registered look is its only use. No shared contract was changed.

## For the physicist

The same question and form as SAME-R ([samer_feasibility.md](samer_feasibility.md)): **quantiles at 5, 25, 50, 75
and 95 % of the count mean diameter `d10` (m) at `net_aging_time` (s)**, from the inputs of
`runs/dev/data_all/work/transfer_same/cases_held_inputs.jsonl` (same names and units as SAME-R), and optionally
**the probability that the ISS detector's scatter channel is at full scale (|V| >= 6.0 V)**. Two differences
from SAME-R: a fifth material, `dibutyl_phthalate` (a liquid held in a cotton wick, 20 of 45 rows), and no Pyrell.
No `prior` is in the inputs file.

## 1. Source and what was read

SAME has no raw file; NASA published 59 analysed run workbooks and, inside PSI-101's archive, the results workbook
of both campaigns (`PSI-101/Analyzed Data/PSI-101_Analyzed Data_SAME, SAME-R_Results.xlsx`). Its sheet "Combined
Rslts Copy this sheet" holds 45 SAME rows (column "Flight 1=same 2=sameR" = 1): 29 primary measurements and 16
aged ("a") rows of runs 5 to 47, 9 September to 5 October 2007. The same script that builds SAME-R builds SAME
(`tools/wsp/ext/samer_cases.py build`), with the same columns and rules. The catalogue has only NASA's 8-row
planned matrix for SAME, so the run rows of the workbook are the test list; all are held.

Read: the methods sections of `PSI-102_Reports_SAME_Final Science Report.pdf` (moment method, instruments, detector
alarm criteria), the file listing and the workbook headers. **Seen by mistake** while learning the sheet layout:
the condition cells, net aging time included, of SAME rows 18, 19 and 24. No SAME outcome was displayed.

## 2. Targets and score

As SAME-R: `d10` = column "M1/M0 (micro-m)--AMD", CRPS of ln(d10) from quantiles; ISS full scale from "ISS
Detector Scatter (volts)", Brier. `score()` of `samer_cases.py` applies unchanged to
`cases_held_outcomes.jsonl`. With one held group, the per-day score is the aggregate.

## 3. Inputs

The names, units and trust of SAME-R section 4, with these SAME notes: the duct flow is nominally 5 or 10 cm/s in
the plan and about 2 to 8 cm/s in flight; DBP points were repeated; the SAME TEM grids were contaminated (not used
here); mass loss is present on 25 of 45 rows; sample temperature on 41.

Shared quantity list: as SAME-R, only `inlet_velocity` (`FlowVelocity`), `sample_temperature` (`Temperature`) and a
mass-loss rate (`MassFlow`) are representable; the targets are lane-local.

## 4. Counts (held: no partition)

| | Rows | Scored for d10 | Scored for ISS full scale |
|---|---|---|---|
| Primary measurements | 29 | 17 | 29 |
| Aged rows | 16 | 8 | 16 |
| All | 45 | 25 | 45 |

Materials: DBP 20, silicone 7, Teflon 6, Kapton 6, lamp wick 6. That d10 is absent on 20 rows is a count of empty
or non-numeric cells, not a value.

## 5. Baselines for the registered look (not computed on SAME)

Computed on SAME-R train days only (constants in samer_feasibility.md section 6) and applied unchanged: B0 the
SAME-R train constant; B1 the SAME-R train per-material quantiles, with B0 for DBP (not in SAME-R); C classical
coagulation from the primary row for aged rows (Hinds constants, nothing fitted); I0 and I1 incidences for the ISS
target, I0 for DBP. P (persistence) and C need the primary row's measurement, which is in the outcomes file: the
evaluator passes it, never an author. No score of any of them on SAME has been computed.

## 6. Images and video

No image in the archive; 2 video files (14.2 MB). Neither is needed for the targets.

## 7. Missing or untrustworthy

As SAME-R, plus: d10 is missing on 20 of 45 rows; the SAME campaign used two DustTraks and two ionization
chambers (SAME-R replaced one enclosure with MPASS), so the column meanings were checked by header only; DBP is a
liquid aerosol from a wick and has no SAME-R counterpart.

## 8. Files

`runs/dev/data_all/work/transfer_same/cases_held_inputs.jsonl` (45) and `cases_held_outcomes.jsonl` (45; never
printed), built by `python -I tools/wsp/ext/samer_cases.py build`.
