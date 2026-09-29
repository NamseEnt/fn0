# WAL CRC Combine Experiment Results

## Outcome

The candidate is rejected for performance. Reusing each page payload CRC with
`crc32c_combine` preserves the WAL bytes and digest value, but its combine work
was much more expensive than rescanning the page payload on the local Apple M1.
The local WAL group encode time was 7.03x the base, above the 1.05x stop limit.
The OCI run was therefore skipped.

## Motivation and change

The direct WAL group encoder calculated the page frame payload CRC over
`page_id || page_image`, then scanned those same bytes again to append them to
the commit digest. A page payload is 8 bytes of page ID plus a 4,096-byte image.

The candidate retains the payload CRC and updates the digest with
`crc32c::crc32c_combine(digest, payload_checksum, 4104)`. The combine API
computes the CRC of the concatenation from the CRCs of its parts and the length
of the right part. No WAL format, checksum meaning, page format, recovery
logic, or dependency changed. `WAL_FORMAT_VERSION` remains 2.

In candidate measurements, the historical field
`group_commit_digest_crc_nanos` now measures digest-combine work. The JSON
field remains `wal_group_commit_digest_crc_nanos_total` for artifact
compatibility.

## Correctness evidence

The combine identity was checked against direct CRC of concatenated streams in
2,048 deterministic randomized A/B cases with varied stream lengths, including
empty streams. The page-shaped check covered 1,200 randomized
`8-byte page_id || 4096-byte page_image` payloads and confirmed both payload
CRC and old incremental digest equal the combine result. The multi-page check
covered 1,200 commits with 1–64 random pages each and compared the old
incremental digest, repeated combine digest, and direct CRC of the complete
commit stream.

The existing 300-case randomized direct-WAL/reference equivalence test passed.
It compares the complete encoded WAL byte vectors and append reports, which
covers page payload CRC, page header CRC, commit digest, commit payload CRC,
commit header CRC, LSNs, batch IDs, frame lengths, and all other encoded bytes.
Existing recovery and scanner tests also passed unchanged.

## Local correctness and smoke

Correctness gates passed on the local Apple M1:

- `cargo fmt --all -- --check`
- `cargo test -p dodb-core`
- `cargo test -p dodb-storage`
- `cargo test -p dodb-storage --bin phase0-bench`
- `cargo test --workspace`
- `git diff --check`

The local base binary was built from
`ff16d09f4bfc19d92961cbe91172e4e7ed3731dc`. The candidate binary used the
implementation source tree subsequently committed as
`b874631b3dcca757f19ef950c046beb6ca01992e`. It was built before that commit,
so the harness `git_commit` field in the raw smoke JSONL identifies the base
HEAD; the source bytes are the implementation recorded by the candidate SHA.
The smoke used planned-blink,
16 writers, width 1, different-leaf-heavy, a 100,000 key working set, 4,096
cache capacity, 16-byte keys, 64-byte values, group limit 64, group bytes
4,194,304, queue 256, zero collection delay, disabled sync, two Tokio workers,
one second warmup and duration, one repetition, and seed `0x3a042026`.

| Metric | Base | Combine candidate | Candidate / base |
| --- | ---: | ---: | ---: |
| Throughput (mutations/s) | 57,297 | 49,872 | 0.870x |
| WAL append (ns/mutation) | 5,126 | 11,554 | 2.25x |
| WAL group encode (ns/mutation) | 1,353 | 9,512 | 7.03x |
| WAL group write (ns/mutation) | 2,619 | 1,262 | 0.48x |
| Page payload CRC (ns/mutation) | 281 | 268 | 0.95x |
| Commit digest CRC/combine (ns/mutation) | 257 | 8,570 | 33.4x |
| Page header CRC (ns/mutation) | 23.4 | 21.3 | 0.91x |
| Commit payload CRC (ns/mutation) | 22.1 | 20.8 | 0.94x |
| Commit header CRC (ns/mutation) | 22.7 | 21.7 | 0.96x |
| Page direct output (ns/mutation) | 437 | 328 | 0.75x |
| Residual group encode (ns/mutation) | 264 | 238 | 0.90x |
| Physical execution (ns/mutation) | 2,963 | 2,200 | 0.74x |
| Page encode (ns/mutation) | 1,565 | 1,209 | 0.77x |
| Planning (ns/mutation) | 1,514 | 1,143 | 0.75x |
| Page images per mutation | 1.00 | 1.00 | 1.00x |
| WAL bytes per mutation | 4,224 | 4,224 | 1.00x |
| Errors / overloads | 0 / 0 | 0 / 0 | — |

This is a one-repetition development smoke, not a production performance
result. The combine cost dominates candidate group encoding despite removing
the second payload scan. Remaining WAL encoding cost is primarily the combine
algorithm itself; page and commit frame checksums are much smaller.

## OCI status and classification

The prescribed OCI host and 2 OCPU / 12 GiB / 200 GB setup were not accessed.
The local stop condition was met: candidate group encode exceeded 1.05x base
and throughput was below 0.97x base. No OCI width-1 repetitions, width-16
control, OCI binaries, or remote backup were produced. Accordingly, the
requested width-16 and OCI metrics are not available. No production claim is
made from the local smoke.

Classification: **Reject**. Both `group encode > 1.05x` and `throughput <
0.97x` satisfy the rejection criteria. The correctness and byte-identity
properties pass, but the CRC combine operation is too costly on this target.

## Artifacts and checksums

Local raw JSONL smoke artifacts are stored under
`docs/experiments/results/oci-a1-2ocpu-12g-200g/wal-crc-combine/` with `-local`
suffixes so they are not confused with OCI runs.

| Artifact | SHA256 |
| --- | --- |
| `planned-base-width1-local.jsonl` | `8e34bdc5119f6e405a611937f4f7b5d705a4b26cc09034f0317131f1ee83e5e2` |
| `planned-combine-width1-local.jsonl` | `026983bde7ab754d5e37a96bf3be3d56c212d4ae8a14f81960935002e88d119e` |
| Base release binary | `de1416c893599f8eb17d9b7e435fee1b14d1db98b408086a9dc1d27676fbaa99` |
| Candidate release binary | `e3bcbb7f9413385389182652a43fc811062bffabe4545629752127f6255fd0df` |

There is no OCI artifact backup because no OCI artifacts were created.

## Commits

- Base: `ff16d09f4bfc19d92961cbe91172e4e7ed3731dc`
- Implementation: `b874631b3dcca757f19ef950c046beb6ca01992e`

The rejected implementation was subsequently reverted from the experimental
branch. The result document and raw local artifacts are retained as historical
evidence. Production WAL encoding continues to use the previous incremental
CRC digest path.

Revert commit: `dc7a776`
