FROM rust:bookworm AS builder
WORKDIR /app

COPY . .

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=id=fn0-compiler-target,type=cache,target=/app/target \
    cargo build --release -p fn0-compiler && \
    cp target/release/fn0-compiler /app/fn0-compiler-bin

FROM scratch
COPY --from=builder /app/fn0-compiler-bin /fn0-compiler
