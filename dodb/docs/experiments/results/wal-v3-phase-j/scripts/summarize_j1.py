import json
import pathlib
import sys


def main():
    sources = [pathlib.Path(argument) for argument in sys.argv[1:]]
    records = [
        json.loads(line)
        for source in sources
        for line in source.read_text().splitlines()
        if line.startswith("{")
    ]
    cpu_runs = {}
    for record in records:
        if record.get("record_type") == "write_cpu":
            cpu_runs[record["engine"], record["segments_before"]] = record
    h1 = cpu_runs["h1_physical", 0]
    print("engine,segments,cpu_ns_per_tx,cpu_ratio_vs_h1,allocations_per_tx")
    for engine, segment_count in sorted(cpu_runs):
        record = cpu_runs[engine, segment_count]
        print(
            f"{engine},{segment_count},{record['cpu_ns_per_tx']:.2f},"
            f"{record['cpu_ns_per_tx'] / h1['cpu_ns_per_tx']:.5f},"
            f"{record['allocations_per_tx']:.3f}"
        )


if __name__ == "__main__":
    main()
