#!/usr/bin/env bash
set -euo pipefail

: "${TREE:?set TREE to the clean canonical experiment source checkout}"
: "${RESULT_ROOT:?set RESULT_ROOT for run logs and JSONL}"
: "${DATA_ROOT:?set DATA_ROOT to the OCI ZFS benchmark directory}"
DURATION=${DURATION:-10s}
WARMUP=${WARMUP:-2s}
REPETITIONS=${REPETITIONS:-3}
WORKING_SET=${WORKING_SET:-4096}
K_COMMIT=08e82ec484bd62aeb659471a39f317b86cfb488f
BRANCH=experiment/b-link-batched-engine-monorepo

test "$(git -C "$TREE" rev-parse HEAD)" = "$K_COMMIT"
test -z "$(git -C "$TREE" status --porcelain)"
mkdir -p "$RESULT_ROOT/bin" "$RESULT_ROOT" "$DATA_ROOT"

cargo build --release -p dodb-storage --bin phase0-bench --manifest-path "$TREE/Cargo.toml"
cp "$TREE/target/release/phase0-bench" "$RESULT_ROOT/bin/phase0-bench-k"
k_binary_sha=$(sha256sum "$RESULT_ROOT/bin/phase0-bench-k" | awk '{print $1}')
cp "$RESULT_ROOT/bin/phase0-bench-k" "$RESULT_ROOT/bin/phase0-bench-h1-mode"
h1_binary_sha=$k_binary_sha
test "$(git -C "$TREE" rev-parse HEAD)" = "$K_COMMIT"
test -z "$(git -C "$TREE" status --porcelain)"

: > "$RESULT_ROOT/run-order.jsonl"
run_index=0
for repetition in $(seq 0 $((REPETITIONS - 1))); do
    for scenario in 0 1 2 3 4 5; do
        case "$scenario" in
            0) writers=16; width=1; distribution=uniform; case_name=16w-width1-uniform ;;
            1) writers=16; width=16; distribution=uniform; case_name=16w-width16-uniform ;;
            2) writers=64; width=1; distribution=uniform; case_name=64w-width1-uniform ;;
            3) writers=64; width=16; distribution=uniform; case_name=64w-width16-uniform ;;
            4) writers=64; width=16; distribution=compact; case_name=64w-width16-compact ;;
            5) writers=64; width=16; distribution=spread; case_name=64w-width16-spread ;;
        esac
        seed=$((867530900 + scenario * 100 + repetition))
        for engine in h1 phase-j phase-k; do
            if [[ "$engine" == h1 ]]; then
                binary="$RESULT_ROOT/bin/phase0-bench-h1-mode"
                source_commit=$K_COMMIT
                binary_sha=$h1_binary_sha
                engine_name=parallel-blink
                bench_distribution=$distribution
                case "$distribution" in
                    compact) bench_distribution=same-leaf-heavy ;;
                    spread) bench_distribution=different-leaf-heavy ;;
                esac
            else
                binary="$RESULT_ROOT/bin/phase0-bench-k"
                source_commit=$K_COMMIT
                binary_sha=$k_binary_sha
                engine_name=$engine
                bench_distribution=$distribution
            fi
            run_id=$(printf '%02d-%s-%s-rep%d' "$run_index" "$engine" "$case_name" "$repetition")
            output="$RESULT_ROOT/$run_id.jsonl"
            log="$RESULT_ROOT/$run_id.log"
            (
                cd "$TREE"
                DODB_BENCH_DIR="$DATA_ROOT" "$binary" \
                    --engine "$engine_name" \
                    --suite write \
                    --writers "$writers" \
                    --widths "$width" \
                    --distribution "$bench_distribution" \
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
            test "$(git -C "$TREE" rev-parse HEAD)" = "$K_COMMIT"
            printf '{"order":%s,"run":"%s","engine":"%s","engine_selector":"%s","monorepo_commit":"%s","branch":"%s","dodb_path":"dodb/","binary_sha256":"%s","writers":%s,"width":%s,"distribution":"%s","repetition":%s,"seed":%s,"duration":"%s","warmup":"%s","sync_mode":"real","data_root":"%s","host":"%s"}\n' \
                "$run_index" "$run_id" "$engine" "$engine_name" "$source_commit" "$BRANCH" "$binary_sha" \
                "$writers" "$width" "$distribution" "$repetition" "$seed" \
                "$DURATION" "$WARMUP" "$DATA_ROOT" "$(hostname -f)" >> "$RESULT_ROOT/run-order.jsonl"
            run_index=$((run_index + 1))
        done
    done
done
