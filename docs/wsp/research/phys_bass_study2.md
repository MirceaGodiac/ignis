# Plastics and fabrics (BASS, BASS-II): second study

Author: `claude-fable-5-2`, 15 November 2026, PROTOCOL 13.6. Addendum to `phys_bass_study.md`.
Data-informed and in-sample except where a section says otherwise. No test day opened.

## A. Fabrics, "kept burning": expectation typed before any fabric outcome was read

**Written 2026-11-15T08:44Z, before opening** `bass3_result.md`, `bass1_coding_v2.md`, or any fabric
row of the case files, the what-if case tables or the study files.

**What I had already seen of fabric outcomes, to be honest about "zero-shot".** (1) Published
statements read for W0: Ultem and Nomex HT90-40 "did not burn" in cabin air at about 15 cm/s;
Nomex-III "did burn" at 22 % and 14 cm/s and went out 1 cm before the end of the sample; SIBAL
burned in cabin air from 10 down to about 1 cm/s opposed and quenched near 1 cm/s concurrent.
(2) My colleague's zero-shot law for SIBAL (`phys_quench_w0.md`), whose constants I reuse.
(3) Two fabric rows that passed in a file head earlier today (two SIBAL burns in air at 5 and
10 cm/s, both coded as burning), and the counts of fabric refusals. (4) The lane's overall train
incidence of "kept burning", 0.888 for all materials together. Nothing else. So: for SIBAL the
numbers below are a law from theory checked against published summaries, not against cases; for
Nomex and Ultem they lean on one published sentence each and are not blind.

**Physics.** A thin fabric has no cold interior; what decides whether it keeps burning once the
igniter is off is the oxygen margin over a limit with a floor, a blow-off rise and a radiative
quench term, as for a film:

```text
X_lim(U) = x_f + s_f U + A_f / U        U = duct speed at ignition, cm/s
P(kept burning) = Phi( (X - X_lim) / s ),   s^2 = sd_x^2 + (0.35 A_f / U)^2 + (0.4 s_f U)^2
```

The case files do not give the flow direction for most fabric burns, so the constants are the
middle of my colleague's concurrent and opposed values, plus his ignition margin (0.004):

| Material | `x_f` | `sd_x` | `A_f`, cm/s | `s_f`, per cm/s | Origin |
|---|---|---|---|---|---|
| SIBAL (75 % cotton, 25 % glass, 18 mg/cm2) | 0.152 | 0.012 | 0.070 (2 cm wide), 0.085 (1 cm) | 0.0012 | `phys_quench_w0.md`: floor 0.148 + 0.004; `A` between 0.054 and 0.089; `s` between 0.0008 and 0.0016 |
| Nomex III (aramid, upward limit 22 % in 1 g) | 0.195 | 0.015 | 0.070 | 0.0012 | microgravity limit 1 to 2.5 points under the 1 g one, as for cellulose and PMMA; flow terms as SIBAL (J) |
| Nomex HT90-40 (upward limit 23.5 %) | 0.220 | 0.015 | 0.070 | 0.0012 | same shift |
| Nomex, type not in the inputs | 0.210 | 0.018 | 0.070 | 0.0012 | between the two |
| Ultem film (upward limit 23.5 %) | 0.220 | 0.015 | 0.070 | 0.0012 | as HT90-40 |

**Registered expectation** (probability of keeping burning):

| Material, oxygen | 1 cm/s | 2 cm/s | 5 cm/s | 10 cm/s | 20 cm/s | 40 cm/s |
|---|---:|---:|---:|---:|---:|---:|
| SIBAL 2 cm, 21.5 % | 0.38 | 0.93 | 1.00 | 1.00 | 0.99 | 0.72 |
| SIBAL 2 cm, 18 % | 0.06 | 0.29 | 0.73 | 0.75 | 0.51 | 0.17 |
| SIBAL 2 cm, 17 % | 0.03 | 0.13 | 0.44 | 0.47 | 0.27 | 0.08 |
| SIBAL 2 cm, 16 % | 0.01 | 0.04 | 0.18 | 0.20 | 0.10 | 0.03 |
| Nomex III, 22 % | | | 0.62 | 0.65 | 0.44 | |
| Nomex HT90-40, 22 % | | | 0.10 | 0.12 | 0.06 | |
| Nomex, type unknown, 22 % | | | 0.29 | 0.31 | 0.19 | |
| Ultem, 22 % | | | 0.10 | 0.12 | 0.06 | |

(The table is the formula above evaluated by hand; arithmetic re-checked at 08:45Z, still before
any fabric outcome was opened.)

What I expect to find: SIBAL kept burning in nearly every cabin-air burn lit at 3 cm/s or more
(rate 0.95 or higher); its failures on the 2013 days vitiated to 16 to 18 %, or at 2 cm/s and
below; Nomex mostly not burning (rate 0.2 to 0.4, the burning ones Nomex III); Ultem not burning.
The expectation fails if SIBAL in air at 5 to 20 cm/s holds in fewer than 85 % of burns, if SIBAL
at 16 to 17 % holds in more than 80 %, or if Nomex holds in more than 70 %.

## B. Fabrics: the cases, opened at 08:48Z, against section A

`runs/dev/gen7/study/bass_fabric.py` evaluates the formula of section A on the 19 fabric and Ultem
burns of the development days (second coding). Nothing was changed in A after this point.

| Material | Burns | Kept burning | Expected (mean of A) | Brier of A | "Always lit" | Incidence 0.888 |
|---|---:|---:|---:|---:|---:|---:|
| SIBAL, cabin air 21.6 to 22.7 %, 5 to 22 cm/s | 9 | 9 | 0.998 | 0.0000 | 0.000 | 0.0125 |
| Nomex (III and blend), 21.0 to 22.5 % | 9 | 1 | 0.38 | 0.165 | 0.889 | 0.702 |
| Ultem sheet, 22.5 % | 1 | 0 | 0.20 | 0.042 | 1.000 | 0.789 |

The Nomex burns one by one:

| Test | Material words | O2, % | cm/s | Kept burning | A said |
|---|---|---:|---:|---|---:|
| PSI-25 F1 | Nomex natural (Nomex III), concurrent | 22.2 | 9.6 | no | 0.70 |
| PSI-25 F2 | same | 21.0 | 15.1 | no | 0.32 |
| PSI-25 F3 | same | 21.7 | 8.1 | no | 0.59 |
| PSI-26 14 | Nomex III natural | 22.0 | 14.6 | **yes** | 0.56 |
| PSI-26 15 | Nomex natural III | 22.5 | 30.2 | no | 0.35 |
| PSI-26 12, 12-R, 12-R2, 13 | Nomex blend | 21.2 to 22.5 | 9 to 15 | no (4 of 4) | 0.18 to 0.35 |

**Verdict.**

- **SIBAL: the expectation holds, and the test is weak.** Nine of nine kept burning where A said
  0.98 to 1.00. No development burn of SIBAL in this lane's files is at reduced oxygen or below
  5 cm/s, so the part of the law that matters for safety (the limit near 17 %, the quench below
  2 cm/s) is untested here. It is zero-shot: constants from `phys_quench_w0.md`, typed before the
  cases.
- **Nomex and Ultem: direction right, Nomex III wrong.** The fail criteria of A are not met (one
  of ten kept burning). But I gave Nomex III 0.32 to 0.70 and four of its five burns did not hold.
  The one that held is the burn the published slides describe, which is why I had it high. The
  assumption that failed: that microgravity lowers the limit of this fabric by 1 to 2.5 points as
  it does for cellulose and PMMA. For a charring aramid it does not: the limit stays at or above
  the 1 g upward one.

**What goes into the law (section E).** SIBAL as in A. Nomex of both kinds and Ultem with one
floor, `x_f = 0.215` (the 1 g upward limits, 22 and 23.5 %, with no microgravity gain), in place
of 0.195, 0.220 and 0.210. That number is **data-informed** (typed after the ten cases); only the
SIBAL row is zero-shot.

## C. "Kept burning" on slabs (second coding)

All 18 slab burns the pair answers (`runs/dev/gen7/whatif_cases/bass3_s1_v2.csv`):

| Slabs | Burns | Kept burning | Pair says |
|---|---:|---:|---|
| two-sided, 1 to 2 mm, 18.8 to 21.8 % | 6 | 6 | 0.95 to 1.00 |
| two-sided, 3 to 5 mm, 17.9 to 18.0 % (M9, M18, M20) | 3 | 3 | 0.94 to 0.97 |
| two-sided, 3 to 5 mm, 20.0 to 22.2 % | 4 | 4 | 0.99 to 1.00 |
| one-sided, PSI-25, 2 cm wide, 1 to 5 mm, 20.8 to 22.2 % (M1, M14, M2, M3) | 4 | 4 | 1.00 |
| one-sided, PSI-26, 1 cm wide, 2 mm, 21.0 %, opposed (tests 72, 73) | 2 | **0** | 0.99, 1.00 |

The two failures are the whole loss (0.111 by case). They are the third and fourth burn of one
sample on one day, lit at the end that sits in the metal holder, at 9 and 15 cm/s. The operators
wrote: "apparently not flammable in opposed flow ... heat losses are different though, due to the
metal holder".

**Is it the two-class physics of the quench speed? Half of it.**

- The thickness-and-oxygen threshold `X_c` of bass3 must **not** go into the pair. It is the
  starting oxygen below which a slab reaches its limit *during a long burn*; it contains the
  oxygen the burn consumes. The three thick two-sided slabs at 17.9 to 18.0 % (below their `X_c`
  of 19.1 to 20.6 %) all lit and burned for 11 to 26 minutes. With `X_c` in the pair they would
  be forecast 0.1 to 0.3 and all three would be wrong.
- The one-sided part is the same physics. The oxygen at which the one-sided slabs died is
  recorded: 19.7, 21.2 and 21.6 % (M3, M2, M14, read after the burn). A slab burning on one side
  has its floor near 21 %, not at the 13.8 % of a film; a burn started at 21.0 % is on that floor.
  This is evidence from the quench side, not from tests 72 and 73.

### S1: a 21 % floor for one-sided flat samples (logged, `runs/dev/gen7/study/bass3_s1.wsp`, coding v2)

Two elements on `flat_holds_low_flow`: one hinge on `is_one_sided` that lowers the logit by
`20 ln(0.21 / 0.138) = 8.40`.

| | Expected (in the log before the run) | Realised |
|---|---|---|
| slabs, case mean | 0.111 to about 0.055 | 0.111 to 0.055 (all cases 0.0328 to 0.0228) |
| tests 72, 73 | 1.0 to 0.25 each | validation 0.098 to 0.041 by case |
| M1, M14, M2 (held at 22.2 %); M3 (held at 20.8 %) | 0 to 0.06 each; 0 to 0.30 | train 0.010 to 0.016 by case |
| pooled, day mean | lower | 0.0467 to 0.0247 (train 0.007 to 0.013, validation 0.122 to 0.048) |
| days | 1 of 3 lower | 1 of 3 lower |
| quench | identical | identical |

Predicted in sign and size. **By the rule it is a comparator, not the model:** three days change
and one is lower. That is also my reading of the evidence: it buys two burns of one sample
against four slabs that held, one of them (M3) below the new floor.

**Settled.** The flat pair does not carry `X_c`. It may carry the one-sided floor; I report S1 as
a comparator and leave the choice to the coordinator. The lines are in section E, marked.

## D. What is left in the quench speed after bass3 (second coding)

`bass3` on v2: 0.152 by day, 0.249 by case, 56 scored cases in the case table, total loss 12.8 cm/s.

| Kind | Cases | Loss | Share |
|---|---:|---:|---:|
| slabs | 14 | 11.83 | 93 % |
| rods (all configurations) | 23 | 0.72 | 6 % |
| spheres | 14 | 0.14 | 1 % |
| films | 5 | 0.09 | 1 % |

The slabs, largest first:

| Test | Slab | Went out at, cm/s | Forecast median | Loss | What it is |
|---|---|---|---:|---:|---|
| M11 | 5 mm, two sides, 20.0 % | 12.1 to 15.1 | 4.2 | 4.28 | wrong class: half-way between floor and limit |
| M20 | 5 mm, two sides, 18.0 % | 8.1 to 10.1 | 12.8 | 1.57 | right class, 35 % high |
| M3 | 5 mm, one side, 20.8 % | 18.1 to 20.1 | 15.7 | 1.56 | right class, 20 % low |
| M14 | 2 mm, one side, 22.2 % | 9.1 to 10.1 | 6.7 | 1.18 | right class, short of the limit level on a day with a low fan curve |
| M2 | 3 mm, one side, 22.2 % | 14.1 to 15.1 | 13.4 | 1.13 | right class, 8 % low: this loss is the band alone |
| M18 | 3 mm, two sides, 17.9 % | 8.1 to 10.1 | 11.0 | 0.91 | right class, 20 % high |
| M5 | 3 mm, two sides, 22.2 %, no restrictor | 5.0 to 8.1 | 3.6 | 0.90 | floor, on the steep curve |
| M17, M8, the rest | | | | 0.3 together | floor |

- **The band.** A forecast centred on a slab that dies at 14 cm/s still costs about 1.1 with an
  ln-sd of 0.35 (M2). The six high-class slabs scatter by 0.25 to 0.3 in ln around the typed
  level, so the band cannot honestly be narrowed. About 6 of the 11.8 is this: the flow the
  operators happened to hold.
- **One-sided slabs** are forecast 8 to 36 % low (three burns, two days). A steeper rise with
  thickness for one-sided slabs would fit them; three burns do not carry a new exponent.
- **M11** is the one case in the wrong class.

### Q1: the 5 mm threshold at the middle of its bracket (logged, `runs/dev/gen7/study/bass3_q1.wsp`, coding v2)

The cases bracket the starting oxygen below which a 5 mm two-sided slab reaches its limit between
20.0 % (M11, did) and 21.8 % (M4, did not). bass3 typed 20.6 %; the middle is 21.0 %. Ten elements
(the thickness slope of the threshold, 0.147 to 0.168, in the median, cap and spread units; 2 mm
stays at 18.0 %).

| | Expected (in the log before the run) | Realised |
|---|---|---|
| M11 | median 4.2 to 7.5 cm/s, loss 4.28 to about 2.2 | slabs 0.845 to 0.693 by case (11.83 to 9.70 over 14) |
| every other slab | unchanged | one more day differs, also lower |
| pooled, day mean | 0.152 to about 0.132 | 0.152 to 0.135 |
| case mean | 0.249 to about 0.213 | 0.249 to 0.211 |
| display scale | not stated | 0.584 to 0.536 |
| days | one differs, lower | 2 of 2 lower |
| kept burning | identical | identical |

Predicted in sign and size, except that I expected one day to change and two did. **A comparator
by the rule** (two days change). It moves one case; I would not call it more than that.

## E. Proposal: `bass4` = bass3 plus a "kept burning" answer for fabrics

New slug (two new inputs). Not run: the tool has no such slug yet. The quench targets and every
PMMA forecast are those of bass3, line for line.

**The law added.** For SIBAL, Nomex and Ultem the flat pair answers with a material floor:

```text
P(kept burning) = the smaller of flat_holds_low_flow and flat_holds_high_flow
low flow:   floor 17.0 % (SIBAL), 23.4 % (Nomex, Ultem); thermal limit of a film below 2 cm/s
high flow:  floor 15.8 % (SIBAL), 21.5 % (Nomex, Ultem), times the blow-off rise of a film
```

The two floors of a material are the additive law of section A read in the form of the pair:
17.0 % is its minimum (`0.152 + 2 sqrt(0.070 * 0.0012)`); 15.8 % is the value which, multiplied by
the blow-off rise of the pair at 10 cm/s, gives the limit of A there (17.1 %). For Nomex and Ultem
the same with `x_f = 0.215`. Below 2 cm/s the pair is more lenient to SIBAL than A (17.0 % against
18.9 % at 2 cm/s); `phys_quench_w0.md` is the reference there.

**Expected, before any run** (hand evaluation of the lines below on the 19 cases): SIBAL 9 burns,
P 0.95 to 0.99, Brier about 0.0005; Nomex and Ultem 10 burns, P 0.03 to 0.31, Brier about 0.10
(the one burn that held costs 0.74). All 19: about 0.054, against 0.38 for the incidence and 0.47
for "always lit" on the same burns. PMMA: identical to bass3.

### For the engineer

Eleven features: the nine of `bass3` in the same order, two appended.

| # | `name` | SI unit | transform | centre | scale | domain | Derivation from the condition fields |
|---|---|---|---|---|---|---|---|
| 0 to 8 | as `bass3` | | | | | | unchanged for PMMA |
| 9 | `is_sibal` | 1 | identity | 0 | 1 | 0 to 1 | 1 if `material_class` is `fabric_sibal`, else 0 |
| 10 | `is_nomex_ultem` | 1 | identity | 0 | 1 | 0 to 1 | 1 if `material_class` is `fabric_nomex` (any Nomex) or `ultem_sheet`, else 0 |

Rows for fabrics and Ultem (kept burning only; `quench_u` stays refused for them):
`x_o2` initial oxygen; `flow_speed` the speed at ignition; `nose_radius` 0.01; `heat_time` 200;
`is_concurrent` 1 concurrent, 0 opposed, **0.5 where the records state no direction**; `is_rod` 0;
`flat_thickness` 0.00032 (SIBAL), 0.0003 (Nomex), 0.00025 (Ultem); `u_pot1` as for any case;
`is_one_sided` 0. PMMA flat samples with no stated direction stay refused, as in bass3.

### Edits

All lines of the `bass3` release unchanged. New lines (columns 9 and 10 are zero wherever not
written):

```text
# bass4: bass3 plus fabrics in the kept-burning pair. SIBAL: zero-shot (phys_bass_study2.md A).
# Nomex and Ultem: data-informed (one floor after ten development burns).
x.is_sibal.centre = 0.0
x.is_sibal.scale = 1.0
x.is_sibal.lo = 0.0
x.is_sibal.hi = 1.0
x.is_nomex_ultem.centre = 0.0
x.is_nomex_ultem.scale = 1.0
x.is_nomex_ultem.lo = 0.0
x.is_nomex_ultem.hi = 1.0
# low flow: the floor moves from the 13.8 % of a film to 17.0 % (SIBAL: -20 ln(0.170 / 0.138)) and to
# 23.4 % (Nomex, Ultem: -20 ln(0.234 / 0.138)); the same shift inside the hinge keeps min(l_f, l_th).
flat_holds_low_flow.logit.skip[9] = -4.17
flat_holds_low_flow.logit.skip[10] = -10.56
flat_holds_low_flow.logit.W1[0, 9] = -4.17
flat_holds_low_flow.logit.W1[0, 10] = -10.56
# high flow: floors 15.8 % and 21.5 % under the blow-off rise of a film.
flat_holds_high_flow.logit.skip[9] = -2.71
flat_holds_high_flow.logit.skip[10] = -8.87
```

**Comparators, not part of the model unless the coordinator decides so** (both run on bass3,
sections C and D):

```text
# S1: one-sided flat samples keep a flame only above about 21 % oxygen (1 of 3 days lower).
flat_holds_low_flow.logit.W1[1, 8] = 8.40
flat_holds_low_flow.logit.w2[1] = -1.0
# Q1: threshold of a 5 mm two-sided slab at 21.0 % instead of 20.6 % (2 of 2 days lower).
flat_quench_speed.median.W1[3, 6] = 5.05
flat_quench_speed.median.b1[3] = -7.00
flat_quench_speed.median.W1[4, 6] = 4.75
flat_quench_speed.median.b1[4] = -8.382
flat_quench_speed.spread.W1[3, 6] = 5.05
flat_quench_speed.spread.b1[3] = -7.00
flat_quench_speed.spread.W1[4, 6] = 5.05
flat_quench_speed.spread.b1[4] = -7.93
flat_quench_speed.spread.W1[5, 6] = 5.05
flat_quench_speed.spread.b1[5] = -8.87
```

## F. What remains, and the count

- Logged candidates on `bass3` in this round: two (S1, Q1), both run on the second coding through
  `runs/dev/gen7/study/whatif_v2.py`, which sets the lane's own switch `law_score.BASS_CODING = "v2"`
  and calls the tool unchanged; the notes say "CODING V2". One earlier on `bass2` (K1).
- The limit of SIBAL is untested in this lane's files: every development burn is in cabin air at
  5 cm/s or more. If the test days hold SIBAL at 16 to 18 % or below 2 cm/s, that is where A will
  be judged.
- Nomex: five burns of Nomex III, one held. Whether that one differs by flow (14.6 cm/s; the
  others 8 to 10, 15 at lower oxygen, and 30) cannot be said from five burns; the law gives all
  of them 0.03 to 0.31.
- The quench speed of a slab that reaches its limit is the flow the operators were holding; the
  band of 0.35 in ln is about half of what is left.
- M5 and any other burn without a restrictor: one scored case on a curve 2.5 times steeper.
