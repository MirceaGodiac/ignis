# bass2: second law (PROTOCOL 13.4), development, in-sample

- Dossier: `docs/wsp/research/phys_bass2.md`, sha256 `f2bdb28c05bbe83f23fe12820d0c068e02e0e17cf407eccdb885318c51574b17`.
- Release: `sha256:e008cba3f5101f218c1e7f353fed928a8e11a22500ebf1197dc9f050c57ca4c9` (slug `bass2`, `s1-fable`); parent: bass1 D1 `sha256:b3bb1869df2fc837f7957e925f55b357ecc6272d312f1ddbb46346da15376ca0`. Edits: the W0 block (one changed line in place), the D1 block and the new lines, verbatim.
- Inputs (`tools/wsp/ext/law_inputs.py`, `bass2_row`): the bass1 features with every flat sample at nose_radius 0.01; `is_rod` 1 for rods in any stated configuration; `flat_thickness` the thickness of films and slabs, 0.0005 for rods, spheres and sections. Rods with no stated configuration and flat samples with no stated direction stay refused.
- Neutral check (forecasts only): on the samples with the nose to the flow, with is_rod = 0 and flat_thickness at its centre, bass2 reproduces bass1 D1 (train 0 of 49 differ, validation 0 of 18). Flat samples differ by design (the changed film line and the thickness hinges replacing the D1 slab flag).

Development, in-sample. Interval CRPS cm/s (quench_u) and Brier (sustain), mean of day means, on the cases both answer; lower is better.

| target | stratum | cases | parent (bass1 D1) | bass2 | constant / train incidence | Bhattacharjee 2016 / always lit |
|---|---|---|---|---|---|---|
| quench | all, pooled | 48 | 0.8135 | 0.4505 | 0.7079 | 0.8423 |
| quench | all, train | 39 | 0.8987 | 0.5065 | 0.8406 | 1.0263 |
| quench | all, validation | 9 | 0.5067 | 0.2489 | 0.2299 | 0.1797 |
| quench | rods, pooled | 19 | 0.9741 | 0.2637 | 0.3034 | 0.2856 |
| quench | slabs, pooled | 10 | 2.3707 | 1.7805 | 3.0052 | 4.0225 |
| quench | films, pooled | 5 | 0.1627 | 0.0612 | 0.1477 | 0.0440 |
| quench | spheres, pooled | 14 | 0.1183 | 0.1183 | 0.1737 | 0.1082 |
| quench | all cases bass2 answers, pooled | 52 | - | 0.3926 | | |
| sustain | all, pooled | 82 | 0.0405 | 0.0405 | 0.0669 | 0.0700 |
| sustain | all, train | 60 | 0.0147 | 0.0147 | 0.0368 | 0.0312 |
| sustain | all, validation | 22 | 0.0862 | 0.0862 | 0.1203 | 0.1389 |
| sustain | rods, pooled | 18 | 0.0904 | 0.0904 | 0.1850 | 0.2222 |
| sustain | slabs, pooled | 18 | 0.0146 | 0.0146 | 0.0125 | 0.0000 |
| sustain | films, pooled | 13 | 0.0277 | 0.0277 | 0.0557 | 0.0556 |
| sustain | spheres, pooled | 33 | 0.0274 | 0.0274 | 0.0302 | 0.0227 |
| sustain | all cases bass2 answers, pooled | 101 | - | 0.0233 | | |

quench_u by day (mean interval CRPS):

| day | parent | bass2 |
|---|---|---|
| 2012-04-09 | 0.0000 | 0.0000 |
| 2012-06-11 | 0.0185 | 0.0185 |
| 2012-06-14 | 0.0000 | 0.0000 |
| 2013-04-09 | 0.2895 | 0.2895 |
| 2013-04-16 | 0.0000 | 0.0000 |
| 2013-04-23 | 0.2793 | 0.2793 |
| 2013-04-29 | 0.1218 | 0.1218 |
| 2013-05-17 | 0.2375 | 0.2375 |
| 2014-02-19 | 0.0000 | 0.0000 |
| 2014-02-20 | 1.8134 | 1.3218 |
| 2014-06-16 | 0.9672 | 0.2000 |
| 2014-06-23 | 1.4638 | 0.3423 |
| 2014-06-26 | 1.0962 | 0.1966 |
| 2014-07-02 | 0.2556 | 0.1227 |
| 2014-07-03 | 1.4130 | 0.2461 |
| 2014-07-08 | 1.1496 | 0.2681 |
| 2014-07-11 | 4.5531 | 3.4228 |
| 2014-07-28 | 0.4680 | 0.5179 |
| 2014-07-29 | 0.6248 | 0.2885 |
| 2014-07-30 | 0.6101 | 0.0499 |
| 2014-07-31 | 0.2447 | 0.6304 |
| 2014-08-01 | 2.8718 | 1.7472 |
| 2014-08-04 | 0.2324 | 0.0609 |

Days lower (quench_u): **12 of 14** days whose forecasts differ (23 days in all); sustain: identical forecasts on the shared cases (0 days differ).

**Reading (13.4):** quench_u pooled 0.4505 against the parent 0.8135 (best baseline: constant 0.7079), lower on 12 of 14 days; sustain unchanged on the shared cases: **kept**. bass2 also answers the opposed rods for sustain (101 cases in all, 0.0233).
