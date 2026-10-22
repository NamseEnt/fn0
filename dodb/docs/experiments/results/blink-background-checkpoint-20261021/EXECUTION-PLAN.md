# Concurrent physical checkpoint implementation

## Required behavior

An explicitly selected segmented background checkpoint must allow acknowledged writes on the same shard while its fixed page images are being written and synchronized. Legacy layout and disabled checkpoint defaults remain available. Success requires recovery of every acknowledged transaction, preservation of newer roots/revisions/read generations, bounded task and retention ownership, terminal failure propagation, and shutdown that drains the worker.

## Implementation sequence

1. Split segmented checkpoints into snapshot preparation, independent page I/O, and coordinator completion. Move dirty image ownership instead of copying the tree. Rotate at the fixed committed target before admitting subsequent groups. Give the worker only immutable images, fixed metadata, and a positional file handle.
2. Publish checkpoint metadata and reclaim covered closed files from the coordinator using the latest manifest. Concurrent rotations therefore cannot be overwritten by a stale worker manifest. Completion advances checkpoint bookkeeping without replacing live tree state. Synchronous segmented mode uses the identical snapshot operation and waits immediately.
3. Add explicit sync/background execution selection to service configuration and both workload binaries. Use one process-wide checkpoint permit, one task per shard, coalesced threshold requests, and 512 MiB retained-WAL/snapshot budgets. Include capacity waits in request latency; reject a group that cannot fit. Background completion must also be polled while idle, and shutdown must join it.
4. Add deterministic overlap, newer-state preservation, concurrent rotation, capacity/failure, and shutdown regressions. Exercise worker and manifest fault boundaries and process termination with post-target acknowledged writes. Run the existing storage/server contracts and format/recovery model before performance execution.
5. Build and validate the exact committed source on the approved OCI host and ZFS dataset. Run interleaved 30-second A/rw20 and E/width-16 cells, three repetitions, for minimal-native RocksDB, legacy sync 128 MiB, segmented sync 128 MiB, segmented background 32 MiB, and segmented background 128 MiB. Preserve commands, binary/source hashes, resources, full latency counts, input-prefix fingerprints, errors, final/reopen validation, actual overlap, and repeated checkpoints. Never replace failed evidence.

## Acceptance and interpretation

Background correctness requires barrier-controlled successful writes before worker release, including overwrite/delete/recreate and tree growth, with unchanged live state after completion and exact reopen validation. Retained files above the fixed target must survive concurrent rotation and checkpoint publication. No required WAL can be deleted before durable checkpoint publication. Failure must stop subsequent writes and resolve queued clients and shutdown.

Performance requires zero request/checkpoint errors, complete uncapped latency samples, matching fixed input prefixes, at least three measurement checkpoints per checkpoint cell, positive write overlap in background cells, zero remaining tasks after drain, RSS below 3 GiB, database size below 4 GiB, host available memory above 1500 MiB, and filesystem free space above 10 GiB. Report synchronous/background ratios separately from RocksDB comparisons. A 300-second follow-up is permitted only if background 128 MiB improves median throughput by at least 10% versus segmented sync 128 MiB in both workloads without worse median p99 or any correctness/resource failure. Short runs do not establish equal-memory or steady-state performance.

## Evidence ownership

Worktree: `/Users/namse/fn0-dodb-background-checkpoint`, branch `experiment/blink-background-checkpoint-20261021`, starting source `72110449920b08cc10c6ebe67df6990e4705b315`. Existing worktrees and historical evidence remain intact. Repository artifacts use English; user reports use Korean. Commit and push only task-owned files without co-author trailers.
