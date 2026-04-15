FROM --platform=$BUILDPLATFORM rust:bookworm AS builder
WORKDIR /app

ARG TARGETARCH
ARG SCCACHE_VERSION=0.14.0
ARG SCCACHE_BUCKET
ARG SCCACHE_REGION
ARG SCCACHE_ENDPOINT
ARG SCCACHE_S3_USE_SSL=true
ARG AWS_ACCESS_KEY_ID
ARG AWS_SECRET_ACCESS_KEY

ENV SCCACHE_BUCKET=${SCCACHE_BUCKET} \
    SCCACHE_REGION=${SCCACHE_REGION} \
    SCCACHE_ENDPOINT=${SCCACHE_ENDPOINT} \
    SCCACHE_S3_USE_SSL=${SCCACHE_S3_USE_SSL} \
    AWS_ACCESS_KEY_ID=${AWS_ACCESS_KEY_ID} \
    AWS_SECRET_ACCESS_KEY=${AWS_SECRET_ACCESS_KEY} \
    RUSTC_WRAPPER=sccache

RUN curl -fsSL "https://github.com/mozilla/sccache/releases/download/v${SCCACHE_VERSION}/sccache-v${SCCACHE_VERSION}-aarch64-unknown-linux-musl.tar.gz" \
    | tar -xz -C /tmp \
 && mv "/tmp/sccache-v${SCCACHE_VERSION}-aarch64-unknown-linux-musl/sccache" /usr/local/bin/sccache \
 && chmod +x /usr/local/bin/sccache \
 && rm -rf "/tmp/sccache-v${SCCACHE_VERSION}-aarch64-unknown-linux-musl"

RUN if [ "$TARGETARCH" = "arm64" ]; then \
      dpkg --add-architecture arm64 && \
      apt-get update && apt-get install -y gcc-aarch64-linux-gnu libc6-dev-arm64-cross && \
      rustup target add aarch64-unknown-linux-gnu; \
    fi

COPY fn0-worker/Cargo.toml fn0-worker/Cargo.toml
COPY fn0-worker/Cargo.lock fn0-worker/Cargo.lock
RUN mkdir -p fn0-worker/src && echo "fn main(){}" > fn0-worker/src/main.rs && \
    if [ "$TARGETARCH" = "arm64" ]; then \
      CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc \
      cargo build --release --target aarch64-unknown-linux-gnu --manifest-path fn0-worker/Cargo.toml; \
    else \
      cargo build --release --manifest-path fn0-worker/Cargo.toml; \
    fi

COPY fn0-worker/src/ fn0-worker/src/
RUN touch fn0-worker/src/main.rs && \
    if [ "$TARGETARCH" = "arm64" ]; then \
      CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc \
      cargo build --release --target aarch64-unknown-linux-gnu --manifest-path fn0-worker/Cargo.toml && \
      cp fn0-worker/target/aarch64-unknown-linux-gnu/release/fn0-worker /app/fn0-worker-bin; \
    else \
      cargo build --release --manifest-path fn0-worker/Cargo.toml && \
      cp fn0-worker/target/release/fn0-worker /app/fn0-worker-bin; \
    fi \
 && sccache --show-stats

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/fn0-worker-bin ./fn0-worker
EXPOSE 443
CMD ["/bin/sh", "-c", "ulimit -n 100000 && ./fn0-worker"]
