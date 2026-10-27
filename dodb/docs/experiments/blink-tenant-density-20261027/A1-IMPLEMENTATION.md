# A1: Writer pin lifetime and page reuse

Tracking issue: https://github.com/NamseEnt/fn0/issues/126

## Scope and authorization

Implement only A1 from PLAN.md on experiment/blink-tenant-density-20261027, starting at f9a9c0889. The primary agent owns this plan, safety review, evidence assessment, final documentation, and commit/push. One GPT-6 Luna worker at high reasoning effort owns implementation and focused validation. No other agents are authorized for this task.

Do not implement A2-A5, a density harness, shared RocksDB resources, page eviction, epoch reclamation, or a broad performance matrix. Existing defaults, WAL durability, transaction semantics, file formats, and checkpoint policies remain unchanged. Experimental reuse behavior must be opt-in and disabled by default.

## Source finding and questions

The planned execution path pins committed_view for admission and drops its LogicalOverlay without dropping committed_view before checking can_reuse_pages. The latter requires zero active generation pins. The parallel preparation call also creates a temporary publisher pin and must be inspected separately. A serial-only fix must not be reported as fixing parallel execution.

Determine whether the writer-owned pin blocks reuse without an external reader, whether releasing it after admission permits safe free-page reuse, and whether repeated fixed-key overflow replacement then reaches a bounded page high-water mark. Separate source facts, deterministic allocation observations, and unmeasured performance effects.

## Implementation sequence

1. Trace ownership after admission, serial and parallel preparation, immutable published catalogs, allocation/free-list updates, and reader pin acquisition. Record why a reader arriving after the reuse decision cannot observe a repurposed page through its pinned generation. Preserve the conservative external-pin guard; do not implement epoch reclamation.
2. Add a narrow opt-in storage feature or configuration, with the default path retaining baseline behavior. Prefer a Cargo feature named blink-writer-pin-reuse if no existing appropriate switch exists. Release only writer-owned admission references at their actual last use. Handle the parallel internal pin explicitly and conservatively; keep external-reader safety intact.
3. Add narrowly scoped allocation/reuse diagnostics. Record fresh allocations, successful free-list reuse, denied reuse decisions, active pins at the decision, free-page count, and page high-water. Reuse existing metrics where accurate. Clearly label normalized page-image bytes; do not call them exact retained heap. Avoid per-page clocks or maps in the normal default path. Pin lifetime validation can use explicit test phase boundaries instead of production wall-clock tracking.
4. Add deterministic fixed-key overwrite diagnostics for 512, 513, and 4096-byte values. Run the identical workload with the feature disabled and enabled. Cover no external reader, an old pinned reader during overwrites, and further overwrites after releasing that reader. Assert old pinned values remain correct, allocation accounting agrees with high-water, and reuse becomes observable and bounded only where safe. Cover the service's serial planned path and the optional parallel preparation path or its documented conservative fallback.
5. Add real-service durability coverage with the opt-in feature enabled: acknowledged overflow writes, an incomplete transaction/torn WAL tail or an equivalent pre-commit injected failure, reopen and validate, a successful subsequent service write, another unclean close/reopen, and validation of acknowledged state. Exercise legacy WAL and the existing segmented durable-base path. Preserve existing fault/corruption checks and request consistency tests.
6. Run focused checks, summarize results and limits, then return all edits and evidence to the primary agent for review. Do not commit, push, post externally, or spawn agents.

## Acceptance criteria

- Feature-disabled behavior and defaults remain unchanged.
- The diagnostic demonstrates the baseline self-pin condition and distinguishes it from external pins.
- Without external readers, overflow overwrite reuse is observed and high-water stops increasing after warmup in the candidate. The 512-byte case is a boundary control, not assumed to require new overflow pages.
- While an external old generation is pinned, the conservative reuse guard remains effective and every old value read through that generation remains correct. Releasing the pin allows safe reuse on later writes.
- Parallel handling does not subtract an unowned external pin or weaken the reader guard. Any remaining conservative fallback is explicitly reported and covered.
- The real service passes incomplete-transaction recovery -> successful acknowledged write -> second unclean recovery with correct revisions and values.
- Existing targeted storage/server tests pass with default settings and the feature. No added comments, lint suppressions, abstract single-letter names, or non-English repository content.
- Evidence states that local deterministic diagnostics and correctness checks do not establish OCI throughput, RSS savings, disk return, or active-tenant density.

## Checks and evidence

Use the monorepo root. Start with cargo fmt --all -- --check. Run cargo test -p dodb-storage --lib with default features and with --features blink-writer-pin-reuse. Run the focused new server integration target in both configurations, using --features dodb-storage/blink-writer-pin-reuse for the opt-in build. Run existing blink_wal_checkpoint, durable_base_checkpoint, and background_checkpoint server integration targets with the candidate. Run cargo clippy -p dodb-storage -p dodb-server --all-targets --features dodb-storage/blink-writer-pin-reuse -- -D warnings; resolve findings without suppressions. Adapt command filters only to actual target names and record exact commands and outcomes.

Preserve focused diagnostic output and failures under this experiment directory. Keep build caches and large logs outside tracked source unless they are intentional compact evidence. Do not rerun passing diagnostics solely to repair report formatting. If an existing unrelated check fails, preserve the exact failure and identify it separately.

No performance run is required for this A1 correctness increment. If later performance evidence is needed, use only the designated OCI host and ZFS location in PLAN.md after live capacity checks. Do not substitute local performance claims.

## Completion

The primary agent reviews the reuse safety argument and patch, completes any necessary corrections, records exact deterministic results in A1-RESULTS.md, updates the handoff and checklist without claiming the full issue is complete, and commits/pushes task-owned files on the existing experiment branch. Do not merge, deploy, or close issue #126.
