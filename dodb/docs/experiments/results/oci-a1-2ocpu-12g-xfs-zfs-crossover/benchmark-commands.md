# Benchmark Commands

The remote helper is `scripts/run_sample.py`. Each invocation records its exact argv, source commit, binary SHA-256, UTC start/end, exit code, output path, log path, and `/dev/sda` disk-stat deltas. It writes raw output and logs under `/dev/shm/xfs-zfs-crossover/raw/<stage>/`; `scripts/run_stage.py` copies those files into this experiment tree when each stage finishes.

## fio WAL-like payload sweep

The same command structure is used at every point. Only `--bs` changes among 4096, 16384, 65536, 262144, and 1048576 bytes. Output and preallocated file names vary by stage and point.

```sh
fio --name=wal-payload-fdatasync --filename=/bench/btrfs/db/xfs-zfs-crossover-working/fio-payload-PAYLOAD_BYTES.dat --rw=write --bs=PAYLOAD_BYTES --ioengine=psync --iodepth=1 --direct=0 --size=268435456 --time_based=1 --runtime=20 --fdatasync=1 --group_reporting=1 --allow_file_create=0 --output-format=json --output=/dev/shm/xfs-zfs-crossover/raw/STAGE/fio/RESULT.json
```

Each payload point has its own newly preallocated 256 MiB file per stage. The five repetitions for a point reuse that point's file. Payload refinement is run only if XFS and ZFS reverse median operations-per-second ordering at adjacent initial points; the midpoint uses a fresh same-sized file and five repetitions.

## dodb transaction-width sweep

The command arguments remain identical for all widths except `--widths` and the matching deterministic `--seed`. Widths are 1, 2, 4, 8, and 16. The seeds are `979200000 + width * 100 + repetition` and are identical across stages.

```sh
DODB_BENCH_DIR=/bench/btrfs/db/xfs-zfs-crossover-working/data /var/oled/xfs-zfs-crossover/target/release/phase0-bench --suite write --engine parallel-blink --writers 64 --widths WIDTH --distributions uniform --duration 5s --warmup 2s --repetitions 1 --cache-capacity 256 --working-set 100000 --key-size 16 --value-size 64 --group-limit 64 --group-bytes 4194304 --queue-capacity 256 --collection-delay 0us --sync-mode real --tokio-workers 2 --blink-workers 2 --transaction-mode unconditional --seed SEED --output /dev/shm/xfs-zfs-crossover/raw/STAGE/dodb/RESULT.jsonl
```

Every workload point runs five single-repetition invocations. The raw row contains `wal_bytes_delta`, `wal_syncs_delta`, `wal_sync_nanos_total`, `successful_transactions`, `avg_group_requests`, `groups`, `planning_nanos`, `validation_nanos_total`, `publication_nanos_total`, `physical_execution_nanos`, and latency/CPU fields. The analyzer computes WAL bytes per sync, successful transactions per sync, mean WAL sync latency, and groups per second from those per-run values.
