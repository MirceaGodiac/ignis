# SLICE (PSI-106): which burner tube did tests 63 to 67 and 63A of GMT 061 use?

> Delegated data agent (claude-opus-5-5), 15 November 2026, at the coordinator's request after the physicist's
> study (`docs/wsp/research/phys_slice_study.md`, day 061). Answered from conditions and documents only: no
> outcome (marks, notes, blow-off labels) was used. Test days GMT 044, 051 and 083 stay unopened.
> Builder: `tools/wsp/ext/slice_cases_v2.py`. Version 1 (`slice_cases.py`, `cases_*.jsonl`) is unchanged.

## Answer

**The documents say 2.1 mm, not 0.4 mm and not 3.2 mm.** The log's 0.4 mm for these six flames is a copied-down
cell. I am confident of that (high confidence). That the tube was 2.1 mm rests on one independent record, the
investigators' picture table (moderate to high confidence). No document names 3.2 mm for GMT 061. The two
readings, 2.1 mm from the documents and 3.2 mm from the physicist's fit to the blow-off boundary, give jet
velocities a factor (3.2/2.1)^2 = 2.3 apart. One look at a still of these flames would settle it (see "What would
settle it").

## What each source says

| Source | Tube for GMT 061 tests 63-67, 63A | Fuel for the same lines | Independent of the log? |
|---|---|---|---|
| Summary log, `SLICE-RPT-002_Post Flight Report-.pdf`, pdf pages 43-44 (crew and console entries) | "0.4", written on each test line | "100% Methane" | it is the log |
| Picture table `Raw Data/pic_info_ISS.txt.docx` (one row per still: test point, burner, fuel-type code, folder `061_UT061_spice_images_forHSV`, FUEL/AIR/FAN displays, time) | **2.1** on every picture of tests 63, 64, 65, 66 and 67 (14:22 to 14:49 GMT). Its pictures of 14:47:35 to 14:49:11 are labelled "67", but their settings (FUEL 120, FAN stepping from 55 to 134) and time match the log's 63A (GMT 61/1447, FUEL 120, FAN 55 to 135). | code 3 = 40 % CH4 | yes: on this day it disagrees with the log on both the tube and the fuel |
| Bottle table and timeline, final report 2017, Tables 4 and 6; daily status report of 1 March 2012 (post-flight report, p. 16-17) | not stated | 40 % CH4, second and last bottle | yes |
| Crew procedure 2.005 (`SLICE 2.005 SLICE Operations-MGUEMSGSLCEN005.pdf`) | "per SLICE Test Matrix"; the matrix itself is not in the archive | - | - |
| On-orbit crew briefing (`SLICE IP-F`) | shows only an example page of the test matrix (2009 SPICE ethylene points), not day 061 | - | - |
| Hardware documents (final report 2017 Table 5; post-flight report section 3) | five tubes flown: 0.4, 0.8, 1.6, 2.1, 3.2 mm; no installation order per day | - | - |
| Image and video file names and metadata | `GMT2012_061_SLICE_Proof_Sheet.pdf`, `GMT 2012_061_D300s_NEF.zip`, `GMT2012_061_SLICE_num.txt` (ISS photo number to camera file name), the analysis folder `061_UT061_spice_images_forHSV`: no tube in any name; no video file of GMT 061 is named by tube | - | - |
| Giassi et al., Combust. Flame 167 (2016), Table 1 | a microgravity sequence "40 % CH4, 3.2 mm, fuel 17-89 cm/s, coflow 17 cm/s", day not given | 40 % CH4 | yes, but the sequence is most likely from GMT 044: that paper uses CH* maps, which exist on disk only for the 044, 048 and 051 folders, and the picture table's inventory lists the 044 folder with a 3.2 mm tube and fuel code 3. It is not evidence for GMT 061. |

The picture table's fuel code was read off the days whose bottle is not in doubt: 1 = 100 % CH4 (040, 058, 072),
2 = 70 % CH4 (048, 065, 079), 3 = 40 % CH4 (044, 061), 5 = 20 % C2H4 (075), 6 = C2H6 (069), 7 = C3H8 (055).
It agrees with the bottle table on every day, including 061, where the log is wrong.

## How version 1 derives the tube, and why it fails here

`slice_cases.py` takes the "Burner tube ID" cell written beside each test number in the log. Only when a test
line has no tube cell does it carry the previous test's tube forward. On GMT 040 that happens for tests 2 to 4,
and the picture table confirms the carried 1.6 mm there. On GMT 061 the cell is present on every line, so
version 1 did nothing wrong mechanically. The cells themselves are wrong: "100% / Methane / 0.4" on 63 to 67 and
63A repeats, cell for cell, the last entries of GMT 058 (tests 21F to 21H: "100%", "0.4", "Methane"). The day's
spreadsheet was evidently started by copying the previous day's last line, and two of its three cells (fuel and
dilution) are provably wrong on this day. The same day's tests 168 and 169 carry "1.6", and the picture table
agrees with that.

The same failure exists on another train day. On GMT 058, tests 1 and 2 are logged on "0.8", copied from tests 7
and 8 just above them. The picture table says 1.6 mm, at identical times and settings. This agrees with the
test-point numbers, which are matrix identifiers: on GMT 040, test points 1 and 2 used the same FUEL displays (98,
139/140) on 1.6 mm in both sources, and the proof sheet `GMT2012_058_SLICE_TP 7, 8, 1, 2_Proof_Sheet.pdf` groups
these as matrix points. A letter suffix does not carry the tube: on GMT 072, test points 22 to 24 are on 3.2 mm
and 22A to 24A on 2.1 mm.

Where both sources hold a flame, they agree on 25 of 32 label-matched train and validation flames. The 7
disagreements are exactly these two copy-down cases: 058 tests 1 and 2, and 061 tests 63 to 67. Version 2 also
matched further flames by settings and time order (94 lines, 64 train and 30 validation). All of them agree with
the log except 63A of GMT 061 (7 lines).

The rule has to be general, so it is: **take the tube of a flame from the picture table where the table holds
that flame, and from the log otherwise.** A flame is matched by its test-point label first. Failing that, it is
matched by fuel code, time order between its matched neighbours, every logged (FUEL, FAN) setting appearing among
the pictures, and the logged time within 180 s. The run must be unique. Nothing in the rule depends on an
outcome. The same copy-down can occur on any day, test days included. The picture table has folders for 044, 051
and 083/084 as well, so the rule applies there unchanged when the evaluator builds test cases. Version 2 skips
those folders by name, because version 1 holds no test case.

## Version 2

`python tools/wsp/ext/slice_cases_v2.py` writes, beside version 1 in `runs/dev/data_all/work/transfer_slice/`:
- `cases_train_v2.jsonl` and `cases_validation_inputs_v2.jsonl`: the same cases. `burner_nominal_mm`,
  `burner_id_mm` and `u_jet_m_s` are replaced where the rule changes them, and every case gets `v2.tube_source`
  (and `v2.tube_log_mm` when changed).
- `cases_validation_outcomes_v2.jsonl`: a byte copy of version 1, since outcomes do not depend on the tube.
- `tube_changes_v2.json`: every changed line.

**Lines whose tube changes: 37, all train; no validation line changes.**

| Day | Test | Lines | Log tube | v2 tube | Matched by |
|---|---|---|---|---|---|
| 058 | 1 | 4 (`PSI-106:058:p40:1:1` to `:4`) | 0.8 | 1.6 | label |
| 058 | 2 | 4 (`PSI-106:058:p40:2:1` to `:4`) | 0.8 | 1.6 | label |
| 061 | 63 | 5 (`PSI-106:061:p43:63:1` to `:5`) | 0.4 | 2.1 | label |
| 061 | 64 | 5 (`PSI-106:061:p43:64:1` to `:5`) | 0.4 | 2.1 | label |
| 061 | 65 | 6 (`PSI-106:061:p43:65:1` to `:6`) | 0.4 | 2.1 | label |
| 061 | 66 | 4 (`PSI-106:061:p43:66:1` to `:4`) | 0.4 | 2.1 | label |
| 061 | 67 | 2 (`PSI-106:061:p43:67:1` to `:2`) | 0.4 | 2.1 | label |
| 061 | 63A | 7 (`PSI-106:061:p44:63A:1` to `:7`) | 0.4 | 2.1 | settings, fuel code and time order (pictures labelled "67") |

On these lines the jet velocity falls by (0.413/2.140)^2 = 0.037 on GMT 061, for example from 7.6 to 0.28 m/s
on test 63, and by (0.768/1.594)^2 = 0.23 on GMT 058. The fuel of GMT 061 was already 40 % CH4 in version 1.
The picture table confirms it.

Version 2 does **not** take the physicist's 3.2 mm. If 3.2 mm is adopted instead, the change is the same 29 lines
of GMT 061, with tube 3.2 and velocities a further factor 2.3 lower. It would then rest on the outcome-based fit
alone, and it should be named data-informed.

## What would settle it

A still of these flames shows the tube tip at a known scale (about 17 px/mm on the SLICE images). A 2.1 mm and a
3.2 mm tube differ by about 19 px in inner diameter, and more in outer diameter. That needs either
`GMT2012_061_SLICE_Proof_Sheet.pdf` (under 1 MB, not on disk) or `GMT 2012_061_D300s_NEF.zip` (1.94 GB). Neither
was downloaded: a download needs the owner's go-ahead.

## Disclosure

To inventory the picture table I printed, for every folder including the test-day folders 044, 051 and 083/084,
the count of pictures per (burner, fuel code). Those are conditions, not outcomes. No row of a test-day folder
was printed. No test-day log page was opened.

## Addendum, 15 November: the proof sheets (fetched with the coordinator's go-ahead)

Fetched into `runs/dev/data_all/raw/PSI-106/Raw Data/_proof_sheets_gen7/`. Each file has a line appended to
`raw/MANIFEST.jsonl` with its sha256, and every size equals the listing. All are from train days:
`GMT2012_061_SLICE_Proof_Sheet.pdf` (623 KB, 18 pages, 210 stills, 14:22 to 16:10 GMT),
`GMT2012_058_SLICE_TP 7, 8, 1, 2_Proof_Sheet.pdf` (157 KB), and as references `GMT2012_040_SLICE_Data_Proof_Sheet.pdf`
(544 KB), `GMT2012_040_SLICE_Scale_Proof_Sheet.pdf` (123 KB) and `GMT2012_069_SLICE_Proof_Sheet.pdf` (363 KB).

**What the stills show.** Each thumbnail is the full camera frame, about 350 px wide where the original is 4288 px
wide: 12x smaller, so about 1.4 px/mm against about 17 px/mm in the original. The flame photographs are taken in
the dark (ISO 200 to 800, 1/10 to 1/200 s). Only the flame itself is visible: a small blue (GMT 061) or green
(GMT 058) flame near the right edge of an otherwise black frame. The burner tube, the duct walls and every other
fixed part are not lit and not visible on any flame still of the 061 or 058 sheets, nor on the reference days.
The tube, with the ruler beside it, is visible only on the GMT 040 scale sheet, which was shot under cabin light
with long exposures (0.4 to 25 s) before the flames.

**Measurement.** None was possible. There is no visible tube tip to measure against a scale. Even if there were,
the 1.1 mm difference between 2.1 and 3.2 mm would be about 1.5 thumbnail pixels. Measuring the flame instead (base
width, base position) is not admissible: on these lifted and near-blow-off flames it is the outcome.

**Conclusion: cannot tell from the proof sheets**, for GMT 061 tests 63 to 67 and 63A (2.1 against 3.2 mm) and for
GMT 058 tests 1 and 2 (0.8 against 1.6 mm). The raw stills (`GMT 2012_061_D300s_NEF.zip`, 1.94 GB) would not
settle it either. They are the same dark exposures at full resolution, and no 061 still was shot under cabin
light: all 210 stills on the sheet are flame stills between 14:22 and 16:10. So I did not ask to fetch them.
Version 2 (2.1 mm, from the picture table) stands as the documented tube, with the confidence given above. The
3.2 mm reading remains an outcome-based inference.

## Addendum 2, 15 November: version 2 in the final run

`tools/wsp/ext/slice_cases_v2.py` now applies the same rule to the test days, but only when the final run asks it
to. The rule: a flame's tube comes from the picture table where the table holds that flame, otherwise from the log.

- **The flag.** Pass `main(final_run=True)` from Python, or `--final-run` on the command line. Without the flag,
  only the train and validation folders are mapped (`FOLDER_DAY`). With it, the test-day folders are mapped too
  (`TEST_FOLDER_DAY`): `00_UT044_ForHSV_Splice_DSC957` -> 044, `UT051_Splice_DSC957` -> 051,
  `083_forHSV_splice_ut083_DSC957` and `084_ForHSV_splice_ut083c_DSC957` -> 083. The second 083 folder is the "UT083c"
  set of the same day.
- **Fail closed.**
  - If the input cases contain a test day (044, 051, 083) and the flag is not given, the builder stops with
    `refused: cases of test days [...] given without final_run=True`. It never quietly keeps the log's tube there.
  - If the flag is given and the picture table has no folder for a requested test day, it stops with
    `refused: the picture table pic_info_ISS.txt.docx has no folder for test days [...]`. That check reads only the
    folder column.
- **Columns read, on every day, test days included.** Only these columns of `pic_info_ISS.txt.docx`: 0 "Test Point",
  1 "Burner", 2 "Fuel type", 3 "folder", 5 "Fuel" (FUEL display), 7 "Fan" (FAN display), 9 "Time". The header is
  asserted at load. A row is kept only if its folder maps to a day present in the input cases, and only then are
  the other six columns read.
- **Never read.** "Filename Prefix", "Air", "File #", "Exp time", "F #", "Rmax", "Gmax", "Bmax", "Rad", "Frame #",
  "Saturation". The colour maxima, the radiometer and the saturation flag are observations of the flame.
- **Checks run today, none on test data.**
  - `python -P tools/wsp/ext/slice_cases_v2.py --check-flag-noop` builds the train and validation cases with and
    without the flag, in memory. Result: identical, with 37 changed lines either way. Because rows are filtered by
    the input's days, this check reads no test-day row.
  - Rebuilding version 2 without the flag reproduced all four `*_v2` files byte for byte (sha256 checked).
  - The two refusals were exercised with a hand-made fixture table and a hand-made day list, not the real table.
  - The builder was **not** run on the test days.
- **What the final-run tool must change (not edited here; it is the coordinator's file).** In
  `tools/wsp/final/run_final_gen7.py`, `build()` for slice with `coding == "v2"`:
  - replace the refusal that tests `M2.FOLDER_DAY` with the builder's own check, i.e. drop those lines;
  - call `M2.main(final_run=(mode == "final"))` in place of `M2.main()`.

  `check_coverage` then refuses on its own if a test-day folder is missing. The rehearsal (validation days) keeps
  `final_run=False` and is unchanged.
