# Phase I Overlay CPU Prototype

This standalone benchmark-only program models an immutable B-link base plus immutable sorted overlay segments and immutable published views. It does not provide durable overlay storage. Its file adapter deliberately makes `sync_data()` and `sync_all()` no-ops. Do not interpret its throughput or CPU numbers as durability performance.

The program exercises ordered transaction admission, transient group overlay visibility, conditions, Put/Delete, tombstones, pinned views, point GET, Query, and Scan. It does not change production WAL v3 or production storage behavior.

The measured OCI binary was built from the real checkout at `42e53838e7ebabcd5f48ef1e015e0f71e2758668`, with SHA256 `0675ddff2c8bdfd6dc6649d13643329c160538ed23e88fd40d09dccfbb390aa6`. Host, source, binary, and repetition provenance are in `raw/run-order.jsonl`; raw JSONL measurements are in `raw/oci-a1-overlay.jsonl`, and perf counters are retained beside them. Run `python3 scripts/analyze_prototype.py` from the Phase I result directory to reproduce medians and validate source SHA and row counts.
