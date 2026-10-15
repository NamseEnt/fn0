import json
import sys
report = json.load(open(sys.argv[1]))
mode = sys.argv[2]
configuration = report["configuration"]
rocksdb = configuration["rocksdb"]
expected_mode = "shared-8" if mode == "shared" else "split-read-2-write-6"
assert rocksdb.get("blocking_admission") == expected_mode
assert rocksdb.get("blocking_operation_limit") == 8
assert configuration.get("initial_rows_per_tenant") == 10000
assert configuration.get("value_size_bytes") == 512
assert configuration.get("clients") == 16
assert configuration.get("research_metrics_enabled") is False
assert configuration.get("read_stage_metrics_enabled") is False
assert all(result.get("passed") for result in report["validation"].values())
assert all(result.get("passed") for result in report["reopen_validation"].values())
for tenant, result in report["validation"].items():
    assert result.get("mismatch_samples") == []
print("mode=" + rocksdb.get("blocking_admission"))
print("read_slots=" + str(rocksdb.get("read_blocking_slots")))
print("write_slots=" + str(rocksdb.get("write_blocking_slots")))
print("tenant_validation_count=" + str(len(report["validation"])))
print("reopen_validation_count=" + str(len(report["reopen_validation"])))
