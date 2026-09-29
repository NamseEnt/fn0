import json
import re
import sys

CATEGORIES = (
    ("memcpy / memmove", re.compile(r"memcpy|memmove")),
    ("malloc / free", re.compile(r"^(_int_malloc|malloc|cfree@.*|free|_int_free|malloc_consolidate|realloc|_int_realloc|"
                                 r"__libc_malloc|__libc_free|tcache.*|unlink_chunk.*|__aarch64_swp4_rel|__aarch64_cas8_rel|"
                                 r"__aarch64_cas4_acq|__aarch64_cas4_rel|__aarch64_swp4_acq)$")),
    ("Arc refcount atomics (ldadd8)", re.compile(r"^__aarch64_ldadd8_")),
    ("memcmp", re.compile(r"^(memcmp|__memcmp.*|bcmp)$")),
)
EXTRA_CATEGORIES = (
    ("CRC32C", re.compile(r"crc32c")),
    ("planner (plan_batch self)", re.compile(r"plan_batch")),
    ("PageDelta (spans, encode, decode, apply, check)", re.compile(r"canonical_delta_spans|encode_page_delta|decode_page_delta|apply_page_delta|page_delta_rebuilds|check_page_delta_against_base")),
)


def parse(path):
    event_count = None
    rows = []
    for line in open(path, encoding="utf-8", errors="replace"):
        match = re.match(r"# Event count \(approx\.\): (\d+)", line)
        if match:
            event_count = int(match.group(1))
        match = re.match(r"\s+([\d.]+)%\s+\[[.k]\]\s+(\S+)", line)
        if match:
            rows.append((float(match.group(1)), match.group(2)))
    seen = set()
    unique = []
    for percent, symbol in rows:
        if symbol not in seen:
            seen.add(symbol)
            unique.append((percent, symbol))
    return event_count, unique


def main():
    output = {}
    for argument in sys.argv[1:]:
        label, path, tx_per_second, seconds = argument.split(",")
        event_count, rows = parse(path)
        transactions = float(tx_per_second) * float(seconds)
        shares = {name: 0.0 for name, _ in CATEGORIES}
        for percent, symbol in rows:
            for name, pattern in CATEGORIES:
                if pattern.search(symbol):
                    shares[name] += percent
                    break
        total = sum(shares.values())
        extra = {name: 0.0 for name, _ in EXTRA_CATEGORIES}
        for percent, symbol in rows:
            if any(pattern.search(symbol) for _, pattern in CATEGORIES):
                continue
            for name, pattern in EXTRA_CATEGORIES:
                if pattern.search(symbol):
                    extra[name] += percent
                    break
        output[label] = {
            "event_count": event_count,
            "transactions_estimate": transactions,
            "shares_percent": shares,
            "total_percent": total,
            "cycles_per_tx": {name: event_count * share / 100 / transactions for name, share in shares.items()},
            "all_cycles_per_tx": event_count / transactions,
            "copy_alloc_refcount_cycles_per_tx": event_count * total / 100 / transactions,
            "extra_shares_percent": extra,
            "extra_cycles_per_tx": {name: event_count * share / 100 / transactions for name, share in extra.items()},
        }
    print(json.dumps(output, indent=2))
    labels = list(output)
    print("\n| Category | " + " | ".join(f"{label} % | {label} cycles/tx" for label in labels) + " |")
    print("|---|" + "---|---|" * len(labels))
    for name, _ in CATEGORIES:
        print(f"| {name} | " + " | ".join(
            f"{output[label]['shares_percent'][name]:.1f}% | {output[label]['cycles_per_tx'][name]:,.0f}" for label in labels) + " |")
    print("| **total copy/alloc/refcount** | " + " | ".join(
        f"{output[label]['total_percent']:.1f}% | {output[label]['copy_alloc_refcount_cycles_per_tx']:,.0f}" for label in labels) + " |")
    for name, _ in EXTRA_CATEGORIES:
        print(f"| {name} | " + " | ".join(
            f"{output[label]['extra_shares_percent'][name]:.1f}% | {output[label]['extra_cycles_per_tx'][name]:,.0f}" for label in labels) + " |")
    print("| all samples | " + " | ".join(f"100% | {output[label]['all_cycles_per_tx']:,.0f}" for label in labels) + " |")


if __name__ == "__main__":
    main()
