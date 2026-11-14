#!/usr/bin/env bash
# Set up the pinned Codex App Server for IGNIS on this machine (Linux, macOS, Git Bash on Windows).
#
#   tools/codex/setup.sh            # find an existing codex install of the pinned version
#   tools/codex/setup.sh --install  # install it with npm into runs/codex/npm first (downloads ~100 MB)
#
# Writes runs/codex/codex.env with IGNIS_CODEX_BIN (the native binary, not the npm shim) and
# IGNIS_CODEX_SHA256. tools/codex/serve.sh reads it. Nothing here calls a model.
set -euo pipefail

PIN="0.169.0"   # keep equal to ignis_research::runtime::PINNED_CODEX_VERSION
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUT="$ROOT/runs/codex"
NPM_DIR="$OUT/npm"
INSTALL=0
for a in "$@"; do
  case "$a" in
    --install) INSTALL=1 ;;
    -h|--help) sed -n '2,10p' "$0"; exit 0 ;;
    *) echo "unknown argument: $a" >&2; exit 2 ;;
  esac
done
mkdir -p "$OUT"

if [ "$INSTALL" = 1 ]; then
  command -v npm >/dev/null || { echo "npm is required for --install (Node.js >= 16)" >&2; exit 1; }
  echo "installing @openai/codex@$PIN into $NPM_DIR"
  npm install --prefix "$NPM_DIR" --no-audit --no-fund "@openai/codex@$PIN" >/dev/null
fi

# The npm package ships the real executable at <platform package>/vendor/<target triple>/bin/codex.
find_native() {
  local root="$1"
  [ -d "$root/@openai" ] || return 1
  find "$root/@openai" -path '*/vendor/*/bin/*' \( -name codex -o -name codex.exe \) -type f 2>/dev/null | head -n 1
}

BIN="${IGNIS_CODEX_BIN:-}"
if [ -z "$BIN" ]; then BIN="$(find_native "$NPM_DIR/node_modules" || true)"; fi
if [ -z "$BIN" ] && command -v npm >/dev/null; then BIN="$(find_native "$(npm root -g)" || true)"; fi
if [ -z "$BIN" ] || [ ! -f "$BIN" ]; then
  echo "no codex $PIN binary found. Run: tools/codex/setup.sh --install  (or set IGNIS_CODEX_BIN)" >&2
  exit 1
fi

VERSION="$("$BIN" --version | awk 'NR==1 {print $NF}')"
if [ "$VERSION" != "$PIN" ]; then
  echo "$BIN is codex $VERSION, IGNIS is pinned to $PIN. Run: tools/codex/setup.sh --install" >&2
  exit 1
fi

# Forward slashes on Windows (Git Bash), so the env file reads the same from bash and PowerShell.
if command -v cygpath >/dev/null; then BIN="$(cygpath -m "$BIN")"; fi
# Hash from stdin: sha256sum escapes paths with backslashes in its output.
if command -v sha256sum >/dev/null; then SHA="$(sha256sum < "$BIN" | awk '{print $1}')";
else SHA="$(shasum -a 256 < "$BIN" | awk '{print $1}')"; fi

cat > "$OUT/codex.env" <<EOF
# Written by tools/codex/setup.sh. Machine-specific; not committed (runs/ is ignored).
IGNIS_CODEX_BIN=$BIN
IGNIS_CODEX_SHA256=$SHA
EOF

echo "codex $VERSION: $BIN"
echo "sha256:   $SHA"
echo "wrote     $OUT/codex.env"
if [ -n "${CODEX_API_KEY:-}${OPENAI_API_KEY:-}" ]; then
  echo "credentials: an API key is set in this shell (forwarded to the runtime)"
else
  echo "credentials: none set. Real turns need CODEX_API_KEY (or OPENAI_API_KEY) in the server's environment;"
  echo "             the fake runtime (tools/codex/serve.sh --fake) needs none and costs nothing."
fi
