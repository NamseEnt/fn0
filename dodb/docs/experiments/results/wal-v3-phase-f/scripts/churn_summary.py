import glob
import json
import os
import sys
from collections import defaultdict

SITES = ("admission", "planner", "serial_execution", "dispatch", "lane", "worker_thread", "collect", "catalog",
         "wal_assembly", "wal_append", "state_install", "publication", "dirty_tracking", "group_other", "harness")
COUNTERS = ("alloc_calls", "free_calls", "realloc_calls", "alloc_bytes", "free_bytes",
            "leaf_entry_clones", "leaf_entry_drops", "arc_key_clones", "arc_value_clones", "arc_key_drops",
            "arc_value_drops", "payload_arcs_created", "leaf_page_clones", "leaf_vec_capacity_bytes",
            "internal_page_clones", "blink_page_arc_clones", "page_image_copies", "page_image_bytes_copied",
            "page_image_buffers", "page_encodes", "dirty_page_inserts", "dirty_page_replaces",
            "dirty_page_bytes_copied", "delta_payload_buffers", "delta_payload_bytes", "delta_verify_images",
            "planner_map_inserts", "planner_key_copies", "planner_mutation_clones", "jobs_built",
            "leaf_entries_copied", "leaf_slot_bytes_copied", "leaf_payload_bytes_copied", "leaf_compactions",
            "leaf_key_comparisons")


def summarize(paths):
    totals = defaultdict(float)
    transactions = 0
    leaf_jobs = 0
    mutations = 0
    tx_per_second = []
    for path in paths:
        row = json.loads(open(path, encoding="utf-8").readline())
        if row.get("churn_counters") != "enabled":
            raise SystemExit(f"{path}: churn counters disabled")
        transactions += row["successful_transactions"]
        leaf_jobs += row.get("parallel_leaf_jobs_delta", 0)
        mutations += row["mutation_ops"]
        tx_per_second.append(row["logical_tx_per_second"])
        for key, value in row.items():
            if key.startswith("churn_") and key != "churn_counters":
                totals[key[len("churn_"):]] += value
    per_tx = {}
    for site in SITES:
        for counter in COUNTERS:
            per_tx[(site, counter)] = totals.get(f"{site}_{counter}", 0.0) / transactions
    return {
        "files": [os.path.basename(path) for path in paths],
        "transactions": transactions,
        "leaf_jobs_per_tx": leaf_jobs / transactions,
        "mutations_per_tx": mutations / transactions,
        "tx_per_second_instrumented": tx_per_second,
        "per_tx": per_tx,
    }


def table(summary, counters, title):
    lines = [f"#### {title}", ""]
    used_sites = [site for site in SITES if any(summary["per_tx"][(site, counter)] >= 0.005 for counter in counters)]
    lines.append("| per transaction | " + " | ".join(used_sites) + " | **total** |")
    lines.append("|---|" + "---|" * (len(used_sites) + 1))
    for counter in counters:
        values = [summary["per_tx"][(site, counter)] for site in used_sites]
        total = sum(summary["per_tx"][(site, counter)] for site in SITES)
        if total < 0.005:
            continue
        digits = 0 if total >= 100 else 1 if total >= 10 else 2
        lines.append(f"| {counter} | " + " | ".join(f"{value:,.{digits}f}" if value else "" for value in values)
                     + f" | **{total:,.{digits}f}** |")
    lines.append("")
    return lines


def main():
    raw = sys.argv[1]
    output = {}
    lines = []
    for label, pattern in json.loads(sys.argv[2]).items():
        paths = sorted(glob.glob(os.path.join(raw, pattern)))
        if not paths:
            continue
        summary = summarize(paths)
        output[label] = {**summary, "per_tx": {f"{site}.{counter}": value for (site, counter), value in summary["per_tx"].items() if value}}
        lines.append(f"### {label}")
        lines.append("")
        lines.append(f"Files: {', '.join(summary['files'])}. {summary['transactions']:,} transactions, "
                     f"{summary['mutations_per_tx']:.2f} mutations/tx, {summary['leaf_jobs_per_tx']:.2f} leaf jobs/tx, "
                     f"instrumented tx/s {', '.join(f'{value:,.0f}' for value in summary['tx_per_second_instrumented'])} (not a headline).")
        lines.append("")
        lines += table(summary, ("alloc_calls", "free_calls", "realloc_calls", "alloc_bytes", "free_bytes"), "Allocator")
        lines += table(summary, ("leaf_entry_clones", "leaf_entry_drops", "arc_key_clones", "arc_value_clones",
                                 "arc_key_drops", "arc_value_drops", "payload_arcs_created", "leaf_page_clones",
                                 "leaf_vec_capacity_bytes", "internal_page_clones", "blink_page_arc_clones"),
                       "Leaf ownership")
        lines += table(summary, ("page_image_copies", "page_image_bytes_copied", "page_image_buffers", "page_encodes",
                                 "dirty_page_inserts", "dirty_page_replaces", "dirty_page_bytes_copied",
                                 "delta_payload_buffers", "delta_payload_bytes", "delta_verify_images"),
                       "4 KiB images, dirty map, PageDelta")
        lines += table(summary, ("planner_map_inserts", "planner_key_copies", "planner_mutation_clones", "jobs_built",
            "leaf_entries_copied", "leaf_slot_bytes_copied", "leaf_payload_bytes_copied", "leaf_compactions",
            "leaf_key_comparisons"),
                       "Planner and jobs")
    print("\n".join(lines))
    json.dump(output, open(sys.argv[3], "w", encoding="utf-8"), indent=1, sort_keys=True)


if __name__ == "__main__":
    main()
