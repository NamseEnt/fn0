import collections
import json
import re
import subprocess
import sys

CATEGORIES = (
    ("memcpy / memmove", re.compile(r"memcpy|memmove")),
    ("malloc / free", re.compile(r"^(_int_malloc|malloc|cfree@.*|cfree|free|_int_free|malloc_consolidate|realloc|_int_realloc|"
                                 r"__libc_malloc|__libc_free|tcache.*|unlink_chunk.*|__aarch64_swp4_rel|__aarch64_cas8_rel|"
                                 r"__aarch64_cas4_acq|__aarch64_cas4_rel|__aarch64_swp4_acq)$")),
    ("Arc refcount atomics (ldadd8)", re.compile(r"^__aarch64_ldadd8_")),
    ("memcmp", re.compile(r"^(memcmp|__memcmp.*|bcmp)$")),
    ("CRC32C", re.compile(r"crc32c")),
    ("planner (plan_batch self)", re.compile(r"blink::plan_batch$")),
    ("PageDelta encode/apply", re.compile(r"canonical_delta_spans|encode_page_delta|apply_page_delta|decode_page_delta")),
)
STAGES = (
    ("admission", re.compile(r"validate_and_encode_mutation_keys|LogicalOverlay|accept_preencoded|validate_conditions")),
    ("planner", re.compile(r"blink::plan_batch")),
    ("lane: leaf clone", re.compile(r"run_leaf_chain_job")),
    ("dispatch", re.compile(r"prepare_leaf_parallel_execution")),
    ("catalog", re.compile(r"prepare_catalog_delta|prepare_delta")),
    ("publication", re.compile(r"GenerationPublisher>::publish|GenerationPublisher::publish")),
    ("WAL", re.compile(r"wal::|WalLog")),
    ("serial execution", re.compile(r"prepare_planned_serial_execution")),
    ("group (apply_planned_transaction_group)", re.compile(r"apply_planned_transaction_group|apply_transaction_group")),
    ("harness", re.compile(r"phase0_bench")),
)


def demangle(symbols):
    process = subprocess.run(["rustfilt"], input="\n".join(symbols) + "\n", capture_output=True, text=True, check=True)
    readable = process.stdout.splitlines()
    return {original: re.sub(r"::h[0-9a-f]{16}$", "", name) for original, name in zip(symbols, readable)}


def category(symbol):
    for name, pattern in CATEGORIES:
        if pattern.search(symbol):
            return name
    return None


def main():
    perf_data = sys.argv[1]
    transactions = float(sys.argv[2])
    output = subprocess.run(["perf", "script", "-i", perf_data, "-F", "comm,tid,ip,sym"],
                            capture_output=True, text=True, check=True).stdout
    raw_samples = []
    symbols = set()
    for block in output.split("\n\n"):
        lines = block.strip().splitlines()
        if len(lines) < 2:
            continue
        comm = lines[0].split()[0]
        frames = []
        for line in lines[1:]:
            parts = line.strip().split(None, 1)
            if len(parts) == 2:
                symbol = re.sub(r"\.llvm\.\d+$", "", parts[1].split("+0x")[0])
                frames.append(symbol)
                symbols.add(symbol)
        raw_samples.append((comm, frames))
    names = demangle(sorted(symbols))
    event_count = None
    header = subprocess.run(["perf", "report", "-i", perf_data, "--stdio", "--header-only"], capture_output=True, text=True).stdout
    totals = collections.Counter()
    direct_callers = collections.defaultdict(collections.Counter)
    stages = collections.defaultdict(collections.Counter)
    sample_count = 0
    for comm, frames in raw_samples:
        readable = [names.get(frame, frame) for frame in frames if frame != "[unknown]"]
        if not readable:
            continue
        sample_count += 1
        leaf_category = category(readable[0])
        if not leaf_category:
            continue
        totals[leaf_category] += 1
        caller = "(none)"
        for frame in readable[1:]:
            if "dodb" in frame or "phase0_bench" in frame:
                caller = frame[:160]
                break
        direct_callers[leaf_category][caller] += 1
        stage = "other"
        for frame in readable[1:]:
            matched = [name for name, pattern in STAGES if pattern.search(frame)]
            if matched:
                stage = matched[0]
                break
        if comm.startswith("dodb-blink-leaf") and stage in ("other",):
            stage = "lane: leaf clone"
        stages[leaf_category][stage] += 1
    result = {
        "samples": sample_count,
        "transactions": transactions,
        "category_share": {name: count / sample_count for name, count in totals.items()},
        "stages": {name: {stage: count / sample_count for stage, count in counter.most_common()} for name, counter in stages.items()},
        "direct_callers": {name: [(caller, count / sample_count) for caller, count in counter.most_common(20)]
                           for name, counter in direct_callers.items()},
    }
    json.dump(result, sys.stdout, indent=1)


if __name__ == "__main__":
    main()
