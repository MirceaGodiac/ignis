# ACME CLD Flame (PSI-21): one revision of the W0 law from train days (D1)

Author: `claude-fable-5-2`, 15 November 2026. **Data-informed**: typed after reading
`docs/wsp/feedback/cld_train_anatomy.md` (train days only) and the coordinator's summary of the
blind look. No validation outcome, no test day, no per-case file was opened. No fit, no search:
three numbers, each moved about half of what the train strata ask for. Parent: the W0 release of
`phys_cld_w0.md`.

## What the train days show

| Stratum (train) | Forecast | Observed |
|---|---:|---:|
| Loss, methane (87) | 0.043 | 0.149 |
| Loss, ethylene (111) | 0.032 | 0.036 |
| Loss, lowest fuel fraction 0.25 to 0.4 (15) | 0.092 | 0.600 |
| Loss, lowest fuel fraction 0.15 to 0.25 (35) | 0.039 | 0.000 |
| Loss, tests that dilute (7) | 0.262 | 0.857 |
| Radiometer, median error of the median | +0.20 (ethylene), +0.29 (methane) log10 V | |

The fault is on methane only. Ethylene is calibrated, including its dilute flames at 15 to 25 %,
which never lost the flame. Methane at 25 to 40 % loses it six times in ten where the law said one
in ten. So the dilution axis and the ethylene-to-methane shift are not what is wrong; the level at
which a lifted methane flame is lost is.

## Step 1: a lifted methane flame is lost sooner than a lifted ethylene flame

W0 put the loss at `Phi` = 2.5 for both fuels, on the argument that the coflow is a jet and a
lifted base finds slower gas downstream (W0 section 1.2). That argument needs a lifted flame that
can sit in a mixing layer. Methane diffuses faster than momentum (Schmidt number 0.7): in a jet
the fuel has spread out before the velocity has decayed, and a free methane jet has no stable
lifted flame at all (Chung and Lee; I used this in the SLICE law and not here). Ethylene, heavier
and with twice the burning speed, has the wide lifted range.

- `loss.blowoff.median.b0`: 6.403 to **6.03**. Loss threshold of methane `Phi` = 2.5 to 1.7
  (`ln(1.7 / 2.5)` = -0.386; W0 range 1.5 to 4). The train stratum 0.25 to 0.4 asks for about
  -0.73 on this number (0.09 to 0.60 at shape 3.4); the step is half of that.
- `loss.blowoff.median.skip[3]` (ethylene): 0.84 to **1.21**. The same amount added back, so that
  ethylene keeps its threshold of 2.5 and every ethylene forecast is unchanged.

Expected on train: methane at 25 to 40 % from 0.09 to about 0.22 (observed 0.60); methane at 40 %
and 40 cm/s from 0.06 to 0.13 (the 0.4 to 0.7 stratum observed 0.065, so this stratum moves
slightly the wrong way); ethylene unchanged; diluting tests from 0.26 to about 0.4 (observed
0.86).

## Step 2: the volt scale of the radiometer

The scaling with fuel flow and dilution holds (the law beats the best baseline on the blind look
and sits inside its bands); the level is high for both fuels by 0.2 to 0.3 log10 V. The level was
`c_V`, the one number W0 took from the note's range and could not derive.

- `radiometer.median.b0`: -0.404 to **-0.66**. `c_V` 0.5 to 0.39 V/W: -0.11 log10 V, half of the
  median error. Expected: median error from +0.22 to about +0.11.

## Not typed (structural)

- **Diluting tests** remain under-forecast. The forward reads the lowest fuel fraction and the
  window, not that a ramp is run to extinction on purpose. A feature "the script dilutes during
  the test" would carry it; that is a new declaration, not an edit.
- **Losses at igniter retraction** need a class of flames that fail in the first two seconds or
  never; one clock per cause cannot hold it (W0 section 7).
- Twenty train cases are refused because the stage features are missing from the prefix: an
  input matter, not a number.

## Edits

```text
loss.blowoff.median.b0 = 6.03
loss.blowoff.median.skip[3] = 1.21
radiometer.median.b0 = -0.66
```
