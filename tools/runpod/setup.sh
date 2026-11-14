#!/usr/bin/env bash
# Prepare a fresh Linux GPU pod: Rust, the `ignis` release binary, the SAM2 sidecar and its checkpoint.
# Run from the repository root, on the pod's persistent volume. Touches only this machine: it never
# creates, stops or deletes a pod. Safe to re-run.
set -euo pipefail

# Keep toolchains on the persistent volume when there is one, so a pod restart does not lose them.
if [ -d /workspace ]; then
  export CARGO_HOME="${CARGO_HOME:-/workspace/.cargo}"
  export RUSTUP_HOME="${RUSTUP_HOME:-/workspace/.rustup}"
fi
export PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$HOME/.local/bin:$PATH"

CHECKPOINT="data/models/sam2/sam2.1_hiera_small.pt"
CHECKPOINT_URL="https://dl.fbaipublicfiles.com/segment_anything_2/092824/sam2.1_hiera_small.pt"

echo "== system packages"
if command -v apt-get >/dev/null; then
  if [ "$(id -u)" = 0 ]; then SUDO=""; else SUDO="sudo"; fi
  $SUDO apt-get update -y
  $SUDO apt-get install -y build-essential pkg-config curl git rsync ca-certificates
fi

echo "== rust"
if ! command -v cargo >/dev/null; then
  curl -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain "${RUST_VERSION:-stable}"
fi
cargo --version

echo "== build ignis (release)"
cargo build --release -p ignis-cli

echo "== sam2 sidecar"
if ! command -v uv >/dev/null; then
  curl -LsSf https://astral.sh/uv/install.sh | sh
fi
uv sync --project sidecar/sam2

echo "== sam2 checkpoint"
if [ ! -s "$CHECKPOINT" ]; then
  mkdir -p "$(dirname "$CHECKPOINT")"
  curl -fL -o "$CHECKPOINT" "$CHECKPOINT_URL"
fi
sha256sum "$CHECKPOINT"

echo
if command -v nvidia-smi >/dev/null; then nvidia-smi -L; else echo "no nvidia-smi: this machine has no visible GPU"; fi
./target/release/ignis --version
echo "Set device = \"cuda\" in configs/vision/sam2.toml before a GPU run, and dry-run first:"
echo "  ./target/release/ignis vision contours <clip> --calibration <spec> --method sam2 --profile runpod-3090"
