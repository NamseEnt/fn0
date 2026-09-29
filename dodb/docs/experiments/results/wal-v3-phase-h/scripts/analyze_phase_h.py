import json
import re
import statistics
from collections import defaultdict
from pathlib import Path


artifact_root = Path(__file__).resolve().parents[1]
expected_commits = {
    "g0": "a78472377fa16ed491c18bd2d6770f7dba7e9311",
    "h1": "5998f7c47a15850c40bb40c8c378ac5c0deca061",
}
expected_binary_sha256 = {
    "g0": "33be90a6707544a7725acdb140da8394c4de5affdd5266675720c61ed7499e83",
    "h1": "0ac99953510c5f0977b96fd2883547140a284719f45458d5a62869ab8c69badc",
}
groups = defaultdict(list)

for run_order_path in sorted((artifact_root / "raw" / "interleaved").rglob("run-order.jsonl")):
    for line in run_order_path.read_text(encoding="utf-8").splitlines():
        event = json.loads(line)
        variant = event["variant"]
        if event.get("git_commit") != expected_commits[variant]:
            raise SystemExit(f"{run_order_path} has an unexpected source commit")
        if event.get("binary_sha256") != expected_binary_sha256[variant]:
            raise SystemExit(f"{run_order_path} has an unexpected binary SHA256")

for source_path in sorted((artifact_root / "raw" / "interleaved").rglob("*.jsonl")):
    variant = source_path.name.split("-", 1)[0]
    if variant not in expected_commits:
        continue
    log_path = source_path.with_suffix(".log")
    log_text = log_path.read_text(encoding="utf-8")
    cycles_match = re.search(r"(?m)^([0-9,]+),,cycles:u,", log_text)
    if cycles_match is None:
        raise SystemExit(f"missing cycles:u in {log_path}")
    for line in source_path.read_text(encoding="utf-8").splitlines():
        row = json.loads(line)
        if row.get("record_type") != "run":
            continue
        expected_commit = expected_commits[variant]
        if row.get("git_commit") != expected_commit:
            raise SystemExit(
                f"{source_path} git_commit={row.get('git_commit')} expected={expected_commit}"
            )
        if row.get("successful_transactions", 0) <= 0:
            raise SystemExit(f"{source_path} has no successful transactions")
        relative = source_path.relative_to(artifact_root / "raw" / "interleaved")
        scenario_set, sync_mode = relative.parts[:2]
        groups[(scenario_set, sync_mode, row["writers"], row["transaction_width"], row["distribution"], variant)].append(
            {
                "throughput": row["logical_tx_per_second"],
                "cycles_per_tx": int(cycles_match.group(1).replace(",", ""))
                / row["successful_transactions"],
                "commit": row["git_commit"],
                "file": str(source_path.relative_to(artifact_root)),
            }
        )

print("# Phase H paired benchmark results\n")
print("| Set | Sync | Writers | Width | Distribution | Reps | G0 tx/s | H1 tx/s | H1/G0 | G0 cycles/tx | H1 cycles/tx | G0/H1 cycles |")
print("|---|---|---:|---:|---|---:|---:|---:|---:|---:|---:|---:|")
for key in sorted({entry[:5] for entry in groups}):
    scenario_set, sync_mode, writers, width, distribution = key
    g0 = groups[(*key, "g0")]
    h1 = groups[(*key, "h1")]
    if len(g0) != 3 or len(h1) != 3:
        raise SystemExit(f"{key} expected three paired repetitions")
    g0_throughput = statistics.median(value["throughput"] for value in g0)
    h1_throughput = statistics.median(value["throughput"] for value in h1)
    g0_cycles = statistics.median(value["cycles_per_tx"] for value in g0)
    h1_cycles = statistics.median(value["cycles_per_tx"] for value in h1)
    print(
        f"| {scenario_set} | {sync_mode} | {writers} | {width} | {distribution} | 3 | {g0_throughput:.0f} | {h1_throughput:.0f} | {h1_throughput / g0_throughput:.3f}x | {g0_cycles:.0f} | {h1_cycles:.0f} | {g0_cycles / h1_cycles:.3f}x |"
    )

print("\n## Binary provenance\n")
print("| Variant | Source SHA | Binary SHA256 | Runtime checkout |")
print("|---|---|---|---|")
print(f"| G0 | `{expected_commits['g0']}` | `{expected_binary_sha256['g0']}` | `/tmp/dodb-phase-h-baseline` |")
print(f"| H1 | `{expected_commits['h1']}` | `{expected_binary_sha256['h1']}` | `/home/opc/dodb-wal-v3` |")

baseline_rows = [
    json.loads(source_path.read_text(encoding="utf-8").splitlines()[0])
    for source_path in sorted((artifact_root / "raw" / "interleaved" / "gate" / "disabled").glob("g0-*-w64-width16-uniform.jsonl"))
]


def median_per_tx(field):
    return statistics.median(row[field] / row["successful_transactions"] for row in baseline_rows)


transaction_count = 1.0
page_record_calls = median_per_tx("wal_page_delta_records_delta") + median_per_tx("wal_page_image_records_delta")
page_record_payload_bytes = median_per_tx("wal_page_delta_payload_bytes_delta") + median_per_tx(
    "wal_page_image_records_delta"
) * (4096 + 8)
print("\n## H0 call-site attribution, 64w width16 uniform sync-disabled\n")
print("| Site | Calls/tx | Bytes/tx | Timing/tx |")
print("|---|---:|---:|---:|")
print(
    f"| A. Canonical page checksum | {median_per_tx('leaf_encodes'):.3f} | {median_per_tx('leaf_encodes') * 4096:.0f} | {median_per_tx('parallel_worker_encode_nanos_total') / 1000:.2f} µs worker encode stage |"
)
print(
    f"| B. Full image fingerprint | {page_record_calls:.3f} | {page_record_calls * 4096:.0f} | {median_per_tx('parallel_worker_delta_nanos_total') / 1000:.2f} µs worker delta stage |"
)
base_validation_calls = median_per_tx("parallel_leaf_jobs_delta") - median_per_tx("wal_page_image_records_delta")
print(
    f"| C. Base page-chain validation | {base_validation_calls:.3f} | {base_validation_calls * 4096:.0f} | {median_per_tx('parallel_worker_base_nanos_total') / 1000:.2f} µs worker base stage |"
)
print(
    f"| D. WAL redo payload checksum | {page_record_calls:.3f} | {page_record_payload_bytes:.0f} | {median_per_tx('wal_group_page_payload_crc_nanos_total') / 1000:.3f} µs |"
)
digest_bytes = page_record_payload_bytes + page_record_calls * 9
print(
    f"| E. Commit digest | {page_record_calls:.3f} updates | {digest_bytes:.0f} | {median_per_tx('wal_group_commit_digest_crc_nanos_total') / 1000:.3f} µs |"
)
header_calls = page_record_calls + 2
header_bytes = page_record_calls * 48 + transaction_count * 48 + transaction_count * 16
header_time = median_per_tx("wal_group_page_header_crc_nanos_total") + median_per_tx(
    "wal_group_commit_header_crc_nanos_total"
)
print(
    f"| F. Frame headers and commit payload | {header_calls:.3f} | {header_bytes:.0f} | {header_time / 1000:.3f} µs headers; {median_per_tx('wal_group_commit_payload_crc_nanos_total') / 1000:.3f} µs commit payload |"
)
print("| G. Open/recovery/checkpoint | Outside transaction denominator | Full raw-page and frame validation | Per-operation only |")

print("\n## Profile-derived CRC cycles\n")
print("| Variant | Transactions | Total sampled cycles/tx | CRC sample share | CRC cycles/tx, estimated |")
print("|---|---:|---:|---:|---:|")
profile_values = {}
for variant in ("g0", "h1"):
    profile_path = artifact_root / "raw" / "profiles" / f"{variant}-profile.jsonl"
    profile_row = json.loads(profile_path.read_text(encoding="utf-8").splitlines()[0])
    if profile_row.get("git_commit") != expected_commits[variant]:
        raise SystemExit(f"{profile_path} git_commit does not match {variant}")
    report = (artifact_root / "perf" / "verified" / f"{variant}-flat-report.txt").read_text(encoding="utf-8")
    event_match = re.search(r"Event count \(approx\.\): ([\d,]+)", report)
    if event_match is None:
        raise SystemExit(f"{variant} perf report has no cycle event count")
    event_count = int(event_match.group(1).replace(",", ""))
    crc_share = sum(
        float(symbol_match.group(1))
        for line in report.splitlines()
        if (symbol_match := re.match(r"\s*([\d.]+)%\s+\[\.\]\s+(.*)", line))
        and "crc32c" in symbol_match.group(2).lower()
    )
    total_cycles_per_tx = event_count / profile_row["successful_transactions"]
    crc_cycles_per_tx = total_cycles_per_tx * crc_share / 100
    profile_values[variant] = (total_cycles_per_tx, crc_share, crc_cycles_per_tx)
    print(
        f"| {variant.upper()} | {profile_row['successful_transactions']} | {total_cycles_per_tx:.0f} | {crc_share:.2f}% | {crc_cycles_per_tx:.0f} |"
    )
print(
    f"\nCRC estimated cycles reduction: {(1 - profile_values['h1'][2] / profile_values['g0'][2]) * 100:.1f}%; total sampled cycles/tx reduction: {(1 - profile_values['h1'][0] / profile_values['g0'][0]) * 100:.1f}%."
)
