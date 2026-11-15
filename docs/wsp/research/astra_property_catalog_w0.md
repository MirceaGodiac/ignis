# Shared property catalog W0 — source-bound foundation

Author: `gpt-6-astra`. Date: 2026-11-14. Outcome-blind, zero-shot transcription and fixed-law implementation. This is an internal property dataset, **not a model release, trained model, validated combustion model, or complete NASA material inventory**.

The usable physical values are four SI defining constants and two derived constants. Mathematical pi is separately qualified. Six real material heat-capacity records are present but non-admitted. No empirical material property has yet satisfied the required independent-source and state checks; none can currently be returned by `evaluate`.

## Files and interface

- `crates/ignis-wsp/src/property_catalog.rs`: internal serde types, checked reader, evaluator, derivations, invariant tests.
- `configs/models/properties/catalog-w0-astra/catalog.json`: manually typed source values, composition identities, locators, uncertainties, states and admission decisions.
- `configs/models/properties/catalog-w0-astra/catalog.sha256`: SHA-256 pin of those exact JSON bytes.

Dataset SHA-256: `982c31048f31e42d5657890aa9784aacd7a490c433f644b581b3f4af04b3c0b4`.

This pins the **manual transcription dataset**, not an archived original publication or the future contents of a remote URL. The property values and source locators were checked against the displayed source excerpts before hashing. No entire external database or coefficient array was downloaded.

Root integration adds `pub mod property_catalog;`; this author's ownership does not include `lib.rs`. Consumers use:

```rust
use ignis_wsp::property_catalog::{Catalog, Domain};
let catalog = Catalog::bundled()?;
let k = catalog.evaluate("constant.k_B", &Domain::Universal)?;
let r = catalog.molar_gas_constant()?;
let sigma = catalog.stefan_boltzmann_constant()?;
```

`Catalog::from_json`, `validate`, `records`, `record`, `materials`, and `material` are also available. The returned `EvaluatedProperty` includes `value_si`, a closed `Unit`, `dependencies`, and a qualification string. Data are not shared wire contracts, and no schema/TS changes are needed.

Material strings are lookup identities. The reader never predicts from a name. A point query must match temperature, pressure, phase and pure-component identity exactly. A missing pressure remains `null`, and prevents admission. No range, interpolation, extrapolation, ideal-gas limit, enthalpy zero, isotopic composition or formulation is inferred. A property at one temperature is not a correlation. Inspection of a pending row must not be used to bypass `evaluate` in an adapter.

## Admitted values and dependency inventory

| Identity | Literal raw/SI value | Admission | Shared dependence |
|---|---:|---|---|
| `constant.k_B` | 1.380649e-23 J/K | SI exact definition | `si-2019-definitions` |
| `constant.N_A` | 6.02214076e23 mol^-1 | SI exact definition | `si-2019-definitions` |
| `constant.h` | 6.62607015e-34 J s | SI exact definition | `si-2019-definitions` |
| `constant.c` | 299792458 m/s | SI exact vacuum definition | `si-2019-definitions` |
| `mathematics.pi` | 3.141592653589793 | Mathematical binary64 approximation | `mathematical-circle-constant` |

The SI values were read from the [BIPM defining-constants table](https://www.bipm.org/en/measurement-units/si-defining-constants) and compared numerically with the corresponding rows of the [NIST 2022 CODATA listing](https://physics.nist.gov/cuu/Constants/Table/allascii.txt). These are two authoritative statements of one definition, explicitly **not independent measurements**. Planck's J Hz^-1 in NIST is dimensionally identical to BIPM's J s. Temperature, pressure, phase and material composition are inapplicable to these universal definitions; c is the vacuum constant.

Pi's digits and definition were read from [DLMF 3.12.1–3.12.2](https://dlmf.nist.gov/3.12) and checked against the [Rust standard-library declaration](https://doc.rust-lang.org/std/f64/consts/constant.PI.html). Its `5e-16` absolute bound is a conservative manually stated numerical representation enclosure at the stored precision; it is not experimental uncertainty. Mathematical concordance is a separate admission kind and cannot admit an empirical material property.

R is evaluated using `N_A*k_B`; sigma is evaluated using `2*pi^5*k_B^4/(15*h^3*c^2)`. Relations and the printed decimal comparison are from [NIST SP 961, May 2024, page 1](https://physics.nist.gov/cuu/pdf/wall_2022.pdf), rows for molar gas constant and Stefan–Boltzmann constant. The sigma expression substitutes `hbar=h/(2*pi)` into the printed expression. Integers 2, 3, 4, 5 and 15 are exact algebraic constants of that relation, not empirical coefficients. Algebraic rearrangement avoids taking a fourth power of the tiny dimensional Boltzmann constant. Returned values are f64 evaluations of exact defining relations; no exact-decimal-arithmetic claim is made. Dependency lists retain each input record identity, deduplicate the shared SI definition, and add the relation identity. Reusing these constants in six families is shared dependence, not six independent sources.

No material correlation coefficient, predictive free weight, stochastic value or fitted value is included. Consequently this source catalog does not itself supply a `HandwrittenWeightPatch` or `ModelSpec`. A consuming release must bind its source dependency and its own literal construction separately; the catalog does not bypass model construction checks.

## Material scope and pending properties

All 15 requested prospective identities are recorded with source formula, integer atom inventory in C/H/O/N/S/He/Ar order, exact WebBook locator and caveat. Formula counts are literal structural identities; absent elements are structural zeros, not missing measurements.

| Material | Actual thermal sample in catalog | Status |
|---|---|---|
| Oxygen, nitrogen, helium, argon, carbon dioxide, water | None | Identity only; thermal and transport data pending |
| Methane | Gas Cp 35.69 J/(mol K), 298.15 K, 1 bar | Gurvich et al. 1989 via WebBook; independent comparison absent |
| Ethylene | Gas Cp 42.90 J/(mol K), 298.15 K, 1 bar | TRC 1997 via WebBook; independent comparison absent |
| Ethane | Gas Cp 52.49 J/(mol K), 298.15 K, 1 bar | Gurvich et al. 1989 via WebBook; independent comparison absent |
| Propane, n-butane | None | Identity only; thermal and transport data pending |
| Methanol | Liquid Cp 81.11 versus 81.13 J/(mol K), 298.15 K | Khasanshin/Zykova 1989 versus Carlson/Westrum 1971 as attributed by WebBook; originals, pressure and independence not verified |
| n-Heptane | Liquid Cp 224.64 versus 224.72 J/(mol K), 298.15 K | Andreoli-Ball/Patterson et al. 1988 versus Saito/Tanaka 1988 as attributed by WebBook; originals, pressure and independence not verified |
| n-Dodecane | Liquid Cp 376.00 versus 372.12 J/(mol K), 298.15 K | Trejo/Costas et al. 1991 versus Lainez/Rodrigo et al. 1989 as attributed by WebBook; discrepancy retained; originals, pressure and independence not verified |
| Farnesane | None | 2,6,10-trimethyldodecane constitution only; stereochemistry, formulation and physical properties unresolved |

Every stored Cp uncertainty is explicitly unquantified, because the selected row did not provide it. Same-temperature liquid comparisons are **not asserted to be same-state independent corroboration**, because pressure and original provenance remain unresolved. No averaging, preferred-value fitting or nearest-temperature substitution was performed. All six Cp records are blocked by the evaluator. The gas pressure conversion is the exact `1 bar = 10^5 Pa` relation in [NIST SP 811 Chapter 5, Table 9, bar row](https://www.nist.gov/pml/special-publication-811/nist-guide-si-chapter-5-units-outside-si). The original temperature and pressure are retained in the raw observation string; 298.15 K is a source state, not an assumed universal reference temperature.

Molar mass is not inferred from a material name or casually equated with a dimensionless WebBook molecular weight. No atomic mass table or isotopic composition was verified for this catalog. Density, viscosity, conductivity, diffusion, latent heat, vapor pressure, temperature-dependent Cp, chemical enthalpy, emissivity, solid properties and mixture formulations remain absent. No constituent inventory has been promoted to all-NASA completeness.

## Clean-room read and exposure ledger

Local reads: root `AGENTS.md`, `crates/ignis-wsp/Cargo.toml`, and `crates/ignis-wsp/src/physical_core.rs`. The last read was tool-output truncated; visible content covered typed scalar/domain/provenance interfaces, elemental composition and synthetic invariant tests. No historical handoff links, goals, dossiers, feedback, prior author reports, family forwards/candidates, NASA records, outcome artifacts, `runs/dev`, holdout or validation/test data were read. Author inspected only its own new files thereafter, plus branch identity/existence checks. No memory lookup was performed because the clean-room boundary excludes historical context.

External reads were specific generic-property pages, not searches for individual combustion experiments:

- BIPM defining constants page: displayed table and definition text.
- NIST 2022 complete constants listing: initial displayed lines through roughly line 159 and subsequent focused excerpts around k, N_A, h, c, R and sigma; other constants appeared incidentally in these excerpts. No unrelated values were used.
- NIST SP 961 page 1: parsed text, including neighboring constants. Screenshot attempt failed; no visual verification is claimed.
- DLMF 1.1: notation page, found irrelevant. DLMF 3.12: mathematical constants section. Rust PI declaration: displayed page.
- WebBook pages for the 15 materials: displayed identity headers; gas thermochemistry excerpts for methane, ethylene, ethane, nitrogen; liquid thermochemistry excerpts for methanol, heptane, dodecane. Only the six listed Cp rows and three explicit comparison rows were transcribed. Original cited property papers were **not read**. Nitrogen's displayed Shomate table and several formation/combustion/entropy rows appeared incidentally; no coefficient arrays or those quantities were imported.
- NIST SP 811 Chapter 5: displayed unit section and focused bar-table excerpt. Its old non-SI atomic-constant values were incidental and not used.
- Failed accesses: three malformed NIST constants query URLs; early oxygen/helium WebBook requests with Units=SI; a mistaken farnesane CAS URL returned no content; OEIS A000796 returned 403. Correct farnesane CAS 3891-98-3 was established from the subsequent WebBook search result.
- Search result snippets included generic PubChem, Cheméo, Methanol Institute, NREL fuel-property compendium, Wikipedia, journal/arXiv and other mathematical/thermodynamic pages. These were not opened or used as property authorities. The NREL farnesane snippet was incidental; no NASA dataset or combustion outcome was encountered or read.

The web tool sometimes returned only a portion of a page. A URL being opened is not a claim that every source paragraph, linked reference or entire publication was read. Complete original-paper verification is an explicit remaining requirement for the material rows.

## Checks and reproduction

Executed by this author:

- `rustfmt --edition 2021 crates/ignis-wsp/src/property_catalog.rs`: succeeded.
- PowerShell `ConvertFrom-Json` on the literal dataset: succeeded; 15 materials, 11 records.
- PowerShell `Get-FileHash -Algorithm SHA256`: generated the checked dataset pin above.
- `git diff --check -- crates/ignis-wsp/src/property_catalog.rs configs/models/properties/catalog-w0-astra docs/wsp/research/astra_property_catalog_w0.md`: no output/error; new files are untracked, so this alone is not a complete whitespace check.

Cargo was deliberately not launched: the integrator reported roughly 1 GB free on D: and instructed authors to leave compiler execution to the root using the existing target. No private target directory was created, no compiler process launched, no dependency or shared-contract change made. Tests are authored, **not claimed run** here. Root should execute:

```powershell
$env:CARGO_TARGET_DIR = 'runs/tooling/astra-target'
cargo test -p ignis-wsp property_catalog
```

The five tests exercise frozen-data roundtrip/hash verification, SI-derived values and shared dependence, pending-record refusal, units/duplicates/nonfinite inputs/unknown fields and false corroboration, missing pressure/reference mismatch, and a synthetic admitted point's temperature/pressure/phase/composition boundary. Synthetic values in that test are not catalog material observations. Local test success would establish these software invariants, not independent empirical corroboration or combustion performance.
