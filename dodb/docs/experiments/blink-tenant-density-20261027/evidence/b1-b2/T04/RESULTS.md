# T04 durable paged manifest and bounded recovery

Status: accepted for T04. Implementation source commit: `72ce3814f` (`Stream bounded paged WAL recovery`), pushed to `experiment/blink-tenant-density-20261027`.

The paged manifest validates a durable version-index root, source identity, recovery boundary, and page high-water metadata. A complete checkpoint pack is validated by the shared pack parser and streamed into the persistent index. Post-checkpoint WAL replay uses a validation pass followed by a commit-gated streaming pass. The scanner retains bounded batch metadata, rereads committed frames one at a time, validates the committed digest and exact delta base, and passes reconstructed page versions to the index sink. Recovery staging is rolled back when replay fails.

The resident/default route rejects the experimental marker without mutation. The rejection test compares lengths and BLAKE3 hashes for the marker, current-pointer sentinel, and existing database, and verifies an absent database remains absent.

## Verification

- Feature storage library tests, serial: 263 passed, 0 failed, 4 ignored.
- Feature storage library check: passed.
- Default storage library check: passed.
- Rust formatting and tracked diff whitespace checks: passed.
- Strict feature Clippy: exit 101 with 12 existing findings in `blink/mod.rs` and `wal.rs`; the final run has no new T04 finding. The initial run also reported a newly introduced redundant wildcard pattern, fixed before the final run. Both outputs are preserved.

The full test suite covers the actual pack-to-index and checkpoint-plus-WAL delta path, source-header identity rejection, incomplete commit non-delivery, and a 2,048-page single commit through the counting sink. This is storage-library correctness evidence only. The service route remains inactive, and no tenant-capacity, performance, OCI, or production claim is made. T17 remains unstarted pending a separately frozen matrix.

See `receipt.json` for exact commands, hashes, acceptance mapping, and limits. Next task: T05 root reachability, source leases, and safe GC.
