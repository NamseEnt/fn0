#!/usr/bin/env bash
# Reports which stored bundles would still load after a wasmtime bump.
#
# Read-only. Every `original/` bundle is fetched and validated with the same
# wasmparser the platform validates uploads with, which is stricter than the
# wasmtime the fleet runs today: components built with an older forte-sdk are
# async-lowered in a way the Component Model forbids, and wasmtime 43 is the
# only reason they run.
#
# The bump in #81 is gated on this reaching 100%: `ensure_cwasm_pending`
# compiles every `original/` object for the new wasmtime, so one invalid bundle
# stops the coverage gate and with it the worker roll.
#
# Exits non-zero when any bundle is invalid.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export REPO_ROOT

# shellcheck source=lib/pulumi-outputs.sh
source "${REPO_ROOT}/scripts/lib/pulumi-outputs.sh"
# shellcheck source=lib/r2.sh
source "${REPO_ROOT}/scripts/lib/r2.sh"

need aws
need jq
need cargo

load_pulumi_outputs

r2_endpoint="$(pulumi_pick bundleStoreR2Endpoint)"
r2_bucket="$(pulumi_pick bundleStoreR2BucketName)"
r2_ak="$(pulumi_pick bundleStoreR2AccessKeyId)"
r2_sk="$(pulumi_pick bundleStoreR2SecretAccessKey)"
for v in r2_endpoint r2_bucket r2_ak r2_sk; do
  if [[ -z "${!v}" ]]; then
    echo "missing pulumi output (${v})" >&2
    exit 1
  fi
done

echo ">> building validator"
(cd "$REPO_ROOT" && cargo build --quiet --release -p fn0-compiler --bin fn0-validate-bundle)
validator="${REPO_ROOT}/target/release/fn0-validate-bundle"

work_dir="$(mktemp -d)"
trap 'rm -rf "$work_dir"' EXIT

echo ">> listing original/"
r2_list_objects "$r2_endpoint" "$r2_bucket" "$r2_ak" "$r2_sk" "original/" \
  | jq -r 'select(.Key | test("^original/[^/]+/[0-9]+\\.tar$")) | .Key' \
  > "${work_dir}/keys"

total="$(wc -l < "${work_dir}/keys" | tr -d ' ')"
echo ">> ${total} bundle(s) to check"

valid=0
invalid=0
while IFS= read -r key; do
  [[ -z "$key" ]] && continue
  bundle="${work_dir}/bundle.tar"
  r2_get_object "$r2_endpoint" "$r2_bucket" "$r2_ak" "$r2_sk" "$key" "$bundle"
  if reason="$("$validator" "$bundle" 2>&1)"; then
    valid=$((valid + 1))
    printf '  ok      %s\n' "$key"
  else
    invalid=$((invalid + 1))
    printf '  INVALID %s\n    %s\n' "$key" "${reason#*: }"
    echo "$key" >> "${work_dir}/invalid"
  fi
  rm -f "$bundle"
done < "${work_dir}/keys"

echo
echo "valid: ${valid}/${total}"
if (( invalid == 0 )); then
  echo "Every stored bundle survives the bump."
  exit 0
fi

echo "invalid: ${invalid}"
echo
echo "Rebuild and redeploy these projects with forte-sdk >= 0.6.0, or let"
echo "bundle_gc drop them if they are superseded code versions:"
sed -E 's#^original/([^/]+)/([0-9]+)\.tar$#  project \1, code version \2#' "${work_dir}/invalid" | sort -u
exit 1
