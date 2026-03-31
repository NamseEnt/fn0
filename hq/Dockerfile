FROM rust:bookworm AS builder
WORKDIR /app

COPY doc-db/ doc-db/
COPY host-hq-protocol/ host-hq-protocol/
COPY hq/ hq/

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/hq/target \
    cd hq && cargo build --release && cp target/release/hq /app/hq-bin

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/hq-bin ./hq
EXPOSE 8080
CMD ["/bin/sh", "-c", "ulimit -n 100000 && ./hq"]
