import argparse
import json
import tarfile
from pathlib import Path


parser = argparse.ArgumentParser()
parser.add_argument("archive", type=Path)
parser.add_argument("--output", type=Path, required=True)
options = parser.parse_args()
latency_fields = {"count", "sample_count", "p50_nanos", "p95_nanos", "p99_nanos", "p99_9_nanos",
                  "p999_nanos", "maximum_nanos", "minimum_nanos"}


def thin_latency(value):
    if isinstance(value, dict):
        if "count" in value and "sample_count" in value:
            return {key: metric for key, metric in value.items() if key in latency_fields}
        return {key: thin_latency(metric) for key, metric in value.items() if isinstance(metric, dict)}
    return value


with tarfile.open(options.archive, "r:gz") as archive:
    raw = json.load(archive.extractfile("main/profile-summary.json"))
    output = {"flamegraph_revision": raw["flamegraph_revision"], "inclusive_percentages_overlap": True,
              "cases": {}, "profiling_perturbation": {}}
    for name, case in raw["cases"].items():
        report = json.load(archive.extractfile(f"main/{name}/workload-report.json"))
        result = {"run": case["run"], "reopen_and_validation_ms": case["reopen_and_validation_ms"],
                  "measurement_window_unix_nanos": report["measurement_window_unix_nanos"]}
        if "cpu" in case:
            result["cpu"] = {phase: {key: metric for key, metric in values.items()
                                      if key not in ["top_self", "top_inclusive"]}
                             for phase, values in case["cpu"].items()}
            needles = ["append_group_inner", "plan_redo", "read_shard", "BasePackJob>::run", "BlinkCheckpointSnapshot>::run",
                       "TransactionBaseImpl::GetForUpdate", "PessimisticTransaction::Commit", "PessimisticTransaction::TryLock"]
            result["selected_functions"] = {
                needle: next(({"samples": row["samples"], "percent": row["percent"]}
                              for row in case["cpu"]["measurement"]["top_inclusive"] if needle in row["symbol"]), None)
                for needle in needles}
            result["waits"] = {key: metric for key, metric in case["waits"].items()
                               if key != "top_inclusive_kernel_stacks"}
            fold = archive.extractfile(f"main/{name}/cpu/measurement.folded").read().decode()
            shard_lookup = {"total_samples": 0, "metadata_samples": 0, "database_path_samples": 0}
            for line in fold.splitlines():
                stack, count = line.rsplit(" ", 1)
                count = int(count)
                if "read_shard" in stack:
                    shard_lookup["total_samples"] += count
                    if any(term in stack for term in ["statx", "fstat", "vfs_stat", "filename_lookup", "metadata", "try_exists"]):
                        shard_lookup["metadata_samples"] += count
                if "database_path" in stack:
                    shard_lookup["database_path_samples"] += count
            result["shard_lookup"] = shard_lookup
        if case.get("latency"):
            result["latency"] = thin_latency(case["latency"])
        if case.get("per_tenant"):
            result["per_tenant_latency"] = {tenant: thin_latency(tenant_value["counters"]["latency"])
                                             for tenant, tenant_value in case["per_tenant"].items()}
        checkpoints = report.get("blink_checkpoint_metrics", {}).get("checkpoint_events", [])
        window = report["measurement_window_unix_nanos"]
        result["fully_in_window_checkpoints"] = sum(event["succeeded"] and int(window["start"]) <= int(event["started_unix_nanos"])
                                                     and int(event["ended_unix_nanos"]) <= int(window["last_client_completion"])
                                                     for event in checkpoints)
        output["cases"][name] = result
    for workload in ["RW80", "E"]:
        for backend in ["blink", "minimal-native"]:
            control = raw["cases"][f"{workload}-{backend}-control"]["run"]["throughput_per_second"]
            profile = raw["cases"][f"{workload}-{backend}-profile"]["run"]["throughput_per_second"]
            output["profiling_perturbation"][f"{workload}-{backend}"] = (profile / control - 1) * 100
options.output.write_text(json.dumps(output, indent=2) + "\n")
print(json.dumps({"output": str(options.output), "bytes": options.output.stat().st_size,
                  "profiling_perturbation": output["profiling_perturbation"]}, indent=2))
