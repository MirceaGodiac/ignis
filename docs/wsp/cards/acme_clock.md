# Model card — `acme_normal.wsp2` and `acme_inverse.wsp2` (the clock forward)

> **Status, 14 November (fourth generation).** For normal flames the carried model is
> [`acme_normal.wsp4`](acme_blend.md), in which D3 of this card is one of two members, unchanged. For
> inverse flames the carried model is [`acme_inverse.wsp3`](acme_growth.md) D1. This card is the
> second and third generation as they were written.

**State: experimental.** Hand-written weights, not trained. Evaluated on development partitions
only; the test partition has not been read. Uncertainty is assumed, not calibrated. One forward
(`crates/ignis-wsp/src/flame.rs`), two specs, separate weights; normal and inverse flames are never
pooled.

## What they predict

For a burner-stabilised spherical flame of ACME CFI-G (PSI-159) with an established hot flame at the
cutoff, given the conditions set before the test: the cumulative incidence of **self-extinction**
(the end of the hot flame, as the PSI-159 tables label it) and of **flow termination** (operator,
script duration or burner temperature limit), as a partition with the flame still active.

Not predicted: cool flames after the hot flame, flame size or temperature, the effect of an
operator action. Programmed stops are the observed campaign policy and are written as such.

## Releases

Specs `acme_normal-wsp-v2` (`sha256:3c9ab8be…`), `acme_inverse-wsp-v2` (`sha256:760a6f08…`); forward
`sha256:9428278e00ed4aa7c30b89f7354e1c15076f8195b127e7a660d1934ee41a3d2e`; author `claude-fable-5-2`.
Files: `configs/models/acme/wsp2/{normal,inverse}/<candidate>/`. Carried forward: normal D3,
inverse D1. No candidate of this spec is zero-shot.

## Computation

```text
c = (c_logp, c_logm, c_o2, c_xf, c_tad, c_zst, f_ethane, f_butane)

self-extinction   weak class, probability sigmoid(N_flash(c)):  T ~ log-logistic(flash_tau, flash_shape)
                  main class:  T = floor + R,  R ~ log-logistic(exp(N_tau(c)), shape)
flow termination  burner limit T ~ log-logistic(exp(N_bt(c)), shape_bt)
                  programmed stops: at time T_a a burning flame is stopped with probability
                  sigmoid(logit_a + W_a·c)
                  background hazard exp(log_base)
```

The classes are latent and re-weighted by their survival to the cutoff; the causes compete with an
exact split of the lost mass in every 0.1 s step, so the four rows sum to one.

## What the numbers say

### Normal flames, D1 (typed from the 81 train rows)

| Clock | Numbers | Train evidence | Can fail when |
|---|---|---|---|
| Floor | 6 s | no radiative self-extinction before about 5 s; air flames 5–13 s | pressures above 2 atm |
| Radiative life, flow | ln R falls 1.1 per e-fold of flow above 1.5 mg/s, 2.6 below | 39 % O2, 1 atm: 43, 71, 158–232, 304 s at 2.36, 1.5, 1.0, 0.85 mg/s | very dilute fuel at low flow (one train test went out in 8 s) |
| … oxygen | +2.2 per 10 % O2 | 44 s against 88 s at 34 % and 38 %; air 9–13 s | — |
| … pressure, fuel | −0.33 per e-fold; butane −0.3; +0.2 per e-fold of fuel fraction | 59, 43, 34 s at 0.5, 1, 2 atm | propane in enriched O2 is not in train |
| Weak class | all below about 1750 K adiabatic | ten train flames at 1600–1700 K gone within 1–2 s | — |
| Burner limit | median 400 s at 1 mg/s, ×e³ per e-fold of flow; butane sooner | 28–56 s at 0.5 mg/s, 80–160 s at 0.75 mg/s | a different limit setting |
| Programmed stops | 61.5 s (0.12, low flow), 98 s (0.05), 126.5 s (0.10), 275.5 s (0.70) | train flow terminations without a burner comment | another campaign |

### Inverse flames, D1 (typed from the 73 train rows)

| Clock | Numbers | Train evidence |
|---|---|---|
| Floor | 2 s | shortest main-class flames last 3–5 s |
| Life, ambient fuel | ×e³ per e-fold; held for minutes above 13.5 % | 5–7 s at 7 %, 8–18 s at 8–9 %, 46–51 s at 14 %, none above |
| … injected O2 | +0.45 per 25 % O2; air-injected flames that survive ignition do not radiate out | 4–9 s at 50 % against 7–18 s at 85 % |
| … flow | the lowest flows go out within 3–5 s | 1.4 mg/s tests |
| Weak class | air injected, adiabatic temperature near 1850 K | three of five train tests flashed out |
| Burner limit | sooner at low flow and in fuel-rich ambient; not reached with 50 % O2 | 26–48 s at 3.7 mg/s; 12 s in 43 % fuel |
| Programmed stops | 66 s (0.80) and seven smaller ones | 12 of 32 train flow terminations at 65–66 s |

### Normal flames, D3 (14 November)

The audit ([feedback/acme_d1_train_audit.md](../feedback/acme_d1_train_audit.md)) showed that D1 and
D2 could not say two things at once: in air the life does not depend on flow between 2.4 and 1.0 mg/s
(9–10 s), while at 39 % O2 it grows from 43 s to 232 s over the same range. D3 writes the life as a
function of the *reduced flow*, the flow relative to the flow a flame can hold for good:

```text
z    = c_logm − 0.65 c_o2 + 0.24 c_logp
ln R = 2.43 + 0.84 c_o2 − 0.28 c_logp + 1.4 relu(0.50 − z) + 2.0 relu(−0.22 − z) + fuel terms,   T = 4 s + R
```

Above the first hinge (3 mg/s at 39 % O2, 0.9 mg/s in air) flow does not matter; below it the life
lengthens as flow^−1.4, below the second as flow^−3.4. Law against measured life, in seconds: ethane
at 1 atm and 39–40 % O2 43/43, 72/71, 233/232, 169/158, 235/304; in air 9.5/9, 11/11, 16/16–19; at
0.5 atm 54/59, 84/88, 225/258, 45/44; at 2 atm 29/34, 50/51, 41/44, 128/123, 118/131.

| Number | D1 / D2 | D3 | From |
|---|---|---|---|
| Floor | 6 s | 4 s | propane-air flames at 2–3 atm go out at 5–7 s (seven tests) |
| Flow and oxygen | separate terms: −1.1 and −2.6 in flow, +2.2 per 10 % O2 | the reduced flow above; +0.84 per 10 % O2 at fixed reduced flow | three ethane sessions at three pressures, 27 air pairs |
| Pressure | −0.33 per e-fold | −0.28, and the hinges move by 0.24 | 0.5, 1 and 2 atm at three flows |
| Butane | −0.3 everywhere | −0.25 per 10 % O2 above air (−0.45 at 39 %), none in air | nine enriched comparisons in two sessions; three air repeats |
| Dilute butane | +0.2 per e-fold for every fuel | −0.35 per e-fold below 30 % butane, levelling off below 12 %; none for ethane | two butane sessions, one extreme; ethane pure to 25 % shows nothing once oxygen drift is counted |
| Propane in enriched O2 | on the reference law | half of the butane penalty | no train test: an interpolation by carbon number |
| Clock shape | 4 (D1), 2.5 (D2) | 3 | repeats differ by 0.15 in ln T; session medians of D1 ranged from −0.40 to +0.12 |
| Burner-limit clock | 6.0 + 3.0 c_logm, butane −0.6 | ethane 7.04 + 4.5 c_logm, butane −1.1, propane between; shape 2.5 | ethane 28–56 s at 0.5 mg/s, not within 275 s at 0.75 mg/s; butane 72–158 s |
| 275 s stop | 0.70 | 0.50 | one of the two sessions with a flame alive then used it |

### Inverse flames, D3 (14 November): not adopted

Three changes to D1, each argued from what transfers between train sessions: the weak class decided
by air injected into a lean ambient instead of the adiabatic temperature (flashes at 1835–1879 K,
minute-long flames at 1785 K); stops that happened once turned into a background rate instead of
atoms at their own times; a steeper, wider burner-limit clock. On train it is better on both scores.
On validation its extinction score is worse than D1 (0.156 against 0.146) and its flow-termination
score level (0.096 against 0.097). D1 stays the carried candidate. The validation sessions hold
leaner ambients (5.8 % fuel) than any train session; whether the flash law or the background rate
costs the extinction score there cannot be told from an aggregate.

## Results (development)

Integrated Brier score over 6–120 s, lower is better; extinction / operator termination. Validation
has two sessions per configuration, so no interval separates neighbouring rows.

| Normal flames | Validation (84 cases) | Train (210 cases) |
|---|---|---|
| linear baseline, fitted | 0.185 / 0.093 | 0.108 / 0.039 |
| acme_normal.hazard, fitted | 0.187 / 0.093 | 0.111 / 0.044 |
| constant baseline, fitted | 0.214 / 0.099 | 0.225 / 0.072 |

| Inverse flames | Validation (49 cases) | Train (204 cases) |
|---|---|---|
| acme_inverse.hazard, fitted | 0.143 / 0.118 | 0.068 / 0.076 |
| linear baseline, fitted | 0.173 / 0.131 | 0.064 / 0.074 |

## Limits

- `extinction` is the hot-flame end. For n-butane at 2 bar a cool flame can follow; the model says
  nothing about it.
- Programmed stops are campaign facts; a forecast for a test with another script is wrong about them.
- The burner limit differs between sources (450 °C and 650 °C); the clock was typed against train
  outcomes, not against a limit value.
- D2 (both) and the choice between D1 and D2 were made after one aggregate validation look each;
  the validation numbers of the carried candidates are therefore development numbers.
- D3 (both) were committed before their validation scores existed. The normal D3 improved and was
  adopted; the inverse D3 did not and was not: validation acted as a veto, on two sessions.
- The normal D3 misses three train flames it was typed with in view: propane in air at 2–3 atm and
  0.5 mg/s goes out at 5 s where the law says about 11 s (one session; flow does not lengthen the
  life there as it does at 1 atm), and 3 % butane at 0.5 mg/s goes out at 8 s.
- Propane in enriched oxygen is an interpolation with no train test.
