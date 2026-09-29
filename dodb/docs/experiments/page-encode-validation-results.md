# Blink Leaf Page Encode Validation Result

Status: structural/useful success. The width-1 OCI run reduced normalized
physical page encoding to 0.784x the fresh same-host base, raised throughput
to 1.045x, and kept the width-16 throughput control at 0.977x. The width-16
control also showed higher WAL append and WAL group encode costs; these
measurements are reported below without extending the optimization scope.

## Motivation and exact revisions

The task targeted the approximately 2.11 us/mutation width-1 physical page
encoding cost in the planned B-link engine. The fresh OCI base measured
2,147.2 ns/mutation.

- Base: `2d59784eab770ab0705a9201d064063d860f4e54`
- Candidate implementation: `408aaedb1f218e25d28e3bf0e01d91d2b2655f3e`
- Implementation commit: `408aaedb1f218e25d28e3bf0e01d91d2b2655f3e`

## Source audit and design

The production page image path is `encode_blink_page()` to
`encode_leaf_body_into()`. The leaf body first calls `leaf_body_layout()`,
which calls `ensure_sorted_leaf()` to reject unordered keys and validate each
canonical encoded key. It then computes each record length and invokes
`encode_leaf_record_into()`, which repeated canonical validation for the
same key immediately before writing it.

`leaf_body_layout()` is also called from `leaf_fits()` during mutation-time
fit checks, so its ordering and canonical-key validation remain intact. The
single retained strict validation boundary is the layout calculation. The
private record writer is now named `encode_leaf_record_into_validated()` and
is called only after that layout succeeds. It no longer validates each key a
second time.

`encode_blink_page()` is private to the storage implementation. Production
callers encode the initialized root or pages produced by checked decode and
internal mutation paths. Unit tests can construct malformed in-memory pages
and call it directly, so the strict layout validation remains necessary.
Recovery continues to validate decoded page keys and page images. Internal
page encoding validates separators in `internal_body_layout()` and did not
have a matching repeated per-record key validation, so it was not changed.

The change removes one duplicate check only. Page headers, body fields,
offsets, values, and checksums are written the same way; no page/WAL format,
version, CRC implementation, AArch64 target policy, planner, worker, or Phase 5
code changed. Direct/reference encoder comparisons remain byte-identical.

## Correctness gates

All requested local gates passed:

- `cargo fmt --all -- --check`
- `cargo test -p dodb-core` — 6 passed
- `cargo test -p dodb-storage` — 113 library tests, 10 benchmark tests, and 8 recovery integration tests passed
- `cargo test -p dodb-storage --bin phase0-bench` — 10 passed
- `cargo test --workspace` — all workspace unit, integration, and doc tests passed
- `git diff --check`

Coverage relevant to this change includes:

- `page_encoder_rejects_noncanonical_leaf_key`: strict and reference encoders reject malformed keys.
- `leaf_layout_matches_encoder_acceptance`: ordering violations, invalid high keys, and exact-fit/one-byte-over page boundaries.
- `direct_blink_encoding_matches_reference_for_required_shapes` and `randomized_direct_blink_encoding_matches_reference`: direct output equals the reference codec for required and 800 randomized page cases.
- `public_wal_append_rejects_malformed_blink_images`, `malformed_page_is_corruption_not_a_panic`, and the Phase 2 recovery integration tests: malformed/corrupted page validation and recovery behavior remain covered.

## Local release smoke

Same-machine Apple M1 smoke used the requested width-1 workload, 16 writers,
working set 100,000, cache 4,096, key/value sizes 16/64, group and queue
settings, disabled sync, two Tokio workers, 1 s warmup/duration, one repetition,
and seed `0x3a042026`.

| Build | Mutations/s | Physical page encode ns/mutation | Errors | Overloads |
| --- | ---: | ---: | ---: | ---: |
| Base | 77,859.3 | 1,413.2 | 0 | 0 |
| Candidate | 78,410.8 | 1,213.3 | 0 | 0 |

This local run met the OCI-entry gate: encoding cost decreased and throughput
was 1.007x base. It is a local smoke result, not production performance
evidence.

## OCI build environment and method

Host: `opc@217.142.246.204`, Oracle Linux Server 9.8, AArch64 Neoverse-N1,
2 CPUs, rustc 1.98.1. The checkout is on an XFS root filesystem mounted at
`/` with 29.4 GiB total and about 17.6 GiB available after the run. Each
revision was built from a separate detached worktree after `cargo clean`,
using `cargo build --release -p dodb-storage --bin phase0-bench`. No
`RUSTFLAGS` override was used; repository `.cargo/config.toml` applies
`-C target-feature=+crc` for this AArch64 target.

Both workloads used 16 writers, 0 readers, working set 100,000, cache 4,096,
key/value sizes 16/64, group limit 64, group bytes 4,194,304, queue 256,
collection delay 0 us, disabled sync, two Tokio workers, 1 s warmup, 2 s
duration, and 3 repetitions. Width 1 used different-leaf-heavy and seed
`0x3a042026`; width 16 used uniform and seed `0x3a032026`.

This is a sync-disabled CPU/engine diagnostic on the mounted root filesystem,
not a durability result and not a benchmark on the unmounted 200 GB volume.

## OCI A/B results

Throughput is mutations/s; the range is min/max across the three repetitions.
Normalized costs are median ns per successful mutation. Page images and WAL
bytes are also normalized per mutation. Page encode per encoded leaf divides
the page encode time by `leaf_encodes`.

### Width 1, primary workload

| Metric | Base | Candidate | Candidate/base |
| --- | ---: | ---: | ---: |
| Throughput median (min–max) | 43,773.6 (43,439.0–45,727.2) | 45,732.0 (45,338.8–47,716.9) | 1.0447x |
| Physical page encode ns/mutation | 2,147.2 | 1,684.1 | 0.7843x |
| Page encode ns/encoded leaf | 2,147.2 | 1,684.1 | 0.7843x |
| Physical execution ns/mutation | 4,515.0 | 4,045.4 | 0.8960x |
| Physical mutation ns/mutation | 1,969.9 | 1,953.4 | 0.9916x |
| Leaf-load clone ns/mutation | 660.2 | 653.9 | 0.9905x |
| WAL append ns/mutation | 4,877.2 | 4,571.6 | 0.9373x |
| WAL group encode ns/mutation | 2,815.7 | 2,591.8 | 0.9205x |
| Planning ns/mutation | 2,618.4 | 2,663.2 | 1.0171x |
| Catalog construction ns/mutation | 1,614.2 | 1,601.4 | 0.9921x |
| Publication ns/mutation | 1,368.7 | 1,332.5 | 0.9736x |
| State install ns/mutation | 574.3 | 567.8 | 0.9887x |
| Page images/mutation | 1.000 | 1.000 | 1.0000x |
| WAL bytes/mutation | 4,224 | 4,224 | 1.0000x |
| Errors / overloads (all repetitions) | 0 / 0 | 0 / 0 | — |

### Width 16, regression control

| Metric | Base | Candidate | Candidate/base |
| --- | ---: | ---: | ---: |
| Throughput median (min–max) | 40,208.6 (39,644.5–41,096.8) | 39,298.2 (39,120.3–40,312.2) | 0.9774x |
| Physical page encode ns/mutation | 3,006.6 | 2,323.2 | 0.7727x |
| Page encode ns/encoded leaf | 3,009.9 | 2,326.3 | 0.7729x |
| Physical execution ns/mutation | 6,744.9 | 6,164.7 | 0.9140x |
| Physical mutation ns/mutation | 3,516.3 | 3,604.9 | 1.0252x |
| Leaf-load clone ns/mutation | 1,334.6 | 1,386.4 | 1.0388x |
| WAL append ns/mutation | 2,557.8 | 3,240.2 | 1.2668x |
| WAL group encode ns/mutation | 1,645.2 | 2,341.7 | 1.4234x |
| Planning ns/mutation | 3,858.5 | 3,933.1 | 1.0193x |
| Catalog construction ns/mutation | 1,679.0 | 1,808.0 | 1.0768x |
| Publication ns/mutation | 1,389.1 | 1,453.0 | 1.0459x |
| State install ns/mutation | 989.8 | 1,036.4 | 1.0471x |
| Errors / overloads (all repetitions) | 0 / 0 | 0 / 0 | — |

The width-1 normalized costs have no increase of 15% or more. In the
width-16 control, WAL append rose 26.7% and WAL group encode rose 42.3%.
These are measured short-run increases; the three repetitions are retained in
the raw files. No additional optimization or attribution instrumentation was
added.

## Classification and remaining bottleneck

This meets the structural/useful success criteria: page encode is 0.784x
base, width-1 throughput is above base, and width-16 throughput is 0.977x
base (above the 0.95x gate). It does not meet strong success because width-1
throughput is below 1.05x. There were no correctness failures, errors, or
overloads.

WAL append remains the largest named normalized cost in the width-1 metrics
at 4,571.6 ns/mutation; WAL group encoding is 2,591.8 ns/mutation. These
metrics are diagnostic and include overlapping phases as defined by the
existing harness.

## Binary and artifact checksums

OCI release binary SHA256:

- Base: `f1d8252ab249d94171412a3e989d5306ca541d05fac75a4ae783d01c58355e42`
- Candidate: `f8d50a72bceb268729e19f2b2963459d0e0c7d06b551d00ff66d93b100c5ea8e`

Committed raw artifacts:

| File | SHA256 |
| --- | --- |
| `planned-base-width1.jsonl` | `4ce1f2afbf2638757e8bff1f909b15402a3c9516d7b041cf708b6d5918135c86` |
| `planned-candidate-width1.jsonl` | `debd868dc31ca9fed77584aed81652aa48b7a1da431d2d1b51feead12a18a68f` |
| `planned-base-width16.jsonl` | `f74dc03e69fb131ba0e8f810be1aae5b1cc51aa8e3a3eecf9c60daffba14b3b8` |
| `planned-candidate-width16.jsonl` | `130fba96545b554d7abff3d9ceabd6ecad9ec386e6305fc86296d2d2a7d1e2ca` |

Repository artifacts are under
`docs/experiments/results/oci-a1-2ocpu-12g-200g/page-encode-validation/`.
The remote backup is
`/home/opc/dodb-oci-artifacts-page-encode-validation-408aaed/`; all four
remote and repository artifact SHA256 values match.
