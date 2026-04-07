FROM --platform=linux/arm64 rust:bookworm AS builder
WORKDIR /app

RUN dpkg --add-architecture arm64 && \
    apt-get update && apt-get install -y gcc-aarch64-linux-gnu libc6-dev-arm64-cross && \
    rustup target add aarch64-unknown-linux-gnu

COPY fn0-wasmtime/ fn0-wasmtime/

RUN CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc \
    cargo build --release --target aarch64-unknown-linux-gnu --manifest-path fn0-wasmtime/Cargo.toml

FROM scratch
COPY --from=builder /app/fn0-wasmtime/target/aarch64-unknown-linux-gnu/release/fn0-wasmtime /fn0-wasmtime
