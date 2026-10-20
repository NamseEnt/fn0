#!/usr/bin/env bash
set -euo pipefail

result_root=/bench/zfs/db/condition-shared-lock-20261019/engine-health/memory-diagnostic-70128f74e/retry-01
binary_root=/bench/zfs/db/dodb-rocksdb-shared-cache/linux-aarch64-gnu/rust-1.97.1/release-crc/target/release
service_binary="$binary_root/service_workload"
unconditional_binary="$binary_root/unconditional_write_workload"
source_revision=70128f74e116781d9b0ea575e74720aff71af291
mkdir -p "$result_root"
sha256sum "$service_binary" "$unconditional_binary" "$binary_root/memory_diagnostic_worker" > "$result_root/binary-sha256.txt"
printf '%s\n' "$source_revision" > "$result_root/source-revision.txt"
hostname > "$result_root/hostname.txt"
uname -a > "$result_root/uname.txt"
grep -E 'MemTotal|MemAvailable' /proc/meminfo > "$result_root/memory-before.txt"
for command_line in /proc/[0-9]*/cmdline; do
    process_id=${command_line#/proc/}
    process_id=${process_id%/cmdline}
    if [ "$process_id" = "$$" ]; then
        continue
    fi
    process_name=
    IFS= read -r -d '' process_name < "$command_line" || true
    process_name=${process_name##*/}
    case "$process_name" in
        cargo|rustc|service_workload|unconditional_write_workload|memory_diagnostic_worker)
            printf '%s\t%s\n' "$process_id" "$process_name" >> "$result_root/process-preflight.txt"
            exit 31
            ;;
    esac
done

run_service_case() {
    case_name=$1
    backend_name=$2
    case_root="$result_root/$case_name"
    mkdir -p "$case_root/data"
    "$service_binary" --backend "$backend_name" --client-executor dedicated --scenario mt-mix --clients 4 --rows 16 --value-size 512 --seed 20261013 --warmup-ms 1000 --measurement-ms 5000 --call-cap 1000000 --mutation-workload overwrite --overwrite-slots-per-client 16 --latency-storage log1024 --max-write-ledger 10000 --require-full-window 1 --include-width-16 1 --revision-allocator reserved-ranges --condition-key-lock-mode shared --shared-blocking-slots 4 --engine-health-interval-ms 1000 --research-metrics 0 --read-stage-metrics 0 --detailed-write-metrics 0 --memory-diagnostic 1 --source-revision "$source_revision" --data-dir "$case_root/data" --report "$case_root/report.json" > "$case_root/stdout.log" 2> "$case_root/stderr.log"
    find "$case_root/data" -type f -print0 | sort -z | xargs -0 sha256sum > "$case_root/data-sha256-before-cleanup.txt"
    find "$case_root/data" -type f -printf '%P\t%s\n' | sort > "$case_root/data-files-before-cleanup.tsv"
    grep -R '' "$case_root/data"/*/LOG > "$case_root/log-contents.txt" 2>/dev/null || true
    rm -rf "$case_root/data"
    printf '%s\n' complete > "$case_root/status.txt"
}

for backend_entry in 'service-rocksdb rocksdb' 'service-core rocksdb-direct' 'service-blink blink'; do
    read -r case_name backend_name <<< "$backend_entry"
    run_service_case "$case_name" "$backend_name"
done

run_unconditional_case() {
    case_name=$1
    backend_name=$2
    case_root="$result_root/$case_name"
    mkdir -p "$case_root/data"
    "$unconditional_binary" --backend "$backend_name" --width 1 --clients 16 --seed 20261013 --value-size 512 --warmup-ms 1000 --measurement-ms 5000 --call-cap 100000000 --validate-histogram 0 --revision-allocator serialized --condition-key-lock-mode shared --engine-health-interval-ms 1000 --shared-blocking-slots 8 --memory-diagnostic 1 --source-revision "$source_revision" --data-dir "$case_root/data" --report "$case_root/report.json" > "$case_root/stdout.log" 2> "$case_root/stderr.log"
    find "$case_root/data" -type f -print0 | sort -z | xargs -0 sha256sum > "$case_root/data-sha256-before-cleanup.txt"
    find "$case_root/data" -type f -printf '%P\t%s\n' | sort > "$case_root/data-files-before-cleanup.tsv"
    grep -R '' "$case_root/data"/*/LOG > "$case_root/log-contents.txt" 2>/dev/null || true
    rm -rf "$case_root/data"
    printf '%s\n' complete > "$case_root/status.txt"
}

for backend_entry in 'unconditional-raw raw-rocksdb' 'unconditional-core synchronous-core' 'unconditional-service rocksdb-service' 'unconditional-blink blink'; do
    read -r case_name backend_name <<< "$backend_entry"
    run_unconditional_case "$case_name" "$backend_name"
done

grep -E 'MemTotal|MemAvailable' /proc/meminfo > "$result_root/memory-after.txt"
find "$result_root" -maxdepth 2 -type f -print | sort > "$result_root/artifact-index.txt"
printf '%s\n' complete > "$result_root/run-status.txt"

