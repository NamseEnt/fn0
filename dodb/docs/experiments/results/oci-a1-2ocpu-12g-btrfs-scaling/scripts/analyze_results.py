import json
import hashlib
import pathlib
import statistics


ROOT = pathlib.Path(__file__).resolve().parents[1]
RAW = ROOT / "raw"
WORKLOADS = (
    "width1-uniform",
    "width16-uniform",
    "width16-compact",
    "width16-spread",
)
STAGES = ("2", "4", "6", "2-post")


def read_rows(stage, engine, workload):
    paths = sorted(RAW.glob(f"{stage}-{engine}-{workload}-rep*.jsonl"))
    rows = []
    for path in paths:
        lines = [line for line in path.read_text(encoding="utf-8").splitlines() if line.strip()]
        if len(lines) != 1:
            raise RuntimeError(f"expected exactly one JSON row in {path}, found {len(lines)}")
        rows.append(json.loads(lines[0]))
    if len(rows) != 3:
        raise RuntimeError(f"expected three repetitions for {stage}/{engine}/{workload}, found {len(rows)}")
    return rows


def median(rows, getter):
    return statistics.median(float(getter(row)) for row in rows)


def summarize(stage, engine, workload):
    rows = read_rows(stage, engine, workload)
    if engine == "dodb":
        metrics = {
            "tx_s": median(rows, lambda row: row["logical_tx_per_second"]),
            "p50_us": median(rows, lambda row: row["e2e_p50_us"]),
            "p95_us": median(rows, lambda row: row["e2e_p95_us"]),
            "p99_us": median(rows, lambda row: row["e2e_p99_us"]),
            "cpu_seconds": median(rows, lambda row: row["cpu_utilization_percent_one_core"] * row["duration_ms"] / 100_000),
            "cpu_machine_percent": median(rows, lambda row: row["cpu_utilization_percent_machine"]),
            "worker_parallelism": median(rows, lambda row: row["effective_worker_parallelism"]),
            "group_size": median(rows, lambda row: row["avg_transactions_per_group"]),
            "groups_per_second": median(rows, lambda row: row["groups"] / (row["duration_ms"] / 1000)),
            "syncs_per_second": median(rows, lambda row: row["wal_syncs_delta"] / (row["duration_ms"] / 1000)),
            "wal_sync_mean_us": median(rows, lambda row: row["wal_sync_nanos_total"] / row["wal_syncs_delta"] / 1000),
            "wal_sync_total_ms": median(rows, lambda row: row["wal_sync_nanos_total"] / 1_000_000),
            "physical_ms": median(rows, lambda row: row["physical_execution_nanos"] / 1_000_000),
            "planning_ms": median(rows, lambda row: row["planning_nanos"] / 1_000_000),
            "validation_ms": median(rows, lambda row: row["validation_nanos_total"] / 1_000_000),
            "wal_assembly_ms": median(rows, lambda row: row["wal_assembly_nanos_total"] / 1_000_000),
            "publication_total_ms": median(rows, lambda row: row["publication_nanos_total"] / 1_000_000),
            "state_install_ms": median(rows, lambda row: row["state_install_nanos_total"] / 1_000_000),
            "generation_publication_ms": median(rows, lambda row: row["generation_publication_nanos"] / 1_000_000),
            "publication_swap_ms": median(rows, lambda row: row["publication_swap_nanos_total"] / 1_000_000),
            "wal_sync_count": median(rows, lambda row: row["wal_syncs_delta"]),
        }
    else:
        metrics = {
            "tx_s": median(rows, lambda row: row["measured"]["logical_tx_per_second"]),
            "p50_us": median(rows, lambda row: row["measured"]["p50_us"]),
            "p95_us": median(rows, lambda row: row["measured"]["p95_us"]),
            "p99_us": median(rows, lambda row: row["measured"]["p99_us"]),
            "cpu_seconds": median(rows, lambda row: row["cpu_seconds"]),
            "cpu_machine_percent": median(rows, lambda row: row["cpu_utilization_percent_machine"]),
            "worker_parallelism": median(rows, lambda row: row["cpu_seconds"] / row["wall_seconds"]),
            "syncs_per_second": median(rows, lambda row: row["metrics_after"]["wal_sync_statistics"]["wal_file_syncs"] / row["wall_seconds"]),
            "avg_tx_per_sync": median(rows, lambda row: row["measured"]["logical_tx_per_second"] * row["wall_seconds"] / row["metrics_after"]["wal_sync_statistics"]["wal_file_syncs"]),
            "wal_sync_mean_us": median(rows, lambda row: row["metrics_after"]["wal_sync_statistics"]["wal_file_sync_latency_us"]["mean"]),
            "wal_sync_p50_us": median(rows, lambda row: row["metrics_after"]["wal_sync_statistics"]["wal_file_sync_latency_us"]["median"]),
            "wal_sync_p95_us": median(rows, lambda row: row["metrics_after"]["wal_sync_statistics"]["wal_file_sync_latency_us"]["p95"]),
            "wal_sync_p99_us": median(rows, lambda row: row["metrics_after"]["wal_sync_statistics"]["wal_file_sync_latency_us"]["p99"]),
        }
    return metrics


def fio_summary(stage):
    result = json.loads((RAW / f"fio-{stage}.json").read_text(encoding="utf-8"))
    job = result["jobs"][0]
    histogram = job["sync"]["lat_ns"]
    percentiles = histogram["percentile"]
    return {
        "iops": float(job["write"]["iops"]),
        "mb_s": float(job["write"]["bw_bytes"]) / 1_000_000,
        "sync_count": int(histogram["N"]),
        "sync_mean_ms": float(histogram["mean"]) / 1_000_000,
        "sync_p50_ms": float(percentiles["50.000000"]) / 1_000_000,
        "sync_p95_ms": float(percentiles["95.000000"]) / 1_000_000,
        "sync_p99_ms": float(percentiles["99.000000"]) / 1_000_000,
    }


def io_event_medians():
    summaries = {}
    raw_hashes = {
        str(path): hashlib.sha256(path.read_bytes()).hexdigest()
        for path in RAW.glob("*.jsonl")
    }
    for event_path in RAW.rglob("run-events.jsonl"):
        for line in event_path.read_text(encoding="utf-8").splitlines():
            if not line.strip():
                continue
            event = json.loads(line)
            if event.get("event") != "run_end" or not event.get("json_output"):
                continue
            output_path = event["json_output"]
            name = pathlib.Path(output_path).name
            local_path = RAW / name
            if not local_path.is_file():
                continue
            if event.get("json_sha256") != raw_hashes.get(str(local_path)):
                continue
            summaries.setdefault(name, []).append(event)
    return {
        name: {
            "disk_bytes_written": statistics.median(event["disk_bytes_written"] for event in events),
            "block_device_busy_ms": statistics.median(event["block_device_busy_ms"] for event in events),
        }
        for name, events in summaries.items()
    }


def main():
    summary = {"stages": {}, "fio": {}, "io": io_event_medians()}
    for stage in STAGES:
        workload_names = WORKLOADS[:2] if stage == "2-post" else WORKLOADS
        summary["stages"][stage] = {
            workload: {
                engine: summarize(stage, engine, workload)
                for engine in ("dodb", "rocksdb")
            }
            for workload in workload_names
        }
        summary["fio"][stage] = fio_summary(stage)

    baseline = summary["stages"]["2"]
    for stage in ("4", "6", "2-post"):
        for workload, engines in summary["stages"][stage].items():
            for engine, metrics in engines.items():
                metrics["speedup_vs_2"] = metrics["tx_s"] / baseline[workload][engine]["tx_s"]
            metrics = engines["dodb"]
            metrics["dodb_rocks_ratio"] = metrics["tx_s"] / engines["rocksdb"]["tx_s"]

    (ROOT / "summary.json").write_text(json.dumps(summary, indent=2, sort_keys=True) + "\n", encoding="utf-8")

    lines = [
        "# A1 Btrfs durable-write scaling tables",
        "",
        "All throughput, latency, CPU, group, and timing columns are medians across three repetitions. Latencies are end-to-end transaction latencies. `worker parallelism` is dodb's measured effective worker parallelism or RocksDB process CPU-seconds divided by wall-seconds.",
        "",
        "## Throughput, speedup, and dodb/RocksDB ratio",
        "",
        "| OCPU | Workload | dodb tx/s | dodb speedup | RocksDB tx/s | RocksDB speedup | dodb / RocksDB |",
        "|---:|---|---:|---:|---:|---:|---:|",
    ]
    for stage in STAGES:
        for workload, engines in summary["stages"][stage].items():
            dodb = engines["dodb"]
            rocksdb = engines["rocksdb"]
            lines.append(
                f"| {stage} | {workload} | {dodb['tx_s']:.1f} | {dodb.get('speedup_vs_2', 1.0):.3f}x | {rocksdb['tx_s']:.1f} | {rocksdb.get('speedup_vs_2', 1.0):.3f}x | {dodb['tx_s'] / rocksdb['tx_s']:.3f}x |"
            )
    lines.extend([
        "",
        "## CPU and durability pipeline metrics",
        "",
        "| OCPU | Workload | Engine | CPU s | CPU % of VM | Worker parallelism | p50/p95/p99 us | Avg tx/sync group | Groups/syncs per s | WAL sync mean/p50/p95/p99 us |",
        "|---:|---|---|---:|---:|---:|---|---:|---:|---|",
    ])
    for stage in STAGES:
        for workload, engines in summary["stages"][stage].items():
            for engine in ("dodb", "rocksdb"):
                metrics = engines[engine]
                group_size = f"{metrics['group_size']:.2f}" if engine == "dodb" else f"{metrics['avg_tx_per_sync']:.2f}"
                groups_per_second = metrics["groups_per_second"] if engine == "dodb" else metrics["syncs_per_second"]
                syncs_per_second = metrics["syncs_per_second"]
                if engine == "dodb":
                    sync_latency = f"{metrics['wal_sync_mean_us']:.0f}/—/—/—"
                else:
                    sync_latency = f"{metrics['wal_sync_mean_us']:.0f}/{metrics['wal_sync_p50_us']:.0f}/{metrics['wal_sync_p95_us']:.0f}/{metrics['wal_sync_p99_us']:.0f}"
                lines.append(
                    f"| {stage} | {workload} | {engine} | {metrics['cpu_seconds']:.2f} | {metrics['cpu_machine_percent']:.1f}% | {metrics['worker_parallelism']:.2f} | {metrics['p50_us']:.0f}/{metrics['p95_us']:.0f}/{metrics['p99_us']:.0f} | {group_size} | {groups_per_second:.1f} | {sync_latency} |"
                )
    lines.extend([
        "",
        "## dodb physical work, commit, and block I/O",
        "",
        "All timing columns are total milliseconds per five-second measurement window, except WAL sync count, which is the total count in that window. Block bytes and busy time are deltas from the guest block-device counters during each run.",
        "",
        "| OCPU | Workload | Physical | Planning | Validation | WAL assembly | Publication total | State install | Generation publish | Publish swap | WAL sync count | WAL sync total ms | Disk bytes written | Block busy ms |",
        "|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|",
    ])
    for stage in STAGES:
        for workload, engines in summary["stages"][stage].items():
            metrics = engines["dodb"]
            label = f"{stage}-dodb-{workload}-rep1.jsonl"
            io = summary["io"].get(label, {})
            lines.append(
                f"| {stage} | {workload} | {metrics['physical_ms']:.1f} | {metrics['planning_ms']:.1f} | {metrics['validation_ms']:.1f} | {metrics['wal_assembly_ms']:.1f} | {metrics['publication_total_ms']:.1f} | {metrics['state_install_ms']:.1f} | {metrics['generation_publication_ms']:.1f} | {metrics['publication_swap_ms']:.1f} | {metrics['wal_sync_count']:.0f} | {metrics['wal_sync_total_ms']:.1f} | {io.get('disk_bytes_written', 0):.0f} | {io.get('block_device_busy_ms', 0):.0f} |"
            )
    lines.extend([
        "",
        "## Fio WAL-like sequential write and fdatasync",
        "",
        "| OCPU | IOPS | MB/s | fdatasync samples | mean ms | p50 ms | p95 ms | p99 ms |",
        "|---:|---:|---:|---:|---:|---:|---:|---:|",
    ])
    for stage in STAGES:
        fio = summary["fio"][stage]
        lines.append(
            f"| {stage} | {fio['iops']:.1f} | {fio['mb_s']:.3f} | {fio['sync_count']} | {fio['sync_mean_ms']:.3f} | {fio['sync_p50_ms']:.3f} | {fio['sync_p95_ms']:.3f} | {fio['sync_p99_ms']:.3f} |"
        )
    lines.append("")
    (ROOT / "tables.md").write_text("\n".join(lines), encoding="utf-8")


if __name__ == "__main__":
    main()
