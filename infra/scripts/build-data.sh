#!/usr/bin/env bash
# Build the data the server reads, on the origin host, with the release image itself: fetch the
# bounded NASA sample set and the evidence-corpus reports (public NASA servers, ~0.3 GB), parse
# and index them, ingest the records and build the FLEX/ACME builder dataset bundles.
#
#   sudo infra/scripts/build-data.sh          (reads IGNIS_IMAGE from /etc/ignis/release.env)
#
# Writes only under /srv/ignis: raw/ (downloads, kept), data/ (records, dataset bundles; mounted
# read-only into the server as data/work), artifacts/evidence/ (the catalog), build-data/ (a
# writable copy of the committed manifests/splits/corpus/review the commands read and refresh).
# Builder bundles never contain the test partition. Re-running is safe: downloads are skipped when
# the hash matches, bundles are rewritten only when their content changes. Restart the server after.
set -euo pipefail
release="${IGNIS_RELEASE_ENV:-/etc/ignis/release.env}"
image="${IGNIS_IMAGE:-$(sed -n 's/^IGNIS_IMAGE=//p' "$release")}"
[ -n "$image" ] || { echo "IGNIS_IMAGE not set (and not in $release)" >&2; exit 1; }
base=/srv/ignis
build="$base/build-data"

mkdir -p "$base/raw" "$base/data" "$base/artifacts" "$base/state/runs" "$build"
# Fresh copy of the committed inputs from the image (inspect rewrites the listing manifests).
rm -rf "${build:?}/"*
docker run --rm --entrypoint tar "$image" -C /opt/ignis/data -cf - manifests splits corpus datasets review \
  | tar -C "$build" -xf -
chown -R 10001:10001 "$base/raw" "$base/data" "$base/artifacts" "$base/state/runs" "$build"

run() {
  echo; echo "=== ignis $*"
  docker run --rm --user 10001:10001 --read-only --tmpfs /tmp:size=1g \
    --cap-drop ALL --security-opt no-new-privileges:true \
    -e IGNIS_DATA=/data -e IGNIS_RUNS=/srv/ignis/state/runs -e HOME=/tmp \
    -v "$build:/data" -v "$base/raw:/data/raw" -v "$base/data:/data/work" \
    -v "$base/artifacts:/opt/ignis/artifacts" -v "$base/state/runs:/srv/ignis/state/runs" \
    --entrypoint /usr/local/bin/ignis "$image" "$@"
}

run sources inspect PSI-69 PSI-159 PSI-25 PSI-26
run sources samples
run evidence corpus fetch --execute
run evidence corpus status
run evidence corpus ingest
# R7a: parse views of the stored corpus (data/raw/samples) made by the sidecars (sidecar/pdfium for
# every PDF, sidecar/docling where structure or OCR is needed) under data/views/<parser>-<version>/<inv>/.
# Incremental; a Docling view ends active over the PDFium one; refused views are listed, not fatal.
views=()
for d in "$base"/data/views/*/; do [ -d "$d" ] && views+=("/data/work/views/$(basename "$d")"); done
if [ ${#views[@]} -gt 0 ] && [ -d "$base/data/raw/samples" ]; then
  run evidence import-views "${views[@]}" --pdf-root /data/work/raw/samples
fi
run evidence verify
run evidence corpus snapshot
run data ingest
for family in flex acme_normal acme_inverse; do
  run data bundle "$family"
  run data bundle "$family" --partitions train
done
run data bundles
run evidence corpus lockbox

# A changed listing means NASA published a new version: report it, never accept it silently.
echo
echo "=== listing changes against the release (review before the next release; retrieved_at is expected):"
docker run --rm --entrypoint tar "$image" -C /opt/ignis/data -cf - manifests \
  | (d="$(mktemp -d)"; tar -C "$d" -xf -; diff -rq "$d/manifests" "$build/manifests" || true; rm -rf "$d")
echo
echo "done: restart the server (systemctl restart ignis) and check /api/v1/capabilities"
