#!/usr/bin/env bash
# Restore a state snapshot from R2 onto an EMPTY /srv/ignis/state (new host, lost disk, drill).
#
#   infra/scripts/restore.sh <snapshot-name>     e.g. state-20261113T180000Z
#
# The service must be stopped. The restore refuses a non-empty target, verifies every file's
# sha256 against the snapshot manifest and runs PRAGMA integrity_check on each database.
set -euo pipefail
cd "$(dirname "$0")/../.."
name="${1:?usage: restore.sh <snapshot-name>}"
if systemctl is-active --quiet ignis.service; then
  echo "ignis.service is running: stop it first (systemctl stop ignis)" >&2
  exit 1
fi
if [ -n "$(ls -A /srv/ignis/state 2>/dev/null)" ]; then
  echo "/srv/ignis/state is not empty; restore only onto an empty volume" >&2
  exit 1
fi
docker compose -f infra/deploy/compose.yaml --env-file /etc/ignis/release.env \
  run --rm --no-deps ignis state restore \
  --pull "$name" --prefix "${IGNIS_BACKUP_PREFIX:-backups/}" \
  --download-to "/srv/ignis/backups/restore-$name" \
  --target /srv/ignis/state
echo "restored $name; start the service with: systemctl start ignis"
