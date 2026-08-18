# shellcheck shell=bash
# Provisions fn0-control's own Cloudflare resources and records them, standing
# in for `forte cloudflare connect` — which cannot be used here because it
# posts to a control plane that is not serving yet.
#
# Mirrors fn0/deploy/src/cloudflare_provision.rs. When that file changes, this
# one has to follow: the two produce the same three buckets, the same two
# hostnames and the same cache rule.
#
# Source-only.

if [[ -n "${__FN0_CONTROL_CLOUDFLARE_LOADED:-}" ]]; then
  return 0
fi
__FN0_CONTROL_CLOUDFLARE_LOADED=1

# Named after the constants of the same value in cloudflare_provision.rs, so the
# two permission sets can be diffed by eye. ZONE_TRANSFORM_RULES_WRITE is in the
# Rust set and deliberately absent here: nothing on this path writes a transform
# rule, and the provisioning token should not carry a permission it never uses.
readonly __CF_PERMISSION_R2_STORAGE_WRITE="bf7481a1826f439697cb59a20b22293e"
readonly __CF_PERMISSION_R2_BUCKET_ITEM_READ="6a018a9f2fc74eb6b293b0c548f38b39"
readonly __CF_PERMISSION_R2_BUCKET_ITEM_WRITE="2efd5506f9c8494dacb1fa10a3e7d5b6"
readonly __CF_PERMISSION_ZONE_READ="c8fed203ed3043cba015a93ad1616f1f"
readonly __CF_PERMISSION_CACHE_SETTINGS_WRITE="9ff81cbbe65c400b97d92c3c1033cab6"
readonly __CF_PERMISSION_ZONE_SETTINGS_WRITE="3030687196b94b638145a3953da2b699"
readonly __CF_PERMISSION_SSL_AND_CERTIFICATES_WRITE="c03055bc037c4ea9afb9a9f104b7b721"
readonly __CF_PERMISSION_CACHE_PURGE="e17beae8b8cb423a99b1730f21238bed"

__cf_call() {
  local method="$1" path="$2" token="$3" body="${4:-}"
  local args=(-sS -o /tmp/__cf_resp.body -w '%{http_code}'
    -X "$method" "https://api.cloudflare.com/client/v4${path}"
    -H "Authorization: Bearer ${token}"
    -H "Content-Type: application/json")
  [[ -n "$body" ]] && args+=(--data "$body")
  curl "${args[@]}"
}

# Deliberately excludes "already in use": Cloudflare answers that when a
# hostname is attached to a different bucket, which is a failure to report
# rather than a step to skip.
__cf_already_exists() {
  jq -r '.errors // [] | map(.message // "" | ascii_downcase) | .[]' < /tmp/__cf_resp.body 2>/dev/null \
    | grep -qE "already exists|already configured|duplicate"
}

__cf_expect() {
  local code="$1" what="$2" allow_exists="${3:-}"
  [[ "$code" =~ ^2 ]] && return 0
  if [[ -n "$allow_exists" ]] && __cf_already_exists; then
    echo "   ${what}: already there, ok"
    return 0
  fi
  echo "${what} failed (status=${code}):" >&2
  cat /tmp/__cf_resp.body >&2
  return 1
}

control_bucket_names() {
  local project_id="$1"
  echo "fn0-${project_id}-private-object-storage"
  echo "fn0-${project_id}-public-object-storage"
  echo "fn0-${project_id}-frontend-asset"
}

# provision_control_cloudflare <account_id> <cf_token> <zone_id> <zone_name> <project_id> <app_hostname>
provision_control_cloudflare() {
  local account_id="$1" cf_token="$2" zone_id="$3" zone_name="$4" project_id="$5" app_hostname="$6"
  local private_bucket public_bucket asset_bucket code body

  private_bucket="fn0-${project_id}-private-object-storage"
  public_bucket="fn0-${project_id}-public-object-storage"
  asset_bucket="fn0-${project_id}-frontend-asset"

  local bucket
  for bucket in "$private_bucket" "$public_bucket" "$asset_bucket"; do
    echo ">> ensure R2 bucket ${bucket}"
    body=$(jq -nc --arg name "$bucket" '{name:$name, locationHint:"apac"}')
    code=$(__cf_call POST "/accounts/${account_id}/r2/buckets" "$cf_token" "$body")
    __cf_expect "$code" "create bucket ${bucket}" allow_exists || return 1
  done

  # The one origin control answers on, matching `put_cors` in
  # cloudflare_provision.rs. A wider allowlist is not just permissive: the
  # purge in queue_task/public_object_purge.rs enumerates the entries a
  # browser can build from these origins, and an unbounded set cannot be
  # enumerated, so a stale entry would outlive every purge.
  for bucket in "$private_bucket" "$public_bucket" "$asset_bucket"; do
    echo ">> put R2 CORS on ${bucket}"
    body="$(jq -nc --arg origin "https://${app_hostname}" '{rules:[{allowed:{methods:["GET","PUT","HEAD"],origins:[$origin],headers:["*"]},exposeHeaders:["ETag"],maxAgeSeconds:86400}]}')"
    code=$(__cf_call PUT "/accounts/${account_id}/r2/buckets/${bucket}/cors" "$cf_token" "$body")
    __cf_expect "$code" "put CORS on ${bucket}" || return 1
  done

  # A public bucket answers on its own name in the zone, so the bucket and the
  # address it serves from cannot drift apart.
  for bucket in "$asset_bucket" "$public_bucket"; do
    local hostname="${bucket}.${zone_name}"
    # "Domain already in use" means attached to *some* bucket, which is a
    # failure worth reporting unless the bucket is this one — as it is on a
    # re-run. Asking which bucket holds it separates the two.
    code=$(__cf_call GET "/accounts/${account_id}/r2/buckets/${bucket}/domains/custom" "$cf_token")
    if [[ "$code" =~ ^2 ]] && jq -e --arg d "$hostname" \
      '[(.result.domains // .result // [])[] | .domain] | index($d)' < /tmp/__cf_resp.body >/dev/null 2>&1; then
      echo ">> ${hostname} already points at ${bucket}"
    else
      echo ">> attach ${hostname} -> ${bucket}"
      body=$(jq -nc --arg d "$hostname" --arg z "$zone_id" '{domain:$d, zoneId:$z, enabled:true, minTLS:"1.2"}')
      code=$(__cf_call POST "/accounts/${account_id}/r2/buckets/${bucket}/custom_domains" "$cf_token" "$body")
      __cf_expect "$code" "attach ${hostname}" allow_exists || return 1
    fi

    # The POST answers enabled:false even when asked for true. Idempotent.
    body='{"enabled":true}'
    code=$(__cf_call PUT "/accounts/${account_id}/r2/buckets/${bucket}/domains/custom/${hostname}" "$cf_token" "$body")
    __cf_expect "$code" "enable ${hostname}" || return 1
  done

  ensure_fn0_cache_rule "$cf_token" "$zone_id" "$zone_name" "$app_hostname"
  ensure_smart_tiered_cache "$cf_token" "$zone_id"
}

# ensure_fn0_cache_rule <cf_token> <zone_id> <zone_name> <app_hostname>
# One rule for the whole zone, matched by wildcard, so it is written once and
# never grows: a free zone allows ten cache rules and a rule per project would
# run out at ten projects.
ensure_fn0_cache_rule() {
  local cf_token="$1" zone_id="$2" zone_name="$3" app_hostname="$4"
  local description="fn0 frontend assets and public objects"
  local path="/zones/${zone_id}/rulesets/phases/http_request_cache_settings/entrypoint"
  local code rules expression merged

  code=$(__cf_call GET "$path" "$cf_token")
  if [[ "$code" =~ ^2 ]]; then
    rules="$(jq -c '.result.rules // []' < /tmp/__cf_resp.body)"
  elif [[ "$code" == "404" ]]; then
    # A zone with no cache rules has no entrypoint ruleset at all.
    rules='[]'
  else
    echo "read cache rules failed (status=${code}):" >&2
    cat /tmp/__cf_resp.body >&2
    return 1
  fi

  expression="((http.host wildcard \"fn0-*-frontend-asset.${zone_name}\" or http.host wildcard \"fn0-*-public-object-storage.${zone_name}\" or http.host in {\"${app_hostname}\"}) and http.request.method in {\"GET\" \"HEAD\" \"PURGE\"})"

  # Ahead of the zone's own rules: first match wins, and a broad user rule that
  # disabled caching would otherwise swallow these hostnames.
  merged="$(jq -c --arg desc "$description" --arg expr "$expression" '
    map(select(.description != $desc))
    | [{action:"set_cache_settings", expression:$expr, description:$desc,
        action_parameters:{cache:true, browser_ttl:{mode:"respect_origin"}}}] + .
  ' <<<"$rules")"

  echo ">> ensure fn0 cache rule on zone ${zone_name}"
  code=$(__cf_call PUT "$path" "$cf_token" "$(jq -nc --argjson r "$merged" '{rules:$r}')")
  __cf_expect "$code" "write cache rules"
}

ensure_smart_tiered_cache() {
  local cf_token="$1" zone_id="$2"
  local code
  code=$(__cf_call PATCH "/zones/${zone_id}/cache/tiered_cache_smart_topology_enable" "$cf_token" '{"value":"on"}')
  __cf_expect "$code" "enable Smart Tiered Cache"
}

# mint_provisioning_token <user_token> <account_id> <zone_id> <purpose>
# Echoes "<token_id> <token_value>".
#
# The token that provisions is not the token that mints. `cloudflareUserApiToken`
# carries API Tokens -> Edit and nothing else, so it can create this one but
# cannot write a cache rule itself; this one can provision but cannot create
# tokens. Cloudflare grants a new token any permission the owning *user* holds,
# not merely what the creating token holds — which is what makes the split work.
mint_provisioning_token() {
  local user_token="$1" account_id="$2" zone_id="$3" purpose="$4"
  local expires body code
  expires="$(python3 -c "import datetime; print((datetime.datetime.now(datetime.UTC) + datetime.timedelta(minutes=${FN0_PROVISIONING_TOKEN_MINUTES:-15})).strftime('%Y-%m-%dT%H:%M:%SZ'))")"
  body="$(jq -nc \
    --arg n "fn0 setup (${purpose})" \
    --arg exp "$expires" \
    --arg acct "com.cloudflare.api.account.${account_id}" \
    --arg zone "com.cloudflare.api.account.zone.${zone_id}" \
    --arg r2_storage_write "$__CF_PERMISSION_R2_STORAGE_WRITE" \
    --arg zone_read "$__CF_PERMISSION_ZONE_READ" \
    --arg cache_settings_write "$__CF_PERMISSION_CACHE_SETTINGS_WRITE" \
    --arg zone_settings_write "$__CF_PERMISSION_ZONE_SETTINGS_WRITE" \
    --arg ssl_and_certificates_write "$__CF_PERMISSION_SSL_AND_CERTIFICATES_WRITE" '{
      name:$n,
      expires_on:$exp,
      policies:[
        {effect:"allow", resources:{($acct):"*"},
         permission_groups:[{id:$r2_storage_write}]},
        {effect:"allow", resources:{($zone):"*"},
         permission_groups:[
           {id:$zone_read},
           {id:$cache_settings_write},
           {id:$zone_settings_write},
           {id:$ssl_and_certificates_write}
         ]}
      ]
    }')"
  code=$(__cf_call POST "/user/tokens" "$user_token" "$body")
  __cf_expect "$code" "mint provisioning token (${purpose})" || return 1
  jq -r '"\(.result.id) \(.result.value)"' < /tmp/__cf_resp.body
}

# revoke_provisioning_token <user_token> <token_id>
revoke_provisioning_token() {
  local user_token="$1" token_id="$2"
  local code
  code=$(__cf_call DELETE "/user/tokens/${token_id}" "$user_token")
  if [[ ! "$code" =~ ^2 ]]; then
    echo "warning: could not revoke provisioning token ${token_id} (status=${code}); it expires by itself" >&2
  fi
}

# mint_r2_token <cf_token> <account_id> <name> <bucket>...
# Echoes "<access_key_id> <secret_access_key>". R2 takes the SHA-256 of the
# token value as the S3 secret access key.
mint_r2_token() {
  local cf_token="$1" account_id="$2" name="$3"
  shift 3
  local resources body code token_id token_value secret
  resources="$(printf '%s\n' "$@" | jq -R . | jq -sc --arg a "$account_id" \
    'map({key:("com.cloudflare.edge.r2.bucket." + $a + "_default_" + .), value:"*"}) | from_entries')"
  body="$(jq -nc --arg n "$name" --argjson res "$resources" \
    --arg bucket_item_read "$__CF_PERMISSION_R2_BUCKET_ITEM_READ" \
    --arg bucket_item_write "$__CF_PERMISSION_R2_BUCKET_ITEM_WRITE" '{
    name:$n,
    policies:[{effect:"allow", resources:$res, permission_groups:[
      {id:$bucket_item_read},
      {id:$bucket_item_write}
    ]}]
  }')"
  code=$(__cf_call POST "/user/tokens" "$cf_token" "$body")
  __cf_expect "$code" "mint R2 token ${name}" || return 1
  token_id="$(jq -r '.result.id' < /tmp/__cf_resp.body)"
  token_value="$(jq -r '.result.value' < /tmp/__cf_resp.body)"
  secret="$(printf '%s' "$token_value" | shasum -a 256 | cut -d' ' -f1)"
  echo "${token_id} ${secret}"
}

# mint_purge_token <cf_token> <zone_id> <name>
# Echoes "<token_id> <token_value>". The id is not stored anywhere downstream —
# only the value is, encrypted — so revoke_superseded_tokens has no other way to
# tell this run's token from the ones it replaced.
mint_purge_token() {
  local cf_token="$1" zone_id="$2" name="$3"
  local body code
  body="$(jq -nc --arg n "$name" --arg z "$zone_id" \
    --arg cache_purge "$__CF_PERMISSION_CACHE_PURGE" '{
    name:$n,
    policies:[{effect:"allow",
      resources:{("com.cloudflare.api.account.zone." + $z):"*"},
      permission_groups:[{id:$cache_purge}]}]
  }')"
  code=$(__cf_call POST "/user/tokens" "$cf_token" "$body")
  __cf_expect "$code" "mint purge token ${name}" || return 1
  jq -r '"\(.result.id) \(.result.value)"' < /tmp/__cf_resp.body
}

# revoke_superseded_tokens <user_token> <name> <keep_id>
# Every run mints a fresh credential under a fixed per-project name, so without
# this the ones it replaced stay valid forever. Call only once the doc that
# references <keep_id> is written: a crash before that point should leave the
# superseded token usable rather than strand control with a credential that no
# longer exists.
revoke_superseded_tokens() {
  local user_token="$1" name="$2" keep_id="$3"
  local page=1 total_pages=1 code stale_ids="" id
  # Collected across every page before the first delete: deleting mid-scan
  # renumbers the pages that have not been read yet.
  while (( page <= total_pages )); do
    code=$(__cf_call GET "/user/tokens?per_page=200&page=${page}" "$user_token")
    __cf_expect "$code" "list user tokens" || return 1
    total_pages="$(jq -r '.result_info.total_pages // 1' < /tmp/__cf_resp.body)"
    stale_ids+="$(jq -r --arg n "$name" --arg keep "$keep_id" \
      '(.result // [])[] | select(.name == $n and .id != $keep) | .id' < /tmp/__cf_resp.body)"$'\n'
    page=$((page + 1))
  done
  while read -r id; do
    [[ -z "$id" ]] && continue
    echo ">> revoke superseded token ${name} (${id})"
    code=$(__cf_call DELETE "/user/tokens/${id}" "$user_token")
    __cf_expect "$code" "revoke ${name} (${id})" || return 1
  done <<<"$stale_ids"
}

# kms_encrypt <crypto_endpoint> <key_ocid> <plaintext>
# Same ciphertext form control's `vault::encrypt` produces: the secret goes
# straight under the KMS master key, with no data key in between.
kms_encrypt() {
  local endpoint="$1" key_ocid="$2" plaintext="$3"
  oci kms crypto encrypt \
    --endpoint "$endpoint" \
    --key-id "$key_ocid" \
    --plaintext "$(printf '%s' "$plaintext" | base64)" \
    --query 'data.ciphertext' --raw-output
}

# kms_decrypt <crypto_endpoint> <key_ocid> <ciphertext>
# Recovers what kms_encrypt stored. Without this the bootstrap has no way to
# learn a credential it already holds — Cloudflare hands a token's value out
# once, at creation — so every run had to mint a replacement.
kms_decrypt() {
  local endpoint="$1" key_ocid="$2" ciphertext="$3"
  oci kms crypto decrypt \
    --endpoint "$endpoint" \
    --key-id "$key_ocid" \
    --ciphertext "$ciphertext" \
    --query 'data.plaintext' --raw-output \
    | base64 -d
}

# r2_credential_usable <account_id> <access_key_id> <secret_access_key> <bucket>...
# True when the credential opens every bucket named, which is the same list
# mint_r2_token would scope a new one to. Checking all of them is what makes a
# widened policy re-mint on its own: an older token opens the buckets it was
# minted for and fails the one that was added.
r2_credential_usable() {
  local account_id="$1" access_key_id="$2" secret_access_key="$3"
  shift 3
  local endpoint="https://${account_id}.r2.cloudflarestorage.com" bucket
  for bucket in "$@"; do
    if ! AWS_ACCESS_KEY_ID="$access_key_id" \
         AWS_SECRET_ACCESS_KEY="$secret_access_key" \
         AWS_DEFAULT_REGION=auto \
         AWS_SESSION_TOKEN= \
         aws s3api head-bucket --endpoint-url "$endpoint" --bucket "$bucket" \
         >/dev/null 2>&1; then
      return 1
    fi
  done
  return 0
}

# purge_token_usable <token>
# `/user/tokens/verify` reports whether the token is live but not which zone it
# covers, so the caller has to compare the stored zone id itself.
purge_token_usable() {
  local token="$1" code
  code=$(__cf_call GET "/user/tokens/verify" "$token")
  [[ "$code" == "200" ]] && \
    [[ "$(jq -r '.result.status // empty' < /tmp/__cf_resp.body)" == "active" ]]
}
