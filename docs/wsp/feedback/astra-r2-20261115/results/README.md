# Astra R2: frozen TRAIN results and remaining scientific gaps

Protocol and handwritten candidates committed **before scoring**.
Four new releases were authored by explicitly selected GPT-6 Astra high-effort
physical-author agents. All are **data-informed**, not zero-shot. No optimizer,
regression, parameter search or sweep selected a number. No weight changed after
the freeze. The existing BASS joint candidate was retained without a new release.

**Result: R2 removes the ACME regression, but produces no new win over Fable.**
The droplet heat-capacity correction has negligible benefit for methanol and a
small adverse effect for heptane. A correct species assignment is not evidence
that the whole thermal closure predicts better. BASS's earlier joint gain remains
uncertain because its scored tracks share one independent day.

## Scores on the identical frozen TRAIN cases

Lower is better. Values are equal-weight session/split-cluster means. FLEX CRPS
is in micrometres; ACME values are integrated IPCW Brier contributions, not
case-weighted headline Brier scores. Termination is `operator_termination` in the
scorer. The canonical paired report is `summary.json`, keyed by release ID;
the legacy scorer's model-name-keyed `paired_vs_first` map cannot distinguish
multiple candidates with the same model name and was not used for verdicts.

| Family / metric | Fable | Astra R1 | Astra R2 | Result |
|---|---:|---:|---:|---|
| Methanol CRPS | 48.446566 | 48.476691 | 48.476644 | Negligible change; still above Fable |
| Heptane CRPS | 51.084312 | 51.211739 | 51.215472 | Slightly worse than R1 and Fable |
| Normal extinction | 0.063130590 | 0.064068590 | 0.063130590 | Fable performance restored |
| Normal termination | 0.028018480 | 0.028355717 | 0.028018480 | Fable performance restored |
| Inverse extinction | 0.021341771 | 0.022029542 | 0.021341771 | Fable performance restored |
| Inverse termination | 0.095746681 | 0.095754945 | 0.095746681 | Fable performance restored |

Methanol R2 minus R1 CRPS is **-0.0000472 micrometres**, with descriptive 95%
cluster interval [-0.0003397, +0.0002043]. R2 minus Fable is +0.0300782,
interval [-0.0430990, +0.1315277]. This is not a useful accuracy improvement.

Heptane R2 minus R1 CRPS is **+0.0037332 micrometres**, interval
[-0.0002952, +0.0114959]. R2 minus Fable is +0.1311601, interval
[-0.0067801, +0.3988742]. Median absolute error also worsens: R2 minus R1
+0.0060595 micrometres, interval [+0.0000705, +0.0177163]. These exploratory,
unadjusted intervals do not prove a population-wide CRPS degradation or equality.

Both ACME R2 scored case records match Fable exactly, including exposed scores
and end-time probabilities. Normal R2 removes R1's excess extinction contribution
0.0009380 and termination contribution 0.0003372. Inverse removes the smaller,
uncertain excesses 0.0006878 and 0.0000083. This is an explicit nominal-input
rollback, not a newly superior combustion law.

The unchanged joint BASS candidate retains CRPS **0.0734391**, versus Fable
**0.0761009** (3.50% lower); median absolute error and track-ended Brier also
improve. There are only three scored tracks, twelve cutoffs, and **one day**.
No independent-session uncertainty interval or new BASS gain is claimed.

## Coverage and reproduction

All releases in each family have identical admission and scored coverage.
No R2 loses a baseline case or obtains an advantage by refusing difficult cases.

| Family | Scored cases | Refusals | Other exclusions | Independent clusters |
|---|---:|---:|---|---:|
| Methanol | 182 | 3 | 15 not-at-risk; 4 forecasts lack a valid continuation target | 26 |
| Heptane | 125 | 9 | 13 not-at-risk | 20 |
| ACME normal | 210 | 0 | 33 not-at-risk | 7 |
| ACME inverse | 204 | 0 | 15 not-at-risk | 7 |
| BASS | 12 | 0 | One of four tracks lacks six flame stills | 1 |

The pressure/composition conflicts causing FLEX refusals were not repaired or
zero-filled. Inputs, targets, horizons, censoring rules and all prior weights
remained fixed. The fifteen historical baseline/R1/joint release case records
reproduce exactly. Independent arithmetic recomputation checked **58 paired
point estimates**; all seventeen scoring-output hashes in `receipt.json` match.
`verification.json` records these checks. The executable snapshot, commands,
timestamps and protocol commit are in the family execution receipts.

Passed before the protocol commit: formatting check, workspace all-target
compilation, core/WSP/scorer offline tests, candidate assembly and literal-delta
audit, and synthetic smoke forecasts for all four new releases. Source hashes
and record-file metadata matched before and after scoring. Raw logs retain
Windows line endings so their recorded hashes remain valid.

## Why the additions lost, and what is missing for a real improvement

The TRAIN evidence localizes losses; it does not uniquely identify every physical
cause. The detailed pre-score analyses are in
`../../../research/astra_r2_droplet_loss_analysis.md`,
`../../../research/astra_r2_acme_loss_analysis.md`, and
`../../../research/astra_r2_bass_loss_analysis.md`.

**Droplets: the dominant missing information is thermal and flame history, not
a more precise standalone constant.** R1 introduces a post-flame mass/energy
law, but its liquid starting temperature, residual gas heating, internal
temperature gradients and transition history are not independently identified.
Methanol's dominant affected case predicts too large a diameter; heptane's
dominant case too small. A global evaporation adjustment is not a justified
repair. The vapor-Cp correction is physically better specified, but this retest
shows it does not address the dominant forecast errors.

A defensible larger advance needs prospective evidence for those states, or
independently constrained dynamic equations with explicit uncertainty when
states are not observed. Establish what the available instruments can actually
identify; do not invent a temperature measurement or recover a hidden state by
fitting its future diameter. Compare separately defined phase-history hypotheses
before attempting a more detailed transport model. More state variables without
identifiable initial/boundary conditions can simply add more wrong assumptions.

**ACME: sensitivity scenarios need evidence before they become probabilities.**
Equal masses at nominal and perturbed flow/pressure changed the predicted event
probabilities, not just an uncertainty display. The inherited clocks and risk
boundaries are nonlinear, so symmetric condition offsets do not cancel. Normal
losses span all seven TRAIN days; inverse losses concentrate around August 16.
Overlapping uncertainty already represented by the inherited law is plausible,
but was not separately identified. R2 removes the unsupported probability layer.

To improve on Fable, establish admissible additional inputs (such as independently
documented prospective control schedules and calibrated readback conditions),
and the timing, units and provenance of each. Input uncertainty requires an
independently justified error/covariance model, not a narrower scenario width
chosen because it scores better. Alternatively, a genuinely new physical law
must address a localized failure while retaining correct limits. Neither changing
an input table based on the flame outcome nor retuning the existing event clock
to its residual is a defensible shortcut.

**BASS: transient observation and transport closure need a new forward, not a
different mixture weight.** The steady oxygen balance is dimensionally coherent,
but combines ambient density, hot-layer transport and `L = v tau`; it does not
derive transient spread or the visible-front/base relationship. Early and late
cutoffs lose while intermediate cases mix gains and losses. The prefix's tip/base
proxies sometimes disagree. There is also a testable mathematical approximation:
the current encoder evaluates a nonlinear modeled path at a window's mean time
instead of averaging that path at the actual observation times.

A separately named forward could correct that observation operator, with an
invariant that it preserves linear/constant paths, and test a physically defined
transient inventory. Its constants still need independent source or apparatus
justification. Additional independent TRAIN days or an evaluator-authorized
unseen comparison are needed to judge transfer; this author must not open sealed
validation/test tracks to get them. The current joint gain is insufficient to
justify new mixture or uncertainty weights.

**What counts as beating Fable:** a frozen candidate improves the agreed scores
without losing coverage, survives relevant physical/invariance checks, and then
has evidence on admissible independent groups. A lower TRAIN mean alone, especially
after development feedback, is not that evidence. More handwritten parameters or
more elaborate physics by itself is not an advantage.

No existing release was replaced or adopted automatically. All negative results
remain. New clean-room property kernels are still outside this registered-release
comparison because their adapter/input prerequisites are not complete. No new
validation look, protected test access, push, PR or deployment occurred.
