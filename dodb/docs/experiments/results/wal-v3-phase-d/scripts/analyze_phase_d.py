import glob
import json
import math
import os
import re
import statistics
import sys

RESULTS = sys.argv[1]
RAW = os.path.join(RESULTS, "raw")
LABELS = {"uniform": "uniform", "same-leaf-heavy": "compact", "different-leaf-heavy": "spread"}
NAME = re.compile(r"^(?P<phase>[a-z]+)-(?P<sync>real|disabled)-(?P<scenario>\d\d)-rep(?P<rep>\d)-(?P<variant>[a-z0-9-]+?)-w(?P<writers>\d+)-width(?P<width>\d+)-(?P<distribution>.+)\.jsonl$")
COORDINATOR = (
    ("admission", "logical_admission_nanos"),
    ("planning", "planning_nanos"),
    ("serial physical mutation", "physical_mutation_nanos_total"),
    ("serial physical restamp", "physical_restamp_nanos_total"),
    ("serial page encode", "physical_page_encode_nanos_total"),
    ("parallel dispatch (job build)", "parallel_dispatch_nanos_total"),
    ("parallel lanes: coordinator runs leaf jobs", "parallel_coordinator_lane_nanos_total"),
    ("parallel lanes: coordinator waits for worker threads", None),
    ("parallel result collection", "parallel_collect_nanos_total"),
    ("physical other", None),
    ("dirty union", "dirty_union_nanos_total"),
    ("catalog construction", "catalog_construction_nanos"),
    ("WAL assembly", "wal_assembly_nanos_total"),
    ("WAL redo plan (serial: delta encode; parallel: chain check)", "wal_redo_plan_nanos_total"),
    ("WAL frame encode", None),
    ("WAL write", "wal_group_write_nanos_total"),
    ("WAL sync", "wal_sync_nanos_total"),
    ("state install", "state_install_nanos_total"),
    ("generation publication", "generation_publication_nanos"),
    ("dirty tracking", "dirty_tracking_nanos_total"),
    ("unattributed", None),
)
WORKER = (
    ("base image + chain check", "parallel_worker_base_nanos_total"),
    ("mutation", "parallel_worker_mutation_nanos_total"),
    ("page encode", "parallel_worker_encode_nanos_total"),
    ("delta encode + verify + CRC", "parallel_worker_delta_nanos_total"),
)
PHASE_C_CATEGORIES = (
    ("planner (admission + planning)", ("admission", "planning")),
    ("leaf physical on coordinator (serial mutation, restamp, encode, other)",
     ("serial physical mutation", "serial physical restamp", "serial page encode", "physical other")),
    ("leaf jobs run by the coordinator lane", ("parallel lanes: coordinator runs leaf jobs",)),
    ("parallel dispatch + wait for worker threads + collect",
     ("parallel dispatch (job build)", "parallel lanes: coordinator waits for worker threads", "parallel result collection")),
    ("WAL CPU (assembly, redo plan, frame encode, write)",
     ("WAL assembly", "WAL redo plan (serial: delta encode; parallel: chain check)", "WAL frame encode", "WAL write")),
    ("catalog / publication / install / dirty",
     ("dirty union", "catalog construction", "state install", "generation publication", "dirty tracking")),
    ("WAL sync", ("WAL sync",)),
    ("unattributed", ("unattributed",)),
)


def value(row, key):
    return row.get(key, 0) or 0


def load_rows():
    groups = {}
    for path in sorted(glob.glob(os.path.join(RAW, "*.jsonl"))):
        name = os.path.basename(path)
        match = NAME.match(name)
        if not match or name.startswith("confirm-rocksdb"):
            continue
        with open(path, encoding="utf-8") as input_file:
            row = json.loads(input_file.read().splitlines()[0])
        if row["engine"] != "planned-blink" or row["sync_mode"] != match["sync"] or row["errors"] != 0:
            raise SystemExit(f"{path}: bad row")
        key = (match["phase"], match["sync"], int(match["scenario"]), match["variant"])
        groups.setdefault(key, []).append((row, match.groupdict()))
    return groups


def coordinator_components(row):
    values = {name: value(row, key) for name, key in COORDINATOR if key}
    values["parallel lanes: coordinator waits for worker threads"] = max(
        0, value(row, "parallel_join_nanos_total") - value(row, "parallel_coordinator_lane_nanos_total"))
    physical_known = sum(values[name] for name in (
        "serial physical mutation", "serial physical restamp", "serial page encode",
        "parallel dispatch (job build)", "parallel lanes: coordinator runs leaf jobs",
        "parallel lanes: coordinator waits for worker threads", "parallel result collection"))
    values["physical other"] = max(0, value(row, "physical_execution_nanos") - physical_known)
    values["WAL frame encode"] = max(0, value(row, "wal_group_encode_nanos_total") - value(row, "wal_redo_plan_nanos_total"))
    values["unattributed"] = max(0, value(row, "processing_nanos_total") - sum(values.values()))
    return values


def summarize(entries):
    rows = [row for row, _ in entries]
    meta = entries[0][1]
    transactions = sum(row["successful_transactions"] for row in rows)
    mutations = sum(row["mutation_ops"] for row in rows)
    processing = sum(row["processing_nanos_total"] for row in rows)
    duration = sum(row["duration_ms"] for row in rows) * 1e6
    components = {name: sum(coordinator_components(row)[name] for row in rows) for name, _ in COORDINATOR}
    worker = {name: sum(value(row, key) for row in rows) for name, key in WORKER}
    parallel_groups = sum(value(row, "parallel_groups_delta") for row in rows)
    logical_groups = sum(row["groups"] for row in rows)
    busy = sum(value(row, "parallel_worker_nanos_total") for row in rows)
    slots = sum(value(row, "parallel_worker_slot_nanos_total") for row in rows)
    join = sum(value(row, "parallel_join_nanos_total") for row in rows)
    return {
        "writers": int(meta["writers"]),
        "width": int(meta["width"]),
        "distribution": meta["distribution"],
        "runs": len(rows),
        "tx_per_second_runs": [row["logical_tx_per_second"] for row in rows],
        "tx_per_second": statistics.mean(row["logical_tx_per_second"] for row in rows),
        "mutation_ops_per_second": statistics.mean(row["mutation_ops_per_second"] for row in rows),
        "p50_us": statistics.mean(row["e2e_p50_us"] for row in rows),
        "p99_us": statistics.mean(row["e2e_p99_us"] for row in rows),
        "cpu_percent_one_core": statistics.mean(row["cpu_utilization_percent_one_core"] for row in rows),
        "coordinator_busy_fraction": processing / duration,
        "coordinator_cpu_fraction": (processing - components["parallel lanes: coordinator waits for worker threads"]) / duration,
        "tx_per_group": transactions / max(logical_groups, 1),
        "tx_per_sync": transactions / max(sum(value(row, "wal_syncs_delta") for row in rows), 1),
        "mean_sync_ms": sum(value(row, "wal_sync_nanos_total") for row in rows) / max(sum(value(row, "wal_syncs_delta") for row in rows), 1) / 1e6,
        "wal_bytes_per_tx": sum(row["wal_bytes_delta"] for row in rows) / transactions,
        "processing_ns_per_tx": processing / transactions,
        "component_ns_per_tx": {name: total / transactions for name, total in components.items()},
        "component_share": {name: total / processing for name, total in components.items()},
        "worker_ns_per_tx": {name: total / transactions for name, total in worker.items()},
        "worker_busy_ns_per_tx": busy / transactions,
        "worker_idle_ns_per_tx": max(0, slots - busy) / transactions,
        "worker_busy_fraction_of_join": busy / slots if slots else 0,
        "effective_parallelism": busy / join if join else 0,
        "logical_groups": logical_groups,
        "parallel_groups": parallel_groups,
        "parallel_group_fraction": parallel_groups / max(logical_groups, 1),
        "fallback_groups": sum(value(row, "parallel_fallback_groups_delta") for row in rows),
        "fallback_after_dispatch": sum(value(row, "parallel_fallback_after_dispatch_delta") for row in rows),
        "fallback_reasons": {
            reason: sum(value(row, f"parallel_fallback_{reason}_delta") for row in rows)
            for reason in ("no_delta_wal", "route", "overflow", "structural")
        },
        "single_leaf_groups": sum(value(row, "parallel_skipped_single_leaf_delta") for row in rows),
        "leaf_jobs_per_parallel_group": sum(value(row, "parallel_leaf_jobs_delta") for row in rows) / max(parallel_groups, 1),
        "operations_per_job": sum(value(row, "parallel_job_operations_delta") for row in rows)
        / max(sum(value(row, "parallel_leaf_jobs_delta") for row in rows), 1),
        "mutations_per_job": sum(value(row, "parallel_mutations_delta") for row in rows)
        / max(sum(value(row, "parallel_leaf_jobs_delta") for row in rows), 1),
        "page_delta_records_per_tx": sum(value(row, "wal_page_delta_records_delta") for row in rows) / transactions,
        "page_image_records_per_tx": sum(value(row, "wal_page_image_records_delta") for row in rows) / transactions,
    }


def fmt(number, digits=0):
    return f"{number:,.{digits}f}"


def label(summary):
    return f"{summary['writers']}w w{summary['width']} {LABELS[summary['distribution']]}"


def geometric_mean(values):
    return math.exp(sum(math.log(item) for item in values) / len(values)) if values else float("nan")


def comparison_table(lines, summaries, phase, sync, scenarios, variants, baseline):
    lines.append("| Scenario | " + " | ".join(f"{variant} tx/s (runs)" for variant in variants) + " | "
                 + " | ".join(f"{variant} / {baseline}" for variant in variants if variant != baseline)
                 + " | p99 µs " + " → ".join(variants) + " |")
    lines.append("|---|" + "---|" * (2 * len(variants)))
    ratios = {variant: [] for variant in variants}
    for scenario in scenarios:
        row = [summaries.get((phase, sync, scenario, variant)) for variant in variants]
        if any(item is None for item in row):
            continue
        base = summaries[(phase, sync, scenario, baseline)]["tx_per_second"]
        cells = [f"{fmt(item['tx_per_second'])} ({', '.join(fmt(run) for run in item['tx_per_second_runs'])})" for item in row]
        ratio_cells = []
        for variant, item in zip(variants, row):
            if variant != baseline:
                ratio = item["tx_per_second"] / base
                ratios[variant].append(ratio)
                ratio_cells.append(f"{ratio:.3f}")
        lines.append(f"| {label(row[0])} | " + " | ".join(cells) + " | " + " | ".join(ratio_cells) + " | "
                     + " → ".join(fmt(item["p99_us"]) for item in row) + " |")
    lines.append("")
    return ratios


def attribution_tables(lines, summaries, phase, sync, scenarios, variants):
    columns = [(scenario, variant) for scenario in scenarios for variant in variants
               if (phase, sync, scenario, variant) in summaries]
    header = " | ".join(f"{label(summaries[(phase, sync, scenario, variant)])} {variant}" for scenario, variant in columns)
    lines.append(f"### Coordinator time per transaction, ns ({phase}, sync {sync})\n")
    lines.append(f"| Component | {header} |")
    lines.append("|---|" + "---|" * len(columns))
    for name, _ in COORDINATOR:
        lines.append(f"| {name} | " + " | ".join(
            fmt(summaries[(phase, sync, scenario, variant)]["component_ns_per_tx"][name]) for scenario, variant in columns) + " |")
    lines.append("| **total** | " + " | ".join(
        fmt(summaries[(phase, sync, scenario, variant)]["processing_ns_per_tx"]) for scenario, variant in columns) + " |")
    lines.append("")
    lines.append(f"### Grouped as Phase C categories, share of coordinator time ({phase}, sync {sync})\n")
    lines.append(f"| Category | {header} |")
    lines.append("|---|" + "---|" * len(columns))
    for category, names in PHASE_C_CATEGORIES:
        lines.append(f"| {category} | " + " | ".join(
            f"{sum(summaries[(phase, sync, scenario, variant)]['component_share'][name] for name in names):.1%}"
            for scenario, variant in columns) + " |")
    lines.append("")
    lines.append(f"### Worker side per transaction, ns ({phase}, sync {sync})\n")
    lines.append(f"| Item | {header} |")
    lines.append("|---|" + "---|" * len(columns))
    for name, _ in WORKER:
        lines.append(f"| {name} | " + " | ".join(
            fmt(summaries[(phase, sync, scenario, variant)]["worker_ns_per_tx"][name]) for scenario, variant in columns) + " |")
    for name, key in (("all lanes busy (coordinator lane + threads)", "worker_busy_ns_per_tx"),
                      ("lane idle inside the parallel section", "worker_idle_ns_per_tx")):
        lines.append(f"| {name} | " + " | ".join(
            fmt(summaries[(phase, sync, scenario, variant)][key]) for scenario, variant in columns) + " |")
    lines.append("| effective parallelism (lanes busy / parallel wall) | " + " | ".join(
        f"{summaries[(phase, sync, scenario, variant)]['effective_parallelism']:.2f}" for scenario, variant in columns) + " |")
    lines.append("| coordinator busy (incl. join wait) | " + " | ".join(
        f"{summaries[(phase, sync, scenario, variant)]['coordinator_busy_fraction']:.1%}" for scenario, variant in columns) + " |")
    lines.append("| coordinator busy (excl. join wait) | " + " | ".join(
        f"{summaries[(phase, sync, scenario, variant)]['coordinator_cpu_fraction']:.1%}" for scenario, variant in columns) + " |")
    lines.append("")


def parallel_table(lines, summaries, keys):
    lines.append("| Run set | groups | parallel groups | parallel share | fallback groups | after dispatch | overflow / structural / route / no-delta WAL | single-leaf groups | leaf jobs / parallel group | operations / job | mutations / job | tx / group |")
    lines.append("|---|---|---|---|---|---|---|---|---|---|---|---|")
    for key in keys:
        if key not in summaries:
            continue
        item = summaries[key]
        reasons = item["fallback_reasons"]
        lines.append(
            f"| {key[0]} {key[1]} {label(item)} {key[3]} | {item['logical_groups']:,} | {item['parallel_groups']:,} | "
            f"{item['parallel_group_fraction']:.1%} | {item['fallback_groups']} | {item['fallback_after_dispatch']} | "
            f"{reasons['overflow']} / {reasons['structural']} / {reasons['route']} / {reasons['no_delta_wal']} | "
            f"{item['single_leaf_groups']} | {item['leaf_jobs_per_parallel_group']:.1f} | {item['operations_per_job']:.3f} | "
            f"{item['mutations_per_job']:.2f} | {item['tx_per_group']:.1f} |")
    lines.append("")


def rocksdb_summary(scenario):
    paths = sorted(glob.glob(os.path.join(RAW, f"confirm-rocksdb-{scenario:02d}-rep*.jsonl")))
    dodb = sorted(glob.glob(os.path.join(RAW, f"confirm-real-{scenario:02d}-rep*-par2-*.jsonl")))
    if len(paths) != 3 or len(dodb) != 3:
        return None
    rows = []
    for path in paths:
        with open(path, encoding="utf-8") as input_file:
            rows.append(json.loads(input_file.read().splitlines()[0]))
    dodb_rows = []
    for path in dodb:
        with open(path, encoding="utf-8") as input_file:
            dodb_rows.append(json.loads(input_file.read().splitlines()[0]))
    return {
        "rocksdb_tx_per_second": statistics.mean(row["measured"]["logical_tx_per_second"] for row in rows),
        "rocksdb_p99_us": statistics.mean(row["measured"]["p99_us"] for row in rows),
        "dodb_tx_per_second": statistics.mean(row["logical_tx_per_second"] for row in dodb_rows),
        "dodb_p99_us": statistics.mean(row["e2e_p99_us"] for row in dodb_rows),
    }


def main():
    groups = load_rows()
    summaries = {key: summarize(entries) for key, entries in groups.items()}
    lines = ["# WAL v3 Phase D tables\n",
             "planned Blink, 100,000 rows, 16-byte keys, 64-byte values, 2 s warmup, 5 s measure, 3 repetitions unless noted. "
             "`phase-c` = binary `cb7fb54` (serial planned executor). `serial-d`, `par1`, `par2` = Phase D binary with "
             "`--parallel-workers 0 / 1 / 2` (lanes: 1 = the coordinator alone runs the leaf jobs, 2 = coordinator + one worker thread). `disabled` rows skip fsync (CPU control, not durable throughput).\n"]
    gate_scenarios = (9, 10, 11)
    if any(key[0] == "cpu" for key in summaries):
        lines.append("## Sync-disabled CPU gate\n")
        comparison_table(lines, summaries, "cpu", "disabled", gate_scenarios, ("phase-c", "serial-d", "par1", "par2"), "phase-c")
        lines.append("Same-binary ratios:\n")
        lines.append("| Scenario | par2 / serial-d | par1 / serial-d | par2 / par1 |")
        lines.append("|---|---|---|---|")
        for scenario in gate_scenarios:
            try:
                serial = summaries[("cpu", "disabled", scenario, "serial-d")]["tx_per_second"]
                one = summaries[("cpu", "disabled", scenario, "par1")]["tx_per_second"]
                two = summaries[("cpu", "disabled", scenario, "par2")]["tx_per_second"]
            except KeyError:
                continue
            lines.append(f"| {label(summaries[('cpu', 'disabled', scenario, 'par2')])} | {two / serial:.3f} | {one / serial:.3f} | {two / one:.3f} |")
        lines.append("")
        attribution_tables(lines, summaries, "cpu", "disabled", gate_scenarios, ("phase-c", "par1", "par2"))
        lines.append("### Parallel execution counters (sync-disabled gate)\n")
        parallel_table(lines, summaries, [("cpu", "disabled", scenario, variant) for scenario in gate_scenarios for variant in ("par1", "par2")])
    durable = (0, 3, 6, 9, 10, 11)
    if any(key[0] == "gate" for key in summaries):
        lines.append("## Real-sync durable gate (ZFS)\n")
        ratios = comparison_table(lines, summaries, "gate", "real", durable, ("phase-c", "par2"), "phase-c")
        lines.append(f"GM par2 / phase-c over the {len(ratios['par2'])} scenarios: {geometric_mean(ratios['par2']):.3f}\n")
        attribution_tables(lines, summaries, "gate", "real", durable, ("phase-c", "par2"))
        lines.append("### Parallel execution counters (real-sync gate)\n")
        parallel_table(lines, summaries, [("gate", "real", scenario, "par2") for scenario in durable])
        lines.append("### WAL and sync (real-sync gate)\n")
        lines.append("| Scenario | variant | WAL B/tx | deltas/tx | images/tx | tx/sync | mean sync ms | CPU % one core |")
        lines.append("|---|---|---|---|---|---|---|---|")
        for scenario in durable:
            for variant in ("phase-c", "par2"):
                item = summaries.get(("gate", "real", scenario, variant))
                if item:
                    lines.append(f"| {label(item)} | {variant} | {fmt(item['wal_bytes_per_tx'], 1)} | {item['page_delta_records_per_tx']:.3f} | "
                                 f"{item['page_image_records_per_tx']:.4f} | {item['tx_per_sync']:.1f} | {item['mean_sync_ms']:.2f} | "
                                 f"{fmt(item['cpu_percent_one_core'])} |")
        lines.append("")
    if any(key[0] == "matrix" for key in summaries):
        lines.append("## Full matrix (real sync)\n")
        ratios = comparison_table(lines, summaries, "matrix", "real", range(14), ("phase-c", "par2"), "phase-c")
        multiwriter = [summaries[("matrix", "real", scenario, "par2")]["tx_per_second"] / summaries[("matrix", "real", scenario, "phase-c")]["tx_per_second"]
                       for scenario in range(12) if ("matrix", "real", scenario, "par2") in summaries]
        lines.append(f"Multiwriter GM (12 scenarios) par2 / phase-c: {geometric_mean(multiwriter):.3f}\n")
    rocksdb = {scenario: rocksdb_summary(scenario) for scenario in (6, 9)}
    if all(rocksdb.values()):
        lines.append("## RocksDB same session\n")
        lines.append("| Scenario | dodb par2 tx/s | RocksDB tx/s | dodb / RocksDB | p99 µs dodb / RocksDB |")
        lines.append("|---|---|---|---|---|")
        for scenario, item in rocksdb.items():
            lines.append(f"| scenario {scenario} | {fmt(item['dodb_tx_per_second'])} | {fmt(item['rocksdb_tx_per_second'])} | "
                         f"{item['dodb_tx_per_second'] / item['rocksdb_tx_per_second']:.3f} | {fmt(item['dodb_p99_us'])} / {fmt(item['rocksdb_p99_us'])} |")
        lines.append("")
    with open(os.path.join(RESULTS, "tables.md"), "w", encoding="utf-8") as output_file:
        output_file.write("\n".join(lines) + "\n")
    with open(os.path.join(RESULTS, "analysis.json"), "w", encoding="utf-8") as output_file:
        json.dump({"-".join(str(part) for part in key): item for key, item in summaries.items()} | {"rocksdb": rocksdb},
                  output_file, indent=2, sort_keys=True)
    print("\n".join(lines))


if __name__ == "__main__":
    main()
