# Checkpoint pause and reopen cost reduction

## Objective

Reduce the long write pauses and reopen cost of the immutable-base B-link experiment. The decision is whether B-link offers a meaningful advantage over the DODB-contract RocksDB service under the same workload and real durability. Improvements against B-link alone are intermediate evidence.

## Starting point

Start from `e13910390` (measured implementation `400ad0ac6`). Keep that checkout, its rejected attempts, and accepted evidence unchanged. The selected previous path is coalesced immutable bases with background checkpoints, 128 MiB WAL threshold and 32 MiB segments. Its sustained E throughput was 1,487.97 batches/s, p99.9 2,879.390 ms, worst maximum 3,616.128 ms, and reopen plus state oracle 7,093.500 ms. Those historical values are context, not new paired controls.

## Implementation

1. Load the validated checkpoint/WAL recovery images directly into the existing in-memory tree. Preserve manifest-selected metadata, superblock identity checks, complete allocated-page coverage, normal page decoding, invariant checks, revision allocation, and post-checkpoint dirty tracking. Do not rewrite and reread the disposable primary cache on immutable-base reopen. Ordinary segmented and legacy recovery stay on their existing paths.
2. Retain immutable decoded page references for full checkpoint snapshots and perform missing page-image encoding in the checkpoint worker. Dirty images and the fixed target remain authoritative; later mutations cannot alter captured pages. Preserve snapshot-memory admission and terminal error behavior.
3. Bound the amount written between immutable-pack synchronization calls. Intermediate syncs affect only an unpublished new file; the final pack checksum, file sync, directory sync, manifest publication, and reclamation ordering remain intact. Initially evaluate an 8 MiB write quantum. Add fault coverage at intermediate sync boundaries.
4. Add separate opening and final-state-oracle timings if the current drivers combine them, while retaining the combined metric and complete latency accounting.

## Correctness gates

Run focused storage/server recovery and checkpoint tests, including an incomplete transaction followed by recovery, an acknowledged write, checkpoint, and another recovery. Cover background writes after the target, page growth/reuse, corrupted or missing required packs, a damaged disposable cache, and failure/SIGKILL before and after intermediate syncs and manifest publication. Run applicable full storage/server suites and formatting checks. Do not run performance cells before these gates pass.

## Measurement

Use only `opc@217.142.246.204`, the authorized SSH identity and normal host-key checking, ZFS `/bench/zfs/db`, Rust 1.97.1/AArch64 `+crc`, pinned RocksDB 11.8.1, clean Git checkouts and hashed binaries. Preserve conditions, revisions, admission, group sizes, synchronous WAL, seed 20261013 and client scheduling. Reuse existing RW80 (16 callers) and E (64 callers, width 16) drivers and validators.

Begin with one short correctness/resource preflight for previous B-link, candidate B-link, and DODB-contract RocksDB. Then run three rotated 60-second repetitions after ten-second warmup for both workloads and all three paths (18 cells, about 21 minutes plus seeding/validation). Keep the 128 MiB trigger, 512 MiB additional recovery budget and snapshot bound, 3 GiB RSS and 4 GiB directory guards. Resource equality means identical declared limits, not proven equal complete-engine memory use.

Record source/binary identity, fixed input prefixes, every latency count, final/reopen values and revisions, p50/p95/p99/p99.9/max, opening/oracle time, WAL/checkpoint/process-requested bytes, checkpoint phases, full image count, external RSS/disk samples and completion/drain time. Preserve rejected artifacts and do not rerun passed cells for reducer repairs.

Proceed to three 300-second repetitions of candidate B-link and RocksDB only if candidate E throughput improves at least 5% over the paired previous B-link, E p99.9 improves at least 25%, both workload maxima do not regress more than 10%, combined reopen/oracle improves at least 25%, and zero functional/resource/error gates fail. Each sustained B-link cell must finish at least ten checkpoints; extend only insufficient checkpoint coverage to 900 seconds. Otherwise retain the failed gates and diagnose the specific remaining cost before any expansion.

## Decision and delivery

Report old-versus-new changes separately from RocksDB comparisons. A meaningful sustained B-link advantage requires at least 10% more E successful batches/s, no greater than 3% RW80 throughput loss, and no material tail/reopen regression against RocksDB under the declared limits. A failure leaves the optimization experimental and defaults unchanged. Commit and push task-owned implementation, validation and evidence; preserve unrelated work.
