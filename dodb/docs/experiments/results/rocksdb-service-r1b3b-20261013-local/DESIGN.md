# R1b-3b WAL-sync-boundary recovery design

## Native path

`LocalTenantService::execute(Request::Transact)` opens the tenant's native
`BTreeStore<ProductionFile, ProductionFile>` and submits the transaction to
`AsyncShard`. `Wal::finish_group_append` writes the complete commit group,
hits `before_wal_sync`, and then calls `self.file.sync_data()`. The service
returns the committed `Response::Transact` only after that path completes.
The test will use the existing `FaultInjectorFactory` and the
`before_wal_sync` point on this production service path.

The target file is the tenant shard's native `.wal` file. Database open and
the seed request run with a disarmed injector. The test arms a shared atomic
flag only after it has observed a successful seed response. The injector
reports its hit and blocks before returning from `FaultInjector::hit`, so the
existing `sync_data()` has not started and the target response cannot have
returned.

## RocksDB path

`RocksDbService::execute(Request::Transact)` writes through
`dodb_rocksdb_commit` into `TransactionDB::Commit()` with the transaction's
`WriteOptions.sync = true`. In pinned RocksDB 11.8.1, the synchronous write
calls `DBImpl::SyncWAL()`, which calls `WritableFile::Sync()` on the active
WAL file. The test-only `EnvWrapper` will wrap writable files whose basename
is a numeric RocksDB log filename ending in `.log`; the wrapper reports and
blocks in `Sync()` immediately before delegating to the existing
`WritableFile::Sync()`. Other file types and sync methods are forwarded
unchanged. The wrapper is installed only when the child explicitly enables
the test hook, and its atomic arm flag remains false through open and seed.

## Parent handoff and oracle

The child opens a real file-backed service, executes the seed, verifies the
success response, arms the selected backend hook, then executes the width-four
conditional target transaction. The actual storage hook writes and flushes a
backend-specific reached message to child stdout and blocks. The parent reads
that message, verifies the child remains alive, sends SIGKILL to that exact
child PID, reaps it, and requires signal 9 before reopening the same path.

The parent derives two complete expected document maps from the seed and
target request definitions: the seed-only map and the map after every target
mutation. Reopen must match one whole map. Per-key mixtures, missing or extra
scan rows, inconsistent revision/tombstone metadata, or inability to commit a
new normal transaction after reopen fail the case. Revision numbers are
checked for within-state logical consistency and are not compared between
backends.

This point is before the sync call begins. It does not test interruption
during a sync syscall, power loss, page-cache loss, or storage loss.
