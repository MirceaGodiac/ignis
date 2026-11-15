# FLEX source-bound TRAIN comparison: registration draft

Status: not executable authorization until exact candidate/code/input identities
are recorded and the registration is committed in PROTOCOL section 11.

Question: does the new source-bound droplet forward improve liquid-diameter
prediction over the adopted model, and what fraction of the declared population
can it actually forecast? This is a development comparison, not a generalisation
claim. The adapter is data-informed integration of unchanged zero-shot physics.

Population: all TRAIN run identities in the existing frozen methanol and heptane
splits, separately reported. The candidate receives only prospective prefixes.
Use the existing cutoffs 1, 3 and 5 seconds and next-10-second diameter horizon.
Every run/cutoff remains accounted for, including unavailable prefix, missing
properties, unsupported regime, numerical refusal and absent target observations.
No outcome-based subset or post-result exclusion is allowed.

Adopted baselines:
- Methanol `flex_methanol.wsp2`,
  `sha256:c8a90dabbed6af4185de94c3cd5ac27486c19eb52c697d86b028849a37fcb654`.
- Heptane `flex_heptane.wsp2`,
  `sha256:edbad4d6395961030e0b17d9d5c4d40f2de902ddcbcbd0ec2a4f4ec9adbeedef`.

Simple baseline: persist the last valid pre-cutoff liquid-diameter observation.
No slope fitting, parameter fitting or synthetic material substitution.

Primary point comparison: predict at 100 strictly future times, cutoff + 0.1 s
through cutoff + 10 s at 0.1 s spacing. Score valid liquid-diameter observations
inside that interval using linear interpolation of predicted diameter, without
interpolating or extrapolating observations. Aggregate absolute errors first
within each case, then within run, then equal weight per experimental group.
Also report equal-run and observation-weighted MAE, raw paired differences and
counts so different weighting cannot be mistaken for a gain. This is a scoring
grid, not a physical parameter or a claim that observations are independent.

The candidate supplies a deterministic mean, not a distribution. Compare this
with the adopted forecast mean and persistence. Do not fabricate quantiles,
calibration, posterior uncertainty or CRPS for the new model.

Report all-population acceptance and scoreability for each model plus paired
scores on their intersection. A lower paired MAE with lower coverage is not an
unqualified win. If the intersection is empty, scores/differences are null and
the verdict is `not_comparable_no_candidate_predictions`, not a tie or infinity.
This outcome refutes present usability but does not measure the law's accuracy.

Input source: only already admitted source-bound packets. Missing packets must
produce explicit scientific refusals; prospective state cannot be inferred from
future diameter. Neither validation nor test outcomes are read. The final test
and any independent-group look remain outside this registration.

Before execution: complete adapter and independent tests; freeze executable,
release identities, input manifest and source hashes; reproduce the established
TRAIN scorer/baseline where comparable; commit the exact registration. Report
every deviation before examining scores and re-register changed comparisons.
