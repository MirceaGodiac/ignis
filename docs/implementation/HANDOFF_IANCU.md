# Handoff — Iancu: Scientific Models & Literal Weight-Space Programming

Dă agentului tău: acest fișier, `docs/implementation/LANES_V2.md` (§7), `AGENTS.md`, `docs/implementation/CONTRACTS.md` și
`docs/design/IGNIS_MASTER_V2.md` §15–§16 și §24. Branch: unul scurt per task din `main` (`feat/handwritten-science-<task>`).

## 0. Faza 2 — lane-ul tău nu se schimbă; punctele de contact

Planul celorlalți: [LANES_V2](LANES_V2.md) (§7 pentru tine). Continui I1–I5. În faza 2: review pentru
semantica seriilor de predicție din `ViewSpec` (K1.4) și pentru unitățile din `ConditionsTable`;
conținutul skill-ului `model-qualification`; lockbox-ul pentru celelalte 15 investigații când li se
îngheață grupurile de evaluare; atlasul tău (`docs/wsp/atlas/`) e regula de stocare pe care o
implementează Raul (R6). Branch scurt per task din `main` (`feat/handwritten-science-<task>`).

## 1. Obiectiv

Modele FLEX, ACME (normal și inverse) și BASS ale căror ponderi sunt **scrise literal** de Fable/Astra
pe o specificație completă, asamblate fără nicio estimare, rulate printr-un forward strict și evaluate
de alt proces. Livrezi specificații, assemblerul literal, release-uri fără parent NASA-trained, forward
strict, lineage W0/W1 și model cards oneste.

## 2. Ce există și se reutilizează

- `ignis_core::registry` — releases imutabile content-addressed (`artifacts/models/<hex>/`, npz verificat
  la load, verificare feature order). `save_release` aplică acum `ModelRelease::check_construction`.
- `ignis_core::contracts::wsp` — `ModelSpec`, `TensorSpec` (`role`, `fill`: `author_writes_all` /
  `structural_zero` / `fixed`, `bounds`), `HandwrittenWeightPatch` (`set`, `set_box`, `fill_box`),
  `LiteralOp::assignments` (aritmetica row-major a indicilor), `validate_against`, `ModelBuildManifest`,
  `weights_digest`, `ModelSpec::check_release`, `ForecastRequest`.
- `ignis_core::prefix::build_prefix` + `assert_isolated` (izolare cauzală), `analysis.rs` (calea de
  forecast comună, `cache_key`), `PrefixBundle`, `AnalysisArtifact::validate`.
- Experții existenți: `ignis-flex` (dinamica, `structure.rs`: rețeaua reziduală `residual.W1 [h,nin]`,
  `a1 [h]`, `W2 [1,h]`, `a2 [1]`), `ignis-acme` (forecast cu draws stocate + Gauss–Hermite, fără
  optimizer), `ignis-bass`. Numerica din `ignis-science`. Rămân baselines declarate ca fitted.
- `ignis-lab/src/lab/` (show/tensor/diff/lineage) pentru inspecție.
- Fixture-ul toy: `fixtures/contracts/models/{model_spec,patch_w0,release_w0}.json` și materializarea
  literală din `crates/ignis-core/tests/contracts.rs::materialize` — exact semantica pe care o implementezi.

## 3. Ce lipsește

- `crates/ignis-wsp/` (nou): parser de patch (din JSON-ul contractului; dacă vrei și formatul text
  `residual.W1[0, 0] = 0.25`, compilează-l în `LiteralOp`, fără expresii), assembler, forward strict per
  familie, `ModelService` impl, comenzi CLI (`ignis wsp spec|check|assemble|forward`).
- Spec-uri complete: inventarul **tuturor** scalarilor care influențează predicția (nu doar cele 129 de
  ponderi reziduale FLEX), diametru lichid vs flacără, ACME normal vs inverse separate.
- Promptul autorului (`§25.2`) și pachetul de handoff Fable/Astra (spec + surse admise; fără outcomes).
- Audit și ocolire a căilor de optimizare din forecast (vezi §9).

## 4. Fișiere deținute

`crates/ignis-wsp/`, `crates/ignis-flex/`, `ignis-acme/`, `ignis-bass/`, `ignis-science/`, `ignis-train/`,
`ignis-accel/`, `crates/ignis-lab/src/lab/`, `crates/ignis-lab/src/infer.rs`, `configs/models/`,
`crates/ignis-core/src/contracts/wsp.rs`, `fixtures/contracts/models/`, semantic: `contracts.rs`
(modelul științific), `prefix.rs`, `analysis.rs`, `expert.rs`.

## 5. Contracte

Consumi: `DatasetBundle` (Raul), `VisualObservationBundle::require_metric()` (Raul, pentru BASS),
`EvaluationResult` cu `purpose = development_feedback` (Tudor), `ExecutionScope`.
Produci: `ModelSpec`, `HandwrittenWeightPatch`, `ModelRelease` cu `construction`, `AnalysisArtifact`,
implementarea `ModelService`.

## 6. Interfețe

- Mircea cheamă doar `ModelService::{releases, release, forecast}`; handlerul nu știe layout-ul matricilor.
- Tudor rulează releases prin același `ModelService` în procesul evaluatorului.
- `models_forecast` și `models_inspect` din `tool_catalog()` sunt ale tale ca logică.

## 7. Dependențe și cum lucrezi fără să aștepți

I1/I2 depind doar de C0: pornește pe fixture-ul toy, apoi pe datele de dezvoltare existente
(`data/splits/`, `ignis data` views) până la `DatasetBundle`-ul lui Raul (R2). Feedback W1 (I4) cere T2
— până atunci feedbackul e doar de format, nu de scor. BASS (I5) pe fixture până la R5.

## 8. Sugestii tehnice

- Assembler: zero-init → valori `fixed` din spec → (parent, pentru W1) → assignments; respinge întregul
  patch la prima eroare (`validate_against` deja o face; nu duplica verificările, cheamă-le).
- `tensor_entries(..., frozen = toate)` (`trainable = false`), `weights_digest`, manifest cu
  `checks` (`spec_valid`, `patch_valid`, `tensors_complete`, `strict_forward_audit`,
  `finite_forward_on_fixture`) — un check `failed` face release-ul imposibil.
- `forward_hash`: digest al codului forward (ex. sursa crate-ului `ignis-wsp` + versiunea de numerics),
  distinct de weights; păstrează-l stabil între build-uri identice.
- Lineage: W0 = `literal_authoring`, fără parent; W1 = `weight_edit` cu `parent_id` și patch peste parent.
  O arhitectură nouă = spec nou (alt `spec_hash`) = experiment separat, nu revizie.
- Forward-ul e CPU-bound și sincron (`ModelService`); folosește rayon intern dacă ajută, nu Tokio.
- Încărcarea release-ului o dată per id (cache în serviciu), nu per request.

## 9. Riscuri de corectitudine / performanță

- **Optimizeri ascunși în forecast:** FLEX `forecast` → `particles` → `inference::importance_locals` →
  `newton_mode` (Laplace, Hessian din AD) și `prefix_d2_estimate` (least squares)
  (`crates/ignis-flex/src/inference.rs`, `expert.rs`); BASS operator `posterior` → `model::local_inference`
  (MAP prin L-BFGS, `crates/ignis-bass/src/model.rs`), `init_locals` (least squares), proiectorul de
  contur (least squares mascate, `observation.rs`). Forward-ul WSP nu cheamă niciuna; dacă o stare
  locală e necesară, e observată, calculată explicit din măsurători sau dată de un encoder handwritten,
  cu statut declarat. Solverele de ecuații (Newton pentru inversarea dinamicii) sunt permise doar ca
  integrare numerică cu parametri fixați, declarate în spec.
- `ignis lab edit/apply` clonează părintele (inclusiv `construction`); pe un release literal salvarea e
  refuzată (weights_hash nu mai corespunde). Asta e intenționat: o revizie WSP e doar un patch literal.
- Nu modifica labels/splits/metrici pentru un scor mai bun; nu folosi final-test ca feedback.
- `uncertainty_kind = calibrated_prediction` e interzis pe release literal; calibrarea din date e un
  pas separat, raportat separat.

## 10. Criterii de finalizare (verificabile)

- `ignis wsp assemble` reconstruiește tensorii exclusiv din spec + patch(+parent); hash identic la rerun.
- Test: forward strict pe fixture produce `AnalysisArtifact` valid; un test parcurge ruta reală de
  forecast și eșuează dacă se ajunge la `newton_mode`/`local_inference`/`lbfgs`/least squares.
- Un candidat FLEX real, `experimental`, evaluat de procesul lui Tudor (T2/T4).
- Model card: spec hash, patch hash, forward hash, author model exact, expunere, limite, stare.

## 11. Ce NU faci

Nu scrii optimizer, generator de coeficienți, random init sau fitting în calea WSP; nu schimbi
evaluatorul, protocolul, labels sau split-urile; nu atingi `web/`, `ignis-server`, `ignis-evidence`,
`ignis-data`; nu faci agent loop; nu prezinți scoruri pe care evaluatorul nu le-a produs.

## 12. Predare

PR per task (I1…I5) cu: spec-uri, patch-uri literale, release ids, comenzile rulate, fixture
actualizat, `cargo test -p ignis-wsp -p ignis-core`. Mircea înregistrează `ModelService` în composition
root pe baza constructorului tău public; Tudor primește release ids + forward hash.

## Prezentare

Tu conduci povestea tehnică și prezentarea orală (scenariu, model cards, rezultate și limite reale).
Nu promite eficiență sau generalizare înaintea scorecardului.
