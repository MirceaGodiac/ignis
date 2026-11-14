#!/bin/sh
# Export each Compose secret (/run/secrets/<NAME>, one value per file) as the environment variable
# <NAME>, then exec ignis. Secrets never sit in the image, the compose file or Git. The Codex
# child and the runner containers get no host environment (env_clear / no -e), so these values stay
# in this process.
set -eu
if [ -d /run/secrets ]; then
  for f in /run/secrets/*; do
    [ -f "$f" ] || continue
    name=$(basename "$f")
    case "$name" in
      *[!A-Z0-9_]*) echo "ignis-entrypoint: skipping secret with unexpected name: $name" >&2; continue ;;
    esac
    value=$(cat "$f")
    export "$name=$value"
  done
fi
exec /usr/local/bin/ignis "$@"
