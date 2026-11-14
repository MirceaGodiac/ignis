#!/usr/bin/env bash
# One consistent snapshot of /srv/ignis/state, verified and pushed to R2 under $IGNIS_BACKUP_PREFIX.
# Runs inside the running ignis container (same binary, same R2 credentials) and keeps the last
# $KEEP_LOCAL snapshots in /srv/ignis/backups. Used by ignis-backup.service; safe to run by hand.
set -euo pipefail
cd "$(dirname "$0")/../.."
KEEP_LOCAL="${KEEP_LOCAL:-3}"
name="state-$(date -u +%Y%m%dT%H%M%SZ)"
compose=(docker compose -f infra/deploy/compose.yaml --env-file /etc/ignis/release.env)

"${compose[@]}" exec -T ignis ignis state backup \
  --state-dir /srv/ignis/state \
  --out "/srv/ignis/backups/$name" \
  --push --name "$name" --prefix "${IGNIS_BACKUP_PREFIX:-backups/}"

# Local retention only; the bucket keeps every snapshot (lifecycle rule there, see README).
mapfile -t old < <(find /srv/ignis/backups -mindepth 1 -maxdepth 1 -type d -name 'state-*' | sort | head -n "-$KEEP_LOCAL")
for d in "${old[@]}"; do rm -rf -- "$d"; done
echo "snapshot $name pushed"
