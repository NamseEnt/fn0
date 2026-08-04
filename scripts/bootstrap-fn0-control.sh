#!/usr/bin/env bash
# Bootstrap the fn0-control application onto a freshly-stood-up worker site.
# Idempotent end-to-end. Safe to re-run; each step skips work that is already
# done.
#
# Flow:
#   1. Build / push the cwasm-compiler lambda for the workspace fn0-wasmtime
#      version (no control dependency — uses ensure_cwasm_lambda).
#   2. Resolve the already-published fn0-worker image for the workspace
#      fn0-worker version. scripts/deploy-fn0-worker.sh publishes it.
#   3. forte build fn0/control, assemble bundle.raw.tar (manifest.json +
#      backend.wasm + entry.js + env.yaml).
#   4. Upload original/fn0-control.tar to the bundle-store R2 bucket.
#   5. Invoke cwasm-compiler lambda to produce
#      compiled/<wasmtime>/fn0-control/<code_version>.tar.zst.
#   6. Seed the fn0-control turso database with:
#        - Fn0WasmtimeVersionDoc (active=<wasmtime>)
#        - CompiledBundleDoc (project_id=fn0-control, code_version=<cv>)
#        - WorkerManifestDoc (fn0-control mapped to its custom_domain, and its
#          storage target so workers can reach the buckets)
#        - ProjectCloudflareConfigDoc (the buckets and credentials step 3a made)
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
# shellcheck source=lib/cwasm-compiler.sh
source "${REPO_ROOT}/scripts/lib/cwasm-compiler.sh"
# shellcheck source=lib/worker-image.sh
source "${REPO_ROOT}/scripts/lib/worker-image.sh"
# shellcheck source=lib/control-bundle.sh
source "${REPO_ROOT}/scripts/lib/control-bundle.sh"
# shellcheck source=lib/control-deploy.sh
source "${REPO_ROOT}/scripts/lib/control-deploy.sh"
# shellcheck source=lib/control-cloudflare.sh
source "${REPO_ROOT}/scripts/lib/control-cloudflare.sh"
# shellcheck source=lib/control-seed.sh
source "${REPO_ROOT}/scripts/lib/control-seed.sh"
# shellcheck source=lib/control-static-upload.sh
source "${REPO_ROOT}/scripts/lib/control-static-upload.sh"

need pulumi
need jq
need cargo
need docker
need aws
need oci
need curl
need tar
need uuidgen
need python3

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

# Step 2 — resolve the fn0-worker image to seed the target with. Building and
# pushing it belongs to scripts/deploy-fn0-worker.sh; a control-only change must
# not wait on a worker image build, and must not be able to fail on one.
resolve_fn0_worker_image_ref "$target_worker"
worker_image_ref="$FN0_WORKER_IMAGE_REF"
if [[ -z "$worker_image_ref" ]]; then
  echo "resolve_fn0_worker_image_ref did not set FN0_WORKER_IMAGE_REF" >&2
  exit 1
fi

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

# 3a — provision fn0-control's own Cloudflare resources and build the frontend
# against a stable build_id, so vite emits real asset URLs instead of the
# __FORTE_BASE__ placeholder. fn0-control is a connected project like any
# other; it just cannot run `forte cloudflare connect` against itself.
cf_account_id="$(pulumi_pick cloudflareAccountId)"
cf_zone_id="$(pulumi_pick cloudflareZoneId)"
# The operator's own token, the same thing `forte cloudflare connect --api-token`
# is handed. It carries API Tokens -> Edit and nothing else, so it mints the two
# narrow credentials below and a short-lived token to provision with — it cannot
# write a bucket or a cache rule itself.
cf_user_token="$(cd "${REPO_ROOT}/infra/cloud" && pulumi config get fn0Cloud:cloudflareUserApiToken)"
vault_crypto_endpoint="$(pulumi_pick vaultCryptoEndpoint)"
vault_key_ocid="$(pulumi_pick vaultKeyOcid)"
if [[ -z "$cf_account_id" || -z "$cf_zone_id" || -z "$cf_user_token" \
   || -z "$vault_crypto_endpoint" || -z "$vault_key_ocid" ]]; then
  echo "missing pulumi config/outputs: cloudflare* / vault*" >&2
  exit 1
fi
cf_zone_name="${FN0_APEX_DOMAIN:-$(pulumi_pick apexDomain)}"
if [[ -z "$cf_zone_name" ]]; then
  echo "missing pulumi output: apexDomain" >&2
  exit 1
fi

read -r provisioning_token_id provisioning_token < <(mint_provisioning_token \
  "$cf_user_token" "$cf_account_id" "$cf_zone_id" "$CONTROL_PROJECT_ID")
if [[ -z "${provisioning_token_id:-}" || -z "${provisioning_token:-}" ]]; then
  echo "could not mint a provisioning token; check that cloudflareUserApiToken has User -> API Tokens -> Edit" >&2
  exit 1
fi
trap 'revoke_provisioning_token "$cf_user_token" "$provisioning_token_id"; rm -rf "$work_dir"' EXIT

provision_control_cloudflare \
  "$cf_account_id" "$provisioning_token" "$cf_zone_id" "$cf_zone_name" "$CONTROL_PROJECT_ID" "$CONTROL_CUSTOM_DOMAIN"

# Minted here rather than taken from the stack: these are the same narrow
# credentials `forte cloudflare connect` produces, and pulumi no longer holds
# an account-wide R2 token to hand out instead.
read -r worker_key_id worker_secret < <(mint_r2_token \
  "$cf_user_token" "$cf_account_id" "fn0 worker (${CONTROL_PROJECT_ID})" \
  "fn0-${CONTROL_PROJECT_ID}-private-object-storage" \
  "fn0-${CONTROL_PROJECT_ID}-public-object-storage")
read -r asset_key_id asset_secret < <(mint_r2_token \
  "$cf_user_token" "$cf_account_id" "fn0 frontend assets (${CONTROL_PROJECT_ID})" \
  "fn0-${CONTROL_PROJECT_ID}-frontend-asset")
read -r purge_token_id purge_token < <(mint_purge_token \
  "$cf_user_token" "$cf_zone_id" "fn0 cache purge (${CONTROL_PROJECT_ID})")

worker_secret_ct="$(kms_encrypt "$vault_crypto_endpoint" "$vault_key_ocid" "$worker_secret")"
asset_secret_ct="$(kms_encrypt "$vault_crypto_endpoint" "$vault_key_ocid" "$asset_secret")"
purge_token_ct="$(kms_encrypt "$vault_crypto_endpoint" "$vault_key_ocid" "$purge_token")"

static_bucket="fn0-${CONTROL_PROJECT_ID}-frontend-asset"
build_id="$(uuidgen | tr 'A-Z' 'a-z')"
export VITE_PUBLIC_URL="https://${static_bucket}.${cf_zone_name}/${build_id}/"
echo ">> build_id=${build_id} VITE_PUBLIC_URL=${VITE_PUBLIC_URL}"

bundle_path="${work_dir}/bundle.raw.tar"
build_control_raw_bundle "$env_yaml_path" "$bundle_path"

upload_fe_dist \
  "$cf_account_id" "$asset_key_id" "$asset_secret" \
  "$static_bucket" "$build_id" "${REPO_ROOT}/fn0/control/fe/dist"

# Step 4-5 — R2 upload of original/ + cwasm-compiler invoke. code_version =
# epoch milliseconds, the same unit control's deploy action mints. Seconds would
# read as older than every bundle a normal deploy ever produced, and bundle_gc
# only drops a version below the current one — so a bootstrap-seeded control
# would leave the bundle it replaced in R2 forever. Decide it up front so the
# same value is embedded in both R2 key and the cwasm output key.
code_version="$(python3 -c 'import time; print(int(time.time() * 1000))')"
upload_r2_original "$CONTROL_PROJECT_ID" "$code_version" "$bundle_path"
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
if [[ -z "$owner_github_id" ]]; then
  echo "missing pulumi output: controlOwnerGithubId" >&2
  exit 1
fi
seed_project_doc "$control_db_url" "$forte_group_token" \
  "$CONTROL_PROJECT_ID" "$owner_github_id" "$CONTROL_PROJECT_ID"
seed_fn0_wasmtime_version "$control_db_url" "$forte_group_token" "$target_wasmtime"
seed_compiled_bundle "$control_db_url" "$forte_group_token" \
  "$CONTROL_PROJECT_ID" "$code_version" "$target_wasmtime"
seed_worker_manifest "$control_db_url" "$forte_group_token" \
  "$CONTROL_PROJECT_ID" "$code_version" "$CONTROL_CUSTOM_DOMAIN"
seed_cloudflare_config "$control_db_url" "$forte_group_token" \
  "$CONTROL_PROJECT_ID" "$cf_account_id" "$cf_zone_id" "$cf_zone_name" \
  "$worker_key_id" "$worker_secret_ct" \
  "$asset_key_id" "$asset_secret_ct" "$purge_token_ct"
seed_manifest_storage "$control_db_url" "$forte_group_token" "$CONTROL_PROJECT_ID"

# Step 7 — seed the worker-agent's rollout target into the same control DB.
seed_target_fn0_worker_config "$control_db_url" "$forte_group_token" "$worker_image_ref"

# Step 8 — retire the credentials this run replaced. Last, because every doc
# that names the new ones is written by now: until then the superseded token is
# the one control is still serving with.
revoke_superseded_tokens "$cf_user_token" "fn0 worker (${CONTROL_PROJECT_ID})" "$worker_key_id"
revoke_superseded_tokens "$cf_user_token" "fn0 frontend assets (${CONTROL_PROJECT_ID})" "$asset_key_id"
revoke_superseded_tokens "$cf_user_token" "fn0 cache purge (${CONTROL_PROJECT_ID})" "$purge_token_id"

echo ">> bootstrap complete: project=${CONTROL_PROJECT_ID} domain=${CONTROL_CUSTOM_DOMAIN} code_version=${code_version}"
