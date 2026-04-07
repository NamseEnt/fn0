FROM rust:bookworm AS builder
WORKDIR /app

COPY fn0/Cargo.toml fn0/Cargo.toml
COPY fn0-compiler/Cargo.toml fn0-compiler/Cargo.toml
COPY fn0-compiler/Cargo.lock fn0-compiler/Cargo.lock
RUN mkdir -p fn0/src && echo "" > fn0/src/lib.rs && \
    mkdir -p fn0-compiler/src && echo "fn main(){}" > fn0-compiler/src/main.rs && \
    cargo build --release --manifest-path fn0-compiler/Cargo.toml

COPY fn0/src/ fn0/src/
COPY fn0-compiler/src/ fn0-compiler/src/
RUN touch fn0/src/lib.rs fn0-compiler/src/main.rs && \
    cargo build --release --manifest-path fn0-compiler/Cargo.toml

FROM scratch
COPY --from=builder /app/fn0-compiler/target/release/fn0-compiler /fn0-compiler
