set -euo pipefail
cd /bench/zfs/db/rocksdb-contract-baseline-20261016/source
printf 'utc='; date -u '+%Y-%m-%dT%H:%M:%SZ'
printf 'hostname='; hostname -f
printf 'uname='; uname -srm
lscpu | grep -E '^(Architecture|CPU\(s\)|Model name|Core\(s\) per socket|Socket\(s\)|Thread\(s\) per core):'
grep -m 1 '^Features' /proc/cpuinfo
free -b
grep '^MemTotal:' /proc/meminfo
df -B1 / /bench/zfs/db
zfs get -Hp used,available dodbbench/db
active_processes="$(python3 - <<'PY'
from pathlib import Path
wanted_prefixes = ("service_workload", "read_contract", "transaction", "research_metrics", "process_crash", "phase0-bench", "phase1-bench", "phase2-bench", "phase3-bench", "phase4-bench", "phase5-bench", "phase6-bench", "blink-fixed-group", "get_copy_probe", "transaction_header_probe", "response_size_probe")
found_processes = []
for comm_path in Path("/proc").glob("[0-9]*/comm"):
    try:
        process_name = comm_path.read_text().strip()
    except OSError:
        continue
    if process_name in ("cargo", "rustc") or process_name.startswith(wanted_prefixes):
        found_processes.append(f"{comm_path.parent.name} {process_name}")
print("\n".join(sorted(found_processes)))
PY
)"
if [ -n "$active_processes" ]; then
  printf 'active_build_bench_processes=\n%s\n' "$active_processes"
  exit 23
fi
printf 'active_build_bench_processes=none\n'
printf 'source_sha='; git rev-parse HEAD
test -z "$(git status --porcelain)"
printf 'source_status=clean\n'
printf 'source_git_dir='; git rev-parse --path-format=absolute --git-dir
printf 'source_git_common_dir='; git rev-parse --path-format=absolute --git-common-dir
printf 'anchor_head='; git -C /bench/zfs/db/product-build/dodb-product-source rev-parse HEAD
printf 'anchor_git_common_dir='; git -C /bench/zfs/db/product-build/dodb-product-source rev-parse --path-format=absolute --git-common-dir
printf 'rocksdb_head='; git -C /bench/zfs/db/product-build/rocksdb-source rev-parse HEAD
archive_sha="$(sha256sum /bench/zfs/db/product-build/rocksdb-source/librocksdb.a | cut -d ' ' -f 1)"
printf 'rocksdb_archive_sha256=%s\n' "$archive_sha"
test "$archive_sha" = 9e6f7ea77ba2a9b2285b0d85ff90b00925688ab09a1c8e14def3f10f0deca2a5
printf 'rustup_default='; rustup default
rustup toolchain list
rustup run 1.97.1 rustc --version
rustup run 1.97.1 cargo --version
rustup run 1.97.1 rustc -vV
rustup run 1.97.1 rustc -C target-feature=+crc --print cfg | grep '^target_feature="crc"$'
printf 'preflight_result=PASS\n'
