# OCI WAL CRC Backend Diagnostic

## Scope and source state

This is a diagnostic only. No production Rust source, Cargo configuration,
WAL format, or benchmark harness was changed. The local and OCI checkouts were
on `experiment/b-link-batched-engine` at `b0b207fc9d1b135d4d0206529b9c22b4b38d22e7`
with a clean worktree before this document was added.

The OCI host is Oracle Linux 9.8 on OCI A1 (2 OCPU, 12 GiB shape RAM). The
required SSH identity was used with `IdentitiesOnly=yes`. Its repository is
`/home/opc/dodb`; as in prior diagnostics, that path is on the 30 GiB root
filesystem, not on the unmounted 200 GB block device. This task measured CPU
microbenchmarks only, not storage durability.

## OCI CPU and Rust environment

| item | observed value |
|---|---|
| `uname -m` | `aarch64` |
| CPU | ARM Neoverse-N1, 2 online CPUs |
| `/proc/cpuinfo` `Features` | includes `crc` |
| `rustc --version --verbose` | `rustc 1.98.1 (48a229cea 2026-09-01)`, host `aarch64-unknown-linux-gnu`, LLVM 22.1.8 |
| `cargo --version` | `cargo 1.98.1 (797e8a9bc 2026-08-05)` |
| `cargo tree -p dodb-storage | grep crc32c` | `crc32c v0.6.8` |
| rebuilt `target/release/phase0-bench` SHA-256 | `b3e305ae4b07287a4f11433c692d38fd685c59a7891ea848104b9e0a6e67e7cf` |

The lockfile pins `crc32c 0.6.8` (checksum
`3a47af21622d091a8f0fb295b88bc886ac74efcc613efc19f5d0b21de5c89e47`). Its
release build output contains `cargo::rustc-cfg=armsimd`, and a fresh
standalone release build's `crc32c` rustc invocation contains `--cfg armsimd`.
The current `phase0-bench` release binary was rebuilt from the specified
checkout. It contains a runtime feature dispatch: the CRC call checks the
detected CRC feature and branches to the `crc32c::hw_aarch64::crc32c` function
when present, with a software fallback when absent. The AArch64 hardware
function contains CRC instructions including `crc32cx` and `crc32cb`.

**Backend conclusion: hardware CRC is active on this OCI CPU.** This is based
on all three pieces of evidence: `crc` is exposed by the CPU, the `armsimd`
implementation is compiled, and the release binary's runtime dispatch reaches
an implementation containing ARM CRC instructions. The binary is not merely
carrying an unused instruction sequence.

## CRC microbenchmark

A temporary standalone release binary used `crc32c 0.6.8`, deterministic
4,104-byte inputs (`8-byte page ID + 4,096-byte page image`), 250,000
iterations per round, five rounds, median throughput, `black_box`, and
accumulated checksums. The default build and a separate
`RUSTFLAGS="-C target-feature=+crc"` build ran on OCI. The benchmark asserted
that the two-page concatenated checksum, per-page checksums combined with
`crc32c_combine`, and incremental `crc32c_append` produced identical results.

| build | one 4,104-byte CRC | throughput |
|---|---:|---:|
| default current configuration | 592.6 ns/payload | 6.450 GiB/s |
| explicit `+crc` target feature | 258.7 ns/payload | 14.774 GiB/s |

The explicit target-feature build was 2.29x faster in this microbenchmark.
This does not mean the current binary falls back to software: the default
binary's runtime-dispatched ARM instructions were confirmed in disassembly.
It does indicate that target-feature code generation materially changes this
microbenchmark. A production-wide or global `RUSTFLAGS` change was not made;
the current task establishes the active backend and records the result for a
separately scoped, target-specific build investigation.

For two 4,104-byte page payloads per transaction (the current width-1
attribution reports two page images per successful mutation), the same
microbenchmark compared the commit digest work after page checksums had been
calculated:

| digest method | default build, ns/2-page transaction | explicit `+crc`, ns/2-page transaction | correctness |
|---|---:|---:|---|
| second full CRC over concatenated payloads | 2,342 | 1,026 | matched |
| `crc32c_combine` of the two page CRCs | 19,792 | 20,871 | matched |
| `crc32c_append` over the two payload slices | 2,368 | 1,036 | matched |

These are CPU-only method timings; they omit WAL frame construction and buffer
allocation/copying. `crc32c_combine` was about 8.5x slower than the second full
scan in the default build and about 20x slower in the explicit build. The
incremental append path was about 1% slower than scanning a contiguous
concatenation here. It can still avoid constructing and copying a separate
`digest_input` buffer when integrated with direct encoding, but this result
does not support using `crc32c_combine` for the current two-page case.

## Current WAL path and normalized cost

At `b0b207f`, the current user-supplied width-1 medians are:

| component | µs/mutation |
|---|---:|
| WAL assembly | 1.98 |
| WAL append | 10.28 |
| of which: group encode | 7.12 |
| of which: group write | 2.69 |
| append residual | 0.47 |

The top-level attribution still makes WAL the primary target. The existing
trusted-WAL attribution measured component shares on its earlier OCI build.
Applying those shares to the current 7.12 µs/mutation encode cost gives this
normalized estimate (not a fresh category-timer run at `b0b207f`):

| aggregate | source share of group encode | estimated current cost |
|---|---:|---:|
| CRC work | 38.49% | 2.74 µs/mutation |
| copy/materialization | 39.26% | 2.80 µs/mutation |
| other/residual: page-LSN check plus instrumentation residual | 22.25% | 1.58 µs/mutation |
| total | 100.00% | 7.12 µs/mutation |

The CRC aggregate includes page payload CRC, page header CRC, commit digest
CRC, commit payload CRC, and commit header CRC. The copy/materialization
aggregate includes page-image materialization, digest-input copy, page and
commit frame materialization, and frame append into the group buffer. The
earlier document's 37.14% copy/materialization figure covered the page-frame
set; including commit-frame materialization and append adds 2.10 percentage
points. The pure residual was 21.03%; page-LSN validation was 1.23%. Rounded
source percentages sum to 100.01%, so the normalized aggregate rounds its
other/residual row to 22.25%. Percentages are carried forward only to provide
a current cost scale, so category-level changes at the latest HEAD remain
unmeasured.

`WalLog::encode_group()` currently reserves one group buffer, then for each
commit creates a `digest_input` vector. For each page it creates a 4,104-byte
payload vector, copies that payload into `digest_input`, creates a frame vector,
then copies the frame into the group buffer. It computes the commit digest
over `digest_input` after the page frames are assembled. `encode_frame_impl()`
computes a payload CRC and a header CRC for every frame. `BlinkState` assembles
`WalCommit` page-image clones before calling the crate-private trusted append
path.

The attribution evidence and normalized source categories are from
[`trusted-wal-image-results.md`](trusted-wal-image-results.md) and
[`wal-group-encoding-attribution.md`](wal-group-encoding-attribution.md).
Those documents report the older trusted-path group encode median of 6.986
µs/mutation and the category timings used above; the supplied current 7.12
µs/mutation value is used only as a normalization scale.

## Duplicate CRC-pass analysis

The commit digest is exactly CRC32C over the concatenation of page frame
payloads in commit order:

```text
CRC32C(page_payload_0 || page_payload_1 || ...)
```

Each payload is the little-endian page ID followed by the 4,096-byte page
image. `encode_frame_impl()` also computes an individual CRC32C over each full
page payload for that frame's payload checksum. Thus, in the width-1 append
path, the same 4,104-byte page payload is CRC-scanned once for its page frame
checksum and a second time as part of the commit digest: two full payload
passes per page. With two page images per mutation, that is 16,416 payload
bytes scanned by those two CRC purposes per mutation, excluding small frame
headers and the 16-byte commit payload.

WAL scanning/recovery independently verifies each stored frame payload
checksum and then reconstructs the concatenated digest input to verify the
commit digest. Those scans are intentional read-time integrity validation and
are not included in the append-path count above.

The source-level fast-path versus fault-injectable-path test compares WAL bytes
and reports; strict versus trusted valid images are also checked for identical
bytes and metadata. This diagnostic made no digest or format change.

## Candidate comparison

### Direct encoding into a pre-sized group buffer

This could remove per-page payload and frame vectors, the copy into
`digest_input`, and the frame-to-group append copy. The normalized copy and
materialization category is about 2.80 µs/mutation (39.26% of group encoding),
so this is a meaningful CPU target. It would need to preserve page payload
checksums, exact WAL bytes, commit digest semantics, and report lengths. It is
an estimate of the removable category, not a promised speedup; CRC and
validation boundaries are separate.

### Reduce the digest pass

`crc32c_combine()` preserves the concatenated CRC semantics but loses badly for
the current two-page transaction size: about 19.8 µs/transaction against 2.34
µs for the second full digest scan in the default microbenchmark. It is not a
candidate for this workload.

Incremental `crc32c_append()` across page payloads returns the same digest and
measured within about 1% of the contiguous second scan. On its own it does not
remove the second CRC pass. Used while writing page bytes into the final group
buffer, however, it can avoid both `digest_input` materialization and its copy
without changing the digest's byte order or semantics. The append strategy
should therefore be evaluated as part of direct group-buffer encoding, not as
a standalone CRC-compute optimization.

## Shared-descent upper bound and arena

Current borrowed-routing medians are 3.106 µs/mutation planning and 1.242
µs/mutation routing. Removing every route would save at most 1.242
µs/mutation, or 40.0% of planning, leaving 1.864 µs/mutation planning. Across
the listed top-level attributed costs (planning + physical execution + WAL
assembly + WAL append = 22.626 µs/mutation), that is a 5.5% gross cost
reduction, or a theoretical 1.058x speedup if all those costs scaled
serially. This is a counter-scope upper bound, not a throughput prediction.
Shared descent is a later optimization; WAL work has greater measured
potential and is the current priority.

An arena or bump allocator does not directly remove the measured byte copies.
The diagnosis supports direct final-buffer encoding, not introducing an arena.
No arena change is recommended.

## Recommendation

Do not use `crc32c_combine()` for the present two-page width-1 case. The next
WAL experiment should evaluate **direct group-buffer encoding with an
incremental commit digest while writing the page payloads**. This combines
the two promising parts: it targets the measured 39.26% copy/materialization
set and avoids `digest_input`; the microbenchmark does not show a material
compute penalty for incremental append. First validate byte-for-byte WAL and
digest equivalence and measure the integrated implementation under the same
OCI workload. The separate 2.29x `+crc` microbenchmark result is evidence for
a target-specific build investigation, but does not by itself justify a
global build configuration change.

## Final checkout state

Before this diagnostic document was added, both local and OCI checkouts had
branch `experiment/b-link-batched-engine`, HEAD and origin at
`b0b207fc9d1b135d4d0206529b9c22b4b38d22e7`, and clean worktrees. The temporary
CRC project and build artifacts were under `/tmp` on OCI.
