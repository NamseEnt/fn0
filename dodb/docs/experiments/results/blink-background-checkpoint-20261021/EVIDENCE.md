# Concurrent checkpoint evidence inventory

## OCI archives

All archives are stored under `evidence/oci-concurrent/`. They preserve original output bytes and relative remote paths. Frozen workload executables and database contents are not embedded in Git; their paths, sizes, SHA256 identities, final/reopen checks, and successful database file manifests are retained. Frozen executables remain on the approved OCI host. Successful databases were removed only after validation and filename/size/SHA256 capture.

| Archive | Contents | Files | SHA256 |
| --- | --- | ---: | --- |
| `blink-background-checkpoint-20261022-evidence.tar.gz` | Initial 30-cell matrix, ten preflight cells, raw/enriched reports, commands, resource samples, database manifests, build/validation outputs, retained rejected attempts, initial environment, binary provenance, independent summary | 532 | `bc0bcb67404ad868ad0a015a22eee1ce1a3650053e9fd90aefc1602e36836140` |
| `blink-background-checkpoint-20261022-threshold-evidence.tar.gz` | Corrected 30-cell matrix, raw/enriched reports, commands, resource samples, database manifests, final build/314-pass validation, clean final source/environment, frozen binary provenance, independent summary | 380 | `8e80f03fd6f3a65f9b8192e536c37a797706630605f85b256c5a2c11d4df1f56` |
| `blink-background-checkpoint-build-provenance.tar.gz` | Existing build helper's seven task run directories: selected environment, toolchain/archive/config hashes, compiler output/status, sccache observations, time, and original checksum manifests | 63 | `4994037ea71198990e6879ef85492f25438b929d497a786f897e2aefbc0588d3` |

The first two archives contain `evidence-file-sha256.json` inventories for every other archived file. `blink-background-checkpoint-archive-summary.jsonl` records archive size, checksum, and file count. Local `SHA256SUMS` covers archives and extracted summaries; `archive-verification.json` records byte-for-byte inventory verification. Extracted initial/final matrix summaries must equal their respective archived versions.

`archive_oci_evidence.py` generates inventories and archives after both completion markers exist. It prunes `source`, `bin`, and `databases` directories and does not follow source/bin symlinks. `capture_oci_environment.py` checks the final exact commit and clean Git status and captures final source aliases, filesystem properties, compiler, RocksDB archive, and executable identities. Both run only after measurement has finished, so archival activity does not overlap performance cells.

The initial `environment.json` is an early environment/build gate and records an intermediate source commit. Its source field does not identify the later initial matrix binary. Each accepted cell's compile-time Git identity, source record, executable SHA, and matrix plan identify its implementation. The final source worktree reuses the initial source directory at a newer commit; initial frozen binaries, raw reports, build environment, and command records preserve the initial implementation identity independently of that live alias.

## Local and model evidence

| Directory | Coverage |
| --- | --- |
| `evidence/concurrent-local/` | Initial complete storage/server suites, focused overlap/shutdown/recovery, source hashes, finite model, Clippy baseline classification, fixture/build failures |
| `evidence/coalesced-threshold/` | Failing-before and passing-after threshold regression, final 18-function focused selection, source identities and checkpoint-count observation |
| `evidence/final-format/` | Passing format check for the three changed packages and repository-wide failure confined to 11 existing vendor files |
| `evidence/segmented-checkpoint/` | Earlier synchronous segmented checkpoint milestone, preserved separately |

Every directory has its own SHA256 manifest. Rejected output remains distinct from clean accepted validation. Functional suite selections overlap and must not be summed. The 19 injected failure boundaries and ten real SIGKILL boundaries are scenarios inside three test functions, not 29 additional independently counted test functions.

## Rechecking reports

Run `python3 verify_archived_evidence.py` to verify archive/inventory checksums and rederive both summaries offline without changing the stored raw reports. The checked-in `archive-verification.json` records successful reproduction for both 30-cell matrices.

Extract each OCI matrix archive into a separate temporary directory, then run `summarize_matrix.py` with the extracted run root. It requires only the preserved reports/plans/manifests, not the executable or deleted database. It checks source/binary identities, raw/enriched hashes, expected cell set, every histogram count, fixed input prefixes, outcomes, repeated checkpoints, actual overlap, final/reopen checks, zero remaining work, and sampled resource bounds.

The complete workload commands remain in each `reports/<case>/command.json`. Resource samples are in `resource-samples.json`; database contents are represented by `database-manifest.json`. Build and validation commands are stored beside their original outputs. No accepted cell was rerun solely to repair derived reporting.
