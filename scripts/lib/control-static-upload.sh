# shellcheck shell=bash
# Provision the per-project R2 static-asset bucket for fn0-control and
# upload `fe/dist` under `{build_id}/`. Mirrors the bucket provisioning
# performed by fn0-control's own deploy action (see
# fn0/control/rs/src/common/cloudflare.rs) so that the bootstrap path can
# stand fn0-control up without depending on the action being callable.
#
# Source-only.

if [[ -n "${__FN0_CONTROL_STATIC_UPLOAD_LOADED:-}" ]]; then
  return 0
fi
__FN0_CONTROL_STATIC_UPLOAD_LOADED=1

__cf_api_call() {
  local method="$1" path="$2" token="$3" body="${4:-}"
  local url="https://api.cloudflare.com/client/v4${path}"
  local args=(-sS -o /tmp/__cf_resp.body -w '%{http_code}'
    -X "$method" "$url"
    -H "Authorization: Bearer ${token}"
    -H "Content-Type: application/json")
  if [[ -n "$body" ]]; then
    args+=(--data "$body")
  fi
  curl "${args[@]}"
}

__cf_response_indicates_exists() {
  local body_file="$1"
  jq -r '.errors // [] | map(.message // "" | ascii_downcase) | .[]' < "$body_file" 2>/dev/null | grep -qE "already exists|already configured|already in use|duplicate"
}

# ensure_static_asset_bucket <account_id> <cf_token> <bucket> <asset_custom_domain> <zone_id>
# - asset_custom_domain: where the bucket is served (e.g. fn0-control.static.fn0.dev)
# CORS allow-origin is "*": static asset buckets serve publicly readable
# bundles, no credentialed requests cross them, and projects mount the
# assets under arbitrary custom domains added at runtime via fn0 domain
# add — wildcard avoids per-project CORS sync.
ensure_static_asset_bucket() {
  local account_id="$1" cf_token="$2" bucket="$3" asset_custom_domain="$4" zone_id="$5"
  local code body

  echo ">> ensure R2 bucket ${bucket}"
  body=$(jq -nc --arg name "$bucket" '{name:$name, locationHint:"apac"}')
  code=$(__cf_api_call POST "/accounts/${account_id}/r2/buckets" "$cf_token" "$body")
  if [[ ! "$code" =~ ^2 ]]; then
    if __cf_response_indicates_exists /tmp/__cf_resp.body; then
      echo "   bucket already exists, ok"
    else
      echo "create_r2_bucket failed (status=${code}):" >&2
      cat /tmp/__cf_resp.body >&2
      return 1
    fi
  fi

  echo ">> put R2 CORS on ${bucket} (origin=*)"
  body='{"rules":[{"allowed":{"methods":["GET","HEAD"],"origins":["*"],"headers":["*"]},"maxAgeSeconds":86400}]}'
  code=$(__cf_api_call PUT "/accounts/${account_id}/r2/buckets/${bucket}/cors" "$cf_token" "$body")
  if [[ ! "$code" =~ ^2 ]]; then
    echo "put_r2_bucket_cors failed (status=${code}):" >&2
    cat /tmp/__cf_resp.body >&2
    return 1
  fi

  echo ">> register R2 custom domain ${asset_custom_domain}"
  body=$(jq -nc --arg domain "$asset_custom_domain" --arg zid "$zone_id" \
    '{domain:$domain, zoneId:$zid, enabled:true, minTLS:"1.2"}')
  code=$(__cf_api_call POST "/accounts/${account_id}/r2/buckets/${bucket}/custom_domains" "$cf_token" "$body")
  if [[ ! "$code" =~ ^2 ]]; then
    if __cf_response_indicates_exists /tmp/__cf_resp.body; then
      echo "   custom domain already configured, ok"
    else
      echo "register_r2_custom_domain failed (status=${code}):" >&2
      cat /tmp/__cf_resp.body >&2
      return 1
    fi
  fi

  # The custom-domain POST returns enabled:false even when we pass enabled:true,
  # so toggle it explicitly. Idempotent.
  echo ">> enable R2 custom domain ${asset_custom_domain}"
  body=$(jq -nc '{enabled:true}')
  code=$(__cf_api_call PUT "/accounts/${account_id}/r2/buckets/${bucket}/domains/custom/${asset_custom_domain}" "$cf_token" "$body")
  if [[ ! "$code" =~ ^2 ]]; then
    echo "enable_r2_custom_domain failed (status=${code}):" >&2
    cat /tmp/__cf_resp.body >&2
    return 1
  fi
}

# upload_fe_dist <account_id> <access_key_id> <secret_access_key> <bucket> <build_id> <fe_dist_dir>
upload_fe_dist() {
  local account_id="$1" access_key="$2" secret_key="$3" bucket="$4" build_id="$5" fe_dist_dir="$6"
  if [[ ! -d "$fe_dist_dir" ]]; then
    echo "upload_fe_dist: ${fe_dist_dir} not found" >&2
    return 1
  fi
  local endpoint="https://${account_id}.r2.cloudflarestorage.com"
  echo ">> aws s3 sync ${fe_dist_dir} -> s3://${bucket}/${build_id}/"
  AWS_ACCESS_KEY_ID="$access_key" \
  AWS_SECRET_ACCESS_KEY="$secret_key" \
  AWS_DEFAULT_REGION="auto" \
    aws s3 cp --recursive --endpoint-url "$endpoint" \
      "$fe_dist_dir" "s3://${bucket}/${build_id}/" \
      >/dev/null
}
