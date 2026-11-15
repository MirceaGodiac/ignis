# Frozen BASS observation-operator result

15 November 2026. **Data-informed, not zero-shot.** Protocol, forward and frozen
weights before the single scoring execution. Candidate:
`bass_front.wsp4`, release
`sha256:d58751447917a425a5f3d39e2c4144fddc6151b8000874a282648603949e15e4`.

**The correction does not improve joint Astra's TRAIN CRPS.** It remains better
than Fable on the same cases, but that advantage already existed in the joint.
This is a successful implementation correction with a mixed predictive result,
not a new accuracy win or evidence that the physical quench boundary is repaired.

| Metric | Fable | Joint Astra | Corrected operator |
|---|---:|---:|---:|
| Tip CRPS, sample widths; lower is better | 0.0761009351 | 0.0734390855 | 0.0739568181 |
| Median absolute error, sample widths | 0.0880314325 | 0.0696989405 | 0.0702582022 |
| Track-ended Brier | 0.0033918283 | 0.0032058618 | 0.0031864733 |
| Central 50% coverage | 0.825530 | 0.862654 | 0.862713 |
| Central 90% coverage | 0.973649 | 0.989081 | 0.984451 |
| Scored tip cases / burns / independent days | 12 / 3 / 1 | 12 / 3 / 1 | 12 / 3 / 1 |
| Refusals | 0 | 0 | 0 |

The corrected candidate's CRPS is **2.8175% lower than Fable** and **0.7050%
higher than joint Astra**. Its track-ended Brier is slightly lower than both.
No cases were lost or gained. Band coverage is reported as observed, not called
calibrated: all central bands cover substantially more than their nominal rates
in these dependent TRAIN cases.

Four TRAIN tracks entered the population. T10 (`2014-175`) has fewer than six
flame stills and is excluded by the unchanged tip eligibility rule. The other
three burns all belong to `2014-100`. All four enter the separate no-prefix
ignition check; its Brier is `0.0033241994127758434` for both joint and corrected
Astra, with exactly identical per-track outputs. It is not another set of four
tip-forecast runs. No independent-day confidence interval is available for tip
accuracy; the raw scorer's bootstrap over runs is not used as one.

## Where the correction changes TRAIN error

Paired CRPS differences below are corrected minus comparator, in sample widths.
Negative is lower error. All four cutoffs per scored burn are retained.

| Burn | Versus Fable | Versus joint Astra |
|---|---:|---:|
| T16 | -0.002036848 | -0.000073673 |
| T5 | +0.003412885 | +0.002721578 |
| T6 | -0.007808388 | -0.001094708 |
| Equal-burn mean, the one scored day | -0.002144117 | +0.000517733 |

The largest regression relative to joint is T5 at 20 s:
`0.1026065497 -> 0.1139488866`, a difference of `+0.0113423369`.
The largest improvement is T6 at 10 s:
`0.0810925936 -> 0.0768772239`, a difference of `-0.0042153697`.
These are descriptive results of the frozen comparison; no parameter was changed
after seeing either. Full case differences are in `summary.json` and the raw
encoder diagnostics in `cases.json`.

The deterministic identity now matches modeled means to observed means at the
same still timestamps. The old physical laws and their inherited weights can
still be wrong, and a mathematical correction need not reduce error against real
observations. These scores do not identify which remaining physical assumption
is wrong. In particular, no quench-law revision, likelihood recalibration or
weight compensation is justified by this result. No release was automatically
adopted or substituted into the application's adopted model list.

## Verification and reproduction

- All 67 stored tensor entries, spanning **191 scalar positions** (180 authored
  positions including structural zeros, plus 11 fixed constants), remain
  byte-identical to joint Astra after decompression. Their weights digest is
  identical. Zero changed or newly chosen numeric positions.
- Fable and joint reproduce their historical R2 records exactly: 12 tip cases
  plus four ignition records per release, and identical per-release aggregates.
- All 166 frozen file hashes and the six initial result-receipt hashes were
  checked after scoring. A separate direct arithmetic calculation rechecked
  30 paired point values; maximum difference was `1.11e-16`.
- Required local gates passed before the protocol commit and before the results
  commit: `cargo fmt --all -- --check`,
  `cargo check --workspace --all-targets -j 2`,
  `cargo test -j 2 -p ignis-wsp -p ignis-wsp-dev -p ignis-core`.
  The five new synthetic integration tests pass. They are correctness evidence,
  not NASA qualification.

The preregistered command was
`python tools/wsp/astra/bass_operator_train.py --execute`.
It verifies the committed files and immutable scorer snapshot and refuses to
overwrite this result directory. `execution.json` stores the full invocation,
protocol commit, timestamps, executable identity and exit code. `receipt.json`
hashes the initial scored outputs; `verification.json` records the later checks.
`aggregate.json` and `cases.json` are retained without score editing.

The candidate is TRAIN-informed. The earlier aggregate-validation exposure is
disclosed in the protocol and release metadata. No new validation look, per-run
validation read, test read, weight search, post-score model revision, push or PR
was performed. Forecasts still condition on supplied future flow; track-ended
includes leaving the view and is not pure physical extinction. This small TRAIN
comparison cannot establish transfer to an unseen day.
