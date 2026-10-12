#!/usr/bin/env bash
set -euo pipefail

suite=${1:?usage: run-matrix.sh smoke|core|sustained OUTPUT_ROOT}
output_root=${2:?usage: run-matrix.sh smoke|core|sustained OUTPUT_ROOT}
run_mode=${3:-fresh}
data_root=${CROSSDB_DATA_ROOT:-/bench/zfs/db}
required_fstype=${CROSSDB_REQUIRED_FSTYPE:-zfs}
required_source=${CROSSDB_REQUIRED_SOURCE:-dodbbench/db}
export CROSSDB_DATA_ROOT="$data_root"
repo_root=$(git rev-parse --show-toplevel)
harness_root="$repo_root/dodb/docs/experiments/results/oci-a1-2ocpu-12g-zfs-crossdb/harness"
source_sha=$(git -C "$repo_root" rev-parse HEAD)
[[ -z "$(git -C "$repo_root" status --porcelain)" ]] || {
    printf 'source checkout must be clean before measurement\n' >&2
    exit 2
}
case "$output_root" in
    "$data_root"/*) ;;
    *) printf 'output root must be under the configured data root: %s\n' "$output_root" >&2; exit 2 ;;
esac
mount_source=$(findmnt -n -o SOURCE --target "$data_root" 2>/dev/null || true)
mount_fstype=$(findmnt -n -o FSTYPE --target "$data_root" 2>/dev/null || true)
[[ "$required_fstype" == any || "$mount_fstype" == "$required_fstype" ]] || {
    printf 'data root filesystem type mismatch: expected=%s actual=%s\n' "$required_fstype" "$mount_fstype" >&2
    exit 2
}
[[ "$required_source" == any || "$mount_source" == "$required_source" ]] || {
    printf 'data root source mismatch: expected=%s actual=%s\n' "$required_source" "$mount_source" >&2
    exit 2
}
if [[ -e "$output_root" ]]; then
    [[ "$run_mode" == resume && -f "$output_root/attempt-ledger.tsv" ]] || {
        printf 'output root already exists; use resume to preserve and continue: %s\n' "$output_root" >&2
        exit 2
    }
    [[ "$(sed -n 's/^source_sha=//p' "$output_root/provenance.txt" | head -n 1)" == "$source_sha" ]] || {
        printf 'resume source SHA does not match preserved run root\n' >&2
        exit 2
    }
    [[ "$(sed -n 's/^suite=//p' "$output_root/provenance.txt" | head -n 1)" == "$suite" ]] || {
        printf 'resume suite does not match preserved run root\n' >&2
        exit 2
    }
else
    [[ "$run_mode" == fresh ]] || {
        printf 'resume requested for missing run root: %s\n' "$output_root" >&2
        exit 2
    }
    mkdir -p "$output_root/logs" "$output_root/db"
    printf 'run_id\tengine\tcondition\trepetition\tseed\tprocess_exit\tvalidation\n' > "$output_root/attempt-ledger.tsv"
fi

main_binary=${DODB_MAIN_BINARY:?}
planned_binary=${DODB_PLANNED_BINARY:?}
rocks_binary=${ROCKSDB_BINARY:?}
if [[ ! -f "$output_root/provenance.txt" ]]; then
    {
        printf 'source_sha=%s\n' "$source_sha"
        printf 'source_clean=true\n'
        printf 'cargo_lock_sha256=%s\n' "$(sha256sum "$harness_root/dodb-bench/Cargo.lock" | cut -d' ' -f1)"
        printf 'rocksdb_cargo_lock_sha256=%s\n' "$(sha256sum "$harness_root/rocksdb-bench/Cargo.lock" | cut -d' ' -f1)"
        printf 'cargo_config_sha256=%s\n' "$(sha256sum "$repo_root/.cargo/config.toml" | cut -d' ' -f1)"
        printf 'dodb_harness_sha256=%s\n' "$(find "$harness_root/dodb-bench/src" "$harness_root/common" -type f -print0 | sort -z | xargs -0 sha256sum | sha256sum | cut -d' ' -f1)"
        printf 'rocks_harness_sha256=%s\n' "$(find "$harness_root/rocksdb-bench" -type f -not -path '*/target/*' -print0 | sort -z | xargs -0 sha256sum | sha256sum | cut -d' ' -f1)"
        printf 'dodb_binary_sha256=%s\n' "$(sha256sum "$main_binary" | cut -d' ' -f1)"
        printf 'rocksdb_binary_sha256=%s\n' "$(sha256sum "$rocks_binary" | cut -d' ' -f1)"
        printf 'host=%s\n' "$(hostname -f)"
        printf 'kernel=%s\n' "$(uname -a)"
        printf 'data_root=%s\n' "$data_root"
        printf 'filesystem=%s\n' "$(findmnt -n -o SOURCE,FSTYPE,OPTIONS --target "$data_root")"
        printf 'rocksdb_tag=%s\n' "${CROSSDB_ROCKSDB_TAG:?}"
        printf 'rocksdb_commit=%s\n' "${CROSSDB_ROCKSDB_COMMIT:?}"
        printf 'suite=%s\n' "$suite"
    } > "$output_root/provenance.txt"
fi
snapshot_id="$(date -u +%s)-$(wc -l < "$output_root/attempt-ledger.tsv")"
{
    lscpu
    free -h
    cat /proc/swaps
    cat /proc/self/cgroup
    taskset -pc $$ 2>&1 || true
    findmnt -T "$data_root" -o TARGET,SOURCE,FSTYPE,OPTIONS
    if [[ "$mount_fstype" == zfs ]]; then
        zfs get -Hp sync,recordsize,compression,atime "$mount_source" 2>&1 || true
    else
        printf 'ZFS ARC statistics unavailable: data root is not on ZFS\n'
    fi
    cat /proc/pressure/cpu /proc/pressure/memory /proc/pressure/io 2>&1 || true
    cat /proc/spl/kstat/zfs/arcstats 2>&1 || true
    command -v arcstat >/dev/null 2>&1 && arcstat -f time,size,c,hit%,miss% 1 1 2>&1 || true
    ps -eo pid,ppid,comm,%cpu,%mem,rss,etimes --sort=-%cpu | head -n 30
} > "$output_root/environment-before-$snapshot_id.txt"
engines=(main-btree planned-blink rocksdb)
conditions=()
if [[ "$suite" == smoke ]]; then
    warmup_ms=500
    duration_ms=1500
    conditions=(get-c16 query-c16 mixed-50-c16-w1)
elif [[ "$suite" == core ]]; then
    warmup_ms=2000
    duration_ms=10000
    conditions=(get-c1 get-c16 query-c1 query-c16 write-c16-w1 write-c16-w4 write-c16-w8 mixed-95-c16-w1 mixed-50-c16-w1 mixed-20-c16-w1 mixed-95-c16-w4 mixed-95-c16-w8)
elif [[ "$suite" == sustained ]]; then
    warmup_ms=10000
    duration_ms=120000
    conditions=(mixed-50-c16-w1 mixed-95-c16-w8)
elif [[ "$suite" == scale ]]; then
    warmup_ms=2000
    duration_ms=10000
    IFS=, read -r -a scale_counts <<< "${CROSSDB_SCALE_CLIENT_COUNTS:?set CROSSDB_SCALE_CLIENT_COUNTS as comma-separated client counts}"
    for client_count in "${scale_counts[@]}"; do
        [[ "$client_count" =~ ^[1-9][0-9]*$ ]] || {
            printf 'invalid scale client count: %s\n' "$client_count" >&2
            exit 2
        }
        conditions+=("mixed-50-c${client_count}-w1" "write-c${client_count}-w1")
    done
elif [[ "$suite" == scale-sustained ]]; then
    warmup_ms=10000
    duration_ms=120000
    representative_clients=${CROSSDB_SCALE_REPRESENTATIVE_CLIENTS:?set CROSSDB_SCALE_REPRESENTATIVE_CLIENTS}
    [[ "$representative_clients" =~ ^[1-9][0-9]*$ ]] || exit 2
    conditions=("mixed-50-c${representative_clients}-w1" "mixed-95-c${representative_clients}-w8")
else
    printf 'unknown suite: %s\n' "$suite" >&2
    exit 2
fi

run_number=$(date -u +%s%N)
run_one() {
    local condition=$1
    local repetition=$2
    local engine_index=$3
    local engine=${engines[$engine_index]}
    if awk -F '\t' -v target_engine="$engine" -v target_condition="$condition" -v target_repetition="$repetition" '$2 == target_engine && $3 == target_condition && $4 == target_repetition && $7 == "passed" { found = 1 } END { exit !found }' "$output_root/attempt-ledger.tsv"; then
        return 0
    fi
    local operation=write
    local read_percent=0
    local writers=16
    local width=1
    local dodb_workers=2
    local scenario_index=0
    case "$condition" in
        get-c1) operation=get; writers=1; scenario_index=1 ;;
        get-c16) operation=get; scenario_index=2 ;;
        query-c1) operation=query; writers=1; scenario_index=3 ;;
        query-c16) operation=query; scenario_index=4 ;;
        write-c16-w1) operation=write; scenario_index=5 ;;
        write-c16-w4) operation=write; width=4; scenario_index=6 ;;
        write-c16-w8) operation=write; width=8; scenario_index=7 ;;
        mixed-95-c16-w1) operation=mixed; read_percent=95; scenario_index=8 ;;
        mixed-50-c*-w1) operation=mixed; read_percent=50; writers=${condition#mixed-50-c}; writers=${writers%-w1}; scenario_index=9 ;;
        mixed-20-c16-w1) operation=mixed; read_percent=20; scenario_index=10 ;;
        mixed-95-c16-w4) operation=mixed; read_percent=95; width=4; scenario_index=11 ;;
        mixed-95-c*-w8) operation=mixed; read_percent=95; width=8; writers=${condition#mixed-95-c}; writers=${writers%-w8}; scenario_index=12 ;;
        write-c*-w1) operation=write; writers=${condition#write-c}; writers=${writers%-w1}; scenario_index=5 ;;
        *) printf 'unknown condition: %s\n' "$condition" >&2; exit 2 ;;
    esac
    if [[ "$suite" == scale || "$suite" == scale-sustained ]]; then
        dodb_workers=$writers
    fi
    local seed=$((979000000 + scenario_index * 1009 + repetition))
    local run_id=$(printf '%03d-%s-r%d-%s' "$run_number" "$condition" "$repetition" "$engine")
    run_number=$((run_number + 1))
    local data_dir="$output_root/db/$run_id"
    local output_file="$output_root/raw-$engine.jsonl"
    local binary
    local engine_args=()
    case "$engine" in
        main-btree) binary=$main_binary; engine_args+=(--dodb-engine main-btree --dodb-workers "$dodb_workers") ;;
        planned-blink) binary=$planned_binary; engine_args+=(--dodb-engine planned-blink --dodb-workers "$dodb_workers") ;;
        rocksdb) binary=$rocks_binary; engine_args+=(--pipelined off) ;;
    esac
    printf 'suite=%s run=%s engine=%s condition=%s seed=%s start=%s\n' "$suite" "$run_id" "$engine" "$condition" "$seed" "$(date -u +%FT%TZ)" | tee "$output_root/logs/$run_id.log"
    local process_exit=0
    if "$binary" \
        --mode bench \
        --operation "$operation" \
        --read-percent "$read_percent" \
        --read-limit 16 \
        --writers "$writers" \
        --width "$width" \
        --distribution uniform \
        --working-set 10000 \
        --key-size 16 \
        --value-size 512 \
        --warmup-ms "$warmup_ms" \
        --duration-ms "$duration_ms" \
        --window-ms "$(if [[ "$suite" == sustained ]]; then printf 10000; else printf 0; fi)" \
        --monitor-ms 1000 \
        --seed "$seed" \
        --scenario-index "$scenario_index" \
        --repetition "$repetition" \
        --data-dir "$data_dir" \
        --output "$output_file" \
        "${engine_args[@]}" 2>&1 | tee -a "$output_root/logs/$run_id.log"; then
        process_exit=0
    else
        process_exit=$?
    fi
    if [[ "$process_exit" -ne 0 ]]; then
        printf '%s\t%s\t%s\t%s\t%s\t%s\tprocess_failed\n' "$run_id" "$engine" "$condition" "$repetition" "$seed" "$process_exit" >> "$output_root/attempt-ledger.tsv"
        return "$process_exit"
    fi
    if python3 - "$output_file" "$source_sha" "$engine" "$condition" "$dodb_workers" <<'PY'
import json
import sys

path, source_sha, engine, condition, dodb_workers = sys.argv[1:]
with open(path, encoding="utf-8") as source:
    records = [json.loads(line) for line in source if line.strip()]
record = records[-1]
assert record.get("record_type") == "crossdb-run"
assert record.get("engine") == engine
assert record.get("build", {}).get("source_sha") == source_sha
assert record.get("build", {}).get("source_clean") == "true"
assert record.get("verification", {}).get("passed") is True
measured = record.get("measured", {})
assert measured.get("failed_reads") == 0
assert measured.get("failed_write_transactions") == 0
if condition.startswith("mixed-"):
    assert record["verification"]["mixed_read_history"]["status"] == "passed"
settings = record.get("settings", {})
effective = settings.get("effective", {})
if engine == "rocksdb":
    assert effective.get("write_options_sync") is True
    assert effective.get("write_options_disable_wal") is False
    assert effective.get("rocksdb_version") == "11.8.1"
else:
    assert effective.get("sync_mode") == "ProductionFile real sync"
    assert effective.get("tokio_runtime_workers") == int(dodb_workers)
    if engine == "planned-blink":
        assert effective.get("parallel_execution") is False
        assert effective.get("borrowed_page_views") is False
print("run validation passed")
PY
    then
        printf '%s\t%s\t%s\t%s\t%s\t0\tpassed\n' "$run_id" "$engine" "$condition" "$repetition" "$seed" >> "$output_root/attempt-ledger.tsv"
    else
        printf '%s\t%s\t%s\t%s\t%s\t0\tvalidation_failed\n' "$run_id" "$engine" "$condition" "$repetition" "$seed" >> "$output_root/attempt-ledger.tsv"
        return 1
    fi
}

if [[ "$suite" == smoke ]]; then
    for condition in "${conditions[@]}"; do
        for repetition in 1; do
            for engine_index in 0 1 2; do
                run_one "$condition" "$repetition" "$engine_index"
            done
        done
    done
else
    for condition_index in "${!conditions[@]}"; do
        condition=${conditions[$condition_index]}
        for repetition in 1 2 3; do
            for rotation in 0 1 2; do
                engine_index=$(((condition_index + repetition - 1 + rotation) % 3))
                run_one "$condition" "$repetition" "$engine_index"
            done
        done
    done
fi
if [[ "$suite" == smoke ]]; then
    planned_runs=$((${#conditions[@]} * 3))
else
    planned_runs=$((${#conditions[@]} * 9))
fi
completed_runs=$(awk -F '\t' '$7 == "passed" { count += 1 } END { print count + 0 }' "$output_root/attempt-ledger.tsv")
printf 'suite=%s planned_runs=%s completed_runs=%s attempts=%s end=%s\n' "$suite" "$planned_runs" "$completed_runs" "$run_number" "$(date -u +%FT%TZ)" | tee -a "$output_root/logs/SUITE.log"
{
    date -u +%FT%TZ
    free -h
    df -h "$data_root"
    zpool list -o name,size,alloc,free,capacity,health 2>&1 || true
    cat /proc/pressure/cpu /proc/pressure/memory /proc/pressure/io 2>&1 || true
    cat /proc/spl/kstat/zfs/arcstats 2>&1 || true
    command -v arcstat >/dev/null 2>&1 && arcstat -f time,size,c,hit%,miss% 1 1 2>&1 || true
    ps -eo pid,ppid,comm,%cpu,%mem,rss,etimes --sort=-%cpu | head -n 30
} > "$output_root/environment-after-$snapshot_id.txt"
checksum_manifest="$output_root/SHA256SUMS-$snapshot_id"
find "$output_root" -type f ! -name 'SHA256SUMS-*' ! -name 'checksum-verification-*' -print0 | sort -z | xargs -0 sha256sum > "$checksum_manifest"
sha256sum -c "$checksum_manifest" > "$output_root/checksum-verification-$snapshot_id.txt"
sha256sum "$output_root/checksum-verification-$snapshot_id.txt" > "$output_root/checksum-verification-$snapshot_id.sha256"
