# T10 streaming checkpoint progress

## Completed substep: bounded paged base-pack output

- Added an ordered `VersionIndex::visit_descriptors` traversal with fixed-depth stack usage and full page-key/path validation. It does not collect a whole-store page-ID set.
- Added a paged durable base-pack writer that accepts exact page images one at a time, verifies page identity and target LSN, preserves sorted order, coalesces at most four images, syncs bounded chunks, writes the final checksum, and syncs the containing directory.
- Added `PagedReadHandle::write_checkpoint_pack`, which counts descriptors in one bounded pass, reserves checkpoint progress memory before buffer allocation, then reloads each exact page through the version-index root and page loader. It verifies image LSN and CRC against the descriptor before writing.
- The focused fixture streams eight pages through a two-page cache in both coalesced and uncoalesced modes. It scans each completed pack and compares every image byte-for-byte. A pinned old reader is streamed after a newer writer transaction and still produces the exact captured pages. A page-write fault leaves an incomplete pack and does not publish a manifest.

## Verification

- Focused fixture: 1 passed.
- Feature library suite: 294 passed, 0 failed, 4 ignored in 124.83 seconds.
- Default and feature library checks, targeted formatting, and `git diff --check` passed.
- Strict feature Clippy exits 101 with 32 existing crate findings. No finding points to the new descriptor traversal or streamed pack writer; no lint suppressions were added.
- Raw logs: `logs/paged-streamed-pack-focused.log`, `logs/paged-streamed-pack-feature-lib-final.log`, default/feature check logs, and `logs/paged-streamed-pack-strict-clippy-final2.log`.
- Initial source commit: `971bb8143b122c5f44169c72ce8ac897da359011`.

T10 remains partial. Streamed pack writing is not yet connected to durable checkpoint-index promotion, paged manifest publication, or WAL segment boundary rotation. Source/index disk reservations, incremental checkpoints, checkpoint debt scheduling, and the complete crash/reopen matrix remain outstanding. T10 remains the next task; no service route, default, or performance experiment changed.

## Completed substep: incremental paged pack output and index application

- Source commit: `e741dd19d0e6940af6290a85413c81d3292f01be`.

- Added an incremental pack request that emits only descriptors whose image LSN is newer than the previous checkpoint. It returns no pack when the target contains no newer pages and marks emitted packs as incremental.
- Added incremental index application that validates the exact target page set, image LSNs, CRCs, and pack checksum before replacing dirty locators. Unchanged descriptors and version tokens remain intact; the resulting index root is promoted to durable storage.
- The focused writer fixture commits a newer two-key transaction, verifies that only a strict subset of the eight-page target is emitted, and compares every packed image to the target index descriptor.
- The focused index test verifies changed-locator replacement, unchanged-source retention, token preservation, and durable-root validation.
- Full feature library suite at this implementation stage: 295 passed, 0 failed, 4 ignored. Default and feature checks and formatting passed; one default test run emitted an unused feature-only import warning that was fixed and verified clean by the final compile check. Strict feature Clippy remains at 32 existing findings, with no diagnostic in the new incremental code.
- A full-fixture attempt to apply the incremental root while old snapshot handles remained pinned exceeded the fixture's version-index resource capacity. The failed log is preserved. The smaller index test passes; realistic checkpoint/index overlap and progress-pool admission remain open acceptance work.

T10 remains partial. The incremental helper now composes durable index promotion, a synced next WAL segment, PagedManifest publication, reopen, and exact reads from old and new pack sources. The live PagedTransactionWriter still does not atomically capture and rotate at the target LSN, and the complete streamed-pack publication path is not integrated. Disk capacity reservations, crash-boundary coverage, scheduler defer/wake, and checkpoint debt accounting remain outstanding. No service route, default, or performance matrix changed.


## Completed substep: progress reservation and incremental manifest reopen

- Complete and incremental index builders now reserve one 128 KiB non-borrowable progress unit before descriptor updates. Durable root promotion reserves another 128 KiB while that lease remains live, so the tested index-build peak is 256 KiB.
- A deterministic 64 KiB fixture rejects checkpoint index work before changing the root or progress accounting. The 256 KiB fixture builds both complete and incremental roots.
- The incremental integration fixture writes a valid prior pack, applies one newer page through an incremental pack, syncs a new WAL segment boundary, publishes manifest generation 2 at LSN 12, reopens the manifest, and reads byte-exact unchanged and changed pages from their referenced pack sources.
- Default library tests: 221 passed, 0 failed, 4 ignored. Feature library tests: 296 passed, 0 failed, 4 ignored. Default/feature checks and formatting passed. Strict feature Clippy remains at 32 existing crate findings.
- Final source commit: `0bdce39d1174eb71c87dc1fa12b4521969228ab8`.

T10 remains partial. The publication test composes storage primitives, but the actual paged transaction writer does not yet serialize target capture with WAL rotation and publication. Source/index disk reservations, combined loader/output/index overlap, complete-pack publication, crash fault matrix, resumable worker slices, and checkpoint debt scheduling remain outstanding.

## Completed substep: serialized writer checkpoint publication

- Source commits: `d99c3e465` (`Integrate paged checkpoints with writer WAL rotation`) and `8415bcd7c` (`Fail closed on paged checkpoint pointer errors`).
- Added `PagedTransactionWriter::checkpoint_to_manifest`. The mutable writer captures its current committed LSN while excluding later writes, writes and syncs a complete or incremental pack, applies and durably promotes its index root, initializes and syncs the next WAL segment at the captured LSN and current next batch ID, syncs the WAL directory, then publishes the next manifest pointer. Only after pointer publication succeeds does it install the new reader generation and WAL in the writer. Uncertain manifest publication marks the writer unavailable.
- WAL source header identity is exposed from validated scans and newly initialized segments. The writer records the new segment identity for subsequent page locators. The checkpoint API verifies the configured pack directory and that the manifest names the writer's active WAL segment.
- A disk-backed integration fixture performs a complete checkpoint, commits to the newly active WAL, applies an incremental checkpoint, publishes manifest generation 3, reopens the manifest, and reads the exact latest value from the new checkpoint source. A fault immediately before a later pointer write leaves generation 3 selected after reopen and fences the writer after the uncertain result.
- Feature library tests: 297 passed, 0 failed, 4 ignored. Default library tests: 221 passed, 0 failed, 4 ignored. The focused writer/manifest integration test passed. Default and feature checks plus formatting passed. Strict Clippy reports the same 32 existing crate findings; it reports no diagnostic in the new writer integration, WAL source identity, or source-directory helper.
- Raw logs: `logs/paged-writer-manifest-integration-focused.log`, `logs/paged-writer-manifest-feature-lib.log`, `logs/paged-writer-manifest-default-lib.log`, `logs/paged-writer-manifest-default-check.log`, `logs/paged-writer-manifest-feature-check.log`, `logs/paged-writer-manifest-format.log`, and `logs/paged-writer-manifest-strict-clippy-all-targets.log`.

T10 remains partial. The successful test proves serialized in-process target capture, complete and incremental publication, next-segment rotation, and normal manifest reopen. Process-crash and actual pointer-write I/O fault coverage, source-catalog registration and GC retention across the writer path, source/index disk reservations, loader/index/output overlap accounting, the pinned-root capacity counterexample, resumable slices, and checkpoint debt scheduling remain open. No service route, default, or performance matrix changed. T17 was not started because the frozen matrix gate is separate.

## Completed substep: pinned-target checkpoint with concurrent writes

- `PagedTransactionWriter::begin_checkpoint` syncs the active WAL and captures a pinned reader generation, then returns a checkpoint job so pack output can proceed while the writer accepts newer commits.
- `publish_checkpoint` validates the manifest generation and source, verifies every packed image against the pinned target, and merges packed locators into the latest version-index root only when the current page still matches the target version. Pages changed after capture retain their newer WAL locators. The new WAL boundary and manifest prefix advance to the latest committed LSN before the writer switches generations.
- The disk-backed fixture now writes after both complete and incremental target capture, publishes each checkpoint, and reopens to the latest acknowledged value. It also verifies pointer-publication failure keeps the prior manifest selected and fences the writer.
- Fixed a pre-existing race in the page-locator test fixture: parallel tests could reuse a timestamp-only directory name. A process-local atomic sequence now makes fixture paths unique. The initial full feature-suite log preserving the collision is retained.
- Focused locator fixture: 1 passed. Full feature library suite: 297 passed, 0 failed, 4 ignored. Default library suite: 221 passed, 0 failed, 4 ignored. Rust formatting and `git diff --check` passed.
- The first all-target compile found a missing `image_delta_chain_limit` subtraction in `phase0-bench.rs`; added the delta and reran successfully. Existing benchmark harness dead-code warnings remain. Strict library Clippy reports 12 pre-existing diagnostics after the two new checkpoint findings were fixed; no lint suppression was added.
- Raw logs: `logs/paged-locator-fixture-unique-focused.log`, `logs/paged-writer-target-interleave-feature-lib-final2.log`, `logs/paged-writer-target-interleave-default-lib-final.log`, `logs/paged-writer-target-interleave-all-targets-failure.log`, `logs/paged-writer-target-interleave-all-targets-compile-final2.log`, `logs/paged-writer-target-interleave-strict-clippy.log` (initial new findings), and `logs/paged-writer-target-interleave-strict-clippy-final.log` (12 existing diagnostics). `logs/paged-writer-target-interleave-feature-lib.log` preserves the initial fixture collision.

T10 remains partial. Process-kill recovery, actual pointer-write I/O injection, source-catalog registration and GC retention, source/index disk reservations, combined loader/index/output accounting, tiny-capacity defer and progress, a one-worker resumable scheduler, and checkpoint debt/time-above-threshold accounting remain unimplemented. Service routing and defaults remain unchanged; T17 was not started.

## Completed substep: single active checkpoint job

- The writer now keeps a shared active-job flag. A second `begin_checkpoint` while a job owns the pinned target returns overload; dropping or publishing the job releases the slot. Pack output is single-attempt per job, including when incremental output is empty or fails.
- The writer integration fixture verifies a competing job is rejected, duplicate pack output is rejected, and a later checkpoint can start after publication.
- Focused writer fixture: 1 passed. Full feature library suite: 297 passed, 0 failed, 4 ignored. All feature test targets compiled. Strict library Clippy still reports 12 pre-existing diagnostics and no new checkpoint diagnostics.
- Source commit: `70133a4f6885ea2b46904a54f5acb8c20c2ff760`.
- Raw logs: `logs/paged-checkpoint-single-worker-focused.log`, `logs/paged-checkpoint-single-worker-feature-lib.log`, `logs/paged-checkpoint-single-worker-all-targets-compile.log`, and `logs/paged-checkpoint-single-worker-clippy.log`.

T10 remains partial. The active-job guard does not yet provide resumable bounded slices, queue debt/time accounting, or capacity-release wakeups. Process-crash, source catalog/GC, and disk reservation acceptance remains open.

## Completed substep: manifest pointer write fault boundary

- Manifest publication now accepts fault injection at four pointer stages: before temporary creation, after the temporary pointer has been written and synced, after rename, and after containing-directory sync.
- The checkpoint integration fixture injects failure after the temporary pointer sync. It verifies generation 3 remains selected on reopen, the synced temporary pointer is preserved, and the writer rejects further commits after the uncertain publication result.
- Full feature library suite: 297 passed, 0 failed, 4 ignored. All feature targets compiled. The test-only publication helper was excluded from non-test builds after the first compile exposed an unused-method warning. Strict Clippy still reports 12 pre-existing diagnostics and no new checkpoint finding.
- Source commit: `445dce45c585abd9b2d795dd14c468e4e9463791`.
- Raw logs: `logs/paged-checkpoint-pointer-fault-feature-lib.log`, `logs/paged-checkpoint-pointer-fault-all-targets-compile.log` (first compile warning), `logs/paged-checkpoint-pointer-fault-all-targets-compile-final.log`, and `logs/paged-checkpoint-pointer-fault-strict-clippy.log`.

T10 remains partial. Process-kill crash/reopen, source-catalog and GC integration, source/index disk reservations, combined loader/index/output accounting, bounded defer with progress after capacity release, resumable slices, and checkpoint debt/time-above-threshold accounting remain open.

## Completed substep: old-reader retention through checkpoint GC

- The writer integration fixture now holds a pre-checkpoint reader through complete and incremental publication, reads its exact older value afterward, and runs source collection against its WAL segment.
- The collected WAL is reported protected and remains present while the old reader root is pinned. This exercises the writer-created root leases and `SourceManager::collect_garbage` together.
- Focused writer/reader/GC fixture: 1 passed. The complete feature suite had passed after the pointer fault-path change; this substep changes only the focused fixture.
- Source commit: `560d185172d27c266b676d2a44e2508b28515335`.
- Raw log: `logs/paged-writer-old-reader-gc-focused.log`.

T10 remains partial. Persistent source-catalog publication, restart-time root/source retention, source/index disk reservations, combined loader/index/output accounting, bounded defer and progress after capacity release, resumable slices, and checkpoint debt/time-above-threshold accounting remain open.

## Completed substep: failed checkpoint pack stays unpublished

- The checkpoint job exposes an injectable pack-write fault for storage validation. A failure during page output leaves only the pack header, does not advance the manifest, and releases the single-worker lease when the job is dropped.
- The writer then accepts a later transaction and another checkpoint attempt. The same fixture still verifies old-reader source protection and the manifest-pointer temporary-sync fault boundary.
- Focused combined pack-fault/reader-GC/pointer-fault fixture: 1 passed. Full feature suite: 297 passed, 0 failed, 4 ignored. All feature targets compiled. Strict Clippy reports 12 pre-existing diagnostics and no new checkpoint diagnostic.
- Source commit: `03f01f45a982289258aee7b8bc92b024078223a4`.
- Raw logs: `logs/paged-writer-pack-failure-focused.log`, `logs/paged-checkpoint-faults-reader-gc-feature-lib.log`, `logs/paged-checkpoint-faults-reader-gc-all-targets-compile.log`, and `logs/paged-checkpoint-faults-reader-gc-strict-clippy.log`.

T10 remains partial. Persistent source-catalog publication and restart-time root/source retention, source/index disk reservations, combined loader/index/output accounting, bounded defer and progress after capacity release, resumable slices, and checkpoint debt/time-above-threshold accounting remain open.

## Completed substep: resumable bounded pack slices

- Source commit: `22a0e4a673f97ea96d714c88ff1c2b4532e1305b`.
- Added a fixed-depth `VersionIndexDescriptorCursor` that yields one ordered descriptor at a time without retaining loaded node guards. Its sequence is checked against the existing traversal for a sparse randomized index while the node cache is bounded to two nodes.
- Added `PagedCheckpointJob::write_pack_slice`. The job retains its charged output reservation, cursor, and pack writer between calls; each call emits at most the requested number of page images. Existing `write_pack` drains the same cursor to completion. A later transaction is committed between two one-page slices while output still reads the pinned target. A slice fault preserves the incomplete pack, prevents retrying the failed job, and leaves the manifest unpublished.
- Focused writer test: 1 passed. Default library suite: 221 passed, 0 failed, 4 ignored. Full feature suite: storage library 297 passed, 0 failed, 4 ignored; benchmark binary 8 passed; phase0 binary 42 passed; all remaining binaries and doc tests passed; checkpoint/recovery integration tests passed, including SIGKILL suites. Feature all-target check and targeted format/diff checks passed.
- Strict feature Clippy exits 101 with the same 12 existing storage diagnostics; none point to the cursor or sliced checkpoint code. Existing harness dead-code warnings remain in all-target compilation.
- Raw logs: `logs/paged-checkpoint-resumable-slices-focused.log`, `logs/paged-checkpoint-resumable-slices-default-suite.log`, `logs/paged-checkpoint-resumable-slices-feature-suite.log`, `logs/paged-checkpoint-resumable-slices-all-targets-check.log`, and `logs/paged-checkpoint-resumable-slices-strict-clippy.log`.

T10 remains partial. Slices are resumable within one active writer job, but no fair multi-tenant continuation scheduler or debt/time-above-threshold accounting exists. Persistent source-catalog publication and restart-time retention, source/index disk reservations, combined loader/index/output accounting, tiny-capacity defer/wake, process-crash coverage of this paged checkpoint writer, and the complete resource acceptance matrix remain outstanding. The paged service route remains inactive and T17 remains unstarted.

## Completed substep: persistent source catalog publication and restart-time GC retention

- Source commit: `e9b06076ed3b01e3daaa3605155f474cf0fe4323`.
- Added a fixed 4096-byte append-only source catalog with 81 fixed-width source records per page, checksummed page links, store identity, zero-padding validation, and bounded marker-run streaming. Catalog pages are synced and the directory is synced before the manifest points to the new catalog root.
- Checkpoint publication records the exact source set reachable from the durable index plus the next recovery WAL. Reopen validates the catalog identities against the index and source headers. GC marks the active catalog and pending catalog root; it defers while a catalog is being built so an unindexed recovery WAL cannot be reclaimed between creation and manifest publication.
- Multi-page catalog corruption fixture: 1 passed. Pending-publication GC fixture: 1 passed. Restart, manifest validation, and recovery-WAL GC fixture: 1 passed. Full feature library suite: 299 passed, 0 failed, 4 ignored. All-target feature check passed; rustfmt and `git diff --check` passed.
- Strict feature Clippy exits 101 with 12 existing storage diagnostics; no diagnostic points to the catalog changes. The first catalog Clippy run found one new collapsible-if diagnostic, which was fixed without a suppression. One initial marker-run fixture failed because its temporary filename matched the source marker prefix; the parser now validates exact marker names. The fixture cleaned its temporary files, so no raw log was retained.
- Raw logs: `logs/source-catalog-multipage-focused.log`, `logs/source-catalog-pending-publication-focused.log`, `logs/source-catalog-restart-gc-focused.log`, `logs/source-catalog-feature-library-final2.log`, `logs/source-catalog-all-targets-check-final3.log`, and `logs/source-catalog-strict-clippy-final3.log`.

T10 remains partial. Persistent source catalog publication and restart-time recovery-WAL retention pass for the exercised writer path. Catalog-generation reclamation, source/index disk reservations, combined loader/index/output accounting, bounded defer and progress after capacity release, the fair multi-tenant continuation scheduler, debt/time-above-threshold accounting, and process-kill recovery coverage for this paged writer remain open. Service routing and defaults remain unchanged; T17 was not started.

## Completed substep: reclaim unreachable source catalog generations

- Source commit: `fc6f974856750b6a0fadf70b76b48bed3e19504c`.
- Source GC now preserves catalog page ranges reachable from the durable and pending manifest roots, then removes and directory-syncs unreachable catalog pages. The collector verifies each retained chain before removing files and reports reclaimed catalog page and byte counts separately from pack/WAL sources.
- The focused generation test reclaimed two obsolete pages and retained the current root page. The checkpoint restart/GC integration fixture passed and verified an obsolete generation was reclaimed. Full feature library suite: 300 passed, 0 failed, 4 ignored. All-target feature check passed; rustfmt and `git diff --check` passed.
- Strict feature Clippy exits 101 with the same 12 existing storage diagnostics; no diagnostic points to source catalog reclamation.
- Raw logs: `logs/source-catalog-gc-reclaim-focused.log`, `logs/source-catalog-gc-restart-focused.log`, `logs/source-catalog-gc-feature-library.log`, `logs/source-catalog-gc-all-targets-check.log`, and `logs/source-catalog-gc-strict-clippy.log`.

T10 remains partial. Source catalog generations now have bounded retention after GC, but checkpoint pack/index output disk reservations, combined loader/index/output memory accounting, bounded defer and eventual progress after capacity release, fair maintenance slices with debt/time-above-threshold accounting, and process-kill recovery coverage for this paged writer remain open. Service routing and defaults remain unchanged; T17 was not started.

## Partial substep: bounded disk reservations for checkpoint sources

- Source commit: `a96a3155d23f9301f6a7126e8061afbe77593775`.
- Added global, per-tenant, and per-category disk limits and snapshots. Reservations support checked growth, shrink, category transfer, and merge. Existing checkpoint/WAL directory bytes are charged when `SourceManager` opens and reconciled before GC. Checkpoint pack output reserves its exact encoded size before pack creation; source catalog output reserves its calculated page bytes before catalog writes. The writer fixture runs with a 128 KiB global and per-tenant cap and checks measured peak disk reservation stays within the cap.
- Disk reservation fixtures: 2 passed. Finite-cap checkpoint writer fixture: 1 passed. Full feature library suite: 302 passed, 0 failed, 4 ignored. Default library check and all-target feature check passed; rustfmt and `git diff --check` passed.
- The first full feature run found five GC fixtures where source files were created after manager initialization; its log is preserved. GC now reconciles the directory usage before unlink. The rerun passed. Strict feature Clippy exits 101 with the 12 existing storage diagnostics and no new disk-reservation finding.
- Raw logs: `logs/disk-reservation-core-final.log`, `logs/disk-reservation-writer-limited-focused.log`, `logs/disk-reservation-feature-library.log` (initial failures), `logs/disk-reservation-feature-library-final2.log`, `logs/disk-reservation-default-check.log`, `logs/disk-reservation-all-targets-final.log`, and `logs/disk-reservation-strict-clippy-final.log`.

T10 remains partial. Version-index node output and active WAL growth are not reserved before writes, and the required loader memory is not yet reserved with checkpoint/index output. The capped writer fixture passed but does not prove quota rejection before pack creation or continuation progress after capacity release. Fair maintenance slices, debt/time-above-threshold accounting, and paged-writer process-kill recovery remain open. Service routing and defaults remain unchanged; T17 was not started.

## Partial substep: version-index disk accounting

- Source commit: `58faa228edf8252cc83e4579c2df85fb15bf6828`.
- `VersionIndexStore` now charges existing node files when opened and reserves 4096 bytes before creating each new node. Partial node-write failures retain a conservative charge; name collisions and create failures release the unused reservation.
- Full feature library suite: 302 passed, 0 failed, 4 ignored. The checkpoint writer integration test passed under a 2 MiB global and per-tenant disk limit and confirmed both `VersionIndex` and retained-source categories were charged. Default library check and all-target feature check passed; rustfmt and `git diff --check` passed. Strict Clippy remains at 12 existing diagnostics with no new version-index finding.
- A 128 KiB run correctly overloaded when reopened source files and index nodes were both included. The failed test database remains at `/var/folders/wp/zy07g4zs3vg61f9pb50tl0kw0000gn/T/dodb-paged-writer-checkpoint-42580-1793616885919787000`; no raw failure log was captured. The earlier 128 KiB pass charged source and pack output only and is labeled superseded in the receipt.
- Raw logs: `logs/version-index-disk-feature-library.log`, `logs/version-index-disk-writer-focused.log`, `logs/version-index-disk-default-check.log`, `logs/version-index-disk-all-targets-check.log`, and `logs/version-index-disk-strict-clippy.log`.

T10 remains partial. Version-index output is now bounded before each node file, but partial promotions can leave charged unreachable nodes and there is no index-node GC yet. Active WAL growth, whole checkpoint/index preflight, loader overlap, small-capacity retry after release, fair maintenance slices, debt/time-above-threshold accounting, and paged-writer process-kill recovery remain open. Defaults and service routing remain unchanged; T17 was not started.

## Partial substep: reclaim unreachable version-index nodes

- Source commit: `8039b0edb32990a357e87e04acd945135fa4d9ea`.
- Source GC now walks every registered version-index root, marks its reachable node files in bounded-depth traversal scratch, unlinks unmarked node files, syncs the index directory, and releases the reclaimed byte charge. It validates each root through descriptor traversal before deleting nodes. Pinned roots remain intact; a second pass with only the current root reclaims nodes unique to the older generation.
- Focused version-index GC: 1 passed. Source-manager GC suite: 10 passed. Checkpoint restart/GC fixture: 1 passed. Full feature library suite: 303 passed, 0 failed, 4 ignored. Default library check and all-target feature check passed; rustfmt and `git diff --check` passed. Strict feature Clippy remains at 12 existing diagnostics with no new index-GC finding.
- Raw logs: `logs/version-index-gc-pinned-roots-focused.log`, `logs/version-index-gc-source-manager-focused.log`, `logs/version-index-gc-restart-focused.log`, `logs/version-index-gc-feature-library.log`, `logs/version-index-gc-default-check.log`, `logs/version-index-gc-all-targets-check.log`, and `logs/version-index-gc-strict-clippy.log`.

T10 remains partial. Active WAL growth and GC scratch disk are not reserved before writes, and disk charges currently follow manager lifetimes rather than a persistent per-tenant registry. Checkpoint-wide quota preflight, explicit no-pack-on-rejection and retry-after-release tests, required loader overlap, fair maintenance slices, debt/time-above-threshold accounting, and paged-writer process-kill recovery remain open. Defaults and service routing remain unchanged; T17 was not started.

## Partial substep: persistent retained-source disk accounting

- Source commit: `c613f9ba42178609b9ff5269a206844b32c8c4a3`.
- The shared disk ledger now stores retained usage under tenant, category, and canonical source-directory identity. Reopening the same directories reconciles to actual bytes instead of adding a second charge; dropping `SourceManager` no longer releases files that remain on disk. GC reconciles after unlink, and temporary source-catalog or checkpoint output reservations convert atomically into the persistent retained-source charge.
- Focused ledger, atomic reservation-conversion, and manager drop/reopen fixtures each passed. The full feature library suite passed: 306 passed, 0 failed, 4 ignored. Default library check, feature all-target check, formatting, and `git diff --check` passed. Strict Clippy exits 101 with 32 existing crate findings and no diagnostic in the changed resource or source-manager modules.
- The first manager-lifecycle fixture wrote a file after open without invoking the source manager's directory reconciliation; its assertion correctly observed zero charged bytes. The corrected fixture reconciles before drop. The temporary test directory was cleaned automatically, and no raw failure log or database was produced.
- Raw logs: `logs/persistent-disk-usage-ledger-focused.log`, `logs/persistent-source-reservation-conversion-focused.log`, `logs/persistent-source-reopen-focused.log`, `logs/persistent-disk-usage-feature-library.log`, `logs/persistent-disk-usage-default-check.log`, `logs/persistent-disk-usage-all-targets-check.log`, and `logs/persistent-disk-usage-strict-clippy.log`.

T10 remains partial. Persistent retained-source accounting now survives source-manager teardown and avoids duplicate charges on reopen. Version-index disk charges still follow the `VersionIndexStore` lifetime; active WAL append growth and GC scratch disk are not reserved before writes. The pinned-root capacity/defer case, no-pack checkpoint quota rejection and retry, loader overlap, fair maintenance slices, debt/time-above-threshold accounting, and paged-writer process-kill recovery remain outstanding. Service routing and defaults remain unchanged; T17 was not started.
