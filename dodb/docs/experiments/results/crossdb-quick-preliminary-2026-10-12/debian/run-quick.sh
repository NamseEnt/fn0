#!/usr/bin/env bash
set -euo pipefail
root=${1:?usage: run-quick.sh RUN_ROOT BUILD_ROOT SOURCE_ROOT}
build_root=${2:?usage: run-quick.sh RUN_ROOT BUILD_ROOT SOURCE_ROOT}
source_root=${3:?usage: run-quick.sh RUN_ROOT BUILD_ROOT SOURCE_ROOT}
source_sha=$(git -C "$source_root" rev-parse HEAD)
[[ -z "$(git -C "$source_root" status --porcelain)" ]]
mkdir -p "$root"
[[ "$(findmnt -n -o FSTYPE --target "$root")" == ext4 ]]
mkdir -p "$root/logs" "$root/db" "$root/raw"
export CROSSDB_DATA_ROOT="$root/db"
main_binary="$build_root/target-dodb/release/dodb-bench"
rocks_binary="$build_root/target-rocksdb/release/rocksdb-bench"
if [[ ! -e "$root/attempt-ledger.tsv" ]]; then
    printf 'run_id\tengine\tcondition\tseed\tprocess_exit\tvalidation\n' > "$root/attempt-ledger.tsv"
fi
conditions=(get-c12 query-c12 write-c12-w1 mixed-50-c12-w1)
scenarios=(2 4 5 9)
seeds=(979002019 979004037 979005046 979009082)
engines=(main-btree planned-blink rocksdb)
run_number=$(date -u +%s%N)
for condition_index in 0 1 2 3; do
    condition=${conditions[$condition_index]}
    scenario_index=${scenarios[$condition_index]}
    seed=${seeds[$condition_index]}
    operation=get
    read_percent=0
    width=1
    if [[ "$condition" == query-* ]]; then operation=query; fi
    if [[ "$condition" == write-* ]]; then operation=write; fi
    if [[ "$condition" == mixed-* ]]; then operation=mixed; read_percent=50; fi
    for rotation in 0 1 2; do
        engine_index=$(((condition_index + rotation) % 3))
        engine=${engines[$engine_index]}
        if awk -F '\t' -v target_engine="$engine" -v target_condition="$condition" '$2 == target_engine && $3 == target_condition && $6 == "passed" { found = 1 } END { exit !found }' "$root/attempt-ledger.tsv"; then
            continue
        fi
        run_id=$(printf '%03d-%s-r1-%s' "$run_number" "$condition" "$engine")
        run_number=$((run_number + 1))
        data_dir="$root/db/$run_id"
        output_file="$root/raw/$run_id.jsonl"
        log_file="$root/logs/$run_id.log"
        engine_args=()
        binary="$rocks_binary"
        if [[ "$engine" == main-btree ]]; then
            binary="$main_binary"
            engine_args+=(--dodb-engine main-btree --dodb-workers 12)
        elif [[ "$engine" == planned-blink ]]; then
            binary="$main_binary"
            engine_args+=(--dodb-engine planned-blink --dodb-workers 12)
        else
            engine_args+=(--pipelined off)
        fi
        printf 'run=%s engine=%s condition=%s seed=%s start=%s\n' "$run_id" "$engine" "$condition" "$seed" "$(date -u +%FT%TZ)" | tee "$log_file"
        process_exit=0
        if "$binary" --mode bench --operation "$operation" --read-percent "$read_percent" --read-limit 16 --writers 12 --width "$width" --distribution uniform --working-set 10000 --key-size 16 --value-size 512 --warmup-ms 1000 --duration-ms 5000 --window-ms 0 --monitor-ms 1000 --seed "$seed" --scenario-index "$scenario_index" --repetition 1 --data-dir "$data_dir" --output "$output_file" "${engine_args[@]}" 2>&1 | tee -a "$log_file"; then
            process_exit=0
        else
            process_exit=$?
        fi
        if [[ "$process_exit" -ne 0 ]]; then
            printf '%s\t%s\t%s\t%s\t%s\tprocess_failed\n' "$run_id" "$engine" "$condition" "$seed" "$process_exit" >> "$root/attempt-ledger.tsv"
            exit "$process_exit"
        fi
        if python3 - "$output_file" "$source_sha" "$engine" "$condition" "$seed" <<'PY'
import json
import sys

path, source_sha, engine, condition, expected_seed = sys.argv[1:]
with open(path, encoding="utf-8") as source:
    records = [json.loads(line) for line in source if line.strip()]
assert len(records) == 1
record = records[0]
assert record["record_type"] == "crossdb-run"
assert record["engine"] == engine
assert record["build"]["source_sha"] == source_sha
assert record["build"]["source_clean"] == "true"
assert record["seed"] == int(expected_seed)
assert record["writers"] == 12
assert record["warmup_ms"] == 1000
assert record["requested_duration_ms"] == 5000
assert record["verification"]["passed"] is True
assert record["measured"]["failed_reads"] == 0
assert record["measured"]["failed_write_transactions"] == 0
if condition.startswith("mixed-"):
    assert record["verification"]["mixed_read_history"]["status"] == "passed"
settings = record["settings"]["effective"]
if engine == "rocksdb":
    assert settings["write_options_sync"] is True
    assert settings["write_options_disable_wal"] is False
    assert settings["rocksdb_version"] == "11.8.1"
else:
    assert settings["sync_mode"] == "ProductionFile real sync"
    assert settings["tokio_runtime_workers"] == 12
    if engine == "planned-blink":
        assert settings["parallel_execution"] is False
        assert settings["borrowed_page_views"] is False
PY
        then
            printf '%s\t%s\t%s\t%s\t0\tpassed\n' "$run_id" "$engine" "$condition" "$seed" >> "$root/attempt-ledger.tsv"
        else
            printf '%s\t%s\t%s\t%s\t0\tvalidation_failed\n' "$run_id" "$engine" "$condition" "$seed" >> "$root/attempt-ledger.tsv"
            exit 1
        fi
    done
done
[[ $(awk -F '\t' '$6 == "passed" { count += 1 } END { print count + 0 }' "$root/attempt-ledger.tsv") == 12 ]]
{
    date -u +%FT%TZ
    free -h
    df -h "$root"
    cat /proc/pressure/cpu /proc/pressure/memory /proc/pressure/io
    ps -eo pid,ppid,comm,%cpu,%mem,rss,etimes --sort=-%cpu | head -n 30
} > "$root/environment-after.txt"
find "$root" -type f ! -name 'SHA256SUMS' ! -name 'checksum-verification.txt' -print0 | sort -z | xargs -0 sha256sum > "$root/SHA256SUMS"
sha256sum -c "$root/SHA256SUMS" > "$root/checksum-verification.txt"
