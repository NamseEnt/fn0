# Query, Get, Put, and Conditional Put Results

## Result

For this fixed 40/30/15/15 mixed workload, the `minimal-native` DODB-contract
RocksDB adapter completed more total API calls: **20,739 calls/s median**,
versus **19,068 calls/s** for B-link, an **8.8% advantage**. Ordered Query16
latency favored B-link: median per-run p95 was **0.036 ms** for B-link and
**0.087 ms** for RocksDB. Get p50 and p95 also favored B-link, while Get p99
favored RocksDB. The result is workload-specific and does not establish a
general engine ranking.

The workload asked for up to 16 ordered rows, but the seeded key layout has at
most ten sort keys per partition. Each accepted run returned about five rows
per Query16 call on average. Interpret the Query numbers as performance for
this actual result size, not as a 16-row page benchmark.

## Sustained measurements

Each entry is the median across three accepted 300-second runs. The ranges are
the minimum and maximum observed run values. Latency entries are medians of
the three runs' respective percentiles.

| Metric | B-link | `minimal-native` RocksDB |
| --- | ---: | ---: |
| Total API calls/s | 19,068 (18,832–19,078) | 20,739 (20,686–21,043) |
| Query16 calls/s | 7,628 (7,533–7,632) | 8,296 (8,275–8,418) |
| Get calls/s | 5,720 (5,649–5,723) | 6,221 (6,205–6,312) |
| Successful Put/s | 2,860 (2,825–2,862) | 3,111 (3,103–3,156) |
| Successful conditional Put/s | 2,860 (2,825–2,862) | 3,111 (3,103–3,156) |
| Query16 rows/call | 5.000 | 5.000 |
| Peak process-group RSS | 270.2 MiB (266.1–294.5) | 523.5 MiB (503.6–542.9) |
| Maximum database size | 143.9 MiB (143.4–146.7) | 69.3 MiB (69.2–73.2) |
| Reopen and oracle validation | 1,998 ms (1,996–4,840) | 2,412 ms (2,209–2,432) |

### Latency percentiles

| Operation | Backend | p50 | p95 | p99 |
| --- | --- | ---: | ---: | ---: |
| Query16 | B-link | 0.014 ms | 0.036 ms | 0.404 ms |
| Query16 | `minimal-native` | 0.021 ms | 0.087 ms | 0.438 ms |
| Get | B-link | 0.004 ms | 0.013 ms | 0.347 ms |
| Get | `minimal-native` | 0.007 ms | 0.021 ms | 0.044 ms |
| Put | B-link | 2.836 ms | 3.777 ms | 5.317 ms |
| Put | `minimal-native` | 2.374 ms | 3.817 ms | 5.231 ms |
| Conditional Put | B-link | 2.839 ms | 3.774 ms | 5.292 ms |
| Conditional Put | `minimal-native` | 2.384 ms | 3.858 ms | 5.272 ms |

## Workload and validation

- One tenant, 16 dedicated clients, 10,000 seeded 512-byte values, seed
  `20261013`, ten-second warmup, and three 300-second repetitions per backend.
- Each client used the same deterministic schedule: 40% ordered Query16, 30%
  ordinary Get, 15% unconditional Put, and 15% one-key conditional Put. The
  conditional Put checked `Exists` and wrote the same key; every condition
  succeeded. All observed operation shares stayed within 0.01 percentage point
  of their target.
- All six accepted runs used the same source and binary hashes. Per-client
  input-prefix fingerprints matched. Query ordering, partition and value
  checks, final state, and reopen state passed. There were no overloads,
  request errors, natural conflicts, preparatory Gets, or rejected conditions.
- B-link checkpoint runs completed 8, 9, and 9 checkpoints during measurement.
  At least 7, 9, and 9 of those respectively overlapped acknowledged writes;
  checkpoint work drained before validation.
- All six databases were archived and verified file-by-file against their
  database manifests. The archive hashes and per-run metrics are in
  [`sustained-summary.json`](sustained-summary.json). The eight-second
  correctness/resource preflight is in [`preflight-summary.json`](preflight-summary.json);
  those smoke measurements are not used in the comparison above.

## Provenance and retained attempts

- OCI AArch64 host, two vCPUs; ZFS dataset `dodbbench/db` at
  `/bench/zfs/db`; RocksDB 11.8.1; Rust 1.97.1 with ARM64 `+crc`; synchronous
  WAL; unprofiled execution.
- Measurement source commit: `54812222e95e1335a5e9409d3fddbabf38c0f876`.
- `service_workload` SHA256:
  `6837cc7f6691337ef0794e177db7a1a082474607c3e5595a78067679456c57ad`.
- The measured controller had SHA256
  `6159170e97393a3e59b93a7be856e0cffbbcc8fa401c3ec09c2fb4701d5b6f44`.
  Its resume changes are committed as `f1cbf5608b7a6e0c6075abf14fe7fa323273d52e`.
- The first continuation stopped before launching a cell because ZFS free
  space reached the 10 GiB reserve. A later measured attempt was retained but
  rejected when its source checkout did not match the binary build commit.
  Accepted repetitions were resumed only after restoring matching source and
  binary provenance. See [`runner-initial.log`](runner-initial.log) and
  [`runner-resume.log`](runner-resume.log). Rejected databases and all accepted
  database archives remain under
  `/bench/zfs/db/query-get-put-mix-20261027` or the associated
  `/home/opc/query-get-put-mix-20261027` preservation directory on the host.

These results describe one measured configuration. No defaults were changed.
