#!/usr/bin/env bash
# Run a diagnostic on ignis-origin through IAP. Needs the local IPv4 CONNECT proxy on 127.0.0.1:8899
# (the laptop's IPv6 is dead; see docs/implementation/ACCESS.md). Usage:
#   tools/diag/vm.sh state [episode_id]   # episode/turn/tool-call health from state.sqlite
#   tools/diag/vm.sh logs [since]         # server warnings/errors, e.g. "2 hours ago"
#   tools/diag/vm.sh sh '<command>'
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
export CLOUDSDK_PROXY_TYPE=http CLOUDSDK_PROXY_ADDRESS=127.0.0.1 CLOUDSDK_PROXY_PORT=8899
ssh() { gcloud compute ssh ignis-origin --account=ignisardealu@gmail.com --project=ignis-spaceapps-ardealu --zone=europe-west3-a --tunnel-through-iap --command="$1"; }
case "${1:-state}" in
  state) ssh "sudo python3 - ${2:-}" < "$here/vm_state.py" ;;
  logs) ssh "sudo docker logs --since '${2:-3 hours ago}' ignis-ignis-1 2>&1 | grep -E 'WARN|ERROR' | cut -c1-400 | tail -60" ;;
  sh) ssh "$2" ;;
esac
