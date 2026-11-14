#!/usr/bin/env bash
# Copy only what a pod job needs, and bring results back with sha256 verification.
#
#   tools/runpod/sync.sh push <ssh-target> <path> [<path> ...]   # send views, clips, splits, calibrations
#   tools/runpod/sync.sh pull <ssh-target>                       # fetch and verify results; exit 1 on mismatch
#
# <ssh-target> is `user@host`; `-p`/`-i` go in ~/.ssh/config or SSH_OPTS="-p 12345 -i ~/.ssh/key".
# Set REMOTE_DIR to the repository on the pod's persistent volume.
# Run from the repository root on your machine. Never creates, stops or deletes a pod.
set -euo pipefail

MODE="${1:-}"
TARGET="${2:-}"
[ -n "$MODE" ] && [ -n "$TARGET" ] || { echo "usage: $0 push|pull <ssh-target> [paths...]" >&2; exit 64; }
shift 2
: "${REMOTE_DIR:?Set REMOTE_DIR to the absolute remote checkout path}"
SSH_OPTS="${SSH_OPTS:-}"
# Directories fetched by `pull`: releases and vision contours, the job ledger, evaluation records.
PULL_PATHS=(artifacts/models artifacts/vision artifacts/analysis artifacts/edits runs/jobs runs/evaluation)

have_rsync() { command -v rsync >/dev/null; }

copy_to_pod() {
  local rel="$1"
  ssh $SSH_OPTS "$TARGET" "mkdir -p \"$REMOTE_DIR/$(dirname "$rel")\""
  if have_rsync; then
    rsync -a --partial --protect-args -e "ssh $SSH_OPTS" "$rel" "$TARGET:$REMOTE_DIR/$(dirname "$rel")/"
  else
    scp -r $SSH_OPTS "$rel" "$TARGET:$REMOTE_DIR/$(dirname "$rel")/"
  fi
}

copy_from_pod() {
  local rel="$1"
  mkdir -p "$(dirname "$rel")"
  if have_rsync; then
    rsync -a --partial --protect-args -e "ssh $SSH_OPTS" "$TARGET:$REMOTE_DIR/$rel/" "$rel/"
  else
    scp -r $SSH_OPTS "$TARGET:$REMOTE_DIR/$rel" "$(dirname "$rel")/"
  fi
}

case "$MODE" in
  push)
    [ "$#" -gt 0 ] || { echo "push needs at least one path (data/work/views/<cohort>, a clip, ...)" >&2; exit 64; }
    for p in "$@"; do
      [ -e "$p" ] || { echo "no such path: $p" >&2; exit 66; }
      echo "== push $p"
      copy_to_pod "$p"
    done
    ;;
  pull)
    tmp_remote="$(mktemp)"; tmp_local="$(mktemp)"
    trap 'rm -f "$tmp_remote" "$tmp_local"' EXIT
    present=()
    for p in "${PULL_PATHS[@]}"; do
      if ssh $SSH_OPTS "$TARGET" "test -d \"$REMOTE_DIR/$p\""; then
        echo "== pull $p"
        copy_from_pod "$p"
        present+=("$p")
      fi
    done
    [ "${#present[@]}" -gt 0 ] || { echo "nothing to pull from $REMOTE_DIR"; exit 0; }
    echo "== sha256 (pod)"
    ssh $SSH_OPTS "$TARGET" "cd \"$REMOTE_DIR\" && find ${present[*]} -type f -print0 | sort -z | xargs -0 sha256sum" | sed 's/  /\t/' | sort -k2 >"$tmp_remote"
    echo "== sha256 (local)"
    find "${present[@]}" -type f -print0 | sort -z | xargs -0 sha256sum | sed 's/  /\t/' | sort -k2 >"$tmp_local"
    if ! diff -q "$tmp_remote" "$tmp_local" >/dev/null; then
      diff "$tmp_remote" "$tmp_local" | head -20
      echo "MISMATCH between the pod and this copy: DO NOT stop or terminate the pod." >&2
      exit 1
    fi
    echo "verified: $(wc -l <"$tmp_local") files match by sha256. Safe to stop the pod from the Runpod console."
    ;;
  *) echo "unknown mode: $MODE (push|pull)" >&2; exit 64 ;;
esac
