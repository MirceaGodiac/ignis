# Why the ethylene flames were "held": mechanism, train days, and the lines for F2

> Written by an author model (claude-fable-5-2) on 15 November 2026, after the blind score of PROTOCOL 12.8.
> Nothing fitted and no search: six stated questions, each written in
> `runs/dev/gen6/fuel/held/questions.md` before it was run. It is a proposal: no number here is a weight until
> it has been typed into an edits file and assembled.
> **Read for it:** the per-test rows of the Flame Design **train days only** (normal 19115, 19137, 19150,
> 19156, 19171, 19175, 19182, 19189), through `fd_fuel/train_days_cases.jsonl`, the train-day copy
> `fd_fuel/attribution/flame_design/` and the stored artifacts of the seven earlier variants; the 81 CFI-G
> **train** rows through `tools/wsp/analysis/lib.py`; the condition blocks of the CFI-G validation rows (fuel,
> Z_st and T_ad, to count ethane rows above Z_st 0.30: there is none); the three feedback reports of 12.8,
> which hold validation-day aggregates.
> **Not read:** any validation-day or test-day row of Flame Design, any s-Flame outcome, any CFI-G validation
> or test outcome, `holdout/`, `runs/dev/holdout_ext/`. Literature opened for this file: abstracts only
> (section 7); no per-flame table of any paper.
> Origin classes: **P** property, **T** theory, **L** published experiment, **J** judgement, **D** typed after
> reading these train days.

## 0. Main results

- **Most of the 39 "held" flames were stopped, not held.** Flame Design scripts ended the flow after 15 to
  100 s. Four of the 39 are extinctions that the experiment's own script detected and that the frozen sensor
  rule labels as flow stops (19150F1 at 92 s, 19150G2 at 50 s, 19189J1 at 58 s, 19175A3 at 132 s). The other
  35 were stopped at 8 to 110 s while still burning.
- **What separates the two groups is the fuel mass flow, and nothing else in the table.** Every flame at
  36 to 40 % oxygen that ended by itself burned 2.52 mg/s of ethylene (14 flames) or 1.95 mg/s (one). No flame
  at 1.3 mg/s or less ended by itself in any of six sessions. F1's own reduced flow does not separate them
  (1.2 to 3.9 for the stopped flames, 2.0 to 4.1 for those that ended by themselves).
- **The same boundary is in the CFI-G train alkanes, at 0.6 of the flow.** Ethane at 36 to 40 % oxygen went
  out inside 120 s at 1.5 mg/s and above (12 of 12) and never at 1.0 mg/s or below (0 of 16). For ethylene the
  step lies between 1.95 and 2.52 mg/s.
- **One number carries it in the hinge law.** Ethylene burns as ethane does at 0.58 of its fuel mass flow
  (a shift of −0.54 in the logarithm of the flow). With it the hinge member is centred on the eleven
  enriched-oxygen extinctions (median +0.09 in ln time, all within +0.02 to +0.30, four sessions), on the
  air flames at both ends of the flow range (two sessions), and on the 1.95 mg/s flame; and it expects 0.8
  extinctions before the stop among the 35 stopped flames, where none happened.
- **About 0.2 of that shift has a derivation** (oxygen demand through the flame radius, and ethylene's higher
  adiabatic temperature). The rest is consistent with a critical temperature some 200 K lower for ethylene
  than for the alkanes; that is a hypothesis, and a difference between the two campaigns cannot be excluded.
- **The butane direction was the wrong carrier.** Ethylene, the sootiest fuel of the four, is the
  longest-lived. The lead author's Lewis-number position repairs a tenth of what the hinge law needs.
- **F2 is a revision of the ethylene row only.** Coordinates (1, 0) in every member; `fuel.s` −0.31 in the
  hinge members and −0.02 in the growth members (each slot holds its own table). No CFI-G forecast moves: the
  210 train cases are equal to F1 case by case.
- **One error is left that no existing number of the ethylene row can express:** in the growth law a
  nitrogen-diluted flame is a larger flame and goes out sooner, and the diluted ethylene flames above
  Z_st = 0.36 did the opposite. Section 6 specifies the smallest forward change.

## 1. What "held" means for these flames

### 1.1 The three lengths and the clock (T, L)

A flame fed through a porous sphere settles where oxygen diffusing inward meets the fuel coming out:

```text
r_ss  = m_tot / (4 pi rhoD L)            L = ln(1 / (1 - Z_st)),  m_tot = m_F / Y_F
r_ext = sqrt(m_F / (4 pi m"_cr))         the radius at which the fuel flux falls to the critical flux
Phi   = (r_ss / r_ext)^2 = m_F m"_cr / (4 pi rhoD^2 (Y_F L)^2)
t     = r_ext^2 / (4 D erfcinv(Phi^-1/2)^2)      time for the growing flame to reach r_ext; none if Phi <= 1
```

Sources: the radius law and ρD = 0.0334 g m⁻¹ s⁻¹ (Kim et al. 2023, L); a critical temperature of 1130 K and
a critical fuel flux of 0.20 g m⁻² s⁻¹ for ethylene at 1 bar (Irace et al. 2023, L); the growth law as the
far-field oxygen transient (`lit_spherical_flames.md`, derived there). A flame is steady only if its steady
radius is inside the extinction radius *and* it gets there before radiation has cooled it to the critical
temperature; in an unbounded gas the approach takes of the order of 10⁵ s (Irace et al. 2023, second paper, L).
So inside a test of 30 to 150 s no flame is at its steady radius. **"Held" in these data means: the growth
clock is longer than the script.** Near Φ = 1 the clock diverges as `r_ext² / (π D (1 − Φ^−1/2)²)`, so the
band of flows that look held on a short test is wide.

### 1.2 What the published ethylene numbers say for each regime (Q6, T + L, nothing typed on alkanes)

With ρD = 0.0334, the flux 0.20 and D = 21 mm²/s at 1 atm (oxygen in nitrogen at 300 K, as 1/p), evaluated
once on the train-day conditions (`published_law.py`):

| Regime | Holding fuel flow, mg/s | Φ of the flames flown | Clock | Observed |
|---|---|---|---|---|
| Air, 1 atm, undiluted and diluted | 0.22 to 0.30 | 2.2 to 10 | 19 to 35 s, under every script (115 to 125 s) | all 16 went out, at 7 to 39 s |
| 36 to 40 % O2, undiluted, 2.52 mg/s, 1 atm | 0.83 to 0.97 | 2.6 to 3.0 | 79 to 97 s | out at 46 to 101 s |
| the same at 1.95 mg/s, 1.26 atm | 0.95 to 0.98 | 2.0 | 156 to 167 s | 132 s (script detection); stopped at 48 s |
| the same at 1.0 to 1.3 mg/s | 0.83 to 0.86 | 1.2 to 1.5 | 220 to 700 s | stopped at 37 to 67 s, burning |
| 36 to 40 % O2, 30 to 57 % fuel, 1.0 to 1.3 mg/s, 1.26 atm | 0.74 to 0.83 | 1.2 to 1.8 | 156 to 1170 s | stopped at 27 to 68 s, burning |
| 36 to 40 % O2, 17 to 22 % fuel, 1.0 to 1.3 mg/s | 0.60 to 0.68 | 1.5 to 2.2 | 90 to 244 s | stopped at 32 to 70 s, burning |
| 36 to 40 % O2, 5 to 17 % fuel, 0.64 to 1.95 mg/s | 0.28 to 0.58 | 2.0 to 4.1 | **26 to 69 s** | stopped at 8 to 110 s, burning; **seven of them after their clock, at 51 to 110 s** |

- **Air.** Oxygen is scarce, the flame stands far out (r_ss 24 to 100 mm against r_ext 16 to 32 mm), every
  flow is 2 to 10 times the holding flow and the clock is short. Held flames are not possible at these flows.
- **Enriched oxygen, undiluted.** L doubles, r_ss halves, and the holding flow rises to 0.8 to 1.0 mg/s. A flame
  at 1.0 to 1.3 mg/s is within 20 to 50 % of it; its clock is several minutes. Only the highest flow of the
  campaign has a clock shorter than the scripts.
- **Pressure.** ρD does not depend on pressure and the flux was published at 1 bar only; D falls as 1/p, so
  at 1.26 to 1.29 atm every clock is 1.27 times longer. Nothing else in the published law moves.
- **Dilution.** Here the published law fails in one direction: at 5 to 17 % fuel it gives clocks of 26 to 69 s
  for seven flames that were still burning at 51 to 110 s (19150A5, 19150D2, 19156B4, 19182C1, 19182C2,
  19182F1, 19189D1). The published range of the flux ends at 13 % fuel. This is the dilution error of
  section 3.4, and it is not an alkane-law error.
- **Where the published law is off in size:** 1.45 times too long for the undiluted and half-diluted flames
  at 2.52 mg/s (median −0.38 in ln time), about right in air below 1 mg/s (−0.22), twice too long in air at
  1.6 to 2.5 mg/s (−0.72), where the flames die within 7 to 14 s of ignition.

So for 34 of the 41 stopped flames (the six that F1 refuses included) the published law itself says: not out
before the stop. The held flames are what the theory expects; the question is why the alkane laws do not
expect them.

### 1.3 Why laws typed on ethane, propane and butane call them out

| Cause | Member | Size | Evidence |
|---|---|---|---|
| The hinge law reads the fuel mass flow as ethane's. Its clock for ethane is right (18 CFI-G train self-extinctions: median +0.03 in ln time, range −0.09 to +0.26), and ethane goes out inside 120 s from 1.5 mg/s up | hinge | medians of 23 to 37 s for the 2.52 mg/s flames that lasted 46 to 101 s; 56 to 72 s for 1.3 mg/s flames stopped at up to 70 s | section 3.2 |
| F1 placed ethylene half-way to butane | both | hinge ×0.80, growth ×0.75 on the clock at 37 to 39 % O2 | attribution report: two thirds of F1's excess |
| F1's −0.085 raises the growth law's critical flux to 0.196 g m⁻² s⁻¹ | growth | clock ×0.81 | growth residual +0.32 with it, +0.11 without |
| The growth law makes a diluted flame larger (r_ss ∝ m_tot / L) at an unchanged critical flux | growth | Φ ×1.3 to ×2.3 between 29 % and 8 % fuel at one fuel flow | section 3.4 |
| The path without the planned stop counts a flame stopped at 27 s against a forecast of extinction at 55 s | scoring | 19156C1, 19156K1, 19171D3 and the four script-detected extinctions are 45 % of what is left after the hinge shift | section 4 |

The hinge law has no held outcome and did not need one: with the flow read correctly its medians for the
flames at 1.3 mg/s and below are 300 to 3000 s, which is held on any horizon scored here.

## 2. The candidates, weighed

| Candidate | Verdict | Why |
|---|---|---|
| Steady radius from ethylene's stoichiometry | real, small | ethylene needs 8 % less oxygen per gram, so `Y_F L` is 8.4 % (undiluted) to 4.9 % (13 % fuel) larger than ethane's and Φ is 0.16 to 0.10 lower in ln at one fuel mass flow (T). The growth law has this through Z_st; the hinge law has no Z_st and must be told |
| Molar mass | negligible | 28.05 against 30.07: Y_F at 30 % by mole is 0.300 against 0.315 |
| Critical flux by the oxygen rule (m″_cr ∝ 1/σ) | not supported | with ethane's flux (0.180) the growth member is centred on the enriched extinctions (+0.11); with 0.196 it is 1.4 times early (+0.32); its zero is near 0.173. A fixed heat-release flux (ethylene 47.2 against ethane 47.5 kJ/g) fits; the published 0.20 is not reproduced in this law's frame, whose D and ρD are ethane numbers |
| Adiabatic temperature | real, small | ethylene is 70 to 90 K hotter at one oxygen level; Θ = (T_ad − T_cr) / (T_cr − T∞) is 5 % larger; Φ −0.05 (T) |
| Lower critical temperature of ethylene | the leading candidate for the rest, unproven | Φ ∝ 1/Θ. A critical temperature of 1300 to 1350 K for ethane against the published 1130 K for ethylene gives −0.30 to −0.38 at 2700 K and −0.38 to −0.49 at 2100 K, so nearly one shift for air and for enriched oxygen, which is what the flames show. The first dossier read 1270 to 1360 K on a thermocouple at butane extinctions. Against it: Santa et al. 2007 found about 1100 K for propane as for ethylene (short drop-tower flames, numerical) |
| Soot and radiative fraction | wrong sign as a fuel ranking | ethylene has 2.7 times ethane's soot index and outlives it. Soot may still be what switches off above Z_st ≈ 0.3 (section 3.4) |
| Growth clock (D) | not a fuel property | the far-field oxygen diffusivity cannot know the fuel; F1's butane half made it 1.4 times faster at 38 % oxygen, the larger part of F1's excess over the surrogate |
| A campaign difference (ignition procedure, endpoint taken from sensors here and from NASA's table in CFI-G) | cannot be excluded | no alkane flame was flown on a Flame Design train day |

## 3. What the train days show

### 3.1 Ended by itself against stopped (36 to 40 % oxygen, 50 flames that F1 answers, 6 that it refuses)

| Fuel flow, mg/s | Flames | Ended by themselves | Stopped by the script or an abort |
|---|---|---|---|
| 2.52 | 17 | 11 labelled extinction (42 to 101 s) and 3 detected by the script (50, 58, 92 s) | 3, at 27, 27 and 36 s |
| 1.95 | 3 | 1 detected by the script (132 s) | 2, at 48 and 100 s |
| 0.64 to 1.30 | 36 | 0 | 36, at 8 to 110 s (six of them below 1900 K) |

- The eleven labelled extinctions are undiluted (7), 50 % fuel at 1.29 atm (3) and 29 % fuel (1). The lives of
  the undiluted seven follow the oxygen level across three sessions: 46 s at 36.2 %, 57 to 58 s at 37.1 to 37.5 %, 61 s at 37.9 %,
  68 s at 38.7 %, 101 s at 39.4 %.
- "Detected by the script": the test-point file records an extinction 1 to 3 s before the flow change, and the
  frozen sensor rule calls the end a flow stop because a photomultiplier was still lit (these are 4 of the 18
  label disagreements of 11.3). Read as extinctions, they agree with their neighbours: 19189J1 at 58 s beside
  54, 55 and 52 s; 19150G2 at 50 s beside 42 s at 1.2 % less oxygen. The labels are not changed here.
- Pressure (1.0 against 1.26 to 1.29 atm), fuel fraction and adiabatic temperature do not separate the
  groups once the flow is given. The 1.26 to 1.29 atm sessions look "more held" because they flew 1.3 mg/s
  and less in all but six tests.

### 3.2 The same boundary in CFI-G train (ethane and butane, 36 to 40 % oxygen, T_ad above 2000 K)

| Fuel flow, mg/s | Ethane: out by 120 s / out later / flow stopped | Butane |
|---|---|---|
| 1.75 to 2.36 | 7 / 0 / 0 | 2 / 0 / 0 |
| 1.2 to 1.6 | 5 / 0 / 0 | 8 / 0 / 1 (the cycling flame of 21291) |
| 1.0 | 0 / 5 (123 to 258 s) / 5 | 1 / 1 / 1 |
| 0.5 to 0.85 | 0 / 1 (304 s) / 5 (two burning at 274 and 276 s) | 0 / 0 / 4 |

Ethane's step lies between 1.0 and 1.5 mg/s, ethylene's between 1.95 and 2.52: a ratio of 1.7 to 2. Ethane
flown undiluted, at 50 % and at 25 % at 2.0 mg/s lived 50 and 43, 42 and 37 s as the oxygen fell from 39.4 to
37.3 %: no effect of dilution down to Z_st = 0.29, as for ethylene (57 s undiluted, 42 to 50 s at 29 %).

### 3.3 Lives against the laws (seconds; medians include the 4 s floor)

| Test | atm | mg/s | X_F | O2 % | Z_st | End, s | F1 growth / hinge | Ethane law growth / hinge | Hinge at 0.58 of the flow |
|---|---|---|---|---|---|---|---|---|---|
| 19171D1 | 1.00 | 2.52 | 1.00 | 39.4 | 0.11 | self 101 | 39 / 33 | 66 / 37 | 75 |
| 19171D2 | 1.00 | 2.52 | 1.00 | 38.7 | 0.11 | self 68 | 36 / 30 | 60 / 32 | 65 |
| 19171D4 | 1.00 | 2.52 | 1.00 | 37.9 | 0.11 | self 61 | 33 / 26 | 55 / 29 | 57 |
| 19156C2 | 1.03 | 2.52 | 1.00 | 37.5 | 0.11 | self 58 | 32 / 25 | 52 / 27 | 53 |
| 19171D5 | 1.00 | 2.52 | 1.00 | 37.4 | 0.11 | self 57 | 32 / 25 | 52 / 27 | 53 |
| 19156C3 | 1.03 | 2.52 | 1.00 | 37.1 | 0.11 | self 57 | 31 / 23 | 49 / 25 | 49 |
| 19150J1 | 1.05 | 2.52 | 1.00 | 36.2 | 0.10 | self 46 | 28 / 20 | 44 / 23 | 42 |
| 19189J1 | 1.29 | 2.52 | 0.50 | 38.7 | 0.20 | script 58 | 32 / 26 | 52 / 29 | 56 |
| 19189J2 | 1.29 | 2.52 | 0.50 | 38.3 | 0.19 | self 54 | 31 / 25 | 50 / 27 | 53 |
| 19189J3 | 1.29 | 2.52 | 0.50 | 38.0 | 0.19 | self 55 | 30 / 24 | 48 / 26 | 50 |
| 19189K1 | 1.29 | 2.52 | 0.50 | 37.6 | 0.19 | self 52 | 29 / 22 | 46 / 24 | 47 |
| 19150G2 | 1.04 | 2.52 | 0.29 | 37.0 | 0.29 | script 50 | 28 / 23 | 44 / 25 | 49 |
| 19150G3 | 1.05 | 2.52 | 0.29 | 35.8 | 0.28 | self 42 | 26 / 19 | 39 / 22 | 40 |
| 19150F1 | 1.04 | 2.52 | 0.21 | 37.1 | 0.36 | script 92 | 26 / 22 | 41 / 25 | 49 |
| 19175A3 | 1.26 | 1.95 | 1.00 | 39.7 | 0.11 | script 132 | 48 / 42 | 89 / 47 | 148 |
| 19150D2 | 1.03 | 1.95 | 0.17 | 38.0 | 0.42 | stop 100 | 28 / 33 | 45 / 40 | 110 |
| 19182F1 | 1.29 | 0.81 | 0.08 | 37.8 | 0.59 | stop 110 | 29 / 222 | 51 / 257 | 1593 |
| 19150A5 | 1.02 | 0.81 | 0.08 | 39.3 | 0.62 | stop 59 | 32 / 425 | 56 / 508 | 3163 |
| 19175N1 | 1.26 | 1.30 | 0.21 | 38.3 | 0.36 | stop 70 | 41 / 63 | 74 / 62 | 370 |
| 19156J3 | 1.03 | 1.30 | 1.00 | 36.9 | 0.10 | stop 67 | 80 / 55 | 179 / 56 | 302 |
| 19175B4 | 1.26 | 1.30 | 0.30 | 38.8 | 0.29 | stop 68 | 57 / 76 | 113 / 72 | 428 |
| 19115B1 (air) | 1.01 | 0.64 | 1.00 | 20.6 | 0.06 | self 39 | 11 / 12 | 14 / 12 | 27 |
| 19115M4 (air) | 1.02 | 0.64 | 0.49 | 20.1 | 0.12 | self 24 | 10 / 12 | 12 / 12 | 25 |
| 19115F1 (air) | 1.01 | 0.64 | 0.29 | 20.4 | 0.18 | self 21 | 10 / 12 | 13 / 12 | 27 |
| 19115Q1 (air) | 1.02 | 0.80 | 1.00 | 20.1 | 0.06 | self 21 | 9 / 10 | 11 / 9 | 16 |
| 19115A1 (air) | 1.01 | 0.97 | 1.00 | 20.7 | 0.06 | self 34 | 9 / 9 | 11 / 9 | 14 |
| 19137L1, L7 (air) | 1.01 | 0.96 | 0.49 | 20.2 | 0.12 | self 14, 15 | 9 / 9 | 11 / 9 | 13 |
| nine air flames at 1.6 to 2.5 mg/s | 1.0 | | 0.15 to 1.0 | 20.2 to 21.0 | | self 7 to 14 | 9 to 10 / 8 to 9 | 10 to 12 / 9 | 9 |

"Ethane law" is ethylene at ethane's coordinates with no shift (the stored variant zqc).

Median of ln(observed end / median clock), by member, for the flames that ended by themselves:

| Group | Flames, sessions | F1 growth / hinge | Ethane law growth / hinge | Hinge at 0.58 of the flow |
|---|---|---|---|---|
| 36 to 40 % O2, labelled extinction | 11, 4 | +0.60 / +0.84 | +0.11 / +0.76 (+0.64 to +1.01) | **+0.09 (+0.02 to +0.30)** |
| the four script-detected extinctions | 4, 3 | +0.81 / +0.97 | +0.27 / +0.87 | +0.03 (−0.11 to +0.63) |
| air, below 1 mg/s | 7, 2 | +0.86 / +0.73 | +0.65 / +0.74 | +0.13 (−0.25 to +0.89) |
| air, 1.6 to 2.5 mg/s | 9, 2 | +0.06 / +0.20 | −0.12 / +0.08 | +0.08 (the shift does not act there) |

Expected number of extinctions before the stop among the 35 stopped flames at 36 to 40 % oxygen (sum over
flames of the member's probability of extinction by the time the flow was stopped; observed: none):

| | F1 | Ethane law | Hinge at 0.58 of the flow, growth as ethane | the same with the dilution unit of Q4 |
|---|---|---|---|---|
| growth member | 15.3 | 7.7 | 7.7 (6.1 of them at Z_st ≥ 0.3) | 3.1 |
| hinge member | 7.0 | 6.7 | **0.8** | 0.8 |

### 3.4 The dilution error of the growth law

At one fuel mass flow and oxygen level the lives at 2.52 mg/s are 57 s undiluted, 52 to 55 s at 50 % fuel
(1.29 atm), 42 to 50 s at 29 % fuel (Z_st 0.29), and 92 s at 21 % fuel (Z_st 0.36). The growth law forecasts
39 to 53 s for all four and, at one oxygen level, the shortest for the last. Above Z_st = 0.36 nothing else ended by itself:
19150D2 burned 100 s where the growth median is 45 s, 19182F1 110 s against 51 s, and the growth member
expects 6.1 extinctions before the stop among the 23 diluted stopped flames of five sessions (19150, 19156,
19175, 19182, 19189), where none happened. The published law errs the same way (section 1.2).

Flame Design moved nitrogen from the oxidiser to the fuel to make soot-free flames (Sunderland et al. 2003:
soot at low Z_st, none at high Z_st). A flame without soot and with a nitrogen-filled interior loses less heat
per unit area, so its critical flux is lower. That would act above a sooting limit in Z_st and for a sooty
fuel, which is what the one clean pair shows. It is an inference: no flame colour or soot signal was read.
CFI-G train has no strong alkane flame above Z_st = 0.33 (ethane: 0.29), so the alkane laws were never told.

## 4. The diagnostic questions and their answers

Score: integrated Brier 6 to 120 s on the 204 train-day cases F1 answers, extinction / operator termination.
"Centre" is the report's ln(t50 / end) for the committee. F1, the surrogate and zqc were re-run in the same
call and reproduce the registered numbers exactly.

| | Hinge row / growth row (coordinates; `fuel.s`) | No stop | With stop | Centre, air / 36 to 40 % | Sessions better than the surrogate (extinction, no stop) |
|---|---|---|---|---|---|
| F1 | (0.5, 0.5); −0.085 in both | 0.2061 / 0.2905 | 0.1429 / 0.0790 | −0.39 / −0.72 | 0 of 8 |
| surrogate | ethane's indicators | 0.1272 / 0.2763 | 0.0889 / 0.0651 | −0.25 / −0.46 | |
| zqc (stored) | (1, 0); 0 in both | 0.1431 / 0.2801 | 0.0959 / 0.0661 | −0.27 / −0.47 | 1 of 8 |
| **Q1** zha | (1, 0); −0.54 / (1, 0); 0 | 0.0746 / 0.2702 | 0.0540 / 0.0579 | −0.15 / −0.11 | 8 of 8 |
| **Q2** zhb | (1, 0); −0.31 / (1, 0); 0 | 0.0944 / 0.2716 | 0.0692 / 0.0585 | −0.22 / −0.26 | 8 of 8 |
| **Q3** zhc | (1.16, −0.16); −0.035 in both | 0.1314 / 0.2783 | 0.0896 / 0.0639 | −0.27 / −0.41 | 3 of 8 |
| **Q4** zhd | as Q1, and a Z_st unit in the growth law | 0.0577 / 0.2635 | 0.0476 / 0.0583 | −0.15 / −0.11 | 8 of 8 |
| **Q5** zhe | (1, 0); −0.31 / (1, 0); −0.02 (the F2 row) | 0.0990 / 0.2728 | 0.0720 / 0.0589 | −0.24 / −0.28 | 8 of 8 |

- **Q1. Is the hinge law's error one shift of the flow variable?** Yes. One number centres the enriched
  extinctions, the air flames at 0.64 to 0.97 mg/s (which the unshifted law calls 2 to 4 times early) and the
  1.95 mg/s flame, and leaves the air flames at 1.6 to 2.5 mg/s where they were. Had it been a factor on time
  (which is what coordinates beyond ethane give, and only in enriched oxygen), the low-flow air flames would
  have stayed early: under zqe the centre in air stayed at −0.25. Hand estimates written before the run: 53 to 74 s for the 2.52 mg/s
  flames (run: 53 to 75), 29 / 17 / 14 s in air at 0.64 / 0.80 / 0.97 mg/s (run: 25 to 27 / 16 / 13 to 14).
- **Q2. Half the step.** Still better than the surrogate in all eight sessions; the centre is half-way.
- **Q3. The lead author's hypothesis.** Level with the surrogate; the centre at 36 to 40 % moves from −0.47
  to −0.41. See section 5.
- **Q4. Does the growth law's reduced flow rise with dilution where the flames say it does not?** Yes. A unit
  that cancels that rise above Z_st = 0.30 takes the growth member's expected early extinctions among the
  stopped flames from 7.7 to 3.1 and improves five sessions; nothing else moves (the air and undiluted
  sessions change by less than 0.0001). On CFI-G train it is silent: all 210 cases equal to F1, and no train
  or validation ethane row has Z_st above 0.30. It is not proposed as a line (section 6).
- **Q5. The literal F2 row.** Extinction 0.2061 → 0.0990 without the stop and 0.1429 → 0.0720 with it;
  termination 0.2905 → 0.2728 and 0.0790 → 0.0589. By session, extinction without the stop (F1 / surrogate /
  F2 row): 19115 0.078 / 0.068 / 0.060; 19137 0.015 / 0.013 / 0.012; 19150 0.311 / 0.190 / 0.166;
  19156 0.276 / 0.197 / 0.156; 19171 0.277 / 0.190 / 0.151; 19175 0.278 / 0.153 / 0.075;
  19182 0.093 / 0.028 / 0.027; 19189 0.246 / 0.153 / 0.144. With the stop it is better than the surrogate in
  seven sessions and worse in 19182 (0.013 against 0.007). These are the days the row was typed from: this is
  not a test of it. CFI-G train: 0.052162 / 0.023997 on 210 cases, every case equal to F1.
- **Q6. The published ethylene numbers alone.** Section 1.2.

**What is left after the full shift (zha, no stop, 0.0746):** the four script-detected extinctions 0.019; the
three 2.52 mg/s flames stopped at 27 to 36 s 0.015; the eleven extinctions 0.014 (a band 1.4 wide in ln time);
the air flames 0.007; the remaining stopped flames 0.020, most of it the growth member's dilution error. The
first two items, 45 % of the total, are forecasts of extinction for flames that would have gone out, or did.

## 5. The lead author's hypothesis

**(a) Coordinates from the fuel Lewis number, (1.16, −0.16).** It does not survive.

- *Mechanism (T).* For a normal flame the fuel is carried to the sheet by the flow; the standoff and the sheet
  temperature are set on the oxygen side (the fuels dossier, 1.2). The fuel's diffusivity has no first-order
  place. The Lewis number would be a label for whatever butane has, not a cause.
- *Size.* On that axis ethylene would gain +0.07 (hinge) to +0.10 (growth) in ln life at 39 % oxygen and
  +0.03 in air. The hinge member needs +0.76. Q3 moves the centre by 0.06.
- *Shape.* The butane terms act in enriched oxygen and as a factor on time. The ethylene residual is a shift
  of the flow and is as large in air at low flow as in enriched oxygen.
- *What does survive:* the direction "away from butane", already in the attribution report (zqe).

**(b) The oxygen-demand shift nearer −0.02 to −0.05.** Half survives, in the member it was not meant for.

- In the **growth** member the −0.085 should go: the train days put its zero at about +0.04 and theory offers
  −0.085 (oxygen rule), −0.03 (the energy balance, `Y_F L / Θ`) and −0.007 (a fixed heat-release flux). Less
  negative is right.
- In the **hinge** member, which section 3.5 of the fuels dossier was about, the correction for diluted fuel
  is real (the geometry is worth −0.16 undiluted and −0.10 at 13 % fuel) but it is a fifth of what that member
  lacks, and the residual does not shrink with dilution: the 29 % and 50 % flames need the same −0.54 as the
  undiluted ones. The data ask for a more negative number there, not a less negative one.

## 6. The revision

### 6.1 Literal lines for F2

Every slot of `acme_normal.wsp12` holds its own fuel table, so the two laws can be told different things. The
two tables differ in one number. Everything not shown is as in `configs/models/acme/fuel_tables/normal-f1.wsp`.

Hinge members (c0 to c3, `acme_normal.wsp11`; and a0, a1, `acme_normal.wsp9`, which carry no weight):

```text
# Ethylene at ethane's position: no butane term. The butane terms shorten a flame in enriched oxygen and
# ethylene, the sootiest of the four fuels, outlived the ethane law in six sessions.
fuel.se_coord[0:6, 0:2] = [[1.0, 0.0], [0.0, 0.0], [0.0, 1.0], [1.0, 0.0], [1.0, 0.0], [0.0, 0.0]]
# In this law fuel.s is the whole shift of the reduced flow at one fuel mass flow, because the law reads
# no Z_st. Ethylene: -0.16 from its oxygen demand through the flame radius and -0.05 from its adiabatic
# temperature (theory), and a third of the further -0.33 that the train days of Flame Design ask for.
fuel.s[0:6] = [0.0, 0.0, 0.0, -0.31, 0.068, 0.757]
```

Growth members (b0 to b3, `acme_normal.wsp10`):

```text
fuel.se_coord[0:6, 0:2] = [[1.0, 0.0], [0.0, 0.0], [0.0, 1.0], [1.0, 0.0], [1.0, 0.0], [0.0, 0.0]]
# In this law Z_st already carries the oxygen demand into the flame radius; fuel.s is the shift of the
# critical fuel flux alone. Ethylene: 0.184 g m-2 s-1, between the energy balance (-0.03) and a fixed
# heat-release flux (-0.007); the oxygen rule (-0.085) made the clock 1.4 times early on eleven flames.
fuel.s[0:6] = [0.0, 0.0, 0.0, -0.02, 0.068, 0.757]
```

| Number | F1 → F2 | Class | Allowed range | Asked by the train days | Reason for the typed value |
|---|---|---|---|---|---|
| `fuel.se_coord[3]`, all members | (0.5, 0.5) → (1.0, 0.0) | J → D | e 1.0 to 1.5, b −0.5 to 0 | (1.5, −0.5) was the best coordinate-only variant | half of that step lands on ethane's law, which is also the clean statement "no butane term"; what lies beyond is a flow shift, typed below |
| `fuel.s[3]`, hinge members | −0.085 → −0.31 | T + D | −0.70 to −0.10: −0.10 is the oxygen rule at 30 % fuel with the T_ad term; −0.70 is geometry, T_ad and an alkane critical temperature of 1400 K, the top of the first dossier's range | −0.54 (Q1) | the damped step: half of −0.455. Of the −0.31, −0.21 is theory |
| `fuel.s[3]`, growth members | −0.085 → −0.02 | T + D | −0.085 (oxygen rule) to +0.05 | about +0.04 (zero of the member's residual between zqa, zqc and zqd) | half of the step; inside the two theory values that are not the oxygen rule |

The pattern behind each: the hinge residual of +0.64 to +1.01 in four enriched sessions and +0.41 to +1.31
in the low-flow tests of both air sessions; the growth residual with the oxygen rule (median +0.32 at ethane's
position, zqa) in the same four sessions; the butane half worse in all eight sessions (attribution report).

To build: two header files (hinge, growth), `tools/wsp/fuel_member_edits.py` per member, then the committee as
for F1. `runs/dev/gen6/fuel/assemble_held.py` does this for the scratch variant zhe and can serve as the
pattern.

### 6.2 What F2 must not change

- **Methane** (no data here): every entry of its row, including `fuel.s` = +0.068 in both tables. Note for the
  day methane data arrives: that number is the oxygen rule, which ethylene did not obey in either law.
- **Ethane, propane, butane, hydrogen**: unit coordinates, zero shifts, `admit`; with them every CFI-G forecast
  stays as it is, in every partition.
- **`fuel.tad_min_k` = 1900 K for ethylene.** The six refused flames (1683 to 1829 K, Z_st 0.67 to 0.72) all
  burned to their stops, four of them for 51 to 79 s. Below 1900 K the alkane weak-flame law gives early death
  with probability 0.3 to 0.9 at 1680 to 1775 K, and the growth law's dilution error is at its largest.
  Refusing is the honest answer until section 6.4 is built.
- **`fuel.ft_coord`, `fuel.mw_g_mol`, `fuel.dosc`**, every law statement of every member, the committee's
  weights, eras and stop offsets.
- **The inverse table.** It added something as registered; its open question (centre against pressure) is one
  session per pressure.
- **Labels and thresholds of the Flame Design chain.**

### 6.3 What F2 moves

- **CFI-G train:** nothing. Extinction 0.052162, termination 0.023997, 210 cases, each equal to F1
  (`attrib.py decompose` on the scratch build of the F2 row). No alkane row is touched, so the same holds for
  validation and test by construction.
- **Flame Design train days** (typed from them, so not evidence for it): section 4, Q5.
- **Flame Design validation days:** the registered report says every normal validation-day case ended by a
  flow stop. That day can show whether F2 still calls stopped flames out; it cannot test the centre of the
  extinction clock.

### 6.4 What needs a new forward

Two things. Neither is forced into F2.

**(1) Dilution in the growth law.** The ethylene row has no entry that depends on Z_st. The existing tensors
could carry it only as a unit on the ethane vertex (Q4), which would assert about ethane what no ethane flame
has shown, and if the cause is soot it should not hold for ethane at all. Smallest change, in `flame6.rs` (and
its committee slots):

| Tensor | Shape | Bounds | Unit | Meaning |
|---|---|---|---|---|
| `fuel.zst0` | [6] | (0, 1) | 1 | Z_st above which the fuel's flames lose less heat (soot-free, nitrogen-filled interior) |
| `fuel.gz` | [6] | (−10, 10) | ln per unit of Z_st | fall of ln of the critical flux per unit of Z_st above `fuel.zst0` |

```text
growth law, normal flames only:
  ln m_cr = N*_se(se.flux ; c) - fuel.s[k] - fuel.gz[k] * max(0, zst - fuel.zst0[k])
everything else unchanged; the hinge law needs nothing (it has no dilution geometry to undo)
alkanes: fuel.zst0 = 1.0, fuel.gz = 0.0  ->  "- 0.0", bit for bit as now (acceptance test as in 12.8)
```

Values to type when it exists: ethylene `fuel.zst0` = 0.30 (D: between the last flame on the law, 0.29, and
the first off it, 0.36; L: the sooting limit rises with Z_st); `fuel.gz` = 1.0 (D, half of the 2.0 asked in
Q4; range 0 to 2.0, where 2.0 is the value at which the growth law's reduced flow stops rising between 29 %
and 8 % fuel, T). Methane 1.0 and 0.0 until it has data. On the train days the full value takes the
extinction score from 0.0746 to 0.0577 beside the full hinge shift.

**(2) The weak-flame law is an alkane law.** To answer ethylene below 1900 K the forward needs
`fuel.dtad_weak_k` [6], bounds (−1000, 1000), K: `logit weak = N*_se(se.flash ; c with c_tad +
fuel.dtad_weak_k[k] / norm.tad_scale_k)`, zero for the alkanes. The train days say only that it is at least
+150 K for ethylene (flames at 1722 to 1773 K burned 51 to 54 s). It is useless without (1), because the
flames below 1900 K are the most diluted ones.

Not needed: a "held" outcome in the hinge law (section 1.3).

## 7. Sources, checks, exposure

**Literature.** Already in the repository and used through it: Kim et al., PCI 39 (2023) 1647 (radius law,
ρD); Santa et al., Combust. Flame 151 (2007) 665 (loss fraction 0.7 at extinction, about 1100 K for propane
and ethylene, loss growing with the product layer); Sunderland et al., Combust. Flame 132 (2003) 25 (soot and
Z_st); Li and Sunderland 2012 (smoke points); Marrero and Mason (D of O2 in N2). Opened for this file, abstracts
only: Irace, Waddell, Constales, Sunderland and Axelbaum, PCI 39 (2023) 1905 (1130 K; 0.2 and 0.68 g m⁻² s⁻¹; an
equation for the extinction radius from the reactant flow); Irace et al., PCI 39 (2023), "On the existence of
steady-state gaseous microgravity spherical diffusion flames in the presence of radiation heat loss" (the two
conditions for a steady flame; 10⁵ s to steady state in an unbounded gas; steady behaviour seen for up to
870 s); Tang, Im and Atreya, Combust. Flame 157 (2010) 127 (a critical flame temperature near 1130 K over a
wide range of dilution in a computed flame; the fuel is not named in the abstract; no steady flame in an
unbounded domain); Waddell et al., Combust. Theory Model. (2024) (warm-flame chemistry of ethylene delays
extinction and cannot prevent it). The published ranges of Irace et al. are the Flame Design campaign itself;
only the stated laws were used.

**Recomputed by hand and by the scripts named below:** `Y_F L` for ethane and ethylene at four fuel fractions
and two oxygen levels; Θ and its ratio for critical temperatures of 1130, 1300 and 1360 K at 2050 to 2800 K;
the hinge clock of `wsp8/normal/d1-early` typed out and checked against the member's own diagnostics.

**Scratch (local, not in git), all under `runs/dev/gen6/fuel/`:** `assemble_held.py` (builds zha to zhe);
`held/questions.md`; `held/score_held.py` and `held/score/` (train-day scores, per-case rows);
`held/resid.py` (member clocks against ends); `held/published_law.py`; `held/cfig_boundary.py`;
`held/table.py`. Scratch model folders `zh*` under `configs/models/acme/wsp{9,10,11,12}/normal/`
(`assemble_held.py --clean` removes them); their releases are in the registry like those of `zq*`. The
decompose tool wrote to its fixed place, `runs/dev/attrib/held_{f1-fable,zha,zhd,zhe}/`.

```bash
python runs/dev/gen6/fuel/assemble_held.py zha zhb zhc zhd zhe
python runs/dev/gen6/fuel/held/score_held.py --ignis target-merged/release/ignis.exe --with-refs zha zhb zhc zhd zhe
python runs/dev/gen6/fuel/held/table.py acme_normal && python runs/dev/gen6/fuel/held/resid.py zha
python runs/dev/gen6/fuel/held/published_law.py && python runs/dev/gen6/fuel/held/cfig_boundary.py
python tools/wsp/attrib/attrib.py decompose --family acme_normal12 --candidate configs/models/acme/wsp12/normal/zhe --out runs/dev/attrib/held_zhe
```

**Exposure.** Every number typed here is data-informed by the train days of Flame Design. The author knew the
aggregate validation-day scores of 12.8 from the registered report. Six of the ten allowed questions were
used; five built releases.

**Weakest points.**

1. **The size of the hinge shift is read, not derived.** Theory gives −0.21 of the −0.54. The rest needs
   ethylene to go out some 200 K colder than ethane, which no source states for ethane and one source
   contradicts for propane; or it is a difference between the 2019 and 2021 campaigns that these days cannot
   separate from the fuel. If it is the campaign, the shift is right for Flame Design and wrong for ethylene
   elsewhere.
2. **The flow dependence in enriched oxygen rests on two flows.** Fourteen flames at 2.52 mg/s and one at
   1.95 mg/s ended by themselves; everything below is a lower bound from a stop at 8 to 110 s. That one, and
   three others, count as extinctions only on the script's detection, against the frozen label. The air flames
   are the independent support for a shift of the flow instead of a factor on time.
3. **The dilution error stays in F2, and its cause is inferred.** One clean pair in one session (92 s against
   42 to 50 s) and lower bounds in four; soot was not observed. With F2 the growth member still expects about
   six extinctions before the stop among the 23 diluted stopped flames, and it is still 1.9 times early for
   air flames below 1 mg/s, which no entry of the row reaches.
