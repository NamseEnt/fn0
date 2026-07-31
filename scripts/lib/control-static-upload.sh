# shellcheck shell=bash
# Uploads fn0-control's `fe/dist` under `{build_id}/` in its frontend-asset
# bucket. The bucket itself is provisioned by control-cloudflare.sh.
#
# Source-only.

if [[ -n "${__FN0_CONTROL_STATIC_UPLOAD_LOADED:-}" ]]; then
  return 0
fi
__FN0_CONTROL_STATIC_UPLOAD_LOADED=1

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
