# Phase G correctness evidence

## Release tests

- G0 mimalloc attribution and binaries: `cargo test --workspace --release --no-fail-fast` passed; see `correctness/g0-workspace-release.log`.
- G1a, G1b and G1c candidates: OCI workspace release suite passed; see the matching logs in `correctness/`.
- G2 and G2b candidates: OCI workspace release suites passed; see `g2-workspace-release.log` and `g2b-workspace-release.log`.
- G3 two-pass PageDelta encoder: OCI workspace release suite passed in `g3-workspace-release.log`; existing randomized PageDelta, canonical merge-gap, fault-injection, recovery and byte comparison tests passed.
- G4 contiguous transaction-result metadata: OCI workspace release suite passed in `g4-workspace-release.log`; parallel differential, worker failure, WAL append and recovery tests passed.
- `cargo fmt --all -- --check` and `git diff --check` passed for final source.

The G1c arena-reuse test exercised group sizes 1, 64, 3, 128, 2, 32, 1 and 256, then verified direct reads and reopen values. It passed. The G1c candidate did not let arena pointers escape the group call. No arena code is retained in final G.

## Phase F to final G byte identity

The deterministic public-API workload in `crossver/src/main.rs` ran 12 configurations across seeds 11–13, two value limits and serial/two-worker execution. Phase F and final G0 outputs match line-for-line. This includes transaction results, commit LSNs, reads, queries/documents, scans, reopen scans, data-file SHA256/size and WAL SHA256/size. See `crossver/output-phase-f.txt` and `crossver/output-g0.txt`.

## Scope and gated tests

No retained arena candidate passed the primary CPU gate. The 14-scenario matrix, Phase G RocksDB comparison and 120-second RSS run were therefore not triggered. No arena remains in final G, so there is no retained group memory to sample. Phase G's successful OCI durability evidence is the 6-scenario G0 baseline; it is not presented as an arena speedup.
