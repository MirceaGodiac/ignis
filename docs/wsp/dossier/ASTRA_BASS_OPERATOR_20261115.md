# BASS observation operator: frozen-weight correctness comparison

15 November 2026. **Data-informed, not zero-shot.** The owner requested a new
forward, all inherited weights frozen, a protocol committed before scoring,
comparison with adopted Fable and joint Astra on TRAIN, and a results commit.
No weight search, push or PR. This is a correctness change, not a new physical
mechanism or a proposed repair of the BASS quench boundary.

## Scope and identity

The new forward is `crates/ignis-wsp/src/front4.rs`, model `bass_front.wsp4`,
spec `bass_front-wsp-v4`, key `bass_front4`. It copies the frozen `front3.rs`
and changes only the window observation operator and its descriptions.
Registration and `wsp-dev` dispatch add this identity; no scoring function is
changed. Existing forwards, their shared numerical sources, releases, shared
records, contracts and model adoptions remain untouched.

Three releases, in this order:

1. Adopted Fable `bass_front.wsp`,
   `sha256:ac15e49f1cb60d4b4679bcdff82a07c18a4a501454428e35ac86e935e168855c`.
2. Joint Astra `bass_front.wsp3`,
   `sha256:ffbb724a068338c789a7276254d1b9e81ee18a435c760d215b21ac8336a3db37`.
3. The new `front4/window-matched-astra` release, pinned in
   `../feedback/astra-bass-operator-20261115/freeze.json`.

Every stored tensor value, including the structural zeros and fixed constants,
must equal release 2 exactly. The CLI's literal inspection is a transcription,
not a producer of coefficients: it chooses no number. A complete new-spec literal
rewrite records release 2 as its parent; the old release is never edited.
`frozen_weight_audit.json` records the identical weights digest, tensor equality,
counts and unchanged executing hashes of both comparators. **Zero newly chosen
scalar positions.** The inherited weights keep their original mixed Fable/Astra
and TRAIN-informed provenance; this is not a zero-shot model.

## Observation equation and implementation

For the actual admitted timestamps of window W, write
`M_W f = sum_i f(t_i) / n_W`. This is a sample mean, not a continuous-time mean.
The observation equation is `tip(t) = offset + E*x(t) + h(t)`.

- Tip innovation is the log of
  `[(M_L tip - M_L h) - (M_E tip - M_E h)] / [M_L x - M_E x]`,
  with the inherited positivity, minimum-span and clipping rules.
- Base innovation uses its own timestamps. The single inherited base delay
  `tau_b` is evaluated at the late base-window mean time; the predicted advance
  is `M_BL x(t-tau_b) - M_BE x(t-tau_b)`. Delay is not reestimated per sample.
- The inherited gain and channel weights combine those innovations. The future
  tip median is `M_L tip + E*(x(t)-M_L x) + h(t)-M_L h`.
  The base and front trace use the same latent-front anchor.
- The stored encoder's `x_end_w` and `h_end_w` now describe late-window mean
  components. Its `t_end_s` is still the mean-time clock used by gain, settled
  state, event-linger minimum time and drift lead. No shared DTO changes.
- The unchanged variance rule uses the matched increments `E*(x(t)-M_L x)`
  and `h(t)-M_L h`. Its coefficients, observation-noise floor and drift law are
  unchanged. This is not a derivation of covariance for averaged noise; in
  particular no `1/sqrt(n)` shrinkage or new covariance assumption is introduced.

The correction applies to every law/flow scenario, including the empirical
member of the joint. The oxygen capture law, overhang dynamics, base delay,
quench law, initial conditions, fixed integration step, uncertainty coefficients,
scenario weights and coherent first-event calculation are inherited unchanged.
Event probabilities can change because their endpoint-position input changes.

Affine modeled functions satisfy `M_W f = f(M_W t)` even for irregular samples.
Singleton windows and no-prefix forecasts reduce to the predecessor behavior.
A nonlinear path generally does not. Smooth Taylor expansion gives a leading
averaging discrepancy `f''(mean t)*Var_W(t)/2`. Fable's `W^2/(24*tau^2)`
estimate for an exponential follows for a uniformly populated smooth window;
it is not an upper bound for arbitrary still timestamps or a flow kink.
The encoder's ratio can amplify an error when its law advance is small. None
of these identities implies a lower score against a real, imperfectly modeled
flame. There is no score-selected correction size or activation threshold.

## Independent check and synthetic evidence

A fresh explicitly selected **GPT-6 Astra, high effort** checker derived the
window, delayed-base and anchoring equations from the symbolic question before
opening any file. It then compared `front4` against `front3`, the registrations
and synthetic tests, without data or scores. It found no operator correctness
issue. It noted that event-mass algebra alone was a weak assertion; the test
was changed to check absorbing event incidences. This was before NASA scoring.

Five offline synthetic tests cover unchanged tensor definitions except prose,
distinct identity, affine paths with a constant overhang, exact no-prefix and
singleton compatibility, reconstruction of a nonlinear predecessor trajectory
from irregular tip and separately timed base samples, prospective exclusion,
and the registered metric artifact with absorbing events. The nonlinear case
detects a nonzero innovation with the old approximation and zero with the
matched operator. These tests prove implementation identities, not NASA accuracy.

## Frozen TRAIN population, scoring and decisions

Only `tools/wsp/bass_optical/tracks/train/{T5,T6,T10,T16}.json`. Freeze their
exact bytes and the complete folder membership. The three scored burns belong
to TRAIN day `2014-100`; T10 is from TRAIN day `2014-175` and has fewer than six
flame stills. It remains in the ignition check but supplies no tip case. Do not
change these eligibility rules or mistake four catalogued tracks for four scored burns.

Use `front-feedback` with all three releases in one invocation. Preserve its
cutoffs 5, 10, 20, 40 s; minimum two prefix stills and three targets; targets in
`(cutoff, cutoff+60 s]`; quantile levels 0.05, 0.25, 0.5, 0.75, 0.95; event
horizons 10, 20, 30, 45, 60 s; and ignition check at 20 s with no prefix.
No targets, conditions, labels or table discrepancies are repaired.

Primary metric: tip CRPS in sample widths. Guard: track-ended Brier. Secondary:
median absolute error, central 50/90 percent coverage and the existing ignition
score. Pair by run id and cutoff. Report all counts, refusals, lost/gained pairs,
case differences and run differences. Average within case, within run, then
equally across runs of a day and across days. Here all scored runs have the
same four cutoffs. The independent-day interval is **null**, because there is
only one day; the inherited scorer's run bootstrap is retained in raw output
but is not evidence of an independent-session improvement.

Compare the new candidate separately with Fable and joint Astra. A loss of
baseline coverage prevents an unqualified win. With equal coverage, lower CRPS
and no worse track-ended Brier is called a **lower TRAIN mean**, never proof of
generalization. A guard regression is explicitly a trade-off. Equality uses
`1e-12` only as a numerical reporting tolerance, not a performance target.
No automatic adoption follows any result. Keep negative results; do not revise
the candidate after this look. Baseline and joint case records and per-release
aggregates must reproduce the committed R2 results exactly before interpreting
the new scores. Ignition should remain exactly equal to joint because it has
no observation window; verify this in the results.

Limits: the supplied future fan history remains an input; this is a forecast
conditional on that history. The event target includes leaving the field of
view and is not pure quenching. Three burns from one day cannot establish the
size or sign of a generalization gain. This change cannot fix a missing quench
mechanism, specimen properties or flow calibration.

## Execution, exposure and response to Fable

`tools/wsp/astra/bass_operator_train.py --execute` verifies committed protocol,
candidate/code hashes, the exact TRAIN membership and a frozen executable
snapshot outside Cargo targets. It scores once to a new directory, verifies
inputs again, reproduces both historical comparators and writes receipts and
the paired summary. A second run cannot overwrite the first. `freeze.json`
pins the code, literals, registry bytes, test files and historical TRAIN reports.
Builds use `CARGO_TARGET_DIR=runs/tooling/astra-train-comparison-target`.

Before the protocol commit: `cargo fmt --all -- --check`,
`cargo check --workspace --all-targets -j 2`, and
`cargo test -j 2 -p ignis-wsp -p ignis-wsp-dev -p ignis-core`.
Build `ignis-cli` and `ignis-wsp-dev` into that private target. No new candidate
NASA score is read until the protocol, candidate and implementation are committed.

Exposure: prior TRAIN reports, mandatory dossier/handoff aggregate validation
prose, and the owner's relayed Fable review were read. The last includes the
historical quench comparison and droplet validation resolution; these are
aggregate claims, not an independently rerun validation experiment. They are
recorded here, not used to choose a coefficient or this correction, which the
owner had already requested. No per-run validation, sealed folder, test outcome
or new validation look was accessed. Original parent exposure is retained.

Fable's priority objection is accepted: this operator fix is a correctness
cleanup and does not displace the more consequential prospective-input work.
The next proposed burner candidate should classify and verify the pre-ignition
temperature-limit and filament inputs, and independently resolve disputed TRAIN
labels with original values preserved. Those are **proposals, not verified new
inputs in this task**. A temperature-limit schedule or filament branch needs
source-backed physical meaning and its own registration; corrected labels are
an evaluation-data amendment, not secretly another model input. Their transfer
remains unknown. Fable's TRAIN-informed baseline advantage also means a TRAIN
loss cannot by itself refute a property-based mechanism. A later FLEX thermal
history that lacks a measured thermometer input must be disclosed as a scenario,
with its own defended probability, never presented as a liquid measurement.
