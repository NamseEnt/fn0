FROM --platform=$BUILDPLATFORM rust:bookworm AS builder
WORKDIR /app

ARG TARGETARCH

RUN if [ "$TARGETARCH" = "arm64" ]; then \
      dpkg --add-architecture arm64 && \
      apt-get update && apt-get install -y gcc-aarch64-linux-gnu libc6-dev-arm64-cross && \
      rustup target add aarch64-unknown-linux-gnu; \
    fi

COPY host-hq-protocol/Cargo.toml host-hq-protocol/Cargo.toml
COPY fn0-worker/Cargo.toml fn0-worker/Cargo.toml
COPY fn0-worker/Cargo.lock fn0-worker/Cargo.lock
RUN mkdir -p host-hq-protocol/src && echo "" > host-hq-protocol/src/lib.rs && \
    mkdir -p fn0-worker/src && echo "fn main(){}" > fn0-worker/src/main.rs && \
    if [ "$TARGETARCH" = "arm64" ]; then \
      CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc \
      cargo build --release --target aarch64-unknown-linux-gnu --manifest-path fn0-worker/Cargo.toml; \
    else \
      cargo build --release --manifest-path fn0-worker/Cargo.toml; \
    fi

COPY host-hq-protocol/src/ host-hq-protocol/src/
COPY fn0-worker/src/ fn0-worker/src/
RUN touch host-hq-protocol/src/lib.rs fn0-worker/src/main.rs && \
    if [ "$TARGETARCH" = "arm64" ]; then \
      CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc \
      cargo build --release --target aarch64-unknown-linux-gnu --manifest-path fn0-worker/Cargo.toml && \
      cp fn0-worker/target/aarch64-unknown-linux-gnu/release/fn0-worker /app/fn0-worker-bin; \
    else \
      cargo build --release --manifest-path fn0-worker/Cargo.toml && \
      cp fn0-worker/target/release/fn0-worker /app/fn0-worker-bin; \
    fi

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/fn0-worker-bin ./fn0-worker
EXPOSE 443 10000
CMD ["/bin/sh", "-c", "ulimit -n 100000 && ./fn0-worker"]
