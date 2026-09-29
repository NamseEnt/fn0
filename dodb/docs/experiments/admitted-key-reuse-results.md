# Admitted Blink Mutation Key Reuse

## Result

The planned-path optimization was implemented and measured. The initial OCI
reject is retained below as historical evidence, but its candidate binary came
from the wrong checkout and cannot be attributed to the measured source. A
clean forensic rebuild and rerun corrected the result to **structural/useful
success**. Both implementation commits remain reverted; this forensic task did
not reapply candidate source.

## Duplicate-work audit and design

Before the change, planned admission called `validate_request_values()`, which
encoded and validated each mutation key. `LogicalOverlay::accept()` encoded it
again for the overlay's owned `BTreeMap` key. `plan_batch()` encoded and
validated it a third time before routing and dependency construction. The
planner then copied the key into `PlannedMutation.encoded_key` and its owned
`last_key_writer` map. Admission also cloned each accepted `TransactionRequest`,
after which the planner cloned each mutation into `PlannedMutation`.

The final measured candidate SHA was
`1eaabbce8ec29948fe886008dd3e4548d1a8d748`; its initial implementation commit
was `f085e7b8d6e28761e31b55a4b4c0539c968c649f`. Changes were limited to
`crates/dodb-storage/src/blink/mod.rs`.

Planned admission used `validate_and_encode_mutation_keys()` to encode and
validate mutation keys once, preserving canonical and maximum encoded-key
validation and the Put value-size check. `AdmittedTransaction<'a>` borrowed the
original request and held the prepared keys in mutation order. Admission
passed those keys to `LogicalOverlay::accept_preencoded()`, which checked the
mutation/key count and copied each key into the overlay's owned map.
`plan_batch()` checked the count and reused the prepared bytes for routing,
dependency construction, and `PlannedMutation` construction. It no longer
encoded or revalidated admitted mutation keys. The accepted request clone was
removed.

The required `PlannedMutation.mutation` clone and owned planner/overlay keys
remained. No shared-key representation, overlay representation change, or
mutation ownership change was made. Follow-up commit
`1eaabbce8ec29948fe886008dd3e4548d1a8d748` restored the serial
`validate_request_values()` loop unchanged so the preparation vector was only
created on the planned path. `TransactionRequest::validate()` and its duplicate
mutation/condition and empty-mutation semantics were not changed. Condition-key
encoding in `LogicalOverlay::observed_state()` and planner condition dependency
construction also remained unchanged.

## Correctness

Focused tests covered prepared-key count and byte identity, oversized key and
value rejection, unchanged route IDs, same-key and same-page dependencies,
structural-route dependencies, multi-mutation `mutated_key_set` contents,
provisional revisions, and split/reroute behavior. Existing planned admission
tests verified that later conditions see earlier accepted writes and conflicted
requests do not leak writes. Existing core malformed-encoding tests and the
preserved canonical validation cover malformed-key rejection.

All requested gates passed on the final implementation source before revert:

- `cargo fmt --all -- --check`
- `cargo test -p dodb-core` (6 passed)
- `cargo test -p dodb-storage` (115 unit tests and 8 recovery tests passed)
- `cargo test -p dodb-storage --bin phase0-bench` (10 passed)
- `cargo test --workspace` (all tests passed, including testkit and Phase 1
  integration tests)
- `git diff --check`

## Local Apple M1 smoke

Base and final candidate used the same Apple M1 release build and the requested
16 writers, width 1, `different-leaf-heavy`, 100,000 key working set, 4,096
cache, 16-byte key, 64-byte value, group limit 64, 4 MiB group limit, queue
256, zero delay, disabled sync, two Tokio workers, one-second warmup and
duration, one repetition, and seed `0x3a042026`. Both had zero errors and
overloads.

| Metric | Base | Candidate | Candidate/base |
| --- | ---: | ---: | ---: |
| Throughput (mutations/s) | 68,423 | 75,947 | 1.110x |
| Logical admission (ns/mutation) | 574.2 | 398.2 | 0.693x |
| Planning (ns/mutation) | 1,295.0 | 1,055.3 | 0.815x |
| Route (ns/mutation) | 566.2 | 509.6 | 0.900x |
| Front-end total (ns/mutation) | 1,869.2 | 1,453.5 | 0.778x |
| Front-end non-route (ns/mutation) | 1,302.9 | 944.0 | 0.725x |

The candidate passed the local gate. These single-repetition local measurements
are smoke evidence only.

## OCI build and workload

The same-host A/B used `opc@217.142.246.204`, Oracle Linux 9.8 AArch64,
`rustc 1.98.1`, and two logical CPUs. The checkout was clean before creating
detached worktrees. The root XFS filesystem was 30 GiB with 18 GiB free.

Base `ac181f5900ed46a83e616f3d28bcb85bac88ca48` and candidate
`1eaabbce8ec29948fe886008dd3e4548d1a8d748` were built in clean detached
worktrees with separate targets using:

```bash
cargo build --locked --release -p dodb-storage --bin phase0-bench
```

The repository `.cargo/config.toml` supplied `-C target-feature=+crc`; no
`RUSTFLAGS` override was used.

| Binary | SHA256 |
| --- | --- |
| Base | `8f402d6b3660e8b16d8899b49830bc4b6bb93d70398aa66e56cdc3e7fff0baa3` |
| Candidate | `3899baaff31950565f21eed0d832b29fe2f1df268299ac7a740aba57619aa53f` |

Width 1 used 16 writers, width 1, `different-leaf-heavy`, working set 100,000,
cache 4,096, key size 16, value size 64, group limit 64, group bytes
4,194,304, queue 256, delay 0, disabled sync, two Tokio workers, one-second
warmup, two-second duration, three repetitions, and seed `0x3a042026`. Width
16 used the same settings with width 16, `uniform`, and seed `0x3a032026`.
All repetitions had zero errors and overloads.

The following component values are medians of per-repetition values normalized
by successful mutations. Throughput includes median and min–max. Front-end
values use `logical_admission_nanos`, `planning_nanos`, and
`planner_route_nanos`.

### Width-1 primary

| Metric | Base | Candidate | Candidate/base |
| --- | ---: | ---: | ---: |
| Throughput median (min–max), mutations/s | 47,805 (46,063–48,133) | 42,971 (41,960–43,446) | 0.899x |
| Logical admission (ns/mutation) | 931.0 | 974.9 | 1.047x |
| Planning (ns/mutation) | 2,315.0 | 2,630.5 | 1.136x |
| Route (ns/mutation) | 1,106.5 | 1,138.9 | 1.029x |
| Non-route planning (ns/mutation) | 1,208.4 | 1,491.6 | 1.234x |
| Front-end total (ns/mutation) | 3,246.0 | 3,598.5 | 1.109x |
| Front-end non-route (ns/mutation) | 2,175.8 | 2,508.9 | 1.153x |
| Physical execution (ns/mutation) | 4,023.7 | 4,851.6 | 1.206x |
| Page encode (ns/mutation) | 1,699.1 | 2,486.9 | 1.464x |
| WAL append (ns/mutation) | 4,349.3 | 5,097.6 | 1.172x |
| WAL group encode (ns/mutation) | 2,212.1 | 2,973.0 | 1.344x |

### Width-16 control

| Metric | Base | Candidate | Candidate/base |
| --- | ---: | ---: | ---: |
| Throughput median (min–max), mutations/s | 42,431 (41,904–42,936) | 36,820 (36,802–38,470) | 0.868x |
| Logical admission (ns/mutation) | 1,009.5 | 1,052.9 | 1.043x |
| Planning (ns/mutation) | 3,732.8 | 4,010.6 | 1.074x |
| Route (ns/mutation) | 1,625.9 | 1,638.0 | 1.007x |
| Front-end total (ns/mutation) | 4,744.0 | 5,087.9 | 1.073x |
| Front-end non-route (ns/mutation) | 3,160.1 | 3,360.5 | 1.063x |

The post-candidate base repeat produced 46,850 mutations/s median at width 1
and 42,076 at width 16, with zero errors and overloads. The primary base and
repeat were consistent. Candidate-to-repeat throughput ratios were 0.917x at
width 1 and 0.875x at width 16. Repeat raw files are retained alongside the
four required artifacts.

## Historical classification and remaining duplication

The original **Reject** used an OCI binary with invalid candidate provenance.
Its width-1 front-end non-route ratio was 1.153x and throughput
ratio was 0.899x, beyond the reject thresholds. Width-16 throughput was 0.868x,
below its 0.95x floor. Physical execution, page encoding, WAL append, and WAL
group encoding each increased by at least 15% in the primary comparison. The
base repeat was consistent with the primary base. Those values remain
historical raw measurements, not a valid candidate comparison. The corrected
result is in the forensic section below. No WAL changes or follow-up WAL work
were made.

Remaining duplicate work included condition-key encoding in overlay observation
and planner condition-dependency construction, plus necessary owned-key copies
for the overlay, `PlannedMutation`, `last_key_writer`, and `mutated_key_set`.
This task did not change those paths or representations.

## Artifacts and rollback

| Artifact | SHA256 |
| --- | --- |
| [`planned-base-width1.jsonl`](results/oci-a1-2ocpu-12g-200g/admitted-key-reuse/planned-base-width1.jsonl) | `3dc83495e56c2cf6206f0450bf5c9d978cda3abe8a826d317790f19f745d72b5` |
| [`planned-candidate-width1.jsonl`](results/oci-a1-2ocpu-12g-200g/admitted-key-reuse/planned-candidate-width1.jsonl) | `4c4f298252d224b6ec4f9ca08b76507f3850380cd29716669a29ba6742c3c1b5` |
| [`planned-base-width16.jsonl`](results/oci-a1-2ocpu-12g-200g/admitted-key-reuse/planned-base-width16.jsonl) | `7b7e5ed26c57065b2c9152412caa644bc6cf66c5b044c543bc38f9b0910a436f` |
| [`planned-candidate-width16.jsonl`](results/oci-a1-2ocpu-12g-200g/admitted-key-reuse/planned-candidate-width16.jsonl) | `439e8d375b814e4c18161c4b761ac197b8ced4a7151c8c7183420c3d4b999e41` |
| [`planned-base-width1-repeat.jsonl`](results/oci-a1-2ocpu-12g-200g/admitted-key-reuse/planned-base-width1-repeat.jsonl) | `092ad02f63b61be0e6e3924e3f26bba4385b1389bee88860a738a0f3046898c4` |
| [`planned-base-width16-repeat.jsonl`](results/oci-a1-2ocpu-12g-200g/admitted-key-reuse/planned-base-width16-repeat.jsonl) | `fdb9d234de5e55a532105b502e4ddaea9b7094ee7b27bdbdc1e35a38703fa5fd` |

The artifacts are backed up at
`/home/opc/dodb-oci-artifacts-admitted-key-reuse-1eaabbc/`; remote and local
SHA256 values matched.

Implementation commits `f085e7b8d6e28761e31b55a4b4c0539c968c649f` and
`1eaabbce8ec29948fe886008dd3e4548d1a8d748` were reverted normally as
`4ac65c33437fdd224b947c3ddfd9a6f31613ee25` and
`6fb9d7c2e0a475de110c6a875fa1cf17e6d92b5c`, respectively. No history rewrite
was used.

## Build provenance anomaly and forensic rerun

### Why the original candidate SHA was invalid

The original candidate SHA256 `3899baaff31950565f21eed0d832b29fe2f1df268299ac7a740aba57619aa53f`
exactly matches the generic/default AArch64 binary recorded for source
`9e82d9da4db9c1818ce725addf112281020dc6a7` in
[`current-crc-reattribution-results.md`](current-crc-reattribution-results.md).
That older source had no `.cargo/config.toml`; its repository `+crc` build had
a different SHA256, `f1d8252ab249d94171412a3e989d5306ca541d05fac75a4ae783d01c58355e42`.

The root cause was a working-directory error in the previous candidate build
command. It checked out the candidate with `git -C
/home/opc/dodb-admitted-key-reuse-candidate checkout ...`, but then ran Cargo
without changing directory from `/home/opc/dodb`. That checkout was still at
`b309b0fa9f187f7b8f14b1640bb55b9b892926d9`, whose parent is `9e82d9d`; it had
no `.cargo/config.toml`. The target's saved
`bin-phase0-bench.json` fingerprint records `"rustflags":[]`. The earlier
verbose build log was not retained, but the command cwd, checkout lineage,
missing config, fingerprint, and exact generic binary SHA identify the error.
The `b309b0f` commit changes only experiment documentation and raw results
relative to `9e82d9d`; Blink production source and Cargo build configuration are
identical between those commits.

### Fresh worktrees, build flags, and binary identity

New detached worktrees were created at `/home/opc/dodb-forensic-base` and
`/home/opc/dodb-forensic-candidate`. Their clean HEADs were respectively
`ac181f5900ed46a83e616f3d28bcb85bac88ca48` and
`1eaabbce8ec29948fe886008dd3e4548d1a8d748`. The candidate source proof records
`encode_leaf_record_into_validated()` after layout validation,
`RouteHint` without `encoded_key`, `PhysicalTransactionPlan` without
`encoded_keys`, and `AdmittedTransaction<'a>` borrowing its request while
holding `encoded_mutation_keys`.

All builds used brand-new target directories and `env -u RUSTFLAGS`:

```bash
env -u RUSTFLAGS CARGO_TARGET_DIR=<fresh-target> \
  cargo build --locked --release -p dodb-storage --bin phase0-bench -vv
```

The saved verbose invocations show `-C target-feature=+crc` for both
`crc32c` and `dodb_storage` in base and both candidate builds. The repository
`.cargo/config.toml` supplied the flag. Candidate builds A and B used distinct
targets and produced byte-identical binaries.

| Binary | SHA256 | Size |
| --- | --- | ---: |
| Base (`/home/opc/dodb-forensic-binaries/base-phase0-bench`) | `8f402d6b3660e8b16d8899b49830bc4b6bb93d70398aa66e56cdc3e7fff0baa3` | 2,427,072 |
| Candidate A (`/home/opc/dodb-forensic-binaries/candidate-a-phase0-bench`) | `79122436e330be122059f8a4bf5908406e55b74596bfab2ee3cf745dd17fb6cb` | 2,427,600 |
| Candidate B (`/home/opc/dodb-forensic-binaries/candidate-b-phase0-bench`) | `79122436e330be122059f8a4bf5908406e55b74596bfab2ee3cf745dd17fb6cb` | 2,427,600 |

Candidate A/B hashes match each other and differ from both base and the old
`3899ba...` generic binary. Read-only executable copies were hashed again
after copying. `plan_batch` is present in both disassemblies: its symbol size
is `0x20fc` bytes in base and `0x20c0` bytes in candidate. The archived
disassembly diff and its SHA256 provide additional codegen evidence. The
validated leaf-record writer and its call site are present in the candidate
source proof; it is inlined in the final binary.

### Corrected OCI results

The verified immutable binary copies were used for the rerun. Each width used
three measurements with paired seeds: width 1 seeds `973348902`–`973348904`
(`0x3a042026` plus repetition), and width 16 seeds `973283366`–`973283368`
(`0x3a032026` plus repetition). The first pass interleaved base/candidate per
repetition and is retained separately. The canonical JSONL set was then run
with `--repetitions 3`, preserving harness repetition indices 0–2. All
repetitions had zero errors and overloads.

| Width-1 metric | Base | Candidate | Candidate/base |
| --- | ---: | ---: | ---: |
| Throughput median (min–max), mutations/s | 47,609 (46,929–49,205) | 48,368 (47,150–49,410) | 1.016x |
| Logical admission (ns/mutation) | 942.2 | 653.6 | 0.694x |
| Planning | 2,302.7 | 2,254.2 | 0.979x |
| Route | 1,098.2 | 1,053.4 | 0.959x |
| Non-route planning | 1,204.5 | 1,163.7 | 0.966x |
| Front-end total | 3,224.3 | 2,914.0 | 0.904x |
| Front-end non-route | 2,146.2 | 1,815.2 | 0.846x |
| Physical execution | 3,990.1 | 3,994.7 | 1.001x |
| Page encode | 1,690.2 | 1,706.2 | 1.010x |
| WAL append | 4,279.0 | 4,401.3 | 1.029x |
| WAL group encode | 2,283.2 | 2,319.4 | 1.016x |

| Width-16 metric | Base | Candidate | Candidate/base |
| --- | ---: | ---: | ---: |
| Throughput median (min–max), mutations/s | 42,144 (40,793–43,218) | 44,347 (42,898–44,423) | 1.052x |
| Logical admission (ns/mutation) | 971.6 | 583.9 | 0.601x |
| Planning | 3,729.1 | 3,607.7 | 0.968x |
| Route | 1,614.6 | 1,632.9 | 1.011x |
| Front-end total | 4,693.6 | 4,205.8 | 0.896x |
| Front-end non-route | 3,079.0 | 2,584.8 | 0.839x |

There were no unrelated timer regressions of 15% or more. Physical execution,
page encode, WAL append, and WAL group encode were each within 3% of base in the
width-1 primary.

**Corrected classification: Structural/useful success.** Width-1 front-end
non-route was `0.846x`, front-end total `0.904x`, throughput `1.016x`, and
width-16 throughput `1.052x`. This meets all structural/useful thresholds but
does not meet the strong thresholds (`0.80x` front-end non-route and `1.03x`
throughput). The candidate remains reverted as requested; this finding does
not reapply it to production HEAD.

Forensic logs, source/binary proofs, interleaved and canonical raw JSONL, and a
SHA256 manifest are in
[`admitted-key-reuse-forensic`](results/oci-a1-2ocpu-12g-200g/admitted-key-reuse-forensic/).
The manifest SHA256 is
`e60f79a888794997a74cabaffde62cd84f5305bbb94bdf8ce1d40c27b0156032`.
The OCI backup is
`/home/opc/dodb-oci-artifacts-admitted-key-reuse-forensic-1eaabbc/`; local and
remote checksums match.

### Final disposition

The corrected forensic rerun classified the candidate as a
structural/useful success.

The two earlier reverts were subsequently reverted, restoring the
validated candidate source to the experimental branch. The restore commits
are `2797d9f` (`Reapply "storage: reuse admitted Blink mutation keys"`) and
`bda613f` (`Reapply "storage: keep serial Blink validation allocation stable"`).

The restored production source is equivalent to candidate
`1eaabbce8ec29948fe886008dd3e4548d1a8d748`.
