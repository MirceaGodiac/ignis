# ACME CFI-G raw channels on the train tests: burner thermocouple, script timeline, limits

> Written by a delegated analysis agent (claude-opus-5-5) on 14 November 2026; nothing fitted. Train tests only, outcomes included.
> The author rechecked the claims he acted on; which ones, and what stays unchecked, is in
> `docs/wsp/dossier/03_datasets_and_measurements.md`. Paths under `runs/dev/` are local working folders that
> are not in git; the dossier says how to rebuild them.

I measured burner heating, script timelines and test order on the 154 train tests, using direct readings only. 153 of them have a data file. The typed law's medians are close to 1, but one tau does not describe the traces. The programmed stop can be read before ignition from the "as run" file. The burner limit changed between 30 Aug and 27 Sep 2021, from about 500 °C with about 4 s of overshoot to about 450 °C with about 1.5 s.

## 1. Files available, and downloads
- Per train test the listing has: `_data.csv` 152, `_mfcSetpoints.csv` 152, "as run" (the script parameters) 152, `_TP.txt` 133, `_Comments.txt` 138, `_ImagesComments.txt` / `_ImageComments.txt` 12.
- `21196M2` and `21196M2_1` each have a full set. `21130A9` has no data file. 20 tests have no TP file, including all of 2021-08-30 and 2021-09-13.
- I downloaded everything except media: 744 files, all data files included because the total was 727 MB. 25 of them were copied from `data/raw/samples` instead of downloaded. No failures.
- I also downloaded the ACME Engineering Implementation Report (7.5 MB) to use as a channel dictionary.
- **Total downloaded: 712,264,203 bytes** (704,767,879 + 7,496,324).
- The comments files only hold notes about image downlinks; there is nothing physical in them.

## 2. Channel identification in `_data.csv` (about 125 Hz, clock times)
- **TCBurner = burner thermocouple. Very sure.** The engineering report says the spherical burner has a type-K thermocouple embedded about 1 mm in the sphere. Its voltage is the type-K EMF relative to the cold junction. With the NIST type-K table and a cold junction of 23.5 °C, my smoothed peak matches the table's "Peak burner T" within -2.0 to +1.0 °C in every session; session medians run from -0.9 to +0.6 °C.
- **Caveat on absolute temperature:** I could not decode the cold-junction channel TCCJ (0.871–0.877 V, almost constant). The cold junction is therefore an assumed constant. Absolute temperatures carry about ±1.5 °C of uncertainty; rises do not.
- **TCA–TCD = far-field rake, TC200A/TC200B = the two lone thermocouples at 200 mm. Sure about the group.** The report says the two inner rake thermocouples are type B, so TCA/TCB values converted as type K are wrong. I did not establish which channel is at which distance.
- **PMT1–3 = photomultipliers. Sure.** They go to zero when the flame goes out. Where the PMT sum drops below 0.02 V, I take that as the flame end. Inverse flames saturate them (gain 10, sum about 13.5 V).
- **Radiometer channels:** `Radiometer3+60/+30/-10 Lo/HiGain` are radiometers. The HiGain channels saturate at about 4.78 V. LoGain decays slowly after extinction, probably because it sees the hot burner.
- **`RadiometerOffset`** follows the flame (1.2 V jumping to 4.8 V). It is probably a radiometer, but I have not identified it.
- **Mass-flow readbacks: very sure.** They are 0–5 V across the full scale given in the file header, in N2-equivalent slpm.
  - Normal flames: MFCA3 carries the fuel bottle and MFCA2 adds N2.
  - Inverse flames: MFCA1 carries the O2 bottle and MFCA2 adds N2 (5 slpm full scale).
  - The inverse O2 bottle is 85 % O2. I worked that out from the tests with injected XO2 = 0.85 and no N2.
  - Check: readback-derived fuel or O2 flow is 1.083 × the table value (p5–p95 1.058–1.089), and it agrees with the `mfcSetpoints` files.
- **PTA2/PTA3 = chamber pressure.** Two calculator points give psia ≈ 36.0 × (V − 0.508); this is a two-point reading, not a calibration fit.
- **PTA1 is not chamber pressure.** It sits at 0.50 V, then 0.945 V from June on.
- **`IonCurrent` follows the fuel flow, not the flame.** Its label is doubtful.

## 3. Per-test table — `tests_burner.csv` (153 rows)
- Time zero is flame onset from the radiometers. It falls 2.44 s after the TP "Ignition and retraction" stamp (p10–p90 1.8–3.2 s).
- The PMT flame end matches the table burn time: median +0.17 s, 82 % of tests within ±1.5 s.
- Temperatures are left blank after the flow stops or the PMT flame end.
- Order within the day and minutes since the previous test are computed among train tests only. Other tests may fall in between.

## 4. Typed heating law against the traces (measured rise / law rise, using table flows)

| | 10 s | 30 s | 60 s | tau from rises at 30 and 60 s | rise(60 s)/rise(5 s) |
|---|---|---|---|---|---|
| All tests | 1.03 (p10–p90 0.59–1.56, n=110) | 1.05 (0.90–1.51, n=73) | 1.04 (0.86–1.56, n=49) | 31.6 (24–55) | 4.65 (3.65–11.8, n=49) |
| Normal | 1.04 | 1.04 | 1.03 | 29.0 [21.7–37.3] | |
| Inverse | 1.02 | 1.15 | 1.12 | 44.8 [21.8–72.7] | |

The tau column is a closed-form reading, not a fit: tau = −30 / ln(rise60/rise30 − 1).

Session medians and limit trips:

| Session | Kind, fuel, p (bar) | Ratio at 30 s | tau 30/60 | rise60/rise5 | Limit trips: peak °C |
|---|---|---|---|---|---|
| 03-29/30 | normal, ethane, 0.94–1.01 | 1.02 | 26.9 | 4.0 | 530 |
| 04-16/17 | normal, 50 % butane, 0.51 | 0.96 | 34.3 | 5.5 | 506 |
| 05-05/06 | normal, 30 % butane, 1.01 | **2.61** | 24.9 | 4.1 | – |
| 06-02 | normal, 30 % ethane, 0.51 and 2.03 | 1.06 | 26.1 | 4.1 | 509, 511 |
| 06-11 | normal, 30 % ethane, 1.01 | 1.06 | 29.0 | 4.6 | – |
| 07-15 | inverse, ethane, 0.50 and 1.10 | 1.15 | 53.8 | 7.6 | 515 |
| 07-28 | inverse, ethane, 0.55 | **1.49** | 37.4 | 6.6 | 543, 513 |
| 08-10/16 | inverse, propane, 3.3 / 1.1–2.2 | 0.76 | 60.5 | 13.9 | – |
| 08-30 | inverse, propane, 0.55 | – | – | – | **646** (heating at 37 K/s) |
| 09-13 | inverse, butane, 2.2 | 0.74 | 57.5 | 13.8 | – |
| 09-27 | inverse, butane, 1.1 | 1.04 | 23.9 | 3.4 | 457, 452 |
| 10-18/19 | normal, 30 % butane, 2.0 / 1.0 | 1.09 / 0.97 | 24.5 / 31.1 | 3.1 / 4.2 | 454, 452 |

- **One tau does not fit.** The tau read from the 10 and 20 s rises (median 22.8 s; about 13 s in Sep–Oct) is shorter than the one from 30 and 60 s. The early rise is faster than a single exponential with tau ≈ 30. Inverse tests at 0.55 and 2.2 bar show tau of 50–60 s.
- **The 2.61 for 05-05 comes from a table error, not from the law.** The table gives Xf = 0.031 for 21125K1/K2/K3. The setpoints, readbacks and "as run" file all show the 30 % butane bottle undiluted (Xf = 0.30), so total flow is 1.7–6.9 mg/s rather than the 12–48 the table implies.
- **Other table discrepancies:**
  - 21106G6/G7: the table says 0.8 mg/s, but the command and readback are 0.5 mg/s.
  - 21106G9: about 0.71 mg/s.
  - 21196M2_1: the table and "as run" file give O2 5.59 mg/s, but the commanded setpoints and readbacks equal those of 21196M2 (2.79 mg/s).
  - 21228C4, 21228K2, 21256C2: an N2 command of 0.068 slpm shows zero readback, so the real injected XO2 may be 0.85 rather than 0.5. I am not sure of this one.
- **Start temperature does not depend on order in the day.** By rank 1–17, the median start temperature stays at 24.9–25.2 °C. Burner minus the 200 mm thermocouples is 1.4–1.6 °C at every rank and every gap length. The exceptions follow a hot previous test with a gap of 11–16 min:
  - 21153A5 is +1.9 °C after a limit trip.
  - 21209H1, 21270C1 and 21270H3 are +0.7 to +0.9 °C.
  - 21106F3 and 21162F2 are raised by a late flame onset (5.8 and 9.7 s after the stamp).
- **The first 5 s do not settle the later rise.** rise(60)/rise(5) spans 2.7–15.9 (p10–p90 3.65–11.8). Normal sessions sit at 3.1–5.5; inverse sessions at 3.4–13.9.

## 5. Script timelines — `tp_timelines.csv`
- **Stage sequence:** script start → pause/resume → pre-ignition data → ignition and retraction → flame stabilization #1/#2 → constant flow → MFCs off → post-flame → script complete.
- **The "Constant flow" stamp is overwritten with the end time when the stage completes.** So constant flow = MFCs off marks a scripted stop.
- **Programmed stop, measured from flame onset** = constant-flow start (ignition stamp + 7 s; +9–11 s in March) + `constantFlowTime1Sec` − about 2.4 s. That is roughly c1 + 4.5 s (c1 + 6–7 s in March).
  - The values found are:

    | Flame type | c1 (s) | Programmed stop from onset (s) |
    |---|---|---|
    | Normal | 40 | 44 |
    | | 50 | 56 |
    | | 55 | 61 |
    | | 80 | 86 |
    | | 90 | 96 |
    | | 120 | 124–126 |
    | | 180 | 184 |
    | | 240 | 245 |
    | | 270 | 274–276 |
    | | 330 | 334–336 |
    | Inverse | 10 | 14.4 |
    | | 20 | 24.6 |
    | | 30 | 34.6 |
    | | 40 | 45 |
    | | 45 | 49 |
    | | 60 | 64–65 |
    | | 90 | 94.5 |
    | | 105 (stabilization 15 s) | 122 |
    | | 240 (stabilization 30 s) | 272 |

  - The stops you typed (61.5, 98, 126.5, 275.5) are the March c1 = 55/90/120 and the June c1 = 270 cases.
  - c1 is mostly a session default. It does not follow flow: 120 s on 05-05/05-10, 270 s on 06-02/06-11, 60 s on 08-10/08-16/09-13, 90 s on 09-27, 240 s in October.
- **Stop classes:**
  - Extinction-triggered: 88, with MFCs off = extinction stamp + 3 s (`postExtinctionDelaySec`). The extinction stamp lags the PMT flame end by a median of 11 s (5–23). Five of these are labelled FT in the table.
  - Scripted end: 29.
  - Burner-limit abort: 7, with a "Reason" line in the TP file.
  - Extinction abort: 1.
  - Ended with no stamps: 4. These are limit trips judging by temperature (21153C5, 21209A1, 21209B2, 21270H2).
  - No TP file: 24.
- **Limit trips:**
  - Through 2021-08-30 the flow stops 3.6–4.2 s after the trace crosses 500 °C (5.2 s for a very slow rise). The peak is therefore about 500 + 4 s × heating rate: 506 at 0.4 K/s, 530 at 4.6, 543 at 11.8, 646 at 37 K/s.
  - On 09-27 and 10-18/19 the flow stops 1.3–1.8 s after crossing 450 °C, with peaks of 452–457.
  - 21196M2 crossed 500 °C 3.8 s before its scripted end, so it was effectively at the limit too.
- **Knowable before the test started:** c1, the stabilization time, the post-extinction delay and the flow timeout (always c1 + 30 s, sometimes +20 or +25). The scripted stop follows to within about 1 s. The limit is a property of the session. The trip time follows from the heating trajectory plus the 4 s or 1.5 s dwell. Whether the test ends by extinction is not in the script.
- On inverse flames the burner temperature rises again by 4–6 K in the 1 s after the flow stops.

## 6. Hot-flame end
- The PMT end versus the table burn time is in `tests_burner.csv` for all 153 tests.
- Outliers:
  - 09-27 inverse tests: the table burn time is about 6 s shorter than the PMT end and the fed phase.
  - 21291A1 and 21291C1: the PMT goes dark at 41 and 89 s, but the burn time is 167 and 246 s. The flame was probably too dim for the PMT.
  - 21088D1: the PMT stays on 12.7 s after the flow stops.
- The radiometer −10 HiGain channel often fails to drop back, so it is less useful for this than the PMTs.

## 7. Other things known before ignition or in the first 5 s
- **Before ignition:**
  - c1 and the stage times from the "as run" file.
  - The commanded setpoints and the readbacks, which are better than the table and "as run" file (they expose the errors in finding 4).
  - Chamber pressure from PTA3.
  - PMT gains, which vary by session.
- **In the first 5 s (normal flames):** the burner heating slope over 0–5 s separates outcomes clearly.

  | Outcome | Median slope 0–5 s (K/s) |
  |---|---|
  | FT | 20.7 (p10–p90 14–26) |
  | SE, burn ≤ 9 s | 2.6 |
  | SE, burn 9–43 s | 9.1 |
  | SE, burn ≥ 44 s | 11.4 |

  PMT sum at 3–5 s: 0.49 / 4.3 / 11.0 V across the same three SE groups. The slope partly reflects flow.
- **Inverse flames:** the PMTs are saturated, and the slope separates outcomes less well (SE 9.9, FT 7.9 K/s).

## Files written
Everything is in `runs\dev\research5\acme_raw\`:
- `tests_burner.csv` — one row per test.
- `heating_law_check.csv` and `heating_law_summary.json` — the law check.
- `tp_timelines.csv` — stages, "as run" parameters, stop classes.
- `summary.txt` — all session medians.
- `file_inventory.csv` — which file kinds exist per train test.
- `chamber_calculator_train.csv` — train rows of the chamber calculator.
- `burner_traces_0p5s.json` — burner traces at 0.5 s steps.
- `asrun_keys_count.json`, `train_tests.json`, `download_list.json`, `download_report.json`, `dl.log`.
- `scripts\` — `rawlib.py`, `analyze.py` (run with argument 23.5), `lawcheck.py`, `timeline.py`, `summary.py`.

The downloads are in `runs\dev\nasa\PSI-159\`, with the engineering report under `docs\`. No tracked file was modified.

## Anything outside the train lists
- **The chamber calculator xlsx was not filtered on first view.** On the first look at its EventData sheet I printed rows 0–12 before filtering. Those rows were "Cabin Air" and "Vent" operations and the March 11 MFC checkout points 21070MFCA2_01-08, _09-12 and _13-14, plus one 2021-03-29 vent row. None of them is a CFI-G science test. Every later read of it was filtered to train test names.
- **I did not open any non-train file.** Listing `data/raw/samples` showed the names 21088C1 and 21196J1, but I never opened those files.
- **Tables:** the experiment tables were read whole in memory but printed only for train rows.
- **Session-level files not downloaded:** the ACME session logs and daily test-point logs mix in other tests, so I left them alone.
- **Other rules:** I read nothing under any holdout folder, never passed `--final-test`, never ran `dev_eval.py`, and read no published per-test CFI-G papers.
