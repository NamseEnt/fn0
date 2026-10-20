# Minimal Native RocksDB Contract Baseline Plan

## Purpose

Measure a benchmark-only RocksDB implementation that preserves the DODB contract while bypassing the existing `TenantDatabase` transaction execution path. The target is an explicit implementation and configuration baseline, not proof of the absolute maximum RocksDB performance.

## Reuse and bypass boundary

Reuse the public API and request types, deterministic input generators, common correctness/restart contract checks, and result validation where their interfaces permit. Implement transaction execution independently and call RocksDB `TransactionDB` operations directly, including transaction creation, `GetForUpdate`, and `Commit`. Do not route reads or writes through `TenantDatabase`, the existing DODB transaction executor, a new lock manager, or a new transaction engine. Document all reused and bypassed components before interpreting performance.

Preserve conditions (`RevisionEquals`, `Exists`, `NotExists`), condition-only key consistency, multi-key atomicity, one revision for all mutations in a transaction, per-key revision progression, delete/recreate ABA detection, restart-safe revision allocation, Get/Put/Delete/Transact/Query/Scan behavior, per-call query/scan snapshots, sorting/cursors/limits, tenant isolation, response budgets, and synchronous WAL for successful writes. Evaluate the already-reviewed ReservedRanges mechanism for this independent adapter; do not serialize every commit behind a global counter or weaken durability/contract semantics to improve throughput.

## Execution gates

1. Inspect the current final audit and source at `35553ab1906da9e93b6ff129966b970729af7f01`; preserve all existing result and failed-attempt artifacts.
2. Record this plan and the adapter reuse/bypass map before implementing.
3. Implement the independent benchmark path and run the common correctness/restart checks. Do not start performance measurement unless every required contract check passes.
4. Build through the documented OCI helper in bash. Before any build or run, capture active process, memory, disk, host identity, and resource budgets. Never build and benchmark concurrently.
5. On the approved OCI host only, run three repetitions for A (`rw20`, 16 clients) and E (unconditional width-16 writes, 64 clients), each with 10-second warmup and 30-second measurement. Use seed `20261013`, 512-byte values, 10,000 configured rows, and the workload-specific overwrite/preseed settings from the existing reports. Compare the new adapter, the matching existing DODB core, and B-link under paired conditions. Keep raw WriteBatch as a separately labeled semantics-incomplete reference.
6. Report client count, actual execution concurrency, and admission limits separately. Preserve source SHA, executable SHA, commands, configuration, raw reports, p99, CPU, memory, database/WAL size, final-state and reopen validation, and actual RocksDB LOG contents and file size/hash manifests. Preserve failed databases for diagnosis; remove a completed database only after recording its full manifest and logs.
7. Stop the matrix if preflight cannot establish the existing host/data path, available memory reserve, disk headroom, or configured runtime limits. Do not run memory-exceed workloads. Consider longer confirmation only after the short comparison is complete, with a written resource estimate and a reason tied to observed uncertainty or an important gap.

## Interpretation

Attribute differences only to measured path changes after recording concurrency and admission differences. Separate native transactional work, logical revision allocation, condition checking/locking, adapter/API work, and service admission where available. Do not call the narrow raw WriteBatch result an equivalent DODB baseline. Do not relabel historical results as measurements of this source.

## Expected result and trade-offs

The experiment should establish how a minimal direct TransactionDB adapter performs on the selected conditional and unconditional workloads, how it differs from the existing synchronous core and B-link, and which measured path stages explain those differences. The independent adapter adds implementation and maintenance surface and still pays for contract-required revisions, conditions, synchronous WAL, and validation. A short three-run window limits cost and resource exposure but provides less confidence about long-term variance than the prior five-minute subset; a follow-up will be justified only by the observed spread and decision value.
