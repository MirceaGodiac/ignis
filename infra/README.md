# IGNIS deployment runbook (M4)

What runs where, how to deploy it, how to roll back, back up and restore. Everything here is
reproducible from this directory; nothing in it has been run against the production Cloudflare
account or the origin VM yet (see [Status](#status)). The origin is a Google Compute Engine VM
(Space Apps Google Cloud offer); the stack is plain Linux + Docker, so any equivalent host works.

```text
Browser ──► Cloudflare Access (UI app) ──► Worker "ignis-edge"  (infra/worker)
                                             ├─ static UI (web/dist, Workers Static Assets)
                                             └─ /api/* ──► Cloudflare Access (origin app, service token only)
                                                              │ Tunnel (cloudflared, outbound only)
GCE VM ──────────────────────────────────────────────────────┘
  systemd: ignis.service ─► docker compose (infra/deploy/compose.yaml)
     ignis        ignis serve  (API, SSE, jobs, Codex bridge, MCP on 127.0.0.1, runner client)
     cloudflared  tunnel → http://ignis:8787
  runner containers (gVisor runsc, no network) started per analysis through the host engine
  /srv/ignis/{state,backups,data,artifacts}   persistent disk ignis-data
  /srv/ignis/runner                            persistent disk ignis-runner (fixed size)
Cloudflare R2: artifacts (content-addressed), state snapshots (backups/)
```

No Nginx and no published ports: the only way in is the tunnel, and the tunnel hostname admits only
the Worker's service token. The VM's firewall admits nothing from the internet; administrators reach
it over SSH through Identity-Aware Proxy. The domain comes from the Space Apps domain offer (GoDaddy Registry,
registered at Porkbun, free for one year) and is delegated to Cloudflare, which serves every
hostname above.

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

## Credentials

For now every credential is a plain entry in `/etc/ignis/ignis.env` (root 0600), empty when unused;
the server starts with whichever are filled and `/api/v1/capabilities` says what it uses
(`credentials: environment | shared_login_home | none`). Sealing them into Compose secret files
later needs no code change: the entrypoint already exports `/run/secrets/<NAME>` as `<NAME>`.
None of them is in Git, the image or the compose file.

| Credential | Entry | Read by |
|---|---|---|
| Codex, API billing | `CODEX_API_KEY` | only the Codex child (its environment is cleared otherwise) |
| Codex, ChatGPT plan | `IGNIS_CODEX_LOGIN_HOME=/srv/ignis/login/codex` (see below) | every Codex process |
| Claude, plan token or API billing | `CLAUDE_CODE_OAUTH_TOKEN` or `ANTHROPIC_API_KEY` | only the Claude child |
| Claude, signed-in config dir | `IGNIS_CLAUDE_LOGIN_HOME=/srv/ignis/login/claude` (see below) | every Claude process |
| R2 | `IGNIS_R2_ACCESS_KEY_ID`, `IGNIS_R2_SECRET_ACCESS_KEY` | `ignis` (artifacts, backups) |
| Tunnel | `TUNNEL_TOKEN` in `/etc/ignis/cloudflared.env` (root 0600) | cloudflared |
| Worker service token | `ORIGIN_CLIENT_ID`, `ORIGIN_CLIENT_SECRET` via `wrangler secret put` | the Worker |

The runtime is chosen with `IGNIS_AGENT_RUNTIME` (`codex` or `claude`); restart `ignis` after a
change. The R2 API token is scoped to the one bucket (Object Read & Write). Runner containers get no
host environment, no credential, no network (`sandbox_live` tests the env and mounts).

### Plan sign-in

A plan account is shared by everyone using the deployment: its usage limits are shared, and a
turn that hits one fails with the provider's message as its reason.

- **ChatGPT plan (Codex).** On the VM, once the image exists:

  ```bash
  IMAGE="$(sudo sed -n 's/^IGNIS_IMAGE=//p' /etc/ignis/release.env)"
  sudo mkdir -p /srv/ignis/login/codex && sudo chown 10001:10001 /srv/ignis/login/codex
  sudo docker run --rm -it --user 10001:10001 -e CODEX_HOME=/srv/ignis/login/codex \
    -v /srv/ignis/login/codex:/srv/ignis/login/codex --entrypoint /usr/local/bin/codex "$IMAGE" \
    -c 'cli_auth_credentials_store="file"' login --device-auth
  ```

  It prints a URL and a code: open the URL on any device, sign in to ChatGPT and enter the code.
  Then set `IGNIS_CODEX_LOGIN_HOME=/srv/ignis/login/codex`, leave `CODEX_API_KEY` empty, restart.
  Codex refreshes the token in that directory (it must stay writable). Several workspaces' Codex
  processes share it: if one refresh invalidates another's token, turns fail with an
  authentication error, and signing in again fixes it.
- **Claude plan (Claude Code).** On any computer with Claude Code signed in to the plan, run
  `claude setup-token`; put the printed token in `CLAUDE_CODE_OAUTH_TOKEN`, set
  `IGNIS_AGENT_RUNTIME=claude`, restart. No directory to keep: the token is long-lived.

## Runner

The `ignis` container starts runner containers through the host Docker socket. **Access to that
socket is root-equivalent on the host**: a compromise of the `ignis` process is a host compromise.
Mitigations in place: `ignis` runs as uid 10001 with no capabilities, a read-only root and
`no-new-privileges`; it never runs a shell or user code itself (code runs only in the runner); the
public scope has no shell tool. The runner containers do not see the socket. A rootless engine
dedicated to the runner would remove this; it is not adopted because gVisor's support for rootless
Docker is not verified (decision to revisit after the gVisor probe below).

`/srv/ignis/runner` must be a separate, fixed-size filesystem, so a runaway output cannot fill the
state disk (the per-file `fsize` limit and the ingest budget do not bound the directory). On GCE it
is its own persistent disk (`ignis-runner`, see [First deployment](#first-deployment)), mounted
`nodev,nosuid,noexec`. On a host without separate disks, a loop file does the same:

```bash
fallocate -l 50G /srv/ignis-runner.img && mkfs.ext4 -q /srv/ignis-runner.img
echo '/srv/ignis-runner.img /srv/ignis/runner ext4 loop,nodev,nosuid,noexec 0 2' >> /etc/fstab
mkdir -p /srv/ignis/runner && mount /srv/ignis/runner && chown 10001:10001 /srv/ignis/runner
```

## First deployment

Origin: a Compute Engine VM on the Google Cloud free trial ($300 credit for 90 days, no automatic
charge after it; resources stop when the trial ends unless the account is upgraded, so plan the
move or upgrade before day 90 — see [Moving hosts](#backup-and-restore)). Debian 12, 4 vCPU / 16 GB (`e2-standard-4`), Docker Engine + compose
plugin, gVisor. Commands use `gcloud` from an administrator's machine; set the project first
(`gcloud config set project <project>`) and pick a European zone (e.g. `europe-west3-a`).

0. **Domain.** Claim a non-premium domain through the Space Apps offer (spaceapps.earth; the
   registrar is Porkbun), add it as a zone in Cloudflare and, in Porkbun, replace the default
   nameservers with the two Cloudflare gives. Wait until the zone is *Active* before step 4. The
   bundled web hosting is not used. The free registration lasts one year: renew or move it before
   it lapses.
1. **VM, disks, firewall.** No service account (IGNIS calls no Google API: artifacts and backups go
   to R2) and no inbound rule except SSH from IAP:

   ```bash
   ZONE=europe-west3-a
   gcloud compute disks create ignis-data   --zone=$ZONE --size=200GB --type=pd-balanced
   gcloud compute disks create ignis-runner --zone=$ZONE --size=50GB  --type=pd-balanced
   gcloud compute instances create ignis-origin --zone=$ZONE \
     --machine-type=e2-standard-4 --image-family=debian-12 --image-project=debian-cloud \
     --boot-disk-size=30GB --boot-disk-type=pd-balanced --shielded-secure-boot \
     --no-service-account --no-scopes \
     --disk=name=ignis-data,device-name=ignis-data,auto-delete=no \
     --disk=name=ignis-runner,device-name=ignis-runner,auto-delete=no
   # The default network lets the whole internet reach SSH/RDP: remove that, allow SSH from IAP only.
   gcloud compute firewall-rules delete default-allow-ssh default-allow-rdp --quiet
   gcloud compute firewall-rules create allow-ssh-from-iap --network=default \
     --direction=INGRESS --action=allow --rules=tcp:22 --source-ranges=35.235.240.0/20
   gcloud compute ssh ignis-origin --zone=$ZONE --tunnel-through-iap
   ```

   The VM keeps its ephemeral external IP for outbound traffic only (tunnel, image pulls); nothing
   listens on it. On the VM, format and mount the two disks (`/dev/disk/by-id/google-<name>`):

   ```bash
   for d in ignis-data ignis-runner; do sudo mkfs.ext4 -q -m 0 /dev/disk/by-id/google-$d; done
   sudo mkdir -p /srv/ignis
   echo '/dev/disk/by-id/google-ignis-data /srv/ignis ext4 defaults,nofail 0 2' | sudo tee -a /etc/fstab
   sudo mount /srv/ignis && sudo mkdir -p /srv/ignis/runner
   echo '/dev/disk/by-id/google-ignis-runner /srv/ignis/runner ext4 nodev,nosuid,noexec,nofail 0 2' | sudo tee -a /etc/fstab
   sudo mount /srv/ignis/runner
   ```

   Then install Docker Engine and the compose plugin from Docker's Debian repository.
2. **gVisor.** Install `runsc` from gVisor's apt repository, `runsc install`, restart Docker. Then
   build the runner image (`tools/runner/build.sh`, from the `/opt/ignis-deploy` checkout: do the
   `git bundle` copy of step 6 first) and run `infra/scripts/verify-gvisor.sh <id>`.
   The isolation suite must pass under `runsc`; record the output. If it fails, do not set
   `--runner-allow-container` — leave the runner unconfigured (analyses report it unavailable).
   gVisor's default platform (systrap) needs no nested virtualization, so a standard GCE VM is
   enough.
3. **Directories:** `mkdir -p /srv/ignis/{state,backups,data,artifacts,login/codex,login/claude} /etc/ignis`,
   `chown -R 10001:10001 /srv/ignis` (the runner disk is mounted in step 1).
4. **Cloudflare** (one CLI: Wrangler; dashboard for Zero Trust):
   - R2 bucket `ignis-artifacts` + an R2 API token for it. Lifecycle rule on `backups/` for
     retention (e.g. 30 days); none on `<instance>/objects|manifests` (immutable, referenced).
   - Tunnel, created on the VM so its credentials never leave it (the pinned `cloudflared` image,
     state in `/srv/ignis/cloudflared`, owner 65532, mode 700):

     ```bash
     CF="$(sudo sed -n 's/^CLOUDFLARED_IMAGE=//p' /etc/ignis/release.env)"
     C="sudo docker run --rm -v /srv/ignis/cloudflared:/home/nonroot/.cloudflared $CF"
     $C tunnel login                      # prints a URL: authorize it for the zone in the dashboard
     $C tunnel create ignis-origin
     $C tunnel route dns ignis-origin <origin-hostname>
     printf 'TUNNEL_TOKEN=%s
' "$($C tunnel token ignis-origin)" | sudo tee /etc/ignis/cloudflared.env >/dev/null
     sudo chmod 600 /etc/ignis/cloudflared.env
     ```

     The tunnel is locally managed: its single route (`http://ignis:8787`) is the `--url` in
     `compose.yaml`, not a dashboard setting. SSE works through it unchanged.
   - Access app **origin** on `<origin-hostname>`: policy *Service Auth* allowing only the new
     service token. Note its AUD.
   - Access app **UI** on `<ui-hostname>`: policy for the team's identities. Note its AUD.
   - Login methods (Zero Trust → Integrations → Identity providers). One-time PIN and the
     Cloudflare account login exist by default. For Google: in the Google Cloud project, configure
     the Google Auth Platform (External) and create an OAuth client of type *Web application*
     whose only redirect URI is `https://<team>.cloudflareaccess.com/cdn-cgi/access/callback`;
     then add a *Google* identity provider with that client id and secret, PKCE on. The UI app
     accepts all identity providers, so it needs no change; the origin app stays service-token
     only. While the Google app is in Testing mode, every allowlisted address must also be a test
     user there (publishing to production needs a public homepage and privacy policy URL). The
     origin's checks do not depend on the login method: it verifies the Access JWT and its
     `email`.
5. **Config:** `/etc/ignis/ignis.env` from `deploy/ignis.env.example` (root 0600; fill the
   credentials you use, see [Credentials](#credentials)), `/etc/ignis/release.env` from `deploy/release.env.example` (`DOCKER_GID=$(stat -c %g /var/run/docker.sock)`).
6. **Image, on the VM.** The release is built where it runs, so nothing large crosses the
   administrator's machine. Copy the release commit without giving the VM Git credentials:

   ```bash
   git bundle create /tmp/ignis.bundle <release-commit-or-main>      # on the admin machine
   gcloud compute scp /tmp/ignis.bundle ignis-origin:/tmp/ --zone=$ZONE --tunnel-through-iap
   # on the VM:
   sudo git clone -q /tmp/ignis.bundle /opt/ignis-deploy && cd /opt/ignis-deploy
   sudo git checkout -q <release-commit>
   sudo infra/scripts/fetch-codex.sh /opt/ignis-cli              # pinned Codex, npm in a container
   sudo infra/scripts/fetch-claude.sh /opt/ignis-cli             # Claude Code (for the claude runtime)
   sudo infra/scripts/build-server.sh /opt/ignis-cli/codex /opt/ignis-cli/claude   # → runs/release/<commit>.json
   ```

   Set `IGNIS_IMAGE` in `/etc/ignis/release.env` to the image id from the manifest. Pin
   `CLOUDFLARED_IMAGE` by digest too (`docker pull cloudflare/cloudflared:latest`, then
   `docker inspect --format '{{index .RepoDigests 0}}' cloudflare/cloudflared:latest`).
7. **Data, on the VM:** `sudo infra/scripts/build-data.sh`. With the release image it downloads the
   bounded NASA sample set and the evidence-corpus reports from public NASA servers (about 0.3 GB),
   builds the evidence catalog (`/srv/ignis/artifacts/evidence/`), the records and the FLEX/ACME
   builder bundles (`/srv/ignis/data/`). Every step must end without an error; the last section
   lists listing manifests NASA changed since the release (review them, do not commit blindly).
   Re-run it after each release that changes ingest or bundle code.
8. **Start:** copy `infra/deploy/systemd/*` from `/opt/ignis-deploy` to `/etc/systemd/system/`,
   `systemctl daemon-reload && systemctl enable --now ignis ignis-backup.timer`. The log must not
   say `KnowledgeService not configured` or `DatasetService not configured`.
9. **Worker:** `npm ci --prefix web && npm --prefix web run build`, fill the placeholders in
   `worker/wrangler.jsonc`, `cd infra/worker && npm ci && npx wrangler secret put ORIGIN_CLIENT_ID`
   (and `ORIGIN_CLIENT_SECRET`), `npx wrangler deploy`.
10. **Check:** `docker compose … ps` healthy; in the browser, sign in and open
   `/api/v1/capabilities` (agent runtime, runner, artifact store); one cited answer and one
   numeric analysis end to end; `curl https://<origin-hostname>/api/health` without the service
   token is refused by Access.

## Release and rollback

1. Build the new image on the VM (step 6); re-run step 7 if ingest or bundle code changed — the manifest names commit, toolchain, Codex hash, base digests.
2. Back up first: `systemctl start ignis-backup` (and wait for "pushed").
3. Database migrations: `ResearchStore::open` migrates on start. Test it on a copy first:
   `ignis state restore <latest snapshot> --target /tmp/migrate-test` then start the new image
   against that directory with `--port 0`; it must open the store and serve `/livez`.
4. Edit `IGNIS_IMAGE` in `/etc/ignis/release.env`, `systemctl restart ignis`; verify as in step 10.
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
- Persistent-disk snapshots of `ignis-data` are optional (a snapshot schedule can be attached);
  the R2 snapshots above are the backup the restore procedure relies on.
- **Moving hosts** (e.g. when the cloud credits end): provision the new host from
  [First deployment](#first-deployment), restore the latest snapshot there, point the tunnel at it
  (same token), then stop the old VM. Artifacts stay in R2 and need no copy.
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
| Server image, compose, systemd | built on the GCE VM and running under `ignis.service` (2026-11-14) |
| gVisor | **verified** on the GCE VM (2026-11-14): `verify-gvisor.sh` passes; `runsc` 4.19.0-gvisor kernel; runner preflight reports `isolation=Gvisor` |
| Domain, Cloudflare zone, tunnel, Access apps, Worker, R2, GCE VM | **provisioned** (2026-11-14): `ignis-ardealu.app` (bought on Cloudflare Registrar: the Space Apps domain offer failed with "Exceptions Occurred"), UI on the apex, origin on `origin.ignis-ardealu.app`, VM `ignis-origin` in `europe-west3-a` |
| Access login methods | Google, Cloudflare account, one-time PIN on the UI app (Google added 2026-11-14: OAuth client "Cloudflare Access" in `ignis-spaceapps-ardealu`, Testing mode, PKCE). Cloudflare's IdP test returned the expected identity; the allowlist (policy *team*) is unchanged |
| RPO/RTO | not measured (one backup pushed to R2: 386 files in 73 s; no restore drill yet) |

### Live verification on the deployed stack (2026-11-14)

Run from a signed-in browser through the whole path (Access → Worker → service token → tunnel →
server), model `gpt-6.1-terra`, models (FLEX/ACME/BASS) disabled. What was run and what it showed:

| Check | Result |
|---|---|
| Unauthenticated access | origin `/livez` and `/api/v1/capabilities` → 403 (service-token-only Access app); UI → 302 to the Access login |
| Capabilities | agent runtime, knowledge, datasets, artifact store, MCP (6 tools) and analysis runner available; models "disabled in this deployment" |
| Real turn | cited answer (`sources_search` + `sources_read`) |
| Two concurrent sessions | both completed with cited answers; one made 9 tool calls |
| Typed failure | unknown episode → 404 `not_found`, `retry: never`; unknown body field → 422 `invalid_request` |
| Cancel | `POST /episodes/{id}/cancel` 4 s into a turn → `cancelled` |
| Restart | `systemctl restart ignis`: all four episodes still listed with their final status |
| Large artifact | 64 MiB upload (initiate → PUT → complete) in 8.7 s; stored in R2 (not on the VM disk); download byte-identical by sha256 |
| Shell request | not refused: the model ran `uname`/`cat /etc/passwd` through `analysis_execute`, so the output came from the gVisor runner (container `/etc/passwd`, gVisor kernel), not the host. No approval prompt was exercised |
| Backup | `ignis-backup` pushed `backups/state-…/` to R2 |

Not done: refused approval (no prompt was triggered), restore drill (RTO), a turn interrupted by
the restart (the restart ran between turns).
