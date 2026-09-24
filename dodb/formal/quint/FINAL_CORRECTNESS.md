# dodb v1 Correctness Summary

## Scope

Original formal/correctness development: `namse/dodb` branch
`formal/quint-correctness` through commit
`fb2e8cbee0ef137b3daa3aa841a66e057b8c8d60`.

Canonical migrated location: `NamseEnt/fn0/dodb`.

This summary records the correctness work against the migrated monorepo tree.
The full correctness gate was rerun on the monorepo integration tree; the
results and runtimes below describe that run. The standalone branch is retained
as provenance and is not the canonical production tree.

Coverage includes transaction semantics, grouped transaction serialization, coordinator physical collection and logical segmentation, WAL durability and recovery, publication/read visibility, checkpoint/WAL reset, end-to-end contract composition, Rust/model correspondence, and deterministic persisted-prefix crash exploration.

Use [`CORRESPONDENCE.md`](CORRESPONDENCE.md) for the contract-to-code map and detailed concrete/model boundary.

## Formal model inventory

The eight Quint models are finite abstractions. TLC and Apalache check invariants only within their stated bounds. Bounds below come from the current `.qnt` models and `dodb/package.json` commands.

| Model | Contract | Finite bound / abstraction | TLC | Apalache | Rust correspondence |
| --- | --- | --- | --- | --- | --- |
| `transaction_semantics` | Atomic conditions and mutations, conflict behavior, missing revisions, condition-only success | Keys A/B; two finite values; fixed transaction catalog; at most two setup commits; max 20 steps | Pass, max 20 | Pass, max 20 | `formal_correspondence` randomized reference test and existing transaction/revision tests |
| `transaction_group` | Ordered staging against earlier accepted writes, per-request outcomes, atomic group publication | Keys A/B; four fixed request sequences, at most three requests each; finite values; max 20 steps | Pass, max 20 | Pass, max 20 | `formal_correspondence`; transaction-group tests in `phase1_btree` and storage |
| `coordinator_segments` | FIFO mutation segmentation and barriers observing preceding mutations | One fixed nine-operation stream; finite A/B versions and barrier observations; max 20 steps | Pass, max 20 | Pass, max 20 | Fixed split/grouped coordinator pipelines and barrier tests |
| `coordinator_collection` | FIFO conservation, pending handoff, request/byte limits, oversized first request | Five fixed requests with sizes 0, 3, 1, 1, 1; request limit 2, byte limit 2; max 20 steps | Pass, max 20 | Pass, max 20 | Coordinator collector unit tests, including zero-byte then oversized request |
| `wal_durability` | Complete logical COMMIT markers govern recovery; physical group may contain multiple logical commits | Two logical commits; five fixed frames; durable frame-prefix positions 0–5; max 25 steps | Pass, max 25 | Pass, max 25 | Every-byte WAL prefix DST and WAL fault/recovery tests |
| `publication_visibility` | Read view sees a whole publication; response and visibility ordering under named failures | Two logical commits; finite publication phases, response states, and failure kinds; max 25 steps | Pass, max 25 | Pass, max 25 | Atomic read-view, before/after publication, and after-WAL-sync correspondence tests |
| `checkpoint_reset` | Checkpoint recovery authority, WAL reset forms, and history/LSN floor | Old/latest data states; checkpoint LSN 0/5; four WAL forms; finite crash origins; max 30 steps | Pass, max 30 | Pass, max 30 | Checkpoint operation/image matrix and exhaustive reset INIT prefix DST |
| `end_to_end_pipeline` | Fixed request order composing grouping, segmentation, responses, read barrier, checkpoint, and recovery | Six fixed requests; maximum physical group size 4; finite revisions and crash states; max 40 steps | Pass, max 40 | Pass, max 40 | Fixed grouped/split pipelines, coordinator barriers, and randomized Rust correspondence |

Every model also has a Rust-backend `npm run formal:*:test` check and a Quint typecheck in the reusable gate. Passing these checks says nothing beyond the model definitions and bounds listed here.

## Concrete Rust evidence

The concrete results below are deterministic correspondence and crash-testing evidence against the current Rust tree.

### Transaction and coordinator corpus

- Fixed correspondence pipelines exercise split physical groups and grouped mutations.
- The randomized reference corpus uses eight deterministic seeds and reports 4,800 groups / 7,844 requests: 395 condition-only successes, 1,571 conflicts, and 1,957 invalid requests.
- The corpus includes crash/reopen and checkpoint/reopen cycles, plus missing revision and ABA checks.
- Coordinator tests cover a barrier following mutations in the same collected physical group, and committed read-view atomicity.
- Publication failures cover `before_publish`, `after_publish`, and `after_wal_sync`; the observed error response does not imply rollback.

### WAL persisted-prefix DST

`durability_dst::every_unsynced_wal_prefix_recovers_only_a_logical_commit_prefix` explores every byte length from the durable INIT through one complete unsynced physical group containing logical commits A and B:

- Baseline WAL: 104 bytes; complete unsynced WAL: 16,864 bytes; tested prefixes: 16,761.
- Recovery counts: 8,380 with no commits, 8,380 with A only, and 1 with A+B.
- No B-only recovery occurred. The first A prefix is 8,484 bytes; the first A+B prefix is 16,864 bytes.
- WAL scan and BTree recovery are both checked at every prefix. Torn final frames reopen safely; recovered revisions match the corresponding commit LSNs. `next_lsn` is monotonic, including complete uncommitted frames, and the initial history floor remains zero.
- Recovered stores are continued with writes at semantic transition prefixes and the complete prefix.

Sync-error cases return an error to the caller while allowing none, partial (A-only), or all (A+B) of the WAL image to persist. Both prefix persistence and `SyncPersistAllThenIo` are exercised. This confirms that an fsync error response alone does not determine the recovered logical outcome.

### Checkpoint and reset DST

- All 26 named checkpoint operation fault points are crossed with available durable/volatile data and WAL images. Duplicate images are deduplicated, yielding 38 candidate combinations in this run.
- Every candidate combination reopens with exact A/B values and revisions, passes invariants, and permits a new commit whose revision exceeds both earlier revisions.
- The matrix exercises volatile checkpoint metadata after the superblock write, both old durable and empty volatile WAL after an unsynced truncate, a volatile complete reset INIT before its fsync, and an error after reset at checkpoint completion.
- All 105 prefixes of the 104-byte reset INIT image reopen against the durable checkpointed data. The checkpointed values and LSN floor survive; a subsequent write advances beyond the previous last revision.

Data-file candidates in this DST are only the last synced durable image or the current complete volatile image. Whole-file byte-prefix truncation is applied to append-only WAL streams, not to data-file overwrites. Existing short-write and repair tests cover selected partial data-page writes separately.

## Guarantees and evidence boundary

Quint results are **model-checked invariants** over bounded abstract models. Rust results are **deterministic correspondence and crash-testing evidence** for the sampled requests, images, and named failure positions.

The Rust implementation itself has not been formally proven.

The tested contracts include:

- A successful mutation transaction is atomic at the logical level.
- A condition-only transaction consumes no logical commit.
- Missing revision history survives delete and prevents a stale ABA `RevisionEquals` from succeeding.
- Accepted transactions in one group observe the staged state of earlier accepted transactions.
- A conflict or invalid request does not poison later requests in the group.
- Barriers run after the preceding mutation segment.
- Ordinary lock-free reads do not observe a partial grouped publication.
- A success response occurs only after WAL durability and publication.
- An error response does not necessarily mean rollback; durable effects may still recover.
- A complete COMMIT marker defines logical WAL recovery.
- A physical WAL group is not itself a logical atomicity boundary; its logical commits recover as a prefix.
- Checkpoint makes data authoritative before reclaiming WAL history.
- WAL reset preserves the history and LSN floor.

## Concurrency DST decision

The current baseline has one ordered coordinator writer and a committed immutable read view. The existing formal/composition and concrete tests cover the publication boundary and sampled barrier order. Exhaustive scheduler DST is therefore not a blocking requirement for closing this baseline correctness phase; no new concurrency model or scheduler framework is included here.

If the architecture adds page-local multiwriter behavior, parallel mutation publication, pipelined WAL/fsync, or lock-free mutable structures, reopen concurrency verification and add an appropriate concurrency model/DST before relying on this baseline.

## Remaining gaps

This result does not cover:

- Arbitrary sector/block reordering inside the data file.
- Arbitrary partial overwrite persistence for data pages.
- Real filesystem, kernel, or controller power-loss semantics.
- Hardware torn-write guarantees.
- All thread scheduler interleavings.
- Every workload or page-layout history.
- Formal proof of the Rust implementation.
- Future B-link experiments or non-main algorithms unless they are revalidated against this gate.

## Change safety matrix

| Future change | Verification to rerun or revisit |
| --- | --- |
| Transaction condition or mutation semantics | `transaction_semantics`, `transaction_group`, Rust correspondence and workspace tests |
| Coordinator batching or segmentation | `coordinator_collection`, `coordinator_segments`, end-to-end model, coordinator correspondence tests |
| WAL encoding or fsync behavior | `wal_durability`, persisted-prefix DST, Rust correspondence; also `checkpoint_reset` if reset contract changes |
| Publication or read-view behavior | `publication_visibility`, atomic-view and publication-fault correspondence tests |
| Checkpoint or WAL reset | `checkpoint_reset`, checkpoint/reset DST, recovery tests |
| Multiwriter or pipelined mutation/WAL architecture | Entire existing suite plus a new concurrency model and scheduler DST |

## Reproducible gate

The canonical monorepo has GitHub Actions workflows. This reusable gate is maintained at `dodb/scripts/verify-correctness.sh`; this migration does not add a new workflow.

```sh
dodb/scripts/verify-correctness.sh fast
dodb/scripts/verify-correctness.sh full
```

`fast` runs formatting and tests for all eight dodb Rust packages, all eight Quint Rust-backend tests, and all eight typechecks. `full` adds formal correspondence and durability DST tests, all eight TLC checks, all eight Apalache checks, and a final monorepo `git diff --check`.

The migrated monorepo integration gate completed successfully: `fast` passed in 132 seconds and `full` passed in 600 seconds. The full run passed correspondence and durability DST, all eight TLC checks, and all eight Apalache checks.
