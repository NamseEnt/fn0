FROM public.ecr.aws/lambda/nodejs:20-arm64

RUN dnf install -y tar xz gzip zstd gcc gcc-c++ make openssl-devel perl git pkgconfig && dnf clean all

ENV RUSTUP_HOME=/opt/rustup \
    CARGO_TOOLCHAIN_HOME=/opt/rust \
    PATH=/opt/rust/bin:$PATH

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -o /tmp/rustup.sh \
    && CARGO_HOME=$CARGO_TOOLCHAIN_HOME RUSTUP_HOME=$RUSTUP_HOME \
       sh /tmp/rustup.sh -y --default-toolchain stable --profile minimal --no-modify-path \
    && rm /tmp/rustup.sh

WORKDIR ${LAMBDA_TASK_ROOT}
COPY package.json package-lock.json ${LAMBDA_TASK_ROOT}/
RUN npm ci --omit=dev

COPY index.mjs ${LAMBDA_TASK_ROOT}/

CMD ["index.handler"]
