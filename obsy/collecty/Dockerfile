FROM rust:1-bookworm AS build
WORKDIR /src

COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY benches ./benches
RUN mkdir -p src && echo 'fn main() {}' > src/main.rs && : > src/lib.rs \
    && cargo build --release --bin collecty \
    && rm -rf src

COPY src ./src
RUN touch src/lib.rs src/main.rs && cargo build --release --bin collecty

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends systemd \
    && rm -rf /var/lib/apt/lists/*
RUN groupadd --system --gid 10002 collecty \
    && useradd --system --create-home --uid 10002 --gid 10002 collecty
USER collecty

WORKDIR /var/lib/collecty
VOLUME ["/var/lib/collecty"]
ENV COLLECTY_DATA_DIR=/var/lib/collecty
# Every address in the container, not the loopback the binary defaults to:
# a container that only answers itself has nothing to collect.
ENV COLLECTY_LISTEN_ADDR=0.0.0.0:4318
ENV COLLECTY_LOG_FORMAT=json
EXPOSE 4318

COPY --from=build /src/target/release/collecty /usr/local/bin/collecty
ENTRYPOINT ["/usr/local/bin/collecty"]
