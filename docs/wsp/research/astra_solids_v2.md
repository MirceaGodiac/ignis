# Solids V2: domain admission and explicit temporal refinement

Author: `gpt-6-astra`. This new candidate is informed by synthetic/physics review;
it is **not another clean-room W0**. The predecessor source and packet are retained
byte-for-byte. No NASA outcomes or experimental final-test partitions were seen.

## Derivation before implementation

The predecessor finite-volume equations and physical universal literals remain
unchanged. For an exposed cell, its supplied material and boundary properties
must be valid at the initial, heated-intermediate and final temperature. During
the frozen heat stage temperature is linear. During the endothermic frozen-rate
reaction stage, T(s)=Th-Qreaction(s)/C(s), where Qreaction is nondecreasing and
C remains positive and nonincreasing. Therefore T decreases monotonically in
that reaction stage. Checking both ends of each stage against every closed
temperature interval proves membership of the **numerical split path**. It does
not prove that an unknown continuous physical trajectory stays in that interval.

The review's isolated constant-rate single-reactant case has
C(t)=Ci+c m0 exp(-kt), C dT=L dm, hence
T(t)=T0+(L/c) ln(C(t)/C(0)). This independent exact reference exposes a finite-step
temperature error despite exact mass and an almost-zero energy ledger residual.
The predecessor split is first order, so under smooth asymptotic convergence,
an approximation at step h has y_h=y_exact+a h+O(h^2). Consecutive discrepancies
D_h=|y_h-y_(h/2)| contract toward one half. The remaining error at the finest
resolution is approximately D_(h/2), not the energy residual.

V2 therefore offers two explicitly different numerical entry points:

- `advance_checked`: predecessor frozen split with complete numerical-path
  domain admission. It makes no new temporal-accuracy claim.
- `advance_refined`: repeat the entire requested interval from its identical
  initial state at n, 2n and 4n substeps, comparing temperature and every reactant
  inventory at synchronized nodes. Accept only if discrepancies contract within
  a declared numerical ceiling, and the geometric-tail **estimate** is below
  caller-declared temperature/mass absolute-plus-relative tolerances. Otherwise
  double n until the caller's explicit refinement work budget is exhausted.

The numerical contraction ceiling is manually selected as 0.75: it is above the
first-order asymptotic ratio 0.5 to allow pre-asymptotic effects, but below unity.
If contraction continues at no more than r, the tail after the finest mesh is
bounded by D r/(1-r). For r=0.75 this factor is 3. This is an observed-asymptotic
error estimator, **not a rigorous a-posteriori bound**: three meshes cannot prove
future contraction or exclude a shared discretization bias. It is a meaningful
separate admission guard, with an explicit receipt and caller resource limit.
No optimizer or physical-parameter tuning occurs. Whole-interval comparison avoids
representing a sum of unpropagated local indicators as a global error certificate.

Thermal CFL, algebraic conservation tolerances, time refinement, spatial error,
physical-property uncertainty and model discrepancy are separate. V2 estimates
temporal error for the same spatial mesh and fixed held controls only. Control
discontinuities must remain interval boundaries. It does not add gas-phase flame
physics, qualify spread, or certify safety.

## Read/exposure ledger and packet-local source identities

The source IDs below are aliases scoped to this packet, not claims of app database
registration. The new envelope records them as exposure sources.

- `solids-w0-author-source`: complete own predecessor kernel
  `crates/ignis-wsp/src/property_solids_w0.rs`, including its synthetic tests.
- `solids-w0-author-packet`: own predecessor specification, literal assignments,
  metadata and derivation produced in this author session.
- `thermal-review-synthetic-20261114`: complete
  `docs/wsp/research/astra_property_thermal_check.md`. Includes the solid domain
  counterexample and logarithmic-capacity solution, and incidental descriptions
  of other property candidates and their limitations. No other candidate source
  was opened. This synthetic/physics feedback is disclosed, not erased.
- `thermal-review-domain-repro`: complete
  `runs/tooling/astra-thermal-review/domain.rs`, an independently authored synthetic
  reproducer against the predecessor public API.
- `nist-fds-1018e6-solid-theory`: inherited predecessor reading,
  https://nvlpubs.nist.gov/nistpubs/SpecialPublications/NIST.SP.1018e6.pdf,
  intentional sections 7.1–7.2 and incidental excerpts recorded in the W0 ledger.
- `nist-molar-gas-constant`: inherited predecessor NIST constants excerpts,
  https://physics.nist.gov/cuu/pdf/all.pdf and
  https://pages.nist.gov/feasst/docs/source/physical_constants.html.
- `nist-stefan-boltzmann`: inherited predecessor NIST constant excerpt,
  https://physics.nist.gov/cgi-bin/cuu/Value?sigma%7Csearch_for=stefan+boltzmann.

No new web reading. No NASA properties, modeled per-test outcomes, other source
trees, datasets, held-out test, development evaluation or app-registered release
was read. The review refers to Cantera general equations; that page was not opened
by this author. Source-only review exposure is represented in the envelope;
`development_feedback` and `prior_candidates` cannot invent EvaluationId or
ReleaseId records for unregistered local source packets.

## Number and method inventory

Inherited physical literals: R=8.31446261815324 J/(mol K) and
sigma=5.670374419e-8 W/(m2 K4), byte-identical ops to the predecessor.
New authored numerical literal: contraction ceiling 0.75, dimensionless, explicit
numerical origin; not a physical property or a fitted coefficient.
Integers 2 and 4 define binary mesh refinement; the error-tail factor is exact
algebra r/(1-r), not another tuned number. All caller tolerances and work budgets
are numerical request inputs. Physical properties remain mandatory sourced inputs.
There is no temporal-error default hidden in the kernel.

Implementation, verification and independent re-review receipts follow. No freeze
is produced before verification and re-review.

## Author verification

`rustc --edition 2021 --test crates/ignis-wsp/src/property_solids_v2.rs -o configs/models/solids/property-v2/law-review-astra/solids-v2-check.exe`
then the executable with `--nocapture`: **16 passed, 0 failed**, test-runner
duration 0.24 s. The generated executable/PDB were removed afterward.

The first run had 15 passes and one work-budget exhaustion: the nonlinear-E
fixture requested 1e-8 kg despite its explicit check requiring 1e-5 kg accuracy.
The fixture now declares 1e-5 kg, matching that independent comparison. Physical
properties and the estimator policy did not change. Budget exhaustion remains
covered by a separate regression asserting unchanged state.

`rustfmt --edition 2021 --check` on the new kernel and contract harness passed.
All seven hashes in the predecessor `freeze.json` were rechecked and match;
the original source and packet remain unchanged.

Six new synthetic regressions supplement the ten inherited invariants:
boundary-domain counterexample in both modes; invalid hot intermediate with
valid final temperature and narrow back boundary; invalid initial/cold endpoint;
exact logarithmic-capacity solution under two requested tolerances; atomic work
exhaustion and exact inert heating; nonzero-activation-energy comparison against
32,768 fixed time steps. These are numerical checks, not material outcomes.

## Coordinator verification and independent re-review receipts

The coordinator supplied the following final receipts for kernel SHA-256
`efc43f07e245a8fa8ec8148810e3f4d45f56e550248e721c253046d83ebc5b09`:

- Integrated V2 module tests: **16 passed**.
- Actual shared-core `literal_contract` harness: **1 passed**.
- Independent thermal re-review: passed at that exact kernel hash. The original
  boundary-domain counterexample and hot intermediate path now reject atomically;
  valid `advance_checked` behavior agrees with the predecessor; refinement and
  work-budget checks passed. The observed estimator remains explicitly distinct
  from a rigorous bound.

These are coordinator-reported receipts, not additional commands executed by the
author. The final coordinator message did not include the integrated invocation,
so no unobserved command or duration is asserted here. The author's separate
standalone and formatting commands above were executed directly.

Before freezing, the author independently rehashed the current kernel and
confirmed the exact reviewed hash. All seven predecessor freeze entries were
rechecked again and still match. No numerical changes followed review.

Status: **frozen after verification and re-review**. `freeze.json` inventories
the exact kernel, new specification/ops/metadata, packet source inventory,
author report, test harness, manifest and lockfile. It separately records hashes
of the reviewed source/provenance dependencies and shared-contract build inputs
as they exist at freeze time. Those dependency hashes identify the snapshot;
they do not replace the coordinator's test receipt or certify unexecuted checks.
The kernel itself is std-only; no external model/property data is embedded.

Classification remains **informed by synthetic/physics review**, not independent
W0. No NASA outcome feedback, experiment data, final-test exposure or real-material
qualification is claimed. Numerical acceptance does not establish autonomous
flame spread, a rigorous temporal bound, spatial accuracy or safety qualification.
