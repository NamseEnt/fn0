set -euo pipefail

study_source=$(realpath "$1")
study_results=$(realpath -m "$2")
study_sha="$3"
study_cache="${DODB_STUDY_BUILD_CACHE:-/tmp/dodb-mixed-target-04f05824}"
study_data_root="${BENCH_DATA_ROOT:-/bench/zfs/db}"
study_rustc="$(rustup which --toolchain 1.97.1 rustc)"

cd "$study_source"
test "$(git rev-parse HEAD)" = "$study_sha"
test -z "$(git status --porcelain)"
test "$(nproc)" = 2
test "$(findmnt -n -o SOURCE "$study_data_root")" = dodbbench/db
test ! -e "$study_results"
mkdir -p "$study_results/build-proof" "$study_results/bin" "$study_results/profiles/tmp"

{
    printf 'source_sha_before=%s\nsource_status_before=clean\n' "$study_sha"
    printf 'source_directory=%s\n' "$study_source"
    hostname
    uname -a
    lscpu
    findmnt "$study_data_root"
    zfs get sync,recordsize,compression,atime dodbbench/db
    rustup run 1.97.1 rustc -vV
    rustup run 1.97.1 cargo --version
    printf 'target_directory=%s\nrustflags=-C target-cpu=native\n' "$study_cache"
} > "$study_results/build-proof/provenance.txt"

export CARGO_TARGET_DIR="$study_cache"
export RUSTFLAGS='-C target-cpu=native'
export RUSTUP_TOOLCHAIN=1.97.1
export RUSTC="$study_rustc"

(
    set -x
    rustup run 1.97.1 cargo build -p dodb-storage --release --locked -j2 --bin phase0-bench
    cp "$study_cache/release/phase0-bench" "$study_results/bin/phase0-default"
    rustup run 1.97.1 cargo build -p dodb-storage --release --locked -j2 --bin phase0-bench --features blink-borrowed-page-views
    cp "$study_cache/release/phase0-bench" "$study_results/bin/phase0-borrowed"
    rustup run 1.97.1 cargo test -p dodb-storage --lib --release --locked -j2 --features blink-borrowed-page-views --no-fail-fast
) > "$study_results/build-proof/build-and-tests.log" 2>&1

sha256sum "$study_results/bin/phase0-default" "$study_results/bin/phase0-borrowed" > "$study_results/build-proof/binary-sha256.txt"
export PHASE0_BINARY="$study_results/bin/phase0-default"
export PHASE0_BORROWED_PAGES_BINARY="$study_results/bin/phase0-borrowed"

python3 dodb/experiments/mixed-harness/run_read_metrics_matrix.py \
    --results "$study_results/smoke" --data-root "$study_data_root" \
    --variants blink-metrics-on,blink-borrowed-pages,main-btree \
    --repetitions 1 --warmup-ms 200 --duration-ms 300

python3 dodb/experiments/mixed-harness/run_read_metrics_matrix.py \
    --results "$study_results/read-c16" --data-root "$study_data_root" \
    --variants blink-metrics-on,blink-borrowed-pages,main-btree \
    --repetitions 3 --warmup-ms 2000 --duration-ms 5000

for study_kind in get query; do
    for study_variant in default borrowed; do
        study_profile="$study_results/profiles/$study_kind-$study_variant"
        study_binary="$study_results/bin/phase0-$study_variant"
        (
            set -x
            sudo -n env TMPDIR="$study_results/profiles/tmp" perf record \
                -e cycles:u -F 499 --call-graph fp -o "$study_profile.perf.data" -- \
                "$study_binary" --engine parallel-blink --suite read --readers 16 \
                --widths 1 --distributions uniform --read-kinds "$study_kind" \
                --duration 10s --warmup 2s --repetitions 1 --tokio-workers 2 \
                --cache-capacity 16384 --working-set 10000 --key-size 16 \
                --value-size 512 --read-limit 16 --sync-mode real \
                --blink-collection-policy current --blink-workers 2 --seed 3504029734 \
                --output "$study_profile.jsonl"
            sudo -n chown "$(id -u):$(id -g)" "$study_profile.perf.data" "$study_profile.jsonl"
            perf report --stdio --no-children --percent-limit 0.5 -s comm,dso,symbol \
                -i "$study_profile.perf.data" > "$study_profile.report.txt"
            gzip "$study_profile.perf.data"
        ) > "$study_profile.log" 2>&1
    done
done

test "$(git rev-parse HEAD)" = "$study_sha"
test -z "$(git status --porcelain)"
printf 'source_sha_after=%s\nsource_status_after=clean\n' "$study_sha" >> "$study_results/build-proof/provenance.txt"
printf 'study_complete=%s\n' "$study_sha"
