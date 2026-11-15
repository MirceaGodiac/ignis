# Private label records and independent label audit

`tools/wsp/astra/label_records.py` implements protected staging, validation, deterministic
rule membership, dependency closure, immutable export, the section-4 label audit and the
consolidated section-2 Terra qualification. It also validates same-pass search enrichment.
It does **not** assign semantic labels, read NASA files, dispatch a model, create application
contracts or authorize author access. The implementation was exercised only on synthetic
fixtures. No actual labels, page coverage or NASA agreement rate is claimed here.

Authority: [whole-goal owner instruction](../dossier/ASTRA_PROMPT_ALL_NASA.md), especially
roles, rules and steps 2, 4, 4b and 9,
[atlas schema sections 4–5 and 7](../atlas/SCHEMA.md), and
[measurement design section 11](ASTRA_MEASUREMENT_DESIGN.md#11-pagetest-labels-rule-based-clusters-and-evidence-links).
The current private envelope is `astra.private.labels/3`; explicit legacy replay supports
`astra.private.labels/2`. Neither is a second per-test schema.
The atlas identifier stays `ignis.atlas.test/1`; shared contracts, schemas and atlas files are untouched.
The owner's explicit page kinds and edge relations are sufficient authority for
these Astra-owned output envelopes; absence from the per-test atlas schema does not block them.
Additional page kinds and genuinely missing scientific vocabularies still require proposals.

## Versioned authority and preservation of legacy records

`definitions()` reads schema code, schema prose and the owner instruction **from immutable Git
objects**, pinned to reviewed commit `76fbb19244fb148ef94eef508896df5b81825c75` with explicit
expected SHA-256 digests. It does not read mutable worktree metadata. Later HEAD/worktree edits
neither interrupt a sealed qualification nor silently change its definitions. A new authoritative
revision requires a deliberate adapter migration and new receipts. Changed pinned bytes,
headings or mandatory safeguards fail clearly before records are accepted.
Current owner authority is `astra.owner.whole-goal/2`; it includes the full source hash and
section hashes for 2, 4, 4b, roles, research/findings, rules and 9. Current vocabulary digest is
`1967edac72a09fd44ea79916bd564fe2f30c60432b861b485dd6125c4f0b9bb3`.

The current checkout is `D:\Codex\Worktrees\ignis-science-transfer\SpaceApps-Ardealu`, branch
`feat/handwritten-science-ignis`, in the canonical `MirceaGodiac/ignis` repository. The protected
`runs` junction retains its existing storage. The migration does not merge unrelated histories
or modify records because the checkout or remote changed.

Older sealed rows, source metadata, producer/tool hashes and content hashes remain unchanged.
Legacy validation must call:

```python
legacy_defs = definitions(
    legacy_commit="<full 40-character owner commit>",
    schema_commit="<full 40-character schema/prose commit>",
    expected_pins={"schema_sha256": "<sealed hash>",
                   "vocabulary_sha256": "<sealed hash>",
                   "owner_authority": sealed_owner_authority},
    newline="lf",  # Or explicitly "crlf" only when those exact bytes match the sealed hashes.
)
```

All expected pins come from independently retained headers; this is not an instruction to
open first-label payloads. The reader only accesses three fixed Git metadata paths. Two
actual historical header variants were matched without opening any page/label payload:

| Owner Git commit | Schema/prose Git commit | Representation | Reproduced vocabulary digest |
|---|---|---|---|
| `1ca40c0e5fa23159137c331f8729b037c61e4186` | `3577073e7112a5f2ecebb3d0e5951d2ea9e4a5fa` | LF | `292b641f105a8a5737bb6f81ef75199139afc06ed54ff2e6fce63f0bc15a31c0` |
| `91de6a448100bb7a32d13dd81eb04f55add060fd` | same commit | LF | `35bc556862fec9e9c4ff95f188ffcc1935a97e9b42e6525d245a40007b80716c` |

The first owner revision predates the schema file, so its schema/prose reference is separately
explicit and hash-checked. Unknown matches remain refused. `defs.replay_commit` records both
Git identities and newline representation in a new export manifest; it never alters a legacy
record. CLI export replay uses `--legacy-commit`, optional `--legacy-schema-commit`, required
`--legacy-pins <header-pins.json>` and explicit `--legacy-newline lf|crlf`.

For consolidated/whole-goal snapshots, use `definitions(authority_commit=<full reviewed commit>,
expected_pins=<sealed pins>, schema_commit=<explicit schema commit when needed>)`. Known
revisions include `3577073e7112a5f2ecebb3d0e5951d2ea9e4a5fa`,
`1f0eef6ffb4408775d955bba5d9bd85c273fac86`,
`ccb0432eee7f45dd1127157f6767422c6dd97c41`,
`0427549a8bf36c918176cb5fc36d6b6fb57219c9` and the current whole-goal commit. Their authority
versions and full/section hashes are replayed separately; old receipts and failed attempts are
retained, not edited to claim they consumed newer instructions.

`qualification_compatibility(legacy_defs, current_defs, kinds)` records a metadata bridge only
when the actual schema hash and each closed kind's tokens match. It grants no equivalence
between nonexistent legacy enrichment and new generated retrieval cues. Qualification compares
the original strong labels with independent Terra labels; it does not rewrite the strong side.

## Protected storage and execution

The logical outputs requested by the owner are `pages.jsonl`, `test_labels.jsonl`,
`clusters.json`, `edges.jsonl` and `findings.jsonl`. Exports place them together with
`manifest.json` in `<output>/<manifest-sha256>/`; nothing is overwritten with different bytes.
The custodian may choose `runs/dev/data_astra/work/labels/` as `<output>`. That is protected
storage: authors must not open it. Audit universes, packets, routes and comparison reports
are likewise protected. A directory convention is not an operating-system permission boundary.

Every command is a dry run unless `--execute` is present. Inputs must already have been
authorized for the custodian; dry run still reads and validates its supplied input. CLI output
contains the artifact digest/path and execution flag, not semantic records or agreement counts.

```powershell
python tools/wsp/astra/label_records.py export --bundle <protected-bundle.json> --output <protected-output>
python tools/wsp/astra/label_records.py freeze --bundle <census-input.json> --output <protected-audit>
python tools/wsp/astra/label_records.py sample --bundle <sampling-input.json> --output <protected-audit>
python tools/wsp/astra/label_records.py qualify --bundle <qualification-input.json> --output <protected-audit>
python tools/wsp/astra/label_records.py packet --bundle <packet-input.json> --output <protected-audit>
python tools/wsp/astra/label_records.py compare --bundle <comparison-input.json> --output <protected-audit>
```

Adding `--execute` writes hash-addressed files. No command fetches sources, opens a model,
performs fitting, reads partitions or publishes records. The tool operates on already supplied
JSON, so the caller—not an arbitrary model request—must be the authorized custodian.

## Envelope and evidence

An export bundle has exactly `records`, `sources` and `runtime_receipts`.
`sources` maps opaque evidence IDs to SHA-256 digests; it contains no input paths or grants.
`runtime_receipts` maps assignment IDs to the producer objects independently captured by the
host at assignment time. The tool checks equality with that receipt; it cannot authenticate
an arbitrary JSON receipt or infer the actual runtime from a friendly agent name.

Each record has these fields; unknown fields fail validation:

| Field | Meaning |
|---|---|
| `envelope_version`, `origin`, `record_id`, `kind` | Private format, `synthetic_fixture` or `source_evidence`, stable record identity, and `page`, `test_label`, `cluster`, `edge` or `finding`. These are private staging artifact kinds, not newly approved atlas labels. |
| `schema_version`, `schema_sha256`, `vocabulary_sha256` | The actual atlas schema ID, SHA-256 of its metadata implementation, and SHA-256 binding its imported closed lists, implementation digest, complete schema prose digest, owner page kinds/edge relations and owner authority. |
| `owner_authority` | Exact object returned by the selected `definitions().owner_authority`: current owner source path, selected section hashes, version `astra.owner.whole-goal/2`, full-file SHA-256. Historical replay preserves its original authority object unchanged. |
| `content_sha256` | Canonical UTF-8 JSON digest of the entire record except this self-hash field. Keys sorted, no nonfinite numbers, no insignificant JSON whitespace. |
| `producer` | `kind`, `id`, `model_id`, `model_version`, `version_limitation`, `assignment_id`, `runtime_session_id`, `prompt_sha256`, `tool_sha256`, `config_sha256`. |
| `created_at` | RFC 3339 UTC timestamp ending in `Z`. |
| `locators`, `dependencies`, `test_dependencies` | Exact evidence; direct source/record dependencies; catalogue-resolved test IDs, including revealed and merely mentioned tests. Transitive closure is materialized in the export manifest. |
| `labels`, `body` | Evidence-backed assertions and kind-specific content below. |

For a model assignment, `model_id` is the actual provider/runtime model ID from trusted
execution metadata, not `Astra` or `Fable`. An unavailable exact build version stays null with
`version_limitation`; it is never guessed. Every model producer must match its separately
captured receipt. Only deterministic cluster production or the owner-requested scripted
discrepancy edge may use `kind=rule`; it has null
model identity, names the producer implementation, and cannot contain semantic labels.

A locator is `{source_id, source_sha256, page, selector}`. `page` is a positive physical page
number or null for non-page evidence. Supported exact selectors are:

- `text`: `start`, `end` (half-open character offsets in the pinned extraction), `quote`, `extraction_sha256`.
- `cell`: `sheet`, positive `row`, `column`.
- `figure`: `figure_id`, `bbox` (`x0,y0,x1,y1`, normalized to the pinned rendering), `render_sha256`.
- `field`: a JSON `pointer`.
- `whole_page`: `reason` (`blank`, `unreadable`, `unclassified`) and `render_sha256`.

The last selector preserves failure-page coverage and cannot justify an assigned label.
The custodian must verify that quotes, offsets, cells and regions actually match the hashed
source; this structural validator intentionally never opens the source bytes.

An assertion is `{kind, state, value, reason, basis, rule, locators}`. `state` is `assigned`,
`absent` or `unresolved`. Assigned values are a complete set of imported closed-list tokens;
absence is `[]`; unresolved is `null` with a reason. `basis` separates `observed`, `derived`
and `interpretation`. Derivation rules come from atlas section 5. `labels.flame_type` requires
`L1`, `labels.studied` requires `L2`, `labels.fire_safety_need` requires `L3`, and
`map.crew_atmospheres` requires `CREW`, all with derived basis. They are never observed outcomes.

`page.kind` uses the owner's exact seven strings: `methods`, `apparatus`, `requirements`,
`results table`, `figure`, `discussion`, `drawing`. It is a multi-label set: a page can contain
methods and a figure, for example. Blank, title-only, reference-only or otherwise unclassifiable
pages retain null/unresolved with their precise reason when none of the seven applies; they
are not forced into an unrelated kind. Newly needed kinds remain proposals.

`observed.phenomenon`, `document.topic`, `flow.regime`, `material.class` and
`physical.observable` identify **unresolved work requirements**, not approved vocabularies.
They accept only null unresolved assertions. See [pending proposals](VOCABULARY_PROPOSALS.md).

## Kind-specific bodies

| Kind | Body and validation |
|---|---|
| `page` | `page_id`, `document_sha256`, `page_number`, `investigation_ids`, `mentioned_test_ids`, `unresolved_test_mentions`, `reveals_outcome` (boolean or null), `revealed_test_ids`, `quantity_evidence`, `condition_evidence`, and in v3 `search_enrichment` plus `review` (below). A `page.kind` assertion is mandatory: assign the owner-listed set where supported, otherwise null/unresolved with reason. |
| `test_label` | `test_id`, which must appear in `test_dependencies`. The host resolves catalogue aliases; this tool never guesses identities from values. |
| `cluster` | `rule`, `input_ids`, `member_ids`, `unresolved_ids`. All inputs, including negative/missing memberships, are dependencies. Membership is rebuilt and compared during bundle validation. |
| `edge` | `source`, `target`, `relation`, `state`, `reason`. Both endpoints must be dependencies. `state=assigned` uses an owner-requested relation below and `reason=null`; unresolved joins or additional relations use `relation=null`, `state=unresolved`, with a reason. |
| `finding` | `assertion_kind=interpretation`, `subject`, `claim`, `domain`, `disagreements`, `unknowns`. The model supplies this finding and its evidence; the deterministic tool never writes a scientific conclusion. |

Page `quantity_evidence` preserves `{raw_name, raw_unit, unit_reason, locator}`. A missing unit
stays null with its reason; a raw unit is not converted into an approved quantity classification.
`condition_evidence` preserves `{raw_text, locator}`. These source excerpts do not create a
second condition/value schema. Numerical measurements and their raw/SI values remain in the
measurement service/artifact; their exact references can be evidence dependencies here.

`page_coverage(records, expected_pages)` compares cards against an independently captured
document-digest → physical-page-count census, including blank/unreadable/unclassified cards.
It reports missing/unexpected pages to the custodian. A complete structural census is not
evidence that classifications are correct. No such census has been run on real documents here.

## Same-pass search enrichment and strong/conservative screening

The page worker supplies the entire document text with stable page IDs as unchanged leading
context. Output may be divided into target-page runs based on measured response size; it is
not a reason to truncate source context. Only input-limit exceptions use real chapter boundaries
with front matter/abbreviations repeated. Figure/table images accompany the relevant context.
The worker validates target coverage exactly once and records every failure/repair. It starts
at 24 concurrent requests and adapts to provider limits, retaining batches on retry. These are
worker responsibilities; this module validates returned records without dispatching a model.

Method attribution and adaptation are recorded in
[the SIRA research note](../research/astra_sira_page_enrichment.md), based on
[the primary paper §§3.1–3.2](https://arxiv.org/html/2605.06647v3) and the
[official code at the pinned revision](https://github.com/facebookresearch/sira/tree/57f9cbbe5d0a00c91a60f14f3532c8268edd6332).
This prepares offline retrieval vocabulary; it makes no claim about IGNIS retrieval accuracy
and implements no online query expansion or new agent loop.

`body.search_enrichment` has exactly `status`, `reason`, `provenance`, `grounded_terms`,
`retrieval_cues`. Status is `filled`, `unresolved` or `not_applicable`. The latter two require
a reason and empty lists; filled lists may also be empty when there is nothing useful to add.
No enrichment is fabricated to fill a field, and no second model pass is implied.

The dedicated provenance is `{method: "sira-inspired-page-enrichment/1", assignment_id,
prompt_sha256, tool_sha256, config_sha256}`. Every identity/hash must match the same page-reading
producer. A record migrated only structurally cannot claim enrichment was authored during its
original reading; retain unavailable enrichment explicitly instead.

Each `grounded_terms` entry is `{id, kind, text, locators}`. Kinds are structural retrieval roles:
`concept`, `synonym`, `abbreviation`, `instrument`, `material`, `test_id`. Terms and source-used
synonyms/abbreviations must have exact source evidence; they are not new controlled scientific
labels. Each `retrieval_cues` entry is `{kind, text, grounding_ids, new_terms}`. Cue kinds are
`search_phrasing`, `synonym_expansion`, `abbreviation_expansion`, `concept_expansion`.
Grounding IDs resolve to the source terms and hence their exact page locators. `new_terms` names
the generated retrieval vocabulary absent from the page text and actually used in the cue.

The host calls `validate_enrichment_novelty(enrichment, already_supplied_page_text)` to check
case-insensitive term novelty with word boundaries; this uses the current input text without
opening or rereading a file. The structural validator checks grounding IDs, evidence hashes
and same-pass identity. Neither check establishes semantic accuracy or retrieval effectiveness.
Generated cues are indexing hints, never source text, measurements, outcome labels or evidence
for a scientific claim. Protect the entire enrichment and its derived document-frequency
metadata with the page dependency policy. No arbitrary phrase-count or character ceiling is used.

`body.review` has exactly `confidence` (`low`, `medium`, `high`, `unresolved`), `figure_or_table`,
`label_disagreement`, `requires_strong_review` (the last three booleans), and `outcome_decision`.
A low/unresolved-confidence figure/table or any disagreement requires strong review. Figure or
results-table labels cannot hide behind `figure_or_table=false`.

An outcome decision is either `{method: "strong", assignment_id: <captured strong assignment>,
rule: null}` or `{method: "conservative", assignment_id: null, rule: "terra_non_authorizing/1"}`
(also supported: `uncertain_outcome_is_protected/1`). Conservative decisions force
`reveals_outcome=true`: this is a protection decision, **not** an assertion that an outcome was
observed in the source. Terra cannot self-authorize a `strong` outcome decision. Its model-proposed
false flag is never sufficient for release. The host's independent classification and gate
remain necessary; conservative or outstanding-review cards and all derived enrichment are
excluded from `author_projection`, even if a test dependency is TRAIN.

Owner-requested edge relation strings (exact values accepted by the private envelope):

```text
test to investigation
test to file
test to page
page to quantity
investigation to hardware
investigation to fire-safety need
investigation to the crew atmospheres its tests cover
model parameter to the source that justifies it
model to the tests it was scored on, with the score
a table value to the sensor value that contradicts it
table value contradicted by sensor value
```

These expand the owner's abbreviated repeated “to” phrases only; they do not define a new
ontology. Actual endpoints and evidence must be resolved. Parameter-source and model-score
edges are imported by authorized modelling/evaluation owners and never enter blind labeller
packets. A contradiction relation preserves both source assertions and their matching evidence;
it does not decide which source is true. New relation meanings remain unresolved proposals.

The additional exact discrepancy-edge string comes from step 4b and can be emitted by a
deterministic script with null model identity, its real method/tool/config provenance, both
source dependencies and locators. The separate discrepancy record keeps both values, method,
independent checks and cause/verdict; this label tool does not perform that comparison or
manufacture a verdict. Rule 10 prohibits consecutive-test chamber-fill differencing even when
the fields are called conditions. Outcome discrepancies and their edges remain protected.
Scientific finding/refuter checks and step-9 application handoff are separate requirements;
a structurally valid private finding is not proof that a refuter ran or that publication is approved.

## Clusters and author projection

The implemented rule language is deliberately concrete: `set_equals` or `contains_any` over
one supported atlas or owner page-kind field, with a versioned rule ID, schema/vocabulary pins, complete
input universe, `missing=unresolved`, explicit overlap permission and `unit_conversion=null`.
It does not infer flow regimes, define new physical bands or use embeddings. `CREW` labels
may be consumed only with their original rule attribution; this tool does not recompute crew
bands or silently treat oxygen-only candidates as members. Numeric interval rules and further
material/topic/event clusters need an approved extension and implementation.

Dependency validation rejects missing sources, missing edge endpoints, cycles and digest
mismatches. The manifest binds every transitive source/record hash and test dependency,
the producer code digest, receipt-set digest and every output file digest. No new atlas row
or application schema is emitted.

`author_projection(...)` is a host-only API. It validates the dependency closure first and
checks every record through `gate.read_author`, and external sources through `gate.decide`,
before returning any record for aggregation. The trusted gate must independently pin the
SHA-256 of the **full canonical envelope bytes**, including `content_sha256`; that is different
from the envelope's self-hash. Gate screening must include the page/test dependencies and
authoritative frozen splits. Unknown reveal states and unresolved test mentions fail closed
even when another dependency looks harmless. Protected pages also protect their edges,
clusters, missingness patterns and findings. Nothing is released by the CLI.

Integration uses `AuthorGate` from `author_gate.py`. A live gate's reveal recorder and screened
metadata are supplied by the host, not by a label or an author request. The default `live()`
gate denies access. Record inspection and real author projection still require the custodian
to approve exact bytes and wire the live gate; this tool does not claim that integration ran.

## Minimal synthetic producer example

This is a runnable format example only. A real labeller changes `origin` to `source_evidence`
and receives its actual producer object and separately captured receipt from the trusted host;
it never copies the synthetic identity. A separate assignment ID is needed for a new runtime
assignment; several rows from the same sealed assignment can share its receipt.

```python
from tools.wsp.astra import label_records as lr

defs = lr.definitions()
raw = b"Synthetic apparatus description."
h = lr.digest(raw)
locator = {"source_id": "fixture-document", "source_sha256": h, "page": 1,
           "selector": {"kind": "text", "start": 0, "end": len(raw),
                        "quote": raw.decode(), "extraction_sha256": h}}
producer = {"kind": "model", "id": "synthetic-runtime",
            "model_id": "fixture/provider-model", "model_version": None,
            "version_limitation": "Synthetic example; no real runtime claim.",
            "assignment_id": "synthetic-A", "runtime_session_id": "synthetic-session-A",
            "prompt_sha256": h, "tool_sha256": h, "config_sha256": h}
enrichment = {"status": "not_applicable", "reason": "No additional terms in this synthetic example.",
              "grounded_terms": [], "retrieval_cues": [],
              "provenance": {"method": "sira-inspired-page-enrichment/1",
                             **{k: producer[k] for k in
                                ("assignment_id", "prompt_sha256", "tool_sha256", "config_sha256")}}}
record = lr.seal({
    "envelope_version": lr.VERSION, "origin": "synthetic_fixture",
    "record_id": "fixture-document:page:1", "kind": "page",
    "schema_version": defs.schema_version, "schema_sha256": defs.schema_sha256,
    "vocabulary_sha256": defs.vocabulary_sha256, "owner_authority": defs.owner_authority,
    "producer": producer, "created_at": "2026-11-14T12:00:00Z",
    "locators": [locator], "dependencies": ["fixture-document"], "test_dependencies": [],
    "labels": [{"kind": "page.kind", "state": "assigned", "value": ["apparatus"],
                "reason": None, "basis": "interpretation", "rule": None, "locators": [locator]}],
    "body": {"page_id": "fixture-document:page:1", "document_sha256": h, "page_number": 1,
             "investigation_ids": [], "mentioned_test_ids": [], "unresolved_test_mentions": [],
             "reveals_outcome": False, "revealed_test_ids": [],
             "quantity_evidence": [], "condition_evidence": [], "search_enrichment": enrichment,
             "review": {"confidence": "high", "figure_or_table": False, "label_disagreement": False,
                        "requires_strong_review": False,
                        "outcome_decision": {"method": "strong", "assignment_id": "synthetic-A", "rule": None}}},
})
sources = {"fixture-document": h}
receipts = {"synthetic-A": producer}  # Real receipts are captured separately by the host.
lr.validate_bundle([record], sources, defs, receipts)
plan = lr.immutable_export([record], sources, defs, receipts, "synthetic-label-output")
assert plan["execute"] is False  # No directory/file is written.
```

For an edge, use `kind="edge"`, a new record ID, evidence locators and source/target
dependencies, and body `{"source": source_id, "target": target_id, "relation": "page to quantity",
"state": "assigned", "reason": null}`. `sources` must bind each external endpoint to its
real evidence/identity artifact hash; do not fabricate endpoint IDs or digests. A source/target
that is another record refers to its record ID and inherits its dependency closure. Use
`definitions()` and `seal()` after constructing each record; do not hard-code current hashes.

## Frozen independent audit

There are two distinct label checks, both separate from the one-in-ten repeat of numerical
measurements:

- **Section-2 Terra qualification:** `qualification_audit(universe, defs, effort="high")` or
  `effort="xhigh"` (CLI `qualify`, input `{universe, reasoning_effort}`) requires the same frozen
  **200 pages for every kind and both efforts**, including missing
  and uncertain assignments. It compares all 200, never a one-twentieth subset. Strong A stops
  general page annotation at this sample; Terra supplies the independent same-pass cards and
  enrichment. The original strong records and historical pins remain unchanged. The sample is
  a finite convenience sample, not evidence of representative whole-corpus accuracy.
- **Section-4 ongoing label audit:** `sample_audit` uses the random one-twentieth/minimum-100
  procedure below. Do not substitute it for the explicit all-200 qualification.

Both use the same blind packet, original-ledger sealing and comparator. The qualification
comparator requires distinct runtime sessions and an actual Terra B model identity. It reports
`agreement` (including matching unresolved states) separately from `resolved_agreement`
(matching assigned/absent closed values, with all 200 still in the denominator). Only
`resolved_agreement >= 0.90` yields `terra_qualification=eligible_for_this_kind`. Matching nulls,
missing outputs or unavailable vocabularies never qualify a kind. This is a report, not a
signature or access grant; the host must bind it to independently pinned sample, definitions,
runtime receipts and sealed ledgers before accepting it. Free-form generated synonyms/search
phrases receive separate grounding/error review and later retrieval evaluation; there are no
invented reference phrases on the legacy strong side.

The effort-specific manifest is `astra.private.page-qualification/2`; its digest includes
`reasoning_effort`. High and xhigh use independent B assignments/receipts and separate reports,
never a mixture of successful rows from both. `compare_audit` additionally requires host-captured
`request_receipts`, keyed by B assignment ID, with exactly:

```text
{assignment_id, request_id, model_id, reasoning_effort, config_sha256,
 wall_time_seconds,
 usage: {input_tokens, output_tokens, total_tokens,
         input_tokens_details?, output_tokens_details?}}
```

Identity/config fields must match the original producer; effort must equal that audit's effort.
Wall time must be finite/nonnegative and token counts must be nonnegative consistent integers.
Shared per-request metrics are deduplicated by request ID; conflicting copies fail. The worker
retains its complete provider/HTTP receipts, including failures and repairs, separately.
Qualification CLI comparison bundles add `request_receipts` to the normal fields.
`select_qualified_effort(high_report, xhigh_report)` requires identical frozen universe/kinds
and 200-page denominators. It selects high only when every assessed kind passes independently;
otherwise it selects xhigh if that run passes every kind, or requires repair/repeat. It does
not average, pool or substitute rows between efforts. Historical qualification v1 remains
replayable under the corresponding historical definition pins.

The general random audit procedure follows.

1. Before inspecting A's labels, create the census independently: every source-unit ×
   label-kind, including negatives, uncertainty and missing first assignments. Each unit has
   exactly `unit_id`, `kind`, `source_id`, `source_sha256`, `dependency_group`. The last field
   is the originating document/test for dependence-aware uncertainty. `freeze` takes
   `{units:[...]}` and binds its sorted contents to schema/vocabulary digests.
2. `sample` takes `{universe, seed}`. For each kind with population `N`, it selects
   `min(N,max(100,ceil(N/20)))` without replacement. A missing seed is generated randomly;
   a supplied seed must be frozen before assignments are inspected, not tuned after feedback.
   The manifest retains the seed, original universe hash, exact sample, inclusion probability
   and the `N<100` disclosure. No positive-only sampling or replacement of difficult units.
3. Seal the complete first-assignment ledger. `assignment_ledger_hash` canonically hashes
   the sorted `{unit_id,kind,assignment}` rows. `packet` takes
   `{universe,audit,first_assignments_sha256}` and generates independent random aliases.
   Only its `packet` member goes to B. `host_routes` includes the first-ledger hash and
   alias-to-unit mapping and stays with the custodian. The packet contains only task ID,
   frozen definitions digest, label kind and aliased source path/digest. It contains no A
   labels, A producer, partition, source identity, hypothesis or expected class. The host
   stages the original screened source at each opaque path and supplies the pinned definitions;
   names alone do not redact embedded annotations or model information.
4. A/B assignments contain `{state,value,assignment_id}`. `state` also supports `failed` for
   the audit, with null value. B keys are packet aliases; A keys are source-unit/kind pairs.
   The CLI `compare` bundle is `{universe,audit,packet,first,second,runtime_receipts}`;
   `first` is a list of `{unit_id,kind,assignment}`, `second` maps aliases to assignments.
   Different assignment IDs and independent runtime-session IDs are mandatory; the same
   model family can label independently in separate clean contexts. Runtime receipts identify
   the actual assigning models. The first ledger must still match its pre-dispatch hash.
5. Comparison replays the frozen random selection and reports by kind: exact set agreement,
   confusion counts, explicit disagreements, effective count, source dependence groups and
   a descriptive cluster-robust standard error when at least two groups exist. Missing and
   failed responses remain in the denominator and cannot count as agreement. Agreement on
   unresolved states is visible in the confusion table; it is not successful classification.
6. Below 0.90, the action is `redefine_and_relabel_entire_kind_domain`: repair the definition
   with the vocabulary owner and relabel the **entire affected kind/domain**, preserving
   the old version. The tool does not relabel automatically or permit replacing the sample.
   After repair, repeat the frozen audit and independently preselect an unexposed audit where
   possible; disclose exhausted populations and exposed repeats. Review rare-class confusion
   even when overall agreement passes. Agreement measures reproducibility, not truth accuracy.

The custodian seals B before comparison and retains packet/receipt/assignment artifacts.
All these artifacts and agreement summaries inherit protection; no audit statistic may be
shown to model authors merely because it is aggregate.

Blind B must receive only its own `packet` member, pinned vocabulary/owner definitions and
the custodian-staged original evidence listed there. B must never open A's output under
`runs/dev/data_astra/work/labels/`, A's assignment/receipt ledger, any `host_routes`, the
sampling manifest/seed or comparison/adjudication outputs. It must not browse the full
`work/document_pages/` tree: only specifically staged, unannotated source evidence is allowed.
Do not give B `docs/wsp/atlas/splits/`, model/config/release files, `docs/wsp/feedback/`,
`runs/evaluation/`, `holdout/`, `runs/dev/bass_front/val_sealed/`, `runs/dev/holdout_ext/` or
per-run validation series. These paths either disclose partitions/model expectations or are
outside this protected document-labelling assignment. Raw documents may identify tests and
report outcomes; B is the protected non-modelling labeller and must not receive A's judgements
or scores attached to that content. Comparison occurs only after B's assignment is sealed.

The audit supports `page.kind` and `edge.relation` using the owner lists, alongside existing
atlas label kinds. Relation-set agreement is not a test of endpoint-join correctness: the
custodian separately checks endpoint IDs and their evidence during edge review.

## Verification and remaining work

Executed offline on synthetic fixtures:

```powershell
python -m unittest discover -s tools/wsp/astra -p test_label_records.py -v
```

Twenty-seven tests pass at this revision: source/schema/content binding; exact selectors; actual
runtime receipt enforcement; missing vocabularies; closure/cycles; mixed and unknown page
protection; protected edge propagation; rule membership and missingness; interpretation
typing; census gaps; audit sample boundaries, independence, first-ledger sealing, set equality
and missing-response denominators; current-authority/schema drift and bad-heading refusal;
explicit legacy Git replay and unchanged legacy bytes; same-pass enrichment provenance,
grounding and novelty; conservative Terra protection and mandatory strong review;
all-200 qualification with no pass from matching unknowns; metadata-only legacy/current
compatibility; immutable-snapshot independence from worktree changes; explicit effort separation,
request timing/token binding and no mixed-effort selection; dry-run behavior and immutable
collisions. No Rust files
were changed; Rust/workspace gates and live model calls were not run for this tool.

Still required before application integration: approval of genuinely missing scientific vocabularies
and eventual shared application contracts (not a blocker for the owner-requested private page
cards/edges); authorized real-source annotation and locator verification; trusted runtime receipt
capture and source staging; production custodian screening/ledger wiring; model/evaluation
owners' parameter-source and score-artifact imports; actual independent audits and subsequent
whole-domain relabelling where required. These are not implemented semantic outcomes.
