# Independent check of the fabric-flame physics dossier on the train tracks and the earlier campaign

> Written by a delegated analysis agent (claude-fable-5-2, high effort) on 14 November 2026 for the fifth generation;
> nothing fitted. An independent re-derivation with its own code; of PSI-25 only the four train tracks, the two train notes and the fan calibration workbook were opened.
> It is a proposal to the author, not a model: no number here is a weight until the author has rechecked it
> on train and typed it. What was rechecked is recorded in `docs/wsp/dossier/03_datasets_and_measurements.md`.
> Paths under `runs/dev/` are local working folders that are not in git.

# Independent check of `phys_bass.md` claims B1–B8

Five of the eight claims hold as stated; B1, B2 and B7 are weakened. The oxygen balance reproduces on settled flames but is mostly a test of one diffusivity, and the train flow at FAN 42 is not known. The "slow growth" is a time-to-steady, not a time constant. The downstream slow-down appears in one burn of three.

Method throughout: speeds are differences of end-group medians over stated windows, lengths are window medians of tip minus base, and nothing was fitted. Full tables are in `runs\dev\research7\verify_bass\tables.md`.

## B1. Oxygen balance — CONFIRMED for settled flames at the stated inputs; WEAKENED as a law "with no adjusted constant"

I chose 19 train windows (12 settled, 7 transient) and 15 PSI-26 windows, and computed predicted / measured 0.5(v_tip + v_base) with the measured length.

| Set | n | Ratio (min – median – max) | Boundary layer only |
|---|---|---|---|
| Train settled (T5, T6 at FAN 42; T16 at FAN 43) | 12 | 0.87 – 0.98 – 1.09 | 0.56 – 0.69 |
| Train ignition phases (T5 FAN 61, T6 FAN 47, T16 FAN 46) | 3 | 1.21, 1.21, 0.94 | 1.15, 1.00, 0.73 |
| Train after the ramp (T6, T16, T5) | 3 | 0.91, 0.98, 0.71 | — |
| Train T5 during the ramp (tip retreating) | 1 | 3.43 | — |
| The dossier's own 14 windows, my code | 14 | 0.91 – 1.01 – 1.11 | — |
| PSI-26 at 9–22 cm/s (tests 2, 4, 10, 11) | 8 | 0.96 – 1.05 – 1.10 | 0.92 – 1.07 |
| PSI-26 test 5 settled, 5 and 3 cm/s | 3 | 1.15, 1.19, 1.12 | 1.00, 1.03, 0.83 |
| PSI-26 test 5 transients and quench | 4 | 1.34 – 1.58 | — |

Sensitivities on the settled train windows:

| Change | Effect on the ratio |
|---|---|
| Width 20 mm instead of 22 | +7 % (settled range 0.94–1.18) |
| ν and α both ×0.8 / ×1.2 | about −16 % / +16 % |
| α alone ±20 % | ∓12 % at low flow, ±2 % at FAN 61 |
| ν alone ±20 % | ±4 % at low flow, ±9 % at FAN 61 |
| Flow at FAN 42 (T5, T6) from 0 to 3.77 cm/s | 0.72 to 1.24, about +0.08 per cm/s |
| Flow at FAN 42 of 0.5–1 cm/s | 0.76–0.95 |
| PSI-26 flows by the BASS-II calibration | test 11: 1.22–1.26; test 5: 1.28–1.32 |

Why it is weakened:
- **It mostly tests α\*.** In every settled train window the diffusion term is 56–67 % of δ², and δ is 9.3–11.9 mm, not the "6–7 mm" of section 1.2. The zero-flow limit of the formula is already 0.029 w/s at 18.8 %, about 75–80 % of the measured speed. So the train agreement says little about flow dependence.
- **The flow at FAN 42 is not known.** The train note's overlay tables show AIR falling to 0 in T5 and T6 while FAN stays at 41/42. In T16 at FAN 43 it levels at 3, which matches the line's 2.96. The line's 2.37 cm/s for FAN 42 is therefore doubtful.
- **It fails in transients.** Ignition phases give 0.94–1.21 on train and 1.06–1.51 on PSI-26; the T6 value moves from 0.98 to 1.21 when the window starts at 2.8 s instead of 1 s. During and after a flow reduction it gives 0.71–3.4.
- **The diffusion term does not help everywhere.** On PSI-26 test 5, boundary layer alone gives 1.00–1.03 at 5 cm/s; with the diffusion term it is 1.15–1.19.

## B2. Asymmetric response of the length — shrink CONFIRMED; growth WEAKENED; step-up CANNOT TELL

Definition: time for the length to cover 63 % (t63) or 90 % (t90) of the change from its pre-change median to a later settled median.

| Burn, change | t63 from first fan change | t63 behind the fan's own 63 % point | t90 from first change |
|---|---|---|---|
| T5, 61→42 (ramp 6 s) | 2.5 s | 0.0 s | 5.1 s |
| T6, 47→42 (ramp 2.5 s) | 2.3 s | +0.8 s | 4.9 s |
| T16, 46→43 (ramp 1.5 s; change is only 4 px) | 2.4 s | +1.4 s | 3.2 s |
| PSI-26 test 5, 5→3 cm/s (time inferred ±0.7 s) | 1.6 s | — | 6.8 s |

Still spacing is 1.1–1.25 s, so the shrink is within 0–1.6 s of the flow, with a tail to 90 % of 2–7 s.

Growth after ignition (reference is the measured settled length where one exists, else the carried law):

| Burn | L at first still / reference | t63 after igniter-off | after igniter-on | Note |
|---|---|---|---|---|
| T5 (FAN 61) | 0.79 (law) | 3.3 s | 8.9 s | cut by the step at 9.7 s |
| T6 (FAN 47) | 0.78 (law) | 6.9 s | 12.5 s | cut by the step at 8.3 s |
| PSI-26 test 5 | 0.90 (measured) | 2.2 s | 8.5 s | overshoots to 1.05 w at 8 s, back to 0.85 by 16 s |
| PSI-26 test 4 | 0.55 (law) | 7.1 s | 12.3 s | never steady |
| PSI-26 test 2 | 0.42 (law) | not reached in 10 s | — | plateau 1.0–1.13 w from 3.6 s |
| PSI-26 test 10 (narrow) | 0.59 (measured) | 3.8 s (blue edges: about 6 s) | 6.1 s | steady only at about 18 s |

- "10–20 s" is right only as time to steady counted from igniter-on. As a 63 % time, growth is 2–7 s after igniter-off.
- The asymmetry is real, but a factor of 2–5, not 10.
- No track contains a step up with known timing. Train fans rise only after the burn; test 2's fan was raised slowly with unknown flow. The opposed-flow tests were not measured.

## B3. Narrow sample — CONFIRMED as numbers; interpretation as width powers is soft

Carried wide law evaluated unchanged at the published flow and oxygen, against the steady phases in own widths.

| Burn | Speed / wide law | Length / wide law | Same at the BASS-II-calibrated flow |
|---|---|---|---|
| Test 10 (11 cm/s, 21.6 %) | 1.53–1.55 | 0.85–0.87 | 1.52, 0.84 |
| Test 11 (20 cm/s, 21.5 %) | 1.25–1.26 | 0.62–0.65 | 0.98, 0.47 |

- **Millimetres.** The own-width factors do not depend on the mm scale. Only the conversion to a width power does: speed −0.38 to −0.72 at a width ratio of 1.83, −0.30 to −0.57 at 2.14; length +0.25 to +0.78, or +0.19 to +0.62.
- **The reference is itself extrapolated.** At this oxygen the wide burns at 9–22 cm/s (growing flames) run 0.79–1.14 of the law on speed and 0.62–0.75 on length.
- **Measured against measured**, in own widths: speed 1.85 (test 10 vs test 2) and 1.01 (test 11 vs test 4); length 1.48 and 0.75. In mm/s, tests 10 and 2 are equal (2.0 mm/s).
- So "less than 1.83 on both" is solid at 20 cm/s. At about 10 cm/s the speed rule is not contradicted directly. The length "sign" there rests on the law's extrapolation: the narrow flame is 0.86 of the law but 1.3–1.5 times the still-growing wide flame.

## B4. Wide PSI-26 burns against the carried law — CONFIRMED

| Burn | Speed / law (mean; tip; base) | Length / law |
|---|---|---|
| Test 5, 5 cm/s (two windows) | 0.89–0.90; 0.87–0.90; 0.87–0.93 | 0.92–0.95 |
| Test 5, 3 cm/s | 0.91; 0.82; 1.01 | 0.95 |
| Test 2, 9 cm/s (growing) | 0.90; 1.01; 0.79 | 0.62 |
| Test 4, 22 cm/s (growing) | 1.14; 1.40; 0.88 (base over the whole burn 1.02) | 0.75 (0.64–0.89) |

The claimed 0.86–1.04 and 0.63–0.98 are within window choice of mine. At the BASS-II-calibrated flows test 5 drops to 0.72–0.77 and 0.66–0.76.

## B5. Residence time — CONFIRMED, with two edges

| Set | L/v (s) |
|---|---|
| Train settled, first three windows of each burn (9 windows) | 9.2–11.3, median 9.6 |
| Train, last 15–20 s of each burn | 8.7, 9.8, 7.9 |
| PSI-26 test 5 at 5 cm/s | 14.2, 14.9 |
| PSI-26 test 5 at 3 cm/s | 12.7 (11.5 by base, 14.2 by tip; flame still shortening) |
| Narrow, tests 10 and 11 | 8.7, 8.9 and 8.7, 8.3 |
| Growing wide flames, tests 2 and 4 (not steady) | 11.0, 11.2 |

The 14–15 s rests on one burn at one flow.

## B6. Flow near fan stall — CONFIRMED

AIR display for two restrictors (U = 1.0066 × AIR), 23 day blocks:

| Fan display | Blocks | Min | Median | Max | sd |
|---|---|---|---|---|---|
| 42 | 11 | 0 | 1 | 3.75 | 1.26 |
| 46.5 / 47 | 18 | 0 | 4.4 | 5.75 | 1.37 |
| 55 | 23 | 3 | 7 | 9 | 1.54 |
| 72 | 23 | 7 | 13 | 16 | 2.63 |
| 122 | 23 | 18 | 33 | 40 | 6.18 |

- The 42 rows read 0, 0, 0, 0, 0.5, 1, 1, 2, 2, 3, 3.75.
- On the 11 blocks with both rows, AIR(46.5) minus AIR(42) is 2 to 5, median 3. Applied to GMT 100 (46.5 → 5), FAN 42 would be about 2 cm/s, range 0–3.
- Fan 72 by period: February 7–13; GMT 091–203 10.5–16 (median 13.5); GMT 209–210 7–8.
- By-day table: `runs\dev\research7\verify_bass\fan_workbook.md`.

## B7. Downstream slow-down — WEAKENED

First against third settled 20-s window, about 1.5 w apart: T5 −8.7 %, T6 −9.5 %, T16 −4.2 %. Sliding 20-s windows change the picture:

| Burn | What the sliding windows show |
|---|---|
| T5 | All of the drop is between 20–40 s and 30–50 s (settling after the ramp). From 2.25 to 3.72 w the speed is flat at 0.0349–0.0359 w/s. |
| T6 | Progressive: 0.0405 → 0.0342 w/s over 2.25 w, about −7 % per width. This is the burn whose FAN flickers 41/42 and whose AIR falls to 0. |
| T16 | 0.0385 → 0.0356 over 1.86 w (−4 % per width), then +10 % in the last window. Halves of the stretch differ by −1 %. |

One pixel at a window end is 2.4 % of a window speed, so T16's change is 1–2 px. The 7–10 % holds in one burn of three as a progressive effect, and there it cannot be separated from flow decay.

## B8. Ignition — CONFIRMED, with a wording correction

| Burn | Igniter (s) | L at first still / law steady L | / measured settled L | Tip speed over first 5 s / law |
|---|---|---|---|---|
| T5 | 5.6 | 0.79 | none exists | 1.26 (1.64 over stills up to 5 s) |
| T6 | 5.7 | 0.78 | none exists | 1.03 (1.18) |
| PSI-26 test 5 | 6.3 | 0.85 | 0.90 | 1.14 (1.05) |
| PSI-26 test 2 | 6.3 | 0.42 | none | 1.33 |
| PSI-26 test 4 | 5.2 | 0.55 | none | 1.45 |
| T16 | 9.7 | 1.33 | 1.67 | 0.43–0.46 |

- The 0.42–0.85 and 1.33 are ratios to the carried law's steady length, not to "its later steady value". Only PSI-26 test 5 and T16 have a measured one.
- The first still is 0.5–1.4 s after igniter-off. T16's length falls from 0.71 to 0.44 w between 1.3 and 2.5 s.
- The upper end of the tip-speed range moves by ±0.2 with the window.

## Numbers I would hand the author

| Quantity | Value | Rests on |
|---|---|---|
| Oxygen-balance ratio, settled flame, flow known | 1.0 ± 0.12 (0.87–1.10) | T16 (4 windows), PSI-26 tests 10, 11, 2, 4 |
| Same, train at FAN 42 | 0.92–1.09 at 2.4 cm/s; 0.72–1.24 for 0–3.8 cm/s | T5, T6 |
| Same, wide at 3–5 cm/s and 22 % O2 | 1.12–1.19 | PSI-26 test 5 |
| Same, first 10 s after ignition | 0.94–1.5 | 7 burns; do not use it there |
| Length shrink | 63 % within 1.5 s of the flow change; 90 % in 2–7 s | T5, T6, T16, PSI-26 test 5 |
| Length growth after ignition | 63 % in 2–7 s after igniter-off (6–12 s after igniter-on); steady at 15–18 s | T5, T6, PSI-26 tests 5, 4, 10 |
| Length growth after a step up | not measured | — |
| Narrow speed factor, own widths, against wide law | 1.54 at 11 cm/s; 1.26 at 20 cm/s (0.98 if that flow is 26.8) | tests 10, 11, 21.5 % O2 only |
| Narrow length factor, own widths | 0.86 at 11 cm/s; 0.62 at 20 cm/s (0.47) | tests 10, 11 |
| Residence time, near limit | 9.6 s (9.2–11.3); 7.9–8.7 in the last 15–20 s of T5 and T16 | T5, T6, T16 |
| Residence time, wide, 22 % O2 | 14.2–14.9 s at 5 cm/s; 12.7 s (11.5–14.2) at 3 cm/s | PSI-26 test 5 |
| Residence time, narrow | 8.3–8.9 s | tests 10, 11 |
| Flow at FAN 42, two restrictors | 0–3.8 cm/s, median 1.0, sd 1.3, bounded at 0 (4 of 11 read 0) | 11 calibration blocks |
| Flow at FAN 46.5 | median 4.4, sd 1.4 (3–5.75 within GMT 091–203) | 18 blocks |
| Flow scale at FAN 72 | sd about 11 % within GMT 091–203; 7–16 over all days | 23 blocks |
| Downstream slow-down | 0 to −7 % per width; centre about −3 % | T5 (0), T16 (−1 to −4), T6 (−7) |

## What the dossier did not say

- **FAN 42 and FAN 43 flames are alike.** They spread at the same speed (0.035–0.040 w/s) and length (0.36–0.39 w), while the AIR display settles at 0 and 3. The speed line's slope below 3 cm/s is not identified by train.
- **The closed form is not the typed line.** With t_res = 10 s at 18.8 % it gives 0.0371, 0.0480, 0.0771 w/s at 2.37, 5, 10.87 cm/s. That is a slope of 0.004–0.005 per cm/s, and 9–14 % below the typed line at 5–11 cm/s. With t_res = 14 s at 22 % it is 10–28 % above it. The 0.0061 is only the high-flow asymptote.
- **The balance cannot quench.** Its zero-flow speed is 0.025–0.034 w/s for 16.4–22 % O2.
- **Late-burn weakening.** In the last 15–20 s all three train flames shorten 10–20 % and blue brightness falls to 0.43–0.64 of its early value, while speed stays within ±6 % (T16 speeds up).
- **Width in millimetres.** The published test-10 values imply 10.0–10.6 mm per narrow width. By pixel ratio that puts the wide width at 18.4–19.5 mm, which would raise every oxygen-balance ratio by about 7–16 %.
- **Workbook oddities.** The last four days (GMT 212–216) sit in the one-restrictor position and include two more fan-42 rows (2 and 2). The GMT 210 block has identical no-restrictor and two-restrictor columns. The first block is labelled GMT 043 but dated 1 April.
- **One model input is not on the table.** Using cotton-only areal density is a choice: the total 90 g/m² would scale the ratio by 0.75.

## Files and protected material

**Opened**
- `docs\wsp\research\phys_bass.md` (full)
- `runs\dev\research6\bass\tables.md` (full; the scripts there were neither opened nor run)
- `docs\wsp\feedback\bass_train_tracks.md` (full)
- `docs\wsp\feedback\bass_train_measurement.md` (headings, lines 1–67 and section 2)
- `docs\wsp\feedback\bass1_psi26_tracks.md` (full)
- `tools\wsp\bass_optical\tracks\train\T5.json`, `T6.json`, `T16.json`, `T10.json`
- `tools\wsp\bass_optical\tracks\external_psi26\*.json` (all ten loaded; measurements use tests 5, 4, 2, 10, 11)
- `data\raw\samples\PSI-25\PSI-25_Reference Data_Fan calibration data.xlsx`

**Written**
- `runs\dev\research7\verify_bass\verify.py`
- `runs\dev\research7\verify_bass\tables.md`
- `runs\dev\research7\verify_bass\fan_workbook.py`
- `runs\dev\research7\verify_bass\fan_workbook.md`
- Two temporary dumps in the session scratchpad.

No git-tracked file was modified; `runs/` is git-ignored.

**PSI-25 statement.** Of PSI-25 I opened only the four train tracks, the two train notes and the fan calibration workbook. I listed the file names in `data\raw\samples\PSI-25\` and opened none of the others. I did not open `exp_bass.md`, `val_sealed`, `data_ext` or any `holdout` folder, and ran neither `dev_eval.py` nor `front-feedback`. I read no published per-test BASS-II result. I launched no helpers, and nothing was fitted or searched.
