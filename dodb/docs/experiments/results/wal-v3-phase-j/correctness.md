# J0 Correctness

The retained overlay-view test covers:

- newest-overlay wins for repeated keys;
- tombstones suppress older base and overlay values;
- a later overlay can restore a tombstoned key;
- base hits and misses;
- Query ordering and exclusive sort-key cursor behavior;
- Scan ordering and exclusive document-key cursor behavior;
- bounded four-segment publication and rejection of a fifth segment;
- a reader pinned before publication continues to read the old base-only view.

The range merge advances equal keys in all matching sources and resolves the value before applying the row limit. A tombstone does not consume the visible-row limit. The base-only view dispatches directly to the original H1 read routines.

The J0 focused release test passed on the Mac and OCI A1. The complete `cargo test --workspace --release --no-fail-fast` suite passed on the final J0 source. J0 does not cover logical write visibility, WAL durability, recovery, or materialization.

## J1

The J1 focused test covers ordered same-group visibility through `Exists`, a failed conditional transaction with no state effect, a later successful transaction, same-key final-value collapse into one segment record, segment cap enforcement, and old-reader pin stability across publication. It passed on the OCI A1 as part of the full workspace release suite.

J1 is volatile and test-only. It does not test WAL append/sync failure, recovery, or acknowledgement durability. The full workspace release suite passed at the J1 source commit; the later `cfg(test)` gating of prototype-only helpers changes no test behavior.

## J2

The logical WAL tests cover Put, Delete, multi-key transactions, ordered same-group condition visibility, failed-transaction isolation, `RevisionEquals`, repeated keys, one sync per group, physical frame LSN versus logical revision, tail truncation for Put/Delete/Commit, a committed prefix before an incomplete transaction, CRC and digest rejection, short writes, explicit v3/v4 reader rejection, sync uncertainty, failure after WAL sync but before view publication, reopen, more writes, and a second reopen.

A deterministic randomized differential ran 32 groups of four variable-width transactions against the H1 physical engine. It compared result success/failure ordering, visible values, normalized revision ordering/equality, Query results, and Scan results. Logical revisions are compared by their order/equality relation because v3 frame-derived revisions and v4 transaction-order revisions have different numeric values by design.

The full `cargo test --workspace --release --no-fail-fast` suite passed after the J2 implementation. Existing WAL v3 PageImage/PageDelta, crash, checkpoint, QUIC, and randomized physical-engine tests remain enabled.

## J3

The logical materializer test applies four overlay generations containing repeated 2 KiB values, verifies the pre-checkpoint pinned read snapshot, compares the materialized Scan and point reads, confirms the physical/logical checkpoint watermarks, checks WAL reclaim, writes another eight groups across two materializations, and reopens twice. After the first capacity expansion, the second and third materializations reuse the page bank without extending the file. Values and their original revisions survive materialization.

The storage fault matrix covers a partial scratch-page write, data sync, checkpoint superblock publication, WAL reset, truncate, and reset sync boundaries. `dodb-testkit/tests/phase_j_materialization.rs` uses volatile/durable byte separation so every injected boundary is followed by a simulated process loss. It covers pre-watermark and post-watermark recovery, a retained old WAL, a durably truncated WAL, and a synced INIT carrying distinct physical and logical checkpoint boundaries. StorageFull during a page write and an I/O failure during data sync both reopen from the old base plus retained WAL.

The final full workspace release suite passed at Phase J source `0ec2528a7513e507b2033aba095a937373e754b7`. J4 measured durable throughput and materialization latency on OCI A1. Sustained growth and RocksDB were skipped because the required throughput gates failed.
