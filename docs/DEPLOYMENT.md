# IGNIS deployment and operations guide

| | |
|---|---|
| **Audience** | Operators and on-call engineers for the IGNIS staging deployment |
| **Scope** | Day-to-day operation: topology, releases, UI deploys, data rebuilds, backups, rollback, health, logs, incident fixes |
| **Source of truth** | [`infra/README.md`](../infra/README.md) (runbook, first deployment, identity, credentials, status). This guide is the operational summary; where the two differ, the runbook and the scripts in [`infra/scripts/`](../infra/scripts/) win. |
| **Not covered** | First-time provisioning of the VM, Cloudflare zone, tunnel and Access apps: see the runbook §First deployment and [`infra/DEPLOY_HANDOFF.md`](../infra/DEPLOY_HANDOFF.md). |

No secret appears in this document. Every credential lives in a root-only file on the VM or in a
Worker secret (see [Credentials and configuration files](#3-credentials-and-configuration-files)).

---

## 1. Topology

```text
Browser
  └─► Cloudflare Access (UI app, team identities)
        └─► Worker "ignis-edge"  (infra/worker, Wrangler)
              ├─ /*        static UI: web/dist (Workers Static Assets, SPA fallback)
              ├─ /ntrs/*   read-only stream of NTRS downloads (allow-listed path shape only)
              └─ /api/*  ─► Cloudflare Access (origin app, service token only)
                              └─ Cloudflare Tunnel (cloudflared, outbound from the VM)
                                    └─► GCE VM "ignis-origin" (europe-west3-a)
                                          systemd ignis.service
                                            └─ docker compose (infra/deploy/compose.yaml)
                                                 ignis        ignis serve on :8787 (API, SSE, jobs, MCP, runner client)
                                                 cloudflared  tunnel run --url http://ignis:8787
                                          runner containers (gVisor runsc, no network), one per analysis
                                          /srv/ignis/...   persistent disk ignis-data
                                          /srv/ignis/runner persistent disk ignis-runner (fixed size)
Cloudflare R2 bucket ignis-artifacts: artifacts (content-addressed) and state snapshots (backups/)
```

| Component | Identity / name | Defined in |
|---|---|---|
| UI hostname | `ignis-ardealu.app` (apex) | Access app "UI"; Worker route |
| Origin hostname | `origin.ignis-ardealu.app` | Access app "origin" (service token only); tunnel route |
| Edge Worker | `ignis-edge` | [`infra/worker/wrangler.jsonc`](../infra/worker/wrangler.jsonc), [`infra/worker/src/index.ts`](../infra/worker/src/index.ts) |
| Origin VM | `ignis-origin`, zone `europe-west3-a`, Debian 12, `e2-standard-4` | runbook §First deployment |
| Origin service | systemd unit `ignis` | [`infra/deploy/systemd/ignis.service`](../infra/deploy/systemd/ignis.service) |
| Containers | compose project `ignis`: services `ignis`, `cloudflared` | [`infra/deploy/compose.yaml`](../infra/deploy/compose.yaml) |
| Running release | `IGNIS_IMAGE` in `/etc/ignis/release.env` | [`infra/deploy/release.env.example`](../infra/deploy/release.env.example) |
| Serving checkout | `/opt/ignis-deploy` (compose runs from here) | `ignis.service` `WorkingDirectory` |
| Deploy state | `/var/lib/ignis-deploy` (build worktree, manifests, `deploys.jsonl`, lock) | [`infra/scripts/autodeploy.sh`](../infra/scripts/autodeploy.sh) |

### 1.1 Trust chain

- The Worker refuses `/api/*` without the user's `Cf-Access-Jwt-Assertion` (401 `scope_denied`),
  strips client-supplied identity headers, copies the user assertion into `X-Ignis-User-Assertion`
  and adds the service-token headers `cf-access-client-id` / `cf-access-client-secret`.
- The origin verifies both signed tokens in Rust (`crates/ignis-server/src/v1/access.rs`). The two
  Access apps must have different AUD tags; the server refuses to start otherwise.
- The VM has no published ports and no inbound firewall rule except SSH from Identity-Aware Proxy.
  `/livez` is the only unauthenticated route, and it is only reachable from inside the VM.

### 1.2 Data layout on the VM

| Path | Content | Mounted into `ignis` as |
|---|---|---|
| `/srv/ignis/state` | SQLite stores, event logs, reveal log (`IGNIS_RUNS=/srv/ignis/state/runs`) | same path, read-write |
| `/srv/ignis/backups` | local snapshots (last 3 kept) and restore downloads | same path |
| `/srv/ignis/data` | records, conditions table, builder bundles, media indexes | `/opt/ignis/data/work`, **read-only** |
| `/srv/ignis/artifacts` | evidence catalog, legacy artifacts root | `/opt/ignis/artifacts` |
| `/srv/ignis/raw`, `/srv/ignis/build-data` | NASA downloads and the working copy used by `build-data.sh` | not mounted |
| `/srv/ignis/runner` | runner job directories (separate fixed-size disk, `nodev,nosuid,noexec`) | same path |
| `/srv/ignis/login/{codex,claude}` | plan sign-ins, when used | same paths, writable |
| `/srv/ignis/cloudflared` | tunnel credentials created on the VM | not mounted (used by one-off `cloudflared` commands) |

`ignis.service` has `ConditionPathIsDirectory=/srv/ignis/state`: it does not start if the data disk
is not mounted.

---

## 2. Access to the VM

```bash
gcloud compute ssh ignis-origin --zone europe-west3-a --tunnel-through-iap
```

Set the project first (`gcloud config set project <project>`). All commands below run on the VM
unless marked *workstation*. Most need `sudo`.

A convenience used throughout this guide (it is the exact compose invocation of `ignis.service`):

```bash
cd /opt/ignis-deploy
C="sudo docker compose -f infra/deploy/compose.yaml --env-file /etc/ignis/release.env"
$C ps
```

---

## 3. Credentials and configuration files

| File | Mode | Holds | Secrets? |
|---|---|---|---|
| `/etc/ignis/ignis.env` | root 0600 | server configuration, Access AUDs, R2 keys, model credentials | **yes** |
| `/etc/ignis/release.env` | root 0644 | `IGNIS_IMAGE`, `CLOUDFLARED_IMAGE` (both by digest/id), `DOCKER_GID`, `IGNIS_BACKUP_PREFIX` | no |
| `/etc/ignis/cloudflared.env` | root 0600 | `TUNNEL_TOKEN` | **yes** |
| `/etc/ignis/edge.env` | root 0600 | `CLOUDFLARE_API_TOKEN`, `CLOUDFLARE_ACCOUNT_ID`, `UI_HOSTNAME`, `ORIGIN_HOSTNAME` (enables UI autodeploy) | **yes** |
| `/etc/ignis/autodeploy.env` | root 0644 | `REMOTE`, `BRANCH`, `DRAIN_TIMEOUT`, `MIN_FREE_GB` | no |
| `/etc/ignis/deploy_key` | root 0600 | read-only GitHub deploy key | **yes** |
| `/etc/ignis/github_known_hosts` | root 0644 | pinned github.com host key | no |
| `/etc/ignis/autodeploy.hold` | any | presence pauses timer deploys; content is the reason | no |
| Worker secrets | Cloudflare | `ORIGIN_CLIENT_ID`, `ORIGIN_CLIENT_SECRET` (`npx wrangler secret put`) | **yes** |

Rules:

- Never `cat` a file that holds a secret. To check that an entry is filled:
  `sudo grep -c '^NAME=.' /etc/ignis/ignis.env`.
- After any change to `ignis.env`, restart: `sudo systemctl restart ignis`.
- Templates: [`infra/deploy/ignis.env.example`](../infra/deploy/ignis.env.example),
  [`infra/deploy/release.env.example`](../infra/deploy/release.env.example).

---

## 4. Releases: automatic deploy

Production follows `main`. The VM pulls its own releases; nobody pushes to the VM.

| Piece | What |
|---|---|
| Timer | `ignis-autodeploy.timer`: `OnBootSec=10min`, `OnUnitInactiveSec=1min`, `RandomizedDelaySec=10` |
| Service | `ignis-autodeploy.service` (oneshot, `TimeoutStartSec=3h`, `Nice=10`) runs `/usr/local/sbin/ignis-autodeploy` |
| Script | [`infra/scripts/autodeploy.sh`](../infra/scripts/autodeploy.sh), installed by [`infra/scripts/autodeploy-setup.sh`](../infra/scripts/autodeploy-setup.sh) |
| Lock | `flock` on `/var/lib/ignis-deploy/lock`: runs never overlap |
| Record | one JSON line per attempt in `/var/lib/ignis-deploy/deploys.jsonl` |

### 4.1 Flow

Every step fails closed. The running commit is read from the running image's
`org.opencontainers.image.revision` label.

1. **Fetch** `main` with the read-only deploy key. If the tip equals the running commit (or the last
   UI-only commit served), only a pending UI catch-up runs.
2. **Web-only skip.** If every path changed since the running commit matches
   `^(web/|infra/worker/|docs/|[^/]+\.md$)`, no server build, snapshot or restart happens: the
   commit is recorded as `server_unchanged` and only the UI step (9) runs.
3. **Previously failed commit** (`/var/lib/ignis-deploy/failed`) is not retried until `main` moves,
   unless `--force` or `--commit` is used.
4. **Build** in a separate worktree `/var/lib/ignis-deploy/build` (the serving checkout does not
   move). `fetch-codex.sh` runs only when `PINNED_CODEX_VERSION` changed; `build-server.sh` builds
   the image and writes the release manifest, copied to `/var/lib/ignis-deploy/releases/<sha>.json`.
   Before building, if the Docker disk has less than `MIN_FREE_GB` (default 8) free, dangling images
   and week-old build cache are pruned, then the build cache is shrunk, then dropped entirely.
5. **Local snapshot** of the live store (`ignis state backup` inside the running container) into
   `/srv/ignis/backups/state-<UTC time>`. This is the rollback point; the last 3 are kept.
6. **Smoke test** of the *new* image against a restore of that snapshot, with `--network none`,
   local auth on loopback and the local artifact backend: the store must open (and migrate),
   `/livez` and `/api/v1/capabilities` must answer. The tool list is logged.
7. **Drain.** Waits until no `runtime_turns` row is `starting`/`running` and no job is
   `running`/`cancel_requested`/`reconciling`, up to `DRAIN_TIMEOUT` (default 1800 s). If still busy,
   the deploy is recorded `postponed` and retried on the next timer run.
8. **Switch.** Saves the previous `release.env` and checkout, sets `IGNIS_IMAGE`, checks out the
   commit in `/opt/ignis-deploy`, `systemctl restart ignis`. **Healthy** (within 240 s) means: the
   `ignis` container runs the new image, its healthcheck is `healthy`, `/livez` answers inside it, it
   is not restart-looping, `cloudflared` runs, and the journal since the restart has no
   `KnowledgeService not configured` / `DatasetService not configured`.
   - On success: the deploy script and systemd units are reinstalled from the deployed commit when
     they changed; `ignis-server` images other than the running and previous ones are removed (the
     previous one is tagged `ignis-server:rollback`); `ignis-backup.service` is started in the
     background to push an off-site snapshot to R2.
   - On failure: the previous `release.env` and checkout are put back and `ignis` restarted. If the
     old image cannot open the store (the new one migrated it), the state directory is moved to
     `/srv/ignis/state.failed-<time>` (never deleted) and the step-5 snapshot is restored.
9. **UI deploy** at the same commit (only when `/etc/ignis/edge.env` exists): in a
   `node:22-bookworm-slim` container, `npm ci` + `npm run build` in `web/`, `npm ci` in
   `infra/worker/`, then `npx wrangler deploy -c wrangler.deploy.jsonc`. `wrangler.deploy.jsonc` is
   generated from `wrangler.jsonc` with the hostnames from `edge.env`. Worker secrets are untouched.
   A failed UI deploy leaves the server live and the previous UI in place, and is recorded
   `ui_failed`.

### 4.2 Outcomes in `deploys.jsonl`

| `outcome` | Meaning | Action |
|---|---|---|
| `deployed` | server switched and healthy | none |
| `server_unchanged` | web/docs-only commit; server untouched | check the following `ui_deployed` |
| `ui_deployed` | Worker deployed at that commit | none |
| `ui_failed` | web build or Wrangler failed | see [§10](#10-common-failures-and-fixes) |
| `postponed` | turns or jobs still running after `DRAIN_TIMEOUT` | none; retried next run |
| `failed` | build, snapshot, smoke test or health failed; rolled back if it got that far | read the journal, fix forward on `main` |
| `drill_rolled_back` | `--drill-rollback` completed | none |
| `rollback_failed` | **origin down** | [§7.3](#when-automatic-rollback-fails) |

```bash
tail -n 5 /var/lib/ignis-deploy/deploys.jsonl
journalctl -u ignis-autodeploy -n 200 --no-pager
```

### 4.3 Manual controls

All run as root, take the same lock and run the same checks.

```bash
sudo ignis-autodeploy --force            # redeploy the tip of main now
sudo ignis-autodeploy --commit <sha>     # deploy a specific commit (must be on main)
sudo ignis-autodeploy --rollback         # previous image + checkout back (no store restore)
sudo ignis-autodeploy --ui               # build and deploy only the UI at the running commit
sudo ignis-autodeploy --drill-rollback   # deploy, then roll back as if checks failed
sudo ignis-autodeploy --no-drain         # do not wait for running turns/jobs (they are cut off)

echo "data job until 23:00 (<name>)" | sudo tee /etc/ignis/autodeploy.hold   # pause timer deploys
sudo rm /etc/ignis/autodeploy.hold                                           # resume
```

The hold file pauses only timer runs; `--force` and `--commit` ignore it. Long host-side jobs such
as `build-data.sh` are not seen by the drain step: put a hold file while one runs.

### 4.4 One-time setup (reference)

```bash
sudo infra/scripts/autodeploy-setup.sh            # deploy key, known_hosts, autodeploy.env, units
# a repository admin adds the printed PUBLIC key as a read-only GitHub deploy key
sudo infra/scripts/autodeploy-setup.sh            # must print "GitHub access: ok"
sudo infra/scripts/autodeploy-setup.sh --enable   # start the timer
```

To revoke: delete the deploy key on GitHub and `/etc/ignis/deploy_key*` on the VM.

---

## 5. Manual UI deploy (Wrangler)

Normally unnecessary: the UI ships with every merge to `main` (§4.1 step 9). Use the manual path
when `edge.env` is absent, when `--ui` on the VM cannot be used, or to redeploy from a workstation.

> **A manual deploy from a branch is overwritten by the next release.** Ship UI work by merging it.

*Workstation*, from the repository root, at the commit you intend to serve:

```bash
npm ci --prefix web
npm --prefix web run build                     # typecheck + vite build -> web/dist

cd infra/worker
npm ci
# Fill the placeholders into a local copy. wrangler.*.jsonc is gitignored: never commit it.
sed -e 's|<ui-hostname>|ignis-ardealu.app|g' \
    -e 's|<origin-hostname>|origin.ignis-ardealu.app|g' \
    wrangler.jsonc > wrangler.staging.jsonc

export CLOUDFLARE_API_TOKEN=<token: "Edit Cloudflare Workers" template, this account only>
export CLOUDFLARE_ACCOUNT_ID=<account id>
npx wrangler deploy -c wrangler.staging.jsonc
```

Notes:

- Run Wrangler from `infra/worker/` so the locally installed version is used; paths in the config
  (`main: src/index.ts`, `assets.directory: ../../web/dist`) resolve relative to the config file.
- Always regenerate `wrangler.staging.jsonc` from `wrangler.jsonc`; do not reuse an old copy. The
  committed config routes both `/api/*` and `/ntrs/*` to the Worker first (`run_worker_first`); a
  stale copy that lists only `/api/*` serves the SPA fallback for `/ntrs/*` and breaks NTRS originals.
- Worker secrets are set once and survive deploys:
  `npx wrangler secret put ORIGIN_CLIENT_ID` and `npx wrangler secret put ORIGIN_CLIENT_SECRET`
  (prompted; never pass them on the command line).
- Before deploying Worker code changes, run `npm test` and `npm run typecheck` in `infra/worker/`.

---

## 6. Rebuilding data

The server reads NASA-derived data from `/srv/ignis/data` (read-only) and `/srv/ignis/artifacts`.
[`infra/scripts/build-data.sh`](../infra/scripts/build-data.sh) rebuilds it with the release image
itself (reads `IGNIS_IMAGE` from `/etc/ignis/release.env`).

```bash
echo "build-data (<name>)" | sudo tee /etc/ignis/autodeploy.hold
cd /opt/ignis-deploy
sudo infra/scripts/build-data.sh
sudo systemctl restart ignis
sudo rm /etc/ignis/autodeploy.hold
```

What it does, in order (each step must end without an error):

1. Copies the committed manifests, splits, corpus, datasets and review inputs out of the image into
   `/srv/ignis/build-data`.
2. `sources inspect` and `sources samples` (bounded NASA sample set), then `data ingest` (records).
3. Fetches the experimental tables of the investigations the conditions table needs, then
   `data conditions` (Conditions page, atmosphere map, `conditions_query`). Records and conditions
   come before the corpus so a corpus problem cannot leave the Conditions page stale.
4. Evidence corpus: `fetch`, `status`, `ingest`, sidecar parse views when present, `verify`,
   `snapshot`.
5. FLEX/ACME builder bundles (`flex`, `acme_normal`, `acme_inverse`, full and `--partitions train`),
   `data bundles`, `evidence corpus lockbox`. Builder bundles never contain the test partition.
6. Prints listing manifests NASA changed since the release. **Review them; do not commit blindly.**

Downloads total about 0.3 GB and are skipped when the hash matches; bundles are rewritten only when
their content changes, so re-running is safe. Re-run after every release that changes ingest or
bundle code. Afterwards, check `/api/v1/capabilities` from a signed-in browser.

---

## 7. Backups, restore and rollback

### 7.1 Backups

| Kind | When | Where | Retention |
|---|---|---|---|
| Hourly snapshot | `ignis-backup.timer` (`OnCalendar=hourly`, up to 300 s jitter, `Persistent=true`) | local `/srv/ignis/backups/state-<time>` and R2 `backups/<name>/` | local: last 3 (`KEEP_LOCAL`); R2: bucket lifecycle rule on `backups/` |
| Post-deploy snapshot | after every good server deploy (`ignis-backup.service`, background) | same | same |
| Pre-deploy snapshot | every server deploy, before the smoke test | local only | last 3 |

[`infra/scripts/backup.sh`](../infra/scripts/backup.sh) runs `ignis state backup --push` inside the
running container: a consistent snapshot (`VACUUM INTO` per SQLite database while the server writes,
copies of the append-only logs), every file hashed into `manifest.json`, files uploaded before the
manifest. Excluded: WAL/SHM files, upload and artifact staging, credentials. Artifacts are not in
snapshots: they live in R2, immutable and content-addressed.

```bash
sudo systemctl start ignis-backup          # take one now; the journal ends with "snapshot <name> pushed"
journalctl -u ignis-backup -n 50 --no-pager
ls /srv/ignis/backups
```

Verify a local snapshot:

```bash
$C exec -T ignis ignis state verify /srv/ignis/backups/<name>
```

RPO is the time since the last pushed snapshot (at most the hourly interval plus the push). Measured
RTO exists only for a local drill (4.5 to 7.0 s for a 43 MB snapshot, runbook §Local restore
drill); a restore on the VM from R2 has not been measured.

### 7.2 Restore from R2

[`infra/scripts/restore.sh`](../infra/scripts/restore.sh) refuses to run while `ignis.service` is
active or when `/srv/ignis/state` is not empty. It verifies every file's sha256 and runs
`PRAGMA integrity_check` on each database.

```bash
echo "restore (<name>)" | sudo tee /etc/ignis/autodeploy.hold
sudo systemctl stop ignis
sudo mv /srv/ignis/state /srv/ignis/state.before-restore-$(date -u +%Y%m%dT%H%M%SZ)   # keep, do not delete
sudo mkdir /srv/ignis/state && sudo chown 10001:10001 /srv/ignis/state
cd /opt/ignis-deploy
sudo infra/scripts/restore.sh state-<YYYYMMDDTHHMMSSZ>
sudo systemctl start ignis
sudo rm /etc/ignis/autodeploy.hold
```

To restore from a **local** snapshot instead (faster, no download):

```bash
$C run --rm --no-deps ignis state restore /srv/ignis/backups/<name> --target /srv/ignis/state
```

(This is the command the automatic rollback uses; the same empty-target rule applies.)

### 7.3 Rollback

| Situation | Action |
|---|---|
| A deploy failed its checks | Nothing: the script already rolled back (§4.1 step 8). |
| The release is healthy but wrong | `sudo ignis-autodeploy --rollback`, then put a hold file or push a fix/revert to `main` (otherwise the next tip of `main` deploys again). |
| The previous image cannot open the migrated store | Stop `ignis`, move `/srv/ignis/state` aside, restore the pre-release snapshot (§7.2, local variant), start. A code rollback never undoes a schema migration. |
| UI regression | *Workstation*: `cd infra/worker && npx wrangler rollback` (with the Cloudflare env vars set). A server rollback does not roll the UI back. |

#### When automatic rollback fails

`ROLLBACK FAILED: the origin is down` in the journal and `rollback_failed` in `deploys.jsonl`:

1. Put a hold file so the timer does not retry.
2. `journalctl -u ignis -n 200 --no-pager` and `$C ps` to see which container fails and why.
3. Confirm `IGNIS_IMAGE` in `/etc/ignis/release.env` names an image that exists
   (`sudo docker image ls ignis-server`; the previous one is tagged `ignis-server:rollback`).
4. If the store is the problem, restore the newest snapshot from `/srv/ignis/backups` (§7.2).
5. `sudo systemctl restart ignis`, check health (§8), remove the hold file once `main` has a fix.

---

## 8. Health checks

| Check | Command | Expected |
|---|---|---|
| Units | `systemctl status ignis ignis-autodeploy.timer ignis-backup.timer` | `ignis` active; both timers active |
| Containers | `$C ps` | `ignis` healthy, `cloudflared` running |
| Liveness (no data) | `$C exec -T ignis curl -fsS http://127.0.0.1:8787/livez` | `ok` |
| Readiness | signed-in browser: `https://ignis-ardealu.app/api/v1/capabilities` | JSON naming agent runtime, knowledge, datasets, artifact store, MCP, analysis runner, models; unavailable services carry a reason |
| Core services configured | `journalctl -u ignis --since -10min \| grep -E '(KnowledgeService\|DatasetService) not configured'` | no output |
| Access in front of the origin | *workstation*: `curl -s -o /dev/null -w '%{http_code}\n' https://origin.ignis-ardealu.app/livez` | `403` (no service token) |
| Access in front of the UI | *workstation*: `curl -s -o /dev/null -w '%{http_code}\n' https://ignis-ardealu.app/` | `302` to the Access login |
| Last deploys | `tail -n 5 /var/lib/ignis-deploy/deploys.jsonl` | latest outcome `deployed` / `server_unchanged` + `ui_deployed` |

`/livez` is a process liveness probe only. `/api/v1/capabilities` on the live server needs the Access
tokens, so outside a browser it is checked by the smoke test, on the same image, during each deploy
(the last result is kept in `/var/lib/ignis-deploy/releases/smoke-capabilities.json`).

---

## 9. Logs

Both containers log to journald (`logging.driver: journald`, tags `ignis` and `cloudflared`), and
`ignis.service` runs `docker compose up` in the foreground, so the unit journal carries both.

```bash
journalctl -u ignis -f                         # server + tunnel, live
journalctl -u ignis --since "1 hour ago" --no-pager
journalctl -u ignis-autodeploy -n 200 --no-pager   # deploys (UI step lines are prefixed "ui|", smoke "smoke|")
journalctl -u ignis-backup -n 50 --no-pager        # snapshots
$C logs --tail 200 ignis                       # one service through compose
$C logs --tail 200 cloudflared
sudo docker logs --tail 200 <container>        # container names from `$C ps`
```

Worker logs: Workers observability is enabled in `wrangler.jsonc` (Cloudflare dashboard), or live
from a workstation with `cd infra/worker && npx wrangler tail`.

---

## 10. Common failures and fixes

| Symptom | Likely cause | Fix |
|---|---|---|
| `failed` / `build failed` | compile error on Linux, or disk | Read `journalctl -u ignis-autodeploy`. Fix forward on `main`; the commit is not retried until `main` moves (or `--force`). |
| `only N GB free on … (need 8)` | Docker disk full after the three prune stages | `sudo docker system df`; remove unneeded images; lower `MIN_FREE_GB` only with reason. |
| `smoke test failed` | the new image cannot restore or open the store, or does not serve | The `smoke\|` lines in the journal show the container log. Live traffic was never touched. |
| Deploy `postponed` repeatedly | long turns or jobs | Wait, or deploy with `--no-drain` (cuts them off; the server's restart recovery handles it). |
| `on hold (…)` in the journal | `/etc/ignis/autodeploy.hold` exists | Remove it when the reason is gone. |
| Health check: `KnowledgeService/DatasetService not configured` | data under `/srv/ignis/data` or `/srv/ignis/artifacts/evidence` missing | Run `build-data.sh` (§6), restart. |
| `the ignis container keeps restarting` | config error in `ignis.env`, AUD tags equal, store unreadable | `journalctl -u ignis`; fix the config; for store errors see §7.3. |
| `ui_failed` | npm/Vite/TypeScript error, or bad `edge.env` (missing hostname, unfilled placeholder, expired token) | Fix, then `sudo ignis-autodeploy --ui`. The previous UI keeps serving meanwhile. |
| UI deploys never happen | `/etc/ignis/edge.env` missing (`ui: … missing: UI deploys are off`) | Create it (§3) or deploy by hand (§5). |
| Browser: 502 `origin refused the Worker's service token` | service token rotated or mismatched with the origin Access app / `IGNIS_ACCESS_SERVICE_TOKEN_ID` | Update the Worker secrets (`wrangler secret put …`) and `ignis.env` to the same token; restart `ignis`. |
| Browser: 502 `origin unreachable` | tunnel or origin down | `$C ps`; `$C logs cloudflared`; `systemctl status ignis`. |
| Browser: 503 `origin is not configured` | Worker missing `ORIGIN_URL` or a secret | Check `vars` in the deployed config and `npx wrangler secret list`. |
| Browser: 401 `no Access assertion` | request reached the Worker without the UI Access app in front | Check the UI Access application covers the hostname. |
| `/ntrs/...` shows the app instead of a PDF | deployed config lacks `/ntrs/*` in `run_worker_first` | Redeploy from a freshly generated config (§5). |
| Turns fail with an authentication error (plan sign-in) | a shared plan token was refreshed elsewhere | Sign in again (runbook §Plan sign-in), restart. |
| Turns fail with a provider limit message | shared plan usage limit reached | Wait for the limit to reset, or switch credential (runbook §Credentials). |
| After a deploy, a stale tab cannot load a lazy chunk | old build's chunk names gone | The app reloads itself once (`vite:preloadError` handler in `web/src/main.tsx`); a second failure within a minute is shown. |

---

## 11. Change management

- Production follows `main`; merging is releasing. There is no CI: the PR author runs the targeted
  checks before merging (see [`CONTRIBUTING.md`](../CONTRIBUTING.md)).
- Changes to `infra/scripts/autodeploy.sh` and `infra/deploy/systemd/*` install themselves after the
  first good deploy of the commit that contains them. A broken deploy script therefore needs a manual
  fix on the VM (`install -m 0755 infra/scripts/autodeploy.sh /usr/local/sbin/ignis-autodeploy`
  from a good checkout).
- Record operational changes and measurements in the runbook's [Status](../infra/README.md#status)
  table, with the date and what was actually run.
