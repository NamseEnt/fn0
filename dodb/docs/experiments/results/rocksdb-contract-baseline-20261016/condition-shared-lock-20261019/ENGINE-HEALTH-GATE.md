# Engine Health and New-Key Smoke Gate

The opt-in RocksDB property observer and sparse `log1024` report output passed the release gate at source `ee6cd94f1a4e7b2bab7af603bc8983bbf8184542`. The worktree was clean on OCI. The release binaries were built from that exact source with Rust 1.97.1 and the pinned RocksDB 11.8.1 archive.

The release gate passed 49 tests: 28 library tests, 9 `service_workload` tests, 4 `unconditional_write_workload` tests, 1 history contract, 4 process-crash contract tests, 2 read contract tests, and 1 transaction contract. The test command and environment are preserved in `engine-health/evidence-ee6/gate/condition-health-release-gate-ee6cd94f`. The two release binary hashes are:

- `service_workload`: `7c8b35679df63402b3454067d720c53e1a194f5be6fe459eb7bec35862bdd4b5`
- `unconditional_write_workload`: `97b2b38717679bba4000a9eed2bf8c2435114e3d7fa5717e9727381e3586e3f4`

The four RocksDB health runs produced 4–5 complete samples at a 1,000 ms interval. Across 34 observed database samples, each of the nine requested properties reported a value; no property query was unsupported or failed. The direct-core four-tenant smoke reported exactly `tenant_1` through `tenant_4`. The Blink run correctly reported health sampling as unsupported while retaining its host and process metrics.

The new-key `rw50`, 16-client, width-16-enabled smoke passed once for Blink, RocksDB service with shared locks and 16 admission slots, and the synchronous core with shared locks and 16 admission slots. All final-state and reopen oracles passed, and none reported unexpected or verification errors. Preparatory Get handling now accepts both Present and Missing responses and carries the returned revision into the following condition. Intended stale-revision failures avoid the observed token, including revision zero, and intended NotExists failures target a seeded key outside the workload's Delete range. The reports retain distinct intentional-rejection and natural-conflict counters. The run does not count how many preparatory Gets returned Missing.

The raw unconditional smoke used 64 clients, width 16, and 256 preseeded slots per client. It acknowledged 11,741 measurement batches, sampled raw RocksDB properties five times, and passed final-state and reopen validation. Its report states that condition-key locking is not applicable because raw writes have no conditional transactions.

The smoke runner also checked every emitted sparse histogram: schema version, sorted unique nonzero bins, bin-count sum, and reconstructed p50/p95/p99 upper bounds all matched the reports. For successful cases, the run preserved each database file's size and SHA-256, copied the actual RocksDB `LOG` contents, verified the copied LOG hashes, and only then removed the database. Reports, original stdout/stderr, manifests, preflight, and the exact remote runner are under `engine-health/evidence-ee6`.

This was a correctness and observer-coverage smoke, not a performance comparison. The approved A/B/C/E 30-second matrix has started at the same source and fixed binary hashes; cohort D remains pending. No five-minute case has started. See [the live plan and run evidence](engine-health/LONG-MATRIX-PREPARATION.md).
