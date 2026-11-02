# T04 implementation status

Status: accepted for the T04 card. Source implementation is under review for task-owned commit.

The opt-in paged manifest now records and validates the durable index root, source identity, recovery boundary, and page high-water metadata. Checkpoint index construction validates the complete base pack with the shared parser and streams its pages into the persistent index. Post-checkpoint WAL recovery validates the source initialization identity, committed prefix, transaction digests, page images, delta bases, and exact locations through a two-pass streaming scanner. The consumer applies each committed version to copy-on-write index roots without retaining the complete page history or batch history in memory. Failed recovery rolls staged index state back.

Resident/default-route rejection now snapshots lengths and BLAKE3 hashes for the marker, current-pointer sentinel, and existing database. It verifies that rejected opens do not mutate those files and do not create an absent database.

Verification:

- Full serial storage library suite with `blink-demand-paging`: 263 passed, 0 failed, 4 ignored.
- Default and feature-enabled storage library `cargo check`: passed.
- Targeted `rustfmt --edition 2024 --check` and `git diff --check`: passed.
- Strict feature Clippy exits 101 with 12 findings in existing `blink/mod.rs` and `wal.rs` code. The first attempt also found one new redundant wildcard pattern; that was fixed and the final run reports no new finding in the T04 changes.
- The failed first Clippy output is retained alongside the final output.

Captured command output is retained in `logs/`. No service route, tenant-capacity, performance, or OCI claim follows from this card. The paged path remains opt-in. T17's comparative matrix has not started.

Next task: T05 root reachability, source leases, and safe GC.
