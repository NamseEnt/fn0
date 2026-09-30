# OCI A1 Btrfs Durable-Write Scaling

## Result

**Classification: B — H1 scales at first, but a substantial gap remains on width-16 uniform writes.** Raising the same A1 VM from 2 to 4 OCPUs increased dodb `parallel-blink` throughput by 29.4% on width-16 uniform writes, compared with 14.7% for RocksDB. At 6 OCPUs, dodb was 29.6% above its 2-OCPU baseline, while RocksDB was 12.5% above baseline. The dodb/RocksDB ratio improved from 0.577x to 0.651x and 0.665x, respectively, but dodb did not catch up. The 4-to-6 OCPU interval produced no meaningful width-16 uniform throughput gain.

The fio durable-sync probe improved by about 10% from 2 to 4 OCPUs and was flat from 4 to 6. That I/O change explains part of the database gains. It does not explain all of dodb's 2-to-4 gain, which was about three times the fio IOPS increase. Conversely, dodb's flat 4-to-6 result occurred while its measured worker parallelism rose from 3.77 to 5.47 and fio remained flat. The remaining width-16 uniform ceiling is consistent with group admission/commit or coordinator work, rather than a continuing storage or available-worker limit.

Locality still matters. At 6 OCPUs, dodb reached 12.3k tx/s on compact and 10.8k tx/s on spread, versus 6.0k tx/s on width-16 uniform. Compact and spread both exceeded RocksDB at 4 and 6 OCPUs. See [the median tables](tables.md) and [derived summary](summary.json); per-run data and guest snapshots are in [`raw/`](raw/).

## OCI and filesystem provenance

- Region and availability domain: `ap-osaka-1`, `IsRG:AP-OSAKA-1-AD-1`.
- Profile used: `OSAKA-VM`; signing-key fingerprint: `92:3a:52:82:31:a4:c4:13:5d:18:8e:87:03:0a:8f:50`. Key contents were not printed or copied into the repository.
- Instance: `ocid1.instance.oc1.ap-osaka-1.anvwsljrrfkd6xqcuiozrid2bkb7uh6ba3p7zkwkip6kcagbwgd5cp7os2jq`, created 2026-09-23, same instance throughout.
- Shape remained `VM.Standard.A1.Flex`; memory remained 12 GB. OCPUs were changed to 2, 4, 6, then 2. OCI-reported network bandwidth followed 2, 4, 6, then 2 Gbps.
- Boot volume retained throughout: `ocid1.bootvolume.oc1.ap-osaka-1.abvwsljrxzh3px7kxsz7w4jzqiqaytbke42huk7rvadeu6y3r4mynxlr3aiq`.
- The Btrfs filesystem is `/dev/sda4` on that boot volume; the OCI API showed no separate attached data-volume attachment. The unchanged VNIC attachment and VNIC IDs were `ocid1.vnicattachment.oc1.ap-osaka-1.anvwsljrrfkd6xqckiggetmzjvxv3db5yxm3755xcomdvybvbksh3kt62kaa` and `ocid1.vnic.oc1.ap-osaka-1.abvwsljrw5qbri2gezbnysvkklwot4ecmb6a52yfrdtptqkeujdtprbpzpfq`. Private/public IPs remained `10.0.0.97` / `217.142.246.204`.
- Btrfs UUID at all four stages: `be3cac1e-6e47-445c-96d4-e5cf7278c97a`; mount: `/bench/btrfs/db`; options: `rw,noatime,seclabel,discard=async,space_cache=v2,subvolid=5,subvol=/`. The filesystem was read/write-probed before each stage. No format, volume lifecycle, volume-size, or performance-tier operation was performed.
- Guest: Oracle Linux UEK kernel `6.12.0-206.104.4.4.el9uek.aarch64`, Ampere Altra Neoverse-N1. Guest CPU counts and affinities were 2 (`0-1`), 4 (`0-3`), 6 (`0-5`), 2-post (`0-1`). fio was 3.35. `lsblk -f`, `findmnt`, `btrfs filesystem usage`, `/proc/cpuinfo`, and `lscpu` snapshots are in the matching `raw/environment-*.json` files.

OCI documents describe shape changes as in-place updates that preserve the instance, volume attachments, VNIC attachments, and IP addresses; a running instance reboots during the change. They also note that A1 OCPUs are physical Altra cores and that flexible-shape network bandwidth scales with OCPUs. See [Changing the Shape of an Instance](https://docs.oracle.com/en-us/iaas/Content/Compute/Tasks/resizinginstances.htm) and [Compute Shapes](https://docs.oracle.com/en-us/iaas/Content/Compute/References/computeshapes.htm).

## Source, binaries, and commands

- dodb source: exact commit `08e82ec484bd62aeb659471a39f317b86cfb488f`, detached clone, clean checkout; runtime rows report `engine=parallel-blink` and `sync_mode=real`.
- dodb release binary SHA-256: `765f6b3b33f7a85dfaebdeb5582eb10c64ad6a3222cf354146fc8fc13fe08a4e`.
- RocksDB source: commit `abeebd9630f11bd08c28b7bd43c7bdfc62050654`; both tags `v11.8.0` and `v11.8.1` point to this commit. The source header reports version 11.8.1; the harness build metadata pins tag `v11.8.1`.
- RocksDB static library SHA-256: `35f3d79559b802e23328818f2511bdb3715b95411567e101250c1b4f03eb0066`. RocksDB release harness SHA-256: `640f0a19924892c8be7745e9f2c618d7db11f97e32090a09f105bb1855c50615`.
- No source architecture was changed. Measurement harness instrumentation resets and records RocksDB WAL sync statistics around the measured interval.

dodb used 64 writers, working set 100,000, 16-byte keys, 64-byte values, 2-second warm-up, 5-second measurement, three repetitions, durable `real` sync, group limit 64, group byte limit 4 MiB, queue capacity 256, unconditional transactions, and no collection delay. `tokio-workers` and `blink-workers` were set to the full guest OCPU count for each stage. The four workloads were width-1 uniform, width-16 uniform, width-16 compact (`same-leaf-heavy`), and width-16 spread (`different-leaf-heavy`).

RocksDB used the same writer, working-set, key/value, warm-up, measurement, repetition, width, distribution, and seed matrix. It used `WriteOptions.sync=true`, WAL enabled, and pipelining off. Each RocksDB result includes reopen verification. The exact per-run command lines and their JSON output hashes are recorded in `raw/run-events.jsonl` and the archived baseline event log.

The same fio command ran at each OCPU count on the disposable Btrfs file `fio/wal-like.dat` (256 MiB), never on a raw device: 4 KiB sequential writes, `psync`, 15 seconds, `fdatasync=1`, time-based. Fio IOPS/bandwidth and the fdatasync latency histogram (`N`, mean, p50, p95, p99) are summarized in [tables.md](tables.md); full fio JSON and logs are under `raw/`.

## Resize and measurement timeline (UTC)

| Transition/stage | Resize request | RUNNING returned | Benchmark start–end | Next resize request | RUNNING-to-next-resize window |
|---|---|---|---|---|---:|
| 2-OCPU baseline | — | — | 02:51:06.190–02:54:55.834 | 03:05:31.093 (2→4) | — |
| 4 OCPU | 03:05:31.093 | 03:07:11.530 | 03:07:24.812–03:11:15.524 | 03:11:16.723 (4→6) | 4m 05.193s |
| 6 OCPU | 03:11:16.723 | 03:12:57.661 | 03:13:13.016–03:17:01.446 | 03:17:03.347 (6→2) | 4m 05.686s |
| 2-OCPU restore and post-check | 03:17:03.347 | 03:18:46.785 | 03:20:58.834–03:23:09.954 | — | — |

The 4- and 6-OCPU windows include SSH readiness, preflight, fio, and the benchmark matrix; no builds or dependency downloads ran at elevated OCPU counts. The failed first transition started at 02:58:14.155, returned RUNNING at 03:00:07.769, failed SSH immediately, and started the restore at 03:00:13.396; it was excluded from headline data. That attempt is retained in `oci-timeline-ssh-delay-preflight.jsonl`. The initial post-check attempt, which stopped on a stale duplicated RocksDB result row, is retained under `raw/preflight-postcheck-retry/`; the accepted 2-post results are the clean, later run. OCI bills shape use to the nearest second according to its documentation, but the table reports measured RUNNING-to-next-update intervals rather than asserting an invoice total.

## Analysis

### Scaling and I/O allowance

On width-16 uniform, dodb throughput was 4,641.7 / 6,006.4 / 6,017.2 tx/s at 2 / 4 / 6 OCPUs. RocksDB was 8,046.1 / 9,227.9 / 9,051.0 tx/s. Thus dodb's 2→4 and 2→6 speedups were 1.294x and 1.296x, versus RocksDB's 1.147x and 1.125x. Dodb's relative ratio improved monotonically, from 0.577x to 0.651x to 0.665x, but the remaining gap is about one third.

The fio sequential write probe measured 394.1 / 433.1 / 430.9 IOPS and mean fdatasync latency 2.513 / 2.280 / 2.296 ms. The similar 4- and 6-OCPU fio results show that storage allowance stopped increasing over this workload, even though OCI's network allowance rose from 4 to 6 Gbps. Dodb's width-16 uniform throughput also stopped increasing from 4 to 6 OCPUs; RocksDB fell slightly. The 2→4 dodb gain exceeds the fio IOPS gain, so it includes additional CPU/parallel-work benefit. It is not evidence of pure CPU scaling because the 2→4 fio latency and IOPS also improved.

### Group commit and CPU use

For dodb width-16 uniform, average transactions per durable group increased from 41.32 at 2 OCPUs to 62.70 at 4 and 62.71 at 6, close to the configured 64-request cap. Durable groups fell from about 113.3/s to 95.9/s and then stayed at 96.0/s. Multiplying group size by group rate predicts about 4.68k / 6.01k / 6.02k tx/s, matching measured throughput within rounding. WAL sync mean was about 3.19 / 3.18 / 3.43 ms, based on total WAL sync duration divided by the WAL sync count.

Across the same stages, effective dodb worker parallelism rose from 1.92 to 3.77 to 5.47, while process CPU was about 4.50 / 5.99 / 6.23 CPU-seconds over the five-second window (approximately 0.90 / 1.20 / 1.25 cores). That is only about 29.9% / 20.7% of the 4-/6-OCPU VM. Physical execution time fell from 1.359s to 1.095s to 0.900s, but planning remained about 0.613s / 0.766s / 0.795s and validation about 0.161s / 0.205s / 0.210s. This points to a saturated serial/group-commit portion after 4 OCPUs even as leaf workers do more parallel work. WAL assembly and generation publication remain small in this workload; all per-workload phase metrics are in [tables.md](tables.md) and the raw JSON rows.

RocksDB's width-16 uniform WAL syncs were about 228 / 277 / 277 per second, with roughly 35.3 / 33.3 / 32.7 transactions per sync. Its mean WAL sync latency was about 3.23 / 2.97 / 3.08 ms. The durable sync count and transaction/group relationship, alongside the fio plateau, are consistent with RocksDB's smaller 2→4 gain being partly I/O-driven.

### Locality and post-check drift

At 2 / 4 / 6 OCPUs, dodb compact width-16 throughput was 9,607 / 12,928 / 12,263 tx/s; spread was 8,290 / 10,911 / 10,783 tx/s. Compact and spread retain clear advantages over uniform, while effective worker parallelism and physical execution costs vary by leaf placement. Compact reached 1.32x and 1.33x RocksDB at 4 and 6 OCPUs; spread reached 1.09x and 1.16x.

The 2-post check was lower than the original 2-OCPU baseline: dodb width-16 uniform fell 7.9%, RocksDB fell 9.7%, and fio IOPS fell 6.7% while mean fdatasync latency rose from 2.513 to 2.695 ms. The dodb/RocksDB ratio stayed close (0.577x baseline, 0.588x post). This indicates time/storage drift and lowers confidence in small changes; it does not explain the much larger 2→4 width-16 dodb increase or the flat 4→6 result.

## Decision and next step

This is **B**: H1's leaf-parallel physical work converts into a meaningful first scaling step, and it improves relative to RocksDB, but the uniform workload remains materially slower and throughput saturates at 4 OCPUs. Keep the H1 read path and locality advantage. The next focused measurement should isolate width-16 uniform's planning/coordinator and durable-group admission path, since 4→6 adds worker parallelism without additional transactions per second. Do not infer that a new storage architecture is required from this result alone.

The final OCI API query after the post-check confirmed the same instance `RUNNING` as `VM.Standard.A1.Flex / 2 OCPU / 12 GB`. The final query result is also recorded in `oci-timeline.jsonl`.
