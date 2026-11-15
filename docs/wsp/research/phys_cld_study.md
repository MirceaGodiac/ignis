# CLD Flame (PSI-21): study of the development cases, and a third law for the loss of the flame

Author: `claude-fable-5-2`, 15 November 2026, under PROTOCOL 13.6. **Data-informed, in-sample**:
written from the 268 scored development cases (train and validation days), read case by case. No
test day was opened. Throwaway tables: `runs/dev/gen7/study/cld_*.py`. One candidate was run with
the logged tool (1 of 5).

## 1. What the data show

### 1.1 The flame is lost at one value of one group

For every case I took the schedule (`schedule_from_ignition_command`), the fuel fraction and the
coflow speed at the moment of loss, and the most severe point the script plans. With

```text
R = U_co / (k S_L(x)^2),    k = 10.2 s/m,    S_L from the W0 formula (Arrhenius on the adiabatic temperature)
```

the losses separate from the survivals almost without overlap. Counts over the highest `R` the
script reaches (`R` relative to the limit: 0.93 for methane, 1.8 for ethylene; "late" = lost after
3.5 s, "early" = lost at the igniter's retraction):

| highest R over the limit | methane: cases | lost late | lost early | ethylene: cases | lost late | lost early |
|---|---:|---:|---:|---:|---:|---:|
| below 0.54 | 53 | 0 | 0 | 134 | 0 | 4 |
| 0.54 to 0.75 | 22 | 0 | 1 | 4 | 0 | 0 |
| 0.75 to 0.86 | 10 | 0 | 1 | | | |
| 0.86 to 0.97 | 18 | 3 | 1 | | | |
| 0.97 to 1.08 | 1 | 1 | 0 | 2 | 2 | 0 |
| 1.08 to 1.4 | 3 | 3 | 0 | | | |
| 1.4 to 2.15 | 12 | 10 | 2 | 2 | 2 | 0 |
| above 2.15 | 7 | 6 | 0 | | | |

Above the limit 24 of 27 flames are lost in the ramps (two more at the igniter, and the one
"survivor" had its fuel cut 1.3 s after onset). Below 0.86 of the limit none of 223 is. The
transition is one tenth wide in ln, not the half that W0 and D1 assumed.

Where the flames were when they were lost (methane): on a coflow ramp at 30 % fuel when the coflow
reached 0.22 to 0.24 m/s (nine flames); on a dilution ramp at 0.20 m/s when the commanded fraction
reached 0.26 to 0.28 (eight); at 40 % when the coflow reached 0.42 to 0.47 m/s (two). The coflow a
flame stands is 0.235 m/s at 30 % and 0.445 m/s at 40 %: a ratio 1.89, where the burning speeds
are in the ratio 1.35. **The limit goes as the square of the burning speed**, not as its first
power. That is the classical critical velocity gradient of a rim-held flame, `g_c ~ S_L^2 / alpha`:
the coflow's boundary layer on the tube has a gradient `U_co / delta`, so `U_co,crit = (delta /
alpha) S_L^2`. With `k` = 10.2 s/m and `alpha` between 2e-5 (cold) and 6e-5 m2/s (preheated),
`delta` is 0.2 to 0.6 mm, the size of the tube wall. W0's `Phi` had the right variables and the
wrong power.

Dilution ramps lose the flame at a commanded fraction a little below the limit seen on coflow
ramps (R 1.15 to 1.2 against 0.95): the diluted gas takes a second or two to reach the burner.

### 1.2 What the procedure flags stood for

The second law gave a flag to "dilutes" and to "velocity ramp". On the three days where it got
worse:

| Day | What the scripts did | Highest R over the limit | Lost |
|---|---|---|---|
| 17319 | two flames diluted from 40 to 30 % at 0.20 m/s | 0.89 | none |
| 18043 | four flames at 50 to 80 % ramped **up** from 0.10-0.14 to 0.40-0.44 m/s | 0.14 to 0.52 | none |
| 18249 | one ethylene flame diluted from 30 to 15 % at 0.30 m/s; two diluted from 20 to 10 % | 0.34; 1.05 and 1.76 | the two at 10 % |

The flags fired, the limit was not reached, the flames lived. And the velocity ramps `ramp_1..3`:
on days 18003, 18008 and 18011 they go **down** (0.40 to 0.10 m/s) after the script has already
passed its most severe point; on 18043 they go up. No flame in the development data was lost
inside a `ramp_N` stage: on the days with down-ramps every loss came earlier, in the dilution or
in the coflow-matching ramp. The flag "velocity ramp" was a marker of long scripts that also carry
a dilution to 30 % or less.

### 1.3 When

For flames whose schedule goes beyond the limit, the loss time follows the schedule: with
`t_sev` the time at which the script first reaches its most severe point (lowest fuel fraction and
highest coflow), the loss comes at about `t_sev * R_c / R`:

| | lost at (s) | `t_sev R_c / R` (s) | D1-type clock `W (R_c/R)^2` (s) |
|---|---|---|---|
| 30 % methane, coflow ramp to 0.40, long script (18003 B1-B3) | 15.2 to 16.5 | 16.6 | 34 |
| 25 % methane, long script (18008 B2, 18011 H2) | 11.9 to 12.2 | 11.3 | 13.6 |
| 30 % methane, coflow ramp to 0.30 (18022 D1, 18026 B1) | 12.8, 19.9 | 12.2, 21.1 | 23, 29 |
| 25 % methane at 0.20 m/s (18026 H1, J2, K1) | 18.2 to 19.0 | 15.2 | 12.3 |
| ethylene diluted to 10 % at 0.50 m/s (18241 U1, 18249 L1) | 16.5, 20.9 | 12.2, 15.2 | 9.7, 8.4 |

The window of the script (planned fuel-off) is the wrong clock: long scripts hold and ramp down
for a minute after the dangerous part.

### 1.4 The losses the conditions do not explain

Nine flames were lost between 0.9 and 2.4 s after onset, as the igniter retracts: four ethylene
flames at 60 and 100 % fuel (days 18145, 18177; repeats of the same script survived), five methane
flames at 0.63 to 1.43 of the limit (the one at 1.43, 25 % methane at 0.20 m/s, was lit beyond
the limit). They are 3 % of all flames; igniter power and timing do not separate
them. One methane flame (17324 G1) was lost at 10 s at R = 0.4. These stay with the small clock.

## 2. Mechanism, restated

The flame base is held in the boundary layer of the coflow on the fuel tube. It is lost when the
velocity gradient there exceeds what a flame of burning speed `S_L` can follow, `S_L^2 / alpha`:
`R = U_co / (k S_L^2) = 1`. Methane has no stable lifted range on this burner at these dilutions:
loss follows detachment at once (the D1 argument, now with the number). Ethylene stands twice the
gradient for its burning speed. A script loses the flame if, and when, it crosses `R = R_c`.

## 3. Candidates

### C1 (run; proposed): the sharp limit, on the existing declaration

Lines changed: the whole blow-off clock (20 elements). In the law's features (lowest fuel
fraction `x`, highest coflow `U`, window `W`, ethylene flag):

```text
ln R   = ln U - ln 10.2 - 2 ln S_L,   ln S_L = -1.871 + 1.3 ln(x / 0.30)  (methane),  -1.556 + 1.3 ln(x / 0.15)  (ethylene)
g      = ln(R / R_c),   R_c = 0.93 (methane), 1.8 (ethylene)
ln tau = ln W + 5 relu(-g) - 2 relu(g),   shape 4
```

(`S_L` is the W0 formula made straight in ln x around the fractions where the limit is met; it is
too high for nearly pure fuels, where `R` is far below the limit anyway.) Probability of loss
before fuel-off: `1 / (1 + (R_c / R)^20)` below the limit, near 1 above it.

Expectation as logged with the run, and what the tool reports:

| | Expected (written before the run) | Realised |
|---|---|---|
| Integrated Brier, pooled | 0.317 to 0.10-0.14 | **0.043** (D1 0.317; cld2 0.143; best baseline 0.228) |
| Train / validation | (not split in the note) | 0.020 / 0.093 (D1 0.125 / 0.514) |
| Methane schedules reaching R of 1.24 or more | P(lost) from 0.1-0.4 to 0.9 or more (20 of 21 lost) | forecast bins 0.85 and above: 21 cases, 20 lost |
| R 0.78 to 0.86 (40 % to 0.40-0.44 m/s; 30 % at 0.20 m/s) | 0.03 to 0.17 (3 late losses of 28) | forecast bin 0.05 to 0.3: 26 cases, 4 lost |
| Ethylene diluted to 10 % | 0.6 and 0.99 (4 of 4 lost) | the two at 0.30 m/s fall in the bin 0.5 to 0.7 (3 cases, 3 lost), the two at 0.50 m/s in the top bin |
| Ethylene at 15 %, 0.4 to 0.5 m/s | 0.01 or less (0 of 9) | lowest fraction 0.15 to 0.25: mean forecast 0.019, none lost |
| Days | lower on 17324, 18003, 18008, 18011, 18026, 18241, 18249, 18032, 18022, 17342 and slightly on the safe days; unchanged within 0.001 on 18145, 18177, 18005 | lower on **21 of 25**; equal on 18145, 18177; higher on 18005 (0.068 to 0.072) and on **18249** (0.010 to 0.030) |
| Radiometer | unchanged | unchanged (0.119) |

Not in the note, from the tool's strata: tests that dilute, forecast 0.83 against 0.81 observed
(D1 0.39); lowest fraction 0.25 to 0.4, forecast 0.48 against 0.50 (D1 0.16); methane 0.206
against 0.219; ethylene 0.039 against 0.056; Brier of lost before fuel-off 0.041 (D1 0.081).

The sign was predicted for the pooled score and for every stratum, and missed on two days: 18249,
where the two ethylene losses are forecast but 8 to 12 s too early (the window clock of 1.3), and
18005, where one igniter loss at 0.84 of the limit lost the little probability D1 gave it. By the
keep rule C1 passes: lower pooled, lower on 21 of 25 days whose forecasts change, radiometer
unchanged.

### Not run

- **A front-loaded small clock** for the igniter losses (shape 0.2 instead of 0.5). Expected gain
  on the nine early losses about 0.001 pooled, expected cost on 230 safe flames about the same,
  and it would make most no-event days "higher". Not worth a candidate.
- **A lower ethylene limit** (1.5 instead of 1.8, allowing for the dilution lag). It moves two
  cases. Left at the value run.

## 4. Proposal

**C1 as the model on slug `cld`** (a revision of D1; numbers only).

### For the engineer

Nothing new for C1: same declaration, same inputs. Two input matters seen in the cases: 20
development cases are refused because the stage features are missing from the prefix, and the
flame 18008 B3 is scored as "not lost" though its fuel was cut 1.3 s after onset (it is censored
correctly; noted only because it is the single survivor above the limit).

### Edits (changed lines; parent D1, `configs/models/law/cld/d1-fable`)

```text
loss.blowoff.median.b0 = 4.094
loss.blowoff.median.skip[0:6] = [0.0, 0.0, 1.0, 0.0, 0.0, 0.0]
loss.blowoff.median.W1[0, 0] = 2.6
loss.blowoff.median.W1[0, 1] = -1.0
loss.blowoff.median.W1[0, 3] = 3.092
loss.blowoff.median.b1[0] = 3.246
loss.blowoff.median.w2[0] = 5.0
loss.blowoff.median.W1[1, 0] = -2.6
loss.blowoff.median.W1[1, 1] = 1.0
loss.blowoff.median.W1[1, 3] = -3.092
loss.blowoff.median.b1[1] = -3.246
loss.blowoff.median.w2[1] = -2.0
loss.blowoff.median.w2[2] = 0.0
loss.blowoff.median.w2[3] = 0.0
loss.blowoff.shape = 4.0
```

(The file run is `runs/dev/gen7/study/cld_c1.wsp`.)

## 5. What remains structural

**The time of the loss needs the time of the ramp** (1.3). A new slug, `cld3`: the declaration of
`cld` with one feature appended and one cause added to `loss`.

| # | `name` | unit | transform | centre | scale | domain | Derived from |
|---|---|---|---|---|---|---|---|
| 6 | `t_severe` | s | ln | 20 | 1 | 2 to 600 | seconds from flame onset at which the schedule first holds both its lowest `x_fuel` and its highest `v_coflow_m_s` (the later of the two times, less `at_cutoff.onset_from_ignition_command_s`), and not less than 2 s |

Target `loss`: causes `blowoff`, `other`, `crossing`. The new cause is the limit crossed on the
ramp: `tau = t_severe * R_c / R` beyond the limit, and out of reach below it.

```text
# cld3 = C1 with these lines added (typed, NOT run: the tool cannot run a declaration that does not exist)
x.t_severe.centre = 20.0
x.t_severe.scale = 1.0
x.t_severe.lo = 2.0
x.t_severe.hi = 600.0
loss.crossing.median.b0 = 6.242
loss.crossing.median.skip[0:7] = [2.6, -1.0, 0.0, 3.092, 0.0, 0.0, 1.0]
loss.crossing.median.W1[0, 0] = 2.6
loss.crossing.median.W1[0, 1] = -1.0
loss.crossing.median.W1[0, 3] = 3.092
loss.crossing.median.b1[0] = 3.246
loss.crossing.median.w2[0] = 41.0
loss.crossing.shape = 6.0
```

Expectation, to be checked when the slug exists (it will count as a candidate): the 22 ramp losses
above the limit are timed within about 3 s instead of up to 20 s late on the long scripts and 8 to
12 s early on the ethylene dilutions; integrated Brier from 0.043 to about 0.03; day 18249 back
below D1; days 18003 and 18008 lower; no change on days without a loss.

Also structural, and not typed:

- **The igniter losses** (9 of 268, at 0.9 to 2.4 s): a class of flames that fail at retraction
  or never; one clock per cause cannot hold it, and no condition field separates them.
- **The lag of a dilution ramp** between command and burner (about 2 s): would need the line
  volume; it shifts the limit seen on dilution ramps by 0.2 in ln R.
- **Flames lost just below the limit after a long hold** (3 of 18 at 0.86 to 0.97): the law gives
  them 0.1 to 0.2 through the window clock; their timing is not predicted.
