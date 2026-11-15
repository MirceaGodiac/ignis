# Configuration

| | |
|---|---|
| **Scope** | Every environment variable IGNIS reads, the env files of the deployment, and the configuration files under `configs/` |
| **Audience** | Developers running IGNIS locally; operators of the origin host |
| **Template** | [`/.env.example`](../.env.example): local defaults, safe to copy as-is |
| **Related** | [DATA_PIPELINE.md](DATA_PIPELINE.md) · [REPRODUCIBILITY.md](REPRODUCIBILITY.md) · [infra/README.md](../infra/README.md) · [infra/deploy/ignis.env.example](../infra/deploy/ignis.env.example) · [infra/deploy/release.env.example](../infra/deploy/release.env.example) |

## 1. How configuration is read

- **No `.env` loader.** No IGNIS binary loads `.env` by itself. A variable counts only once it is
  exported. Locally:

  ```bash
  cp .env.example .env
  set -a; . ./.env; set +a
  ignis serve --dev
  ```

  In deployment, Docker Compose reads `/etc/ignis/ignis.env` as an `env_file`, and systemd reads
  `/etc/ignis/release.env` (`infra/deploy/systemd/ignis.service`).
- **Flags and variables.** Most server settings are clap flags with an `env =` fallback. The flag
  wins over the variable, and the variable wins over the default. `ignis <command> --help` prints
  each one as `[env: NAME=]`.
- **Empty means set.** An empty value for a flag-backed variable is not the same as unset:
  `IGNIS_AUTH=` makes `ignis serve` fail with `a value is required for '--auth <AUTH>'`. To use the
  default, comment the line out. Credentials read directly from the environment (`CODEX_API_KEY`,
  `OPENAI_API_KEY`) are exceptions: an empty value counts as absent there.
- **Relative paths** resolve against the current directory. For `ignis serve`, they resolve against
  `IGNIS_ROOT`. Run commands from the repository root.
- **Secrets in containers.** `infra/server/entrypoint.sh` exports every file `/run/secrets/<NAME>` as
  `<NAME>` before it execs `ignis`. Only names matching `[A-Z0-9_]+` are exported. Today every
  credential is still a plain entry in `/etc/ignis/ignis.env` (root 0600).
- **Credential isolation.** The runtime child receives only its own credentials (`pass_env`). The
  Codex child gets `CODEX_API_KEY` and `OPENAI_API_KEY`; the Claude child gets
  `CLAUDE_CODE_OAUTH_TOKEN` and `ANTHROPIC_API_KEY`. Runner containers get no host environment.
  `ignis data review auto` and the `ignis evidence` model passes remove every `CLAUDE*` and
  `ANTHROPIC*` variable before they start `claude -p`, so those passes use the local Claude Code
  sign-in.

**Legend.** *Default*: the value used when the variable is unset; "unset" means the feature is off
or not configured. *Req.*: **yes** = required for that mode; otherwise optional.

## 2. Paths and roots

| Variable | Read by | Default | Purpose |
|---|---|---|---|
| `IGNIS_DATA` | `crates/ignis-data/src/paths.rs` | `data` | Data root: `manifests/`, `splits/`, `review/`, `corpus/`, `datasets/`, `raw/`, `work/`, `media/` |
| `IGNIS_ARTIFACTS` | `crates/ignis-core/src/registry.rs`, `crates/ignis-evidence/src/catalog.rs`, `crates/ignis-lab/src/util.rs`, `crates/ignis-vision/src/pipeline.rs`, `ignis serve --artifacts` | `artifacts` | Releases `models/<hex>/`, evidence catalog `evidence/`, `analysis/`, `sealed/`, `reveals/`, `vision/` |
| `IGNIS_RUNS` | `crates/ignis-core/src/resources.rs`, `crates/ignis-lab/src/util.rs`, `crates/ignis-evidence/src/wiki.rs`, `crates/ignis-server/src/lib.rs` | `runs` | Job ledger `jobs/`, `evaluation/` (seals, `reveal_log.jsonl`), `wiki/wiki.sqlite` |
| `IGNIS_HOLDOUT` | `crates/ignis-lab/src/util.rs`, `crates/ignis-data/src/dataset.rs` | `holdout` | Lockbox: sealed continuations and the evaluator bundle (`datasets/`); evaluation scopes only |
| `IGNIS_SPLITS` | `crates/ignis-lab/src/util.rs` | `data/work/splits` | Extra split lookup for `ignis lab` / `ignis eval` |
| `IGNIS_FROZEN_SPLITS` | `crates/ignis-lab/src/util.rs` | `data/splits` | Frozen splits found by id prefix |
| `IGNIS_CONFIGS` | `crates/ignis-core/src/resources.rs` | `configs` | Configuration root (section 11) |
| `IGNIS_CACHE` | `crates/ignis-vision/src/video.rs` | `data/cache` | Decoded-frame cache (`<IGNIS_CACHE>/frames/`) |
| `IGNIS_SAM2_CHECKPOINT` | `crates/ignis-vision/src/sam2.rs` | unset | SAM2.1 checkpoint when `configs/vision/sam2.toml` names none |

## 3. Server (`ignis serve`)

Defined in `crates/ignis-server/src/lib.rs` (`ServeArgs`) unless stated otherwise. The server image
sets `IGNIS_ROOT=/opt/ignis`, `IGNIS_STATE_DIR=/srv/ignis/state`, `IGNIS_CODEX_BIN=/usr/local/bin/codex`,
`DISABLE_AUTOUPDATER=1` and `RUST_LOG=info,tower_http=info` (`infra/server/Dockerfile`). The
deployment's `compose.yaml` adds `IGNIS_AUTH=cloudflare-access`, `IGNIS_ARTIFACT_BACKEND=r2`,
`IGNIS_RUNNER_DIR=/srv/ignis/runner` and `IGNIS_RUNS=/srv/ignis/state/runs`.

| Variable | Read by | Default | Purpose |
|---|---|---|---|
| `IGNIS_PORT` | `ServeArgs.port`; also `web/vite.config.ts` (proxy target) | `8787` | Listen port (host is `--host`, default `127.0.0.1`) |
| `IGNIS_ROOT` | `ServeArgs.root` | `.` | Repository root; the server's relative paths resolve against it |
| `IGNIS_RECORDS` | `ServeArgs.data` | `data/work/records` | Records of the legacy `/api` registry |
| `IGNIS_EVIDENCE_DB` | `ServeArgs.evidence_db` | `artifacts/evidence/evidence.sqlite` | Evidence catalog (SQLite + FTS5) |
| `IGNIS_STATE_DIR` | `ServeArgs.state_dir`; `ignis state backup`, `ignis metrics` | `runs/server` | Runtime state: `state.sqlite` and `artifacts/` |
| `IGNIS_AUTH` | `ServeArgs.auth` | `local` | `local` (one loopback user) or `cloudflare-access` (section 4) |
| `IGNIS_ARTIFACT_BACKEND` | `ServeArgs.artifact_backend` | `local` | `local` or `r2` (section 5) |
| `IGNIS_ENABLE_MODELS` | `ServeArgs.enable_models` | off | Serve the FLEX/ACME/BASS model families through `/api/v1` and the IGNIS tools |
| `IGNIS_WSP_SERVED` | `crates/ignis-cli/src/main.rs` (`served_models`) | `configs/models/served` (under `IGNIS_ROOT`) | Served hand-written releases (`served.json` + `models/<hex>/`); with `--enable-models` only |
| `IGNIS_ANSWER_GROUNDING` | `ServeArgs.answer_grounding` | `log` | `log` or `enforce`: what `answer_submit` does with a number that does not resolve to its reference |
| `IGNIS_MAX_UPLOAD_BYTES` | `ServeArgs.max_upload_bytes` | `2147483648` (2 GiB) | Largest single upload to `/api/v1/uploads` |
| `IGNIS_MCP_PORT` | `ServeArgs.mcp_port` | `0` (free port) | Port of the MCP endpoint, always bound to 127.0.0.1 |
| `IGNIS_PRICE_LIST` | `ServeArgs`, `crates/ignis-server/src/harness_run.rs`, `crates/ignis-server/src/observe.rs` | built-in `configs/pricing/llm_prices.toml` | Versioned LLM price list for cost metrics |
| `IGNIS_SOURCE_MANIFESTS` | `crates/ignis-server/src/v1/sources/mod.rs` | `data/manifests` | Source manifests (`<ACC>.jsonl`, `.listing.jsonl`, `ntrs/*.jsonl`) |
| `IGNIS_MEDIA_INDEX` | same | `data/media/keyframes` | Keyframe indexes of `.ts` sources (R11) |
| `IGNIS_MEDIA_CACHE` | same | `data/media` | ZIP central directories and converted frames. Must be writable; in deployment use `/srv/ignis/state/media` (the image is read-only) |
| `IGNIS_PUBLIC_URL` | same | unset | Public UI origin (`https://…`, no trailing slash) for the NTRS proxy and HLS playlist URLs. Unset: NTRS opens as a link out and `.ts` sources get no playlist |
| `IGNIS_PSI_GEODE_URL` | same | `https://psi.nasa.gov/geode-py/ws` | PSI geode web-service root |
| `IGNIS_ENABLE_WEB` | `crates/ignis-evidence/src/web/mod.rs` | off | Web discovery (R4) when `1`/`true`/`yes`/`on` |
| `PARALLEL_API_KEY` | `crates/ignis-evidence/src/web/provider.rs` | unset | Search provider key. Without it, web discovery is fetch-only |
| `IGNIS_WEB_SEARCH_URL` | same | `https://api.parallel.ai/v1/search` | Search provider endpoint |
| `RUST_LOG` | tracing; passed through to the runtime child (`preflight.rs`) | unset (image: `info,tower_http=info`) | Log filter |
| `DISABLE_AUTOUPDATER` | Claude Code (set in the image) | `1` in the image | Keeps the bundled `claude` from updating itself |

## 4. Cloudflare Access

Read by `crates/ignis-server/src/v1/access.rs` (`AccessArgs`). All are **required** with
`IGNIS_AUTH=cloudflare-access` and unused with `local`. The origin AUD and the user AUD must differ,
or the server refuses to start.

| Variable | Default | Purpose |
|---|---|---|
| `IGNIS_ACCESS_TEAM_DOMAIN` | unset | Team domain, e.g. `https://<team>.cloudflareaccess.com` (JWT issuer) |
| `IGNIS_ACCESS_ORIGIN_AUD` | unset | AUD tag of the Access application on the origin (tunnel) hostname |
| `IGNIS_ACCESS_USER_AUD` | unset | AUD tag of the Access application on the UI hostname |
| `IGNIS_ACCESS_SERVICE_TOKEN_ID` | unset | Client id of the Worker's service token (`common_name` of the origin JWT) |
| `IGNIS_ACCESS_CERTS_URL` | `<team>/cdn-cgi/access/certs` | Hidden override of the key endpoint, for tests |

## 5. Artifact store (R2) and state backups

| Variable | Read by | Default | Purpose |
|---|---|---|---|
| `IGNIS_R2_ENDPOINT` | `crates/ignis-server/src/ops.rs` (`R2Args`) | unset | `https://<account-id>.r2.cloudflarestorage.com`; **required** with `r2` |
| `IGNIS_R2_BUCKET` | same | unset | Bucket; **required** with `r2` |
| `IGNIS_R2_ACCESS_KEY_ID` | same | unset | R2 API token key id; **required** with `r2` |
| `IGNIS_R2_SECRET_ACCESS_KEY` | same (environment only, never a flag) | unset | Secret; **required** with `r2` |
| `IGNIS_BACKUP_PREFIX` | `ignis state backup/restore` (`ops.rs`); `infra/scripts/backup.sh`, `restore.sh` | `backups/` | Bucket prefix of state snapshots |
| `KEEP_LOCAL` | `infra/scripts/backup.sh` | `3` | Local snapshots kept in `/srv/ignis/backups` |

## 6. Agent runtime

Read by `crates/ignis-server/src/v1/agent.rs` (`CodexArgs`; `ignis serve` and `ignis harness run`)
unless stated otherwise. If the selected runtime has no binary, the runtime is reported unavailable
and the rest of the server still works.

| Variable | Default | Purpose |
|---|---|---|
| `IGNIS_AGENT_RUNTIME` | `codex` | `codex` (Codex App Server) or `claude` (Claude Code, headless). Restart after a change |
| `IGNIS_CODEX_BIN` | unset (image: `/usr/local/bin/codex`) | Native `codex` executable of the pinned release (`PINNED_CODEX_VERSION` = `0.169.0`) |
| `IGNIS_CODEX_SHA256` | unset | Expected sha256 of that executable. Unset: the hash is recorded but not enforced |
| `IGNIS_CODEX_MODEL` | unset (Codex default) | Model of the Codex runtime |
| `IGNIS_CODEX_LOGIN_HOME` | unset | A `CODEX_HOME` signed in with ChatGPT, used instead of an API key; shared by every workspace; must be writable |
| `IGNIS_CLAUDE_BIN` | unset for `serve` (image: `/usr/local/bin/claude` via `ignis.env`); `claude` for `ignis data review auto` (`crates/ignis-data/src/auto.rs`) and the evidence model passes (`crates/ignis-evidence/src/llm_pass.rs`) | Claude Code executable |
| `IGNIS_CLAUDE_MODEL` | unset (Claude Code default) | Model of the Claude runtime |
| `IGNIS_CLAUDE_LOGIN_HOME` | unset | Signed-in Claude config directory, used when no token or key is set; must be writable |
| `IGNIS_CLAUDE_DELEGATION` | off | Lets the Claude investigator hand one bounded subtask to a native subagent (depth 1) |
| `IGNIS_HARNESS_DIR` | unset (bundle compiled from `harness/`) | Harness directory (`investigator.md`, `skills/<name>/SKILL.md`) |
| `IGNIS_AUTO_BACKOFF_MS` | `30000` | Backoff after consecutive model failures in `ignis data review auto` (`auto.rs`) |

Credentials (secrets). One per runtime is enough:

| Variable | Read by | Purpose |
|---|---|---|
| `CODEX_API_KEY` | `crates/ignis-research/src/runtime/preflight.rs` | Codex, API billing. The first non-empty of `CODEX_API_KEY`, `OPENAI_API_KEY` is used when no login home is set |
| `OPENAI_API_KEY` | same | Fallback name for the Codex key |
| `CLAUDE_CODE_OAUTH_TOKEN` | `crates/ignis-research/src/runtime/claude.rs` (passed to the child) | Claude plan token (`claude setup-token`) |
| `ANTHROPIC_API_KEY` | same | Claude, API billing |
| `DEEPINFRA_API_KEY` | `crates/ignis-evidence/src/deepinfra.rs` | Dense retrieval, reranking and `ignis evidence embed`. Being removed by decision (LANES_V2 R12a); not set in deployment |

Set by the host for the child process, never by operators: `IGNIS_MCP_TOKEN` (the runtime's MCP
bearer token, `crates/ignis-research/src/runtime/grants.rs`), `CODEX_HOME`, `CLAUDE_CONFIG_DIR`.

## 7. Isolated analysis runner

Read by `crates/ignis-server/src/v1/sandbox.rs` (`RunnerArgs`). If no image is set, the runner is
reported unavailable.

| Variable | Default | Purpose |
|---|---|---|
| `IGNIS_RUNNER_IMAGE` | unset | Runner image id (`sha256:…`, printed by `tools/runner/build.sh`); the host refuses any other |
| `IGNIS_RUNNER_RUNTIME` | `runsc` | OCI runtime (`runsc` = gVisor) |
| `IGNIS_RUNNER_ALLOW_CONTAINER` | off | Allows a runtime other than gVisor on development machines; accepted with `IGNIS_AUTH=local` only |
| `IGNIS_RUNNER_ENGINE` | `docker` | Container engine CLI |
| `IGNIS_RUNNER_DIR` | `<state-dir>/runner` | Per-run mounts. Absolute, shareable with the engine; in deployment a separate fixed-size filesystem |

Limits without a variable (flags only): `--runner-wall-seconds` (600), `--runner-memory-mib` (2048),
`--runner-cpu-millis` (2000), `--runner-pids` (256).

## 8. Deployment (origin host)

These variables live in env files on the host, not in the repository. Templates:
`infra/deploy/release.env.example`, `infra/deploy/ignis.env.example`; layout in
[infra/README.md](../infra/README.md).

| Variable | File on the host | Read by | Default | Purpose |
|---|---|---|---|---|
| `IGNIS_IMAGE` | `/etc/ignis/release.env` | `infra/deploy/compose.yaml`, `infra/scripts/build-data.sh`, `autodeploy.sh` | none; **required** | Server image by digest (`ignis-server@sha256:…`); switched by `autodeploy.sh` |
| `CLOUDFLARED_IMAGE` | `release.env` | `compose.yaml` | none; **required** | `cloudflare/cloudflared@sha256:…` |
| `DOCKER_GID` | `release.env` | `compose.yaml` (`group_add`) | none; **required** | gid of `/var/run/docker.sock` (`stat -c %g`) |
| `IGNIS_ENV_FILE` | `release.env` | `compose.yaml` | `/etc/ignis/ignis.env` | Env file of the `ignis` service |
| `CLOUDFLARED_ENV_FILE` | `release.env` | `compose.yaml` | `/etc/ignis/cloudflared.env` | Env file of the `cloudflared` service |
| `IGNIS_SECRETS_DIR` | `release.env` | not read by `compose.yaml` or any script in this repository today (declared for the planned move to Compose secrets) | `/etc/ignis/secrets` | Directory of secret files |
| `IGNIS_BACKUP_PREFIX` | `release.env` | `backup.sh`, `restore.sh`, `ignis state` | `backups/` | See section 5 |
| `IGNIS_RELEASE_ENV` | environment of the caller | `infra/scripts/build-data.sh` | `/etc/ignis/release.env` | Where `build-data.sh` reads `IGNIS_IMAGE` |
| `TUNNEL_TOKEN` | `/etc/ignis/cloudflared.env` (root 0600) | `cloudflared` | none; **required** | Tunnel credential |
| `CLOUDFLARE_API_TOKEN` | `/etc/ignis/edge.env` (root 0600) | `autodeploy.sh` (Wrangler container) | unset: UI step off | "Edit Cloudflare Workers" token for this account |
| `CLOUDFLARE_ACCOUNT_ID` | `edge.env` | same | unset | Cloudflare account id |
| `UI_HOSTNAME` | `edge.env` | same | unset | Fills `<ui-hostname>` in `wrangler.jsonc` |
| `ORIGIN_HOSTNAME` | `edge.env` | same | unset | Fills `<origin-hostname>` in `wrangler.jsonc` |
| `REMOTE`, `BRANCH` | `/etc/ignis/autodeploy.env` | `autodeploy.sh` | the GitHub repository, `main` | What the VM deploys |
| `DRAIN_TIMEOUT` | `autodeploy.env` | same | `1800` s | Wait for running turns and jobs before postponing |
| `MIN_FREE_GB` | `autodeploy.env` | same | `8` | Free Docker disk needed before a build |
| `HEALTH_TIMEOUT`, `BUILD_CACHE_KEEP_GB`, `NODE_IMAGE` | `autodeploy.env` (the file is sourced after the defaults, so any default in the script's configuration block can be overridden) | same | `240`, `8`, `node:22-bookworm-slim` | Restart health wait, build cache kept, image that builds `web/` and runs Wrangler |

**Cloudflare Worker** (`infra/worker`):

| Name | Kind | Where | Purpose |
|---|---|---|---|
| `ORIGIN_URL` | var | `infra/worker/wrangler.jsonc` (`vars`) | `https://<origin-hostname>` |
| `ORIGIN_CLIENT_ID` | secret | `npx wrangler secret put ORIGIN_CLIENT_ID` | Service-token client id the Worker presents to the origin |
| `ORIGIN_CLIENT_SECRET` | secret | `npx wrangler secret put ORIGIN_CLIENT_SECRET` | Service-token client secret |
| `ASSETS` | binding | `wrangler.jsonc` (`assets`) | `web/dist` static assets |

## 9. Tests and tooling

Live tests are opt-in: they run only when their variables are set.

| Variable | Read by | Purpose |
|---|---|---|
| `IGNIS_LIVE_PSI` | `crates/ignis-server/tests/sources.rs` | `1` runs the live PSI source test |
| `IGNIS_CODEX_LIVE_BIN`, `IGNIS_CODEX_LIVE_TURN`, `IGNIS_CODEX_LIVE_LOGIN_HOME`, `IGNIS_CODEX_LIVE_MODEL` | `crates/ignis-research/tests/runtime_live.rs`, `crates/ignis-mcp/tests/live_codex.rs` | Real Codex turn |
| `IGNIS_LIVE_CLAUDE_BIN`, `IGNIS_LIVE_CLAUDE_DELEGATION` | `crates/ignis-research/tests/runtime_claude.rs` | Real Claude Code turn |
| `IGNIS_R2_LIVE_ENDPOINT`, `IGNIS_R2_LIVE_BUCKET`, `IGNIS_R2_LIVE_REGION` (default `auto`), `IGNIS_R2_LIVE_ACCESS_KEY_ID`, `IGNIS_R2_LIVE_SECRET_ACCESS_KEY` | `crates/ignis-research/tests/r2.rs` | Live bucket test (`cargo test -p ignis-research --test r2 r2_live -- --ignored`) |
| `IGNIS_RUNNER_IMAGE`, `IGNIS_RUNNER_RUNTIME` (default `runsc`), `IGNIS_RUNNER_DOCKER` | `crates/ignis-research/tests/sandbox_live.rs`, `crates/ignis-mcp/tests/analysis_live.rs` | Live runner isolation tests |
| `IGNIS_FAKE_DIR`, `IGNIS_FAKE_CODEX_FRAMING`, `IGNIS_FAKE_CODEX_SCHEMA`, `IGNIS_FAKE_CODEX_VERSION`, `IGNIS_FAKE_CLAUDE_TOKEN`, `IGNIS_TEST_HOST_SECRET` | test binaries and tests in `crates/ignis-data`, `crates/ignis-research` | Set by the tests themselves; do not set them |
| `FDS`, `GEOMETRY_TRIM` | `crates/ignis-bass/tests/convergence.rs`, `crates/ignis-flex/examples/geometry.rs` | Variants of one test and one example |
| `IGNIS`, `FAKE_CODEX`, `EPISODES` (6), `PORT_A` (18741), `PORT_B` (18742) | `infra/scripts/restore-drill-local.sh` | Binaries and parameters of the local restore drill |
| `CODEX_HOME` (`~/.codex`), `CARGO_TARGET_DIR` | `tools/codex/serve.sh` | Local Codex login and build directory |
| `RUST_VERSION` (`stable`), `CARGO_HOME`, `RUSTUP_HOME`, `REMOTE_DIR` (required remote checkout path), `SSH_OPTS` | `tools/runpod/setup.sh`, `sync.sh` | GPU pod setup and sync |
| `PORT` | `web/vite.config.ts` | Vite dev-server port (default `5173`) |
| `HF_HUB_OFFLINE`, `HF_HUB_DISABLE_XET` | Hugging Face hub (Docling sidecar) | Offline model use; a workaround for a stuck download (`sidecar/docling/README.md`) |
| `CLANG_PATH` | `mupdf-sys` build (Windows) | LLVM `clang.exe` for the `pdf` feature |
| `COMPUTERNAME` / `HOSTNAME`, `USERNAME` / `USER` | `crates/ignis-core/src/resources.rs`, `crates/ignis-lab/src/eval/vault.rs` | Host and user recorded in job and vault records |

`tools/codex/setup.sh` writes `runs/codex/codex.env` with `IGNIS_CODEX_BIN` and `IGNIS_CODEX_SHA256`.
`tools/codex/serve.sh` and `login.sh` parse that file; they do not source it.

## 10. The staging key set

The staging origin's env files hold these keys. Each one is documented above: `IGNIS_ACCESS_TEAM_DOMAIN`,
`IGNIS_ACCESS_ORIGIN_AUD`, `IGNIS_ACCESS_USER_AUD`, `IGNIS_ACCESS_SERVICE_TOKEN_ID` (§4);
`ORIGIN_CLIENT_ID`, `ORIGIN_CLIENT_SECRET` (§8, Worker); `IGNIS_AGENT_RUNTIME`, `IGNIS_CODEX_MODEL`,
`IGNIS_CLAUDE_BIN`, `CODEX_API_KEY`, `CLAUDE_CODE_OAUTH_TOKEN`, `ANTHROPIC_API_KEY` (§6);
`IGNIS_RUNNER_IMAGE`, `IGNIS_RUNNER_RUNTIME` (§7); `TUNNEL_TOKEN`, `IGNIS_IMAGE`, `CLOUDFLARED_IMAGE`,
`DOCKER_GID`, `IGNIS_ENV_FILE`, `CLOUDFLARED_ENV_FILE`, `IGNIS_SECRETS_DIR`, `IGNIS_BACKUP_PREFIX`,
`CLOUDFLARE_API_TOKEN`, `CLOUDFLARE_ACCOUNT_ID` (§8).

## 11. Configuration files (`configs/`)

The root moves with `IGNIS_CONFIGS`. The server image copies `configs/` to `/opt/ignis/configs`
(read-only).

| Path | Read by | Contents |
|---|---|---|
| `resources/{cpu-local,gpu-local,runpod-3090,runpod-4090}.toml` | `ignis_core::resources::load_profile` (`--profile`, default `cpu-local`) | Execution profiles for the resource guard: `device`, `usd_per_hour`, `max_hours`, `cpu_threads`, `batch_scale` |
| `training/defaults.toml` | `ignis fit` | Planning constants and eval/forecast/support defaults, merged under every model config (the model file wins) |
| `models/<family>/<model>.toml` | `ignis fit run` (`--set key=value` overrides) | Fitted models and baselines: `flex/{v3,v1,d2_population,d2_prefix}`, `acme/{normal,inverse}`, `bass/…` |
| `models/<family>/wsp*/`, `models/{aerosol,bre,droplets,jets,law,properties,solids,spherical}/` | `ignis wsp` | Literal WSP work: `spec.json` (`ModelSpec`), `binding.json`, and one directory per author candidate (its `patch.json` and the assembled `release.json`) |
| `models/served/served.json` + `models/served/models/<hex>/` | `ignis serve --enable-models` (`IGNIS_WSP_SERVED`) | The hand-written releases a deployment serves, verified at startup |
| `accel/{acme_normal,bass}.toml` | `ignis accel` | Optional inference accelerators |
| `evidence/corpus.toml` | `ignis evidence corpus …` | Corpus seeds (why each work is in or rejected) |
| `evidence/retrieval.toml` | `ignis evidence` search/embed | Provider, embedding, reranker and fusion settings |
| `evidence/firewall.toml` | `ignis evidence firewall` | Crates that may never depend on `ignis-evidence` |
| `evidence/run_patterns.toml` | evidence ingest and import | Attribution of evidence text to runs |
| `vision/segmentation.toml`, `vision/sam2.toml`, `vision/calibration.*.json`, `vision/bass_qualification.psi25.json` | `ignis vision` | Classical segmenter, SAM2 sidecar (checkpoint, device), calibration specs |
| `evaluation/landmarks.toml` | `ignis eval deposit-cohort`, `ignis infer forecast-cohort` | Cutoffs in s per family |
| `evaluation/protocols/` | `ignis eval freeze-protocol --dir` (default) | Frozen evaluation protocols (write-once; created on first use) |
| `pricing/llm_prices.toml` | built into the binary; `IGNIS_PRICE_LIST` overrides | Versioned LLM price list. Never edit a published version in place |

Other configuration outside `configs/`: `rust-toolchain.toml`, `infra/deploy/compose.yaml`,
`infra/deploy/systemd/*`, `infra/worker/wrangler.jsonc` (and `wrangler.staging.jsonc`), the sidecars'
`pyproject.toml`, `infra/runner/requirements.lock`, and `harness/` (the investigator harness compiled
into the binary).
