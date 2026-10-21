# Measurement accounting corrections

## Scope and provenance

The performance matrix remains the 24 original runs from measurement source `876d5a1b4b7c0b2792efc27237fa3367cd8d1d9d`, using the original executable hashes. The implementation correction is commit `817f4173e90b9a6c0dbae0fde263ea3876915719`. No performance run was repeated, replaced, or relabeled as a measurement of the corrected source.

`original-evidence-sha256.json` records the SHA-256 of all 367 pre-existing files under `oci/`, including raw and enriched reports, controller records, manifests, failed attempts, and historical summaries. The verifier checks every recorded hash before producing `../oci/matrix-verification-corrected-v4.json`. That derived summary also records the verifier hash and the original v3 summary hash. Original measured and corrected derived counter values are separate fields.

## WAL file sizes

The old Python collector matched file names containing `wal`; both Rust workload collectors matched only the `.wal` extension. All missed RocksDB's ASCII-numeric `.log` WAL files. The corrected collectors recognize `.wal` and ASCII-numeric `.log` names and exclude diagnostic logs and workload output.

Historical RocksDB peak WAL size is unavailable, not zero. The missing filename-level observations cannot be reconstructed from total directory size. The v4 aggregate therefore uses JSON null, and RESULTS.md labels the peak unavailable. A separate `retained_wal_bytes_after_reopen` field sums WAL file sizes from each saved database manifest. It is explicitly after reopen and is never substituted for a measurement peak.

## WAL reset synchronization

Physical and logical WAL resets already synchronized truncation and then reinitialization. Only the second successful sync incremented the counter. Both now count the first successful sync immediately, including when a later fault prevents reset completion. No sync was added, removed, or reordered.

All measured physical checkpoint events succeeded, none were omitted, and the original counters equal the successful event counts. V4 validates these preconditions and adds one omitted truncation sync per successful checkpoint. Median physical checkpoint WAL syncs are A 32 MiB: 30, A 128 MiB: 6, E 32 MiB: 54, and E 128 MiB: 14. Commit sync counts are unchanged. DB-file synchronization is not included in these WAL-only counts.

## E throughput denominator

The E runner waited for final coordinator activity but omitted that final checkpoint wait from throughput's denominator. It now records fractional-millisecond final checkpoint drain and reports total elapsed milliseconds and denominator seconds including the wait. Regression tests cover zero, 1,250 ms, and submillisecond drain.

V4 recomputes historical E throughput from acknowledged batches divided by original admission-plus-last-completion time plus recorded final checkpoint drain. All twelve historical E drain fields are zero integer milliseconds, so their reported throughput is unchanged. Lost submillisecond precision is not reconstructed or presented as a new measurement.

## Repeated-checkpoint evidence

The old E verifier assumed one final queued group from client count and group capacity. V4 instead uses a lower bound based on serial checkpoint events longer than the complete drain interval, including a 2 ms allowance for original integer-millisecond rounding, and subtracts one possible deadline-crossing checkpoint. Short events that could fit in drain do not qualify. Every threshold E run still has at least six qualifying checkpoints; all 12 threshold cells retain at least three. Insufficient repeated-checkpoint evidence now causes verifier failure as well as an explicit status.

## Interpretation

Throughput and latency aggregates are unchanged. A's 32 MiB policy has a 0.6% higher median than minimal-native, but their three-run ranges overlap; this is not an established throughput advantage. The workload-dependent conclusion remains: synchronous checkpoint reclamation removes B-link's measured advantage on E. The comparison still has no equal memory budget and does not establish long-term steady state.

## Regression validation

Local validation passed 24 WAL tests, 6 checkpoint integration tests including process termination, 1 checkpoint fault matrix, and 5 Python correction tests. OCI validation at the correction implementation commit passed 24 driver, classification, history, read-contract, and transaction-contract tests after a successful monitored release test build. See ../VALIDATION.md, local-validation.json, oci-build/, oci-helper-run/, and oci-tests/ for commands, results, provenance, and resource bounds.
