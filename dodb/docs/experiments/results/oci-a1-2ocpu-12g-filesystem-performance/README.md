# OCI A1 dodb Filesystem Performance Comparison

## Scope

This experiment compares ext4, XFS, Btrfs, and ZFS performance for the same dodb release binary on the same OCI A1 boot-volume partition. It evaluates filesystem performance only. It does not compare snapshots, backups, S3, send/receive, or operational convenience, and it does not select a filesystem.

The required storage-safety gate is complete. `/dev/sda4` is a 153.4 GiB partition labeled `dodbbench`, with GPT partition label `dodb-zfs`, mounted only at `/bench/btrfs/db`. The root, boot, EFI, LVM, and swap devices are separate. The partition is the benchmark filesystem and is not production data. Existing Btrfs scaling result artifacts are already committed in `oci-a1-2ocpu-12g-btrfs-scaling/`; temporary build caches on the benchmark partition are not source artifacts.

## Fixed inputs

- OCI instance: `VM.Standard.A1.Flex`, 2 OCPUs, 12 GB RAM, same instance and boot volume throughout.
- Device and partition: `/dev/sda4`, same 153.4 GiB partition throughout.
- dodb source: `08e82ec484bd62aeb659471a39f317b86cfb488f`.
- dodb engine: `parallel-blink`.
- Release binary SHA-256: `765f6b3b33f7a85dfaebdeb5582eb10c64ad6a3222cf354146fc8fc13fe08a4e`.
- The binary was built once from the pinned source before this matrix and reused unchanged. Its hash was checked on the benchmark host before testing.
- dodb: 64 writers, working set 100,000, 16-byte keys, 64-byte values, 2-second warmup, 5-second measurement, 5 repetitions, real sync, unconditional transactions, cache capacity 256, group limit 64, group bytes 4 MiB, queue capacity 256, collection delay 0, 2 Tokio workers, 2 Blink workers.
- fio: fio 3.35, 256 MiB preallocated file, buffered I/O, `psync`, 4 KiB operations, one `fdatasync` per operation, time based, 20 seconds, 5 repetitions.

## Run order

1. Btrfs-pre
2. ext4
3. XFS
4. ZFS
5. Btrfs-post

Each filesystem uses the same partition, mount directory, binary, command arguments, working set, test duration, and repetition count. The Btrfs pre/post pair measures time-related drift.

## Results

The median and CV tables, fastest-metric comparison, observed run variation, and Btrfs drift are in [`summary-tables.md`](summary-tables.md). Exact filesystem commands are in [`filesystem-setup.md`](filesystem-setup.md); fio and dodb invocations are in [`benchmark-commands.md`](benchmark-commands.md). Environment and source/binary provenance are in [`environment.md`](environment.md). UTC stage bounds are in [`run-order.csv`](run-order.csv). Raw fio JSON, dodb JSONL, logs, per-run block counters, and host snapshots are in [`raw/`](raw/).

One initial Btrfs-pre width-1 dodb pilot had `git_commit=unknown`, so it is excluded from all summaries. Its original JSON, JSONL, and metadata are preserved in `raw/rejected-provenance/`; the five accepted repetitions per workload all report the pinned source commit.

For direct throughput and latency medians, Btrfs was slowest in 16 of 18 metrics; ZFS was slowest in dodb width-16 p95 and p99. ZFS had the highest dodb width-1 median throughput (23,344 tx/s), while XFS had the highest width-16 median throughput (4,949 tx/s). The comparison table marks whether each top-two median gap exceeded the combined within-filesystem sample standard deviation. These are observed performance results only; no filesystem is selected.

Btrfs-pre to Btrfs-post changes ranged from +3.4% sequential fio IOPS to -4.7% fdatasync p50 latency, +0.5% width-1 dodb throughput, and +2.0% width-16 throughput. The Btrfs pre/post interpretation and its filesystem-age limitation are documented in `environment.md`.
