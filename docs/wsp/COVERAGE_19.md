# The 19 flight investigations: the hand-written model of each and what it scored

15 November 2026. One row per NASA flight combustion investigation in the harness. Every model is a
literal release (each number typed by an author model, `trained_on = not_trained`, experimental).
"Zero-shot" means typed before any outcome of that investigation was seen; "data-informed" means
revised from train groups; any other exposure is said in the row. Scores are lower-is-better and are
given beside the baseline they were registered against. Small sets are small: read the counts.
Details and rules: [PROTOCOL.md](PROTOCOL.md) sections 12 and 13, [RESULTS_FINAL.md](RESULTS_FINAL.md),
[RESULTS_GEN6.md](RESULTS_GEN6.md), [feedback/](feedback/).

## Droplets

| Investigation | Model | How written | Held data it was scored on | Score against baseline |
|---|---|---|---|---|
| FLEX (PSI-69), methanol | `flex_methanol.wsp2` D3 | data-informed (train) | sealed test partition: 65 cases, 9 sessions | diameter CRPS **18.5 µm** against 31.6 for the d-squared law; difference −13.1 [−19.0, −7.7] |
| FLEX (PSI-69), n-heptane | `flex_heptane.wsp2` D2 | data-informed (train) | sealed test partition: 43 cases, 7 sessions | **32.2 µm** against 54.0; difference −21.8 [−47.1, −4.9] |
| FLEX-2 (PSI-68) | the FLEX n-heptane model, unchanged; zero-shot candidates for n-decane, n-octane, iso-octane | transfer (heptane); zero-shot (other fuels) | 8 n-heptane tests on 2 train-split days, measured from NASA's frames by this lane | n-heptane **20.9 µm** against 32.6 for the d-squared law, lower on both days. n-decane candidate 45.8 against 38.8 for the heptane law: undecided. Test days: 2 droplets, an anecdote |
| CFI (PSI-39) | `flex_dodecane.wsp7` W0 | zero-shot | 7 n-dodecane tests on 5 days; test day: 1 droplet | 239.9 µm against 259.1 for the heptane law, −19.3 [−85.6, +58.5]: no conclusion. On the one test-day droplet 14.8 against 39.7 |

## Spherical burner flames

| Investigation | Model | How written | Held data it was scored on | Score against baseline |
|---|---|---|---|---|
| CFI-G (PSI-159), normal flames | `acme_normal.wsp12` F2 | data-informed (train; train days of Flame Design) | sealed test partition: 60 cases, 2 sessions | integrated Brier, extinction / termination: **0.0414 / 0.0494** against 0.0456 / 0.0558 for the model it replaced. Since replaced by `acme_normal.wsp13` F3 stage 1 (PROTOCOL 13.5), which scores 0.1536 / 0.0956 against 0.1537 / 0.0961 for F2 on CFI-G validation; the sealed test partition was scored once, with F2, and is not scored again |
| CFI-G (PSI-159), inverse flames | `acme_inverse.wsp12` F2 | data-informed (train); ethylene zero-shot | sealed test partition, 3 sessions | **0.0998 / 0.1103**, equal to the model it replaced on ethane, propane and butane |
| Flame Design (PSI-10) | the two F2 models, ethylene from the typed fuel table | fuel table zero-shot, ethylene row revised once after a blind run | test days | normal 0.0681 / 0.1451 against 0.0803 / 0.1412 with ethylene treated as ethane; inverse on validation days 0.108 against 0.147 |
| s-Flame (PSI-23) | `acme_normal.wsp13` F3 stage 1 | data-informed (train days of s-Flame, CFI-G and Flame Design): burner limit as an input of the test, corrected heating law, one step on methane | validation days: 46 cases, 16 tests, 2 days (the author had seen F2's aggregates on them) | sum **0.205** against 0.262 for F2 and 0.279 for the incidence baseline without the planned stop; 0.205 against 0.370 and 0.279 with it. The first blind look of F2 was undecided. Helium, hydrogen and blends still refused (typed zero-shot, not yet assembled) |

## Fabric and solids

| Investigation | Model | How written | Held data it was scored on | Score against baseline |
|---|---|---|---|---|
| BASS-II (PSI-25), fabric | `bass_front.wsp` D1 | data-informed (four train burns) | validation: 12 burns measured blind; test: 2 burns | flame-tip CRPS **0.117 sample widths** against 0.234 for persistence; its quench boundary fails (0.170 against 0.125 for a constant) |
| Saffire I (PSI-98) | D1, unchanged | blind transfer from 2 cm strips to a 40.6 cm sheet | 1 concurrent sheet, 4 cases | with Saffire III: **0.058** against 0.250 for persistence, lower on both sheets |
| Saffire II (PSI-99) | D1, unchanged | blind transfer | 2 strips of 5 cm; the measured tip is unreliable | 0.654 against 0.852; reported, decides nothing |
| Saffire III (PSI-100) | D1, unchanged | blind transfer | 1 concurrent sheet, 4 cases | see Saffire I |
| BASS and BASS-II plastics (PSI-26, PSI-25) | `bass1` D1 | zero-shot to the project's files (four published papers on these burns read), then one step from train days (thick slabs conduct heat inward) | validation: 41 burns on 12 days, 28 answered for the quench speed, 22 for "kept burning" | quench speed 0.507 cm/s against 0.230 for constant quantiles: still **loses** (zero-shot parent 0.517). Kept burning after the igniter (given a flame was established): **0.086** against 0.120, beats every baseline |

## Burner rig, jets, aerosols

| Investigation | Model | How written | Held data it was scored on | Score against baseline |
|---|---|---|---|---|
| BRE (PSI-20) | `bre` W0 | zero-shot | validation: 87 cases answered, 3 refused, 4 days | extinction 0.2755 against 0.2274 for the incidence: **loses**. Operator termination 0.1383 against 0.1799 (beats the incidence; the planned-stop baseline scores 0.0924). One revision from train days was tried and did not replace it (PROTOCOL 13.3) |
| CLD Flame (PSI-21) | `cld` D1 | typed with train aggregates seen, then one step from train days (methane has no stable lifted flame) | validation: 61 test points answered, 11 refused, 6 days | flame lost 0.514 against 0.478 for the incidence: still **loses** (zero-shot parent 0.593). Radiometer level **0.100** against 0.263 and 0.174, beats every baseline |
| E-FIELD Flames (PSI-22) | `efield` W0 | typed with train aggregates seen | validation: 36 tests, 1,289 voltage levels, 4 days | ion current 0.546 µA against 0.620 for a constant (the space-charge baseline scores 0.374). Flame out under the field **0.0753** against 0.0884, beats every baseline |
| SLICE (PSI-106) | `slice` W0 | zero-shot | validation: one methane day, 34 lines; the 41 lines of the ethylene day are refused | blow-off 0.155 against 0.161 for the incidence (by-tube baseline 0.149). Lifting 0.089 against 0.056: **loses**. One revision from train days was tried and did not replace it (PROTOCOL 13.3) |
| SPICE (PSI-107) | `spice` W0 | zero-shot (a second candidate built on the published fit is reported beside) | validation: 7 smoke points, 2 days | smoke-point length 5.86 mm against 6.10 for a constant (per-fuel baseline 5.04). The candidate built on the investigators' published fit scores 2.91; it is not blind |
| SAME-R (PSI-101) | `samer` W0 | zero-shot | validation: 23 rows answered, 1 refused, 3 days | particle diameter **0.167** against 0.393 and 0.339; detector at full scale **0.228** against 0.411 and 0.247: beats every baseline on both. One revision from train days was tried and did not replace it (PROTOCOL 13.3) |
| SAME (PSI-102) | `same` W0 | zero-shot | the whole investigation, held: 45 rows | particle diameter **0.184** against 0.220 and 0.192 (beats every baseline); detector at full scale 0.059 against 0.205 (per-material baseline 0.055) |
| DAFT (PSI-47) | `daft` W0 | zero-shot | the whole investigation, held: 53 readings, 2 groups | number concentration **0.777** against 2.678 for the one baseline (smoke number per mass from SAME-R) |

## After the final run (PROTOCOL 13.8; typed with every partition seen; in-sample; no held-out score)

The owner asked for the weak answers to be revised although the test days were spent. Three were
looked at, once each.

| Answer | What was found | Result | Served |
|---|---|---|---|
| Plastics quench speed | the worst test burn was mis-coded (pot settings and fan displays in swapped cells), a general coding fault with it (a quench coded without checking the flow was lowered); the other was a slab in the wrong class, separated by width | `bass5`: 0.114 / 0.108 / 0.126 on train, validation and test days against 0.671 / 0.547 / 0.916 for a constant, on the corrected coding | yes: the harness answers the quench speed again |
| s-Flame helium and hydrogen | no change to the helium row; 11 of 26 test-day tests ramp their flow after ignition and were scored as constant-flow tests | `f4-hydrogen`: helium 0.111 against 0.330 on validation days and 0.242 against 0.414 on test days with the ramped tests excluded; hydrogen on train days 0.314 against 0.909 | yes: helium and hydrogen answered; blends and flow-ramped tests refused |
| BASS-II fabric quench boundary | the burns bracket the boundary without placing it | one typed number, checked once: does not beat a constant | no: the question stays refused; "keeps burning" comes from the solids model |

## Final run on the unread test days (PROTOCOL 13.7; run once, 15 November)

| Investigation | Target | Cases | Model | Best baseline | Result |
|---|---|---|---|---|---|
| BRE | extinction | 95 | 0.139 | 0.222 | holds |
| E-FIELD | ion current, µA | 788 levels | 0.319 | 0.575 | holds |
| E-FIELD | flame out in the window | 39 | 0.055 | 0.133 | holds |
| SPICE | smoke-point length, mm | 6 | 4.18 | 10.33 | holds |
| SPICE | flame length given the flow, mm | 6 | 1.18 | 2.56 | holds |
| SAME-R | particle diameter | 33 | 0.167 | 0.256 | holds |
| SAME-R | detector at full scale | 33 | 0.014 | 0.019 | holds |
| CLD Flame | flame lost, time course | 63 | 0.124 (newest law); 0.073 (first revision) | 0.102 | newest law does not hold; the first revision does, and is the one served (chosen after the run) |
| CLD Flame | radiometer | 63 | 0.115 | 0.144 | holds |
| Plastics and fabrics | kept burning | 42 | 0.048 | 0.071 | holds |
| Plastics | quench speed | 38 | 2.91 by day, 1.54 by case | 2.70 by day, 2.11 by case | does not hold by the registered day mean |
| SLICE | blow-off; lifting | 31 lines | 0.002; 0.004 | 0.000 | no event on the answered lines: says little |
| s-Flame, helium | sum | 49 | 0.708 | 0.416 | the helium fix does not hold; helium stays refused |

## Second laws on the development days (in-sample; PROTOCOL 13.4 to 13.6)

After the blind looks the validation days of the new models became study data. These numbers are
train and validation days together and are not a test; the unread test days judge.

| Investigation | Second law | Target | Before | After | Best baseline | Days lower | Kept |
|---|---|---|---|---|---|---|---|
| Plastics | `bass2`: rods as rods, a thickness input | quench speed | 0.8135 | 0.4505 | 0.7079 | 12 of 14 | yes |
| SLICE | `slice2`: lifting as a noted window between two limits | lifting | 0.1474 | 0.0333 | 0.0293 | 8 of 8 | yes |
| CLD Flame | `cld2`: dilution and ramp flags | flame lost | 0.317 | 0.143 | 0.228 | 5 of 8 | no (comparator) |
| BRE | extinction read from the law's own `lasts` target | extinction | 0.2576 | 0.1955 | 0.2269 | being counted | pending |
| BRE | the same plus a pressure step | extinction | 0.2576 | 0.1800 | 0.2269 | 9 of 14 | no |
| E-FIELD | `efield2`: coflow | ion current | 0.659 | 0.721 | 0.426 | 7 of 15 | no |
| CLD Flame | C1: the loss decided by coflow speed over burning speed squared (study of the cases) | flame lost | 0.317 | 0.043 | 0.228 | 21 of 25 | yes |
| E-FIELD | E1: saturation cap, polarity of the mesh, rise as V^1.4 | ion current | 0.659 | 0.253 | 0.426 | 15 of 15 | yes |
| SPICE | P2: nozzle dependence that is not a power law; coflow floor | smoke-point length, mm | 6.51 | 2.13 | 6.63 | 9 of 9 | yes |
| Plastics | `bass3`: fan floor on the day's scale, one or two burning sides | quench speed | 0.393 | 0.131 | 0.579 | 20 of 22 | yes |
| SPICE | P3: fuel-dependent luminous tip, gradual passage from jet to coflow | flame length given the flow, mm | 1.94 | 1.25 | 1.84 | 8 of 9 | yes |
| SLICE | S2: blow-off boundary | blow-off | 0.109 | 0.058 | 0.048 | 5 of 8 | no (comparator) |
| BRE | d3: burner heat share cut, no pressure term, clock 0.42; read through `lasts` | extinction | 0.258 | 0.112 | 0.227 | 13 of 14 | yes |
| CLD Flame | `cld3`: the loss time follows the ramp | flame lost | 0.043 | 0.033 | 0.228 | 7 of 9 | yes |
| E-FIELD | `efield3`: flame-out by burner, coflow and polarity | flame out | 0.148 | 0.063 | 0.070 | 13 of 15 | yes (worse than E1 on validation days) |
| E-FIELD | `efield3` | ion current | 0.253 | 0.210 | 0.426 | 8 of 9 | yes |
| Plastics and fabrics | `bass4`: also answers SIBAL, Nomex and Ultem | kept burning, fabric burns | refused | 0.053 | 0.380 | | yes |
| SLICE | corrected tube entries (no new release) | blow-off | 0.109 | 0.046 | 0.047 | | data fix |
| s-Flame | F4: methane step; revised helium row; hydrogen row | sum, nitrogen validation cases | 0.205 | 0.082 | 0.279 | fewer than six days change | no (comparators; the test days are all helium and judge the helium row only) |
| SAME-R | S1: one starting number for every smoke, size set by smoke mass | particle diameter | 0.238 | 0.172 | 0.276 | 12 of 12 | yes |
| SAME-R | S1 | detector at full scale | 0.175 | 0.056 | 0.101 | 10 of 10 | yes |

## What is not done

- **Four targets lose to a constant baseline:** BRE extinction, the quench speed of plastics, SLICE lifting, CLD flame loss. Their authors name the cause as structural: a held-flame class for BRE, a thickness input for plastics, a window between lifting and blow-off for SLICE, a feature for tests that dilute until the flame is lost for CLD.
- **Thin evidence:** SLICE is judged on one day, SPICE on 7 smoke points, Saffire on two sheets, CFI on 7 tests; SAME and DAFT are scored whole and have no data left to judge a revision.
- **s-Flame:** F3 stage 1 replaced F2 there (PROTOCOL 13.5); it still refuses helium, hydrogen and blends, whose zero-shot lines are typed but not assembled, and the 7 low-flow methane tests whose burner reaches 250 C remain worse than a constant.
- **Droplet fuels other than methanol and n-heptane:** zero-shot candidates only; a second FLEX-2 batch was measured, but the ambient of most n-decane and n-octane tests is not in NASA's text files, so no revision could be judged.
- **Test days** of the ten investigations added in the seventh generation are unread; no final run has been made for them.
