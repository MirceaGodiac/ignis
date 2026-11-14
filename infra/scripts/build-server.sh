#!/usr/bin/env bash
# Build the origin image from a clean checkout and write the release manifest.
#
#   infra/scripts/build-server.sh <codex-binary>      (Linux x86_64 codex, the pinned version)
#
# Checks the codex binary's version and sha256 against the pin, stages it into the build context,
# resolves every base image to a digest, builds, and writes runs/release/<git-sha>.json naming
# all of it (git commit, toolchain, codex, base digests, image id). Nothing is pushed.
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$root"
codex="${1:?usage: build-server.sh <codex-binary>}"
pin="$(grep -oE 'PINNED_CODEX_VERSION: &str = "[^"]+"' crates/ignis-research/src/runtime/*.rs | head -1 | sed -E 's/.*"([^"]+)"/\1/')"
[ -n "$pin" ] || { echo "cannot read PINNED_CODEX_VERSION" >&2; exit 1; }
version="$("$codex" --version | awk 'NR==1 {print $NF}')"
[ "$version" = "$pin" ] || { echo "codex is $version, IGNIS pins $pin" >&2; exit 1; }
if [ -n "$(git status --porcelain)" ]; then
  echo "working tree is not clean: a release is built from a commit" >&2
  exit 1
fi

mkdir -p infra/server/vendor
install -m 0755 "$codex" infra/server/vendor/codex
codex_sha="$(sha256sum < infra/server/vendor/codex | awk '{print $1}')"

digest() { docker buildx imagetools inspect "$1" --format '{{json .Manifest}}' | sed -E 's/.*"digest":"([^"]+)".*/\1/'; }
toolchain="$(sed -nE 's/^channel = "([^"]+)"/\1/p' rust-toolchain.toml)"
rust_ref="rust:${toolchain}-bookworm"
rust_img="rust@$(digest "$rust_ref")"
runtime_img="debian@$(digest debian:bookworm-slim)"
cli_img="docker@$(digest docker:29.7-cli)"

commit="$(git rev-parse HEAD)"
docker build -f infra/server/Dockerfile \
  --build-arg "RUST_IMAGE=$rust_img" \
  --build-arg "RUNTIME_IMAGE=$runtime_img" \
  --build-arg "DOCKER_CLI_IMAGE=$cli_img" \
  --label "org.opencontainers.image.revision=$commit" \
  -t "ignis-server:$commit" .
image_id="$(docker image inspect "ignis-server:$commit" --format '{{.Id}}')"

mkdir -p runs/release
out="runs/release/$commit.json"
cat > "$out" <<EOF
{
  "git_commit": "$commit",
  "rust_toolchain": "$toolchain",
  "codex_version": "$pin",
  "codex_sha256": "$codex_sha",
  "base_images": {"rust": "$rust_img", "runtime": "$runtime_img", "docker_cli": "$cli_img"},
  "image_id": "$image_id",
  "built_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF
echo "built ignis-server:$commit ($image_id); manifest $out"
