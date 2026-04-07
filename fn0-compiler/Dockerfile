FROM rust:bookworm AS builder
WORKDIR /app

COPY fn0/ fn0/
COPY fn0-compiler/ fn0-compiler/

RUN cargo build --release --manifest-path fn0-compiler/Cargo.toml

FROM scratch
COPY --from=builder /app/fn0-compiler/target/release/fn0-compiler /fn0-compiler
