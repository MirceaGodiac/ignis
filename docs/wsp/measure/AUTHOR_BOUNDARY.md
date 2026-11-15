# Author boundary: implementation and pending integration

Status: a host-screened conditions projector and a committed split metadata adapter are
implemented. **No real author-outcome route has been activated or exercised.** Fable's
15 `ignis.atlas.split/1` files were verified against committed authority
`5c75f6b91e4acdab8f3bd983981223194faf6148`. They describe **9 split investigations and
6 held investigations**; Flame Design's two strata make 10 split strata, not 10
investigations. Outcome activation still requires independently screened inputs and
host authorization; committed metadata alone does not approve any payload.
This adapter implementation used synthetic rows and opened no real atlas row or outcome
payload. No measurement or label was generated here. Separate host custody runs may use
approved conditions metadata; their output/version records belong to the custody worker.

This implements the routing boundary required by
`docs/wsp/dossier/ASTRA_PROMPT_ALL_NASA.md`, rules 1, 2 and 5 and section 2b.
Fable owns the atlas schema, catalogue and group freezes. This work does not define a
competing atlas schema, choose groups, or infer a split from the draft atlas code.

Fable's `docs/wsp/atlas/SCHEMA.md` now exists and identifies `ignis.atlas.test/1`.
The prose and metadata-only `tools/wsp/atlas/atlas_schema.py` were read. The producer's
outcome separation is useful but not sufficient to expose complete rows: they retain
raw strings, source notes, free-text descriptions, labels, and flags such as
`conditioned_on_ignition`. No source-level promise overrides the projection boundary.

## Runnable conditions projection

`AtlasConditions` is usable by a trusted host now, without a split or any outcome read.
Its constructor accepts independent `Pin` objects for the exact schema source bytes,
prose bytes and approved-row mapping. Both schema pins must name `ignis.atlas.test/1`.
The only files the constructor reads are the two fixed schema paths, and it verifies
their hashes before evaluating Fable's metadata-only schema code. `jsonschema` is the
only non-standard Python dependency; missing dependencies make the adapter unavailable.

A host supplies an opaque row key and a `ConditionsApproval` containing the exact row
SHA-256, the independently screened identity tuple `(atlas_id, investigation_id, test_id,
level)` and the exact condition paths approved for release. Optional `approved_species`
is a tuple of exact material/species identifiers individually reviewed by the host; it
does not create a new global vocabulary. The canonical approval
mapping binds the schema/prose hashes and is pinned as `author-conditions-approvals/1`.
The receipt is a local host policy object, not a new atlas schema, not proof that a host
actually screened the bytes, and never a user-supplied grant. Computing a hash over
unscreened rows does not approve them. The host must establish those pins independently.

Usage from a trusted caller (arguments below are already approved metadata):

```python
gate = AtlasConditions(schema_pin, prose_pin, approvals, approvals_pin)
result = gate.read(opaque_row_key, host_read_one_approved_row)
# Send only result.projection when result.decision.state == "allowed".
```

Unknown keys, unavailable metadata and requests for unapproved field paths never invoke
the row reader. Known keys must return exact pinned row bytes. Full row validation uses
Fable's `row_schema()`, its closed lists and its `value_problems()` status invariants;
unknown keys/tokens, duplicate JSON keys and non-finite numbers fail closed. Errors do
not quote row values. Unit tests use synthetic rows; actual custody runs are recorded
separately by the trusted caller.

The output is a **partial projection**, not an atlas row: `source_schema` identifies its
source contract. It carries the approved identity, source/row hashes and selected
conditions with only `status`, `value_si`, `unit_si`, `basis` and `scope` (plus `set` for
an approved nominal planned set). Scalar identifiers have only status/value/unit. The
constant omission reason is `only_host_approved_fields_are_projected`; no per-row quality,
conflict or ignition flag is exposed as an omission reason. The original
provenance stays with the host and is addressable by exact row hash; this route does not
grant an author permission to open the original row or its cited sources.

The explicit exposure allowlist covers hardware rack/insert, fuel phase, configuration,
orientation, atmosphere/oxidiser composition and pressure/temperature, flow direction/speed,
injection flows/fractions and gravity. Indexed composition and geometry paths are also
supported, as listed below. Requested value objects must have no flags or alternatives
and be independently screened as pre-outcome information. A numeric
value is not automatically safe because it occupies a conditions field. Values are not
inferred from missing fields. Resolved scalars are preserved. Multiple values are admitted
only as nominal planned **numeric** lists/ranges, with finite values and the original
`status = multiple`, `value_si = null`, set kind and values unchanged. Ranges need two
ordered endpoints. These sets are not collapsed to a representative value. Other
unresolved or flagged requested fields seal the whole response;
the host can separately screen a smaller projection. Enum values and units are validated
against Fable's schema, never a copied vocabulary.

Requesting raw values, descriptions, notes, unreviewed species/fuel names, source text, extra
conditions, labels, outcome metadata, partitions or arbitrary paths is rejected. None of
those fields are exported incidentally from an otherwise approved row. The added paths are:

- `fuel.name`: resolved canonical token, or tokens joined by `+`, each present in the
  pinned `approved_species` tuple. No case folding, alias guessing or free text is allowed.
- `fuel.components.N.species`, `.role`, `.fraction`: species must be an approved exact
  token; roles use Fable's enum. Fractions are screened separately. A fraction also requires
  a resolved, unflagged and unconflicted parent `fuel.name`, preventing parent fill-series
  repair flags from disappearing during extraction.
- `atmosphere.diluents.items.N.species`, `.fraction`: exact approved species and separately
  screened numeric fractions. Approval of a species does not approve its fraction.
- `geometry.sizes.N.dimension`, `.value`: the authoritative dimension enum and a separately
  screened dimensional value. No arbitrary new dimension labels are accepted.

`N` is an exact zero-based integer index without leading zeros, negatives or wildcard
selection. Out-of-range indices seal the response. Every selected path is explicitly
pinned in the receipt. Identity-only selections may retain a known species or dimension
while omitting an unresolved numeric value; they never invent a zero fraction or size.
Dates and other free-text fields remain unsupported. A syntactically plausible species
name is insufficient: membership in the independently reviewed tuple is mandatory.

The schema's `labels.flame_type` is derived from apparatus configuration; `labels.studied`
is derived from investigation titles. Neither is an observed what-happened label. The
authoritative schema currently has **no observed-phenomenon vocabulary or page-kind enum**
for section 2b page cards. Those cannot be invented here. Page-card/observed-label live
validation remains pending Fable's schema extension.

## Runnable committed split adapter

The 15 split JSON files, their README and the static `freeze_groups.py` source were read
as metadata only. The freezer was not executed. `AuthorGate.from_atlas(...)` now consumes
the actual `ignis.atlas.split/1` format, without opening a catalogue file or any outcome.

```python
gate = AuthorGate.from_atlas(
    conditions, split_metadata, split_pins, catalogue_pin, code_pins,
    mapping, mapping_pin, commit, reveal_recorder=host_durable_reveal_record,
)
decision = gate.decide(payload_key)  # metadata only, not a reveal
result = gate.read_author(payload_key, host_payload_reader)
```

The host builds this from:

- An `AtlasConditions` instance that has successfully validated the exact approved
  catalogue rows. To enable group routing, a `ConditionsApproval` also carries
  `group_id` and `catalogue_sha256`. `read()` checks the group's exact equality and records
  the verified identity only after the complete row validates. Both identities are also
  included in the safe conditions projection when both were explicitly approved, allowing
  exact catalogue joins without exposing partition grants. Blind worker manifests still
  omit these identities.
  Empty `fields` is permitted for an identity-only verification. Merely constructing an
  approval does not populate the verified index. The host attests membership of the
  exact row in the pinned catalogue snapshot; this adapter does not reread or hash the
  whole catalogue to prove membership. All such attestations remain trusted-host inputs.
- Exact split bytes and `Pin("ignis.atlas.split/1", sha256)` for each investigation,
  plus `catalogue_pin` using `ignis.atlas.test/1`. `code_pins` has exactly `catalogue`
  (the `normalize_tests.py` hash, version `ignis.atlas.test/1`) and `freeze`
  (the `freeze_groups.py` hash, version `ignis.atlas.split/1`).
- A 40-character Git commit. The adapter reads only fixed safe metadata/code paths
  using `git show <commit>:<path>` and verifies the schema source/prose, catalogue
  code, freezer code and every split against their exact pins. Working files alone,
  a missing committed split, or changed bytes cannot grant author access. The host must
  select an approved commit; existence of an arbitrary Git object is not team approval.
- A canonical mapping from `outcome_approval_bytes(...)`, independently screened and
  pinned as `author-outcome-approvals/1`. It binds commit, schema/prose, catalogue and
  producer hashes, every split identity, verified atlas/group identities and complete
  payload dependencies. `runs` contains exact `atlas_id` strings; `pages` contains exact
  mapping keys. Payload hashes cover **all bytes returned by the reader**, including
  a label record's full envelope and any embedded content-hash field.

The split parser rejects unknown fields/statuses, duplicated group membership anywhere
across partitions or strata, invalid counts and wrong investigation/version identities.
`held_too_few_groups` and `held_until_test_list` admit only held groups and **never per-run
author outcomes**. Known rows in newly discovered groups are held. Unknown atlas ids,
alternate aliases, changed case or leading whitespace are not guessed or normalized.
The four project families absent from these new splits cannot acquire grants through
this adapter; their existing evaluator rules remain separate.

The existing recursive dependency gate applies unchanged: every outcome's entire run/page
closure must resolve to train, and a missing or mixed dependency or cycle seals the item.
Both `decide` and `read_author` remain compatible with the label-record worker. A decision
is only routing metadata; use `read_author` for the actual release. On live adapters a
durable host reveal recorder must accept an authorization event before any reader call.
Missing/failed recording returns unavailable without opening the payload. Events identify
the commit and mapping digest and log allowed/denied authorization attempts, not a claim
that a payload was successfully returned. The host owns durable storage, UTC timestamps,
actor scope, and the required `runs/evaluation/reveal_log.jsonl` integration. The callback
does not itself implement that storage or authenticate its caller.

Only synthetic payloads have exercised this route. The separate real-metadata smoke
check below verifies the committed split identities; offline tests alone do not establish
a real freeze. No live authorization, real corpus membership or outcome release is claimed.

## Implemented

`tools/wsp/astra/author_gate.py` contains a small metadata policy kernel:

- `AuthorGate.live()` (the outcome/dependency gate) returns `unavailable` without opening
  any file. It is the unconfigured default; `from_atlas` is the explicit host adapter.
- `synthetic_fixture(...)` is an offline test constructor, not a production importer.
  Its private fixture dictionaries do not specify the future atlas format. All three
  metadata blobs must explicitly have `origin = synthetic_fixture`. Each blob's exact
  SHA-256 and version must match a separate host pin. The split binds the schema hash;
  the approved payload mapping binds both the schema and split hashes. Unknown fields,
  duplicate JSON keys, unknown categories, invalid group membership and inconsistent
  group partitions fail closed. A fixture asserts `frozen_by = Fable`; that string is
  **not authentication** and cannot establish a real freeze.
- Conditions and public-method pages require a reviewed, exact payload mapping and a
  complete dependency listing. Public availability alone does not mean a page is safe.
  Measurements, what-happened labels, outcome pages and interpretations require train
  membership for every run dependency, including dependencies reached through pages.
  Train requirements propagate through condition/method dependencies of an outcome.
  A page or interpretation containing several runs is indivisible at this boundary.
  Unknown runs/pages, cycles, missing screening, incomplete dependencies, empty outcome
  dependency sets, or any validation/test dependency remain sealed. Per-run validation
  access has no route here; registered aggregate looks belong to the evaluator.
- `read_author` decides from metadata before invoking its reader. Denied routes never
  invoke the reader. Allowed reads return only the exact bytes whose hash
  matches the approved mapping. Hash mismatches return no payload. The dependency
  metadata must be independently screened: the gate cannot discover omitted outcomes
  without reading content, and does not claim to do so.
- `blind_task` creates a **plan**, with an opaque random task id, numbered
  opaque input paths, content hashes, a pinned method hash, operation and output path.
  It does not copy or open inputs. The worker manifest contains no source/run identity,
  partition, model, hypothesis, expected outcome, or method-version text. Separate
  `host_routes` retain source-key routing, method version and metadata identities.
  Only `manifest`, never the entire `BlindTask` object, may be sent to a worker.
  Its origin is `synthetic_fixture` for fixture gates and `host_screened` for the real
  adapter; neither is a claim about the scientific provenance of future measurements.

The policy structures are local Python host adapters/test scaffolding, not shared IGNIS contracts.
No Rust contracts, generated schemas, shared records or Fable-owned sources changed.

## Required before live outcome use

1. Commit and independently approve exact catalogue/split authority and screened payload
   mappings. The format adapter is implemented; page-card and observed-label schemas
   still need authoritative forms. Pins and grants come from the
   trusted host, never from author requests, prompts or incoming documents. A change
   in any pinned input invalidates approval and requires review; hashing alone cannot
   authenticate approval or demonstrate that dependency screening was complete.
2. An independent screener resolves every page/run dependency and classifies the exact
   bytes to be released. Outcome-bearing material with unresolved or mixed dependencies
   stays in protected storage. This includes cluster findings, contradiction edges,
   captions, annotations and derived summaries, not only numeric measurement tables.
   An author-safe excerpt requires its own reviewed bytes, identity and dependencies;
   simply relabelling a mixed page as methods is insufficient.
3. A separate host staging process gives measurers the instrument/method information
   and frames/traces they need. It screens the method and removes embedded partition,
   model and expected-outcome clues from staged metadata, overlays and filenames.
   Opaque manifest paths do not sanitize payload contents. Instrument/calibration
   information must itself be screened and supplied as pinned opaque inputs.
4. Measurements and labels of other groups are permitted only in the isolated measuring
   or labelling lane. Their outputs return to protected storage through host routing.
   The host attaches catalogue test/group ids and source locators after blind work;
   it does not give partitions to workers. Authors receive only the train projection
   admitted by the gate. Independent repeat measurements/labels need separate blind
   jobs without the first worker's answer. Such jobs and their agreement reports have
   not been implemented or run here.
5. Production release must integrate host scope, isolated storage permissions, streaming
   and the required reveal ledger before access is enabled. This module does not provide
   an OS sandbox, a CLI for reading data, a deployment, or a production access-control
   service. It cannot restrict an agent that independently has direct filesystem access.

The live outcome adapter must not guess a frozen-split format from `atlas_schema.py`.
Missing or changed authority remains `unavailable`, without opening protected payloads.
No new label vocabulary is introduced; any future vocabulary change goes through Fable.

## Reproduction and actual verification

```powershell
python -m unittest discover -s tools/wsp/astra -p test_author_gate.py -v
```

Ran on 2026-11-14: **26 tests passed**. Tests use in-memory synthetic metadata and temporary
synthetic files. Denied routes receive a reader that raises if invoked. The live route
also runs with filesystem `read_bytes` trapped. Tests cover metadata hashes/versions and
cross-bindings, pending inputs, protected and mixed dependencies, recursive page routing,
payload tampering, allowed screened fixtures, and separation of blind and host manifests.
The conditions tests read only Fable's schema source/prose and generate synthetic rows
from that schema. They cover safe projection despite embedded forbidden free text,
rejection of unsafe requested fields, closed vocabulary/status/finite-number checks,
schema/mapping tampering, row mutation and caller mutation of the approved-row dictionary.
The indexed extension tests cover reviewed versus unreviewed materials, identity without
an available fraction, exact index syntax, separately approved dimensions, preservation
of nominal matrix sets, rejection of measured/flagged/malformed sets, and propagation of
parent fuel-identity flags to component fractions. Generic omission metadata does not
expose per-case conflict or ignition flags.
Split-adapter tests use the actual split shape with synthetic ids/bytes and a mocked Git
metadata reader. They cover held statuses, new groups, committed-digest mismatch, aliases,
unverified rows, mixed pages, missing pages, cycles, overlapping strata, unknown status,
and missing/failed reveal recording. Mocked committed bytes are not evidence that the
working NASA split files have been committed or approved.
A separate **metadata-only smoke check passed** against the owner-relayed commit
`5c75f6b91e4acdab8f3bd983981223194faf6148`: actual Git blobs for schema source/prose,
catalogue/freezer code and all 15 split files matched the pins, and each on-disk split
parsed successfully. The catalogue identity was taken from split README metadata:
`35546e129a160c6ab73c630be82fab98700bb5fd5fb92fa83954eb7af06cdfbd`.
This check created an in-memory adapter with **zero verified rows and zero approved
payloads**; an unmapped request was sealed. It did not read the catalogue itself, verify
its full byte content, read any outcome or install a live author bundle.

No live API, NASA media, protected outcomes, validation series or test partition was read.
No scientific qualification follows from these tests. No Rust files changed and no commit
or push was performed; the repository-wide pre-commit gates were not run for this patch.
