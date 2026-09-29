# Current Blink CRC Reattribution Results

## Motivation

The leaf payload-sharing result left two open measurements: WAL group encoding
rose to 3.450 µs/mutation, and physical page encoding rose to 2.475
µs/mutation. Earlier same-host measurements had found a 5.91% throughput gain
from a global AArch64 `+crc` build and CRC counters at 52.44% of WAL group
encoding, but those measurements predate superblock WAL elision and leaf
payload sharing. This diagnostic repeats the default versus global `+crc`
comparison at the current source revision. It makes no production source or
format changes.

These are sync-disabled CPU and engine diagnostics. They are not durability
throughput measurements.

## Source and Environment

Both binaries were built from source SHA
`9e82d9da4db9c1818ce725addf112281020dc6a7` on branch
`experiment/b-link-batched-engine`.

The measurement host was OCI VM.Standard.A1.Flex, ARM64 Neoverse-N1, 2 OCPU,
12 GiB RAM, Oracle Linux Server 9.8, kernel
`6.12.0-206.104.4.4.el9uek.aarch64`, and rustc
`1.98.1 (48a229cea 2026-09-01)`. `lscpu` reported the CPU feature `crc32`;
`/proc/cpuinfo` reported `crc32` as well. The checkout is on the 30-GiB root
XFS filesystem with 18 GiB available. The 200-GiB block device was not used.

Default release build:

```bash
CARGO_TARGET_DIR=target/oci-current-default \
  cargo build --locked --release -p dodb-storage --bin phase0-bench
```

Binary SHA256: `3899baaff31950565f21eed0d832b29fe2f1df268299ac7a740aba57619aa53f`.

Global CRC build:

```bash
RUSTFLAGS='-C target-feature=+crc' \
CARGO_TARGET_DIR=target/oci-current-plus-crc \
  cargo build --locked --release -p dodb-storage --bin phase0-bench
```

Binary SHA256: `f1d8252ab249d94171412a3e989d5306ca541d05fac75a4ae783d01c58355e42`.
The global feature flag makes this binary specific to AArch64 CPUs with the
CRC extension; it is not a portable generic AArch64 build.

## Workload and Method

Both binaries ran the same `planned-blink` width-1 write workload: 16 writers,
`different-leaf-heavy`, working set 100,000, cache capacity 4,096, 16-byte
keys, 64-byte values, maximum group requests 64, maximum group bytes 4,194,304,
queue capacity 256, collection delay 0 µs, sync disabled, two Tokio workers,
1-second warmup, 2-second duration, three repetitions, and seed base
`0x3a042026`. Repetition seeds were identical (`0x3a042026` through
`0x3a042028`). The exact CLI invocation was:

```bash
phase0-bench --engine planned-blink --suite write --writers 16 --readers 0 \
  --widths 1 --distributions different-leaf-heavy --working-set 100000 \
  --cache-capacity 4096 --key-size 16 --value-size 64 --group-limit 64 \
  --group-bytes 4194304 --queue-capacity 256 --collection-delay 0us \
  --sync-mode disabled --tokio-workers 2 --warmup 1s --duration 2s \
  --repetitions 3 --seed 0x3a042026 --output <artifact.jsonl>
```

Throughput is reported as mutation operations per second. Every timing counter
is divided by that repetition's `successful_operations`; table cells show the
median of the three per-run ratios and their min..max. No ratios were computed
from aggregate totals across repetitions.

## Primary Results

| Metric | Default median (min..max) | Global `+crc` median (min..max) | `+crc` / default |
| --- | ---: | ---: | ---: |
| Mutation ops/s | 43,416.6 (43,138.0..43,953.2) | 45,121.9 (44,705.5..45,952.9) | **1.039x** |
| Physical execution, ns/mutation | 4,739 (4,710..4,826) | 4,416 (4,377..4,436) | 0.932x |
| Physical mutation, ns/mutation | 1,906 (1,880..1,950) | 1,926 (1,895..1,931) | 1.010x |
| Leaf-load clone, ns/mutation | 634 (623..664) | 636 (625..645) | 1.003x |
| Physical page encode, ns/mutation | 2,450 (2,446..2,476) | 2,105 (2,096..2,115) | 0.859x |
| Physical superblock encode, ns/mutation | 0 (0..0) | 0 (0..0) | — |
| Planning, ns/mutation | 2,523 (2,474..2,607) | 2,618 (2,477..2,621) | 1.038x |
| Planner route, ns/mutation | 1,129 (1,080..1,138) | 1,131 (1,067..1,152) | 1.001x |
| Catalog construction, ns/mutation | 1,597 (1,574..1,623) | 1,595 (1,573..1,609) | 0.999x |
| Generation publication, ns/mutation | 1,334 (1,333..1,365) | 1,354 (1,330..1,375) | 1.015x |
| State install, ns/mutation | 575 (557..584) | 556 (554..574) | 0.967x |
| WAL assembly, ns/mutation | 1,052 (1,027..1,159) | 1,096 (1,059..1,186) | 1.041x |
| WAL append, ns/mutation | 5,233 (5,110..5,522) | 4,506 (4,425..5,023) | **0.861x** |
| WAL group encode, ns/mutation | 3,247 (3,182..3,650) | 2,423 (2,297..3,072) | **0.746x** |
| WAL group write, ns/mutation | 1,641 (1,594..1,696) | 1,793 (1,667..1,824) | 1.092x |

The default build still has WAL append as the largest measured top-level
component. Global `+crc` lowers median WAL group encode by 25.4%, WAL append by
13.9%, and raises throughput by 3.9%. WAL group write is not lower in the
`+crc` median; it is not the source of the throughput gain.

## WAL Group-Encode Attribution

Attribution timers are exclusive. Page-image validation, page materialization,
digest-copy, and old frame-append/materialization counters are zero on this
trusted direct-encoding path. The direct encode counters cover writing page
and commit output into the final group buffer. CRC aggregate is the sum of the
five CRC counters below. As above, the timing values are medians of per-run
normalized costs.

| WAL encode component | Default ns/mutation (min..max) | Global `+crc` ns/mutation (min..max) |
| --- | ---: | ---: |
| Page LSN validation | 43 (42..44) | 40 (40..42) |
| Page-image validation | 0 (0..0) | 0 (0..0) |
| Page payload CRC32C | 686 (677..708) | 356 (343..359) |
| Page header CRC32C | 53 (52..53) | 46 (46..46) |
| Commit digest CRC32C | 641 (640..641) | 297 (296..298) |
| Commit payload CRC32C | 50 (49..52) | 43 (42..46) |
| Commit header CRC32C | 51 (51..52) | 47 (47..49) |
| **All CRC attribution** | **1,483 (1,471..1,502)** | **790 (774..796)** |
| Page direct output / frame write | 952 (914..1,211) | 855 (813..1,190) |
| Commit direct output | 50 (49..58) | 49 (47..55) |
| Residual encode time | 729 (675..853) | 688 (622..989) |

The median per-run CRC share of WAL group encode is 45.3% for default
(40.6..47.2%) and 32.6% for global `+crc` (25.9..33.7%). The median CRC cost
falls by 693 ns/mutation. Page payload and commit digest CRC are the largest
CRC categories in both builds. Residual is calculated per run as group encode
minus all exclusive attribution counters, before normalization.

## Invariants and Output Shape

Every repetition in both artifacts reported zero errors, zero overloads, zero
full-state clones, zero leaf-entry clones, zero leaf-install clones, and zero
cached-refresh clones. Each repetition produced exactly 1.0 page image per
successful mutation and 4,224 WAL bytes per successful mutation. The repetition
seeds and workload fields match between builds. CRC compilation changed neither
logical page count nor WAL shape.

## Comparison with Historical Measurements

The previous current-source baseline at the leaf payload-sharing result had
42,948.3 mutations/s, 5,387.8 ns/mutation WAL append, 3,450.4 ns/mutation WAL
group encode, and 2,474.5 ns/mutation physical page encode. The new default
repeats at 43,416.6 mutations/s, 5,233 ns WAL append, 3,247 ns group encode,
and 2,450 ns page encode. The group-encode increase seen in the earlier
leaf-sharing comparison is not reproduced by this default median.

The earlier direct-WAL-buffer comparison's global `+crc` throughput gain was
5.91%. The current same-source comparison is 3.93% by the two median
throughputs. The old 52.44% CRC share is also lower at the current default
build's 45.3%; both comparisons include later superblock WAL elision and leaf
payload sharing only in the current result. The historical measurements are
not reused as current performance evidence.

## Physical Page Encoding and Planning

Default physical page encoding is 2.450 µs/mutation, and each of its three
runs is at least 2.446 µs. It therefore remains above the 2.3-µs stability
threshold and is a larger total cost than the 0.693-µs median WAL CRC reduction
available in this diagnostic. The `+crc` build's page-encode timer is lower
(2.105 µs/mutation), illustrating that global target features also affect
non-CRC code generation; this A/B is an upper-bound control, not an isolated
CRC-only intervention.

Read-only inspection of `encode_leaf_body_into()` shows that
`leaf_body_layout()` calls `ensure_sorted_leaf()`, which checks ordering and
validates every encoded key. The subsequent `encode_leaf_record_into()` also
validates each encoded key before copying the same key bytes. This is a visible
duplicate validation pass in the timed page encoding path. It is an
engineering candidate only; this diagnostic does not change it.

Planning is 2.523 µs/mutation in default. The existing route timer accounts
for 1.129 µs (44.8%); the per-run median of `planning - route` is 1.394
µs/mutation. The borrowed-routing result had 1.830 µs/mutation of non-route
planning cost at its earlier source. Current routing is still a material
component, but its measured cost does not make shared descent an automatic next
priority.

## Decision

The diagnostic CRC high-priority criteria all pass: global `+crc` WAL group
encode is 0.746x default (at most 0.80x), WAL append is 0.861x (at most 0.90x),
and throughput is 1.039x (at least 1.03x). CRC remains the next engineering
priority for a portable backend/runtime-specialization investigation. The
global `+crc` binary is a known-hardware upper-bound/control and does not mean a
production code change is complete or suitable for generic AArch64 machines.

After that CRC investigation, physical page encoding is the clearest measured
follow-up: it is stable above 2.3 µs/mutation, and the source shows repeated
key validation within the encoding pass. The current result does not establish
which portion of the page timer is recoverable. Planner routing and non-route
planning remain behind these measured candidates. No next optimization was
started as part of this task.

## Artifacts

| Repository artifact | SHA256 |
| --- | --- |
| `docs/experiments/results/oci-a1-2ocpu-12g-200g/current-crc-reattribution/planned-current-default-width1.jsonl` | `f2a1f2647c54a86322a45329f1834d12d08c696629da6a3b07adae14bbbba3bd` |
| `docs/experiments/results/oci-a1-2ocpu-12g-200g/current-crc-reattribution/planned-current-plus-crc-width1.jsonl` | `c27ac8d54614c0b43a1a09eec1926d19dea5a4d8ae88e4b2a5081d3f7994422e` |

Both repository artifact hashes match their raw OCI backup copies at
`/home/opc/dodb-oci-artifacts-current-crc-reattribution-9e82d9d/`.
