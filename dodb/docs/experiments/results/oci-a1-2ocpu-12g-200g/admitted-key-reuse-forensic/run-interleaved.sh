#!/bin/bash
set -euo pipefail
artifact=/home/opc/dodb-oci-artifacts-admitted-key-reuse-forensic-1eaabbc
base_binary=/home/opc/dodb-forensic-binaries/base-phase0-bench
candidate_binary=/home/opc/dodb-forensic-binaries/candidate-a-phase0-bench
run_point() {
    build_label=$1
    benchmark_binary=$2
    transaction_width=$3
    repetition_index=$4
    repetition_seed=$5
    if [ "$transaction_width" -eq 1 ]; then
        workload_distribution=different-leaf-heavy
    else
        workload_distribution=uniform
    fi
    raw_file="$artifact/planned-$build_label-width$transaction_width.jsonl"
    temporary_output="/tmp/admitted-key-reuse-forensic-$build_label-width$transaction_width-seed$repetition_seed.jsonl"
    run_log="$artifact/run-$build_label-width$transaction_width-seed$repetition_seed.log"
    printf "%s width=%s repetition=%s seed=%s binary=%s\n" "$build_label" "$transaction_width" "$repetition_index" "$repetition_seed" "$benchmark_binary" >> "$artifact/remeasurement-order.txt"
    "$benchmark_binary" --suite write --engine planned-blink --writers 16 --readers 0 --widths "$transaction_width" --distributions "$workload_distribution" --working-set 100000 --cache-capacity 4096 --key-size 16 --value-size 64 --group-limit 64 --group-bytes 4194304 --queue-capacity 256 --collection-delay 0us --sync-mode disabled --tokio-workers 2 --warmup 1s --duration 2s --repetitions 1 --seed "$repetition_seed" --output "$temporary_output" > "$run_log" 2>&1
    cat "$temporary_output" >> "$raw_file"
    cat "$run_log"
}
: > "$artifact/planned-base-width1.jsonl"
: > "$artifact/planned-candidate-width1.jsonl"
: > "$artifact/planned-base-width16.jsonl"
: > "$artifact/planned-candidate-width16.jsonl"
: > "$artifact/remeasurement-order.txt"
for repetition_index in 0 1 2; do
    width1_seed=$((0x3a042026 + repetition_index))
    width16_seed=$((0x3a032026 + repetition_index))
    if [ "$repetition_index" -eq 1 ]; then
        run_point candidate "$candidate_binary" 1 "$repetition_index" "$width1_seed"
        run_point base "$base_binary" 1 "$repetition_index" "$width1_seed"
        run_point candidate "$candidate_binary" 16 "$repetition_index" "$width16_seed"
        run_point base "$base_binary" 16 "$repetition_index" "$width16_seed"
    else
        run_point base "$base_binary" 1 "$repetition_index" "$width1_seed"
        run_point candidate "$candidate_binary" 1 "$repetition_index" "$width1_seed"
        run_point base "$base_binary" 16 "$repetition_index" "$width16_seed"
        run_point candidate "$candidate_binary" 16 "$repetition_index" "$width16_seed"
    fi
done
sha256sum "$base_binary" "$candidate_binary" /home/opc/dodb-forensic-binaries/candidate-b-phase0-bench
