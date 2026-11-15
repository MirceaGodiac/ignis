# Flame Design (PSI-10): can the frozen burner-flame models be run on it?

> Written by a delegated analysis agent (claude-fable-5-2, high effort) on 14 November 2026 for the fifth generation;
> nothing fitted. It reads the file listing, the two experiment tables and 22 small text files of ACME Flame Design, an investigation outside every partition; it scores no model and reports no outcome of a test. The author has not opened the trial endpoints file it names.
> It is a proposal to the author, not a model: no number here is a weight until the author has rechecked it
> on train and typed it. What was rechecked is recorded in `docs/wsp/dossier/03_datasets_and_measurements.md`.
> Paths under `runs/dev/` are local working folders that are not in git.

## Flame Design (PSI-10) transfer test: feasibility report

The transfer is feasible for the measurement side, but the unchanged models must refuse every ethylene test. Only 17 propane normal-flame tests (one session) are inside their domain as written. Nothing was scored and no model or existing file was changed.

### (a) Raw files and how they map to tests

- **Naming:** test names are `YYDDD` + letter + number (e.g. `19115A1`), and every raw file is prefixed `PSI-10_Raw Data_`.
- **Sessions:** normal flames ran on 13 days in 2019 (19115–19206) plus 21301; inverse flames on 8 days (21328–22035). No test name overlaps the CFI-G splits.
- **Small text files per test** (same formats as CFI-G):

| File | Content | Size | Tests with it (of 327) |
|---|---|---|---|
| `<T>_data.csv` | 35 channels at 125 Hz on the script clock: 3 MFC readbacks, PMT1–3, radiometers, thermocouples, pressures | 1.9–41 MB, median about 3.8 MB | 307 |
| `<T>_mfcSetpoints.csv` | command log of the three MFC setpoints | 2–34 KB | 307 |
| `<T>_TP.txt` | stage stamps at 1 s, "Extinction detected", "MFCs off", abort "Reason" | about 1.1 KB | 304 (4 via alias) |
| `as run<T>.txt`, or `<T>.txt` on 19115/19123 | script parameters, key:value | about 2.1 KB | 307 |
| `<T>_Comments.txt` | image downlink notes only | under 3 KB | 303 |

- **Tests without a data file:** 20 in total — 16 on day 19206, plus 19137C1, 21301J1, 21301J2 and 22042M1.
- **TP alias:** the TP files of 19189K1–K3 and P1 appear to be stored as `19189JK1`–`JK3` and `JP1`. I inferred this from the names; the script flags a mismatch with the name written inside the file.
- **Format differences by era:**
  - 2019 radiometers are named `Radiometer+60/+30/Center/-30/-60`; 2021–22 use `Radiometer3±..Lo/HiGain`.
  - 2019 scripts spell the stage `constantFlowTimeSec`. The existing `acme_script.py` requires `constantFlowTime1Sec`, so it would skip those tests.
  - The earliest 2019 script has no burner-limit key; later ones give 500 or 450 °C.
- **Everything else is bulk:** 1,183 zip files (2.1 TB), 317 `.ts` videos (75 GB), camera `.mdipsug` files, PDFs. None of it was touched.
- **Downloaded:** 22 text files, 20.7 MB, each after a dry run. That is 10 for format inspection and 12 more (data, setpoints, TP, script) for three further trial tests, so above the "about ten" you set.

### (b) Column-to-input mapping

| Model key | Normal table | Inverse table | Status |
|---|---|---|---|
| `pressure` | `pinit (atm)` × 101325 | same | present, all rows |
| `injected_mass_flow` | `mfuel (mg/s)` (fuel alone) | `mO2 (mg/s)` (O2 alone) | present for constant-flow rows |
| `x_o2_role` | `XO2 (init)` (ambient) | `XO2` (injected) | present |
| `x_fuel_role` | `Xfuel` (injected) | `XC2H4 initial` (ambient) | present |
| `zst` | `mean Zst` | `mean Zst` | present, but recompute (see below) |
| `tad` | `Tad (K)` | `Tad (K)` | present, table only |
| `fuel_species` | `Fuel`: C2H4 on 187 rows, C3H8 on 20 | no column; header says C2H4 on all 120 | present, but ethylene is not a model fuel |
| `programmed_stop` (optional) | script file | script file | derivable; both spellings read |

- **No constant condition:** 60 normal rows (RAMP/RAMPED) and 11 inverse rows (9 RAMP, 2 LOW O2). The models assume constant flow, so these cannot be represented.
- **Leak in the table:** `mean Zst` is computed at the mean of the initial and final composition. It matches my closed-form Zst at the mean to within 0.15 %, but differs from the initial-composition value by up to 1 % (normal) and 3.9 % (inverse). That is after-test information, so the script recomputes Zst from the initial composition. `XO2 (final)`, `XC2H4 final`, `pfinal` and `pmean` are never read.
- **Tad cannot be recomputed here:** no equilibrium solver is installed, so it is taken from the table and presumably carries the same small leak. The 20 propane rows have no after-test columns, and their Zst matches the initial value.
- **Declared domain:** all 147 constant-flow normal rows are inside. Of 109 constant-flow inverse rows, 9 are outside (5 on `c_zst`, 4 on `c_xf`).

### (c) Ethylene is outside the models' domain

- **How fuel enters:** both forwards take it only as two indicators, ethane and butane, with propane as the reference. In `flame3.rs` the indicators also pick the fuel molar mass for normal flames.
- **Refusal:** the records store only the `fuel_species` text. Any species other than ethane, propane or butane is refused at the condition gate, and the committee refuses when any member does.
- **Not executed:** this is from reading `features()` in `flame.rs` and `flame3.rs` and the committee notes; I did not run a forward.
- **Only way to get a forecast unchanged:** write stored indicators into the prefix, which the code lets win over the text. That runs ethylene as a surrogate fuel: ethane is closest (C2, 30.07 against 28.05 g/mol), propane is 57 % off in molar mass. It is a declared extrapolation, not an in-domain application.
- **In-domain subset:** 17 propane normal tests in session 21301, at XO2 0.40 and 0.93 atm. Train propane normal flames were at 2–3 atm and XO2 about 0.21, a combination the card already distrusts.

### (d) Endpoints

Method, by analogy with CFI-G, in `D:\dev\SpaceApps-Ardealu\tools\wsp\ext\flame_design.py`:

- **Onset:** radiometer sum rises 0.15 V (or 8 sd) above its pre-ignition level near the TP "Ignition and retraction" stamp.
- **Flame end:** PMT sum returns to within 0.05 V of its armed dark level for 0.5 s. The CFI-G absolute 0.02 V rule fails here: the PMTs idle at about 4.5 V until armed, and one channel idles near 0.13 V.
- **Flow change:** first commanded departure from the test setpoints after the constant-flow stage begins (stop, post-extinction change or ramp).
- **Label:** extinction if the flame end precedes the flow change by more than 0.5 s, otherwise operator termination at the flow change.
- **Cross-checks:** the TP file's own account, and the steepest radiometer fall.

Trial on five tests chosen by conditions and era (19115A1, 19182N1, 21301J3, 21349A1, 22035J6):

- All five ran with status ok and resolved endpoints.
- Sensor and script labels agreed on 5 of 5, and both label branches were exercised.
- The three end markers agreed within about 0.5 s.
- Outcomes for these five are in `D:\dev\SpaceApps-Ardealu\runs\dev\data_all\work\flame_design\endpoints_trial.csv`; the author should not open it. The console prints counts only.

Two things that may not transfer:

- Onset fell 0.4–2.6 s after the stamp in the trial; the CFI-G median was 2.4 s.
- Where the command log shows a scripted stop, it came about 2 s later than the script's stage sum. The models' typed `script.onset_s = 0.4` may therefore misplace the planned stop.

### (e) Proposed inclusion rule and metric

Inclusion, from conditions and the file listing only (flags are in `D:\dev\SpaceApps-Ardealu\runs\dev\data_all\work\flame_design\conditions.csv`):

1. The row is constant-flow: all inputs numeric.
2. All six features are inside the release's declared domain, using `pinit`, initial compositions and initial-composition Zst.
3. `_data.csv` and `_TP.txt` exist in the listing.

| | Normal | Inverse |
|---|---|---|
| Included | 132 (13 sessions) | 100 (8 sessions) |
| Excluded: ramp / low O2 | 60 | 11 |
| Excluded: outside domain | 0 | 9 |
| Excluded: no data file | 12 | 0 |
| Excluded: no TP file | 3 | 0 |

Arms, to be fixed before unblinding:

- **A, unchanged:** the 17 propane normal tests, plus the refusal count for ethylene.
- **B, surrogate fuel:** ethylene run as ethane (primary), with propane and butane as sensitivity. 115 normal and 100 inverse tests.
- Each arm is run without the planned stop (primary) and with it.

Metric, the same as the models' development metric:

- Cases are (test, cutoff) at 1, 3 and 5 s after onset with the flame still burning.
- Integrated Brier score over 6–120 s, separately for extinction and operator termination.
- Compared with the frozen CFI-G train incidence curve as baseline.
- Intervals by bootstrap over sessions. Arm A is a single session, so no interval is informative there.

### (f) Remaining work and download

- **Download:** 912 files, 924 MB for the 232 included tests (dry-run plan only, nothing fetched). All 307 data files would be 1.22 GB. Download time was not measured.
- **Endpoint run:** a few minutes.
- **Engineering, roughly one to two working days:**
  - Convert `conditions.csv` and the endpoints into a private records root (experiments, endpoints, a split) so the frozen releases can be run.
  - Validate the endpoint script on CFI-G train files, where the table burn time and label are known. The script is currently hard-wired to PSI-10 names.
  - Optionally compute Tad at the initial composition.
  - Write the registration (arms, baseline, and whether the five trial tests stay in).

### (g) Not verified

- That Tad uses the mean composition (inferred from Zst only), and whether the CFI-G records' own Zst and Tad are initial or mean.
- Table flows against the commanded setpoints: MFC gas codes and conversion to mg/s were not checked for PSI-10.
- Channel names on days other than the four inspected. Unknown columns give a status, not a wrong value.
- PMT behaviour on dim flames; CFI-G had cases where the PMT went dark early.
- The 19189 TP alias.
- Whether the burner limit setting per session matches what the models assume.
