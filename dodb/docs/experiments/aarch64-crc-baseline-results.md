# AArch64 Linux CRC Baseline Results

## Decision

dodb AArch64 Linux builds require the ARM CRC32 extension. The production
target `aarch64-unknown-linux-gnu` is compiled with
`-C target-feature=+crc` by the repository Cargo configuration. AArch64 Linux
CPUs without CRC32 are unsupported for this build target, and deployment
operators must ensure the extension is available.

The WAL and page checksum hot path benefits materially from CRC-enabled code
generation. The project adopts this hardware baseline instead of adding
runtime CPU detection and a fallback path. The existing `crc32c = "0.6"`
dependency remains unchanged. No target CPU model is selected.

## Source and Configuration

The exact policy source is commit
`d4ca509ba4fda2b353397aa891734b5cb1457b95`, based on source commit
`b309b0fa9f187f7b8f14b1640bb55b9b892926d9`. The OCI checkout had the same
`.cargo/config.toml` content as the policy commit while the benchmark ran; its
reported `git_commit` field therefore names the unmodified base commit.

The configuration is:

```toml
[target.aarch64-unknown-linux-gnu]
rustflags = ["-C", "target-feature=+crc"]
```

Cargo's target-specific flags apply to the host-native OCI build even when
`--target` is omitted. This was first checked with a fresh target directory and
a temporary equivalent Cargo setting, then verified on the repository-default
build with `RUSTFLAGS` unset. Verbose rustc output contains
`-C target-feature=+crc` for `crc32c`, `dodb_storage`, and `phase0_bench`, so
the dependency graph receives the feature. macOS AArch64 verbose build output
contains no `+crc` flag; the Apple target is outside this policy.

The default binary disassembly contains the `crc32cb` ARM instruction inside
the `crc32c` dependency's AArch64 hardware backend. The OCI host reports
`aarch64-unknown-linux-gnu`, Neoverse-N1, and the `crc32` CPU feature in
`lscpu`. The selected verbose compiler invocations and disassembly are saved
in `codegen-proof.txt` beside the raw benchmark files.

## Local Correctness

On the local Apple AArch64 development machine, all requested checks passed:

- `cargo fmt --all -- --check`
- `cargo test -p dodb-core` (6 passed)
- `cargo test -p dodb-storage` (112 library tests and 10 benchmark tests passed)
- `cargo test -p dodb-storage --bin phase0-bench` (10 passed)
- `cargo test --workspace` (all tests passed)
- `git diff --check`

The local macOS release build's verbose rustc invocations for `crc32c`,
`dodb_storage`, and `phase0_bench` did not contain `-C
target-feature=+crc`.

## OCI Build Comparison

The OCI precheck found a clean checkout at base SHA
`b309b0fa9f187f7b8f14b1640bb55b9b892926d9`, branch
`experiment/b-link-batched-engine`, Oracle Linux Server 9.8, 2 OCPU, and
18 GiB free on the mounted 30 GiB root filesystem. The separate 200 GiB block
device was not used. `lscpu` reported `crc32`.

Repository-default build, with `RUSTFLAGS` unset:

```bash
cargo build --locked --release -p dodb-storage --bin phase0-bench
```

The build used a separate `CARGO_TARGET_DIR` for artifact isolation. Its
verbose rustc output showed `-C target-feature=+crc` on the `crc32c`,
`dodb_storage`, and `phase0_bench` compiler invocations.

Explicit control build from the same source:

```bash
RUSTFLAGS='-C target-feature=+crc' \
  cargo build --locked --release -p dodb-storage --bin phase0-bench
```

Both builds produced the same binary SHA256:

| Build | Binary SHA256 |
| --- | --- |
| Repository default | `f1d8252ab249d94171412a3e989d5306ca541d05fac75a4ae783d01c58355e42` |
| Explicit `+crc` control | `f1d8252ab249d94171412a3e989d5306ca541d05fac75a4ae783d01c58355e42` |

The byte-identical binaries are direct evidence that repository-default and
explicit-control code generation match for this source and toolchain.

## OCI Benchmark

The workload used `planned-blink`, 16 writers, 0 readers, width 1,
`different-leaf-heavy`, working set 100,000, cache capacity 4,096, 16-byte
keys, 64-byte values, maximum group requests 64, maximum group bytes 4,194,304,
queue capacity 256, 0 µs collection delay, sync disabled, 2 Tokio workers,
1-second warmup, 2-second duration, and seeds `0x3a042026` through
`0x3a042028`.

An initial pair of three-repetition runs showed execution-order drift in both
throughput and timing counters. To reduce that effect, the accepted comparison
uses three seed-matched pairs of independent one-repetition runs, alternating
which build ran first. Each run used the same workload settings and 1-second
warmup. The primary JSONL files below concatenate those unmodified raw records;
the individual paired records and the two full three-repetition runs are also
preserved under the result directory.

Metrics are normalized per successful mutation before calculating the median
and range across the three samples. The ratio is repository-default divided by
explicit-control.

| Metric | Repository default median (min..max) | Explicit `+crc` median (min..max) | Ratio |
| --- | ---: | ---: | ---: |
| Mutations/s | 44,068.9 (43,488.0..44,293.2) | 44,514.6 (43,818.3..45,283.4) | 0.990x |
| WAL group encode, ns/mutation | 3,030.8 (3,016.5..3,089.4) | 2,926.0 (2,908.6..3,064.4) | 1.036x |
| WAL append, ns/mutation | 5,027.1 (5,020.0..5,174.4) | 4,890.9 (4,852.4..5,135.1) | 1.028x |
| Physical page encode, ns/mutation | 2,110.5 (2,110.3..2,115.7) | 2,117.2 (2,093.1..2,128.1) | 0.997x |

All three samples for both builds reported zero errors and zero overloads,
exactly 1 page image per mutation, and 4,224 WAL bytes per mutation.

All four metrics meet the configured equivalence ranges: throughput 0.97x to
1.03x, WAL group encode and WAL append 0.90x to 1.10x, and physical page encode
0.90x to 1.10x. The repository-default build is equivalent to the explicit
`+crc` control.

## Artifacts

The primary raw results are:

| Repository artifact | SHA256 |
| --- | --- |
| `docs/experiments/results/oci-a1-2ocpu-12g-200g/aarch64-crc-baseline/planned-repo-default-width1.jsonl` | `7d0e5158aac68bfa740fa5e7012543063555f13cb0573ecb04fee735c01e1eea` |
| `docs/experiments/results/oci-a1-2ocpu-12g-200g/aarch64-crc-baseline/planned-explicit-crc-width1.jsonl` | `44eafd6fed20bf8d690a9bee86c02c85b792c72427496cf452f3667ba0428524` |

The `paired-runs/` directory preserves each raw one-repetition record. The
`full-repetition-runs/` subdirectories preserve the two additional exact
three-repetition A/B runs that exposed run-order timing drift. `SHA256SUMS`
lists and verifies every JSONL record and the codegen proof file.

The OCI backup is
`/home/opc/dodb-oci-artifacts-aarch64-crc-baseline-d4ca509/`. Its JSONL
SHA256 values were checked against the repository copies and match.
