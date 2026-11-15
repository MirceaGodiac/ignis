# CFI (PSI-39) and FLEX-2 (PSI-68): can the frozen droplet models be run on them?

> Written by a delegated analysis agent (claude-fable-5-2, high effort) on 14 November 2026 for the fifth generation;
> nothing fitted. It reads the file listings, the experiment tables, the analysed-data sheets of CFI and a few small files of both investigations, which are outside every partition; it scores no model and reports no outcome of a test.
> It is a proposal to the author, not a model: no number here is a weight until the author has rechecked it
> on train and typed it. What was rechecked is recorded in `docs/wsp/dossier/03_datasets_and_measurements.md`.
> Paths under `runs/dev/` are local working folders that are not in git.

No CFI test can be used for the transfer test: the whole campaign burned n-dodecane, dodecane/iso-dodecane mixtures and farnesane, with no methanol or n-heptane. I therefore did not write `tools/wsp/ext/cfi.py`. FLEX-2 is the only candidate (pure heptane is in its matrix), but it has no processed diameters and no per-test fuel or gas composition in any file I checked.

## (a) What CFI's 32 "Analyzed Data" files are

They total 13.6 MB and are now in `runs\dev\data_all\raw\samples\PSI-39\`.

- **31 PDFs:** Igor "summary graph" sheets, vector graphics, for 25 distinct tests (FLEX-1206 to 1264, plus 1393); five tests have a preliminary and a final version.
- **Content of a sheet:** a plot of (D/D0)² against t/D0² in s/mm², flame diameter and standoff ratio, radiometer traces, and on later sheets the cool-flame diameter. The annotation gives date, FLEX number, CIR test point, pressure in mmHg, mole fractions of O2, N2, CO2 and He, fuel, D0 in mm, and the investigators' derived rates and extinction diameters (post-run facts).
- **1 Igor file** (`FLEX-1206_SummaryGraph.pxp`, 77 KB): six numeric waves, including `Droplet_Data/t_Do_sq` and `DA_Do_sq` (1,744 points).
- **Comparable to FLEX's (time, diameter) files?** Not directly. The history is a normalised curve drawn in a PDF for 24 readable tests, and numeric (still normalised) for one test.
- **Where the dimensional series is:** in the Raw Data Igor files `FLEX_<n>_<date>_Dod_v<k>.pxp`. I fetched one (14 MB): it holds `DropletTime_sec`, `DropletSizeA_mm` and positions at 30 frames/s, with time zero inside the series, as in FLEX. With D0 from the sheet, they reproduce the normalised waves to about 1e-5.
- **Coverage of those raw files:** 22 tests in 28 files, 975 MB (736 MB taking the latest version of each).
- **Fuels and configuration of the 25 analysed tests:** 24 n-dodecane, 1 farnesane. Eight are free-floating (1206, 1215, 1221, 1223, 1239, 1240, 1250, 1264), 16 fibre-supported, 1 blank (1393).

## (b) Counts, condition ranges, and the models' domain

The table `CFI.csv` has 87 rows but 79 tests (FLEX-1285 to 1289 and 1295 to 1297 appear twice, identically). The complete campaign is in `PSI-39_Science Documents_CFI_DataSummary_Spreadsheet_Jan2021.xlsx` (0.2 MB, fetched): 227 tests.

| | CFI.csv (79) | Spreadsheet (227) |
|---|---|---|
| n-dodecane | 22 | 80 |
| 75/25 dodecane/iso-dodecane | 16 | 51 |
| 60/40 dodecane/iso-dodecane | 41 | 52 |
| Farnesane | 0 | 44 |
| Methanol or n-heptane | **0** | **0** |
| Fibre No / Yes / blank | 25 / 45 / 9 | 55 / 103 / 69 |

- **CFI conditions:** oxygen 0.17 or 0.21; nominal pressure 0.5, 0.75, 1, 1.25, 2, 3 or 5 atm; nitrogen 0.20 to 0.83; helium 0, 0.29 or 0.59; no CO2 column.
- **D0** (spreadsheet, 148 tests): 3.79 to 6.45 mm.

FLEX train ranges, read through `lib.py`:

| | Methanol (68 runs) | n-Heptane (49 runs) |
|---|---|---|
| O2 | 0.12 to 0.34 | 0.14 to 0.34 |
| CO2 | 0 to 0.70 | 0 to 0.70 |
| He | 0 to 0.45 | 0 to 0.35 |
| Pressure | 0.69 to 3.05 atm | 0.70 to 2.04 atm |
| D0 | 1.61 to 4.28 mm | 1.35 to 4.16 mm |
| Diluent class N2 / CO2 / He | 29 / 27 / 12 | 21 / 20 / 8 |

The methanol pressure range excludes the one refused record stored as zero. Above 1 atm, train has only CO2-diluted runs.

**Inside the models' domain: 0 of 79 and 0 of 227**, because of fuel. The forward refuses a fuel other than its own, and both releases declare "single free-floating droplet" of that fuel. Ignoring fuel, of the 227 tests:

| Filter | Methanol box | Heptane box |
|---|---|---|
| Free-floating and atmosphere inside the train box | 48 | 45 |
| Also D0 known and inside the train range | 23 | 10 |

## (c) Column mapping and gaps

| CFI.csv column | FLEX record key | Notes |
|---|---|---|
| `Flex Number` | alias `FLEX-<n>` | |
| `Test Date` + `CIR` | `id` = `PSI-39:<CIR>:<date>`, `session_id` = `PSI-39:day:<date>` | CIR is lower case with an "f" prefix |
| `Pressure` ("1 atm") | `pressure`, atm to Pa | Nominal fill value; FLEX stores measured mmHg |
| `Oxygen`, `Nitrogen`, `Helium` | `x_o2`, `x_n2`, `x_he`, unit 1 | Nominal |
| `Fiber` | `fiber`, and the flag `free_floating:yes/no` | Blank for 9 of 79 |
| `Fuel` | `fuel`, `fuel_or_material` | |
| `Local Time` (EST) | none | FLEX identity uses GMT |

Missing from the table:
- **`x_co2`:** the forward requires it. The summary sheets give 0.000 to 0.002.
- **`initial_diameter`:** in the spreadsheet for 148 tests and on the sheets. The two frozen models do not read it; they take D0 from the head of the prefix.
- **`ambient_temperature`**, measured pressure and GMT time: the test sheets (`FLEX-<n>_TestSheet.rtf`, 22 tests, same fields as FLEX's) carry all three; `<CIR>_Chamber_Readings.txt` has pressure and temperature, but I did not confirm it exists for every test.

Table defects: rows 1219 to 1222 sum to 0.97; FLEX-1213 has oxygen 0.17 in the table and 0.21 on its summary sheet.

## (d) FLEX-2 (PSI-68)

- **No processed diameter data in the listing.** It holds 7,925 text files (0.53 GB) and 2,057 image zips (1,220 GB); nothing is named as droplet data, summary or Igor file. Diameters would have to be measured from the images.
- **Size for one test:** of 722 test points, 519 have an "A" archive, which I take to be the backlit droplet camera by analogy with CFI. Its median is 1.28 GB (10th to 90th percentile 0.77 to 2.5 GB; smallest 0.24 GB). Everything for one test is 2.1 GB median.
- **Conditions per test:** `Flex2.csv` is only an 8-row planned matrix. I fetched one test point's small files (0.45 MB). They give the igniter-on stamp, a fibre-usage flag, flow-nozzle speeds, and chamber pressure and temperature. The event-log entry is a picture of the same timeline.
- **Blocking gap:** none of those files gives the fuel or the oxygen and diluent fractions.

## (e) Converter

`tools/wsp/ext/cfi.py` does not exist and nothing was written under `runs/dev/data_all/work/cfi/`. The reason is the one in (b): no CFI test is methanol or n-heptane.

A dodecane converter is technically straightforward. A 60-line reader of Igor files that I wrote and ran on two files is at `D:\Caches\Temp\claude\4770acc3d7bcc157\09a9b946-ecdb-454d-a1e6-4da199bfa676\scratchpad\pxp.py`; the scratchpad is not durable.

## (f) Proposed inclusion rule and metric

Inclusion, fixed before any image is opened, for FLEX-2 n-heptane only (no methanol in its matrix):
1. Fuel recorded as pure n-heptane in a pre-test source.
2. Single droplet, fibre flag off, both flow-nozzle speeds zero.
3. Atmosphere inside the heptane train box, by diluent class:
   - N2 only: O2 0.14 to 0.34, 0.71 to 1.01 atm.
   - CO2: O2 0.17 to 0.25, CO2 up to 0.70, 0.70 to 2.04 atm.
   - He: O2 0.19 to 0.21, He up to 0.35, 0.70 to 1.01 atm.
   - Tests inside the declared domain but outside this box form a separate, labelled extrapolation cohort.
4. D0 between 1.35 and 4.16 mm, measured only on frames at or before ignition.
5. Backlit archive and igniter-on stamp present.
6. Every eligible test is taken; if download must be capped, take test points in id order at a fixed stride, set beforehand.

Metric: the development one. CRPS of liquid diameter in µm, cases are run × cutoff (1, 3, 5 s), with mean and median over cases and coverage of the 50 % and 90 % bands. The primary result is the paired difference against the D²-law baseline of the same spec, with a bootstrap over day-sessions. Refused cases are counted, not scored. Report the level beside FLEX validation (46.1 µm) as context only, because the optics and the tracker differ.

## (g) Remaining work and download

- **CFI:** nothing, for these models.
- **FLEX-2, three steps:**
  1. Obtain fuel and gas composition per test (outside the archive files I checked).
  2. Write a droplet tracker and check it against NASA's own processed series on FLEX train runs.
  3. Download about 1.3 GB per test: roughly 26 GB for 20 tests, 65 GB for 50.
- The number of heptane free-floating tests is unknown until step 1.

## (h) Not verified, and what was touched

- `PSI-39_Analyzed Data_FLEX-1248_Summary.pdf` is not a readable PDF as served (382,777 bytes, matching the listing, no PDF header); its fuel is assumed from the test-number range.
- For FLEX-2 I did not confirm that the "A" archive is the backlit view, nor its image format or scale. I did not open the metadata zip, the SRD or the calibration files, which may hold the reservoir-to-fuel mapping.
- I did not digitise the PDF curves or read the other 27 raw Igor files.
- While checking formats I saw derived outcome values for three CFI dodecane tests and one test-sheet comment; none is in this report.
- Total downloaded: 31.3 MB, no zip, image or video. The one FLEX-2 event-log `.docx` (40 KB) is technically a zip container; I opened it only to read its fields.
- The fetch command added rows to `manifests/PSI-39.jsonl` and `PSI-68.jsonl`.
- `runs/dev/data_all/work/flame_design/` and `tools/wsp/ext/flame_design.py` appeared during the session; they are not mine and I left them alone.
