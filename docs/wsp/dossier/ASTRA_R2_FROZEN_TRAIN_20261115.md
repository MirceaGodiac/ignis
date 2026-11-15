# Astra R2: source-backed revision and frozen TRAIN retest

15 November 2026. Owner requested analysis of where Astra loses, handwritten
improvements by **GPT-6 Astra**, no weight search, protocol committed before
scoring and results committed afterwards. All revised candidates are
**data-informed, not zero-shot**. This is one revision round, not a parameter sweep.

## Analysis and literal decisions made before scores

The fixed prior evidence is commit `84f24c7`, with protocol `39f39df`.
`tools/wsp/astra/r2_loss_diagnostics.py` deterministically distils those TRAIN
reports; it writes no coefficient. The three explicitly selected `gpt-6-astra`
high-effort physical-author subagents wrote the following reports and literal
deltas. The coordinator assembled their exact assignments; it chose no values.

| Family | Where R1 loses | R2 decision |
|---|---|---|
| Methanol | Sparse affected cases; N06M101 after prospective flame loss has increasingly positive diameter bias, while other runs improve | Replace N2 Cp in the Stefan vapor-enthalpy term with author-rounded methanol vapor Cp, **1380 J/kg/K** at the inherited 300 K reference |
| Heptane | Dominated by the 1 s forecast of N04H101; negative diameter bias increases, unlike methanol's dominant positive bias | Same physical species correction, **1660 J/kg/K** heptane vapor Cp; no adjustment of initial temperature or flame probabilities |
| ACME normal | Extinction loss across all seven TRAIN days; nonlinear condition perturbations affect the event probabilities | `condition.w = [1,0,0,0,0]`: withdraw unsupported probabilistic condition spread |
| ACME inverse | Loss concentrates on August 16, especially 21228C2; some other cases improve | Same nominal-input rollback; no flow-table repair or clock tuning |
| BASS | Early and late cutoffs degrade under the pure oxygen law, intermediate cutoffs mixed; only one independent day | **No new candidate.** Retain the frozen joint candidate as carry comparator; no weight is justified by this evidence |

The ACME rollbacks are expected to reproduce adopted Fable event tables, not
beat Fable or introduce a new mechanism. The droplet change corrects species
identity in a constant-property energy approximation; it is not expected to
repair missing liquid/gas thermal history, and may worsen TRAIN prediction.
Those risks are accepted before scores. A new BASS release with an empty patch
would imply novelty where none exists, so none is assembled.

Detailed evidence, derivations, source reading and exposure disclosures:

- `../research/astra_r2_droplet_loss_analysis.md`
- `../research/astra_r2_acme_loss_analysis.md`
- `../research/astra_r2_bass_loss_analysis.md`

All authors record incidental exposure to historical aggregate validation prose
in mandatory instructions; they did not open per-run validation or protected
test outcomes. These are not clean-room candidates. Parent exposure records
remain intact. All new numeric values were handwritten once before any R2 score;
no regression, optimization, search, scan or coefficient inversion was used.

## Frozen identities and checks

`../feedback/astra-r2-20261115/freeze.json` pins four new release IDs, their exact
R1 parents, the five adopted Fable baselines, all ten prior R1/joint releases,
the unchanged split/source hashes and data-file metadata from the previous run.
It also hashes each new literal patch, compiled patch, assembly receipt, author
report, diagnostic and wrapper. Old forwards, weights, contracts, data and
releases remain untouched. Every new release is experimental and not trained.

`literal_audit.json` records that exactly two scalar heat-capacity positions
and eight condition-mass positions change numerically across the four releases.
The nominal condition mass is explicitly assigned one but is unchanged. Every
other stored number matches its R1 parent. All four actual release forwards
admit their synthetic fixture. These checks are not NASA accuracy evidence.
The existing droplet7 and condition-committee synthetic invariant suites pass.

## Frozen scoring and decision rules

Use precisely the data roots, eligibility, cutoffs, horizons, scoring functions,
pairing, session/cluster aggregation, uncertainty bootstrap and verdict rules
in `ASTRA_FROZEN_TRAIN_20261114.md`. No cases are removed because they previously
lost; no outcome label or input value is repaired. The BASS future-flow and
track-ended conventions remain disclosed. Mixed-record files are streamed for
TRAIN identity before materializing records, as in the existing scorer; no
protected outcome is exposed to an author.

For each family the first three releases remain Fable, R1 and joint; a fourth
is R2 where one exists. They share exactly the same cases and settings. Reproduce
the previous baseline/R1/joint point scores before interpreting R2. The primary
comparison is R2 versus Fable; additionally report the paired difference from
its R1 parent. An ACME tie with Fable and improvement from R1 is called **removal
of a regression**, never superior science. Retained BASS joint results are not
presented as a newly improved model.

Coverage loss vetoes an overall win. FLEX primary is CRPS in micrometres; ACME
co-primary metrics are integrated IPCW contributions for extinction and
`operator_termination`; BASS primary is CRPS in sample widths, with track-ended
Brier as a guard. Report median absolute error and central band coverage where
available. Pair by run and cutoff, average within independent clusters then
equally across clusters, and report 10,000-resample descriptive 95% intervals
with seed 20261114. No interval is estimated from BASS's single day. Intervals
remain exploratory and not multiplicity-adjusted; no generalisation claim or
automatic adoption follows. Numerical equality tolerance remains 1e-12 in
reported units. The diagnostic IPCW bootstrap keeps the observed censoring
distribution fixed, as previously disclosed.

After this single scored round, **no R2 literal is revised**. Keep gains, harms,
ties and refusals. A failed physical improvement is a result, not permission to
search for a different number under the same protocol. New clean-room property
kernels remain outside this registered-release test for the previously recorded
adapter/input reasons.

## Execution and record

Commit this protocol, manifest, candidates, author reports and scripts before:

```powershell
python tools/wsp/astra/r2_frozen_train_comparison.py --execute --binary runs/tooling/astra-train-comparison-target/debug/wsp-dev.exe
```

The wrapper refuses uncommitted protocol files and existing output, verifies
frozen hashes and metadata before and after, and executes a byte-verified binary
snapshot. The scorer build identity is retained from its actual prior build;
source hashes prove the relevant code has not changed. Each receipt records
the protocol commit, command, executable hash and timing. Results go to
`../feedback/astra-r2-20261115/results/`, followed by a separate results commit.
No push, PR, deployment, validation look or final test.
