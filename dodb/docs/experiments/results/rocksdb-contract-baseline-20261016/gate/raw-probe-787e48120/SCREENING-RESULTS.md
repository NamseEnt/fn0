# Raw RocksDB and Full-API Short Screening Results

This is a diagnostic screening result, not an authoritative performance comparison. All 60 cases passed the per-case checks at source `5d772a7448b1b6c29a0f00f501bd51fd83be03b9` and binary SHA-256 `8ee11c9afeff6c3687643ad14d6f1c35597d984f988d4f8290935547566d8d48`. Each profile has three serial repetitions, each with a 1-second warmup and 5-second measurement. The runner’s final print failed after all measurements because Python 3.9 rejects `flush` in `json.dumps`; [the original output](raw-screen-ssh-output.txt) and original metadata are preserved. A separate postprocessor validated the existing 60 reports and wrote corrected completion metadata; no measurement was repeated.

Every run used seed `20261013`, 512-byte values, 256 prepopulated slots per client, no compression, synchronized WAL, full-window timing, and the same canonical user-key prefix and paired input fingerprints. Widths were 1 and 16; client counts were 16 and 64. The raw path is an unconditional subset and explicitly makes no DODB revision, OCC, or ABA claim. Full-API paths were synchronous core with reserved ranges, RocksDB service with reserved ranges and the listed shared-slot setting, and B-link.

Rates below are acknowledged write batches per second. Each value shows median [minimum–maximum] across three repetitions; full-API ratios are median(path) / median(raw) for the same clients and width. RSS is peak sampled RSS median [minimum–maximum] in MiB (1024 KiB).

| Clients | Width | Path | Shared slots | Batches/s median [min–max] | Relative to raw | Peak RSS MiB median [min–max] |
| ---: | ---: | --- | ---: | ---: | ---: | ---: |
| 16 | 1 | raw-rocksdb | — | 7977.8 [7844.4–8257.9] | 1.000× | 50.9 [50.9–51.3] |
| 16 | 1 | synchronous-core | — | 7413.8 [7083.0–7832.8] | 0.929× | 50.2 [49.0–51.6] |
| 16 | 1 | rocksdb-service | 16 | 7071.4 [6980.4–7268.3] | 0.886× | 49.1 [47.9–49.2] |
| 16 | 1 | blink | — | 8087.6 [7988.0–8089.4] | 1.014× | 37.4 [37.4–38.5] |
| 16 | 16 | raw-rocksdb | — | 2760.7 [2676.9–2833.9] | 1.000× | 143.7 [135.0–160.4] |
| 16 | 16 | synchronous-core | — | 2056.6 [1954.0–2161.5] | 0.745× | 247.0 [242.4–247.4] |
| 16 | 16 | rocksdb-service | 16 | 1949.0 [1941.6–2015.2] | 0.706× | 216.8 [213.9–220.4] |
| 16 | 16 | blink | — | 2054.2 [2023.8–2079.1] | 0.744× | 274.3 [272.7–274.6] |
| 64 | 1 | raw-rocksdb | — | 17905.6 [17776.9–18266.8] | 1.000× | 113.4 [113.3–113.6] |
| 64 | 1 | synchronous-core | — | 16571.4 [16435.3–16598.8] | 0.925× | 119.7 [118.0–123.6] |
| 64 | 1 | rocksdb-service | 16 | 6639.5 [6614.5–6829.3] | 0.371× | 78.8 [78.4–79.6] |
| 64 | 1 | rocksdb-service | 32 | 11330.6 [11118.0–11429.1] | 0.633× | 95.8 [94.9–98.5] |
| 64 | 1 | rocksdb-service | 64 | 15088.1 [13720.6–15138.7] | 0.843× | 110.1 [108.4–116.0] |
| 64 | 1 | blink | — | 14734.2 [14075.9–15005.4] | 0.823× | 105.4 [105.2–106.8] |
| 64 | 16 | raw-rocksdb | — | 2391.9 [2107.4–2696.4] | 1.000× | 372.4 [365.6–378.3] |
| 64 | 16 | synchronous-core | — | 1753.8 [1530.5–1891.2] | 0.733× | 468.6 [439.1–473.4] |
| 64 | 16 | rocksdb-service | 16 | 1750.6 [1736.2–1795.1] | 0.732× | 436.0 [429.1–439.9] |
| 64 | 16 | rocksdb-service | 32 | 1760.2 [1751.5–1831.0] | 0.736× | 441.3 [441.1–445.1] |
| 64 | 16 | rocksdb-service | 64 | 1721.9 [1693.5–1767.2] | 0.720× | 441.9 [438.1–453.1] |
| 64 | 16 | blink | — | 2864.5 [2743.3–2865.2] | 1.198× | 985.6 [836.0–996.5] |

All cases recorded full 5-second measurement windows, and each case’s latency sample count matched its acknowledged write-batch count. Final and reopened state matched the expected row count and values; the full-API paths also matched transaction revision groups. The raw path was checked against its explicit no-logical-revision/OCC/ABA claim boundary. The first 100 measurement input batches per client had matching key/payload fingerprints across paired paths. Every case recorded CPU time, RSS samples, disk use, source and binary provenance, and a report/command/output/exit status. Per-run reports, DB-file hashes, LOG content, corrected aggregate, and manifests are preserved under [screen-r1-artifacts](screen-r1-artifacts/).

The result does not justify a default change or establish a maximum RocksDB baseline. It is specific to these short overwrite workloads and this host. No 30-second confirmation or five-minute measurement has run. The condition-key shared-lock diagnostic is a separate mixed-workload investigation; this unconditional raw screen does not measure that lock cost. See the proposal-only [condition-key shared-lock note](../../../../rocksdb-condition-shared-lock-proposal.md).
