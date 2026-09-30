# Benchmark commands

The helper in `scripts/run_sample.py` runs every test and records the exact argument vector, wall time, `/dev/sda` block counters before and after, exit status, log path, and raw output path in the matching `.meta.json` file. The helper sends output to tmpfs so result-file writes do not enter the tested filesystem. The database and fio data files reside under the tested filesystem.

## Common fio file

For each filesystem, the same path and size were used:

```sh
fallocate -l 268435456 /bench/btrfs/db/fsperf-working/fio.dat
```

The file was created once per filesystem, retained for all fio repetitions, and not deleted between the sequential and random-overwrite jobs. The fio job used `allow_file_create=0`.

## Sequential durable sync

```sh
fio --name=wal-like-fdatasync --filename=/bench/btrfs/db/fsperf-working/fio.dat --rw=write --bs=4k --ioengine=psync --iodepth=1 --direct=0 --size=268435456 --time_based=1 --runtime=20 --fdatasync=1 --group_reporting=1 --allow_file_create=0 --output-format=json --output=/dev/shm/dodb-filesystem-performance/raw/RESULT.json
```

## 4 KiB random overwrite

```sh
fio --name=wal-like-fdatasync --filename=/bench/btrfs/db/fsperf-working/fio.dat --rw=randwrite --bs=4k --ioengine=psync --iodepth=1 --direct=0 --size=268435456 --time_based=1 --runtime=20 --fdatasync=1 --group_reporting=1 --allow_file_create=0 --randrepeat=1 --randseed=20260930 --norandommap=1 --output-format=json --output=/dev/shm/dodb-filesystem-performance/raw/RESULT.json
```

Each fio workload ran 5 times per filesystem. `RESULT.json` was unique per filesystem, workload, and repetition.

## dodb durable writes

The helper used this command shape for each width and repetition:

```sh
DODB_BENCH_DIR=/bench/btrfs/db/fsperf-working/data /home/opc/phase0-bench-filesystem-matrix --suite write --engine parallel-blink --writers 64 --widths WIDTH --distributions uniform --duration 5s --warmup 2s --repetitions 1 --cache-capacity 256 --working-set 100000 --key-size 16 --value-size 64 --group-limit 64 --group-bytes 4194304 --queue-capacity 256 --collection-delay 0us --sync-mode real --tokio-workers 2 --blink-workers 2 --transaction-mode unconditional --seed SEED --output /dev/shm/dodb-filesystem-performance/raw/RESULT.jsonl
```

`WIDTH` was 1 or 16. Seeds were identical across filesystems: width 1 used `979100101` through `979100105`; width 16 used `979101601` through `979101605`. Each workload ran as five single-repetition invocations so block-device counters could be preserved per repetition.

## Repetitions and order

The order was Btrfs-pre, ext4, XFS, ZFS, Btrfs-post. Within each stage, the order was five fio sequential runs, five fio random-overwrite runs, five dodb width-1 runs, then five dodb width-16 runs. `run-order.csv` contains the captured UTC bounds for each stage.
