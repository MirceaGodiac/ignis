# Flame Design train days: where F1's loss sits (PROTOCOL 12.3, step 2)

> Written by a scoring agent (claude-opus-5-5) on 15 November 2026, after the 12.8 score.
> **Train days only:** normal 19115, 19137, 19150, 19156, 19171, 19175, 19182, 19189; inverse 21328, 21349, 22024, 22027, 22035.
> No validation-day row is read or described here. Source: `runs/dev/data_all/work/transfer_acme/fd_fuel/train_days_cases.jsonl`
> (766 rows: one per family, path and case). Tables: `fd_fuel/anatomy_nostop.txt` and `anatomy_stop.txt`, from
> `tools/wsp/ext/fd_train_anatomy.py`. Nothing was fitted. This file decomposes errors and proposes no value.

**Definitions.**
- **IBS** is F1's per-case integrated Brier contribution, as scored.
- **"Centre"** is ln(t50 / t_end), one value per test at cutoff 1 s. t50 is the time at which F1's total event probability (extinction plus operator termination) reaches one half, read off a dense grid from 1.5 to 600 s.
  - A negative centre means the forecast calls the end too early.
- **"Width"** is ln(t90 / t10).
- **"Inside"** counts the tests whose end falls in [t10, t90].
- The path is the one without the planned stop unless stated.

## Normal flames (204 answered cases, 68 tests; 18 cases of 6 tests refused at T_ad < 1900 K)

**By session** (F1 extinction / termination IBS; F1 minus surrogate on extinction; observed fraction of cases ending by extinction):

| Session | Tests | O2, pressure | Observed ext | F1 ext / term | F1 − surr, ext | F1 P(ext ≤ 120 s) |
|---|---|---|---|---|---|---|
| 19115 | 8 | air, 1 atm | 1.00 | 0.078 / 0.000 | +0.010 | 0.99 |
| 19137 | 8 | air, 1 atm | 1.00 | 0.015 / 0.000 | +0.002 | 1.00 |
| 19150 | 14 | 0.29–0.40, 1 atm | 0.14 | 0.311 / 0.395 | +0.121 | 0.75 |
| 19156 | 7 | 0.37–0.39, 1 atm | 0.29 | 0.276 / 0.347 | +0.080 | 0.75 |
| 19171 | 6 | 0.36–0.39, 1 atm | 0.67 | 0.277 / 0.172 | +0.087 | 0.85 |
| 19175 | 12 | 0.38–0.40, 1.26 atm | 0.00 | 0.278 / 0.502 | +0.125 | 0.75 |
| 19182 | 8 | 0.38–0.40, 1.28 atm | 0.00 | 0.093 / 0.469 | +0.065 | 0.48 |
| 19189 | 5 | 0.38–0.40, 1.28 atm | 0.60 | 0.246 / 0.199 | +0.093 | 0.87 |

- **Where the loss sits.** The tests that ended by operator termination (40 of 68, all but one at 36–40 % O2) carry 90 % of F1's summed loss and 76 % of its extinction loss.
- **In air,** F1 is almost as good as the surrogate: 0.047 against 0.040.
- **By pressure:** at 1.25–1.29 atm the observed extinction fraction is 0.12 while F1 forecasts 0.69 (surrogate 0.54). At 1 atm the figures are 0.56 observed, 0.85 for F1 and 0.80 for the surrogate.
- **By fuel mole fraction:** the diluted 0.05–0.22 fuels have the lowest observed extinction (0.14) against F1's 0.70.
- **By horizon:** the extinction Brier of F1 departs from the surrogate's from 25 s on. At 40 s it is 0.190 against 0.087; at 120 s, 0.300 against 0.210. The termination Brier is equal up to 50 s and slightly worse after.
- **By cutoff:** the loss is the same at 1, 3 and 5 s.

**Centre against spread:**

| Tests | F1 centre median [IQR] | F1 width, inside | Surrogate centre | Surrogate width, inside |
|---|---|---|---|---|
| ended by extinction (28) | −0.67 [−0.75, −0.32] | 1.02, 15/28 | −0.41 [−0.50, −0.20] | 1.23, 23/28 |
| of which air (16) | −0.39 [−0.76, −0.07] | 0.78, 9/16 | −0.25 [−0.60, +0.10] | 0.93, 12/16 |
| of which 36–40 % O2 (11) | −0.72 [−0.74, −0.68] | 1.34, 6/11 | −0.46 [−0.47, −0.40] | 1.61, 11/11 |
| ended by termination (40), no stop | +0.26 [−0.16, +0.55] | 1.76, 33/40 | +0.51 [+0.09, +0.89] | 1.76, 26/40 |
| ended by termination (40), with stop | −0.21 [−0.27, −0.06] | 0.56, 15/40 | −0.19 [−0.25, −0.05] | 0.17, 18/40 |

- **Ends called too early.** F1 calls self-extinction too early by about a factor of two (median −0.67 in ln time), and its band is narrower than the surrogate's. Both centre and spread are off.
- **The surrogate errs the same way, by less** (−0.41).
- **The model ratio.** F1's t50 over the surrogate's t50 has median 0.87 in air (0.84–0.91), 0.84 at 29–30 % O2 (2 tests) and 0.76 at 36–40 % (0.52–1.00).

## Inverse flames (161 cases, 55 tests; none refused)

| Session | Tests | Pressure, injected O2, ambient C2H4 | Observed ext | F1 ext / term | F1 − surr, ext | F1 / surr P(ext ≤ 120 s) |
|---|---|---|---|---|---|---|
| 21328 | 4 | 1 atm, 0.21–0.40, 0.08–0.27 | 0.25 | 0.011 / 0.235 | −0.009 | 0.26 / 0.26 |
| 21349 | 8 | 0.5–1 atm, 0.21–0.50, 0.25–0.28 | 0.00 | 0.000 / 0.267 | 0.000 | 0.02 / 0.02 |
| 22024 | 14 | 0.5–1 atm, 0.26–0.85, 0.18–0.19 | 0.00 | 0.000 / 0.308 | 0.000 | 0.02 / 0.02 |
| 22027 | 14 | 3 atm, 0.32–0.85, 0.10 | 0.54 | 0.292 / 0.331 | −0.085 | 0.83 / 0.93 |
| 22035 | 15 | 0.5–2 atm, 0.29–0.85, 0.09–0.17 | 0.20 | 0.163 / 0.253 | −0.081 | 0.40 / 0.45 |

- **The fuel table acts only on lean ambients (ethylene 0.08–0.10).** There the model ratio of F1 to the surrogate is 1.81 (1.63–1.81). At 0.17 and above it is 1.00.
- **All the extinction loss is in the lean tests** (22 tests, all but one in 22027 and 22035): F1 0.299 against the surrogate's 0.412. Observed extinction there is 0.52, against 0.84 for F1 and 0.93 for the surrogate.
- **Centre for the lean tests that ended by extinction (11):** F1 +0.05 [−0.70, +0.26], width 1.48, inside 8/11; surrogate −0.54 [−1.19, −0.33].
- **The centre depends on pressure:** −0.71 at 0.5 atm (3 tests, session 22035) and +0.26 at 3 atm (7 tests, session 22027). Each pressure is a single session, so pressure and session are confounded.
- **Termination dominates without the stop.** Without the planned stop, 71 % of F1's summed loss is operator termination, and it is shared with the surrogate. Its centre is +1.0 to +1.2 in ln time (the flow stop comes far before the forecast end).
- **With the stop,** the termination centre is −0.03 to −0.13 with width 0.06, and only 8 of 44 tests fall inside the band. The stop is placed 3–12 % early with almost no spread. At 3 atm (22027) the centre stays +1.24, because the lean flames there are forecast to go out first.

## The registered expectations (dossier section 4) against the train days

| Expectation | What F1 says (model ratio F1/surrogate) | What the train days show | Reading |
|---|---|---|---|
| 4.1 normal, air: ethylene life = ethane's, 1.00 (0.90–1.10) | 0.87 (0.84–0.91), below the stated range | all 16 tests went out. Ends came later than both forecasts: t_end/t50 is 1.28 against the surrogate and 1.48 against F1 | the table shortens air flames where the dossier expected no change, and the data say ethylene lives longer than the ethane forecast |
| 4.1 normal, 30 % O2: 0.90 (0.70–1.25) | 0.84 | 2 tests | not testable |
| 4.1 normal, 38–40 % O2: 0.85 (0.55–1.25) | 0.76 (0.52–1.00), inside the range | 39 of 50 tests were held to the flow stop, where F1 forecast extinction with probability 0.73. The 11 that went out lived 1.6 times the surrogate's t50 | the direction is wrong: ethylene at 38–40 % outlived the ethane forecast; the table moves it further the other way |
| 4.1 within 30 % of the holding flow: held more often | not separated here (holding flow not tabulated) | n/a | not tested |
| 4.3 inverse, lean: 2.0 times the adopted clock (1.2–3.7) | 1.81 at ambient 0.08–0.10 | lean extinction ends came 1.7 times the surrogate's t50 (e^0.54); F1's centre is +0.05 | consistent, within the registered range |
| 4.3 inverse, ambient above 13.5 %: table changes nothing | 1.00 | no flame at ambient 0.17 or above went out by itself (33 tests) | consistent |
| 4.5 refuse below 1900 K | refused 6 normal tests at 1683–1829 K | all 6 were held to the flow stop; the surrogate scores 0.202 extinction on them | refusal works as written; the surrogate's forecast there is poor |

## The three largest structured errors

1. **Normal, 36–40 % O2: extinction forecast for flames that were held.**
   - 39 of the 50 tests at 36–40 % (sessions 19150, 19156, 19175, 19182 and part of 19171 and 19189) ran to the flow stop.
   - F1 gave them P(ext ≤ 120 s) of about 0.7, and the surrogate about 0.6.
   - This is 76 % of F1's extinction loss, and it is where the table loses most against the surrogate (+0.065 to +0.125 in each of the six sessions at enriched O2, against +0.002 to +0.010 in the two air sessions; worse at 1.26–1.29 atm).
   - It is a centre error in the event probability, not a width error. It holds across sessions.
2. **Normal: self-extinction called too early, with too narrow a band.**
   - Median −0.67 in ln time (a factor of about 2), coverage 15 of 28.
   - It holds in air (−0.39, 16 tests, 2 sessions) and at 36–40 % O2 (−0.72, 11 tests, 4 sessions).
   - The surrogate has the same sign at about 60 % of the size, so part of the error is in the ethane-based clock. The fuel table then adds −0.14 (air) to −0.26 (36–40 %) in ln time.
3. **Planned-stop placement on Flame Design: the stop comes 3–20 % early with almost no width.**
   - Termination ends sit −0.13 (inverse) and −0.21 (normal) in ln time from the forecast, with widths of 0.06 and 0.56.
   - Coverage is 8 of 44 inverse and 15 of 40 normal tests.
   - It is the same for F1 and the surrogate, so it is not a fuel-table error. It matches the about 2 s late scripted stop noted in 11.1.
   - The offsets were typed on CFI-G.

**Weaker, noted but not yet structured:** for the lean inverse flames, the centre goes from −0.71 at 0.5 atm to +0.26 at 3 atm. Each pressure is one session, so this cannot be separated from session.
