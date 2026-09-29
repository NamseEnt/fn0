# Deferred Committed Overlay Design Analysis

This is a feasibility design, not a production format or implementation. Production WAL v3 remains unchanged.

## Candidate published state

Use an immutable `Arc<PublishedView>` containing a pinned immutable B-link base generation and an ordered immutable list of `Arc<OverlaySegment>`. A durable WAL group is first admitted through a transient mutable group overlay. After sync succeeds, freeze successful mutations into a sorted immutable segment and atomically publish a new view. Existing readers retain their prior `Arc<PublishedView>` and therefore continue to observe the old base and segment list.

Each segment contains sorted encoded keys and the newest successful state for each key in that group: value or tombstone plus revision/commit sequence. Duplicate key mutations within the group must still be evaluated sequentially during admission so later conditions observe earlier successful transactions. Only the final state per key is needed in the published segment because intermediate values are not independently committed states visible outside the serialized group; each transaction still has its own commit marker and response outcome.

Lookup checks the transient group overlay during admission, then published segments newest-to-oldest, then B-link base. A tombstone is a present logical version whose value is absent; it must mask both older segments and base. Query/Scan need a sorted multi-way merge with newest-version-wins and tombstone filtering. The prototype's all-entry BTreeMap merge is only a correctness/CPU sketch and is too expensive for Scan.

## Transaction ordering and atomicity

Transactions in one `apply_transaction_group()` remain serialized in request order. For each transaction, evaluate all conditions against the currently admitted state, stage all mutations, and make the transaction's staged mutations visible to later requests only after that transaction is accepted. A failed transaction contributes no state to the transient overlay. A multi-key successful transaction is all-or-nothing in the logical segment and retains one transaction commit marker. WAL durability can share one fsync while preserving independent transaction markers.

Published views change only after the group sync succeeds. If sync outcome is unknown, do not publish a possibly non-durable segment or return a definitive success; retain WAL evidence and use the existing unknown-outcome/recovery contract. Retry identity/idempotence must be specified before a logical WAL lands.

## WAL v4 record candidate

Candidate records:

```text
LogicalPut { encoded_key, value, committed_revision }
LogicalDelete { encoded_key, committed_revision }
Commit { transaction_id_or_batch_id, commit_sequence, record_count, digest }
```

Only already-accepted committed mutations enter the WAL. Conditions are not rerun during recovery. Each transaction keeps an independent commit marker and digest boundary; the physical write/sync can still batch many transaction records. Recovery accepts complete committed transactions and ignores/truncates or quarantines incomplete/torn transaction tails according to an explicit corruption policy.

## Revision and Commit Sequence Semantics

Current revisions are `Revision::from(commit_lsn)`, and the physical WAL page-record count affects commit-LSN assignment. A logical record count changes that numbering unless the format deliberately preserves compatibility.

| Option | Consequence |
|---|---|
| 1. Preserve exact current numeric LSN behavior | Requires assigning logical records/commit markers numbers that emulate current physical-record allocation, including cases where PageImage/PageDelta counts differ. This preserves public numeric revisions but couples the new logical format to legacy physical layout rules and is difficult to keep stable. |
| 2. Preserve only monotonic ordering | Existing APIs expose numeric `Revision`; exact values may change across format/version boundaries. Recovery must retain the logical order and old WAL migration must define a mapping. Clients that persist numeric revisions may observe a compatibility break. |
| 3. Separate logical commit sequence from physical WAL position | Assign one monotonic logical commit sequence to each accepted transaction; physical WAL byte/record offsets are independent. Revisions remain tied to logical commit sequence, while recovery/checkpoints track both logical sequence and physical WAL position. This is cleanest for future compaction and replay. |

Recommendation: **Option 3**, with an explicit format/version boundary and migration mapping before any implementation. It preserves the important monotonic transaction ordering and `RevisionEquals` behavior within a database, avoids making user revisions depend on physical record encoding, and gives checkpoint/recovery separate watermarks. It does not preserve exact legacy numeric revisions automatically; migration must either retain the existing numeric space as the initial logical sequence and continue above its maximum, or document an explicit revision compatibility rule. Do not choose this silently in Phase I.

## Recovery and materialization

Checkpoint state needs a base generation and logical commit watermark. A safe conceptual sequence is:

1. Keep the current checkpoint base and all logical WAL after its watermark.
2. Apply retained committed mutations to a new B-link generation.
3. Write dirty physical pages and data-sync them.
4. Write and sync a checkpoint/superblock that names the new generation and logical watermark.
5. Publish the new base generation while atomically retiring only overlay segments covered by that watermark.
6. Truncate/reset the WAL prefix only after the new checkpoint is durable and recoverable.

Crash before step 4 uses the old base plus retained committed logical WAL. Crash after step 4 recovers the new base and replays only later logical commits. A crash during page materialization cannot lose a commit because the old checkpoint and WAL remain authoritative. Recovery must reject torn/checksum-invalid records and verify commit count/digest before applying each transaction.

## Disk full, I/O failure, and structural limits

After response, logical WAL plus the published overlay is the committed source of truth. ENOSPC, temporary I/O errors, allocation failures, or page growth failures in the materializer pause/retry materialization and must not roll back committed transactions. WAL truncation is blocked until the corresponding checkpoint watermark is durable. Admission must apply backpressure before WAL/overlay capacity is exhausted; otherwise a durable logical log can grow without bound.

Moving splits, root changes, overflow allocation, and physical fit checks out of the response path is safe only if logical admission enforces limits that guarantee every accepted key/value is representable eventually. Validate encoded key and value limits, maximum document size, page/overflow format limits, and arithmetic bounds at admission. Materialization must have a recoverable allocation strategy for split/root/overflow growth. If the current configured maximum can exceed representable storage limits, the deferred model cannot accept those values without changing the public error boundary.

## Read amplification and bounds

The read view is snapshot-stable, but segment count cannot be unbounded. Prototype point GET fallback is near H1 at one or two segments; at four segments it is 1.51–1.59x and at eight it is 2.16–2.18x. Prototype Query is 3.19–3.40x at 1–16 segments and 4.62x at 32. Prototype Scan is 49.7x at one segment and 1,457x at 32 due to over-fetching base rows and materializing a full merge map.

Candidate policies to compare later: trigger materialization on segment count, overlay byte/key count, or oldest-segment age; merge multiple overlay segments into a compact immutable run; or materialize the oldest segment into a new B-link generation. A bounded read path likely needs a hard segment/byte ceiling plus a background merge/materialization path that can pause safely. A two-segment target is a reasonable prototype constraint, not a final policy.

## Open compatibility and operational requirements

- Preserve `Get`, `Query`, and `Scan` order, tombstone, exclusive cursor, and pinned-generation behavior.
- Preserve per-transaction atomicity and commit order, including failed transactions and later same-group condition visibility.
- Define logical commit sequence, public revision migration, and physical WAL offsets independently.
- Preserve corruption detection and define whether damaged logical WAL blocks recovery or permits only a verified prefix.
- Preserve checkpoint/reset ordering and retain all WAL needed for old pinned views or recovery.
- Define the result of ambiguous fsync outcomes and retry identity before responding.
- Bound overlay memory and WAL growth with backpressure and resumable materialization.

No WAL v4 record or production storage code was implemented in Phase I.
