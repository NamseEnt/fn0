#!/usr/bin/env bash
set -euo pipefail
root=/bench/zfs/db/dodb-service-blocking-admission-85cadf099
binary="$root/service_workload-85cadf099"
for mode in shared split-read-write; do
  run_dir="$root/smoke/$mode"
  mkdir -p "$run_dir"
  sha256sum "$binary" > "$run_dir/binary-before.sha256"
  {
    printf source_sha=%sn 85cadf0991de2b2052f5b6f7647dc46647b4ede6
    printf binary=%sn "$binary"
    printf command=
    printf %q  "$binary" --backend rocksdb --scenario smoke --seed 20261013 --warmup-ms 1000 --measurement-ms 500 --clients 16 --rows 10000 --value-size 512 --call-cap 1000000 --revision-allocator reserved-ranges --blocking-admission "$mode" --research-metrics 0 --read-stage-metrics 0 --data-dir "$run_dir/db" --report "$run_dir/report.json"
    printf n
  } > "$run_dir/command.txt"
  set +e
  "$binary" --backend rocksdb --scenario smoke --seed 20261013 --warmup-ms 1000 --measurement-ms 500 --clients 16 --rows 10000 --value-size 512 --call-cap 1000000 --revision-allocator reserved-ranges --blocking-admission "$mode" --research-metrics 0 --read-stage-metrics 0 --data-dir "$run_dir/db" --report "$run_dir/report.json" > "$run_dir/stdout.log" 2>&1
  run_status=$?
  set -e
  printf %sn "$run_status" > "$run_dir/exit-status.txt"
  sha256sum "$binary" > "$run_dir/binary-after.sha256"
  python3 - "$run_dir/report.json" "$mode" <<'PY' > "$run_dir/assessment.txt"
import json
import sys
report = json.load(open(sys.argv[1]))
mode = sys.argv[2]
config = report["configuration"]["rocksdb"]
expected_mode = "shared-8" if mode == "shared" else "split-read-2-write-6"
assert config["blocking_admission"] == expected_mode
assert config["blocking_operation_limit"] == 8
assert all(item["passed"] for item in report["validation"].values())
assert all(item["passed"] for item in report["reopen_validation"].values())
assert report["configuration"]["initial_rows_per_tenant"] == 10000
print(f"mode={config[blocking_admission]}")
print(f"read_slots={config[read_blocking_slots]}")
print(f"write_slots={config[write_blocking_slots]}")
print(f"validation_tenants={len(report[validation])}")
print(f"reopen_validation_tenants={len(report[reopen_validation])}")
PY
  if [[ "$run_status" -ne 0 ]]; then exit "$run_status"; fi
done
