# AArch64 CRC Specialization Diagnostic

## Motivation

The OCI A1 direct-buffer result reported 29,370.88 mutations/s for the default build and 31,105.51 mutations/s for a separate `RUSTFLAGS='-C target-feature=+crc'` control, a 5.91% throughput increase. The supplied default WAL group-encode attribution was 5.391 µs/mutation, of which page payload CRC was 1.341 µs/mutation and commit digest CRC was 1.286 µs/mutation. Their combined 2.627 µs/mutation is nearly half the encode time.

An earlier OCI microbenchmark reported 592.6 ns per 4,104-byte payload (6.450 GiB/s) for the default build and 258.7 ns (14.774 GiB/s) for global `+crc`. This diagnostic investigates whether a CRC-enabled caller can recover that improvement while preserving a generic AArch64 binary.

## `crc32c` 0.6.8 dispatch structure

The inspected registry sources are `crc32c-0.6.8/src/lib.rs` and `src/hw_aarch64.rs`.

- `crc32c(data)` is inline and delegates to `crc32c_append(0, data)`.
- `crc32c_append(crc, data)` is inline. With `target_arch = "aarch64"` and `armsimd`, it checks `is_aarch64_feature_detected!("crc")` and calls `hw_aarch64::crc32c` on success; otherwise it calls the software implementation.
- `hw_aarch64::crc32c` itself has no `target_feature` attribute. Its `crc_u8` helper is `#[inline]` and `#[target_feature(enable = "crc")]`. `crc_u64_append` is `#[inline(always)]` and calls `__crc32cd`; it has no crate-level target-feature attribute. The standard-library AArch64 `__crc32cb` and `__crc32cd` intrinsics are each `#[inline]` and `#[target_feature(enable = "crc")]`. Thus the helper/intrinsic boundaries remain explicit in the generic dependency build.
- The parallel-three path applies the LONG/SHORT tables to combine three independent CRC accumulators. No checksum algorithm or table logic was changed here.

## Assembly comparison

The available machine for this prototype was Apple AArch64 (`aarch64-apple-darwin`, Apple M1), not OCI A1. The disassembly therefore explains the local code generation and is not claimed as OCI assembly evidence.

In the generic build, the `#[target_feature(enable = "crc")]` specialized caller still contains the dependency's cached runtime-feature test and a branch between `hw_aarch64::crc32c` and `sw::crc32c`. It also retains the slow-path call to `std_detect::detect::cache::detect_and_initialize`. The feature test is inside the inlined `crc32c_append` body, so it remains in each CRC call even though the outer encoder-style caller was specialized and dispatched once.

In the global `+crc` build, the callsite has no runtime feature-selection branch and calls the hardware function directly. In the generic build the CRC backend delegates work across helper calls; in the global build more CRC operations are emitted directly in the backend body. This supports the hypothesis that compiling the dependency with the feature known enabled removes repeated dispatch and relaxes helper target-feature code-generation boundaries. It does not establish the corresponding OCI instruction layout.

The local symbol counts make those differences concrete:

| symbol | build | instructions | branches | calls | direct CRC instructions |
|---|---|---:|---:|---:|---:|
| specialized caller loop | generic | 76 | 8 | 6 | 0 |
| caller loop | global `+crc` | 51 | 1 | 4 | 0 |
| `hw_aarch64::crc32c` | generic | 79 | 2 | 5 | 0 |
| `hw_aarch64::crc32c` | global `+crc` | 109 | 11 | 2 | 9 |

The generic backend's CRC instructions are in separately emitted helper/intrinsic symbols, rather than absent from the binary. The LONG/SHORT table combination remains in both builds; the global backend inlines more CRC work and uses a different tail/control-flow shape.

The symbol-level disassembly excerpts and branch/call counts are in [`assembly-disassembly.txt`](results/aarch64-crc-specialization-local-macos-arm64/assembly-disassembly.txt). The raw release prototype source and run output are in the same result directory.

## Prototype microbenchmark

The prototype uses deterministic 4,104-byte payloads, 250,000 calls per sample, seven samples per variant, and reports the median. The portable variant checks `is_aarch64_feature_detected!("crc")` once before the measured CRC loop, then calls a function marked `#[target_feature(enable = "crc")]`. That models dispatch once for a larger encoder operation. All variants produced the same accumulated checksum on this host.

| variant | median | throughput |
|---|---:|---:|
| generic caller, `-crc` control | 420.1 ns/payload | 9.098 GiB/s |
| runtime-dispatched specialized caller in generic build | 420.3 ns/payload | 9.093 GiB/s |
| global `+crc` | 216.8 ns/payload | 17.626 GiB/s |

The local prototype recovers approximately -0.1% of the global gain: `(portable - generic) / (global - generic)`, using GiB/s, is approximately `-0.06%`. The specialized caller does not meet the 80% gate. These figures are local Apple M1 results and must not be substituted for the OCI baseline or compared directly to the earlier OCI numbers.

## Portable design and safety

The prototype dispatches on runtime CRC support before calling the target-feature function. On this host the feature detector returned true. The generic caller remains compiled with `-crc`; the specialized function is isolated behind its CRC target-feature attribute. The prototype did not execute on an AArch64 CPU lacking CRC, so fallback hardware execution is not evidenced here.

The assembly shows why caller-only specialization is insufficient with the current dependency build: `crc32c_append` still carries its runtime feature-selection path inside the specialized caller. The next candidate would need dependency-level specialization that keeps an explicit safe runtime fallback, or a different CRC implementation. This diagnostic does not choose or implement either option.

## Correctness and semantics

The prototype compared only checksum results across the three benchmark variants. It did not encode WAL groups and therefore did not check WAL byte equivalence, `WalAppendReport`, LSN/batch IDs, recovery, strict/trusted validation, or page-shape coverage. No production Rust code, checksum ordering, WAL format, or validation path changed. The requested WAL correctness suite was not run because the production gate failed before any production edit.

## OCI method and measurements

The OCI A1 instance is running but has no public IP. Direct SSH to its private address timed out. OCI reports the instance Bastion plugin's desired state as `ENABLED`, but the Bastion service reports that the plugin is not running, so a managed SSH session could not be created. A temporary port-forward session also could not authenticate to the instance with the locally available identities and was deleted after the attempt. No OCI files, build, benchmarks, or instance configuration were changed.

Consequently, this diagnostic did not run the OCI prototype or the requested same-SHA end-to-end comparison (A generic forced, B portable runtime-specialized, C global `+crc`). The previously supplied OCI values remain historical input only. There are no fresh OCI WAL append, encode, per-purpose CRC, group-write, recovery, or throughput measurements for this diagnostic. The OCI checkout's HEAD, origin, and cleanliness could not be verified, and no OCI backup was made.

The previously supplied default normalized WAL values are 8.370 µs/mutation append, 5.391 µs/mutation group encode, and 2.553 µs/mutation group write. CRC attribution was 1,341 ns/mutation page payload, 105 ns page header, 1,286 ns commit digest, 46 ns commit payload, and 50 ns commit header. The CRC total was 2,827 ns/mutation, or 52.44% of encode. A/B/C normalized WAL CRC and timing values are unavailable; the local checksum microbenchmark above does not produce WAL metrics.

No `target-cpu` control was run. The local prototype does not establish the OCI CPU's concrete target-cpu result.

The previous OCI direct-buffer artifacts remain at [`planned-direct-wal-buffer-width1.jsonl`](results/oci-a1-2ocpu-12g-200g/direct-wal-buffer/planned-direct-wal-buffer-width1.jsonl) and [`planned-direct-wal-buffer-width1-plus-crc.jsonl`](results/oci-a1-2ocpu-12g-200g/direct-wal-buffer/planned-direct-wal-buffer-width1-plus-crc.jsonl). They are not results from this diagnostic commit.

## Interpretation and next priority

The local prototype failed its 80% recovery gate, and the assembly identifies a dependency-level boundary the proposed caller specialization does not cross. No production WAL implementation was made, and there is no `AARCH64_CRC_SPECIALIZATION_SHA`. Global `-C target-feature=+crc` must remain an opt-in known-hardware deployment artifact: it is not a general AArch64 binary for CPUs without the CRC extension. No global feature flag was added to production configuration.

The CRC-specialization experiment is **diagnostic-only and incomplete for OCI**. For caller-only specialization, stop here: it does not pass the gate. Reopening the OCI portion requires a usable target login or restoring the instance Bastion plugin to `RUNNING`. Once a measured same-host prototype can pass the gate, the planned production candidate must specialize the dependency/backend dispatch rather than merely its caller.

The last supplied top-level OCI measurement still has WAL append as the largest component at 8.370 µs/mutation; inside group encoding, CRC remains the largest aggregate at 52.44%. There is no fresh measurement to re-rank physical execution, WAL assembly copies, remaining direct encoding, or catalog/publication. The next engineering action is to restore target access and complete the OCI prototype gate before selecting another optimization. Shared descent remains low priority at the supplied 1.242 µs/mutation routing cost. An arena is unrelated to this CRC issue and remains unindicated.
