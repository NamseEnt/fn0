# R1b-3b setup and validation attempts that did not start a crash case

All attempts below are retained as run history. None started a crash child.

1. The initial `cargo check --locked --offline -p dodb-rocksdb --test
   process_crash_contract` stopped because the newly added dev dependency
   required a `Cargo.lock` update. The lockfile was updated offline, then the
   same check passed with `--locked`.
2. The first local native focused-test invocation compiled the wrapper but
   failed while linking. The reused macOS RocksDB archive had no RTTI symbols,
   while the C++ wrapper referenced RTTI for `rocksdb::EnvWrapper` and
   `rocksdb::WritableFile`. The failed linker log is in
   `local-execution-logs.tar.gz`. Adding `-fno-rtti` to the shim build fixed
   the mismatch; subsequent native and RocksDB focused runs passed.
3. The first OCI helper invocation used the documented direct script form and
   returned `bash: dodb/scripts/run-oci-rocksdb-build.sh: Permission denied`.
   The checkout helper file is not executable. No helper run directory or
   Cargo process was created by that attempt. The same helper was then invoked
   with `bash`, without changing the checkout, and both OCI cases passed.

The separate preliminary passing local crash runs, accepted local runs, and
failed native link log are all retained in `local-execution-logs.tar.gz`.
