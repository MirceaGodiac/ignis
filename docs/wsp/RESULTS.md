# Development results of the literal (WSP) releases

Produced by `tools/wsp/dev_eval.py`. Validation: sealed forecasts scored and paired by `ignis eval compare` (group-equal means, cluster bootstrap; the validation partitions have few independent clusters, so intervals are wide). Train: `wsp-dev feedback` (case means, evaluator scoring functions). Cutoffs 1, 3 and 5 s. **The test partition was not read.** A release marked data-informed was written after its author read development data; only releases marked zero-shot were written before any outcome of their fuel or family was seen.

## FLEX methanol (PSI-69)

Cohort `sha256:9e31b1aa6b56b2086aac85776d27778bb1680a83fcdfc543dad62e0385da637e` · split `sha256:660b7d9ede629b3009b4b761ea9e5875560719566e3dea21112f607cddd49894`.

| release | id | forward | nonzero / authored |
|---|---|---|---|
| flex_methanol.wsp2 D3 — data-informed (train) | `sha256:c8a90dabbed6af4185de94c3cd5ac27486c19eb52c697d86b028849a37fcb654` | `sha256:71ea3173…` | 68 / 378 |
| flex_methanol.wsp2 D2-law — hand-written reference | `sha256:abf919b6199864c0a0eb6d27beffb295421c9d080f664d7c9a5929d6f254247b` | `sha256:71ea3173…` | 28 / 378 |
| flex.d2_population — fitted on train | `sha256:4bfddff5060f096d8f6a18cb382058ebd231dffd76b9b7f4b52f48bbbbef4894` | — | fitted |
| flex.d2_prefix — fitted on train | `sha256:d99ac9b6a533c6ba878ae9d9694d435edcd33996881a4454489f28eebcc9ea7e` | — | fitted |
| flex.v1 (MAP) — fitted on train | `sha256:1ee28f9879d6db84053cbfadc2de75ee2f33a9ea0981679f820ece3a48e11cc7` | — | fitted |
| flex.v3 (MAP) — fitted on train | `sha256:6e89480df15640e7fa19205893722c2a821ea5735a94adeb9e96d0c296b0b032` | — | fitted |

**Validation** — 56 common cases; CRPS and MAE in µm.

| release | CRPS | MAE of median | coverage 50 % | coverage 90 % | lpd |
|---|---|---|---|---|---|
| flex.v1 (MAP) (fitted on train) | 93.8 | 124.7 | 0.53 | 0.82 | 7.40 |
| flex.d2_population (fitted on train) | 137.5 | 149.9 | 0.05 | 0.11 | -6.09 |
| flex_methanol.wsp2 D3 (data-informed (train)) | 40.0 | 53.7 | 0.71 | 0.90 | 8.60 |
| flex.v3 (MAP) (fitted on train) | 77.7 | 108.6 | 0.34 | 0.75 | 7.08 |
| flex_methanol.wsp2 D2-law (hand-written reference) | 102.1 | 126.8 | 0.53 | 0.72 | 7.57 |
| flex.d2_prefix (fitted on train) | 103.4 | 145.0 | 0.50 | 0.79 | 6.80 |

Paired difference to flex_methanol.wsp2 D3 (data-informed (train)) (other − reference, µm, 95 % CI):

- flex.v1 (MAP) (fitted on train): CRPS +53.8 [+6.4, +116.9] (56 cases, 9 clusters)
- flex.d2_population (fitted on train): CRPS +97.5 [+42.7, +157.2] (56 cases, 9 clusters)
- flex.v3 (MAP) (fitted on train): CRPS +37.7 [+14.2, +60.8] (56 cases, 9 clusters)
- flex_methanol.wsp2 D2-law (hand-written reference): CRPS +62.0 [+26.4, +105.2] (56 cases, 9 clusters)
- flex.d2_prefix (fitted on train): CRPS +63.4 [+38.3, +97.8] (56 cases, 9 clusters)

**Train** — µm. The mean is dominated by a few artefact runs (soot shell, gas bubble, droplet leaving and re-entering the view); the median and the mean without the worst 5 % of cases are given beside it.

| release | cases | refused | CRPS mean | CRPS median | CRPS mean, worst 5 % dropped | MAE of median | coverage 50 % | coverage 90 % |
|---|---|---|---|---|---|---|---|---|
| flex_methanol.wsp2 D3 (data-informed (train)) | 182 | 3 | 29.9 | 17.7 | 19.2 | 37.8 | 0.77 | 0.94 |
| flex_methanol.wsp2 D2-law (hand-written reference) | 182 | 3 | 87.2 | 37.9 | 59.7 | 112.9 | 0.49 | 0.77 |
| flex.d2_population (fitted on train) | 164 | 21 | 149.9 | 113.7 | 128.5 | 165.5 | 0.06 | 0.15 |
| flex.d2_prefix (fitted on train) | 164 | 21 | 81.1 | 58.9 | 68.9 | 124.3 | 0.53 | 0.93 |
| flex.v1 (MAP) (fitted on train) | 164 | 21 | 127.3 | 60.6 | 92.2 | 187.8 | 0.55 | 0.89 |
| flex.v3 (MAP) (fitted on train) | 164 | 21 | 68.2 | 47.3 | 57.1 | 97.4 | 0.47 | 0.85 |

## FLEX n-heptane (PSI-69)

Cohort `sha256:8ffa1875d0c963fe3b3ce6886ba7197a7b0c8675ffd837bbf5139049c1655797` · split `sha256:c43bbe3580ffce2a33c400d5aea73b3330cf721133f925c2963579dee6d32001`.

| release | id | forward | nonzero / authored |
|---|---|---|---|
| flex_heptane.wsp2 D2 — data-informed (train) | `sha256:edbad4d6395961030e0b17d9d5c4d40f2de902ddcbcbd0ec2a4f4ec9adbeedef` | `sha256:71ea3173…` | 93 / 575 |
| flex_heptane.wsp2 D2-law — hand-written reference | `sha256:8e40db9dceb898e336f69422d3e2cff2df52b6050188ef84af1a9d0a7ba4ebb1` | `sha256:71ea3173…` | 33 / 575 |

**Validation** — 39 common cases; CRPS and MAE in µm.

| release | CRPS | MAE of median | coverage 50 % | coverage 90 % | lpd |
|---|---|---|---|---|---|
| flex_heptane.wsp2 D2 (data-informed (train)) | 46.1 | 63.0 | 0.71 | 0.97 | 8.25 |
| flex_heptane.wsp2 D2-law (hand-written reference) | 105.1 | 134.3 | 0.61 | 0.79 | 7.69 |

Paired difference to flex_heptane.wsp2 D2 (data-informed (train)) (other − reference, µm, 95 % CI):

- flex_heptane.wsp2 D2-law (hand-written reference): CRPS +59.0 [+10.4, +134.2] (39 cases, 7 clusters)

**Train** — µm. The mean is dominated by a few artefact runs (soot shell, gas bubble, droplet leaving and re-entering the view); the median and the mean without the worst 5 % of cases are given beside it.

| release | cases | refused | CRPS mean | CRPS median | CRPS mean, worst 5 % dropped | MAE of median | coverage 50 % | coverage 90 % |
|---|---|---|---|---|---|---|---|---|
| flex_heptane.wsp2 D2 (data-informed (train)) | 125 | 9 | 46.2 | 21.8 | 22.6 | 54.1 | 0.76 | 0.93 |
| flex_heptane.wsp2 D2-law (hand-written reference) | 125 | 9 | 75.4 | 34.9 | 46.9 | 97.2 | 0.57 | 0.82 |

## FLEX methanol, fourth generation (PSI-69)

Cohort `sha256:9e31b1aa6b56b2086aac85776d27778bb1680a83fcdfc543dad62e0385da637e` · split `sha256:660b7d9ede629b3009b4b761ea9e5875560719566e3dea21112f607cddd49894`.

| release | id | forward | nonzero / authored |
|---|---|---|---|
| flex_methanol.wsp3 D4 — data-informed (train) | `sha256:2dab61e41007f1500095afaeacc10f83a0237956b349f922943c6403cecaa2b1` | `sha256:37a6e557…` | 87 / 391 |
| flex_methanol.wsp3 D5 — data-informed (train); the third-generation numbers plus the broad component only; written after the first candidate failed on validation | `sha256:bfaaf4962e23e56f4a3a695f7dd1e408b198369dd5b26ca9fbe13f0313dae46a` | `sha256:37a6e557…` | 80 / 391 |
| flex_methanol.wsp3 carry of D3 — the third-generation numbers on the fourth forward | `sha256:637921fa3dba9addc9a2730b6eba463a6ed4d7848014e282f8844ff4fdb01cf6` | `sha256:37a6e557…` | 74 / 391 |
| flex_methanol.wsp3 D2-law — hand-written reference | `sha256:38d9059ceea276daa8547b8d177d123ceb2fe1af83deaecbf55b612242dbcee1` | `sha256:37a6e557…` | 34 / 391 |

**Validation** — 59 common cases; CRPS and MAE in µm.

| release | CRPS | MAE of median | coverage 50 % | coverage 90 % | lpd |
|---|---|---|---|---|---|
| flex_methanol.wsp3 D5 (data-informed (train); the third-generation numbers plus the broad component only; written after the first candidate failed on validation) | 40.1 | 52.8 | 0.72 | 0.90 | 8.61 |
| flex_methanol.wsp3 carry of D3 (the third-generation numbers on the fourth forward) | 39.5 | 52.8 | 0.71 | 0.90 | 8.64 |
| flex_methanol.wsp3 D2-law (hand-written reference) | 91.1 | 113.6 | 0.55 | 0.76 | 7.81 |
| flex_methanol.wsp3 D4 (data-informed (train)) | 44.8 | 62.9 | 0.61 | 0.82 | 8.69 |

Paired difference to flex_methanol.wsp3 D4 (data-informed (train)) (other − reference, µm, 95 % CI):

- flex_methanol.wsp3 D5 (data-informed (train); the third-generation numbers plus the broad component only; written after the first candidate failed on validation): CRPS -4.7 [-14.8, +1.9] (59 cases, 9 clusters)
- flex_methanol.wsp3 carry of D3 (the third-generation numbers on the fourth forward): CRPS -5.3 [-15.4, +1.4] (59 cases, 9 clusters)
- flex_methanol.wsp3 D2-law (hand-written reference): CRPS +46.2 [+19.6, +76.9] (59 cases, 9 clusters)

**Train** — µm. The mean is dominated by a few artefact runs (soot shell, gas bubble, droplet leaving and re-entering the view); the median and the mean without the worst 5 % of cases are given beside it.

| release | cases | refused | CRPS mean | CRPS median | CRPS mean, worst 5 % dropped | MAE of median | coverage 50 % | coverage 90 % |
|---|---|---|---|---|---|---|---|---|
| flex_methanol.wsp3 D4 (data-informed (train)) | 182 | 3 | 26.9 | 16.4 | 17.6 | 37.1 | 0.67 | 0.93 |
| flex_methanol.wsp3 D5 (data-informed (train); the third-generation numbers plus the broad component only; written after the first candidate failed on validation) | 182 | 3 | 28.7 | 17.7 | 19.4 | 37.7 | 0.78 | 0.96 |
| flex_methanol.wsp3 carry of D3 (the third-generation numbers on the fourth forward) | 182 | 3 | 29.9 | 17.7 | 19.2 | 37.8 | 0.77 | 0.94 |
| flex_methanol.wsp3 D2-law (hand-written reference) | 182 | 3 | 87.2 | 37.9 | 59.7 | 112.9 | 0.49 | 0.77 |

## FLEX n-heptane, fourth generation (PSI-69)

Cohort `sha256:8ffa1875d0c963fe3b3ce6886ba7197a7b0c8675ffd837bbf5139049c1655797` · split `sha256:c43bbe3580ffce2a33c400d5aea73b3330cf721133f925c2963579dee6d32001`.

| release | id | forward | nonzero / authored |
|---|---|---|---|
| flex_heptane.wsp3 D3 — data-informed (train) | `sha256:e032ba3fc186d897c662ac9331b8d9b2b38e2b2dc6265cc7d4b41afa85abcc55` | `sha256:37a6e557…` | 107 / 588 |
| flex_heptane.wsp3 carry of D2 — the third-generation numbers on the fourth forward | `sha256:f312235d994181ff68540911198cbbab91ba54079f156264111ab2350d0c2c29` | `sha256:37a6e557…` | 99 / 588 |
| flex_heptane.wsp3 D2-law — hand-written reference | `sha256:24d751a0c57f18e838e799a2236fc5ffd12b8543cfc9606daaab50fc6f5825a1` | `sha256:37a6e557…` | 39 / 588 |

**Validation** — 39 common cases; CRPS and MAE in µm.

| release | CRPS | MAE of median | coverage 50 % | coverage 90 % | lpd |
|---|---|---|---|---|---|
| flex_heptane.wsp3 D2-law (hand-written reference) | 105.1 | 134.3 | 0.61 | 0.79 | 7.69 |
| flex_heptane.wsp3 carry of D2 (the third-generation numbers on the fourth forward) | 46.1 | 63.0 | 0.71 | 0.97 | 8.25 |
| flex_heptane.wsp3 D3 (data-informed (train)) | 57.6 | 79.7 | 0.50 | 0.87 | 7.88 |

Paired difference to flex_heptane.wsp3 D3 (data-informed (train)) (other − reference, µm, 95 % CI):

- flex_heptane.wsp3 D2-law (hand-written reference): CRPS +47.4 [+10.4, +101.0] (39 cases, 7 clusters)
- flex_heptane.wsp3 carry of D2 (the third-generation numbers on the fourth forward): CRPS -11.6 [-33.0, +0.9] (39 cases, 7 clusters)

**Train** — µm. The mean is dominated by a few artefact runs (soot shell, gas bubble, droplet leaving and re-entering the view); the median and the mean without the worst 5 % of cases are given beside it.

| release | cases | refused | CRPS mean | CRPS median | CRPS mean, worst 5 % dropped | MAE of median | coverage 50 % | coverage 90 % |
|---|---|---|---|---|---|---|---|---|
| flex_heptane.wsp3 D3 (data-informed (train)) | 125 | 9 | 39.6 | 19.1 | 21.1 | 53.2 | 0.62 | 0.91 |
| flex_heptane.wsp3 carry of D2 (the third-generation numbers on the fourth forward) | 125 | 9 | 46.2 | 21.8 | 22.6 | 54.1 | 0.76 | 0.93 |
| flex_heptane.wsp3 D2-law (hand-written reference) | 125 | 9 | 75.4 | 34.9 | 46.9 | 97.2 | 0.57 | 0.82 |

## ACME normal flames (PSI-159)

Cohort `sha256:92fb4728e88ed5175bd47da77707d042a0f046abca1bd8df351fd1d69fea4701` · split `sha256:c77bc5baee98d3e3494de144fa3f98ce0dee904efa35d5f7c0b48b91dbc82d14`.

| release | id | forward | nonzero / authored |
|---|---|---|---|
| acme_normal.wsp4 D1 — data-informed (train); wsp2 D3 and wsp3 D1 with equal weights; written after both members' aggregate validation scores were seen | `sha256:dfdaa65d9171c063a5ff5d05820096aac1acee32e3821609c519173be2537c6b` | `sha256:d3f06357…` | 111 / 854 |
| acme_normal.wsp4 baseline pair — the two reference baselines with equal weights, hand-written reference | `sha256:d7bd7e2c3aed4b843cdf30f95a2bda3d5d0e7396293ca844dab7dd31a583796a` | `sha256:d3f06357…` | 43 / 854 |
| acme_normal.wsp3 D1 — data-informed (train; published theory) | `sha256:af1d3f4a83bc7469aed313b3a5f7cfe2b9be2d9dc5f91a5102b520af3fde094e` | `sha256:bf1ec6a8…` | 55 / 499 |
| acme_normal.wsp3 theory — published constants only, hand-written reference | `sha256:3d8d3b7a6e5cfbb035060711fa0cdaf392755936b42da55d38c6ac5df46115ad` | `sha256:bf1ec6a8…` | 25 / 499 |
| acme_normal.constant_baseline — fitted on train | `sha256:c2ca9d6961c8d69b2d9df77b5034fc6b7bc24c34935928f9d1623978fa84ce11` | — | fitted |
| acme_normal.piecewise_baseline — fitted on train | `sha256:948952618a0e18e096d291aec98d184affe02c6681c0f6e879538c5415a8c222` | — | fitted |
| acme_normal.linear_baseline — fitted on train | `sha256:cb255ed369e2c26776b013c30bb51ff1840457b773fcfad36cb7f500d429346b` | — | fitted |
| acme_normal.hazard — fitted on train | `sha256:49a9a3e3051a37eb779451130ac26f166766a93d6ad8253735fbcb9712c04f13` | — | fitted |

**Validation** — 84 common cases; integrated Brier score over 6–120 s (lower is better).

| release | extinction | operator termination |
|---|---|---|
| acme_normal.wsp3 D1 (data-informed (train; published theory)) | 0.1629 | 0.0997 |
| acme_normal.wsp3 theory (published constants only, hand-written reference) | 0.2064 | 0.1143 |
| acme_normal.hazard (fitted on train) | 0.1873 | 0.0933 |
| acme_normal.wsp4 D1 (data-informed (train); wsp2 D3 and wsp3 D1 with equal weights; written after both members' aggregate validation scores were seen) | 0.1553 | 0.0984 |
| acme_normal.piecewise_baseline (fitted on train) | 0.2168 | 0.1001 |
| acme_normal.constant_baseline (fitted on train) | 0.2137 | 0.0993 |
| acme_normal.linear_baseline (fitted on train) | 0.1854 | 0.0928 |
| acme_normal.wsp4 baseline pair (the two reference baselines with equal weights, hand-written reference) | 0.1809 | 0.0994 |

**Train** — integrated Brier score.

| release | cases | refused | extinction | operator termination |
|---|---|---|---|---|
| acme_normal.wsp4 D1 (data-informed (train); wsp2 D3 and wsp3 D1 with equal weights; written after both members' aggregate validation scores were seen) | 210 | 0 | 0.0517 | 0.0275 |
| acme_normal.wsp4 baseline pair (the two reference baselines with equal weights, hand-written reference) | 210 | 0 | 0.1507 | 0.0681 |
| acme_normal.wsp3 D1 (data-informed (train; published theory)) | 210 | 0 | 0.0631 | 0.0237 |
| acme_normal.wsp3 theory (published constants only, hand-written reference) | 210 | 0 | 0.1411 | 0.0751 |
| acme_normal.constant_baseline (fitted on train) | 201 | 9 | 0.2253 | 0.0715 |
| acme_normal.piecewise_baseline (fitted on train) | 201 | 9 | 0.2241 | 0.0700 |
| acme_normal.linear_baseline (fitted on train) | 201 | 9 | 0.1082 | 0.0391 |
| acme_normal.hazard (fitted on train) | 201 | 9 | 0.1113 | 0.0439 |

## ACME inverse flames (PSI-159)

Cohort `sha256:ed3e9beb717728fc4d371bccfc7896a940d38deea28502728be977f9fb79e45f` · split `sha256:747032c921c8c8f2b976fceb6bda8c90e4c5e18c761fdbd8dafeb5bc44e2b598`.

| release | id | forward | nonzero / authored |
|---|---|---|---|
| acme_inverse.wsp4 D1 — data-informed (train); wsp2 D1 and wsp3 D1 with equal weights; written after both members' aggregate validation scores were seen | `sha256:c7c9b1d3e079c758243950e2f65e2014e06d98d1cb9ad0bd49756d19f64e9cbc` | `sha256:d3f06357…` | 98 / 854 |
| acme_inverse.wsp4 baseline pair — the two reference baselines with equal weights, hand-written reference | `sha256:b971945ec13fbd4de33f07df26a6020849301b4f451af554b32569bd7ec05003` | `sha256:d3f06357…` | 43 / 854 |
| acme_inverse.wsp3 D1 — data-informed (train; published theory) | `sha256:a43fdbcf5c73206a5d9d442a182d5e7e5c4a407d53e64b44231c014cbb00c1de` | `sha256:bf1ec6a8…` | 54 / 499 |
| acme_inverse.wsp3 theory — published constants only, hand-written reference | `sha256:d0a1366034b9f10aa91633c670071bb8029bcf7d9c945608f2cd1d8f5a8b184b` | `sha256:bf1ec6a8…` | 25 / 499 |
| acme_inverse.constant_baseline — fitted on train | `sha256:c2b9870860fe7b82ea3455a14da959d36c7ce2e07a70dffd87b55cfbaa990ace` | — | fitted |
| acme_inverse.piecewise_baseline — fitted on train | `sha256:a60a58f764abf9e4ddd8aa3dfdda6476748611b6d44139364ab8322a9b519617` | — | fitted |
| acme_inverse.linear_baseline — fitted on train | `sha256:cb16419ceeff23e5430fdae5a29ed8f7aadf16f55a17706cce2ef38b0730efe4` | — | fitted |
| acme_inverse.hazard — fitted on train | `sha256:b70287b9544139b83f760cfcc8e9b9a04361acd69edd0af690891fd07acc876c` | — | fitted |

**Validation** — 49 common cases; integrated Brier score over 6–120 s (lower is better).

| release | extinction | operator termination |
|---|---|---|
| acme_inverse.wsp3 D1 (data-informed (train; published theory)) | 0.1373 | 0.0879 |
| acme_inverse.wsp4 D1 (data-informed (train); wsp2 D1 and wsp3 D1 with equal weights; written after both members' aggregate validation scores were seen) | 0.1416 | 0.0918 |
| acme_inverse.wsp4 baseline pair (the two reference baselines with equal weights, hand-written reference) | 0.2548 | 0.1548 |
| acme_inverse.piecewise_baseline (fitted on train) | 0.2512 | 0.1265 |
| acme_inverse.hazard (fitted on train) | 0.1426 | 0.1178 |
| acme_inverse.constant_baseline (fitted on train) | 0.2521 | 0.1532 |
| acme_inverse.linear_baseline (fitted on train) | 0.1731 | 0.1314 |
| acme_inverse.wsp3 theory (published constants only, hand-written reference) | 0.3462 | 0.1933 |

**Train** — integrated Brier score.

| release | cases | refused | extinction | operator termination |
|---|---|---|---|---|
| acme_inverse.wsp4 D1 (data-informed (train); wsp2 D1 and wsp3 D1 with equal weights; written after both members' aggregate validation scores were seen) | 204 | 0 | 0.0410 | 0.0612 |
| acme_inverse.wsp4 baseline pair (the two reference baselines with equal weights, hand-written reference) | 204 | 0 | 0.3148 | 0.1975 |
| acme_inverse.wsp3 D1 (data-informed (train; published theory)) | 204 | 0 | 0.0357 | 0.0639 |
| acme_inverse.wsp3 theory (published constants only, hand-written reference) | 204 | 0 | 0.4883 | 0.2466 |
| acme_inverse.constant_baseline (fitted on train) | 192 | 12 | 0.2598 | 0.1622 |
| acme_inverse.piecewise_baseline (fitted on train) | 192 | 12 | 0.2420 | 0.1443 |
| acme_inverse.linear_baseline (fitted on train) | 192 | 12 | 0.0639 | 0.0736 |
| acme_inverse.hazard (fitted on train) | 192 | 12 | 0.0677 | 0.0758 |

## BASS (PSI-25)

**Validation** — 12 burns with cases of 17 tracks, 40 cases (not scoreable: 3 fewer than 6 flame stills, 1 flow unreadable, 1 position unit not sample width). Sample widths.

| release | CRPS | abs. error of median | coverage 50 % | coverage 90 % | track ended (Brier) | spread or not (Brier) |
|---|---|---|---|---|---|---|
| bass_front.wsp D1 (data-informed (train; published theory)) | 0.117 | 0.162 | 0.66 | 0.90 | 0.178 | 0.170 |
| bass_front.wsp D2 (D1 without the oxygen quench boundary; written after D1's aggregate validation scores, not blind) | 0.117 | 0.162 | 0.66 | 0.90 | 0.183 | 0.121 |
| bass_front.wsp persistence (hand-written reference) | 0.234 | 0.315 | 0.34 | 0.62 | 0.277 | 0.125 |

Paired difference to bass_front.wsp D1 (data-informed (train; published theory)) (other − reference, sample widths, 95 % bootstrap over burns): bass_front.wsp D2 (D1 without the oxygen quench boundary; written after D1's aggregate validation scores, not blind): CRPS +0.000 [+0.000, +0.000] (40 cases, 12 burns; reference better in 0).

Paired difference to bass_front.wsp D1 (data-informed (train; published theory)) (other − reference, sample widths, 95 % bootstrap over burns): bass_front.wsp persistence (hand-written reference): CRPS +0.117 [+0.051, +0.182] (40 cases, 12 burns; reference better in 10).

By cutoff (CRPS, bass_front.wsp D1): 5 s 0.182, 10 s 0.123, 20 s 0.094, 40 s 0.040. By lead: 0-10 s 0.067, 10-30 s 0.148, 30-60 s 0.167.

**Train** — 3 burns with cases of 4 tracks, 12 cases (not scoreable: 1 fewer than 6 flame stills). Sample widths.

| release | CRPS | abs. error of median | coverage 50 % | coverage 90 % | track ended (Brier) | spread or not (Brier) |
|---|---|---|---|---|---|---|
| bass_front.wsp D1 (data-informed (train; published theory)) | 0.076 | 0.088 | 0.83 | 0.97 | 0.003 | 0.003 |
| bass_front.wsp D2 (D1 without the oxygen quench boundary; written after D1's aggregate validation scores, not blind) | 0.076 | 0.088 | 0.83 | 0.97 | 0.003 | 0.241 |
| bass_front.wsp persistence (hand-written reference) | 0.242 | 0.319 | 0.61 | 0.74 | 0.043 | 0.250 |

Paired difference to bass_front.wsp D1 (data-informed (train; published theory)) (other − reference, sample widths, 95 % bootstrap over burns): bass_front.wsp D2 (D1 without the oxygen quench boundary; written after D1's aggregate validation scores, not blind): CRPS +0.000 [+0.000, +0.000] (12 cases, 3 burns; reference better in 0).

Paired difference to bass_front.wsp D1 (data-informed (train; published theory)) (other − reference, sample widths, 95 % bootstrap over burns): bass_front.wsp persistence (hand-written reference): CRPS +0.166 [+0.035, +0.359] (12 cases, 3 burns; reference better in 3).

By cutoff (CRPS, bass_front.wsp D1): 5 s 0.089, 10 s 0.096, 20 s 0.087, 40 s 0.033. By lead: 0-10 s 0.043, 10-30 s 0.060, 30-60 s 0.098.
