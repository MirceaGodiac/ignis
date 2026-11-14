#!/usr/bin/env bash
# Send one message through a running IGNIS server and follow the episode until the turn ends.
#
#   tools/codex/smoke.sh                                  # http://127.0.0.1:8787/api/v1
#   tools/codex/smoke.sh --base http://127.0.0.1:8788/api/v1 --text "tool approval" --approve
#   tools/codex/smoke.sh --spend-tokens                   # required when the runtime is the real Codex
#
# Approvals the runtime asks for are resolved with --approve or --deny (default: deny).
# Needs curl. Exit code 0 when the episode reached `completed`.
set -euo pipefail

BASE="http://127.0.0.1:8787/api/v1"
TEXT="Reply with one short sentence: what is IGNIS?"
DECISION="denied"
SPEND=0
TIMEOUT=180
while [ $# -gt 0 ]; do
  case "$1" in
    --base) BASE="$2"; shift ;;
    --text) TEXT="$2"; shift ;;
    --approve) DECISION="approved" ;;
    --deny) DECISION="denied" ;;
    --spend-tokens) SPEND=1 ;;
    --timeout) TIMEOUT="$2"; shift ;;
    -h|--help) sed -n '2,10p' "$0"; exit 0 ;;
    *) echo "unknown argument: $1" >&2; exit 2 ;;
  esac
  shift
done

# The FIRST occurrence of "key":"value" (top-level keys come before nested ones, e.g. an error's
# retry.kind), without a greedy prefix that would pick the last one.
field() { grep -o "\"$1\":\"[^\"]*\"" | head -n 1 | sed 's/^"[^"]*":"//; s/"$//'; }
json_str() { printf '%s' "$1" | sed 's/\\/\\\\/g; s/"/\\"/g'; }

CAPS="$(curl -fsS "$BASE/capabilities")"
case "$CAPS" in
  *'"agent_runtime":{"available":true'*) ;;
  *) echo "agent runtime unavailable: $(printf '%s' "$CAPS" | sed -n 's/.*"agent_runtime":{[^}]*"detail":"\([^"]*\)".*/\1/p')" >&2; exit 1 ;;
esac
UA="$(printf '%s' "$CAPS" | field user_agent)"
echo "runtime: $(printf '%s' "$CAPS" | field runtime_build_id)  ($UA)"
case "$UA" in
  ignis-fake-codex*) ;;
  *) if [ "$SPEND" != 1 ]; then
       echo "this server runs the REAL Codex: a message calls the model and spends tokens. Re-run with --spend-tokens." >&2
       exit 3
     fi ;;
esac

RUN="smoke-$(date +%s)-$$"
EP="$(curl -fsS -X POST "$BASE/episodes" -H 'content-type: application/json' \
  -d "{\"goal\":\"Smoke test $RUN\",\"client_request_id\":\"$RUN-ep\"}" | field episode_id)"
echo "episode: $EP"
curl -fsS -X POST "$BASE/episodes/$EP/messages" -H 'content-type: application/json' \
  -d "{\"text\":\"$(json_str "$TEXT")\",\"client_request_id\":\"$RUN-msg\"}" >/dev/null
echo "sent:    $TEXT"

CURSOR=0
STATUS=""
START=$(date +%s)
while [ $(( $(date +%s) - START )) -lt "$TIMEOUT" ]; do
  # Read whatever is in the log after the cursor (the stream stays open; cut it after 2 s).
  CHUNK="$(curl -s -N --max-time 2 "$BASE/episodes/$EP/events?after=$CURSOR" 2>/dev/null || true)"
  while IFS= read -r line; do
    case "$line" in
      id:*) CURSOR="${line#id:}"; CURSOR="${CURSOR// /}" ;;
      data:*)
        DATA="${line#data:}"
        KIND="$(printf '%s' "$DATA" | field kind)"
        case "$KIND" in
          message.delta) ;;
          message.user|message.completed) echo "  [$CURSOR] $KIND: $(printf '%s' "$DATA" | field text)" ;;
          episode.status_changed)
            STATUS="$(printf '%s' "$DATA" | sed -n 's/.*"to":"\([^"]*\)".*/\1/p')"
            echo "  [$CURSOR] status -> $STATUS  $(printf '%s' "$DATA" | field reason)" ;;
          approval.requested)
            APR="$(printf '%s' "$DATA" | field approval_id)"
            echo "  [$CURSOR] approval requested: $(printf '%s' "$DATA" | field action) -> $DECISION"
            curl -fsS -X POST "$BASE/episodes/$EP/approvals/$APR" -H 'content-type: application/json' \
              -d "{\"decision\":\"$DECISION\"}" >/dev/null ;;
          tool.started) echo "  [$CURSOR] tool: $(printf '%s' "$DATA" | field tool)" ;;
          *) echo "  [$CURSOR] $KIND" ;;
        esac ;;
    esac
  done <<< "$CHUNK"
  case "$STATUS" in
    completed) echo "OK: the turn completed"; exit 0 ;;
    failed|cancelled|waiting_user) echo "turn ended as $STATUS" >&2; exit 1 ;;
  esac
done
echo "timed out after ${TIMEOUT}s (last status: ${STATUS:-none})" >&2
exit 1
