FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY fn0-worker ./fn0-worker
RUN chmod +x /app/fn0-worker
EXPOSE 443 10000
CMD ["/bin/sh", "-c", "ulimit -n 100000 && ./fn0-worker"]
