FROM --platform=$BUILDPLATFORM rust:bookworm AS builder
WORKDIR /app

ARG TARGETARCH
ARG SCCACHE_BUCKET
ARG SCCACHE_REGION
ARG SCCACHE_ENDPOINT
ARG SCCACHE_S3_USE_SSL=true
ARG SCCACHE_VERSION=0.14.0

ENV SCCACHE_BUCKET=${SCCACHE_BUCKET} \
    SCCACHE_REGION=${SCCACHE_REGION} \
    SCCACHE_ENDPOINT=${SCCACHE_ENDPOINT} \
    SCCACHE_S3_USE_SSL=${SCCACHE_S3_USE_SSL} \
    RUSTC_WRAPPER=sccache

RUN arch="$(uname -m)" && \
    case "$arch" in \
      aarch64|arm64) SCCACHE_ARCH=aarch64 ;; \
      x86_64) SCCACHE_ARCH=x86_64 ;; \
      *) echo "unsupported build arch: $arch" >&2; exit 1 ;; \
    esac && \
    curl -fsSL "https://github.com/mozilla/sccache/releases/download/v${SCCACHE_VERSION}/sccache-v${SCCACHE_VERSION}-${SCCACHE_ARCH}-unknown-linux-musl.tar.gz" \
      | tar -xz -C /tmp \
 && mv "/tmp/sccache-v${SCCACHE_VERSION}-${SCCACHE_ARCH}-unknown-linux-musl/sccache" /usr/local/bin/sccache \
 && chmod +x /usr/local/bin/sccache \
 && rm -rf "/tmp/sccache-v${SCCACHE_VERSION}-${SCCACHE_ARCH}-unknown-linux-musl"

RUN if [ "$TARGETARCH" = "arm64" ]; then \
      dpkg --add-architecture arm64 && \
      apt-get update && apt-get install -y gcc-aarch64-linux-gnu libc6-dev-arm64-cross && \
      rustup target add aarch64-unknown-linux-gnu; \
    fi

COPY . .

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=id=hq-target-${TARGETARCH},type=cache,target=/app/target \
    --mount=type=secret,id=aws_access_key_id \
    --mount=type=secret,id=aws_secret_access_key \
    AWS_ACCESS_KEY_ID="$(cat /run/secrets/aws_access_key_id)" \
    AWS_SECRET_ACCESS_KEY="$(cat /run/secrets/aws_secret_access_key)" \
    sh -c 'if [ "$TARGETARCH" = "arm64" ]; then \
      CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc \
      cargo build --release -p hq --target aarch64-unknown-linux-gnu && \
      cp target/aarch64-unknown-linux-gnu/release/hq /app/hq-bin; \
    else \
      cargo build --release -p hq && \
      cp target/release/hq /app/hq-bin; \
    fi && \
    sccache --show-stats'

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/hq-bin ./hq
EXPOSE 8080
CMD ["/bin/sh", "-c", "ulimit -n 100000 && ./hq"]
