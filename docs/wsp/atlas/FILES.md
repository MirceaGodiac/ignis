# The files of NASA's flight combustion investigations

NASA's PSI repository lists 66,501 files (13.5 TB) for its 19 flight combustion investigations. This page
says what each of them is and which test it belongs to, from the listings alone: the name of the file, its PSI
category and the folder it sits in. Nothing was downloaded to build it, apart from 38 small files opened to
settle what a kind of file contains (section 8).

The catalogue is one row per listed file in `runs/dev/data_all/work/atlas/files.jsonl` and one summary in
`files_summary.json`, written by `tools/wsp/atlas/classify_files.py` (rules only, no model call) and checked
by `tools/wsp/atlas/check_files.py`. The tables of section 6 are written by the same script
(`--tables`); the text around them is by hand. Paths under `runs/dev/` are local working folders that are not
in git.

No outcome of any test was read. The experiment tables of FLEX, ACME CFI-G, BASS and BASS-II were not opened;
the known test ids of those four were read through `tools/wsp/analysis/lib.py` (train and validation ids only)
to check the naming.

## 1. What is counted

| Investigation | Acronym | Listed files | Listed bytes | Distinct files | Unclassified | Sure: name | sibling | opened | unknown | Files with a test id | Files with a test day |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| PSI-10 | ACME/Flame Design | 3807 | 2.2 TB | 3807 | 0 | 1646 | 559 | 1602 | 0 | 3583 | 3790 |
| PSI-20 | ACME BRE | 2361 | 2.0 TB | 2361 | 0 | 1156 | 1196 | 9 | 0 | 2186 | 2292 |
| PSI-21 | ACME CLD Flame | 4883 | 2.0 TB | 4134 | 0 | 2561 | 2213 | 99 | 10 | 4084 | 4813 |
| PSI-22 | ACME E-FIELD Flames | 2590 | 950.2 GB | 2578 | 0 | 1022 | 1221 | 347 | 0 | 2427 | 2500 |
| PSI-23 | ACME s-Flame | 4165 | 782.2 GB | 4165 | 0 | 2148 | 1805 | 212 | 0 | 3956 | 4101 |
| PSI-25 | BASS-II | 695 | 274.6 GB | 695 | 0 | 278 | 0 | 416 | 1 | 685 | 579 |
| PSI-26 | BASS | 212 | 163.6 GB | 212 | 0 | 177 | 0 | 35 | 0 | 0 | 173 |
| PSI-39 | CFI | 2812 | 718.9 GB | 2812 | 0 | 361 | 830 | 1621 | 0 | 2766 | 2549 |
| PSI-47 | DAFT / DAFT-2 | 33 | 41.8 MB | 33 | 0 | 27 | 0 | 6 | 0 | 12 | 9 |
| PSI-68 | FLEX-2 | 10156 | 1.2 TB | 10077 | 0 | 16 | 1000 | 9133 | 7 | 10140 | 9767 |
| PSI-69 | FLEX | 6555 | 923.8 GB | 6453 | 0 | 646 | 4607 | 1302 | 0 | 6522 | 2171 |
| PSI-98 | SAFFIRE-I | 76 | 12.3 GB | 76 | 0 | 75 | 0 | 0 | 1 | 68 | 0 |
| PSI-99 | SAFFIRE-II | 217 | 16.9 GB | 214 | 0 | 216 | 0 | 0 | 1 | 208 | 0 |
| PSI-100 | SAFFIRE-III | 87 | 35.2 GB | 87 | 0 | 85 | 0 | 0 | 2 | 71 | 0 |
| PSI-101 | SAME-R | 344 | 6.3 GB | 344 | 0 | 127 | 172 | 44 | 1 | 290 | 118 |
| PSI-102 | SAME | 101 | 657.6 MB | 101 | 0 | 42 | 59 | 0 | 0 | 59 | 59 |
| PSI-106 | SLICE | 23463 | 72.9 GB | 18707 | 1 | 16791 | 6532 | 43 | 97 | 0 | 22788 |
| PSI-107 | SPICE | 188 | 6.9 GB | 188 | 0 | 183 | 0 | 2 | 3 | 79 | 142 |
| PSI-159 | ACME CFI-G | 3756 | 2.1 TB | 3756 | 0 | 1237 | 1197 | 1322 | 0 | 3588 | 3748 |
| **all 19** | | **66501** | **13.5 TB** | **60800** | **1** | **28794** | **21391** | **16193** | **123** | **40724** | **59599** |

- **A listed file is a listing row.** 5,701 rows repeat an earlier row of the same investigation: the same
  file id, name, size and URL, in the same folder (SLICE 4,756, CLD Flame 749, FLEX 102, FLEX-2 79, E-FIELD
  Flames 12, SAFFIRE-II 3). There are 60,800 distinct files, 13.47 TB. Each repeat keeps its row, with
  `repeat_of_row` pointing at the first; counts per test use distinct files.
- **Versions.** BRE lists 54 names in several versions. Each version is a row; `latest_version` marks the one
  to take.
- **NASA's own totals** agree with the listings for 16 investigations. For CFI, SAFFIRE-II and SLICE the
  total in the investigation record is larger than the sum of the listed files by 2.5 MB, 13.6 MB and 0.4 MB.
- **Classified.** 66500 of 66501 files (99.998 %) are classified by a rule other than the fallback; 66378 (99.82 %) carry a confidence other than "unknown"; 43.3 % of the files rest on the name alone, 32.2 % on a sibling, 24.4 % on a file that was opened. "Unknown" counts files whose kind is a guess from an uninformative name
  (section 7). A kind read from a name is a reading, not a verification.

By kind, all investigations together:

| Kind | Files | Bytes |
|---|---:|---:|
| experiment table | 66 | 3.2 MB |
| script or settings | 7558 | 35.7 MB |
| sensor time series | 4575 | 8.6 GB |
| camera settings | 5571 | 36.7 MB |
| image archive | 17905 | 12.9 TB |
| video | 3922 | 571.5 GB |
| processed or analysed data | 17977 | 9.8 GB |
| calibration | 778 | 11.0 GB |
| science document | 109 | 273.5 MB |
| engineering document | 312 | 1.2 GB |
| report | 36 | 148.8 MB |
| presentation | 120 | 1.4 GB |
| reference data | 59 | 1.1 GB |
| metadata | 6923 | 1.1 GB |
| other | 590 | 243.2 MB |

By what one would download:

| What one downloads | Files | Bytes |
|---|---:|---:|
| images | 18075 | 12.9 TB |
| video | 3955 | 572.9 GB |
| text, tables and documents | 44471 | 23.4 GB |

Images and video are 99.8 % of the bytes. Everything else in the 19 investigations, the sensor series, scripts,
settings, tables and documents of every test, is 23.4 GB (22.2 GB without the repeated rows); the sensor time
series alone are 8.6 GB.

## 2. How a file is classified

Three things are read for each row: the file name, the PSI category, and the folder. The folder is not in the
flat listings (`<id>.listing.jsonl`); it is in the raw listing tree that the same download step saved beside
them (`raw/samples/_meta/files_<id>.json`), and it is joined by file id. Every file id sits in exactly one
folder. If the tree is missing the catalogue still runs, with `folder` empty and the folder rules silent.

**Kind** is one of a closed list:

| Kind | Meaning here |
|---|---|
| experiment table | the PSI experiment table; also a campaign table or a daily test point log that lists tests |
| script or settings | what was commanded: script parameters, setpoint logs, test point settings, the as-run timeline |
| sensor time series | numbers against time from a sensor, or a gas composition reading |
| camera settings | camera configuration, or camera telemetry against time |
| image archive | frames of a camera, zipped or single, raw or converted; proof sheets of stills |
| video | a video file, or videos zipped |
| processed or analysed data | derived by the investigators: diameter histories, summary sheets, maps, plots, workbooks |
| calibration | a calibration run or file (by PSI category or by a calibration folder) |
| science document, engineering document, report, presentation | documents, by PSI category, or by name when filed under "Raw Data" |
| reference data | the PSI category "Reference Data" |
| metadata | information about files and tests that is neither a measurement nor a command: the PSI metadata archive, per-frame indexes, operator comments, test sheets, event log entries |
| other | software logs, error reports, screenshots, representative images, and the one unclassified file |

Two kinds carry outcomes although they are not measurements: the as-run timeline (`_TP.txt` of ACME, with its
"Extinction detected" stamp) and the test sheets of FLEX and CFI (with the investigators' derived values).
The `what` column says so.

**Sure** is one of four values:

| In the tables | In `files.jsonl` | Meaning |
|---|---|---|
| name | stated in the name | the file name, the folder name or the PSI category says it |
| sibling | inferred from a sibling investigation | the same pattern was opened in another investigation of the same hardware, or in sibling files of this one |
| opened | checked by opening one file | one file of this pattern was opened in this investigation, by this catalogue or by an earlier analysis of this lane; for an image archive, the small text file that describes it |
| unknown | unknown | the name does not say and nothing was opened |

**Test id.** Each family of investigations has its own rule for finding the test in a name (section 6). The
name is read first and the folder second. When the two disagree the name is kept and the folder's id is
recorded in `test_id_alternative`, except when the two differ only in the day and the day in the name is
impossible or hardly used elsewhere: then the name is taken as mistyped and the folder's id is used
(`test_id_source` = `folder`). 29 ACME files, 18 test names, were refiled this way.

## 3. Columns of `files.jsonl`

| Column | Content |
|---|---|
| `investigation`, `listing_row` | the investigation and the line of its listing (the key of the row) |
| `file_name`, `category`, `size_bytes`, `url`, `psi_file_id`, `file_version` | as listed (`url` is the listing's `stable_url`) |
| `folder` | the folder under the category, from the raw listing tree |
| `repeat_of_row` | the first listing row with the same file id, when this row repeats it |
| `latest_version` | false when the same file name is listed with a higher version |
| `extension`, `medium` | file extension; `images`, `video` or `text, tables and documents` |
| `kind`, `what`, `instrument`, `confidence`, `rule` | the classification, a sentence on what the file is, the instrument or camera, how sure, and the rule that decided |
| `test_id`, `test_id_scheme`, `test_id_source` | the test, the naming scheme it follows, and whether it came from the name or the folder |
| `test_id_variant` | a suffix of the test name that is not a different test (`_1`, `a`, `b`) |
| `test_ids_other` | other ids in the same name (the FLEX number beside the test point, a second test) |
| `test_id_alternative` | the id the folder (or the name) gave when the two disagree |
| `test_day`, `test_day_scheme` | the test day when the name or folder carries one, and how it is written |
| `group` | the folder level that groups tests (fuel, burner, investigator, material) |

Schemes that are not tests are kept apart in `test_id_scheme`: auxiliary ACME runs, rack calibration points,
chamber gas readings, SAFFIRE flow visualizations.

## 4. Tests and their measurements

Ids are counted from the file names, one download per file name (latest version, repeats dropped).

| Investigation | Id scheme | A test? | Ids | with sensor time series | with image archive | with video | with processed data | with script or settings | with sensor, images and video |
|---|---|---|---:|---:|---:|---:|---:|---:|---:|
| PSI-10 | test point (YYDDD + letter + number) | yes | 323 | 317 | 315 | 317 | 0 | 321 | 313 |
| PSI-20 | auxiliary run (fibre reference, radiometer or flow-controller run) | no | 40 | 0 | 39 | 0 | 0 | 0 | 0 |
| PSI-20 | test point (YYDDD + letter + number) | yes | 208 | 187 | 207 | 189 | 0 | 192 | 181 |
| PSI-21 | auxiliary run (fibre reference, radiometer or flow-controller run) | no | 104 | 51 | 51 | 9 | 0 | 34 | 0 |
| PSI-21 | test point (YYDDD + letter + number) | yes | 359 | 352 | 324 | 350 | 0 | 355 | 317 |
| PSI-22 | test point (YYDDD + letter + number) | yes | 225 | 204 | 219 | 215 | 0 | 208 | 195 |
| PSI-23 | auxiliary run (fibre reference, radiometer or flow-controller run) | no | 44 | 0 | 0 | 0 | 0 | 0 | 0 |
| PSI-23 | test point (YYDDD + letter + number) | yes | 291 | 286 | 289 | 285 | 0 | 288 | 282 |
| PSI-25 | test (letter of the investigator + number) | yes | 115 | 0 | 115 | 112 | 0 | 0 | 0 |
| PSI-39 | rack test point | yes | 133 | 123 | 85 | 86 | 0 | 124 | 76 |
| PSI-39 | FLEX number | yes | 27 | 0 | 0 | 23 | 25 | 0 | 0 |
| PSI-47 | instrument test | yes | 6 | 6 | 0 | 0 | 6 | 0 | 0 |
| PSI-68 | calibration point (CAL, LCTF, BKGD, RDTST) | no | 68 | 2 | 9 | 0 | 0 | 59 | 0 |
| PSI-68 | chamber gas reading (type letter R) | no | 126 | 126 | 0 | 0 | 0 | 124 | 0 |
| PSI-68 | rack test point | yes | 596 | 586 | 535 | 0 | 0 | 591 | 0 |
| PSI-69 | calibration point (CAL, LCTF, BKGD, RDTST) | no | 29 | 0 | 0 | 0 | 1 | 0 | 0 |
| PSI-69 | chamber gas reading (type letter R) | no | 32 | 0 | 0 | 0 | 0 | 0 | 0 |
| PSI-69 | rack test point | yes | 415 | 278 | 411 | 120 | 0 | 280 | 20 |
| PSI-69 | FLEX number | yes | 272 | 0 | 0 | 0 | 269 | 0 | 0 |
| PSI-98 | pre-test flow visualization | no | 2 | 0 | 2 | 0 | 0 | 0 | 0 |
| PSI-98 | sample | yes | 2 | 2 | 2 | 0 | 2 | 0 | 0 |
| PSI-99 | pre-test flow visualization | no | 2 | 0 | 2 | 0 | 0 | 0 | 0 |
| PSI-99 | sample | yes | 9 | 9 | 9 | 0 | 9 | 0 | 0 |
| PSI-100 | pre-test flow visualization | no | 2 | 0 | 2 | 0 | 0 | 0 | 0 |
| PSI-100 | sample | yes | 2 | 2 | 2 | 0 | 2 | 0 | 0 |
| PSI-101 | test (run) number | yes | 67 | 44 | 45 | 0 | 66 | 0 | 0 |
| PSI-102 | test (run) number | yes | 42 | 0 | 0 | 0 | 42 | 0 | 0 |
| PSI-107 | test point number | yes | 53 | 0 | 53 | 0 | 0 | 0 | 0 |
| PSI-159 | test point (YYDDD + letter + number) | yes | 338 | 337 | 331 | 335 | 0 | 337 | 328 |

Notes:

- The ACME counts are test names, not flames: they include test points that did not ignite and a few names
  that only one file carries. PSI-159 has 338 names against 260 tests in the two splits.
- FLEX and CFI have two rows because raw files and processed files use different names for the same test
  (section 6). The rows cannot be added.
- "with processed data" is zero for the ACME investigations: the archive holds no processed series for them,
  only day plots.

Investigations whose files carry a day instead of a test, and days in general:

| Investigation | Days named in files | with sensor time series | with image archive | with video | with processed data |
|---|---:|---:|---:|---:|---:|
| PSI-10 | 29 | 28 | 22 | 22 | 18 |
| PSI-20 | 18 | 17 | 18 | 17 | 5 |
| PSI-21 | 34 | 32 | 29 | 32 | 5 |
| PSI-22 | 25 | 19 | 21 | 20 | 0 |
| PSI-23 | 30 | 27 | 18 | 18 | 0 |
| PSI-25 | 28 | 0 | 28 | 28 | 0 |
| PSI-26 | 37 | 0 | 35 | 12 | 0 |
| PSI-39 | 35 | 32 | 26 | 25 | 0 |
| PSI-47 | 2 | 0 | 0 | 0 | 2 |
| PSI-68 | 97 | 96 | 85 | 0 | 0 |
| PSI-69 | 71 | 46 | 66 | 23 | 0 |
| PSI-98 | 0 | 0 | 0 | 0 | 0 |
| PSI-99 | 0 | 0 | 0 | 0 | 0 |
| PSI-100 | 0 | 0 | 0 | 0 | 0 |
| PSI-101 | 28 | 11 | 0 | 0 | 17 |
| PSI-102 | 17 | 0 | 0 | 0 | 17 |
| PSI-106 | 17 | 0 | 17 | 6 | 13 |
| PSI-107 | 12 | 0 | 12 | 3 | 4 |
| PSI-159 | 25 | 23 | 22 | 23 | 23 |

"Days named in files" counts every day a name or folder states, including setup days and the dates of logs;
it is not the number of test days of the experiment table.

## 5. What one test costs to download

Median over the ids of the scheme, with the 10th and 90th percentiles; bytes of the latest version of each
file that carries the id.

| Investigation | Id scheme | Ids | Text, tables and documents | Images | Video |
|---|---|---:|---|---|---|
| PSI-10 | test point (YYDDD + letter + number) | 323 | 3.7 MB (2.6 MB to 5.6 MB) | 5.1 GB (2.3 GB to 11.8 GB) | 219.5 MB (115.7 MB to 339.7 MB) |
| PSI-20 | test point (YYDDD + letter + number) | 208 | 5.0 MB (1.1 MB to 10.8 MB) | 8.7 GB (3.5 GB to 15.8 GB) | 178.9 MB (34.3 MB to 588.3 MB) |
| PSI-21 | test point (YYDDD + letter + number) | 359 | 2.8 MB (1.9 MB to 4.7 MB) | 5.5 GB (1.7 GB to 8.8 GB) | 90.3 MB (55.3 MB to 142.3 MB) |
| PSI-22 | test point (YYDDD + letter + number) | 225 | 7.5 MB (1.7 MB to 11.1 MB) | 4.1 GB (1.6 GB to 6.5 GB) | 98.1 MB (55.2 MB to 157.4 MB) |
| PSI-23 | test point (YYDDD + letter + number) | 291 | 2.6 MB (2.1 MB to 4.0 MB) | 2.3 GB (1.6 GB to 4.1 GB) | 120.2 MB (64.5 MB to 160.9 MB) |
| PSI-25 | test (letter of the investigator + number) | 115 | none | 1.9 GB (587.3 MB to 3.6 GB) | 246.2 MB (29.6 MB to 942.0 MB) |
| PSI-39 | rack test point | 133 | 1.5 MB (911.0 KB to 1.8 MB) | 5.4 GB (0 B to 8.2 GB) | 796.6 MB (0 B to 2.0 GB) |
| PSI-39 | FLEX number | 27 | 36.0 MB (2.9 MB to 70.0 MB) | none | 503.9 MB (0 B to 616.0 MB) |
| PSI-47 | instrument test | 6 | 287.0 KB (286.1 KB to 287.7 KB) | none | none |
| PSI-68 | rack test point | 596 | 723.2 KB (555.2 KB to 1.3 MB) | 2.0 GB (0 B to 3.5 GB) | none |
| PSI-69 | rack test point | 415 | 607.0 KB (0 B to 1.0 MB) | 2.2 GB (1.3 GB to 2.7 GB) | 0 B (0 B to 48.2 MB) |
| PSI-69 | FLEX number | 272 | 1.4 MB (737.8 KB to 2.1 MB) | none | none |
| PSI-98 | sample | 2 | 30.3 MB (25.9 MB to 30.3 MB) | 9.3 GB (2.4 GB to 9.3 GB) | none |
| PSI-99 | sample | 9 | 18.5 MB (13.3 MB to 24.7 MB) | 1.6 GB (614.4 MB to 2.3 GB) | none |
| PSI-100 | sample | 2 | 23.8 MB (6.5 MB to 23.8 MB) | 21.7 GB (7.0 GB to 21.7 GB) | none |
| PSI-101 | test (run) number | 67 | 7.0 MB (5.8 MB to 10.3 MB) | 12.7 MB (0 B to 53.2 MB) | none |
| PSI-102 | test (run) number | 42 | 4.9 MB (1.2 MB to 10.9 MB) | none | none |
| PSI-107 | test point number | 53 | none | 7.9 MB (2.1 MB to 15.5 MB) | none |
| PSI-159 | test point (YYDDD + letter + number) | 338 | 4.0 MB (3.1 MB to 6.8 MB) | 4.3 GB (2.4 GB to 10.1 GB) | 233.3 MB (122.9 MB to 348.8 MB) |

For an ACME flame the text of one test (sensor series, script, setpoints, timeline, comments, frame indexes) is
3 to 7 MB; its images are 2 to 9 GB and its video 0.1 to 0.2 GB. For a droplet test of FLEX or FLEX-2 the text
is under 1 MB and the images about 2 GB. The text of every test of an ACME investigation fits in 1 to 2 GB.

## 6. The investigations

### PSI-10 ACME/Flame Design

Advanced Combustion via Microgravity Experiments / Flame Design. Hardware: Advanced Combustion via Microgravity Experiments (ACME), Combustion Integrated Rack (CIR), Reconfiguration of Combustion Integrated Rack (CIR). Licence: not stated in the record. 3807 listed files, 2.2 TB.

**Naming.** A test is `YYDDD` + letter + number, for example `19115A1`: two digits of the year, the GMT day of
the year, a letter for the test series of the day, a number for the variant or repeat. The id stands at the
start of the name (`19115A1_data.csv`), after `as run` (`as run19137A1.txt`), after the camera prefix
(`ACME_19115A1.mdipsug`, `INTNS_19115A1.mdipsug`) or at the end of a video name
(`vlc-record-2019-04-25-19h19m36s-udp____19115A1.ts`). Day files start with the day (`19115 test sheets.pdf`).

**Folders.** `Reduced Gravity/<n>_<normal|inverse>_<fuel>_flames_burner_<id>/<day>/` holds the text files, the
videos and the day paperwork; `.../<day>/<test>_Images/` holds the image archives and the comments of one test.
The four fuel folders give 120 inverse ethylene tests, 93 and 92 normal ethylene tests on the two burners and
18 normal propane tests. Three files take their test from the folder: two with a mistyped name
(`190123F2_ACME.zip`, `191196D2_ACME.zip`) and one named for day 19124 that sits in the folder of 19123.

**Gas chromatograph.** `moduleDataACD_<nn>_<date>_<time>.asc` is named by date and time, not by test: it is
tied to a test day, not to a test.

| Pattern | Kind | What it is | Instrument | Files | Bytes | Sure |
|---|---|---|---|---:|---:|---|
| `# Daily Test Point Log.pdf` ; `# daily test point log.pdf` and 1 more spellings | experiment table | test point log of the day | - | 15 | 953.0 KB | name |
| every file of the PSI category "Experimental table" | experiment table | PSI experiment table | - | 2 | 44.2 KB | name |
| `<test>_mfcSetpoints.csv` | script or settings | command log of the three mass-flow-controller setpoints | mass flow controllers | 317 | 2.2 MB | opened |
| `as run<test>.txt` ; `<test>.txt` | script or settings | test point script parameters (key:value) | - | 330 | 706.0 KB | opened |
| `<test>_TP.txt` | script or settings | as-run stage timeline of the test point, with stamps (outcome-bearing) | - | 314 | 348.9 KB | opened |
| `<test>_data.csv` | sensor time series | analog channels at 125 Hz: flow readbacks, pressures, photomultipliers, thermocouples, radiometers | ACME data acquisition | 317 | 1.2 GB | opened |
| `moduleDataACD_#_#_#.asc` | sensor time series | gas chromatograph run: three channels at 100 Hz | gas chromatograph | 66 | 31.0 MB | sibling |
| `as runAcmeCameraConfigure_#-#-#_#_#.#.#.txt` ; `AcmeCameraConfigure_FusedSilica_#.txt` and 1 more spellings | camera settings | camera configuration script of the day | ACME colour camera | 36 | 15.4 KB | name |
| `<test>_ACME_Converted.zip` ; `<test>A_ACME_Converted.zip` and 3 more spellings | image archive | frames of one test converted to a viewable format | ACME colour camera | 314 | 883.4 GB | name |
| `<test>_INTNS_ALL.zip` ; `<test>A_INTNS_ALL.zip` and 1 more spellings | image archive | all frames of one test | ACME intensified camera | 303 | 594.2 GB | name |
| `<test>_ACME.zip` ; `#D#_ACME.zip` and 1 more spellings | image archive | frames of one test as downlinked | ACME colour camera | 315 | 262.3 GB | name |
| `<test>_INTNS.zip` | image archive | frames of one test (selection) | ACME intensified camera | 101 | 219.9 GB | name |
| `<test>A_HiBMS.zip` ; `<test>_HiBMS.zip` | image archive | frames of one test | CIR HiBMs camera | 150 | 144.0 GB | name |
| `vlc-record-#-#-#-#h#m#s-udp____<test>.ts` ; `vlc-record-#-#-#-#h#m#s-udp_<test>.ts` and 3 more spellings | video | recording of the downlinked operations video | operations video downlink | 317 | 75.1 GB | name |
| `# flow traces etc.pdf` ; `# flow traces etc #.pdf` and 1 more spellings | processed or analysed data | plots of the day's flow or velocity traces | - | 18 | 46.6 MB | name |
| `Final ACME Chamber Calculator - #-#-# to #-#-#.xlsx` | processed or analysed data | chamber atmosphere or flow calculator workbook | - | 2 | 1.0 MB | sibling |
| every file of the PSI category "Science Documents" | science document | science document | - | 3 | 2.6 MB | name |
| every file of the PSI category "Engineering Documents" | engineering document | engineering document | - | 1 | 7.5 MB | name |
| every file of the PSI category "Reports" | report | report | - | 3 | 4.5 MB | name |
| `INTNS_<test>.mdipsug` ; `INTNS_<test>.mdipsug.ERRORS` | metadata | per-frame index of the archive (time stamp, exposure, gain) | ACME intensified camera | 241 | 76.8 MB | sibling |
| `# test sheets.pdf` | metadata | test sheets of the day (scan) | - | 18 | 35.6 MB | name |
| `ACME_<test>.mdipsug` | metadata | per-frame index of the archive (time stamp, exposure, gain) | ACME colour camera | 250 | 32.7 MB | sibling |
| `# test parameter notes.pdf` ; `# test parameters notes.pdf` | metadata | test parameter notes of the day (scan) | - | 18 | 27.0 MB | name |
| `<test>_Comments.txt` | metadata | operator notes on the image downlink of the test | - | 313 | 100.8 KB | opened |
| every file of the PSI category "Investigation Metadata Files" | metadata | PSI investigation metadata archive | - | 1 | 17.0 KB | name |
| `#-#-#_#-#ACME.log` ; `#-#-#_#-#_ACME.log` | other | message log of the avionics software | - | 25 | 11.5 MB | name |
| `<test> bracking exposure histogram screenshot.PNG` | other | screenshot | - | 1 | 1.5 MB | name |
| every file of the PSI category "Representative Image(s)" | other | representative image of the investigation | - | 5 | 871.7 KB | name |
| `GC_Coverter_#Dec#_#_#_#.log` ; `GC_Coverter_#Jul#_#.log` and 5 more spellings | other | gas chromatograph converter log or screenshot | gas chromatograph | 11 | 497.4 KB | opened |

### PSI-20 ACME BRE

Advanced Combustion via Microgravity Experiments - Burning Rate Emulator. Hardware: Combustion Integrated Rack (CIR), ACME. Licence: CC0-1.0. 2361 listed files, 2.0 TB.

**Naming.** The same test names as the other ACME investigations (`19039A1`). The investigation's readme says
"where the different numbers can be for repeats but are typically variants". Auxiliary runs are named
`<day>Fiber<n>` (thin-filament reference images) and `<day>_IgniterCheck`.

**Folders.** `Reduced Gravity/<burner>_<fuel>/<test> Raw Data` and `<test> Video and Image Data`, for four
groups: 25-mm burner with 100 % ethylene (94 tests), 50 % ethylene (45), 100 % methane (39), and 50-mm burner
with 100 % ethylene (30). Some tests sit directly under `Reduced Gravity/<test>`.

**Versions.** 54 file names are listed in several versions (up to eight), 69 superseded rows in all. Each
version is its own row; `latest_version` marks the one to download. `_BRE.txt` is the script of the test point
(checked), an older spelling of `as run<test>.txt`.

| Pattern | Kind | What it is | Instrument | Files | Bytes | Sure |
|---|---|---|---|---:|---:|---|
| `# daily test point log.pdf` | experiment table | test point log of the day | - | 5 | 282.3 KB | name |
| every file of the PSI category "Experimental table" | experiment table | PSI experiment table | - | 1 | 1.1 KB | name |
| `<test>_mfcSetpoints.csv` ; `<test>_#_mfcSetpoints.csv` | script or settings | command log of the three mass-flow-controller setpoints | mass flow controllers | 191 | 985.0 KB | sibling |
| `as run<test>.txt` ; `<test>.txt` and 1 more spellings | script or settings | test point script parameters (key:value) | - | 184 | 343.6 KB | sibling |
| `<test>_TP.txt` ; `<test>_#_TP.txt` | script or settings | as-run stage timeline of the test point, with stamps (outcome-bearing) | - | 211 | 246.7 KB | sibling |
| `<test>_BRE.txt` | script or settings | test point script parameters (key:value) | - | 9 | 16.0 KB | opened |
| `<test>_data.csv` ; `<test>_#_data.csv` and 2 more spellings | sensor time series | analog channels at 125 Hz: flow readbacks, pressures, photomultipliers, thermocouples, radiometers | ACME data acquisition | 191 | 1.1 GB | sibling |
| `#_GC.zip` | sensor time series | gas chromatograph files of the day, zipped | gas chromatograph | 9 | 9.1 MB | sibling |
| `AcmeCameraConfigure_FusedSilica_#.txt` ; `AcmeCameraConfigure_#-#-#_#_#.#.#.txt` | camera settings | camera configuration script of the day | ACME colour camera | 50 | 21.3 KB | name |
| `<test>_ACME_Converted.zip` ; `<test>_#_ACME_Converted.zip` | image archive | frames of one test converted to a viewable format | ACME colour camera | 203 | 976.4 GB | name |
| `<test>_INTNS_ALL.zip` ; `<test>_#_INTNS_ALL.zip` and 3 more spellings | image archive | all frames of one test | ACME intensified camera | 171 | 398.2 GB | name |
| `<test>_INTNS.zip` ; `<test>_#_INTNS.zip` and 2 more spellings | image archive | frames of one test (selection) | ACME intensified camera | 150 | 288.5 GB | name |
| `<test>_ACME.zip` ; `<test>_#_ACME.zip` and 3 more spellings | image archive | frames of one test as downlinked | ACME colour camera | 210 | 264.6 GB | name |
| `<test>A_HiBMS.zip` | image archive | frames of one test | CIR HiBMs camera | 29 | 51.0 GB | name |
| `<test>_ACME.zip` ; `<test>.zip` | image archive | thin-filament reference images | ACME colour camera | 39 | 669.5 MB | name |
| `#_IgniterPosition#_Originals.zip` ; `#-mm burner, #% ethylene_#_IgniterPosition#_Originals.zip` and 2 more spellings | image archive | igniter position check frames | - | 7 | 35.3 MB | name |
| `vlc-record-#-#-#-#h#m#s-udp____<test>.zip` ; `vic-record-#-#-#-#h#m#s-udp____<test>.zip` and 6 more spellings | video | the original transport stream of the recording, zipped | operations video downlink | 196 | 38.8 GB | sibling |
| `vlc-record-#-#-#-#h#m#s-udp____<test>.mp#` ; `vic-record-#-#-#-#h#m#s-udp____<test>.mp#` and 6 more spellings | video | recording of the downlinked operations video | operations video downlink | 196 | 15.8 GB | name |
| `# flow traces etc.pdf` | processed or analysed data | plots of the day's flow or velocity traces | - | 5 | 6.1 MB | name |
| `Final ACME Chamber Calculator - #-#-# to #-#-#.xlsx` | processed or analysed data | chamber atmosphere or flow calculator workbook | - | 5 | 2.5 MB | sibling |
| every file of the PSI category "Science Documents" | science document | science document | - | 22 | 72.1 MB | name |
| every file of the PSI category "Engineering Documents" | engineering document | engineering document | - | 25 | 161.4 MB | name |
| every file of the PSI category "Reports" | report | report | - | 2 | 18.4 MB | name |
| every file of the PSI category "Presentations" | presentation | presentation | - | 13 | 184.3 MB | name |
| `# test sheets.pdf` | metadata | test sheets of the day (scan) | - | 5 | 19.6 MB | name |
| `# test parameter notes.pdf` | metadata | test parameter notes of the day (scan) | - | 5 | 17.5 MB | name |
| `INTNS_<test>.mdipsug` | metadata | per-frame index of the archive (time stamp, exposure, gain) | ACME intensified camera | 4 | 1.8 MB | sibling |
| `ACME_<test>.mdipsug` | metadata | per-frame index of the archive (time stamp, exposure, gain) | ACME colour camera | 6 | 354.3 KB | sibling |
| `<test>_Comments.txt` ; `<test>_#_Comments.txt` and 3 more spellings | metadata | operator notes on the image downlink of the test | - | 198 | 56.4 KB | sibling |
| every file of the PSI category "Investigation Metadata Files" | metadata | PSI investigation metadata archive | - | 1 | 15.1 KB | name |
| `ACME_<test>.next` | metadata | frame counter of the camera unit | - | 1 | 3 B | sibling |
| `#_Logs.zip` ; `<test>_Logs.zip` | other | logs, zipped | - | 17 | 5.8 MB | name |

### PSI-21 ACME CLD Flame

Advanced Combustion via Microgravity Experiments / Coflow Laminar Diffusion Flame. Hardware: Combustion Integrated Rack (CIR), ACME. Licence: CC0-1.0. 4883 listed files, 2.0 TB (4134 distinct).

**Naming.** The same test names (`17319A3`). Everything, documents included, is filed under the PSI category
"Raw Data", in a folder "Legacy", and the file names carry no `PSI-21_Raw Data_` prefix. 749 listing rows repeat
an earlier row (same file id, same folder).

**Folders.** `Legacy/<fuel group>/<test> Raw Data` and `<test> Video and Image Data`, for five groups: methane
dilute (132 tests), methane rich (32), ethylene 100 % (88), 50 % (30) and 30 % (73). 80 files take their test
from the folder (the day logs and camera configuration files copied into each test folder do not).

**Auxiliary runs.** `<day>RAD<n>` and `<day>MFCA<n>_<k>` have data and setpoint files like a test but no flame
name. They are read as radiometer and flow-controller runs from the tokens alone; no file of them was opened.

**Documents.** 64 documents sit in "Raw Data": publications in folders named by their citation, the final
report, requirement documents and drawing packages. They are classified by folder and by name, or by the
category the same file has in a sibling ACME investigation.

| Pattern | Kind | What it is | Instrument | Files | Bytes | Sure |
|---|---|---|---|---:|---:|---|
| every file of the PSI category "Experimental table" | experiment table | PSI experiment table | - | 1 | 2.5 KB | name |
| `<test>_mfcSetpoints.csv` ; `<test>_#_mfcSetpoints.csv` | script or settings | command log of the three mass-flow-controller setpoints | mass flow controllers | 388 | 2.8 MB | sibling |
| `<test>.txt` ; `as run<test>.txt` and 1 more spellings | script or settings | test point script parameters (key:value) | - | 352 | 576.0 KB | sibling |
| `<test>_TP.txt` ; `<test>-#_TP.txt` | script or settings | as-run stage timeline of the test point, with stamps (outcome-bearing) | - | 299 | 333.9 KB | sibling |
| `<test>_data.csv` ; `<test>_#_data.csv` | sensor time series | analog channels at 125 Hz: flow readbacks, pressures, photomultipliers, thermocouples, radiometers | ACME data acquisition | 404 | 1.4 GB | sibling |
| `<test>_ACME_TP_Analysis_FCU_PTandMFC_Readings.txt` ; `<test>_ACME_TP_Analysis_FCU_PTandMFC_Readings.txt.log` and 1 more spellings | sensor time series | rack fluids-control-unit pressure transducers and flow controllers against time | CIR fluids control unit | 17 | 1.8 MB | opened |
| `<test>_ACME_CAM#_Exposures.txt` ; `<test>_ACME_CAM#_Exp.xlsx` and 1 more spellings | camera settings | camera telemetry against time (exposure, gain, frame rate) | ACME colour camera | 82 | 10.0 MB | opened |
| `AcmeCameraConfigure_FusedSilica_#.txt` ; `as runAcmeCameraConfigure_#-#-#_#_#.#.#.txt` and 1 more spellings | camera settings | camera configuration script of the day | ACME colour camera | 292 | 124.9 KB | name |
| `<test>_ACME_Converted.zip` ; `#l#_ACME_Converted.zip` and 3 more spellings | image archive | frames of one test converted to a viewable format | ACME colour camera | 323 | 1.1 TB | name |
| `<test>_ACME.zip` ; `<test>_IPSUG_ACME.zip` and 1 more spellings | image archive | frames of one test as downlinked | ACME colour camera | 325 | 314.0 GB | name |
| `<test>_Intensifier_ALL.zip` ; `<test>_INTNS_ALL.zip` and 2 more spellings | image archive | all frames of one test | ACME intensified camera | 185 | 192.9 GB | name |
| `<test>_Intensifier.zip` ; `<test>_INTNS.zip` and 6 more spellings | image archive | frames of one test (selection) | ACME intensified camera | 243 | 167.3 GB | name |
| `<test>A_HiBMS.zip` ; `<test>_HiBMS.zip` and 3 more spellings | image archive | frames of one test | CIR HiBMs camera | 218 | 121.1 GB | name |
| `<test>_Intensifier_B#_Ignition.zip` ; `<test>_Intensifier_Ignition.zip` and 3 more spellings | image archive | frames around ignition | ACME intensified camera | 25 | 31.4 GB | name |
| `<test>.zip` ; `<test>_ACME.zip` and 1 more spellings | image archive | thin-filament reference images | ACME colour camera | 55 | 1.0 GB | name |
| `<test>.zip` | image archive | archive named only by the test | - | 1 | 764.7 MB | unknown |
| `<test>_TP_Image_Selections_Originals_#.zip` | image archive | selected frames of one test | - | 3 | 4.5 MB | name |
| `vlc-record-#-#-#-#h#m#s-udp____<test>.zip` ; `vlc-record-#-#-#-#h#m#s-udp____#_IgniterCheck.zip` and 4 more spellings | video | the original transport stream of the recording, zipped | operations video downlink | 396 | 20.3 GB | sibling |
| `vlc-record-#-#-#-#h#m#s-udp____<test>.mp#` ; `vlc-record-#-#-#-#h#m#s-udp____#_IgniterCheck.mp#` and 4 more spellings | video | recording of the downlinked operations video | operations video downlink | 396 | 16.5 GB | name |
| `# velocity traces.pdf` ; `#_MFCA_all.pdf` | processed or analysed data | plots of the day's flow or velocity traces | - | 52 | 44.2 MB | name |
| `Final ACME Chamber Calculator - #-#-# to #-#-#.xlsx` | processed or analysed data | chamber atmosphere or flow calculator workbook | - | 2 | 1.1 MB | sibling |
| `# Giassi & Long Demosaicing algorithms for the improvement of spatial resolution and accuracy ....pdf` ; `# Giassi et al ACME Coflow Laminar Diffusion Flame Burner - Preliminary Characterization and Results.pdf` and 9 more spellings | science document | publication (folder is a citation) | - | 11 | 89.4 MB | name |
| `PS-# ACME info sheet.pdf` ; `ACME fact sheet #.pdf` and 10 more spellings | science document | science document | - | 15 | 34.9 MB | name |
| `ACME_drawings_QDs_&_hoses.zip` ; `ACME_drawings_analog_camera.zip` and 17 more spellings | engineering document | engineering drawing or drawing package | - | 19 | 149.7 MB | name |
| `ACME CLD burner hot wire anemometry measurements - REVISED.pdf` ; `ACME Imaging.zip` and 4 more spellings | engineering document | engineering document | - | 6 | 25.0 MB | name |
| `CLD_Flame_final_report_#.pdf` | report | report (folder name says so) | - | 1 | 5.1 MB | name |
| `ACME_ASGSR_#.pdf` ; `ACME_CSSCI_#.pdf` and 8 more spellings | presentation | presentation or outreach video | - | 12 | 193.8 MB | name |
| `INTNS_<test>.mdipsug` | metadata | per-frame index of the archive (time stamp, exposure, gain) | ACME intensified camera | 26 | 9.6 MB | sibling |
| `# test sheets.pdf` | metadata | test sheets of the day (scan) | - | 10 | 9.6 MB | name |
| `ACME_<test>.mdipsug` | metadata | per-frame index of the archive (time stamp, exposure, gain) | ACME colour camera | 53 | 3.9 MB | sibling |
| `<test>_Comments.txt` ; `<test>_Comment.txt` and 2 more spellings | metadata | operator notes on the image downlink of the test | - | 286 | 417.3 KB | sibling |
| every file of the PSI category "Investigation Metadata Files" | metadata | PSI investigation metadata archive | - | 1 | 15.5 KB | name |
| `#.txt` | metadata | day note | - | 1 | 1.1 KB | name |
| `ACME_<test>.next` | metadata | frame counter of the camera unit | - | 7 | 21 B | sibling |
| `#-#-#_#-#ACME.log` ; `#-#-#_#-#_ACME.log` and 1 more spellings | other | message log of the avionics software | - | 349 | 115.1 MB | name |
| `#-#-#_#-#_ACMECommands.log` | other | command log of the avionics software | - | 18 | 34.0 MB | name |
| `#.pdf` | other | day document named only by the day | - | 9 | 17.3 MB | unknown |

### PSI-22 ACME E-FIELD Flames

Advanced Combustion via Microgravity Experiments Electric-Field Effects on Laminar Diffusion Flames. Hardware: Combustion Integrated Rack (CIR), ACME. Licence: CC0-1.0. 2590 listed files, 950.2 GB (2578 distinct).

**Naming.** The same test names (`18073A3`). The script of a test point is `<test>_VoltageSweep.txt` (checked:
the same key:value form as `as run<test>.txt`). The per-frame index is `<date>_<time>_ACME_<test>.metadata` in the
first campaign and `ACME_<test>.mdipsug` later.

**Folders.** `Reduced Gravity/<burner> - <polarity> Power Supply/<test>/Raw Data` and `.../Video and Image Data`:
co-flow burner negative (48 tests) and positive (75), gas-jet burner negative (57) and positive (45).

| Pattern | Kind | What it is | Instrument | Files | Bytes | Sure |
|---|---|---|---|---:|---:|---|
| every file of the PSI category "Experimental table" | experiment table | PSI experiment table | - | 1 | 2.5 KB | name |
| `<test>_mfcSetpoints.csv` | script or settings | command log of the three mass-flow-controller setpoints | mass flow controllers | 205 | 392.7 KB | sibling |
| `<test>_VoltageSweep.txt` | script or settings | test point script parameters (key:value) | - | 202 | 347.3 KB | opened |
| `<test>_TP.txt` | script or settings | as-run stage timeline of the test point, with stamps (outcome-bearing) | - | 202 | 235.7 KB | sibling |
| `<test>A.txt` | script or settings | test point script parameters (key:value) | - | 2 | 2.4 KB | sibling |
| `<test>_data.csv` | sensor time series | analog channels at 125 Hz: flow readbacks, pressures, photomultipliers, thermocouples, radiometers | ACME data acquisition | 204 | 1.5 GB | sibling |
| `<test>_data.zip` | sensor time series | the 125 Hz data file, zipped | ACME data acquisition | 1 | 1.7 MB | sibling |
| `<test>_ACME_CAM#_Exposures.txt` ; `<test>_ACME_CM#_Exposures.txt` | camera settings | camera telemetry against time (exposure, gain, frame rate) | ACME colour camera | 23 | 7.9 MB | sibling |
| `AcmeCameraConfigure_FusedSilica_#.txt` | camera settings | camera configuration script of the day | ACME colour camera | 21 | 8.9 KB | name |
| `<test>_Intensifier.zip` ; `<test>_Intenifier.zip` and 2 more spellings | image archive | frames of one test (selection) | ACME intensified camera | 215 | 311.1 GB | name |
| `<test>_ACME.zip` ; `<test>_ACME#.zip` and 2 more spellings | image archive | frames of one test as downlinked | ACME colour camera | 225 | 238.7 GB | name |
| `<test>A_HiBMS.zip` ; `<test>_HiBMS.zip` and 1 more spellings | image archive | frames of one test | CIR HiBMs camera | 142 | 210.8 GB | name |
| `<test>_Intensifier_ALL.zip` ; `<test>_INTNS_ALL.zip` and 4 more spellings | image archive | all frames of one test | ACME intensified camera | 90 | 162.3 GB | name |
| `vlc-record-#-#-#-#h#m#s-udp_<test>.zip` ; `vlc-record-#-#-#-#h#m#s-udp_<test>_ts.zip` and 5 more spellings | video | the original transport stream of the recording, zipped | operations video downlink | 214 | 12.9 GB | sibling |
| `vlc-record-#-#-#-#h#m#s-udp_<test>.mp#` ; `vlc-record-#-#-#-#h#m#s-udp____<test>.mp#` and 6 more spellings | video | recording of the downlinked operations video | operations video downlink | 216 | 11.9 GB | name |
| `ACMEFlowAndPressuresRawDataConversion.xlsx` ; `ACMEFlowCalculator.xlsx` | processed or analysed data | chamber atmosphere or flow calculator workbook | - | 2 | 4.5 MB | sibling |
| every file of the PSI category "Science Documents" | science document | science document | - | 13 | 26.4 MB | name |
| every file of the PSI category "Engineering Documents" | engineering document | engineering document | - | 26 | 183.9 MB | name |
| every file of the PSI category "Reports" | report | report | - | 9 | 18.7 MB | name |
| every file of the PSI category "Presentations" | presentation | presentation | - | 32 | 808.3 MB | name |
| every file of the PSI category "Reference Data" | reference data | reference data | - | 2 | 3.9 MB | name |
| `INTNS_<test>.mdipsug` ; `#_#.#_INTNS_<test>.metadata` | metadata | per-frame index of the archive (time stamp, exposure, gain) | ACME intensified camera | 138 | 54.9 MB | sibling |
| `ACME_<test>.mdipsug` ; `#_#.#_ACME_<test>.metadata` | metadata | per-frame index of the archive (time stamp, exposure, gain) | ACME colour camera | 145 | 19.6 MB | opened |
| `<test>_Comments.txt` ; `<test>A_Comments.txt` | metadata | operator notes on the image downlink of the test | - | 200 | 196.7 KB | sibling |
| every file of the PSI category "Investigation Metadata Files" | metadata | PSI investigation metadata archive | - | 1 | 15.2 KB | name |
| `<test>_data.csv.ERRORS` | other | error report beside a data file | - | 2 | 11.2 MB | name |
| `#-#-#_#-#ACME.log` | other | message log of the avionics software | - | 22 | 7.4 MB | name |
| `#-#-#_#-#IPSUG.log` ; `#-#-#_#-#_IPSUG.log` | other | message log of the camera unit | - | 30 | 6.2 MB | sibling |
| every file of the PSI category "Representative Image(s)" | other | representative image of the investigation | - | 4 | 351.1 KB | name |
| `<test>_Event_Log_comments.JPG` | other | screenshot | - | 1 | 58.5 KB | name |

### PSI-23 ACME s-Flame

Advanced Combustion via Microgravity Experiments-Structure and Response of Spherical Diffusion Flames. Hardware: Combustion Integrated Rack (CIR), ACME. Licence: CC0-1.0. 4165 listed files, 782.2 GB.

**Naming.** The same test names (`19254A1`). The UV camera of the rack is added: `<test>B_LLL-UV.zip`. Thin-filament
reference archives (`<day>Fiber<n>_ACME.zip`) are filed under the PSI category "Reference Data".

**Folders.** `Reduced Gravity/<test>/`. Ten tests named 20013xx sit in folders named 20006xx with all their
files; the name is kept and the folder is recorded as the alternative. Eleven files (three test names) differ
from their folder only in the day and use a day that hardly occurs elsewhere (`19245K1` in `19254K1`): those
are filed under the folder's id.

**Gas chromatograph.** `moduleDataACD_*.asc` files (checked: a chromatogram header, three channels at 100 Hz),
some prefixed with a test name.

| Pattern | Kind | What it is | Instrument | Files | Bytes | Sure |
|---|---|---|---|---:|---:|---|
| `ACME Daily Test Point Log #.xlsx` | experiment table | test point log of the day | - | 2 | 252.4 KB | name |
| every file of the PSI category "Experimental table" | experiment table | PSI experiment table | - | 1 | 2.5 KB | name |
| `<test>_mfcSetpoints.csv` ; `<test>_#_mfcSetpoints.csv` | script or settings | command log of the three mass-flow-controller setpoints | mass flow controllers | 288 | 2.2 MB | sibling |
| `as run<test>.txt` ; `<test>.txt` and 1 more spellings | script or settings | test point script parameters (key:value) | - | 394 | 746.6 KB | sibling |
| `<test>_TP.txt` | script or settings | as-run stage timeline of the test point, with stamps (outcome-bearing) | - | 266 | 310.8 KB | sibling |
| `<test>_data.csv` ; `<test>_#_data.csv` | sensor time series | analog channels at 125 Hz: flow readbacks, pressures, photomultipliers, thermocouples, radiometers | ACME data acquisition | 288 | 771.3 MB | sibling |
| `<test>_moduleDataACD_#_#_#.asc` ; `moduleDataACD_#_#_#.asc` | sensor time series | gas chromatograph run: three channels at 100 Hz | gas chromatograph | 170 | 79.1 MB | opened |
| `AcmeCameraConfigure_FusedSilica_#.txt` ; `as runAcmeCameraConfigure_#-#-#_#_#.#.#.txt` | camera settings | camera configuration script of the day | ACME colour camera | 27 | 11.5 KB | name |
| `<test>_INTNS_ALL.zip` | image archive | all frames of one test | ACME intensified camera | 286 | 204.5 GB | name |
| `<test>_ACME_Converted.zip` ; `#A#_ACME_Converted.zip` | image archive | frames of one test converted to a viewable format | ACME colour camera | 281 | 184.0 GB | name |
| `<test>A_HiBMS.zip` ; `<test>A_HIBMS.zip` and 1 more spellings | image archive | frames of one test | CIR HiBMs camera | 284 | 130.2 GB | name |
| `<test>_INTNS.zip` ; `#A#_INTNS.zip` and 4 more spellings | image archive | frames of one test (selection) | ACME intensified camera | 265 | 124.3 GB | name |
| `<test>_ACME.zip` ; `<test>_ACME_.zip` and 1 more spellings | image archive | frames of one test as downlinked | ACME colour camera | 287 | 69.1 GB | name |
| `<test>B_LLL-UV.zip` | image archive | frames of one test | CIR LLL-UV camera | 286 | 34.6 GB | name |
| `vlc-record-#-#-#-#h#m#s-udp_<test>.zip` ; `vlc-record-#-#-#-#h#m#s-udp____<test>.zip` and 5 more spellings | video | the original transport stream of the recording, zipped | operations video downlink | 285 | 23.9 GB | sibling |
| `vlc-record-#-#-#-#h#m#s-udp_<test>.mp#` ; `vlc-record-#-#-#-#h#m#s-udp____<test>.mp#` and 7 more spellings | video | recording of the downlinked operations video | operations video downlink | 286 | 9.3 GB | name |
| every file of the PSI category "Science Documents" | science document | science document | - | 7 | 9.8 MB | name |
| every file of the PSI category "Engineering Documents" | engineering document | engineering document | - | 29 | 190.4 MB | name |
| every file of the PSI category "Reports" | report | report | - | 2 | 3.5 MB | name |
| every file of the PSI category "Presentations" | presentation | presentation | - | 17 | 50.7 MB | name |
| `<test>_ACME.zip` ; `<test>_Images.zip` and 1 more spellings | reference data | reference: thin-filament reference images | ACME colour camera | 43 | 813.9 MB | name |
| every file of the PSI category "Reference Data" | reference data | reference data | - | 6 | 286.8 MB | name |
| `<test>_ACME.zip` | reference data | reference: frames of one test as downlinked | ACME colour camera | 1 | 19.7 MB | name |
| `INTNS_<test>.mdipsug` | metadata | per-frame index of the archive (time stamp, exposure, gain) | ACME intensified camera | 8 | 1.2 MB | sibling |
| `ACME_<test>.mdipsug` | metadata | per-frame index of the archive (time stamp, exposure, gain) | ACME colour camera | 8 | 220.9 KB | opened |
| `<test>_Comments.txt` | metadata | operator notes on the image downlink of the test | - | 276 | 59.6 KB | sibling |
| every file of the PSI category "Investigation Metadata Files" | metadata | PSI investigation metadata archive | - | 1 | 20.1 KB | name |
| `ACME_<test>.next` ; `INTNS_<test>.next` | metadata | frame counter of the camera unit | - | 2 | 9 B | opened |
| `<test>_#-#-#_#-#_ACME.log` ; `#-#-#_#-#_ACME.log` and 1 more spellings | other | message log of the avionics software | - | 36 | 17.8 MB | name |
| `#-#-#_#-#_IPSUG.log` ; `<test>_#-#-#_#-#_IPSUG.log` | other | message log of the camera unit | - | 32 | 3.5 MB | opened |
| `vlc-record-#-#-#-#h#m#s-udp_<test>.mp#.sfk` | other | editor sidecar of a video | - | 1 | 242.6 KB | name |

### PSI-25 BASS-II

Burning and Suppression of Solids - II. Hardware: Microgravity Science Glovebox (MSG). Licence: CC0-1.0. 695 listed files, 274.6 GB.

**Naming.** A test is the letter of the investigator and a number, `T10`: T (T'ien), B (Bhattacharjee), M
(Miller), O (Olson), P (Fernandez-Pello), F (Ferkul). It is the last folder
(`Reduced Gravity/<investigator>/<test>/`) and it appears in names as `TEST_T5`, `TESTB16` or `TPT1`, with the
sample number (`SAMPLE_131`, `S128`) and the GMT day. A burn split in two has a letter after the number
(`TEST_O3a`); the catalogue keeps `O3` as the test and the letter as the variant, so the split ids `T28a` and
`T28b` are test `T28`, variants a and b.

**Stills carry no test in the name.** The still sequences are `iss038e048838.zip` (the ISS photo number of the
first frame); 112 files take their test from the folder alone. In five files the name and the folder disagree
(`TESTB15` in folder `B17`): the name is kept.

**Check against known ids.** All 21 train and validation tests of the split are found, each with an image
archive and a video. There is no sensor time series in this investigation: the fan, anemometer and radiometer
readings are on the video overlay (dossier section 2.5).

| Pattern | Kind | What it is | Instrument | Files | Bytes | Sure |
|---|---|---|---|---:|---:|---|
| every file of the PSI category "Experimental table" | experiment table | PSI experiment table | - | 1 | 16.6 KB | name |
| `iss#e#.zip` | image archive | still sequence of the burn, named by ISS photo number | still camera | 91 | 135.2 GB | opened |
| `GMT#_#_BASS_S#M_TP<test>.zip` ; `GMT#_#_BASS_S#M_TP<test>-#.zip` and 12 more spellings | image archive | still sequence of the burn | still camera | 58 | 95.8 GB | opened |
| `GMT#_#_BASS_S#M_TP<test>_ProofSheet.pdf` ; `GMT#_#_BASS_S#M_TP<test>a_ProofSheet.pdf` and 11 more spellings | image archive | proof sheet of the still sequence (PDF) | still camera | 125 | 123.1 MB | opened |
| `BASS_GMT#_SAMPLE_#_TEST_<test> original videos.zip` ; `BASS_GMT#_SAMPLE_#_TEST<test> original videos.zip` and 16 more spellings | video | original videos of the burn, zipped | video cameras | 114 | 25.5 GB | name |
| `BASS_GMT#_SAMPLE_#_TEST_<test>_#ND_VIEW.mp#` ; `BASS_GMT#_SAMPLE_#_TEST<test>_#ND_VIEW.mp#` and 11 more spellings | video | video of the burn, second view | video camera, second view | 102 | 9.0 GB | name |
| `BASS_GMT#_SAMPLE_#_TEST_<test>.mp#` ; `BASS_GMT#_SAMPLE_#_TEST<test>.mp#` and 26 more spellings | video | video of the burn | video camera | 142 | 8.2 GB | opened |
| `BASS_GMT#_SAMPLE_#_TEST_<test>_Still_Sequence.mp#` ; `BASS_GMT#_SAMPLE_#_TEST<test>_Still_Sequence.mp#` and 6 more spellings | video | movie made from the still sequence | still camera | 52 | 450.2 MB | name |
| `BASS_GMT#_SAMPLE_#_TEST_<test>.zip` | video | archive named like the video of the burn; the name does not say what it holds | - | 1 | 254.0 MB | unknown |
| every file of the PSI category "Science Documents" | science document | science document | - | 3 | 1.2 MB | name |
| every file of the PSI category "Engineering Documents" | engineering document | engineering document | - | 2 | 2.2 MB | name |
| every file of the PSI category "Reports" | report | report | - | 1 | 49.0 MB | name |
| every file of the PSI category "Reference Data" | reference data | reference data | - | 1 | 24.0 KB | name |
| every file of the PSI category "Investigation Metadata Files" | metadata | PSI investigation metadata archive | - | 1 | 13.4 KB | name |
| every file of the PSI category "Representative Image(s)" | other | representative image of the investigation | - | 1 | 37.5 KB | name |

### PSI-26 BASS

Burning and Suppression of Solids. Hardware: Microgravity Science Glovebox. Licence: CC0-1.0. 212 listed files, 163.6 GB.

**Naming.** No test id in any name. The unit is the day: folders `Reduced Gravity/2013_04_05`, stills of the day
in `2012_05_03_orig.zip`, and one video per burn named by date, time and ISS video number
(`20130405 18.17.49 iss035m020941810.mp4`). A burn is found by its time stamp. `_xmp.zip` and `_txt.zip` hold
the sidecars of the stills.

| Pattern | Kind | What it is | Instrument | Files | Bytes | Sure |
|---|---|---|---|---:|---:|---|
| every file of the PSI category "Experimental table" | experiment table | PSI experiment table | - | 1 | 4.7 KB | name |
| `#_#_#_orig.zip` ; `#_#_#.zip` | image archive | original stills of the day | still camera | 35 | 140.7 GB | opened |
| `# #.#.# iss#m#.mp#` ; `# #.# # iss#m#.mp#` and 2 more spellings | video | video of one burn, named by date, time and ISS video number | video camera | 114 | 22.8 GB | name |
| every file of the PSI category "Science Documents" | science document | science document | - | 5 | 9.9 MB | name |
| every file of the PSI category "Engineering Documents" | engineering document | engineering document | - | 24 | 13.8 MB | name |
| every file of the PSI category "Reports" | report | report | - | 2 | 1.1 MB | name |
| every file of the PSI category "Presentations" | presentation | presentation | - | 6 | 15.0 MB | name |
| `#_#_#_xmp.zip` | metadata | XMP sidecars of the day's stills, zipped | - | 14 | 8.6 MB | name |
| `#_#_#_txt.zip` | metadata | text sidecars of the day's stills, zipped | - | 10 | 2.2 MB | name |
| every file of the PSI category "Investigation Metadata Files" | metadata | PSI investigation metadata archive | - | 1 | 11.6 KB | name |

### PSI-39 CFI

Cool Flames Investigation. Hardware: Multi-User Droplet Combustion Apparatus (MDCA), Combustion Integrated Rack (CIR). Licence: CC0-1.0. 2812 listed files, 718.9 GB.

**Naming.** Two names for one test. Raw files use the rack test point, `F068A03`: a letter, the GMT day of the
year, a type letter and a two-digit sequence. Processed files use the FLEX number, `FLEX-1206`. 43 pairs are
linked by a file or folder that carries both (`FLEX-1251_2017-05-05-16h23m23s_F125A04.mp4`, folder
`FLEX-1290 - F177D01`); the other tests need the experiment table to be linked.

**Camera letter.** A letter after the test point names the camera unit: `F068A03A.txt` is the frame index of
unit A, `F125A04A.zip` its frames. A is the HiBMs camera (IPSU1), C the LCTF or HFR camera (IPSU3), D the colour
camera (IPSU4), E the intensified camera (`E_IPSUG.zip`). The letters are tied to the units by the header of
the camera settings files, opened in FLEX-2 and here.

**Folders.** `Reduced Gravity/CFI - <fuel>/<FLEX number> - <test point>/`, four fuel groups (dodecane, two
dodecane and iso-dodecane mixtures, farnesane). One folder name has a broken character in NASA's listing; it is
kept as listed.

| Pattern | Kind | What it is | Instrument | Files | Bytes | Sure |
|---|---|---|---|---:|---:|---|
| `CFI_DataSummary_Spreadsheet_Jan#.xlsx` | experiment table | campaign table of every test, wider than the PSI experiment table | - | 1 | 208.7 KB | opened |
| every file of the PSI category "Experimental table" | experiment table | PSI experiment table | - | 1 | 7.8 KB | name |
| `<test>.txt` | script or settings | test point timeline: sequence stamps and the image counts of each camera unit | - | 121 | 3.9 MB | opened |
| `<test>_MDCA_CFI_Testpoint_Settings.txt` | script or settings | test point settings of the insert against time (needles, igniters, dispensers) | MDCA | 123 | 474.4 KB | sibling |
| `<test>-<test> Settings.zip` | script or settings | settings files, zipped | - | 1 | 89.0 KB | name |
| `<test>_Chamber_Readings.txt` | sensor time series | chamber pressures and temperatures against time | CIR fluids control unit | 123 | 43.3 MB | opened |
| `<test>_Radiometer_Readings.txt` | sensor time series | radiometers at 100 Hz with igniter state | MDCA radiometers | 123 | 28.5 MB | sibling |
| `<test>_Filter_Positions.txt` | sensor time series | sequence phase and radiometer filter positions against time | MDCA | 123 | 3.3 MB | opened |
| `<test>_INTNS_Camera_Settings.txt` | camera settings | camera settings against time | intensified camera (IPSU-G5) | 123 | 184.5 KB | opened |
| `<test>_MDCA_Color_Camera_Settings.txt` | camera settings | camera settings against time | MDCA colour camera (lens and exposure) | 123 | 154.9 KB | opened |
| `<test>_HIBMS_Camera_Settings.txt` | camera settings | camera settings against time | HiBMs camera (IPSU1) | 123 | 105.8 KB | opened |
| `<test>_HIBMS_CFI_Camera_Settings.txt` | camera settings | camera settings against time | LCTF or HFR camera (IPSU3) | 123 | 68.3 KB | opened |
| `<test>_Color_Camera_Settings.txt` | camera settings | camera settings against time | colour camera (IPSU4) | 123 | 38.7 KB | opened |
| `<test>A.zip` | image archive | frames of one test point | HiBMs camera (IPSU1) | 79 | 248.6 GB | sibling |
| `<test>E_IPSUG.zip` ; `<test>E_IPSUG .zip` | image archive | frames of one test point | intensified camera (IPSU-G5) | 78 | 229.3 GB | sibling |
| `<test>D.zip` ; `<test>D_IPSUG.zip` | image archive | frames of one test point | colour camera (IPSU4) | 81 | 51.5 GB | sibling |
| `IPSU-# IMAGES.zip` ; `IPSU#.zip` and 6 more spellings | image archive | frames by camera unit | CIR IPSU camera | 12 | 24.3 GB | name |
| `<test>C.zip` | image archive | frames of one test point | LCTF or HFR camera (IPSU3) | 59 | 21.8 GB | sibling |
| `vlc-record-#-#-#-#h#m#s-udp_<test>.mp#` ; `vlc-record-#-#-#-#h#m#s-udp_<test>.ts` and 22 more spellings | video | recording of the downlinked operations video | operations video downlink | 242 | 127.9 GB | name |
| `<test>_Movies.zip` ; `<test> Movies.zip` and 2 more spellings | video | movies, zipped | - | 27 | 13.6 GB | name |
| `Droplet_View_Movie.mp#` ; `Flame_View_Movie.mp#` and 2 more spellings | video | sample movie of one camera view | - | 4 | 850.6 MB | name |
| `<test>_#Mar#_Dod_v#.pxp` ; `<test>_#May#_Dod_v#.pxp` and 3 more spellings | processed or analysed data | Igor experiment file with the processed series | - | 29 | 974.9 MB | opened |
| `<test>_RadiometerData.txt` | processed or analysed data | radiometer traces on the test clock | MDCA radiometers | 24 | 21.2 MB | name |
| `<test>_Summary.pdf` ; `<test>_Fr#_DualFlame.pdf` and 6 more spellings | processed or analysed data | summary sheet or figure of one test | - | 32 | 13.7 MB | name |
| `CoolFlameImage.pdf` ; `Images_From_CFI.pptx` and 9 more spellings | processed or analysed data | sample figure | - | 11 | 10.2 MB | name |
| `<test>_DropletData.txt` | processed or analysed data | droplet diameter against time | HiBMs camera (IPSU1) | 24 | 1.4 MB | sibling |
| `<test>_CoolFlameData.txt` | processed or analysed data | cool-flame diameter against time | intensified camera (IPSU-G5) | 17 | 902.9 KB | opened |
| `<test>_ColorCamData.txt` | processed or analysed data | visible flame diameter against time | colour camera | 24 | 379.8 KB | sibling |
| every file of the PSI category "Science Documents" | science document | science document | - | 2 | 15.9 MB | name |
| every file of the PSI category "Engineering Documents" | engineering document | engineering document | - | 1 | 115.5 KB | name |
| every file of the PSI category "Reports" | report | report | - | 1 | 811.5 KB | name |
| every file of the PSI category "Presentations" | presentation | presentation | - | 2 | 5.9 MB | name |
| `<test>_TestSheet.rtf` ; `<test>_TestSheet.txt` | metadata | test sheet: conditions, narrative and the investigators' derived values (outcome-bearing) | - | 44 | 31.8 MB | opened |
| `#_#.#_INTNS_<test>E_.metadata` ; `#_#.#_INTNS_#A#E_.metadata` | metadata | per-frame index of the archive (time stamp, exposure, gain) | intensified camera (IPSU-G5) | 116 | 31.6 MB | sibling |
| `<test>A.txt` | metadata | per-frame index of the archive (frame number, time stamp) | HiBMs camera (IPSU1) | 116 | 26.3 MB | opened |
| `<test>D.txt` | metadata | per-frame index of the archive (frame number, time stamp) | colour camera (IPSU4) | 117 | 26.1 MB | opened |
| `<test>C.txt` | metadata | per-frame index of the archive (frame number, time stamp) | LCTF or HFR camera (IPSU3) | 80 | 18.6 MB | opened |
| `<test>_Comments.txt` ; `<test>_Comments .txt` and 1 more spellings | metadata | operator notes on the image downlink of the test point | - | 123 | 635.2 KB | sibling |
| `#_#.#_INTNS_<test>E_#.meta` ; `#_#.#_INTNS_#A#E_#.meta` | metadata | camera state at the first or last frame | intensified camera (IPSU-G5) | 234 | 22.4 KB | opened |
| `<test>E.txt` | metadata | per-frame index of the archive (frame number, time stamp) | intensified camera (IPSU-G5) | 1 | 17.6 KB | opened |
| every file of the PSI category "Investigation Metadata Files" | metadata | PSI investigation metadata archive | - | 1 | 10.2 KB | name |

### PSI-47 DAFT / DAFT-2

Dust and Aerosol Measurement Feasibility Test / Dust and Aerosol Measurement Feasibility Test-2. Hardware: TSI P-Trak. Licence: CC0-1.0. 33 listed files, 41.8 MB.

**Naming.** `DTrk0002.txt` and `PTrk0002.txt`: the instrument (DustTrak, P-Trak) and its own test number
(checked: an instrument log with serial number and test id). Analysed workbooks are named by GMT day
(`flight data 063_DTrk0002.xls`). The two sessions of the experiment table (D1 on GMT 54, D2 on GMT 63) are not
in the raw file names.

| Pattern | Kind | What it is | Instrument | Files | Bytes | Sure |
|---|---|---|---|---:|---:|---|
| every file of the PSI category "Experimental table" | experiment table | PSI experiment table | - | 1 | 518 B | name |
| `<test>.txt` | sensor time series | instrument log of one test | DustTrak | 3 | 9.5 KB | opened |
| `<test>.txt` | sensor time series | instrument log of one test | P-Trak | 3 | 9.0 KB | opened |
| every file of the PSI category "Analyzed Data" | processed or analysed data | analysed data | - | 12 | 3.5 MB | name |
| every file of the PSI category "Engineering Documents" | engineering document | engineering document | - | 6 | 8.0 MB | name |
| every file of the PSI category "Reports" | report | report | - | 3 | 13.9 MB | name |
| every file of the PSI category "Presentations" | presentation | presentation | - | 3 | 15.1 MB | name |
| every file of the PSI category "Investigation Metadata Files" | metadata | PSI investigation metadata archive | - | 1 | 8.5 KB | name |
| every file of the PSI category "Representative Image(s)" | other | representative image of the investigation | - | 1 | 1.3 MB | name |

### PSI-68 FLEX-2

Flame Extinguishment Experiment - 2. Hardware: Combustion Integrated Rack (CIR), Multi-user Droplet Combustion Appartatus (MDCA). Licence: CC0-1.0. 10156 listed files, 1.2 TB (10077 distinct).

**Naming.** The rack test point only, `A109D03`: a letter, the GMT day of the year, a type letter and a
two-digit sequence. There are no FLEX numbers and no processed files. The file names carry no
`PSI-68_Raw Data_` prefix for most rows; 79 listing rows repeat an earlier row.

**Type letter R is not a burn.** The 126 ids of the form `A109R01` have chamber and radiometer readings and no
image archive, and 124 of them have a `_Mixture.txt` (checked: one row with date, O2, He, CO2, N2, H2O and nine
fuel vapours in percent). They are catalogued as chamber gas readings.

**Fuel is in the folder, not in the name.** `Reduced Gravity/<series> <year> - <fuel>/` holds the files of a
test point: decane, heptane, n-octane, iso-octane, toluene, propylbenzene, ethanol, their mixtures and
propanol with glycerol. For example 18 test points sit under "SurrogateFuel 2013 - Heptane". The flat listing
does not have this; the raw listing tree does.

**Camera letter.** As in CFI: A HiBMs (IPSU1), B LLL-UV (IPSU2), C LCTF or HFR (IPSU3), D colour camera (IPSU4).
Checked: the settings files name the unit in their first column, and the comments file counts the frames of
`<test>A` with the count the timeline gives for IPSU1.

**Calibration points.** `LCTF###`, `BKGD###`, `RDTST##` are not tests.

| Pattern | Kind | What it is | Instrument | Files | Bytes | Sure |
|---|---|---|---|---:|---:|---|
| every file of the PSI category "Experimental table" | experiment table | PSI experiment table | - | 1 | 1.0 KB | name |
| `<test>.txt` ; `<test>.docx` | script or settings | test point timeline: sequence stamps and the image counts of each camera unit | - | 773 | 9.7 MB | opened |
| `<test>_MDCA_Testpoint_Settings.txt` ; `<test>_MDCA_Testpiont_Settings.txt` and 5 more spellings | script or settings | test point settings of the insert against time (needles, igniters, dispensers) | MDCA | 776 | 2.2 MB | opened |
| `<test>_Cool_Flames.txt` | script or settings | cool-flame trigger settings of the insert | MDCA | 59 | 46.0 KB | opened |
| `<test>_Chamber_Readings.txt` ; `<test>_Chamber_Readings#.txt` and 1 more spellings | sensor time series | chamber pressures and temperatures against time | CIR fluids control unit | 718 | 198.7 MB | opened |
| `<test>_Radiometer_Readings.txt` ; `<test>_Radiomter_Readings.txt` and 1 more spellings | sensor time series | radiometers at 100 Hz with igniter state | MDCA radiometers | 722 | 155.6 MB | opened |
| `<test>_Mixture.txt` ; `<test>_MIxture.txt` | sensor time series | chamber gas composition reading (one row per reading) | gas chromatograph | 125 | 37.6 KB | opened |
| `<test>C_LCTF_Wavelengths.txt` ; `<test>_LCTF_Wavelengths.txt` and 3 more spellings | camera settings | tunable-filter wavelengths against time | LCTF or HFR camera (IPSU3) | 124 | 14.2 MB | opened |
| `<test>_MDCA_Color_Camera_Settings.txt` ; `<test>_MDCA_Camera_Settings.txt` | camera settings | camera settings against time | MDCA colour camera (lens and exposure) | 598 | 758.4 KB | opened |
| `<test>_HIBMS_Camera_Settings.txt` | camera settings | camera settings against time | HiBMs camera (IPSU1) | 596 | 512.6 KB | opened |
| `<test>_LLLUV_Camera_Settings.txt` | camera settings | camera settings against time | LLL-UV camera (IPSU2) | 502 | 439.7 KB | opened |
| `<test>_LCTF_Camera_Settings.txt` ; `<test>_HFR_Camera_Settings.txt` and 1 more spellings | camera settings | camera settings against time | LCTF or HFR camera (IPSU3) | 653 | 367.3 KB | opened |
| `<test>_Color_Camera_Settings.txt` | camera settings | camera settings against time | colour camera (IPSU4) | 594 | 186.9 KB | opened |
| `<test>_LLL_Camera_Settings.txt` | camera settings | camera settings against time | LLL camera (IPSU2) | 59 | 51.3 KB | opened |
| `<test>_LLLIR_Camera_Settings.txt` | camera settings | camera settings against time | LLL-IR camera (IPSU2) | 33 | 28.9 KB | opened |
| `<test>_Camera_Settings.txt` | camera settings | camera settings against time | - | 1 | 564 B | opened |
| `<test>A.zip` | image archive | frames of one test point | HiBMs camera (IPSU1) | 527 | 756.7 GB | opened |
| `<test>D.zip` | image archive | frames of one test point | colour camera (IPSU4) | 523 | 216.7 GB | opened |
| `<test>C.zip` | image archive | frames of one test point | LCTF or HFR camera (IPSU3) | 470 | 136.1 GB | opened |
| `<test>B.zip` ; `<test>B_#.zip` | image archive | frames of one test point | LLL-UV camera (IPSU2) | 534 | 110.3 GB | opened |
| `<test>.zip` | image archive | archive named only by the test point | - | 2 | 13.9 MB | unknown |
| `<test>C.zip` | calibration | calibration run: frames of one test point | LCTF or HFR camera (IPSU3) | 54 | 2.6 GB | opened |
| every file of the PSI category "Calibration" | calibration | calibration file | - | 3 | 1.5 GB | name |
| `<test>A.zip` | calibration | calibration run: frames of one test point | HiBMs camera (IPSU1) | 1 | 307.9 MB | opened |
| `IPSU#.zip` | calibration | calibration run: frames by camera unit | CIR IPSU camera | 1 | 3.0 MB | name |
| `<test>.zip` ; `<test>_#.zip` | calibration | calibration run: archive named only by the test point | - | 5 | 12.2 KB | unknown |
| every file of the PSI category "Science Documents" | science document | science document | - | 3 | 1.7 MB | name |
| every file of the PSI category "Engineering Documents" | engineering document | engineering document | - | 7 | 8.6 MB | name |
| `<test>D.txt` | metadata | per-frame index of the archive (frame number, time stamp) | colour camera (IPSU4) | 290 | 42.7 MB | sibling |
| `<test>B.txt` | metadata | per-frame index of the archive (frame number, time stamp) | LLL-UV camera (IPSU2) | 257 | 35.8 MB | sibling |
| `<test>A.txt` | metadata | per-frame index of the archive (frame number, time stamp) | HiBMs camera (IPSU1) | 253 | 32.1 MB | sibling |
| `<test>C.txt` | metadata | per-frame index of the archive (frame number, time stamp) | LCTF or HFR camera (IPSU3) | 200 | 31.3 MB | sibling |
| `<test>_Event_Log_entry.docx` ; `<test>_Event_Log_Entry.docx` and 2 more spellings | metadata | event log entry of the test point | - | 93 | 3.5 MB | opened |
| `<test>_Comments.txt` ; `<test> _Comments.txt` and 3 more spellings | metadata | operator notes on the image downlink of the test point | - | 598 | 2.5 MB | opened |
| every file of the PSI category "Investigation Metadata Files" | metadata | PSI investigation metadata archive | - | 1 | 24.1 KB | name |

### PSI-69 FLEX

Flame Extinguishment Experiment. Hardware: Combustion Integrated Rack (CIR)-Fluids and Combustion Facility (FCF), Multi-user Droplet Combustion Appartatus (MDCA). Licence: CC0-1.0. 6555 listed files, 923.8 GB (6453 distinct).

**Naming.** Two names for one test, and three spellings of the first. Raw files use the rack test point:
`053F001` (GMT day, letter, three-digit sequence; 81 ids), `H10M101` or `C01H101` (letter, two digits, letter,
three digits; 183 ids) and, for the 2013 points, `X013M01` or `C021H02` (letter, GMT day, type letter,
two-digit sequence; 151 ids). Processed files use the FLEX number, `FLEX-001`. No file name carries both, so the
link needs the experiment table or the records; this catalogue reads neither and reports the two schemes
side by side.

**Camera letter.** As in FLEX-2: `<test>A.zip` HiBMs, `B` LLL-UV, `C` LCTF or HFR, `D` colour camera. Inferred
from FLEX-2. All 49 test points with an LCTF settings file have a `C` archive; 127 more have a `C` archive and
no settings file at all (the 2013 points).

**Folders.** `Reduced Gravity/<fuel>[_<diluent>]/`: MeOH, Heptane, and both with carbon dioxide, helium,
nitrogen, xenon or air. Two folders hold calibration files under "Raw Data" ("Radiometer Calibration",
"HiBMs-LCTF Calibration Data"); their files are catalogued as calibration. The 32 type-R points sit there.

**Check against known ids.** Of the 156 train and validation runs of the two FLEX splits, 150 are found as a
test point in file names (136 with a sensor series, 139 with an image archive). Six have no file named by the
test point: `256F002`, `256F007`, `256F008`, `C02M201`, `C02H201`, `C02H202`. 102 listing rows repeat an
earlier row.

| Pattern | Kind | What it is | Instrument | Files | Bytes | Sure |
|---|---|---|---|---:|---:|---|
| every file of the PSI category "Experimental table" | experiment table | PSI experiment table | - | 1 | 24.8 KB | name |
| `<test>.txt` | script or settings | test point timeline: sequence stamps and the image counts of each camera unit | - | 284 | 4.1 MB | sibling |
| `<test>_MDCA_Testpoint_Settings.txt` ; `<test>_Testpoint_Settings.txt` and 1 more spellings | script or settings | test point settings of the insert against time (needles, igniters, dispensers) | MDCA | 284 | 762.3 KB | sibling |
| `<test>_Chamber_Readings.txt` | sensor time series | chamber pressures and temperatures against time | CIR fluids control unit | 282 | 90.5 MB | sibling |
| `<test>_Radiometer_Readings.txt` | sensor time series | radiometers at 100 Hz with igniter state | MDCA radiometers | 283 | 80.8 MB | sibling |
| `<test>_Chamber_Contents.txt` | sensor time series | chamber gas composition reading (one row per reading) | gas chromatograph | 1 | 4.0 KB | sibling |
| `<test>_MDCA_Color_Camera_Settings.txt` ; `<test>_MDCA_Camera_Settings.txt` | camera settings | camera settings against time | MDCA colour camera (lens and exposure) | 283 | 360.1 KB | sibling |
| `<test>_LLLUV_Camera_Settings.txt` ; `<test>_LLLUV_Camera_Setttings.txt` | camera settings | camera settings against time | LLL-UV camera (IPSU2) | 283 | 247.9 KB | sibling |
| `<test>_HIBMS_Camera_Settings.txt` ; `<test>_HIMBS_Camera_Settings.txt` and 1 more spellings | camera settings | camera settings against time | HiBMs camera (IPSU1) | 284 | 244.9 KB | sibling |
| `<test>_Color_Camera_Settings.txt` ; `<test>_Color_Camaera_Settings.txt` | camera settings | camera settings against time | colour camera (IPSU4) | 283 | 89.1 KB | sibling |
| `<test>_LCTF_Camera_Settings.txt` | camera settings | camera settings against time | LCTF or HFR camera (IPSU3) | 53 | 29.9 KB | sibling |
| `<test>A.zip` | image archive | frames of one test point | HiBMs camera (IPSU1) | 396 | 625.9 GB | sibling |
| `<test>D.zip` | image archive | frames of one test point | colour camera (IPSU4) | 396 | 133.2 GB | sibling |
| `<test>B.zip` | image archive | frames of one test point | LLL-UV camera (IPSU2) | 408 | 69.6 GB | sibling |
| `<test>C.zip` | image archive | frames of one test point | LCTF or HFR camera (IPSU3) | 176 | 49.7 GB | sibling |
| `vlc-record-#-#-#-#h#m#s-udp___<test>.mp#` ; `vlc-record-#-#-#-#h#m#s-udp___<test>B.mp#` and 18 more spellings | video | recording of the downlinked operations video | operations video downlink | 199 | 38.1 GB | name |
| `HiBMS# View Movie.mp#` ; `Droplet Movie View.mp#` and 11 more spellings | video | sample movie of one camera view | - | 15 | 61.4 MB | name |
| `<test>_Summary.pdf` ; `<test>_ResultsSummary.pdf` and 9 more spellings | processed or analysed data | summary sheet or figure of one test | - | 391 | 104.8 MB | name |
| `<test>_DropletData.txt` ; `<test>_DropletHistory.txt` | processed or analysed data | droplet diameter against time | HiBMs camera (IPSU1) | 261 | 10.1 MB | opened |
| `<test>_LLUVData.txt` | processed or analysed data | UV flame diameter against time | LLL-UV camera (IPSU2) | 258 | 6.5 MB | opened |
| `<test>_BacklitSample.pdf` ; `<test>_Layout.pdf` and 5 more spellings | processed or analysed data | sample figure | - | 7 | 3.4 MB | name |
| `<test>_ColorCamData.txt` | processed or analysed data | visible flame diameter against time | colour camera | 222 | 2.9 MB | opened |
| `<test>-Test# Results.xls` ; `Test # Results.xls` | processed or analysed data | results workbook | - | 2 | 1.9 MB | name |
| `<test>A.zip` | calibration | calibration run: frames of one test point | HiBMs camera (IPSU1) | 7 | 2.4 GB | sibling |
| `<test>D.zip` | calibration | calibration run: frames of one test point | colour camera (IPSU4) | 8 | 2.1 GB | sibling |
| `<test>B.zip` | calibration | calibration run: frames of one test point | LLL-UV camera (IPSU2) | 8 | 964.1 MB | sibling |
| `<test>C.zip` | calibration | calibration run: frames of one test point | LCTF or HFR camera (IPSU3) | 20 | 916.9 MB | sibling |
| `vlc-record-#-#-#-#h#m#s-udp___<test>.mp#` | calibration | calibration run: recording of the downlinked operations video | operations video downlink | 15 | 106.3 MB | name |
| `<test>_Radiometer_Readings.txt` | calibration | calibration run: radiometers at 100 Hz with igniter state | MDCA radiometers | 39 | 3.4 MB | sibling |
| `<test>_Chamber_Readings.txt` | calibration | calibration run: chamber pressures and temperatures against time | CIR fluids control unit | 39 | 2.3 MB | sibling |
| `<test>D.txt` | calibration | calibration run: per-frame index of the archive (frame number, time stamp) | colour camera (IPSU4) | 8 | 1.5 MB | sibling |
| `<test>A.txt` | calibration | calibration run: per-frame index of the archive (frame number, time stamp) | HiBMs camera (IPSU1) | 8 | 1.3 MB | sibling |
| `<test>B.txt` | calibration | calibration run: per-frame index of the archive (frame number, time stamp) | LLL-UV camera (IPSU2) | 8 | 1.3 MB | sibling |
| `<test>.txt` | calibration | calibration run: test point timeline: sequence stamps and the image counts of each camera unit | - | 56 | 355.0 KB | sibling |
| `<test>_MDCA_Testpoint_Settings.txt` ; `<test>_Testpoint_Settings.txt` and 2 more spellings | calibration | calibration run: test point settings of the insert against time (needles, igniters, dispensers) | MDCA | 57 | 154.1 KB | sibling |
| `<test>_MDCA_Camera_Settings.txt` | calibration | calibration run: camera settings against time | MDCA colour camera (lens and exposure) | 8 | 10.1 KB | sibling |
| `<test>_LCTF_Camera_Settings.txt` | calibration | calibration run: camera settings against time | LCTF or HFR camera (IPSU3) | 18 | 10.0 KB | sibling |
| `<test>_Mixture.txt` ; `<test>_Chamber_Gases.txt` | calibration | calibration run: chamber gas composition reading (one row per reading) | gas chromatograph | 39 | 8.2 KB | sibling |
| `<test>_LLLUV_Camera_Settings.txt` | calibration | calibration run: camera settings against time | LLL-UV camera (IPSU2) | 8 | 7.0 KB | sibling |
| `<test>_HIBMS_Camera_Settings.txt` | calibration | calibration run: camera settings against time | HiBMs camera (IPSU1) | 8 | 6.9 KB | sibling |
| `<test>_Color_Camera_Settings.txt` | calibration | calibration run: camera settings against time | colour camera (IPSU4) | 8 | 2.5 KB | sibling |
| `<test>_Readme.txt` ; `filelist.txt` | calibration | calibration run: note about the files | - | 2 | 670 B | name |
| every file of the PSI category "Science Documents" | science document | science document | - | 2 | 204.6 KB | name |
| every file of the PSI category "Engineering Documents" | engineering document | engineering document | - | 3 | 5.7 MB | name |
| every file of the PSI category "Presentations" | presentation | presentation | - | 4 | 7.1 MB | name |
| `<test>_TestSheet.txt` ; `<test>_TestSheet.rtf` and 3 more spellings | metadata | test sheet: conditions, narrative and the investigators' derived values (outcome-bearing) | - | 561 | 253.9 MB | opened |
| `<test>B.txt` | metadata | per-frame index of the archive (frame number, time stamp) | LLL-UV camera (IPSU2) | 106 | 12.8 MB | sibling |
| `<test>A.txt` | metadata | per-frame index of the archive (frame number, time stamp) | HiBMs camera (IPSU1) | 101 | 12.7 MB | sibling |
| `<test>D.txt` | metadata | per-frame index of the archive (frame number, time stamp) | colour camera (IPSU4) | 104 | 12.3 MB | sibling |
| `<test>C.txt` | metadata | per-frame index of the archive (frame number, time stamp) | LCTF or HFR camera (IPSU3) | 23 | 3.5 MB | sibling |
| `<test>_Comments.txt` | metadata | operator notes on the image downlink of the test point | - | 230 | 194.9 KB | sibling |
| every file of the PSI category "Investigation Metadata Files" | metadata | PSI investigation metadata archive | - | 1 | 24.8 KB | name |
| `<test>_MissingImageData.txt` ; `<test>_Readme.txt` and 1 more spellings | metadata | note about the files | - | 4 | 783 B | name |

### PSI-98 SAFFIRE-I

Spacecraft Fire Safety Experiment-I. Hardware: Saffire 1 enclosure in Cygnus. Licence: CC0-1.0. 76 listed files, 12.3 GB.

**Naming.** `S1C1` is sample 1 seen by camera 1; `F1C1` is the flow visualization before sample 1 (the experiment
table says so). The id is in the folder (`Reduced Gravity/SAFFIRE_I S1C1`) and in most names. The telemetry is
per sample (`Saffire-I Sample 1 Sensor Data Complete TM graphs.xlsx`). The same frames are offered in several
forms: JPEG as downlinked, JPEG mapped, raw converted to TIFF, PNG.

| Pattern | Kind | What it is | Instrument | Files | Bytes | Sure |
|---|---|---|---|---:|---:|---|
| every file of the PSI category "Experimental table" | experiment table | PSI experiment table | - | 1 | 469 B | name |
| `Saffire-I Sample # Sensor Data Complete TM graphs.xlsx` ; `Saffire-I Telemetry Data Good Process Version.xlsx` | sensor time series | sensor telemetry | telemetry | 3 | 52.7 MB | name |
| `SAFFIRE_I_<test>C#_jpeg#png_icmad_part#.zip` ; `SAFFIRE_I_<test>C#_jpeg#png_icmad.zip` | image archive | frames converted to PNG | camera 1 | 4 | 4.7 GB | name |
| `SAFFIRE_I_<test>C#_jpeg#png_icmad_part#.zip` ; `SAFFIRE_I_<test>C#_jpeg#png_icmad.zip` | image archive | frames converted to PNG | camera 2 | 4 | 4.5 GB | name |
| `SAFFIRE_I_<test>C#_JPEG_Mapped_part#.zip` ; `SAFFIRE_I_<test>C#_JPEG_Mapped.zip` and 2 more spellings | image archive | JPEG frames, mapped | camera 2 | 7 | 778.7 MB | name |
| `SAFFIRE_I_<test>C#_JPEG_Mapped_part#.zip` ; `SAFFIRE_I_<test>C#_JPEG_Mapped.zip` and 1 more spellings | image archive | JPEG frames, mapped | camera 1 | 6 | 653.3 MB | name |
| `SAFFIRE_I_<test>C#_jpeg_part#.zip` ; `SAFFIRE_I_<test>C#_jpeg.zip` and 1 more spellings | image archive | JPEG frames as downlinked | camera 1 | 6 | 422.8 MB | name |
| `SAFFIRE_I_<test>C#_jpeg.zip` ; `SAFFIRE_I_<test>C#_jpeg_part#.zip` | image archive | JPEG frames as downlinked | camera 2 | 6 | 398.4 MB | name |
| `SAFFIRE_I_<test>C#_Raw_DistroMaster.zip` ; `<test>C#-#_icmad.zip` and 1 more spellings | image archive | raw frames as distributed | camera 1 | 4 | 304.5 MB | name |
| `SAFFIRE_I<test>C#_Raw_DistroMaster.zip` ; `SAFFIRE_I_<test>C#-Raw_DistroMaster.zip` and 1 more spellings | image archive | raw frames as distributed | camera 2 | 3 | 249.5 MB | name |
| `SAFFIRE-I_Mapped_<test>C#_Raw#TIFF.zip` ; `SAFFIRE_I_<test>C#Raw#TIFF_Complete_icmad.zip` and 3 more spellings | image archive | raw frames converted to TIFF | camera 1 | 5 | 45.9 MB | name |
| `SAFFIRE_I_<test>C#_Raw#TIFF_Complete.zip` ; `SAFFIRE_I_<test>C#_RAW#TIFF_Complete_icmad.zip` and 3 more spellings | image archive | raw frames converted to TIFF | camera 2 | 6 | 38.3 MB | name |
| `SAFFIRE_I_<test>C#_Raw#TIFF_Mapped_JPEGS.zip` | image archive | raw frames converted to TIFF and mapped | camera 1 | 2 | 26.0 MB | name |
| `SAFFIRE_<test>C#_RAW#TIFF_Mapped_JPEGS.zip` ; `SAFFIRE_I_<test>C#_Raw#TIFF_Mapped.zip` | image archive | raw frames converted to TIFF and mapped | camera 2 | 2 | 20.8 MB | name |
| `Saffire I Sample # TM graphs.docx` ; `Saffire-I Sample # TM graphs.docx` | processed or analysed data | plots of the telemetry | - | 2 | 2.4 MB | name |
| `SAFFIRE-#_<test>C#_DistortionGrids_#x#_originals.zip` ; `Mapped_SAFFIRE-#_<test>C#_DistortionGrid_#x#.zip` | calibration | distortion grid images of one camera | camera 1 | 3 | 1.4 MB | name |
| `Mapped_SAFFIRE-#_<test>C#_DistortionGrid_#x#.zip` ; `SAFFIRE-#_<test>C#_DistortionGrid_#x#.zip` and 1 more spellings | calibration | distortion grid images of one camera | camera 2 | 3 | 1.4 MB | name |
| every file of the PSI category "Science Documents" | science document | science document | - | 3 | 514.5 KB | name |
| every file of the PSI category "Engineering Documents" | engineering document | engineering document | - | 2 | 1.7 MB | name |
| `SFSDP-VIS-#_Saffire # and #_Thermocouple Placement.pdf` | engineering document | engineering note | - | 1 | 1.1 MB | name |
| `SAFFIRE-I_ImageFrameSizes_ROI_Offsets_Grid_#x#_v#.#.pdf` | engineering document | engineering note | camera 1 | 1 | 784.3 KB | name |
| `SAFFIRE-#_MergeResult-jpeg_Update_Final_v#.xlsx` | metadata | merge table of the image files | - | 1 | 12.3 MB | unknown |
| every file of the PSI category "Investigation Metadata Files" | metadata | PSI investigation metadata archive | - | 1 | 10.0 KB | name |

### PSI-99 SAFFIRE-II

Spacecraft Fire Experiment-II. Hardware: Cygnus Pressurized Cargo Module, SAFFIRE. Licence: CC0-1.0. 217 listed files, 16.9 GB (214 distinct).

**Naming.** As SAFFIRE-I, with nine samples `S1` to `S9` (the experiment table numbers them 2-1 to 2-9) and
two cameras. Telemetry workbooks are per sample (`Saffire Sensor Data Sample 3.xlsx`). Three listing rows
repeat an earlier row.

| Pattern | Kind | What it is | Instrument | Files | Bytes | Sure |
|---|---|---|---|---:|---:|---|
| every file of the PSI category "Experimental table" | experiment table | PSI experiment table | - | 1 | 2.0 KB | name |
| `Saffire Sensor Data Sample #.xlsx` ; `Saffire-II Telemetry Data.xlsx` | sensor time series | sensor telemetry | telemetry | 10 | 210.1 MB | name |
| `SAFFIRE-II_<test>C#_Image_Data_jpegs#png_icmad_part#.zip` ; `SAFFIRE-II_<test>C#_Data_jpegs#png_icmad_part#.zip` and 1 more spellings | image archive | frames converted to PNG | camera 2 | 16 | 5.6 GB | name |
| `SAFFIRE-II_<test>C#_Image_Data_RAW#TIFF_Mapped.zip` ; `SAFFIRE-II_<test>C#_Image_Data_RAW#TIFF_Mapped_ICMAD.zip` and 4 more spellings | image archive | raw frames converted to TIFF and mapped | camera 1 | 15 | 3.6 GB | name |
| `SAFFIRE_II_<test>C#_Image_Data_RAW#TIFF.zip` ; `SAFFIRE_II_<test>C#_Raw#TIFF.zip` | image archive | raw frames converted to TIFF | camera 1 | 8 | 2.1 GB | name |
| `SAFFIRE_II_<test>C#_Image_Data_jpegs#png_icmad_part#.zip` ; `SAFFIRE-II_<test>C#_Image_Data_jpegs#pnc_icmad_part#.zip` | image archive | frames converted to PNG | camera 1 | 7 | 1.5 GB | name |
| `SAFFIRE-II_<test>C#_Image_Data_RAW#TIFF_Mapped.zip` ; `SAFFIRE-II_<test>C#_Image_Data_RAW#TIFF_Mapped_ICMAD.zip` and 2 more spellings | image archive | raw frames converted to TIFF and mapped | camera 2 | 13 | 1.4 GB | name |
| `SAFFIRE-II_<test>C#_Image_Data_All_JPEGs_Mapped_part#.zip` ; `SAFFIRE-II_<test>C#_Image_Data_All-JPEGs_Mapped_part#.zip` and 1 more spellings | image archive | JPEG frames, mapped | camera 2 | 18 | 576.9 MB | name |
| `SAFFIRE-II_<test>C#_Image_Data_All_JPEGs_Mapped_part#.zip` ; `SAFFIRE-II_<test>C#_Image_Data_All-JPEGs_Mapped_part#.zip` and 3 more spellings | image archive | JPEG frames, mapped | camera 1 | 23 | 574.3 MB | name |
| `SAFFIRE_II_<test>C#_Image_Data_All_JPEGs_part#.zip` ; `SAFFIRE-II_<test>C#_Image_Data_All_JPEGs_part#.zip` and 7 more spellings | image archive | JPEG frames as downlinked | camera 2 | 20 | 456.0 MB | name |
| `SAFFIRE_II_<test>C#_Image_Data_RAW#TIFF.zip` | image archive | raw frames converted to TIFF | camera 2 | 7 | 423.0 MB | name |
| `SAFFIRE-II_<test>C#_Image_Data_All_JPEGs_part#.zip` ; `SAFFIRE-II_<test>C#_Image_Data_All_JPEGS_part#.zip` and 3 more spellings | image archive | JPEG frames as downlinked | camera 1 | 25 | 411.5 MB | name |
| `SAFFIRE_II_<test>C#_Image_Data_RAWs.zip` | image archive | raw frames as distributed | camera 2 | 1 | 21.7 MB | name |
| `SAFFIRE_II_<test>C#_Illustrative_Images.zip` ; `SAFFIRE_II_<test>C#_Illustrative_images.zip` | image archive | illustrative images | camera 1 | 2 | 207.4 KB | name |
| `SAFFIRE_II_<test>C#_Illustrative_images.zip` | image archive | illustrative images | camera 2 | 1 | 167.6 KB | name |
| `Saffire-II Sample # TM Graphs.docx` | processed or analysed data | plots of the telemetry | - | 9 | 7.6 MB | name |
| `SaffireIIStitchedCO#V#.pdf` | processed or analysed data | stitched CO2 plot | - | 1 | 25.9 KB | name |
| `SAFFIRE-II_<test>C#_DistortionGrids_Sample_ROI_Mapped JPEGS.zip` ; `SAFFIRE_II_<test>C#_DistortionGrids_Sample_ROI.zip` and 3 more spellings | calibration | distortion grid images of one camera | camera 2 | 16 | 14.1 MB | name |
| `SAFFIRE-II_<test>C#_DistortionGrids_Sample_ROI_Mapped JPEGS.zip` ; `SAFFIRE_II_<test>C#_DistortionGrids_ROI.zip` and 6 more spellings | calibration | distortion grid images of one camera | camera 1 | 16 | 6.5 MB | name |
| `#_-_Overlap_verification_<test>C#.zip` | calibration | camera overlap verification images | camera 1 | 1 | 20.3 KB | name |
| every file of the PSI category "Science Documents" | science document | science document | - | 3 | 1.1 MB | name |
| `SFS-W-#_MFB#-Saffire-# SIBAL samples-Gas phase TC height nonconformance.pdf` | engineering document | engineering note | - | 1 | 20.0 KB | name |
| `SAFFIRE-II Operations.pdf` | engineering document | engineering document | camera 1 | 1 | 14.0 KB | name |
| `SAFFIRE-II_MergeResult-jpeg_v#_Final.xlsx` | metadata | merge table of the image files | - | 1 | 20.8 MB | unknown |
| every file of the PSI category "Investigation Metadata Files" | metadata | PSI investigation metadata archive | - | 1 | 10.5 KB | name |

### PSI-100 SAFFIRE-III

Spacecraft Fire Experiment-III. Hardware: Saffire III enclosure in Cygnus. Licence: CC0-1.0. 87 listed files, 35.2 GB.

**Naming.** As SAFFIRE-I: samples `S1`, `S2`, flow visualizations `F1`, `F2`, cameras `C1`, `C2`. The telemetry
also comes as one raw CSV for the flight (`Raw telemetry file.csv`).

| Pattern | Kind | What it is | Instrument | Files | Bytes | Sure |
|---|---|---|---|---:|---:|---|
| every file of the PSI category "Experimental table" | experiment table | PSI experiment table | - | 1 | 239 B | name |
| `Saffire III Sample # Sensors.xlsx` ; `Raw telemetry file.csv` and 3 more spellings | sensor time series | sensor telemetry | telemetry | 6 | 64.8 MB | name |
| `SAFFIRE-III_<test>C#_Raw#TIFF_Mapped.zip` ; `SAFFIRE-III_<test>C#_Raw#TIFF_Map_icmad.zip` and 1 more spellings | image archive | raw frames converted to TIFF and mapped | camera 2 | 6 | 10.2 GB | name |
| `SAFFIRE_III_<test>C#_jpeg#png_part#.zip` ; `SAFFIRE-III_<test>C#_jpeg#png.zip` | image archive | frames converted to PNG | camera 2 | 7 | 7.0 GB | name |
| `SAFFIRE_III_<test>C#_jpeg#png_part#.zip` ; `SAFFIRE-III_<test>C#_jpeg#png.zip` | image archive | frames converted to PNG | camera 1 | 7 | 5.7 GB | name |
| `SAFFIRE-III_<test>C#_Raw#TIFF_Mapped.zip` ; `SAFFIRE-III_<test>C#_RAW#TIFF_Mapped.zip` and 1 more spellings | image archive | raw frames converted to TIFF and mapped | camera 1 | 4 | 5.6 GB | name |
| `SAFFIRE-III_<test>C#_RAW#TIFF.zip` | image archive | raw frames converted to TIFF | camera 2 | 4 | 2.7 GB | name |
| `SAFFIRE-III_<test>C#_RAW#TIFF.zip` | image archive | raw frames converted to TIFF | camera 1 | 3 | 1.5 GB | name |
| `SAFFIRE-III_<test>C#_JPEG_Mapped.zip` ; `SAFFIRE_III_<test>C#_JPEG_Mapped_Part#.zip` and 1 more spellings | image archive | JPEG frames, mapped | camera 2 | 7 | 796.9 MB | name |
| `SAFFIRE_III_<test>C#_JPEG_Mapped_part#.zip` ; `SAFFIRE-III_<test>C#_JPEG_Mapped.zip` | image archive | JPEG frames, mapped | camera 1 | 7 | 662.4 MB | name |
| `SAFFIRE_III_<test>C#_jpeg_part#.zip` ; `SAFFIRE-III_<test>C#_<test>_jpeg.zip` and 1 more spellings | image archive | JPEG frames as downlinked | camera 2 | 9 | 528.4 MB | name |
| `SAFFIRE_III_<test>C#_jpeg_part#.zip` ; `SAFFIRE-III_<test>C#_<test>_jpeg.zip` and 2 more spellings | image archive | JPEG frames as downlinked | camera 1 | 9 | 456.2 MB | name |
| `Saffire-III Sample # Figure for TM.docx` | processed or analysed data | plots of the telemetry | - | 2 | 2.4 MB | name |
| `SAFFIRE-III_<test>C#_Distortion Grid_#x#.zip` | calibration | distortion grid images of one camera | camera 1 | 2 | 315.5 KB | name |
| `SAFFIRE-III_<test>C#_Distortion Grid_#x#.zip` | calibration | distortion grid images of one camera | camera 2 | 2 | 314.1 KB | name |
| every file of the PSI category "Science Documents" | science document | science document | - | 3 | 525.2 KB | name |
| every file of the PSI category "Engineering Documents" | engineering document | engineering document | - | 2 | 1.7 MB | name |
| every file of the PSI category "Presentations" | presentation | presentation | - | 3 | 10.2 MB | name |
| `SAFFIRE-III_MergeResult-jpeg_Final.xlsx` ; `SAFFIRE-III_MergeResult-raw_Final.xlsx` | metadata | merge table of the image files | - | 2 | 15.6 MB | unknown |
| every file of the PSI category "Investigation Metadata Files" | metadata | PSI investigation metadata archive | - | 1 | 8.4 KB | name |

### PSI-101 SAME-R

Smoke Aerosol Measurement Experiment-Reflight. Hardware: Comparitive Soot Diagnostic Hardware; Refer to final report. Licence: CC0-1.0. 344 listed files, 6.3 GB.

**Naming.** A test is the SAME test number of the experiment table (`51`); a letter after it (`51a`) is the aged
repeat. It is the folder `Reduced Gravity/<material>/Run 51` and appears in names as
`2010_214_10_kapton_run61.xls` (year, GMT day, hour) or as the leading number of a sample file
(`55 8L Lampwick.pptx`, `#78 Postage 31L Lampwick.zip`, where `31L` is the sample code). The housekeeping
files `HKPData_gmt236_2.csv` name the day and a sequence only: 221 files take their test from the folder.
Materials: Kapton, cotton lamp wick, Pyrell, silicone, Teflon.

| Pattern | Kind | What it is | Instrument | Files | Bytes | Sure |
|---|---|---|---|---:|---:|---|
| every file of the PSI category "Experimental table" | experiment table | PSI experiment table | - | 1 | 5.7 KB | name |
| `HKPData_gmt#_#.csv` ; `HKPData_gmt#_#_run<test>.csv` | sensor time series | housekeeping record of the run: instrument readings against time | SAME avionics | 44 | 27.7 MB | opened |
| `#<test> - #P Pyrell.zip` ; `#<test> Postage #L Lampwick.zip` and 42 more spellings | image archive | images of the run's particle sample (folder: Videos, Images) | - | 70 | 1.2 GB | name |
| `<test> #L Lampwick.pptx` ; `<test> #L Lampwick.pdf` and 52 more spellings | image archive | images of the run's particle sample, as slides | - | 84 | 271.1 MB | sibling |
| `#K-K#-# -#x-#y #k.tif` ; `#K-K#-# -#x-#y #.tif` | image archive | one image of the run's particle sample | - | 3 | 1.8 MB | name |
| `vlc-record-#-#-#-#h#m#s-udp___#.#.#.# _#-.mp#` ; `# Kapton vlc-record-#-#-#-#h#m#s-udp___#.#.#.# _#-.mp#` and 4 more spellings | video | recording of the downlinked operations video | operations video downlink | 8 | 3.7 GB | name |
| `vlc-record-#-#-#-#h#m#s-udp___#.#.#.# _#-.zip` ; `# Kapton vlc-record-#-#-#-#h#m#s-udp___#.#.#.# _#-.zip` and 4 more spellings | video | the recording, zipped | operations video downlink | 8 | 480.9 MB | name |
| `#_#_#_kapton_run<test>.xls` ; `#_#_#_lampwick_run<test>.xls` and 24 more spellings | processed or analysed data | run workbook (housekeeping and instrument data) | - | 88 | 480.2 MB | sibling |
| every file of the PSI category "Analyzed Data" | processed or analysed data | analysed data | - | 1 | 28.6 MB | name |
| `SAME, SAME-R_Results.xlsx` | processed or analysed data | results workbook of both campaigns | - | 1 | 4.1 MB | name |
| every file of the PSI category "Science Documents" | science document | science document | - | 2 | 947.1 KB | name |
| `ISS Coagulation.doc` | science document | note | - | 1 | 141.3 KB | unknown |
| every file of the PSI category "Engineering Documents" | engineering document | engineering document | - | 25 | 55.6 MB | name |
| every file of the PSI category "Reports" | report | report | - | 1 | 4.6 MB | name |
| every file of the PSI category "Presentations" | presentation | presentation | - | 4 | 33.3 MB | name |
| every file of the PSI category "Reference Data" | reference data | reference data | - | 2 | 671.4 KB | name |
| every file of the PSI category "Investigation Metadata Files" | metadata | PSI investigation metadata archive | - | 1 | 11.0 KB | name |

### PSI-102 SAME

Smoke and Aerosol Measurement Experiment. Hardware: Comparitive Soot Diagnostic Hardware. Licence: CC0-1.0. 101 listed files, 657.6 MB.

**Naming.** The run number in the folder (`Reduced Gravity/<material>/Run 17`; one folder holds two runs,
`Run 19, 20`) and in most names (`GMT264 Run 17 lampwick.xls`, `GMT272lampwick31.xls`). Only analysed
workbooks are published; there is no raw file.

| Pattern | Kind | What it is | Instrument | Files | Bytes | Sure |
|---|---|---|---|---:|---:|---|
| every file of the PSI category "Experimental table" | experiment table | PSI experiment table | - | 1 | 331 B | name |
| `GMT# Run <test> DBP.xls` ; `GMT#dbp<test>hkp.xls` and 44 more spellings | processed or analysed data | run workbook (housekeeping and instrument data) | - | 59 | 228.1 MB | sibling |
| every file of the PSI category "Science Documents" | science document | science document | - | 2 | 947.1 KB | name |
| every file of the PSI category "Engineering Documents" | engineering document | engineering document | - | 28 | 362.0 MB | name |
| every file of the PSI category "Reports" | report | report | - | 1 | 3.7 MB | name |
| every file of the PSI category "Presentations" | presentation | presentation | - | 6 | 61.5 MB | name |
| every file of the PSI category "Reference Data" | reference data | reference data | - | 3 | 1.4 MB | name |
| every file of the PSI category "Investigation Metadata Files" | metadata | PSI investigation metadata archive | - | 1 | 13.1 KB | name |

### PSI-106 SLICE

Structure and Liftoff In Combustion Experiment. Hardware: not stated. Licence: CC0-1.0. 23463 listed files, 72.9 GB (18707 distinct).

**Naming.** No test id. Everything is under "Raw Data" in a folder "Legacy", without the `PSI-106_Raw Data_`
prefix. The units are the GMT day of 2012 and the picture number of the still camera:

- `GMT2012_040_D300s/` holds the stills of day 040 (Nikon raw archive, proof sheets, and the map from ISS photo
  number to camera file name);
- `OT_040_data/`, `RoomT_040_data/` and `OMA Format/<...>040<...>_data/` hold one binary array per picture:
  `1727.dat` the picture, `color_1727_1.dat`, `T_1727_ave.dat`, `fv_1727.dat`, `CH_sig_8271.dat` what was
  computed from it;
- `UT_040_048/` holds the videos of a range of days;
- `SLICE 1g 040/` holds JPEG archives and picture tables.

What `OT`, `RoomT`, `UT` and `1g` stand for is not stated in any name; the catalogue keeps them in `folder`
and does not interpret them. 4,756 listing rows repeat an earlier row (same file id, same folder): there are
18,707 distinct files.

| Pattern | Kind | What it is | Instrument | Files | Bytes | Sure |
|---|---|---|---|---:|---:|---|
| every file of the PSI category "Experimental table" | experiment table | PSI experiment table | - | 1 | 734 B | name |
| `pic_info_#.csv` ; `pic_info_#_##.xls` and 13 more spellings | camera settings | per-picture exposure record (number, time, shutter) | Nikon D300s | 32 | 598.0 KB | opened |
| `#.dat` | image archive | one picture as a binary array | Nikon D300s | 6527 | 34.2 GB | sibling |
| `RoomT_#_jpg.zip` ; `OT_#_jpg.zip` and 9 more spellings | image archive | stills, zipped | Nikon D300s | 23 | 9.9 GB | name |
| `GMT #_#_D#s_NEF.zip` ; `GMT #_#_SLICE_D#s_NEF.zip` and 5 more spellings | image archive | original stills of the day (Nikon raw) | Nikon D300s | 15 | 9.1 GB | name |
| `GMT#_#_SLICE_DarkFrameCal_Proof_Sheet.pdf` ; `GMT#_#_SLICE_Proof_Sheet.pdf` and 12 more spellings | image archive | proof sheet of the day's stills (PDF) | Nikon D300s | 30 | 7.5 MB | name |
| `UT_#_#.zip` | video | videos of the test days, zipped | video camera | 5 | 7.6 GB | sibling |
| `DIMSG#M_#.mp#` | video | video named by an imagery id | video camera | 26 | 3.2 GB | name |
| `#b#e#.mp#` ; `#b#e#.zip` and 2 more spellings | video | video named by a hexadecimal id | video camera | 4 | 1.2 GB | name |
| `color_#_#.dat` | processed or analysed data | colour plane of one picture (binary array; 'color' read from the name) | Nikon D300s | 7301 | 3.4 GB | name |
| `T_#_ave.dat` ; `T_#_bg.dat` and 4 more spellings | processed or analysed data | temperature map of one picture (binary array; 'T' read as temperature) | Nikon D300s | 6869 | 3.2 GB | name |
| `fv_#.dat` | processed or analysed data | soot volume fraction map of one picture (binary array; 'fv' read as soot volume fraction) | Nikon D300s | 1456 | 681.7 MB | name |
| `CH_sig_#.dat` ; `CH_#_txt.dat` and 2 more spellings | processed or analysed data | CH* signal map of one picture (binary array; 'CH' read as CH*) | Nikon D300s | 628 | 359.1 MB | name |
| `#_#.pdf` ; `#CH#_#.#_#_#_#.pdf` | processed or analysed data | PDF named by a range of picture numbers, in a folder named pdf | - | 91 | 27.6 MB | unknown |
| `T_#.eps` ; `Fv_#.eps` and 32 more spellings | processed or analysed data | analysis script, constant or figure | - | 37 | 8.8 MB | name |
| `#.dat` ; `SLICE Flow Duct Cal - # mm Station x # mm Height, CENTER.zip` and 35 more spellings | calibration | flow or boundary-condition calibration file (one `<n>.dat` opened: a binary array) | - | 272 | 12.0 MB | name |
| `CoflowBC.xlsx` ; `D#s_BBcal.xlsx` and 1 more spellings | calibration | calibration workbook | - | 3 | 354.3 KB | name |
| `Fan_cal.xlsx` ; `SLICE fan calibration #.xls` | calibration | flow calibration | - | 2 | 171.8 KB | name |
| `Analysis_of_CH_concentration_and_flame_heat_release_rate_in_laminar_coflow_diffusion_flames_under_microgravity_and_normal_gravity.pdf` | science document | publication (folder is a citation) | - | 1 | 2.3 MB | name |
| `SLICE DoD.docx` ; `SLICE SRD #.pdf` and 3 more spellings | science document | science document | - | 5 | 1.1 MB | name |
| `PS-#_SLICE_Infosheet.pdf` | science document | information sheet | - | 1 | 609.5 KB | name |
| `#EAFH# Titler Board Modification.pdf` ; `#EAFH#-A Video Amplifier Board Modification.pdf` and 77 more spellings | engineering document | engineering drawing or drawing package | - | 79 | 18.6 MB | name |
| `SLICE Drawings, Overview, Sept #.pdf` ; `SLICE ED-# BASS#-slides for lunch talk.pdf` and 7 more spellings | engineering document | engineering document | - | 9 | 11.3 MB | name |
| `SLICE #.# SLICE Fan Colibration-MGUEMSGSLCEN#.pdf` ; `SLICE #.# SLICE Flash Card Exchange-MGUEMSGSLCEN#.pdf` and 7 more spellings | engineering document | crew procedure | - | 9 | 4.5 MB | name |
| `SLICE final report #.pdf` ; `SLICE-RPT-#_Post Flight Report-.pdf` | report | report (folder name says so) | - | 5 | 18.4 MB | name |
| `SPICE-RPT-#_Science Verif Report.doc` | report | report | - | 1 | 244.7 KB | name |
| `SLICE EP-# # ASGSR.pdf` ; `SLICE EP-# # ISS R and D.pdf` and 10 more spellings | presentation | presentation (folder name says so) | - | 13 | 25.6 MB | name |
| `#CH#_#.txt.docx` ; `#CH#_#_noBG.txt.docx` and 1 more spellings | metadata | text file saved as a document, named by fuel and day | - | 5 | 537.9 KB | unknown |
| `GMT#_#_SLICE_num.txt` ; `GMT#_#_SPICE_num.txt` | metadata | map from ISS photo number to camera file name | Nikon D300s | 11 | 95.8 KB | opened |
| every file of the PSI category "Investigation Metadata Files" | metadata | PSI investigation metadata archive | - | 1 | 8.7 KB | name |
| `Preflight vs Yale.xlsx` | other | not classified | - | 1 | 59.9 KB | unknown |

### PSI-107 SPICE

Smoke Point In Co-flow Experiment. Hardware: Microgravity Science Glovebox (MSG). Licence: CC0-1.0. 188 listed files, 6.9 GB.

**Naming.** The day is the folder (`Reduced Gravity/SPICE Day 045`, GMT days of 2009). Frame-grab archives
carry a test point number (`Downlink Frame Grabs TP 0091.zip`, 53 numbers). The experiment table lists flames by
GMT day and ignition time, not by test point number, so the two are not linked here. Ground calibration files
are under `Earth Gravity/Calibration Data, ...`.

| Pattern | Kind | What it is | Instrument | Files | Bytes | Sure |
|---|---|---|---|---:|---:|---|
| every file of the PSI category "Experimental table" | experiment table | PSI experiment table | - | 1 | 31.2 KB | name |
| `SPICE_GMT#_#_NEF.zip` ; `Focus Check - Flight Hdwr.nef` | image archive | original stills (Nikon raw) | Nikon still camera | 6 | 2.1 GB | name |
| `SPICE_GMT#_#_NikonD#_PictureBook.ppt` | image archive | picture book of the day's stills (slides) | Nikon still camera | 6 | 913.5 MB | name |
| `Downlink Frame Grabs <test>.zip` ; `Downlink Frame Grabs - <test>.zip` and 32 more spellings | image archive | frames grabbed from the downlinked video | video camera | 83 | 572.8 MB | name |
| `SPICE_GMT#_#_ContactSheet.pdf` ; `SPICE GMT# # Contact Sheets.zip` and 4 more spellings | image archive | contact sheet of the day's stills | Nikon still camera | 10 | 505.7 MB | name |
| `SPICE GMT# # jpegs.zip` ; `SPICE_GMT#_#_jpegs.zip` and 3 more spellings | image archive | stills, zipped | Nikon still camera | 7 | 355.4 MB | name |
| `GMT#_#.#.mp#` ; `SPICE_GMT#_Ign#.mp#` and 4 more spellings | video | video of the test session | video camera | 19 | 2.4 GB | name |
| `SPICE_GMT#_#_Movie Files.zip` | video | movies, zipped | video camera | 1 | 71.8 MB | name |
| `GMT#nikon.xls` ; `GMT#nikon_propy.xls` | processed or analysed data | workbook named after the still camera | Nikon still camera | 3 | 21.7 MB | unknown |
| `Ethylene assembled smoke hieght slide.  GMT # #      # # #.pptx` ; `SmokePointReview-#g-#.xlsx` and 2 more spellings | processed or analysed data | results workbook or slide | - | 4 | 2.2 MB | name |
| `SPICE Flow Duct Cal - # mm Station x # mm Height, CENTER.xls` ; `SPICE Flow Duct Cal - # mm Station x # mm Height, INSIDE.xls` and 13 more spellings | calibration | flow calibration file | - | 30 | 5.9 MB | name |
| `SPICE Image Organization Guide.docx` ; `SPICE_SRD.doc` | science document | science document | - | 2 | 1.3 MB | name |
| every file of the PSI category "Engineering Documents" | engineering document | engineering document | - | 4 | 5.4 MB | name |
| every file of the PSI category "Reports" | report | report | - | 3 | 3.0 MB | name |
| every file of the PSI category "Presentations" | presentation | presentation | - | 5 | 36.9 MB | name |
| every file of the PSI category "Reference Data" | reference data | reference data | - | 1 | 1.1 MB | name |
| every file of the PSI category "Investigation Metadata Files" | metadata | PSI investigation metadata archive | - | 1 | 9.3 KB | name |
| `#_SPICE_DSC_num.txt` ; `#_SPICE_num.txt` | metadata | note on the day's still downlink | Nikon still camera | 2 | 3.1 KB | opened |

### PSI-159 ACME CFI-G

Advanced Combustion via Microgravity Experiments / Cool Flames Investigation with Gases. Hardware: ACME, Combustion Integrated Rack (CIR). Licence: CC0-1.0. 3756 listed files, 2.1 TB.

**Naming.** The same test names (`21088A1`). A test point opened twice has a second set of files with `_1`
after the name (`21196M2_1_data.csv`); the catalogue keeps `21196M2` as the test and `_1` as the variant.

**Image archive names changed during the campaign:** `<test>_Images.zip`, then `Images_<test>.zip` and
`Images_Converted <test>.zip`, then `<test>_ACME.zip` and `<test>_ACME_Converted.zip` as in the other ACME
investigations. The camera of the `Images` archives is inferred from that sibling pattern.

**Folders.** `Reduced Gravity/<n>_<normal|inverse>_flames_<fuel>/<day>/<test>/`, six fuel groups. The folders of
day 21209 are spelled `21909...`; 120 of the 127 files that disagree with their folder do so for that reason, and
keep the name.

**Check against known ids.** Of the 206 train and validation test ids of the two splits, 205 are found in file
names with a sensor series; `21130A9` has no file.

| Pattern | Kind | What it is | Instrument | Files | Bytes | Sure |
|---|---|---|---|---:|---:|---|
| `# daily test point log.pdf` ; `# dail;y test point log.pdf` | experiment table | test point log of the day | - | 22 | 1.3 MB | name |
| every file of the PSI category "Experimental table" | experiment table | PSI experiment table | - | 2 | 61.6 KB | name |
| `as run<test>.txt` ; `as run<test>_#.txt` | script or settings | test point script parameters (key:value) | - | 338 | 692.8 KB | opened |
| `<test>_mfcSetpoints.csv` ; `<test>_#_mfcSetpoints.csv` and 1 more spellings | script or settings | command log of the three mass-flow-controller setpoints | mass flow controllers | 339 | 631.9 KB | opened |
| `<test>_TP.txt` ; `<test>_#_TP.txt` | script or settings | as-run stage timeline of the test point, with stamps (outcome-bearing) | - | 305 | 333.2 KB | opened |
| `<test>.txt` | script or settings | test point script parameters (key:value) | - | 1 | 2.1 KB | sibling |
| `<test>_data.csv` ; `<test>_#_data.csv` and 1 more spellings | sensor time series | analog channels at 125 Hz: flow readbacks, pressures, photomultipliers, thermocouples, radiometers | ACME data acquisition | 339 | 1.5 GB | opened |
| `as runAcmeCameraConfigure_#-#-#_#_#.#.#.txt` ; `AcmeCameraConfigure_FusedSilica_#.txt` | camera settings | camera configuration script of the day | ACME colour camera | 47 | 20.0 KB | name |
| `Images_Converted <test>.zip` ; `<test>_ACME_Converted.zip` and 12 more spellings | image archive | frames of one test converted to a viewable format | ACME colour camera | 313 | 954.0 GB | name |
| `<test>_INTNS_ALL.zip` ; `<test>_INTNS_ALLzip.zip` and 1 more spellings | image archive | all frames of one test | ACME intensified camera | 330 | 791.4 GB | name |
| `Images_<test>.zip` ; `Images_ <test>.zip` and 3 more spellings | image archive | frames of one test as downlinked | ACME colour camera | 253 | 192.3 GB | sibling |
| `<test>_ACME.zip` ; `<test>_#_ACME.zip` | image archive | frames of one test as downlinked | ACME colour camera | 79 | 49.7 GB | name |
| `<test>A_HiBMS.zip` | image archive | frames of one test | CIR HiBMs camera | 10 | 12.8 GB | name |
| `<test>A_IPSU#.zip` | image archive | frames of one test | CIR IPSU camera | 1 | 1.9 GB | name |
| `vlc-record-#-#-#-#h#m#s-udp____<test>.ts` ; `vlc-record-#-#-#-#h#m#s-udp____#A#.ts` and 6 more spellings | video | recording of the downlinked operations video | operations video downlink | 337 | 81.7 GB | name |
| `# flow traces etc.pdf` ; `# flow traces etc-#.pdf` | processed or analysed data | plots of the day's flow or velocity traces | - | 24 | 90.5 MB | name |
| `Final ACME Chamber Calculator - #-#-# to #-#-#.xlsx` | processed or analysed data | chamber atmosphere or flow calculator workbook | - | 1 | 929.7 KB | opened |
| every file of the PSI category "Engineering Documents" | engineering document | engineering document | - | 1 | 7.5 MB | name |
| every file of the PSI category "Reports" | report | report | - | 1 | 3.9 MB | name |
| `INTNS_<test>.mdipsug` ; `INTNS_<test>.mdipsug.ERRORS` and 1 more spellings | metadata | per-frame index of the archive (time stamp, exposure, gain) | ACME intensified camera | 306 | 117.7 MB | sibling |
| `ACME_<test>.mdipsug` ; `ACME_<test>_#.mdipsug` | metadata | per-frame index of the archive (time stamp, exposure, gain) | ACME colour camera | 305 | 28.3 MB | sibling |
| `# test sheets.pdf` ; `# tests sheets.pdf` | metadata | test sheets of the day (scan) | - | 22 | 27.8 MB | name |
| `# test parameter notes.pdf` ; `# test parameters notes.pdf` and 1 more spellings | metadata | test parameter notes of the day (scan) | - | 23 | 16.6 MB | name |
| `<test>_Comments.txt` ; `<test>_ImagesComments.txt` and 2 more spellings | metadata | operator notes on the image downlink of the test | - | 331 | 75.8 KB | sibling |
| every file of the PSI category "Investigation Metadata Files" | metadata | PSI investigation metadata archive | - | 1 | 8.4 KB | name |
| `ACME_<test>.next` | metadata | frame counter of the camera unit | - | 1 | 4 B | sibling |
| `#-#-#_#-#_ACME.log` | other | message log of the avionics software | - | 22 | 8.3 MB | name |
| every file of the PSI category "Representative Image(s)" | other | representative image of the investigation | - | 2 | 137.8 KB | name |

## 7. What could not be classified, and weak spots

**Unclassified (1 file, 59.9 KB).** `Preflight vs Yale.xlsx` (PSI-106): the name does not say what it compares.

**Confidence "unknown" (123 files, 1.1 GB, the unclassified one included):**

| Investigation | Pattern | Files | Why |
|---|---|---:|---|
| PSI-106 | `<n>_<m>.pdf` in folders named `..._pdf` | 91 | a range of picture numbers; filed as processed data without being opened |
| PSI-21 | `18257.pdf` | 9 rows of one file | a day number and nothing else |
| PSI-68 | `LCTF033.zip`, `BKGD260.zip`, `B341D04.zip` and 4 more | 7 | an archive named only by a point, without a camera letter |
| PSI-98, 99, 100 | `SAFFIRE-..._MergeResult-jpeg_....xlsx` | 4 | 8 to 21 MB workbooks under "Science Documents"; "merge result" of the image files, content not stated |
| PSI-107 | `GMT055nikon.xls`, `GMT052nikon_propy.xls`, `GMT058nikon.xls` | 3 | workbooks named after the still camera; filed as processed data |
| PSI-21 | `18214F1.zip` | 1 | 765 MB archive named only by the test; filed as image archive |
| PSI-25 | `BASS_GMT51_SAMPLE_223_TEST_M14.zip` | 1 | 254 MB archive named like the video of the burn |
| PSI-101 | `ISS Coagulation.doc` | 1 | a note in a run folder |
| PSI-106 | `100CH4_040.txt.docx` and 4 more | 5 | text files saved as documents, named by fuel and day |

**Weak spots that are not marked unknown:**

- **Day paperwork of ACME** (`<day> test sheets.pdf`, `<day> test parameter notes.pdf`, `<day> flow traces etc.pdf`,
  `<day> daily test point log.pdf`; 244 files). The kinds are read from the names; none was opened. The test
  sheets are filed as metadata, the test point logs as experiment tables, the traces as processed data.
- **SLICE arrays.** `T_`, `fv_` and `CH_` are read as temperature, soot volume fraction and CH*. One `<n>.dat`
  was opened, from a calibration folder, and only to see that it is a binary array with a header. 16,254
  computed arrays and 6,527 picture arrays rest on that and on the folder name "OMA Format".
- **Converted images.** `_ACME_Converted.zip` is 4.1 TB, the largest pattern of the archive. "Converted to a
  viewable format" is read from the name; the format was not checked.
- **`_INTNS.zip` against `_INTNS_ALL.zip`.** Both exist for many tests. The catalogue calls the first a
  selection and the second all frames, from the names.
- **Video archives.** `vlc-record-....zip` beside an `.mp4` of the same name is taken to hold the original
  transport stream, because E-FIELD Flames names such archives `_ts.zip`.
- **SAME-R sample files.** The slide decks and archives named by run and sample code are filed as image
  archives because their folder is "Videos, Images" and sibling files are named "images".
- **Auxiliary ACME runs** (`RAD`, `MFCA`, `Fiber`): read from the tokens.
- **Documents by category.** A document category decides the kind even when the file is something else
  (three outreach videos under "Presentations", a photo archive under "Engineering Documents").
- **A test id from the folder** (522 files) is as good as the filing: a file put in the wrong folder gets
  the wrong test.

## 8. What was opened

Fetched for this catalogue with `ignis sources fetch` after a dry run: 38 files, 1.6 MB in all, no archive,
image or video. They are in `runs/dev/data_all/raw/samples/PSI-<id>/`; the fetch step appended their records to
`runs/dev/data_all/manifests/PSI-<id>.jsonl`.

| Investigation | File | What it settled |
|---|---|---|
| PSI-23 | `ACME_20190V1.mdipsug` | a per-frame index: time stamp, file name, exposure, gain, frame rate |
| PSI-22 | `20180416_224527.774_ACME_18106J1.metadata` | the same index under the earlier extension |
| PSI-23 | `ACME_20013A1.next` | four bytes: a frame counter |
| PSI-23 | `moduleDataACD_02_20200613_0300.asc` | a gas chromatograph run: header, three channels at 100 Hz |
| PSI-10 | `GC_Coverter_2021Dec07_193617_20211207_0001.log` | the log of the converter that writes the `.asc` files |
| PSI-23 | `2020-07-09_11-15_IPSUG.log` | the message log of the camera unit |
| PSI-22 | `18073A3_VoltageSweep.txt` | the script of the test point, key:value |
| PSI-20 | `19039G2_BRE.txt` | the same |
| PSI-21 | `18022F1_ACME_CAM3_Exposures.txt` | camera telemetry against time |
| PSI-21 | `18026A1_ACME_TP_Analysis_FCU_PTandMFC_Readings.txt` | rack pressure transducers and flow controllers against time |
| PSI-20 | `ACME_non-imaging_data_file_description.pdf` (0.38 MB) | the columns of `_data.csv`, the message log and the command log. A document, not a tiny text file: the one fetch above the brief's "few KB" |
| PSI-68 | `E066R02_Mixture.txt` | one row: date and the chamber gas composition in percent |
| PSI-68 | `D033E01_Cool_Flames.txt` | cool-flame trigger settings |
| PSI-68 | `E169M01_HIBMS_`, `_LLLUV_`, `_LCTF_`, `_Color_`, `_MDCA_Color_Camera_Settings.txt`; `H350N03_HFR_`, `_LLLIR_`; `D349D06_LLL_` | which camera unit each settings file belongs to (IPSU1 to IPSU4) |
| PSI-68 | `LCTF101_LCTF_Wavelengths.txt`, `E169M01_Radiometer_Readings.txt`, `E169M01.txt`, `E169M01_MDCA_Testpoint_Settings.txt` | filter wavelengths, radiometers at 100 Hz, the timeline, the insert settings |
| PSI-39 | `F177D04_Filter_Positions.txt`, `F086A04_INTNS_Camera_Settings.txt`, `F086A05_HIBMS_CFI_Camera_Settings.txt`, `..._INTNS_068A03E_00000.meta`, `FLEX-1261_CoolFlameData.txt` (header only) | sequence telemetry, the units of the two cameras, the per-frame camera state, the columns of the cool-flame series |
| PSI-106 | `1.dat` (from a calibration folder), `pic_info_040.csv`, `GMT2012_072_SLICE_num.txt`, `SLICE_vel.csv` | a binary array with a header; a per-picture exposure record; the photo-number map; a velocity table |
| PSI-47 | `DTrk0002.txt`, `PTrk0002.txt` | instrument logs |
| PSI-107 | `045_SPICE_DSC_num.txt` | a note on the day's still downlink |
| PSI-101 | `HKPData_gmt250_1.csv` | the housekeeping record of a run |

Already on disk from earlier analyses of this lane and used here: the Flame Design text files
(`docs/wsp/feedback/flame_design_transfer_feasibility.md`), the CFI and FLEX-2 samples
(`docs/wsp/feedback/cfi_flex2_transfer_feasibility.md`), and the classification of the FLEX, CFI-G and BASS
files in `docs/wsp/dossier/02_data_classification.md` (sections 2.3 to 2.5). Where "opened" rests on those
documents and not on a file opened for this catalogue, the file kinds are: the 125 Hz data, setpoints,
timeline, script and comments of PSI-10 and PSI-159; the droplet, UV and colour-camera series and the test
sheets of PSI-69; the stills, proof sheets and videos of PSI-25 and the stills of PSI-26. Two descriptive
documents already on disk were read for the naming: `SLICE DoD.docx` and `ACME_BRE_Readme.docx`.

## 9. Two findings that change earlier notes

- **FLEX-2 fuel and atmosphere.** `cfi_flex2_transfer_feasibility.md` reports that no file gives the fuel or
  the gas composition of a FLEX-2 test. The folder of each test point names the fuel (18 test points under
  "SurrogateFuel 2013 - Heptane"), and 124 `_Mixture.txt` files of 215 to 500 bytes give the chamber
  composition at a stated time. Which reading applies to which burn was not worked out here.
- **Flat listings lose the folder.** Fuel, burner, polarity, investigator, material, sample or day are folder
  names in 18 of the 19 investigations. A catalogue built from the flat listing alone cannot tell them.

## 10. Rerun and check

```bash
python tools/wsp/atlas/classify_files.py            # writes files.jsonl and files_summary.json
python tools/wsp/atlas/classify_files.py --tables   # also files_tables.md, the tables of section 6
python tools/wsp/atlas/check_files.py               # one row per listed file, bytes, closed lists, two identical runs
```

Both scripts run from the repository root, read only the listings and the raw listing trees, and take about
five seconds. The check passed on 14 November 2026: 66,501 rows, 13,496,418,513,932 bytes, 60,800 distinct
files, two fresh runs byte-identical.
