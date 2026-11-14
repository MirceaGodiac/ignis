# Access: repo, machines, secrets, data

The one page to read before your first task. Staging for the hackathon: keep it simple. Secret
**values** are never written in the repo (git history becomes public when the repo does); this page
says where each one lives. For local work **use your own key** where you can (your own
ChatGPT/Claude login); otherwise ask **Mircea**, who sends it privately. He owns every account
(GitHub, Google Cloud, Cloudflare, provider keys); server secrets stay on the VM and only he edits them.

## 1. Repository

- GitHub, private: `MirceaGodiac/ignis`. Collaborators: `MirceaGodiac`, `iancuuandrei`,
  `raul-gliga-24`, `TApop26`.
- One short branch per task from fresh `main`, deleted on merge:

  ```bash
  git clone https://github.com/MirceaGodiac/ignis.git && cd ignis
  git fetch origin
  git worktree add -b feat/<lane>-<task> ../ignis-<name> origin/main
  ```

  Lane prefixes: `feat/handwritten-science-` (Iancu), `feat/agent-backend-` (Mircea),
  `feat/scientific-data-` (Raul), `feat/research-product-` (Tudor).
- First build: the Rust toolchain is pinned (`rust-toolchain.toml`); `cargo check --workspace`;
  `npm ci --prefix web`. Commands and gates: `AGENTS.md` §Commands.
- **No CI for now** (`AGENTS.md`): run the relevant commands locally, list them in the PR.
- **LF line endings.** On Windows: `git config core.autocrlf false`, editor set to LF; scripts that
  rewrite files write bytes or `newline=""` (Python), or every line shows as changed.

## 2. Working locally

| You want | You need |
|---|---|
| UI with no backend | nothing: `npm --prefix web run dev`, mock mode (C0 fixtures in the browser) |
| API on fixtures | nothing: `ignis serve --dev` (fixture routes at `/api/fixtures/v1`) |
| Tests | nothing for offline tests (fake Codex/Claude binaries, fixtures); live tests are opt-in |
| Search | nothing: BM25 with the agent's expansion terms (R12). No embeddings, no reranker, no DeepInfra |
| A live agent turn | Codex: `CODEX_API_KEY` or a ChatGPT login (`codex login`); Claude: `CLAUDE_CODE_OAUTH_TOKEN` (`claude setup-token`) or `ANTHROPIC_API_KEY` |
| Labels / claim checks (R8, K1.5) | `TYPESAFE_API_KEY` (Jev). Summaries and index-time expansion (R7b, R12b): the chosen generative model's key, named in the PR that adopts it |
| NASA files | see §5; nothing secret (PSI and NTRS are public) |

IGNIS reads keys from the process environment. Put yours in your shell profile or a `.env` at the
repo root (gitignored) loaded by your shell. Anything paid or networked still needs `--execute`.

## 3. The deployed staging system

- **App:** `https://ignis-ardealu.app`, behind Cloudflare Access: sign in with an e-mail on the
  Access allowlist (ask Mircea to add yours). Login methods: **Google**, a Cloudflare account, or
  a one-time PIN sent to the e-mail. Google works only for addresses that are also test users of
  the Google OAuth app (it is in Testing mode), so Mircea adds you in both places; the PIN works
  as soon as you are on the allowlist. The API is reachable only through the app (Worker →
  service token → tunnel); the VM has no public port.
- **VM:** Google Compute Engine `ignis-origin` (Debian 12, e2-standard-4). SSH:

  ```bash
  gcloud compute ssh ignis-origin --zone <zone> --tunnel-through-iap
  ```

  Needs a Google account added to the project with IAP tunnel + OS Login roles (Mircea). Zone and
  project: in Mircea's deployment log (`runs/deploy/log.md`, gitignored) and the runbook.

  **If it times out** (`WinError 10060` / "Remote side unexpectedly closed" while testing the
  tunnel) although the firewall allows IAP: check `curl -6 https://tunnel.cloudproxy.app` vs
  `curl -4 …`. On a network with a dead IPv6 route, gcloud's Python tries IPv6 first for the IAP
  websocket. Force IPv4 for gcloud only (no system setting changes) with a `sitecustomize.py` in
  a directory of your own:

  ```python
  import socket
  _orig = socket.getaddrinfo
  socket.getaddrinfo = lambda host, port, family=0, *a, **k: _orig(host, port, socket.AF_INET, *a, **k)
  ```

  then, in the shell that runs gcloud (Git Bash on Windows):
  `export PYTHONPATH="$(cygpath -w <that-dir>)" CLOUDSDK_PYTHON_SITEPACKAGES=1`. `gcloud compute
  ssh` and `gcloud compute scp` with `--tunnel-through-iap` then work. (Equivalent: a local
  IPv4-only HTTP CONNECT proxy and `CLOUDSDK_PROXY_TYPE=http CLOUDSDK_PROXY_ADDRESS=127.0.0.1
  CLOUDSDK_PROXY_PORT=<port>`.) The Cloud Console's SSH-in-browser also works but needs an
  Authorize click on every load.
- **Automatic deploys:** the VM deploys `main` by itself (`infra/README.md` §Automatic deploys);
  `journalctl -u ignis-autodeploy`, history in `/var/lib/ignis-deploy/deploys.jsonl`.
- **Layout on the VM:**

  | Path | What |
  |---|---|
  | `/srv/ignis/state` | episodes, events, jobs (SQLite) |
  | `/srv/ignis/data` | NASA corpus and builder bundles (read-only for `ignis serve`) |
  | `/srv/ignis/artifacts` | evidence catalog, model releases, local artifact root |
  | `/srv/ignis/backups` | snapshots before upload to R2 |
  | `/srv/ignis/runner` | runner scratch (separate fixed-size disk) |
  | `/srv/ignis/login/{codex,claude}` | plan sign-ins, when used |
  | `/etc/ignis/ignis.env` | server secrets and settings (§4) |
  | `/etc/ignis/cloudflared.env` | tunnel token |
  | `/etc/ignis/release.env` | pinned image ids |

- **Deploy, release, backup, restore:** `infra/README.md` (runbook). Data build on the VM:
  `sudo infra/scripts/build-data.sh`.

## 4. Where every secret lives

| Secret | Lives in | Used by |
|---|---|---|
| `CODEX_API_KEY` or ChatGPT login home | `/etc/ignis/ignis.env` / `/srv/ignis/login/codex` | Codex runtime |
| `CLAUDE_CODE_OAUTH_TOKEN` or `ANTHROPIC_API_KEY` | `/etc/ignis/ignis.env` | Claude runtime |
| `IGNIS_R2_ACCESS_KEY_ID`, `IGNIS_R2_SECRET_ACCESS_KEY` (+ `IGNIS_R2_BUCKET`, `IGNIS_R2_ENDPOINT`) | `/etc/ignis/ignis.env` | artifacts, backups |
| `IGNIS_ACCESS_*` (team domain, AUDs, service-token id; not secret but deployment-specific) | `/etc/ignis/ignis.env` | origin JWT checks |
| `TUNNEL_TOKEN` | `/etc/ignis/cloudflared.env` | cloudflared |
| `ORIGIN_CLIENT_ID`, `ORIGIN_CLIENT_SECRET` | Worker secrets (`wrangler secret put`) | the Worker |
| `TYPESAFE_API_KEY` | `.env` locally (Mircea's); on the VM only if label jobs run there | Jev labels (R8), claim support checks |
| Summary / expansion model key (R7b, R12b) | your environment when you run those jobs; on the VM only if they run there | Raul's batch jobs |
| Google sign-in (ADR 18, not merged yet) | OAuth client id (public) in the web build env; allowlist on the server | Google auth mode |
| Accounts: GitHub, Google Cloud project, Cloudflare (zone, Access, Tunnel, R2, Worker), domain | Mircea's accounts | — |

Full list of server settings: `infra/deploy/ignis.env.example`. Check that an entry is set without
printing it: `sudo grep -c '^NAME=.' /etc/ignis/ignis.env`.

## 5. Data

- **Stored NASA corpus** (atlas rule, `docs/wsp/atlas/`: 636 documents + 24,482 sensor text files,
  ~13.9 GB): the canonical copy lives **on the VM**, `/srv/ignis/data/raw/<PSI-id>/` on the 200 GB
  `ignis-data` disk, downloaded **there** straight from NASA by a scripted step in
  `infra/scripts/build-data.sh` (Raul supplies the selection from the atlas CSVs; Mircea wires and
  runs it; `--execute`). Re-runnable and incremental. No copy in Git or R2: NASA is the source and the
  manifests carry versions and hashes.
- **On your laptop:** only what you develop against, e.g. one or two investigations:
  `ignis sources fetch PSI-<id> --category '<category>' --pattern '<regex>'` (dry run), then
  `--execute`. Never sync a laptop copy to the VM.
- **NASA media** (35,446 files, 13.44 TB): never downloaded; the UI streams it from NASA through
  `SourceAccess` (`LANES_V2.md` §10).
- **Derived data** (parse views, indexes, labels, graph, Parquet): built where the corpus is, stored
  as artifacts (`/srv/ignis/artifacts`, R2).
- **Test partition and benchmark hold-out:** `holdout/` and the evaluator vault are never read by
  builders; the private gold hold-out stays in Tudor's vault, outside the repo (`AGENTS.md`).

## 6. What each person needs on day one

| Person | Needs |
|---|---|
| Iancu | GitHub; local toolchain. No deployment secrets. |
| Mircea | everything above (owner). Wires the corpus step into `build-data.sh`. |
| Raul | GitHub; `TYPESAFE_API_KEY` (from Mircea) when R8 starts; a local subset of NASA files. VM access only if he runs jobs there (ask Mircea). |
| Tudor | GitHub; Access allowlist for the deployed app; a Codex or Claude login for live benchmark runs; nothing for UI work in mock/fixture mode. |
