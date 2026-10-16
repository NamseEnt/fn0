# Required semantics and incidental wrapper costs

## Objective and boundaries

Explain the historical raw-RocksDB versus DodbService gap without attributing
unmatched functionality to pure wrapping overhead. The historical raw harness
uses RocksDB DB plus synced atomic WriteBatch, without conditional read-set
locking, revisions, or missing-state revision tracking. The service uses
TransactionDB and those dodb contracts. Preserve WAL, sync, conditions,
protected keys, transaction widths, revision semantics, budgets, and workload.

Add a research-only synchronous tenant handle over the same TenantDatabase
implementation. It retains dodb encoding, conditions, key locks, revision
allocator, transactions, query snapshots, and response budgets. It removes
service tenant lookup/lifecycle and per-request spawn_blocking. It is an
engine-core control, not a fully unwrapped C++ or maximum-RocksDB baseline.
Known tenants are preopened outside the measured window. The driver closes
these handles by dropping the drained service before reopening.

Dedicated-client mode runs each client workflow on one blocking worker with
Handle::block_on. Direct engine requests execute synchronously there, after
an eight-slot semaphore admission. A service control uses the identical
client executor and eight slots. Existing async clients remain the default.
The direct core cannot run with async clients or research metrics enabled.

## Candidate improvements

1. Skip read research contexts and request timing when metrics are disabled.
2. Validate successful response size using the protocol's encoding rules
   without cloning payloads or creating a serialized frame for the size check.

Keep Serialized and Shared 8 defaults. Measure default Serialized and opt-in
ReservedRanges separately. Preserve real transport encoding; only replace
its redundant invocation for service budget enforcement.

## Matrix declared before execution

Use four conditions: before-service/async, after-service/async,
after-service/dedicated, and after-core/dedicated. Run both revision modes
on RW95, RW80, RW50, RW20, and MT-MIX, three independent repetitions: 120
primary metrics-OFF runs. Rotate condition order and run serially, with fresh
DBs, seed 20261013, 16 clients, 10,000 seed rows/tenant, 512-byte values, one
second warmup and five seconds admission plus drain. Compare optimization
only within async-client service pairs; compare engine-core overhead only
within dedicated-client pairs. Do not pool the two executors or allocators.

Use the authorized OCI A1 ARM64 host and ZFS /bench/zfs/db, Rust 1.97.1
release +crc, pinned RocksDB 11.8.1 archive, clean committed source, and
immutable before/after binaries. Preserve per-run commands, hashes, exit
status, elapsed time, CPU/RSS/completion, exact latency sample counts,
expected-state replay/reopen checks, category reachability, and matched
schedule/first-100 input fingerprints. Preserve any rejected attempts.
Archive each newly generated validated DB with per-file content hashes,
verify archive contents, then remove only its recoverable working directory
to bound ZFS space growth. Keep the archive on OCI and capture its hash.

Report medians, ranges, and paired repetition ratios for successful writes,
API rate, ordinary/preparatory Get, Query16, write and workflow tails, CPU,
and RSS, separately for every MT tenant. Three short repetitions remain
preliminary; default promotion and maximum-native-engine claims are excluded.
The completion audit must also locate actual consumers for conditional,
missing-revision, atomic-transaction and budget requirements, and distinguish
required behavior from incidental implementation choices.

## Verification probes

Before the primary matrix, run eight metrics-OFF smoke cases and four
metrics-ON after-service smoke cases (both allocators and client executors).
The latter check retained instrumentation counts. Run a separate response
size probe comparing the canonical serializer's lengths/errors with the
counting writer across every response kind, exact/undersized frame budgets,
field limits, 64 MiB value boundaries, binary keys, and document lists. Its
CPU timing compares clone-and-encode against counting for 512 B, 64 KiB and
1 MiB values; it does not measure service or I/O latency. No unit/integration
suite is added by these research executables.
