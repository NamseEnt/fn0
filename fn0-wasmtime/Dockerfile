FROM rust:bookworm AS builder
WORKDIR /app

COPY fn0-wasmtime/ fn0-wasmtime/

RUN cargo build --release --manifest-path fn0-wasmtime/Cargo.toml

FROM scratch
COPY --from=builder /app/fn0-wasmtime/target/release/fn0-wasmtime /fn0-wasmtime
