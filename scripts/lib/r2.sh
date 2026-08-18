# shellcheck shell=bash
# Bundle-store R2 access. Source-only.
#
# r2_list_objects <endpoint> <bucket> <access_key_id> <secret_access_key> <prefix>
#   Emits one {Key, LastModified} JSON object per line, following pagination.
#
# r2_get_object <endpoint> <bucket> <access_key_id> <secret_access_key> <key> <dest>

if [[ -n "${__FN0_R2_LOADED:-}" ]]; then
  return 0
fi
__FN0_R2_LOADED=1

r2_list_objects() {
  local r2_endpoint="$1" r2_bucket="$2" r2_ak="$3" r2_sk="$4" prefix="$5"
  local token="" out=""
  while :; do
    if [[ -n "$token" ]]; then
      out="$(AWS_ACCESS_KEY_ID="$r2_ak" \
             AWS_SECRET_ACCESS_KEY="$r2_sk" \
             AWS_DEFAULT_REGION=auto \
             AWS_SESSION_TOKEN= \
             aws s3api list-objects-v2 \
               --endpoint-url "$r2_endpoint" \
               --bucket "$r2_bucket" \
               --prefix "$prefix" \
               --starting-token "$token" \
               --output json)"
    else
      out="$(AWS_ACCESS_KEY_ID="$r2_ak" \
             AWS_SECRET_ACCESS_KEY="$r2_sk" \
             AWS_DEFAULT_REGION=auto \
             AWS_SESSION_TOKEN= \
             aws s3api list-objects-v2 \
               --endpoint-url "$r2_endpoint" \
               --bucket "$r2_bucket" \
               --prefix "$prefix" \
               --output json)"
    fi
    jq -c '.Contents[]? | {Key, LastModified}' <<<"$out"
    token="$(jq -r '.NextContinuationToken // empty' <<<"$out")"
    [[ -z "$token" ]] && break
  done
}

r2_get_object() {
  local r2_endpoint="$1" r2_bucket="$2" r2_ak="$3" r2_sk="$4" key="$5" dest="$6"
  AWS_ACCESS_KEY_ID="$r2_ak" \
  AWS_SECRET_ACCESS_KEY="$r2_sk" \
  AWS_DEFAULT_REGION=auto \
  AWS_SESSION_TOKEN= \
  aws s3api get-object \
    --endpoint-url "$r2_endpoint" \
    --bucket "$r2_bucket" \
    --key "$key" \
    "$dest" >/dev/null
}
