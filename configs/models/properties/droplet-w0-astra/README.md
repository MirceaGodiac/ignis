# Frozen-author inputs: property droplet Astra W0

- `spec.json`: ModelSpec declaration for the new unregistered scientific kernel.
- `literal-ops.json`: complete manually typed literal assignments; no generated coefficients.
- `author-envelope.json`: exposure/identity source for the coordinator to construct the
  final HandwrittenWeightPatch with the owning canonical spec hash and unchanged ops.
- `freeze.json`: SHA-256 byte hashes after synthetic verification, before any outcome feedback.
- `verification.json`: commands actually executed and synthetic-only results.

The scientific input/output types and balances are in
`crates/ignis-wsp/src/property_droplet_w0.rs`; derivations and numerical origins are in
`docs/wsp/research/astra_property_droplet_w0.md`. These are new paths only.

No module registration, CLI/forecast adapter, existing forward, shared contract, release,
dataset, score or evaluation record was changed by the numerical author. The extra
analytic APIs explicitly describe a thermostatted isothermal pure-liquid limit and an
existing ideal flame-sheet limit; neither produces an optical or finite-chemistry label.

Standalone synthetic reproduction (from repository root, Windows PowerShell):

```powershell
rustfmt --edition 2021 --check crates/ignis-wsp/src/property_droplet_w0.rs
rustc --edition 2021 --test crates/ignis-wsp/src/property_droplet_w0.rs -o configs/models/properties/droplet-w0-astra/synthetic-tests.exe
& .\configs\models\properties\droplet-w0-astra\synthetic-tests.exe --nocapture
```

The temporary executable/PDB are not candidate source and are removed after verification.
No experimental score is produced. Downstream compatibility and schema-envelope checks
remain coordinator work and must not be represented as already passed here.
