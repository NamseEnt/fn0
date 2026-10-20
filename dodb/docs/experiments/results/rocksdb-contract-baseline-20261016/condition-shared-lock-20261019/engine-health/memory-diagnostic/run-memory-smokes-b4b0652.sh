#!/usr/bin/env bash
set -euo pipefail

result_root=/bench/zfs/db/condition-shared-lock-20261019/engine-health/memory-diagnostic-b4b0652
binary_root=/bench/zfs/db/dodb-rocksdb-shared-cache/linux-aarch64-gnu/rust-1.97.1/release-crc/target/release
service_binary="$binary_root/service_workload"
unconditional_binary="$binary_root/unconditional_write_workload"
source_revision=b4b0652b710d896a39649dc0f8341b4730da2439
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

parent_root="$result_root/parent-death"
mkdir -p "$parent_root/data"
"$unconditional_binary" --backend raw-rocksdb --width 1 --clients 16 --seed 20261013 --value-size 512 --warmup-ms 1000 --measurement-ms 5000 --call-cap 100000000 --validate-histogram 0 --revision-allocator serialized --condition-key-lock-mode shared --engine-health-interval-ms 1000 --shared-blocking-slots 8 --memory-diagnostic 1 --memory-worker-startup-timeout-ms 300000 --source-revision "$source_revision" --data-dir "$parent_root/data" --report "$parent_root/report.json" > "$parent_root/stdout.log" 2> "$parent_root/stderr.log" &
parent_pid=$!
worker_pid=
for attempt in $(seq 1 100); do
    if [ -r "/proc/$parent_pid/task/$parent_pid/children" ]; then
        worker_pid=$(cat "/proc/$parent_pid/task/$parent_pid/children" | awk '{print $1}')
        if [ -n "$worker_pid" ]; then
            break
        fi
    fi
    sleep 0.1
done
if [ -z "$worker_pid" ]; then
    kill -KILL "$parent_pid" 2>/dev/null || true
    wait "$parent_pid" 2>/dev/null || true
    printf '%s\n' worker-not-observed > "$parent_root/parent-death-status.txt"
    exit 32
fi
parent_group=$(awk '{print $5}' "/proc/$parent_pid/stat")
worker_group=$(awk '{print $5}' "/proc/$worker_pid/stat")
printf 'parent_pid=%s\nworker_pid=%s\nparent_pgid=%s\nworker_pgid=%s\n' "$parent_pid" "$worker_pid" "$parent_group" "$worker_group" > "$parent_root/processes.txt"
if [ "$parent_group" != "$worker_group" ]; then
    exit 33
fi
kill -KILL "$parent_pid"
wait "$parent_pid" 2> "$parent_root/parent-wait.log" || true
worker_gone=0
for attempt in $(seq 1 200); do
    if [ ! -e "/proc/$worker_pid/stat" ]; then
        worker_gone=1
        break
    fi
    worker_state=$(awk '{print $3}' "/proc/$worker_pid/stat" 2>/dev/null || true)
    if [ "$worker_state" = Z ]; then
        worker_gone=1
        break
    fi
    sleep 0.1
done
printf 'worker_exited_after_parent_kill=%s\n' "$worker_gone" >> "$parent_root/processes.txt"
if [ "$worker_gone" != 1 ]; then
    kill -KILL "$worker_pid" 2>/dev/null || true
    exit 34
fi

validate_report() {
    python3 - "$1" <<'PYREPORT'
import json
import sys

report = json.load(open(sys.argv[1]))
passed_values = []
def inspect(value):
    if isinstance(value, dict):
        if isinstance(value.get("passed"), bool):
            passed_values.append(value["passed"])
        for nested in value.values():
            inspect(nested)
    elif isinstance(value, list):
        for nested in value:
            inspect(nested)
for key in ("validation", "final_validation", "reopen_validation"):
    if key in report:
        inspect(report[key])
if not passed_values or not all(passed_values):
    raise SystemExit("validation oracle did not pass")
diagnostic = report.get("memory_only_diagnostic")
if not isinstance(diagnostic, dict) or diagnostic.get("enabled") is not True:
    raise SystemExit("memory diagnostic report missing")
workers = [entry for entry in diagnostic.get("processes", []) if entry.get("role") == "backend-process"]
if len(workers) != 2:
    raise SystemExit(f"expected initial and reopened worker identities, found {len(workers)}")
events = diagnostic.get("worker_lifecycle", [])
if sum(event.get("event") == "started" for event in events) != 2 or sum(event.get("event") == "stopped" for event in events) != 2:
    raise SystemExit("worker restart/shutdown lifecycle incomplete")
health_reports = diagnostic.get("engine_health_reports", [])
if len(health_reports) != 2:
    raise SystemExit(f"expected two worker shutdown reports, found {len(health_reports)}")
for health in health_reports:
    if health.get("shutdown_complete") is not True or not health.get("worker_epoch_id"):
        raise SystemExit("worker shutdown report lacks identity or completion")
    if health.get("worker_process_high_water_rss_kb_after_backend_shutdown") is None:
        raise SystemExit("worker post-shutdown VmHWM is missing")
    if health.get("shutdown_property_snapshot") is None:
        raise SystemExit("worker shutdown property snapshot is missing")
if not diagnostic.get("residency_scans"):
    raise SystemExit("residency scan reports are missing")
PYREPORT
}

run_service_case() {
    case_name=$1
    backend_name=$2
    case_root="$result_root/$case_name"
    mkdir -p "$case_root/data"
    "$service_binary" --backend "$backend_name" --client-executor dedicated --scenario mt-mix --clients 4 --rows 64 --value-size 512 --seed 20261013 --warmup-ms 1000 --measurement-ms 5000 --call-cap 1000000 --mutation-workload overwrite --overwrite-slots-per-client 16 --latency-storage log1024 --max-write-ledger 10000 --require-full-window 1 --include-width-16 1 --revision-allocator reserved-ranges --condition-key-lock-mode shared --shared-blocking-slots 4 --engine-health-interval-ms 1000 --research-metrics 0 --read-stage-metrics 0 --detailed-write-metrics 0 --memory-diagnostic 1 --memory-worker-startup-timeout-ms 300000 --source-revision "$source_revision" --data-dir "$case_root/data" --report "$case_root/report.json" > "$case_root/stdout.log" 2> "$case_root/stderr.log"
    validate_report "$case_root/report.json"
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
    "$unconditional_binary" --backend "$backend_name" --width 1 --clients 16 --seed 20261013 --value-size 512 --warmup-ms 1000 --measurement-ms 5000 --call-cap 100000000 --validate-histogram 0 --revision-allocator serialized --condition-key-lock-mode shared --engine-health-interval-ms 1000 --shared-blocking-slots 8 --memory-diagnostic 1 --memory-worker-startup-timeout-ms 300000 --source-revision "$source_revision" --data-dir "$case_root/data" --report "$case_root/report.json" > "$case_root/stdout.log" 2> "$case_root/stderr.log"
    validate_report "$case_root/report.json"
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

