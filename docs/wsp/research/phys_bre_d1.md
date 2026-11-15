# ACME BRE (PSI-20): one revision of the W0 law from the train days (D1)

Author: `claude-fable-5-2`, 15 November 2026. Kind: **data-informed revision** of
`docs/wsp/research/phys_bre_w0.md` (release `sha256:aa0eef5b...2b4d`); not zero-shot. Read for it:
`docs/wsp/feedback/bre_train_anatomy.md` and `runs/dev/gen7/sensitivity/bre_aa0eef5bc7653922.json`
(train days only), and the coordinator's message with four validation aggregates of the W0 look
(extinction 0.2755 against 0.2274 and 0.2129; operator termination 0.1383 against 0.1799). No
validation or test outcome file was opened and nothing below was chosen from those aggregates. No
program was run except to print the sensitivity file. One step, typed by hand.

## 1. What the train days say

Extinction score on train 0.2511 against 0.2300 for the incidence baseline. The forecast puts too
many flames out by 60 s, and the excess sits at reduced pressure:

| stratum (train) | n | forecast out by 60 s | observed | 
|---|---|---|---|
| 65 to 85 kPa | 46 | 0.73 | 0.15 |
| below 65 kPa | 65 | 0.41 | 0.26 |
| 85 kPa and above | 129 | 0.63 | 0.70 |
| oxygen 0.25 to 0.30 | 58 | 0.73 | 0.33 |
| oxygen 0.30 to 0.36 | 76 | 0.52 | 0.33 |

At cabin pressure the law is about right (and slightly late in air: 0.89 against 1.00). The oxygen
strata of 25 to 36 % are the reduced-pressure atmospheres (10.2 and 8.2 psia), so the two rows are
largely one fault: **at reduced pressure the flames live longer than the law says.**

## 2. Which number, and why

The only place pressure enters the radiative limit is the exponent `a` of the critical flux,
`q_x ~ p^a` (W0 section 1.3). I typed `a = 0.5 +/- 0.3` as a judgement between the optically thin
limit (`a = 1`), band saturation with line broadening at a fixed path (0.6 to 0.9, and about 1.1 when
the fall of total pressure is counted in the broadening), and a hot layer that thins as `p^-1/2`.
The W0 text names it among the three weakest assumptions and lists "pressure does nothing near the
boundary" as a falsifier in the other direction. The train days say the opposite of that falsifier:
lower pressure helps the flame more than `a = 0.5` allows. The physical reading is that these blue,
soot-free flames of a few centimetres radiate close to the thin limit in what matters (the loss
scales with the partial pressure of CO2 and H2O), and that the thinning of the outer layer, which I
used to lower `a`, does not reduce the loss seen by the sheet.

The sensitivities agree in sign for every entry that carries `a` (per unit of the coefficient, train
extinction score): `skip[3]` +0.029, the five `W1[j, 3]` of the size ratio +0.085 in sum, the
small-flame `W1[5, 3]` -0.031. All say: more pressure dependence, in the direction of a larger `a`.

Size of the step. To bring the 65 to 85 kPa stratum from 0.73 to its observed 0.15 by `a` alone
would need `a` near 2.5, which no radiation law allows; the train days alone therefore point beyond
the physical ceiling (1.0 to 1.1). I go about half-way from the W0 value to that ceiling:

**`a`: 0.5 -> 0.8.**

It is one physical number. It is typed in seven places because the law's three linear forms carry it
(`b`: `0.541 a`; `base`: `1 - 1.082 a`; `ln h`: `-0.541 a`; the small-flame margin: `-a`):

| line | old | new | derivation |
|---|---|---|---|
| `flame_end.extinction.median.skip[3]` | 0.28 | -0.15 | `base - 0.65 b` = 0.13 - 0.65 x 0.43 |
| `flame_end.extinction.median.W1[0:5, 3]` | -0.27 | -0.43 | minus the pressure coefficient of `b`, 0.541 x 0.8 |
| `flame_end.extinction.median.W1[5:7, 3]` | 0.5 | 0.8 | small-flame margin, `+a` |
| `lasts.logit.skip[3]` | -1.08 | -1.72 | `-4.0 x 0.43` |
| `lasts.logit.W1[0:6, 3]` | 0.46 | 0.13 | pressure coefficient of `base`, 1 - 1.082 x 0.8 |
| `lasts.logit.W1[6:8, 3]` | 0.5 | 0.8 | small-flame margin |
| `height_at_end.median.skip[3]` | -0.27 | -0.43 | `-0.541 x 0.8` |

Nothing else moves: `q_c0`, `rhoD`, `D0`, the knots, the shape and every other coefficient stay.

## 3. Expected effect on the train strata

At `z_p = ln(p / 1 atm)`: the clock's median changes by
`(-0.43 z_p) + (sum of active hinge weights) x (-0.16 z_p) + 1.8 x 0.3 x |z_p|` where the small-flame
hinge is active.

- 85 kPa and above (`z_p` above -0.18): medians change by less than 10 %; forecast 0.63 stays within
  0.03.
- 65 to 85 kPa (`z_p` about -0.3): medians longer by a factor 1.3 to 1.5; forecast out by 60 s falls
  from 0.73 to about 0.60. The observed 0.15 is not reached; see section 4.
- below 65 kPa (`z_p` about -0.56): medians longer by 1.6 to 2.2; forecast falls from 0.41 to about
  0.27 (observed 0.26).
- oxygen 0.25 to 0.36: the same cases; forecasts fall by 0.1 to 0.15.
- oxygen 0.36 and above was already slightly low (0.32 against 0.41) and gets lower where its
  pressure is reduced: a cost of this step, accepted.
- Train extinction score: the first-order sum of the sensitivities is -0.036; for a step this
  large I expect about half, 0.251 to about 0.23, near the baseline and not clearly below it.

## 4. What this step does not repair

- **The rest of the 65 to 85 kPa fault is not a number.** Whole train days show almost no
  self-extinction by 60 s (0 of 21, 0 of 18, 3 of 18) where the law says 0.4 to 0.8; the anatomy does
  not give their pressures, so that they are the reduced-pressure days is my inference. No
  admissible exponent closes a gap of that size. The likely cause is structural: the flames there are *held* (or stopped by the
  operator before they could go out), and the event clock of the forward has one shape and no mass
  at infinity (W0 section 7.5). A held class needs a new forward, not a patch.
- **Methane** (one day, every flame out by 60 s, forecast 0.41) errs the other way. Left alone: one
  day, one step.
- **The 50 % bottle** (0.61 against 0.32) and the low-flux sensitivities (`b1[5]`, `W1[5, 1]`) point
  at the small-flame hinge with opposite signs; not touched.
- 39 of 279 train cases are refused for inputs outside the declared domain or missing; the step
  does not change admission.

## Edits

```text
# bre D1: pressure exponent of the critical flux a = 0.5 -> 0.8 (data-informed, train days only)
flame_end.extinction.median.skip[3] = -0.15
flame_end.extinction.median.W1[0:5, 3] = -0.43
flame_end.extinction.median.W1[5:7, 3] = 0.8
lasts.logit.skip[3] = -1.72
lasts.logit.W1[0:6, 3] = 0.13
lasts.logit.W1[6:8, 3] = 0.8
height_at_end.median.skip[3] = -0.43
```
