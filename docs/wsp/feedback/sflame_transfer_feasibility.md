# s-Flame (PSI-23): can the burner-flame models be run on it?

> Written by a scoring agent (claude-opus-5-5) on 15 November 2026. Feasibility only: **no outcome of any s-Flame test
> was computed or read.** No burn time, extinction stamp or endpoint run, and no TP file (its stamps include
> "Extinction detected"), Comments file, log, setpoint log, image, spreadsheet or video was opened.
> Read: the file names; the condition columns of the experiment table (`Flames Ignited` and `Notes` were not used);
> for the tests of the train and validation days only, the script parameter files, the two header lines of
> `<T>_data.csv` and its first data row (chamber pressure at the start of the record, before the flame).
> Nothing of the three test days, or of day 19245 (not in the split file, so held), was opened.
> Tool: `tools/wsp/ext/sflame_feasibility.py` (new); counts in `runs/dev/data_all/work/transfer_acme/sflame/feasibility.json` (local).

**In short.**

- The unchanged models would refuse every s-Flame test, for two independent reasons. No fuel is ethane, propane or n-butane, and no adiabatic flame temperature is tabulated.
- The closest analogue of the Flame Design arm B covers 29 tests: ethylene under ethane's indicators, at constant flow, on train and validation days. It needs Tad computed at the initial composition, and 8 of those tests use a helium diluent the models do not represent.
- The endpoint method's channels exist under the same names in every data file.

## (a) Files and how they map to tests

- **Naming.** Test names are `YYDDD` + letter + number (e.g. `19254A1`), as in Flame Design; every raw file is prefixed `PSI-23_Raw Data_`.
- **On disk.** 1,798 raw files (about 0.9 GB). The listing also has 1,974 zip files (771 GB) and 286 `.mp4` videos (9.3 GB), which were not downloaded.
- **Test names by frozen partition:** 168 on train days, 61 on validation days, 60 on test days (counted from file names only).
- **Opened, train and validation days only:** 229 tests.

| File | Content | Tests with it (of 229) |
|---|---|---|
| `<T>_data.csv` | sensor channels, same header format as Flame Design; 1.8–4.8 MB, median 2.6 MB | 228 (one test on 20013 has none) |
| `<T>_mfcSetpoints.csv` | command log of the three MFC setpoints | 228 |
| `<T>_TP.txt` | stage stamps (not opened) | 209 (17 tests on 20190 and one each on 20013, 20024 and 20164 have none) |
| `as run<T>.txt` | script parameters, key:value | 223 |
| `<T>.txt` | the planned script; on the one test compared it differs from "as run" (constant-flow time 30 s against 60 s) | 90 |
| `<T>_Comments.txt` | image notes (not opened) | 216 |

- **No script file:** 6 tests (4 on 20190, 1 on 20029, 1 on 20160).
- **Other kinds:** day 19254 also has `moduleDataACD` `.asc` files and per-test ACME/IPSUG logs, and two days (validation day 19256 and test day 20176) have a second data file `_<n>_data.csv`. None was opened.
- **Script format.** It is the 2019 one: a single stage `constantFlowTimeSec` (45 s on 114 constant-flow tests, 60 s on 12) and ramp keys `numberOfRampHolds`, `rampSizeFuelSlpm`, `rampSizeN2Slpm`.
  - `flame_design.py`'s stage sum reads it.
  - `acme_script.py` requires `constantFlowTime1Sec`, so it would skip every test.
- **Burner limit.** `burnerTemperatureLimitC` is 250 °C on every script that gives it (the earliest scripts have no such key). Flame Design gave 450 or 500 °C.

## (b) Sessions, dates, fuels and diluents (experiment table, condition columns)

| Day | Partition | Date (GMT) | Atmosphere (nominal) | Fuel bottle | Tests opened |
|---|---|---|---|---|---|
| 19254 | train | 2019-09-11 | ~21 % O2, N2 | 50 % C2H4 in N2 | 15 |
| 19256 | validation | 2019-09-13 | ~21 % O2, N2 | 50 % C2H4 in N2 | 16 |
| 19304 | train | 2019-10-31 | ~21 % O2, N2 | 45 % CH4 in N2 | 9 |
| 19308 | validation | 2019-11-04 | ~30 % O2, N2 | 45 % CH4 in N2 | 22 |
| 19352 | train | 2019-12-18 | ~21 % O2, N2 | 100 % H2 | 10 |
| 20006 | train | 2020-01-06 | ~21 % O2, N2 | 44.4 % H2 in CH4 | 17 |
| 20013 | train | 2020-01-13 | ~21 % O2, N2 | 66.7 % H2 in CH4 | 23 |
| 20024 | train | 2020-01-24 | ~21 % O2, N2 | 100 % H2 | 16 |
| 20029 | train | 2020-01-29 | ~21 % O2, N2 | 50 % C2H4 in N2 | 21 |
| 20132 | train | 2020-05-11 | ~21 % O2, He | 25 % H2, 20 % CH4 in He | 9 |
| 20136 | train | 2020-05-15 | ~21 % O2, He | 20 % H2, 25 % CH4 in He | 11 |
| 20155 | train | 2020-06-03 | ~30 % and ~21 % O2, He | 45 % H2 in He | 11 |
| 20160 | validation | 2020-06-08 | ~30 % and ~21 % O2, He | 45 % CH4 in He | 12 |
| 20164 | validation | 2020-06-12 | ~30 % O2, He | 20 % C2H4 in He | 11 |
| 20190 | train | 2020-07-08 | ~30 % and ~21 % O2, He | 35 % H2 in He | 26 |

- **Test days, not opened:** 20142, 20170 and 20176 (all helium-diluted). Day 19245 has logs and module data only and is not in the split file.
- **Table quirks.** The table writes "N3", "N4"… on some atmosphere cells; these look like typos for N2. A footnote says pressure rises and O2 falls with vitiation during a day.
- **Fuels.** Ethylene on 4 opened days (63 tests), methane on 3 (43), hydrogen on 4 (63), hydrogen–methane mixtures on 4 (60).
- **Era.** Every date precedes 30 August 2021, so P2 and P3 would read every test as early era. Their run ids carry the date in `YYDDD` form, as in Flame Design.

## (c) Ranges of the model inputs (train and validation days)

| Input | Source | Range | Note |
|---|---|---|---|
| `pressure` | not tabulated; chamber transducer PTA2 at the start of the record | 0.920–0.940 V, about 1.00–1.05 atm | V-to-atm scale taken from five Flame Design files against their table `pinit`; the scale was not checked on PSI-23 |
| fuel-bottle flow | `fuelBottleTestFlowRateN2Slpm` | 0.037–2.00 slpm (N2-equivalent) | the bottle is premixed fuel + diluent |
| N2 diluent flow | `nitrogenTestFlowRateN2Slpm` | 0–0.66 slpm | 0 on all helium days (He is in the bottle) |
| `injected_mass_flow` (ethylene days) | flows × bottle fraction × nominal MFC gas factor (C2H4 0.50, He 1.45) | about 0.28–2.4 mg/s with N2, 1.5–4.4 mg/s with He | approximate; the MFC gas codes (header: 6 = 50 % C2H4/N2, 16 = 20 % C2H4/He, …) were not checked against conversion tables |
| `x_fuel_role` (ethylene) | same | about 0.09–0.22 with N2, 0.20 with He | approximate |
| `x_o2_role` | table, nominal | 0.21 or 0.30 | on 20155, 20160 and 20190 (49 tests) the table gives both values, and the per-test value is not in the scripts or the headers |
| `zst` (ethylene) | computable from the initial compositions | about 0.24–0.43 | approximate, follows the flows |
| `tad` | **not tabulated** | n/a | needs an equilibrium computation at the initial composition, which is not installed here |
| `programmed_stop` | script stage sum | 0–60 s (constant-flow tests: 45 or 60 s) | far shorter than the 120 s horizon |
| ignition flow equal to test flow | script | 214 of 223 scripts | |

Against the normal-flame domain of `flame.rs` (the same bounds in flame3.rs):

- **N2-diluted ethylene:** inside on pressure, O2, x_fuel and zst, with c_logm from about −1.3 to +0.9.
- **He-diluted ethylene (20164):** one of 8 constant-flow tests lies above the c_logm bound (about 4.4 against 4.0 mg/s). The forward has no diluent input, so helium is an extrapolation it cannot see.

## (d) Constant flow, geometry, and what the models would refuse

- **Constant flow.** No ramp holds or ramp sizes in the script, and a constant-flow stage. That holds for 126 of 229 tests; 97 are ramped and 6 have no script.

| Fuel | Constant flow, train | Constant flow, validation | Ramped or no script |
|---|---|---|---|
| ethylene | 18 (19254, 20029) | 11 (19256, 20164) | 34 |
| methane | 9 | 23 | 11 |
| hydrogen | 33 | 0 | 30 |
| H2/CH4 mixtures | 32 | 0 | 28 |

- **Geometry.** Every test burns a fuel stream from the burner into an oxidizer ambient, so all are normal flames. No test is an inverse flame: the scripts command only a fuel-bottle flow and a nitrogen flow, and no oxygen flow.
  - That they are spherical is taken from the investigation's name and the shared hardware. The science documents were not read.
- **Unchanged models:** 229 of 229 refused at the condition gate.
  - The fuel is not ethane, propane or butane, as on Flame Design.
  - `tad` is missing.
  - On 49 tests the initial O2 per test is also unknown.
- **Surrogate arm, if registered (ethane's indicators for ethylene, Tad computed):** 29 constant-flow ethylene tests in 4 sessions (18 train, 11 validation).
  - 8 of them are He-diluted, and one of those is outside the mass-flow bound.
  - Methane, hydrogen and their mixtures have no defensible surrogate among the three model fuels.
  - Running H2 as anything the models know would be a much larger extrapolation than ethylene as ethane.

## (e) Endpoints: can the method be applied?

- **Channels.** All 228 data files carry every channel `flame_design.py` reads (PMT1–3, MFCA1StaticMixerFlow, MFCA2BypassFlow, MFCA3FuelFlow). They also carry the radiometers under the 2019 Flame Design names (`Radiometer+60` … `RadiometerOffset`). The column set is that of Flame Design day 19115; two layouts occur, differing only by an empty trailing column.
- **Other inputs.** The setpoint log exists for 228 tests and the script for 223. The TP file is missing for 20 tests; for those the script falls back on the flow-on rule for its ignition reference.
- **Running it.** The method is measurable as written once the file prefix is set to `PSI-23_Raw Data_`, as was done for CFI-G. It must first be checked on s-Flame train tests, as 11.1 did on CFI-G.
- **Open questions** (cannot be answered without reading outcomes):
  - whether the radiometer onset and the PMT dark-level rule work on dim hydrogen and helium-diluted flames;
  - how the 250 °C burner limit and `extinctionDetectionStopsTest = true` (most scripts) shape the endings.

## (f) Frozen groups (docs/wsp/atlas/splits/PSI-23.json)

- **Train:** 11 days (19254, 19304, 19352, 20006, 20013, 20024, 20029, 20132, 20136, 20155, 20190).
- **Validation:** 4 days (19256, 19308, 20160, 20164).
- **Test:** 3 days (20142, 20170, 20176), not opened.
- **Held:** day 19245 is not listed, so it is held.
- **Surrogate arm by group:** train days 19254 and 20029; validation days 19256 and 20164.

## Not verified

- MFC gas-code meanings and correction factors, so mass flows and fuel fractions above are approximate.
- The PTA2 voltage-to-pressure scale on PSI-23.
- The per-test O2 on the mixed-O2 days. The daily test-point logs (two `.xlsx` files) may hold it but may also hold outcomes, and were not opened.
- Whether `as run` differs from the planned `.txt` on purpose or by edit.
- Spherical geometry and burner identity, from documents not read.
