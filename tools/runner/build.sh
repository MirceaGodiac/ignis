#!/usr/bin/env bash
# Build the analysis runner image and print the id to pin (IGNIS_RUNNER_IMAGE).
# Usage: tools/runner/build.sh [docker-binary]
set -euo pipefail
docker_bin="${1:-docker}"
here="$(cd "$(dirname "$0")/../.." && pwd)"
"$docker_bin" build --pull=false -t ignis-runner:local "$here/infra/runner" >&2
id="$("$docker_bin" image inspect ignis-runner:local --format '{{.Id}}')"
echo "IGNIS_RUNNER_IMAGE=$id"
