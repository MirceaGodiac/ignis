# Model card — `flex_methanol.wsp2` and `flex_heptane.wsp2` (the droplet forward, third generation)

**State: experimental, hand-written, data-informed. Not NASA-trained, not validated, not a posterior.**
Every parameter is a number the author model (claude-fable-5-2) typed; nothing was fitted. The test
partition has not been read.

## Releases

| Model | Candidate | Release | How written |
|---|---|---|---|
| `flex_methanol.wsp2` | **D3** (carried) | `sha256:c8a90dabbed6af4185de94c3cd5ac27486c19eb52c697d86b028849a37fcb654` | data-informed: train runs, train feedback of D1/D2, a train-only audit of D2 |
| | D²-law baseline | `sha256:abf919b6199864c0a0eb6d27beffb295421c9d080f664d7c9a5929d6f254247b` | reference and lineage root, no data |
| `flex_heptane.wsp2` | **D2** (carried) | `sha256:edbad4d6395961030e0b17d9d5c4d40f2de902ddcbcbd0ec2a4f4ec9adbeedef` | data-informed: train runs, train feedback of W0/D1, a train-only audit of D1 |
| | D²-law baseline | `sha256:8e40db9dceb898e336f69422d3e2cff2df52b6050188ef84af1a9d0a7ba4ebb1` | reference and lineage root, no data |

Forward hash `sha256:` of `droplet3.rs` plus the shared files is in each `release.json`. Authored
elements: 378 (methanol, 68 non-zero in D3), 575 (heptane, 93 non-zero in D2).

## What changed in the computation

2. **The run offset of a cool flame is read where the cool flame is.** It is the end-rate innovation
   against the cool-flame law, scaled by a written share (`enc.cool_gain`).

## What the numbers say

Full reasons are in the edits files; the audited facts are in
[EXPERIMENTAL_SCIENCE.md](../EXPERIMENTAL_SCIENCE.md), section 5.1.

**Methanol D3** (against D2):

| Law | D2 | D3 | Why |
|---|---|---|---|
| Radiative clock `ln s_R` | 0.90 at D_crit, −1.2 per unit of ln(D0/D_crit), a cliff beyond 0.45 | 1.0 at D_crit, −3.8 per unit from 0.05, no second term; −0.22 per 20 % He | 13 runs in 9 sessions lie on one line; four same-atmosphere pairs; three helium runs |
| Evaporation after extinction | 0.06 mm²/s, decay 4.5 s·r | 0.13 mm²/s·r^0.45, decay 2.8 s·r^0.2 | 20 runs, 15 sessions, two size classes, four time windows |
| Heat-up | one deficit, all atmospheres | deeper and longer below 16 % O2; plateau 5 % lower at ≤ 15 % | 4 of 5 low-oxygen sessions |
| Run-offset gain at 1 / 3 / 5 s | 0.12 / 0.92 / 0.98 | 0.08 / 0.36 / 0.55 | a fifth to a half of a prefix's offset persists |
| Rate spread before / after a prefix | 0.18 / 0.08 | 0.10 / 0.06 | train run-level sd 0.06, session medians off by 0.05 in 8 of 23 sessions; margin for unseen sessions |
| Unchanged | rate level and its oxygen, CO2, helium, pressure, size and water-uptake terms; critical diameter; class; diffusive extinction | | audit: median rate residual −0.003 over 263 windows; no run on the wrong side of D_crit; diffusive progress 0.81 against 0.79 |

**Heptane D2** (against D1):

| Law | D1 | D2 | Why |
|---|---|---|---|
| Early rate | a fast heat-up deficit on top of the flame-development deficit | no fast deficit; the first 0.3 clock units 0.19–0.15 below the plateau | measured rate 16 % above D1 at 0.3–1 s and 7 % at 1–2 s in 13 and 12 of 16 sessions |
| Radiative clock | −2.0 per unit of ln(D0/D_crit) | the same, and −1.0 per e-fold of D0 above 2.3 mm | six same-atmosphere pairs in five sessions; residual ordered by size, not by supercriticality |
| Helium in the clock | none | none, on purpose | the helium runs lose the visible flame early, but what follows is not an unlit droplet and has no law here; correcting the clock alone made those forecasts worse on train |
| Evaporation after the cool flame | 0.06 mm²/s, decay 4 s·r | 0.12 mm²/s·r^0.3, decay 3.7 s·r^0.2 | 13 runs, 8 sessions |
| Hot run-offset gain at 1 / 3 / 5 s | 0.12 / 0.92 / 0.98 | 0.09 / 0.23 / 0.31 | on train the law alone predicts the next seconds better than the law with D1's offset, at every cutoff |
| Cool-flame run offset | all of the mid-to-tail innovation | half of the end-rate innovation | the rate D1 used missed the cool rate by 21 %; the law alone by 6 % |
| Rate spread before / after a prefix | 0.20 / 0.07 | 0.10 / 0.06 | D1's 50 % band held 0.92 of the burning train cases at 1 s |
| Unchanged | critical diameter and class; diffusive extinction; cool-flame onset, rate and plateau diameter | | audit: no run on the wrong side; cool rate within 6 %, plateau within 10 % |

## Results (development)

Selection was made on train by the rule in [PROTOCOL.md](../PROTOCOL.md), section 8, and committed
 before any validation score of these candidates existed. CRPS of the diameter in µm.

| | Train median / mean | Train sessions better or level | Validation | Validation coverage 50 % / 90 % |
|---|---|---|---|---|
| Methanol D3 | 17.7 / 29.9 | 24 of 26 against D2 | **40.0** | 0.71 / 0.90 |
| Best fitted model (`flex.v3`) | | | 77.7 | 0.34 / 0.75 |
| D²-law (this spec) | | | 102.1 | 0.53 / 0.72 |
| Heptane D2 | 21.8 / 46.2 | 17 of 20 against D1 | **46.1** | 0.71 / 0.97 |
| Heptane W0 (zero-shot on heptane) | 33.0 / 58.3 | | 60.3 | 0.69 / 0.97 |
| D²-law (this spec) | | | 105.1 | 0.61 / 0.79 |

Validation is 56 methanol cases in 9 sessions and 39 heptane cases in 7 sessions; paired intervals
are in [RESULTS.md](../RESULTS.md) and are wide. The validation numbers were read once, as an
aggregate table, and changed nothing.

## Limits

- **Tracker artefacts are scored as data.** About a third (methanol) and two fifths (heptane) of
  the train error is glare, soot, bubbles, dropout frames, droplets leaving the view and frozen
  values after a droplet is gone. No model forecasts those.
- **A flame that went out less than a second before the cutoff still reads as burning**; the end
  windows straddle it (H15M102 at a 5 s cutoff).
- **Helium.** The methanol clock has a helium term from three runs. For heptane the stage after the
  visible flame in 30–35 % helium has no law (one train session), and the clock is deliberately not
  corrected.
- **Radiative clock above D_crit rests on CO2-diluted pairs** for methanol; N2 runs exist only near
  the threshold. If the critical diameter in CO2 is off, the slope is still right within an
  atmosphere and wrong between atmospheres.
- **Spread is assumed.** It was set from train residuals with a margin, not calibrated; the heptane
  90 % band is still too wide on validation (0.97).
- 2 atm heptane (one problem run), 14 % O2 cool flames (one session) and burn order within a fill
  are outside what was written.
