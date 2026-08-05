# shellcheck shell=bash
# Uploads fn0-control's `fe/dist` under `{build_id}/` in its frontend-asset
# bucket. The bucket itself is provisioned by control-cloudflare.sh.
#
# Source-only.

if [[ -n "${__FN0_CONTROL_STATIC_UPLOAD_LOADED:-}" ]]; then
  return 0
fi
__FN0_CONTROL_STATIC_UPLOAD_LOADED=1

# Mirrors STATIC_ASSET_CACHE_CONTROL in
# fn0/control/rs/src/actions/deploy/mod.rs, which every other project's assets
# are stored with. Without it the objects carry no Cache-Control, and the zone's
# cache rule bypasses the edge for exactly those — every visitor's asset request
# would go to R2 instead of the nearest colo. Each build lands under its own
# `build_id`, so the objects really are immutable.
STATIC_ASSET_CACHE_CONTROL="public, max-age=31536000, immutable"

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
      --cache-control "$STATIC_ASSET_CACHE_CONTROL" \
      "$fe_dist_dir" "s3://${bucket}/${build_id}/" \
      >/dev/null
}
