# Phase H — CRC32C scan elimination

## Status

H1 is retained. On the primary 64-writer, width-16, uniform workload it removes about 15.22 full-page CRC32C scans per committed transaction and reduces the sampled CRC category from 11.17% to 6.38%. Sync-disabled throughput improved 1.078x; real-sync throughput improved 1.003x. The durable primary result does not meet the 1.04x full-matrix gate, so the 14-scenario matrix and RocksDB rerun were not run.

The first H0/H1 artifacts in this directory were produced from source-only temporary copies and reported `git_commit=unknown`. They are preserved under `raw/rejected-provenance/`, `hardware/rejected-provenance/`, `perf/rejected-provenance/`, and `correctness/rejected-provenance/` for audit only. All results below use the canonical OCI Git checkout and the detached baseline worktree described below.

## Benchmark provenance and hardware CRC

The OCI canonical checkout is `/home/opc/dodb-wal-v3`, clean at H1 commit `5998f7c47a15850c40bb40c8c378ac5c0deca061`. The only additional source checkout was a temporary detached worktree at `/tmp/dodb-phase-h-baseline`, commit `a78472377fa16ed491c18bd2d6770f7dba7e9311`, used for G0 comparisons and removed after measurements. Each benchmark process ran with that corresponding checkout as its working directory. The runner verifies the source SHA before launch and rejects a raw JSONL row whose `git_commit` differs.

| Variant | Source commit | Binary SHA256 |
|---|---|---|
| G0 | `a78472377fa16ed491c18bd2d6770f7dba7e9311` | `33be90a6707544a7725acdb140da8394c4de5affdd5266675720c61ed7499e83` |
| H1 | `5998f7c47a15850c40bb40c8c378ac5c0deca061` | `0ac99953510c5f0977b96fd2883547140a284719f45458d5a62869ab8c69badc` |

Both binaries were built on the AArch64 OCI host with rustc 1.98.1, target `aarch64-unknown-linux-gnu`, and the repository target setting `-C target-feature=+crc`. `RUSTFLAGS`, `CARGO_ENCODED_RUSTFLAGS`, and `CARGO_BUILD_RUSTFLAGS` were empty. `crc32c` 0.6.8 compiled with `cfg armsimd`; `/proc/cpuinfo` reports `crc32`. `objdump` found `crc32cb` and `crc32cx` in both binaries. The software fallback was not used. Invocation, rustc command excerpts, instruction output, build logs, and hashes are in `hardware/verified/`.

The host was the supplied OCI A1 instance with 2 OCPU, 12 GiB RAM, AArch64 Neoverse N1, and `/bench/zfs/db` on OpenZFS 2.2.11. Paired throughput gates used the same arguments, 2 Tokio workers, 2 parallel workers, 5 seconds measured time, 2 seconds warmup, three repetitions, identical paired seeds, and interleaved order. `perf stat` collected cycles and instructions. The separate G0/H1 sampled profile used 10 seconds with no warmup. Every accepted benchmark row reports the expected commit. Raw rows and run order are under `raw/interleaved/`; `scripts/analyze_phase_h.py` validates the rows and regenerates `tables.md`.

## H0 call-site attribution

The primary attribution workload is 64 writers, width 16, uniform, sync-disabled. The three G0 raw repetitions report a median of 27,404 committed transactions per run. Counts below come from page-encode, page-chain, WAL-record, and parallel-job counters divided by committed transactions. Byte counts use the exact 4,096-byte page size. The CRC implementation is shared and inlined across A/B/C, so the hardware-cycle profile can establish their combined category but cannot reliably split cycles among those source sites. Existing stage timers include adjacent encode or delta work and are shown as timing proxies, not isolated CRC timers.

| Site | Calls / committed tx | Bytes CRC'd / tx | Timing / tx |
|---|---:|---:|---:|
| A. Canonical Blink checksum in `finalize_encoded_page()` | 15.982 | 65,464 | 29.03 µs worker-encode stage |
| B. Full image fingerprint after encode | 15.982 | 65,464 | 32.00 µs worker-delta stage |
| C. Existing base-page chain validation | 15.226 | 62,366 | 27.93 µs worker-base stage |
| D. WAL redo payload checksum | 15.982 | 1,709 bytes including rare image payloads | 1.316 µs payload-CRC timer |
| E. Transaction commit digest | 15.982 record updates; 4 CRC append operations per current-format record | 1,853 payload plus digest metadata | 1.686 µs digest-CRC timer |
| F. WAL frame and commit metadata | About 16 page-frame headers, one commit header, and one 16-byte commit payload | About 831 header/commit-payload bytes | 0.758 µs combined header CRCs; 0.043 µs commit payload |
| G. Open, recovery, checkpoint validation | Not transaction-proportional | Full validation remains enabled | Outside the transaction hot-loop timer |

Before H1 there were 47.191 full 4 KiB CRC scans per committed transaction: A + B + C, or 193,295 full-page bytes per transaction. The final no-warmup profile recorded about 640,068 sampled cycles per transaction and 11.17% CRC32C samples, approximately 71,496 sample-equivalent CRC cycles per transaction. These are profile estimates, not isolated hardware counters for each call site.

## H1 trusted page-chain fingerprint reuse

The internal immutable page wrapper carries the image with its page ID, page LSN, chain fingerprint, and embedded canonical page checksum. Before trusted reuse, the WAL chain metadata and the image's page ID, LSN, and embedded checksum must still agree. Wrong cached LSN, fingerprint, page ID, or a different image with the same page ID and LSN is rejected before WAL append. The checksum value does not replace the existing full-image chain fingerprint and does not change the PageDelta base field.

The optimization removes the second full-page scan used to validate the previous immutable chain image. G0 performed 15.226 such scans and hashed 62,344 bytes per committed transaction. H1 performs no full-page CRC for that validation; the O(1) metadata checks remain. Canonical encoding, the new image fingerprint required by the existing chain semantics, all untrusted-byte validation, and every persisted checksum remain unchanged.

| Metric, 64w width16 uniform | G0 | H1 | Change |
|---|---:|---:|---:|
| Full-page CRC scans / tx | 47.191 | 31.965 | -15.226 |
| Full-page CRC bytes / tx | 193,295 | 130,929 | -62,366 |
| Profile CRC cycles / tx, estimated | 71,496 | 39,970 | -44.1% |
| Profile total cycles / tx | 640,068 | 626,483 | -2.1% |

The paired CPU gate medians were 5,707 → 6,150 tx/s (1.078x) sync-disabled, 4,461 → 4,473 tx/s (1.003x) real-sync, and 23,134 → 23,066 tx/s (0.997x) for 64w width1 real-sync. Real-sync width1 remains above the required 0.98 floor. Full paired results, cycles per transaction, raw rows, and run order are in `tables.md` and `raw/interleaved/`.

The 12-configuration public-API comparison against G0 matches exactly: transaction results, commit LSNs, reads, queries, scans, reopen scans, data-file SHA256, and WAL SHA256. The empty diff is `correctness/g0-h1-byte-identity.diff`; G0 output is the Phase G final output at commit `a78472377fa16ed491c18bd2d6770f7dba7e9311`, and H1 was rebuilt from `5998f7c47a15850c40bb40c8c378ac5c0deca061`.

## H2 digest-combine experiment — rejected

The randomized equivalence test covers 4,096 cases, multiple records, PageImage and PageDelta payloads, small and large lengths, mixed records, multiple payload segments, legacy/current versions, and zero, maximum, and varied record indexes. The combined digest matched the existing digest bit-for-bit after every record.

`crc32c_combine` was rejected on measured cost. The equivalence and timing test ran on the OCI AArch64 host from H1 commit `5998f7c47a15850c40bb40c8c378ac5c0deca061`. For 50,000 records, direct digest update versus payload-CRC reuse plus combine took 73 ns vs 8,510 ns at 159 bytes, 545 ns vs 17,737 ns at 4,079 bytes, and 543 ns vs 18,725 ns at 4,104 bytes. The combine polynomial work is much more expensive than scanning these payloads a second time. No production digest code changed; WAL bytes remain identical.

## H3 incremental canonical checksum — skipped

The final H1 profile attributes about 1.1% of samples to the canonical page checksum path, below the 5% CPU prerequisite. H3's required profile gate fails, so no incremental checksum prototype was implemented.

## Durable gate and next bottleneck

The six required real-sync scenarios were run with three interleaved repetitions each. The 64w width1 result is 0.997x G0, above its 0.98 floor. The 64w width16 uniform result is 0.987x, below the 1.04 full-matrix gate. Therefore no 14-scenario matrix or same-session RocksDB comparison was triggered. The previous Phase F RocksDB ratios are historical only and are not presented as a Phase H result.

The largest remaining storage-owned single category in the H1 profile is `encode_page_delta()` at 9.85% of sampled cycles. It exceeds the remaining CRC32C category at 6.38%. This is the one next bottleneck selected from the final profile.

The production executable allocator item remains open: this checkout's `dodb-server` is library-only, so the eventual production binary must select mimalloc at its binary boundary. `dodb-storage` has no global allocator.

## Correctness and artifacts

`cargo test --workspace --release --no-fail-fast` passed on H1, including WAL corruption, torn WAL/checkpoint, PageDelta base validation, commit-digest checks, parallel differential, pinned-generation immutability, and packed-leaf differential suites. The focused trusted-image test also checks wrong LSN, fingerprint, image association, and page ID. The public API byte-identity comparison passed across all 12 configurations.

`hardware/`, `raw/`, `perf/`, `correctness/`, and `scripts/` retain verified inputs, raw measurements, profiling data, correctness logs, and reproducible analysis scripts. Rejected-provenance files are preserved separately and are excluded from all tables. `SHA256SUMS` covers the full result directory except itself.
