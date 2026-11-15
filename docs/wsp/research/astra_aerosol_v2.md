# Aerosol v2: synthetic-review-informed correction

Author: GPT-6 Astra. Status: author implementation/checks complete; independent re-review pending;
not frozen, registered, qualified or NASA-evaluated. This revision is informed by
synthetic code/physics review and is distinct from the frozen zero-shot W0.

## Review exposure and confirmed defects

Read only the authorized independent transport report
`docs/wsp/research/astra_property_transport_check.md` and its aerosol reproducer
`runs/tooling/astra-transport-review/aerosol_independent.rs`, plus this author's
own predecessor source/packet. The report was marked in progress at read time;
its independent shell/jet/aerosol derivations were visible, but no other family
implementation, NASA data/outcomes, development scores or holdout was accessed.

Compiled and ran the independent reproducer unchanged against W0. It confirmed:

1. Initial particle volume fraction 2 was accepted after dilution to
   0.2706705664732254 because occupancy was checked only after a step.
2. Actual particle diameter 2e-6 m was accepted despite compact-particle support
   restricted to at most 1e-9 m.
3. The initial particle-volume input's own 1e-9 m diameter limit was bypassed.
4. A prediction initialized from `truth-reused-as-initial` could later be passed
   to `observe` with that source designated detector validation truth, because
   the prediction did not preserve its input dependency lineage.

The same reproducer retained closed coagulation volume/count balances and causal
source-line delay. Those valid behaviors must remain in v2.

Original W0 kernel SHA256:
`aaaf4a97cb92a4d278f77246391768bbaaa3afa638e23e6711ccdd9108a7f0b5`.
All original source/packet files remain byte-for-byte untouched. V2 copies the
seven existing literals unchanged; their original author, provenance and domains
remain those recorded by the original author/parameter ledger. No new fitted,
searched or generated scientific coefficient is introduced.

## Named physical branch: steady drift-diffusion absorbing film

Derivation before implementation: let x increase from the well-mixed bulk at
x=0 toward a planar perfectly absorbing wall at x=h. For constant diffusivity
D>0 and signed drift w (positive toward the wall), a steady number flux obeys
`J=w n-D dn/dx`, `dJ/dx=0`, `n(0)=N`, `n(h)=0`.
Solving the first-order equation gives
`n(x)=J/w+(N-J/w) exp(w x/D)` when w is nonzero. Applying the wall boundary gives
`J/N=w/[1-exp(-w h/D)]`.
For w=0 the solution is linear and the wall-transfer speed is exactly D/h.
The same speed acts on number and particle volume for a representative section.

Writing Pe=w h/D, the small-Pe expansion is
`J/N=(D/h)[1+Pe/2+Pe^2/12+...]`. Positive large Pe tends to w; negative large Pe
tends to zero while remaining nonnegative. The predecessor additive rule
`D/h+max(w,0)` is not this boundary-value solution: it doubles the first-order
positive-drift increment and misses suppression by drift away from the wall.

V2 exposes both branches explicitly. Existing valid predecessor scenarios use
`AdditivePredecessorApproximation` and retain that behavior. The new
`SteadyDriftDiffusionAbsorbingFilm` branch requires its own independent applicability
sample covering particle size and condition: steady planar film, constant D/w,
perfect absorption, bulk Dirichlet concentration and negligible cross-film
curvature/other transport. It is not a universal turbulent deposition law.
The diffusion-only and signed-drift limits are algebraic checks, not calibration.
Stable `expm1` forms avoid cancellation/overflow; no empirical coefficient or
parameter tuning is needed. Partial gravity remains unqualified in either branch.

## Correction boundary

Reject impossible initial occupancy before any dilution or kinetics. Validate
diameter support at the actual supplied particle size and at every evolved
representative size. Retain complete supplied source/dependency lineage through
prediction and detector observation, and recheck it against the observation's
validation truth set. The independence guarantee still depends on the caller
supplying complete transitive lineage; the kernel cannot discover omitted raw
measurement identities.

The missing shared aerosol, size-moment and detector contracts remain unresolved.
There is no counterfeit ModelSpec, spec hash, HandwrittenWeightPatch wrapper or
registered release. Private structs remain a scientific boundary only.

## Author verification (not a freeze)

Standalone compilation and all 21 synthetic tests pass: eleven retained
predecessor cases and ten review/film/context regression cases. The regression inputs
are derived from the independent reproducer, including U=2 before strong dilution,
actual d=2e-6 m against maximum 1e-9 m support, direct initial truth reuse and
transitive source-history raw-detector reuse. Additional checks reject the U=1
boundary and a coagulation-created representative size outside compact support.

The named exact-film branch reproduces its steady boundary solution, diffusion
limit, signed-drift limits, small-Pe derivative and a conservative chamber
deposition trajectory. The predecessor additive branch remains explicit and its
existing valid numerical checks still pass. Every observation now checks retained
initial/history/property/geometry/composition evidence against the observation's
truth dependencies; the result exposes this lineage read-only and carries it
forward into detector output. No numerical fitting or coefficient revision occurred.

The independent re-review then found another reproducer: keeping condition/time
labels while changing particle phase, all composition-fraction phases and optical
property phases from liquid to solid remained internally valid and was accepted.
The author read and ran the strengthened
`runs/tooling/astra-transport-review/aerosol_v2_independent.rs` unchanged to confirm
this before correcting it. V2 now retains the complete generating Context
privately and requires exact physical/provenance equality at `observe` and
`moments`, plus the original admissibility check. Only an expansion of the
validation-truth set is allowed; retained initial/history dependencies are
rechecked. The original density sample/provenance is also retained privately.
An independently sourced alternate detector remains admissible. Three additional
regressions cover the exact self-consistent phase substitution, full physical
context/provenance substitution, and legitimate alternate-detector/truth expansion.

Reproduce with the commands in
`configs/models/aerosol/property-v2/law-review-astra/README.md`.
Independent re-review remains a required gate before any v2 freeze manifest.

Coordination disclosure: after v2 implementation and its 18 tests were complete,
an agent-status tool returned sibling completion summaries (broad behavior,
file hashes and synthetic test counts). No sibling parameters, implementation
bodies, source data or NASA outcomes were shown. No physics/literal change was
made from that incidental status exposure.
