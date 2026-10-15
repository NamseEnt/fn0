#!/usr/bin/env bash
set -uo pipefail
root=/bench/zfs/db/dodb-service-blocking-admission-85cadf099
binary="$root/service_workload-85cadf099"
source_sha=85cadf0991de2b2052f5b6f7647dc46647b4ede6
failed_runs=0
run_one() {
  local mode="$1"
  local scenario="$2"
  local run_name="$3"
  local run_dir="$root/runs/$run_name"
  local data_dir="$run_dir/db"
  local report_path="$run_dir/report.json"
  local run_status
  if [[ -e "$run_dir" ]]; then
    printf run directory already exists: %sn "$run_dir" >&2
    failed_runs=$((failed_runs + 1))
    return
  fi
  mkdir -p "$run_dir"
  sha256sum "$binary" > "$run_dir/binary-before.sha256"
  {
    printf source_sha=%snmode=%snscenario=%sn "$source_sha" "$mode" "$scenario"
    printf binary=%sn "$binary"
    printf command=
    printf %q  "$binary" --backend rocksdb --scenario "$scenario" --seed 20261013 --warmup-ms 1000 --measurement-ms 5000 --clients 16 --rows 10000 --value-size 512 --call-cap 1000000 --revision-allocator reserved-ranges --blocking-admission "$mode" --research-metrics 0 --read-stage-metrics 0 --data-dir "$data_dir" --report "$report_path"
    printf n
  } > "$run_dir/command.txt"
  set +e
  "$binary" --backend rocksdb --scenario "$scenario" --seed 20261013 --warmup-ms 1000 --measurement-ms 5000 --clients 16 --rows 10000 --value-size 512 --call-cap 1000000 --revision-allocator reserved-ranges --blocking-admission "$mode" --research-metrics 0 --read-stage-metrics 0 --data-dir "$data_dir" --report "$report_path" > "$run_dir/stdout.log" 2>&1
  run_status=$?
  set -e
  printf %sn "$run_status" > "$run_dir/exit-status.txt"
  sha256sum "$binary" > "$run_dir/binary-after.sha256"
  if [[ "$run_status" -eq 0 ]]; then
    if python3 "$root/check-report.py" "$report_path" "$mode" > "$run_dir/assessment.txt" 2> "$run_dir/assessment.stderr"; then
      printf passed %s %s %sn "$mode" "$scenario" "$run_name"
    else
      printf report validation failed %s %s %sn "$mode" "$scenario" "$run_name" >&2
      failed_runs=$((failed_runs + 1))
    fi
  else
    printf workload failed %s %s %s status=%sn "$mode" "$scenario" "$run_name" "$run_status" >&2
    failed_runs=$((failed_runs + 1))
  fi
}
run_one shared rw95 shared-rw95
run_one split-read-write rw95 split-rw95
run_one split-read-write rw80 split-rw80
run_one shared rw80 shared-rw80
run_one shared rw50 shared-rw50
run_one split-read-write rw50 split-rw50
run_one split-read-write rw20 split-rw20
run_one shared rw20 shared-rw20
run_one shared mt-mix shared-mt-mix
run_one split-read-write mt-mix split-mt-mix
printf planned_runs=10nfailed_runs=%sn "$failed_runs"
[[ "$failed_runs" -eq 0 ]]
