# File catalogue of the 19 flight combustion investigations: the agent's report

> Written by a delegated analysis agent (claude-fable-5-2, high effort) on 14 November 2026 for the fifth generation;
> nothing fitted. It classifies the 66,501 rows of NASA's file listings by name, category and folder, with rules only; it reads no outcome of any test of FLEX, ACME CFI-G, BASS or BASS-II.
> It is a proposal to the author, not a model: no number here is a weight until the author has rechecked it
> on train and typed it. What was rechecked is recorded in `docs/wsp/dossier/03_datasets_and_measurements.md`.
> Paths under `runs/dev/` are local working folders that are not in git.

The catalogue is built: every one of the 66,501 listed files has exactly one row, the bytes add up to the listings (13,496,418,513,932), and two fresh runs are byte-identical. One file is unclassified, and 123 files (1.1 GB) carry confidence "unknown".

## Files
- `tools\wsp\atlas\classify_files.py` (rules only)
- `tools\wsp\atlas\check_files.py`
- `docs\wsp\atlas\FILES.md` (naming scheme and file-kind table per investigation)
- `runs\dev\data_all\work\atlas\files.jsonl`, `files_summary.json`, `files_tables.md`

## Summary per investigation
"Tests" are ids found in file names; the four right-hand columns count tests that have that kind of file.

| Investigation | Files | Bytes | Unclassified / unknown | Tests | Sensor series | Image archive | Video | Processed |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| PSI-10 Flame Design | 3,807 | 2.18 TB | 0 / 0 | 323 | 317 | 315 | 317 | 0 |
| PSI-20 BRE | 2,361 | 2.04 TB | 0 / 0 | 208 | 187 | 207 | 189 | 0 |
| PSI-21 CLD Flame | 4,883 | 2.01 TB | 0 / 10 | 359 | 352 | 324 | 350 | 0 |
| PSI-22 E-FIELD Flames | 2,590 | 950 GB | 0 / 0 | 225 | 204 | 219 | 215 | 0 |
| PSI-23 s-Flame | 4,165 | 782 GB | 0 / 0 | 291 | 286 | 289 | 285 | 0 |
| PSI-159 CFI-G | 3,756 | 2.09 TB | 0 / 0 | 338 | 337 | 331 | 335 | 0 |
| PSI-69 FLEX | 6,555 | 924 GB | 0 / 0 | 415 test points; 272 FLEX numbers | 278 | 411 | 120 | 269 (by FLEX number) |
| PSI-68 FLEX-2 | 10,156 | 1.22 TB | 0 / 7 | 596 | 586 | 535 | 0 | 0 |
| PSI-39 CFI | 2,812 | 719 GB | 0 / 0 | 133 test points; 27 FLEX numbers | 123 | 85 | 86 | 25 (by FLEX number) |
| PSI-25 BASS-II | 695 | 275 GB | 0 / 1 | 115 | 0 | 115 | 112 | 0 |
| PSI-26 BASS | 212 | 164 GB | 0 / 0 | none (37 days) | 0 | 35 days | 12 days | 0 |
| PSI-98 SAFFIRE-I | 76 | 12.3 GB | 0 / 1 | 2 | 2 | 2 | 0 | 2 |
| PSI-99 SAFFIRE-II | 217 | 16.9 GB | 0 / 1 | 9 | 9 | 9 | 0 | 9 |
| PSI-100 SAFFIRE-III | 87 | 35.2 GB | 0 / 2 | 2 | 2 | 2 | 0 | 2 |
| PSI-101 SAME-R | 344 | 6.3 GB | 0 / 1 | 67 | 44 | 45 | 0 | 66 |
| PSI-102 SAME | 101 | 0.66 GB | 0 / 0 | 42 | 0 | 0 | 0 | 42 |
| PSI-106 SLICE | 23,463 | 72.9 GB | 1 / 97 | none (17 days) | 0 | 17 days | 6 days | 13 days |
| PSI-107 SPICE | 188 | 6.9 GB | 0 / 3 | 53 | 0 | 53 | 0 | 0 |
| PSI-47 DAFT | 33 | 41.8 MB | 0 / 0 | 6 | 6 | 0 | 0 | 6 |

- **How sure:** 43.3 % of files rest on the name alone, 32.2 % on a sibling pattern, 24.4 % on a file of that kind that was opened.
- **ACME counts are test names, not flames:** PSI-159 has 338 names against 260 tests in the splits.
- **FLEX and CFI have two naming schemes** (rack test point for raw files, FLEX number for processed files). No FLEX file name carries both, so the two rows cannot be joined without the experiment table, which I did not open.

## Left unclassified or unknown
- **Unclassified:** `Preflight vs Yale.xlsx` (PSI-106, 59.9 KB).
- **Unknown confidence:**
  - PSI-106: 91 PDFs named only by a range of picture numbers, and 5 `.txt.docx` files.
  - PSI-21: `18257.pdf` (9 rows of one file) and `18214F1.zip` (765 MB).
  - PSI-68: 7 archives named only by a point (`LCTF033.zip`, `BKGD260.zip`, …).
  - SAFFIRE: 4 `MergeResult` workbooks.
  - PSI-107: 3 `GMT###nikon.xls` workbooks.
  - PSI-25: `BASS_GMT51_SAMPLE_223_TEST_M14.zip`.
  - PSI-101: `ISS Coagulation.doc`.
- **Weak but not marked unknown** (listed in FILES.md section 7): the ACME day PDFs, the SLICE `T_`/`fv_`/`CH_` arrays, "Converted" image archives (4.1 TB), and `_INTNS` against `_INTNS_ALL`.

## What one test costs to download
Median per test, with the 10th to 90th percentile in brackets.

| Investigation | Text and tables | Images | Video |
|---|---|---|---|
| PSI-10 | 3.7 MB (2.6–5.6) | 5.1 GB (2.3–11.8) | 220 MB |
| PSI-20 | 5.0 MB | 8.7 GB | 179 MB |
| PSI-21 | 2.8 MB | 5.5 GB | 90 MB |
| PSI-22 | 7.5 MB | 4.1 GB | 98 MB |
| PSI-23 | 2.6 MB | 2.3 GB | 120 MB |
| PSI-159 | 4.0 MB (3.1–6.8) | 4.3 GB (2.4–10.1) | 233 MB |
| PSI-69 (test point) | 0.6 MB | 2.2 GB | none for most |
| PSI-69 (FLEX number) | 1.4 MB | none | none |
| PSI-68 | 0.7 MB | 2.0 GB | none |
| PSI-39 (test point) | 1.5 MB | 5.4 GB | 0.8 GB |
| PSI-25 | none | 1.9 GB | 246 MB |
| SAFFIRE-I / II / III (per sample) | 30 / 18 / 24 MB | 9.3 / 1.6 / 21.7 GB | none |
| PSI-101 | 7.0 MB | 12.7 MB | none |
| PSI-102 | 4.9 MB | none | none |
| PSI-107 | none | 7.9 MB | none |
| PSI-47 | 0.3 MB | none | none |

Across all 19 investigations, images are 12.9 TB and video 573 GB. Everything else is 23.4 GB, of which the sensor time series are 8.6 GB.

## What surprised me
- **The listings repeat rows.** 5,701 rows are exact repeats of an earlier row (SLICE 4,756, CLD Flame 749), so there are 60,800 distinct files, 13.47 TB. Each repeat keeps its row and is flagged.
- **The flat listings drop the folder, and the folder carries the conditions.** The raw listing trees (`_meta/files_PSI-<id>.json`) are on disk for all 19; I joined them by file id. They give fuel, burner, polarity, investigator or material in 18 investigations, and the test id for 522 files whose name has none.
- **FLEX-2 fuel and atmosphere are available.** The earlier feasibility note says no file gives them. The folder of each test point names the fuel (18 test points under "SurrogateFuel 2013 - Heptane"), and 124 tiny `_Mixture.txt` files give the chamber composition; I opened one. I did not work out which reading belongs to which burn.
- **Names and folders are mistyped.** The CFI-G folders of day 21209 are spelled 21909, 29 ACME files have a wrong day in the name (refiled by folder and flagged), and ten s-Flame tests sit in folders named for another day.
- **Some known tests have no raw files.** Six known FLEX runs and one CFI-G test (`21130A9`) have no file named by their test point.
- **Three investigations file everything under "Raw Data".** CLD Flame, FLEX-2 and SLICE do this, without the `PSI-<id>_` prefix; CLD Flame has 64 documents there.
- **NASA's own totals differ slightly.** For CFI, SAFFIRE-II and SLICE the record total exceeds the listed files by 0.4 to 13.6 MB.

## What I opened or changed outside the brief
- **Fetched:** 38 files, 1.6 MB, no zip, image or video. One is a 0.38 MB PDF (the ACME non-imaging data description), which is above your "few KB" limit. The fetch command appended records to `runs/dev/data_all/manifests/PSI-<id>.jsonl`.
- **Shared FLEX records:** I printed the first 1,500 bytes of `data/work/records/flex/experiments.jsonl` to see the id format. It showed the aliases and the start of the conditions of one record, no outcome field.
- **Sealed folders:** not opened.
- **FILES.md prose:** the tables are regenerated by `--tables`, but the prose was assembled once by a script in the scratchpad that is not in the repo.

## Rerun
```bash
python tools/wsp/atlas/classify_files.py            # files.jsonl, files_summary.json
python tools/wsp/atlas/classify_files.py --tables   # also files_tables.md
python tools/wsp/atlas/check_files.py               # rows, bytes, closed lists, two identical runs
```
