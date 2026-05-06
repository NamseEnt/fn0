FROM --platform=linux/arm64 rust:bookworm AS wasmtime-builder

WORKDIR /build
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=id=cwasm-target,type=cache,target=/build/target \
    cargo build --release -p fn0-wasmtime \
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
