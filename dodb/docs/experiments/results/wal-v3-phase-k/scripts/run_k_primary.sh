#!/usr/bin/env bash
set -euo pipefail

: "${TREE:?set TREE to the clean canonical experiment worktree}"
: "${RESULT_ROOT:?set RESULT_ROOT for run logs and JSONL}"
: "${DATA_ROOT:?set DATA_ROOT to the OCI ZFS benchmark directory}"
DURATION=${DURATION:-10s}
WARMUP=${WARMUP:-2s}
REPETITIONS=${REPETITIONS:-3}
WORKING_SET=${WORKING_SET:-4096}

test -z "$(git -C "$TREE" status --porcelain)"
mkdir -p "$RESULT_ROOT" "$DATA_ROOT"
cargo build --release -p dodb-storage --bin phase0-bench --manifest-path "$TREE/Cargo.toml"

source_commit=$(git -C "$TREE" rev-parse HEAD)
branch=$(git -C "$TREE" branch --show-current)
binary="$TREE/target/release/phase0-bench"
binary_sha=$(sha256sum "$binary" | awk '{print $1}')
test -x "$binary"
: > "$RESULT_ROOT/run-order.jsonl"
run_index=0

for repetition in $(seq 0 $((REPETITIONS - 1))); do
    for engine in phase-j phase-k; do
        seed=$((867531000 + repetition))
        run_id=$(printf '%02d-%s-64w-width16-uniform-rep%d' "$run_index" "$engine" "$repetition")
        output="$RESULT_ROOT/$run_id.jsonl"
        log="$RESULT_ROOT/$run_id.log"
        (
            cd "$TREE"
            DODB_BENCH_DIR="$DATA_ROOT" "$binary" \
                --engine "$engine" \
                --suite write \
                --writers 64 \
                --widths 16 \
                --distribution uniform \
                --duration "$DURATION" \
                --warmup "$WARMUP" \
                --repetitions 1 \
                --working-set "$WORKING_SET" \
                --value-size 64 \
                --key-size 16 \
                --tokio-workers 2 \
                --blink-workers 2 \
                --group-limit 64 \
                --sync-mode real \
                --seed "$seed" \
                --output "$output"
        ) > "$log" 2>&1
        test "$(git -C "$TREE" rev-parse HEAD)" = "$source_commit"
        printf '{"order":%s,"run":"%s","engine":"%s","monorepo_commit":"%s","branch":"%s","dodb_path":"dodb/","binary_sha256":"%s","writers":64,"width":16,"distribution":"uniform","repetition":%s,"seed":%s,"duration":"%s","warmup":"%s","data_root":"%s","host":"%s"}\n' \
            "$run_index" "$run_id" "$engine" "$source_commit" "$branch" "$binary_sha" \
            "$repetition" "$seed" "$DURATION" "$WARMUP" "$DATA_ROOT" "$(hostname -f)" \
            >> "$RESULT_ROOT/run-order.jsonl"
        run_index=$((run_index + 1))
    done
done
