# Handoff — Mircea: Codex Integration, Backend & Infrastructure

Dă agentului tău: acest fișier, `AGENTS.md`, `docs/implementation/{CONTRACTS,INTEGRATION}.md` și
`docs/design/IGNIS_MASTER_V2.md` §5–§9 și §17–§21. Branch: unul scurt per task din `main` (`feat/agent-backend-<task>`).
**Nu ai nicio sarcină de prezentare** (deck, script, voce, capturi, montaj).

## 0. Faza 2 (citește întâi) — harness & platformă

Detalii, motivare, surse: [LANES_V2](LANES_V2.md) §4 (lane-ul tău), §9 (contracte), §10 (date NASA).

**Misiune:** agentul live produce răspunsuri validate și legate de surse, sub un harness versionat al
cărui efect e măsurat, cu un cost cunoscut per întrebare; originalele NASA ajung în browser fără copii.
Runtime-ul păstrează bucla (fără buclă proprie, compactor sau gateway).

**P0, în ordine:** (1) PR-ul `research.v2` (K1.1–K1.9; `ViewSpec` împreună cu Tudor), scris în paralel
cu Raul și Tudor, intră după `evidence.v2`/`dataset.v2`; (2) **M7** `answer_submit` (efect `compute`;
azi niciun turn live nu produce `ResearchAnswer`); (3) **M6** harness minimal: `harness/investigator.md`
+ 2 skills, secțiunea de tool-uri generată, injectat în ambele runtime-uri, digest pe episod;
(4) **M12** `SourceAccess`: URL S3 semnat PSI (~1 h), proxy Worker pentru NTRS, playlist HLS pe
byte-range pentru `.ts` din indexul de cadre-cheie al lui Raul. **P1:** M10 trace-uri, M8 ergonomia
tool-urilor, M11 rute v1. **P2:** M9 delegare depth-1 și date montate prin ref, M10b căutare offline
de harness doar pe dev, M4/M5.

**Cum alegi singur:** metoda din LANES_V2 §3 (întrebare + metrică, mini-evaluare pe datele noastre,
max. 3 candidați din surse primare, măsurat, decizia cu cifre în PR). Pentru câmpurile de instrucțiuni
citește schema protocolului exportată de preflight și documentația headless a Claude Code; verifică
prompt caching din câmpurile de usage reale.

**Eficiență:** prefix de instrucțiuni identic byte cu byte (cache); tool-uri `concise` implicit,
rezultate mari ca `ArtifactRef`; resolver de URL-uri semnate single-flight cu TTL sub expirare;
proxy NTRS în streaming, fără stocare; raportezi p50/p95 per tool, tokeni și dolari per întrebare,
procentul de cache.

## 1. Obiectiv

Backendul care face din serviciile științifice un produs: Codex App Server pin-uit integrat prin stdio
privat, API Rust/Axum versionat, evenimente SSE reluabile, joburi durabile, artifact store, scope și
autorizare, transport MCP peste serviciile colegilor, runner izolat, deployment Cloudflare + VM Google Compute Engine.
Ești și integratorul: composition root, `Cargo.toml`/`Cargo.lock`, CI.

## 2. Ce există și se reutilizează

- `crates/ignis-server`: Axum, `Strict<T>` care refuză câmpuri necunoscute, joburi `spawn_blocking` +
  rayon cu `catch_unwind`, cache moka cu cheie completă (`analysis::cache_key`), servire video cu range.
- `crates/ignis-cli/src/main.rs`: composition root actual (experți injectați ca `Vec<Box<dyn Expert>>`).
- `ignis_core::contracts::research` (`ResearchEpisode`, `ResearchEvent` + `check_event_log`, `JobRef`,
  `ToolResult::validate`, `ToolDescriptor`, `tool_catalog()`, `ResearchAnswer::validate`),
  `common` (`ExecutionScope`, `ArtifactRef`, `ErrorEnvelope`, `RetryPolicy`), `services.rs`
  (`ArtifactStore`, trait-urile colegilor).
- Fixture-uri: `fixtures/contracts/research/` (episod, 15 evenimente, tool accepted/failed/insufficient,
  answer), `common/scope_research.json`.
- `ignis_core::resources` (profiluri, guard `--execute`, job ledger).

## 3. Ce lipsește

Livrat în M2 (`feat/agent-backend`): `crates/ignis-research/` (store SQLite de episoade/evenimente/
joburi/uploads, `ArtifactStore` local), rutele `/api/v1` cu SSE, runner de joburi în proces, OpenAPI.

Livrat în M1 (branch `claude/agent-runtime-m1-b40a11`, peste M2): `ignis_research::runtime` —
trait-ul `AgentRuntime` și `CodexBridge` peste `codex app-server` 0.141.0 pin-uit (preflight: versiune,
sha256 al binarului, schema protocolului generată din binar, handshake pe stdio; manifestul intră în
`/capabilities`), JSON-RPC pe stdio privat cu asamblare incrementală, un proces per workspace, un thread
per episod, mapping thread/turn/item → `ResearchEvent`, aprobări rezolvate de host
(`POST /api/v1/episodes/{id}/approvals/{approval_id}`), profil public fără shell, pause/cancel prin
`turn/interrupt`, resume și recovery care reconciliază din thread-ul Codex fără duplicare (marks în store).
Teste offline cu un app-server fals (`ignis-fake-codex`); preflight + handshake + `thread/start` verificate
și pe binarul real 0.141.0 (`tests/runtime_live.rs`, opt-in, fără apel de model). Neverificat: un turn
cu modelul real (`live_turn`, cere `CODEX_API_KEY`) și tool-urile IGNIS prin MCP (M3). Un mesaj prin
HTTP cu binarul real ajunge la API și eșuează vizibil cu 401 fără cheie. Setup pe alte mașini:
`tools/codex/` (setup / serve / smoke, bash + PowerShell).

Livrat în M3 (partea MCP, branch `feat/m3-mcp`): `crates/ignis-mcp/` — `ToolHost`, o singură
implementare a fiecărui tool din `tool_catalog()` peste serviciile colegilor (decodare în contractul
declarat cu `deny_unknown_fields` + `validate()`, capabilitățile din descriptor verificate pe scope-ul
host-ului, `ToolResult` cu `refused`/`failed` separate, rezultate > 64 KiB stocate ca artefact cu preview
`partial`), folosit și de `/capabilities` și `POST /api/v1/analyses`. Server rmcp 3.10.0 (streamable
HTTP) pe un listener separat legat doar la `127.0.0.1` (`--mcp-port`, implicit port liber), câte o cale
per instanță (`/registry/mcp`, `/fixture/mcp`) cu grant-uri separate. Bridge-ul înregistrează serverul în
fiecare proces Codex (`-c mcp_servers.ignis.url/bearer_token_env_var`) și îi dă un token
(`IGNIS_MCP_TOKEN`) legat de scope-ul workspace-ului, revocat când procesul dispare; scope-ul nu vine
niciodată din argumente. Codex primește `enabled_tools` = tool-urile înregistrate și
`default_tools_approval_mode = "prompt"`, cu `approve` doar pentru efect `read`/`compute` (scope și
capabilități verificate de host la fiecare apel); bridge-ul refuză elicitările, deci un tool nou de scriere
eșuează închis până primește o cale de aprobare prin host. Tool nou `jobs_get` (`JobLookup` → `JobRef`) ca agentul să urmărească
`models_forecast`; un job trimis prin MCP e atașat episodului de `tool.finished` (în aceeași tranzacție),
deci `job.updated` ajunge în log și cancel-ul episodului îl oprește. Teste: dispatcher (`tests/tools.rs`),
MCP pe fir cu două sesiuni concurente, token lipsă/fals/revocat (`tests/http.rs`), atașarea jobului
(`ignis-research/tests/store.rs`); opt-in pe Codex real: `initialize` → `tools/list` cu tokenul, grant revocat la shutdown
(`ignis-mcp/tests/live_codex.rs`, fără apel de model) și un turn real cu model (login ChatGPT al
aplicației Codex) în care modelul cheamă `sources_search` prin MCP, rezultat `completed` cu scope-ul
host-ului (`live_turn_calls_an_ignis_tool`, opt-in). Pin-ul Codex a trecut de la 0.141.0 la 0.169.0:
0.141.0 primea `model_not_found` pe conturi ChatGPT; 0.169.0 trece preflight-ul (schema protocolului
conține toate metodele bridge-ului), handshake-ul și turn-ul fără schimbări de cod.

Livrat în M3 (runner-ul, branch `feat/m3-runner`): `ignis_research::sandbox` + tool-ul `analysis_execute`
(`AnalysisExecuteRequest` → job → `ExecutionRecord`, contracte noi în `research.rs`, fixture
`research/execution_record.json`). Programul (Python) rulează o dată într-un container OCI nou pornit
prin CLI-ul engine-ului, niciodată ca proces pe host: `--network none`, root read-only, `/tmp` tmpfs,
uid/gid 65534, `--cap-drop ALL`, `no-new-privileges`, bugete de memorie (fără swap), CPU, pids, timp
și dimensiune de fișier; exact trei mount-uri sub un director per rulare (`/job` ro, `/in` ro = copii
verificate prin digest ale artefactelor pe care scope-ul le poate citi, `/out`); niciun env de pe host.
Imaginea (`infra/runner/Dockerfile`, bază pin-uită prin digest, dependențe cu hash-uri în
`requirements.lock`) e acceptată doar prin id (`tools/runner/build.*`). Preflight-ul verifică engine-ul,
runtime-ul OCI și id-ul imaginii, apoi rulează o probă prin aceeași cale ca o execuție reală și refuză
runner-ul dacă dinăuntru se vede rețea, root scriibil, `/in` scriibil, capabilități, socket de engine
sau variabile de mediu în plus. Fără preflight trecut tool-ul nu e înregistrat, scope-ul nu primește
`ExecuteCode`, iar `/capabilities` (`analysis_runner`, `runner_environment`) spune de ce; nu există
fallback pe host. `runsc` (gVisor) e cerut implicit; `--runner-allow-container` (runc) doar cu
`--auth local`. stdout/stderr (prefix până la buget, `truncated` explicit) și fișierele regulate din
`/out` devin artefacte cu lineage (input + program); link-urile simbolice și fișierele speciale sunt
raportate ca `rejected_outputs`, nu urmate. Timeout/OOM sunt rezultate (`outcome`), nu erori de host;
cancel-ul episodului/jobului omoară containerul. Teste: offline (flag-urile de izolare, mount-uri,
link-uri, trunchiere, probă eșuată, tool neînregistrat fără runner, `/capabilities`) și opt-in pe
Docker real cu imaginea pin-uită (`ignis-research/tests/sandbox_live.rs`: program științific, program
ostil — rețea/DNS/scriere root și input/socket/setuid/mount/secret din env refuzate —, timeout, OOM,
fork bomb, trunchiere, cancel, alt workspace; `ignis-mcp/tests/analysis_live.rs`: tool → job →
container → `ExecutionRecord` citit prin `artifacts_read`). Verificat aici sub `runc` (Docker Desktop);
**neverificat sub gVisor** — VM-ul GCE cu `runsc` intră în M4. Disk-ul `/out` e limitat doar per fișier
(`fsize`) și la ingestie; o cotă de volum pentru `--runner-dir` ține de M4.

Livrat în M4 (branch `feat/m4-infra`, peste `feat/m3-runner`): CI (`.github/workflows/ci.yml`: fmt,
check, teste offline, drift de scheme, tipuri/build web, Worker, shellcheck, `compose config`) și
toolchain pin-uit (`rust-toolchain.toml`). `--auth cloudflare-access` nu mai crede header-ul nesemnat:
originea verifică două JWT RS256 semnate de Cloudflare (`v1::access`) — assertion-ul aplicației Access
de pe hostname-ul tunelului (`aud` origin, `common_name` = service token-ul Workerului) și assertion-ul
utilizatorului copiat de Worker în `X-Ignis-User-Assertion` (`aud` UI, `email` = principalul); chei din
JWKS-ul echipei, refetch la `kid` necunoscut, configurare incompletă = serverul nu pornește; `/livez`
singura rută fără identitate; SIGTERM oprește grațios. Backend R2 pentru `ArtifactStore`
(`ignis_research::{s3, r2}`: SigV4 propriu verificat pe exemplul AWS, PUT semnat pe hash-ul corpului,
multipart peste 64 MiB, manifest publicat ultimul cu `If-None-Match`), `--artifact-backend r2` cu probă
la pornire. Snapshot-uri de stare (`ignis_research::backup`, `ignis state backup|verify|restore`):
`VACUUM INTO` pe bazele vii, manifest cu hash-uri, restore doar pe volum gol cu `integrity_check`,
push/pull în R2, fără credențiale/WAL/staging. `infra/`: imaginea originii, Compose (ignis +
cloudflared, fără porturi publicate), systemd (serviciu + timer de backup), Worker (UI + proxy `/api`
în streaming cu service token), scripturi (build cu manifest de release, backup, restore, verificare
gVisor) și runbook (`infra/README.md`). Teste offline: tokenuri semnate pe un JWKS local (falsificate,
expirate, inversate, alt service token, cheie nepublicată), un S3 local care reverifică semnătura și
hash-ul fiecărui request, restore al unui store viu pe volum gol, upload prin `/api/v1` cu backend R2.
**Neverificate:** bucket R2 real (`r2_live`, opt-in), build-ul imaginii, contul Cloudflare/VM-ul GCE/domeniul
(neprovizionate), gVisor pe VM-ul GCE, RPO/RTO. Riscul socket-ului Docker în containerul `ignis` e
documentat în runbook.

## 4. Fișiere deținute

`crates/ignis-server/`, `crates/ignis-research/`, `crates/ignis-mcp/`, `crates/ignis-cli/src/main.rs`,
root `Cargo.toml`, `Cargo.lock`, `infra/`, `configs/resources/`, `tools/runpod/`, `tools/runner/`,
`crates/ignis-core/src/contracts/research.rs`, `crates/ignis-core/src/resources.rs`,
`fixtures/contracts/research/`, documentele comune (cu review Iancu).

## 5. Contracte

Produci: `ResearchEpisode`, `ResearchEvent`, `JobRef`, `ToolResult`, `ArtifactRef`, `ResearchAnswer`
(stocare și stream), implementarea `ArtifactStore`, `ExecutionScope` derivat de host.
Consumi: `KnowledgeService`/`DatasetService` (Raul), `ModelService` (Iancu), `EvaluationService` doar
pentru rute interne de dezvoltare (Tudor) — niciodată test în runtime-ul public.

## 6. Interfețe

- UI (Tudor): `/api/v1` (tabelul din `INTEGRATION.md`) + SSE (`CONTRACTS.md` §Streaming).
- MCP: un adaptor per intrare din `tool_catalog()`: decodează argumentul în contractul declarat,
  cheamă serviciul cu scope-ul sesiunii, întoarce `ToolResult`. Un serviciu neconfigurat apare în
  capability status, nu ca rezultat negativ, și nu e înregistrat ca tool.

## 7. Dependențe

M1 și M2 depind doar de C0: folosește fixture-urile ca implementări „fixture” ale serviciilor
(`origin = synthetic_fixture`, aceleași contracte, aceleași rute) până livrează colegii.

## 8. Sugestii tehnice

- **Codex:** `codex app-server` pin-uit (versiune/hash în `RuntimeBuildManifest`), handshake
  `initialize`/`initialized`, schema protocolului exportată din binarul pin-uit; stdout rezervat
  protocolului; mesajele asamblate incremental (un read OS ≠ un mesaj); cererile server→client de
  approval rezolvate explicit (`approval.requested/resolved`). Unknown → `runtime.opaque`.
- **Evenimente:** SQLite WAL, un writer dedicat, tranzacții scurte; `seq` alocat în aceeași tranzacție
  cu append-ul (outbox după commit); `check_event_log` în teste și la replay.
- **Joburi:** claim tranzacțional, lease + fencing token, heartbeat; output scris în staging, hash
  verificat, promovat atomic; `reconciling` pentru efecte ambigue.
- **ArtifactStore:** content-addressed, `put` din `Read` cu hash incremental, publicare atomică
  (staging → cheie imutabilă → manifest); `open` cu range; URL-urile R2 semnate se rezolvă la cerere.
- **Concurență:** serviciile sunt sincrone ⇒ `spawn_blocking` sau pool rayon dedicat, cu semafor per
  clasă (interactive > evaluare > ingestie). Nimic CPU pe reactor.
- **Middleware:** limitele 30 s / 1 MiB actuale rămân doar pe comenzi scurte; uploaduri mari direct în
  storage, rezultate mari prin artefacte; SSE fără timeout de request și fără buffering în proxy.
- **OpenAPI:** generat din handler-e (ex. `utoipa` sau `aide` peste schemars — alege și justifică),
  nu scris manual.

## 9. Riscuri

- **Scurgeri de test existente** (fix în M2): `/experiments/{id}/observations`, `/series`, `/overlay`
  fără verificare de partiție; `/seal` care ia partiția de la client; `/reveal` pe validare fără
  `reveal_log`; `/evidence/search` și `/brief` fără `strip_for_sealed` — detalii și linii în
  `INTEGRATION.md`. Toate trec prin `ExecutionScope::require_partition`.
- Nu duplica retry-ul upstream; erorile de auth/schemă nu se reîncearcă.
- `thread_id` de la browser nu dă acces fără verificarea episodului și utilizatorului.
- Shell-ul Codex: dezactivat în profilul public până la izolare demonstrată; calculul prin runner.
- Runner-ul nu vede Docker socket, home, chei, alt workspace, final-test.

## 10. Criterii de finalizare

Un turn Codex real → tool fixture → eveniment în UI, inclusiv un failure și un resume după restart;
reconectare fără duplicarea operațiilor; două sesiuni fără amestec de date; un approval refuzat;
evaluatorul și secretele inaccesibile din runner; deploy pornit din runbook; restore testat pe volum gol.

## 11. Ce NU faci

Agent loop propriu, compactor, gateway universal LLM, fork Codex preventiv, conversii improvizate
Chat Completions→Responses; matematică, ranking sau scoring în wrappere; Nginx; expunerea Codex/MCP/shell
pe internet; provisionare plătită fără autorizare; slide-uri.

## 12. Predare

PR per task (M1…M5), cu preflight-ul, testele de conformance (stdio framing, approval, resume,
replay SSE, izolare de scope) și runbook. Ca integrator: merge după review-ul ownerului; contractele
cer și consumatorul.
