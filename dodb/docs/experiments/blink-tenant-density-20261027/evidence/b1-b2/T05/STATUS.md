# T05 source retention and safe collection status

Status: accepted for the T05 source-manager and GC foundation. Implementation source commit: `5d7d267ab`, pushed to `experiment/blink-tenant-density-20261027`.

The feature-gated source manager binds root pins to one store identity, retains durable-manifest and runtime roots, checks source header identity on every open, charges open handles and root registrations, and enforces configured handle/root limits. GC snapshots the bounded root list, streams index dependencies into disk-backed mark files, blocks new roots and source opens during marking/sweep, preserves uncertain/unlisted files, and unlinks only validated candidates that no root or active handle can reach. Directory metadata is synced after unlink.

WAL delta ancestry can extend beyond the immediate base locator. If an indexed delta names a WAL base, GC conservatively retains all WAL segment candidates until a checkpoint root removes that dependency. This favors safe retention over exact per-segment reclamation and may retain more disk than necessary.

Verification:

- Full serial feature storage library suite: 271 passed, 0 failed, 4 ignored.
- Focused source-manager tests: 8 passed.
- Feature/default storage library checks: passed.
- Targeted rustfmt and diff whitespace checks: passed.
- Strict feature Clippy exits 101 with 12 existing findings in `blink/mod.rs` and `wal.rs`; no T05 finding remains.

The source manager is available to the experimental storage path but is not yet connected to service startup, checkpoint scheduling, or a directory candidate enumerator. Existing resident segmented-WAL reclamation remains unchanged; the resident route rejects the paged format marker. The focused tests close and reopen the manifest and source handles, but do not claim SIGKILL crash-matrix evidence. No service, capacity, performance, OCI, or production claim follows from T05.

See `receipt.json` for exact checks, source hashes, acceptance mapping, and remaining integration work. Next task: T06 fallible page access and writer interfaces.
