# A1: Writer pin lifetime and page reuse

Status: A1 complete as an opt-in implementation and local allocation/recovery diagnostic. Defaults remain unchanged; density and performance conclusions remain open.

Tracking issue: https://github.com/NamseEnt/fn0/issues/126

Implementation plan: [A1-IMPLEMENTATION.md](A1-IMPLEMENTATION.md).

## Scope

The candidate adds the opt-in dodb-storage Cargo feature `blink-writer-pin-reuse`. It releases the writer-owned admission generation pin after admission finishes. For parallel preparation, the candidate checks the conservative external-pin guard before acquiring its own preparation pin. The default build retains the original self-pin behavior. WAL durability, on-disk formats, checkpoint settings, and transaction/revision semantics remain unchanged.

This increment investigates fixed-key overflow churn. It does not implement a memory budget, page eviction, epoch reclamation, physical file shrink, or active-tenant density measurement.

## Deterministic allocation result

The identical planned-path unit diagnostic uses MemoryFile and one fixed key. It writes the initial value, performs 16 overwrites without an external reader, holds an old generation through eight overwrites, then releases that generation and performs another 64 overwrites. The warmup column is after the first eight overwrites. The released column is after 32 post-release overwrites; final is after another 32. These are correctness/allocation diagnostics, not performance benchmarks.

| Value bytes | Feature | Initial high-water | Warmup | External pin held | Released | Final |
| ---: | --- | ---: | ---: | ---: | ---: | ---: |
| 512 | Disabled | 2 | 2 | 2 | 2 | 2 |
| 512 | Enabled | 2 | 2 | 2 | 2 | 2 |
| 513 | Disabled | 3 | 11 | 27 | 59 | 91 |
| 513 | Enabled | 3 | 4 | 12 | 12 | 12 |
| 4096 | Disabled | 4 | 20 | 52 | 116 | 180 |
| 4096 | Enabled | 4 | 6 | 22 | 22 | 22 |

The 512-byte inline control does not require overflow allocation. With larger values, the baseline high-water continues to grow. The candidate reaches a plateau without an external pin, grows conservatively while a reader pins an old generation, and stabilizes after release. The old reader retains its original value in both builds. High-water is a page-ID bound, not an RSS, retained-heap, allocated-disk, or application-write measurement.

All counts below exclude the initial insert and cover all 88 overwrites. Fresh allocations are high-water increases. Reuse counts match the new overflow chain's page IDs against the pre-write free pages. The diagnostic asserts that total fresh allocations equal the high-water increase. It verifies zero reuse while the external reader is pinned and positive reuse with zero fresh allocations after release in the candidate.

| Value bytes | Feature | Fresh allocations | Reused pages | Reused before external pin | Reused during external pin | Reused after release | Fresh after release | Final free pages |
| ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 512 | Disabled / enabled | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| 513 | Disabled | 88 | 0 | 0 | 0 | 0 | 64 | 88 |
| 513 | Enabled | 9 | 79 | 15 | 0 | 64 | 0 | 9 |
| 4096 | Disabled | 176 | 0 | 0 | 0 | 0 | 128 | 176 |
| 4096 | Enabled | 18 | 158 | 30 | 0 | 128 | 0 | 18 |

The inferred unavailable reuse opportunities are 87 for each baseline overflow case and eight for each candidate overflow case. These count writes with an existing free page but no reused overflow page. They are not production allocation-attempt counters.

The final diagnostic also records actual active pins at the last reuse decision using a test-only atomic field. For all three sizes, baseline phase counts are 1 without an external reader, 2 with the old reader pinned, and 1 after release. Candidate counts are 0, 1, and 0 respectively. This confirms the writer-owned pin mechanism directly. The field and recording store are absent from production builds. Reader lifetime is controlled by explicit test phases; no production pin-age or retained-heap measurement is added.

The final raw diagnostics are [default](evidence/a1/A1-diagnostic-pins-default.log.gz) and [candidate](evidence/a1/A1-diagnostic-pins-feature.log.gz). Earlier allocation-only outputs are retained separately, not substituted for the direct pin observation.

## Safety review

LogicalOverlay borrows committed_view only during admission. Admitted transaction plans own their encoded mutation keys and provisional revisions, so they do not require that admission generation during physical mutation. The candidate releases the admission pin after the overlay is dropped.

The zero-pin check is not an exclusion barrier: pin acquisition clones the current generation under its read lock and increments the pin count afterward. A reader can therefore arrive around the reuse decision. Safety relies on the immutable data structures: a PublishedGeneration owns an Arc<PageCatalog>, its PageCell retains an Arc<BlinkPage>, and WorkingBlinkState writes a separate overlay. Publication replaces changed cells/chunks. Reusing a logical page ID does not mutate an older generation's page object. The candidate retains the existing conservative rule for external pins observed at the decision; it does not introduce retirement epochs.

Parallel preparation has a separate internal pin. Checking before creating that owned pin avoids treating it as an external reader. The actual parallel fast path accepts only in-place leaf changes and rejects overflow allocation. The 4096-byte diagnostic enables two parallel workers and asserts that its single-leaf workload takes the existing fallback. It validates the final serial decision; it does not claim parallel overflow allocation. Existing parallel differential and fault-matrix tests remain part of the full library suite.

The deterministic interleaving test obtains an old reader after a true reuse decision, deletes and replaces the overflow value through a working overlay with that saved decision, and checks unchanged high-water and the old reader's original value. The ordered-group test deletes an overflow value and conditionally reinserts under NotExists in the same group. It verifies actual page-ID reuse in the candidate, increasing revisions, and complete state after WAL reopen.

The real-service subprocess test covers legacy WAL and segmented durable bases with an actual `.pages` base pack. It acknowledges four overflow overwrites, pauses before the next commit record, and receives SIGKILL. A second process recovers the acknowledged value/revision, acknowledges another service write, pauses before a further commit record, and receives SIGKILL. A final service instance verifies the second acknowledged value/revision. Each parent checks signal 9, and ChildGuard kills/reaps the child on a failed assertion or timeout. The separate clean-close/torn-tail test is supplementary and is not counted as subprocess crash evidence.

## Validation and limits

The primary authored the implementation plan and reviewed safety and evidence. Exactly one GPT-6 Luna worker with high reasoning effort implemented the patch. Local correctness checks use Rust 1.97.1 on Darwin arm64. The source base is `f9a9c0889c36d4c0ba360ff779648b9ac459fffa`; final source hashes and validation evidence are recorded with this increment.

- Default and candidate storage library suites pass after the final test-only pin observation addition: 205 passed, zero failed, and four existing ignored probes per build. Final logs are [default](evidence/a1/final-storage-default.log.gz) and [candidate](evidence/a1/final-storage-feature.log.gz).
- New service integration tests pass in both configurations: two tests per build, including the two-layout, two-crash sequence. Their original execution output was not redirected; this result is retained in the worker handback, and no passing service run was repeated solely to obtain a log. The subsequent pin observation is test-only within the storage library and does not change those service binaries.
- Existing candidate checkpoint integration targets pass: blink_wal_checkpoint eight tests, durable_base_checkpoint six, and background_checkpoint seven. Raw output is [preserved](evidence/a1/a1-server-existing.log.gz).
- Package-specific formatting passes for dodb-storage and dodb-server. Whole-workspace formatting fails before formatting because Cargo metadata rejects vendor/deno/ext/crypto workspace membership; the [failure](evidence/a1/fmt-workspace.log.gz) is preserved. No vendor files were changed.
- Strict Clippy fails on existing storage findings, including 23 findings in the library test target. Advisory Clippy completes; diagnostics are audited against changed source lines. No lint suppression or unrelated cleanup was added.
- The worker's interrupted full-suite retry is preserved as [exit-130 evidence](evidence/a1/a1-storage-default-final.log.gz). It has no final result and is not counted as a pass. Earlier passing suites and final review suites are distinct evidence.

Exact worker commands and earlier outcomes are in [A1-WORKER-EVIDENCE.md](A1-WORKER-EVIDENCE.md). The primary retains original logs, including the interrupted retry and the strict lint failure, as lossless `.log.gz` files under [evidence/a1](evidence/a1). Compression preserves their original bytes, including blank output lines. The validation inventory records both compressed-file and decompressed-original hashes; no passing workload was rerun to alter log formatting.

The primary's final commands, run from the worktree root, are:

```sh
cargo test -p dodb-storage --lib
cargo test -p dodb-storage --lib --features blink-writer-pin-reuse
cargo fmt --manifest-path dodb/crates/dodb-storage/Cargo.toml -- --check
cargo fmt --manifest-path dodb/crates/dodb-server/Cargo.toml -- --check
cargo fmt --all -- --check
cargo clippy -p dodb-storage -p dodb-server --all-targets --features dodb-storage/blink-writer-pin-reuse
git diff --check
```

The first two commands each pass 205 tests; both package formatting checks pass; workspace formatting exits 1 at metadata discovery; advisory Clippy exits 0 with pre-existing warnings; diff checking passes. The final source/diagnostic audit is [validation.json](evidence/a1/validation.json). It checks Clippy's reported primary source locations against added lines, verifies all three-size diagnostic totals and direct pin counts, and records source and raw-log hashes. This audit does not claim a fresh full baseline lint run. Source integrity can be checked from the worktree root with `shasum -a 256 -c dodb/docs/experiments/blink-tenant-density-20261027/evidence/a1/source-sha256.txt`.

No OCI benchmark, throughput improvement, memory saving, physical disk return, or greater active-tenant capacity is claimed. The full tree remains resident, and continuous external readers can still prevent conservative page reuse. Issue #126 remains open for the remaining density and architectural work.
