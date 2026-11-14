#!/usr/bin/env bash
# Start `ignis serve --dev` with the agent runtime.
#
#   tools/codex/serve.sh                  # real Codex from runs/codex/codex.env (run setup.sh first)
#   tools/codex/serve.sh --fake           # the offline fake app-server: no model, no credentials, no cost
#   tools/codex/serve.sh --use-app-login  # reuse the login of your own Codex/ChatGPT app (~/.codex)
#   tools/codex/serve.sh -- --port 9000   # anything after -- goes to `ignis serve`
#
# Real turns use your Codex account (tools/codex/login.sh, or --use-app-login) or
# CODEX_API_KEY / OPENAI_API_KEY.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
FAKE=0
APP_LOGIN=0
EXTRA=()
while [ $# -gt 0 ]; do
  case "$1" in
    --fake) FAKE=1 ;;
    --use-app-login) APP_LOGIN=1 ;;
    --) shift; EXTRA=("$@"); break ;;
    -h|--help) sed -n '2,10p' "$0"; exit 0 ;;
    *) echo "unknown argument: $1" >&2; exit 2 ;;
  esac
  shift
done

if [ "$FAKE" = 1 ]; then
  cargo build -q -p ignis-research --bin ignis-fake-codex
  BIN="${CARGO_TARGET_DIR:-$ROOT/target}/debug/ignis-fake-codex"
  [ -f "$BIN.exe" ] && BIN="$BIN.exe"
  export IGNIS_CODEX_BIN="$BIN"
  unset IGNIS_CODEX_SHA256
  STATE="runs/server-fake-codex"
else
  ENV_FILE="$ROOT/runs/codex/codex.env"
  [ -f "$ENV_FILE" ] || { echo "missing $ENV_FILE: run tools/codex/setup.sh first" >&2; exit 1; }
  # Parsed, not sourced: values are paths and hashes, never shell code. CR stripped for files
  # written on Windows.
  while IFS='=' read -r key value; do
    case "$key" in IGNIS_CODEX_BIN|IGNIS_CODEX_SHA256) export "$key=$value" ;; esac
  done < <(tr -d '\r' < "$ENV_FILE")
  STATE="runs/server"
  APP_HOME="${CODEX_HOME:-$HOME/.codex}"
  # Native Windows programs cannot read Git Bash paths (/c/...) from the environment.
  if command -v cygpath >/dev/null; then APP_HOME="$(cygpath -m "$APP_HOME")"; ROOT_NATIVE="$(cygpath -m "$ROOT")"; else ROOT_NATIVE="$ROOT"; fi
  if [ "$APP_LOGIN" = 1 ]; then
    # The login of your own Codex / ChatGPT app, used in place (never copied): its config.toml
    # applies too and the runtime's conversations appear in that app's history.
    [ -f "$APP_HOME/auth.json" ] || { echo "no Codex login in $APP_HOME" >&2; exit 1; }
    export IGNIS_CODEX_LOGIN_HOME="$APP_HOME"
    echo "login: your Codex app's login at $APP_HOME (shared, not copied)"
  elif [ -f "$ROOT/runs/codex/login/auth.json" ]; then
    # Signed in with tools/codex/login.sh (local development).
    export IGNIS_CODEX_LOGIN_HOME="$ROOT_NATIVE/runs/codex/login"
    echo "login: Codex account signed in at $IGNIS_CODEX_LOGIN_HOME"
  elif [ -z "${CODEX_API_KEY:-}${OPENAI_API_KEY:-}" ]; then
    echo "warning: not signed in (tools/codex/login.sh or --use-app-login) and no CODEX_API_KEY/OPENAI_API_KEY: real turns will fail to authenticate" >&2
  fi
fi

echo "codex: $IGNIS_CODEX_BIN"
exec cargo run -q -p ignis-cli -- serve --dev --state-dir "$STATE" ${EXTRA[@]+"${EXTRA[@]}"}
