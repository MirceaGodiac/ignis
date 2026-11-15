# Frozen TRAIN results: Astra versus adopted Fable

Protocol committed before scoring: `39f39dfb7bddcafaa02e6285e0a897badc2c201a`.
All ten candidates are **data-informed**, not zero-shot. No model or weight changed.

**No candidate establishes an overall improvement.** Nine have worse primary TRAIN means. The joint BASS candidate has lower CRPS, median absolute error and track-ended Brier, but all scored tracks come from one test day. Its improvement is uncertain across sessions.

## Primary scores

Equal-weight session/split-cluster means on identical paired cases; lower is better. FLEX CRPS is in micrometres; BASS CRPS in sample widths. ACME numbers are integrated IPCW Brier contributions, averaged equally across clusters, not case-weighted headline Brier scores. Flow termination is the scorer category `operator_termination`.

| Family / metric | Fable | Astra R1 | Joint Astra |
|---|---:|---:|---:|
| flex_methanol / crps | 48.446566 | 48.476691 | 48.462140 |
| flex_heptane / crps | 51.084312 | 51.211739 | 51.140260 |
| acme_normal / ibs_contrib[extinction] | 0.063131 | 0.064069 | 0.063592 |
| acme_normal / ibs_contrib[operator_termination] | 0.028018 | 0.028356 | 0.028181 |
| acme_inverse / ibs_contrib[extinction] | 0.021342 | 0.022030 | 0.021674 |
| acme_inverse / ibs_contrib[operator_termination] | 0.095747 | 0.095755 | 0.095748 |
| bass_front / crps | 0.076101 | 0.080217 | 0.073439 |

## Paired differences and frozen verdicts

Positive difference means Astra is worse. Intervals are descriptive 95% cluster bootstrap intervals, not multiplicity-adjusted tests or evidence of unseen-data generalisation.

| Candidate / metric | Astra minus Fable | 95% interval | Clusters | Verdict |
|---|---:|---|---:|---|
| flex_methanol r1-astra / crps | +0.030125 | [-0.043303, 0.131868] | 26 | worse primary mean |
| flex_methanol joint-astra / crps | +0.015575 | [-0.020237, 0.066087] | 26 | worse primary mean |
| flex_heptane r1-astra / crps | +0.127427 | [-0.006479, 0.387381] | 20 | worse primary mean |
| flex_heptane joint-astra / crps | +0.055948 | [-0.003485, 0.170845] | 20 | worse primary mean |
| acme_normal r1-astra / ibs_contrib[extinction] | +0.000938 | [0.000725, 0.001181] | 7 | worse primary mean |
| acme_normal r1-astra / ibs_contrib[operator_termination] | +0.000337 | [0.000037, 0.000625] | 7 | worse primary mean |
| acme_normal joint-astra / ibs_contrib[extinction] | +0.000462 | [0.000355, 0.000581] | 7 | worse primary mean |
| acme_normal joint-astra / ibs_contrib[operator_termination] | +0.000163 | [0.000014, 0.000306] | 7 | worse primary mean |
| acme_inverse r1-astra / ibs_contrib[extinction] | +0.000688 | [-0.000006, 0.001893] | 7 | worse primary mean |
| acme_inverse r1-astra / ibs_contrib[operator_termination] | +0.000008 | [-0.000445, 0.000581] | 7 | worse primary mean |
| acme_inverse joint-astra / ibs_contrib[extinction] | +0.000332 | [-0.000005, 0.000917] | 7 | worse primary mean |
| acme_inverse joint-astra / ibs_contrib[operator_termination] | +0.000001 | [-0.000223, 0.000283] | 7 | worse primary mean |
| bass_front r1-astra / crps | +0.004116 | Not estimable (one test day) | 1 | worse primary mean |
| bass_front joint-astra / crps | -0.002662 | Not estimable (one test day) | 1 | lower TRAIN mean; improvement uncertain |

## Coverage and secondary evidence

Admission and scored coverage are identical across the three releases within each family; no candidate gains by refusing a baseline case.

| Family | TRAIN runs/tracks | Scored cases per release | Refused | Other exclusions |
|---|---:|---:|---:|---|
| FLEX methanol | 68 | 182 | 3 | 15 not-at-risk; 4 forecasts without a valid continuation target |
| FLEX heptane | 49 | 125 | 9 | 13 not-at-risk |
| ACME normal | 81 | 210 | 0 | 33 not-at-risk |
| ACME inverse | 73 | 204 | 0 | 15 not-at-risk |
| BASS | 4 | 12 | 0 | One track lacks six flame stills; 3 scored tracks share one day |

Methanol refusals concern nonpositive pressure; heptane refusals concern unresolved composition conflicts. Those inputs were not repaired or zero-filled for this comparison.

Joint BASS CRPS improves from 0.076101 to 0.073439 (3.50%); median absolute error from 0.088031 to 0.069699 (20.83%); track-ended Brier from 0.003392 to 0.003206 (5.48%). Its 90% band covers 98.91%, versus Fable 97.36%: both are conservative on this tiny set. The standalone BASS candidate worsens CRPS and track-ended Brier.

FLEX primary differences are small and their intervals cross zero. Heptane median absolute error worsens for both Astra candidates, with diagnostic intervals above zero. ACME normal worsens on both primary metrics with intervals wholly above zero; ACME inverse differences are uncertain. Full secondary metrics and per-cluster differences are in `summary.json`; raw scorer diagnostics, including BASS ignition, remain in the family reports.

## Evidence and checks

`freeze.json` in the parent directory fixes inputs and source identities. `receipt.json` hashes every scored report, log, execution receipt and summary. Each family execution receipt records the executable SHA-256, command, timestamps and protocol commit. Input hashes and record metadata matched before and after execution.

Passed: scorer build, three offline `ignis-wsp-dev` tests, synthetic checks of pairing/equal-cluster aggregation/bootstrap behavior, and independent arithmetic reproduction of 24 generic and 10 BASS paired point estimates from raw per-case reports. All receipt hashes were verified. An initial parallel build failed without a detailed compiler diagnostic; rebuilding with two jobs succeeded before scoring. No scoring run failed.

No protected test or per-run validation outcomes were read. No new validation look occurred. The ten registered historical Astra candidates were compared; the newer clean-room property kernels were not scored because they lack complete registered harness releases and admitted inputs. These TRAIN results neither qualify a release nor justify replacing Fable automatically.

The canonical paired comparison is `summary.json`, keyed by release ID. The legacy scorer also emits a `paired_vs_first` object keyed by model name; standalone and joint candidates share a model name, so that legacy object cannot represent both. It was not used for these verdicts. Pairing was recomputed from the per-release case arrays. Raw logs retain Windows line endings so their recorded hashes remain valid.
