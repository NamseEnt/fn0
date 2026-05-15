FROM node:20-bookworm AS handler-deps
WORKDIR /handler
COPY package.json ./
RUN npm install --omit=dev

FROM public.ecr.aws/lambda/nodejs:20-arm64
RUN dnf install -y zstd && dnf clean all
COPY fn0-wasmtime /opt/fn0-wasmtime
COPY --from=handler-deps /handler/node_modules ${LAMBDA_TASK_ROOT}/node_modules
COPY handler.mjs ${LAMBDA_TASK_ROOT}/handler.mjs
COPY package.json ${LAMBDA_TASK_ROOT}/package.json
CMD ["handler.handler"]
