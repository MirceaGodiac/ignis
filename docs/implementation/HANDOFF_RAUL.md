# Handoff — Raul: Scientific Data, Retrieval & Knowledge

Dă agentului tău: acest fișier, `docs/implementation/LANES_V2.md` (§5, §9, §10), `AGENTS.md`, `docs/implementation/CONTRACTS.md` și
`docs/design/IGNIS_MASTER_V2.md` §10–§14 și §16. Branch: unul scurt per task din `main` (`feat/scientific-data-<task>`).

## 0. Faza 2 (citește întâi) — cunoaștere & date NASA

**Acces:** [ACCESS](ACCESS.md). Ai nevoie de GitHub și, la R8, de `TYPESAFE_API_KEY` (Jev, de la Mircea). Fără DeepInfra: nici embeddings, nici reranker. Corpusul complet se descarcă **pe VM** (`/srv/ignis/data/raw/`, pas în `build-data.sh` rulat de Mircea, cu selecția ta din atlas); local ții doar un subset.

Detalii, motivare, surse: [LANES_V2](LANES_V2.md) §5 (lane-ul tău), §9 (contracte), §10 (date NASA și
testele de acces măsurate). Regula de stocare: atlasul lui Iancu, `docs/wsp/atlas/`.

**Misiune:** toate cele 19 investigații de zbor sunt în sistem: documente deschise la locul citat, text
de senzori ca dataset-uri tipizate, condiții comparabile între investigații, etichete și graf pe care
agentul le poate interoga, iar media NASA rămâne la NASA.

**P0, în ordine:** (1) deschizi PR-ul pentru `feat/scientific-data-k1` (`evidence.v2` + `dataset.v2`,
deja scris; intră primul); (2) **R6** corpusul după atlas: 636 documente (1,63 GB) + 24.482 fișiere text
(12,26 GB) cu hash, 35.446 fișiere media (13,44 TB) doar în manifest; licența PSI-10 de verificat;
**test de lockbox** pentru fiecare document nou din FLEX, CFI-G, BASS-II, BASS; (3) **R9a**
`ConditionsTable` pentru toate cele 19, din cele 21 de CSV-uri „Experimental table” (cel mai mare câștig
pentru challenge, cost mic); (4) **R7a** parse views cu bbox, parser ales prin măsurători (unele
rapoarte PSI sunt scanate); (5) **R12a** retrieval în stil SIRA (arXiv 2605.06647, cod MIT
`facebookresearch/sira`): **embeddings, reranker și clientul DeepInfra eliminate**, schița de termeni a agentului + filtru DF +
BM25 ponderat; PR-ul de contract K2 (`evidence.v3`: `expected_terms`). **P1:** R12b expansiunea de
vocabular la indexare (în aceeași trecere LLM cu R7b/R8), R7b arbore de secțiuni, R8 etichete (gold set ~100), R2b senzori →
Parquet, R9b graf, R11 indexuri media (`.ts`, ZIP, TIFF → PNG). **P2:** R4 wiki/web, R10.

**Cum alegi singur:** metoda din LANES_V2 §3. Parsere: 10 PDF-uri PSI (2 scanate, 2 cu tabele, 2 lungi,
2 prezentări, 2 documente inginerești), comparat pe acuratețea bbox, a celulelor de tabel, secunde CPU
pe pagină și licență. Etichete: Jev vs model mic cu output structurat pe gold set. SIRA: BM25 simplu vs
expansiune de query vs query + document pe întrebările de retrieval din T10 (nDCG@10, recall@10, p95);
τ și w se aleg acolo, nu din lucrare (domeniul nostru e de nișă). Graf/PPR: doar când întrebările
multi-hop din benchmark-ul lui Tudor arată un gol; fără indexuri de embeddings.

**Eficiență:** ingestie incrementală pe (investigație, fișier, versiune, mărime); parsare o dată per
(hash, parser, versiune); expansiunea de vocabular în batch, cache pe (hash chunk, model, prompt); senzori în Parquet zstd
citit cu proiecție de coloane; prompt static înaintea documentului (cache); directorul central al
fiecărei arhive ZIP în cache (0,44 MB pentru 18 GB, apoi 2 cereri per cadru); raportezi documente/oră,
CPU/pagină, cost per 1.000 de etichete, recall și p95.

## 1. Obiectiv

Materialul științific comun pentru modele și harness: originale cu identitate și drepturi, observații
numerice cu unități și timp, parse views și citări verificabile; un singur serviciu de căutare hibridă
și citire folosit de CLI, HTTP și MCP; web discovery cu snapshot; wiki și muchii de proveniență;
observațiile BASS din video. Corpus util și selectiv, nu mirror de 3,45 TB și nu plafon arbitrar.

## 2. Ce există și se reutilizează

- `crates/ignis-data`: discovery (`sources/psi.rs`, `manifest.rs`, `intl/*`), manifeste
  `data/manifests/<ACC>.jsonl` (un `Source` per asset, id = sha256 bytes) + `.listing.jsonl`, familii
  (`families/{flex,acme,bass}.rs`), store `data/work/records/`, cohorte și split-uri înghețate
  (`cohort.rs`), views Parquet (`views.rs`), coada de review (`review.rs`, `auto.rs`).
- `crates/ignis-evidence`: catalog SQLite + FTS5 (`schemas/evidence_catalog.sql`), `firewall` (dependency
  graph + `strip_for_sealed`). *(Faza 2: dense, RRF și rerank-ul prin DeepInfra se elimină în R12a.)* Rest:
  graph + `strip_for_sealed`), parser PDF (feature `pdf`) care produce `SourceLocator`.
- `crates/ignis-vision` + `sidecar/sam2`: PTS din container, Otsu sau SAM2.1, contur pe grilă K,
  calibrare DLT + Monte Carlo → `Calibration`, `ContourObservation`.
- Contracte: `Source`/`IdBasis`/`Rights`, `SourceLocator` extins (pagină fizică 1-based, `page_label`,
  `parse_view_id` + `offset_unit`, `bbox` normalizat, `table_id`), `EvidenceRef`, `SearchRequest/Page`,
  `ReadRequest/Result`, `ProvenanceLink`, `WikiPatch/Revision`, `DatasetBundle`,
  `VisualObservationBundle`; trait-urile `KnowledgeService`, `DatasetService`.
- Fixture-uri: `fixtures/contracts/{evidence,data}/`.

## 3. Ce lipsește

- Implementarea `KnowledgeService` (search/read/resolve) cu ieșire tipizată `SearchPage` (azi
  `ignis-evidence` întoarce `serde_json::Value` și n-are paginare).
- Parse views versionate cu offseturi documentate; reader de pagină/pasaj/tabel.
- `DatasetService` + bundle-uri FLEX/ACME (Parquet + `ColumnSpec` + `time_origin` + quality report) și
  bundle-ul separat al evaluatorului cu outcomes de test (`audience = evaluator`).
- Corpus manifest / snapshot reproductibil (`CorpusSnapshotId`), web discovery cu drepturi și snapshot,
  wiki cu CAS, `ProvenanceLink`.
- BASS: clip→run, calibrare, `VisualObservationBundle` cu `units_status` corect.

## 4. Fișiere deținute

`crates/ignis-data/` (discovery și numeric), `crates/ignis-evidence/`, `crates/ignis-knowledge/` (nou,
doar dacă separarea îl justifică), `crates/ignis-vision/`, `sidecar/sam2/`, `configs/evidence/`,
`configs/vision/`, `data/`, `schemas/evidence_catalog.sql`, `crates/ignis-core/src/contracts/{evidence,dataset}.rs`,
`fixtures/contracts/{evidence,data}/`.

## 5. Contracte

Produci: `SearchPage`, `ReadResult`, `EvidenceRef`, `ProvenanceLink`, `WikiRevision`, `DatasetBundle`,
`VisualObservationBundle`, `Source` (manifeste). Consumi: `ExecutionScope` (filtrare pe partiții și pe
corpus snapshot), `ArtifactStore` (Parquet, snapshoturi, parse views).

## 6. Interfețe

Mircea înregistrează tool-urile `sources_search`, `sources_read`, `datasets_inspect`, `wiki_patch`
peste serviciile tale (fără reimplementare). Iancu și Tudor consumă bundle-urile prin `DatasetService`.
Tudor afișează `EvidenceRef` în viewer (pagină, bbox, quote).

## 7. Dependențe

R1/R2 depind doar de C0. R3 după R1. R5 după R2, dar nu blochează R3. Nu aștepta corpusul complet:
prima predare = un document primar cu citare verificabilă + un rezultat de search + un bundle numeric.

## 8. Sugestii tehnice

- Direcția (faza 2): BM25 cu expansiune SIRA (R12), fără embeddings și fără reranker; migrarea de la
  FTS5 la Tantivy o decizi pe măsurători (recall, latență, memorie). Indiferent de backend:
  o singură bază de adevăr pentru BM25 și expansiune, generații de index construite separat și activate
  prin pointer atomic, citările vechi rămân rezolvabile.
- `SearchPage.backends` raportează fiecare backend (`used`/`unavailable`/`not_requested`) — o etapă
  care n-a rulat (ex. expansiunea) e vizibilă, niciodată tăcută. `total_count` absent când nu e exhaustiv.
- Paginare cu cursor opac (codifică snapshot + poziție); fără plafon semantic de rezultate.
- `strip_for_sealed` sau echivalent aplicat în serviciu pe baza scope-ului, nu la latitudinea clientului.
- Bundle-urile numerice: Parquet columnar, `ColumnSpec.unit` în SI, `ColumnRole::Time`/`AvailableAt` în
  secunde de la `time_origin`; valorile originale + SI + transformarea rămân legate (`Measured`).
- `dataset_id` stabil (nu path local); `DatasetBundle::content_digest()` e ce citează evaluarea.
- MacBook: verifică MPS efectiv pentru SAM2; 24 GB memorie unificată nu înseamnă CUDA.

## 9. Riscuri

- `ignis-vision` scrie azi contururi `pixel_only` ca `ContourObservation` (care înseamnă metri), cu
  unitatea doar în `derivation.notes` — trebuie emise ca `VisualObservationBundle{units_status: pixel_only}`
  cu alt contract de conținut; doar `metric` (cu calibrare) folosește `ContourObservation`.
- Serverul are un al doilea motor FTS (`crates/ignis-server/src/evidence.rs`) și rute fără
  `strip_for_sealed`; după R3, Mircea îl înlocuiește cu serviciul tău.
- O legătură doar semantică nu e identitate exactă (`resolution = unknown`).
- Nu indexa necontrolat PDF-uri de test ca articole; publicațiile care conțin rezultate de test moștenesc
  restricția de lockbox.

## 10. Criterii de finalizare

Sursa se deschide la locator (pagină + bbox/offset în parse view-ul declarat); datele păstrează valori
originale și SI; identitatea incertă e explicită; snapshotul de corpus e reproductibil; tool-urile
funcționează prin contract (fixture + test de serviciu); `SearchPage`/`ReadResult`/bundle-urile trec
`validate()`; un test arată că un scope fără `test` nu primește outcomes de test.

## 11. Ce NU faci

Nu descarci 3,45 TB; nu faci obligatorie extragerea fiecărui finding în SQL; nu inventezi metri din
pixeli; nu schimbi labels/splits după ce vezi rezultate; nu modifici unilateral contractele comune;
nu operezi infrastructura lui Mircea; nu scrii modelul BASS.

## 12. Predare

PR per task (R1…R5) cu manifeste, snapshot-uri, coverage de corpus, bundle-uri, fixture-uri actualizate
și probe de retrieval/citare; constructorii publici ai serviciilor pentru composition root.

## Prezentare

Dovezile NASA: bibliografie, explicația surselor și a proveniența, verificarea fiecărei afirmații de pe
slide-uri. Nu operezi infrastructura.
