# IGNIS deployment runbook (M4)

What runs where, how to deploy it, how to roll back, back up and restore. Everything here is
reproducible from this directory; nothing in it has been run against the production Cloudflare
account or the Netcup host yet (see [Status](#status)).

```text
Browser ──► Cloudflare Access (UI app) ──► Worker "ignis-edge"  (infra/worker)
                                             ├─ static UI (web/dist, Workers Static Assets)
                                             └─ /api/* ──► Cloudflare Access (origin app, service token only)
                                                              │ Tunnel (cloudflared, outbound only)
Netcup host ─────────────────────────────────────────────────┘
  systemd: ignis.service ─► docker compose (infra/deploy/compose.yaml)
     ignis        ignis serve  (API, SSE, jobs, Codex bridge, MCP on 127.0.0.1, runner client)
     cloudflared  tunnel → http://ignis:8787
  runner containers (gVisor runsc, no network) started per analysis through the host engine
  /srv/ignis/{state,runner,backups,data,artifacts}   local NVMe
Cloudflare R2: artifacts (content-addressed), state snapshots (backups/)
```

No Nginx and no published ports: the only way in is the tunnel, and the tunnel hostname admits only
the Worker's service token.

## Identity

Two Access applications, two signed tokens, both verified in Rust (`ignis-server/src/v1/access.rs`):

| Token | Set by | Checked at the origin |
|---|---|---|
| `Cf-Access-Jwt-Assertion` | Access on the **origin** hostname | RS256 against `<team>/cdn-cgi/access/certs`, `iss` = team domain, `aud` = origin AUD, `common_name` = the Worker's service-token client id |
| `X-Ignis-User-Assertion` | the Worker, copied from the user's own `Cf-Access-Jwt-Assertion` on the **UI** hostname | same keys and issuer, `aud` = UI AUD, `email` present |

The verified e-mail (lowercased) is the principal; the workspace and `ExecutionScope` are derived
from it by the host. The unsigned `Cf-Access-Authenticated-User-Email` header is ignored. The two
applications must have different AUD tags (the server refuses to start otherwise), so a user token
can never open the origin directly. The Worker drops any client-supplied copy of these headers.
`/livez` (no data) is the only unauthenticated route; everything else, including the legacy `/api/*`,
needs at least the origin token.

## Secrets

Never in Git, the image, the compose file or the Worker config.

| Secret | Where it lives | Who reads it |
|---|---|---|
| `CODEX_API_KEY` | `/etc/ignis/secrets/CODEX_API_KEY` (root:10001 0440) | entrypoint → `ignis` env → only the Codex child (env cleared otherwise) |
| `IGNIS_R2_SECRET_ACCESS_KEY` | `/etc/ignis/secrets/IGNIS_R2_SECRET_ACCESS_KEY` (root:10001 0440) | `ignis` (R2 artifacts, backups) |
| `TUNNEL_TOKEN` | `/etc/ignis/cloudflared.env` (root 0600) | cloudflared |
| Worker service token (`ORIGIN_CLIENT_ID`, `ORIGIN_CLIENT_SECRET`) | `wrangler secret put` | the Worker |

The R2 API token is scoped to the one bucket (Object Read & Write). Runner containers get no host
environment, no secret mounts, no network (`sandbox_live` tests the env and mounts).

## Runner

The `ignis` container starts runner containers through the host Docker socket. **Access to that
socket is root-equivalent on the host**: a compromise of the `ignis` process is a host compromise.
Mitigations in place: `ignis` runs as uid 10001 with no capabilities, a read-only root and
`no-new-privileges`; it never runs a shell or user code itself (code runs only in the runner); the
public scope has no shell tool. The runner containers do not see the socket. A rootless engine
dedicated to the runner would remove this; it is not adopted because gVisor's support for rootless
Docker is not verified (decision to revisit after the gVisor probe below).

`/srv/ignis/runner` must be a separate, fixed-size filesystem, so a runaway output cannot fill the
state disk (the per-file `fsize` limit and the ingest budget do not bound the directory):

```bash
fallocate -l 50G /srv/ignis-runner.img && mkfs.ext4 -q /srv/ignis-runner.img
echo '/srv/ignis-runner.img /srv/ignis/runner ext4 loop,nodev,nosuid,noexec 0 2' >> /etc/fstab
mkdir -p /srv/ignis/runner && mount /srv/ignis/runner && chown 10001:10001 /srv/ignis/runner
```

## First deployment

Netcup host (Debian 12/Ubuntu 24.04, ~16 GB RAM): Docker Engine + compose plugin, gVisor.

1. **gVisor.** Install `runsc` from gVisor's apt repository, `runsc install`, restart Docker. Then
   build the runner image (`tools/runner/build.sh`) and run `infra/scripts/verify-gvisor.sh <id>`.
   The isolation suite must pass under `runsc`; record the output. If it fails, do not set
   `--runner-allow-container` — leave the runner unconfigured (analyses report it unavailable).
2. **Directories:** `mkdir -p /srv/ignis/{state,backups,data,artifacts} /etc/ignis/secrets`,
   `chown -R 10001:10001 /srv/ignis`, the runner filesystem above.
3. **Cloudflare** (one CLI: Wrangler; dashboard for Zero Trust):
   - R2 bucket `ignis-artifacts` + an R2 API token for it. Lifecycle rule on `backups/` for
     retention (e.g. 30 days); none on `<instance>/objects|manifests` (immutable, referenced).
   - Tunnel → token in `/etc/ignis/cloudflared.env`; public hostname `<origin-hostname>` →
     `http://ignis:8787`; tunnel settings: no chunked encoding issues for SSE (default ok).
   - Access app **origin** on `<origin-hostname>`: policy *Service Auth* allowing only the new
     service token. Note its AUD.
   - Access app **UI** on `<ui-hostname>`: policy for the team's identities. Note its AUD.
4. **Config:** `/etc/ignis/ignis.env` from `deploy/ignis.env.example`, the two secret files,
   `/etc/ignis/release.env` from `deploy/release.env.example` (`DOCKER_GID=$(stat -c %g /var/run/docker.sock)`).
5. **Image:** on a build machine, from a clean checkout of the release commit:
   `infra/scripts/build-server.sh <linux codex binary>` → `runs/release/<commit>.json`. Transfer the
   image (`docker save | ssh host docker load`) and set `IGNIS_IMAGE` to its id/digest. Pin
   `CLOUDFLARED_IMAGE` by digest too.
6. **Start:** checkout to `/opt/ignis-deploy`, copy `deploy/systemd/*` to `/etc/systemd/system/`,
   `systemctl daemon-reload && systemctl enable --now ignis ignis-backup.timer`.
7. **Worker:** `npm ci --prefix web && npm --prefix web run build`, fill the placeholders in
   `worker/wrangler.jsonc`, `cd infra/worker && npm ci && npx wrangler secret put ORIGIN_CLIENT_ID`
   (and `ORIGIN_CLIENT_SECRET`), `npx wrangler deploy`.
8. **Check:** `docker compose … ps` healthy; in the browser, sign in and open
   `/api/v1/capabilities` (agent runtime, runner, artifact store); one cited answer and one
   numeric analysis end to end; `curl https://<origin-hostname>/api/health` without the service
   token is refused by Access.

## Release and rollback

1. Build the new image (step 5) — the manifest names commit, toolchain, Codex hash, base digests.
2. Back up first: `systemctl start ignis-backup` (and wait for "pushed").
3. Database migrations: `ResearchStore::open` migrates on start. Test it on a copy first:
   `ignis state restore <latest snapshot> --target /tmp/migrate-test` then start the new image
   against that directory with `--port 0`; it must open the store and serve `/livez`.
4. Edit `IGNIS_IMAGE` in `/etc/ignis/release.env`, `systemctl restart ignis`; verify as in step 8.
5. Rollback: restore the previous `IGNIS_IMAGE` and restart. A code rollback does **not** undo a
   schema migration: if the new release migrated the store and the old one cannot read it, stop,
   `mv /srv/ignis/state /srv/ignis/state.failed`, restore the pre-release snapshot, restart.

## Backup and restore

- `ignis-backup.timer` runs `scripts/backup.sh` hourly: `ignis state backup` inside the running
  container takes a consistent snapshot (`VACUUM INTO` for each SQLite database while the server
  writes, plain copies of the append-only Codex logs and reveal log), hashes every file into
  `manifest.json`, uploads files then manifest under `backups/<name>/`. Excluded: WAL/SHM files,
  upload and artifact staging, credentials (`auth.json`, `*.key`, `*.env`).
- Artifacts are not in the snapshot: they live in R2, immutable and content-addressed; restored
  manifests point at the same objects.
- **Restore** onto an empty volume: `systemctl stop ignis`, empty/new `/srv/ignis/state`,
  `infra/scripts/restore.sh <name>`, `systemctl start ignis`. The restore verifies every hash and
  runs `PRAGMA integrity_check`; it refuses a non-empty target.
- **Drill** (do it after the first deploy and after schema changes): restore the latest snapshot
  into a scratch directory and start a second instance on it (`--state-dir`, `--port 0`, local
  auth on loopback); check an episode's events replay. Measure how long it took (that is the RTO
  we have, not a promise); RPO is the timer interval.

## Status

| Piece | State |
|---|---|
| Access JWT verification, Worker forwarding | implemented, tested offline (signed test tokens, local JWKS; Worker unit tests) |
| R2 artifact store, snapshots, push/pull | implemented, tested offline against an S3 stand-in that verifies SigV4 and body hashes; SigV4 checked against AWS's published example. Live bucket test: `cargo test -p ignis-research --test r2 r2_live -- --ignored` with `IGNIS_R2_LIVE_*` — **not run yet** |
| Server image, compose, systemd | written; compose validated with `docker compose config`; **image not built here yet** |
| gVisor | **not verified** — `scripts/verify-gvisor.sh` on the Netcup host |
| Cloudflare account, tunnel, Access apps, Netcup host | **not provisioned** (needs the account owner's authorization) |
| RPO/RTO | not measured |
