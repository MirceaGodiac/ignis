# FLEX source-bound adapter: registered TRAIN result

The adapter is implemented and the registered run completed. **The candidate
has not demonstrated an improvement over the adopted baseline: it produced
zero predictions on all 351 declared cases.** Its accuracy is unmeasured.
This is a failed operational-coverage result, not successful model qualification.

Registration: PROTOCOL 11.4, committed before execution as
`35e707e0e8bc68e78c9e3ad7c6f6bfcdb44c1ef7`. Candidate integration is
**data-informed**, wrapping unchanged zero-shot physical literals. No coefficients
were changed after scoring; no validation or final-test data was accessed.

| Fuel | Declared cases | New model predictions/scored | Adopted historical-route predictions/scored | Historical-route session MAE | Persistence predictions/scored | Persistence session MAE |
|---|---:|---:|---:|---:|---:|---:|
| Methanol | 204 | 0 / 0 | 186 / 181 | 63.31 micrometres | 204 / 184 | 322.93 micrometres |
| Heptane | 147 | 0 / 0 | 125 / 124 | 60.08 micrometres | 147 / 133 | 356.42 micrometres |

These are separately supported reference scores, not paired estimates of a
candidate improvement. Historical baseline and persistence have different
scoreable-case populations. No candidate MAE, CRPS or calibration statistic
exists; absence is represented as null, never zero.

## Why all candidate cases refused

The preregistered input manifest contains no admitted source-bound packets.
No complete initial liquid state and corroborated material/transport property
bundle was available. The adapter actually executed its admission path for
every prospective prefix and returned the recorded missing-evidence refusal;
it did not simulate a fabricated material. This failure was expected and
disclosed before scoring. The run measures current admission, not the accuracy
the physical law might have with sufficient inputs.

The new property catalogue's saturated-liquid path heat capacity cannot be
substituted for isobaric heat capacity. Diameter alone does not identify liquid
temperature. Constant gas properties require defensible temperature domains.
These are missing physical inputs, not a compiler or scoring-pipeline failure.

## Baseline information-policy limitation

The strict common-prefix baseline also refused every case because its required
activity marker was absent. The separate historical route uses the unchanged
baseline activity-marker policy, which can depend on full-run records. Its
311 predictions and 305 scored cases are a labelled diagnostic, not evidence
that the new strict model lost a fair paired accuracy comparison. Persistence
uses prospective diameter only and predicted all 351 cases; 317 had targets.
The paired strict set is empty: `not_comparable_no_candidate_predictions`.

## Reproduction and checks

Exact run command and source identities are in
`tools/wsp/astra/flex_comparison_design.md`; use the registration commit above
and a new output path. The process exited 0 with 351 cases.
Raw report: `runs/dev/astra_flex_source_bound_comparison_v1/report.json`.
File SHA-256: `6f59851646efb2f449a75b69c33abc74a3fb8490e9cc756f5794622b212c7da5`.
Content identity: `sha256:fa2ad9f087f7d14b295e20aeaf38be1ac2b586b513f799dd05e377f22c18a3fb`.
Committed aggregate: `docs/wsp/feedback/astra_flex_source_bound_train_summary.json`.
All candidate refusals and per-fuel session aggregation were independently
recomputed from the case-score rows after execution.

Before registration: formatting, workspace/all-target compilation and required
Rust suites passed; nine adapter integration tests and five comparator/loader
tests passed. Independent review resolved phase-evidence and synthetic-origin
dependency defects before the final release identities were frozen.

The next accuracy experiment requires admitted physical packets or a separately
authored, explicitly assumption-based candidate and new preregistration. Current
input failure must not be concealed by relaxing domains or assigning guessed
temperatures. The all-19 objective and an accuracy comparison remain incomplete.
