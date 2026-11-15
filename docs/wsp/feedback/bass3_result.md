# Plastics: bass2 K1 and bass3 (PROTOCOL 13.6), development, in-sample

- Dossier: `docs/wsp/research/phys_bass_study.md` (sha256 `08b20a41160c880a43c98fcabef63539d96bdc3dfcd6012b2d19eadcab894b21`), sections 3 and 4.
- K1: `sha256:6298ab84749e6d65ae21e6e21b04710444541d4d11e0359390adc430c81fcae1` (`configs/models/law/bass2/k1-fable`, revision of bass2 s1 `sha256:e008cba3f5101f218c1e7f353fed928a8e11a22500ebf1197dc9f050c57ca4c9`): the fifteen lines of `runs/dev/gen7/study/bass_k1.wsp` (sha256 `2f794051…`) verbatim.
- bass3: `sha256:a1eb577ae257a788109329a8cc54509925ecab4c1e3e88ddcf9decf0fd7703d1` (new slug, `configs/models/law/bass3/s1-fable`): the declaration of bass2 with `u_pot1` and `is_one_sided` appended (`tools/wsp/ext/law_inputs.py`, `bass3_row`, dossier lines quoted); edits: every line of the bass2 s1 block with the 58 elements of section 4.4 written in place (21 of them inside the parent's own lines, by `tools/wsp/ext/law_merge_edits.py`), checked element by element: bass3 equals bass2 s1 except exactly the block's elements, which carry the block's numbers.
- Check the dossier names ("no line of the four holds targets changes"): the kept-burning forecasts of bass3 equal bass2's on every case (differing: train 0 of 78, validation 0 of 28).
- Scores: Rust forward and the lane scorer, on the cases all three answer; interval CRPS cm/s (quench_u) and Brier (sustain), mean of day means (case mean in brackets); lower is better.

| target | stratum | partition | cases | bass2 s1 | K1 | bass3 | bass3 expected | constant / train incidence | Bhattacharjee / always lit |
|---|---|---|---|---|---|---|---|---|---|
| quench | all | pooled | 52 | 0.3926 (0.5254) | 0.3138 (0.4550) | 0.1310 (0.2020) | about 0.15 | 0.5786 (0.8194) | 0.6552 (0.9607) |
| quench | all | train | 43 | 0.4304 (0.5842) | 0.3401 (0.5038) | 0.1611 (0.2413) |  | 0.6704 (0.9405) | 0.7803 (1.1226) |
| quench | all | validation | 9 | 0.2489 (0.2445) | 0.2139 (0.2222) | 0.0169 (0.0146) |  | 0.2299 (0.2409) | 0.1797 (0.1869) |
| quench | rods | pooled | 23 | 0.2354 (0.2155) | 0.1230 (0.1071) | 0.0296 (0.0312) | about 0.05 | 0.2779 (0.2649) | 0.2512 (0.2295) |
| quench | rods | train | 20 | 0.2275 (0.2067) | 0.1052 (0.0898) | 0.0298 (0.0310) |  | 0.2559 (0.2406) | 0.2234 (0.1964) |
| quench | rods | validation | 3 | 0.2673 (0.2744) | 0.1942 (0.2226) | 0.0288 (0.0320) |  | 0.3660 (0.4270) | 0.3621 (0.4498) |
| quench | spheres | pooled | 14 | 0.1183 (0.0909) | 0.0314 (0.0240) | 0.0140 (0.0101) | 0.01 to 0.02 | 0.1737 (0.1653) | 0.1082 (0.0845) |
| quench | spheres | train | 11 | 0.1326 (0.1107) | 0.0344 (0.0277) | 0.0157 (0.0121) |  | 0.1756 (0.1665) | 0.1186 (0.0979) |
| quench | spheres | validation | 3 | 0.0185 (0.0185) | 0.0103 (0.0103) | 0.0026 (0.0026) |  | 0.1607 (0.1607) | 0.0355 (0.0355) |
| quench | films | pooled | 5 | 0.0612 (0.0858) | 0.0381 (0.0521) | 0.0145 (0.0180) | about 0.03 | 0.1477 (0.1902) | 0.0440 (0.0583) |
| quench | films | train | 4 | 0.0613 (0.0920) | 0.0366 (0.0549) | 0.0116 (0.0175) |  | 0.1672 (0.2106) | 0.0400 (0.0599) |
| quench | films | validation | 1 | 0.0609 (0.0609) | 0.0412 (0.0412) | 0.0203 (0.0203) |  | 0.1087 (0.1087) | 0.0519 (0.0519) |
| quench | slabs | pooled | 10 | 1.7805 (2.0665) | 1.7766 (2.0601) | 0.9349 (0.9558) | about 1.0 | 3.0052 (3.3253) | 4.0225 (4.3203) |
| quench | slabs | train | 8 | 2.1639 (2.4255) | 2.1588 (2.4177) | 1.2453 (1.1938) |  | 3.9576 (4.1196) | 5.3343 (5.3786) |
| quench | slabs | validation | 2 | 0.6304 (0.6304) | 0.6299 (0.6299) | 0.0037 (0.0037) |  | 0.1481 (0.1481) | 0.0870 (0.0870) |
| sustain | all | pooled | 101 | 0.0233 (0.0254) | 0.0233 (0.0254) | 0.0233 (0.0254) |  | 0.0371 (0.0433) | 0.0316 (0.0396) |
| sustain | all | train | 74 | 0.0114 (0.0144) | 0.0114 (0.0144) | 0.0114 (0.0144) |  | 0.0262 (0.0335) | 0.0175 (0.0270) |
| sustain | all | validation | 27 | 0.0459 (0.0555) | 0.0459 (0.0555) | 0.0459 (0.0555) |  | 0.0578 (0.0700) | 0.0583 (0.0741) |
| sustain | rods | pooled | 37 | 0.0206 (0.0187) | 0.0206 (0.0187) | 0.0206 (0.0187) |  | 0.0513 (0.0545) | 0.0500 (0.0541) |
| sustain | rods | train | 31 | 0.0059 (0.0070) | 0.0059 (0.0070) | 0.0059 (0.0070) |  | 0.0287 (0.0376) | 0.0208 (0.0323) |
| sustain | rods | validation | 6 | 0.0794 (0.0794) | 0.0794 (0.0794) | 0.0794 (0.0794) |  | 0.1419 (0.1419) | 0.1667 (0.1667) |
| sustain | spheres | pooled | 33 | 0.0274 (0.0311) | 0.0274 (0.0311) | 0.0274 (0.0311) |  | 0.0302 (0.0361) | 0.0227 (0.0303) |
| sustain | spheres | train | 22 | 0.0006 (0.0005) | 0.0006 (0.0005) | 0.0006 (0.0005) |  | 0.0125 (0.0125) | 0.0000 (0.0000) |
| sustain | spheres | validation | 11 | 0.0743 (0.0923) | 0.0743 (0.0923) | 0.0743 (0.0923) |  | 0.0610 (0.0831) | 0.0625 (0.0909) |
| sustain | films | pooled | 13 | 0.0277 (0.0384) | 0.0277 (0.0384) | 0.0277 (0.0384) |  | 0.0557 (0.0722) | 0.0556 (0.0769) |
| sustain | films | train | 11 | 0.0416 (0.0453) | 0.0416 (0.0453) | 0.0416 (0.0453) |  | 0.0772 (0.0831) | 0.0833 (0.0909) |
| sustain | films | validation | 2 | 0.0000 (0.0000) | 0.0000 (0.0000) | 0.0000 (0.0000) |  | 0.0125 (0.0125) | 0.0000 (0.0000) |
| sustain | slabs | pooled | 18 | 0.0146 (0.0190) | 0.0146 (0.0190) | 0.0146 (0.0190) |  | 0.0125 (0.0125) | 0.0000 (0.0000) |
| sustain | slabs | train | 10 | 0.0282 (0.0335) | 0.0282 (0.0335) | 0.0282 (0.0335) |  | 0.0125 (0.0125) | 0.0000 (0.0000) |
| sustain | slabs | validation | 8 | 0.0010 (0.0009) | 0.0010 (0.0009) | 0.0010 (0.0009) |  | 0.0125 (0.0125) | 0.0000 (0.0000) |

quench_u by day (mean of the day):

| day | bass2 s1 | K1 | bass3 |
|---|---|---|---|
| 2012-04-09 | 0.0000 | 0.0000 | 0.0000 |
| 2012-06-11 | 0.0185 | 0.0103 | 0.0026 |
| 2012-06-14 | 0.0000 | 0.0034 | 0.0009 |
| 2013-04-09 | 0.2895 | 0.0412 | 0.0174 |
| 2013-04-16 | 0.0000 | 0.0164 | 0.0047 |
| 2013-04-23 | 0.2793 | 0.0889 | 0.0372 |
| 2013-04-29 | 0.1218 | 0.0413 | 0.0357 |
| 2013-05-17 | 0.2375 | 0.0495 | 0.0140 |
| 2014-02-19 | 0.0000 | 0.0000 | 0.0000 |
| 2014-02-20 | 1.3218 | 1.3217 | 0.6912 |
| 2014-06-16 | 0.2000 | 0.0848 | 0.0286 |
| 2014-06-23 | 0.3423 | 0.1290 | 0.0419 |
| 2014-06-26 | 0.1966 | 0.0750 | 0.0315 |
| 2014-07-02 | 0.1227 | 0.0732 | 0.0233 |
| 2014-07-03 | 0.2461 | 0.1088 | 0.0193 |
| 2014-07-08 | 0.2681 | 0.0771 | 0.0492 |
| 2014-07-11 | 3.4228 | 3.4072 | 1.4773 |
| 2014-07-22 | 0.1836 | 0.0653 | 0.0292 |
| 2014-07-28 | 0.5179 | 0.3914 | 0.0232 |
| 2014-07-29 | 0.2885 | 0.2795 | 0.0384 |
| 2014-07-30 | 0.0499 | 0.0013 | 0.0292 |
| 2014-07-31 | 0.6304 | 0.6299 | 0.0037 |
| 2014-08-01 | 0.6233 | 0.5945 | 0.5263 |
| 2014-08-04 | 0.0609 | 0.0412 | 0.0203 |

Days lower (quench_u): K1 against bass2 s1 **20 of 22**; bass3 against bass2 s1 **20 of 22**; bass3 against K1 21 of 22 (days whose forecasts differ).

**Reading (13.6).** K1: pooled 0.3138 against bass2 0.3926, lower on 20 of 22 days, kept burning identical: kept by the rule (it reproduces the what-if run, 0.393 to 0.314, through the Rust forward). bass3: pooled 0.1310 against bass2 0.3926 and K1 0.3138, lower than bass2 on 20 of 22 days and than K1 on 21 of 22; kept burning identical: **bass3 kept** (it is also lower than both baselines in every stratum and partition). Against the expectation: rods 0.030 (expected about 0.05), spheres 0.014 (0.01 to 0.02), films 0.015 (about 0.03), slabs 0.935 (about 1.0), pooled 0.131 (about 0.15): right in sign in every stratum, the gains a little larger than written.

## Both codings (version 1 as registered, version 2), development, in-sample

- Coding v1: `tools/wsp/ext/bass1_cases.py`, files `cases_*.jsonl` (the registered looks). Coding v2: `tools/wsp/ext/bass1_cases_v2.py`, files `cases_*_v2.jsonl`, rules R1 to R10 of `docs/wsp/feedback/bass1_coding_v2.md` (an open upper end costs nothing above the lower end). The inputs are the same in both codings; only outcomes change, so every release gives the same forecasts and only the scores move.
- Scorer: `tools/wsp/ext/law_score.py`, `bass1_slug`, with the module switch `BASS_CODING = "v1" | "v2"` (default v1, so every registered command reproduces). Producer: `runs/dev/gen7/bass_codings_dev.json` (all four releases) and `runs/dev/gen7/bass_codings_dev_three.json` (bass2 s1, K1, bass3).
- Baselines on v2: the lane's v2 constants (`transfer_bass1/baselines_v2.json`: quench quantiles 0.66, 1.22, 1.46, 1.80, 14.08 cm/s, display quantiles 40.40, 40.55, 40.80, 41.20, 74.35, sustain incidence 0.888); Bhattacharjee et al. 2016 keeps its formula and its v1 train log width (no v2 width exists and I did not fit one); always lit is unchanged.
- **Display scale (v2 only), how.** A release forecasts quench_u quantiles in cm/s. Each case carries its fan curve (display to cm/s, an input built from the calibration day, never from the case's outcome). I map every quantile through the inverse of that curve (`bass1_cases.u_to_fan`: the curve's points with flat stretches dropped so that it is strictly increasing, linear between points, linear extrapolation beyond the last point, clamped at the first point). The map is increasing, so quantile levels are kept and the mapped quantiles are quantiles of the forecast on the display scale; they are scored against `quench_display_lo/hi` with the same interval CRPS. No outcome is read by the map. The constant baseline uses its own display quantiles (fitted on the display scale by the lane, not mapped). Bhattacharjee is mapped like a release. Clamping and extrapolation would be the only non-clean parts; I counted them: on these cases no forecast quantile of any of the four releases lies below the first or above the last point of its case's curve (0 of 500 to 530 quantiles), so the map is the curve itself throughout.
- Case sets: the first block is the set of the table above (cases bass2 s1, K1 and bass3 all answer); bass1 D1 is scored there on the cases it answers within that set (its counts are smaller, not shown per cell). The second block puts all four on the cases all four answer.

**Coding v1 (as registered), the three releases on the cases all three answer** (bass1 D1 is not in this set; v1 reproduces the table above to the last digit). Interval CRPS cm/s (quench), on the fan display (quench_display, v2 only), Brier (sustain); mean of day means (case mean).

| target | stratum | partition | cases | bass2 s1 | bass2 K1 | bass3 | baseline 1 | baseline 2 |
|---|---|---|---|---|---|---|---|---|
| quench | all | pooled | 52 | 0.3926 (0.5254) | 0.3138 (0.4550) | 0.1310 (0.2020) | constant (simple): 0.5786 (0.8194) | Bhattacharjee et al. 2016, train log width: 0.6552 (0.9607) |
| quench | all | train | 43 | 0.4304 (0.5842) | 0.3401 (0.5038) | 0.1611 (0.2413) | constant (simple): 0.6704 (0.9405) | Bhattacharjee et al. 2016, train log width: 0.7803 (1.1226) |
| quench | all | validation | 9 | 0.2489 (0.2445) | 0.2139 (0.2222) | 0.0169 (0.0146) | constant (simple): 0.2299 (0.2409) | Bhattacharjee et al. 2016, train log width: 0.1797 (0.1869) |
| quench | rods | pooled | 23 | 0.2354 (0.2155) | 0.1230 (0.1071) | 0.0296 (0.0312) | constant (simple): 0.2779 (0.2649) | Bhattacharjee et al. 2016, train log width: 0.2512 (0.2295) |
| quench | rods | train | 20 | 0.2275 (0.2067) | 0.1052 (0.0898) | 0.0298 (0.0310) | constant (simple): 0.2559 (0.2406) | Bhattacharjee et al. 2016, train log width: 0.2234 (0.1964) |
| quench | rods | validation | 3 | 0.2673 (0.2744) | 0.1942 (0.2226) | 0.0288 (0.0320) | constant (simple): 0.3660 (0.4270) | Bhattacharjee et al. 2016, train log width: 0.3621 (0.4498) |
| quench | spheres | pooled | 14 | 0.1183 (0.0909) | 0.0314 (0.0240) | 0.0140 (0.0101) | constant (simple): 0.1737 (0.1653) | Bhattacharjee et al. 2016, train log width: 0.1082 (0.0845) |
| quench | spheres | train | 11 | 0.1326 (0.1107) | 0.0344 (0.0277) | 0.0157 (0.0121) | constant (simple): 0.1756 (0.1665) | Bhattacharjee et al. 2016, train log width: 0.1186 (0.0979) |
| quench | spheres | validation | 3 | 0.0185 (0.0185) | 0.0103 (0.0103) | 0.0026 (0.0026) | constant (simple): 0.1607 (0.1607) | Bhattacharjee et al. 2016, train log width: 0.0355 (0.0355) |
| quench | films | pooled | 5 | 0.0612 (0.0858) | 0.0381 (0.0521) | 0.0145 (0.0180) | constant (simple): 0.1477 (0.1902) | Bhattacharjee et al. 2016, train log width: 0.0440 (0.0583) |
| quench | films | train | 4 | 0.0613 (0.0920) | 0.0366 (0.0549) | 0.0116 (0.0175) | constant (simple): 0.1672 (0.2106) | Bhattacharjee et al. 2016, train log width: 0.0400 (0.0599) |
| quench | films | validation | 1 | 0.0609 (0.0609) | 0.0412 (0.0412) | 0.0203 (0.0203) | constant (simple): 0.1087 (0.1087) | Bhattacharjee et al. 2016, train log width: 0.0519 (0.0519) |
| quench | slabs | pooled | 10 | 1.7805 (2.0665) | 1.7766 (2.0601) | 0.9349 (0.9558) | constant (simple): 3.0052 (3.3253) | Bhattacharjee et al. 2016, train log width: 4.0225 (4.3203) |
| quench | slabs | train | 8 | 2.1639 (2.4255) | 2.1588 (2.4177) | 1.2453 (1.1938) | constant (simple): 3.9576 (4.1196) | Bhattacharjee et al. 2016, train log width: 5.3343 (5.3786) |
| quench | slabs | validation | 2 | 0.6304 (0.6304) | 0.6299 (0.6299) | 0.0037 (0.0037) | constant (simple): 0.1481 (0.1481) | Bhattacharjee et al. 2016, train log width: 0.0870 (0.0870) |
| sustain | all | pooled | 101 | 0.0233 (0.0254) | 0.0233 (0.0254) | 0.0233 (0.0254) | train incidence (simple): 0.0371 (0.0433) | always lit: 0.0316 (0.0396) |
| sustain | all | train | 74 | 0.0114 (0.0144) | 0.0114 (0.0144) | 0.0114 (0.0144) | train incidence (simple): 0.0262 (0.0335) | always lit: 0.0175 (0.0270) |
| sustain | all | validation | 27 | 0.0459 (0.0555) | 0.0459 (0.0555) | 0.0459 (0.0555) | train incidence (simple): 0.0578 (0.0700) | always lit: 0.0583 (0.0741) |
| sustain | rods | pooled | 37 | 0.0206 (0.0187) | 0.0206 (0.0187) | 0.0206 (0.0187) | train incidence (simple): 0.0513 (0.0545) | always lit: 0.0500 (0.0541) |
| sustain | rods | train | 31 | 0.0059 (0.0070) | 0.0059 (0.0070) | 0.0059 (0.0070) | train incidence (simple): 0.0287 (0.0376) | always lit: 0.0208 (0.0323) |
| sustain | rods | validation | 6 | 0.0794 (0.0794) | 0.0794 (0.0794) | 0.0794 (0.0794) | train incidence (simple): 0.1419 (0.1419) | always lit: 0.1667 (0.1667) |
| sustain | spheres | pooled | 33 | 0.0274 (0.0311) | 0.0274 (0.0311) | 0.0274 (0.0311) | train incidence (simple): 0.0302 (0.0361) | always lit: 0.0227 (0.0303) |
| sustain | spheres | train | 22 | 0.0006 (0.0005) | 0.0006 (0.0005) | 0.0006 (0.0005) | train incidence (simple): 0.0125 (0.0125) | always lit: 0.0000 (0.0000) |
| sustain | spheres | validation | 11 | 0.0743 (0.0923) | 0.0743 (0.0923) | 0.0743 (0.0923) | train incidence (simple): 0.0610 (0.0831) | always lit: 0.0625 (0.0909) |
| sustain | films | pooled | 13 | 0.0277 (0.0384) | 0.0277 (0.0384) | 0.0277 (0.0384) | train incidence (simple): 0.0557 (0.0722) | always lit: 0.0556 (0.0769) |
| sustain | films | train | 11 | 0.0416 (0.0453) | 0.0416 (0.0453) | 0.0416 (0.0453) | train incidence (simple): 0.0772 (0.0831) | always lit: 0.0833 (0.0909) |
| sustain | films | validation | 2 | 0.0000 (0.0000) | 0.0000 (0.0000) | 0.0000 (0.0000) | train incidence (simple): 0.0125 (0.0125) | always lit: 0.0000 (0.0000) |
| sustain | slabs | pooled | 18 | 0.0146 (0.0190) | 0.0146 (0.0190) | 0.0146 (0.0190) | train incidence (simple): 0.0125 (0.0125) | always lit: 0.0000 (0.0000) |
| sustain | slabs | train | 10 | 0.0282 (0.0335) | 0.0282 (0.0335) | 0.0282 (0.0335) | train incidence (simple): 0.0125 (0.0125) | always lit: 0.0000 (0.0000) |
| sustain | slabs | validation | 8 | 0.0010 (0.0009) | 0.0010 (0.0009) | 0.0010 (0.0009) | train incidence (simple): 0.0125 (0.0125) | always lit: 0.0000 (0.0000) |

quench, v1, by day (mean of the day):

| day | bass2 s1 | bass2 K1 | bass3 |
|---|---|---|---|
| 2012-04-09 | 0.0000 | 0.0000 | 0.0000 |
| 2012-06-11 | 0.0185 | 0.0103 | 0.0026 |
| 2012-06-14 | 0.0000 | 0.0034 | 0.0009 |
| 2013-04-09 | 0.2895 | 0.0412 | 0.0174 |
| 2013-04-16 | 0.0000 | 0.0164 | 0.0047 |
| 2013-04-23 | 0.2793 | 0.0889 | 0.0372 |
| 2013-04-29 | 0.1218 | 0.0413 | 0.0357 |
| 2013-05-17 | 0.2375 | 0.0495 | 0.0140 |
| 2014-02-19 | 0.0000 | 0.0000 | 0.0000 |
| 2014-02-20 | 1.3218 | 1.3217 | 0.6912 |
| 2014-06-16 | 0.2000 | 0.0848 | 0.0286 |
| 2014-06-23 | 0.3423 | 0.1290 | 0.0419 |
| 2014-06-26 | 0.1966 | 0.0750 | 0.0315 |
| 2014-07-02 | 0.1227 | 0.0732 | 0.0233 |
| 2014-07-03 | 0.2461 | 0.1088 | 0.0193 |
| 2014-07-08 | 0.2681 | 0.0771 | 0.0492 |
| 2014-07-11 | 3.4228 | 3.4072 | 1.4773 |
| 2014-07-22 | 0.1836 | 0.0653 | 0.0292 |
| 2014-07-28 | 0.5179 | 0.3914 | 0.0232 |
| 2014-07-29 | 0.2885 | 0.2795 | 0.0384 |
| 2014-07-30 | 0.0499 | 0.0013 | 0.0292 |
| 2014-07-31 | 0.6304 | 0.6299 | 0.0037 |
| 2014-08-01 | 0.6233 | 0.5945 | 0.5263 |
| 2014-08-04 | 0.0609 | 0.0412 | 0.0203 |

bass2 K1 against bass2 s1: lower on 20 of 22 days whose scores differ; bass3 against bass2 s1: lower on 20 of 22 days whose scores differ; bass3 against bass2 K1: lower on 21 of 22 days whose scores differ.

**Coding v2, the three releases on the cases all three answer** (bass1 D1 is not in this set; v1 reproduces the table above to the last digit). Interval CRPS cm/s (quench), on the fan display (quench_display, v2 only), Brier (sustain); mean of day means (case mean).

| target | stratum | partition | cases | bass2 s1 | bass2 K1 | bass3 | baseline 1 | baseline 2 |
|---|---|---|---|---|---|---|---|---|
| quench | all | pooled | 57 | 0.5052 (0.7455) | 0.4290 (0.6803) | 0.1522 (0.2487) | constant (simple): 0.7890 (1.1365) | Bhattacharjee et al. 2016, v1 train log width: 0.8758 (1.4050) |
| quench | all | train | 45 | 0.5122 (0.8100) | 0.4220 (0.7331) | 0.1661 (0.2868) | constant (simple): 0.8334 (1.2660) | Bhattacharjee et al. 2016, v1 train log width: 0.9512 (1.6110) |
| quench | all | validation | 12 | 0.4830 (0.5035) | 0.4510 (0.4824) | 0.1082 (0.1057) | constant (simple): 0.6482 (0.6508) | Bhattacharjee et al. 2016, v1 train log width: 0.6368 (0.6324) |
| quench | rods | pooled | 23 | 0.2354 (0.2155) | 0.1230 (0.1071) | 0.0296 (0.0312) | constant (simple): 0.3909 (0.3777) | Bhattacharjee et al. 2016, v1 train log width: 0.2512 (0.2295) |
| quench | rods | train | 20 | 0.2275 (0.2067) | 0.1052 (0.0898) | 0.0298 (0.0310) | constant (simple): 0.3690 (0.3528) | Bhattacharjee et al. 2016, v1 train log width: 0.2234 (0.1964) |
| quench | rods | validation | 3 | 0.2673 (0.2744) | 0.1942 (0.2226) | 0.0288 (0.0320) | constant (simple): 0.4786 (0.5435) | Bhattacharjee et al. 2016, v1 train log width: 0.3621 (0.4498) |
| quench | spheres | pooled | 14 | 0.1183 (0.0909) | 0.0314 (0.0240) | 0.0140 (0.0101) | constant (simple): 0.2851 (0.2762) | Bhattacharjee et al. 2016, v1 train log width: 0.1082 (0.0845) |
| quench | spheres | train | 11 | 0.1326 (0.1107) | 0.0344 (0.0277) | 0.0157 (0.0121) | constant (simple): 0.2865 (0.2764) | Bhattacharjee et al. 2016, v1 train log width: 0.1186 (0.0979) |
| quench | spheres | validation | 3 | 0.0185 (0.0185) | 0.0103 (0.0103) | 0.0026 (0.0026) | constant (simple): 0.2751 (0.2751) | Bhattacharjee et al. 2016, v1 train log width: 0.0355 (0.0355) |
| quench | films | pooled | 5 | 0.0612 (0.0858) | 0.0381 (0.0521) | 0.0145 (0.0180) | constant (simple): 0.2368 (0.2710) | Bhattacharjee et al. 2016, v1 train log width: 0.0440 (0.0583) |
| quench | films | train | 4 | 0.0613 (0.0920) | 0.0366 (0.0549) | 0.0116 (0.0175) | constant (simple): 0.2516 (0.2870) | Bhattacharjee et al. 2016, v1 train log width: 0.0400 (0.0599) |
| quench | films | validation | 1 | 0.0609 (0.0609) | 0.0412 (0.0412) | 0.0203 (0.0203) | constant (simple): 0.2071 (0.2071) | Bhattacharjee et al. 2016, v1 train log width: 0.0519 (0.0519) |
| quench | slabs | pooled | 15 | 2.0659 (2.3889) | 2.0599 (2.3812) | 0.8799 (0.8818) | constant (simple): 3.1248 (3.3914) | Bhattacharjee et al. 2016, v1 train log width: 4.4518 (4.8887) |
| quench | slabs | train | 10 | 2.6819 (3.0731) | 2.6778 (3.0669) | 1.2770 (1.2082) | constant (simple): 4.3915 (4.5725) | Bhattacharjee et al. 2016, v1 train log width: 6.4166 (6.7250) |
| quench | slabs | validation | 5 | 1.1420 (1.0205) | 1.1330 (1.0097) | 0.2842 (0.2289) | constant (simple): 1.2248 (1.0292) | Bhattacharjee et al. 2016, v1 train log width: 1.5047 (1.2162) |
| sustain | all | pooled | 100 | 0.0467 (0.0328) | 0.0467 (0.0328) | 0.0467 (0.0328) | train incidence (simple): 0.0571 (0.0513) | always lit: 0.0575 (0.0500) |
| sustain | all | train | 74 | 0.0070 (0.0099) | 0.0070 (0.0099) | 0.0070 (0.0099) | train incidence (simple): 0.0262 (0.0335) | always lit: 0.0175 (0.0270) |
| sustain | all | validation | 26 | 0.1221 (0.0981) | 0.1221 (0.0981) | 0.1221 (0.0981) | train incidence (simple): 0.1160 (0.1021) | always lit: 0.1333 (0.1154) |
| sustain | rods | pooled | 37 | 0.0209 (0.0190) | 0.0209 (0.0190) | 0.0209 (0.0190) | train incidence (simple): 0.0513 (0.0545) | always lit: 0.0500 (0.0541) |
| sustain | rods | train | 31 | 0.0059 (0.0070) | 0.0059 (0.0070) | 0.0059 (0.0070) | train incidence (simple): 0.0287 (0.0376) | always lit: 0.0208 (0.0323) |
| sustain | rods | validation | 6 | 0.0812 (0.0812) | 0.0812 (0.0812) | 0.0812 (0.0812) | train incidence (simple): 0.1419 (0.1419) | always lit: 0.1667 (0.1667) |
| sustain | spheres | pooled | 32 | 0.0060 (0.0026) | 0.0060 (0.0026) | 0.0060 (0.0026) | train incidence (simple): 0.0125 (0.0125) | always lit: 0.0000 (0.0000) |
| sustain | spheres | train | 22 | 0.0006 (0.0005) | 0.0006 (0.0005) | 0.0006 (0.0005) | train incidence (simple): 0.0125 (0.0125) | always lit: 0.0000 (0.0000) |
| sustain | spheres | validation | 10 | 0.0155 (0.0070) | 0.0155 (0.0070) | 0.0155 (0.0070) | train incidence (simple): 0.0125 (0.0125) | always lit: 0.0000 (0.0000) |
| sustain | films | pooled | 13 | 0.0277 (0.0384) | 0.0277 (0.0384) | 0.0277 (0.0384) | train incidence (simple): 0.0557 (0.0722) | always lit: 0.0556 (0.0769) |
| sustain | films | train | 11 | 0.0416 (0.0453) | 0.0416 (0.0453) | 0.0416 (0.0453) | train incidence (simple): 0.0772 (0.0831) | always lit: 0.0833 (0.0909) |
| sustain | films | validation | 2 | 0.0000 (0.0000) | 0.0000 (0.0000) | 0.0000 (0.0000) | train incidence (simple): 0.0125 (0.0125) | always lit: 0.0000 (0.0000) |
| sustain | slabs | pooled | 18 | 0.1663 (0.1110) | 0.1663 (0.1110) | 0.1663 (0.1110) | train incidence (simple): 0.1419 (0.0988) | always lit: 0.1667 (0.1111) |
| sustain | slabs | train | 10 | 0.0006 (0.0004) | 0.0006 (0.0004) | 0.0006 (0.0004) | train incidence (simple): 0.0125 (0.0125) | always lit: 0.0000 (0.0000) |
| sustain | slabs | validation | 8 | 0.3321 (0.2492) | 0.3321 (0.2492) | 0.3321 (0.2492) | train incidence (simple): 0.2712 (0.2065) | always lit: 0.3333 (0.2500) |
| quench_display | all | pooled | 57 | 1.4677 (2.2551) | 1.3009 (2.1046) | 0.5844 (0.9942) | constant (simple): 1.9102 (2.8494) | Bhattacharjee et al. 2016, v1 train log width: 2.3103 (3.7035) |
| quench_display | all | train | 45 | 1.5463 (2.5136) | 1.3462 (2.3345) | 0.6812 (1.1866) | constant (simple): 2.0618 (3.2240) | Bhattacharjee et al. 2016, v1 train log width: 2.4885 (4.2277) |
| quench_display | all | validation | 12 | 1.2187 (1.2856) | 1.1573 (1.2422) | 0.2778 (0.2730) | constant (simple): 1.4301 (1.4446) | Bhattacharjee et al. 2016, v1 train log width: 1.7462 (1.7376) |
| quench_display | rods | pooled | 23 | 0.6514 (0.6065) | 0.3900 (0.3425) | 0.0789 (0.0860) | constant (simple): 0.7150 (0.7070) | Bhattacharjee et al. 2016, v1 train log width: 0.8218 (0.7433) |
| quench_display | rods | train | 20 | 0.6100 (0.5548) | 0.3171 (0.2667) | 0.0743 (0.0810) | constant (simple): 0.7175 (0.7064) | Bhattacharjee et al. 2016, v1 train log width: 0.6765 (0.5824) |
| quench_display | rods | validation | 3 | 0.8169 (0.9507) | 0.6819 (0.8480) | 0.0974 (0.1190) | constant (simple): 0.7051 (0.7111) | Bhattacharjee et al. 2016, v1 train log width: 1.4034 (1.8155) |
| quench_display | spheres | pooled | 14 | 0.2308 (0.1769) | 0.0606 (0.0464) | 0.0271 (0.0195) | constant (simple): 0.7094 (0.6953) | Bhattacharjee et al. 2016, v1 train log width: 0.2089 (0.1632) |
| quench_display | spheres | train | 11 | 0.2587 (0.2154) | 0.0664 (0.0536) | 0.0303 (0.0234) | constant (simple): 0.7127 (0.6976) | Bhattacharjee et al. 2016, v1 train log width: 0.2290 (0.1890) |
| quench_display | spheres | validation | 3 | 0.0357 (0.0357) | 0.0199 (0.0199) | 0.0050 (0.0050) | constant (simple): 0.6870 (0.6870) | Bhattacharjee et al. 2016, v1 train log width: 0.0685 (0.0685) |
| quench_display | films | pooled | 5 | 0.1156 (0.1566) | 0.0724 (0.0955) | 0.0281 (0.0334) | constant (simple): 0.4834 (0.5822) | Bhattacharjee et al. 2016, v1 train log width: 0.0840 (0.1072) |
| quench_display | films | train | 4 | 0.1091 (0.1636) | 0.0650 (0.0975) | 0.0207 (0.0310) | constant (simple): 0.4887 (0.6095) | Bhattacharjee et al. 2016, v1 train log width: 0.0710 (0.1066) |
| quench_display | films | validation | 1 | 0.1287 (0.1287) | 0.0871 (0.0871) | 0.0429 (0.0429) | constant (simple): 0.4728 (0.4728) | Bhattacharjee et al. 2016, v1 train log width: 0.1099 (0.1099) |
| quench_display | slabs | pooled | 15 | 6.2334 (7.4220) | 6.2143 (7.3971) | 3.1050 (3.6169) | constant (simple): 8.1553 (8.9007) | Bhattacharjee et al. 2016, v1 train log width: 11.5591 (12.7455) |
| quench_display | slabs | train | 10 | 8.5511 (9.8990) | 8.5329 (9.8740) | 4.7004 (5.1393) | constant (simple): 11.5888 (12.0842) | Bhattacharjee et al. 2016, v1 train log width: 16.7678 (17.6094) |
| quench_display | slabs | validation | 5 | 2.7570 (2.4679) | 2.7363 (2.4432) | 0.7119 (0.5721) | constant (simple): 3.0052 (2.5336) | Bhattacharjee et al. 2016, v1 train log width: 3.7461 (3.0179) |

quench, v2, by day (mean of the day):

| day | bass2 s1 | bass2 K1 | bass3 |
|---|---|---|---|
| 2012-04-09 | 0.0000 | 0.0000 | 0.0000 |
| 2012-06-11 | 0.0185 | 0.0103 | 0.0026 |
| 2012-06-14 | 0.0000 | 0.0034 | 0.0009 |
| 2013-04-09 | 0.2895 | 0.0412 | 0.0174 |
| 2013-04-16 | 0.0000 | 0.0164 | 0.0047 |
| 2013-04-23 | 0.2793 | 0.0889 | 0.0372 |
| 2013-04-29 | 0.1218 | 0.0413 | 0.0357 |
| 2013-05-17 | 0.2375 | 0.0495 | 0.0140 |
| 2014-02-19 | 0.0000 | 0.0000 | 0.0000 |
| 2014-02-20 | 2.5083 | 2.5084 | 0.8018 |
| 2014-06-16 | 0.2000 | 0.0848 | 0.0286 |
| 2014-06-23 | 0.3423 | 0.1290 | 0.0419 |
| 2014-06-26 | 0.1966 | 0.0750 | 0.0315 |
| 2014-07-02 | 0.1227 | 0.0732 | 0.0233 |
| 2014-07-03 | 0.2461 | 0.1088 | 0.0193 |
| 2014-07-08 | 0.2681 | 0.0771 | 0.0492 |
| 2014-07-11 | 3.7901 | 3.7777 | 1.4615 |
| 2014-07-15 | 1.7495 | 1.7495 | 0.5607 |
| 2014-07-22 | 0.1836 | 0.0653 | 0.0292 |
| 2014-07-28 | 0.5179 | 0.3914 | 0.0232 |
| 2014-07-29 | 0.2885 | 0.2795 | 0.0384 |
| 2014-07-30 | 0.0499 | 0.0013 | 0.0292 |
| 2014-07-31 | 0.5344 | 0.5165 | 0.0077 |
| 2014-08-01 | 0.6233 | 0.5945 | 0.5263 |
| 2014-08-04 | 0.0609 | 0.0412 | 0.0203 |

bass2 K1 against bass2 s1: lower on 20 of 23 days whose scores differ; bass3 against bass2 s1: lower on 21 of 23 days whose scores differ; bass3 against bass2 K1: lower on 22 of 23 days whose scores differ.

quench_display, v2, by day (mean of the day):

| day | bass2 s1 | bass2 K1 | bass3 |
|---|---|---|---|
| 2012-04-09 | 0.0000 | 0.0000 | 0.0000 |
| 2012-06-11 | 0.0357 | 0.0199 | 0.0050 |
| 2012-06-14 | 0.0000 | 0.0066 | 0.0017 |
| 2013-04-09 | 0.5590 | 0.0796 | 0.0336 |
| 2013-04-16 | 0.0000 | 0.0317 | 0.0090 |
| 2013-04-23 | 0.5425 | 0.1718 | 0.0721 |
| 2013-04-29 | 0.2418 | 0.0797 | 0.0686 |
| 2013-05-17 | 0.4674 | 0.0956 | 0.0269 |
| 2014-02-19 | 0.0000 | 0.0000 | 0.0000 |
| 2014-02-20 | 10.4424 | 10.4145 | 7.1002 |
| 2014-06-16 | 0.3379 | 0.1433 | 0.0484 |
| 2014-06-23 | 0.5786 | 0.2182 | 0.0711 |
| 2014-06-26 | 0.4150 | 0.1584 | 0.0665 |
| 2014-07-02 | 0.2181 | 0.1301 | 0.0414 |
| 2014-07-03 | 0.4156 | 0.1838 | 0.0325 |
| 2014-07-08 | 0.5784 | 0.1628 | 0.1039 |
| 2014-07-11 | 10.5023 | 10.4745 | 3.9977 |
| 2014-07-15 | 4.2021 | 4.2020 | 1.4108 |
| 2014-07-22 | 0.4431 | 0.1577 | 0.0705 |
| 2014-07-28 | 2.1869 | 1.6531 | 0.0986 |
| 2014-07-29 | 1.2182 | 1.1801 | 0.1623 |
| 2014-07-30 | 0.2106 | 0.0053 | 0.1233 |
| 2014-07-31 | 1.3118 | 1.2707 | 0.0130 |
| 2014-08-01 | 1.6557 | 1.5951 | 1.0091 |
| 2014-08-04 | 0.1287 | 0.0871 | 0.0429 |

bass2 K1 against bass2 s1: lower on 21 of 23 days whose scores differ; bass3 against bass2 s1: lower on 21 of 23 days whose scores differ; bass3 against bass2 K1: lower on 22 of 23 days whose scores differ.

**Coding v1, all four releases (bass1 D1 included) on the cases all four answer** (bass1 D1 answers fewer quench cases, so the counts are smaller).

| target | stratum | partition | cases | bass1 D1 | bass2 s1 | bass2 K1 | bass3 | baseline 1 | baseline 2 |
|---|---|---|---|---|---|---|---|---|---|
| quench | all | pooled | 48 | 0.8135 (0.9684) | 0.4505 (0.5590) | 0.3747 (0.4895) | 0.1807 (0.2174) | constant (simple): 0.7079 (0.8731) | Bhattacharjee et al. 2016, train log width: 0.8423 (1.0313) |
| quench | all | train | 39 | 0.8987 (1.1036) | 0.5065 (0.6316) | 0.4194 (0.5512) | 0.2263 (0.2642) | constant (simple): 0.8406 (1.0189) | Bhattacharjee et al. 2016, train log width: 1.0263 (1.2261) |
| quench | all | validation | 9 | 0.5067 (0.3822) | 0.2489 (0.2445) | 0.2139 (0.2222) | 0.0169 (0.0146) | constant (simple): 0.2299 (0.2409) | Bhattacharjee et al. 2016, train log width: 0.1797 (0.1869) |
| quench | rods | pooled | 19 | 0.9741 (0.9050) | 0.2637 (0.2351) | 0.1434 (0.1209) | 0.0327 (0.0341) | constant (simple): 0.3034 (0.2836) | Bhattacharjee et al. 2016, train log width: 0.2856 (0.2539) |
| quench | rods | train | 16 | 0.9592 (0.9083) | 0.2625 (0.2277) | 0.1264 (0.1019) | 0.0340 (0.0344) | constant (simple): 0.2825 (0.2568) | Bhattacharjee et al. 2016, train log width: 0.2601 (0.2171) |
| quench | rods | validation | 3 | 1.0189 (0.8875) | 0.2673 (0.2744) | 0.1942 (0.2226) | 0.0288 (0.0320) | constant (simple): 0.3660 (0.4270) | Bhattacharjee et al. 2016, train log width: 0.3621 (0.4498) |
| quench | spheres | pooled | 14 | 0.1183 (0.0909) | 0.1183 (0.0909) | 0.0314 (0.0240) | 0.0140 (0.0101) | constant (simple): 0.1737 (0.1653) | Bhattacharjee et al. 2016, train log width: 0.1082 (0.0845) |
| quench | spheres | train | 11 | 0.1326 (0.1107) | 0.1326 (0.1107) | 0.0344 (0.0277) | 0.0157 (0.0121) | constant (simple): 0.1756 (0.1665) | Bhattacharjee et al. 2016, train log width: 0.1186 (0.0979) |
| quench | spheres | validation | 3 | 0.0185 (0.0185) | 0.0185 (0.0185) | 0.0103 (0.0103) | 0.0026 (0.0026) | constant (simple): 0.1607 (0.1607) | Bhattacharjee et al. 2016, train log width: 0.0355 (0.0355) |
| quench | films | pooled | 5 | 0.1627 (0.1998) | 0.0612 (0.0858) | 0.0381 (0.0521) | 0.0145 (0.0180) | constant (simple): 0.1477 (0.1902) | Bhattacharjee et al. 2016, train log width: 0.0440 (0.0583) |
| quench | films | train | 4 | 0.1278 (0.1917) | 0.0613 (0.0920) | 0.0366 (0.0549) | 0.0116 (0.0175) | constant (simple): 0.1672 (0.2106) | Bhattacharjee et al. 2016, train log width: 0.0400 (0.0599) |
| quench | films | validation | 1 | 0.2324 (0.2324) | 0.0609 (0.0609) | 0.0412 (0.0412) | 0.0203 (0.0203) | constant (simple): 0.1087 (0.1087) | Bhattacharjee et al. 2016, train log width: 0.0519 (0.0519) |
| quench | slabs | pooled | 10 | 2.3707 (2.7014) | 1.7805 (2.0665) | 1.7766 (2.0601) | 0.9349 (0.9558) | constant (simple): 3.0052 (3.3253) | Bhattacharjee et al. 2016, train log width: 4.0225 (4.3203) |
| quench | slabs | train | 8 | 3.0794 (3.3155) | 2.1639 (2.4255) | 2.1588 (2.4177) | 1.2453 (1.1938) | constant (simple): 3.9576 (4.1196) | Bhattacharjee et al. 2016, train log width: 5.3343 (5.3786) |
| quench | slabs | validation | 2 | 0.2447 (0.2447) | 0.6304 (0.6304) | 0.6299 (0.6299) | 0.0037 (0.0037) | constant (simple): 0.1481 (0.1481) | Bhattacharjee et al. 2016, train log width: 0.0870 (0.0870) |
| sustain | all | pooled | 82 | 0.0405 (0.0312) | 0.0405 (0.0312) | 0.0405 (0.0312) | 0.0405 (0.0312) | train incidence (simple): 0.0669 (0.0504) | always lit: 0.0700 (0.0488) |
| sustain | all | train | 60 | 0.0147 (0.0177) | 0.0147 (0.0177) | 0.0147 (0.0177) | 0.0147 (0.0177) | train incidence (simple): 0.0368 (0.0384) | always lit: 0.0312 (0.0333) |
| sustain | all | validation | 22 | 0.0862 (0.0681) | 0.0862 (0.0681) | 0.0862 (0.0681) | 0.0862 (0.0681) | train incidence (simple): 0.1203 (0.0831) | always lit: 0.1389 (0.0909) |
| sustain | rods | pooled | 18 | 0.0904 (0.0385) | 0.0904 (0.0385) | 0.0904 (0.0385) | 0.0904 (0.0385) | train incidence (simple): 0.1850 (0.0988) | always lit: 0.2222 (0.1111) |
| sustain | rods | train | 17 | 0.0133 (0.0128) | 0.0133 (0.0128) | 0.0133 (0.0128) | 0.0133 (0.0128) | train incidence (simple): 0.0643 (0.0582) | always lit: 0.0667 (0.0588) |
| sustain | rods | validation | 1 | 0.4759 (0.4759) | 0.4759 (0.4759) | 0.4759 (0.4759) | 0.4759 (0.4759) | train incidence (simple): 0.7885 (0.7885) | always lit: 1.0000 (1.0000) |
| sustain | spheres | pooled | 33 | 0.0274 (0.0311) | 0.0274 (0.0311) | 0.0274 (0.0311) | 0.0274 (0.0311) | train incidence (simple): 0.0302 (0.0361) | always lit: 0.0227 (0.0303) |
| sustain | spheres | train | 22 | 0.0006 (0.0005) | 0.0006 (0.0005) | 0.0006 (0.0005) | 0.0006 (0.0005) | train incidence (simple): 0.0125 (0.0125) | always lit: 0.0000 (0.0000) |
| sustain | spheres | validation | 11 | 0.0743 (0.0923) | 0.0743 (0.0923) | 0.0743 (0.0923) | 0.0743 (0.0923) | train incidence (simple): 0.0610 (0.0831) | always lit: 0.0625 (0.0909) |
| sustain | films | pooled | 13 | 0.0277 (0.0384) | 0.0277 (0.0384) | 0.0277 (0.0384) | 0.0277 (0.0384) | train incidence (simple): 0.0557 (0.0722) | always lit: 0.0556 (0.0769) |
| sustain | films | train | 11 | 0.0416 (0.0453) | 0.0416 (0.0453) | 0.0416 (0.0453) | 0.0416 (0.0453) | train incidence (simple): 0.0772 (0.0831) | always lit: 0.0833 (0.0909) |
| sustain | films | validation | 2 | 0.0000 (0.0000) | 0.0000 (0.0000) | 0.0000 (0.0000) | 0.0000 (0.0000) | train incidence (simple): 0.0125 (0.0125) | always lit: 0.0000 (0.0000) |
| sustain | slabs | pooled | 18 | 0.0146 (0.0190) | 0.0146 (0.0190) | 0.0146 (0.0190) | 0.0146 (0.0190) | train incidence (simple): 0.0125 (0.0125) | always lit: 0.0000 (0.0000) |
| sustain | slabs | train | 10 | 0.0282 (0.0335) | 0.0282 (0.0335) | 0.0282 (0.0335) | 0.0282 (0.0335) | train incidence (simple): 0.0125 (0.0125) | always lit: 0.0000 (0.0000) |
| sustain | slabs | validation | 8 | 0.0010 (0.0009) | 0.0010 (0.0009) | 0.0010 (0.0009) | 0.0010 (0.0009) | train incidence (simple): 0.0125 (0.0125) | always lit: 0.0000 (0.0000) |

quench, v1, by day (mean of the day):

| day | bass1 D1 | bass2 s1 | bass2 K1 | bass3 |
|---|---|---|---|---|
| 2012-04-09 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| 2012-06-11 | 0.0185 | 0.0185 | 0.0103 | 0.0026 |
| 2012-06-14 | 0.0000 | 0.0000 | 0.0034 | 0.0009 |
| 2013-04-09 | 0.2895 | 0.2895 | 0.0412 | 0.0174 |
| 2013-04-16 | 0.0000 | 0.0000 | 0.0164 | 0.0047 |
| 2013-04-23 | 0.2793 | 0.2793 | 0.0889 | 0.0372 |
| 2013-04-29 | 0.1218 | 0.1218 | 0.0413 | 0.0357 |
| 2013-05-17 | 0.2375 | 0.2375 | 0.0495 | 0.0140 |
| 2014-02-19 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| 2014-02-20 | 1.8134 | 1.3218 | 1.3217 | 0.6912 |
| 2014-06-16 | 0.9672 | 0.2000 | 0.0848 | 0.0286 |
| 2014-06-23 | 1.4638 | 0.3423 | 0.1290 | 0.0419 |
| 2014-06-26 | 1.0962 | 0.1966 | 0.0750 | 0.0315 |
| 2014-07-02 | 0.2556 | 0.1227 | 0.0732 | 0.0233 |
| 2014-07-03 | 1.4130 | 0.2461 | 0.1088 | 0.0193 |
| 2014-07-08 | 1.1496 | 0.2681 | 0.0771 | 0.0492 |
| 2014-07-11 | 4.5531 | 3.4228 | 3.4072 | 1.4773 |
| 2014-07-28 | 0.4680 | 0.5179 | 0.3914 | 0.0232 |
| 2014-07-29 | 0.6248 | 0.2885 | 0.2795 | 0.0384 |
| 2014-07-30 | 0.6101 | 0.0499 | 0.0013 | 0.0292 |
| 2014-07-31 | 0.2447 | 0.6304 | 0.6299 | 0.0037 |
| 2014-08-01 | 2.8718 | 1.7472 | 1.7474 | 1.5675 |
| 2014-08-04 | 0.2324 | 0.0609 | 0.0412 | 0.0203 |

bass2 K1 against bass2 s1: lower on 18 of 21 days whose scores differ; bass3 against bass2 s1: lower on 19 of 21 days whose scores differ; bass3 against bass2 K1: lower on 20 of 21 days whose scores differ; bass3 against bass1 D1: lower on 19 of 21 days whose scores differ.

**Coding v2, all four releases (bass1 D1 included) on the cases all four answer** (bass1 D1 answers fewer quench cases, so the counts are smaller).

| target | stratum | partition | cases | bass1 D1 | bass2 s1 | bass2 K1 | bass3 | baseline 1 | baseline 2 |
|---|---|---|---|---|---|---|---|---|---|
| quench | all | pooled | 53 | 0.9451 (1.2393) | 0.5654 (0.7925) | 0.4922 (0.7285) | 0.2007 (0.2661) | constant (simple): 0.9145 (1.2005) | Bhattacharjee et al. 2016, v1 train log width: 1.0642 (1.5024) |
| quench | all | train | 41 | 1.0137 (1.4175) | 0.5928 (0.8771) | 0.5059 (0.8006) | 0.2315 (0.3131) | constant (simple): 1.0033 (1.3614) | Bhattacharjee et al. 2016, v1 train log width: 1.2067 (1.7571) |
| quench | all | validation | 12 | 0.7393 (0.6305) | 0.4830 (0.5035) | 0.4510 (0.4824) | 0.1082 (0.1057) | constant (simple): 0.6482 (0.6508) | Bhattacharjee et al. 2016, v1 train log width: 0.6368 (0.6324) |
| quench | rods | pooled | 19 | 0.9741 (0.9050) | 0.2637 (0.2351) | 0.1434 (0.1209) | 0.0327 (0.0341) | constant (simple): 0.4166 (0.3965) | Bhattacharjee et al. 2016, v1 train log width: 0.2856 (0.2539) |
| quench | rods | train | 16 | 0.9592 (0.9083) | 0.2625 (0.2277) | 0.1264 (0.1019) | 0.0340 (0.0344) | constant (simple): 0.3959 (0.3690) | Bhattacharjee et al. 2016, v1 train log width: 0.2601 (0.2171) |
| quench | rods | validation | 3 | 1.0189 (0.8875) | 0.2673 (0.2744) | 0.1942 (0.2226) | 0.0288 (0.0320) | constant (simple): 0.4786 (0.5435) | Bhattacharjee et al. 2016, v1 train log width: 0.3621 (0.4498) |
| quench | spheres | pooled | 14 | 0.1183 (0.0909) | 0.1183 (0.0909) | 0.0314 (0.0240) | 0.0140 (0.0101) | constant (simple): 0.2851 (0.2762) | Bhattacharjee et al. 2016, v1 train log width: 0.1082 (0.0845) |
| quench | spheres | train | 11 | 0.1326 (0.1107) | 0.1326 (0.1107) | 0.0344 (0.0277) | 0.0157 (0.0121) | constant (simple): 0.2865 (0.2764) | Bhattacharjee et al. 2016, v1 train log width: 0.1186 (0.0979) |
| quench | spheres | validation | 3 | 0.0185 (0.0185) | 0.0185 (0.0185) | 0.0103 (0.0103) | 0.0026 (0.0026) | constant (simple): 0.2751 (0.2751) | Bhattacharjee et al. 2016, v1 train log width: 0.0355 (0.0355) |
| quench | films | pooled | 5 | 0.1627 (0.1998) | 0.0612 (0.0858) | 0.0381 (0.0521) | 0.0145 (0.0180) | constant (simple): 0.2368 (0.2710) | Bhattacharjee et al. 2016, v1 train log width: 0.0440 (0.0583) |
| quench | films | train | 4 | 0.1278 (0.1917) | 0.0613 (0.0920) | 0.0366 (0.0549) | 0.0116 (0.0175) | constant (simple): 0.2516 (0.2870) | Bhattacharjee et al. 2016, v1 train log width: 0.0400 (0.0599) |
| quench | films | validation | 1 | 0.2324 (0.2324) | 0.0609 (0.0609) | 0.0412 (0.0412) | 0.0203 (0.0203) | constant (simple): 0.2071 (0.2071) | Bhattacharjee et al. 2016, v1 train log width: 0.0519 (0.0519) |
| quench | slabs | pooled | 15 | 2.6909 (3.0811) | 2.0659 (2.3889) | 2.0599 (2.3812) | 0.8799 (0.8818) | constant (simple): 3.1248 (3.3914) | Bhattacharjee et al. 2016, v1 train log width: 4.4518 (4.8887) |
| quench | slabs | train | 10 | 3.7692 (4.1601) | 2.6819 (3.0731) | 2.6778 (3.0669) | 1.2770 (1.2082) | constant (simple): 4.3915 (4.5725) | Bhattacharjee et al. 2016, v1 train log width: 6.4166 (6.7250) |
| quench | slabs | validation | 5 | 1.0735 (0.9231) | 1.1420 (1.0205) | 1.1330 (1.0097) | 0.2842 (0.2289) | constant (simple): 1.2248 (1.0292) | Bhattacharjee et al. 2016, v1 train log width: 1.5047 (1.2162) |
| sustain | all | pooled | 81 | 0.0679 (0.0405) | 0.0679 (0.0405) | 0.0679 (0.0405) | 0.0679 (0.0405) | train incidence (simple): 0.0901 (0.0604) | always lit: 0.1000 (0.0617) |
| sustain | all | train | 60 | 0.0095 (0.0122) | 0.0095 (0.0122) | 0.0095 (0.0122) | 0.0095 (0.0122) | train incidence (simple): 0.0368 (0.0384) | always lit: 0.0312 (0.0333) |
| sustain | all | validation | 21 | 0.1717 (0.1215) | 0.1717 (0.1215) | 0.1717 (0.1215) | 0.1717 (0.1215) | train incidence (simple): 0.1850 (0.1234) | always lit: 0.2222 (0.1429) |
| sustain | rods | pooled | 18 | 0.0923 (0.0391) | 0.0923 (0.0391) | 0.0923 (0.0391) | 0.0923 (0.0391) | train incidence (simple): 0.1850 (0.0988) | always lit: 0.2222 (0.1111) |
| sustain | rods | train | 17 | 0.0133 (0.0128) | 0.0133 (0.0128) | 0.0133 (0.0128) | 0.0133 (0.0128) | train incidence (simple): 0.0643 (0.0582) | always lit: 0.0667 (0.0588) |
| sustain | rods | validation | 1 | 0.4871 (0.4871) | 0.4871 (0.4871) | 0.4871 (0.4871) | 0.4871 (0.4871) | train incidence (simple): 0.7885 (0.7885) | always lit: 1.0000 (1.0000) |
| sustain | spheres | pooled | 32 | 0.0060 (0.0026) | 0.0060 (0.0026) | 0.0060 (0.0026) | 0.0060 (0.0026) | train incidence (simple): 0.0125 (0.0125) | always lit: 0.0000 (0.0000) |
| sustain | spheres | train | 22 | 0.0006 (0.0005) | 0.0006 (0.0005) | 0.0006 (0.0005) | 0.0006 (0.0005) | train incidence (simple): 0.0125 (0.0125) | always lit: 0.0000 (0.0000) |
| sustain | spheres | validation | 10 | 0.0155 (0.0070) | 0.0155 (0.0070) | 0.0155 (0.0070) | 0.0155 (0.0070) | train incidence (simple): 0.0125 (0.0125) | always lit: 0.0000 (0.0000) |
| sustain | films | pooled | 13 | 0.0277 (0.0384) | 0.0277 (0.0384) | 0.0277 (0.0384) | 0.0277 (0.0384) | train incidence (simple): 0.0557 (0.0722) | always lit: 0.0556 (0.0769) |
| sustain | films | train | 11 | 0.0416 (0.0453) | 0.0416 (0.0453) | 0.0416 (0.0453) | 0.0416 (0.0453) | train incidence (simple): 0.0772 (0.0831) | always lit: 0.0833 (0.0909) |
| sustain | films | validation | 2 | 0.0000 (0.0000) | 0.0000 (0.0000) | 0.0000 (0.0000) | 0.0000 (0.0000) | train incidence (simple): 0.0125 (0.0125) | always lit: 0.0000 (0.0000) |
| sustain | slabs | pooled | 18 | 0.1663 (0.1110) | 0.1663 (0.1110) | 0.1663 (0.1110) | 0.1663 (0.1110) | train incidence (simple): 0.1419 (0.0988) | always lit: 0.1667 (0.1111) |
| sustain | slabs | train | 10 | 0.0006 (0.0004) | 0.0006 (0.0004) | 0.0006 (0.0004) | 0.0006 (0.0004) | train incidence (simple): 0.0125 (0.0125) | always lit: 0.0000 (0.0000) |
| sustain | slabs | validation | 8 | 0.3321 (0.2492) | 0.3321 (0.2492) | 0.3321 (0.2492) | 0.3321 (0.2492) | train incidence (simple): 0.2712 (0.2065) | always lit: 0.3333 (0.2500) |
| quench_display | all | pooled | 53 | 2.5009 (3.4644) | 1.6376 (2.4036) | 1.4783 (2.2560) | 0.6889 (1.0662) | constant (simple): 2.2458 (3.0210) | Bhattacharjee et al. 2016, v1 train log width: 2.8079 (3.9623) |
| quench_display | all | train | 41 | 2.7232 (3.9931) | 1.7772 (2.7309) | 1.5853 (2.5528) | 0.8259 (1.2983) | constant (simple): 2.5178 (3.4823) | Bhattacharjee et al. 2016, v1 train log width: 3.1618 (4.6135) |
| quench_display | all | validation | 12 | 1.8338 (1.6578) | 1.2187 (1.2856) | 1.1573 (1.2422) | 0.2778 (0.2730) | constant (simple): 1.4301 (1.4446) | Bhattacharjee et al. 2016, v1 train log width: 1.7462 (1.7376) |
| quench_display | rods | pooled | 19 | 2.3031 (2.2963) | 0.7427 (0.6739) | 0.4631 (0.3941) | 0.0883 (0.0954) | constant (simple): 0.7497 (0.7345) | Bhattacharjee et al. 2016, v1 train log width: 0.9589 (0.8421) |
| quench_display | rods | train | 16 | 2.2333 (2.2479) | 0.7179 (0.6220) | 0.3902 (0.3090) | 0.0853 (0.0909) | constant (simple): 0.7646 (0.7389) | Bhattacharjee et al. 2016, v1 train log width: 0.8107 (0.6596) |
| quench_display | rods | validation | 3 | 2.5126 (2.5546) | 0.8169 (0.9507) | 0.6819 (0.8480) | 0.0974 (0.1190) | constant (simple): 0.7051 (0.7111) | Bhattacharjee et al. 2016, v1 train log width: 1.4034 (1.8155) |
| quench_display | spheres | pooled | 14 | 0.2308 (0.1769) | 0.2308 (0.1769) | 0.0606 (0.0464) | 0.0271 (0.0195) | constant (simple): 0.7094 (0.6953) | Bhattacharjee et al. 2016, v1 train log width: 0.2089 (0.1632) |
| quench_display | spheres | train | 11 | 0.2587 (0.2154) | 0.2587 (0.2154) | 0.0664 (0.0536) | 0.0303 (0.0234) | constant (simple): 0.7127 (0.6976) | Bhattacharjee et al. 2016, v1 train log width: 0.2290 (0.1890) |
| quench_display | spheres | validation | 3 | 0.0357 (0.0357) | 0.0357 (0.0357) | 0.0199 (0.0199) | 0.0050 (0.0050) | constant (simple): 0.6870 (0.6870) | Bhattacharjee et al. 2016, v1 train log width: 0.0685 (0.0685) |
| quench_display | films | pooled | 5 | 0.3151 (0.3708) | 0.1156 (0.1566) | 0.0724 (0.0955) | 0.0281 (0.0334) | constant (simple): 0.4834 (0.5822) | Bhattacharjee et al. 2016, v1 train log width: 0.0840 (0.1072) |
| quench_display | films | train | 4 | 0.2272 (0.3408) | 0.1091 (0.1636) | 0.0650 (0.0975) | 0.0207 (0.0310) | constant (simple): 0.4887 (0.6095) | Bhattacharjee et al. 2016, v1 train log width: 0.0710 (0.1066) |
| quench_display | films | validation | 1 | 0.4910 (0.4910) | 0.1287 (0.1287) | 0.0871 (0.0871) | 0.0429 (0.0429) | constant (simple): 0.4728 (0.4728) | Bhattacharjee et al. 2016, v1 train log width: 0.1099 (0.1099) |
| quench_display | slabs | pooled | 15 | 7.7609 (9.0434) | 6.2334 (7.4220) | 6.2143 (7.3971) | 3.1050 (3.6169) | constant (simple): 8.1553 (8.9007) | Bhattacharjee et al. 2016, v1 train log width: 11.5591 (12.7455) |
| quench_display | slabs | train | 10 | 11.1178 (12.4018) | 8.5511 (9.8990) | 8.5329 (9.8740) | 4.7004 (5.1393) | constant (simple): 11.5888 (12.0842) | Bhattacharjee et al. 2016, v1 train log width: 16.7678 (17.6094) |
| quench_display | slabs | validation | 5 | 2.7255 (2.3264) | 2.7570 (2.4679) | 2.7363 (2.4432) | 0.7119 (0.5721) | constant (simple): 3.0052 (2.5336) | Bhattacharjee et al. 2016, v1 train log width: 3.7461 (3.0179) |

quench, v2, by day (mean of the day):

| day | bass1 D1 | bass2 s1 | bass2 K1 | bass3 |
|---|---|---|---|---|
| 2012-04-09 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| 2012-06-11 | 0.0185 | 0.0185 | 0.0103 | 0.0026 |
| 2012-06-14 | 0.0000 | 0.0000 | 0.0034 | 0.0009 |
| 2013-04-09 | 0.2895 | 0.2895 | 0.0412 | 0.0174 |
| 2013-04-16 | 0.0000 | 0.0000 | 0.0164 | 0.0047 |
| 2013-04-23 | 0.2793 | 0.2793 | 0.0889 | 0.0372 |
| 2013-04-29 | 0.1218 | 0.1218 | 0.0413 | 0.0357 |
| 2013-05-17 | 0.2375 | 0.2375 | 0.0495 | 0.0140 |
| 2014-02-19 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| 2014-02-20 | 3.4498 | 2.5083 | 2.5084 | 0.8018 |
| 2014-06-16 | 0.9672 | 0.2000 | 0.0848 | 0.0286 |
| 2014-06-23 | 1.4638 | 0.3423 | 0.1290 | 0.0419 |
| 2014-06-26 | 1.0962 | 0.1966 | 0.0750 | 0.0315 |
| 2014-07-02 | 0.2556 | 0.1227 | 0.0732 | 0.0233 |
| 2014-07-03 | 1.4130 | 0.2461 | 0.1088 | 0.0193 |
| 2014-07-08 | 1.1496 | 0.2681 | 0.0771 | 0.0492 |
| 2014-07-11 | 4.9860 | 3.7901 | 3.7777 | 1.4615 |
| 2014-07-15 | 1.8254 | 1.7495 | 1.7495 | 0.5607 |
| 2014-07-28 | 0.4680 | 0.5179 | 0.3914 | 0.0232 |
| 2014-07-29 | 0.6248 | 0.2885 | 0.2795 | 0.0384 |
| 2014-07-30 | 0.6101 | 0.0499 | 0.0013 | 0.0292 |
| 2014-07-31 | 0.3215 | 0.5344 | 0.5165 | 0.0077 |
| 2014-08-01 | 2.8718 | 1.7472 | 1.7474 | 1.5675 |
| 2014-08-04 | 0.2324 | 0.0609 | 0.0412 | 0.0203 |

bass2 K1 against bass2 s1: lower on 18 of 22 days whose scores differ; bass3 against bass2 s1: lower on 20 of 22 days whose scores differ; bass3 against bass2 K1: lower on 21 of 22 days whose scores differ; bass3 against bass1 D1: lower on 20 of 22 days whose scores differ.

quench_display, v2, by day (mean of the day):

| day | bass1 D1 | bass2 s1 | bass2 K1 | bass3 |
|---|---|---|---|---|
| 2012-04-09 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| 2012-06-11 | 0.0357 | 0.0357 | 0.0199 | 0.0050 |
| 2012-06-14 | 0.0000 | 0.0000 | 0.0066 | 0.0017 |
| 2013-04-09 | 0.5590 | 0.5590 | 0.0796 | 0.0336 |
| 2013-04-16 | 0.0000 | 0.0000 | 0.0317 | 0.0090 |
| 2013-04-23 | 0.5425 | 0.5425 | 0.1718 | 0.0721 |
| 2013-04-29 | 0.2418 | 0.2418 | 0.0797 | 0.0686 |
| 2013-05-17 | 0.4674 | 0.4674 | 0.0956 | 0.0269 |
| 2014-02-19 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| 2014-02-20 | 11.9455 | 10.4424 | 10.4145 | 7.1002 |
| 2014-06-16 | 1.6336 | 0.3379 | 0.1433 | 0.0484 |
| 2014-06-23 | 2.4730 | 0.5786 | 0.2182 | 0.0711 |
| 2014-06-26 | 2.3141 | 0.4150 | 0.1584 | 0.0665 |
| 2014-07-02 | 0.4544 | 0.2181 | 0.1301 | 0.0414 |
| 2014-07-03 | 2.3865 | 0.4156 | 0.1838 | 0.0325 |
| 2014-07-08 | 2.4265 | 0.5784 | 0.1628 | 0.1039 |
| 2014-07-11 | 13.7071 | 10.5023 | 10.4745 | 3.9977 |
| 2014-07-15 | 4.7207 | 4.2021 | 4.2020 | 1.4108 |
| 2014-07-28 | 1.9760 | 2.1869 | 1.6531 | 0.0986 |
| 2014-07-29 | 2.6386 | 1.2182 | 1.1801 | 0.1623 |
| 2014-07-30 | 2.5766 | 0.2106 | 0.0053 | 0.1233 |
| 2014-07-31 | 0.7303 | 1.3118 | 1.2707 | 0.0130 |
| 2014-08-01 | 7.7009 | 4.7086 | 4.7096 | 3.0034 |
| 2014-08-04 | 0.4910 | 0.1287 | 0.0871 | 0.0429 |

bass2 K1 against bass2 s1: lower on 19 of 22 days whose scores differ; bass3 against bass2 s1: lower on 20 of 22 days whose scores differ; bass3 against bass2 K1: lower on 21 of 22 days whose scores differ; bass3 against bass1 D1: lower on 20 of 22 days whose scores differ.

**Reading (both codings).** The order of the releases does not change between codings: on quench, pooled, bass3 < K1 < bass2 s1 < bass1 D1 on v1 (0.1310, 0.3138, 0.3926, 0.8135) and on v2 (0.1522, 0.4290, 0.5052, 0.9451), and the display scale gives the same order (0.5844, 1.3009, 1.4677, 2.5009). v2 admits five more quench cases (52 to 57, all of them slabs, 10 to 15; three of them validation), which raises every pooled number; on the strata other than slabs the forecasts and the outcomes are the same in both codings and the scores are identical. bass3 is lower than both baselines in every stratum and partition on v2 and on the display scale, as on v1; on v2 validation slabs (5 cases) bass3 0.2842 against bass2 1.1420, constant 1.2248. The day rule on v2: K1 lower than bass2 s1 on 20 of 23 days, bass3 lower than bass2 s1 on 21 of 23 and than K1 on 22 of 23 (display scale: 21, 21, 22 of 23). The 13.6 decisions (K1 kept, bass3 kept) read the same on v2. Kept burning: identical across the four releases on both codings (its lines are bass1 D1's); v2 changes sustain outcomes on spheres and slabs, and on v2 slabs the release (0.1663) is above train incidence (0.1419) and equal to always lit (0.1667) on 18 cases; on v1 slabs it was 0.0146 against 0.0125 and 0.0000. All numbers are development, in-sample.

**Final run.** `tools/wsp/final/run_final_gen7.py` takes the coding version from the frozen list (`"coding": {"bass": "v2"}` in `releases_final_gen7.json`); the rehearsal scores v1 against the registered numbers (REPRODUCED, 20 rows, 4 registered checks) and, because the list names v2, also rebuilds v2 with the lane's v2 builder through the same relabelled partition map and compares it with the v2 work files (REPRODUCED, 30 rows, quench_display included); report `runs/final/gen7/rehearsal/rehearsal_report.json`. The list's sha256 is now `613bfee6329bcf61c092b41e84605db17527048544fb8410ca2580fc85a14422` (still frozen false; its bass model is still bass2 s1, as the template had it).
