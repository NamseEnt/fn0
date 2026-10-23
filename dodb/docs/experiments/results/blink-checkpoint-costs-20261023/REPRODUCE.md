# Reproduction

Use the designated OCI host `opc@217.142.246.204` with `/Users/namse/.ssh/id_late_sh_ed25519`, `IdentitiesOnly=yes`, and normal host-key validation. The experiment root is `/bench/zfs/db/blink-checkpoint-costs-20261023`. Its clean detached `source` checkout is frozen at `cc80d1fdc7a98792be1a739056a3013debf39268`; `bin` contains frozen, hashed workload executables. The previous implementation uses `/bench/zfs/db/blink-durable-bases-20261023-primary-budget/source` at `400ad0ac6b7d8e6306bcb2ab7946747bddcb06c4` and that root's original hashed `bin` files.

The source/environment and build records identify the Git checkout, Rust/Cargo 1.97.1, AArch64 `+crc`, pinned RocksDB 11.8.1 source and archive, sccache, locked dependency graph, host and ZFS dataset properties. Builds and runs use fresh unique evidence roots; do not overwrite accepted roots or reuse build IDs.

From the clean frozen checkout, the build, twenty-executable functional validation, six-cell 8-second preflight and eighteen-cell 60-second matrix run through:

```sh
PYTHONDONTWRITEBYTECODE=1 \
DODB_COST_CANDIDATE_ROOT=/bench/zfs/db/blink-checkpoint-costs-20261023 \
python3 dodb/docs/experiments/results/blink-checkpoint-costs-20261023/run_pipeline.py
```

The pipeline reuses the existing monitored OCI build and validation helpers without changing storage defaults or workloads. Its `run_matrix.py` records all commands, process-group resources, source identities and executable hashes. RW80 has sixteen dedicated callers, an 80:20 API read/write mixture, 512-byte values, conditions, revisions and width-1/4/8/16 writes. E has sixty-four dedicated callers and unconditional width-16 overwrite batches. Both retain seed 20261013, real synchronous WAL and the existing admission/grouping policies. B-link uses coalesced immutable bases with background execution, a 128 MiB checkpoint trigger and 32 MiB WAL segments. RocksDB is the independent DODB-contract TransactionDB baseline, including logical revisions and conditional/multi-key semantics, not raw WriteBatch. Its native scheduling/admission and B-link's coordinator/grouping remain their existing distinct implementation paths.

The outside-checkout `summarize.py` and `run_followup.py` are copied from this report's scripts before follow-up execution. Keeping them outside the frozen checkout preserves its measured Git identity. The follow-up controller waits for successful short-stage completion, independently reduces preflight and short reports, records the short-summary hash and predeclared gate outcomes, and runs the 300-second candidate/RocksDB matrix only if every gate passes. Gate failure is retained as an explicit skipped sustained stage.

The exact controller invocation is:

```sh
PYTHONDONTWRITEBYTECODE=1 \
DODB_COST_CANDIDATE_ROOT=/bench/zfs/db/blink-checkpoint-costs-20261023 \
python3 /bench/zfs/db/blink-checkpoint-costs-20261023/run_followup.py
```

The reducer can also run independently with `DODB_COST_ACCOUNTING_HELPER` set to the frozen checkout's `dodb/docs/experiments/results/blink-background-checkpoint-20261021/performance-loss-20261022/analyze_loss.py`, passing a completed stage root as its argument. It reads existing evidence and never launches a workload. The final report keeps original raw data, corrected summaries and any rejected attempts distinct.

## Final archive and offline verification

After all measurement processes exit, copy `archive_evidence.py` outside the frozen checkout and invoke it with the same `DODB_COST_CANDIDATE_ROOT`. It rechecks both source checkouts and every frozen binary, verifies that the workload/native/server/core/service/protocol sources did not change, copies controller and prior-build provenance, and calls the existing archive helper. Compression runs only after all performance cases finish.

The resulting `blink-checkpoint-costs-20261023-evidence.tar.gz` contains 539 inventoried files and 292,945,693 uncompressed bytes, excluding source/executable/database payloads as recorded in the sidecar. Its 18,647,192 compressed bytes have SHA256 `1769daa047eee863a7656365e870fb88ec324c59d594642e7e43a38ae4945795`. Local raw validation and baseline-Clippy logs are separately preserved in `evidence/local/local-validation-logs.tar.gz` with individual original-file hashes.

From this report's fn0 checkout, reproduce the offline verification without running workloads:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 \
  dodb/docs/experiments/results/blink-checkpoint-costs-20261023/verify_results.py
```

This checks the archive SHA256, every archived original file, source/binary identities after measurement, normalized commands, cross-stage fixed input prefixes, complete latency counts, final/reopen oracles, resource/overlap/coverage gates, all twenty functional logs and the five local raw logs. It extracts into a temporary directory, reproduces every original summary object exactly, and verifies the original short-summary selection hash before any temporary reserialization. All 36 accepted cells reproduce; no measured run is repeated. `evidence/offline-verification.json` records the result.
