# Contracte C0 — referință

Sursa de adevăr pentru orice tip care traversează granița dintre lane-uri este Rust, în
`crates/ignis-core/src/contracts.rs` (modelul de date științific, `SCHEMA_VERSION` 2.1.0) și
`crates/ignis-core/src/contracts/*.rs` (suprafața C0). JSON Schema (`schemas/`) și TypeScript
(`web/src/contracts/generated.ts`) sunt **derivate**; nu se editează manual și nu există YAML/OpenAPI
scris de mână paralel.

```text
Rust (ignis-core) ──ignis schemas export──▶ schemas/*.json ──npm run gen:types──▶ web/src/contracts/generated.ts
        ▲                                         │                                     │
        └──── cargo test -p ignis-core ───────────┴── fixtures/contracts/*.json ─────────┘
              (schema drift, fixture vs schema, round-trip, invariante)      npm run check:types (drift TS)
```

## Familii, owneri, tag-uri

| Modul | Tag `schema_version` | Owner semantic | Review obligatoriu | Tipuri |
|---|---|---|---|---|
| `contracts.rs` | `SCHEMA_VERSION` (în `$id`) | Iancu (istoric) | consumatorul afectat | `Source`, `Observation`, `SourceLocator`, `PrefixBundle`, `ModelRelease`, `AnalysisArtifact`, `Cohort`, `Split`, `Partition`, … |
| `contracts/common.rs` | — | comun | câte un reviewer din fiecare lane consumator | id-uri, `Digest`, `Timestamp`, `Finite`, `Origin`, `ExecutionScope`, `ArtifactRef`, `EntityRef`, `InlinePayload`, `ArtifactReadRequest`, `ErrorEnvelope` |
| `contracts/research.rs` | `research.v1`, `tools.v1` | Mircea | Tudor (UI) | `ResearchEpisode`, `ResearchEvent`, `JobRef`, `ToolResult`, `ToolDescriptor` + `tool_catalog()`, `ResearchAnswer` |
| `contracts/evidence.rs` | `evidence.v1` | Raul | Mircea (transport), Tudor (viewer) | `EvidenceRef`, `SearchRequest/SearchPage`, `ReadRequest/ReadResult`, `ProvenanceLink`, `WikiPatch/WikiRevision` |
| `contracts/dataset.rs` | `dataset.v1` | Raul | Iancu, Tudor (consum numeric) | `DatasetBundle`, `VisualObservationBundle` |
| `contracts/wsp.rs` | `wsp.v1` | Iancu | Tudor (evaluabilitate) | `ModelSpec`, `HandwrittenWeightPatch`, `ModelBuildManifest`, regulile `ModelRelease::check_construction`, `ForecastRequest` |
| `contracts/evaluation.rs` | `evaluation.v1` | Tudor | review științific înainte de freeze | `EvaluationRequest`, `EvaluationResult` |
| `services.rs` | — | comun | Mircea (composition root) | `KnowledgeService`, `DatasetService`, `ModelService`, `ArtifactStore`, `EvaluationService` |

## Reutilizat, nu duplicat

- **Versiunea unei surse** este `Source` existent: `Source.id` = `SourceId`, `IdBasis` spune dacă id-ul
  dovedește bytes (`bytes`) sau doar o listare/metadate. Nu există `SourceVersion` separat.
- **Locatorul** este `SourceLocator` existent, extins aditiv (`page_label`, `parse_view_id`,
  `offset_unit`, `bbox`, `table_id`). Câmpurile noi sunt omise când lipsesc, deci forma serializată a
  locatorilor vechi — inclusiv `input_hash`-urile prefixelor care îi conțin — nu se schimbă (test).
- **Releases** rămân `ModelRelease` + registry-ul content-addressed. Construcția WSP intră ca
  `ModelRelease.construction: Option<ModelBuildManifest>`, acoperit de id-ul release-ului.
- **Prefix, observații, artefacte de analiză**: `PrefixBundle`, `Observation`, `ContourObservation`,
  `AnalysisArtifact` neschimbate. `ForecastSettings` (din `analysis.rs`) este reutilizat în
  `ForecastRequest` și `EvaluationRequest`.

## Convenții de wire (suprafața nouă)

| Regulă | De ce | Unde e verificată |
|---|---|---|
| `deny_unknown_fields` pe orice struct | un typo sau un câmp al altei versiuni nu trece tăcut | serde + JSON Schema `additionalProperties: false` |
| câmpurile opționale sunt **omise** când lipsesc; lipsa ≠ zero | „unknown/null nu înseamnă zero” | test round-trip pe fixture-uri |
| `schema_version` pe fiecare document de nivel superior, tip literal | un payload al altei familii/versiuni nu se parsează | `wire_primitives_reject_ambiguous_values` |
| id-uri tipizate (`SourceId`, `ArtifactId`, `ReleaseId`, …), nevide, fără spații | nu amesteci sursă/versiune/artefact | compilator + deserializare |
| `Digest` = `sha256:<64 hex>` doar pentru identități de conținut | un `listing_id` nu e hash de fișier | deserializare |
| `Timestamp` = RFC 3339 UTC cu `Z`; timpii experimentali = secunde de la `time_origin` explicit | nu pierzi originea timpului | deserializare + `ColumnRole::Time` în `s` |
| `Finite` pentru numerele care trebuie să fie finite | serde_json scrie NaN ca `null` (= „lipsă”); `Finite` refuză la producător | `Finite` Serialize/Deserialize |
| `origin: system \| synthetic_fixture` pe payloadurile afișabile | fixture-urile nu devin rezultate NASA | `Origin::combine`, `EvaluationResult::validate` |
| `ArtifactRef` în loc de bytes mari/path/URL | fără copii, fără path-uri locale, URL-uri care expiră nu schimbă identitatea | `ArtifactRef::validate` |
| `ExecutionScope` e argument separat al serviciilor, derivat de host | modelul/documentul nu poate lărgi accesul | semnăturile din `services.rs` |

Unitățile: datele sunt stocate în SI (`Quantity::unit_si`); afișarea în mm nu schimbă contractul.

## Invariante (cod: `validate()` lângă tip; teste: `crates/ignis-core/tests/contracts.rs`)

| Risc | Invariant | Funcție |
|---|---|---|
| model handwritten etichetat NASA-trained | `TrainedOn` fără default; `not_trained` ⇔ manifest literal; literal ⇒ fără posterior, `experimental`, lineage `literal_authoring`/`weight_edit`, `weights_hash` = tensorii, parent = `parent_id`; registry-ul verifică la save și load | `ModelRelease::check_construction`, `registry::save_release/parse_release` |
| patch nevalid / cod ascuns | doar `set`/`set_box`/`fill_box` cu valori `Finite`; indici/box în limite; fiecare element scris o dată per patch; tensorii `fixed` nu se scriu; patch-ul rădăcină scrie complet tensorii `author_writes_all`; valori în `bounds`; `spec_hash` corect; expunerea fără `test` | `HandwrittenWeightPatch::validate_against` |
| feature order schimbat implicit | release ↔ spec: aceeași ordine, unități, set de tensori și shape-uri | `ModelSpec::check_release`, `registry::load_release(expect_features)` |
| W0 contaminat | W0 fără parent, fără feedback, fără candidatul celuilalt autor; literal ⇒ forward `strict_no_fitting`, fără `calibrated_prediction` | `ModelBuildManifest::validate` |
| acces la final-test | doar `ScopeRole::Evaluation` poate avea `Partition::Test`; scope-ul copil ⊆ părinte; authoring fără `execute_code`; bundle `builder` fără `test`; `development_feedback` nu rulează pe `test` | `ExecutionScope::{validate, require_partition, derive_child}`, `DatasetBundle::validate`, `EvaluationRequest::validate` |
| eroare tehnică amestecată cu rezultat științific | `execution_status` ≠ `scientific_status`; `accepted` ⇒ job; `failed/refused/cancelled` ⇒ eroare, fără rezultat, `not_assessed` | `ToolResult::validate` |
| preview luat drept rezultat complet | `partial = true` ⇒ `full: ArtifactRef` | `InlinePayload::validate` |
| predicție raportată ca măsurătoare | `observed` ⇒ doar evidență `original_observation`/`source_derived`; `predicted` ⇒ valoare dintr-un `AnalysisArtifact`; `computed` ⇒ valoare de artefact | `ResearchAnswer::validate` |
| evenimente pierdute la reconectare | `seq` contiguu per episod, id-uri unice, tranziții de status consistente, job-urile nu ies din stări terminale | `check_event_log` |
| rezultat necalificat prezentat ca validat | `qualified` doar din `final_test`, `origin = system`, cu cazuri scorate; refuzurile rămân în numitor; coverage = scored/total | `EvaluationResult::validate` |
| pixeli prezentați ca metri | `pixel_only`/`normalized` nu pot fi `ContourObservation`; `metric` cere calibrare; consumatorul cere `require_metric()` (și identitate rezolvată) | `VisualObservationBundle::{validate, require_metric}` |
| căutare confundată cu documentul | hit = `EvidenceRef` + snippet; citirea prin `ReadRequest`; `ReadResult` incomplet ⇒ `next`; `total_count` absent când nu e exhaustiv; backend lipsă ⇒ `unavailable` | `SearchPage::validate`, `ReadResult::validate` |
| citare neverificabilă | offseturi ⇒ `parse_view_id` + `offset_unit`; bbox în [0,1] pe pagină fizică 1-based | `SourceLocator::validate`, `EvidenceRef::validate` |

## Streaming (contractul SSE pentru Mircea și Tudor)

- Un stream per episod: `GET /api/v1/episodes/{id}/events?after=<seq>`. Fiecare mesaj SSE:
  `id: <seq>`, `event: <body.kind>`, `data: <ResearchEvent JSON>`. Heartbeat ca comentariu SSE.
- Clientul reia cu `Last-Event-ID` sau `?after=` și deduplică după `(episode_id, seq)`. Reconectarea
  nu reexecută tool-uri și nu retrimite comanda de creare (`client_request_id` = idempotency key).
- Dacă jurnalul nu mai acoperă cursorul: `410 Gone` cu `ErrorEnvelope{code: conflict}`; clientul
  reîncarcă `GET /episodes/{id}` și continuă de la `last_seq`.
- `message.delta` nu e singura copie: `message.completed` și `answer.completed` (artefact
  `ResearchAnswer`) sunt snapshot-urile.
- Evenimente Codex necunoscute → `runtime.opaque` (arhivate, neinterpretate).

Endpointurile propuse (§17 din design) și maparea lor pe contracte: `INTEGRATION.md`. OpenAPI se
**generează** din handler-ele Axum când există (task M2); nu se scrie manual.

## Fixture-uri

`fixtures/contracts/manifest.json` listează fiecare fixture, contractul și ce demonstrează. Toate sunt
`origin = synthetic_fixture`, cu texte care spun explicit „Synthetic fixture”. Hash-urile din ele sunt
calculate de Rust (spec → patch → tensori → release id → request hash) și verificate de
`fixtures_are_mutually_consistent`. Tensorii release-ului fixture sunt reconstruibili exclusiv din spec
+ patch (testul face materializarea literală).

Pentru UI/mock: citiți fixture-urile direct (`import` JSON sau fetch static). Nu există mock server în
C0 — nu a fost necesar; dacă devine necesar, Mircea servește aceleași fișiere prin aceleași rute.

## Cum schimbi un contract după C0

1. PR mic, separat de implementare, deschis de ownerul semantic (tabelul de sus), review de la
   consumatori. Nimeni altcineva nu editează fișierul familiei.
2. În același PR: tipul Rust + `validate()` dacă e o invariantă, fixture actualizat/nou în manifest,
   `cargo run -p ignis-cli -- schemas export`, `npm --prefix web run gen:types`, consumatorii actualizați
   (sau confirmați neafectați), `cargo test -p ignis-core`, `npm --prefix web run check:types`.
3. **Breaking** (cu `deny_unknown_fields`, aproape orice): ștergere/redenumire de câmp, enum nou,
   schimbare de unitate, de default sau de semnificație, câmp nou obligatoriu. Breaking ⇒ tag nou
   (`research.v2`) sau `SCHEMA_VERSION` minor/major, plus reader de compatibilitate/migrare pentru date
   durabile (ex. `registry::parse_release`). Un câmp opțional nou omis când lipsește e compatibil pentru
   scriitori, dar cititorii vechi stricți îl refuză — coordonează cu consumatorul înainte de deploy.
4. Hash-urile vechi nu se recalculează sub o schemă nouă; obiectele content-addressed nu se rescriu.
