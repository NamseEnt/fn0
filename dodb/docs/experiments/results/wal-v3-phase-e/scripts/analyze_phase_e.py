import glob
import importlib.util
import json
import math
import os
import re
import statistics
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
spec = importlib.util.spec_from_file_location(
    "analyze_phase_d", os.path.join(HERE, "..", "..", "wal-v3-phase-d", "scripts", "analyze_phase_d.py"))
RESULTS = sys.argv[1]
sys.argv = [sys.argv[0], RESULTS]
phase_d = importlib.util.module_from_spec(spec)
spec.loader.exec_module(phase_d)

NAME = re.compile(r"^(?P<phase>[a-z0-9]+)-(?P<sync>real|disabled)-(?P<scenario>\d\d)-rep(?P<rep>\d)-(?P<variant>[a-z0-9-]+?)"
                  r"-w(?P<writers>\d+)-width(?P<width>\d+)-(?P<distribution>.+)\.jsonl$")
LABELS = {"uniform": "uniform", "same-leaf-heavy": "compact", "different-leaf-heavy": "spread"}


def load_rows():
    groups = {}
    for path in sorted(glob.glob(os.path.join(RESULTS, "raw", "*.jsonl"))):
        name = os.path.basename(path)
        match = NAME.match(name)
        if not match or name.startswith("confirm-rocksdb") or match["phase"] in ("counters", "perf"):
            continue
        lines = open(path, encoding="utf-8").read().splitlines()
        if not lines:
            continue
        row = json.loads(lines[0])
        if row["engine"] != "planned-blink" or row["sync_mode"] != match["sync"] or row["errors"] != 0:
            raise SystemExit(f"{path}: bad row")
        if row.get("churn_counters") == "enabled":
            raise SystemExit(f"{path}: instrumented binary in a timed run")
        key = (match["phase"], match["sync"], int(match["scenario"]), match["variant"])
        groups.setdefault(key, []).append((row, match.groupdict()))
    return groups


def geometric_mean(values):
    return math.exp(sum(math.log(value) for value in values) / len(values)) if values else float("nan")


def summaries():
    return {key: phase_d.summarize(entries) for key, entries in load_rows().items()}


def label(summary):
    return f"{summary['writers']}w w{summary['width']} {LABELS[summary['distribution']]}"


def coordinator_view(summary):
    components = summary["component_ns_per_tx"]
    planner = components["admission"] + components["planning"]
    leaf = (components["parallel lanes: coordinator runs leaf jobs"] + components["serial physical mutation"]
            + components["serial physical restamp"] + components["serial page encode"] + components["physical other"])
    dispatch = (components["parallel dispatch (job build)"] + components["parallel lanes: coordinator waits for worker threads"]
                + components["parallel result collection"])
    publication = sum(components[name] for name in ("dirty union", "catalog construction", "state install",
                                                    "generation publication", "dirty tracking"))
    wal = sum(components[name] for name in ("WAL assembly", "WAL redo plan (serial: delta encode; parallel: chain check)",
                                            "WAL frame encode", "WAL write"))
    return {
        "coordinator_us_per_tx": summary["processing_ns_per_tx"] / 1000,
        "planner_us_per_tx": planner / 1000,
        "leaf_on_coordinator_us_per_tx": leaf / 1000,
        "lanes_busy_us_per_tx": summary["worker_busy_ns_per_tx"] / 1000,
        "dispatch_wait_collect_us_per_tx": dispatch / 1000,
        "catalog_publication_install_dirty_us_per_tx": publication / 1000,
        "wal_cpu_us_per_tx": wal / 1000,
        "wal_sync_us_per_tx": components["WAL sync"] / 1000,
        "unattributed_us_per_tx": components["unattributed"] / 1000,
        "worker": {name: value / 1000 for name, value in summary["worker_ns_per_tx"].items()},
    }


def table(lines, all_summaries, phase, rows, variants, baseline):
    lines.append(f"| Run | " + " | ".join(f"{variant} tx/s (runs)" for variant in variants) + " | "
                 + " | ".join(f"{variant} / {baseline}" for variant in variants if variant != baseline) + " |")
    lines.append("|---|" + "---|" * (2 * len(variants) - 1))
    ratios = {variant: [] for variant in variants if variant != baseline}
    for sync, scenario in rows:
        items = [all_summaries.get((phase, sync, scenario, variant)) for variant in variants]
        if any(item is None for item in items):
            continue
        base = all_summaries[(phase, sync, scenario, baseline)]["tx_per_second"]
        cells = [f"{item['tx_per_second']:,.0f} ({', '.join(f'{run:,.0f}' for run in item['tx_per_second_runs'])})" for item in items]
        ratio_cells = []
        for variant, item in zip(variants, items):
            if variant != baseline:
                ratio = item["tx_per_second"] / base
                ratios[variant].append(ratio)
                ratio_cells.append(f"{ratio:.3f}")
        lines.append(f"| {label(items[0])} sync {sync} | " + " | ".join(cells) + " | " + " | ".join(ratio_cells) + " |")
    lines.append("")
    return ratios


def coordinator_table(lines, all_summaries, phase, sync, scenario, variants):
    items = [(variant, all_summaries.get((phase, sync, scenario, variant))) for variant in variants]
    items = [(variant, item) for variant, item in items if item]
    if not items:
        return
    lines.append(f"Coordinator µs per transaction, {label(items[0][1])}, sync {sync} ({phase}):")
    lines.append("")
    lines.append("| µs / tx | " + " | ".join(variant for variant, _ in items) + " |")
    lines.append("|---|" + "---|" * len(items))
    views = [coordinator_view(item) for _, item in items]
    for key in ("coordinator_us_per_tx", "planner_us_per_tx", "leaf_on_coordinator_us_per_tx", "lanes_busy_us_per_tx",
                "dispatch_wait_collect_us_per_tx", "catalog_publication_install_dirty_us_per_tx", "wal_cpu_us_per_tx",
                "wal_sync_us_per_tx", "unattributed_us_per_tx"):
        lines.append(f"| {key.replace('_us_per_tx', '').replace('_', ' ')} | " + " | ".join(f"{view[key]:.1f}" for view in views) + " |")
    for name in views[0]["worker"]:
        lines.append(f"| lane: {name} | " + " | ".join(f"{view['worker'][name]:.1f}" for view in views) + " |")
    lines.append("")


def main():
    all_summaries = summaries()
    spec_text = json.loads(os.environ.get("PHASE_E_TABLES", "[]"))
    lines = []
    output = {}
    for entry in spec_text:
        phase, variants, baseline = entry["phase"], entry["variants"], entry["baseline"]
        rows = [tuple(row) for row in entry["rows"]]
        lines.append(f"## {entry['title']}")
        lines.append("")
        ratios = table(lines, all_summaries, phase, rows, variants, baseline)
        for variant, values in ratios.items():
            if len(values) > 1:
                lines.append(f"GM {variant} / {baseline} over {len(values)} rows: {geometric_mean(values):.3f}")
        lines.append("")
        for sync, scenario in entry.get("coordinator_rows", []):
            coordinator_table(lines, all_summaries, phase, sync, scenario, variants)
        output[entry["title"]] = {
            f"{sync}-{scenario:02d}": {variant: {**all_summaries[(phase, sync, scenario, variant)],
                                                 "coordinator_view": coordinator_view(all_summaries[(phase, sync, scenario, variant)])}
                                       for variant in variants if (phase, sync, scenario, variant) in all_summaries}
            for sync, scenario in rows
        }
    print("\n".join(lines))
    if len(sys.argv) > 1 and os.environ.get("PHASE_E_JSON"):
        json.dump(output, open(os.environ["PHASE_E_JSON"], "w", encoding="utf-8"), indent=1, sort_keys=True)


if __name__ == "__main__":
    main()
