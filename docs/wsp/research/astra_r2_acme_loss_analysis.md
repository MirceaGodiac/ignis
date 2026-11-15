# ACME R2: TRAIN loss analysis and nominal-input rollback

Author decision, 2026-11-15: one R2 per geometry, **DATA-INFORMED, not zero-shot**.
Experimental; `trained_on = not_trained` describes the literal construction route,
not freedom from TRAIN feedback. This is a rollback of the outer condition mixture,
not a new physical mechanism or a claim of improvement over Fable. No R2 forecast,
candidate score, search, sweep, regression or parameter fit was run by this author.
The decision and literal files precede the host's assembly, freeze and single score.

## Evidence and boundaries

The authorized evidence is the two distilled TRAIN diagnostics at
`docs/wsp/feedback/astra-r2-20261115/acme_{normal,inverse}-diagnostic.json` and the
frozen TRAIN aggregate `docs/wsp/feedback/astra-frozen-train-20261114/results/summary.json`.
The comparisons use the same supplied conditions and shared-record fallback for
planned termination. R1, joint and inherited literal configurations and the
`condition_committee.rs`, `committee.rs`, `flame.rs`, `flame3.rs` implementations
were inspected. No new outcome table, protected partition, media, private validation
or per-test published validation/test result was opened.

Exposure disclosure: the root AGENTS instruction required the lane handoff. Its
first 45 lines incidentally included historical aggregate validation scores. They
were not used to choose or justify R2; no follow-up validation source was opened.
This report does not claim that the author's context contained only TRAIN numbers.

The frozen aggregate weights independent session/split clusters equally. Its mean
is therefore not the case-weighted average of cutoff means in the diagnostics.
Cutoffs from a single run are repeated forecasts, not independent experiments.
The supplied largest-loss/gain lists are selected examples, not a complete
condition-stratified dataset. Associations below do not establish physical causes.

## Where R1 loses

Lower integrated Brier contribution is better. All numbers in this section are
existing TRAIN observations, not R2 results.

| Geometry / target | Fable | R1 | Joint | R1 minus Fable |
|---|---:|---:|---:|---:|
| Normal extinction | 0.063130590 | 0.064068590 | 0.063592213 | +0.000938000 |
| Normal operator termination | 0.028018480 | 0.028355717 | 0.028181130 | +0.000337237 |
| Inverse extinction | 0.021341771 | 0.022029542 | 0.021673501 | +0.000687770 |
| Inverse operator termination | 0.095746681 | 0.095754945 | 0.095747970 | +0.000008265 |

There are 210 normal and 204 inverse scored run/cutoff cases, seven clusters each;
baseline and R1 have identical coverage and zero refusals in these comparisons.
The normal extinction cluster difference is positive on all seven TRAIN days.
October 18 (+0.001444762), April 16 (+0.001305408) and June 11 (+0.001054047)
are the largest day means. The case-mean excess at cutoffs 1, 3 and 5 seconds is
about +0.000983, +0.000987 and +0.000949: the loss is not isolated to one cutoff.

The selected largest normal losses include `21291D1`, `21162A2`, `21291G2` and
`21106A4`. They have roughly **1.25-1.6 milligrams/s**
injected reactant flow (1.25e-6 to 1.6e-6 kg/s), oxygen mole fractions near 0.39,
and interval-labelled extinction near 47-71 seconds. For `21291D1` at cutoff 1,
the extinction contribution rises from 0.096479085 to 0.103321394. This is evidence
of local forecast damage, not evidence that those physical input values are wrong.
Selected gains include October 18 operator-terminated `21291A1` and `21291C1`
at 202660 Pa. For `21291A1` at cutoff 5, extinction improves by 0.004259235;
R2 can lose that gain. Nominal rollback is not pointwise domination.

Inverse extinction loss is strongly concentrated on August 16: +0.004164659
as a cluster mean, versus +0.000413374 on August 10 and +0.000377573 on
September 13. July 15, July 28 and August 30 have tiny negative changes;
September 27 improves by 0.000139892. The descriptive frozen 95% interval
for the overall inverse extinction difference includes zero. The normal interval
does not, but neither interval establishes new-session generalisation.

The largest shown inverse loss is `21228C2`: at cutoff 5 its extinction
contribution increases 0.418270401 -> 0.478403082 and operator termination
0.227938273 -> 0.256194356. It has an operator-termination interval
65.5-66.5 s, injected oxygen flow 9.069954649e-6 kg/s, pressure 220660 Pa,
injected oxygen fraction 0.5 and ambient fuel fraction 0.089068575.
Other selected losses (`21256A5`, `21228J1`, `21222A4`) are short-lived
extinction cases with injected oxygen fraction 0.85. Gains include both short
extinction (`21256H3`) and operator termination (`21270H3`). There is no single
error sign by endpoint cause that justifies adjusting all extinction clocks.

## Mechanism supported by the implementation

R1 assigns normalized mass 0.2 to each of nominal, flow x0.9, flow x1.1,
pressure x0.95 and pressure x1.05. The joint has masses 0.6, 0.1, 0.1, 0.1,
0.1. Thus the joint event table is exactly the conceptual half-mixture of
nominal and R1 (subject to floating arithmetic). An intermediate score is
consistent with less displacement; it does not estimate a correct uncertainty.

`condition_committee::forecast` recomputes every positive-mass node's entire
predecessor forecast. It averages the resulting four-category event tables,
using fixed masses conditional on the admitted cutoff. These are not mere
error bars appended to a nominal forecast. The law mixture, its competing
risks and its prefix survival conditioning remain inside every node.

The inherited normal law committee uses `a0` and `b0` equally without an
admitted programmed stop; inverse uses `b0` and `b1` in ratio 2:1. Those
law disagreements remain independent of the outer scenario question.
The normal clock member has a flow coefficient 4.5 in the log burner-limit
time, so the existing +/-10% flow scenarios propagate nonlinearly into a
termination clock. Its extinction clock is also a hinge function of log flow.
The normal growth member already has `se.sigma_phi = 0.25` and burner
heating spread 0.12; inverse growth members already have 0.3 and 0.25.
Adding an independent outer condition spread may overlap uncertainty these
closures already represent. That overlap is a hypothesis, not an identified
variance decomposition.

In `flame3`, with composition and a flow-independent critical flux held fixed,
`r_ss` is proportional to reactant flow, `r_ext` to its square root, and
`Phi = (r_ss/r_ext)^2` to reactant flow. The clock contains
`erfcinv(Phi^-1/2)` and becomes held when `Phi <= 1`. A symmetric perturbation
of flow does not produce symmetric event-probability changes near this boundary.
The empirical inverse oscillation law contains flow/composition hinges as well.
Both condition nodes and law nodes feed competing extinction/termination risks.
These equations explain why modest scenario changes can alter a forecast
materially and why either cause's score can worsen. The distilled losses do
not give the per-node clocks or probability residual curves needed to identify
which mechanism caused each selected loss; no such attribution is asserted.

## Literal decision and parameter origin

Apply exactly the following assignment to each corresponding **R1** parent:

```text
condition.w[0:5] = [1, 0, 0, 0, 0]
```

Normal parent: `sha256:bd0daef42fa3b21ac067c6ea7286b978b98945efe15fa6a81cec2302fcaf821d`.
Inverse parent: `sha256:6d4353237550ccdb4528ee4875237f4a8709e66136c1e0784631c0557c480c35`.
The literal files are `configs/models/acme/wsp6/{normal,inverse}/r2-astra/changes.wsp`.

Origin: **S, structural selection**, five explicitly assigned positions per
geometry, of which four change numerically. One selects the existing nominal
condition; zeros deactivate the other hypotheses. There is no property value,
instrument precision, fitted probability, or directly transcribed outcome in
these new entries. The choice to withdraw the mixture is nevertheless informed
by TRAIN losses and the missing probabilistic justification. Inherited tensors
remain mixed-origin and TRAIN-informed; their origin is not erased by this patch.
All multipliers and all law/clock/uncertainty tensors remain unchanged.

The R1 documentation explicitly calls the nodes sensitivity scenarios, not a
calibrated instrument-error distribution. Equal scenario weights are sufficient
to define a sensitivity exercise; they do not establish the probability of each
condition given an instrument reading. No independent calibration, error model
or covariance evidence in this authorized material supplies such probabilities.
R2 therefore conditions on the provided nominal values and withdraws the extra
probability mixture. It does **not** claim the instruments have zero uncertainty.
Remaining uncertainty is the inherited model's uncertainty, not calibrated
coverage for omitted input error. No arbitrary narrower widths replace R1's.

A physically motivated diffusivity correction was considered but not authored.
The active normal `se.diff` is an effective closure with pressure coefficient
0.45, composition terms and hinges; inverse has constant `se.diff`. Interpreting
these immediately as a single molecular diffusivity and substituting a transport
exponent would change the meaning of an inherited closure without independently
bound transport temperature/composition or evidence separating the other clocks.
Likewise, no condition-table repair, new species property, flow calibration,
planned-stop input, per-case exception or outcome-derived clock is smuggled
into the patch. None is justified within this fixed-input, existing-spec revision.

## Expected effects, limitations and verification

One positive-mass neutral node runs the unmodified predecessor once; the outer
forward directly clones its event table. R1 contains the exact inherited Fable
committee values. Consequently R2 is expected to be bitwise equal in event
tables to the corresponding Fable baseline on the same admitted prefixes,
settings and executable build. Release/artifact identities and diagnostics will
differ. The outer wrapper has extra semantic input checks, so this is not a
claim that every conceivable baseline-admitted prefix is accepted by R2.

Expected TRAIN effect is removal of R1's aggregate excess shown above, including
loss of R1's individual gains. This is an implementation-derived expectation,
not an executed R2 result or a better-than-Fable result. The nominal baseline's
substantial absolute errors, uncertain control-plan provenance, label/input
issues and effective-closure limitations remain. No new uncertainty calibration,
causal discovery, independent-session qualification or NASA validation follows.

Existing synthetic integration test
`neutral_nodes_equal_predecessor_exactly_and_leave_inputs_unchanged` in
`crates/ignis-wsp/tests/condition_committee_forward.rs` expresses the neutral-node
invariant and was inspected, not executed in this author subtask. Static checks
confirmed the assignment form matches existing WSP syntax, weights are finite
and nonnegative with one positive entry, and inherited nominal multipliers are
exactly one. No source code, shared contract, prior release or forward changed.
Host assembly and the single frozen comparison remain to be performed. Keep
R1 and joint as negative-result records; do not revise R2 after seeing its score.
