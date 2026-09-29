# Allocation-Free Encoded-Key Validation Results

## Motivation

The exact-layout fit check removed repeated page-body serialization, but each
canonical encoded-key check still called `DocumentKey::decode()`. Decoding
materialized owned primary and sort key components even though callers only
needed to know whether the encoded bytes were canonical. This validation also
runs during actual Blink page encoding.

## Existing Allocation

`DocumentKey::decode()` called `decode_component()` twice. Each call built a
`Vec<u8>`, after which decoding constructed `PrimaryKey`, `SortKey`, and
`DocumentKey`. Blink's `validate_encoded_key()` discarded that decoded value.

## Design

### Component scanner

`skip_component()` walks a component by offset and returns the offset directly
after its terminator. It recognizes ordinary nonzero bytes, `00 FF` zero
escapes, and the `00 00` terminator without creating decoded bytes.

### Exact codec error semantics

The scanner preserves `decode_component()`'s `MissingComponent`,
`TruncatedEscape`, and `InvalidEscape` variants, offsets, and invalid escape
byte. `DocumentKey::validate_encoded()` scans exactly two components and
preserves the decoder's `TrailingBytes` offset.

### Allocation-free validation

The public `DocumentKey::validate_encoded()` method uses only slice reads,
integer offsets, and `Result` values. `skip_component()` and the validator
contain no `Vec`, `String`, `Box`, `collect`, or `to_vec`. The implementation is
allocation-free by construction. Blink retains its maximum encoded-key size
check and maps scanner errors through the existing storage error path. All
existing validation call sites use this scanner; duplicate validation calls
remain in place.

No page format, split policy, batch semantics, WAL semantics, or encoder format
changed. Only canonical encoded-key validation changed from allocating decode
to allocation-free scanning. Page encoder buffers and record construction
remain unchanged.

## Correctness

`dodb-core` retains the existing round-trip, ordering, malformed-input, and
encoded-length tests. New deterministic malformed and boundary cases include
all requested inputs, escaped zero cases, an incomplete second component,
trailing bytes, and invalid escapes in each component. Each case compares the
exact `Result` from `validate_encoded()` with `decode().map(|_| ())`.

The valid-key property generates arbitrary binary primary and sort keys,
encodes them canonically, and verifies validation returns `Ok(())` alongside
the existing decode round trip. The arbitrary-byte property compares the
validator and decoder results exactly, including error variants and fields.
Both property tests passed.

Passed locally:

- `cargo fmt --all -- --check`
- `cargo test -p dodb-core` (6 tests)
- `cargo test -p dodb-storage` (92 storage unit tests, 10 benchmark tests, 8 recovery integration tests)
- `cargo test -p dodb-storage --bin phase0-bench` (10 tests)
- `cargo test --workspace` (all unit, integration, and doc tests)
- `git diff --check`
- Local planned-blink width-1 smoke: 16 writers, width 1, different-leaf-heavy, working set/cache 4,096, sync disabled, zero delay, 1 s warmup and duration; errors 0, overloads 0, full-state clones 0, state-clone nanoseconds 0.

Storage coverage includes fit-check/encoder equivalence, leaf and internal
splits, randomized transaction differential, WAL recovery, catalog COW,
parallel fallback, and pinned generations.

## OCI Method

The benchmark binary was built in release mode at implementation commit
`5c574558e644690c9ac0632b98198de9b9b56a09` on the OCI A1 host. Width 1 used
16 writers, transaction width 1, different-leaf-heavy distribution, working
set 100,000, cache 4,096, 16-byte keys, 64-byte values, group limit 64, group
byte limit 4 MiB, queue 256, zero collection delay, sync disabled, two Tokio
workers, 1 s warmup, 2 s duration, three repetitions, and seed `0x3a042026`.
Width 16 used the same settings with transaction width 16, uniform
distribution, and seed `0x3a032026`.

These are sync-disabled CPU and engine diagnostics. `/home/opc/dodb` resides on
the 30 GB XFS root filesystem; the 200 GB block device is not mounted as the
benchmark filesystem. They are not durability measurements.

## Width-1 Result

The before values are the medians from the preceding fit-check artifact. After
values are medians of the three key-validation repetitions. Time values are
milliseconds.

| metric | before | after | change |
|---|---:|---:|---:|
| throughput | 15,746.54 mut/s | 17,683.23 mut/s | +12.30% |
| processing | 1,900.878 ms | 1,891.908 ms | -0.47% |
| physical execution | 542.277 ms | 490.236 ms | -9.60% |
| physical mutation | 223.468 ms | 214.466 ms | -4.03% |
| mutation residual | 110.602 ms | 86.661 ms | -21.65% |
| leaf load clone | 26.075 ms | 28.995 ms | +11.20% |
| leaf entries clone | 35.750 ms | 42.035 ms | +17.58% |
| leaf install clone | 51.041 ms | 56.775 ms | +11.23% |
| cached refresh | 42.268 ms | 46.775 ms | +10.67% |
| page encode | 201.190 ms | 142.722 ms | -29.06% |
| superblock encode | 32.446 ms | 35.565 ms | +9.61% |
| planning | 354.038 ms | 398.405 ms | +12.53% |
| WAL append | 605.107 ms | 576.668 ms | -4.70% |
| WAL group encode | 491.455 ms | 442.159 ms | -10.03% |

Mutation residual is physical mutation minus the three leaf clone medians,
floored at zero. The medians are calculated per metric from the three runs.

## Mutation Residual Effect

The residual fell from 110.602 ms to 86.661 ms, a 21.65% reduction. This is
below the encoder-focused page-encoding reduction, but it shows a measurable
reduction in the fit/layout mutation path where canonical validation had been
allocating decoded key components.

## Page Encoding Effect

Actual page encoding fell from 201.190 ms to 142.722 ms, a 29.06% reduction.
Canonical key validation also runs inside actual leaf record and page
validation, so the result covers both fit/layout checks and repeated validation
during page encoding. Superblock encoding, which is unrelated to key
validation, moved from 32.446 ms to 35.565 ms (+9.61%).

## Width-16 Check

| metric | before | after | change |
|---|---:|---:|---:|
| throughput | 15,372.02 mut/s | 16,902.53 mut/s | +9.96% |

The result clears the 10% regression threshold by a wide margin; no width-16
regression was observed.

## Interpretation

The combined target cost is mutation residual plus physical page encoding:

```text
before: 110.602 + 201.190 = 311.792 ms
after:   86.661 + 142.722 = 229.383 ms
reduction: 26.43%
```

This is a strong success: target cost is below 75% of baseline and throughput
is more than 5% above baseline. Width-1 throughput increased by 12.30%.

Clone timing variation was below 15% for leaf load (+11.20%) and leaf install
(+11.23%), but leaf entries clone increased 17.58%. Cached refresh increased
10.67%. The physical WAL group-write timer increased 16.85%; WAL append fell
4.70%. Catalog lifecycle timers increased between 10.36% and 13.45%. These
above-threshold clone and WAL write variations are recorded as run variance;
no further benchmark was run. Superblock encoding changed by 9.61%.

## Next Bottleneck

Physical page encoding remains 29.11% of physical execution (`142.722 / 490.236`
ms), above the 25% priority threshold. Page and superblock encoding together
account for 36.37% of physical execution. The next engineering priority is
direct fixed-size Blink page encoding: remove the `BODY_SIZE` temporary vector,
per-entry record vectors, slot vector, and final `body.to_vec()` copy, then
write directly into `[u8; PAGE_SIZE]`. This is a separate experiment and was
not started here.

## Experiment Status

Implementation commit: `5c574558e644690c9ac0632b98198de9b9b56a09`.
Classification: strong success. B-link format, split policy, batching,
transaction semantics, WAL semantics, and encoder representation are
unchanged. Phase 4 status is unchanged; Phase 5 has not started.

## Artifacts

- Width 1: [`planned-key-validation-width1.jsonl`](results/oci-a1-2ocpu-12g-200g/key-validation/planned-key-validation-width1.jsonl), SHA256 `947b134d6efb64df6a01846812baa41ac1d8e8ea1e21998c5bd3b674fe9a2869`
- Width 16: [`planned-key-validation-width16.jsonl`](results/oci-a1-2ocpu-12g-200g/key-validation/planned-key-validation-width16.jsonl), SHA256 `02d1f060c264cc277dc04f1b74812e49c7846c1052e60f41d123ae66b86e2111`

Each artifact contains three records at the implementation SHA with
`engine=planned-blink`, `sync_mode=disabled`, zero errors, zero overloads, zero
full-state clones, and zero state-clone nanoseconds. Local SHA256 values match
the OCI files. OCI copies were moved outside the worktree after copying.
