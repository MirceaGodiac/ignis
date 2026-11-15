# bre d2: second law (PROTOCOL 13.4), development, in-sample

- Dossier: `docs/wsp/research/phys_bre2.md`, sha256 `23372bd3a57c733bab552e53815b289672646350d51b14712b275aa0d07b6ebb`.
- Release: `sha256:429f47431773d02e9608a2fdfd9e24ec4af171ffe9bea98400beb44ea4f18340` (`bre` `d2-fable`, a revision of W0 `sha256:aa0eef5bc7653922c7a7838796dab6946294e89f79c0b6c500124d9a4b212b4d`; its changed lines are the D1 lines).
- Scoring-time rule (dossier section 2): extinction by h = 1 - lasts(min(h, stop)) / lasts(c), operator termination from the stop = lasts(stop) / lasts(c), c = the cutoff (not below 1 s), stop = the lane's planned stop (planned_stop_s + 0.569 - 0.796 s); ratios clipped to [0, 1]. `lasts` evaluated by the Python copy of law.rs (identical to the Rust forward to 2e-16 on bre). The same rule applied to W0 separates the effect of the rule from the pressure step.
- Check: lasts non-increasing over the horizons in every case: violations train W0 0, d2 0; validation W0 0, d2 0.

Development, in-sample, on the test x cutoff cases all three answer. IPCW integrated Brier, lower is better.

| | cases | W0 by flame_end (registered) | W0 by the lasts rule | d2 by the lasts rule | planned stop (best baseline) | incidence (simple) |
|---|---|---|---|---|---|---|
| extinction, pooled | 327 | 0.2576 | 0.1955 | 0.1800 | 0.2269 | 0.2293 |
| extinction, train | 240 | 0.2511 | 0.1909 | 0.1674 | 0.2320 | 0.2300 |
| extinction, validation | 87 | 0.2755 | 0.2083 | 0.2149 | 0.2129 | 0.2274 |
| operator termination, pooled | 327 | 0.0730 | 0.0577 | 0.0561 | 0.0699 | 0.0952 |
| operator termination, train | 240 | 0.0494 | 0.0426 | 0.0399 | 0.0618 | 0.0646 |
| operator termination, validation | 87 | 0.1383 | 0.0994 | 0.1007 | 0.0924 | 0.1799 |

Extinction by day:

| day | W0 flame_end | W0 lasts | d2 lasts | planned stop |
|---|---|---|---|---|
| 19039 | 0.0582 | 0.0806 | 0.0810 | 0.2365 |
| 19051 | 0.1385 | 0.1708 | 0.2046 | 0.1996 |
| 19056 | 0.0921 | 0.1046 | 0.1072 | 0.2341 |
| 19063 | 0.0831 | 0.1288 | 0.1285 | 0.1878 |
| 19066 | 0.4554 | 0.2486 | 0.1200 | 0.2663 |
| 19070 | 0.3697 | 0.2378 | 0.1692 | 0.2595 |
| 19087 | 0.4792 | 0.2503 | 0.1773 | 0.2591 |
| 19094 | 0.2460 | 0.2125 | 0.2456 | 0.1979 |
| 20247 | 0.2885 | 0.2187 | 0.2204 | 0.2220 |
| 20261 | 0.1951 | 0.1587 | 0.1411 | 0.2102 |
| 20339 | 0.2493 | 0.2632 | 0.2598 | 0.2395 |
| 20346 | 0.2577 | 0.1785 | 0.1797 | 0.2024 |
| 20352 | 0.4783 | 0.3065 | 0.3078 | 0.2458 |
| 21028 | 0.1340 | 0.1336 | 0.1339 | 0.2038 |

Days on which d2 (lasts rule) is lower than the parent W0 (flame_end, as registered): **9 of 14** days whose forecasts differ; lower than W0 under the same lasts rule: 6 of 14.

**Reading (13.4):** pooled extinction d2 0.1800 against the parent W0 0.2576 (best baseline 0.2269); operator termination 0.0561 against 0.0730; days lower 9 of 14: **not kept**. The rule alone (W0 by lasts) gives 0.1955; the pressure step adds -0.0155 pooled (-0.0235 train, +0.0066 validation).

## The registered W0 scored by the `lasts` rule (a candidate in its own right; PROTOCOL 13.4, 13.6)

The registered W0 numbers (`sha256:aa0eef5bc7653922c7a7838796dab6946294e89f79c0b6c500124d9a4b212b4d`), unchanged, scored by the rule of `phys_bre2.md` section 2 instead of by `flame_end`. Development, in-sample.
Rust check of the Python copy on a random sample of `lasts` evaluations (seed 13): 240 rows, admission agrees on 240, largest difference of the probability 3.3e-16.

| target | partition | cases | W0 by `flame_end` (registered) | W0 by the `lasts` rule | planned stop (best baseline) |
|---|---|---|---|---|---|
| extinction | pooled | 327 | 0.2576 | 0.1955 | 0.2269 |
| extinction | train | 240 | 0.2511 | 0.1909 | 0.2320 |
| extinction | validation | 87 | 0.2755 | 0.2083 | 0.2129 |
| operator termination | pooled | 327 | 0.0730 | 0.0577 | 0.0699 |
| operator termination | train | 240 | 0.0494 | 0.0426 | 0.0618 |
| operator termination | validation | 87 | 0.1383 | 0.0994 | 0.0924 |

Extinction by day: see the table above (columns W0 flame_end, W0 lasts).

Days on which W0 by the `lasts` rule is lower than W0 by `flame_end` (extinction): **9 of 14** days whose forecasts differ (14 days).

**Reading (13.4):** extinction pooled 0.1955 against 0.2576 (best baseline 0.2269), lower on 9 of 14 days; operator termination 0.0577 against 0.0730: **not kept** by the rule.
