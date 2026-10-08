#!/usr/bin/env bash
set -euo pipefail

source_branch="experiment/b-link-batched-engine-monorepo"
source_sha="$(git rev-parse HEAD)"
expected_start="a9821e4c1c69150ebd42871d442db8e3e79f22a2"
host="opc@217.142.246.204"
remote_repository="https://github.com/NamseEnt/fn0.git"

if [[ "$(git branch --show-current)" != "$source_branch" ]]; then
  printf 'wrong branch: %s\n' "$(git branch --show-current)" >&2
  exit 1
fi

if ! git merge-base --is-ancestor "$expected_start" "$source_sha"; then
  printf 'source SHA is not based on the requested start commit\n' >&2
  exit 1
fi

ssh -o IdentitiesOnly=yes -o StrictHostKeyChecking=yes "$host" "bash -s -- '$source_sha' '$source_branch' '$remote_repository'" <<'REMOTE'
set -euo pipefail

source_sha="$1"
source_branch="$2"
remote_repository="$3"
storage_root="/bench/zfs/db"
checkout_path="$storage_root/process-crash-checkout-$source_sha"
result_path="$storage_root/process-crash-recovery-$source_sha"
target_path="$storage_root/process-crash-target-$source_sha"
tmp_path="$storage_root/process-crash-tmp-$source_sha"

if [[ -e "$checkout_path" || -e "$result_path" || -e "$target_path" || -e "$tmp_path" ]]; then
  printf 'refusing to overwrite an existing checkout or result path\n' >&2
  exit 1
fi

filesystem_type="$(findmnt -n -o FSTYPE -T "$storage_root")"
if [[ "$filesystem_type" != "zfs" ]]; then
  printf 'expected ZFS at %s, found %s\n' "$storage_root" "$filesystem_type" >&2
  exit 1
fi

mkdir -p "$result_path" "$target_path" "$tmp_path"
{
  date -u '+utc=%Y-%m-%dT%H:%M:%SZ'
  printf 'host=%s\n' "$(hostname -f)"
  printf 'source_sha=%s\n' "$source_sha"
  printf 'filesystem_type=%s\n' "$filesystem_type"
  findmnt -T "$storage_root"
  df -hT "$storage_root"
  uname -a
  rustc -vV
  cargo -V
  git --version
  printf '\n'
  printf 'checkout_path=%s\nresult_path=%s\ntarget_path=%s\ntmp_path=%s\n' "$checkout_path" "$result_path" "$target_path" "$tmp_path"
} > "$result_path/environment.txt" 2>&1

git clone --no-checkout "$remote_repository" "$checkout_path" > "$result_path/clone.log" 2>&1
git -C "$checkout_path" fetch origin "$source_branch" >> "$result_path/clone.log" 2>&1
if ! git -C "$checkout_path" merge-base --is-ancestor "$source_sha" "origin/$source_branch"; then
  printf 'source SHA is not present in the pushed branch\n' >&2
  exit 1
fi
git -C "$checkout_path" checkout --detach "$source_sha" > "$result_path/checkout.log" 2>&1
actual_sha="$(git -C "$checkout_path" rev-parse HEAD)"
if [[ "$actual_sha" != "$source_sha" ]]; then
  printf 'checkout SHA mismatch: %s\n' "$actual_sha" >&2
  exit 1
fi
if [[ -n "$(git -C "$checkout_path" status --porcelain)" ]]; then
  printf 'remote checkout is not clean\n' >&2
  exit 1
fi

{
  git -C "$checkout_path" status --short --branch
  printf 'checkout_sha=%s\n' "$actual_sha"
  printf 'rust_target='; rustc -vV | sed -n 's/^host: / /p'
  if git -C "$checkout_path" cat-file -e '95954ecaa^{commit}' 2>/dev/null; then
    if git -C "$checkout_path" diff --quiet 95954ecaa "$source_sha" -- \
      dodb/crates/dodb-storage/src/btree \
      dodb/crates/dodb-storage/src/blink \
      dodb/crates/dodb-storage/src/wal.rs \
      dodb/crates/dodb-storage/src/durable_file.rs; then
      printf 'engine_wal_source_matches_95954ecaa=true\n'
    else
      printf 'engine_wal_source_matches_95954ecaa=false\n'
      git -C "$checkout_path" diff --stat 95954ecaa "$source_sha" -- \
        dodb/crates/dodb-storage/src/btree \
        dodb/crates/dodb-storage/src/blink \
        dodb/crates/dodb-storage/src/wal.rs \
        dodb/crates/dodb-storage/src/durable_file.rs
    fi
  else
    printf 'engine_wal_source_matches_95954ecaa=base_unavailable\n'
  fi
} > "$result_path/source.txt" 2>&1

export TMPDIR="$tmp_path"
export CARGO_TARGET_DIR="$target_path"
export DODB_CRASH_ARTIFACT_DIR="$result_path/cases"
mkdir -p "$DODB_CRASH_ARTIFACT_DIR"
cp "$checkout_path/dodb/scripts/run-process-crash-recovery-oci.sh" "$result_path/run-script.sh"
printf '%s\n' 'cargo test --release --locked -p dodb-storage --test process_crash_recovery -- --nocapture' > "$result_path/command.txt"

set +e
cd "$checkout_path"
cargo test --release --locked -p dodb-storage --test process_crash_recovery -- --nocapture \
  > "$result_path/test.log" 2>&1
test_status=$?
set -e
printf '%s\n' "$test_status" > "$result_path/test-exit-status.txt"
find "$target_path/release" -type f -perm -111 \
  \( -path "$target_path/release/deps/process_crash_recovery-*" \
  -o -path "$target_path/release/deps/process_crash_recovery_child-*" \
  -o -path "$target_path/release/process-crash-recovery-child" \) \
  -print0 | sort -z | xargs -0 -r sha256sum > "$result_path/binary-sha256.txt"

find "$result_path" -type f ! -name SHA256SUMS -print0 | sort -z | xargs -0 sha256sum > "$result_path/SHA256SUMS"
exit "$test_status"
REMOTE
