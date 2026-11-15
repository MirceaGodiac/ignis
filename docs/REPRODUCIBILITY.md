# Reproducibility

| | |
|---|---|
| **Scope** | Rebuilding IGNIS from a clean checkout: toolchains, build, data, pinned identities, model releases, evaluation and benchmarks |
| **Audience** | Anyone who must reproduce a result, a release or a deployment, or check one |
| **Related** | [DATA_PIPELINE.md](DATA_PIPELINE.md) · [CONFIGURATION.md](CONFIGURATION.md) · [GUIDE.md](GUIDE.md) · [infra/README.md](../infra/README.md) |

## 1. Principles

- **Content identity.** Every durable object has a hash for its id: raw files (`sha256` in the
  manifest), cohorts and splits, dataset bundle artifacts, corpus snapshots, model releases and the
  server image. A rebuild is correct when it produces the same ids. A different id means different
  content.
- **Pinned inputs.** Toolchains, base images, the Codex binary, parser versions and the runner's
  Python stack are pinned by version, by digest, or both (section 4).
- **Recorded non-determinism.** Some inputs cannot be reproduced: NASA can republish, and LLM
  answers vary. Their output is recorded and committed or ledgered instead (section 8).
- **Dry run first.** Commands that download in bulk, run model calls or fit print a plan and stop
  unless you pass `--execute`.

## 2. Toolchain

| Tool | Version | Pinned where | Needed for |
|---|---|---|---|
| Rust | **1.99.1**, components `rustfmt`, profile `minimal` | `rust-toolchain.toml` (rustup selects it automatically). The server image builds with `rust:1.99.1-bookworm` (`infra/server/Dockerfile`, `ARG RUST_IMAGE`) | everything |
| Cargo dependencies | exact versions | `Cargo.lock`; the image builds with `--locked` | everything |
| C toolchain for the `pdf` feature | `clang`, `libclang-dev`, `nasm`, `cmake`, `pkg-config` (Debian packages in the image) | `infra/server/Dockerfile` | `evidence corpus ingest`, `evidence ingest`, `evidence verify` (MuPDF). On Windows, set `CLANG_PATH` to LLVM's `clang.exe` and build from a Visual Studio `vcvars64` shell ([GUIDE.md §1](GUIDE.md#1-setup)) |
| Node.js | **22** (`node:22-bookworm-slim` builds the UI and runs Wrangler in automatic deploys) | not pinned in `package.json` (no `engines` field, no `.nvmrc`); dependencies pinned by `web/package-lock.json` and `infra/worker/package-lock.json` | `web/`, `infra/worker/` |
| Python | sidecars: 3.11–3.13 for `sidecar/docling` and `sidecar/sam2`, ≥ 3.10 for `sidecar/pdfium`; parser benchmark: 3.12 (`uv venv --python 3.12`); runner image: `python:3.12-slim-bookworm` pinned by digest | each `pyproject.toml`; `infra/runner/Dockerfile` | parse views, SAM2 segmentation, `tools/`, `benchmarks/` |
| uv | any recent | not pinned | the sidecars (`uv sync --project sidecar/<name>`) and the runner lock (`uv pip compile --generate-hashes`) |
| Docker Engine + compose plugin | not pinned on the host; the image's CLI comes from `docker:29.7-cli` | `infra/server/Dockerfile` (`ARG DOCKER_CLI_IMAGE`) | the server image, the runner, `build-data.sh` |
| gVisor (`runsc`) | not pinned; verified with `infra/scripts/verify-gvisor.sh` | n/a | the isolated analysis runner in deployment |

`docs/GUIDE.md` §1 still says the port was developed with cargo 1.94. `rust-toolchain.toml` wins.

Run everything from the repository root. On Windows, use Git Bash.

## 3. Build

```bash
# Rust
cargo build --release -p ignis-cli                    # target/release/ignis(.exe)
cargo build --release -p ignis-cli --features pdf     # + MuPDF (PDF ingest, evidence verify)
export PATH="$PWD/target/release:$PATH"

# Contracts → schemas → TypeScript (after any contract change)
ignis schemas export
npm ci --prefix web
npm --prefix web run gen:types
npm --prefix web run check:types                       # fails if generated.ts is stale

# Web
npm --prefix web run typecheck
npm --prefix web run build                             # web/dist

# Checks (no CI: run them locally, see AGENTS.md)
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test --workspace                                 # or -p <crate>
ignis evidence firewall                                # protected crates do not depend on ignis-evidence
```

Server image (on the host that will run it, from a clean working tree):

```bash
infra/scripts/fetch-codex.sh /opt/ignis-cli            # pinned Codex, npm inside node:22-bookworm-slim
infra/scripts/fetch-claude.sh /opt/ignis-cli [<ver>]   # Claude Code; pass a version to pin it
infra/scripts/build-server.sh /opt/ignis-cli/codex /opt/ignis-cli/claude
```

`build-server.sh` refuses a dirty working tree and a Codex binary whose version differs from the
pin. It writes `runs/release/<git-sha>.json`:

```json
{
  "git_commit": "...", "rust_toolchain": "...",
  "codex_version": "...", "codex_sha256": "...", "codex_code_mode_host_sha256": "...",
  "claude_version": "...", "claude_sha256": "...",
  "base_images": {"rust": "...", "runtime": "...", "docker_cli": "..."},
  "image_id": "...", "built_at": "..."
}
```

The image carries the label `org.opencontainers.image.revision=<commit>`. Deployments name images by
digest only (`IGNIS_IMAGE=ignis-server@sha256:…`, `CLOUDFLARED_IMAGE=cloudflare/cloudflared@sha256:…`,
`infra/deploy/release.env.example`).

Runner image: `tools/runner/build.sh` prints `IGNIS_RUNNER_IMAGE=<image id>`. The image installs only
from `infra/runner/requirements.lock` with `--require-hashes --only-binary :all:`, and the host
refuses any other image id.

## 4. Pinned identities

| Identity | Form | Where it lives | Verified by |
|---|---|---|---|
| NASA file | `sha256:<hex>` of the bytes, plus `file_version` and `investigation_version` | `data/manifests/<INV>.jsonl` | `ignis sources manifest <INV> --verify` (exits non-zero on missing/mismatch) |
| NASA listing | rows with `file_version`, `size_bytes`, `updated` | `data/manifests/<INV>.listing.jsonl` | `git diff data/manifests/` after `sources inspect`; step 9 of `build-data.sh` |
| Listing errata | served size + sha256 | `data/manifests/listing_errata.jsonl` | enforced on fetch |
| Stored-corpus coverage | `manifest_hash` | `data/manifests/coverage.json` | `ignis sources coverage` |
| Cohort | `sha256:` of criteria + members | `data/work/cohorts/<hex>.json`, embedded in each split file | `ignis data cohort build` prints the id |
| Frozen split | `sha256:` content hash, checked on load; seed recorded (`20261107` for every committed split) | `data/splits/<family>-<hex16>.json` | load fails on mismatch; `split freeze` refuses to refreeze |
| Dataset bundle | `content_digest`; Parquet artifacts `art-<sha32>` | `data/work/datasets/`, committed copy `data/datasets/<dataset_id>.json` | `ignis data bundles`; the committed copy changes only when the digest changes |
| Corpus snapshot | `corpus-<hex16>`, first 16 hex of the sha256 of the canonical JSON of the catalog version, the member list (source, parse view, filter metadata, page and chunk counts, chunk digest) and the embedding set | `data/corpus/snapshots/<id>.json` (committed: `corpus-d0f2d2c5a4e1381c`) | `ignis evidence corpus snapshot` on the same catalog gives the same id |
| Parse view | `pv-<pub>@<parser_version>`; parser versions `0.3.0-mupdf` (`PARSER_VERSION` in `crates/ignis-evidence/src/lib.rs`), `pypdfium2==5.15.2`, `docling==2.150.1` | catalog | `ignis evidence verify` |
| Model release | `sha256:<hex>`, content hash of the release record without `id` and `created_at` | `artifacts/models/<hex>/` (legacy fits); `configs/models/served/models/<hex>/` + `served.json` (served literal WSP releases) | section 5 |
| Codex binary | `PINNED_CODEX_VERSION` (`0.169.0` in `crates/ignis-research/src/runtime/preflight.rs`) + sha256 | release manifest; `IGNIS_CODEX_SHA256` | `build-server.sh`, and the server preflight when `IGNIS_CODEX_SHA256` is set |
| Harness bundle | `sha256:` digest of `harness/` compiled into the binary | the binary; `ignis harness run --variant sha256:…` | checked against the pin of the first episode |
| Evaluation protocol | `frozen_hash` | `configs/evaluation/protocols/` (default `--dir`) | `ignis eval freeze-protocol` (write-once per id and version) |
| Server image | image id / digest | `runs/release/<commit>.json`, `IGNIS_IMAGE` | `docker image inspect` |

## 5. Model releases

Releases are immutable and content-addressed. Never edit `artifacts/models/`: a change is a new
release with its own id and a `parent_id`.

**What loading checks** (`load_release` in `crates/ignis-core/src/registry.rs`):

1. `release.json` parses, and `ModelRelease::check_construction` passes. A literal WSP release must
   be `trained_on = not_trained`, `state = experimental` and carry a `construction` manifest. A
   schema-2.0.0 file with neither field keeps the documented legacy `trained_on = "nasa"`.
2. The id in the record equals the directory id (`release id mismatch` otherwise).
3. The tensor set in `params.npz` / `posterior.npz` equals the manifest, and every tensor's sha256
   matches its recorded hash.
4. Optionally, the feature order matches what the caller expects.

When a release is written, its id is computed as `content_hash(record, excluding ["id", "created_at"])`,
and an existing directory is never overwritten.

**Served releases** (`ignis serve --enable-models`). `crates/ignis-wsp-serve` (`verify_release`,
`ServedModels::open`) verifies every entry of `configs/models/served/served.json` (or
`$IGNIS_WSP_SERVED`) at startup. One failure refuses the whole list, and the server then reports
models unavailable with the reason.

**Rebuild from the literal patch.** Each served release can be rebuilt from its committed patch.
`tools/wsp/verify_served.py` copies each release's parent into a temporary registry, runs
`ignis wsp assemble` on the committed `patch.json` and `binding.json`, and checks that the rebuilt
release has the served id and an equal record (all fields except `created_at`). It writes nothing
outside its temporary directory.

```bash
python tools/wsp/verify_served.py --ignis target/release/ignis --parents <registry holding models/<parent hex>/>
```

## 6. Data build

The commands, inputs and outputs are in [DATA_PIPELINE.md](DATA_PIPELINE.md). The short version:

**Local** (Git Bash on Windows; network access to `psi.nasa.gov` and `ntrs.nasa.gov`):

```bash
ignis sources inspect PSI-69 PSI-159 PSI-25 PSI-26
ignis sources samples
ignis data ingest
for inv in PSI-10 PSI-20 PSI-21 PSI-22 PSI-23 PSI-39 PSI-47 PSI-68 PSI-98 PSI-99 PSI-100 PSI-101 PSI-102 PSI-106 PSI-107; do
  ignis sources fetch "$inv" --pattern '_Experimental table_' --execute
done
ignis data conditions
ignis evidence corpus fetch --execute
ignis evidence corpus status
ignis evidence corpus ingest            # binary built with --features pdf
ignis evidence verify                   # binary built with --features pdf
ignis evidence corpus snapshot
for f in flex acme_normal acme_inverse; do
  ignis data bundle "$f"
  ignis data bundle "$f" --partitions train
done
ignis data bundles
ignis evidence corpus lockbox
git status data/                        # what changed against the committed references
```

This is the order of `infra/scripts/build-data.sh` without the `import-views` step. That step needs
parse views made beforehand by the sidecars.

**On the server:**

```bash
sudo infra/scripts/build-data.sh        # runs every step inside the release image IGNIS_IMAGE
```

**What a reproduction must match:**

- `git status data/` is clean except for `retrieved_at` fields in the manifests. Any other diff in
  `data/manifests/` is a NASA change (section 8).
- `ignis sources manifest <INV> --verify` passes for every fetched investigation.
- `data bundle` prints `copy_changed: false` for every committed bundle.
- `evidence corpus snapshot` prints the committed `snapshot_id`. A different id means the catalog
  differs: different seeds, parser version, PDFs or views.

**Do not re-freeze splits.** `ignis data split freeze` refuses an existing id. A reproduction uses
the committed `data/splits/`.

## 7. Evaluation and benchmarks

| What | Command | Code | Deterministic? |
|---|---|---|---|
| Unit, integration and parity tests | `cargo test --workspace` (offline; live tests are opt-in, see [CONFIGURATION.md §9](CONFIGURATION.md#9-tests-and-tooling)) | every crate's `tests/` | yes |
| Sealed evaluation | `ignis eval deposit-cohort` → `ignis infer forecast-cohort … --execute --seal` (or `ignis eval seal` by hand) → `ignis eval score --release <id> --partition validation` → `ignis eval compare` → `ignis eval report --md` / `ignis eval scorecard`; step by step in [GUIDE.md §6](GUIDE.md#6-sealed-evaluation) | `crates/ignis-lab/src/eval/` | yes for a fixed release, split and landmarks: the same run, cutoff, settings and release give the same content-addressed forecast artifact, and the bootstrap is seeded (`--seed`, default `0`; `--n-boot`, default `2000`) |
| Landmarks | `configs/evaluation/landmarks.toml` (cutoffs in s per family) | | |
| Protocol | `ignis eval freeze-protocol <protocol.json>` (write-once, prints `frozen_hash`) | | |
| Agent harness runs | `ignis harness run --variant <dir\|sha256:…> --tasks fixtures/harness/dev_tasks.json --out <dir> [--repeat N] --execute` | `crates/ignis-server/src/harness_run.rs` | no (LLM). Each episode is recorded as a `TraceBundle` |
| Episode metrics | `ignis metrics episode` / `ignis metrics workspace` (tool latency, tokens, cache hit, cost from a state directory; prices from `configs/pricing/llm_prices.toml` or `IGNIS_PRICE_LIST`) | `crates/ignis-server/src/observe.rs` | yes, over recorded state |
| Retrieval | `ignis evidence retrieval-eval --tasks <dev GoldTask set>` (nDCG@10, recall@10; writes `report.json`, `report.md`, `proposal.json`, never applied) | `crates/ignis-evidence/src/retrieval_eval.rs` | yes for a fixed catalog and pack |
| Parser choice (R7a) | `benchmarks/parsers/` (`build_gold.py`, `run_eval.py mupdf\|docling`, `merge_results.py`; recipe in its README) | Python | measured once on one machine (Apple M5); timings are not portable |
| Restore drill | `cargo build -p ignis-cli -p ignis-server --bins && bash infra/scripts/restore-drill-local.sh` | `infra/scripts/` | functionally yes; timings are measurements |
| Isolation | `infra/scripts/verify-gvisor.sh <runner image id>` | | host-dependent |

**Test partition.** Never pass `--final-test` and never score or compare on `test`. `ignis harness
run` and `ignis evidence retrieval-eval` refuse any path under `holdout/`, any task set that declares
`test`, and any scope that may read the test partition. Every reveal is appended to
`runs/evaluation/reveal_log.jsonl`. The final test runs once, on a frozen protocol, by the
evaluator. See [AGENTS.md](../AGENTS.md#non-negotiable-rules).

## 8. Determinism notes

- **Hashing.** Ids come from canonical JSON (`ignis_core::hashing::canonical_json`, `content_hash`).
  Timestamps (`retrieved_at`, `created_at`, `built_at`) are excluded from content ids.
- **Splits.** The group shuffle is a bit-exact port of NumPy's `default_rng(seed).permutation`
  (`crates/ignis-data/src/npyrng.rs`), so ids frozen by the earlier Python pipeline still hold.
- **Sample selection** is a deterministic function of the listing (`crates/ignis-data/src/sources/samples.rs`,
  `flex_stage2`).
- **Bundles.** An unchanged Parquet artifact keeps its first `created_at`, so the bundle
  `content_digest` is stable across rebuilds.
- **Parsing.** One parser version gives one parse view id. Changing a pin creates a new view, and the
  old one stays resolvable.
- **Port parity.** The README reports log-likelihoods and gradients within about 1e-12 of the earlier
  JAX code and byte-identical data ingest. The JAX parity fixtures are in each model crate's
  `tests/parity.rs`. The scripts that froze them (`tools/parity/`) need deleted Python code and cannot
  run again.

## 9. What is not reproducible, and how it is recorded

| Not reproducible | Why | Recorded as |
|---|---|---|
| NASA PSI listings and bytes | NASA republishes files and investigations (`file_version`, `investigation_version`) and fixes metadata | Committed manifests with sha256, versions and `retrieved_at`; `listing_errata.jsonl`; the manifest diff at the end of `build-data.sh` |
| NTRS records | Live public service | `data/raw/evidence/ntrs/NTRS-<id>.json` next to each PDF; status in `data/corpus/manifest.jsonl` |
| Adopted files | The original NASA fetch time is unknown | `retrieved_at` = local mtime (`sources adopt`) |
| Agent answers (Codex, Claude Code) | Non-deterministic models; provider-side model changes | Episode event logs in the state store; `TraceBundle` per harness episode (harness digest, runtime build, model, effort, event range, tool I/O as artifacts, usage, cost, partitions touched); the release manifest pins the Codex version and sha256 |
| Claude Code binary | `fetch-claude.sh` installs the latest version unless you pass one | `claude_version` and `claude_sha256` in the release manifest |
| Model-made curation (`data review auto`, `evidence labels`, `outline-summaries`, `llm-pass`) | LLM output | The output is the committed or stored input: `data/review/<family>/adjudications.jsonl` (each with `agent` or `reviewer`), `label_answers.jsonl`, `outline_summaries.jsonl`, expansion packs |
| Literal WSP weights | Written by the author model | The committed `patch.json` + `binding.json` per candidate; `tools/wsp/verify_served.py` reproduces the release from them |
| Timings and throughput | Hardware-dependent | Stated with the machine and date where reported (for example `benchmarks/parsers/README.md`, the restore drill in `infra/README.md`) |
| Container base images when built from tags | Tags move | `build-server.sh` resolves every base to a digest and records it; release builds pass digests as build args |
