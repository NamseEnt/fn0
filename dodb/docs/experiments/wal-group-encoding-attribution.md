# WAL Group Encoding Cost Attribution

## Motivation

The OCI width-1 planned Blink result identified WAL group encoding as the
largest top-level CPU component, at about 12.74 µs per mutation. This
instrumentation-only experiment measures where that time is spent. It does not
change WAL bytes, validation boundaries, or execution order.

## Normalized Baseline

The preceding fixed-duration OCI result reported a median throughput of
20,123.54 mutations/s, about 40,283 mutations, 80,566 page images, and 4,362
logical groups per repetition. Its normalized WAL group encode costs were
approximately 12.74 µs/mutation, 6.37 µs/page image, and 117.6 µs/logical group.
The earlier direct-encoding baseline was approximately 13.23 µs/mutation,
6.61 µs/page image, and 124.3 µs/logical group.

Because the benchmark uses fixed durations, cumulative nanoseconds are
normalized by each repetition's successful mutation, page-image, and logical
group counts.

## Current Encoding Path

The fast `WalLog::encode_group()` path currently performs these steps for each
page image:

1. Check the page LSN against its commit LSN.
2. Allocate and materialize a payload `Vec` containing the page ID and 4096-byte
   image.
3. Validate the page image. Experimental Blink normal pages are decoded and
   checked again here.
4. Copy the payload into the commit digest input.
5. Calculate the payload CRC32C and header CRC32C, allocate and materialize a
   frame `Vec`, then append that frame to the group buffer.

For each logical commit it also calculates the digest CRC32C, encodes and
materializes the commit frame, and appends that frame to the group buffer.

## Instrumentation

`WalEncodeAttribution` accumulates exclusive durations locally for one group
encoding. The categories do not overlap: payload materialization ends before
validation; frame CRC timers do not include frame materialization; and frame
append timers cover only appending a completed frame to the group buffer. The
existing `group_encode_nanos` remains the enclosing encode duration, including
timer and bookkeeping overhead. Counts identify page frames, commit frames,
and page validations.

The instrumentation is used by the fast encoding path. The fault-injectable
path retains its existing encoding calls and semantics.

## Correctness

The focused fast-path versus fault-injectable-path test passed. It compares the
encoded WAL bytes and `WalAppendReport`s, checks `next_lsn` and
`next_batch_id`, and verifies attribution frame and validation counts.

All requested local gates passed:

- `cargo fmt --all -- --check`
- `cargo test -p dodb-core`
- `cargo test -p dodb-storage` (97 unit tests; recovery child 8/8)
- `cargo test -p dodb-storage --bin phase0-bench` (10 tests)
- `cargo test --workspace` (including recovery child 8/8)
- `git diff --check`

No WAL format or semantics changed.

## OCI Method

The release binary ran on the 2-OCPU, 12-GiB OCI A1 using the specified SSH
identity (`SHA256:i9NJ3mGGroLtw6CjNS8XJCfT4OjDlfp384E0cOk71ro`). The repository
is on the 29.4-GiB root XFS filesystem; the 200-GB block device is not mounted
for this run. This is a sync-disabled CPU and engine diagnostic, not a
durability result or a measurement on the 200-GB volume.

Each repetition used `planned-blink`, 16 writers, width 1,
`different-leaf-heavy`, working set 100,000, cache 4,096, 16-byte keys,
64-byte values, group limit 64, group byte limit 4,194,304, queue 256, zero
collection delay, disabled sync, two Tokio workers, 1-second warmup,
2-second duration, and seed `0x3a042026`.

The median throughput was 19,141.38 mutations/s, 4.88% below the previous
20,123.54 mutations/s baseline. This is 95.12% of baseline and passes the
85% timer-overhead gate (17,105.01 mutations/s). The throughput comparison is
the gate check; it does not isolate timer cost from run-to-run variation.

## Results

The table uses the median per-repetition component totals. Percentages divide
those totals by the median `wal_group_encode_nanos`. Normalized columns use the
median per-repetition ratios. Commit-only rows also show nanoseconds per
logical commit record.

| component | total ms | % encode | ns/mutation | ns/page image | ns/commit |
|---|---:|---:|---:|---:|---:|
| Page LSN validation | 3.609 | 0.63% | 94 | 47 | — |
| Page-image materialization | 20.838 | 3.63% | 537 | 269 | — |
| Page-image validation | 277.551 | 48.34% | 7,191 | 3,595 | — |
| Digest-input copy | 12.724 | 2.22% | 332 | 166 | — |
| Page payload CRC32C | 51.716 | 9.01% | 1,349 | 675 | — |
| Page header CRC32C | 4.100 | 0.71% | 107 | 53 | — |
| Page frame materialization | 24.325 | 4.24% | 626 | 313 | — |
| Page frame append | 62.377 | 10.86% | 1,628 | 814 | — |
| Commit digest CRC32C | 49.324 | 8.59% | 1,289 | — | 1,289 |
| Commit payload CRC32C | 1.671 | 0.29% | 44 | — | 44 |
| Commit header CRC32C | 2.015 | 0.35% | 52 | — | 52 |
| Commit frame materialization | 1.937 | 0.34% | 51 | — | 51 |
| Commit frame append | 1.756 | 0.31% | 46 | — | 46 |

Per-repetition normalization and throughput:

| repetition | throughput mut/s | mutations | page images | logical groups | encode µs/mutation | encode µs/page | encode µs/group |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 0 | 18,842.82 | 37,711 | 75,422 | 3,872 | 15.98 | 7.99 | 155.63 |
| 1 | 19,141.38 | 38,323 | 76,646 | 4,056 | 14.98 | 7.49 | 141.55 |
| 2 | 19,419.07 | 38,875 | 77,750 | 4,086 | 14.42 | 7.21 | 137.15 |
| Median | 19,141.38 | 38,323 | 76,646 | 4,056 | 14.98 | 7.49 | 141.55 |

All three records report 0 errors, 0 overloads, and 0 full-state clones. Each
record has two page images per successful mutation, one page frame per image,
and one commit frame per mutation.

## Residual

The residual is computed for each repetition as
`max(0, group_encode_nanos - sum(exclusive measured categories))`.
The median residual is 60.714 ms, or 10.58% of median group encode time. The
three repetitions range from 9.48% to 11.18%, below the 15% coverage threshold.
The residual includes unmeasured group/digest buffer allocation, header and
commit-payload construction, validation-independent bookkeeping, and timer
overhead.

## Interpretation

Page-image validation is the largest category at 277.551 ms, 48.34% of group
encoding and about 3.60 µs/page image. The LSN check adds 0.63%; together, page
LSN and image validation account for 48.97% of group encoding.

The materialization/copy set from decision rule B totals 20.95%. The CRC set
from decision rule C totals 18.96%. Neither crosses its threshold. The
measurement points to repeated page-image validation, which includes Blink
page decoding/checking, as the leading measured cost.

## Next Engineering Priority

Decision rule A applies: page LSN validation plus page-image validation is
48.97% of group encoding, above 35%. The next engineering task should evaluate
avoiding redundant full-page decode for trusted internally generated WAL
images while preserving strict validation at public, fault-injection, and
recovery boundaries. Candidate designs are an internally validated
page-image type or a fast trusted internal append path. No such change is part
of this experiment.

## Arena / Buffer Strategy Assessment

**Arena/bumpalo: not recommend.** The measured temporary materialization,
digest-copy, and frame-append set is not dominant, and the data does not show
that allocation/deallocation is the primary cost. Direct pre-sized group-buffer
encoding is a later candidate if a follow-up measurement makes those copies
dominant; it is not the next priority while page-image validation accounts
for 48.34% of encoding.

## Out-of-Scope Copies

The current caller constructs `WalCommit` values with
`transaction.images.clone()`. After encoding, `append_group()` constructs
`CommittedWalBatch` values with `commit.pages.clone()`. The caller also inserts
`image.image` into `final_images`, copying a 4-KiB array. These are outside
`group_encode_nanos`; caller assembly and append timing may include some of
them. They were not mixed into this attribution and remain separate candidates
for `wal_assembly_nanos` and append residual analysis.

## Experiment Status

This was an instrumentation-only change. B-link, logical batching, page
ownership, planner behavior, WAL format and semantics, and recovery behavior
are unchanged. The existing Phase 4 worker-pool implementation and its
correctness remain complete, while the prior OCI result still reports
insufficient worker overlap and a throughput regression requiring review.
Phase 5 has not started.

## Artifacts

- Instrumentation commit: `dfd6b08a86c8bb23a270ab2e98595066bef5228c`
- Local OCI raw artifact:
  [`planned-wal-encoding-attribution-width1.jsonl`](results/oci-a1-2ocpu-12g-200g/wal-encoding-attribution/planned-wal-encoding-attribution-width1.jsonl)
- Raw artifact SHA256:
  `d2594df7593b0781649d1692512845bb52e1aaafd38e3464d9eaf4abeee72e49`
- OCI backup after moving the raw file outside the worktree:
  `/home/opc/dodb-oci-artifacts-wal-encoding-attribution-dfd6b08/`
