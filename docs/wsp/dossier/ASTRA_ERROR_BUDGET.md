# Astra: train error budget and ranked extensions

Author: **gpt-6-astra**, explicitly selected for analysis alongside Fable. Date: 2026-11-14.
All candidates informed by this document are **data-informed, not zero-shot**.

**Chronology:** the first module prototypes had already been written when this quantified budget
was prepared. Candidate weights were still unwritten at this document's preparation. This is a
pre-weight analysis, not a backdated pre-implementation registration. It constrains the proposed
scientific claims and the subsequent first candidate; it does not retroactively make the initial
architecture choices blind to the dossier.

**Evidence status:** numbers below are extracted from the named, allowed train-only reports. I
read and compared their statements; I did not replay forecasts, remeasure images, fit anything,
download raw files, or write weights. Their reported numerical checks remain the original
analysts' checks. The reports concern earlier D1/D2/D3 models, not a fresh error decomposition of
the currently carried fifth-generation ACME releases. Their gains cannot be transferred to a new
candidate by subtraction. No validation case, sealed continuation, test/holdout artifact or
contaminated `exp_*` dossier was opened for this budget.

## 1. Measured concentrations, not promised recoverable gains

| Family and historical reference | Reported train budget | What carries it | Meaning for this generation |
|---|---|---|---|
| Methanol D3 | Mean CRPS **29.9 µm**, sum 5436 µm over **182 cases** | Artefact class: **38.8%**, 15 cases. Extinction within horizon: **27.3%**, 57 cases. Already out at cutoff: **4.9%**, 8 cases. Burning through horizon: **27.6%**, 96 cases | A post-flame law can affect already-out cases and the post-extinction part of transition cases, not all error in those classes. The fraction specifically due to a deficient nitrogen/hot-radiative evaporation law is **not measured** |
| Heptane D2 | Mean CRPS **46.2 µm**, sum 5770 µm over **125 cases** | Problem runs: **57.7%**, 20 cases. Artefact-labelled cases: about **57%** overall, overlapping the problem category. Hot-to-helium: **7.2%**, 9 cases in **one session** | Helium is the largest identified physics problem in this audit, but the proposed nitrogen-dominated hot-radiative law deliberately does not solve it. No numeric gain for the admitted nitrogen subset is established |
| ACME normal D3 | Extinction/termination IBS **0.05159 / 0.04125**, **210 cases** in the transfer report | Five runs carry **52% / 64%** of the respective summed losses. `21125A2` alone carries **26% / 26%**; cycling `21291A1/C1` add **20%** of extinction loss | Regime, input and event-meaning mistakes have disproportionate influence. Persistent uncertainty in measured conditions cannot by itself repair wrong labels or missing mechanisms |
| ACME inverse D1 | Extinction/termination IBS **0.05724 / 0.06682**, **204 cases** | Five runs carry **64%** of extinction loss; `21228C2` contributes **24%**, and `21196F1/F2/F6` together **35%** | The latter cluster was a weak-class mistake. It is evidence to examine regimes, not evidence that adding arbitrary condition spread will improve the current model |
| BASS train tracks | Three spreading runs (`T5/T6/T16`), one non-spreading case (`T10`) | The three spreading runs share one day. Low-flow steady tip speeds are about **0.033–0.043 sample widths/s**; the early phases and step response are strongly transient | These measurement reports do **not** apportion the model's CRPS by mechanism. They support a steady-limit check, not a quantified attainable gain or independent cross-session transfer estimate |

Sources: [methanol anatomy](../feedback/flex_methanol_d3_train_anatomy.md) §1;
[heptane anatomy](../feedback/flex_heptane_d2_train_anatomy.md) §1;
[ACME transfer audit](../feedback/acme_transfer_audit.md) §1;
[BASS tracks](../feedback/bass_train_tracks.md) §§b–e.
Case counts and run counts differ because several cutoffs belong to the same run. CRPS in µm,
CRPS in sample widths, and event IBS are not comparable numerical scales.

### Droplet timing, rate and measurement limits

For methanol, leads of **5–10 s carry 47%** of total CRPS and **62%** of the non-artefact remainder.
The audit estimates extinction timing contributes about **5% of total CRPS**, roughly **1.5 µm**
of the case mean, using an attribution comparison rather than a modified forward. Much of the
error after a transition is rate drift that would remain even with better timing. At cutoff 5 s,
end-rate ordering persists into the next 3 s across 9–10 sessions per tercile; the report finds no
such ordering at cutoff 1 s. Thus a late-prefix measurement and an early-prefix physical prior
have different information budgets. [Methanol anatomy](../feedback/flex_methanol_d3_train_anatomy.md),
§§1–3 and 6.

Concrete methanol failures include `H15M102@5` (**122 µm**, end-rate windows straddle extinction and
the filter wrongly restores the hot state), `H05M102@5` (**52 µm**, implied radiative ending **2.3 s**
late), and `175R001@5` (**46 µm**, **3.7 s** late). Near the radiative boundary the report attributes
late implied endings to mass retained on the long-lived diffusive branch. A better evaporation
integrator does not fix that upstream state assignment. [Methanol anatomy](../feedback/flex_methanol_d3_train_anatomy.md),
§§1–2 and 7.

For heptane, the non-helium hot-flame ending is already close to unbiased: median timing error
between **−0.06 and +0.04 s** over 25 cases, 13 runs and 10 sessions. In the report's approximate
attribution, hot-to-cool cases assign **54%** of their loss to the next-stage rate and **19%** to
timing. Helium endings are **2.7 s late** at the median over 8 timed cases in 3 runs/1 session;
`H12H101@5` scores **91 µm**, and `H13H201@5` **76 µm**. The high-pressure/high-CO2 run `248F001`
scores **269/184/65 µm** at its three cutoffs. Both regimes are outside the proposed narrow cooling
scope. Cool-flame offsets can also be misread during the early dip (`C04H201`, `N04H101`); a hot-
radiative post-flame law should not silently replace that chemistry. The attribution uses a
mixture median as a path and is explicitly approximate. [Heptane anatomy](../feedback/flex_heptane_d2_train_anatomy.md),
§§1–5.

### What the tracker budget does—and does not—bound

- **Methanol:** artefact cases carry **2109 µm** of the summed score. Prefix rules identify 5 cases
  carrying **1220 µm**, with two runs dominating: `240R003` (bubble) and `40CAL01` (view loss/re-entry).
  Wrong-atmosphere `C12M401`, later view loss `H12M101` and future dropouts can have clean prefixes.
  This is loss *on flagged cases*, not loss that a causal detector is guaranteed to remove.
- **Heptane:** 32 artefact cases carry **3279 µm**. A union of prefix signatures finds 16 of them
  carrying **2857 µm**, but also fires on **12 clean cases** carrying 223 µm. The uncaught artefact
  cases carry **422 µm**. `195F002@1`, `195F001@3` and `20CAL13@1` alone score **1023, 744 and
  689 µm**; soot/glare distort the measured target as well as the prefix.
- **No physical perfect-law score is identified.** Methanol's roughly **5.5 µm** short-lead score
  is attributed to its assumed noise width (including the 20 µm noise term); reported frame scatter
  is only about **1.5 µm**. That is a model-imposed score contribution, not irreducible sensor error.
  Heptane's **13.1 µm** “band moved onto the data” diagnostic uses the future observed value to
  recenter every forecast. It isolates width under an oracle operation; it is neither an attainable
  causal score nor a guaranteed error floor. Narrowing on train may harm unseen artefacts or regimes.

Sources: [methanol anatomy](../feedback/flex_methanol_d3_train_anatomy.md) §§4–5;
[heptane anatomy](../feedback/flex_heptane_d2_train_anatomy.md) §§1, 6–7.

BASS has measurement scales rather than a quantified irreducible forecast floor: **0.018 widths
per pixel**, about **±0.05 widths** in the between-test origin, and approximately **±0.1–0.2 widths**
for the duct-to-top-view conversion. Near-zero detrended scatter reflects quantization and does
not prove a noise-free trajectory. Still/overlay timing uncertainty is **0.33/0.78/1.58 s** for
T5/T6/T16. Origin and scale errors can be correlated between prefix and target, so these quantities
cannot simply be added to produce a CRPS floor. [BASS tracks](../feedback/bass_train_tracks.md),
§§c, g–h.

## 2. Highest input priority: the prospective control plan

**Priority 0, deferred for provenance acceptance:** a genuinely pre-ignition ACME stop plan is more
directly actionable than a refined physical clock for predicting administrative flow termination.
The older audit attributes **12–13 of 81 normal runs** and about **30 of 73 inverse runs** to facts
outside its input bundle, including stops, apparatus state and other causes; these counts are not
all scheduled-stop cases. The later raw-signal report identifies **29 scripted ends**. An
independent train recount finds actual stop minus stage sum between **−1.3 and +1.7 s**, median
**−0.4 s**; in **99** early-ending rows the script target remains later than the recorded end.
([ACME D1 audit](../feedback/acme_d1_train_audit.md), §F;
[raw signals](../feedback/acme_raw_signals_train.md), §5;
[F-001 recount](review/F-001-programmed-stop.md).)

These measurements verify a timing relationship. They do **not** prove that an archived “as run”
file existed unchanged before ignition. The new generation therefore does not turn that proxy
into an unconditionally admissible prospective input. Acceptance requires source-format/temporal
provenance evidence from the data/evaluation owners. Until then, use the shared-record fallback
and label any plan-conditioned result as conditional on that acceptance. No numerical unseen
gain is promised. Control termination and physical extinction remain distinct targets.

A second input priority is trustworthy cutoff-local flow/composition readback. The raw-signal
report finds table/readback discrepancies in `21125K1/K2/K3`, `21106G6/G7/G9`, and `21196M2_1`,
with uncertain N2-delivery discrepancies in `21228C4/K2` and `21256C2`. Small condition nodes cannot
stand in for correcting an order-of-magnitude input error. Their raw channel interpretation and
causal ingestion need separate review; no shared record is rewritten here.

## 3. Ranked chosen extensions before their weights

This ranking is **scientific implementation priority**, not a cross-family numerical gain ranking.
Every expected unseen effect is qualitative, may be zero or negative, and remains unmeasured.

| Rank / status | Chosen change and measurable target | Why build it | Expected unseen effect and decisive limitation |
|---|---|---|---|
| **1 — prototype already written; weights not yet authored at preparation** | **Persistent-condition ACME committee.** Carry one composition/flow/pressure scenario through the complete run and combine complete event distributions | Inputs and class boundaries matter greatly; historical loss is concentrated in a few regime failures. This supplies coherent uncertainty around uncertain conditions without independent horizon-by-horizon worlds | May reduce overconfidence near supported boundaries, but may blur correct predictions. The transfer audit's ±0.4 log-clock shifts move normal extinction IBS from about .052 to .067–.077 and inverse from .057 to .062–.070: generic session broadening did not explain the problem. This is not evidence of a positive gain for condition nodes, which perturb a different object. Cannot cure labels, gross input errors, false mechanisms or unavailable plans |
| **2 — prototype already written; weights not yet authored at preparation** | **Coupled mass/energy cooling after a hot radiative ending in the admitted nitrogen-dominated droplet regime.** Keep other regimes on the predecessor | Replaces an approximate decay closure by an explicit coupled state and checks a physically meaningful limit; methanol already-out and post-transition cases give a real target | Possible better size/time transfer of evaporation within scope. Exact scope-specific loss share is unknown, so no µm saving is justified. It deliberately leaves heptane helium, high-CO2/high-pressure anomalies, cool-flame history and methanol composition uncertainty unresolved. Wrong state/timing can erase any benefit |
| **3 — prototype already written; weights not yet authored at preparation** | **BASS oxygen-supply steady closure.** Preserve prefix anchoring and finite flame-length response, compare settled speed/length relations | T5/T6/T16 have similar settled tip/base speeds, and post-step length/speed ratios are **9.4–10.4 s**. A conservation-based steady law has a checkable implication beyond an empirical speed coefficient | Possible transfer of steady dependence where physical flow and geometry are known; no quantified CRPS saving. Three spreading runs on one day do not identify a general oxygen/flow law. Low fan values are partly extrapolated; pre-step phases are ignition transients. A steady relation does not justify transient behavior or extinction probabilities |

Sources for rank 1: [ACME transfer audit](../feedback/acme_transfer_audit.md) §§1, 6–7 and
[raw signals](../feedback/acme_raw_signals_train.md) §§4, 7. Sources for ranks 2 and 3 are the
anatomy/track sections above. The chosen scope is a restriction, not evidence that the admitted
subset has already shown a gain.

**Why the largest scored components are not this generation's main physics changes.** Tracker
artefacts dominate droplet loss but require trustworthy observation semantics; a transport law
cannot predict a future tracker failure. Some old audits propose narrower bands or sharper class
boundaries and attach hand-estimated savings. Those are not executed candidate gains and are not
adopted as this budget's forecast. ACME class/label failures require physical and observational
discrimination; input spread is not a substitute. BASS T5 retreats **0.21 widths** during its flow
step and recovers its pre-step tip position after **11.7 s**, while T6 stalls and T16 barely stalls:
the steady closure must leave that response mechanism explicit. T10 provides one non-spreading
case, not an identified probabilistic ignition/extinction boundary.

## 4. What the first train checks must establish

1. **Zero new scientific claims from neutral behavior:** each disabled extension reproduces its
   predecessor; complete literals and actual ancestry remain auditable.
2. **ACME:** coherent nonnegative event mass/survival across time; changes are localized to their
   declared condition assumptions. Report harmful as well as helpful regimes. Do not infer a node
   width from the historical loss table or choose it by scanning scores.
3. **Droplets:** mass/energy and time-step checks first; then report the admitted hot-radiative
   nitrogen subset separately from the total. Record unsupported/unchanged regimes explicitly.
   Do not call an overall change evidence that the physical post-flame explanation is correct.
4. **BASS:** steady-limit and transient checks separately. Preserve sample-width observables,
   declare fan-calibration assumptions and distinguish track loss from physical flame loss.
5. Preserve the first candidate and every result. An implementation correction follows a stated
   invariant; a scientific revision after feedback receives its own identity and exposure. This
   budget authorizes no validation look and asserts no new attainable score, calibrated coverage
   or promoted release.

**Reading boundary:** the two droplet anatomy reports and both ACME audit reports were read in
full; the ACME raw-signal report's substantive sections and BASS track sections a–i were read.
The much longer BASS measurement report was read selectively for measurement resolution, flow
calibration, step response and target limitations; its complete frame tables were not reviewed.
No result in this document was independently remeasured.
