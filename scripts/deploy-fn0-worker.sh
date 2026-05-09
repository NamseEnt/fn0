#!/usr/bin/env bash
# Deploys the current cargo workspace's fn0-worker (and the fn0-wasmtime it
# bakes) to all worker hosts. Idempotent end-to-end. Re-run from anywhere it
# previously failed and it picks up.
#
# Flow:
#   1. Resolve target fn0-wasmtime + fn0-worker versions from cargo workspace.
#   2. Read Fn0WasmtimeVersionDoc directly from doc-db.
#   3. If target wasmtime != active and != pending,
#      ensure_cwasm_pending(target) — builds the new cwasm-compiler lambda
#      (idempotent), registers pending in control, fans out R2 sync compile
#      until full coverage.
#   4. Build & push fn0-worker image (idempotent — same digest skips push).
#   5. Write TargetFn0WorkerConfigDoc.image_ref = new ref.
#   6. Wait until every live host's WorkerHostStatusDoc reports
#      active_image_ref == new ref.
#   7. If we set pending in step 3, call promote — pending → active and the
#      previous active's lambda + ECR tag are dropped.
#
# Rollback = git checkout old commit, run this. Same path; build skips push if
# the image still exists, and wasmtime / promote handle the reverse direction
# the same way.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export REPO_ROOT

# shellcheck source=lib/pulumi-outputs.sh
source "${REPO_ROOT}/scripts/lib/pulumi-outputs.sh"
# shellcheck source=lib/control-admin.sh
source "${REPO_ROOT}/scripts/lib/control-admin.sh"
# shellcheck source=lib/cwasm-compiler.sh
source "${REPO_ROOT}/scripts/lib/cwasm-compiler.sh"
# shellcheck source=lib/worker-image.sh
source "${REPO_ROOT}/scripts/lib/worker-image.sh"
# shellcheck source=lib/worker-target.sh
source "${REPO_ROOT}/scripts/lib/worker-target.sh"

need pulumi
need jq
need cargo
need docker
need aws
need oci
need curl

load_pulumi_outputs

target_wasmtime="$(cd "$REPO_ROOT" && cargo pkgid -p fn0-wasmtime | sed -E 's/.*[#@]([^:]+)$/\1/')"
target_worker="$(cd "$REPO_ROOT" && cargo pkgid -p fn0-worker | sed -E 's/.*[#@]([^:]+)$/\1/')"
if [[ -z "$target_wasmtime" || -z "$target_worker" ]]; then
  echo "failed to resolve cargo versions" >&2
  exit 1
fi
echo ">> target fn0-wasmtime=${target_wasmtime} fn0-worker=${target_worker}"

version_doc_json="$(get_fn0_wasmtime_version_doc)"
active=""
pending=""
if [[ -n "$version_doc_json" ]]; then
  active="$(jq -r '.active // empty' <<<"$version_doc_json")"
  pending="$(jq -r '.pending // empty' <<<"$version_doc_json")"
fi
echo ">> control fn0-wasmtime active=${active:-<none>} pending=${pending:-<none>}"

needs_promote=0
if [[ -z "$active" ]]; then
  echo ">> no active fn0-wasmtime in control; seeding pending=${target_wasmtime}"
  ensure_cwasm_pending "$target_wasmtime"
  needs_promote=1
elif [[ "$target_wasmtime" == "$active" ]]; then
  echo ">> target fn0-wasmtime already active; no migration needed"
elif [[ "$target_wasmtime" == "$pending" ]]; then
  echo ">> target fn0-wasmtime already pending; ensuring full coverage"
  ensure_cwasm_pending "$target_wasmtime"
  needs_promote=1
else
  echo ">> target fn0-wasmtime differs from active/pending; promoting via pending"
  ensure_cwasm_pending "$target_wasmtime"
  needs_promote=1
fi

build_and_push_fn0_worker
target_image_ref="$FN0_WORKER_PUSHED_IMAGE_REF"
if [[ -z "$target_image_ref" ]]; then
  echo "build_and_push_fn0_worker did not set FN0_WORKER_PUSHED_IMAGE_REF" >&2
  exit 1
fi

write_worker_target_image "$target_image_ref"
wait_worker_target_converged "$target_image_ref"

if [[ "$needs_promote" -eq 1 ]]; then
  echo ">> promoting pending fn0-wasmtime to active"
  promote_resp="$(control_promote_pending_fn0_wasmtime)"
  if [[ -z "$promote_resp" ]]; then
    echo ">> nothing to promote (pending was already empty)"
  else
    old_active="$(jq -r '.old_active' <<<"$promote_resp")"
    new_active="$(jq -r '.new_active' <<<"$promote_resp")"
    echo ">> active fn0-wasmtime: ${old_active} -> ${new_active}"
  fi
fi

echo ">> deploy complete."
