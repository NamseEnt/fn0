#!/usr/bin/env bash
# Bootstrap the fn0-control application onto a freshly-stood-up worker site.
# Idempotent end-to-end. Safe to re-run; each step skips work that is already
# done.
#
# Flow:
#   1. Build / push the cwasm-compiler lambda for the workspace fn0-wasmtime
#      version (no control dependency — uses ensure_cwasm_lambda).
#   2. Build / push the fn0-worker image for the workspace fn0-worker version.
#   3. forte build fn0/control, assemble bundle.raw.tar (manifest.json +
#      backend.wasm + entry.js + env.yaml).
#   4. Upload original/fn0-control.tar to the bundle-store R2 bucket.
#   5. Invoke cwasm-compiler lambda to produce
#      compiled/<wasmtime>/fn0-control/<code_version>.tar.zst.
#   6. Seed the fn0-control turso database with:
#        - Fn0WasmtimeVersionDoc (active=<wasmtime>)
#        - CompiledBundleDoc (project_id=fn0-control, code_version=<cv>)
#        - WorkerManifestDoc (fn0-control mapped to its custom_domain)
#   7. Seed the worker-agent turso database (fn0-doc-db) with
#      TargetFn0WorkerConfigDoc.image_ref = <new fn0-worker image_ref>.
#
# Re-running picks up where it stopped — same image_refs and same code_version
# input means same R2 keys and same doc PKs.
#
# Required tools: pulumi, jq, cargo, docker, aws, oci, curl, forte, tar.
# Required env (used to address the control turso DB, which is not on the
# stack output by default):
#   - none directly; the script pulls forteDbGroupToken / forteDbHostSuffix
#     from the pulumi stack output. Make sure index.ts exports them.

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
# shellcheck source=lib/control-bundle.sh
source "${REPO_ROOT}/scripts/lib/control-bundle.sh"
# shellcheck source=lib/control-deploy.sh
source "${REPO_ROOT}/scripts/lib/control-deploy.sh"
# shellcheck source=lib/control-seed.sh
source "${REPO_ROOT}/scripts/lib/control-seed.sh"

need pulumi
need jq
need cargo
need docker
need aws
need oci
need curl
need tar

load_pulumi_outputs

CONTROL_PROJECT_ID="${CONTROL_PROJECT_ID:-fn0-control}"
CONTROL_CUSTOM_DOMAIN="${CONTROL_CUSTOM_DOMAIN:-}"
if [[ -z "$CONTROL_CUSTOM_DOMAIN" ]]; then
  control_url="$(pulumi_pick controlUrl)"
  if [[ -z "$control_url" ]]; then
    echo "missing pulumi output: controlUrl (and CONTROL_CUSTOM_DOMAIN not set)" >&2
    exit 1
  fi
  CONTROL_CUSTOM_DOMAIN="${control_url#https://}"
  CONTROL_CUSTOM_DOMAIN="${CONTROL_CUSTOM_DOMAIN%/}"
fi

target_wasmtime="$(cd "$REPO_ROOT" && cargo pkgid -p fn0-wasmtime | sed -E 's/.*[#@]([^:]+)$/\1/')"
target_worker="$(cd "$REPO_ROOT" && cargo pkgid -p fn0-worker | sed -E 's/.*[#@]([^:]+)$/\1/')"
if [[ -z "$target_wasmtime" || -z "$target_worker" ]]; then
  echo "failed to resolve cargo versions (fn0-wasmtime / fn0-worker)" >&2
  exit 1
fi
echo ">> target fn0-wasmtime=${target_wasmtime} fn0-worker=${target_worker}"

# Step 1 — cwasm-compiler lambda for target wasmtime.
ensure_cwasm_lambda "$target_wasmtime"

# Step 2 — fn0-worker image (idempotent — same digest skips push).
build_and_push_fn0_worker
worker_image_ref="$FN0_WORKER_PUSHED_IMAGE_REF"
if [[ -z "$worker_image_ref" ]]; then
  echo "build_and_push_fn0_worker did not set FN0_WORKER_PUSHED_IMAGE_REF" >&2
  exit 1
fi
echo ">> worker image_ref=${worker_image_ref}"

# Step 3 — build control raw bundle.
work_dir="$(mktemp -d)"
trap 'rm -rf "$work_dir"' EXIT

control_env_yaml="$(pulumi_pick controlBootstrapEnvYaml)"
if [[ -z "$control_env_yaml" ]]; then
  echo "missing pulumi output: controlBootstrapEnvYaml" >&2
  exit 1
fi
env_yaml_path="${work_dir}/env.yaml"
printf '%s' "$control_env_yaml" > "$env_yaml_path"

bundle_path="${work_dir}/bundle.raw.tar"
build_control_raw_bundle "$env_yaml_path" "$bundle_path"

# Step 4 — R2 upload of original/.
upload_r2_original "$CONTROL_PROJECT_ID" "$bundle_path"

# Step 5 — cwasm-compiler invoke. code_version = epoch seconds (monotonic
# across re-runs; the worker treats each as a new version, but the manifest
# below only references the latest one).
code_version="$(date +%s)"
compile_via_cwasm "$CONTROL_PROJECT_ID" "$code_version" "$target_wasmtime" "$CWASM_LAMBDA_FUNCTION_NAME"

# Step 6 — seed the fn0-control turso DB.
forte_group_token="$(pulumi_pick forteDbGroupToken)"
forte_host_suffix="$(pulumi_pick forteDbHostSuffix)"
if [[ -z "$forte_group_token" || -z "$forte_host_suffix" ]]; then
  echo "missing pulumi output: forteDbGroupToken / forteDbHostSuffix (add them to index.ts exports)" >&2
  exit 1
fi
control_db_url="https://${CONTROL_PROJECT_ID}${forte_host_suffix}"
ensure_docs_table "$control_db_url" "$forte_group_token"
owner_github_id="$(pulumi_pick controlOwnerGithubId)"
owner_github_login="$(pulumi_pick controlOwnerGithubLogin)"
if [[ -z "$owner_github_id" || -z "$owner_github_login" ]]; then
  echo "missing pulumi output: controlOwnerGithubId / controlOwnerGithubLogin" >&2
  exit 1
fi
seed_user_doc "$control_db_url" "$forte_group_token" \
  "$owner_github_id" "$owner_github_login" "$CONTROL_PROJECT_ID"
seed_project_doc "$control_db_url" "$forte_group_token" \
  "$CONTROL_PROJECT_ID" "$owner_github_id" "$CONTROL_PROJECT_ID"
seed_fn0_wasmtime_version "$control_db_url" "$forte_group_token" "$target_wasmtime"
seed_compiled_bundle "$control_db_url" "$forte_group_token" \
  "$CONTROL_PROJECT_ID" "$code_version" "$target_wasmtime"
seed_worker_manifest "$control_db_url" "$forte_group_token" \
  "$CONTROL_PROJECT_ID" "$code_version" "$CONTROL_CUSTOM_DOMAIN"

# Step 7 — seed the worker-agent turso DB (fn0-doc-db).
agent_db_url="$(pulumi_pick docDbUrl)"
agent_db_token="$(pulumi_pick docDbToken)"
if [[ -z "$agent_db_url" || -z "$agent_db_token" ]]; then
  echo "missing pulumi output: docDbUrl / docDbToken" >&2
  exit 1
fi
ensure_docs_table "$agent_db_url" "$agent_db_token"
seed_target_fn0_worker_config "$agent_db_url" "$agent_db_token" "$worker_image_ref"

echo ">> bootstrap complete: project=${CONTROL_PROJECT_ID} domain=${CONTROL_CUSTOM_DOMAIN} code_version=${code_version}"
