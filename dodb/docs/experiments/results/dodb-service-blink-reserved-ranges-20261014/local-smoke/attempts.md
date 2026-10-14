# Local Smoke Attempts

The first invocation used `target/release/service_workload` from the `dodb/`
subdirectory, so it exited with status 127 before starting. The retry used the
binary at the repository-level `target/release/service_workload` path.

The first paired smoke used 64 seed rows. Both runs completed, but independent
replay validation failed because the existing smoke schedule's seed-delete
oracle assumes at least 500 seeded rows. Those reports are preserved as
`blink.log` and `rocksdb-reserved-ranges.log` and excluded.

The retry at the pre-final implementation SHA `82160803` used 10,000 seed rows
with fresh database directories. Both backends passed independent replay and
shutdown/reopen validation. The RocksDB run explicitly selected
`reserved-ranges`. After strengthening the rollback test with a passing
revision condition, the final implementation SHA `3dd51326` reran both
10,000-row local smoke scenarios; their logs are `blink-3dd513265.log` and
`rocksdb-reserved-ranges-3dd513265.log`, and both passed replay and reopen
validation.
