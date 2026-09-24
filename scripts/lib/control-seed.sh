# shellcheck shell=bash
# Seed control-plane documents. Source-only.

if [[ -n "${__FN0_CONTROL_SEED_LOADED:-}" ]]; then
  return 0
fi
__FN0_CONTROL_SEED_LOADED=1

# shellcheck source=scripts/lib/control-db.sh
source "${REPO_ROOT}/scripts/lib/control-db.sh"

__seed_insert_only() {
  local pk="$1" data="$2" status
  if control_db_put_if_missing "$pk" "" "$data" >/dev/null; then
    return 0
  else
    status=$?
  fi
  if [[ "$status" == 3 ]]; then
    echo ">> seed skipped existing document pk=${pk}"
    return 0
  fi
  return "$status"
}

seed_project_doc() {
  local project_id="$1" owner_github_id="$2" name="$3"
  local created_at data
  created_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  data="$(jq -nc \
    --arg pid "$project_id" \
    --argjson owner "$owner_github_id" \
    --arg name "$name" \
    --arg created "$created_at" \
    '{project_id:$pid, owner_github_id:$owner, name:$name, created_at:$created,
      telemetry_policy:{revision:1, base_retention:"30d", log_retention_override:null, trace_retention_override:null, metric_retention_override:null, max_stored_bytes:"512MiB"}}')"
  echo ">> seed ProjectDoc project_id=${project_id} (insert-only)"
  __seed_insert_only "ProjectDoc/project_id=${project_id}" "$data"
}

seed_fn0_wasmtime_version() {
  local version="$1"
  local data
  data="$(jq -nc --arg v "$version" '{active:$v, pending:null}')"
  echo ">> seed Fn0WasmtimeVersionDoc active=${version} (insert-only)"
  __seed_insert_only "Fn0WasmtimeVersionDoc" "$data"
}

seed_compiled_bundle() {
  local project_id="$1" code_version="$2" wasmtime="$3"
  local cv_padded
  printf -v cv_padded '%020d' "$code_version"
  local pk="CompiledBundleDoc/project_id=${project_id}&code_version=${cv_padded}"
  local data
  data="$(jq -nc \
    --arg pid "$project_id" \
    --argjson cv "$code_version" \
    --arg w "$wasmtime" \
    '{project_id:$pid, code_version:$cv, fn0_wasmtime_versions:[$w]}')"
  echo ">> seed CompiledBundleDoc project=${project_id} code_version=${code_version}"
  control_db_put "$pk" "" "$data"
}

seed_worker_manifest() {
  local project_id="$1" code_version="$2" domain="$3"
  local observed found revision existing_data merged mv attempt status
  for attempt in $(seq 1 12); do
    observed="$(control_db_get_observed "WorkerManifestDoc" "")"
    found="$(jq -r '.found' <<<"$observed")"
    revision="$(jq -r '.revision' <<<"$observed")"
    existing_data="$(jq -r '.data_base64 // empty | @base64d' <<<"$observed")"
    if [[ "$found" != true ]]; then existing_data='{"manifest_version":0,"project_manifests":{}}'; fi
    merged="$(jq -c --arg pid "$project_id" --argjson cv "$code_version" --arg dom "$domain" --argjson floor "$code_version" '
      .project_manifests[$pid] = (((.project_manifests[$pid] // {}) | del(.custom_domain)) + {code_version:$cv, domain:$dom, static_cache_state:"active", pending_code_version:null})
      | .manifest_version = ([(.manifest_version // 0) + 1, $floor] | max)
    ' <<<"$existing_data")"
    if [[ "$found" == true ]]; then
      if control_db_put_if_revision "WorkerManifestDoc" "" "$revision" "$merged" >/dev/null; then break; else status=$?; fi
    else
      if control_db_put_if_missing "WorkerManifestDoc" "" "$merged" >/dev/null; then break; else status=$?; fi
    fi
    if [[ "$status" != 3 ]]; then return "$status"; fi
    if [[ "$attempt" == 12 ]]; then
      echo "WorkerManifestDoc CAS retry exhausted after 12 conflicts" >&2
      return 1
    fi
  done
  mv="$(jq -r '.manifest_version' <<<"$merged")"
  echo ">> seed WorkerManifestDoc project=${project_id} domain=${domain:-<none>} manifest_version=${mv} (siblings preserved)"
}

seed_target_fn0_worker_config() {
  local image_ref="$1"
  local data
  data="$(jq -nc --arg r "$image_ref" '{image_ref:$r}')"
  echo ">> seed TargetFn0WorkerConfigDoc image_ref=${image_ref}"
  control_db_put "TargetFn0WorkerConfigDoc" "" "$data"
}

# seed_cloudflare_config <project_id> <account_id> <zone_id> <zone_name>
#   <worker_key_id> <worker_secret_ct> <asset_key_id> <asset_secret_ct>
#   <purge_token_ct>
#
# Stands in for the `cloudflare_connect` action, which cannot run here because
# it posts to a control plane that is not serving yet. Upsert rather than
# insert-only: connect refuses a project that is already connected, so a
# re-bootstrap has no other way to correct a credential.
seed_cloudflare_config() {
  local project_id="$1" account_id="$2" zone_id="$3" zone_name="$4"
  local worker_key_id="$5" worker_secret_ct="$6"
  local asset_key_id="$7" asset_secret_ct="$8" purge_token_ct="$9"
  local checked_at data
  checked_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  data="$(jq -nc \
    --arg pid "$project_id" \
    --arg acct "$account_id" \
    --arg zid "$zone_id" \
    --arg zname "$zone_name" \
    --arg wkey "$worker_key_id" \
    --arg wsec "$worker_secret_ct" \
    --arg akey "$asset_key_id" \
    --arg asec "$asset_secret_ct" \
    --arg purge "$purge_token_ct" \
    --arg checked "$checked_at" '{
      project_id:$pid,
      account_id:$acct,
      zone_id:$zid,
      zone_name:$zname,
      frontend_asset_hostname:("fn0-" + $pid + "-frontend-asset." + $zname),
      public_object_storage_hostname:("fn0-" + $pid + "-public-object-storage." + $zname),
      private_object_storage_bucket:("fn0-" + $pid + "-private-object-storage"),
      public_object_storage_bucket:("fn0-" + $pid + "-public-object-storage"),
      frontend_asset_bucket:("fn0-" + $pid + "-frontend-asset"),
      worker_access_key_id:$wkey,
      worker_secret_ciphertext:$wsec,
      frontend_asset_access_key_id:$akey,
      frontend_asset_secret_ciphertext:$asec,
      purge_token_ciphertext:$purge,
      state:"Ok",
      checked_at:$checked,
      config_version:1
    }')"
  echo ">> seed ProjectCloudflareConfigDoc project_id=${project_id}"
  control_db_put "ProjectCloudflareConfigDoc/project_id=${project_id}" "" "$data"
}

# seed_manifest_storage <project_id>
#
# The bash half of `connect_project`, which is what puts a connected project's
# storage in front of the fleet. Seeding ProjectCloudflareConfigDoc alone tells
# control where the buckets are but leaves workers with no target, so the
# project's guest code gets no object storage.
#
# Derived from the seeded config rather than from the caller's variables so the
# bucket names have exactly one source.
seed_manifest_storage() {
  local project_id="$1"
  local config storage observed found revision existing merged attempt status

  observed="$(control_db_get_observed "ProjectCloudflareConfigDoc/project_id=${project_id}" "")"
  config="$(jq -r '.data_base64 // empty | @base64d' <<<"$observed")"
  if [[ -z "$config" ]]; then
    echo "seed_manifest_storage: no ProjectCloudflareConfigDoc for ${project_id}" >&2
    return 1
  fi

  storage="$(jq -c '{
    account_id,
    region: "auto",
    credential: {access_key_id: .worker_access_key_id, secret_ciphertext: .worker_secret_ciphertext},
    private_object_storage_bucket,
    public_object_storage_bucket,
    public_object_storage_base_url: ("https://" + .public_object_storage_hostname),
    config_version
  }' <<<"$config")"

  for attempt in $(seq 1 12); do
    observed="$(control_db_get_observed "WorkerManifestDoc" "")"
    found="$(jq -r '.found' <<<"$observed")"
    if [[ "$found" != true ]]; then
      echo "seed_manifest_storage: no WorkerManifestDoc; seed the manifest first" >&2
      return 1
    fi
    revision="$(jq -r '.revision' <<<"$observed")"
    existing="$(jq -r '.data_base64 | @base64d' <<<"$observed")"
    merged="$(jq -c --arg pid "$project_id" --argjson storage "$storage" '
      if .project_manifests[$pid].storage == $storage then .
      else .project_manifests[$pid].storage = $storage | .manifest_version += 1
      end
    ' <<<"$existing")"
    if [[ "$merged" == "$existing" ]]; then break; fi
    if control_db_put_if_revision "WorkerManifestDoc" "" "$revision" "$merged" >/dev/null; then break; else status=$?; fi
    if [[ "$status" != 3 ]]; then return "$status"; fi
    if [[ "$attempt" == 12 ]]; then
      echo "WorkerManifestDoc storage CAS retry exhausted after 12 conflicts" >&2
      return 1
    fi
  done
  if [[ "$found" != true ]]; then
    echo "seed_manifest_storage: no WorkerManifestDoc; seed the manifest first" >&2
    return 1
  fi
  echo ">> seed WorkerManifestDoc storage for project=${project_id} manifest_version=$(jq -r '.manifest_version' <<<"$merged")"
}
