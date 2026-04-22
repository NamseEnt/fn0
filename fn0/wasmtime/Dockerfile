FROM --platform=$BUILDPLATFORM rust:bookworm AS builder
WORKDIR /app

ARG TARGETARCH

RUN if [ "$TARGETARCH" = "arm64" ]; then \
      dpkg --add-architecture arm64 && \
      apt-get update && apt-get install -y gcc-aarch64-linux-gnu libc6-dev-arm64-cross && \
      rustup target add aarch64-unknown-linux-gnu; \
    fi

COPY . .

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=id=fn0-target-${TARGETARCH},type=cache,target=/app/target \
    if [ "$TARGETARCH" = "arm64" ]; then \
      CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc \
      cargo build --release -p fn0-wasmtime --target aarch64-unknown-linux-gnu && \
      cp target/aarch64-unknown-linux-gnu/release/fn0-wasmtime /app/fn0-wasmtime-bin; \
    else \
      cargo build --release -p fn0-wasmtime && \
      cp target/release/fn0-wasmtime /app/fn0-wasmtime-bin; \
    fi

FROM scratch
COPY --from=builder /app/fn0-wasmtime-bin /fn0-wasmtime
