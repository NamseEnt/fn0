# Ryzen H1 Multicore Scaling

## Decision

**Classification C: multicore scaling does not close the width-16 gap.** The retained H1 `parallel-blink` path does not gain end-to-end throughput as physical cores increase. At width-16 uniform, dodb falls from 4,400 tx/s at 1C to 3,542 tx/s at 2C, then remains around 3,700–3,850 tx/s through 6C and 6C/12T. RocksDB stays around 4,600–4,900 tx/s over the same range. The dodb/RocksDB ratio falls from 0.920x at 1C to 0.757–0.789x from 2C onward.

The result supports a focused reduction in durable physical write work, such as a leaf-local mini-delta or finer-grained physical representation. It does not support reviving the rejected global committed-overlay/WAL-v4 architecture. No storage architecture or product deployment setting was changed in this experiment.

## Repository and Build Provenance

- Canonical worktree: `/Users/namse/fn0-dodb-blink-experiment`
- Branch: `experiment/b-link-batched-engine-monorepo`
- H1-equivalent source commit and benchmark row `git_commit`: `08e82ec484bd62aeb659471a39f317b86cfb488f`
- Source checkout on the benchmark host was detached at that exact commit and clean before the release build and benchmark.
- H1 implementation selected explicitly with `--engine parallel-blink`; the Phase K source retains this H1 physical path.
- dodb build: `cargo build --locked --release --manifest-path /home/namse/dodb-scaling-source/Cargo.toml -p dodb-storage --bin phase0-bench`
- dodb benchmark binary SHA256: `6bbc08a70124711d23a2077a8f119e63e45430b6e426ac929fa05856fdbdebfb`
- RocksDB source tag/commit: `v11.8.0` / `abeebd9630f11bd08c28b7bd43c7bdfc62050654`
- RocksDB static library SHA256: `cd990dd3b76a497e22e5f4c8199596b69154615feaf50dcee8e1eac9c723dd0c`
- RocksDB harness binary SHA256: `57f3a241b9fdfb5f2e0337d3365d5c80dcd0c1dbd67a2345ce241884e1b0c074`
- The unmodified harness build script supplied an AArch64 compiler flag. An external compiler wrapper filtered that one architecture-specific flag for the x86-64 build; wrapper SHA256: `b68ff6592db323484225f56c2f6b667077937ec2678bc0c17d39eae4c028c8cf`.
- `run-order.jsonl` retains the full start command, affinity, source commit, result row, and completion record for every run. `SHA256SUMS` covers the archived evidence.
- The protected `/Users/namse/fn0` checkout was not modified.

## Machine and Filesystem

- Host: Debian 13, kernel `6.12.107+deb13-amd64`
- CPU: AMD Ryzen 5 5600, 6 physical cores / 12 logical CPUs; SMT enabled; siblings 0↔6 through 5↔11
- CPU governor: `powersave`
- SSD: Samsung SSD 860 EVO 250GB, `/dev/sda`, non-rotational, `mq-deadline`
- Filesystem: ext4 on `/dev/sda2`, mounted as `/` with `rw,relatime,errors=remount-ro`; no `discard` mount option
- Dodb data path `/var/tmp/dodb-scaling-data/dodb` and RocksDB path `/bench/zfs/db/rocksdb` both resolve to `/dev/sda2` ext4. `/bench/zfs/db` is a harness-required pathname; it is not a ZFS mount.
- No Btrfs filesystem or loopback image was used. The benchmark used the existing filesystem as requested.
- Full captured device, mount, CPU topology, governor, binary hash, and source information is in `environment.txt`.

## Method

The matrix compared both engines in the same session, on the same host and ext4 filesystem, with paired seeds and identical workload sizes. Runs were sequential and randomized across configurations. Each benchmark command used `taskset --cpu-list` and selected these CPU sets:

| Allocation | CPU set | dodb Tokio/B-link workers |
|---|---|---:|
| 1C | `0` | 2 / 2 |
| 2C | `0,1` | 2 / 2 |
| 4C | `0,1,2,3` | 4 / 4 |
| 6C | `0,1,2,3,4,5` | 6 / 6 |
| 6C/12T | `0,1,2,3,4,5,6,7,8,9,10,11` | 12 / 12 |

All configurations used 64 writers, a 100,000-row working set, 16-byte keys, 64-byte values, a 2-second warm-up, and a 5-second measured interval. Workloads were width-1 uniform, width-16 uniform, width-16 compact (`same-leaf-heavy`), and width-16 spread (`different-leaf-heavy`). Dodb used real WAL sync, group limit 64, group byte limit 4 MiB, queue capacity 256, and zero collection delay. RocksDB used `--pipelined off` and synchronous durable-return writes.

There are 144 successful runs: three repetitions per configuration, six for 6C/12T width-16 uniform after high variance, and twelve for 1C width-16 compact after high variance. All 72 RocksDB runs passed the harness verification. All dodb runs reported zero errors. Start/completion pairs, source SHA, selected engine, complete command line, CPU set, and per-run counters were checked for all runs. The earlier interrupted matrix, which overlapped another application, is excluded from headline results and was retained separately on the benchmark host.

The per-run CSV includes tx/s, p50/p95/p99, user/system CPU, context switches, elapsed time, block-device write IOPS/bytes, device busy time, and dodb group/sync metrics. Hardware cycles, instructions, and LLC misses were not collected. RocksDB fsync latency is not directly exposed by this harness.

## Results

See `summary.md` for throughput, ratios, speedups, and width-16 uniform latency tables. Raw JSONL and logs are retained under `raw/matrix/`; `per-run.csv`, `medians.csv`, `medians.json`, and `run-order.jsonl` preserve per-run and aggregate evidence.

Width-16 uniform scaling from 1C to 6C is 0.86x for dodb and 1.01x for RocksDB. At 6C/12T, dodb reaches 0.87x its 1C throughput; RocksDB reaches 1.02x. The H1 physical workers did execute concurrently: dodb's effective worker parallelism median is 5.40 at 6C and 10.73 at 6C/12T for width-16 uniform. That parallel work did not raise transaction throughput because end-to-end commit and sync stages dominate.

At 6C, width-16 compact reaches 4,913 tx/s and spread reaches 4,541 tx/s, compared with 3,771 tx/s for uniform. Dodb's median physical execution time is 143 ms for compact and 108 ms for spread, versus 359 ms for uniform; average transactions per group remains about 32 for all three. This supports locality-sensitive physical work rather than larger batches as the source of the locality advantage.

## Bottleneck Evidence and Limits

For width-16 uniform at 6C, dodb reports about 48% of one core (about 8% of the six-core machine), with a median 6.45 ms per WAL sync and groups averaging about 32 transactions. The root device reports roughly 70% busy time, about 296 write IOPS, and 43 MiB/s. RocksDB reports about 24% of one core (about 4% of the machine), while the device is about 85% busy at roughly 443 write IOPS and 10 MiB/s. These measurements point to durable sync / storage wait and serialized commit work rather than saturated CPU. Device busy time is below 100%, so this does not prove the SSD is at its absolute limit. A per-operation RocksDB fsync timer and hardware performance counters were not available.

SMT adds no reliable dodb gain: relative to 6C, width-16 uniform improves about 2%, compact falls about 1%, and spread falls about 5%. RocksDB changes by about 1–3% on width-16 workloads. These deltas are small beside run-to-run variance.

The 1C width-16 compact samples remained bimodal after twelve repetitions (dodb coefficient of variation 37%, RocksDB 41%). Its median is retained for completeness but should not be used as a precise scaling estimate. The 6C/12T width-16 uniform runs also retained a high coefficient of variation near 30% because one of six paired repetitions was a fast outlier; its median uses all six runs. The primary width-16 uniform physical-core results at 1C through 6C had 0.9–9.4% throughput CV across three runs.

## Next Research Step

Keep the H1 B-link read path and test a minimal reduction in leaf rewrite work, with real sync and the same-core RocksDB comparison. Profile any proposed change separately for physical page encoding, WAL assembly, and durable sync. Do not restart the rejected J/K overlay design. Do not begin an io_uring implementation as part of this result.
