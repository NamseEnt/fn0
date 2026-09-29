#!/usr/bin/env bash
set -euo pipefail

: "${H1_TREE:?set H1_TREE to the clean Phase I checkout}"
: "${J_TREE:?set J_TREE to the clean Phase J checkout}"
: "${RESULT_ROOT:?set RESULT_ROOT for run logs and JSONL}"
: "${DATA_ROOT:?set DATA_ROOT to the ZFS benchmark directory}"
DURATION=${DURATION:-10s}
WARMUP=${WARMUP:-2s}
WORKING_SET=${WORKING_SET:-4096}

mkdir -p "$RESULT_ROOT" "$DATA_ROOT"
for tree in "$H1_TREE" "$J_TREE"; do
    test -z "$(git -C "$tree" status --porcelain)"
done

cargo build --release -p dodb-storage --bin phase0-bench --manifest-path "$H1_TREE/Cargo.toml"
cargo build --release -p dodb-storage --bin phase0-bench --manifest-path "$J_TREE/Cargo.toml"

h1_commit=$(git -C "$H1_TREE" rev-parse HEAD)
j_commit=$(git -C "$J_TREE" rev-parse HEAD)
h1_binary="$H1_TREE/target/release/phase0-bench"
j_binary="$J_TREE/target/release/phase0-bench"
h1_binary_sha=$(sha256sum "$h1_binary" | awk '{print $1}')
j_binary_sha=$(sha256sum "$j_binary" | awk '{print $1}')
test -x "$h1_binary"
test -x "$j_binary"

run_index=0
: > "$RESULT_ROOT/run-order.jsonl"
for repetition in 0 1 2; do
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
        for engine in h1 j; do
            if [[ "$engine" == h1 ]]; then
                tree=$H1_TREE
                binary=$h1_binary
                source_commit=$h1_commit
                binary_sha=$h1_binary_sha
                engine_name=parallel-blink
            else
                tree=$J_TREE
                binary=$j_binary
                source_commit=$j_commit
                binary_sha=$j_binary_sha
                engine_name=logical-overlay-blink
            fi
            bench_distribution=$distribution
            if [[ "$engine" == h1 ]]; then
                case "$distribution" in
                    compact) bench_distribution=same-leaf-heavy ;;
                    spread) bench_distribution=different-leaf-heavy ;;
                esac
            fi
            run_id=$(printf '%02d-%s-%s-rep%d' "$run_index" "$engine" "$case_name" "$repetition")
            output="$RESULT_ROOT/$run_id.jsonl"
            log="$RESULT_ROOT/$run_id.log"
            printf 'run %s commit=%s writers=%s width=%s distribution=%s seed=%s\n' "$run_id" "$source_commit" "$writers" "$width" "$distribution" "$seed"
            (
                cd "$tree"
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
            test "$(git -C "$tree" rev-parse HEAD)" = "$source_commit"
            printf '{"order":%s,"run":"%s","engine":"%s","source_commit":"%s","binary_sha256":"%s","writers":%s,"width":%s,"distribution":"%s","repetition":%s,"seed":%s,"duration":"%s","warmup":"%s","data_root":"%s","host":"%s"}\n' \
                "$run_index" "$run_id" "$engine_name" "$source_commit" "$binary_sha" \
                "$writers" "$width" "$distribution" "$repetition" "$seed" \
                "$DURATION" "$WARMUP" "$DATA_ROOT" "$(hostname -f)" >> "$RESULT_ROOT/run-order.jsonl"
            run_index=$((run_index + 1))
        done
    done
done
