# SAME-R (PSI-101): one revision of the W0 law from the train days (D1)

Author: claude-fable-5-2, 15 November 2026. **Data-informed, not zero-shot**: typed after reading
`docs/wsp/feedback/samer_train_anatomy.md` (train days only) and, for one cross-table, the train per-case
file `runs/dev/data_all/work/transfer_samer/law_study_05f77ae2f7a666e0.jsonl` (error by material and by
primary or aged row; the sample temperatures of the train rows, which are inputs). No validation outcome
and no test day was opened. Parent: the W0 of [phys_samer_w0.md](phys_samer_w0.md), release
`sha256:05f77ae2…7fb0`. One step, damped, no fit and no search; nothing was run after typing.

## 1. What the train days say

The forecast diameter is too small by 0.27 in ln (median), but the fault is not common to the materials:

| Material | Primary rows: median error (n) | Aged rows: median error (n) | Sample temperature on train, K | `T_ref` typed in W0, K |
|---|---|---|---|---|
| Lamp wick | -0.01 (5) | -0.04 (2) | 570 | 560 |
| Kapton | -0.05 (4) | -0.40 (3; -0.55, -0.40, +0.42) | 847 | 830 |
| Teflon | -0.27 (6) | +0.48 (2) | 801 | 790 |
| Silicone | -0.23 (11) | -0.66 (6) | 653 | 670 |
| Pyrell | -0.50 (5) | -0.80 (3) | 515 | 570 |

Two faults carry it (silicone and Pyrell hold 62 % of the loss):

1. **Pyrell was heated at 515 K, 55 K below the reference temperature I typed.** The law therefore read
   every Pyrell test as a cold, weak one: 0.22 smaller in ln, a coagulation time 1.7 times longer, and a
   detector logit 0.66 lower. The reference temperature of a material is the one at which it gives a
   measurable smoke without igniting; for this foam I put it too high (I took the decomposition of a
   polyurethane, and an intumescent fire-retarded foam smokes earlier).
2. **Silicone smoke grows with aging much faster than forecast** (aged rows 0.43 further off than primary
   rows). In the law that is the coagulation time: `t_c = 1/(K N_0)` was 1400 s, the longest of the five,
   from an assumed 15 mg/m3 of smoke. The detector is at full scale on 17 of 17 silicone rows, which says
   the same thing independently: the silicone smoke is denser than I assumed, so more numerous at a given
   size, so it coagulates faster. W0 section 8.3 named this as the expected direction.

The 90 % band holds 43 of 47 rows (42 expected): the width is right. The 50 % band holds 19 (23.5
expected) because of the offset, not of the width. The spread is not touched.

## 2. The step

| # | Number | Old | New | Physical reason | Expected effect on train |
|---|---|---|---|---|---|
| 1 | Pyrell reference temperature, as it enters the size: `d10.median.skip[3]` | 0.104 | 0.324 | `T_ref` 570 K -> 515 K, the temperature at which the foam was run (an input): +0.20 * 55/50 | Pyrell `d10` +0.22 in ln; covers under half of the -0.50 of its primary rows |
| 2 | The same in the coagulation time: `d10.median.W1[3:7, 3]` | -0.282 | 0.268 | `t_c` of Pyrell is 1200 s at its own set point, not 2080 s: +0.5 * 55/50 | Pyrell aged rows a further +0.09 (0.31 of their -0.80 in all) |
| 3 | Silicone coagulation time: `d10.median.W1[3:7, 2]` | -1.436 | -0.536 | `t_c` 1400 s -> 570 s: a smoke about 2.5 times denser than assumed | silicone rows aged 600 to 1900 s +0.19; rows under 100 s unchanged |
| 4 | Silicone condensation size: `d10.median.skip[2]` | -0.225 | -0.125 | `D_ref` 310 -> 343 nm: a denser vapour condenses into larger drops (W0 1.5: `D_n ~ J^0.15 to 0.3`; 2.5 times the vapour gives +0.14 to +0.27, I take 0.10) | all silicone rows +0.10; with step 3 the aged rows gain 0.29 of their -0.66, the short primary rows 0.10 of -0.23 |
| 5 | Silicone detector logit: `iss_full_scale.logit.skip[2]` | -1.57 | -0.77 | the same denser smoke: 2.5 times the mass is +0.9 in ln, 0.8 in logit at the W0 scale | mean forecast for silicone 0.64 -> about 0.80 (observed 17 of 17) |
| 6 | Pyrell detector logit: `iss_full_scale.logit.skip[3]` | -0.47 | 0.19 | the `T_ref` correction of step 1: +0.6 * 55/50 | mean forecast for Pyrell 0.58 -> about 0.72 (observed 8 of 8) |

Steps 1, 2 and 6 are one physical number (the Pyrell reference temperature); steps 3, 4 and 5 are one
physical statement (the silicone smoke is about 2.5 times denser than assumed). The same two changes are
typed into the unscored `d30` network so that the two diameters stay one law.

How the size of the step was set. Pyrell: the temperature correction is what the physics gives, and it
happens to be 44 % of the train offset of the primary rows; nothing was added to reach the rest. Silicone:
closing the aging gap would need `t_c` divided by about 5; I divide by 2.5 (+0.9 in `ln Co`), which gives
half of the missing growth in ln. The size step of 0.10 is under half of the primary offset and inside the
range the condensation scaling allows. The anatomy's sensitivities agree in sign and order: +1 % on
`d10.median.skip[2]` improves the train score by 0.0004, on `iss_full_scale.logit.skip[2]` by 0.0009.

Expected on train (estimated from the table above, not computed): median error of `d10` from -0.27 to
about -0.12; CRPS of ln d10 from 0.272 to about 0.23; Brier of the detector from 0.150 to about 0.12.

## 3. What was left alone, and why

- **No common offset** (`d10.median.b0`). It has the largest sensitivity, but lamp wick sits at -0.01 and
  Teflon's primary and aged rows disagree in sign; a common shift would be paid for by the two materials
  the law has right.
- **Coagulation times of Kapton, Teflon and lamp wick.** Kapton's aged rows (-0.55, -0.40, +0.42) and
  Teflon's (+0.41, +0.56) are two or three rows each and point opposite ways; lamp wick is on the law.
- **Teflon at the detector** (forecast 0.26, observed 0 of 8) and lamp wick (0.76, 7 of 7): real, smaller
  shares of the loss, and each would be a third physical statement. Next batch.
- **The spread.** Right at 90 %; the protocol sets widths from blind errors of earlier batches.
- Pyrell still under-forecast after the step (about -0.28 primary, -0.5 aged): its `D_ref` (270 nm) is
  probably too small as well, but the temperature error had to be removed first to see by how much.

## Edits

Changed lines only; a revision of the W0 release.

```text
# samer D1: data-informed revision of W0 (docs/wsp/research/phys_samer_d1.md). Author claude-fable-5-2.
# Pyrell reference temperature 570 K -> 515 K (the foam was run there): size, coagulation time, detector
d10.median.skip[3] = 0.324
d10.median.W1[3:7, 3] = 0.268
d30.median.skip[3] = 0.257
d30.median.W1[3:7, 3] = 0.268
iss_full_scale.logit.skip[3] = 0.19
# silicone smoke about 2.5 times denser than assumed: t_c 1400 s -> 570 s, D_ref 310 -> 343 nm, detector
d10.median.W1[3:7, 2] = -0.536
d10.median.skip[2] = -0.125
d30.median.W1[3:7, 2] = -0.536
d30.median.skip[2] = -0.125
iss_full_scale.logit.skip[2] = -0.77
```
