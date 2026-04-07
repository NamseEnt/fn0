FROM --platform=$BUILDPLATFORM rust:bookworm AS builder
WORKDIR /app

ARG TARGETARCH

RUN if [ "$TARGETARCH" = "arm64" ]; then \
      dpkg --add-architecture arm64 && \
      apt-get update && apt-get install -y gcc-aarch64-linux-gnu libc6-dev-arm64-cross && \
      rustup target add aarch64-unknown-linux-gnu; \
    fi

COPY fn0-wasmtime/Cargo.toml fn0-wasmtime/Cargo.toml
COPY fn0-wasmtime/Cargo.lock fn0-wasmtime/Cargo.lock
RUN mkdir -p fn0-wasmtime/src && \
    echo "" > fn0-wasmtime/src/lib.rs && \
    echo "fn main(){}" > fn0-wasmtime/src/main.rs && \
    if [ "$TARGETARCH" = "arm64" ]; then \
      CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc \
      cargo build --release --target aarch64-unknown-linux-gnu --manifest-path fn0-wasmtime/Cargo.toml; \
    else \
      cargo build --release --manifest-path fn0-wasmtime/Cargo.toml; \
    fi

COPY fn0-wasmtime/src/ fn0-wasmtime/src/
RUN touch fn0-wasmtime/src/lib.rs fn0-wasmtime/src/main.rs && \
    if [ "$TARGETARCH" = "arm64" ]; then \
      CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc \
      cargo build --release --target aarch64-unknown-linux-gnu --manifest-path fn0-wasmtime/Cargo.toml && \
      cp fn0-wasmtime/target/aarch64-unknown-linux-gnu/release/fn0-wasmtime /app/fn0-wasmtime-bin; \
    else \
      cargo build --release --manifest-path fn0-wasmtime/Cargo.toml && \
      cp fn0-wasmtime/target/release/fn0-wasmtime /app/fn0-wasmtime-bin; \
    fi

FROM scratch
COPY --from=builder /app/fn0-wasmtime-bin /fn0-wasmtime
