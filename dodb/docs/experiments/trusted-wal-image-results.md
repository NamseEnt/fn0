# Trusted Internal WAL Image Results

## Motivation

The instrumented OCI width-1 baseline spent 277.551 ms per run, 48.34% of
WAL group encoding, decoding and validating page images that the planned Blink
engine had just encoded. WAL group encoding took 574.123 ms per run, or
14.981 µs per mutation, while median throughput was 19,141.38 mutations/s.

## Validation Boundary

`WalLog::append_group()` retains strict full page-image validation for
arbitrary callers. WAL scanning, recovery, and replay keep their strict
validation. Calls with a fault injector use the strict fault-injectable
implementation and preserve hook order and failure behavior.

Only the planned Blink release path uses the crate-private trusted entrypoint.
Its commits contain page and superblock images returned moments earlier by
this process's `encode_blink_page()` and `encode_blink_superblock()` calls.
The trusted path is not for caller-provided image bytes.

## Design

### Strict public path

`append_group()` delegates to the shared private append implementation with
`PageImageValidationMode::Strict`. This still performs page LSN validation and
full format-specific page-image validation before encoding each payload.

### Trusted internal Blink path

`append_group_trusted_internal()` is crate-private. It shares the same group
encoder and append implementation while selecting `TrustedInternal`. It keeps
commit, batch, LSN, page-count, length, frame, CRC, digest, and group-structure
checks. It also keeps `validate_page_image_lsn()`.

In release builds it skips only the redundant call to
`validate_page_image()`. In debug builds it performs that full validation and
records the validation attribution as before.

### Fault-injection fallback

If the trusted entrypoint receives an injector, it delegates to public
`append_group()`. The existing strict fault path and hook order remain
unchanged, including `before_wal_append`, `after_group_records_written`,
`before_wal_sync`, `during_wal_sync`, and `after_wal_sync`.

### Debug invariant checking

Debug builds continue to decode and validate all trusted images. Release
benchmarks show zero page-image validation count and time on the planned Blink
trusted path.

## Correctness

The public Blink WAL test rejects a bad page checksum, a mismatched page ID,
and a malformed Blink body with a recomputed valid page checksum. A trusted
call with a fault injector still rejects a bad checksum. Fault-injector calls
through the trusted and public entrypoints produce identical reports, bytes,
and hook sequences.

Valid Blink page and superblock images from the internal encoders produce
byte-identical WAL output through strict and trusted paths. The append reports,
next LSN, and next batch ID also match. Debug validation counts are two for
both paths in that test.

Local gates passed:

- `cargo fmt --all -- --check`
- `cargo test -p dodb-core`
- `cargo test -p dodb-storage` (102 unit tests; recovery child 8/8)
- `cargo test -p dodb-storage --bin phase0-bench` (10 tests)
- `cargo test --workspace` (recovery child 8/8)
- `git diff --check`

The WAL tests retain coverage for short writes, torn tails, reopen, scan, and
recovery. No WAL format or recovery code changed.

## OCI Method

The OCI A1 run used the required SSH identity with fingerprint
`SHA256:i9NJ3mGGroLtw6CjNS8XJCfT4OjDlfp384E0cOk71ro`. The repository was clean
at implementation commit `47287bdb55ea3920f6449581bddfcd81db1fbc10` before
the run. The release binary ran the same instrumented width-1 workload as the
attribution baseline: planned Blink, 16 writers, width 1,
`different-leaf-heavy`, working set 100,000, cache 4,096, key 16 bytes, value
64 bytes, group limit 64, group byte limit 4,194,304, queue 256, zero
collection delay, sync disabled, two Tokio workers, 1-second warmup,
2-second duration, three repetitions, seed `0x3a042026`.

`/home/opc/dodb` is on the 30 GB XFS root filesystem. The 200 GB block device
is not mounted as the benchmark filesystem. These results are sync-disabled
CPU and engine diagnostics, not durability measurements or measurements on
the 200 GB volume.

## Results

The table compares the specified instrumented baseline medians with medians
from the three trusted-path repetitions. Timings use each repetition's
successful mutations, page images, and logical groups for normalization.

| metric | before | after | change |
|---|---:|---:|---:|
| throughput (mut/s) | 19,141.38 | 22,747.81 | +18.84% |
| WAL group encode (ms/run) | 574.123 | 318.143 | -44.59% |
| WAL encode (µs/mutation) | 14.981 | 6.986 | -53.37% |
| WAL encode (µs/page image) | 7.491 | 3.493 | -53.37% |
| WAL encode (µs/group) | 141.549 | 66.585 | -52.96% |
| full page-image validation (ms/run) | 277.551 | 0.000 | -100.00% |
| page LSN validation (ms/run) | 3.609 | 3.904 | +8.17% |
| page LSN validation (µs/mutation) | 0.094 | 0.086 | -8.97% |

The median absolute group-encode time is the median of per-run totals. The
normalized values are medians of each run's per-mutation, per-page, or group
ratio, matching the fixed-duration attribution method.

All three records have the exact implementation SHA, `planned-blink`,
`sync_mode=disabled`, zero errors, zero overloads, zero full-state clones,
zero full page-image validations, and zero full page-image validation time.
The median successful mutation count is 45,543 and the median page-image
count is 91,086.

## Normalized WAL Encoding Cost

The group encoder fell from 574.123 ms to 318.143 ms per run. Its normalized
cost fell from 14.981 to 6.986 µs per mutation, 7.491 to 3.493 µs per page
image, and 141.549 to 66.585 µs per logical group. The normalized costs
account for the higher mutation and group counts in the fixed-duration runs.

## Updated Attribution

These are medians of the three after-run category totals. Percentages divide
by median WAL group encode time, and residual is calculated per run before
taking the median.

| component | total ms | % encode |
|---|---:|---:|
| Page LSN validation | 3.904 | 1.23% |
| Full page-image validation | 0.000 | 0.00% |
| Page-image materialization | 24.609 | 7.74% |
| Digest-input copy | 14.144 | 4.45% |
| Page payload CRC32C | 58.404 | 18.36% |
| Page header CRC32C | 4.586 | 1.44% |
| Page frame materialization | 17.079 | 5.37% |
| Page frame append | 62.341 | 19.60% |
| Commit digest CRC32C | 55.197 | 17.35% |
| Commit payload CRC32C | 2.019 | 0.63% |
| Commit header CRC32C | 2.249 | 0.71% |
| Commit frame materialization | 4.644 | 1.46% |
| Commit frame append | 2.031 | 0.64% |
| Residual | 66.909 | 21.03% |

LSN validation remains enabled. Its absolute time rose slightly as the run
processed more mutations, while its normalized cost per mutation fell.

## Interpretation

**Strong success.** Full validation fell below 10% of baseline, group encode
fell below 65% of baseline, and throughput exceeded the 8% improvement gate.
The observed values meet all three strong-success thresholds.

The release trusted path removed the measured decode and canonical-check work
without changing WAL bytes or boundary behavior. The result is a CPU/engine
diagnostic under disabled sync and the mounted root filesystem conditions
described above.

## Next Bottleneck

Among measured WAL encoding categories, page frame append is now largest at
62.341 ms, followed by page payload CRC at 58.404 ms and commit digest CRC at
55.197 ms. Residual is 66.909 ms (21.03%) and remains unattributed. At the
top-level timing scope, planning is now the largest component at 500.810 ms,
slightly above WAL append at 478.387 ms. Aggregated CRC categories total
122.454 ms (38.49% of encode), just above the page materialization, digest
copy, frame materialization, and frame append set at 118.173 ms (37.14%).
The WAL-scope next candidate is therefore CRC pass and digest strategy; the
close margin and the new top-level planning cost call for measurement before
selecting a broader optimization.

## Arena Assessment

**Not recommend.** Page frame append, CRC work, and commit digest CRC dominate
the named WAL encoding categories; the result does not isolate small
temporary-object allocation as the remaining cost. The next WAL candidate is
CRC pass and digest strategy. A direct pre-sized group buffer with ownership
transfer and reuse remains a close candidate if later measurements show the
copy/materialization set dominates. Do not introduce an arena based on this
evidence.

## Experiment Status

Implementation commit (`TRUSTED_WAL_SHA`):
`47287bdb55ea3920f6449581bddfcd81db1fbc10`.

WAL format is unchanged. Public WAL append validation remains strict.
Fault-injection and recovery paths remain strict. Only the release fast path
for internally generated Blink WAL images skips redundant full page decoding.

The B-link planned engine and batching behavior are unchanged by this task.
The existing Phase 4 worker-pool implementation remains complete with its
previous OCI overlap and throughput concerns; Phase 5 has not started.

## Artifacts

- OCI raw artifact:
  [`planned-trusted-wal-images-width1.jsonl`](results/oci-a1-2ocpu-12g-200g/trusted-wal-images/planned-trusted-wal-images-width1.jsonl)
- Raw artifact SHA256:
  `d33cae7dce2bcdb962b7c19d999a4bb89937c67ee143260dbf21cf52b3f3b90f`
- OCI backup after moving the raw file outside the worktree:
  `/home/opc/dodb-oci-artifacts-trusted-wal-47287bd/`
