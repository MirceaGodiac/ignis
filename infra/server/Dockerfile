# IGNIS origin image: `ignis serve` (API, SSE, jobs, MCP on loopback, Codex bridge, runner client).
#
# Build with infra/scripts/build-server.sh, which stages the pinned Codex binary (verified by
# sha256) into infra/server/vendor/ and records the resolved base-image digests and the image id in
# the release manifest. Build context: the repository root (see /.dockerignore).
#
# Base images are passed by digest in releases (`--build-arg RUST_IMAGE=rust@sha256:…`); the tags
# below are the human-readable defaults for a local build.
ARG RUST_IMAGE=rust:1.99.1-bookworm
ARG RUNTIME_IMAGE=debian:bookworm-slim
ARG DOCKER_CLI_IMAGE=docker:29.7-cli

FROM ${DOCKER_CLI_IMAGE} AS dockercli

FROM ${RUST_IMAGE} AS build
# mupdf-sys (bindgen → libclang), openh264 (nasm).
RUN apt-get update \
 && apt-get install -y --no-install-recommends clang libclang-dev nasm cmake pkg-config \
 && rm -rf /var/lib/apt/lists/*
WORKDIR /src
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target \
    cargo build --release --locked -p ignis-cli --bin ignis \
 && install -D -m 0755 target/release/ignis /out/ignis

FROM ${RUNTIME_IMAGE}
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates curl tini sqlite3 \
 && rm -rf /var/lib/apt/lists/* \
 && groupadd --system --gid 10001 ignis \
 && useradd --system --uid 10001 --gid ignis --home-dir /srv/ignis --shell /usr/sbin/nologin ignis
# The engine CLI only: the runner containers are started through the host engine's socket, which
# compose mounts (see infra/README.md §Runner for what that grants).
COPY --from=dockercli /usr/local/bin/docker /usr/local/bin/docker
COPY infra/server/vendor/codex /usr/local/bin/codex
COPY --from=build /out/ignis /usr/local/bin/ignis
# Read-only application root: configs, schemas, manifests and the C0 contract fixtures (dev only).
COPY configs /opt/ignis/configs
COPY schemas /opt/ignis/schemas
COPY fixtures/contracts /opt/ignis/fixtures/contracts
COPY data/manifests /opt/ignis/data/manifests
# Frozen splits: the evidence lockbox (which passages a scope may read) refuses to start without
# them. Corpus manifest/snapshot and builder bundle documents are the committed references.
COPY data/splits /opt/ignis/data/splits
COPY data/corpus /opt/ignis/data/corpus
COPY data/datasets /opt/ignis/data/datasets
COPY infra/server/entrypoint.sh /usr/local/bin/ignis-entrypoint
RUN chmod 0755 /usr/local/bin/ignis-entrypoint /usr/local/bin/codex

USER 10001:10001
WORKDIR /opt/ignis
ENV IGNIS_ROOT=/opt/ignis \
    IGNIS_STATE_DIR=/srv/ignis/state \
    IGNIS_CODEX_BIN=/usr/local/bin/codex \
    RUST_LOG=info,tower_http=info
EXPOSE 8787
ENTRYPOINT ["/usr/bin/tini", "--", "/usr/local/bin/ignis-entrypoint"]
CMD ["serve", "--host", "0.0.0.0"]
