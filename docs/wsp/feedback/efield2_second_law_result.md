# efield2: second law (PROTOCOL 13.4), development, in-sample

- Dossier: `docs/wsp/research/phys_efield2.md`, sha256 `4b1be3810bccafe6a11e96f4a5c8283910e439b5951180034db2d82147c69fb8`.
- Release: `sha256:cf5459b54b0a880713cbfcec0e22bae473b1bc265e064bfc9ab0295f193bd8c5` (slug `efield2`, candidate `s1-fable`); parent: efield W0 `sha256:73f91b55c81dc724fe2441c2254e109c65f8e08b253fcbe4c00a841f7a0986a9`.
- Edits: the W0 block with the three changed `b0` lines in place, then the new lines, verbatim.
- Neutral check (forecasts only): with `has_coflow` = 0 the ion-current forecasts differ from the parent on every level (train 2735 of 2760, validation 1289 of 1289), as the dossier states: `ion_current.median.b0` was changed on purpose (-12.939 to -13.039, 0.10 lower in still gas), as were `peak_ion_current.median.b0` and `flame_end.other.median.b0`.

Development, in-sample (train and validation together). Lower is better.

| target | parent (W0) | efield2 | best baseline |
|---|---|---|---|
| T1 ion current CRPS uA, pooled (3235 levels) | 0.6592 | 0.7206 | 0.4256 (space-charge law) |
| T2 flame out, integrated Brier, pooled | 0.1478 | 0.1001 | 0.0697 (stratified incidence) |
| by partition: train | T1 0.7202, T2 0.1761 | T1 0.7857, T2 0.1015 | |
| by partition: validation | T1 0.5463, T2 0.0753 | T1 0.6002, T2 0.0975 | |

By day:

| day | T1 parent | T1 efield2 | T2 parent | T2 efield2 |
|---|---|---|---|---|
| 18073 | 0.6537 | 0.8134 | 0.0012 | 0.0290 |
| 18078 | 0.3519 | 0.3604 | 0.2129 | 0.1127 |
| 18080 | 0.1490 | 0.1521 | 0.0078 | 0.0590 |
| 18092 | 1.1302 | 1.0904 | 0.0032 | 0.0032 |
| 18095 | 0.4016 | 0.3590 | 0.0038 | 0.0038 |
| 18102 | 0.1500 | 0.1829 | 0.1494 | 0.0685 |
| 18106 | 0.2080 | 0.2176 | 0.2503 | 0.1252 |
| 18113 | 0.8922 | 1.1239 | 0.3999 | 0.2277 |
| 18130 | 0.5264 | 0.5701 | 0.2941 | 0.1432 |
| 18137 | 1.9050 | 2.4087 | 0.0010 | 0.0010 |
| 18282 | 0.4073 | 0.3542 | 0.0911 | 0.0707 |
| 18285 | 0.9231 | 0.7998 | 0.0019 | 0.1141 |
| 18298 | 0.1993 | 0.1800 | 0.1931 | 0.1687 |
| 18312 | 0.9369 | 0.8360 | 0.3390 | 0.1632 |
| 18319 | 0.6521 | 0.6517 | 0.0019 | 0.0878 |

Days lower: T1 7 of 15; T2 8 of 15.

**Reading (13.4):** on the ion current (the target the failure analysis ranks first for efield) efield2 is higher than its parent pooled (0.7206 against 0.6592): **not kept**. The flame-out target improves (0.1001 against 0.1478) but stays above the stratified incidence.
