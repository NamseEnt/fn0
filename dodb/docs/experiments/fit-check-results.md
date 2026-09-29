# Exact Blink Fit Check Results

## Motivation

The physical execution attribution identified `leaf_fits()` as a leading
candidate inside the width-1 mutation residual. A normal leaf mutation asks
whether the updated entries fit before encoding the actual page image.

## Existing Problem

`leaf_fits()` called `encode_leaf_body()` and discarded the result. This
allocated and populated a page body, allocated each leaf record, allocated the
slot list, and copied the finished body. `internal_fits()` used the same full
encoding path. Neither fit check needs serialized bytes.

## Allocation Scope

The new fit path no longer creates the page body, per-entry record vectors,
slot vector, or final body copy. It is not strictly allocation-free: key
canonicality checks call `validate_encoded_key()`, which calls
`DocumentKey::decode()` and materializes owned key components. The performance
classification below applies to removing duplicate page serialization; the
original zero-`Vec`-allocation requirement remains unmet for key validation.

## Design

### Exact leaf layout calculation

`leaf_body_layout()` validates sorted canonical leaf keys, accounts for the
header, slots, optional high key, and each record length, and checks the lower
and upper regions for overlap. `leaf_record_encoded_len()` accounts for the
fixed record header, encoded key, and inline value bytes; tombstones and
overflow references have no inline value bytes.

### Exact internal layout calculation

`internal_body_layout()` validates the leftmost child and sorted canonical
separators, then calculates header, slots, optional high key, and separator
record lengths with checked arithmetic.

### Encoder equivalence

`leaf_fits()` and `internal_fits()` now call the layout calculators. The
right-sibling and level arguments remain in the existing signatures because
they do not affect fit. The page encoders and their output bytes are unchanged.

## Correctness

The leaf differential covers empty and single-entry leaves, inline values,
tombstones, overflow references, high keys, invalid ordering, invalid high
keys, and a constructed exact-capacity value plus one extra byte. The internal
differential covers single and many separators, high keys, invalid child IDs,
invalid ordering, invalid high keys, and the largest fitting collection plus
one additional separator. Layout offsets are compared with the encoded header.

A fixed-seed test checks 500 randomized leaf and internal layouts, including
valid and invalid ordering, varying key/value lengths, optional high keys, and
invalid child IDs. All layout acceptance results matched the existing
encoders.

Passed locally:

- `cargo fmt --all -- --check`
- `cargo test -p dodb-storage` (92 tests)
- `cargo test -p dodb-storage --bin phase0-bench` (10 tests)
- `cargo test --workspace`
- `git diff --check`
- `cargo build --release -p dodb-storage --bin phase0-bench`

The requested local debug smoke completed one planned-blink width-1
different-leaf-heavy run with 16 writers, working set and cache 4096, disabled
sync, 1 s warmup, and 1 s duration. It reported zero errors, zero overloads,
zero full-state clones, and zero state-clone nanoseconds. This smoke is not
performance evidence.

## OCI Method

The initial SSH failure was caused by using the wrong local identity. The
specified `/Users/namse/Downloads/ssh-key-2026-09-23.key` was readable with
permission `0600`; using it with `IdentitiesOnly=yes` authenticated
successfully. The OCI worktree was clean and fast-forwarded to
`8e53d3f979e72fc31629a897e3a9b76204a07713`. Its source diff from implementation
commit `f168b40074f1d9578015f11d258a9d609790079d` was empty.

The release benchmark binary was built at the docs-only result commit. Both
runs used 16 writers, working set 100,000, cache 4,096, 16-byte keys, 64-byte
values, group limit 64, group byte limit 4 MiB, queue 256, zero collection
delay, disabled sync, two Tokio workers, 1 s warmup, 2 s duration, and three
repetitions. Width 1 used different-leaf-heavy and seed `0x3a042026`; width 16
used uniform and seed `0x3a032026`.

`/home/opc/dodb` is on a 30 GB XFS root filesystem. The 200 GB block device is
not mounted as the benchmark filesystem. These sync-disabled measurements are
CPU and engine diagnostics on the same host path used for the physical
attribution baseline; they are not durability results or measurements on the
200 GB volume.

## Width-1 Result

The physical-attribution result is the before value. After values are medians
of the three OCI repetitions. Mutation residual subtracts the median leaf load,
entries, and install clone times from median physical mutation, floored at
zero.

| metric | before | after | change |
|---|---:|---:|---:|
| throughput | 15,222.51 mut/s | 15,746.54 mut/s | +3.44% (1.034x) |
| processing | 1,905.748 ms | 1,900.878 ms | -0.26% |
| physical execution | 619.932 ms | 542.277 ms | -12.53% |
| physical mutation | 319.628 ms | 223.468 ms | -30.09% |
| mutation residual | 209.696 ms | 110.602 ms | -47.26% |
| physical restamp | 3.266 ms | 3.341 ms | +2.28% |
| cached refresh | 40.518 ms | 42.268 ms | +4.32% |
| page encode | 185.561 ms | 201.190 ms | +8.42% |
| superblock encode | 31.052 ms | 32.446 ms | +4.49% |
| planning | 346.575 ms | 354.038 ms | +2.15% |
| WAL append | 567.568 ms | 605.107 ms | +6.61% |

The mutation residual fell by more than the 30% structural threshold, and
throughput rose by more than 3%. The strong thresholds were not both reached:
residual reduction was below 50%, and throughput gain was below 8%.

## Physical Execution Breakdown

| component | before | after | change |
|---|---:|---:|---:|
| physical execution | 619.932 ms | 542.277 ms | -12.53% |
| physical mutation | 319.628 ms | 223.468 ms | -30.09% |
| └ mutation residual | 209.696 ms | 110.602 ms | -47.26% |
| physical restamp | 3.266 ms | 3.341 ms | +2.28% |
| cached refresh | 40.518 ms | 42.268 ms | +4.32% |
| page encode | 185.561 ms | 201.190 ms | +8.42% |
| superblock encode | 31.052 ms | 32.446 ms | +4.49% |

The three mutation clone medians were leaf load `24.133 -> 26.075 ms`
(+8.05%), entries `36.696 -> 35.750 ms` (-2.58%), and install
`49.104 -> 51.041 ms` (+3.95%). These shifts are all below 15%. Page and
superblock encoding also changed by less than 15%; these encode actual
transaction images and are separate from fit checking.

## Width-16 Regression Check

| metric | before | after | change |
|---|---:|---:|---:|
| throughput | 14,575.44 mut/s | 15,372.02 mut/s | +5.47% |
| physical execution | 591.779 ms | 518.692 ms | -12.35% |
| physical mutation | 392.672 ms | 304.149 ms | -22.55% |
| mutation residual | 228.007 ms | 130.730 ms | -42.66% |
| page encode | 183.652 ms | 198.290 ms | +7.97% |
| superblock encode | 2.019 ms | 2.137 ms | +5.84% |

Throughput did not regress; it increased by 5.47%, clearing the 10% regression
gate.

## Interpretation

The implementation removes the duplicate full-body serialization from both
fit checks and the differential tests establish matching acceptance for the
tested cases. The width-1 result is **structural success**: residual was at
most 70% of baseline and throughput increased by at least 3%. Width-16 passed
its regression gate.

## Next Bottleneck

The strict allocation-free fit-check requirement still needs a non-allocating
canonical-key validation path. After that, the measured next physical
execution priority is actual page image encoding: page plus superblock encoding
is `233.635 ms`, or `43.08%` of width-1 physical execution. Candidate work is
removing the `BODY_SIZE` temporary vector, per-entry record vectors, and final
`body.to_vec()` copy. Neither change was implemented here.

## Experiment Status

B-link layout, split selection, page and WAL formats, batching semantics, and
WAL semantics are unchanged. The implementation commit is
`f168b40074f1d9578015f11d258a9d609790079d`. The result artifacts identify the
docs-only commit `8e53d3f979e72fc31629a897e3a9b76204a07713`; source is unchanged
from the implementation commit. Phase 4 was not changed or started by this
task; Phase 5 has not started.

## Artifacts

- Local debug smoke: `/tmp/dodb-fit-check-smoke.jsonl`; smoke only, not
  committed performance data.
- Width 1: [`planned-fit-check-width1.jsonl`](results/oci-a1-2ocpu-12g-200g/fit-check/planned-fit-check-width1.jsonl), SHA256 `0c6bb9c106fb7da61c69e79a2bafec2b069b4c02afa5e616cb95a4015e821baf`.
- Width 16: [`planned-fit-check-width16.jsonl`](results/oci-a1-2ocpu-12g-200g/fit-check/planned-fit-check-width16.jsonl), SHA256 `39170bb47baab05eb8d1c5807c7aefe2cba992c3b2ebbed79aa3076dfda6405d`.
- OCI hashes matched the local copies. The OCI copies were moved to `/home/opc/dodb-oci-artifacts-fit-check-f168b40/`; the OCI worktree was clean afterward.
