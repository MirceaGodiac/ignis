# Plastics: bass3 comparators S1 and Q1 (PROTOCOL 13.6), development, in-sample, on both codings

- S1: `sha256:cf0b3c0163a6c95ca3b2f1fac0aa7bb320603a10bc246592e31eddd3e6378396` (`configs/models/law/bass3/os1-fable`), a revision of the bass3 model `sha256:a1eb577ae257a788109329a8cc54509925ecab4c1e3e88ddcf9decf0fd7703d1` (`bass3/s1-fable`): `runs/dev/gen7/study/bass3_s1.wsp` (sha256 `cb27400ba1f53214164fb66b88282cf0a305bf9063ccaeafe824730ffa2f6f45`) verbatim; its lines are the marked comparator block of `docs/wsp/research/phys_bass_study2.md` section E (sha256 `c360d112399edc92bacebf5efc3ec70c8c683f5e57b67cefd8183b7fe3230c75`). Element check: equals bass3 except exactly the 2 elements written; mismatches 0.
- Q1: `sha256:3c0a5aaeaa67fb01d7c7f56f1041d9853c87cf599315c7a91a14323048b49b6a` (`configs/models/law/bass3/q1-fable`), a revision of the bass3 model `sha256:a1eb577ae257a788109329a8cc54509925ecab4c1e3e88ddcf9decf0fd7703d1` (`bass3/s1-fable`): `runs/dev/gen7/study/bass3_q1.wsp` (sha256 `ef6bcbd80bd7bb6be6f169b07a447f8ea416bb414ec93909bd8d2f2d4a466aad`) verbatim; its lines are the marked comparator block of `docs/wsp/research/phys_bass_study2.md` section E (sha256 `c360d112399edc92bacebf5efc3ec70c8c683f5e57b67cefd8183b7fe3230c75`). Element check: equals bass3 except exactly the 10 elements written; mismatches 0.
- Folder: S1 is in `os1-fable` because `bass3/s1-fable` is the bass3 model itself.
- Scores: Rust forward through the lane scorer (`law_score.py`, lane `score()`), by `tools/wsp/ext/law_revision.py dev --coding v1|v2`; train and validation days together (in-sample), on the cases bass3 and the candidate both answer; other releases and baselines on the same cases. Interval CRPS cm/s (quench), Brier (sustain = kept burning); value is the mean of day means, case mean in brackets. Lower is better. v2 = the second coding (the final run; the author's what-if runs); v1 = the registered coding.

## S1, coding v2

| target | stratum | partition | bass3 | S1 | bass2 K1 | bass2 s1 | constant (simple) | Bhattacharjee et al. 2016, v1 train log width | train incidence (simple) | always lit |
|---|---|---|---|---|---|---|---|---|---|---|
| quench | all | pooled | 0.1522 (0.2487) | 0.1522 (0.2487) | 0.4290 (0.6803) | 0.5052 (0.7455) | 0.7890 (1.1365) | 0.8758 (1.4050) | - (-) | - (-) |
| quench | all | train | 0.1661 (0.2868) | 0.1661 (0.2868) | 0.4220 (0.7331) | 0.5122 (0.8100) | 0.8334 (1.2660) | 0.9512 (1.6110) | - (-) | - (-) |
| quench | all | validation | 0.1082 (0.1057) | 0.1082 (0.1057) | 0.4510 (0.4824) | 0.4830 (0.5035) | 0.6482 (0.6508) | 0.6368 (0.6324) | - (-) | - (-) |
| quench | films | pooled | 0.0145 (0.0180) | 0.0145 (0.0180) | 0.0381 (0.0521) | 0.0612 (0.0858) | 0.2368 (0.2710) | 0.0440 (0.0583) | - (-) | - (-) |
| quench | rods | pooled | 0.0296 (0.0312) | 0.0296 (0.0312) | 0.1230 (0.1071) | 0.2354 (0.2155) | 0.3909 (0.3777) | 0.2512 (0.2295) | - (-) | - (-) |
| quench | sections | pooled | - (-) | - (-) | - (-) | - (-) | - (-) | - (-) | - (-) | - (-) |
| quench | slabs | pooled | 0.8799 (0.8818) | 0.8799 (0.8818) | 2.0599 (2.3812) | 2.0659 (2.3889) | 3.1248 (3.3914) | 4.4518 (4.8887) | - (-) | - (-) |
| quench | spheres | pooled | 0.0140 (0.0101) | 0.0140 (0.0101) | 0.0314 (0.0240) | 0.1183 (0.0909) | 0.2851 (0.2762) | 0.1082 (0.0845) | - (-) | - (-) |
| sustain | all | pooled | 0.0467 (0.0328) | 0.0247 (0.0228) | 0.0467 (0.0328) | 0.0467 (0.0328) | - (-) | - (-) | 0.0571 (0.0513) | 0.0575 (0.0500) |
| sustain | all | train | 0.0070 (0.0099) | 0.0126 (0.0165) | 0.0070 (0.0099) | 0.0070 (0.0099) | - (-) | - (-) | 0.0262 (0.0335) | 0.0175 (0.0270) |
| sustain | all | validation | 0.1221 (0.0981) | 0.0476 (0.0408) | 0.1221 (0.0981) | 0.1221 (0.0981) | - (-) | - (-) | 0.1160 (0.1021) | 0.1333 (0.1154) |
| sustain | films | pooled | 0.0277 (0.0384) | 0.0277 (0.0384) | 0.0277 (0.0384) | 0.0277 (0.0384) | - (-) | - (-) | 0.0557 (0.0722) | 0.0556 (0.0769) |
| sustain | rods | pooled | 0.0209 (0.0190) | 0.0209 (0.0190) | 0.0209 (0.0190) | 0.0209 (0.0190) | - (-) | - (-) | 0.0513 (0.0545) | 0.0500 (0.0541) |
| sustain | sections | pooled | 0.0577 (0.0577) | 0.0577 (0.0577) | 0.0577 (0.0577) | 0.0577 (0.0577) | - (-) | - (-) | 0.0125 (0.0125) | 0.0000 (0.0000) |
| sustain | slabs | pooled | 0.1663 (0.1110) | 0.0600 (0.0553) | 0.1663 (0.1110) | 0.1663 (0.1110) | - (-) | - (-) | 0.1419 (0.0988) | 0.1667 (0.1111) |
| sustain | spheres | pooled | 0.0008 (0.0008) | 0.0008 (0.0008) | 0.0008 (0.0008) | 0.0008 (0.0008) | - (-) | - (-) | 0.0125 (0.0125) | 0.0000 (0.0000) |
| quench_display | all | pooled | 0.5844 (0.9942) | 0.5844 (0.9942) | 1.3009 (2.1046) | 1.4677 (2.2551) | 1.9102 (2.8494) | 2.3103 (3.7035) | - (-) | - (-) |
| quench_display | all | train | 0.6812 (1.1866) | 0.6812 (1.1866) | 1.3462 (2.3345) | 1.5463 (2.5136) | 2.0618 (3.2240) | 2.4885 (4.2277) | - (-) | - (-) |
| quench_display | all | validation | 0.2778 (0.2730) | 0.2778 (0.2730) | 1.1573 (1.2422) | 1.2187 (1.2856) | 1.4301 (1.4446) | 1.7462 (1.7376) | - (-) | - (-) |
| quench_display | films | pooled | 0.0281 (0.0334) | 0.0281 (0.0334) | 0.0724 (0.0955) | 0.1156 (0.1566) | 0.4834 (0.5822) | 0.0840 (0.1072) | - (-) | - (-) |
| quench_display | rods | pooled | 0.0789 (0.0860) | 0.0789 (0.0860) | 0.3900 (0.3425) | 0.6514 (0.6065) | 0.7150 (0.7070) | 0.8218 (0.7433) | - (-) | - (-) |
| quench_display | sections | pooled | - (-) | - (-) | - (-) | - (-) | - (-) | - (-) | - (-) | - (-) |
| quench_display | slabs | pooled | 3.1050 (3.6169) | 3.1050 (3.6169) | 6.2143 (7.3971) | 6.2334 (7.4220) | 8.1553 (8.9007) | 11.5591 (12.7455) | - (-) | - (-) |
| quench_display | spheres | pooled | 0.0271 (0.0195) | 0.0271 (0.0195) | 0.0606 (0.0464) | 0.2308 (0.1769) | 0.7094 (0.6953) | 0.2089 (0.1632) | - (-) | - (-) |

By day, quench_u: candidate lower on **0 of 0** days whose forecasts change (25 days).
By day, sustain: candidate lower on **1 of 3** days whose forecasts change (29 days): 2013-04-26 0.9934 to 0.2483; 2014-02-20 0.0000 to 0.0465; 2014-07-11 0.0005 to 0.0608.

Agreement with the logged what-if run of S1 (2026-11-15T08:49:29+00:00, coding v2, Python copy of the forward): largest pooled difference 3.3e-16; days identical.

## Q1, coding v2

| target | stratum | partition | bass3 | Q1 | bass2 K1 | bass2 s1 | constant (simple) | Bhattacharjee et al. 2016, v1 train log width | train incidence (simple) | always lit |
|---|---|---|---|---|---|---|---|---|---|---|
| quench | all | pooled | 0.1522 (0.2487) | 0.1348 (0.2106) | 0.4290 (0.6803) | 0.5052 (0.7455) | 0.7890 (1.1365) | 0.8758 (1.4050) | - (-) | - (-) |
| quench | all | train | 0.1661 (0.2868) | 0.1432 (0.2386) | 0.4220 (0.7331) | 0.5122 (0.8100) | 0.8334 (1.2660) | 0.9512 (1.6110) | - (-) | - (-) |
| quench | all | validation | 0.1082 (0.1057) | 0.1082 (0.1057) | 0.4510 (0.4824) | 0.4830 (0.5035) | 0.6482 (0.6508) | 0.6368 (0.6324) | - (-) | - (-) |
| quench | films | pooled | 0.0145 (0.0180) | 0.0145 (0.0180) | 0.0381 (0.0521) | 0.0612 (0.0858) | 0.2368 (0.2710) | 0.0440 (0.0583) | - (-) | - (-) |
| quench | rods | pooled | 0.0296 (0.0312) | 0.0296 (0.0312) | 0.1230 (0.1071) | 0.2354 (0.2155) | 0.3909 (0.3777) | 0.2512 (0.2295) | - (-) | - (-) |
| quench | sections | pooled | - (-) | - (-) | - (-) | - (-) | - (-) | - (-) | - (-) | - (-) |
| quench | slabs | pooled | 0.8799 (0.8818) | 0.7928 (0.7372) | 2.0599 (2.3812) | 2.0659 (2.3889) | 3.1248 (3.3914) | 4.4518 (4.8887) | - (-) | - (-) |
| quench | spheres | pooled | 0.0140 (0.0101) | 0.0140 (0.0101) | 0.0314 (0.0240) | 0.1183 (0.0909) | 0.2851 (0.2762) | 0.1082 (0.0845) | - (-) | - (-) |
| sustain | all | pooled | 0.0467 (0.0328) | 0.0467 (0.0328) | 0.0467 (0.0328) | 0.0467 (0.0328) | - (-) | - (-) | 0.0571 (0.0513) | 0.0575 (0.0500) |
| sustain | all | train | 0.0070 (0.0099) | 0.0070 (0.0099) | 0.0070 (0.0099) | 0.0070 (0.0099) | - (-) | - (-) | 0.0262 (0.0335) | 0.0175 (0.0270) |
| sustain | all | validation | 0.1221 (0.0981) | 0.1221 (0.0981) | 0.1221 (0.0981) | 0.1221 (0.0981) | - (-) | - (-) | 0.1160 (0.1021) | 0.1333 (0.1154) |
| sustain | films | pooled | 0.0277 (0.0384) | 0.0277 (0.0384) | 0.0277 (0.0384) | 0.0277 (0.0384) | - (-) | - (-) | 0.0557 (0.0722) | 0.0556 (0.0769) |
| sustain | rods | pooled | 0.0209 (0.0190) | 0.0209 (0.0190) | 0.0209 (0.0190) | 0.0209 (0.0190) | - (-) | - (-) | 0.0513 (0.0545) | 0.0500 (0.0541) |
| sustain | sections | pooled | 0.0577 (0.0577) | 0.0577 (0.0577) | 0.0577 (0.0577) | 0.0577 (0.0577) | - (-) | - (-) | 0.0125 (0.0125) | 0.0000 (0.0000) |
| sustain | slabs | pooled | 0.1663 (0.1110) | 0.1663 (0.1110) | 0.1663 (0.1110) | 0.1663 (0.1110) | - (-) | - (-) | 0.1419 (0.0988) | 0.1667 (0.1111) |
| sustain | spheres | pooled | 0.0008 (0.0008) | 0.0008 (0.0008) | 0.0008 (0.0008) | 0.0008 (0.0008) | - (-) | - (-) | 0.0125 (0.0125) | 0.0000 (0.0000) |
| quench_display | all | pooled | 0.5844 (0.9942) | 0.5362 (0.8884) | 1.3009 (2.1046) | 1.4677 (2.2551) | 1.9102 (2.8494) | 2.3103 (3.7035) | - (-) | - (-) |
| quench_display | all | train | 0.6812 (1.1866) | 0.6179 (1.0525) | 1.3462 (2.3345) | 1.5463 (2.5136) | 2.0618 (3.2240) | 2.4885 (4.2277) | - (-) | - (-) |
| quench_display | all | validation | 0.2778 (0.2730) | 0.2778 (0.2730) | 1.1573 (1.2422) | 1.2187 (1.2856) | 1.4301 (1.4446) | 1.7462 (1.7376) | - (-) | - (-) |
| quench_display | films | pooled | 0.0281 (0.0334) | 0.0281 (0.0334) | 0.0724 (0.0955) | 0.1156 (0.1566) | 0.4834 (0.5822) | 0.0840 (0.1072) | - (-) | - (-) |
| quench_display | rods | pooled | 0.0789 (0.0860) | 0.0789 (0.0860) | 0.3900 (0.3425) | 0.6514 (0.6065) | 0.7150 (0.7070) | 0.8218 (0.7433) | - (-) | - (-) |
| quench_display | sections | pooled | - (-) | - (-) | - (-) | - (-) | - (-) | - (-) | - (-) | - (-) |
| quench_display | slabs | pooled | 3.1050 (3.6169) | 2.8644 (3.2146) | 6.2143 (7.3971) | 6.2334 (7.4220) | 8.1553 (8.9007) | 11.5591 (12.7455) | - (-) | - (-) |
| quench_display | spheres | pooled | 0.0271 (0.0195) | 0.0271 (0.0195) | 0.0606 (0.0464) | 0.2308 (0.1769) | 0.7094 (0.6953) | 0.2089 (0.1632) | - (-) | - (-) |

By day, quench_u: candidate lower on **2 of 2** days whose forecasts change (25 days): 2014-02-20 0.8018 to 0.7941; 2014-07-11 1.4773 to 0.9542.
By day, sustain: candidate lower on **0 of 0** days whose forecasts change (29 days).

Agreement with the logged what-if run of Q1 (2026-11-15T08:50:07+00:00, coding v2, Python copy of the forward): largest pooled difference 3.3e-16; days identical.

## S1, coding v1

| target | stratum | partition | bass3 | S1 | bass2 K1 | bass2 s1 | constant (simple) | Bhattacharjee et al. 2016, train log width | train incidence (simple) | always lit |
|---|---|---|---|---|---|---|---|---|---|---|
| quench | all | pooled | 0.1310 (0.2020) | 0.1310 (0.2020) | 0.3138 (0.4550) | 0.3926 (0.5254) | 0.5786 (0.8194) | 0.6552 (0.9607) | - (-) | - (-) |
| quench | all | train | 0.1611 (0.2413) | 0.1611 (0.2413) | 0.3401 (0.5038) | 0.4304 (0.5842) | 0.6704 (0.9405) | 0.7803 (1.1226) | - (-) | - (-) |
| quench | all | validation | 0.0169 (0.0146) | 0.0169 (0.0146) | 0.2139 (0.2222) | 0.2489 (0.2445) | 0.2299 (0.2409) | 0.1797 (0.1869) | - (-) | - (-) |
| quench | films | pooled | 0.0145 (0.0180) | 0.0145 (0.0180) | 0.0381 (0.0521) | 0.0612 (0.0858) | 0.1477 (0.1902) | 0.0440 (0.0583) | - (-) | - (-) |
| quench | rods | pooled | 0.0296 (0.0312) | 0.0296 (0.0312) | 0.1230 (0.1071) | 0.2354 (0.2155) | 0.2779 (0.2649) | 0.2512 (0.2295) | - (-) | - (-) |
| quench | sections | pooled | - (-) | - (-) | - (-) | - (-) | - (-) | - (-) | - (-) | - (-) |
| quench | slabs | pooled | 0.9349 (0.9558) | 0.9349 (0.9558) | 1.7766 (2.0601) | 1.7805 (2.0665) | 3.0052 (3.3253) | 4.0225 (4.3203) | - (-) | - (-) |
| quench | spheres | pooled | 0.0140 (0.0101) | 0.0140 (0.0101) | 0.0314 (0.0240) | 0.1183 (0.0909) | 0.1737 (0.1653) | 0.1082 (0.0845) | - (-) | - (-) |
| sustain | all | pooled | 0.0233 (0.0254) | 0.0388 (0.0388) | 0.0233 (0.0254) | 0.0233 (0.0254) | - (-) | - (-) | 0.0371 (0.0433) | 0.0316 (0.0396) |
| sustain | all | train | 0.0114 (0.0144) | 0.0218 (0.0260) | 0.0114 (0.0144) | 0.0114 (0.0144) | - (-) | - (-) | 0.0262 (0.0335) | 0.0175 (0.0270) |
| sustain | all | validation | 0.0459 (0.0555) | 0.0711 (0.0742) | 0.0459 (0.0555) | 0.0459 (0.0555) | - (-) | - (-) | 0.0578 (0.0700) | 0.0583 (0.0741) |
| sustain | films | pooled | 0.0277 (0.0384) | 0.0277 (0.0384) | 0.0277 (0.0384) | 0.0277 (0.0384) | - (-) | - (-) | 0.0557 (0.0722) | 0.0556 (0.0769) |
| sustain | rods | pooled | 0.0206 (0.0187) | 0.0206 (0.0187) | 0.0206 (0.0187) | 0.0206 (0.0187) | - (-) | - (-) | 0.0513 (0.0545) | 0.0500 (0.0541) |
| sustain | sections | pooled | 0.0577 (0.0577) | 0.0577 (0.0577) | 0.0577 (0.0577) | 0.0577 (0.0577) | - (-) | - (-) | 0.0125 (0.0125) | 0.0000 (0.0000) |
| sustain | slabs | pooled | 0.0146 (0.0190) | 0.0898 (0.0946) | 0.0146 (0.0190) | 0.0146 (0.0190) | - (-) | - (-) | 0.0125 (0.0125) | 0.0000 (0.0000) |
| sustain | spheres | pooled | 0.0244 (0.0303) | 0.0244 (0.0303) | 0.0244 (0.0303) | 0.0244 (0.0303) | - (-) | - (-) | 0.0319 (0.0368) | 0.0250 (0.0312) |

By day, quench_u: candidate lower on **0 of 0** days whose forecasts change (24 days).
By day, sustain: candidate lower on **0 of 3** days whose forecasts change (29 days): 2013-04-26 0.0000 to 0.2517; 2014-02-20 0.0829 to 0.2218; 2014-07-11 0.0005 to 0.0608.

## Q1, coding v1

| target | stratum | partition | bass3 | Q1 | bass2 K1 | bass2 s1 | constant (simple) | Bhattacharjee et al. 2016, train log width | train incidence (simple) | always lit |
|---|---|---|---|---|---|---|---|---|---|---|
| quench | all | pooled | 0.1310 (0.2020) | 0.1088 (0.1612) | 0.3138 (0.4550) | 0.3926 (0.5254) | 0.5786 (0.8194) | 0.6552 (0.9607) | - (-) | - (-) |
| quench | all | train | 0.1611 (0.2413) | 0.1330 (0.1919) | 0.3401 (0.5038) | 0.4304 (0.5842) | 0.6704 (0.9405) | 0.7803 (1.1226) | - (-) | - (-) |
| quench | all | validation | 0.0169 (0.0146) | 0.0169 (0.0146) | 0.2139 (0.2222) | 0.2489 (0.2445) | 0.2299 (0.2409) | 0.1797 (0.1869) | - (-) | - (-) |
| quench | films | pooled | 0.0145 (0.0180) | 0.0145 (0.0180) | 0.0381 (0.0521) | 0.0612 (0.0858) | 0.1477 (0.1902) | 0.0440 (0.0583) | - (-) | - (-) |
| quench | rods | pooled | 0.0296 (0.0312) | 0.0296 (0.0312) | 0.1230 (0.1071) | 0.2354 (0.2155) | 0.2779 (0.2649) | 0.2512 (0.2295) | - (-) | - (-) |
| quench | sections | pooled | - (-) | - (-) | - (-) | - (-) | - (-) | - (-) | - (-) | - (-) |
| quench | slabs | pooled | 0.9349 (0.9558) | 0.8016 (0.7435) | 1.7766 (2.0601) | 1.7805 (2.0665) | 3.0052 (3.3253) | 4.0225 (4.3203) | - (-) | - (-) |
| quench | spheres | pooled | 0.0140 (0.0101) | 0.0140 (0.0101) | 0.0314 (0.0240) | 0.1183 (0.0909) | 0.1737 (0.1653) | 0.1082 (0.0845) | - (-) | - (-) |
| sustain | all | pooled | 0.0233 (0.0254) | 0.0233 (0.0254) | 0.0233 (0.0254) | 0.0233 (0.0254) | - (-) | - (-) | 0.0371 (0.0433) | 0.0316 (0.0396) |
| sustain | all | train | 0.0114 (0.0144) | 0.0114 (0.0144) | 0.0114 (0.0144) | 0.0114 (0.0144) | - (-) | - (-) | 0.0262 (0.0335) | 0.0175 (0.0270) |
| sustain | all | validation | 0.0459 (0.0555) | 0.0459 (0.0555) | 0.0459 (0.0555) | 0.0459 (0.0555) | - (-) | - (-) | 0.0578 (0.0700) | 0.0583 (0.0741) |
| sustain | films | pooled | 0.0277 (0.0384) | 0.0277 (0.0384) | 0.0277 (0.0384) | 0.0277 (0.0384) | - (-) | - (-) | 0.0557 (0.0722) | 0.0556 (0.0769) |
| sustain | rods | pooled | 0.0206 (0.0187) | 0.0206 (0.0187) | 0.0206 (0.0187) | 0.0206 (0.0187) | - (-) | - (-) | 0.0513 (0.0545) | 0.0500 (0.0541) |
| sustain | sections | pooled | 0.0577 (0.0577) | 0.0577 (0.0577) | 0.0577 (0.0577) | 0.0577 (0.0577) | - (-) | - (-) | 0.0125 (0.0125) | 0.0000 (0.0000) |
| sustain | slabs | pooled | 0.0146 (0.0190) | 0.0146 (0.0190) | 0.0146 (0.0190) | 0.0146 (0.0190) | - (-) | - (-) | 0.0125 (0.0125) | 0.0000 (0.0000) |
| sustain | spheres | pooled | 0.0244 (0.0303) | 0.0244 (0.0303) | 0.0244 (0.0303) | 0.0244 (0.0303) | - (-) | - (-) | 0.0319 (0.0368) | 0.0250 (0.0312) |

By day, quench_u: candidate lower on **2 of 2** days whose forecasts change (24 days): 2014-02-20 0.6912 to 0.6809; 2014-07-11 1.4773 to 0.9542.
By day, sustain: candidate lower on **0 of 0** days whose forecasts change (29 days).
**Reading (13.6).**

- **S1** (target: kept burning). Coding v2: 0.0247 against bass3 0.0467 pooled (slabs 0.0600 against 0.1663), lower on 1 of 3 days that change; quench identical. Coding v1 (registered): **higher**, 0.0388 against 0.0233 (slabs 0.0898 against 0.0146), lower on 0 of 3 days. Three days change, fewer than six: a **comparator**, not the model. Its effect has opposite signs on the two codings: day 2013-04-26 (tests 72 and 73) scores 0.0000 under bass3 on v1 and 0.9934 on v2, so the two codings record those burns differently, and on v1 the floor only costs the one-sided slabs that held.
- **Q1** (target: quench speed). Coding v2: 0.1348 against 0.1522 pooled (slabs 0.7928 against 0.8799; display scale 0.5362 against 0.5844), lower on 2 of 2 days; kept burning identical. Coding v1: 0.1088 against 0.1310 (slabs 0.8016 against 0.9349), lower on 2 of 2 days. Two days change: a **comparator**. bass3 stays the plastics model.

Development numbers, in-sample; nothing is validated.
