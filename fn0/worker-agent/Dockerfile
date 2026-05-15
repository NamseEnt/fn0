FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y ca-certificates podman \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY fn0-worker-agent /usr/local/bin/fn0-worker-agent
ENTRYPOINT ["/usr/local/bin/fn0-worker-agent"]
