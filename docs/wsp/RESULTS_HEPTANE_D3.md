# n-Heptane droplets: D3 of `flex_heptane.wsp2`

15 November 2026. Author `claude-fable-5-2`. Registered in [PROTOCOL.md](PROTOCOL.md) section 13.7.
The candidate is **data-informed**. Sections 1 to 6 were written and committed before any validation
score of D3 existed; section 7 was added after the one registered look.

## 1. The question, and what was already known

The adopted release D2 (`sha256:edbad4d6…edef`) loses to the detailed simulation of Farouk and Dryer
(2014) on FLEX-035, a 3.85 mm droplet in air ([RESULTS_POINT_METRICS.md](RESULTS_POINT_METRICS.md)).
Three suspects were named there: the hot flame ends late and its end is spread wide; the droplet then
burns too fast in the cool flame; the point forecast is the median of a mixture.

**Exposure.** FLEX-035 (`PSI-69:181F001:2009-11-27`) is a validation run. Its 0 to 38 s trajectory was
studied for that comparison, and I read the diagnosis: hot flame lost near 7 to 9 s, cool flame at
0.37 to 0.40 mm²/s, stall at 1.27 mm. I did not open its series. It is recorded as **studied, not used
to set numbers**: everything below comes from the train partition, and the five numbers of the
candidate were typed on 14 November, before that study. What the study did do is decide which laws I
looked at again. That is a selection effect and the validation cases of FLEX-035 are left out of the
look for it.

## 2. What the train runs say

Tables: `python tools/wsp/analysis/flex_heptane_train_rates.py main` (sheet facts and windowed rates),
`… series 2 34 2 <run>` (rates through the cool flame), `runs/dev/fb_train_flex_heptane_d2.json` (the
laws of D2 per run). Train only; nothing fitted.

### 2.1 Does the pattern of FLEX-035 appear on train?

Large droplets in atmospheres close to air (20 to 21 % oxygen, at most 15 % diluent other than
nitrogen):

| Run | Atmosphere | D0 mm | Hot flame | After |
|---|---|---|---|---|
| 209F001 | air, 1 atm | 3.36 | oscillated for seconds, burned on to 17 s | no cool flame seen |
| N01H101 | air, 1 atm | 3.56 | oscillated, burned on to 24 s; drifting fast | — |
| N01H102 | air, 1 atm | 3.58 | oscillated, burned on to 24 s; drifting fast | — |
| H19H101 | 21 % O2, 15 % He by its record ("air" in its narrative) | 3.54 | lost at 7.6 s | cool flame 0.47 falling to 0.41 mm²/s, knee at 1.63 mm |
| 194F001 | 20 % O2, 0.72 atm | 3.56 | lost at 7.9 s | left the view; back at 2.39 mm, stalled |
| C01H302 | 20 % O2, 5 % CO2 | 3.18 | lost at 8.0 s | left the view |

So yes, in three runs of three sessions: the hot flame is lost at 7.6 to 8.0 s and a cool flame
follows. None of them is a droplet in plain air: the three air runs above 3.3 mm kept their flame.
H19H101 is refused by the forward because its sheet and table disagree on the atmosphere. Its
hot-flame rate (0.53 to 0.59 mm²/s at 3.54 mm) is 12 to 18 % above the air law, which is what 15 % of
helium gives, and the next run of that fill (2.82 mm) burned at 0.62 to 0.66; I read it as the helium
atmosphere of its record.

### 2.2 The end of the hot flame

Clock at the loss, ln(measured / D2's median), for the runs that lost the flame to radiation:

| Group | Runs | Residual |
|---|---|---|
| Nitrogen or CO2, 1 atm | 13 in 8 sessions | −0.13 to +0.13 for twelve; +0.23 for a 1.8 mm droplet at 14 % O2 |
| Near-critical (0 to 10 % above D_crit), nitrogen or CO2 | C08H101, C02H201, C01H302, 194F001 | +0.06, +0.13, +0.03, −0.09 |
| CO2 of 25 % and more at 0.7 atm | C09H101, C09H501, C10H101 | +0.20, +0.35, +0.39 |
| Helium (15, 30, 35 %) | H19H101, H12H101, H13H201, H13H101 | −0.27, −0.22, −0.34, −0.39 |

- For a 3.85 mm droplet in air D2 gives a critical diameter of 3.7 mm, a radiative class of 0.74 and
  a clock median of 10.3 s. Train neither confirms nor contradicts that: no train droplet in air is
  above 3.58 mm, and the two largest did not lose their flame.
- What separates the near-critical runs is not in a prefix of diameters. Rate over law in the prefix:
  0.91 to 1.03 for those that lost the flame, 0.88 to 1.10 for those that kept it. The two air
  droplets that kept the flame at 3.56 and 3.58 mm drifted at 5.2 and 3.5 (profile units) against
  1.2 to 1.8 for the others, and of the runs that did lose the flame the three that drifted most lost
  it late (+0.20 to +0.39). Drift is not among the prefix observations (the records hold the liquid
  diameter only). The fifth-generation dossier reached the same place ("igniter energy and drift
  decide it").
- The clock is drawn about three times wider than the train residual at 1 atm. Widths are not set from the
  runs a law was typed from (PROTOCOL 8 and 12.3), and no blind error of the clock exists.

Nothing typed. The mechanism that is missing is an input (drift of the droplet in the prefix), and
that is a change to the shared records, not a weight.

### 2.3 The cool-flame rate

Steady stage, mm²/s (mine; the independent table of 14 November in
[feedback/verify_phys_flex_heptane.md](feedback/verify_phys_flex_heptane.md) agrees within about 0.01):

| Run | O2 / CO2 / atm | D0 mm | Measured | D2 law | D3 law |
|---|---|---|---|---|---|
| N03H101 | 0.15 / 0 / 1.0 | 2.55 | 0.31–0.32 | 0.314 | 0.340 |
| N02H101 | 0.16 / 0 / 1.0 | 3.34 | 0.34 | 0.344 | 0.353 |
| N02H203 | 0.16 / 0 / 1.0 | 2.52 | 0.33 | 0.326 | 0.338 |
| C04H101 | 0.17 / 0.20 / 1.0 | 3.76 | 0.35 | 0.349 | 0.357 |
| C04H201 | 0.17 / 0.20 / 1.0 | 2.90 | 0.35 | 0.331 | 0.344 |
| 301F001 | 0.18 / 0 / 1.0 | 3.43 | 0.36 | 0.375 | 0.355 |
| C03H501 | 0.18 / 0.15 / 1.0 | 3.74 | 0.35 | 0.369 | 0.360 |
| C03H601 | 0.18 / 0.15 / 1.0 | 2.40 | 0.34 | 0.337 | 0.334 |
| C02H101 | 0.19 / 0.10 / 1.0 | 3.40 | 0.36 | 0.380 | 0.354 |
| C02H201 | 0.19 / 0.10 / 1.0 | 2.48 | 0.31–0.32 | 0.357 | 0.337 |
| C09H501 | 0.23 / 0.25 / 0.70 | 3.32 | 0.36 | 0.413 | 0.351 |
| C08H101 | 0.24 / 0.20 / 0.70 | 3.53 | 0.34 | 0.441 | 0.354 |

Twelve runs, nine sessions: 0.31 to 0.36, no trend beyond 0.02 in oxygen, CO2 or pressure below
1 atm. Size: three same-atmosphere pairs give exponents 0.16, 0.10 and −0.05. D2 carried the
literature scalings of W0 (+0.2 per 5 % O2, +0.1 per e-fold of pressure, +0.2 per e-fold of D0) and is
right at 15 to 17 % and 11, 15 and 29 % high on three runs above. The physical reading, from the
fifth-generation dossier: a cool flame burns at a fixed flame temperature, so its rate follows the gas
conductivity and not the oxygen. Outside the table: H19H101 (15 % He) 0.43 against 0.41 by the D3 law;
the one run at 2 atm 0.47 to 0.52 against 0.49 (D2: 0.37); at 14 % O2 the flame does not hold.

### 2.4 Where D2's long forecasts go wrong on train

D2 from a 1 s prefix to the end of the record, on runs that lose the hot flame on time: nRMSE 1.6 %
(301F001), 2.3 % (C02H101), 1.0 % (C08H101), 0.9 % (C01H302). The large train errors are the end of the cool flame (C02H201 burned through its plateau to
0.5 mm; the plateau law is within about 10 % elsewhere) and the two helium runs with no law for what
follows the flame. Around an early loss in a large droplet the rate sags for a few seconds (0.25 to
0.33 before it reaches 0.35), which leaves D2 15 to 60 µm low at 5 to 10 s in five runs of five
sessions; that needs a forward with a transition stage and was not attempted here.

## 3. The candidate

`flex_heptane.wsp2` D3 = D2 with the cool-rate law replaced; forward `droplet3.rs`, frozen.

| | |
|---|---|
| Files | `configs/models/flex/wsp2/heptane/d3-fable/` (`edits.wsp`, `meta.json`, `patch.json`, `release.json`) |
| Release | `sha256:b879b3da39a591b27e33926e4f03265879930b39a9f503ae2655be1e361d5b52` |
| Parent | D2 `sha256:edbad4d6395961030e0b17d9d5c4d40f2de902ddcbcbd0ec2a4f4ec9adbeedef` |
| Edits sha256 | `8a0cbe4efde0b550fbb47f3bcab399e944a7171007656718e305682e1a9e0422` (patch `sha256:cb583551…0b14`) |

| Typed | Meaning | D2 |
|---|---|---|
| `cool.k.b0 = -1.125` | 0.325 mm²/s at 2 mm (0.345 at 3 mm, 0.358 at 3.85 mm) | −0.97 (0.379) |
| `cool.k.skip = [0, 0, 0.2, 0, 0.15]` | no oxygen, CO2 or pressure term; helium +0.2 per 20 %; size exponent 0.15 | [0.2, −0.05, 0.14, 0.1, 0.2] |
| `cool.k.W1[0,3] = 1`, `b1[0] = 0`, `w2[0] = 0.5` | +0.5 per e-fold of pressure above 1 atm (one run) | none |

These are lines 27 to 31 of `configs/models/flex/wsp5/heptane/p0-fable/edits.wsp`, typed on 14 November
from the train runs. That candidate carried three more changes and was refused on validation as a
whole (47.2 against 46.1 µm); this law was never judged alone.

**Dropped.** The timing of the hot-flame end (2.2: no train support, a missing input). The point
forecast along the most probable path: under the model's own distribution the median has the least
expected absolute error and the 0.5 row has to stay the median for the band to be a distribution; at
the diagnosed condition the most probable path loses the hot flame at the clock median or later (10.3 s and more),
which is later than what was diagnosed, so it would not move that error. No forward was written.

## 4. Expectations, written before the candidate was assembled or run

(`runs/dev/hept_d3/expectations_20261115.txt`, local.)

| | Expected | Realised on train |
|---|---|---|
| E1 | 10 s horizon, 125 cases: mean CRPS about 2 µm lower, nearly all from the 2 atm run; median within 0.5; no session worse by more than 3; the session rule passes | 44.2 against 46.2; without the 2 atm run 43.4 against 43.1; median 21.7 against 21.8; worst session +2.7; rule passes (3 better, 14 level, 3 worse) |
| E2 | 39 s horizon: mean MAE of the median 3 to 8 µm lower. Better: 2009-07-01, 2010-03-11, 2011-02-21, 2010-06-15, 2011-08-17. Worse: 2011-09-01, 2011-08-29/30. Mixed: 2011-04-20 | 77.0 against 81.4 (without the 2 atm run 75.5 against 76.7). By session six of eight as expected. Wrong twice: 2011-02-21 came out 1.0 µm worse (C03H501 from a 5 s cutoff), 2011-09-01 came out better, on one case whose state at the cutoff flipped (N04H101, 300 to 18 µm), not on the law |
| E3 | Droplets that burn down in a hot flame: unchanged within 1 µm | unchanged |
| E4 | Air, 1 atm, 3.85 mm: 0.358 against 0.432 mm²/s; hot-flame end unchanged at 10.3 s | not checked by me (FLEX-035 is exposed) |

What I did not expect, and should have: outside the 2 atm run the forecasts do not gain on train. The
law is closer to the measured rates, but D2's fast cool flame was offsetting other errors. Where D2
loses the hot flame too early, the slower cool flame makes it worse (C09H501, +10 to +17 µm); where
the hot flame ends on time, better (301F001, −1 to −8; C02H101 at 10 s, −1 to −11); at 15 to 16 %
oxygen slightly worse (N03H101, N02H101, +3 to +8). One logged candidate; nothing was retyped after
the check.

## 5. The rule

PROTOCOL 13.7. One look at the validation days without FLEX-035, aggregates only. D3 replaces D2 if
(1) CRPS at 10 s is not higher, (2) the MAE of the median at 39 s is lower by at least 1.0 µm, and
(3) at 39 s more sessions are lower than higher. Otherwise D2 stays.

## 6. Is there n-heptane data nobody has read?

Surveyed from catalogues, splits and listings only (a read-only agent; I rechecked the counts of rows
2, 3 and 5 against `runs/dev/data_all/work/atlas/files.jsonl`, `atlas/splits/PSI-68-batch1.json` and the
split file). Nothing below was scored or opened.

| Set | Pure n-heptane free droplets | Measured series? | State |
|---|---|---|---|
| FLEX cohort (PSI-69, 2009–2011): train 49 runs / 22 sessions, validation 15 / 7, test 16 / 7 | 80 | yes (NASA) | all read: train per run; validation about seven aggregate looks and FLEX-035 per run; the test was spent in the final run of 12.11 and reopened for the point metrics |
| FLEX-2 (PSI-68) batch 1 | 10 on two train days (2013-03-08, 2013-03-14), 8 usable | yes (our measurement) | scored blind on 15 November, then study data |
| **FLEX-2 validation day 2013-03-28** (E087C01–C09) | **8** | **no**: about 10 GB of backlit frames, never downloaded | never read. No gas reading that day (nearest readings are air); the ambient would have to be registered as an assumption |
| FLEX-2 test days | 0 | — | the 18 heptane points of FLEX-2 are all on March 2013 days |
| **PSI-69 points of 2013 to January 2014**, absent from NASA's FLEX table and from every cohort | **23 with CO2** on 4 days (38.0 GB of backlit archives); 41 with xenon on 9 days (43.2 GB) | **no**: no test sheet, no processed diameters | never catalogued per test, split, measured or scored. Free or fibre-supported is not known. Xenon is not an input of any droplet forward |
| FLEX 2009–2011 heptane left out of the cohort | 33 records, at most about 6 free droplets | NASA series for most | fibre-supported or unresolved; not examined |
| CFI (PSI-39) | 0 | — | no heptane |
| FLEX-2 heptane mixtures (iso-octane, toluene) | not pure heptane | no | never downloaded |

**Answer.** No fair, unread, ready set exists: every pure n-heptane free droplet that has a diameter
series has been read or scored. What is unread has no series yet: 8 tests on one FLEX-2 day, and 23
CO2-diluted tests on four PSI-69 days of 2013, about 31 tests on 5 days and 48 GB before losses (the
measurement kept 8 of 10 heptane series in batch 1). Using them needs, in this order: a per-test
catalogue and a day split from conditions only, the ambient assumption for 2013-03-28, the release
and its rule registered, the frames measured blind with the frozen tool, one score. Five days give a
day-level check, not an interval. Note for whoever registers it: PROTOCOL 12.7 says the batch-1
heptane tests are on three days; the batch file holds two, and the third March day is this unread one.

## 7. The validation look

Taken once on 15 November, as registered: `wsp-dev feedback --partition
validation` at 10 s and 39 s, read through `partition_aggregate.py --skip PSI-69:181F001:2009-11-27`.
Aggregates only; I read no per-run and no per-session value. 36 common cases in 6 sessions (FLEX-035
was the only run of the seventh).

| | D3 | D2 | D3 − D2 | Sessions lower / level / higher |
|---|---|---|---|---|
| CRPS, 10 s, session-weighted (criterion 1) | 51.3 µm | 49.9 µm | +1.4 | 0 / 4 / 2 |
| MAE of the median, 10 s | 69.2 µm | 67.4 µm | +1.8 | 1 / 3 / 2 |
| CRPS, 39 s | 69.5 µm | 67.5 µm | +2.0 | 0 / 4 / 2 |
| MAE of the median, 39 s (criteria 2 and 3) | 101.6 µm | 98.6 µm | +3.0 | 0 / 3 / 3 |
| Coverage 50 % / 90 %, 10 s | 0.74 / 0.97 | 0.74 / 0.97 | | |
| Coverage 50 % / 90 %, 39 s | 0.71 / 0.97 | 0.72 / 0.97 | | |

Case means: CRPS 53.2 against 52.2 µm at 10 s and 75.8 against 74.3 at 39 s; MAE of the median 72.4
against 71.3 and 115.1 against 112.9.

**Not adopted.** All three criteria fail: D3 is 1.4 µm worse where it had to be not worse, 3.0 µm worse
where it had to be 1.0 better, and no session is better at 39 s. D2 stays the carried model.

How I read it. The differences are small (3 %), but they point one way on every table, and this is
the second time: P0 carried the same law to these days and came back 1.1 µm worse. Train had already
said that the forecasts do not gain outside one run. So the measured cool-flame rate is real and the
law is closer to it, but it is not what limits these forecasts; the fast cool flame of D2 offsets its
other errors, and removing one error of a pair makes the sum worse. The loss to Farouk and Dryer on
FLEX-035 is therefore not explained by a law that train can correct with the present inputs: of the
three suspects one was corrected and did not help elsewhere, one needs an input the prefix does not
carry (drift), and one was not a fault. Whether D3 does better than D2 on FLEX-035 itself is for the
lane that made that comparison; it would be an exposed, single, selected case, and by this rule it
cannot adopt D3.

Not done: the transition after an early loss in a large droplet (2.4), which train supports in five
sessions and which needs a forward with a transition stage; the drift of the droplet as a prefix
input; a judge on unread data (section 6).

## 8. Reproduce

```bash
ignis wsp compile --family flex_heptane2 --edits configs/models/flex/wsp2/heptane/d3-fable/edits.wsp --meta configs/models/flex/wsp2/heptane/d3-fable/meta.json --out configs/models/flex/wsp2/heptane/d3-fable/patch.json
```

```bash
ignis wsp assemble --family flex_heptane2 --patch configs/models/flex/wsp2/heptane/d3-fable/patch.json --binding configs/models/flex/wsp2/heptane/binding.json
```

```bash
wsp-dev feedback --release sha256:b879b3da39a591b27e33926e4f03265879930b39a9f503ae2655be1e361d5b52 --release sha256:edbad4d6395961030e0b17d9d5c4d40f2de902ddcbcbd0ec2a4f4ec9adbeedef --partition train --horizon-s 39 --out runs/dev/hept_d3/fb_train_d3_d2_h39.json
```

```bash
python tools/wsp/analysis/session_compare.py runs/dev/hept_d3/fb_train_d3_d2_h10.json runs/dev/analysis/flex_heptane_profile.json 0 1
```

`tools/wsp/reassemble.py` does not list D3 yet: the file holds another lane's uncommitted work. The
line to add is `("flex_heptane2", "configs/models/flex/wsp2/heptane", "binding.json", ["baseline-d2", "d2-fable", "d3-fable"])`.
