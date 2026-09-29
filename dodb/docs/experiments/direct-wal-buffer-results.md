# Direct WAL Group-Buffer Results

## Motivation

At the borrowed-routing OCI baseline, trusted WAL group encoding cost about
7.115 µs per mutation. Earlier attribution estimated that temporary payload,
digest, and frame materialization plus copies accounted for about 39% of that
cost. This experiment encodes each frame directly into the final contiguous
WAL group buffer and computes each commit digest incrementally.

Incremental digest calculation does not remove a CRC scan. Every page payload
still participates in its frame payload checksum and in its commit digest.
The optimization targets allocation and copy elimination.

## Old Encoding Pipeline

For every page, the fast encoder allocated a payload vector containing
`page_id_le || page.image`, copied that payload into a per-commit digest
vector, allocated a frame vector, and copied the completed frame into the final
group vector. It then scanned the digest vector for the commit CRC, allocated
a commit-frame vector, and copied that frame into the group vector.

## Direct Encoding Design

`encode_group()` calculates the exact maximum encoded group size and reserves
the final `Vec<u8>` once. It writes frame headers, page IDs, page images,
commit payloads, and trailers directly into that vector. Page payloads remain
split as the 8-byte little-endian page ID and the borrowed 4-KiB image while
they are written; no contiguous page payload is constructed. The commit payload
is a stack array.

The old materialized encoding remains available only as the existing
fault-injectable path used by tests as a byte reference. Production fast-path
encoding does not select it.

## Checksum / Digest Semantics

The page frame payload checksum is computed as CRC32C over the page ID bytes,
then `crc32c_append()` over the page image. For each commit, the accumulator
starts at zero and appends each page ID and image in order. The crate's API
defines `crc32c(data)` as `crc32c_append(0, data)`, so this produces the same
CRC as one CRC over the concatenated payloads.

Randomized checksum differential coverage passed for 512 page sequences. For
each page, split-slice frame CRC matched CRC32C over the old contiguous payload;
each incremental multi-page digest matched CRC32C over the old concatenated
digest input. No `crc32c_combine()` call is used. Page bytes are still scanned
for both checksum purposes; this change does not eliminate duplicate CRC
scanning.

## Correctness

The direct encoder matched the existing fault-injectable materialized encoder
for 300 deterministic randomized valid groups with varying transaction and
page counts, page IDs, and page contents. Reports, WAL bytes, next LSN, and
next batch ID matched in every case. A separate 1,008-page group near the
4-MiB group-size bound also matched byte-for-byte, including reports and next
IDs.

Strict mode continues to validate every page image. Trusted internal release
mode continues to skip only full image decoding while retaining page LSN
validation; debug trusted mode retains full validation. Fault-injected calls
still use the previous materialized path and hook ordering. The WAL byte
format, commit digest semantics, recovery format, and append/sync boundary did
not change.

The Blink format-specific byte comparison for leaf, internal, overflow, and
superblock images was added in test commit
`4a29ed78d2d2ac627805380dd351c34a7ab9cd6a` after the implementation
benchmark. It exercises the unchanged production implementation at
`32da35d9fe6a397213613ae2f481076373550589`.

The storage tests passed short writes, torn-tail handling, WAL reopen and scan,
recovery, malformed strict-image rejection, strict/trusted equivalence, and
fault-hook ordering. Blink recovery and strict/trusted Blink image tests also
passed.

Local gates passed:

- `cargo fmt --all -- --check`
- `cargo test -p dodb-core`
- `cargo test -p dodb-storage`
- `cargo test -p dodb-storage --bin phase0-bench`
- `cargo test --workspace`
- `git diff --check`

## OCI Method

The benchmark ran on the OCI A1 2-OCPU, 12-GiB ARM64 host at implementation
commit `32da35d9fe6a397213613ae2f481076373550589`. The checkout was clean before
the build and benchmark. `/home/opc/dodb` is on the 30-GiB XFS root filesystem;
the 200-GB block device is not mounted as the benchmark filesystem. Sync was
disabled, so these are CPU and engine diagnostics, not durability results.

The primary and `+crc` control each used `planned-blink`, 16 writers, width 1,
`different-leaf-heavy`, working set 100,000, cache 4,096, 16-byte keys,
64-byte values, group limit 64, group byte limit 4,194,304, queue 256, zero
collection delay, disabled sync, two Tokio workers, 1-second warmup,
2-second duration, three repetitions, and seed `0x3a042026`.

The primary binary used the default release configuration. The separate CRC
control used `CARGO_TARGET_DIR=target-crc` and
`RUSTFLAGS='-C target-feature=+crc'`; it did not modify or commit repository
Cargo configuration or global build settings. All six JSONL records identify
the implementation SHA and report zero errors and zero overloads.

The previous comparison values are the supplied borrowed-routing OCI baseline:
27,529.42 mut/s; 55,111 mutations; WAL append 566.517 ms, 10.279 µs/mutation;
group encode 392.155 ms, 7.115 µs/mutation; group write 148.516 ms,
2.695 µs/mutation; and WAL assembly 108.919 ms, 1.976 µs/mutation. The new
fixed-duration values use the median of each per-run normalized ratio, and
cumulative millisecond values use the median per-run total. The fixed-duration
comparison relies on normalized values because each run completed a different
number of mutations.

## Primary Results

Default release build, median of three repetitions:

| metric | before | after | change |
|---|---:|---:|---:|
| throughput (mut/s) | 27,529.42 | 29,370.88 | +6.69% |
| WAL append (ms/run) | 566.517 | 492.270 | -13.10% |
| WAL append (µs/mutation) | 10.279 | 8.370 | -18.57% |
| WAL group encode (ms/run) | 392.155 | 317.054 | -19.15% |
| WAL group encode (µs/mutation) | 7.115 | 5.391 | -24.23% |
| WAL group write (ms/run) | 148.516 | 150.418 | +1.28% |
| WAL group write (µs/mutation) | 2.695 | 2.553 | -5.26% |
| WAL assembly (ms/run) | 108.919 | 117.417 | +7.80% |
| WAL assembly (µs/mutation) | 1.976 | 1.993 | +0.86% |

The group encoder cost normalized to page images fell from 3.558 to
2.695 µs/page image. Across all three source baseline records, the median
per-run encode ratio fell from 75.907 to 62.740 µs/logical group. The supplied
baseline's 392.155-ms encode, 55,111 mutations, and 5,046 groups are from its
median-throughput repetition; that paired repetition was 77.734 µs/group.
The normalized comparison uses the median of per-run ratios for each side.

## Normalized Results

Per-run ratios for the default direct-buffer result were:

| repetition | mut/s | WAL append µs/mutation | encode µs/mutation | encode µs/page image | encode µs/group | group write µs/mutation | assembly µs/mutation |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 0 | 27,570.51 | 9.496 | 6.250 | 3.125 | 62.740 | 2.763 | 2.529 |
| 1 | 29,370.88 | 8.370 | 5.391 | 2.695 | 65.449 | 2.510 | 1.929 |
| 2 | 29,432.45 | 8.352 | 5.322 | 2.661 | 61.825 | 2.553 | 1.993 |
| Median | 29,370.88 | 8.370 | 5.391 | 2.695 | 62.740 | 2.553 | 1.993 |

## Updated Attribution

The attribution below uses the median of each repetition's component time per
successful mutation. Percentages use median group encode cost of 5.391
µs/mutation. `direct encode/store` measures writing the already prepared
header, payload slices, and trailer into the final group vector. Header
preparation and commit-payload construction remain in the residual. Empty old
materialization/copy categories are reported separately under temporary
allocation elimination.

| component | ns/mutation | % encode |
|---|---:|---:|
| Page LSN validation | 87 | 1.62% |
| Full page-image validation | 0 | 0.00% |
| Page payload CRC32C | 1,341 | 24.88% |
| Page header CRC32C | 105 | 1.94% |
| Commit digest CRC32C | 1,286 | 23.85% |
| Commit payload CRC32C | 46 | 0.85% |
| Commit header CRC32C | 50 | 0.92% |
| Direct page encode/store | 1,589 | 29.49% |
| Direct commit encode/store | 52 | 0.96% |
| Residual | 826 | 15.32% |

The summed CRC components are about 2.827 µs/mutation, 52.44% of group
encoding. The residual is about 48.7 ms in the median run. It includes header
and commit-payload preparation, group bookkeeping, report creation, and timing
overhead. Ratios are medians per component, so rounding and independent
medians can make displayed rows differ slightly from the displayed total.

## Temporary Allocation / Copy Elimination

Source inspection and zeroed fast-path metrics confirm these group-encoding
temporaries and transfers were removed:

- Per-page payload `Vec`: removed; page ID and image slices are written into
  the final group vector.
- Per-commit `digest_input` `Vec` and payload copy: removed; the digest is
  accumulated directly over the source slices.
- Per-page frame `Vec` and frame-to-group copy: removed.
- Per-commit frame `Vec` and frame-to-group copy: removed.
- Old page materialization, digest-copy, page-frame materialization/append,
  and commit-frame materialization/append attribution: zero on the fast path.

The final group vector is reserved once using the precomputed size. Direct
page and commit store time remains measured and was not hidden as zero. The
page image still participates in two CRC calculations: its frame payload CRC
and the commit digest CRC.

## Optional +crc Control

The separate `+crc` release build measured 31,105.51 mut/s median, compared
with 29,370.88 mut/s for the default direct-buffer release build: +5.91%.
Its median group encode cost was 4.123 µs/mutation. This is a deployment- and
target-specific build opportunity, not the primary result, and the two raw
artifacts remain separate.

Because the default CRC aggregate is at least 45% of group encode and the
same-workload `+crc` control improves throughput by at least 5%, target-specific
AArch64 compile-time CRC specialization investigation has **high priority**.
No global `RUSTFLAGS`, Cargo configuration, or custom assembly was added.

## Interpretation

This is a **structural success**. The fast path removes the requested temporary
buffers and copies, group encode falls to 75.8% of the supplied baseline cost
per mutation (better than the 80% structural threshold), and throughput is
above baseline. It does not meet strong success: group encode is above 70% of
baseline and throughput is below the required +8%.

The primary workload's default-build CRC aggregate is now the largest measured
group-encoding component at 52.44%. Direct page output stores account for
29.49%. The `+crc` control demonstrates a meaningful target-specific codegen
gain, so CRC specialization is the next engineering priority. No shared
descent or arena work is part of this experiment.

## Next Bottleneck

The default direct-buffer fast path's largest aggregate is page payload,
header, and commit CRC work at 52.44% of group encode. After the scoped
compile-time CRC investigation, reattribute the default and target-specific
builds before considering further WAL work. If target-specific CRC
specialization is not viable, study a WAL-format-preserving dual-checksum or
CRC-pass-reduction design. Do not infer that incremental digest calculation
already removed a CRC pass.

## Arena Assessment

No arena is warranted by this result. The final output is one contiguous
buffer with direct ownership, and the per-page payload and frame allocations
are gone. The remaining measured data points to CRC work and direct writes,
not arena-managed temporary lifetimes.

## Experiment Status

- WAL byte format unchanged.
- Commit digest semantics unchanged.
- Fault-injection path and hook ordering unchanged.
- Strict and trusted validation semantics unchanged.
- Recovery format and replay behavior unchanged.
- B-link plus batching remains experimental; it has not been adopted as the
  main engine.
- Phase 4's persistent worker-pool implementation and correctness remain
  complete, with the existing OCI overlap and throughput concerns. Phase 5 has
  not started.

## Artifacts

- Implementation commit: `32da35d9fe6a397213613ae2f481076373550589`
- Primary OCI JSONL:
  [`planned-direct-wal-buffer-width1.jsonl`](results/oci-a1-2ocpu-12g-200g/direct-wal-buffer/planned-direct-wal-buffer-width1.jsonl)
- Primary OCI JSONL SHA256:
  `effac8ebf2a4d6bee01bb71a0b2669db4de6e5d233467ec65cbe337e49f0a2b8`
- Optional `+crc` OCI JSONL:
  [`planned-direct-wal-buffer-width1-plus-crc.jsonl`](results/oci-a1-2ocpu-12g-200g/direct-wal-buffer/planned-direct-wal-buffer-width1-plus-crc.jsonl)
- Optional `+crc` JSONL SHA256:
  `0aedd2184b00e0384beb6509240c7eb34c1d564fecd408d067f79416281e64a9`
- OCI backup outside the worktree:
  `/home/opc/dodb-oci-artifacts-direct-wal-buffer-32da35d/`
