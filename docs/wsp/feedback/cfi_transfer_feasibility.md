# CFI (PSI-39): can the frozen droplet models be run on it?

> Written by a delegated scoring and data agent (claude-opus-5-5) on 15 November 2026, before any
> transfer test of the droplet models on CFI is registered. Nothing fitted, nothing scored, no model
> or existing file changed. It reads conditions only: the campaign spreadsheet's condition sheets,
> the PSI experiment table, the header block and pre-ignition rows of the rack timelines of train and
> validation days, and the file catalogue. **No file of a test day was opened.** Disclosures are in
> the last section. Paths under `runs/dev/` are local working folders that are not in git.
>
> Built by `tools/wsp/ext/cfi_conditions.py` (`python -I tools/wsp/ext/cfi_conditions.py`), which
> writes `runs/dev/data_all/work/transfer_droplet/cfi/conditions.jsonl` (one row per test,
> conditions only) and `summary.json`. The scoring chain that a registered test would use is
> `tools/wsp/ext/transfer_droplet.py` (it reproduces `wsp-dev feedback` on FLEX train exactly).

## The answer in five lines

- 24 tests have NASA's processed diameter series (`FLEX-<n>_DropletData.txt`): 17 on train days, 4 on validation days, 3 on test days. All 24 are n-dodecane. 22 of them also have an Igor `.pxp`; no `.pxp` exists for a test without the text series.
- CFI burned n-dodecane (80 tests), n-dodecane/iso-dodecane blends (103) and farnesane (44), in air-like O2/N2 at 0.17 or 0.21 O2, with 0.29 or 0.59 helium on 52 tests, at 0.5 to 5 atm. No CFI test used a fuel of the FLEX models (methanol, n-heptane).
- By conditions, the frozen `flex_heptane` and `flex_methanol` releases refuse all 227 tests on the fuel. Of the candidates in `phys_droplet_fuels.md` section 0, only `wsp5/dodecane/w0-fable` answers any test: 30 by conditions (18 train, 7 validation, 5 test days). Of those, **7 have a processed series off the test days (5 train, 2 validation)**. The others (octane, iso-octane, decane, ethanol, farnesane) answer none.
- The main refusals: fuel (every model; the 103 blends have no forward); fibre support (103 tests say Yes and 69 do not say, among them all 44 farnesane tests); pressure above the declared domain (the 16 tests at 5 atm, c_logp = 1.62 > 1.5).
- The dodecane candidate is not assembled (`configs/models/flex/wsp5/dodecane/w0-fable` holds `edits.wsp` and `meta.json`, no `release.json`; `droplet6.rs` has no such fuel yet). Today nothing frozen can answer a CFI test without a declared surrogate.

## (a) Files and how a processed test maps to a test point and a day

| What | Name | Tests | Notes |
|---|---|---|---|
| Processed diameter series | `PSI-39_Raw Data_FLEX-<n>_DropletData.txt` | 24 (FLEX-1206 to 1264) | all downloaded; the same four tab-separated columns as FLEX's file (time, diameter in mm, two positions in mm) |
| Igor experiment | `PSI-39_Raw Data_FLEX_<n>_<ddMonyyyy>_Dod_v<k>.pxp`, and one `Analyzed Data_FLEX-1206_SummaryGraph.pxp` | 22 | 14 to 89 MB; `tools/wsp/ext/igor_pxp.py` reads them; not opened |
| Other processed series | `_ColorCamData`, `_RadiometerData`, `_CoolFlameData` (17) | 24 | outcome-bearing; not opened |
| Test sheet | `FLEX-<n>_TestSheet.txt` / `.rtf` | 22 | carries the investigators' derived values (outcome-bearing); not opened |
| Rack timeline | `<CIR>.txt` (e.g. `F068A03.txt`) | 121 test points (91 off the test days) | header block of settings and sequence stamps, then a table of radiometers, image counts and chamber pressure |
| Operator notes | `<CIR>_Comments.txt` | 121 | image-downlink notes; may carry outcome remarks; not opened |
| Test-point settings | `<CIR>_MDCA_CFI_Testpoint_Settings.txt` | 123 | needle, igniter and dispenser parameters; **no fibre flag** (FLEX's `TP_PARAMETER_FIBER_USAGE_FLAG` is absent) |
| Images, video | `<CIR>A/C/D.zip`, `E_IPSUG.zip`, `.ts`, `.mp4` | | bulk; not touched |

**FLEX number to test point.** The campaign spreadsheet's `Master Sheet` gives, per FLEX number, the
CIR test point (`f068a03`), the date, the local time and the conditions. The PSI table gives the same
columns for 79 distinct FLEX numbers (87 rows, 8 exact repeats) and agrees with the spreadsheet on
every one of them. Video names (`FLEX-1251_..._F125A04.ts`) and the `.pxp` names (which carry the
date) agree with that mapping where they exist.

**Day group.** The test point name carries the GMT day of year (`F068` = 9 March 2017). The frozen
groups of `docs/wsp/atlas/splits/PSI-39.json` follow that CIR day, in the year of the table's date,
as `tools/wsp/atlas/normalize_tests.py` assigns it. Recomputed here from the spreadsheet, this gives
136 train, 53 validation and 38 test rows, the counts the split file records. Three dates in the
table are wrong and the CIR day decides: FLEX-1237 (table 2 May, CIR day 1 May), FLEX-1301 (table
29 July, CIR day 29 June) and FLEX-1411 (table year 2011; it forms the train group
`PSI-39:day:2011-09-06` by itself, which is how the split was frozen).

**Processed tests by partition.**

| Partition | FLEX numbers with a series | Free droplets among them |
|---|---|---|
| train (17) | 1206, 1207, 1208, 1209, 1210, 1211, 1227, 1231, 1232, 1238, 1239, 1240, 1248, 1249, 1250, 1253, 1264 | 1206, 1239, 1240, 1250, 1264 |
| validation (4) | 1218, 1221, 1223, 1226 | 1221, 1223 |
| test (3) | 1213, 1215, 1261 | not opened; counted from the catalogue only |

**Format and time base, from the one exception.** The one processed series opened was
`PSI-39_Raw Data_FLEX-1206_DropletData.txt` (FLEX-1206, test point F068A03, train day 2017-03-09).
Its header and column layout are identical to FLEX's `DropletData.txt` (`DropletTime_sec`,
`DropletDiameter_mm`, `ZeroedDropletXpos_mm`, `ZeroedDropletYpos_mm`), the time column starts before
zero and advances in steps of about 1/30 s, the diameter is in mm. The FLEX ingest stores that same
column as `time_s` with the origin "NASA DropletData processed time base (phase entry not
adjudicated)", and every droplet release refuses a prefix with any other origin string. Handing a
CFI series to the releases under the FLEX origin is therefore an equivalence of two NASA products of
the same rig (MDCA) and the same processing; it is plausible and not adjudicated against the
igniter stamps of the timeline. No value of the file is described here.

## (b) Conditions, and how far they can be trusted

| Condition | Source | Values | Trust |
|---|---|---|---|
| Fuel | `Master Sheet` `Fuel` | n-dodecane 80; blends `[.75]/[.25]` or `[.60]/[.40]` n-dodecane/iso-dodecane 103; farnesane 44 | stated per test; the blend fractions do not say mass or volume |
| Ambient composition | `Oxygen`, `Nitrogen`, `Helium` | O2/N2 0.21/0.79 on 132 tests, 0.17/0.83 on 39, 0.21/0.76 on 4 (sum 0.97: FLEX-1219 to 1222), O2/N2/He 0.21/0.50/0.29 on 26, 0.21/0.20/0.59 on 26; no CO2 | nominal fill values; the four rows summing to 0.97 are unexplained |
| Pressure | `Pressure` (text, "1 atm") | 0.5 (19), 0.75 (18), 1 (85), 1.25 (19), 2 (18), 3 (52), 5 atm (16) | nominal; the mean chamber pressure in the 120 s before the igniter came on (timeline table, 91 test points off the test days) is within 10 % of it everywhere, within 3 % on 85 of 91 (worst FLEX-1323: 0.91 of nominal) |
| Support | `Fiber` | free 55, fibre 103, not stated 69 (all 44 farnesane rows and 25 blend or dodecane rows) | stated where given; no other condition file states it (no fibre flag in the CFI settings logs; needle positions do not separate free from fibre); test sheets and operator notes may say, both carry outcomes |
| Initial diameter | sheet `DATA`, column `D0 ( mm )` alone | 112 of the 189 tests off the test days have one; 3.79 to 6.43 mm | a measured initial state, used as a condition by the project's catalogue; the forward does not read it (it estimates D0 from the prefix), so it is only a domain check here |
| Igniter | sheet `Igniter`; timeline header | 6 to 14 W; 3.7 to 15 s on | settings; the timeline (the rack's own record) and the sheet agree on 82 of the 91 test points read; on 8 the on time differs by 1 to 5 s and on one the power by 0.9 W, so the timeline is the better source |
| Record length | timeline header `Burn to Completion` | 53, 60, 63, 65, 70, 80, 85 or 90 s (91 test points); images run from 2 s before the igniter came on to 1 s after igniter-on plus that time | a planned setting, not the length of the burn |
| Dispensing | timeline stamps | the dispense and stretch stamps are empty on all 91 read | not informative |

The timelines of 33 test points begin with rows of earlier test points of the same session (and
`F139A01.txt` with rows stamped on 15 May, another day); those rows were skipped by their time stamp.

## (c) Each candidate model, by conditions

The releases gate a prefix on: the fuel text against their label; resolved conditions x_o2, x_co2,
x_he and pressure; the declared domain (c_o2 = (x_o2 − 0.21)/0.05 in [−2.2, 4.8], c_co2 = x_co2/0.2
in [0, 4], c_he = x_he/0.2 in [0, 3], c_logp = ln(p / 1e5 Pa) in [−1, 1.5], c_logd0 = ln(D0 / 2 mm)
in [−0.9, 1.26]); at least two (wsp2) or five (wsp5 candidates) valid diameters in [0, cutoff]; the
diameter at the cutoff above a floor; cutoff below the 45 s horizon. Fibre support is **not** a
prefix condition: the releases declare "single free-floating droplet" as applicability and cannot
refuse a fibre test themselves, so a registered test has to exclude fibre and unstated support by
its own inclusion rule. The counts below apply the fuel, the domain and that inclusion rule; the
measurement and horizon gates need the series and were not evaluated.

| Model | Answers, all 227 | Answers, train + validation (189) | Answers with a series, train + validation (21) | Why the rest are refused |
|---|---|---|---|---|
| `flex_heptane.wsp2` D2, `flex_heptane.wsp5` (frozen) | 0 | 0 | 0 | fuel, on every test |
| `flex_methanol.wsp2` D3, `flex_methanol.wsp5` (frozen) | 0 | 0 | 0 | fuel, on every test |
| `wsp5/octane`, `isooctane`, `decane`, `ethanol` (not assembled) | 0 | 0 | 0 | fuel, on every test |
| `wsp5/dodecane/w0-fable` (not assembled) | 30 | 25 | 7 | fuel on the 103 blends and 44 farnesane tests; fibre (103) or support not stated (69); c_logp on the 16 tests at 5 atm |
| `wsp5/farnesane/w0-fable` (not assembled) | 0 | 0 | 0 | support not stated on all 44 farnesane tests; fuel on the rest |
| any model, blends (103 tests) | 0 | 0 | 0 | no forward carries a composition (`phys_droplet_fuels.md` 3.2) |

Every other condition is inside the domain: oxygen 0.17 to 0.21 (c_o2 −0.8 to 0), helium up to 0.59
(c_he 2.95, just inside 3.0), pressure 0.5 to 3 atm (c_logp −0.68 to 1.11), D0 3.79 to 6.43 mm
(c_logd0 0.64 to 1.17). The 7 answerable processed tests: FLEX-1206, 1239, 1240, 1250, 1264 (train)
and 1221, 1223 (validation); 0.5, 1 and 1.25 atm, O2 0.21, one with 0.29 helium.

**A surrogate arm**, as in section 11.1's arm B, is the only way a frozen model answers CFI: the
n-heptane release handed n-dodecane tests under its own fuel label. The gates are the same, so it
would answer the same 7 processed tests (and the 30 by conditions). It would be a declared
extrapolation across a 2.5-fold change of molar mass and a different cool-flame regime, not an
application in the domain.

**What limits any CFI test beyond the counts** (from `phys_droplet_fuels.md` 3.1, for the dodecane
candidate): reignition, a separate warm-flame stage and a cool flame lit directly by a weak igniter
have no state in the forward; the igniter settings (6 to 15 W, up to 15 s on) vary far more than in
FLEX; the forward's horizon (45 s) is shorter than a large n-dodecane burn, which matters for cutoffs
near the end of the record but not for cutoffs 1, 3 and 5 s with a 10 s horizon.

## What is unknown

- The support of 69 tests (all farnesane). Without it the farnesane candidate has no test.
- Whether `DropletTime_sec` = 0 marks the same event in CFI as in FLEX (igniter on, ignition or
  needle retraction). Both are NASA's processed product; no adjudication was made.
- The meaning of the four compositions that sum to 0.97, and whether the blend fractions are by
  mass or by volume.
- Whether the 24 processed series are the complete processed set (the `Analyzed Data` folder holds
  summary PDFs for FLEX-1206 to 1264 and FLEX-1393 only).
- The release label the dodecane candidate will check its fuel text against, once assembled.

## Disclosures (what was seen that a reader of outcomes would want to know)

- While printing the first rows of every sheet of the campaign spreadsheet to learn its layout, the
  first two data rows of sheet `DATA` (FLEX-1206 and FLEX-1207, outcome columns such as burning-rate
  constants and extinction diameters) and the two `Comments` cells of the same tests were printed to
  this agent's console. Both tests are on train day 2017-03-09. The values are not written anywhere
  and are not used; no other outcome cell of the spreadsheet was read.
- Opening the one permitted series (FLEX-1206, train) printed its first rows, all before time zero.
- `F139A01.txt` (train) begins with one pre-ignition table row stamped 15 May (a test day): chamber
  pressure and zero radiometer readings. It was printed once while checking the parser, before the
  parser was restricted to the 120 s before the test point's own ignition.
- Nothing of the 38 test-day tests was read beyond their `Master Sheet` row (conditions) and the file
  catalogue.
