set -euo pipefail

study_source=$(realpath "$1")
study_results=$(realpath -m "$2")
study_sha="$3"
study_cache="${DODB_STUDY_BUILD_CACHE:-/bench/zfs/db/experiment-build/main-compact-target}"
study_data_root="/bench/zfs/db/experiment-data/main-compact-$study_sha"
study_rustc="$(rustup which --toolchain 1.97.1 rustc)"

cd "$study_source"
test "$(git rev-parse HEAD)" = "$study_sha"
test -z "$(git status --porcelain)"
test "$(nproc)" = 2
test "$(findmnt -n -o SOURCE /bench/zfs/db)" = dodbbench/db
test ! -e "$study_results"
test ! -e "$study_data_root"
mkdir -p "$study_results/build-proof" "$study_results/bin" "$study_results/tmp" "$study_data_root"

export CARGO_TARGET_DIR="$study_cache"
export TMPDIR="$study_results/tmp"
export RUSTFLAGS='-C target-cpu=native'
export RUSTUP_TOOLCHAIN=1.97.1
export RUSTC="$study_rustc"
export PYTHONDONTWRITEBYTECODE=1
export BENCH_DATA_ROOT="$study_data_root"

{
    printf 'source_sha_before=%s\nsource_status_before=clean\n' "$study_sha"
    printf 'source_directory=%s\n' "$study_source"
    hostname
    uname -a
    lscpu
    findmnt /bench/zfs/db
    zfs get sync,recordsize,compression,atime dodbbench/db
    rustup run 1.97.1 rustc -vV
    rustup run 1.97.1 cargo --version
    printf 'target_directory=%s\ntmpdir=%s\nrustflags=-C target-cpu=native\n' "$study_cache" "$TMPDIR"
} > "$study_results/build-proof/provenance.txt"

(
    set -x
    rustup run 1.97.1 cargo build -p dodb-storage --release --locked -j2 --bin phase0-bench
    cp "$study_cache/release/phase0-bench" "$study_results/bin/phase0-default"
    rustup run 1.97.1 cargo build -p dodb-storage --release --locked -j2 --bin phase0-bench --features blink-borrowed-page-views
    cp "$study_cache/release/phase0-bench" "$study_results/bin/phase0-borrowed"
    rustup run 1.97.1 cargo test -p dodb-storage --lib --release --locked -j2 --features blink-borrowed-page-views --no-fail-fast
    rustup run 1.97.1 cargo test -p dodb-testkit --release --locked -j2 --test compact_wal
) > "$study_results/build-proof/build-and-tests.log" 2>&1

sha256sum "$study_results/bin/phase0-default" "$study_results/bin/phase0-borrowed" > "$study_results/build-proof/binary-sha256.txt"
export PHASE0_BINARY="$study_results/bin/phase0-default"
export PHASE0_BORROWED_PAGES_BINARY="$study_results/bin/phase0-borrowed"

python3 dodb/experiments/mixed-harness/run_read_metrics_matrix.py \
    --results "$study_results/read-smoke" --data-root "$study_data_root" \
    --variants main-btree,main-btree-compact-wal,blink-borrowed-pages \
    --repetitions 1 --warmup-ms 200 --duration-ms 300

python3 dodb/experiments/mixed-harness/run_payload_batching_matrix.py \
    --results "$study_results/mixed-smoke" --clients 4,64 --value-modes changing \
    --variants main-btree,main-btree-compact-wal,parallel-blink-main-parity,parallel-blink-main-parity-borrowed \
    --repetitions 1 --warmup-ms 200 --duration-ms 300

python3 dodb/experiments/mixed-harness/run_read_metrics_matrix.py \
    --results "$study_results/read-c16" --data-root "$study_data_root" \
    --variants main-btree,main-btree-compact-wal,blink-borrowed-pages \
    --repetitions 3 --warmup-ms 2000 --duration-ms 10000

python3 dodb/experiments/mixed-harness/run_payload_batching_matrix.py \
    --results "$study_results/mixed-c4-c64" --clients 4,64 --value-modes changing \
    --variants main-btree,main-btree-compact-wal,parallel-blink-main-parity,parallel-blink-main-parity-borrowed \
    --repetitions 3 --warmup-ms 2000 --duration-ms 5000

test "$(git rev-parse HEAD)" = "$study_sha"
test -z "$(git status --porcelain)"
printf 'source_sha_after=%s\nsource_status_after=clean\n' "$study_sha" >> "$study_results/build-proof/provenance.txt"
printf 'study_complete=%s\n' "$study_sha"
