# Phase K Status

## Decision

**Reject the committed-overlay architecture.** Background materialization improves the primary 64-writer width-16 uniform throughput over synchronous J by 1.181x, but it misses the 1.30x K/J target and reaches only 0.961x H1. Width-1 protection is far below the required 0.95x: K/H1 is 0.229x at 64 writers and 0.181x at 16 writers. The primary writer-blocked reduction is 61.99%, below the 90% target. Overlay hard-limit backpressure and retained WAL remain substantial.

No sustained 120-second run or RocksDB rerun was performed because the required H1 gates failed. The experiment does not establish bounded long-run WAL or RSS.

## Repository and Provenance

- Canonical branch: `experiment/b-link-batched-engine-monorepo`
- Canonical worktree: `/Users/namse/fn0-dodb-blink-experiment`
- Phase K benchmark source SHA: `08e82ec484bd62aeb659471a39f317b86cfb488f`
- Source path: `dodb/`
- OCI target: `opc@217.142.246.204`, VM.Standard.A1.Flex, AArch64 Neoverse-N1, 2 OCPU
- Filesystem: ZFS dataset `dodbbench/db`, mounted at `/bench/zfs/db`
- Phase K benchmark binary SHA256: `1b65b5f66a64e12251f4a39cef570047cad125371fc2cc29134b24264593f0b6`
- Phase K results use the fn0 monorepo source SHA, branch, subtree path, host, and binary hash in raw run-order files.
- Main checkout was not modified.

## Materializer and Primary Gate

The worker pins a base generation plus an immutable overlay prefix, builds and syncs a replacement B-link base on one thread, and publishes only when base identity and exact prefix compatibility hold. A newer suffix is preserved. WAL reset remains restricted to a durable watermark covering the current WAL tail.

The three-run 64-writer width-16 uniform J/K diagnostic measured 6.00 s median writer blocking for J and 2.28 s for K, a 61.99% reduction. K still generated 320 median backpressure events and spent 2.58 s in backpressure as its overlay count reached the hard limit of eight. K materialization took 8.33 s cumulative, including 4.05 s CPU, 2.15 s data sync, and 0.61 s checkpoint sync. Cumulative publication time was 68.2 ms over the interval; per-publication percentiles are not available.

## Durable Matrix

The three-repetition H1/J/K matrix completed for all six workloads with real sync on the OCI ZFS dataset. Primary throughput was H1 5,815.3 tx/s, J 4,711.1 tx/s, and K 5,568.9 tx/s; paired ratios were K/J 1.181x and K/H1 0.961x. Width-1 ratios failed badly. Compact and spread K/H1 were 0.346x and 0.390x.

The matrix H1 selector is the retained `parallel-blink` H1 physical path from the same Phase K source and binary. The historical Phase I source snapshot could not be built against current monorepo storage contracts. Historical H1 values remain documented in Phase J results. No standalone repository SHA is used as the Phase K source provenance.

K materialized 870.0 MB of B-link data and appended 151.7 MB WAL in the primary 10-second run. It reclaimed no WAL bytes and retained a median 186.8 MB WAL at run end. The matrix measured 36 materializations/s and 144 consumed segments/s while reaching the eight-segment overlay cap. Checkpoint byte counts and separate H1 physical bytes are not exposed, so total write amplification is reported as a known subtotal rather than a complete engine total.

See `tables.md` and `write-amplification.md` for all six workloads and ratios.

## Read Regression

On the same OCI source SHA, the bounded-overlay read microbenchmark recorded maximum four-overlay J0/H1 ratios of 1.407x GET, 1.750x Query limit 8, and 1.705x Scan limit 8. Phase J values were 1.459x, 1.745x, and 1.710x. The read path did not materially regress.

## Correctness and Formatting

On the OCI A1 host, the requested release gate passed for `dodb-core`, `dodb-storage`, `dodb-testkit`, `dodb-service`, `dodb-protocol`, `dodb-client`, `dodb-server`, and `dodb-soak`: 314 tests passed, zero failed, and four existing tests were ignored. The bounded-overlay read microbenchmark also passed. `cargo fmt --check --manifest-path dodb/crates/dodb-storage/Cargo.toml` passed on OCI.

Deterministic concurrency tests cover commits during a paused materializer, same-key updates, suffix preservation, pinned readers, and a pre-checkpoint write failure. The exhaustive per-stage crash/fault matrix, stale-result injection, and repeated-request collapse tests remain unimplemented and are not claimed as covered.

## Next Bottleneck

The materializer cannot stay ahead of overlay arrivals under the measured workload. The eight-segment hard cap converts that lag into foreground backpressure; the retained WAL also grows because K reclaimed no prefix while writes continued.

## Phase Boundary

The Phase K code source SHA remains `08e82ec484bd62aeb659471a39f317b86cfb488f`. This status records measurements and limitations only. No Phase L work is included.
