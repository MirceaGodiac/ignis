#!/usr/bin/env bash
# On the origin VM: confirm gVisor is installed as a Docker runtime and that the runner's
# isolation suite passes under it (M3 verified it only under runc).
#
#   infra/scripts/verify-gvisor.sh <runner-image-id>
#
# Needs a Rust toolchain on the host (or run from a checkout on the same machine). Prints the
# commands it runs; record their output in the release notes.
set -euo pipefail
cd "$(dirname "$0")/../.."
image="${1:?usage: verify-gvisor.sh <runner-image-id>}"
# One test at a time: the cancel test checks that no runner container is left, which a
# concurrent test's container would falsely trip (gVisor runs are slower, so they overlap).
set -x
runsc --version
docker info --format '{{json .Runtimes}}' | grep -q '"runsc"' || { echo "runsc is not a registered Docker runtime" >&2; exit 1; }
# gVisor answers dmesg with its own boot messages: proof the container runs on the sentry.
docker run --rm --runtime runsc --network none "$image" sh -c 'dmesg 2>/dev/null | head -n 3 || true'
IGNIS_RUNNER_IMAGE="$image" IGNIS_RUNNER_RUNTIME=runsc \
  cargo test -p ignis-research --test sandbox_live -- --ignored --nocapture --test-threads=1
IGNIS_RUNNER_IMAGE="$image" IGNIS_RUNNER_RUNTIME=runsc \
  cargo test -p ignis-mcp --test analysis_live -- --ignored --nocapture --test-threads=1
