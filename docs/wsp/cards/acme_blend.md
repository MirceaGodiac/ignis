# ACME burner flames, two laws answering together (`acme_normal.wsp4`, `acme_inverse.wsp4`)

**State: experimental, hand-written, data-informed. Normal flames: carried. Inverse flames: not
adopted.** Forward `blend.rs`. A blend release holds one
complete set of numbers for each of two existing hand-written laws and two weights; its forecast is
the weighted mean of what the two member forwards return for the same prefix. No number of either
member was changed, and the only numbers typed for these models are the weights.

| Model | Candidate | Release | Members (weights 1 : 1) | Non-zero numbers |
|---|---|---|---|---|
| | baseline pair | `sha256:d7bd7e2c3aed4b843cdf30f95a2bda3d5d0e7396293ca844dab7dd31a583796a` | the two reference baselines | 43 / 854 |
| | baseline pair | `sha256:b971945ec13fbd4de33f07df26a6020849301b4f451af554b32569bd7ec05003` | the two reference baselines | 43 / 854 |

## What it computes

```text
P(t) = (w_a P_a(t) + w_b P_b(t)) / (w_a + w_b)      for each row of
       (active, extinction, operator termination, unresolved)

a   tensors a.*: a full flame.rs release, run by flame::forecast       (hinge-law clocks)
b   tensors b.*: a full flame3.rs release, run by flame3::forecast     (growth clock, heating law)
w   tensor mix.w
```

Each member runs unchanged, by the code its own releases use, on the same prefix and forecast times.
A weighted mean of two cumulative incidences is a cumulative incidence, so the output is a forecast
of the same kind. Read as a model: with probability one half the hinge law is the right description
of this flame, otherwise the growth law is. The weights are constants; nothing in a prefix moves
them. A member with weight zero is not run. If a member refuses the prefix, the forecast is refused
with that member's reasons.

## Why

```text
score(mean of two forecasts) = mean of the two scores − ¼ · (their squared disagreement)
```

so the blend is never worse than the average member, and better by as much as the members disagree.
That is arithmetic, not a fit, and it holds on data nobody has seen.

## Provenance

The member values are the stored values of the member releases, copied statement by statement with
the prefix `a.` or `b.` by `tools/wsp/blend_edits.py`. That script chooses and computes nothing, and
`crates/ignis-wsp/tests/blend_forward.rs` compares the committed blend releases with the member
releases tensor by tensor. The weights were typed after the author had seen the aggregate
validation scores of all four members (the two normal members tie; the inverse `wsp3` member is
ahead). They are equal, which is what one types knowing nothing, and no other pair was tried.

The forward identity of a blend covers `blend.rs`, `flame.rs` and `flame3.rs`: a change in either
member's code is a change of the blend.

## Results

Committed before any validation score of them existed. The rule, the one in force for
burner flames: a blend replaces the carried candidate unless its validation extinction score is
worse.

| Integrated Brier 6–120 s | Train: extinction | Train: flow termination | Validation: extinction | Validation: flow termination |
|---|---|---|---|---|
| `acme_normal.wsp4` D1 | 0.052 | 0.028 | **0.155** | 0.098 |
| member b: `acme_normal.wsp3` D1 | 0.063 | 0.024 | 0.163 | 0.100 |
| baseline pair | 0.151 | 0.068 | 0.181 | 0.099 |
| best fitted (linear hazard) | 0.108 | 0.039 | 0.185 | 0.093 |
| `acme_inverse.wsp4` D1 | 0.041 | 0.061 | 0.142 | 0.092 |
| member b: `acme_inverse.wsp3` D1 (carried) | 0.036 | 0.064 | **0.137** | **0.088** |
| baseline pair | 0.315 | 0.198 | 0.255 | 0.155 |
| best fitted (`acme_inverse.hazard`) | 0.068 | 0.076 | 0.143 | 0.118 |

**Normal flames: adopted.** 0.155 against 0.159 for the carried hinge law, and ahead on flow
termination too. `acme_normal.wsp4` D1 is the carried normal-flame model. The members disagree by
about 0.15 (root mean square) on validation as on train, and the blend gains exactly what the
identity above says. 84 cases in two sessions: the margin is within the noise of that partition, and
only the sign was asked of it.

The pair of reference baselines (a condition-free clock, and the published theory; no number from
the PSI-159 tables) scores 0.181 on normal-flame extinction, where each alone scores 0.216 and 0.206
and the best fitted model 0.185. One partition, two sessions; it is reported, not claimed.

## Limits

- Everything in the limits of the two member cards holds for the member it belongs to. Where both
  members are wrong in the same direction (a regime neither describes: 3 % butane, the flames that
  cycle between hot and cool) the blend is as wrong as they are.
- Equal weights are a prior, not a finding. The weights do not depend on the conditions, although
  the members plainly have different strengths (enriched oxygen near the holding flow, where the
  growth law is physics; fuels and dilutions with many train flames, where the hinge law has seen
  them). Typing a weight law from the train table would be one more thing typed from seven sessions.
- The spread of a blend is two-peaked where the members disagree. That is the honest statement of
  not knowing which law holds, and it is not a calibrated uncertainty.
- Twice the numbers, twice the forward time (still far below a millisecond per forecast time).
