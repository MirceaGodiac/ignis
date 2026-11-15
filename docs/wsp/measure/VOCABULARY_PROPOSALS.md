# Pending vocabulary and integration proposals

Current authority is the whole-goal owner instruction at
`76fbb19244fb148ef94eef508896df5b81825c75`, including sections 2 (page cards/search enrichment),
4 (test labels/clusters/links/findings), 4b, 9, roles and rules. The original section-2b references below preserve
the provenance of these proposals; those requirements survive in the consolidated sections.
They do not direct readers to a now-missing heading. Explicit legacy replay and new v3 authority
pins are documented in [LABELS.md](LABELS.md#versioned-authority-and-preservation-of-legacy-records).

Status: additional scientific vocabularies and eventual shared application integration remain
pending. The seven page kinds and the relationships explicitly requested by the owner in §2b
remain authoritative for Astra's private outputs; they do not await a second approval
merely because the per-test atlas schema has no page/edge format. This file grants no extra
enum token, changes no shared schema and reports no experiment outcome. Its sources are the owner's
requirements and metadata-only schema, not NASA payloads. Protected labellers must add exact
source-page/figure/cell locators when actual evidence exposes a further gap; no such source
was opened by this implementation task.

The [original owner instruction, section 2b; consolidated section 4](../dossier/ASTRA_PROMPT_ALL_NASA.md#4-test-labels-clusters-links-findings)
required: “Where a list lacks a value, propose it in `docs/wsp/measure/VOCABULARY_PROPOSALS.md`
with the source that shows the need; do not invent a free label.” The proposals below preserve
that boundary: additional staging identifiers name unresolved work, never accepted scientific labels.
Current owner-list authority is `astra.owner.whole-goal/2`, bound to immutable Git metadata
and explicit full/section SHA-256 by `definitions().owner_authority` in `label_records.py`. Original
`astra.owner.section2b/1` metadata survives unchanged in explicitly replayed legacy records.

| ID | Exact requirement locator and wording | Existing authority and gap | Requested owner decision; current behavior |
|---|---|---|---|
| VP-01 | Owner §2b, Documents: “What kind of page it is (methods, apparatus, requirements, results table, figure, discussion, drawing)” | [SCHEMA §3–4](../atlas/SCHEMA.md#3-row-fields) concerns test rows; its absence of page kinds does not override the explicit owner instruction. | **Implemented owner scope:** all seven exact strings, multi-label where appropriate, with source evidence and version/hash authority. **Pending extras:** title/reference-only or other additional page kinds and eventual shared application contract. Blank/unreadable/unclassifiable pages retain null/unresolved with reasons when the seven listed meanings do not apply; no extra token is assigned. |
| VP-02 | Owner §2b, Tests: “what happened … as a value of the closed phenomenon list with its evidence” | [SCHEMA §4](../atlas/SCHEMA.md#4-closed-lists-in-full) and [§5 L1/L2](../atlas/SCHEMA.md#5-rules): `labels.flame_type` derives geometry and `labels.studied` derives investigation-title matches. Neither asserts a test outcome. | Define a distinct observed-event vocabulary, temporal/measurement evidence, reported-vs-observed semantics, negatives, ambiguity and conflict. Do not reuse `extinction`/`cool_flame` title topics as outcomes. `observed.phenomenon` stays null/unresolved. Actual event candidates require protected source evidence before approval. |
| VP-03 | Owner §2b, Documents: “which quantities it reports, with units”; §2, group-specific measurement list | [SCHEMA §4 `measurements.items.kind`](../atlas/SCHEMA.md#4-closed-lists-in-full) lists modalities (imaging, radiometer, etc.), not physical observables or their definitions. | Define physical-observable identifiers with measurement definitions, raw/SI units, time origin, uncertainty and qualification references. Retain raw quantity names/units and exact locators meanwhile; `physical.observable` stays unresolved. No diameter/front/end-time aliases are inserted into modality lists. |
| VP-04 | Owner §2b, Clusters: “flow regimes; classes of fuel and material” and “Documents by topic” | [SCHEMA §4](../atlas/SCHEMA.md#4-closed-lists-in-full) provides `flow.direction`, `fuel.phase`, geometry and investigation-title topics, but no transport-regime, material-class or page-topic lists. | Approve the three missing controlled lists and their rules. Existing flow direction/phase clusters remain usable; `flow.regime`, `material.class`, `document.topic` remain unresolved. Material species names stay source text, not a fabricated taxonomy. |
| VP-05 | Owner §2b, Links: “kept as tables of edges, not as a graph database”, followed by the full requested relationship list | [SCHEMA §3–5](../atlas/SCHEMA.md#3-row-fields) concerns test fields and derivation IDs; private edge envelopes are explicitly requested separately. | **Implemented owner scope:** the ten requested relationship meanings, documented as exact accepted strings in [LABELS.md](LABELS.md), with endpoints, evidence, dependencies and section-2b authority. **Pending extras:** any additional relation and eventual shared application contracts. Unresolved joins remain null/unresolved; authorised parameter/score imports remain their owners' responsibility. |
| VP-06 | Owner §2b, Interpretation: “It is marked as an interpretation and never as a measurement.” | [SCHEMA §2 `basis`](../atlas/SCHEMA.md#basis) concerns test values, not evidence-backed claim records. | Approve finding/claim integration with scope, domain, openable evidence, conflicting assertions and unknowns. Private findings explicitly use `assertion_kind=interpretation`; this creates no measurement or shared contract. |
| VP-07 | Owner §2b, Output: “Every row has the schema version, the label, its locator and the id of the model that assigned it.” | [SCHEMA §2 source and §3 labels](../atlas/SCHEMA.md#2-the-value-object) provides cell/page metadata and rule attribution but no exact spans/regions or runtime assignment identity. | Approve shared provenance/locator contracts. Private staging retains actual runtime ID/build limitation, assignment/session IDs, prompt/tool/config hashes, schema/vocabulary/content hashes and source selectors without changing the per-test atlas. |
| VP-08 | Owner §2b, Documents: “whether it reveals how a test ended, with the tests it reveals”; Rules 1/2 require frozen groups and separation from authors | [SCHEMA §3 `project_partition`, §8](../atlas/SCHEMA.md#8-outcome-separation) lacks page-reveal/test-link and transitive derived-record visibility fields. `outside_partitions` is not an author outcome grant. | Approve page-reveal tri-state, unresolved alias handling, source/test dependency identities and host-derived visibility integration. Private page/edge/finding closure fails closed for unknown/mixed dependencies; no caller partition grants exist. |
| VP-09 | Owner §2b, Check: “A second agent labels a random twentieth again, blind, at least 100 labels of each kind.” | Existing test schema has no assignment audit-unit, frozen universe, blind-repeat or adjudication contract. [Measurement design §11.5](ASTRA_MEASUREMENT_DESIGN.md#115-blind-random-one-in-twenty-label-audit-minimum-100-per-kind) spells out minimum/sample/denominator/whole-kind repair semantics. | Approve eventual application audit contracts. Implemented private artifacts freeze source-unit/kind coverage including absent/missing labels, apply the exact formula, separate aliases from host routes, retain producer receipts and require whole-kind/domain relabelling below 0.90. No real agreement has been measured. |

Approval must identify the accepting owner, versioned definition/schema and effective source
digest. Until then, pending values remain null with reasons. Acceptance triggers explicit
reprocessing of every affected record/kind/domain and a new immutable artifact version;
rejection and original evidence remain auditable. Protected evidence and proposals with outcome
content stay in the protected store and are not copied into author-readable documentation.

Same-pass enrichment is now an explicitly authorized private envelope capability, not a pending
scientific vocabulary extension. Source-used concepts/synonyms/abbreviations and named
instruments/materials/test IDs carry exact evidence. Generated NEW retrieval vocabulary is
kept in separate indexing-cue fields with grounding and dedicated same-assignment provenance.
It never becomes an observed-phenomenon token, source assertion or evidence for a model.
The new 200-page Terra/strong qualification applies to closed labels; enrichment grounding
and retrieval utility are separate evaluations, and legacy strong phrases are not fabricated.
Current qualification compares high and xhigh separately on the same frozen 200 pages, with
full-document context and per-request effort/time/token receipts. It never blends the efforts.
Step 4b additionally authorizes the scripted `table value contradicted by sensor value` edge;
this does not authorize LLM-assigned causes or replacement of NASA values. Rule 10's prohibition
of consecutive-fill differencing and step 9's application-only protected exports remain in force.
