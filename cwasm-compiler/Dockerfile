FROM --platform=linux/arm64 rust:bookworm AS wasmtime-builder
ARG SCCACHE_BUCKET
ARG SCCACHE_REGION
ARG SCCACHE_ENDPOINT
ARG SCCACHE_S3_USE_SSL=true
ARG AWS_ACCESS_KEY_ID
ARG AWS_SECRET_ACCESS_KEY

ENV SCCACHE_BUCKET=${SCCACHE_BUCKET} \
    SCCACHE_REGION=${SCCACHE_REGION} \
    SCCACHE_ENDPOINT=${SCCACHE_ENDPOINT} \
    SCCACHE_S3_USE_SSL=${SCCACHE_S3_USE_SSL} \
    AWS_ACCESS_KEY_ID=${AWS_ACCESS_KEY_ID} \
    AWS_SECRET_ACCESS_KEY=${AWS_SECRET_ACCESS_KEY} \
    RUSTC_WRAPPER=sccache

ARG SCCACHE_VERSION=0.14.0
RUN curl -fsSL "https://github.com/mozilla/sccache/releases/download/v${SCCACHE_VERSION}/sccache-v${SCCACHE_VERSION}-aarch64-unknown-linux-musl.tar.gz" \
    | tar -xz -C /tmp \
 && mv "/tmp/sccache-v${SCCACHE_VERSION}-aarch64-unknown-linux-musl/sccache" /usr/local/bin/sccache \
 && chmod +x /usr/local/bin/sccache \
 && rm -rf "/tmp/sccache-v${SCCACHE_VERSION}-aarch64-unknown-linux-musl"

WORKDIR /build
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=id=cwasm-target,type=cache,target=/build/target \
    cargo build --release -p fn0-wasmtime \
 && sccache --show-stats \
 && cp target/release/fn0-wasmtime /build/fn0-wasmtime-bin

FROM --platform=linux/arm64 node:20-bookworm AS handler-deps
WORKDIR /handler
COPY cwasm-compiler/package.json ./
RUN npm install --omit=dev

FROM public.ecr.aws/lambda/nodejs:20-arm64
RUN dnf install -y zstd && dnf clean all
COPY --from=wasmtime-builder /build/fn0-wasmtime-bin /opt/fn0-wasmtime
COPY --from=handler-deps /handler/node_modules ${LAMBDA_TASK_ROOT}/node_modules
COPY cwasm-compiler/handler.mjs ${LAMBDA_TASK_ROOT}/handler.mjs
COPY cwasm-compiler/package.json ${LAMBDA_TASK_ROOT}/package.json
CMD ["handler.handler"]
