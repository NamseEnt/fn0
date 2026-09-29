# Build the release binary against a pinned toolchain, then ship it on a base
# with nothing but the CA bundle it needs to reach an S3 endpoint over TLS.
FROM rust:1-bookworm AS build
WORKDIR /src

# Dependencies first, so editing our own sources does not re-resolve or
# re-download the tree. `rust-toolchain.toml` is copied here rather than beside
# `src` so the toolchain download happens once, in the cached layer: without it
# in the image at all, `FROM rust:1-bookworm` decides the compiler and the
# shipped binary is built by a toolchain no check in CI ever ran.
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
# `Cargo.toml` declares six `[[bench]]` targets. Cargo refuses to *parse* a
# manifest whose declared targets are missing, so without this the image build
# fails before it compiles anything — which it has done since the benches
# landed. Nothing here builds them; they only have to exist.
COPY benches ./benches
# The stub needs a `src/lib.rs` as well as a `src/main.rs`: `Cargo.toml`
# declares a `[lib]` target and the binary depends on it, so a stub with only
# `main.rs` fails this layer outright rather than merely skipping the cache.
RUN mkdir -p src && echo 'fn main() {}' > src/main.rs && : > src/lib.rs \
    && cargo build --release --bin signy \
    && rm -rf src

COPY src ./src
# `build_info` and the buildinfo endpoint read this. Without it a scraped
# series cannot be attributed to code, which is the first question asked when
# two deployments behave differently.
ARG SIGNY_BUILD_REVISION=unknown
ARG SIGNY_BUILD_BRANCH=unknown
ENV SIGNY_BUILD_REVISION=${SIGNY_BUILD_REVISION}
ENV SIGNY_BUILD_BRANCH=${SIGNY_BUILD_BRANCH}
# Touch so the dependency-cache layer above does not make cargo skip our code.
# Both targets, not just the binary: the cache layer compiled an empty
# `src/lib.rs`, and touching only `main.rs` leaves that fingerprint valid, so
# the build fails on a library that appears to have no contents.
RUN touch src/lib.rs src/main.rs && cargo build --release --bin signy

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# The data directory holds the WAL, and until a flush lands the WAL is the only
# copy of acknowledged data. The container is disposable and this directory is
# not: it must be a bind mount or a named volume that outlives every `docker rm`
# and follows the workload if it ever moves. See docs/RUNBOOK.md.
# The uid and gid are pinned because a bind mount carries the host's numbers
# through untranslated, so docs/DEPLOYMENT.md has to name a number to chown to.
RUN groupadd --system --gid 10001 signy \
    && useradd --system --create-home --uid 10001 --gid 10001 signy \
    && mkdir -p /var/lib/signy \
    && chown signy:signy /var/lib/signy
USER signy
WORKDIR /var/lib/signy
VOLUME ["/var/lib/signy"]
ENV SIGNY_DATA_DIR=/var/lib/signy

COPY --from=build /src/target/release/signy /usr/local/bin/signy

# The binary binds loopback by default, so that not configuring an address
# cannot silently expose a listener with no TLS and no authentication. An image
# exists to be reached, so it makes that decision explicitly here — which also
# keeps the decision visible to anyone reading the Dockerfile.
# An image's logs are read by a collector, not by a person at a terminal.
ENV SIGNY_LOG_FORMAT=json
ENV SIGNY_LISTEN_ADDR=0.0.0.0:3100
EXPOSE 3100
# No TLS in this process by design; put it behind a proxy inside a trust
# boundary. See docs/ARCHITECTURE.md, "Transport security".
ENTRYPOINT ["/usr/local/bin/signy"]
