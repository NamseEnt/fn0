import glob
import json
import math
import os
import re
import statistics
import sys

CROSSDB = sys.argv[1]
REAL_SYNC = sys.argv[2]

DISTRIBUTIONS = ("uniform", "same-leaf-heavy", "different-leaf-heavy")
REPORT = {"uniform": "uniform", "same-leaf-heavy": "compact-locality", "different-leaf-heavy": "spread-locality"}
PRIMARY = ("turso-wal", "turso-mvcc-gc", "rocksdb")
SECONDARY = ("turso-mvcc-nogc", "rocksdb-pipelined")
LABEL = {
    "dodb": "dodb Planned",
    "turso-wal": "Turso WAL",
    "turso-mvcc-gc": "Turso MVCC GC-on",
    "turso-mvcc-nogc": "Turso MVCC GC-off",
    "rocksdb": "RocksDB",
    "rocksdb-pipelined": "RocksDB pipelined",
}


def scenarios():
    listing = []
    for writers in (16, 64):
        for width in (1, 16):
            for distribution in DISTRIBUTIONS:
                listing.append((writers, width, distribution))
    for width in (1, 16):
        listing.append((1, width, "uniform"))
    return listing


SCENARIOS = scenarios()


def read_jsonl(path):
    with open(path, encoding="utf-8") as handle:
        return [json.loads(line) for line in handle if line.strip()]


def run_order_peaks(path, key):
    peaks = {}
    if not os.path.exists(path):
        return peaks
    for record in read_jsonl(path):
        if record.get("event") == "complete":
            peaks[key(record)] = record
    return peaks


def mean(values):
    values = [value for value in values if value is not None]
    return statistics.fmean(values) if values else None


def geometric_mean(values):
    values = [value for value in values if value is not None]
    if not values or any(value <= 0 or math.isinf(value) for value in values):
        return None
    return math.exp(statistics.fmean(math.log(value) for value in values))


def load_dodb():
    runs = {}
    order = run_order_peaks(
        os.path.join(REAL_SYNC, "run-order.txt"),
        lambda record: (record["engine"], record["scenario_index"], record["repetition"]),
    )
    for path in sorted(glob.glob(os.path.join(REAL_SYNC, "raw", "*-planned-*.jsonl"))):
        name = os.path.basename(path)
        scenario_index = int(name[:2])
        repetition = int(name.split("-rep")[1].split("-")[0])
        record = [row for row in read_jsonl(path) if row.get("record_type") == "run"][0]
        peak = order.get(("Planned", scenario_index, repetition), {})
        runs.setdefault(scenario_index, []).append({
            "repetition": repetition,
            "seed": record["seed"],
            "attempted_tx_per_second": record["attempted_transactions"] / (record["duration_ms"] / 1000),
            "logical_tx_per_second": record["logical_tx_per_second"],
            "mutation_ops_per_second": record["mutation_ops_per_second"],
            "p50_us": record["e2e_p50_us"],
            "p95_us": record["e2e_p95_us"],
            "p99_us": record["e2e_p99_us"],
            "cpu_one_core": record["cpu_utilization_percent_one_core"],
            "rss_peak_kib": peak.get("rss_peak_sampled_kib"),
            "busy": 0,
            "busy_snapshot": 0,
            "conflicts": record["conflicts"],
            "errors": record["errors"] + record["overloads"],
            "retries": 0,
            "abandoned": record["attempted_transactions"] - record["successful_transactions"],
            "attempted": record["attempted_transactions"],
            "successful": record["successful_transactions"],
            "verified": None,
            "wal_syncs": record["wal_syncs_delta"],
        })
    return runs


def load_crossdb(prefix=""):
    runs = {}
    order = run_order_peaks(
        os.path.join(CROSSDB, "run-order.txt"),
        lambda record: (record["engine"], record["scenario_index"], record["repetition"]),
    )
    pattern = os.path.join(CROSSDB, "raw", f"{prefix}*.jsonl")
    for path in sorted(glob.glob(pattern)):
        name = os.path.basename(path)
        if not prefix and name.startswith("sustained-"):
            continue
        if name.endswith(".process-monitor.jsonl"):
            continue
        rows = [row for row in read_jsonl(path) if row.get("record_type") == "crossdb-run"]
        if not rows:
            continue
        record = rows[0]
        measured = record["measured"]
        engine = record["engine"]
        peak = order.get((engine, record["scenario_index"], record["repetition"]), {})
        runs.setdefault((engine, record["scenario_index"]), []).append({
            "record": record,
            "repetition": record["repetition"],
            "seed": record["seed"],
            "attempted_tx_per_second": measured["attempted_tx_per_second"],
            "logical_tx_per_second": measured["logical_tx_per_second"],
            "mutation_ops_per_second": measured["mutation_ops_per_second"],
            "p50_us": measured["p50_us"],
            "p95_us": measured["p95_us"],
            "p99_us": measured["p99_us"],
            "cpu_one_core": record.get("cpu_utilization_percent_one_core"),
            "rss_peak_kib": peak.get("rss_peak_sampled_kib"),
            "rss_growth_kib": (record["rss_kib"]["end"] or 0) - (record["rss_kib"]["after_seed"] or 0),
            "busy": measured["busy"],
            "busy_snapshot": measured["busy_snapshot"],
            "conflicts": measured["conflicts"],
            "errors": measured["errors"],
            "retries": measured["retries"],
            "abandoned": measured["abandoned_transactions"],
            "attempted": measured["attempted_transactions"],
            "successful": measured["successful_transactions"],
            "attempts": measured["attempts"],
            "verified": record["verification"]["passed"],
            "verification": record["verification"],
            "messages": measured["messages"],
        })
    return runs


def summarize(run_list):
    return {
        "repetitions": len(run_list),
        "attempted_tx_per_second": mean([run["attempted_tx_per_second"] for run in run_list]),
        "logical_tx_per_second": mean([run["logical_tx_per_second"] for run in run_list]),
        "mutation_ops_per_second": mean([run["mutation_ops_per_second"] for run in run_list]),
        "p50_us": mean([run["p50_us"] for run in run_list]),
        "p95_us": mean([run["p95_us"] for run in run_list]),
        "p99_us": mean([run["p99_us"] for run in run_list]),
        "cpu_one_core": mean([run["cpu_one_core"] for run in run_list]),
        "rss_peak_kib": max((run["rss_peak_kib"] or 0) for run in run_list),
        "rss_growth_kib": mean([run.get("rss_growth_kib") for run in run_list]),
        "busy": sum(run["busy"] for run in run_list),
        "busy_snapshot": sum(run["busy_snapshot"] for run in run_list),
        "conflicts": sum(run["conflicts"] for run in run_list),
        "errors": sum(run["errors"] for run in run_list),
        "retries": sum(run["retries"] for run in run_list),
        "abandoned": sum(run["abandoned"] for run in run_list),
        "attempted": sum(run["attempted"] for run in run_list),
        "successful": sum(run["successful"] for run in run_list),
        "verified": sum(1 for run in run_list if run["verified"]),
        "seeds": sorted(run["seed"] for run in run_list),
    }


def fmt(value, digits=0):
    if value is None:
        return "n/a"
    if isinstance(value, float) and math.isinf(value):
        return "inf"
    if digits == 0:
        return f"{value:,.0f}"
    return f"{value:,.{digits}f}"


def ratio_text(value):
    if value is None:
        return "n/a"
    if math.isinf(value):
        return "inf"
    return f"{value:.3f}x"


def parse_dbstats(text):
    if not text:
        return {}
    parsed = {}
    number = r"([\d.]+[KMG]?)"
    wal = re.search(rf"Interval WAL: {number} writes, {number} syncs, ([\d.]+) writes per sync", text)
    if wal:
        parsed["wal_writes"] = scaled(wal.group(1))
        parsed["wal_syncs"] = scaled(wal.group(2))
        parsed["writes_per_sync"] = float(wal.group(3))
    writes = re.search(rf"Interval writes: {number} writes, {number} keys, {number} commit groups, ([\d.]+) writes per commit group, ingest: ([\d.]+) MB", text)
    if writes:
        parsed["writes"] = scaled(writes.group(1))
        parsed["keys"] = scaled(writes.group(2))
        parsed["commit_groups"] = scaled(writes.group(3))
        parsed["ingest_mb"] = float(writes.group(5))
    stall = re.search(r"Interval stall: (\d+):(\d+):([\d.]+) H:M:S, ([\d.]+) percent", text)
    if stall:
        parsed["stall_seconds"] = int(stall.group(1)) * 3600 + int(stall.group(2)) * 60 + float(stall.group(3))
        parsed["stall_percent"] = float(stall.group(4))
    return parsed


def scaled(text):
    multiplier = {"K": 1e3, "M": 1e6, "G": 1e9}.get(text[-1], 1)
    number = text[:-1] if text[-1] in "KMG" else text
    return float(number) * multiplier


def rocksdb_counters(record):
    counters = parse_dbstats(record["metrics_after"].get("dbstats"))
    before_map = record["metrics_before"].get("cfstats_map") or {}
    after_map = record["metrics_after"].get("cfstats_map") or {}
    for key in ("compaction.Sum.ReadGB", "compaction.Sum.WriteGB", "total-delays", "total-stops",
                "l0-file-count-limit-delays", "l0-file-count-limit-stops", "memtable-limit-delays",
                "memtable-limit-stops", "pending-compaction-bytes-delays", "pending-compaction-bytes-stops"):
        if key in after_map:
            counters[key] = float(after_map[key]) - float(before_map.get(key, 0))
    events = record["metrics_after"].get("events") or []
    flushes = [event for event in events if event["type"] == "flush"]
    compactions = [event for event in events if event["type"] == "compaction"]
    stalls = [event for event in events if event["type"] == "stall"]
    counters["flushes"] = len(flushes)
    counters["compactions"] = len(compactions)
    counters["compaction_input_bytes"] = sum(event["total_input_bytes"] for event in compactions)
    counters["compaction_output_bytes"] = sum(event["total_output_bytes"] for event in compactions)
    counters["stall_transitions"] = len(stalls)
    counters["stall_states"] = sorted({event["current"] for event in stalls})
    monitor = record.get("monitor") or []
    level0 = [sample["engine"]["properties"].get("rocksdb.num-files-at-level0") for sample in monitor]
    pending = [sample["engine"]["properties"].get("rocksdb.estimate-pending-compaction-bytes") for sample in monitor]
    stopped = [sample["engine"]["properties"].get("rocksdb.is-write-stopped") for sample in monitor]
    delayed = [sample["engine"]["properties"].get("rocksdb.actual-delayed-write-rate") for sample in monitor]
    counters["max_l0_files"] = max([value for value in level0 if value is not None], default=None)
    counters["end_l0_files"] = record["metrics_after"]["integer_properties"].get("rocksdb.num-files-at-level0")
    counters["max_pending_compaction_bytes"] = max([value for value in pending if value is not None], default=None)
    counters["write_stopped_samples"] = sum(1 for value in stopped if value)
    counters["delayed_rate_samples"] = sum(1 for value in delayed if value)
    return counters


def main():
    output = []
    dodb = load_dodb()
    crossdb = load_crossdb()
    summary = {}
    for scenario_index, _ in enumerate(SCENARIOS):
        summary[("dodb", scenario_index)] = summarize(dodb[scenario_index])
        for engine in PRIMARY + SECONDARY:
            run_list = crossdb.get((engine, scenario_index))
            if run_list:
                summary[(engine, scenario_index)] = summarize(run_list)

    seed_mismatches = 0
    for scenario_index, _ in enumerate(SCENARIOS):
        for engine in PRIMARY + SECONDARY:
            entry = summary.get((engine, scenario_index))
            if entry and entry["seeds"] != summary[("dodb", scenario_index)]["seeds"]:
                seed_mismatches += 1
    output.append(f"Seed sets that differ from the dodb Planned run for the same scenario: {seed_mismatches}\n")

    def throughput_table(field, title, engines):
        output.append(f"### {title}\n")
        header = "| # | Writers | Width | Distribution | " + " | ".join(LABEL[engine] for engine in engines) + " |"
        output.append(header)
        output.append("|" + "---:|" * 3 + "---|" + "---:|" * len(engines))
        for scenario_index, (writers, width, distribution) in enumerate(SCENARIOS):
            cells = []
            for engine in engines:
                entry = summary.get((engine, scenario_index))
                cells.append(fmt(entry[field]) if entry else "—")
            output.append(f"| {scenario_index} | {writers} | {width} | {REPORT[distribution]} | " + " | ".join(cells) + " |")
        output.append("")

    all_engines = ("dodb",) + PRIMARY + SECONDARY
    throughput_table("logical_tx_per_second", "Successful logical tx/s (mean of 3 repetitions)", all_engines)
    throughput_table("mutation_ops_per_second", "Successful mutation ops/s (mean of 3 repetitions)", all_engines)
    throughput_table("attempted_tx_per_second", "Attempted logical tx/s (mean of 3 repetitions)", all_engines)

    output.append("### Latency of successful transactions, p50 / p95 / p99 µs (mean of repetition percentiles)\n")
    output.append("| # | Writers | Width | Distribution | " + " | ".join(LABEL[engine] for engine in all_engines) + " |")
    output.append("|" + "---:|" * 3 + "---|" + "---:|" * len(all_engines))
    for scenario_index, (writers, width, distribution) in enumerate(SCENARIOS):
        cells = []
        for engine in all_engines:
            entry = summary.get((engine, scenario_index))
            cells.append(f"{fmt(entry['p50_us'])} / {fmt(entry['p95_us'])} / {fmt(entry['p99_us'])}" if entry else "—")
        output.append(f"| {scenario_index} | {writers} | {width} | {REPORT[distribution]} | " + " | ".join(cells) + " |")
    output.append("")

    output.append("### CPU (% of one core, mean) and sampled peak RSS (MiB, max of repetitions)\n")
    output.append("| # | Writers | Width | Distribution | " + " | ".join(LABEL[engine] for engine in all_engines) + " |")
    output.append("|" + "---:|" * 3 + "---|" + "---:|" * len(all_engines))
    for scenario_index, (writers, width, distribution) in enumerate(SCENARIOS):
        cells = []
        for engine in all_engines:
            entry = summary.get((engine, scenario_index))
            cells.append(f"{fmt(entry['cpu_one_core'])}% / {fmt(entry['rss_peak_kib'] / 1024 if entry['rss_peak_kib'] else None)}" if entry else "—")
        output.append(f"| {scenario_index} | {writers} | {width} | {REPORT[distribution]} | " + " | ".join(cells) + " |")
    output.append("")

    output.append("### Busy / conflict / retry / abandoned / error counts (sum of 3 measured intervals)\n")
    output.append("| # | Writers | Width | Distribution | Engine | Attempted tx | Committed tx | Busy | Busy snapshot | Conflicts | Retries | Abandoned | Errors | Verified |")
    output.append("|---:|---:|---:|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|")
    for scenario_index, (writers, width, distribution) in enumerate(SCENARIOS):
        for engine in PRIMARY + SECONDARY:
            entry = summary.get((engine, scenario_index))
            if not entry:
                continue
            output.append(
                f"| {scenario_index} | {writers} | {width} | {REPORT[distribution]} | {LABEL[engine]} | {entry['attempted']:,} | {entry['successful']:,} | "
                f"{entry['busy']:,} | {entry['busy_snapshot']:,} | {entry['conflicts']:,} | {entry['retries']:,} | {entry['abandoned']:,} | {entry['errors']:,} | {entry['verified']}/{entry['repetitions']} |"
            )
    output.append("")

    ratios = {}
    for engine in PRIMARY + SECONDARY:
        for scenario_index in range(12):
            entry = summary.get((engine, scenario_index))
            if not entry:
                continue
            planned = summary[("dodb", scenario_index)]["mutation_ops_per_second"]
            other = entry["mutation_ops_per_second"]
            ratios[(engine, scenario_index)] = planned / other if other else float("inf")

    output.append("### Planned / comparator mutation throughput per multiwriter scenario\n")
    output.append("| # | Writers | Width | Distribution | " + " | ".join(f"Planned / {LABEL[engine]}" for engine in PRIMARY + SECONDARY) + " |")
    output.append("|" + "---:|" * 3 + "---|" + "---:|" * len(PRIMARY + SECONDARY))
    for scenario_index in range(12):
        writers, width, distribution = SCENARIOS[scenario_index]
        cells = [ratio_text(ratios.get((engine, scenario_index))) if (engine, scenario_index) in ratios else "—" for engine in PRIMARY + SECONDARY]
        output.append(f"| {scenario_index} | {writers} | {width} | {REPORT[distribution]} | " + " | ".join(cells) + " |")
    output.append("")

    groups = {
        "overall (12)": lambda scenario: True,
        "width 1": lambda scenario: scenario[1] == 1,
        "width 16": lambda scenario: scenario[1] == 16,
        "writers 16": lambda scenario: scenario[0] == 16,
        "writers 64": lambda scenario: scenario[0] == 64,
        "uniform": lambda scenario: scenario[2] == "uniform",
        "compact locality": lambda scenario: scenario[2] == "same-leaf-heavy",
        "spread locality": lambda scenario: scenario[2] == "different-leaf-heavy",
    }
    geometric = {}
    output.append("### Geometric means of Planned / comparator (primary comparators)\n")
    output.append("| Group | " + " | ".join(f"Planned / {LABEL[engine]}" for engine in PRIMARY) + " |")
    output.append("|---|" + "---:|" * len(PRIMARY))
    for group_name, predicate in groups.items():
        cells = []
        for engine in PRIMARY:
            values = [ratios[(engine, index)] for index in range(12) if predicate(SCENARIOS[index]) and (engine, index) in ratios]
            value = geometric_mean(values)
            geometric[(engine, group_name)] = value
            cells.append(ratio_text(value) + f" (n={len(values)})")
        output.append(f"| {group_name} | " + " | ".join(cells) + " |")
    output.append("")

    output.append("### Secondary geometric means (not part of the primary headline)\n")
    output.append("| Group | Planned / RocksDB pipelined | Planned / Turso MVCC GC-off | Turso MVCC GC-on / GC-off | RocksDB default / pipelined |")
    output.append("|---|---:|---:|---:|---:|")
    for group_name, predicate in groups.items():
        pipelined = [ratios[("rocksdb-pipelined", index)] for index in range(12) if predicate(SCENARIOS[index]) and ("rocksdb-pipelined", index) in ratios]
        nogc = [ratios[("turso-mvcc-nogc", index)] for index in range(12) if predicate(SCENARIOS[index]) and ("turso-mvcc-nogc", index) in ratios]
        gc_gain = [summary[("turso-mvcc-gc", index)]["mutation_ops_per_second"] / summary[("turso-mvcc-nogc", index)]["mutation_ops_per_second"]
                   for index in range(12) if predicate(SCENARIOS[index]) and ("turso-mvcc-nogc", index) in summary]
        rocks_gain = [summary[("rocksdb", index)]["mutation_ops_per_second"] / summary[("rocksdb-pipelined", index)]["mutation_ops_per_second"]
                      for index in range(12) if predicate(SCENARIOS[index]) and ("rocksdb-pipelined", index) in summary]
        output.append(
            f"| {group_name} | {ratio_text(geometric_mean(pipelined))} (n={len(pipelined)}) | {ratio_text(geometric_mean(nogc))} (n={len(nogc)}) | "
            f"{ratio_text(geometric_mean(gc_gain))} (n={len(gc_gain)}) | {ratio_text(geometric_mean(rocks_gain))} (n={len(rocks_gain)}) |"
        )
    output.append("")

    output.append("### RocksDB write-path counters in the measured interval (mean of repetitions)\n")
    output.append("| # | Writers | Width | Distribution | Variant | WAL writes | WAL syncs | Writes/sync | Ingest MB | Flushes | Compactions | Compaction read MiB | Compaction write MiB | Stall s / delay+stop count | Max L0 files | Max pending compaction MiB |")
    output.append("|---:|---:|---:|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|")
    rocksdb_detail = {}
    for scenario_index, (writers, width, distribution) in enumerate(SCENARIOS):
        for engine in ("rocksdb", "rocksdb-pipelined"):
            run_list = crossdb.get((engine, scenario_index))
            if not run_list:
                continue
            counters = [rocksdb_counters(run["record"]) for run in run_list]
            rocksdb_detail[f"{engine}-{scenario_index}"] = counters
            wal_writes = mean([entry.get("wal_writes") for entry in counters])
            wal_syncs = mean([entry.get("wal_syncs") for entry in counters])
            writes_per_sync = mean([entry.get("writes_per_sync") for entry in counters])
            output.append(
                f"| {scenario_index} | {writers} | {width} | {REPORT[distribution]} | {LABEL[engine]} | {fmt(wal_writes)} | {fmt(wal_syncs)} | "
                f"{fmt(writes_per_sync, 2)} | {fmt(mean([entry.get('ingest_mb') for entry in counters]), 1)} | "
                f"{fmt(mean([entry['flushes'] for entry in counters]), 1)} | {fmt(mean([entry['compactions'] for entry in counters]), 1)} | "
                f"{fmt(mean([entry['compaction_input_bytes'] for entry in counters]) / 2**20, 1)} | {fmt(mean([entry['compaction_output_bytes'] for entry in counters]) / 2**20, 1)} | "
                f"{fmt(mean([entry.get('stall_seconds') for entry in counters]), 3)} / {fmt(sum(entry.get('total-delays', 0) + entry.get('total-stops', 0) for entry in counters))} | {max((entry['max_l0_files'] or 0) for entry in counters)} | "
                f"{fmt(max((entry['max_pending_compaction_bytes'] or 0) for entry in counters) / 2**20, 1)} |"
            )
    output.append("")

    output.append("### Turso files at the end of the measured interval (mean bytes) and RSS growth\n")
    output.append("| # | Writers | Width | Distribution | Engine | kv.db | kv.db-wal | kv.db-log | RSS growth after seed (MiB) |")
    output.append("|---:|---:|---:|---|---|---:|---:|---:|---:|")
    for scenario_index, (writers, width, distribution) in enumerate(SCENARIOS):
        for engine in ("turso-wal", "turso-mvcc-gc", "turso-mvcc-nogc"):
            run_list = crossdb.get((engine, scenario_index))
            if not run_list:
                continue
            files = [run["record"]["files_before_close"] for run in run_list]
            output.append(
                f"| {scenario_index} | {writers} | {width} | {REPORT[distribution]} | {LABEL[engine]} | "
                f"{fmt(mean([entry.get('kv.db') for entry in files]))} | {fmt(mean([entry.get('kv.db-wal') for entry in files]))} | "
                f"{fmt(mean([entry.get('kv.db-log') for entry in files]))} | {fmt(mean([run['rss_growth_kib'] for run in run_list]) / 1024, 1)} |"
            )
    output.append("")

    verification = {}
    for (engine, scenario_index), run_list in crossdb.items():
        stats = verification.setdefault(engine, {"runs": 0, "passed": 0, "sampled_keys": 0, "keys_with_writes": 0})
        for run in run_list:
            stats["runs"] += 1
            stats["passed"] += 1 if run["verified"] else 0
            stats["sampled_keys"] += run["verification"]["sampled_keys"]
            stats["keys_with_writes"] += run["verification"]["sampled_keys_with_committed_writes"]
    output.append("### Post-reopen verification of the core matrix\n")
    output.append("| Engine | Runs passed | Sampled keys checked | Sampled keys with committed writes |")
    output.append("|---|---:|---:|---:|")
    for engine, stats in sorted(verification.items()):
        output.append(f"| {LABEL[engine]} | {stats['passed']}/{stats['runs']} | {stats['sampled_keys']:,} | {stats['keys_with_writes']:,} |")
    output.append("")

    sustained = load_sustained()
    output.extend(sustained["text"])

    analysis = {
        "summary": {f"{engine}|{index}": {key: value for key, value in entry.items()} for (engine, index), entry in summary.items()},
        "ratios": {f"{engine}|{index}": value for (engine, index), value in ratios.items()},
        "geometric_means": {f"{engine}|{group}": value for (engine, group), value in geometric.items()},
        "rocksdb_counters": rocksdb_detail,
        "verification": verification,
        "sustained": sustained["data"],
        "seed_mismatches": seed_mismatches,
    }
    with open(os.path.join(CROSSDB, "analysis.json"), "w", encoding="utf-8") as handle:
        json.dump(analysis, handle, indent=1, sort_keys=True, default=str)
    with open(os.path.join(CROSSDB, "tables.md"), "w", encoding="utf-8") as handle:
        handle.write("\n".join(output) + "\n")
    print("\n".join(output))


def load_monitor(path):
    if not os.path.exists(path):
        return []
    return read_jsonl(path)


def load_sustained():
    text = []
    data = {}
    files = sorted(glob.glob(os.path.join(CROSSDB, "raw", "sustained-*.jsonl")))
    files = [path for path in files if not path.endswith(".process-monitor.jsonl")]
    text.append("## Sustained 120 s results\n")
    text.append("| Writers | Engine | Successful tx/s | Attempted tx/s | p50 / p95 / p99 µs | CPU % one core | Peak RSS MiB | Max disk MiB | Busy | Conflicts | Abandoned | Errors | Verified |")
    text.append("|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|")
    window_rows = []
    for path in files:
        rows = read_jsonl(path)
        if not rows:
            continue
        record = rows[0]
        monitor = load_monitor(path.replace(".jsonl", ".process-monitor.jsonl"))
        peak_rss = max((sample["rss_kib"] or 0) for sample in monitor) / 1024 if monitor else None
        max_disk = max(sample["disk_allocated_bytes"] for sample in monitor) / 2**20 if monitor else None
        end_disk = monitor[-1]["disk_allocated_bytes"] / 2**20 if monitor else None
        if record.get("record_type") == "run":
            engine = "dodb"
            writers = record["writers"]
            seconds = record["duration_ms"] / 1000
            entry = {
                "tx": record["logical_tx_per_second"],
                "attempted": record["attempted_transactions"] / seconds,
                "p": (record["e2e_p50_us"], record["e2e_p95_us"], record["e2e_p99_us"]),
                "cpu": record["cpu_utilization_percent_one_core"],
                "busy": 0, "conflicts": record["conflicts"],
                "abandoned": record["attempted_transactions"] - record["successful_transactions"],
                "errors": record["errors"] + record["overloads"],
                "verified": "reopen not run by phase0-bench; WAL sync counters positive" if record["wal_syncs_delta"] > 0 else "no WAL syncs",
                "windows": [
                    {
                        "index": index,
                        "tx": record[f"window_{index:02d}_logical_tx_per_second"],
                        "p95": record[f"window_{index:02d}_p95_us"],
                        "p99": record[f"window_{index:02d}_p99_us"],
                        "span": record[f"window_{index:02d}_span_seconds"],
                    }
                    for index in range(record.get("window_count", 0))
                ],
            }
        else:
            engine = record["engine"]
            writers = record["writers"]
            measured = record["measured"]
            entry = {
                "tx": measured["logical_tx_per_second"],
                "attempted": measured["attempted_tx_per_second"],
                "p": (measured["p50_us"], measured["p95_us"], measured["p99_us"]),
                "cpu": record.get("cpu_utilization_percent_one_core"),
                "busy": measured["busy"], "conflicts": measured["conflicts"],
                "abandoned": measured["abandoned_transactions"], "errors": measured["errors"],
                "verified": str(record["verification"]["passed"]),
                "windows": [
                    {"index": window["window_index"], "tx": window["logical_tx_per_second"], "p95": window["p95_us"], "p99": window["p99_us"], "span": window["span_s"]}
                    for window in record["windows"]
                ],
            }
            if engine.startswith("rocksdb"):
                entry["rocksdb"] = rocksdb_counters(record)
                entry["rocksdb_event_timeline"] = rocksdb_event_timeline(record)
        entry["peak_rss_mib"] = peak_rss
        entry["max_disk_mib"] = max_disk
        entry["end_disk_mib"] = end_disk
        data[f"{engine}|{writers}"] = entry
        text.append(
            f"| {writers} | {LABEL[engine]} | {fmt(entry['tx'])} | {fmt(entry['attempted'])} | {fmt(entry['p'][0])} / {fmt(entry['p'][1])} / {fmt(entry['p'][2])} | "
            f"{fmt(entry['cpu'])} | {fmt(peak_rss)} | {fmt(max_disk, 1)} | {entry['busy']:,} | {entry['conflicts']:,} | {entry['abandoned']:,} | {entry['errors']:,} | {entry['verified']} |"
        )
        window_rows.append((writers, engine, entry))
    text.append("")
    for writers in (16, 64):
        rows = [(engine, entry) for row_writers, engine, entry in window_rows if row_writers == writers]
        if not rows:
            continue
        text.append(f"### 10-second windows, {writers} writers: successful tx/s (p95 / p99 µs)\n")
        text.append("| Window | " + " | ".join(LABEL[engine] for engine, _ in rows) + " |")
        text.append("|---:|" + "---:|" * len(rows))
        window_count = max(len(entry["windows"]) for _, entry in rows)
        for window_index in range(min(window_count, 12)):
            cells = []
            for _, entry in rows:
                window = entry["windows"][window_index] if window_index < len(entry["windows"]) else None
                cells.append(f"{fmt(window['tx'])} ({fmt(window['p95'])} / {fmt(window['p99'])})" if window else "—")
            text.append(f"| {window_index * 10}-{window_index * 10 + 10} s | " + " | ".join(cells) + " |")
        text.append("")
    for writers in (16, 64):
        entry = data.get(f"rocksdb|{writers}")
        if not entry:
            continue
        counters = entry["rocksdb"]
        text.append(f"### RocksDB sustained {writers} writers: flush, compaction and stall behaviour\n")
        text.append(f"- WAL writes {fmt(counters.get('wal_writes'))}, WAL syncs {fmt(counters.get('wal_syncs'))}, writes per sync {fmt(counters.get('writes_per_sync'), 2)}, ingest {fmt(counters.get('ingest_mb'), 1)} MB, commit groups {fmt(counters.get('commit_groups'))}.")
        text.append(f"- cfstats deltas: compaction read {fmt((counters.get('compaction.Sum.ReadGB') or 0) * 1024, 1)} MiB, flush plus compaction write {fmt((counters.get('compaction.Sum.WriteGB') or 0) * 1024, 1)} MiB, delays {fmt(counters.get('total-delays'))}, stops {fmt(counters.get('total-stops'))}.")
        text.append(f"- Flushes {counters['flushes']}, compactions {counters['compactions']}, compaction read {fmt(counters['compaction_input_bytes'] / 2**20, 1)} MiB, compaction write {fmt(counters['compaction_output_bytes'] / 2**20, 1)} MiB.")
        text.append(f"- Stall time from dbstats {fmt(counters.get('stall_seconds'), 3)} s; stall-condition transitions {counters['stall_transitions']} {counters['stall_states']}; samples with write stopped {counters['write_stopped_samples']}, with a delayed write rate {counters['delayed_rate_samples']}.")
        text.append(f"- Max L0 files {counters['max_l0_files']}, L0 files at end {counters['end_l0_files']}, max pending compaction bytes {fmt((counters['max_pending_compaction_bytes'] or 0) / 2**20, 1)} MiB.")
        text.append("- Event timeline (seconds from measured start): " + ", ".join(entry["rocksdb_event_timeline"]) if entry["rocksdb_event_timeline"] else "- No flush or compaction events inside the measured interval.")
        text.append("")
    return {"text": text, "data": data}


def rocksdb_event_timeline(record):
    start = record["metrics_before"]["t_mono"]
    events = record["metrics_after"].get("events") or []
    timeline = []
    for event in events:
        offset = event["t_mono"] - start
        if event["type"] == "flush":
            timeline.append(f"{offset:.1f} flush ({event['num_entries']} entries)")
        elif event["type"] == "compaction":
            timeline.append(f"{offset:.1f} compaction L{event['base_input_level']}->L{event['output_level']} ({event['total_input_bytes'] / 2**20:.1f}->{event['total_output_bytes'] / 2**20:.1f} MiB, {event['elapsed_micros'] / 1000:.0f} ms)")
        else:
            timeline.append(f"{offset:.1f} stall {event['previous']}->{event['current']}")
    return timeline


main()
