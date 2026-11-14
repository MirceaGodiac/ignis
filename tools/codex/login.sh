#!/usr/bin/env bash
# Sign the IGNIS runtime in with your Codex (ChatGPT) account, once per machine.
# Local development only: deployments use an API key.
#
#   tools/codex/login.sh            # browser sign-in of `codex login`
#   tools/codex/login.sh --status   # only show whether it is signed in
#   tools/codex/login.sh --force    # sign in again (starting a login first clears the current one)
#
# The login lives in runs/codex/login (its own CODEX_HOME, ignored by git), not in your personal
# ~/.codex: IGNIS never shares or refreshes the login your own Codex uses. serve.sh picks it up.
# Run setup.sh first.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
ENV_FILE="$ROOT/runs/codex/codex.env"
[ -f "$ENV_FILE" ] || { echo "missing $ENV_FILE: run tools/codex/setup.sh first" >&2; exit 1; }
BIN="$(tr -d '\r' < "$ENV_FILE" | sed -n 's/^IGNIS_CODEX_BIN=//p' | head -n 1)"
LOGIN_HOME="$ROOT/runs/codex/login"
mkdir -p "$LOGIN_HOME"
export CODEX_HOME="$LOGIN_HOME"
# Keep the credentials a file in that home (not the OS keyring), as the runtime expects.
STORE='cli_auth_credentials_store="file"'
MODE="${1:-}"
if [ -z "$MODE" ] && [ -f "$LOGIN_HOME/auth.json" ]; then
  # `codex login` clears the existing login as soon as it starts; never do that by accident.
  echo "already signed in (use --force to sign in again)"
  MODE="--status"
fi
if [ "$MODE" != "--status" ]; then
  "$BIN" -c "$STORE" login
fi
"$BIN" -c "$STORE" login status || true
if [ -f "$LOGIN_HOME/auth.json" ]; then
  echo "signed in: tools/codex/serve.sh will use $LOGIN_HOME"
else
  echo "not signed in"
  exit 1
fi
