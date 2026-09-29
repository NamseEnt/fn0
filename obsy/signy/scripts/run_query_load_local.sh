#!/usr/bin/env bash
set -euo pipefail

# Path-independent for the reason `run_load_local.sh` states: a script that
# `cd`s to one developer's home directory is not a reproduction script.
REPO_ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$REPO_ROOT"

# Query-heavy profile.
#
# `run_load_local.sh` drives ingest and issues one query per N pushes from the
# same loop, so query concurrency there is one by construction. That is why the
# largest term in the configured memory budget —
# `max_concurrent_query_scans x max_query_memory_bytes`, four of five GiB at the
# defaults — has never been exercised by a load run. This script drives the
# other axis: modest ingest, many independent readers, and a retention period
# long enough that a wide scan has something to read.

DATA_DIR=$(mktemp -d)
REMOTE_DIR=$(mktemp -d)
SERVER_LOG=$(mktemp)
RESULT="${SIGNY_LOAD_RESULT_PATH:-docs/query_heavy_result.json}"

echo "data=$DATA_DIR remote=$REMOTE_DIR log=$SERVER_LOG"

cleanup() {
  if [ -n "${SERVER_PID:-}" ] && kill -0 "$SERVER_PID" 2>/dev/null; then
    kill -TERM "$SERVER_PID" 2>/dev/null || true
    wait "$SERVER_PID" 2>/dev/null || true
  fi
}
trap cleanup EXIT

export SIGNY_DATA_DIR="$DATA_DIR"
export SIGNY_OBJECT_STORE_URL="file://$REMOTE_DIR"
export SIGNY_LISTEN_ADDR="127.0.0.1:3100"
# Tier B fault injection (seeded, reproducible). Read latency is what a restore
# pays, so it stays on: the point of the small cache below is that the readers
# miss.
export SIGNY_OBJECT_STORE_LATENCY_MS="${SIGNY_OBJECT_STORE_LATENCY_MS:-5}"
export SIGNY_OBJECT_STORE_LATENCY_JITTER_MS="${SIGNY_OBJECT_STORE_LATENCY_JITTER_MS:-10}"
export SIGNY_OBJECT_STORE_READ_LATENCY_MS="${SIGNY_OBJECT_STORE_READ_LATENCY_MS:-20}"
export SIGNY_OBJECT_STORE_ERROR_RATE="${SIGNY_OBJECT_STORE_ERROR_RATE:-0.03}"
export SIGNY_OBJECT_STORE_FAULT_SEED="${SIGNY_OBJECT_STORE_FAULT_SEED:-20260728}"

# Far below the 8 MiB the ingest profile uses. The payload is padded with a
# repeated byte, so parquet compresses a 300k-event run down to single-digit
# megabytes and an 8 MiB cache holds all of it — the first attempt at this
# profile evicted thirteen times and restored zero, because nothing a reader
# wanted had ever left the disk.
export SIGNY_CACHE_MAX_BYTES="${SIGNY_CACHE_MAX_BYTES:-262144}"
export SIGNY_CACHE_EVICTION_INTERVAL="${SIGNY_CACHE_EVICTION_INTERVAL:-3s}"
export SIGNY_FLUSH_MAX_INTERVAL="${SIGNY_FLUSH_MAX_INTERVAL:-2s}"
export SIGNY_MERGE_INTERVAL="${SIGNY_MERGE_INTERVAL:-8s}"
# Retention has to outlast the query range or every wide scan reads an empty
# window and the run measures the scheduler admitting scans that do nothing.
# Pushed by the harness when it onboards its tenants: retention lives in the
# tenant policy now, and the SIGNY_RETENTION_PERIOD this used to set stopped
# being read when it moved there.
export SIGNY_LOAD_TENANT_RETENTION="${SIGNY_LOAD_TENANT_RETENTION:-600s}"
export SIGNY_RETENTION_GRACE_PERIOD="${SIGNY_RETENTION_GRACE_PERIOD:-5s}"
export SIGNY_RETENTION_INTERVAL="${SIGNY_RETENTION_INTERVAL:-30s}"
# The term this profile exists to exercise. Named here rather than passed to the
# harness: it is the server that admits scans.
export SIGNY_MAX_CONCURRENT_QUERY_SCANS="${SIGNY_MAX_CONCURRENT_QUERY_SCANS:-8}"

./target/release/signy >"$SERVER_LOG" 2>&1 &
SERVER_PID=$!

for _ in $(seq 1 60); do
  if curl -fsS "http://127.0.0.1:3100/ready" >/dev/null 2>&1; then break; fi
  if ! kill -0 "$SERVER_PID" 2>/dev/null; then echo "server died:"; cat "$SERVER_LOG"; exit 1; fi
  sleep 0.5
done

if [ -z "${SIGNY_MACHINE_PROFILE:-}" ]; then
  if [ "$(uname -s)" = "Darwin" ]; then
    CPUS=$(sysctl -n hw.ncpu 2>/dev/null || echo '?')
    RAM_GIB=$(( $(sysctl -n hw.memsize 2>/dev/null || echo 0) / 1073741824 ))
  else
    CPUS=$(nproc 2>/dev/null || echo '?')
    RAM_GIB=$(awk '/MemTotal/ {printf "%d", $2/1048576}' /proc/meminfo 2>/dev/null || echo 0)
  fi
  SIGNY_MACHINE_PROFILE="$(uname -sm); ${CPUS} logical CPUs; ${RAM_GIB} GiB RAM"
fi

# Same two exit codes as `run_load_local.sh`: non-zero is a verdict other than
# PASS, and 3 is the bed never getting its load into the server. `set -e` must
# not cut off the server log that explains either.
STATUS=0
SIGNY_LOAD_SERVER_PID="$SERVER_PID" \
SIGNY_LOAD_TIER=B \
SIGNY_LOAD_RESULT_PATH="$RESULT" \
SIGNY_BUILD_REVISION="$(git rev-parse --short HEAD)" \
SIGNY_MACHINE_PROFILE="$SIGNY_MACHINE_PROFILE" \
SIGNY_LOAD_SECONDS="${SIGNY_LOAD_SECONDS:-150}" \
SIGNY_LOAD_WARMUP_SECONDS="${SIGNY_LOAD_WARMUP_SECONDS:-45}" \
SIGNY_LOAD_TARGET_EPS="${SIGNY_LOAD_TARGET_EPS:-2000}" \
SIGNY_LOAD_ENTRIES_PER_PUSH="${SIGNY_LOAD_ENTRIES_PER_PUSH:-100}" \
SIGNY_LOAD_TENANTS="${SIGNY_LOAD_TENANTS:-24}" \
SIGNY_LOAD_QUERY_CONNECTIONS="${SIGNY_LOAD_QUERY_CONNECTIONS:-24}" \
SIGNY_LOAD_QUERY_WINDOW_SECONDS="${SIGNY_LOAD_QUERY_WINDOW_SECONDS:-120}" \
SIGNY_LOAD_QUERY_LIMIT="${SIGNY_LOAD_QUERY_LIMIT:-5000}" \
  ./target/release/load || STATUS=$?

echo "=== server log tail ==="
tail -20 "$SERVER_LOG"
echo "=== harness exit status: $STATUS (0 = PASS, 3 = load never landed) ==="
exit "$STATUS"
