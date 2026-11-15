# Independent conditions custody

This pass is owned by a non-modelling condition custodian. It authors no weights, reads
no model or outcome file, fits nothing and consumes no validation look. The owner-relayed
authority is commit `5c75f6b91e4acdab8f3bd983981223194faf6148`. The allowed input is precisely
`runs/dev/data_all/work/atlas/tests.jsonl`; `tests_outcomes.jsonl` is forbidden.

The producer code was inspected statically, not executed. No linked source documents,
old releases, held outcome payloads, holdout directories or sealed validation series
were opened. The current catalogue contains 1,612 rows, 288 exact groups and 18 of the
19 investigations. PSI-26 is explicitly absent. Its levels are 1,017 `test`, 477
`test_point`, 102 `test_day` and 16 `matrix_row`. These are catalogue counts, not a count
of burns, an outcome selection or evidence that all NASA tests are represented.

## Executable screening policy

`tools/wsp/astra/screen_conditions.py` is a host policy for this exact snapshot, not a
new scientific schema or an author-request endpoint. It verifies the catalogue SHA-256,
schema source, schema prose and normalizer source against fixed independently inspected
digests, then verifies the three source files against the owner-relayed Git commit.
Changed source or catalogue bytes fail closed. The normalizer is never imported.

Each line is inspected in streaming order. The custodian creates a real
`ConditionsApproval` with its exact byte hash, identity tuple, exact group id, catalogue
hash, approved field paths and reviewed canonical species tokens. A canonical approvals
mapping is pinned and passed to the actual `AtlasConditions` gate. A second streaming
pass verifies every line through that gate; it cannot publish a partially failed pass.
Only approved metadata remains resident across passes, not full raw rows or a corpus.

The screening policy releases:

- Flag-free nominal settings and planned lists/ranges from reviewed condition fields.
  Multiple values keep `status = multiple`, their `set.kind` and their full set; a matrix
  remains a matrix. A test-day list is never turned into an individual test.
- Closed hardware, geometry, phase, direction and gravity tokens derived by the static
  apparatus rules inspected in Fable's normalizer and schema.
- Canonical fuel/material identities from an explicit reviewed token policy. This policy
  is an exposure boundary, not an expanded atlas vocabulary or a property lookup. Mixture
  names use exact `+`-separated tokens. No name is inferred from a free-text description.
- Independently approved component species/roles/fractions and geometry dimensions/values
  through exact indexed paths. A component identity can remain useful when its fraction
  is unavailable. Different fuels listed for one day are not asserted to be a mixture;
  their component fractions are omitted. No fraction basis is guessed.
- Initial/before-test numeric readings only where their timing is explicit in the
  inspected producer: Flame Design `pinit (atm)`, `XO2 (init)`, `XC2H4 initial`; CFI-G
  ` XO2 before test`, `Xf  before test`, including the corresponding oxidizer copy.

Every flagged value and alternative is omitted, including ignition-conditioned values,
analysis-sheet diameters, ambiguous mixture fraction bases, conflicting source values
and repairs. Measured values without independently established preignition timing and
other unstated numeric values are omitted. A safe-looking SI number is not sufficient.
CLD Flame's `Fuel conc. Range` values are omitted even when nominal: the producer's
source note explicitly includes concentrations changed during tests and does not establish
which value preceded ignition. Two provisional local projections generated before this
additional rule were moved to private custody and are not author releases.
Parent fuel-identity problems propagate to component fields; they cannot disappear through
the producer's derived `PURE` fraction. No missing value becomes zero.

Raw values, raw units, descriptions, notes, source text, labels, measurement file counts,
outcome status/field names, partitions and aliases are not exported. Exact original atlas
and group ids are used unchanged. Provenance is preserved as catalogue hash, row hash and
exact field path. It does not authorize an author to open the raw row or its source.

Every row remains in the author projection, including identity-only or sparse rows. Safe
omission reasons are uniformly `not_released_by_preignition_screen`; detailed rejection
reasons stay in private custody. There is no selection on case quality. Structural
availability remains visible through which approved fields are present, as required for
condition coverage; these approvals grant no outcome access.

## Outputs and requirements inventory

Private policy, approvals and omission receipts live under
`runs/dev/data_astra/work/condition_custody/<screening SHA-256>/`.
Author-safe `conditions.jsonl`, `inventory.json` and `manifest.json` live under
`runs/dev/data_astra/work/author_conditions/<screening SHA-256>/`.
The version binds authority, schema/producer/catalogue pins, gate code, screener code and
approval mapping. Published files are immutable and hash checked on rerun. A pending
projection stays private until every exact row has passed the gate.

The inventory includes all 19 investigations, distinct safe species/material identities,
geometry dimension names and released-value counts, level counts, group counts and
present/unreleased physical-condition counts. It lists property research requirements by
physical family: thermochemistry and transport for gases; liquid density, phase change,
transport and mixture basis for droplets; material grade, thermal properties and geometry
for solids; thermal decomposition and aerosol properties for smoke/dust. Those are
research requirements, not assertions that any value has been sourced or checked.

Material names such as lampwick, Pyrell, SIBAL fabric and Arizona road dust identify source
materials but do not establish composition, grade, porosity or a universal property value.
`iso_dodecane` and `farnesane` retain their original catalogue tokens; chemical identity and
mixture specification need authoritative research before supplying properties.

## Reproduction

```powershell
python -m unittest discover -s tools/wsp/astra -p test_screen_conditions.py -v
python tools/wsp/astra/screen_conditions.py
python tools/wsp/astra/screen_conditions.py --execute
```

Dry run performs complete screening/gate checks but writes no receipts or projections.
`--execute` is required to write. The tool prints only mode, row/family metadata counts,
output version and author directory; errors are generic and never quote input values.
No scientific qualification follows from a successful projection or synthetic boundary tests.

Verification on 2026-11-14: all **9 synthetic custody tests passed**. They cover source
drift, raw-text leakage, canonical names versus disguised notes, nominal multiple ranges,
matrix-level preservation, ignition-conditioned omissions, parent conflict propagation,
analysis-sheet geometry and explicit initial timing, including the CLD source-specific rule.
The complete real catalogue passed the gate's dry run before publication. This does not
exercise any outcome route or scientifically qualify a measurement or model.

The final `--execute` run published and independently re-read all 1,612 rows at:

`runs/dev/data_astra/work/author_conditions/84fa8febc84acc04768c6f9ce31896b08b8a6eb7acbce3978547b6fdb1f026f8/`

The projection SHA-256 is
`b001d711ab32109a3b1563436fc80174baba5a35f48056bac0d2704a3f59a9f9`.
Manifest and inventory hashes matched. A recursive check found no forbidden raw/source/
outcome/partition keys in the released records, and confirmed that no CLD concentration
was released. The output holds 22,318 approved structured fields, 1,580 released canonical
fuel-name values, 32 distinct species/material/diluent tokens and 45 multiple-value fields.
These are projection coverage counts, not experimental sample sizes.

The diameter fields from CFI analysis sheets and unestablished FLEX measurement timing
remain omitted. FLEX-2's eight diameter ranges are planned matrix conditions, not measured
initial diameters for eight burns. The inventory therefore exposes a concrete need for
independent preignition geometry and settings recovery without inventing missing values.
