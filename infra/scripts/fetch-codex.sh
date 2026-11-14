#!/usr/bin/env bash
# Fetch the pinned Linux x86_64 Codex binary for the server image, without installing Node.js on
# the host: npm runs in a throwaway container and only the native executable is kept.
#
#   infra/scripts/fetch-codex.sh <out-dir>      → <out-dir>/codex (prints its version and sha256)
#
# Same source as tools/codex/setup.sh (the @openai/codex npm package at the pinned version).
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd)"
out="${1:?usage: fetch-codex.sh <out-dir>}"
pin="$(grep -oE 'PINNED_CODEX_VERSION: &str = "[^"]+"' "$root"/crates/ignis-research/src/runtime/*.rs | head -1 | sed -E 's/.*"([^"]+)"/\1/')"
[ -n "$pin" ] || { echo "cannot read PINNED_CODEX_VERSION" >&2; exit 1; }
mkdir -p "$out"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
docker run --rm -v "$work:/w" -w /w node:22-bookworm-slim \
  sh -c "npm install --no-audit --no-fund --prefix /w @openai/codex@$pin >/dev/null && chown -R $(id -u):$(id -g) /w"
bin="$(find "$work/node_modules/@openai" -path '*/vendor/x86_64-unknown-linux-*/bin/codex' -type f | head -n 1)"
[ -n "$bin" ] || { echo "no Linux x86_64 codex binary in @openai/codex@$pin" >&2; exit 1; }
install -m 0755 "$bin" "$out/codex"
version="$("$out/codex" --version | awk 'NR==1 {print $NF}')"
[ "$version" = "$pin" ] || { echo "fetched codex $version, IGNIS pins $pin" >&2; exit 1; }
echo "codex $version $(sha256sum < "$out/codex" | awk '{print $1}') $out/codex"
