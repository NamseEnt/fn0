# Accelerated soak and crash-recovery validation

dodb-soak is an explicit validation executable. It drives the production
client and server path over QUIC and keeps an in-memory logical reference model
beside the workload. It is intentionally a high-intensity test harness rather
than a general benchmark.

The accelerated profile compresses operation count, state transitions,
optimistic conflicts, checkpoint cycles, connection and stream churn, abrupt
restarts, and observability into roughly 30-45 minutes. A short run is not
mathematically equivalent to a multi-day soak: it samples a different part of
the failure space and cannot prove production safety. It is useful because it
reaches many more transitions and recovery boundaries per wall-clock hour.

## Profiles and commands

Build or run the dedicated workspace executable with commands such as:

    cargo run -p dodb-soak -- --profile smoke --seed 1
    cargo run -p dodb-soak -- --profile accelerated --seed 1
    cargo run -p dodb-soak -- --phase growth --duration 30s --seed 1
    cargo run -p dodb-soak -- --phase crash --duration 10m --seed 7

The smoke profile is a few minutes and is suitable for manual or CI sanity
checks. The accelerated profile has approximately eight minutes of bounded
growth-then-churn load, six minutes of bounded-state overwrite/delete/reinsert load, eight
minutes of high contention, and ten minutes of crash/restart cycles, followed
by quiescent verification. --duration overrides each selected phase, so a
quick local crash check can use --phase crash --duration 8s. --tenants,
--concurrency, --checkpoint-ms, --output, and --data-dir make resource and
phase sizing explicit. Connection churn can be disabled with
--no-connection-churn when isolating another failure.

The generator is seeded and does not use thread-local entropy after startup.
Recent and failing operation artifacts record the run seed, phase, phase-local
operation index, global operation index, workload configuration, generated
operation, elapsed time, tenant, and result in a bounded ring. Operation
generation is deterministic for a recorded phase seed/index; wall-clock
scheduling and the total number of operations completed by a timed phase are
not deterministic. Replaying a failure therefore uses the recorded phase and
index rather than depending on a previous run producing the same phase length.
Task scheduling can still change the order in which concurrent requests commit.

For a limited validation window, three independent short seeds are often more
informative than one partially completed long run:

    for seed in 1 2 3; do
      cargo run -p dodb-soak -- --profile smoke --phase contention --duration 10m --seed "$seed"
    done

## Workload and model

The default mix covers Get, Put, Delete, Query, Scan, concurrent independent
Gets, and Transact with weights approximately 35/20/10/10/5/10/10. The generator uses an 80%
hot-key and 20% wide-key distribution, multiple tenants, fixed-size hot values,
and finite large-value probes. This keeps hot revision churn and contention
fast while still exercising inline values, overflow pages, response budgeting,
and QUIC framing.

The growth phase fills an explicit target of 1,024 ordinary keys per tenant
over its first 16,384 generated operations, then keeps mutating that same finite
keyspace. The bounded phase uses 256 ordinary keys per tenant. Contention uses
64 ordinary keys per tenant with a 95% hot-key distribution and a 40%
transaction weight, so collision probability stays high without creating new
keys. Crash uses 128 ordinary keys per tenant and reuses its ABA and large-value
probe keys on every restart cycle.

Every phase also has finite transition and large-value probe sets. The growth
phase therefore exercises leaf/internal/root growth before spending the rest of
its budget on overwrite, delete/reinsert, value-size transitions,
checkpoint/WAL reclaim, and allocator reuse. Logical key counts and
phase-labelled RSS, virtual-memory, database-byte, FD, stream, connection, and
reference-model retained-value observations are reported together; database
growth alone is not treated as a leak.

The model stores tenant, document key, presence, and opaque revision history.
Values at or below 512 bytes may be retained inline; larger values retain only
their length and SHA-256 digest. Revisions are learned only from successful
dodb responses. Full quiescent checks compare scans, queries, cursors, and
concurrent independent Get results, exact revisions, value lengths, and value
digests.
During concurrent traffic, observations may reflect a legal linearization point
between model updates; generated mutation fingerprints are used to validate
the returned value while the exact quiescent comparison remains authoritative.

Each phase also runs explicit Put/Delete/Put/Delete sequences and stale
RevisionEquals checks for both missing and present revisions. Transaction
workers deliberately build conditions from the current model, so serialized
successes, conflicts, missing-key ABA conflicts, and atomic multi-key results
are distinguished rather than counted as generic errors.

## Checkpoints, crash recovery, and unknown outcomes

The server child has a validation-only administrative path for checkpointing and
storage invariant checks. It checkpoints repeatedly while requests continue,
records WAL bytes before and after through the storage report, and records
checkpoint latency, LSN, reclaim bytes, and invariant results. The parent also
samples DB and WAL sizes directly. This harness does not create, copy, restore,
or emulate an application-level snapshot; dodb's backup contract is the
crash-consistent storage snapshot described in storage-backup.md.

For crash phases the harness starts the server as a separate process, waits for
QUIC readiness, drives load, sends an abrupt process kill, restarts the same
storage directory, waits for normal opening and recovery, reconciles unknown
mutations, and runs a full logical verification. On Unix the child is killed
with the process termination operation used by Tokio; the same child-process
kill path is used on other supported platforms.

A lost mutation response is not treated as a failed mutation. Values contain a
bounded, human-readable soak_operation_id marker. At every quiescent boundary,
the harness stops production, waits for workload and connection-churn tasks,
reads the affected keys, and reconciles the complete overlapping pending set
against the recovered final state. Several single-key mutations may have
committed and overwritten one another; multi-key transactions are explored as
atomic events and a true partial transaction fails the run. Unknown outcomes,
checkpoint failures, crash history, recent operations, server logs, resource
samples, and the last successful verification are included in artifacts.

## Metrics and artifacts

The report contains operation counts and throughput, commit/conflict/overload
counts, transport and application errors, checkpoints, crashes and recoveries,
verification and invariant passes, p50/p95/p99/max latency, connection churn,
RSS, virtual memory, file descriptors, threads, active streams/connections, WAL
size, database size, and the latest reliable mimalloc statistics JSON. On
Linux, /proc supplies RSS, virtual memory, threads, and descriptor counts;
other platforms mark unavailable process metrics rather than fabricating them.

Resource diagnostics ignore a warm-up window and compare later windows. A
sustained late-window slope emits a warning; noisy RSS behavior is reported as
a warning rather than a brittle single-sample failure. Normal phase cleanup is
observed after reconciliation and full verification, after the workload QUIC
connection is closed, and after a bounded settle window. Post-quiescence
active streams must be zero and active connections must return to the idle
baseline; the report records the post-quiescence stream, connection, and FD
samples. Semantic failures such as model mismatches, partial atomic
transactions, recovery failures, checkpoint or invariant failures, and
unreconciled outcomes fail immediately.

Each run writes a timestamped directory below target/dodb-soak by default.
It contains report.json, recent operations, resource samples, per-child
stdout/stderr, per-child event JSONL, TLS test material, and failure.txt when
needed. The operation log is bounded; it is not an unbounded request trace.

## Allocator and limitations

The global mimalloc allocator is installed only in the dodb-soak executable.
The reusable dodb libraries do not select a global allocator. The child records
mimalloc's stable extended JSON statistics when available, alongside process
RSS and committed/reserved values exposed by that API. Exotic allocator FFI is
not added when a statistic is unavailable.

The harness uses a local one-process server and LocalTenantService, not a
replication or cloud-snapshot coordinator. It cannot validate provider-specific
ZFS or block-volume snapshot implementations, and it cannot make concurrency
scheduling deterministic. It should therefore be combined with ordinary crash
fault tests and operational storage-snapshot validation.
