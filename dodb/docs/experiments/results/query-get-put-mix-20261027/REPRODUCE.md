# Reproduction

Use only the authorized AArch64 OCI host `opc@217.142.246.204` and ZFS
`/bench/zfs/db`. Keep ordinary SSH host-key validation and the configured SSH
identity. The run root is `/bench/zfs/db/query-get-put-mix-20261027`. The
controller refuses to start any measured cell while free benchmark-filesystem
space is below 10 GiB, available host memory is below 1,500 MiB, process-group
RSS reaches 3 GiB, or a case database reaches 4 GiB.

Create a detached clean worktree at the source revision recorded in
`source-revision.txt`, then run from that worktree on the OCI host:

```sh
export DODB_QUERYMIX_ROOT=/bench/zfs/db/query-get-put-mix-20261027
python3 dodb/docs/experiments/results/query-get-put-mix-20261027/run_pipeline.py build
python3 dodb/docs/experiments/results/query-get-put-mix-20261027/run_pipeline.py measure
```

The build runs the schedule test and compiles the optimized service workload
with the pinned Rust 1.97.1, ARM64 `+crc`, and RocksDB 11.8.1 cache. The
measurement first runs one eight-second preflight per backend. If both pass,
it runs three 300-second repetitions per backend in alternating order, with
ten-second warmup and real synchronous WAL. B-link uses the recorded segmented
immutable-base background checkpoint configuration; RocksDB uses the
`minimal-native` DODB-contract adapter. Both use sixteen dedicated clients,
10,000 seeded rows, 512-byte values, and seed `20261013`.

The runner records the exact command, source and binary hashes, full request
latencies, per-client fixed-input fingerprints, external resource samples,
final/reopen validation, and checkpoint coverage. It losslessly archives each
completed database and checks each archived file's size and SHA256 against the
original database manifest before reclaiming only that run's database
directory. Archive work runs between cells. Failed and incomplete attempts
remain in place and are never overwritten.
