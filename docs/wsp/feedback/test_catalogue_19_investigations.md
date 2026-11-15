# Test catalogue of the flight combustion investigations: the agent's report

> Written by a delegated analysis agent (claude-fable-5-2, high effort) on 14 November 2026 for the fifth generation;
> nothing fitted. It normalises the experiment tables of 15 investigations outside the partitions and the conditions of train and validation runs of the four covered ones into one schema; outcome values are kept in a separate file and none is in this report.
> It is a proposal to the author, not a model: no number here is a weight until the author has rechecked it
> on train and typed it. What was rechecked is recorded in `docs/wsp/dossier/03_datasets_and_measurements.md`.
> Paths under `runs/dev/` are local working folders that are not in git.

The atlas is written and checked: 1,612 rows from 18 of the 19 investigations, with `check_atlas.py` passing (two fresh runs byte-identical, every row valid against the schema, no outcome value in `tests.jsonl`). PSI-26 has no row, because no record of it is reachable through `lib.py`.

The coordinator's mid-task change is applied: outcome values are only in `tests_outcomes.jsonl`, `tests.jsonl` carries outcome field names and a `group_id` per row, and the docs describe outcome fields by name, count and unit only.

## Counts

| PSI | Acronym | Rows | Level | Groups | Oxygen and pressure, one value each |
|---|---|---|---|---|---|
| 10 | Flame Design | 327 | test | 22 | 316 (207 ambient, 109 injected oxidiser) |
| 20 | BRE | 18 | test day | 18 | 0 |
| 21 | CLD Flame | 32 | test day | 32 | 0 |
| 22 | E-FIELD Flames | 20 | test day | 20 | 0 |
| 23 | s-Flame | 18 | test day | 18 | 13 (nominal, about 1 atm) |
| 25 | BASS-II | 21 | test | 3 | 0 (oxygen only) |
| 26 | BASS | 0 | | | |
| 39 | CFI | 227 | test | 51 | 227 |
| 47 | DAFT | 2 | test day | 2 | 0 |
| 68 | FLEX-2 | 8 | matrix row | 1 | 0 (sets) |
| 69 | FLEX | 156 | test | 60 | 156 |
| 98 / 99 / 100 | Saffire I / II / III | 2 / 9 / 2 | test | 1 each | 0 (oxygen for 1 / 7 / 0) |
| 101 | SAME-R | 109 | 67 tests, 42 test points | 16 | 0 |
| 102 | SAME | 8 | matrix row | 1 | 0 |
| 106 | SLICE | 12 | test day | 12 | 0 |
| 107 | SPICE | 435 | test point | 11 | 0 |
| 159 | CFI-G | 206 | test | 18 | 206 (109 ambient, 97 injected) |

- **By level:** 1,017 tests, 477 test points, 102 test days, 16 matrix rows.
- **By fuel phase:** gas 1,068; liquid droplet 391; solid 150; other liquid 1; not applicable 2.
- **Oxygen and pressure:** 918 rows have one value of each; 712 of those are ambient oxygen, the only ones compared with crew atmospheres.
- **Earth and ISS** (oxygen 0.20–0.22, 14.7 ± 0.5 psi): 186 rows — Flame Design 64, CFI 63, FLEX 35, CFI-G 14, s-Flame 10 day rows. Nine more are in the oxygen band with pressure unknown.
- **Moon base and rovers** (oxygen ≥ 0.30, ≤ 10.2 psi): 17 rows, all CFI-G. Another 29 day or matrix rows (BRE 16, s-Flame 5, FLEX-2 8) list an oxygen value in the band without a single pressure.
- **Spacesuit** (oxygen ≥ 0.99, 4.3 ± 0.5 psi): none.

The ± 0.5 psi and ± 0.01 oxygen tolerances are my choice; they are stated in `SCHEMA.md` and the summary.

## Field coverage (of 1,612 rows)

- **Well covered:** gravity (all), fuel name (1,611), phase (1,610), rack (1,598), date (1,583), configuration (1,406), flow direction (1,198).
- **Partly covered:** pressure (934), ambient oxygen (743 single, 29 sets, plus 206 injected-oxidiser values), start time (646), flow speed (549; not applicable for 580 burner rows), diluents (440).
- **Mostly empty:** ambient temperature (164, FLEX and FLEX-2 only), and `measurements` (defined, not filled, as asked).
- **Basis:** 4,428 resolved values are marked `unstated` rather than guessed as nominal or measured.

## Table defects found

- **Flame Design:** test `22042M1` sits in Day 6 among the 22024 tests; the listing has files for `22024M1` and none for `22042M1`, so the day code is transposed.
- **BRE:** two rows carry the wrong month ("Oct 17" for 17 September, "Dec 5" for 5 October), contradicted by the Readme table and the file listing. Two oxygen cells read "3400%" and "2100%".
- **CLD Flame and s-Flame:** 26 and 8 cells are spreadsheet fill-series artefacts ("40% CH5" … "40% CH14", "~21% O2 with N3"). One s-Flame row is labelled nitrogen-diluted while its gases are helium-diluted.
- **E-FIELD:** the block for day 18319 is repeated exactly.
- **CFI:** 8 repeated CSV rows; fractions sum to 0.97 for Flex 1219–1222; two date cells (Flex 1237, 1301) contradict their CIR test-point day and their neighbours.
- **Saffire:**
  - Fabric thickness is given in cm in I and III against 0.33 mm in the Saffire-II Readme.
  - Length and width are exchanged in I.
  - In II, three thickness cells read "037 mm", two material cells are blank, and air flow is 20 cm/s against 30 in the Readme for samples 6 and 9.
- **SAME-R:** test number 57 appears under both Lampwick and Teflon; file names have it only as lampwick. For 15 runs the date is a day off the GMT day in file names.
- **SPICE:** 34 repeated rows, 12 unreadable ignition times, 7 rows with a spurious zero fuel rate, 8 negative air rates, and 11 days against the report's twelve sessions.
- **SLICE:** a sentence is split across the two columns on the last row.

## Not resolved, and judgement calls

- **PSI-26:** zero rows. The `bass` family in `lib.py` holds only PSI-25 runs. A project doc says PSI-26 is outside every partition; if you confirm, its table can be ingested like the others.
- **CFI-G burner:** the records give only flame orientation, so configuration and flame type are empty for all 206 rows. I did not open its report because it holds outcomes.
- **Carry-over inside a day:** in Flame Design, 249 of 307 final compositions equal the initial composition of another test of the same day. Initial conditions are legitimate but carry information about the previous test, so splits must keep a day whole.
- **Groups that depart from the plain test day:** Flame Design uses the table's Day column, CFI uses the CIR day code, and SAME-R secondary runs take their parent's group. Otherwise 8 tests would have been separated from their day or their parent run.
- **SPICE flows are kept as conditions.** The table does not mark smoke points; if its rows turn out to be smoke points, the fuel rate is an outcome and must move to the held file.
- **Zst and Tad are treated inconsistently:** held as outcomes for Flame Design (computed with the final composition) but copied as conditions for CFI-G, because the project records class them so.
- **Oxygen lists for BRE and E-FIELD** are flagged `conditioned_on_ignition`: the tables list only values of tests that ignited.
- **Three Saffire oxygen cells** given as a range over the burn are withheld.
- **Hand transcriptions:** two Readme tables exist only as pictures; I transcribed them into `tools/wsp/atlas/transcribed/`, leaving out the outcome column.
- **Fire-safety need** is a keyword rule on title, objective and approach. It is coarse; for example it labels Flame Design "prevention" from "pseudo-flammability limits".
- **What I saw:** outcome values of the 15 new tables while profiling them, and the top of `bass1_psi26_tracks.md`, which holds PSI-26 as-run outcomes. None of it is in the outputs or this report.
- **Downloads:** about 28 MB of reports, Readmes and small logs; no zip, image or video. The fetch command appended rows to `runs/dev/data_all/manifests/PSI-*.jsonl`.

## Rerun

```
python tools/wsp/atlas/normalize_tests.py
python tools/wsp/atlas/check_atlas.py
```

Needs `openpyxl` (CFI spreadsheet) and `jsonschema` (check).

## Next source per investigation

| Investigation | Ingest next |
|---|---|
| Flame Design | Per-test `as run<T>.txt` and `_mfcSetpoints.csv` for ramp flows and the balance gas |
| BRE, CLD Flame | `Final ACME Chamber Calculator` workbooks (about 0.5 MB each). I opened one: it has pressure before and after and gas sources per test point. For CLD they do not cover the first 14 days. |
| E-FIELD | Per-test `_TP.txt` and the flow-and-pressure conversion workbook |
| s-Flame | Per-test `as run<T>.txt` and set-point files. The Daily Test Point Log holds only image counts. |
| CFI | Test sheets and `Chamber_Readings.txt` for measured pressure, temperature and CO2 |
| FLEX-2 | Per-test settings files for the test list, pressure and temperature. Fuel and gas per test are not in any file checked; try the Experiment Data record names or the metadata zip. |
| Saffire | Per-sample sensor workbooks (12–33 MB each) for pressure and initial oxygen |
| SAME-R | `SAME, SAME-R_Results.xlsx` and the per-run workbooks for heating temperature |
| SAME | Run table in the final report's appendix (already fetched), which gives test-level rows |
| SLICE | `pic_info_<day>.csv` and the boundary-condition workbooks |
| SPICE | `Spice Flight Test Data II modified.xls` (fetched; not parsed, no `.xls` reader installed) |
| DAFT | Analysed-data workbooks; the report mentions 2006 sessions the table lacks |
| FLEX, BASS-II, CFI-G | Pending your decision on which further runs and fields may be read |
| BASS (PSI-26) | Its experiment table, once you confirm it may be read |

## Files

- docs\wsp\atlas\SCHEMA.md
- docs\wsp\atlas\INVESTIGATIONS.md
- tools\wsp\atlas\normalize_tests.py
- tools\wsp\atlas\check_atlas.py
- tools\wsp\atlas\atlas_schema.py
- tools\wsp\atlas\transcribed\psi20_bre_readme_table1.csv
- tools\wsp\atlas\transcribed\psi99_saffire2_readme_table1.csv
- runs\dev\data_all\work\atlas\tests.jsonl
- runs\dev\data_all\work\atlas\tests_outcomes.jsonl
- runs\dev\data_all\work\atlas\tests_summary.json
- runs\dev\data_all\work\atlas\tests.schema.json
