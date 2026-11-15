# Astra: six-family error budgets before further construction

Author: **gpt-6-astra**. Date: **2026-11-14**. Authority: the complete
[ASTRA_GOAL_WSP.md](../dossier/ASTRA_GOAL_WSP.md), supplied as commit `08b7841`.
This is physical interpretation of allowed TRAIN evidence. Every proposal here is
**data-informed**, with **no W0 eligibility**. No new weights, constants, forward code,
measurements, labels or scoring runs were produced for this document.

## 1. Chronology and what this budget can identify

This budget precedes **further** construction. It cannot preregister earlier prototypes,
the ten R1/joint releases, or the property-droplet kernel already described in the goal.
The earlier [budget](../dossier/ASTRA_ERROR_BUDGET.md) followed prototypes and preceded
their weights; [results](../dossier/ASTRA_RESULTS.md) now report their TRAIN outcomes.
Those mechanisms remain existing experimental alternatives, not newly justified successes.
The goal reports a frozen property-droplet W0 kernel; its freeze and code were not audited
here. Reading this document cannot preserve an author's clean-room status.

Status vocabulary in the rankings:

- **planned/pending**: a prospective physical hypothesis or input requirement; no new
  construction by this task, and the named evidence/contract gate remains unsatisfied.
- **none**: no candidate justified from the permitted evidence yet.
- **frozen/existing**: an earlier kernel or release described by the allowed sources;
  retained without revision here. This is chronology, not a qualification label.

The central distinction is between **loss concentrated on certain observations** and
**loss caused by a removable model defect**. CRPS and competing-event IBS are not additive
mechanistic energy budgets. Correlated input errors, state mistakes, observation bias,
model spread and physical misspecification interact. Subtracting flagged-case losses
does not yield the score of a perfect physical law. These reports lack the independent
replicates, calibrated observation model and intervention comparisons needed to identify
a physical irreducible floor or a perfect-law score for **any** family.

A perfect law supplied with wrong conditions still predicts the wrong experiment. A
perfect physical trajectory compared with a biased image tracker may score poorly. A
perfect prediction of physical extinction does not determine an administrative stop.
Conversely, a smaller score against one tracker is not necessarily better physics.
Requests below separate those cases instead of assigning unmeasured percentages.

No new validation aggregate, per-run validation outcome or held/test data was requested.
The historical transfer numbers already printed in the owner's goal are not used to
rank or tune these proposals. One train-named report contains an incidental historical
unseen-session coverage assertion; the exposure is recorded in section 10 and excluded.

## 2. Evidence map and denominators

The latest permitted comparison is the original R1/joint TRAIN checkpoint, not an
all-investigation evaluation. Generic cases are run-cutoff pairs at 1/3/5 s. FLEX's
target horizon is 10 s; ACME's fixed event horizons are listed in `ASTRA_RESULTS.md`.
BASS uses cutoffs 5/10/20/40 s after igniter-off and a 60 s target window. Repeated
cutoffs, frames and event horizons are not independent experiments.

| Family/evidence | Train support available here | Latest reference; R1; equal joint | Interpretation |
|---|---|---|---|
| Droplets: methanol | 68 runs in TRAIN split; 182 scored cases; 26 scored sessions | Diameter case-mean CRPS **29.8681; 29.9267; 29.8979 µm** | R1 and joint worsen this score; 3 domain-refused cases and 4 admitted unscorable cases are separate |
| Droplets: heptane | 49 runs in TRAIN split; 125 scored cases; 20 scored sessions | Diameter case-mean CRPS **46.1562; 46.3022; 46.2205 µm** | 9 domain-refused cases; R1 and joint worsen this score |
| Spherical burners: CFI-G normal | 81 runs in TRAIN split; 210 scored cases; 7 scored sessions | Extinction IBS contribution **0.051752; 0.052725; 0.052232**; operator termination **0.027399; 0.027804; 0.027596** | No domain refusals here; both endpoints worsen |
| Spherical burners: CFI-G inverse | 73 runs in TRAIN split; 204 scored cases; 7 scored sessions | Extinction IBS contribution **0.027091; 0.028194; 0.027624**; operator termination **0.060816; 0.061072; 0.060939** | No domain refusals here; both endpoints worsen |
| Burning-rate emulator (BRE) | No family-specific TRAIN feedback in sources read | **Unmeasured** | No measured score, scoreable-test denominator or independent-group denominator established |
| Jets/coflows | No family-specific TRAIN feedback in sources read | **Unmeasured** | CFI-G's spherical burner scores cannot qualify a jet or coflow forward |
| Solids: BASS-II SIBAL tracks | 4 TRAIN tracks; 3 scoreable spreading runs, all one day; 12 cases | Tip CRPS, mean over runs, **0.076101; 0.080217; 0.073439 sample widths** | Joint improves slightly on one day; standalone R1 worsens. T10 is the fourth, non-spreading track, not a fourth scored trajectory |
| Smoke/aerosol | No family-specific TRAIN feedback in sources read | **Unmeasured** | No calibrated smoke-target forecast error is available here |

Source: [machine summary](../feedback/astra_train_summary.json), `models.*.releases`,
`runs_in_train_split`, `counts`, `protocol`; [results](../dossier/ASTRA_RESULTS.md).
Split-run counts are not claims that every run contributed a scored case. The summary
separately records 15/13/33/15 not-at-risk cases for methanol/heptane/normal/inverse.
Do not add those counts to the scored totals without the scorer's accounting rules.
The BASS event comparison has 59 observable pairs and **one** ended pair; its small Brier
score does not demonstrate calibrated extinction or non-spread probabilities.

Equal-session and case-weighted means answer different questions. For example the
methanol reference is **48.4466 µm** with equal-session weighting versus **29.8681 µm**
with case weighting; heptane is **51.0843 versus 46.1562 µm**. Neither is an unseen-session
estimate. Proposed registrations must retain both aggregation rules and report refusals.

## 3. Droplets: measurement/state errors precede another narrow cooling law

### Identified TRAIN concentrations

[Methanol D3 anatomy](../feedback/flex_methanol_d3_train_anatomy.md) reports a summed
CRPS of 5436 µm over 182 cases. Artefact-labelled cases carry **38.8%** (15 cases),
extinction within the horizon **27.3%** (57), already-out cases **4.9%** (8), continuing
burns **27.6%** (96), and no sustained flame **1.4%** (6). These are disjoint classes in
that audit, not identified causal components. Leads 5-10 s carry **47%** of total loss.
Its approximate timing attribution is about **5%** of total loss, not the entire
post-extinction loss. Prefix rules locate five artefact cases carrying 1220 µm of the
2109 µm artefact total, concentrated in two runs; future view loss and wrong-atmosphere
cases can have clean prefixes. A tracker-state input could help some forecasts, but
cannot promise removal of those 1220 µm.

[Heptane D2 anatomy](../feedback/flex_heptane_d2_train_anatomy.md) reports 5770 µm over
125 cases. Twenty problem-run cases carry **57.7%**; 32 artefact cases carry **3279 µm**.
Those classifications **overlap** and must not be added. A prefix-signature union finds
16 artefact cases carrying 2857 µm but also 12 clean cases carrying 223 µm. Helium-stage
cases carry **7.2%**, nine cases in **one session**. The eight timed helium cases in three
runs have a median hot-flame ending about **2.7 s late**. Outside helium the hot-to-cool
timing median is near zero over 25 cases, 13 runs and ten sessions; an approximate
attribution gives **54%** of hot-to-cool loss to next-stage rate and **19%** to timing.
These are historical D2 diagnostics, not a causal decomposition of every later release.

The allowed R1 checkpoint is a direct counterexample to assuming that a more detailed
post-flame integrator improves the score. Methanol's 77 nitrogen-eligible cases worsen
by **0.1385 µm** for R1; heptane's 52 worsen by **0.3511 µm**. Eligibility does not reveal
the latent transition. Equal-session R1 differences are **+0.0301/+0.1274 µm** for the
two fuels. No numerical gain is assigned to rewriting this closure again.

### What remains nonidentified

The methanol audit's approximately 5.5 µm short-lead contribution follows its assumed
noise width; frame scatter near 1.5 µm is a different quantity. Neither is a physical
irreducible floor. Heptane's 13.1 µm oracle-recentred-band diagnostic uses future truth
and is not an attainable causal score. Image scale, glare, soot, fuel identity, absorbed
water, thermal initial state and late stage identity are not separately resolved by
these residuals. A helium post-flame diameter decrease does not alone identify cool
chemistry versus sensible-heat-driven evaporation. No perfect-law score is available.

### Prospective ranking (within this family)

| Rank/id; status now | Change and expected unseen-group benefit | Refuting test and gate |
|---|---|---|
| 1 / D1; planned/pending | Consume document-lane condition identity and causal tracker-quality metadata; separate physical diameter from observation failure. High leverage is plausible because loss is concentrated on measurement/identity problems across both fuels; numerical gain **not identified** | On a registered group-held comparison, conditioning on prefix-only quality fails to reduce proper score or harms clean-group coverage. Require duplicate-measurement TRAIN sample, calibration and false-positive reporting; never select future artefacts as a causal input |
| 2 / D2; planned/pending | Property-based fuel/mixture transport and thermal state, preserving predecessor behavior on predecessor cases. Expected benefit is acceptance and physically meaningful extrapolation to new fuels, not an established CRPS reduction | Limit checks fail, two-source properties disagree beyond stated uncertainty, or new-fuel group scores fail the registered baseline. Require exact mixture/phase, temperature, gas properties and allowed domain; retain fallback/refusal for unresolved conditions |
| 3 / D3; planned/pending | Distinguish an extinction inside an observation window from a still-burning state; represent hot/cool/post-flame alternatives with physical observations rather than sharpening a train-derived boundary | Independent TRAIN state labels and prefix-cutoff perturbation show that the state correction merely moves errors across windows, or it worsens registered new-session scores. Require state-definition and clock uncertainty; one conspicuous methanol case is insufficient to determine coefficients |
| 4 / D4; planned/pending | Scope helium thermal relaxation and cool-flame chemistry as distinct hypotheses. Potential benefit is regime transfer; evidence is too narrow for a signed numerical expectation | Thermal-only predictions cannot explain both size and temperature/species observations, or a new helium session contradicts the trend. Require independent TRAIN observables and admitted intermediate-He conditions; no direct typing of the audit's suggested multiplier |
| Existing / D0; frozen/existing | R1 coupled cooling and its joint remain negative-result references. Property W0 kernel is reported frozen in the goal, not rescored here | New work must reproduce predecessors on declared predecessor cases and preserve original results. This row does not retroactively justify construction |

## 4. Spherical burners: conditions and competing events dominate the next question

CFI-G is evidence for spherical burner flames; do not silently relabel it as a jet.
The earlier budget reports historical loss concentrated in a few runs (five normal runs
carrying 52%/64% of two endpoint losses; five inverse runs carrying 64% of extinction
loss). That is **secondary evidence from the earlier budget**, whose older metric values
are not substituted for the current table above. The underlying non-train-named transfer
audit was not opened for this task.

[Raw TRAIN signals](../feedback/acme_raw_signals_train.md) examines 154 train tests,
153 with a data file. It identifies **29 scripted ends**, distinct from extinction and
burner-limit stops. Table/readback disagreements include `21125K1/K2/K3` fuel fraction
0.031 versus 0.30 and several flow mismatches. This is a request to resolve provenance,
units and sensor interpretation; no shared conditions were changed here. The report's
claim that an archived "as run" file is pre-ignition knowledge is **not accepted merely
because the resulting stop time agrees**. The older budget explicitly leaves temporal
provenance unresolved.

Burner heating observations do not support one universal time constant: normal and
inverse closed-form rise diagnostics differ, with horizon-dependent values. The TRAIN
report has 110/73/49 surviving observations at 10/30/60 s; these are changing risk sets,
not 110 independent complete heating trajectories. Thermocouple cold junction is assumed,
absolute uncertainty is estimated at about 1.5 degrees C, PMTs saturate for inverse
flames, and dim/cycling flames can challenge the PMT endpoint. A reported timing agreement
or heat-rise ratio does not separately identify sensor, law and termination-label error.

R1's five persistent condition scenarios worsen normal extinction in all seven TRAIN
sessions. Inverse improves in four and worsens in three, but its aggregate worsens.
Generic broadening has therefore not established positive expected transfer benefit.
No measurement floor or perfect-law IBS is identified; an `unresolved_support` contribution
of zero in this particular score is not proof that conditions or outcomes are unambiguous.

| Rank/id; status now | Change and expected unseen-group benefit | Refuting test and gate |
|---|---|---|
| 1 / S1; planned/pending | Causal planned stages and controller settings; separate scheduled termination, limit trip and physical extinction. Best supported prospective change in this family because it supplies a different cause of the event, rather than adjusting a combustion clock | An immutable pre-ignition plan cannot be established, or plan-conditioned forecasts fail to improve the registered stop endpoint on unseen groups. Require original timestamp/version and actual controller semantics; no future recorded stop time as input |
| 2 / S2; planned/pending | Correctly identified gas properties and cutoff-local commanded/readback flows, with retained raw units and uncertainties. May remove systematic domain/input mistakes that small perturbations cannot cover | Independently calibrated commands/readbacks contradict the interpretation, or unchanged physical laws with accepted corrections do not improve prospective scores. Require gas correction factors, pressure reference and channel dictionary; keep original values and discrepancies |
| 3 / S3; planned/pending | Distinct burner thermal inertia/controller response and combustion state, conditioned only on available prefix history. Plausible improvement for limit trips and transfer between hardware/flow regimes, no identified gain | Heating-shape predictions fail on a new session or calibrated prefix temperature adds no value over accepted plan/flow inputs. Require sensor lag/cold-junction information and documented control limit; do not derive limits by outcome-selected session tuning |
| Existing / S0; frozen/existing | Persistent-condition R1 and equal joint retained unchanged as negative-result comparisons | Do not tune scenario widths from their worst train cases or claim they correct gross condition errors |

## 5. Burning-rate emulator: observable and command semantics before an error number

No BRE TRAIN score, scored test list or group denominator appears in the sources reviewed.
Consequently measurement contribution, parameter/condition contribution, structural law
error and extrapolation error are all **unmeasured**, not zero. A burner feed rate that
emulates burning is not automatically an observed material regression rate. A commanded
quantity cannot be its own independent verification target.

The physical hypothesis to test is a conservation link between feed, transport, heat
feedback and an independently observed flame response. It does not follow from this
budget that BRE has a scoreable regression, extinction or heat-flux endpoint. Document-lane
apparatus cards must first identify what was prescribed, what was measured, which surfaces
and areas define flux, the clock, and how sensor delay relates to commanded steps.

| Rank/id; status now | Change and expected unseen-group benefit | Refuting test and gate |
|---|---|---|
| 1 / B1; planned/pending | Declare command versus response and add typed geometry/properties to a conservative forward. Expected benefit is removing a circular target and enabling meaningful transfer, not a promised score decrease | Apparatus cards show the proposed response is imposed rather than measured, or conservation fails in the admitted regime. Require a TRAIN target, units, calibration, command timeline and frozen group identifiers |
| 2 / B2; planned/pending | Separate apparatus thermal/flow lag from the combustion response using independently known hardware parameters | A step-response comparison with calibrated sensor lag cannot distinguish the proposed states, or group transfer is worse than the registered simple baseline. Require TRAIN step traces and sensor response; no fitted relaxation coefficient |
| 3 / B3; none | No empirical uncertainty-width refinement or new event law justified yet | Obtain repeated or independent measurements and a registered endpoint before a spread can be evaluated. A symbolic uncertainty model alone is not calibrated coverage |

The first protocol must register one observable supported by the released measurement
package, denominator and independent grouping, baseline, score, censoring/refusal handling
and interpretation of failure. Until then the attainable score is **not identified**.

## 6. Jets/coflows: geometry, boundary conditions and optical target identity

No jet/coflow TRAIN forecast errors were read. The goal's already-known transfer aggregates
are not used as a surrogate error budget. In particular neither spherical CFI-G scores
nor a fuel-name acceptance count establishes a jet flame-shape or soot model's accuracy.

Candidate error sources are hypotheses: nozzle/coflow geometry; fuel and oxidizer
properties; velocity and temperature boundary conditions; chamber pressure/gravity;
finite transport/chemistry; radiative loss; and the optical definition of the observed
flame. Luminous/sooting boundaries, reaction zones and temperature contours are different
quantities. Projection, exposure, saturation and line-of-sight integration can change a
measured shape without the underlying reaction surface moving. No fractional attribution
or perfect-law floor can be derived without a target-specific calibration and TRAIN data.

| Rank/id; status now | Change and expected unseen-group benefit | Refuting test and gate |
|---|---|---|
| 1 / J1; planned/pending | Property/geometry-based transport coordinates with distinct nozzle and coflow conditions; preserve old cases in a new forward | A registered new-fuel or new-geometry comparison fails the specified baseline or the proposed similarity coordinates collapse different regimes. Require independent mixture properties, boundary-condition uncertainties, geometry and admissibility envelope |
| 2 / J2; planned/pending | Match output to a typed optical/physical flame observable and measurement operator | Calibration shows that predicted and observed boundaries are not the same quantity, or independent views contradict the proposed observation model. Require masks/edge definitions and uncertainty from the document lane, TRAIN-only outcomes |
| 3 / J3; planned/pending | Separate extinction, imposed termination, soot/radiation and finite-rate limits only where each has an independent endpoint | New-group event or shape scores show no benefit versus the frozen simpler forward; competing-event mass or physical limits fail. Require pre-ignition plans and distinguishable event labels; no fuel-name-specific constants from residual inspection |

An initial registration must choose a scoreable endpoint rather than treating flame size,
soot intensity and extinction as interchangeable. Register grouping by actual independent
experimental units, not frames or adjacent tests in a chamber fill.

## 7. Solids in flow: one train day does not identify material transfer

[BASS TRAIN tracks](../feedback/bass_train_tracks.md) supports three spreading SIBAL
runs, T5/T6/T16, on the same day, plus non-spreading T10 on another day. Steady tip speeds
are about **0.033-0.043 sample widths/s**, and post-step flame-length/speed ratios are
**9.4-10.4 s**. These are observed relationships, not universally authored constants.
Every pre-step interval still contains ignition transients. T5 retreats **0.21 widths**
and regains its pre-step position after **11.7 s**; smaller steps mainly stall the tip.
A steady oxygen-supply balance alone does not explain that transient response.

The scale is **0.018 sample widths/pixel**; between-test origin uncertainty is about
**0.05 widths**, duct-to-top-view conversion about **0.1-0.2 widths**, and still/overlay
timing uncertainty **0.33/0.78/1.58 s** for the three spreading runs. These errors can be
shared between prefix and target and cannot simply be summed into a CRPS floor.
Near-zero detrended scatter is affected by quantisation. Low-fan physical velocities
are partly extrapolated, and the report marks the zero-flow calibration unreliable.

R1's mean tip CRPS is **0.004116 widths worse** than the reference; joint is
**0.002662 widths better**, with two of three runs improving. The report's run-bootstrap
interval for the joint difference spans zero; resampling runs from one day does not
create independent session transfer. One ended event pair cannot identify quench
calibration. Fuel depletion, reaching a field-of-view edge and true extinction need
separate labels; the T5 endpoint itself may coincide with the end of the fuel.

| Rank/id; status now | Change and expected unseen-group benefit | Refuting test and gate |
|---|---|---|
| 1 / F1; planned/pending | Physical sample geometry/loading/material properties and independently calibrated flow, with explicit ignition history. Expected benefit is cross-material/cross-day applicability and meaningful units, with unquantified score effect | Newly released TRAIN conditions or independent calibration contradict the mapping, or a registered different material/session fails the baseline. Require BASS PSI-26 condition rows, thickness/areal density, composition, exposure history and flow calibration |
| 2 / F2; planned/pending | Separate steady spread from finite heat-storage/front-length response to a planned flow step | It cannot reproduce the sign of tip/base response without train-chosen constants, or fails a new step/session against frozen steady and prefix-persistence baselines. Require synchronized TRAIN tip/base and command traces plus uncertainty; preserve igniter effects |
| 3 / F3; planned/pending | Separate physical quench, fuel exhaustion and loss of view in the target contract | Blind TRAIN relabelling cannot distinguish endpoints or adequate independent positive/negative events remain absent. Require explicit censoring and sample-end geometry before evaluating event probabilities |
| Existing / F0; frozen/existing | R1 oxygen-supply closure and equal joint retained; modest joint train improvement is not a new-material result | Preserve prior event semantics and negative standalone result; do not select a sharper boundary from T10 alone |

## 8. Smoke/aerosol: source, transport and detector are different unknowns

There is no calibrated smoke/aerosol TRAIN forecast score or group count in the permitted
feedback. The BASS table's chamber CO/CO2 entries are not an aerosol size distribution,
soot yield or independently attributable emission rate. Consecutive chamber-fill tests
must not be differenced; no proposal here uses that operation.

For a safety prediction, the following quantities need separate identities: generation
rate/yield at a source; number or mass versus particle size; transport/dilution/deposition;
detector response/threshold; and exposure at a specified location. Optical brightness
does not uniquely identify any of them without an observation model. Unknown source
yield and unknown detector efficiency can compensate, so a small residual on one channel
would not identify a physical law. No perfect-law score or irreducible measurement floor
is available.

| Rank/id; status now | Change and expected unseen-group benefit | Refuting test and gate |
|---|---|---|
| 1 / A1; planned/pending | Conservative source-to-chamber transport with explicit volume, flow, sampling and deposition assumptions; source strength remains an independent input unless observed | Calibrated mass/number balance fails, or source strength cannot be independently supplied. Require apparatus/method cards, blank/background procedure, sampling locations and TRAIN time series with units |
| 2 / A2; planned/pending | Instrument-specific observation operator for one declared safety endpoint | Paired instruments or calibration particles show the operator cannot distinguish size/composition response, or a new-session detection score fails the frozen baseline. Require detector spectral/size sensitivity, saturation, sample-line loss and delay |
| 3 / A3; none | No universal material-to-soot/smoke-yield law justified from current evidence | Require independent TRAIN source measurements across identified materials, a yield definition and frozen source/group splits. Never infer a yield from one optical channel and validate it on that same channel as independent truth |

Register concentration, distribution or detector event according to the actual released
observable; no renaming a detector voltage as concentration. If PSI cannot score the
chosen target, record that reason with the investigation rather than reporting zero error.

## 9. Cross-family priority and requests to the document/evaluation lanes

This is an **ordered work priority based on defensible mechanisms and evidence**, not a
numerical expected-gain league table. Units and support differ. None of the prospective
changes has an identified numerical gain on unseen groups; each may be neutral or harmful.
Within the six family tables, higher rank means the prerequisite or stronger current
reason to expect benefit. The cross-family order is:

1. **S1/S2, D1, F1: accept real prospective conditions and observation identities.**
   Existing TRAIN errors directly support these failure modes. More accurate physics
   cannot undo an unavailable plan, wrong mixture or an incorrectly defined target.
2. **D2, J1, B1, A1: common typed properties, geometry and conservation.** This enables
   new-family/new-material applicability; no score improvement follows automatically.
   Cross-family agreement is a check on shared properties, not independent confirmation
   if both models reuse the same wrong source.
3. **F2, S3, D3: explicit state/history where simple steady or single-clock descriptions
   conflict with TRAIN observations.** Mechanisms have observable refuters; parameters
   must come from declared physics/property sources, not scans of these residuals.
4. **J2, A2, F3: match endpoints and observation operators.** These are prerequisites
   before those targets can be scored at all, even though current gain is unmeasured.
5. **D4/J3 and new source-yield or calibration refinements:** defer until independent
   observables and group support exist. One-session helium evidence and one-day spread
   evidence cannot support fine numerical tuning for unseen NASA investigations.

The main author owns `docs/wsp/measure/REQUESTS.md`. The following is the actionable
request map for that author, not an instruction to read restricted data or perform
measurement in this lane. Release all material through `runs/dev/data_docs/release/`.

| Request | Tests/scope; required payload | Why; sufficient next decision |
|---|---|---|
| Q-D | Droplet TRAIN groups, including known artifact/state cases; FLEX-2/other droplet **conditions** as allowed for all groups. Fuel/mixture identity and provenance; scale/time-zero uncertainty; independent TRAIN diameter/state checks; causal prefix-quality fields | D1-D4: distinguish wrong conditions and tracker failures from state/transport error; define admitted property regimes |
| Q-S | CFI-G TRAIN signal comparisons; pre-ignition controller/flow configuration and condition discrepancies for allowed groups. Original plan versions/timestamps, MFC gas corrections, cold-junction/lag metadata, distinct TRAIN event labels | S1-S3: decide admissibility of plans/readbacks before they enter a forecast; keep physical extinction separate from termination |
| Q-B | BRE methods/apparatus page cards and frozen-group inventory; TRAIN-only measurements of the independently observed response with calibration, command timeline and geometry | B1-B2: choose a noncircular endpoint and identify whether apparatus dynamics are separately observable |
| Q-J | Jet/coflow methods/properties/geometry cards; TRAIN-only optical/event endpoints; condition supplements with exact mixtures, flow temperatures and nozzle/coflow dimensions | J1-J3: specify what flame observable is measured and register an evaluable property-based transfer |
| Q-F | BASS PSI-26 and other solid conditions; independently calibrated flow and sample loading; TRAIN-only synchronized front/base and endpoint labels; igniter/step provenance | F1-F3: enable material units, transient testing and valid censoring instead of treating every disappearance as quench |
| Q-A | Smoke/aerosol apparatus and detector response cards, independent source/blank/calibration information and TRAIN-only signals; no inter-test chamber-fill differencing | A1-A3: determine whether concentration, distribution or detector event is identifiable and scoreable |
| Q-G | For each family/investigation, release-safe split identifiers and independence unit; prospective protocol subsection with frozen releases, target, metric, baseline, censoring/refusals and interpretation | Establish the denominator before evaluation. No outcomes of held investigations or protected partitions are needed by this author |

All new inputs require the page-02 causal-input classification before consumption. New
quantities require a contract proposal, not a local reinterpretation of an existing field.
No request authorizes a final-test look. A later evaluation result belongs to a registered
comparison committed beforehand; this document is a research priority list, **not** an
executed or committed scoring registration. A fresh clean-room author must receive only
the approved spec/physics/class-A/B source package, never this budget or its TRAIN numbers.

## 10. Sources, exact reading extent and limits

Only local author-authorized documents below support the numerical claims. No source
linked inside them was silently treated as newly read. Counts are report counts, not an
independent replay. No whole NASA report or new primary-paper claim was introduced;
prospective physical mechanisms are hypotheses awaiting the separate source/derivation
work, not measured conclusions.

| Source | Extent actually inspected for this task | Use |
|---|---|---|
| `docs/wsp/dossier/ASTRA_GOAL_WSP.md` | Entire file, 225 lines | Scope, chronology, six families, access boundaries; its already printed transfer aggregates excluded from ranking |
| `docs/wsp/dossier/ASTRA_ERROR_BUDGET.md` | All substantive sections 1-4 and final reading-boundary paragraph; 167-line file, initial combined output followed by explicit tail read | Historical TRAIN concentrations and warnings; indirect ACME transfer-audit statements remain marked secondary |
| `docs/wsp/dossier/ASTRA_RESULTS.md` | Entire file, 168 lines | Built mechanisms, latest checkpoint scores and negative results; no linked result artifact opened beyond allowed summary |
| `docs/wsp/feedback/astra_train_summary.json` | Whole file parsed; 3723 lines. Initial full display was truncated, so **not claimed fully reviewed**. Explicit projections inspected model keys, split/not-at-risk counts, all release CRPS/IBS metrics, refusal/unscorable counts and paired session summaries; BASS counts/protocol/release and paired blocks visible in initial output | Quantitative denominators and scores; no scorer rerun |
| `docs/wsp/feedback/flex_methanol_d3_train_anatomy.md` | Entire 200-line report including proposals and limitations | Historical TRAIN class/lead/state/measurement concentrations; hand-estimated proposal gains excluded |
| `docs/wsp/feedback/flex_heptane_d2_train_anatomy.md` | Entire 235-line report including proposals and limitations | Historical TRAIN concentrations; hand-estimated gains excluded. **Incidental exposure:** proposal 2 contains a historical unseen-session coverage assertion, encountered inside this train-named report; it is not used or repeated numerically here, and the main author was informed |
| `docs/wsp/feedback/acme_raw_signals_train.md` | Entire 169-line report | TRAIN channel, plan, input and event-meaning limitations; no referenced raw files opened |
| `docs/wsp/feedback/bass_train_tracks.md` | Entire 202-line report, sections a-j, calibration and files | TRAIN track/flow/geometry/time limitations; no media or track files opened |

Source SHA-256 at inspection (ordered as the table):

```text
a5fadaf512a4538ce7204feabca727d489a000d17967319257adb64d4ad4af71  ASTRA_GOAL_WSP.md
336af178beb6b25afc69b49ca6376d9b5336da5c4c67b990822c0b4d40593933  ASTRA_ERROR_BUDGET.md
d0a8b5488d86c16474fcaa714adbaae2b48141034733b54cb1c436f6de898010  ASTRA_RESULTS.md
d9da5b61409328f6948e77fcfaac6a51a1aff8d17f0f389dd722d097af8155c1  astra_train_summary.json
4908de9291d21b10f4d1216d6f33e9aa943e6489b30a979eaae1b8bd1cd1f01d  flex_methanol_d3_train_anatomy.md
543fa352bdbc3ef518daf5bcc3a8805fce805c236af907eddb280cee59a7e8dd  flex_heptane_d2_train_anatomy.md
65c329501e1443fef790cd4cf8b5974a6927dee42500929e2ea684bcfbc095e3  acme_raw_signals_train.md
d1c220653c5a664de7a31af77cafe4a4345b4f07ae58b353b10319e69e2d9b80  bass_train_tracks.md
```

This task changed only this document. It ran no model, measurement or discrepancy script,
and opened no `runs/dev/data_docs/`, protected partitions, per-run validation artifacts,
whole-report NASA downloads or model releases. Existing releases and other authors'
working files were preserved. No commit, push, validation look or qualification claim
was made. Physical error budgets remain partial where source identifiability is partial.
