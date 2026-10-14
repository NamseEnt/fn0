#!/usr/bin/env bash
set -u

artifact_root="/bench/zfs/db/dodb-read-fastpath-b4325ee98"
baseline_sha="de790c685dee9551170d6094c945ff03ca762ac4"
fast_sha="b4325ee98be07ed7efe10119bf4e7b7c997a2b25"
baseline_binary="$artifact_root/binaries/service_workload-$baseline_sha"
fast_binary="$artifact_root/binaries/service_workload-$fast_sha"
runs_root="$artifact_root/oci/runs"
failed_runs=0

run_one() {
  local run_name="$1"
  local version="$2"
  local scenario="$3"
  local rows="$4"
  local warmup_ms="$5"
  local measurement_ms="$6"
  local binary
  local source_sha
  local run_dir
  local data_dir
  local report_path
  local run_status
  if [[ "$version" == baseline ]]; then
    binary="$baseline_binary"
    source_sha="$baseline_sha"
  else
    binary="$fast_binary"
    source_sha="$fast_sha"
  fi
  run_dir="$runs_root/$run_name"
  data_dir="$run_dir/db"
  report_path="$run_dir/report.json"
  if [[ -e "$run_dir" ]]; then
    printf 'run directory already exists: %s\n' "$run_dir" >&2
    return 2
  fi
  mkdir -p "$run_dir"
  sha256sum "$binary" > "$run_dir/binary-before.sha256"
  {
    printf 'source_sha=%s\n' "$source_sha"
    printf 'binary=%s\n' "$binary"
    printf 'command='
    printf '%q ' "$binary" --backend rocksdb --scenario "$scenario" --seed 20261013 --warmup-ms "$warmup_ms" --measurement-ms "$measurement_ms" --clients 16 --rows "$rows" --value-size 512 --call-cap 1000000 --revision-allocator reserved-ranges --research-metrics 0 --read-stage-metrics 0 --data-dir "$data_dir" --report "$report_path"
    printf '\n'
  } > "$run_dir/command.txt"
  set +e
  "$binary" --backend rocksdb --scenario "$scenario" --seed 20261013 --warmup-ms "$warmup_ms" --measurement-ms "$measurement_ms" --clients 16 --rows "$rows" --value-size 512 --call-cap 1000000 --revision-allocator reserved-ranges --research-metrics 0 --read-stage-metrics 0 --data-dir "$data_dir" --report "$report_path" > "$run_dir/stdout.log" 2>&1
  run_status=$?
  set -e
  printf '%s\n' "$run_status" > "$run_dir/exit-status.txt"
  sha256sum "$binary" > "$run_dir/binary-after.sha256"
  if [[ "$run_status" -ne 0 ]]; then
    failed_runs=$((failed_runs + 1))
    printf 'failed %s status=%s\n' "$run_name" "$run_status"
  else
    printf 'passed %s\n' "$run_name"
  fi
}

run_one baseline-rw95 baseline rw95 10000 1000 5000
run_one fast-rw95 fast rw95 10000 1000 5000
run_one fast-rw80 fast rw80 10000 1000 5000
run_one baseline-rw80 baseline rw80 10000 1000 5000
run_one baseline-rw50 baseline rw50 10000 1000 5000
run_one fast-rw50 fast rw50 10000 1000 5000
run_one fast-rw20 fast rw20 10000 1000 5000
run_one baseline-rw20 baseline rw20 10000 1000 5000
run_one baseline-mt-mix baseline mt-mix 10000 1000 5000
run_one fast-mt-mix fast mt-mix 10000 1000 5000
printf 'failed_runs=%s\n' "$failed_runs"
[[ "$failed_runs" -eq 0 ]]
