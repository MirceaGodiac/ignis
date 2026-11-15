# F-012: the frozen droplet W0 specification is not valid

- **About:** new; Astra's own frozen property-droplet packet, not a Fable model
- **Reviewer:** `gpt-6-astra`, 2026-11-14, following the separately recorded engineering audit
- **Verdict:** confirmed invalid specification; scientific accuracy not tested
- **Severity:** model
- **Partitions touched while reviewing:** none; author packet and synthetic construction only

## What the dossier says

The packet under `configs/models/properties/droplet-w0-astra/` is a frozen,
zero-shot numerical kernel, not a registered release. Its JSON specification
sets both bounds of `geometry.pi` to the same mathematical value.

## What I did

Read the integration audit, the frozen specification/envelope/ops and the actual
Rust `ModelSpec` and patch validators. Added an outcome-free reproducible example
that reads only those fixed paths. Ran the example through the existing private
Cargo target. It saves no release and reads no dataset.

## Evidence

Reproduce with `cargo run --offline -p ignis-wsp --example
astra_construction_audit -- --execute`, using Astra's private `CARGO_TARGET_DIR`.
Both `ModelSpec::validate` and `assemble::materialize` return
`ModelSpec: geometry.pi: empty bounds`. The owning Rust canonical spec hash is
`sha256:15bd54f7610888b9c5b243ac99322630e58c2e4a498789ef9137bc1c205921b8`.

Removing the bound from an in-memory control changes the canonical hash to
`sha256:03d629cfdd9417bb577dfe0724781cd7056ff6e9ecfa6d011688d5a0c807ee19`.
That control materializes the original literal operation, but is a different
specification. It is not saved, qualified or substituted for the frozen packet.

## What should change

Preserve the refused packet. A valid candidate needs a separately identified
specification; shared validators must not be weakened to accept this packet.
The newly commissioned independent Astra droplet candidate has separate paths,
source and exposure records. Its prospective acceptance remains unproved until
its own exact specification, patch, forward and fixtures pass.

## Response

Astra accepts the finding. The ledger names the refusal. No original weight,
forward, specification or freeze hash was changed.
