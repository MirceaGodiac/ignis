# FLEX-2 (PSI-68): a per-test catalogue, conditions only

> Written by a delegated scoring and data agent (claude-opus-5-5) on 15 November 2026. FLEX-2 is
> held whole (`docs/wsp/atlas/splits/PSI-68.json`: NASA's table is a test matrix of 8 rows, so the
> investigation had no per-test list). This catalogue gives it one, from the text files of every
> rack test point, so that a later split by test day can be frozen. Nothing fitted, nothing scored,
> no image or video downloaded or opened, no outcome recorded. Disclosures are in the last section.
>
> Built by `tools/wsp/ext/flex2_catalogue.py` (`python -I tools/wsp/ext/flex2_catalogue.py`).
> Output: `runs/dev/data_all/work/atlas/flex2_tests.jsonl` (one row per test point) and
> `runs/dev/data_all/work/atlas/flex2_tests_summary.json` (every count below, and the count per
> date). Paths under `runs/dev/` are local working folders that are not in git.

## The answer in five lines

- **596 rack test points** (cir_test_point names such as `A109D03`, `E163L01`) on **106 test days** from January 2012 to November 2016; 5 have no date (no timeline and no settings stamp). 123 gas reading points (`<x>R<nn>`) carry the ambient analyses; 58 LCTF calibration points, 9 background points and one `RDTS` point are not test points and are left out.
- **Fuel is established for 569** (95 %): 541 from NASA's own data records (the PSI listing files a test point's data under a record named "<test matrix> <year> - <fuel>"), 6 joined by date, 22 inferred from the neighbouring filed tests of the same rack series; 27 are filed under two records that disagree and stay unresolved (26 of them iso-octane or n-octane, June to August 2014). No test file states the fuel.
- Fuels: n-decane 190, n-octane 59, 1-propanol/glycerol 95/5 48 and 80/20 34, n-heptane/toluene 50/50 33, toluene 31, n-heptane/iso-octane 50/50 30, three n-decane/propylbenzene blends 78, propylbenzene 21, n-heptane 18, ethanol 17, iso-octane 10, unresolved 27.
- **187 test points are single free droplets of a pure fuel that a candidate model covers** (n-decane 108, n-octane 51, n-heptane 18, iso-octane 10), on 41 days; **179 of them have a backlit archive `<test>A.zip`, 282 GB in all** (n-decane 179.9, n-octane 69.0, n-heptane 21.3, iso-octane 11.8 GB). Of the 187, 108 are inside the declared domain by their measured conditions (101 with an archive, 177 GB), 17 are outside (oxygen in 60 to 75 % helium, c_he > 3) and 62 have no gas analysis on their day.
- The frozen `flex_heptane` releases cover 18 of them (10 inside the domain, all 2013-03); ethanol was flown only in droplet arrays, so the ethanol candidate has no single-droplet test. FLEX-2 has no processed diameter series: every test would need its diameter measured from the frames first.

## How each field was established

| Field | Source | Method | Coverage |
|---|---|---|---|
| Test point, kind | `files.jsonl` (`test_id_scheme`) | as catalogued | 596 |
| Date (the later group) | timeline `<test>.txt`: "AS Burn to Completion/Igniter On" (else "AS Started") | GMT stamp | 578; 13 more from the settings log's `MAIN_COMPUTER_TIME`; 5 unknown |
| Fuel | PSI data-record folder of the test point's files (`files.jsonl` `folder`), then the rules below | see the next table | 569 established, 27 conflicting |
| Test matrix | the same folder | Quiescent, SurrogateFuel, Condensed Phase, Binary Array, Convective Flow | 568 filed (567 under a record that names a matrix); 28 not filed |
| Configuration | `<test>_MDCA_Testpoint_Settings.txt` (`TP_PARAMETER_FIBER_USAGE_FLAG`, flow nozzle speeds, dispenser profiles) and the matrix | array if the record is Binary Array; imposed flow if Convective Flow or a nozzle speed is not 0; fibre if the flag says Yes; two dispensers active; else single free droplet | 581 with settings; 8 unknown |
| Ambient composition | `<R>_Mixture.txt` of the gas reading point of the same day taken last before the test's igniter came on (else the first after it that day) | joined by date and sequence | 479; 117 unknown (no gas reading on their day) |
| Pressure | timeline table: mean "Ave PSIA" over the rows stamped in the 120 s before igniter on; if none, mean `FCU_MP_PT16` of `<test>_Chamber_Readings.txt` over the same window | measured before ignition | 524 + 49; 23 unknown |
| Igniter, planned record length | timeline header, settings log | as written | with the timeline |
| Backlit archive | `files.jsonl`: `<test>A.zip` and its `size_bytes` (not downloaded) | as listed | 519 of 596 have one, 755.8 GB in all |
| Operator notes | `<test>_Comments.txt` | only deployment and record-length remarks taken | 4 remarks (one manual deployment, two record-length errors, one 1-image-per-second archive) |

**How the fuel is established (`fuel_how`).**

| How | Test points | Meaning |
|---|---|---|
| `psi_data_record` | 540 | the test point's files are filed by NASA under one fuel-named data record |
| `psi_data_record (two records; the other names another year)` | 1 | filed under two records; only one is consistent with the test's year |
| `joined_by_date` | 6 | not filed; every filed test point of that day has the same fuel |
| `inferred_campaign_neighbours` | 22 | not filed and not joined; the filed test points of the same rack series within 30 days all have one fuel (all 22 are `C0xxB` points of February and March 2016: 1-propanol/glycerol 95/5) |
| `psi_data_record (conflicting records)` | 27 | filed under two fuel records that cannot both be right; left unresolved (the gas analyses of those days show neither octane nor iso-octane vapour) |
| stated in a file of the test | 0 | no timeline, settings log, comment or event-log file names a fuel |

The two rules `joined_by_sequence` (filed neighbours before and after on the same day agree) and
`inferred_vapour` (one fuel vapour in the day's gas analyses) were tried and decided no test point
that the rules before them had left open.

## Counts

**By year and month** (days, test points, fuels). The count per date is in `flex2_tests_summary.json`.

| Month | Days | Test points | Fuels |
|---|---|---|---|
| 2012-01 to 2012-03 | 5 | 18 | n-octane |
| 2012-04 to 2012-05 | 9 | 55 | propylbenzene 21, n-decane/propylbenzene 50/50 24, n-octane 10 |
| 2012-06 | 2 | 11 | n-octane |
| 2012-07 to 2012-12 | 17 | 78 | n-decane |
| 2013-03 | 3 | 18 | n-heptane |
| 2013-11 to 2013-12 | 4 | 19 | n-octane |
| 2014-02 to 2014-04 | 9 | 34 | 1-propanol/glycerol 80/20 |
| 2014-06 to 2014-08 | 14 | 107 | iso-octane 10, n-heptane/iso-octane 30, iso-octane or n-octane (unresolved) 26, n-octane 1, toluene 25, n-heptane/toluene 15 |
| 2014-09 to 2014-10 | 10 | 78 | n-heptane/toluene 18, toluene 6, n-decane/propylbenzene 75/25 28 and 25/75 26 |
| 2014-12 to 2015-04 | 13 | 69 | n-decane 51, ethanol 17, one conflicting |
| 2016-02 to 2016-03 | 9 | 46 | 1-propanol/glycerol 95/5 |
| 2016-04 to 2016-11 | 11 | 58 | n-decane |
| no date | | 5 | 1-propanol/glycerol 95/5 2, n-decane 3 |

**By configuration.** Single free droplet 468; droplet array (Binary Array) 69; imposed flow 28;
fibre-supported 12; two dispensers active 11; unknown 8.

**Ambient** (479 test points with an analysis). O2/N2 near air (0.20 to 0.21 O2, no helium) on 334;
O2 in nitrogen at 0.17 on 35, 0.30 on 9, 0.25 on 5, 0.24 on 3; oxygen in helium (0.24/0.75 to
0.40/0.60) on 93. One analysis (`E216R01`: 0.003 % O2, 78.9 % CO2, the
gases summing to 79 %) is implausible and was not used. 29 Mixture files carry a date that differs
from their reading point's timeline: the 18 readings of 6 June 2016 (`A158R01` to `R18`) are dated
18 May, 8 differ in the year only (2015 for 2016, 2013 for 2014), 3 by a few days. Their
compositions were used and the row says so.

**Pressure** (before ignition, binned): about 0.5 atm on 40, 1 atm on 479, 1.25 atm on 17, 2 atm on 5,
3 atm on 32; 23 unknown. Where both the timeline table and `FCU_MP_PT16` exist (465 test points) they
agree within 9 %.

## Single droplets of a pure fuel a candidate covers

| Fuel | Candidate | Single free droplets | With backlit archive (GB) | Inside the domain | Outside (c_he > 3) | No gas analysis that day |
|---|---|---|---|---|---|---|
| n-decane | `wsp5/decane/w0-fable` (not assembled) | 108 | 102 (179.9) | 64 | 17 | 27 |
| n-octane | `wsp5/octane/w0-fable` (not assembled) | 51 | 49 (69.0) | 32 | 0 | 19 |
| n-heptane | `flex_heptane.wsp2` D2, `flex_heptane.wsp5` (frozen) | 18 | 18 (21.3) | 10 | 0 | 8 |
| iso-octane | `wsp5/isooctane/w0-fable` (not assembled) | 10 | 10 (11.8) | 2 | 0 | 8 |
| ethanol | `wsp5/ethanol/w0-fable` (not assembled) | 0 | | | | |
| **total** | | **187** | **179 (282.0)** | **108** (101 with archive, 177.2 GB) | **17** | **62** |

The 26 test points whose fuel is iso-octane or n-octane (unresolved) are single free droplets and
are not in this table. If the records were resolved they would add 26 to one of the two rows.
"Inside the domain" uses the measured conditions: c_o2 = (x_o2 − 0.21)/0.05 in [−2.2, 4.8], c_co2 =
x_co2/0.2 in [0, 4], c_he = x_he/0.2 in [0, 3], c_logp = ln(p/1e5 Pa) in [−1, 1.5]. The initial
diameter (c_logd0) is not known here: it is estimated by the forward from the measured prefix.

## What is unknown

- **The fuel of 27 test points**: 26 filed under both "SurrogateFuel 2014 - IsoOctane" and "100%
  Octane" (days 2014-06-24, 2014-07-02, 2014-07-28, 2014-08-04) and one under a decane and a
  propanol/glycerol record (2014-12-03). No text file of those test points names the fuel.
- **The ambient of 117 test points** (62 of the 187), on days without a gas reading point. The nearest
  reading of another day could be joined, but the chamber is refilled between sessions, so it was not.
- **Droplet size.** No text file states an intended diameter; needle positions and dispenser moves are
  in the settings log but are not calibrated to a size. The matrix gives 1.5 to 5 mm.
- **Whether a test burned at all**, and whether the droplet stayed in view: these are outcomes and were
  not recorded, though some operator notes speak of them. A transfer test must define its cases from
  the measured series, as in FLEX.
- **The diameter series.** FLEX-2 has none; a test would first need the backlit frames measured by an
  agent that sees no forecast (PROTOCOL 12.2), on a time base tied to the timeline's igniter stamp.
  The backlit archives of the 101 in-domain candidates with an archive are 177 GB.
- **The frame time base**: comments report frames with irregular time deltas on many archives; their
  per-frame index files (`<test>A.txt`) exist but were not read.

## Disclosures

- No image, video, `.docx` event log beyond a text check (all 88 are empty of fuel names), radiometer
  series or cool-flame log was opened for its content. The `Cool_Flames.txt` header was printed once to
  learn its kind (trigger-time settings); no row was used.
- While checking why some timelines gave no pre-ignition pressure, two table rows of `D005D04.txt`
  stamped just after its igniter came on (radiometer readings of the first second, chamber pressure)
  were printed to this agent's console. They are not recorded anywhere.
- Operator comments were scanned by keyword. Their remarks on ignition and on what happened in a test
  were seen on the console for a few test points and are not recorded in the catalogue or here.
