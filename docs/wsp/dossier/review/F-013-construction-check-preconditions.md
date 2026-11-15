# F-013: construction checks can be reported without being executed

- **About:** new; literal assembly and strict-route evidence
- **Reviewer:** `gpt-6-astra`, 2026-11-14
- **Verdict:** confirmed counterexamples; not evidence that historical releases are invalid
- **Severity:** integrity
- **Partitions touched while reviewing:** none; synthetic in-memory construction only

## What the dossier says

Literal construction records describe specification, patch, tensor, strict-route
and forward-fixture checks. Their labels must distinguish an executed check from
a caller precondition or an absent route.

## What I did

Read `assemble::build_release`, `audit::strict_route_check_of`, their dependencies
and the shared validators. Ran `examples/astra_construction_audit.rs` on the
frozen author packet from F-012. It uses a separately valid in-memory control to
obtain literal tensors, then deliberately calls the lower-level builder with the
invalid original specification and its matching patch. It does not save either
record or access a dataset. An unknown synthetic model name exercises the route
audit independently.

## Evidence

The example command in F-012 returned:

```text
frozen_spec_validation_error = ModelSpec: geometry.pi: empty bounds
direct_build_accepted_invalid_spec = true
direct_build_checks = spec_valid: passed; patch_valid: passed; tensors_complete: passed
unknown_route_check = strict_forward_audit: passed
forecast_fixture_run = false
registry_write_performed = false
dataset_access = false
```

`assemble::build_release` records its first three checks as passed without calling
the validators. Its normal caller is expected to materialize first, which rejects
this specification. A direct caller can omit that precondition. The construction
contract also permits an empty supplied-check list; acceptance does not establish
that a forward fixture or strict audit occurred.

`banned_hits_of` uses an empty source list when the model name is unknown. The
resulting empty hit list becomes a passed strict audit. In contrast, the forward
hash routine refuses an unknown model by panicking. These are different contracts.

## What should change

Require explicit validation and materialization before every new candidate build;
require an identified, nonempty source closure and an executed meaningful fixture.
Do not treat the unknown-route result as evidence or save the deliberately invalid
audit record. Existing callers/releases need a separate review before making a
claim about their actual construction. Shared or existing implementations remain
unchanged under the owner's preservation rule; a durable API correction belongs
to their owner. New authoring paths must fail closed on these preconditions.

## Response

Astra will apply these preconditions to the new candidate path and records this
finding for the first author. No historical release has been reclassified from
the counterexample alone.
