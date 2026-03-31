FROM rust:bookworm AS builder
WORKDIR /app

COPY doc-db/Cargo.toml doc-db/Cargo.toml
COPY host-hq-protocol/Cargo.toml host-hq-protocol/Cargo.toml
COPY hq/Cargo.toml hq/Cargo.lock hq/build.rs hq/

RUN mkdir -p doc-db/src host-hq-protocol/src hq/src && \
    echo "pub struct DocDb;" > doc-db/src/lib.rs && \
    echo "" > host-hq-protocol/src/lib.rs && \
    echo "fn main() {}" > hq/src/main.rs && \
    mkdir -p hq/src && echo "" > hq/src/args.rs

RUN cd hq && cargo build --release 2>/dev/null || true

COPY doc-db/ doc-db/
COPY host-hq-protocol/ host-hq-protocol/
COPY hq/src/ hq/src/
COPY hq/build.rs hq/build.rs

RUN touch hq/src/main.rs && cd hq && cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/hq/target/release/hq .
EXPOSE 8080
CMD ["/bin/sh", "-c", "ulimit -n 100000 && ./hq"]
