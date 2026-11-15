# 6. The models and where their numbers come from

**Astra addition:** `droplet7.rs`/`thermal.rs`, `condition_committee.rs`, and `front3.rs`
add five new model identities and ten **data-informed experimental** candidates. None
replaces a carried release. Cards, counts, origin limits and train scores are indexed in
[ASTRA_RESULTS.md](ASTRA_RESULTS.md). The historical catalogue below remains Fable's record.

One section per carried model: what the forward computes, where the typed numbers are, which kind
of source each group of numbers has, and which numbers the author trusts least. The cards
([../cards/](../cards/)) give every law with its value; the edits files give the reason beside each
statement. [registers/candidates.md](registers/candidates.md) lists all 58 candidates with their
release ids and declared exposure.

Kinds of origin used below:

- **[L]** a published constant or law, typed as published ([01_sources.md](01_sources.md), 1.4)
- **[T]** read off a train table by the author; the table is named in the edits file
- **[J]** a judgment: no table gives it (a margin, a weight, a width where train is silent)
- **[I]** a fact about an instrument or a procedure (frame rate, script delay, resolution)
- **[V]** informed by an aggregate validation table (only where stated)

## 6.1 Forwards and model names

| Forward (`crates/ignis-wsp/src/`) | Models | Generation | Status |
|---|---|---|---|
| `droplet3.rs` | `flex_methanol.wsp2`, `flex_heptane.wsp2` | third | **carried** |
| `droplet4.rs` | `flex_methanol.wsp3`, `flex_heptane.wsp3` | fourth | refused by validation |
| `flame.rs` | `acme_normal.wsp2`, `acme_inverse.wsp2` | phase 2, third | member of the carried normal committee |
| `flame3.rs` | `acme_normal.wsp3`, `acme_inverse.wsp3` | fourth, fifth | members of both carried committees; carried by itself for inverse until the fifth generation |
| `blend.rs` | `acme_normal.wsp4`, `acme_inverse.wsp4` | fourth | carried for normal until the fifth generation; refused for inverse |
| `front.rs` | `bass_front.wsp` | fourth | **carried** |
| `droplet5.rs` | `flex_methanol.wsp4`, `flex_heptane.wsp4` | fifth | its candidates R1 and F1 were never registered; superseded by `droplet6.rs` |
| `droplet6.rs` | `flex_methanol.wsp5`, `flex_heptane.wsp5` | fifth | refused by validation ([card](../cards/flex_droplet6.md)) |
| `committee.rs` | `acme_normal.wsp5`, `acme_inverse.wsp5` | fifth | **carried** for both ([card](../cards/acme_committee.md)) |
| `front2.rs` | `bass_front.wsp2` | fifth | refused by validation ([card](../cards/bass_front.md)) |

Every forward is deterministic: features from the prefix, typed weights, explicit integration. No
gradient, sampler, root finder or optimiser exists on the route (`tests/strict.rs` checks the
dependency graph). Uncertainty is of the kind `assumed_noise`: the bands are typed, not inferred. No
release claims a Bayesian posterior or NASA training.

## 6.2 `flex_methanol.wsp2` D3 (carried)

`configs/models/flex/wsp2/methanol/d3-fable/edits.wsp`: 59 statements over 378 elements, 68
non-zero. Card: [flex_droplet3.md](../cards/flex_droplet3.md).

The forecast is a mixture over the droplet's state at the cutoff and after it. A rate law gives the
hot burning rate from the atmosphere and the size; a class law says on which side of the critical
diameter the droplet is; a radiative clock and a burn-progress law say when the hot flame ends;
an evaporation law carries the unlit droplet; an encoder reads the run's own rate offset and its
state from the prefix.

| Group | Origin | Remark |
|---|---|---|
| Critical diameter by atmosphere | [T] brackets between train pairs | a bracket, not a point; for CO2 and helium few pairs |
| Radiative clock: 1.0 at the critical diameter, −3.8 per unit of ln(D0/D_crit) from 0.05, faster with helium | [T] nine runs on one line | M-07 says the clock runs 15 to 28 % late for radiative losses |
| Evaporation after the flame: 0.13 mm²/s · r^0.45, decay 2.8 s · r^0.2 | [T] 20 runs in 15 sessions | |
| Heat-up, deeper below 16 % oxygen | [T] four of five low-oxygen sessions | |
| Encoder: trust in the run's rate offset 0.08 / 0.36 / 0.55 at 1 / 3 / 5 s; state read from the end rate | [T] how much of a prefix offset persists on train | |
| Rate spread 0.10 before a prefix, 0.06 after | [T] train residuals + **[J] margin** | the margin is the least defensible number of the model; narrowing it failed (C-15) |
| Validity gate: 5 % growth, 2 mm²/s | [J], [I] | twice the fastest measured rate |

**Trusted least:** the spread margin; the helium factor of the clock (three runs); everything at
pressures other than 1 atm (few runs; one 2 atm CO2 run is plainly wrong in the heptane model).

## 6.3 `flex_heptane.wsp2` D2 (carried)

`configs/models/flex/wsp2/heptane/d2-fable/edits.wsp`: 78 statements over 575 elements, 93 non-zero.
Same card. Adds the cool-flame state: onset after a radiative loss, a cool burning rate, an end
diameter.

| Group | Origin | Remark |
|---|---|---|
| Critical diameter (3.7 mm in air, falling with CO2) | [T] | the zero-shot W0 had 2.8 mm from the literature |
| Radiative clock: −2.0 per unit of ln(D0/D_crit), −1.0 more per e-fold of D0 above 2.3 mm; no helium term, on purpose | [T] six same-atmosphere pairs in five sessions | M-07: the clock is 1.5 times late in helium at 0.7 atm, and far too early for the 2 atm CO2 run (0.37 of the measured time; at 3 and 5 s the model gives probability 1.0 to a flame that the UV camera shows until 8 s) |
| Cool flame: always after a radiative loss in N2 and CO2 at 15 to 21 % oxygen, none in 30 to 35 % helium; rate 0.30 to 0.40 mm²/s; end diameter about D0^0.8 | [T]; the end diameter also [L] one published curve | |
| Evaporation after the cool flame: 0.12 mm²/s · r^0.3, decay 3.7 s · r^0.2 | [T] 13 runs, 8 sessions | under-predicts the shrinking after a flame lost in helium (three runs, one session) |
| Encoder gains 0.09 / 0.23 / 0.31; half of the end-rate innovation for the cool rate | [T] | |
| Spread 0.10 / 0.06 | [T] + [J] | |

**Trusted least:** helium beyond 35 % (validation has 40 to 45 %); pressure above 1 atm; the end of
the cool flame, which published theory calls flat near its maximum.

## 6.4 `acme_normal.wsp4` D1 (carried until the fifth generation): two laws with equal weights

`configs/models/acme/wsp4/normal/d1-fable/edits.wsp`: 92 statements, of which two were typed for
this model (`mix.w = [1, 1]`); the others are the stored values of its two members copied with a
prefix by `tools/wsp/blend_edits.py`. Card: [acme_blend.md](../cards/acme_blend.md).

| Member | Edits | What it is | Origin of its numbers |
|---|---|---|---|
| `acme_normal.wsp3` D1 | `wsp3/normal/d1-fable` (43 statements) | Growth clock from the published theory; burner heating law; two limits ([acme_growth.md](../cards/acme_growth.md)) | [L] `ρD`, critical flux, diffusivity; [T] four corrections the theory lacks (pressure as p^−0.45, air twice as fast as 39 % oxygen, butane in enriched oxygen, dilute burner gas); [T] heating law from peak burner temperatures |
| Weights 1 : 1 | | | [J], typed knowing the members' validation scores (they tie) |

**Trusted least:** the four corrections of the growth clock, which have no mechanism behind them;
propane in enriched oxygen (no train test; validation is exactly that); the stop atoms.

## 6.5 `acme_inverse.wsp3` D1 (carried until the fifth generation; now the first member of the carried committee)

`configs/models/acme/wsp3/inverse/d1-fable/edits.wsp`: 39 statements over 499 elements, 54
non-zero. Card: [acme_growth.md](../cards/acme_growth.md).

| Group | Origin | Remark |
|---|---|---|
| Early-quench clock set mostly by the ambient fuel fraction | [T] seven sessions | empirical; the published growth law is switched off for inverse flames because with its published constants it gives lean flames minutes |
| Weak class at ignition (air injected into a lean ambient) | [T] three of six such flames | |
| Burner heating law and two limits | [T], shared with the normal model | inverse sessions at 0.55 and 2.2 bar heat with 50 to 60 s, not 30 s (M-08) |
| Stop atoms (66 s and others) | [T] | the 66 s stop was used by four of five sessions; the whole-campaign cluster had also been read (X-03) |

**Trusted least:** everything about the validation regime (ambient fuel at 5.7 %, 3.3 atm, injected
oxygen 0.40 to 0.70: outside train in three coordinates at once); the missing dependence on total
burner flow (C-24).

## 6.6 `bass_front.wsp` D1 (carried)

`configs/models/bass/front/d1-fable/edits.wsp`: 32 statements over 159 elements, 35 non-zero. Card:
[bass_front.md](../cards/bass_front.md).

| Group | Origin | Remark |
|---|---|---|
| Spread rate `0.0535 + 0.032 u`, oxygen power 1.5, width power −1 | [T] three burns, four flows, two oxygen levels; the exponent is within the 1 to 2 the published models give [L]; the width rule [J] | typed 4 % below the first version after it ran fast on train prefixes |
| Overhang `0.065 U`, oxygen power 3, relaxation 2 s | [T] three fan steps; the power from one burn | |
| Burn-through time 6.3 s (`burn.b0 = 1.84` is its logarithm) | [T] 6.6, 6.6, 6.0 s | one number for all conditions |
| Quench: tissue-paper boundary [L] moved up by 3 % oxygen [T, two burns]; a unit below 17.2 % oxygen | see X-01 | **failed on validation** |
| Encoder gains by prefix length and time since a fan step | [T] offsets at 4, 14 and 35 s on three burns | |
| Spread 25 % before a prefix, 6 % after | [J] | three burns of one day agree within 5 % |

**Trusted least:** the quench law (known wrong); the oxygen exponents; the narrow sample; flows
above 11 cm/s; anything at 21 % oxygen.

## 6.7 What all five have in common, as weaknesses

1. Their bands are typed. Coverage on validation is the only check, and validation is worn.
2. Each was typed from 3 to 27 train sessions, and several laws rest on one to three runs.
3. Corrections were added where train disagreed with a simple law, without a mechanism. The owner's
   instruction of 14 November is to replace such patches by physics or drop them; the fifth
   generation is that work.

## 6.8 The two committees carried since the fifth generation

Card with members, weights, scores and what to distrust: [acme_committee.md](../cards/acme_committee.md).

| Model | Release | What was typed for it | Origin |
|---|---|---|---|
| `acme_normal.wsp5` P1 | `sha256:1336c6307fc9023f2434994f49e56cada9ff844000b3ebda29e23bc6bb9a77f4` | weights 1 : 1 of the hinge law and the growth clock; the same pair without guessed stops when the planned stop is known; offset 0.4 s. In the growth member one statement differs from the model of 6.4: the weights of the two burner limits, 0.15 and 0.85 | weights [J]; offset [T] from 29 train tests; limit weights [T], the share of normal-flame train sessions under each setting |
| `acme_inverse.wsp5` P1 | `sha256:aa05ddcbe976dbb1a5537987d05270b28066551c39ab24db671be65692dcea0e` | weights 2 : 1 of the law of 6.5 and of the same law with a hold scenario (four statements: a flame in the marginal band of the mixture fraction at a high oxygen flow is held); the same pair without guessed stops when the planned stop is known; offset 0.4 s | weights [J]: one third is the author's probability; the hold scenario [T] from two train flames, with the band from M-17 |

Everything else in the two edits files is the stored value of a member release, copied by
`tools/wsp/committee_edits.py`. **Trusted least:** the admissibility of the planned stop (C-21); the
hold scenario (C-44); and, inherited from the members, what 6.4 and 6.5 say.
