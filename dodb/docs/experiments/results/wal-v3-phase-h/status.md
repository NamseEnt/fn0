# Phase H status

**Decision:** retain H1 trusted full-page fingerprint reuse. Reject H2 digest combine. Skip H3 incremental checksum because canonical page CRC is about 1.1% of sampled cycles, below the 5% prerequisite.

**Valid comparison:** G0 `a78472377fa16ed491c18bd2d6770f7dba7e9311`, binary SHA256 `33be90a6707544a7725acdb140da8394c4de5affdd5266675720c61ed7499e83`; H1 `5998f7c47a15850c40bb40c8c378ac5c0deca061`, binary SHA256 `0ac99953510c5f0977b96fd2883547140a284719f45458d5a62869ab8c69badc`. Every accepted raw benchmark row self-reports its source commit. Hardware CRC instructions were verified in both binaries.

**Primary result:** H1 removes 15.226 full 4 KiB CRC scans per transaction (62,366 bytes/tx). Sampled CRC cost falls from about 71,496 to 39,970 cycles/tx; total profile cycles fall 2.1%. The three-run 64w width16 uniform gate is 1.078x G0 sync-disabled and 1.003x real-sync.

**Durable gate:** six scenarios × three paired repetitions completed. 64w width16 uniform is 0.987x G0, below the 1.04 full-matrix threshold. Both width1 real-sync protections pass. The 14-scenario matrix and RocksDB rerun were not triggered.

**Correctness:** `cargo test --workspace --release --no-fail-fast` passed. Deterministic public API checks match results, commit LSNs, reads, queries, scans, reopen scans, data bytes, and WAL bytes across 12 configurations. H2 randomized digest equivalence passed before its performance rejection.

**Next bottleneck:** `encode_page_delta()` at 9.85% of H1 sampled cycles, the largest remaining storage-owned category.

Older source-only artifacts with `git_commit=unknown` remain in `rejected-provenance/` and were excluded from all conclusions. Full tables, files, and checksums are in this directory.
