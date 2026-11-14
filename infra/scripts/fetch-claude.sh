#!/usr/bin/env bash
# Fetch the Linux x86_64 Claude Code binary for the server image (IGNIS_AGENT_RUNTIME=claude),
# with Anthropic's installer run in a throwaway container: nothing is installed on the host.
#
#   infra/scripts/fetch-claude.sh <out-dir> [<version>]   → <out-dir>/claude
#
# Pass a version to pin it (recorded in the release manifest by build-server.sh).
set -euo pipefail
out="${1:?usage: fetch-claude.sh <out-dir> [<version>]}"
version="${2:-}"
mkdir -p "$out"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
docker run --rm -v "$work:/out" debian:bookworm-slim sh -c "
  set -e
  apt-get update -qq && apt-get install -y -qq --no-install-recommends curl ca-certificates >/dev/null
  curl -fsSL https://claude.ai/install.sh | bash -s $version >/dev/null
  cp -L /root/.local/bin/claude /out/claude
  chown $(id -u):$(id -g) /out/claude"
install -m 0755 "$work/claude" "$out/claude"
got="$("$out/claude" --version | awk 'NR==1 {print $1}')"
if [ -n "$version" ] && [ "$got" != "$version" ]; then
  echo "fetched claude $got, asked for $version" >&2
  exit 1
fi
echo "claude $got $(sha256sum < "$out/claude" | awk '{print $1}') $out/claude"
