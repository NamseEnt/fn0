FROM --platform=$BUILDPLATFORM rust:bookworm AS builder
WORKDIR /app

ARG TARGETARCH

RUN if [ "$TARGETARCH" = "arm64" ]; then \
      dpkg --add-architecture arm64 && \
      apt-get update && apt-get install -y gcc-aarch64-linux-gnu libc6-dev-arm64-cross && \
      rustup target add aarch64-unknown-linux-gnu; \
    fi

COPY doc-db/ doc-db/
COPY host-hq-protocol/ host-hq-protocol/
COPY oci-rust-sdk/ oci-rust-sdk/
COPY hq/ hq/

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/hq/target \
    if [ "$TARGETARCH" = "arm64" ]; then \
      CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc \
      cargo build --release --target aarch64-unknown-linux-gnu --manifest-path hq/Cargo.toml && \
      cp hq/target/aarch64-unknown-linux-gnu/release/hq /app/hq-bin; \
    else \
      cd hq && cargo build --release && cp target/release/hq /app/hq-bin; \
    fi

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/hq-bin ./hq
EXPOSE 8080
CMD ["/bin/sh", "-c", "ulimit -n 100000 && ./hq"]
