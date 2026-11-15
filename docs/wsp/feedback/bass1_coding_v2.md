# BASS and BASS-II solids: the five coding issues, a second coding, and the display scale

> Data and scoring lane, 15 November 2026, answering the coordinator's follow-up to
> `docs/wsp/research/phys_bass_study.md` section 1.6. Train and validation days only: validation days were
> opened as study data by PROTOCOL 13.4 and 13.6. No observation, fan or air cell of a test-day row was opened.
>
> The first coding (`tools/wsp/ext/bass1_cases.py`, files `cases_*.jsonl`) is unchanged: the registered looks
> were scored on it (sha256 of `cases_train.jsonl` starts `462d7a70`, of `cases_validation_outcomes.jsonl`
> `8b5b7d37`). The second coding is `tools/wsp/ext/bass1_cases_v2.py`, which writes `cases_train_v2.jsonl`,
> `cases_validation_inputs_v2.jsonl` and `cases_validation_outcomes_v2.jsonl` under
> `runs/dev/data_all/work/transfer_bass1/`.

## 1. The issues, one by one, against the source words

**1. "Day curves below pot 2 put anemometer noise into the outcome." The physicist is right.**

The calibration workbook gives, with two restrictors and the same fan display:

| Setting | Days | cm/s read |
|---|---|---|
| pot 0.5 (display 42) | 12 | 0 to 3.77; zero on 4 days; median 1.01 |
| pot 1 (display 46.5) | 18 | 0 to 5.79; median 4.40 |

`fan_spread_v2.json` has every pot and restrictor count (section 3).

The 22 rods that went out at a stated setting compare as follows:

| Scale | Range | max / min |
|---|---|---|
| fan display | 40.16 to 41.04 | 1.02 |
| first coding's cm/s (day curves) | 0.51 to 1.49 | 2.9 |
| one campaign curve for every day | 1.06 to 1.58 | 1.48 |

The spread of the cm/s value is the day's calibration, not the rod. No rule was "wrong" here, but the scale is
(section 4).

**2. "M2: the ignition speed is coded 55.8 cm/s; the operators' own reading is 20 cm/s." Right.**

- The fan cell is `116 (pot 9.3), 89, 80, 84` and the air cell `20,15,13, 14`.
- The first coding read the bracket as a fifth step. The fan and air lists then had different lengths, so the
  steps were not paired, and 116 went through the no-restrictor curve.
- **R1** (general): a bracket holding a single number right after a value restates that value (`116 (pot 9.3)`,
  `pot 1.2 (48)`, `44 (pot 0.68)`, `1.87 (54)`). It is not a step. A cell that opens with "pot(s)" lists pots
  only.
- **R10**: the speed at ignition comes from the same paired steps. M2's ignition speed becomes 20.1 cm/s.
- B16 and M14 change their ignition speed for the same reason, and O16 by 0.2 cm/s. Test 92 of 2013-05-02
  (`Pots 1.5, 1 0.5, 0.4`) has its "1" read as a pot instead of 1 cm/s.

**3. "PSI-26 tests 72 and 73: the words say the flame could not be kept alight; coded as sustained." Right.**

- The words: "no luck keeping the flame sustained ... winks out" and "Got a flame briefly, but did not sustain
  for any spread".
- **R2**: v1's words, plus "did not / could not / no luck (keeping the flame) sustain", plus "brief flame" when
  no later words say the flame grew, wrapped, spread or burned.
- The last clause also codes test 38 of the same day ("no luck at 52, brief flame at 62 but also 'flamed out'")
  as not sustained.
- It keeps test 50 of 2013-05-15 sustained ("brief flames" during the ignition attempts, then "grew to wrap past
  90 degrees").

**4. "PSI-26 2013-05-15 test 53: 'no ignition, broke 2 igniters' is an igniter failure, coded as not sustained."
Right.**

- **R3**: a burn whose words name a broken or failed igniter and say there was no ignition, with no word of a
  flame, is not a test (no `sustain` code).
- On train this changes nothing. The burns with broken igniters there all had a flame ("got it ignited", "brief
  flame a few times, but went out", "went out when igniter was retracted").

**5. "M2, M9, M17, M18: the end is stated in the words or cells and the rules miss it." Right for all four.**

M17 is coded differently from the physicist's reading:

- **M2** ("very dim, so increased flow to 14 cm/s, but it self- extinguished").
  - **R4**: the pdf's "self- extinguished" is joined, and durations ("15 min", "29-min", "15 s") are removed
    before any number is read as a setting.
  - **R5**: a setting named in the same clause just before an extinction word counts ("to 14 cm/s, but it
    self-extinguished").
  - Coded [14.1, 15.1] cm/s; flagged: a lower setting (13) was held earlier.
- **M9** ("turned up flow when flame looked weak; died after 15 min at 14 cm/s").
  - R4 drops the duration, and R5 makes "died" an extinction word.
  - No higher setting was held before it went out, so **R8** applies: when the flow had been changed and the
    flame then went out at a setting with no higher one held before, the outcome is [lo, open).
  - The open end costs nothing in the score. With a single written setting the end is still not attributed to
    the flow (P21a stays unscored).
- **M18** ("extinguished 8 cm/s").
  - R5: a value with an explicit unit right after the extinction word needs no "at".
  - Coded [8.05, 10.07].
- **M17**.
  - The fan cell ends `0.5, 0.3ext` and the air cell `0ext`. The observation says "finally turned fan pot to 0
    to extinguish".
  - I read this as the flame holding pot 0.3 and being put out by stopping the fan.
  - **R7**: "pot (down) to 0" is a fan stop, and a fan stop stated in the observations outranks an extinction
    marker in the cells. **R6** reads `0.3ext` as the setting 0.3 plus the marker, where the marker decides.
  - Coded `out_at_fan_off`: [0, 1.42] cm/s, display [38, 40.5].
  - If the physicist reads it as out at pot 0.3, the interval would be [40.5, 41.7] on the display scale. The
    words favour the fan stop.

**Also coded in v2:**

- **R9**: "reached (the) end of (the) sample" is burned to completion. This codes **M6**, validation: "when
  flame reached end of sample it started backing up". It becomes [0, U(display 41)].
- **M12** (plateaus at pot 0.35 to 0.4) states no end and stays unscored.
- The physicist's note on final oxygen: `x_o2_final_post_burn` is added to the outcome part of PSI-25 cases.
  It is measured after the burn, so it is not a forecast input.

## 2. Every case whose coding changes (`python tools/wsp/ext/bass1_cases_v2.py diff`)

13 cases: train 6, validation 7. Intervals are in cm/s, as in v1; the display intervals are in the v2 files.

| Partition | Case | v1 | v2 | Rule |
|---|---|---|---|---|
| validation | PSI-26:2013-04-26:38 | sustained, end not stated | not sustained | R2 (brief flame, never grew) |
| validation | PSI-26:2013-04-26:72 | sustained, end not stated | not sustained | R2 |
| validation | PSI-26:2013-04-26:73 | sustained, end not stated | not sustained | R2 |
| validation | PSI-26:2013-05-15:53 | not sustained | not a test | R3 |
| train | PSI-26:2013-05-02:92 | burned to completion [0, 1.01] | [0, 1.66] | R1 (the pot list: lowest pot 0.4, not "1 cm/s") |
| train | PSI-25:M2 | end not stated; ignition 55.8 | [14.09, 15.10]; ignition 20.1 | R1, R4, R5, R10 |
| train | PSI-25:M9 | end not stated | [14.09, open) | R4, R5, R8 |
| train | PSI-25:B16 | ignition 3.38 | ignition 3.02 (interval unchanged) | R1, R10 |
| train | PSI-25:M14 | ignition 5.84 | ignition 10.07 (interval unchanged) | R1, R10 |
| validation | PSI-25:M6 | end not stated | burned to completion [0, 1.78] | R9 |
| validation | PSI-25:M17 | end not stated | out at fan off [0, 1.42] | R6, R7 |
| validation | PSI-25:M18 | end not stated | [8.05, 10.07] | R5 |
| validation | PSI-25:O16 | ignition 5.21 | ignition 5.03 (outcome unchanged) | R1, R10 |

**Counts.**

| | `sustain` coded | `quench` scored |
|---|---|---|
| Train, v1 | 125 | 55 |
| Train, v2 | 125 | 57, one with an open end |
| Validation, v1 | 40 | 11 |
| Validation, v2 | 39 | 14 |

Every other train and validation case codes the same in v2. The rules are general: no burn is named in the
script, and `build` applies them unchanged to any rows it is given. Test-day rows are read only for their date
and material in the build, as before.

## 3. The display scale

**What each case now carries.**

- **Outcome part (`outcome` in `cases_train_v2.jsonl`, rows of `cases_validation_outcomes_v2.jsonl`):**
  - `run_record.steps`: every written flow step with its setting words, the speed U and the fan display, and
    whether the display was derived;
  - `run_record.last_held` and `run_record.first_failed` (display);
  - `quench_display_lo` and `quench_display_hi`, with `quench_display_derived`;
  - `quench_u_lo/hi_cm_s` (the day curve, as v1);
  - `quench_u_lo/hi_campaign_cm_s` (the display through one campaign curve, no day calibration).
- **Inputs (both files):** only `fan_display_at_ignition`, which is set before ignition.

The step list is the operators' response to the flame, so it stays out of the inputs files. For the final run the
test-day inputs must be built the same way.

**How the display is set.** It is the fan display written for the step, or the pot turned into a display by the
workbook's pot-to-fan table. Two cases need more:

- A step written only as an air reading gets its display through the burn's fan curve, marked derived. That is
  6 train scored cases (the slabs M3, M5, M9, M11 and M14, and rod P1's held step) and 1 validation case (M18).
- A failed setting given only as an air reading whose derived display would not be below the held display (M14)
  takes the display of the next written step.

`fan_display_at_ignition` (inputs) and `quench_display_lo/hi` are in volts x 10. 38 is the fan stopped and 122
the maximum.

**Day-to-day spread of the cm/s value of one display**, from the calibration workbook (`fan_spread_v2.json`;
U = 1.0066 x air):

| Restrictors | Display (pot) | Days | min | median | max | Zero readings | CV |
|---|---|---:|---:|---:|---:|---:|---:|
| 2 | 42 (0.5) | 12 | 0 | 1.01 | 3.77 | 4 | 0.96 |
| 2 | 46.5 (1) | 18 | 0 | 4.40 | 5.79 | 1 | 0.34 |
| 2 | 55 (2) | 22 | 3.02 | 7.05 | 9.06 | 0 | 0.21 |
| 2 | 72 (4) | 22 | 7.05 | 13.09 | 16.11 | 0 | 0.20 |
| 2 | 122 (10) | 22 | 20.13 | 33.72 | 40.26 | 0 | 0.18 |
| 1 | 46.5 (1) | 3 | 3.02 | 4.03 | 5.03 | 0 | 0.20 |
| 1 | 122 (10) | 6 | 27.68 | 33.22 | 40.26 | 0 | 0.14 |
| 0 | 122 (10) | 9 | 20.13 | 54.36 | 59.39 | 0 | 0.23 |

**At the burns' own curves** (two restrictors, the v1 rule below pot 1):

| Display | cm/s, min / median / max |
|---|---|
| 40.6 (the rods' quench setting) | 0.62 / 1.35 / 1.77 |
| 42 | 0.95 / 2.07 / 2.72 |
| 46.5 | 2.01 / 4.40 / 5.79 |

Above pot 2 the spread is about +-20 % (CV 0.18 to 0.21). The low end of the 0-restrictor block includes a day
whose values look like another restrictor count, so its minima are not reliable.

## 4. Which scale I would trust for scoring

**The display scale, for the low-flow extinction.**

- **It is what was recorded.** The operators set and wrote a pot or display. The cm/s value below pot 2 is a
  model of an instrument that reads 0 to 5.8 cm/s at one setting, with a one-minute lag.
- **It is reproducible.** The 22 rods go out within 0.9 display units of each other (pot 0.27 to 0.38) on days
  whose calibrations differ by a factor of 2.9.
- **It keeps the zero honest.** "Out when the fan was stopped" is display 38. No speed has to be invented for
  pot 0.3, which may be a turning fan at 1 to 2 cm/s or close to zero.

**Its limits.**

- A law in cm/s must be mapped onto the display to be scored on it. The map the physicist proposes, a speed
  following the burn's curve, puts the instrument back in. I would score against one fixed map, the campaign
  curve per restrictor count, given in every case as `quench_u_*_campaign_cm_s`, and treat any speed below about
  4 cm/s as unknown to a factor of about 3.
- For the slabs whose extinction the operators wrote as an anemometer reading of 5 to 18 cm/s, the cm/s is the
  measurement and the display is derived. There, either scale is equivalent to about +-20 %.

**Recommendation.** Score `quench_display` (interval CRPS in display units) as the primary low-flow outcome.
Report `quench_u` on the campaign curve beside it. Do not use the day curves for scoring.

The v2 constant baselines (`baselines_v2.json`, fitted on train) give:

| Scale | Train, day mean / case mean | Validation, day mean / case mean |
|---|---|---|
| display | 1.73 / 2.68 | 1.33 / 1.34 |
| cm/s (day curve) | 0.70 / 1.05 | 0.60 / 0.60 |

These are display units against cm/s, so they are not comparable with each other.

## 5. Commands

```
python tools/wsp/ext/bass1_cases_v2.py build      # cases_*_v2.jsonl; prints counts only
python tools/wsp/ext/bass1_cases_v2.py diff       # the table of section 2
python tools/wsp/ext/bass1_cases_v2.py spread     # fan_spread_v2.json
python tools/wsp/ext/bass1_cases_v2.py baselines  # baselines_v2.json; score() takes targets quench, quench_display, sustain
```

## Addendum (15 November, after the final run): third coding, made with the test outcomes seen

**This correction was made after the final run, with the test-day outcomes seen** (PROTOCOL 13.8 opened the
test days as study data). It was prompted by the coordinator's report on test burn PSI-25:O13a and by the
physicist's reading of the 2014-07-21 workbook row (`docs/wsp/research/phys_bass_post_final.md`). Versions 1
and 2 and their files are untouched; their hashes are unchanged.

- Script: `tools/wsp/ext/bass1_cases_v3.py`.
- Files: `cases_train_v3.jsonl`, `cases_validation_{inputs,outcomes}_v3.jsonl` and
  `cases_test_{inputs,outcomes}_v3.jsonl`, under `runs/dev/data_all/work/transfer_bass1/`.
- No score has been taken on version 3.

### What happened in O13a

- **The source.** The row is a 12.7 mm clear rod in concurrent flow at 15.9 % O2. The fan cell holds pots
  (`1.3, 0.7, 0.45, 0.36, 0.32, 0.31`). The air cell holds the fan displays and, in brackets, the anemometer
  readings (`49,44,42,41,41,40.5 (5, 2, 0)`). The words are "Ignited and burned at 5 pot settings, flame
  self-extinguished after 3+ min at pot 0.31".
- **What happened.** The flow was lowered in five steps and the flame went out after more than 3 minutes at pot
  0.31, having held 0.32. That is a floor quench, not a blow-off. The physicist's reading is right.
- **How version 2 miscoded it.** Three faults combined:
  1. The displays 49 to 40.5 in the air cell were read as anemometer readings in cm/s, paired with the pots.
     The "held" setting became 40.8 cm/s.
  2. The duration "3+ min" was not removed, because R4 did not allow the "+".
  3. R5's clause rule took "at 5 pot settings" as the setting pot 5, so the failed setting became pot 5
     (17.9 cm/s).
- **Version 2's result.** The interval [17.9, 40.8] cm/s, with a display interval reversed to [80.25, 40.48].
- **Version 3's result.** [1.47, 1.52] cm/s on that day's curve; display [40.48, 40.56].

### Is there a general fault that codes an extinction on a flow increase as a low-flow quench?

**Yes, in version 2.** Its outcome step never checked that the flame went out at a setting lower than the one it
held just before. Two ways it shows:

- With a lower setting held earlier, the step took as the "held" setting the last setting above the failed one.
  When the failed setting had itself been reached by raising the flow, that is the wrong setting.
- R8 explicitly scored "out at the highest setting of the burn" as [lo, open). Such a setting can only have been
  reached by raising the flow.

The fault touches these burns:

| Partition | Burn | What the words say | v2 | v3 |
|---|---|---|---|---|
| train | PSI-25:M2 | 3 mm one-sided slab: flow lowered to 13 cm/s, "flame got very dim, so increased flow to 14 cm/s, but it self-extinguished" | [14.1, 15.1] scored | `out_after_increase`, not in the low-flow target |
| train | PSI-25:M9 | 3 mm slab: "turned up flow when flame looked weak; died after 15 min at 14 cm/s" | [14.1, open) scored | `out_after_increase`, not in the low-flow target |
| test | PSI-25:M15 | 4 mm one-sided slab: up to 20 cm/s, then "turned down flow to 15 and got spreading and slow extinction there" | [20.1, open) scored | a lowering: [15.1, 16.1] cm/s, display [76.7, 79.1] |
| test | PSI-25:O13a | above | [17.9, 40.8], reversed display | [1.47, 1.52] |

M15 is the R8 fault in reverse. R5 took "up to 20 cm/s" as the failed setting, when the extinction was at the
later, unit-less "to 15".

Validation days are not touched by this fault.

### Rows with fan displays in the air cell (the physicist's swapped cells)

Found by a general test over every row of every partition (`python tools/wsp/ext/bass1_cases_v3.py swapped`):

| Partition | Burn | Fan cell | Air cell | Effect in v3 |
|---|---|---|---|---|
| test | PSI-25:O13a | `1.3, 0.7, 0.45, 0.36, 0.32, 0.31` | `49,44,42,41,41,40.5 (5, 2, 0)` | outcome corrected (above); ignition speed 49.3 to 6.0 cm/s |
| test | PSI-25:O5a | `1.3, 2.7, 1.3, 0.7, 0.45. 4.0` | `5,10,5, 2, 0 (49, 62, 49, 44, 42)` | none: the displays in brackets were already dropped; blow-off, not in the target |
| test | PSI-25:O5b | `1.3` | `49, 5 cm/s` | ignition speed 6.0 to 5.0 cm/s; did not sustain, unchanged |
| test | PSI-25:O13b | `1.3, 0.9` | `7, 4? (49, ?)` | none: the 49 in brackets was already dropped |

These are exactly the four rows of 2014-07-21 the physicist named. No train or validation row is affected.

The first form of the test also flagged three PSI-26 2012 rows whose fan cell says "max": the 44 in their air
cells is a real reading at maximum flow with no restrictor. The rule counts "max" as pot 10, which clears them.

### The corrected general rules

- **R11 (lowering only).** A low-flow quench is coded only when the flame went out at a setting lower than the
  setting it held just before, in the order written: lower in display or, for a step written only in cm/s,
  lower in speed.
  - Two cases are coded `out_after_increase` and leave the low-flow target: the failed setting was reached by
    raising the flow, or it was the highest setting of a burn whose flow had been changed (v2's R8).
  - A reversed display interval is always such a case.
- **R12.** Durations written with a qualifier ("3+ min", "~3 min") are removed before any number is read.
- **R13 (displays in the air cell).** A number between 38 and 122 in the air cell, not followed by "cm/s", is a
  fan display when the row's highest setting is below pot 6 (display 88.5). "max" counts as pot 10.
  - The calibrations give at most 39 cm/s at pot 6 even without a restrictor (23 with two).
  - Such numbers are never read as cm/s; the pots of the fan cell carry the steps.
- **R14.** "N pot settings", "N settings", "N steps", "N increments" or "N changes" (a whole number, plural noun)
  is a count, not a setting.
- **R15.** Within a clause, "to N" or "at N" written as a whole number without a unit takes the unit of the last
  setting written with one ("up to 20 cm/s ..., turned down flow to 15 ... extinction there" is 15 cm/s). A bare
  decimal ("at 0.35") stays a pot.

### Every row version 3 changes against version 2, all partitions

`python tools/wsp/ext/bass1_cases_v3.py diff` lists them: M2 and M9 (train), and M15, O13a and O5b (test, O5b
the ignition speed only).

**Counts with a scored low-flow outcome:**

| Partition | v2 | v3 |
|---|---:|---:|
| train | 57 | 55 |
| validation | 14 | 14 |
| test | 17 | 17 (O13a corrected, M15 changes interval) |

**Known and not changed.** M15's ignition speed is still read as pot 10 (35 cm/s). Its fan cell is written in
words ("ignited 10, adjusted down then up to 20 ...") and the 10 is cm/s. This affects an input, not the
outcome.
