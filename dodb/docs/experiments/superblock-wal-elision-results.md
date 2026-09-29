# Blink Superblock WAL Elision Results

## Result

Superblock WAL after-images are omitted from planned Blink transactions when
the transaction leaves the durable tree metadata unchanged. The implementation
passed the required local correctness gates and the OCI width-1 workload met
the strong-success thresholds.

`SUPERBLOCK_ELISION_SHA` is
`00ad22837ae23b6efca6d46e435bbbe574cb9a43`.

## Why Each Transaction Previously Emitted a Superblock

The planned serial executor advanced `BlinkSuperblock.generation`, toggled the
A/B slot, encoded the superblock, and appended that page image for every
logical transaction. A normal point update therefore logged one leaf image
and one superblock image even when the root, free list, and high-water mark
were unchanged.

## Durable Metadata Audit

The superblock stores the root page ID, free-list head, high-water page ID,
checkpoint LSN, and database/shard identity, as well as its generation. The
normal transaction path inherits the checkpoint LSN and identity unchanged.
The tree metadata needed to locate and load pages is the root, free-list head,
and high-water page ID.

For a metadata-stable transaction, recovery can replay its committed leaf page
image without a new superblock image. `recover_data_file()` validates and
writes committed images in WAL order. `load_file()` then selects the highest
valid durable A/B superblock. Its root/free-list/high-water values remain
valid, and the replayed leaf's page ID is within the known page range. Recovery
validates page LSNs against their owning commit and applies the leaf image
before the reopened store is exposed.

The covered crash sequence is: an older valid superblock is on disk; a
metadata-stable leaf update is WAL-synced without a superblock image; the data
file is not flushed; the process crashes; WAL scan validates the complete
commit; recovery writes the leaf; and file loading uses the older superblock
to interpret the recovered page. Structural commits still WAL-log a
superblock image before any later stable commits, so replay leaves the latest
metadata-changing image in place.

## Generation and A/B Slot Semantics

`generation` serves as the process-local publication epoch. The publisher uses
it for immutable `PublishedGeneration` values and `PageVersion` visibility.
Each logical transaction continues to increment the in-memory generation, but
that increment alone no longer requires a durable superblock image. On
restart, no reader pin from the prior process survives; the publisher starts
from the selected durable generation and assigns that epoch to loaded pages.
Revision allocation is restored from recovered page revisions and the WAL's
next LSN. Page reuse is gated by active in-process generation pins, which start
at zero after reopening. Checkpoint writes its own newer generation and
checkpoint LSN after flushing the data pages.

`active_slot` now identifies the last materialized superblock slot. It changes
only when a transaction changes durable tree metadata and emits a new image,
or when checkpoint writes the other slot. A metadata-stable transaction does
not toggle it. The pending `dirty_superblock` image is updated only when a
group contains a new superblock image; a group without one preserves any
earlier pending image.

## Elision Rule and LSNs

Before and after each transaction, the executor compares `root_page_id`,
`free_list_head`, and `high_water_page_id`. A difference in any field emits a
superblock after-image. Generation by itself does not. Normal transaction
execution does not change `checkpoint_lsn` or identity.

The WAL frame format is unchanged. For each commit, the commit LSN is based on
the actual page-image count. Leaf page LSNs are stamped with that commit LSN;
record LSNs remain strictly increasing, commit LSNs and batch IDs advance in
transaction order, and revisions retain their order. No virtual image or
padding record is used. The eligible parallel leaf assembly uses the same
image-count and LSN policy without changing worker scheduling.

## Correctness Matrix

| Case | Expected WAL images | Result |
| --- | --- | --- |
| Existing-leaf value replacement | Leaf only | Reopen restored value and revision |
| Several metadata-stable commits | One data image per commit | All committed values and revisions recovered before data flush |
| Non-splitting insert | Leaf only | Verified |
| Leaf split and root split | Data pages plus superblock | Verified |
| New overflow allocation | Data pages plus superblock when high-water/free-list changes | Verified |
| Overflow free-list change and reuse | Data pages plus superblock | Verified |
| Structural commit followed by stable commit | Structural superblock, then leaf only | Reopen restored root, high-water, and values |
| Checkpoint after elided commits | Checkpoint superblock after data flush | Reopen preserved contents and revisions |
| WAL sync failure | No committed state or publication install | Verified for elided and structural transactions |
| Pinned reader and page reuse | Existing generation/page lifetime rules | Existing regression tests passed |

All required local gates passed: format check, `dodb-core`, `dodb-storage`,
the `phase0-bench` binary tests, workspace tests, and `git diff --check`. The
storage suite passed 110 tests, including its eight crash-recovery child tests.

## OCI Method

The workload used `planned-blink`, 16 writers, width 1,
`different-leaf-heavy`, working set 100,000, cache 4,096, 16-byte keys,
64-byte values, group limit 64, group byte limit 4,194,304, queue 256, zero
collection delay, disabled sync, two Tokio workers, 1-second warmup,
2-second duration, three repetitions, and seed `0x3a042026`.

The default release build ran first. The separate A1 `+crc` control used
`CARGO_TARGET_DIR=target-crc` and `RUSTFLAGS='-C target-feature=+crc'` without
changing repository Cargo configuration. The OCI checkout was clean and
fast-forwarded to the implementation SHA before building. The host's benchmark
checkout is on a 30-GiB XFS root filesystem with 19 GiB free; the historical
artifact directory name includes `200g`, but the benchmark did not run on a
200-GiB mounted filesystem. Sync was disabled, so these results are CPU and
engine measurements, not durability throughput.

The before values come from the same workload's direct-WAL-buffer records at
`32da35d9fe6a397213613ae2f481076373550589`. The prior source emitted one
superblock image per mutation; those older JSONL files predate the explicit
emitted/elided counters. New values are medians of three per-run normalized
ratios. The raw artifacts report zero errors and zero overloads.

## Default OCI Result

| Metric | Before | After | Change |
| --- | ---: | ---: | ---: |
| Throughput (mut/s) | 29,370.88 | 34,360.85 | +16.98% |
| Page images / mutation | 2.00 | 1.00 | -50.00% |
| WAL bytes / mutation | 8,380 | 4,224 | -49.59% |
| Superblock images emitted / mutation | 1.00 | 0.00 | -100% |
| Superblock images elided / mutation | 0.00 | 1.00 | +1.00 |
| Physical superblock encode (µs/mutation) | 0.993 | 0.000 | -100% |
| WAL assembly (µs/mutation) | 1.993 | 1.193 | -40.12% |
| WAL append (µs/mutation) | 8.370 | 5.565 | -33.43% |
| Group encode (µs/mutation) | 5.391 | 3.443 | -36.13% |
| Group write (µs/mutation) | 2.553 | 1.814 | -28.96% |
| Physical execution (µs/mutation) | 7.315 | 5.855 | -19.96% |
| Planning (µs/mutation) | 3.047 | 2.883 | -5.39% |
| Catalog (µs/mutation) | 2.857 | 2.754 | -3.58% |
| Publication (µs/mutation) | 1.898 | 1.819 | -4.17% |
| State install (µs/mutation) | 1.491 | 1.464 | -1.80% |

## `+crc` OCI Result

| Metric | Before | After | Change |
| --- | ---: | ---: | ---: |
| Throughput (mut/s) | 31,105.51 | 36,295.02 | +16.68% |
| Page images / mutation | 2.00 | 1.00 | -50.00% |
| WAL bytes / mutation | 8,380 | 4,224 | -49.59% |
| Superblock images emitted / mutation | 1.00 | 0.00 | -100% |
| Superblock images elided / mutation | 0.00 | 1.00 | +1.00 |
| Physical superblock encode (µs/mutation) | 0.570 | 0.000 | -100% |
| WAL assembly (µs/mutation) | 1.857 | 1.108 | -40.34% |
| WAL append (µs/mutation) | 6.956 | 4.562 | -34.42% |
| Group encode (µs/mutation) | 4.123 | 2.331 | -43.47% |
| Group write (µs/mutation) | 2.377 | 1.919 | -19.27% |
| Physical execution (µs/mutation) | 6.562 | 5.558 | -15.30% |
| Planning (µs/mutation) | 2.931 | 2.737 | -6.62% |
| Catalog (µs/mutation) | 2.982 | 2.781 | -6.74% |
| Publication (µs/mutation) | 1.955 | 1.853 | -5.21% |
| State install (µs/mutation) | 1.545 | 1.442 | -6.71% |

## Remaining Cost and Classification

This is a **strong success** against the declared criteria: all correctness
gates passed, ordinary transactions emit no superblock image, structural
metadata changes emit one, page images fell to 1.00 per mutation, WAL bytes
fell by 49.59%, and both throughput results improved by more than 10%.

Physical execution is now the largest measured top-level component at about
5.86 µs/mutation for default and 5.56 µs/mutation for `+crc`, just above WAL
append at 5.56 and 4.56 µs/mutation. Within physical execution, leaf-load
cloning costs about 2.04 µs/mutation in both builds; planner routing costs
about 1.17 and 1.21 µs/mutation. Leaf-load clone work therefore has higher
priority than shared descent based on this workload. The arena assessment is
unchanged: these timings do not show an arena-friendly temporary-allocation
bottleneck.

This experiment did not change worker scheduling. The eligible parallel leaf
assembly shares the elision policy. Phase 4 remains an experimental path; no
Phase 5 work started.

## Artifacts

- [Default OCI JSONL](results/oci-a1-2ocpu-12g-200g/superblock-elision/planned-superblock-elision-width1.jsonl)
  SHA256: `4f6b2ccc825b583885e8c70bc690ebd1135f45cc99a9f43b2bfec86dd24da88f`
- [`+crc` OCI JSONL](results/oci-a1-2ocpu-12g-200g/superblock-elision/planned-superblock-elision-width1-plus-crc.jsonl)
  SHA256: `ff0a26b47b2eb4f97c5acfbc2c1ba0f77221b530f7f9aa2727ddfcc9baec39b7`
- OCI backup: `/home/opc/dodb-oci-artifacts-superblock-elision-00ad228/`
- Both local SHA256 values match the OCI backup files.
