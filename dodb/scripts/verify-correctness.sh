#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
dodb_root="$(cd "$script_dir/.." && pwd)"
monorepo_root="$(cd "$dodb_root/.." && pwd)"
mode="${1:-}"

if [[ "$mode" != "fast" && "$mode" != "full" ]]; then
  printf 'Usage: %s {fast|full}\n' "$0" >&2
  exit 2
fi

cd "$monorepo_root"
start_seconds=$SECONDS
dodb_packages=(
  dodb-client
  dodb-core
  dodb-protocol
  dodb-server
  dodb-service
  dodb-soak
  dodb-storage
  dodb-testkit
)
formal_tests=(
  formal:test
  formal:group:test
  formal:segments:test
  formal:collection:test
  formal:wal:test
  formal:publication:test
  formal:checkpoint:test
  formal:e2e:test
)
formal_typechecks=(
  formal:typecheck
  formal:group:typecheck
  formal:segments:typecheck
  formal:collection:typecheck
  formal:wal:typecheck
  formal:publication:typecheck
  formal:checkpoint:typecheck
  formal:e2e:typecheck
)
formal_tlc=(
  formal:verify:tlc
  formal:group:verify:tlc
  formal:segments:verify:tlc
  formal:collection:verify:tlc
  formal:wal:verify:tlc
  formal:publication:verify:tlc
  formal:checkpoint:verify:tlc
  formal:e2e:verify:tlc
)
formal_apalache=(
  formal:verify:apalache
  formal:group:verify:apalache
  formal:segments:verify:apalache
  formal:collection:verify:apalache
  formal:wal:verify:apalache
  formal:publication:verify:apalache
  formal:checkpoint:verify:apalache
  formal:e2e:verify:apalache
)

section() {
  printf '\n== %s ==\n' "$1"
}

run() {
  local label="$1"
  shift
  section "$label"
  "$@"
}

rust_packages=()
for package in "${dodb_packages[@]}"; do
  rust_packages+=(--package "$package")
done
run "Rust formatting" cargo fmt "${rust_packages[@]}" -- --check
run "dodb Rust packages" cargo test "${rust_packages[@]}" --no-fail-fast

for formal_test in "${formal_tests[@]}"; do
  run "Quint model test: $formal_test" npm --prefix "$dodb_root" run "$formal_test"
done

for formal_typecheck in "${formal_typechecks[@]}"; do
  run "Quint typecheck: $formal_typecheck" npm --prefix "$dodb_root" run "$formal_typecheck"
done

if [[ "$mode" == "full" ]]; then
  run "Rust correspondence" cargo test -p dodb-testkit --test formal_correspondence -- --nocapture
  run "Persisted-prefix durability DST" cargo test -p dodb-testkit --test durability_dst -- --nocapture

  for formal_check in "${formal_tlc[@]}"; do
    run "Quint TLC: $formal_check" npm --prefix "$dodb_root" run "$formal_check"
  done

  for formal_check in "${formal_apalache[@]}"; do
    run "Quint Apalache: $formal_check" npm --prefix "$dodb_root" run "$formal_check"
  done

  run "Final whitespace check" git -C "$monorepo_root" diff --check
fi

section "Correctness gate complete"
printf 'mode=%s elapsed_seconds=%s\n' "$mode" "$((SECONDS - start_seconds))"
