#!/usr/bin/env bash
set -euo pipefail
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT/fn0/test-wasm/outbound-post"
cargo build --release --target wasm32-wasip2
echo "built: $(pwd)/target/wasm32-wasip2/release/outbound_post.wasm"
