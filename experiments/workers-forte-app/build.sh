#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")"

FORTE="${FORTE:-../../target/debug/forte}"

"$FORTE" build

npx -y @bytecodealliance/jco@latest transpile dist/backend.wasm -o worker/gen \
  --async-mode jspi --async-wasi-imports --async-wasi-exports \
  --instantiation async --no-wasi-shim --name backend

# `fe/dist` also holds the SSR bundle, which the Worker imports and must not
# serve, so the public files are staged rather than pointing Workers at it.
rm -rf worker/public
mkdir -p worker/public
cp -R fe/dist/assets worker/public/assets
cp fe/dist/client.js worker/public/client.js
cp fe/dist/robots.txt worker/public/robots.txt
