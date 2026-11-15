# NASA's condition tables against NASA's own sensor and settings files

Document lane, 15 November 2026. Conditions only: no outcome is computed, read or recorded for any test.
The script is `tools/wsp/docs/crosscheck.py` (no arguments; about 3 minutes). It writes one record per
discrepancy to `runs/dev/data_docs/discrepancies/discrepancies.jsonl` (schema `ignis.docs.discrepancy/1`) and
the counts, comparison families and limits to `summary.json` beside it. NASA's values are never replaced
(`nasa_value_replaced: false` on every record).

**Of 155 discrepancies, the cause of 54 is established** (37 `table_value_wrong`, 13
`table_records_plan_not_execution`, 4 `two_definitions`); 101 are `unresolved`. 18 further differences were
our reading, not NASA's table (`our_reading_wrong`, section 5): they are not records. 12,790 single
comparisons were made; 12,637 agree.

## 1. Method

**Table side.** `runs/dev/data_all/work/atlas/tests.jsonl`, which copies each condition cell with its file,
column and row. Raw tables are not opened, so no outcome column is parsed. For the project families (PSI-25,
PSI-69, PSI-159) the atlas holds only train and validation rows; the test partition is never read.

**NASA files.** Through the upload log of the Astra worktree (`file_name` to `source_path` and `sha256`). One
gate opens every file: it refuses the protected paths of the brief, checks the SHA-256 (no failure in 7,134
openings), and for the project families serves only files whose name carries a test that `tests.jsonl` lists.
The CFI-G chamber calculator covers the whole campaign; only the rows of listed tests are read from it.

**Channels and time.** Only flow, setpoint, readback, pressure, temperature-before-ignition and composition
channels are parsed, and only up to ignition:

| Apparatus | Ignition reference | What is read before it |
|---|---|---|
| ACME (PSI-10, 20-23, 159) | timeline stamp `Ignition and retraction` + 0.999 s (the stamp's resolution); without a timeline file, the first command of the stream that lights the flame | setpoint log (MFCA1-3), readbacks `MFCA1StaticMixerFlow`, `MFCA2BypassFlow`, `MFCA3FuelFlow`, chamber `PTA2/PTA3`; the script's flow keys and stage times; the header's gas codes, full scales, burner, polarity, experiment |
| ACME chamber calculator | the test point's event row | `Pstart`, Source-ID of the bottle, start mixture; not the ignite decision, litres delivered, end pressure or finish mixture |
| MDCA (PSI-39, 68, 69) | log stamp `AS Burn to Completion/Igniter On` | log header up to its `Note` line; chamber PT16/PT17 and thermistors over the 10 s before; FLEX test sheets up to `TEST CONDITIONS` (so the fitted rate, extinction diameter and narrative are never parsed); FLEX-2 mixture readings (O2) |
| Saffire (PSI-98-100) | first `Igniter Power Enable` | O2, anemometers, ambient pressure, fan speed over the 60 s before |
| SAME-R (PSI-101) | no flame | test number, time, heating flag, sample resistance; no detector channel |
| DAFT, SLICE, SPICE, BASS-II | none needed | the `GMT:` line of the DAFT instrument files; file names only |

Photomultiplier, radiometer, camera and flame channels are never parsed, and no flow is read after ignition.
That rule shapes the ACME results: the steady test flow is not readable, so the comparison reading is the
**command at ignition** when the script lights the flame with the test flow (one stage, ignition flow =
test flow), and the **script's longest constant-flow stage** otherwise. Readbacks before ignition are an
independent second reading only where they have settled (flat, at least 0.8 s after the command, at least
10 % of it); a readback flat at zero after 3 s of a non-zero command is recorded as "reads no flow".

**Conversions.** ACME controllers: Hastings HFC-D-302, nitrogen-calibrated, 0-5 V over the header's full
scale, standard 0 C and 760 torr (E-FIELD and s-Flame data interpretation documents, section 4.A.1). Gas
conversion factors and molecular weights from the calculator workbooks (`SourceTable`, `Property Table`;
the same factors are in `ACMEFlowCalculator.xlsx`), mixture rule 1/sum(x/GCF). Header gas codes 0-18 from
`ACMEFlowCalculator.xlsx`; codes 19-25 (2021) are read from the calculator's bottle of the same test point
(19 propane, 20 and 23 30 % butane, 21 ethane, 22 30 % propane, 24 50 % butane, 25 30 % ethane). The ACME
pressure transducer PTA3 is uncalibrated; it is read with a straight line through the calculator's start
pressures (PSI-10: psia = -18.30 + 36.03 V; PSI-159: -18.30 + 36.03 V). This line is the only fitted thing.

**Check 1 on every family.** Each comparison (one investigation, one field, one reading) is a family. Its
median table/reading ratio (or difference) is the common factor and its robust sigma the method's own error;
a test is a discrepancy when it lies more than max(floor, 5 sigma) from the median (floors: 3 % for flows,
0.004 to 0.02 for fractions, 2-5 % for pressures). Families of fewer than 5 tests are compared directly. A
common factor beyond its own spread becomes one investigation-level record.

**One record** gives investigation, test, group, field, table value with file, column, row and unit, the
reading with file, channel, method and unit, the other NASA sources read for it, the difference against the
method's error and threshold, a status (`confirmed`: a second independent reading agrees with the first;
`seen_once`; `unsettled`: the readings disagree among themselves), the five cause checks with their results,
the check that decided, and the verdict.

**Causes, in order.** (1) the family's own spread; (2) a third NASA source: the script (as-run file), the
command log, the calculator, the CIR name, a test sheet, the Saffire Readme table, the fan speed (an operator
comment is not read: ACME `Comments` files describe the downlinked images and the flame); (3) physics: no
diluent flowing means the injected fraction is the bottle's; a gas's fractions add to one; flows and speeds
are not negative; (4) shape: factor 2 or 10, a copy of the row above (only when that row's own reading has the
value and this row's does not), the neighbour's reading (shifted rows), a round nominal value repeated on a
day, twelve hours (am/pm); (5) the same quantity under two definitions. A test's outcome is never used.

## 2. The known cases of dossier section 2.7

All reproduced. The two burn-time rows of 2.7 were skipped (outcomes).

| Tests | 2.7 | This method | Status | Verdict (check) |
|---|---|---|---|---|
| 21125K1, K2, K3 | Xf 0.031 vs 0.30 | 0.031 vs 0.300: commands and script give the 30 % butane bottle (calculator and table agree on the bottle) with no nitrogen; the nitrogen readback reads zero | confirmed | `table_value_wrong` (2); shape factor 1/10 |
| 21106G6, G7 | 0.8 vs 0.5 mg/s | 0.8 vs 0.494 on the table's basis (0.539 on the controllers' 0 C basis); script = command | seen once (fuel readback not settled before ignition) | `table_value_wrong` (2) |
| 21106G9 | about 0.71 mg/s | 0.8 vs 0.659 on the table's basis, 0.718 on the controllers' basis (2.7's number is on that basis) | seen once | `table_value_wrong` (2) |
| 21196M2_1 | O2 5.59 vs 2.79 mg/s, as-run agrees with the table | 5.59 vs 2.80; the script gives 5.59, the command log gives M2's command. Inside the files the timeline names itself 21196M2 and the script 21196M3 | seen once | `table_records_plan_not_execution` (2); two name records `unresolved` |
| 21125D2 | 0.045 vs 0.037 | 0.045 vs 0.0372, script = command | seen once | `table_value_wrong` (2) |
| 21228C4, 21228K2, 21256C2 | unsettled: N2 0.068 slpm command, no readback | the same: XO2 0.50 by command and script, 0.85 if the nitrogen is taken as not flowing. The command is 1.4 % of full scale, inside the controller's own pre-ignition offset (settled readback minus command: median -0.030 V, p10 -0.050 V, n 20) | unsettled | `unresolved` |
| CFI-G flow factor | readback/table 1.083 (p5-p95 1.058-1.089) | command/table 1.090 on train (79 normal, p5-p95 1.086-1.095; 73 inverse 1.091, 1.087-1.094), the same on validation. It is a standard litre at 24.7 C against the controllers' 0 C (25 C gives 1.0915). Settled readbacks before ignition read 0.2-1.2 % under their command, which is how a readback-based factor comes out near 1.083 | confirmed | `two_definitions` (5), one record per orientation |

Found in the same class: 21196M2, 21196M2_1 and 21235C1 (inverse) also read no nitrogen before ignition; for
M2 and M2_1 the command is 5.5 % of full scale, well above the offset band. What flowed after ignition is not
read, so these stay `unsettled`.

## 3. Counts

| Investigation | table_value_wrong | plan_not_execution | two_definitions | unresolved | all |
|---|---:|---:|---:|---:|---:|
| PSI-10 Flame Design | 2 | 11 | 2 | 2 | 17 |
| PSI-20 BRE | 0 | 0 | 0 | 2 | 2 |
| PSI-21 CLD Flame | 0 | 0 | 0 | 3 | 3 |
| PSI-22 E-FIELD | 1 | 0 | 0 | 2 | 3 |
| PSI-23 s-Flame | 0 | 0 | 0 | 3 | 3 |
| PSI-39 CFI | 18 | 0 | 0 | 11 | 29 |
| PSI-47 DAFT | 0 | 0 | 0 | 1 | 1 |
| PSI-68 FLEX-2 | 0 | 1 | 0 | 0 | 1 |
| PSI-69 FLEX | 0 | 0 | 0 | 12 | 12 |
| PSI-98 Saffire-I | 0 | 0 | 0 | 1 | 1 |
| PSI-99 Saffire-II | 1 | 0 | 0 | 1 | 2 |
| PSI-100 Saffire-III | 0 | 0 | 0 | 2 | 2 |
| PSI-101 SAME-R | 0 | 0 | 0 | 6 | 6 |
| PSI-106 SLICE | 0 | 0 | 0 | 1 | 1 |
| PSI-107 SPICE | 8 | 0 | 0 | 33 | 41 |
| PSI-159 CFI-G | 7 | 1 | 2 | 21 | 31 |
| PSI-25, PSI-26, PSI-102 | 0 | 0 | 0 | 0 | 0 |
| **all** | **37** | **13** | **4** | **101** | **155** |

| Field group | Fields | wrong | plan | two def. | unresolved |
|---|---|---:|---:|---:|---:|
| flows | fuel, oxidiser, inert mass flow; flow speed | 14 | 1 | 4 | 4 |
| composition | injected and ambient fractions, diluents, bottle, sums | 8 | 11 | 0 | 38 |
| pressure and chamber | pressure | 0 | 1 | 0 | 1 |
| ids and dates | date, start time, test name | 6 | 0 | 0 | 15 |
| other settings | igniter power and time, experiment code, units | 0 | 0 | 0 | 12 |
| the table against itself | repeated rows | 9 | 0 | 0 | 31 |

Status: 34 confirmed, 115 seen once, 6 unsettled. Deciding check: 2 (third source) 14, 3 (physics) 12,
4 (shape) 24, 5 (definitions) 4.

## 4. What was found

- **PSI-159 CFI-G.** The seven 2.7 cases and the factor (section 2). Plus: on days 21196 and 21209 the inverse
  table's ambient fuel fraction before the test is 0.007 to 0.019 above the calculator's start mixture on 13
  tests, while all other inverse tests agree to 0.001 (unresolved; both days precede the August 2021 gas-factor
  correction of the calculator described in the s-Flame interpretation document, which is a lead, not a proof).
  Pressure and ambient oxygen agree with the calculator and with PTA3.
- **PSI-10 Flame Design.** The table's pressure is the calculator's to the digit (so the calculator counts with
  the table) and agrees with PTA3 on all 306 tests. On day 21301 eleven rows give XO2 0.40 where the calculator's
  start mixture falls from 0.389 to 0.371: the day's nominal value (`table_records_plan_not_execution`, 4). Two
  nitrogen flows (19175N2, N3) differ from script and command (`table_value_wrong`, 2). 22027B6 tabulates the
  script's second stage, not its longest (`two_definitions`, 5). 19150A4: fuel and nitrogen both 20 % above the
  script (unresolved). The flows carry small common offsets (table/command 0.988 fuel, 0.992 oxygen, 0.97
  nitrogen with 3 % spread), within the controllers' accuracy and not records: the PSI-10 flows look measured
  (unrounded), which a pre-ignition command cannot confirm.
- **PSI-20 to 23 day tables.** Bottles, burners, mesh polarity, controller full scales and experiment codes agree
  with the headers on every day except: s-Flame 20136, 20142 (header code 12 on both, the table names two other
  hydrogen-methane-helium bottles of the code list) and 20190 (35 % hydrogen in helium against code 14, 45 %; the
  code list has no 35 % bottle); one CLD test of day 18199 headed as an E-FIELD experiment; BRE day 19091 and
  E-FIELD day 18274 with no test-point file; CLD 18053 and 18145 with ignition mixtures outside the day's range
  (the range includes changes during tests, unread here); one E-FIELD listed value with no test commanded at it;
  one BRE listed oxygen value 0.40 with the nearest test starting at 0.381. All unresolved. The E-FIELD block of
  day 18319 is repeated (`table_value_wrong`, 4). s-Flame "about 1 atm": PTA3 reads 1.00 to 1.03 atm.
- **PSI-39 CFI.** Dates of Flex 1237 and 1301 (typing errors; the CIR day settles them, check 2). Four local
  times are twelve hours from the log (am/pm inverted, check 4); one is ten minutes off. Oxygen + nitrogen +
  helium is 0.97 for Flex 1219-1222 (physics, check 3). Eight repeated CSV rows (check 4). Nine igniter times and
  one igniter power differ from the test point log (unresolved). Pressure "1 atm" agrees with PT16/17 (median
  table/reading 0.98).
- **PSI-69 FLEX** (train and validation). Table against test sheet: C05M101/C05M201 (oxygen, nitrogen, carbon
  dioxide), C08H502 (carbon dioxide 0.20 against none), H19H101/H19H201 (nitrogen and helium), 196F002 (pressure
  0.713 against 0.70 atm): unresolved, no third composition source. Pressure (57 tests with chamber files) and
  temperature agree with the chamber readings.
- **PSI-68 FLEX-2.** 17 of 184 test points start at about 1.27 atm, outside the matrix's pressure list
  (`table_records_plan_not_execution`, 2: the table is the planned matrix). Oxygen and temperature inside the sets.
- **Saffire.** Saffire-II sample 6: table 20 cm/s, anemometers 25 cm/s, fan 17 % faster than the other 20 cm/s
  samples, Readme 30 cm/s: the table's value is wrong (check 2), the true flow is not settled. Saffire-III: both
  samples 30 cm/s in the table, 25 cm/s on the anemometers (unresolved: no other sample at 30 to compare the fan
  with). Saffire-I sample 2: oxygen about 21.5 % against 22.3 % (unresolved). Thickness column in cm and mm.
- **PSI-101 SAME-R.** Resistance ratio agrees on all 44 runs. Six runs are dated one day before the housekeeping
  GMT day although they start at 12 h GMT, so the time zone does not explain it (unresolved).
- **PSI-107 SPICE.** Eight negative air rates (check 3). 31 groups of rows equal in their condition cells; the
  rows have no test id and the outcome cells are not read, so a repeat cannot be told from one setting held twice
  (unresolved). Days 62 and 166 in file names, day 168 in the table without files.
- **PSI-47, PSI-106.** DAFT session 1 (GMT 54) has no instrument file; the files are of session 2. SLICE files name
  test days 55 and 59 (with fuels) that the table does not list.

## 5. Readings that were ours (`our_reading_wrong`; not records)

- PSI-20 (8 days) and PSI-23 (9 days): the atlas fills `injection.x_fuel_injected` from the Fuel Bottle label,
  but test points of those days add nitrogen to the bottle before ignition. The NASA table names the bottle and is
  right; the atlas field should read as the bottle's fraction, not the injected one.
- PSI-25 T18: the project record's session is dated 2018-06-24; the proof sheet name gives 2014-06-24. The record's
  session id is ours (data lane), not a NASA cell.

During the work three readings of this script were fixed: commands issued after the ignition stamp were first read
as zero flow; a readback flat at zero was first taken as settled; E-FIELD continuation rows were first taken as
repeats. They produce no record.

## 6. Limits

- **No flow after ignition.** The ACME steady test flow, the MFC readbacks during the flame and every stage after
  the first are not read. Where a flame is lit with a different flow than the test flow, only the script is
  compared, and readback confirmation exists only for settled channels (PSI-159: 35 settled, all within 1 % of full
  scale + 5 % of the command; PSI-10: 28 of 41). Inverse oxygen never settles before ignition.
- **Operator comments** (ACME `Comments`) are not read: they describe the downlinked images and the flame. 2.7's
  "No nitrogen flowed" for 21256C2 is therefore not re-read.
- **Test partition** of PSI-25, PSI-69 and PSI-159 never read.
- **Not compared**: PSI-26 (no atlas row); PSI-102 SAME (a planned matrix with no test; its run data are legacy
  `.xls`, no reader installed); BASS-II flows ("Air display" is a post-run fact) and any BASS sensor (only proof
  sheet images); SPICE and SLICE flows (no flow file on disk); SAME-R inlet velocity (needs the duct area), heating
  duration and aging times; CFI oxygen and diluents (no chamber gas reading), fibre flag; FLEX composition beyond
  the test sheet (the chamber gas readings of PSI-69 are not on disk) and pressure for the 99 tests whose chamber
  file is missing; s-Flame atmosphere (no calculator covers its period); BRE pressure (not tabulated).
- **Ignition-conditioned lists** (BRE oxygen, E-FIELD concentrations) are judged only in the outcome-free direction
  (a listed value that no test started at). The CLD range includes in-test changes; only the ignition mixture is read.
- **PSI-10 pressure** and the calculator are one source; PTA3 is the independent reading, through our line.
- **Nominal tables** (CFI pressure, Saffire flows, s-Flame pressure) are compared like readings; a common factor is
  removed only for families of five tests or more.
- The CFI-G day-21196/21209 ambient differences and the FLEX test-sheet conflicts need a third composition source
  that is not on disk.

## 7. Reproduce

```powershell
python tools/wsp/docs/crosscheck.py
```

Inputs are pinned by the upload log's SHA-256; the console prints aggregates only.
