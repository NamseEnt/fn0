# Transaction semantics model

Correctness gate and current dodb v1 coverage: [FINAL_CORRECTNESS.md](FINAL_CORRECTNESS.md).
Contract-to-code mapping and remaining correspondence gaps: [CORRESPONDENCE.md](CORRESPONDENCE.md).

This directory contains the eight-model correctness suite for dodb v1. The
suite covers transaction semantics and groups, coordinator collection and
segmentation, WAL durability, publication visibility, checkpoint reset, and
an end-to-end request pipeline. The models use bounded finite abstractions;
they do not prove the Rust implementation or hardware persistence behavior.

## Toolchain

The `dodb/` package pins `@informalsystems/quint` to `0.32.0` in
`dodb/package.json` and `dodb/package-lock.json`. Install the local dependency
from the monorepo root with:

```sh
cd dodb
npm ci
```

The verification scripts use Apalache `0.56.1`, which Quint downloads when it
is not already available. TLC uses the JDK installed on the host through the
Quint CLI. The checked environment used JDK 21.0.9.

## Model

`common.qnt` defines the finite logical vocabulary:

- `Key`: `KeyA` and `KeyB` are used by the initial model.
- `Value`: two finite values.
- `LogicalState`: `Present((value, revision))` or `Missing(revision)`.
- `Condition`: `RevisionEquals`, `Exists`, and `NotExists`.
- `Mutation`: `Put` and `Delete`.

`transaction_semantics.qnt` has one active transaction and permits at most two
setup commits before it. Setup commits create the states needed for the ABA
trace. The candidate transaction moves through `Validating`, `Validated`,
`Prepared`, and one of `Committed`, `Conflict`, `ConditionSatisfied`, or
`InvalidRequest`.

`transaction_group.qnt` models ordered logical processing of one fixed request
sequence at a time. It keeps the group result private until `PublishGroup` and
checks each processed prefix against a separate serial reference evaluator.
The four bounded sequences cover accepted/conflict/accepted, staged-state
condition-only success, insert-if-absent conflict, and invalid-then-accepted.
The invalid case uses an empty transaction as a representative invalid request
to check that one invalid request does not block later requests in the group;
it does not formalize all Rust input-validation cases, such as duplicate
conditions or mutations and oversized input.

`coordinator_segments.qnt` models FIFO coordinator segmentation for one fixed
request sequence. Mutation-like requests, including `ConditionOnly`, are
consumed as maximal contiguous segments; each barrier is processed separately
and records the current logical state. Independent serial and segmentation
references check each processed prefix. The model does not reimplement
transaction semantics.

`coordinator_collection.qnt` verifies physical group formation for one fixed
FIFO-admitted request stream. It checks request conservation and FIFO order,
pending-request handoff, request-count and byte bounds, and the oversized-first
request exception. Timing is not modeled; sealing a group early is an
nondeterministic choice. Transaction and barrier semantics, WAL, and fsync are
the responsibility of the other models or are outside this model's scope.

Validation reads the immutable `baseState`. Mutation preparation computes a
private `stagedState` with one commit LSN for every changed key. Only
`commitPrepared` changes `committedState` and advances `nextLsn`. A
condition-only success stops before preparation and returns `NoCommit`.

The model uses a finite catalog of transactions rather than modeling B+Tree
pages. The catalog includes empty, condition-only, one-key, two-key, delete,
conflicting, and stale-missing-revision cases. This is intentional: the model
checks logical state and transaction ordering, not the Rust page algorithm.

## Named invariants

- `TypeOK`
- `Atomicity`
- `ConflictHasNoEffect`
- `OneRevisionPerCommit`
- `MissingRevisionPreserved`
- `ConditionOnlyHasNoCommit`
- `CommittedStateWellFormed`
- `EmptyRequestInvalid`
- `ValidationBeforeMutation`
- `RevisionConditionSound`
- `AbaStaleRevisionRejected`

The `state_invariants` value combines all of them for simulation and model
checking.

## Commands

```sh
cd dodb
npm run formal:typecheck
npm run formal:test
npm run formal:simulate
npm run formal:verify:tlc
npm run formal:verify:apalache
npm run formal:group:typecheck
npm run formal:group:test
npm run formal:group:simulate
npm run formal:group:verify:tlc
npm run formal:group:verify:apalache
npm run formal:segments:typecheck
npm run formal:segments:test
npm run formal:segments:simulate
npm run formal:segments:verify:tlc
npm run formal:segments:verify:apalache
npm run formal:collection:typecheck
npm run formal:collection:test
npm run formal:collection:simulate
npm run formal:collection:verify:tlc
npm run formal:collection:verify:apalache
```

The initial verification bound is two keys, two setup commits, finite values,
bounded integer revisions/LSNs, and `--max-steps=20`. The simulation uses seed
`0x20260923` and 200 samples.

## Concrete mapping

| dodb concept | Model concept |
| --- | --- |
| `RevisionState::Present` | `Present((value, revision))` |
| `RevisionState::Missing` | `Missing(revision)` |
| `TransactionCondition` | `Condition` |
| `TransactionMutation` | `Mutation` |
| `TransactionRequest` | `Transaction` |
| `Overlay` | `stagedState` and `baseState` |
| transaction commit LSN | `HasCommit(nextLsn)` |
| `TransactionResult { commit_lsn: None }` | `NoCommit` |
| `prepare_transaction` validation | `validateConditionOnly`, `validateMutation`, `rejectConflict` |
| `finish_transaction` logical result | `prepareTransaction` |
| committed publication in this logical model | `commitPrepared` |

WAL records, fsync, page images, superblocks, read views, coordinator groups,
and network responses are intentionally absent. They belong to later models.

## WAL durability model

`wal_durability.qnt` models two logical commits written in fixed WAL frame order:
`APage`, `ACommit`, `BPage1`, `BPage2`, `BCommit`. Each logical commit is
recoverable only when its own COMMIT marker is part of the complete durable
prefix. Both commits share one physical fsync.

Unsynced writes may partially persist before a crash. A failed fsync has an
uncertain persistence outcome, so recovery reads the complete durable prefix
and can preserve an earlier logical commit from a failed physical group. The
physical group is a durability amortization unit, not a logical atomicity
boundary. A successful fsync makes the full frame group durable, even if a
fault occurs before `append_group()` reports success.

The model stops after WAL append success and recovery. B+Tree publication,
client acknowledgment, checkpointing, WAL reset, and page flush are outside
its scope.

```sh
cd dodb
npm run formal:wal:typecheck
npm run formal:wal:test
npm run formal:wal:simulate
npm run formal:wal:verify:tlc
npm run formal:wal:verify:apalache
```

## Publication and read visibility model

`publication_visibility.qnt` checks the publication pipeline for the fixed
logical group `CommitA` and `CommitB`. It covers WAL durability before store
publication, transient per-commit store publication, atomic committed read-view
publication, ordinary read visibility, coordinator response issuance ordering,
crash recovery from WAL authority, and the distinct before/after publish
failure outcomes. Recovery models reopening from durable WAL state; checkpoint,
WAL reset, and data-file flush are outside this model.

The store may transiently publish A before B internally, but ordinary
lock-free reads use the committed read view and do not observe that partial
group in the live process.

A failed or missing response does not imply rollback; once WAL durability has
been established, crash recovery may restore the commit.

The modeled responses are coordinator oneshot response issuance. Network
delivery and client acknowledgment are outside the model.

The production mapping is `WalAppendSuccess` to successful
`wal.append_group(...)`, `WalAppendFailure` to its error path and broken store,
`BeforePublishFailure` to the `before_publish` fault after WAL durability but
before store publication, `PublishA` and `PublishB` to the prepared store
publication loop, `PublishReadView` to `coordinator::publish_read_updates()`,
and the success or error response actions to `queued.response_tx.send(...)`.
`AfterPublishFailure` captures an error after both store publications: the
coordinator can still publish the committed read view, leaving visible A+B
alongside error responses. Ordinary `Get`, `Query`, and `Scan` read the
`CommittedReadView`; they do not observe the store's transient A-only state.

```sh
cd dodb
npm run formal:publication:typecheck
npm run formal:publication:test
npm run formal:publication:simulate
npm run formal:publication:verify:tlc
npm run formal:publication:verify:apalache
```

## Checkpoint and WAL reset model

`checkpoint_reset.qnt` starts with A+B durable in the old WAL and models
checkpointing the latest logical state through data-file durable flush,
alternate checkpoint-superblock durability, the invariant gate, WAL truncation,
WAL reset INIT history boundary, and crash recovery from old, empty, torn-init,
and new WAL states. It checks checkpoint recovery and next-LSN/history
monotonicity across each crash point, including completion failure after reset.

A checkpoint makes the data file authoritative before reclaiming WAL history.

Data survival alone is insufficient: after WAL reset, the recovered next LSN
must not move behind the reclaimed history boundary.

The abstract production sequence is dirty pages and current transaction
superblock writes followed by data sync; alternate checkpoint metadata write and
sync; `check_invariants()`; WAL truncate and sync; new INIT write and sync; then
checkpoint completion. An empty WAL reinitializes from the checkpoint hint,
and a torn reset INIT is repaired and reinitialized from that same hint. A
failure before checkpoint completion leaves an already durable checkpoint in
place. Exact page writes, checksums, superblock encoding, byte-level INIT
prefixes, and B+Tree page layout are outside this model and remain covered by
Rust tests.

```sh
cd dodb
npm run formal:checkpoint:typecheck
npm run formal:checkpoint:test
npm run formal:checkpoint:simulate
npm run formal:checkpoint:verify:tlc
npm run formal:checkpoint:verify:apalache
```

## End-to-end contract-composition model

`end_to_end_pipeline.qnt` composes the contracts checked by the smaller models
into one representative request pipeline. The fixed request stream is
`PutA`, `ConditionOnlyA`, `ConflictingPutB`, `PutB`, `ReadBarrier`, and
`CheckpointBarrier`. Physical groups are selected nondeterministically in FIFO
order with a maximum size of four requests. A physical boundary can split the
contiguous mutation segment at any position before the read barrier.

The serial reference expects the ordered response list `ACommitted`,
`ConditionSatisfied`, `ConflictResult`, `BCommitted`, `ReadBoth`, and
`CheckpointSucceeded`. Its logical states progress from `(A=0, B=0)` to
`(A=1, B=0)` after `PutA` and then to `(A=1, B=2)` after `PutB`.

The staged transaction semantics use logical commit ordinals: `PutA` commits
ordinal 1, the condition-only request succeeds without consuming an ordinal,
the conflicting conditional `PutB` has no effect and consumes no ordinal, and
the final `PutB` commits ordinal 2. These ordinals are model revisions, not raw
WAL LSNs. Each mutation segment uses staged state so later requests see prior
successful mutations in that segment.

Successful segment completion abstracts the contract sequence WAL durable,
committed store published, committed read view published, then responses
issued. Responses remain private until durability and publication complete. The
read barrier observes `A revision 1` and `B revision 2` only through the
committed read view. The checkpoint barrier
abstracts durable checkpoint state followed by safe WAL history reclamation,
preserving the logical history floor and next commit ordinal. Recovery uses the
durable WAL state before reclamation and the checkpoint state after reclamation.

The integration model does not re-prove the internal WAL, publication, or
checkpoint algorithms. It checks that the contracts proven by the smaller
models compose into one consistent request pipeline.

Physical grouping may change the number of WAL syncs, but not the serial
logical result, response order, barrier observation, or recoverable committed
state. `walSyncCount` is a performance-related outcome of those group
boundaries.

This model is not a proof of the Rust implementation itself. Rust/model
correspondence remains a separate validation target.

```sh
cd dodb
npm run formal:e2e:typecheck
npm run formal:e2e:test
npm run formal:e2e:simulate
npm run formal:e2e:verify:tlc
npm run formal:e2e:verify:apalache
```
