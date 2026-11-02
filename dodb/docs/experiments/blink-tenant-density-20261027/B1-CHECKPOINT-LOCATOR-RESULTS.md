# B1.0 checkpoint-image locator correctness results

Tracking issue: https://github.com/NamseEnt/fn0/issues/126

Implemented the first read-only B1 primitive: an exact checkpoint image can be located and reloaded without retaining all pack payloads in the locator index. It is available only with the default-disabled `dodb-storage/blink-checkpoint-locators` Cargo feature. Source commit is `aca32bbab2dd3136eebd9cacf20c42cac65194ce`; the implementation follows the separately committed [plan](B1-CHECKPOINT-LOCATOR-PLAN.md), `40c3ded06`.

This completes the checkpoint-image primitive, not B1 demand loading or B2 resource enforcement. No service reader/writer uses these locators, no resident page is evicted, and no new benchmark or OCI execution was performed. Earlier capacity/maintenance evidence remains frozen at its recorded source/binaries, including cb0 for the latest cohorts. The B-link adoption assessment is unchanged.

## Implemented behavior

`blink::page_locator::ValidatedCheckpointPack::open` accepts a caller-supplied expected pack entry, WAL identity, and high-water bound. It opens that specific source once and runs the same complete pack scanner used by ordinary recovery. The scanner checks file size, header identity/format/reserved bytes/checksum, ordered page IDs within bounds, image decoding/checksum and checkpoint LSN, aggregate CRC, and trailer. An incomplete or invalid scan returns no usable pack/locator.

The successful index stores only page ID, record offset, image LSN, and image checksum, together with one shared source handle and expected pack/tenant/database/shard identity. `locate` returns an exact source-bound `CheckpointPageLocator`; missing pages return `None`. `read_image` reads one fixed-size image through the held file, verifies source length, expected image CRC, page decoding/ID, and exact image LSN, and returns an owned encoded page. A caller can mutate/drop that returned buffer without changing later loads. A mutex protects the shared seek cursor; no `unsafe` or new dependency was added.

The scanner was extracted from the existing recovery reader rather than duplicating its parser. Default recovery still collects its normal resident images and owner map. The new feature changes neither the disk format nor manifest publication/sync/reclamation policy. Existing service routes remain resident.

Validation timing is explicit: the entire pack is validated on open; each reload validates its requested image against the recorded version. The prototype does not rescan the entire pack on every read. It validates pack contents, not a complete database's tree ownership/links, trusted manifest membership, or cross-pack reachability. Callers must supply the expected recovery metadata; this API does not discover or publish a manifest.

## Correctness and compatibility evidence

Eleven new unit tests cover sparse leaf/overflow/free images against original bytes and the existing recovery reader; exact page versus checkpoint LSN; missing IDs; full-scan rejection of an unrequested corrupt image, trailer/header/padding/identity/ordering/range/length damage; changed valid images and truncation after validation; concurrent independent reloads; and old-source ownership after index drop, unlink, and pathname replacement.

One new actual-store integration test runs both coalesced and uncoalesced pack output. It retains a locator from a real checkpoint, modifies seeded keys across leaf pages until existing consolidation removes the old pack from the manifest and unlinks its pathname, then checks that the old locator returns the exact original bytes while current reads and normal reopen preserve the newer acknowledged state. The test does not route service reads through locators or claim crash-safe old-generation retention.

| Configuration/check | Passing tests | Existing ignored tests | Result |
| --- | ---: | ---: | --- |
| Locator feature: library | 216 | 4 | Pass |
| Locator feature: durable-base checkpoint, background/SIGKILL, segmented checkpoint/SIGKILL and WAL/SIGKILL integration | 24 | 0 | Pass |
| Default: library and durable-base checkpoint integration | 219 | 4 | Pass |
| Locator plus writer-pin-reuse: library | 216 | 4 | Pass |
| Targeted formatting for scanner, locator module, and checkpoint tests | N/A | N/A | Pass |
| Strict library Clippy, default and locator feature | N/A | N/A | Both fail with the same 12 pre-existing diagnostics |

These configurations overlap; do not sum them into a count of unique tests. Twelve tests are newly added: eleven unit tests and one actual-store test. Existing manual probes remain ignored. The SIGKILL/recovery cases validate compatibility of the shared pack scanner; they are not the future real-service eviction/WAL reconstruction crash gate.

[Validation receipt](evidence/checkpoint-locators/validation.json) records source hashes, host/toolchain, exact commands, exit statuses, test summaries, and raw log hashes. All execution here was local correctness testing, not designated-host performance evidence. Strict Clippy's twelve errors are in pre-existing functions in `blink/mod.rs` and `wal.rs`; the default/feature diagnostics match exactly, and neither the new locator nor shared scanner has a finding. No lint exception or suppression was added. The integration build also retains existing mixed-harness dead-code warnings in its raw logs.

The first development integration attempt overwrote only key 90 and never reached the intended unlink boundary: two owning packs remained live under the unchanged consolidation policy. Its assertion failure was a test-setup mistake, not a locator reload failure. The final test varies existing keys across leaf pages to exercise actual consolidation; engine policy was not changed. [Development receipt](evidence/checkpoint-locators/development-attempt.json) identifies and hashes the preserved failed test database. The original local temporary directory remains intact, and a hash-verified `failed-development-db.tar.gz` additionally preserves every recorded file in the repository. This is not an OCI benchmark database or new capacity evidence. The receipt explicitly distinguishes its transcribed failure marker from a full raw terminal transcript.

## Limits and next gate

The index still has O(pack page count) metadata and performs an O(pack bytes) validated opening scan. It has no global byte cap, payload cache, load coalescing, admission policy, dirty-page budget, or retained-heap measurement. Reloading a page performs I/O/decoding and returns a caller-owned allocation. No service-memory reduction or opening-latency advantage is measured.

Holding an open Unix file keeps its original bytes readable after unlink/path replacement and consumes an open-file reference and retained storage. This is process-lifetime protection only. It is not a persisted generation/recovery root, page-reuse incarnation scheme, safe crash-time GC policy, or physical disk return. A crash loses these handles. Do not integrate eviction with the current latest-manifest-only reclamation policy based on this result.

Next implement committed-WAL image/delta locators and bounded reconstruction under an opt-in correctness boundary. A locator may become visible only for a verified committed history; do not infer durability merely from a frame offset or the latest page-chain LSN/CRC. Then define pinned generation/recovery-root reachability across checkpoint publication, page-ID reuse, and source reclamation before connecting reader/writer miss paths. B2 must account for metadata, in-flight loads, dirty/snapshot/queue bytes and retention debt with fair bounded admission. Existing cgroups and the global worker count do not supply those contracts.

One-tenant forced eviction, pinned-old versions, dirty WAL-backed reconstruction, checkpoint races, corruption gates, and real-service incomplete-transaction recovery -> acknowledged write -> second crash/recovery must pass before any many-tenant comparison. Growing-key, cold-store/host-cache, physical disk-return, and adoption gates remain open. Keep defaults unchanged and #126 open.

## Reproduction

Use the exact commands and feature combinations in `evidence/checkpoint-locators/validation.json`. The focused new checks are:

```sh
cargo test -p dodb-storage --features blink-checkpoint-locators --lib blink::page_locator::tests
cargo test -p dodb-storage --features blink-checkpoint-locators --test durable_base_checkpoint held_checkpoint_locator
```
