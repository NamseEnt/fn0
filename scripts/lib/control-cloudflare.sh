# shellcheck shell=bash
# Provisions fn0-control's own Cloudflare resources and records them, standing
# in for `forte cloudflare connect` — which cannot be used here because it
# posts to a control plane that is not serving yet.
#
# Mirrors fn0/deploy/src/cloudflare_provision.rs. When that file changes, this
# one has to follow: the two produce the same four buckets, the same two
# hostnames and the same cache rule.
#
# Source-only.

if [[ -n "${__FN0_CONTROL_CLOUDFLARE_LOADED:-}" ]]; then
  return 0
fi
__FN0_CONTROL_CLOUDFLARE_LOADED=1

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
  echo "fn0-${project_id}-rendered-html-cache"
}

# provision_control_cloudflare <account_id> <cf_token> <zone_id> <zone_name> <project_id>
provision_control_cloudflare() {
  local account_id="$1" cf_token="$2" zone_id="$3" zone_name="$4" project_id="$5"
  local private_bucket public_bucket asset_bucket cache_bucket code body

  private_bucket="fn0-${project_id}-private-object-storage"
  public_bucket="fn0-${project_id}-public-object-storage"
  asset_bucket="fn0-${project_id}-frontend-asset"
  cache_bucket="fn0-${project_id}-rendered-html-cache"

  local bucket
  for bucket in "$private_bucket" "$public_bucket" "$asset_bucket" "$cache_bucket"; do
    echo ">> ensure R2 bucket ${bucket}"
    body=$(jq -nc --arg name "$bucket" '{name:$name, locationHint:"apac"}')
    code=$(__cf_call POST "/accounts/${account_id}/r2/buckets" "$cf_token" "$body")
    __cf_expect "$code" "create bucket ${bucket}" allow_exists || return 1
  done

  # Origin "*" because these buckets serve publicly readable objects and no
  # credentialed request crosses them.
  for bucket in "$private_bucket" "$public_bucket" "$asset_bucket"; do
    echo ">> put R2 CORS on ${bucket}"
    body='{"rules":[{"allowed":{"methods":["GET","PUT","HEAD"],"origins":["*"],"headers":["*"]},"exposeHeaders":["ETag"],"maxAgeSeconds":86400}]}'
    code=$(__cf_call PUT "/accounts/${account_id}/r2/buckets/${bucket}/cors" "$cf_token" "$body")
    __cf_expect "$code" "put CORS on ${bucket}" || return 1
  done

  # A public bucket answers on its own name in the zone, so the bucket and the
  # address it serves from cannot drift apart.
  for bucket in "$asset_bucket" "$public_bucket"; do
    local hostname="${bucket}.${zone_name}"
    echo ">> attach ${hostname} -> ${bucket}"
    body=$(jq -nc --arg d "$hostname" --arg z "$zone_id" '{domain:$d, zoneId:$z, enabled:true, minTLS:"1.2"}')
    code=$(__cf_call POST "/accounts/${account_id}/r2/buckets/${bucket}/custom_domains" "$cf_token" "$body")
    __cf_expect "$code" "attach ${hostname}" allow_exists || return 1

    # The POST answers enabled:false even when asked for true. Idempotent.
    body='{"enabled":true}'
    code=$(__cf_call PUT "/accounts/${account_id}/r2/buckets/${bucket}/domains/custom/${hostname}" "$cf_token" "$body")
    __cf_expect "$code" "enable ${hostname}" || return 1
  done

  ensure_fn0_cache_rule "$cf_token" "$zone_id" "$zone_name"
}

# ensure_fn0_cache_rule <cf_token> <zone_id> <zone_name>
# One rule for the whole zone, matched by wildcard, so it is written once and
# never grows: a free zone allows ten cache rules and a rule per project would
# run out at ten projects.
ensure_fn0_cache_rule() {
  local cf_token="$1" zone_id="$2" zone_name="$3"
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

  expression="((http.host wildcard \"fn0-*-frontend-asset.${zone_name}\" or http.host wildcard \"fn0-*-public-object-storage.${zone_name}\") and http.request.method in {\"GET\" \"HEAD\" \"PURGE\"})"

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
    --arg zone "com.cloudflare.api.account.zone.${zone_id}" '{
      name:$n,
      expires_on:$exp,
      policies:[
        {effect:"allow", resources:{($acct):"*"},
         permission_groups:[{id:"bf7481a1826f439697cb59a20b22293e"}]},
        {effect:"allow", resources:{($zone):"*"},
         permission_groups:[
           {id:"c8fed203ed3043cba015a93ad1616f1f"},
           {id:"9ff81cbbe65c400b97d92c3c1033cab6"},
           {id:"c03055bc037c4ea9afb9a9f104b7b721"}
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
  body="$(jq -nc --arg n "$name" --argjson res "$resources" '{
    name:$n,
    policies:[{effect:"allow", resources:$res, permission_groups:[
      {id:"6a018a9f2fc74eb6b293b0c548f38b39"},
      {id:"2efd5506f9c8494dacb1fa10a3e7d5b6"}
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
mint_purge_token() {
  local cf_token="$1" zone_id="$2" name="$3"
  local body code
  body="$(jq -nc --arg n "$name" --arg z "$zone_id" '{
    name:$n,
    policies:[{effect:"allow",
      resources:{("com.cloudflare.api.account.zone." + $z):"*"},
      permission_groups:[{id:"e17beae8b8cb423a99b1730f21238bed"}]}]
  }')"
  code=$(__cf_call POST "/user/tokens" "$cf_token" "$body")
  __cf_expect "$code" "mint purge token ${name}" || return 1
  jq -r '.result.value' < /tmp/__cf_resp.body
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
