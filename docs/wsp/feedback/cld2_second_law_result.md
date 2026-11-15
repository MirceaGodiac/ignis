# cld2: second law (PROTOCOL 13.4), development, in-sample

- Dossier: `docs/wsp/research/phys_cld2.md`, sha256 `970688997e13dd17c20e2ce3b00089c9b28745fb243276cd0b3f0b66cc1a751b`.
- Release: `sha256:870419c33f9b4e39588ccabe884959fdde910e6337da39c7eee3d2be3f698721` (slug `cld2`, candidate `s1-fable`); parent: cld D1 `sha256:a03ec603f0926cf522df5d5a9f09d73f077ba22e6cabd5e8b29970794ddf2d4f`.
- Edits: the parent block (W0 block with the three D1 lines) and the dossier's new lines, verbatim. Form only: the D1 line `loss.blowoff.median.skip[3] = 1.21` is written inside the W0 list line (`skip[0:6] = [0.55, -1.0, 1.0, 1.21, 0.0, 0.0]`), because the compiler refuses an element written twice; same number.
- Neutral check (forecasts only): with `dilutes` = `velocity_ramp` = 0, cld2 reproduces the parent's loss and radiometer forecasts on every parent case: train 0 of 218 differ, validation inputs 0 of 72 differ.

All numbers below are development, in-sample (train and validation days together; validation is study data since 13.4). Lower is better.

| target | cases (both answer) | parent (cld D1) | cld2 | best baseline |
|---|---|---|---|---|
| loss, integrated Brier, pooled | 256 | 0.3174 | 0.1428 | 0.2282 (incidence by dilution ramp) |
| loss, train | 198 | 0.1253 | 0.0702 | 0.0705 |
| loss, validation | 61 | 0.5139 | 0.2420 | 0.3282 |
| radiometer, CRPS log10 V, pooled | 195 | 0.1186 | 0.1186 | (unchanged target) |

Loss by day (integrated Brier):

| day | parent | cld2 | |
|---|---|---|---|
| 17319 | 0.0025 | 0.0046 | higher |
| 17324 | 0.3136 | 0.0853 | lower |
| 17342 | 0.0648 | 0.0648 | equal |
| 18003 | 0.5276 | 0.1837 | lower |
| 18005 | 0.1013 | 0.1013 | equal |
| 18008 | 0.7226 | 0.4576 | lower |
| 18011 | 0.1906 | 0.0515 | lower |
| 18022 | 0.5481 | 0.5481 | equal |
| 18026 | 0.4007 | 0.4007 | equal |
| 18032 | 0.0606 | 0.0606 | equal |
| 18043 | 0.0014 | 0.0067 | higher |
| 18053 | 0.0004 | 0.0004 | equal |
| 18145 | 0.3943 | 0.3943 | equal |
| 18177 | 0.2598 | 0.2598 | equal |
| 18192 | 0.0001 | 0.0001 | equal |
| 18214 | 0.0001 | 0.0001 | equal |
| 18218 | 0.0002 | 0.0002 | equal |
| 18235 | 0.0001 | 0.0001 | equal |
| 18239 | 0.0002 | 0.0002 | equal |
| 18241 | 0.2178 | 0.0952 | lower |
| 18249 | 0.0771 | 0.1068 | higher |
| 18253 | 0.0004 | 0.0004 | equal |
| 18257 | 0.0004 | 0.0004 | equal |
| 21046 | 0.0009 | 0.0009 | equal |
| 21062 | 0.0016 | 0.0016 | equal |

Days on which cld2 is lower: **5 of 25** (8 days change at all: the flags are 1 on 8 days; 3 of those are higher).

**Reading (13.4):** pooled, cld2 is lower than its parent (0.143 against 0.317) and lower than every baseline; the radiometer is unchanged. It is lower on 5 of 25 days, short of two thirds (on the 8 days where the flags change anything, 5 of 8), so by the rule as written it is **not kept** for the final run; the coordinator decides whether days with identical forecasts count.
