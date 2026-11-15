# Astra jets v2 — enthalpy reference-path admission

Author: `openai / gpt-6-astra`. Date: 2026-11-14.

This is a **synthetic/physics-review data-informed implementation revision**. It
is not a fresh zero-shot trial, a fitted model, a NASA-qualified release, or a
change to the authored scientific numbers. The original W0 source and packet
remain intact at SHA-256
`1fbde4e90c869dfe5b85278a2fd4254cfcd91c8844db0075fbe04f82ab8ba81b`.

## Confirmed defect and independent check

The coordinator supplied finding F4 in
`docs/wsp/research/astra_property_transport_check.md`: a jet's heat-capacity
domain begins at 310 K, its common sensible-enthalpy reference is 300 K, and its
inlet temperature is 320 K. W0 admits the inlet and resulting 320 K endpoint,
but evaluates `cp * (320 - 300)` using the property over an unadmitted interval.
This is a real domain-admission failure even though the resulting arithmetic
can be finite and the conservation equations can balance.

The v2 regression independently constructs this case through the public API.
It asserts that immutable W0 accepts 320 K and that all three v2 entry points
refuse the case. It also covers the coflow property, the product property,
the molar-mass conversion domain, a reference-pressure gap, and admission exactly
on a closed domain boundary. All material values in those tests are synthetic;
they do not represent actual molecular-property determinations or NASA results.

## Correction and proof of path coverage

`property_jets_v2.rs` is a small admission wrapper. It reuses W0's public input
and output types, its literal coefficients, and its numerical/thermochemical
functions. It does not duplicate or alter the transport solver.

Before calling W0, it requires **every** jet, coflow and product component's
heat-capacity and molar-mass property domain to contain:

1. The common enthalpy reference state.
2. That reference temperature at the specified operating pressure.

Molar mass is included because a supplied molar heat capacity is converted to
a mass-specific one in the caloric balance. W0 already checks properties at
the actual inlet and reconstructed output states and enforces a shared reference
convention and common operating pressure. The property domains are closed
rectangles in temperature and pressure, with an explicit gas phase. Consequently,
admitting the reference and endpoint states admits the entire constant-property
reference-to-state path: every intermediate temperature/pressure lies inside
the same rectangle. A missing or out-of-domain interval is refused rather than
silently extended, clipped or filled.

The wrapper's public `caloric_state`, `analytic_centerline_reference` and
`forward` all apply this gate. There is no public unguarded alias for those
functions in the v2 module. Direct use of W0 remains a separately identified
historical candidate with its known F4 limitation.

For inputs with admitted reference paths, v2 delegates to the identical W0
function with the identical arguments. Exact predecessor equality tests cover
caloric states, the analytical centreline reference, equal/unequal inlet speeds,
normal/inverse streams and axial gravity. The full diagnostic record is compared,
including the retained input snapshot and numerical/conservation fields.

## Numbers, dependencies and classification

The literal operations are copied **byte-for-byte** from the frozen W0 packet:
pi `3.141592653589793`, `2.0`, and `4.0`. Their mathematical roles and original
scientific provenance remain unchanged. No physical coefficient, property value,
transport multiplier or extinction threshold was selected or revised. The new
wrapper contains domain comparisons only; the test values are explicit synthetic
fixtures and are not model coefficients.

The new spec has a distinct id and a v2 forward entry point. Its PatchMeta records
the W0 packet and the independent synthetic/physics review as exposure sources.
There are no registered candidate-release ids or formal `EvaluationResult` ids
for these artifacts, so none is invented in the corresponding metadata fields.
`revision_manifest.json` preserves the implementation's **data-informed** status
and its inherited numerical authorship explicitly. A root patch in the assembler
must not be used to relabel this implementation revision as a fresh outcome-blind
trial. No release is created by this author.

The executable code dependency closure is:

- `crates/ignis-wsp/src/property_jets_v2.rs` — new admission gate;
- `crates/ignis-wsp/src/property_jets_w0.rs` — immutable numerical law;
- `crates/ignis-wsp/src/physical_core.rs` — unchanged outcome-free algebra.

A future freeze/release must bind all three hashes; hashing only the wrapper
would omit code that determines the answer. All W0 physical and numerical
limitations persist, including conditional Boussinesq axial gravity, unsupported
transverse/electric forcing, unqualified parabolic and complete-reaction
assumptions, no optical/event mapping, and unpropagated property uncertainty.

## Exposure and supplemental verification

For this revision the coordinator's synthetic counterexample and the complete
independent transport-review report were read. That report also contained
cross-family synthetic conservation findings and admission/provenance defects;
none supplied a jets coefficient or an experimental outcome. Original W0 reading
extent remains documented in its frozen note. No NASA payload, test/validation
partition, development measurement, prior released candidate or outcome-driven
parameter search was read or run for v2.

The coordinator reported a fresh check of the **final frozen W0 bytes**:

```text
cargo test --offline -p ignis-wsp --lib property_ --quiet
CARGO_TARGET_DIR = runs/tooling/astra-target
46 property tests passed, including all 6 W0 jets tests.
Pre/post W0 SHA-256: 1fbde4e90c869dfe5b85278a2fd4254cfcd91c8844db0075fbe04f82ab8ba81b
```

This is supplemental verification, attributed to the coordinator. It does not
erase F4: the original six tests did not exercise the missing reference path.
The frozen original files are not rewritten to turn that test result into a
blanket scientific PASS.

Author-executed v2 formatting and JSON/literal-identity checks passed. The
coordinator integrated the module and reported **all five v2 tests passed** on
the wrapper identified below, including the reference-gap regression, admitted
predecessor equality and core contract materialization. The coordinator's freeze
instruction did not provide the exact invocation for that five-test run; the
receipt records this explicitly rather than inventing a command transcript.
The standalone reproduce command for those tests is
`cargo test --offline -p ignis-wsp --lib property_jets_v2`.

The independent transport reviewer then reproduced F4 against the original and
confirmed refusal through all three v2 entry points. Their fresh admitted
equal-speed, unequal-speed and signed axial-gravity cases produced exactly
identical full predecessor result records, caloric states and analytical
reference output where applicable. Their intentionally light inverse-stream
fixture exceeded the unchanged Boussinesq domain, and both versions rejected
it; that case is not represented as an admitted independent inverse-flow test.
The report records successful execution of:

```powershell
rustc --edition 2021 runs/tooling/astra-transport-review/physics_v2_independent.rs -o runs/tooling/astra-transport-review/physics_v2_independent.exe
& runs/tooling/astra-transport-review/physics_v2_independent.exe
```

These are synthetic numerical/domain checks and a physical-law review, not NASA
validation or complete scientific qualification. The independent report also
contains findings for other families; its jets acceptance does not assert that
those other findings are resolved.

## Freeze

On the coordinator's explicit instruction, the current source and dependency
bytes were rehashed before freezing. No numerical code or original W0 file was
changed during freeze.

| Dependency | SHA-256 |
|---|---|
| `property_jets_v2.rs` | `7f2fa2fd7a93ccf9c15bd81678e93c4be3499b639c4140096f16ef938ad38ff7` |
| `property_jets_w0.rs` | `1fbde4e90c869dfe5b85278a2fd4254cfcd91c8844db0075fbe04f82ab8ba81b` |
| `physical_core.rs` | `c6d4ead14d4a85ccc20113fe94fc61529d5cacf7aad73e97b7569d48e0946644` |
| Unchanged W0/v2 literal-ops bytes | `7fa8b5e8f1fc066bbaca4ab674d8e136fa71fee4590a68bd319710a4146bf0e2` |

`verification_receipt.json` binds the coordinator's attestation and independent
review snapshot/harness identities. `freeze.json` binds that receipt, the full
code dependency closure, spec, ops, PatchMeta, revision manifest and this note.
The classification remains **data-informed by synthetic/physics review, with no
NASA outcomes and unchanged authored literals**. This freeze creates no release,
qualification, commit or publication.
