# BRE: the third law d3 and its comparator d3m (PROTOCOL 13.6), development, in-sample

- Dossier: `docs/wsp/research/phys_bre_study.md` (sha256 `dbdced679233e2aee34d82094c1dc35da558d67cce75da205fe5ade28577d10a`), section 5.
- d3: `sha256:27e2af51340e7ba2a8ba73894abcec1c1342e8116834eda1aa7a0a5454d7b955` (`configs/models/law/bre/d3-fable`, revision of W0 `sha256:aa0eef5bc7653922c7a7838796dab6946294e89f79c0b6c500124d9a4b212b4d`): `runs/dev/gen7/study/bre_d3.wsp` (sha256 `defb7d19…3e04`) verbatim; its assignment lines are identical to the dossier's block (checked).
- d3m: `sha256:4b7d155ce91823425c6bf52c62f8d96318601ec60db13be1e3818664b55c2a9f` (`configs/models/law/bre/d3m-fable`): d3 with the methane block; as the dossier says, its `skip[5]` elements are written inside d3's `skip[0:7]` lines and `flame_end.extinction.median.W1[0:7, 5]` replaces d3's line (same form, the dossier's number); `lasts.logit.W1[0:6, 5]` is new.
- Element check: d3 equals W0 except exactly the 144 elements of its file (0 mismatches); d3m equals d3 except exactly the 16 elements of the methane block (0 mismatches).
- **Domain.** The widened lower bounds (`x.x_fuel.lo` 0.3 to 0.2, `x.burner_mass_flux.lo` 0.0008 to 0.0003) are typed tensors of the release (`x.<f>.lo`, `docs/wsp/research/law_forward.md` section 3), not part of the declaration: the declaration of `bre` is unchanged and d3 stays on slug `bre`.
- Scores: Rust forward and the lane scorer (IPCW integrated Brier, lower is better); d3 and d3m read through the `lasts` rule of `phys_bre2.md` section 2.

On the cases W0 answers (the newly admitted cases apart, below):

| | cases | W0 by `flame_end` | W0 by `lasts` | d3 by `lasts` | d3m by `lasts` | planned stop | incidence |
|---|---|---|---|---|---|---|---|
| extinction, pooled | 327 | 0.2576 | 0.1955 | 0.1121 | 0.0980 | 0.2269 | 0.2293 |
| extinction, train | 240 | 0.2511 | 0.1909 | 0.1226 | 0.1034 | 0.2320 | 0.2300 |
| extinction, validation | 87 | 0.2755 | 0.2083 | 0.0833 | 0.0833 | 0.2129 | 0.2274 |
| operator termination, pooled | 327 | 0.0730 | 0.0577 | 0.0398 | 0.0358 | 0.0699 | 0.0952 |
| operator termination, train | 240 | 0.0494 | 0.0426 | 0.0352 | 0.0297 | 0.0618 | 0.0646 |
| operator termination, validation | 87 | 0.1383 | 0.0994 | 0.0526 | 0.0526 | 0.0924 | 0.1799 |

Newly admitted cases (W0 refuses them; d3 answers them through its wider domain), shown apart:

| | cases | d3 by `lasts` | d3m by `lasts` | planned stop | incidence |
|---|---|---|---|---|---|
| extinction, pooled | 33 | 0.1288 | 0.1288 | 0.2414 | 0.2253 |
| extinction, train | 30 | 0.1388 | 0.1388 | 0.2216 | 0.2203 |
| extinction, validation | 3 | 0.0290 | 0.0290 | 0.4398 | 0.2760 |
| operator termination, pooled | 33 | 0.0643 | 0.0643 | 0.1460 | 0.1545 |
| operator termination, train | 30 | 0.0695 | 0.0695 | 0.1280 | 0.1677 |
| operator termination, validation | 3 | 0.0129 | 0.0129 | 0.3265 | 0.0223 |

Extinction by day (cases W0 answers):

| day | W0 flame_end | W0 lasts | d3 lasts | d3m lasts | planned stop |
|---|---|---|---|---|---|
| 19039 | 0.0582 | 0.0806 | 0.0466 | 0.0466 | 0.2365 |
| 19051 | 0.1385 | 0.1708 | 0.1270 | 0.1270 | 0.1996 |
| 19056 | 0.0921 | 0.1046 | 0.0909 | 0.0909 | 0.2341 |
| 19063 | 0.0831 | 0.1288 | 0.0846 | 0.0846 | 0.1878 |
| 19066 | 0.4554 | 0.2486 | 0.1462 | 0.1462 | 0.2663 |
| 19070 | 0.3697 | 0.2378 | 0.1142 | 0.1142 | 0.2595 |
| 19087 | 0.4792 | 0.2503 | 0.1176 | 0.1176 | 0.2591 |
| 19094 | 0.2460 | 0.2125 | 0.1091 | 0.1091 | 0.1979 |
| 20247 | 0.2885 | 0.2187 | 0.1995 | 0.1995 | 0.2220 |
| 20261 | 0.1951 | 0.1587 | 0.1002 | 0.1002 | 0.2102 |
| 20339 | 0.2493 | 0.2632 | 0.1921 | 0.0386 | 0.2395 |
| 20346 | 0.2577 | 0.1785 | 0.0921 | 0.0921 | 0.2024 |
| 20352 | 0.4783 | 0.3065 | 0.0484 | 0.0484 | 0.2458 |
| 21028 | 0.1340 | 0.1336 | 0.1132 | 0.1132 | 0.2038 |

Days lower (extinction): d3 against W0 by `flame_end` **13 of 14**; d3 against W0 by `lasts` 14 of 14; d3m against d3 1 of 1 (days whose forecasts differ).

**d3 read two ways.** Its `flame_end` (one log-logistic cause, no held class) and its `lasts` (the yes/no read by the rule) do not give the same incidence: largest disagreement in probability over all cases, by horizon: 6 s 0.82, 10 s 0.88, 20 s 0.94, 30 s 0.95, 60 s 0.95, 120 s 0.95, 210 s 0.95 (extinction); the operator-termination rows differ by up to 0.95. The two readings disagree most for flames the `lasts` target holds and the clock, which has no mass at infinity, puts out; the registered reading of d3 is `lasts`.

**Reading (13.6):** d3 by `lasts`: extinction pooled 0.1121 against W0 by `flame_end` 0.2576 (best baseline 0.2269), lower on 13 of 14 days; operator termination 0.0398 against 0.0730: **kept** (it reproduces the what-if run C3, 0.1121, through the Rust forward). d3m (comparator): 0.0980 pooled (C4 logged 0.0980), lower than d3 on 1 of 1 days: fewer than six days change (the methane day), so by 13.6 it is carried as a comparator, not as the model.
