#!/usr/bin/env bash
# Pull-model release of the origin: take the tip of a branch (main) from GitHub to production.
#
#   ignis-autodeploy                     # what the timer runs: deploy the branch tip if it is new
#   ignis-autodeploy --force             # redeploy the branch tip even if it is already running
#   ignis-autodeploy --commit <sha>      # deploy this commit (must be reachable from the remote)
#   ignis-autodeploy --drill-rollback    # deploy, then treat the release as failed: checks rollback
#   ignis-autodeploy --rollback          # put the previous release back (image + checkout)
#   ignis-autodeploy --no-drain          # do not wait for running turns/jobs (they are cut off)
#
# Installed as /usr/local/sbin/ignis-autodeploy by infra/scripts/autodeploy-setup.sh and run by
# ignis-autodeploy.service (timer: ignis-autodeploy.timer). Steps, each one fail-closed:
#   fetch (read-only deploy key) → build in a separate worktree (build-server.sh, Codex fetched when
#   its pin changed) → backup (ignis-backup.service) → smoke test of the NEW image against a
#   restore of that backup (store opens/migrates, /livez, /api/v1/capabilities, no network) →
#   wait until no turn or job is running → switch IGNIS_IMAGE and the checkout → restart → health
#   checks → on failure, automatic rollback to the previous image and checkout (and, if the old
#   image cannot open the store the new one migrated, a restore of the pre-release snapshot).
#
# Nothing here reads or writes credentials besides the deploy key path. A commit whose deploy
# failed is not retried by the timer until the branch moves (or --force / --commit).
#
# The whole body is in main() so bash has parsed it before the checkout under it changes.
set -euo pipefail

main() {
  local force=0 drill=0 rollback_only=0 drain=1 commit=""
  while [ $# -gt 0 ]; do
    case "$1" in
      --force) force=1 ;;
      --commit) commit="${2:?--commit needs a sha}"; shift ;;
      --drill-rollback) drill=1; force=1 ;;
      --rollback) rollback_only=1 ;;
      --no-drain) drain=0 ;;
      *) echo "unknown argument: $1" >&2; return 64 ;;
    esac
    shift
  done

  # Configuration (/etc/ignis/autodeploy.env, root 0644; no secrets in it).
  REMOTE="git@github.com:MirceaGodiac/ignis.git"
  BRANCH="main"
  DEPLOY_KEY="/etc/ignis/deploy_key"
  KNOWN_HOSTS="/etc/ignis/github_known_hosts"
  REPO="/opt/ignis-deploy"
  CLI_DIR="/opt/ignis-cli"
  STATE="/var/lib/ignis-deploy"
  RELEASE_ENV="/etc/ignis/release.env"
  IGNIS_ENV="/etc/ignis/ignis.env"
  HOLD_FILE="/etc/ignis/autodeploy.hold"
  DRAIN_TIMEOUT=1800      # seconds to wait for running turns/jobs before postponing
  HEALTH_TIMEOUT=240      # seconds for the restarted service to become healthy
  MIN_FREE_GB=8           # free space needed on the Docker disk before a build
  # shellcheck disable=SC1091
  [ -f /etc/ignis/autodeploy.env ] && . /etc/ignis/autodeploy.env

  BUILD="$STATE/build"
  COMPOSE=(docker compose -f "$REPO/infra/deploy/compose.yaml" --env-file "$RELEASE_ENV")
  export GIT_SSH_COMMAND="ssh -i $DEPLOY_KEY -o IdentitiesOnly=yes -o StrictHostKeyChecking=yes -o UserKnownHostsFile=$KNOWN_HOSTS -o BatchMode=yes"
  mkdir -p "$STATE/releases"

  exec 9>"$STATE/lock"
  if ! flock -n 9; then log "another deploy is running"; return 0; fi

  if [ "$rollback_only" = 1 ]; then
    manual_rollback
    return
  fi
  if [ -e "$HOLD_FILE" ] && [ "$force" = 0 ] && [ -z "$commit" ]; then
    log "on hold ($HOLD_FILE exists): $(head -c 200 "$HOLD_FILE" | tr '\n' ' ')"
    return 0
  fi

  T0=$(date +%s)
  git -C "$REPO" fetch --quiet "$REMOTE" "+refs/heads/$BRANCH:refs/remotes/deploy/$BRANCH"
  local target
  if [ -n "$commit" ]; then
    target="$(git -C "$REPO" rev-parse --verify "$commit^{commit}")"
    git -C "$REPO" merge-base --is-ancestor "$target" "refs/remotes/deploy/$BRANCH" \
      || { log "$target is not on $BRANCH"; return 1; }
  else
    target="$(git -C "$REPO" rev-parse "refs/remotes/deploy/$BRANCH")"
  fi

  local cur_image cur_commit
  cur_image="$(env_get IGNIS_IMAGE)"
  cur_commit="$(image_commit "$cur_image")"
  if [ "$force" = 0 ] && [ -z "$commit" ]; then
    if [ "$target" = "$cur_commit" ]; then return 0; fi
    if [ "$(cat "$STATE/failed" 2>/dev/null)" = "$target" ]; then
      log "$target failed before; not retrying until $BRANCH moves (or --force)"
      return 0
    fi
  fi
  log "deploying $target (running: ${cur_commit:-unknown}, image $cur_image)"
  DEPLOY_ID="$(date -u +%Y%m%dT%H%M%SZ)-${target:0:12}"

  # 1. Build, in a worktree of its own: the serving checkout does not move until the switch.
  ensure_disk
  if [ ! -d "$BUILD/.git" ] && [ ! -f "$BUILD/.git" ]; then
    git -C "$REPO" worktree prune
    git -C "$REPO" worktree add --quiet --force --detach "$BUILD" "$target"
  fi
  git -C "$BUILD" checkout --quiet --force --detach "$target"
  git -C "$BUILD" clean -qfdx
  local pin have=""
  pin="$(grep -ohE 'PINNED_CODEX_VERSION: &str = "[^"]+"' "$BUILD"/crates/ignis-research/src/runtime/*.rs | head -1 | sed -E 's/.*"([^"]+)"/\1/')"
  [ -x "$CLI_DIR/codex" ] && have="$("$CLI_DIR/codex" --version | awk 'NR==1 {print $NF}')"
  if [ "$have" != "$pin" ]; then
    log "codex pin $pin (have ${have:-none}): fetching"
    "$BUILD/infra/scripts/fetch-codex.sh" "$CLI_DIR"
  fi
  local claude=""
  [ -x "$CLI_DIR/claude" ] && claude="$CLI_DIR/claude"
  local tb
  tb=$(date +%s)
  if ! "$BUILD/infra/scripts/build-server.sh" "$CLI_DIR/codex" $claude; then
    fail_record "$target" "build failed"
    return 1
  fi
  BUILD_S=$(( $(date +%s) - tb ))
  cp "$BUILD/runs/release/$target.json" "$STATE/releases/"
  local new_image
  new_image="$(sed -nE 's/.*"image_id": "([^"]+)".*/\1/p' "$STATE/releases/$target.json")"
  [ -n "$new_image" ] || { fail_record "$target" "no image id in the manifest"; return 1; }
  log "built $new_image in ${BUILD_S}s"

  # 2. Backup of the live store (the rollback point), then 3. smoke test against a copy of it.
  systemctl start ignis-backup.service
  local snap
  snap="$(find /srv/ignis/backups -mindepth 1 -maxdepth 1 -type d -name 'state-*' | sort | tail -n 1)"
  [ -n "$snap" ] || { fail_record "$target" "no local snapshot after the backup"; return 1; }
  echo "$snap" > "$STATE/last_snapshot"
  log "backup $snap"
  if ! smoke_test "$new_image" "$snap"; then
    fail_record "$target" "smoke test failed"
    return 1
  fi

  # 4. Drain: never cut off a running turn or job blindly.
  if [ "$drain" = 1 ] && ! wait_idle; then
    log "still busy after ${DRAIN_TIMEOUT}s: postponed (next timer run retries)"
    record "$target" postponed
    return 0
  fi

  # 5. Switch and restart.
  echo "$cur_image $(git -C "$REPO" rev-parse HEAD)" > "$STATE/previous"
  cp "$RELEASE_ENV" "$STATE/release.env.previous"
  local td=$(date +%s)
  switch_to "$new_image" "$target"
  local ok=0
  if healthy "$new_image" "$td"; then
    ok=1
    DOWN_S=$(( HEALTHY_AT - td ))
    log "$target healthy ${DOWN_S}s after the restart"
  fi
  if [ "$ok" = 1 ] && [ "$drill" = 0 ]; then
    install_tools "$target"
    rm -f "$STATE/failed"
    prune_images "$new_image" "$cur_image"
    record "$target" deployed
    log "deployed $target in $(( $(date +%s) - T0 ))s (build ${BUILD_S}s, restart-to-healthy ${DOWN_S}s)"
    return 0
  fi

  # 6. Roll back.
  if [ "$drill" = 1 ]; then
    [ "$ok" = 1 ] || { fail_record "$target" "drill: the new release did not come up"; }
    log "drill: treating $target as failed"
  fi
  local prev_commit
  prev_commit="$(awk '{print $2}' "$STATE/previous")"
  cp "$STATE/release.env.previous" "$RELEASE_ENV"
  local tr=$(date +%s)
  switch_to "$cur_image" "$prev_commit"
  if healthy "$cur_image" "$tr"; then
    log "rolled back to $prev_commit ($cur_image) in $(( HEALTHY_AT - tr ))s"
  elif restore_snapshot "$snap" && healthy "$cur_image" "$(date +%s)"; then
    log "rolled back to $prev_commit after restoring $snap"
  else
    log "ROLLBACK FAILED: the origin is down; see infra/README.md §Automatic deploys"
    record "$target" rollback_failed
    return 2
  fi
  if [ "$drill" = 1 ] && [ "$ok" = 1 ]; then
    record "$target" drill_rolled_back
    return 0
  fi
  fail_record "$target" "unhealthy after restart; rolled back"
  return 1
}

log() { echo "ignis-autodeploy: $*"; }

env_get() { sed -n "s/^$1=//p" "$RELEASE_ENV" | tail -n 1; }

image_commit() {
  docker image inspect --format '{{index .Config.Labels "org.opencontainers.image.revision"}}' "$1" 2>/dev/null || true
}

record() {
  local now
  now=$(date +%s)
  printf '{"deploy":"%s","commit":"%s","outcome":"%s","build_s":%s,"restart_to_healthy_s":%s,"total_s":%s,"at":"%s"}\n' \
    "${DEPLOY_ID:-}" "$1" "$2" "${BUILD_S:-null}" "${DOWN_S:-null}" "$(( now - ${T0:-now} ))" \
    "$(date -u +%Y-%m-%dT%H:%M:%SZ)" >> "$STATE/deploys.jsonl"
}

fail_record() {
  log "FAILED $1: $2"
  echo "$1" > "$STATE/failed"
  record "$1" failed
}

# Free space on the Docker disk: drop dangling images and build cache older than a week first.
ensure_disk() {
  local root avail
  root="$(docker info --format '{{.DockerRootDir}}')"
  avail=$(( $(df --output=avail -k "$root" | tail -n 1) / 1048576 ))
  if [ "$avail" -lt "$MIN_FREE_GB" ]; then
    log "${avail} GB free on $root: pruning"
    docker image prune -f >/dev/null
    docker builder prune -f --filter until=168h >/dev/null
    avail=$(( $(df --output=avail -k "$root" | tail -n 1) / 1048576 ))
  fi
  [ "$avail" -ge "$MIN_FREE_GB" ] || { log "only ${avail} GB free on $root (need $MIN_FREE_GB)"; return 1; }
}

# The new image opens (and migrates) a restored copy of the live store and serves /livez and
# /api/v1/capabilities, with no network (no model call, no R2 write), local auth on loopback.
smoke_test() {
  local image="$1" snap="$2" dir=/srv/ignis/deploy-smoke name=ignis-deploy-smoke ok=1
  docker rm -f "$name" >/dev/null 2>&1 || true
  rm -rf "$dir" && mkdir -p "$dir" && chown 10001:10001 "$dir"
  if ! docker run --rm --network none --user 10001:10001 -v "$snap:/snap:ro" -v "$dir:/smoke" \
      "$image" state restore /snap --target /smoke/state >/dev/null; then
    log "smoke: the new image cannot restore $snap"
    return 1
  fi
  docker run -d --name "$name" --network none --user 10001:10001 --read-only --tmpfs /tmp:size=256m \
    --env-file "$IGNIS_ENV" -e IGNIS_AUTH=local -e IGNIS_ARTIFACT_BACKEND=local \
    -e IGNIS_STATE_DIR=/smoke/state -e IGNIS_RUNS=/smoke/state/runs -e IGNIS_MCP_PORT=0 \
    -v "$dir:/smoke" -v /srv/ignis/data:/opt/ignis/data/work:ro -v /srv/ignis/artifacts:/opt/ignis/artifacts:ro \
    "$image" serve --host 127.0.0.1 --port 8787 >/dev/null
  local i
  for i in $(seq 1 60); do
    docker exec "$name" curl -fsS -o /dev/null http://127.0.0.1:8787/livez 2>/dev/null && break
    [ "$(docker inspect -f '{{.State.Running}}' "$name" 2>/dev/null)" = true ] || break
    sleep 2
  done
  if docker exec "$name" curl -fsS http://127.0.0.1:8787/api/v1/capabilities > "$STATE/releases/smoke-capabilities.json" 2>/dev/null \
     && head -c 1 "$STATE/releases/smoke-capabilities.json" | grep -q '{'; then
    log "smoke: store opened, /livez and /api/v1/capabilities ok; tools: $(grep -oE '"(sources|datasets|analysis|answer|models|wiki)_[a-z_]+"' "$STATE/releases/smoke-capabilities.json" | tr -d '"' | sort -u | tr '\n' ' ')"
  else
    ok=0
    log "smoke: the new image did not serve; its log:"
    docker logs --tail 30 "$name" 2>&1 | sed 's/^/  smoke| /'
  fi
  docker rm -f "$name" >/dev/null 2>&1 || true
  rm -rf "$dir"
  [ "$ok" = 1 ]
}

# Running turns and jobs in the live store (read-only query inside the running container).
busy() {
  "${COMPOSE[@]}" exec -T ignis sqlite3 -readonly /srv/ignis/state/state.sqlite \
    "SELECT (SELECT count(*) FROM runtime_turns WHERE state IN ('starting','running'))
          + (SELECT count(*) FROM jobs WHERE status IN ('running','cancel_requested','reconciling'));" 2>/dev/null \
  || "${COMPOSE[@]}" exec -T ignis sqlite3 -readonly /srv/ignis/state/state.sqlite \
    "SELECT count(*) FROM jobs WHERE status IN ('running','cancel_requested','reconciling');" 2>/dev/null \
  || echo 0   # service down: nothing to drain
}

wait_idle() {
  local start n
  start=$(date +%s)
  while :; do
    n="$(busy | tr -dc 0-9)"
    [ "${n:-0}" = 0 ] && return 0
    [ $(( $(date +%s) - start )) -ge "$DRAIN_TIMEOUT" ] && return 1
    log "waiting: $n running turn(s)/job(s)"
    sleep 20
  done
}

switch_to() {
  local image="$1" commit="$2"
  sed -i "s|^IGNIS_IMAGE=.*|IGNIS_IMAGE=$image|" "$RELEASE_ENV"
  git -C "$REPO" checkout --quiet --force --detach "$commit"
  systemctl restart ignis.service
}

# Healthy = the ignis container runs the expected image and its healthcheck passes, /livez answers,
# cloudflared is up, and the log since the restart has no "… not configured" for the core services.
healthy() {
  local image="$1" since="$2" id cf i base_id="" base_rc=0 rc
  for i in $(seq 1 $(( HEALTH_TIMEOUT / 5 ))); do
    sleep 5
    id="$("${COMPOSE[@]}" ps -q ignis 2>/dev/null)" || continue
    [ -n "$id" ] || continue
    [ "$(docker inspect -f '{{.Image}}' "$id")" = "$(docker image inspect -f '{{.Id}}' "$image")" ] || continue
    # RestartCount is cumulative per container: count only restarts seen during this check.
    rc="$(docker inspect -f '{{.RestartCount}}' "$id")"
    if [ "$id" != "$base_id" ]; then base_id="$id"; base_rc="$rc"; fi
    if [ $(( rc - base_rc )) -ge 3 ]; then
      log "the ignis container keeps restarting"
      break
    fi
    [ "$(docker inspect -f '{{.State.Health.Status}}' "$id")" = healthy ] || continue
    docker exec "$id" curl -fsS -o /dev/null http://127.0.0.1:8787/livez || continue
    cf="$("${COMPOSE[@]}" ps -q cloudflared 2>/dev/null)"
    [ -n "$cf" ] && [ "$(docker inspect -f '{{.State.Running}}' "$cf")" = true ] || continue
    if journalctl -u ignis.service --since "@$since" --no-pager -q | grep -qE '(KnowledgeService|DatasetService) not configured'; then
      log "log says a core service is not configured"
      return 1
    fi
    HEALTHY_AT=$(date +%s)
    return 0
  done
  log "not healthy within ${HEALTH_TIMEOUT}s"
  journalctl -u ignis.service --since "@$since" --no-pager -q | tail -n 20 | sed 's/^/  ignis| /'
  return 1
}

# The old image could not open the store (the new one migrated it): put the pre-release snapshot
# back. The failed state is moved aside, never deleted.
restore_snapshot() {
  local snap="$1" aside
  aside="/srv/ignis/state.failed-$(date -u +%Y%m%dT%H%M%SZ)"
  log "restoring $snap (current state moved to $aside)"
  systemctl stop ignis.service
  mv /srv/ignis/state "$aside"
  mkdir /srv/ignis/state && chown 10001:10001 /srv/ignis/state
  "${COMPOSE[@]}" run --rm --no-deps ignis state restore "$snap" --target /srv/ignis/state \
    && systemctl start ignis.service
}

# After a good deploy: the deploy tooling and units follow the deployed commit (when it has them).
install_tools() {
  local f changed=0
  if [ -f "$REPO/infra/scripts/autodeploy.sh" ] && ! cmp -s "$REPO/infra/scripts/autodeploy.sh" /usr/local/sbin/ignis-autodeploy; then
    install -m 0755 "$REPO/infra/scripts/autodeploy.sh" /usr/local/sbin/ignis-autodeploy
    log "updated /usr/local/sbin/ignis-autodeploy"
  fi
  for f in "$REPO"/infra/deploy/systemd/*; do
    [ -f "$f" ] || continue
    if ! cmp -s "$f" "/etc/systemd/system/$(basename "$f")"; then
      install -m 0644 "$f" /etc/systemd/system/
      changed=1
    fi
  done
  [ "$changed" = 0 ] || { systemctl daemon-reload; log "systemd units updated"; }
}

# Keep the running and the previous image (the rollback target); drop older ignis-server images.
prune_images() {
  local keep1 keep2 id
  keep1="$(docker image inspect -f '{{.Id}}' "$1")"
  keep2="$(docker image inspect -f '{{.Id}}' "$2" 2>/dev/null || true)"
  for id in $(docker images --no-trunc --format '{{.ID}}' ignis-server | sort -u); do
    [ "$id" = "$keep1" ] || [ "$id" = "$keep2" ] || docker rmi -f "$id" >/dev/null 2>&1 || true
  done
  docker image prune -f >/dev/null
}

manual_rollback() {
  [ -f "$STATE/previous" ] || { log "no previous release recorded"; return 1; }
  local prev_image prev_commit cur_image
  read -r prev_image prev_commit < "$STATE/previous"
  cur_image="$(env_get IGNIS_IMAGE)"
  log "rolling back to $prev_commit ($prev_image)"
  echo "$cur_image $(git -C "$REPO" rev-parse HEAD)" > "$STATE/previous"
  local t=$(date +%s)
  switch_to "$prev_image" "$prev_commit"
  healthy "$prev_image" "$t" && log "rolled back in $(( HEALTHY_AT - t ))s"
}

main "$@"
exit $?
