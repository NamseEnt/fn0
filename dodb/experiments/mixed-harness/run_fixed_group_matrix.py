import argparse
import hashlib
import json
import os
from pathlib import Path
import statistics
import subprocess
import time


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def git(repository, *arguments):
    return subprocess.check_output(["git", *arguments], cwd=repository, text=True).strip()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--results", required=True, type=Path)
    parser.add_argument("--data-root", required=True, type=Path)
    parser.add_argument("--binary", required=True, type=Path)
    parser.add_argument("--groups", default="4,16,64")
    parser.add_argument("--sync-modes", default="real,disabled")
    parser.add_argument("--repetitions", type=int, default=3)
    parser.add_argument("--warmup-ms", type=int, default=2000)
    parser.add_argument("--duration-ms", type=int, default=5000)
    args = parser.parse_args()
    repository = Path(__file__).resolve().parents[3]
    source = git(repository, "rev-parse", "HEAD")
    if git(repository, "status", "--porcelain", "--untracked-files=no"):
        raise RuntimeError("tracked source must be clean")
    groups = [int(value) for value in args.groups.split(",")]
    modes = args.sync_modes.split(",")
    if not groups or any(value not in (4, 16, 64) for value in groups):
        parser.error("groups must contain 4, 16, or 64")
    if not modes or any(value not in ("real", "disabled") for value in modes):
        parser.error("sync-modes must contain real or disabled")
    if args.repetitions < 1 or args.duration_ms < 1 or args.warmup_ms < 0:
        parser.error("invalid repetitions or duration")
    args.results = args.results.resolve()
    args.data_root = args.data_root.resolve()
    args.binary = args.binary.resolve()
    args.results.mkdir(parents=True, exist_ok=False)
    args.data_root.mkdir(parents=True, exist_ok=False)
    binary_hash = digest(args.binary)
    variants = [("planned-blink", 0), ("parallel-blink", 1), ("parallel-blink", 2)]
    metadata = {
        "source_commit": source, "binary": str(args.binary), "binary_sha256": binary_hash,
        "groups": groups, "sync_modes": modes, "repetitions": args.repetitions,
        "warmup_ms": args.warmup_ms, "duration_ms": args.duration_ms,
        "value_mode": "changing", "logical_cpus": os.cpu_count(),
        "hostname": os.uname().nodename, "architecture": os.uname().machine,
        "planned_count": len(groups) * len(modes) * args.repetitions * len(variants),
        "accepted": False, "interpretation": "Fixed-group write diagnostic; disabled sync is not durable",
    }
    metadata_path = args.results / "manifest.json"
    metadata_path.write_text(json.dumps(metadata, indent=2) + "\n")
    records = []
    traces = {}
    with (args.results / "run-order.jsonl").open("x") as order:
        for repetition in range(1, args.repetitions + 1):
            for group in groups:
                for mode in modes:
                    offset = (repetition + group + (mode == "disabled")) % len(variants)
                    for engine, workers in variants[offset:] + variants[:offset]:
                        stem = f"g{group}-{mode}-{engine}-w{workers}-r{repetition}"
                        output = args.results / f"{stem}.jsonl"
                        command = [str(args.binary), "--engine", engine, "--workers", str(workers),
                                   "--group-size", str(group), "--sync-mode", mode,
                                   "--value-mode", "changing", "--seed", str(979000000 + repetition),
                                   "--repetition", str(repetition), "--warmup-ms", str(args.warmup_ms),
                                   "--duration-ms", str(args.duration_ms), "--data-dir", str(args.data_root / stem),
                                   "--output", str(output)]
                        order.write(json.dumps({"command": command, "started_unix": time.time()}) + "\n")
                        order.flush()
                        print(stem, flush=True)
                        with (args.results / f"{stem}.log").open("x") as log:
                            subprocess.run(command, cwd=repository, stdout=log, stderr=subprocess.STDOUT, check=True)
                        lines = output.read_text().splitlines()
                        if len(lines) != 1:
                            raise RuntimeError("expected exactly one raw row")
                        record = json.loads(lines[0])
                        expected = {"source_git_commit": source, "release": True, "engine": engine,
                                    "workers": workers, "group_size": group, "sync_mode": mode,
                                    "value_mode": "changing", "seed": 979000000 + repetition,
                                    "repetition": repetition, "trace_prefix_operations": 1000}
                        for field, value in expected.items():
                            if record.get(field) != value:
                                raise RuntimeError(f"raw {field} mismatch: {record.get(field)} != {value}")
                        if record["measured_transactions"] != record["measured_groups"] * group:
                            raise RuntimeError("group size mismatch")
                        if record["wal_syncs"] != record["measured_groups"]:
                            raise RuntimeError("WAL sync call count mismatch")
                        prefix = record["trace_prefix_hash"]
                        if traces.setdefault(repetition, prefix) != prefix:
                            raise RuntimeError("request prefix differs between variants")
                        records.append(record)
    if git(repository, "rev-parse", "HEAD") != source or digest(args.binary) != binary_hash:
        raise RuntimeError("source or binary changed")
    if git(repository, "status", "--porcelain", "--untracked-files=no"):
        raise RuntimeError("tracked source changed")
    summary = []
    for group in groups:
        for mode in modes:
            for engine, workers in variants:
                rows = [row for row in records if (row["group_size"], row["sync_mode"], row["engine"], row["workers"]) == (group, mode, engine, workers)]
                rates = [row["transactions_per_second"] for row in rows]
                summary.append({"group_size": group, "sync_mode": mode, "engine": engine, "workers": workers,
                                "tx_per_second_median": statistics.median(rates), "tx_per_second_min": min(rates),
                                "tx_per_second_max": max(rates),
                                "wal_bytes_per_tx_median": statistics.median(row["wal_bytes"] / row["measured_transactions"] for row in rows),
                                "wal_sync_mean_ms_median": statistics.median(row["wal_sync_nanos"] / row["wal_syncs"] / 1000000 for row in rows)})
    (args.results / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    metadata.update(accepted=True, completed_count=len(records))
    metadata_path.write_text(json.dumps(metadata, indent=2) + "\n")
    paths = sorted(path for path in args.results.iterdir() if path.is_file())
    (args.results / "SHA256SUMS").write_text("".join(f"{digest(path)}  {path.name}\n" for path in paths))


if __name__ == "__main__":
    main()
