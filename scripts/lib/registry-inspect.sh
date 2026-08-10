# shellcheck shell=bash
# Remote registry v2 inspection over plain HTTP (curl + jq). Source-only.
#
# Exists so "does this tag exist / what does it carry" never depends on the
# local container runtime: `docker manifest inspect` needs docker plugins and
# apple/container has no remote inspect at all. Speaks both bearer-challenge
# (OCIR, GHCR) and basic-auth (ECR) registries.

if [[ -n "${__FN0_REGISTRY_INSPECT_LOADED:-}" ]]; then
  return 0
fi
__FN0_REGISTRY_INSPECT_LOADED=1

REGISTRY_INSPECT_ACCEPT_MANIFEST_TYPES="application/vnd.oci.image.index.v1+json,application/vnd.oci.image.manifest.v1+json,application/vnd.docker.distribution.manifest.list.v2+json,application/vnd.docker.distribution.manifest.v2+json"

__registry_inspect_sha256_file() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{print $1}'
  else
    shasum -a 256 "$1" | awk '{print $1}'
  fi
}

# Prints a bearer token for pull scope, or nothing when the registry does not
# issue a bearer challenge (then plain basic auth is the right fallback).
__registry_inspect_bearer_token() {
  local registry_url="$1" repository="$2" username="$3" password="$4"
  local challenge realm service
  challenge="$(curl -sS -o /dev/null -D - "https://${registry_url}/v2/" 2>/dev/null \
    | tr -d '\r' | grep -i '^www-authenticate: *bearer' || true)"
  if [[ -z "$challenge" ]]; then
    return 0
  fi
  realm="$(sed -E 's/.*realm="([^"]+)".*/\1/' <<<"$challenge")"
  service="$(sed -E 's/.*service="([^"]+)".*/\1/' <<<"$challenge")"
  curl -sS -u "${username}:${password}" \
    "${realm}?service=${service}&scope=repository:${repository}:pull" \
    | jq -r '.token // .access_token // empty'
}

# Fills REGISTRY_INSPECT_AUTHORIZATION_ARGS with curl auth arguments.
__registry_inspect_authorization_args() {
  local registry_url="$1" repository="$2" username="$3" password="$4"
  local token
  token="$(__registry_inspect_bearer_token "$registry_url" "$repository" "$username" "$password")"
  if [[ -n "$token" ]]; then
    REGISTRY_INSPECT_AUTHORIZATION_ARGS=(-H "Authorization: Bearer ${token}")
  else
    REGISTRY_INSPECT_AUTHORIZATION_ARGS=(-u "${username}:${password}")
  fi
}

# registry_inspect_manifest_fetch <registry_url> <repository> <reference> <username> <password> <output_file>
# Returns 0 with the manifest written to <output_file>, 1 when the reference
# does not exist, 2 on any other failure.
registry_inspect_manifest_fetch() {
  local registry_url="$1" repository="$2" reference="$3" username="$4" password="$5" output_file="$6"
  local http_status
  __registry_inspect_authorization_args "$registry_url" "$repository" "$username" "$password"
  if ! http_status="$(curl -sS -o "$output_file" -w '%{http_code}' \
    "${REGISTRY_INSPECT_AUTHORIZATION_ARGS[@]}" \
    -H "Accept: ${REGISTRY_INSPECT_ACCEPT_MANIFEST_TYPES}" \
    "https://${registry_url}/v2/${repository}/manifests/${reference}")"; then
    echo "registry request failed: ${registry_url}/${repository}:${reference}" >&2
    return 2
  fi
  case "$http_status" in
    200) return 0 ;;
    404) return 1 ;;
    *)
      echo "registry manifest GET ${registry_url}/${repository}:${reference} -> HTTP ${http_status}" >&2
      return 2
      ;;
  esac
}

# registry_inspect_manifest_exists <registry_url> <repository> <reference> <username> <password>
# Same return contract as registry_inspect_manifest_fetch.
registry_inspect_manifest_exists() {
  local manifest_file fetch_status
  manifest_file="$(mktemp)"
  if registry_inspect_manifest_fetch "$1" "$2" "$3" "$4" "$5" "$manifest_file"; then
    fetch_status=0
  else
    fetch_status=$?
  fi
  rm -f "$manifest_file"
  return "$fetch_status"
}

# registry_inspect_arm64_manifest <registry_url> <repository> <reference> <username> <password> <output_file>
# Resolves <reference> (following an index to its linux/arm64 entry) to a
# single image manifest: writes the manifest JSON to <output_file> and prints
# its digest. Returns 0/1/2 like registry_inspect_manifest_fetch.
registry_inspect_arm64_manifest() {
  local registry_url="$1" repository="$2" reference="$3" username="$4" password="$5" output_file="$6"
  local fetch_status arm64_digest
  if registry_inspect_manifest_fetch "$registry_url" "$repository" "$reference" "$username" "$password" "$output_file"; then
    fetch_status=0
  else
    fetch_status=$?
  fi
  if [[ "$fetch_status" -ne 0 ]]; then
    return "$fetch_status"
  fi

  if jq -e '.manifests' "$output_file" >/dev/null 2>&1; then
    arm64_digest="$(jq -r '[.manifests[] | select(.platform.architecture == "arm64" and .platform.os == "linux")][0].digest // empty' "$output_file")"
    if [[ -z "$arm64_digest" ]]; then
      echo "${registry_url}/${repository}:${reference} is an index with no linux/arm64 entry" >&2
      return 2
    fi
    if ! registry_inspect_manifest_fetch "$registry_url" "$repository" "$arm64_digest" "$username" "$password" "$output_file"; then
      echo "failed to fetch ${registry_url}/${repository}@${arm64_digest}" >&2
      return 2
    fi
    printf '%s\n' "$arm64_digest"
    return 0
  fi

  printf 'sha256:%s\n' "$(__registry_inspect_sha256_file "$output_file")"
}

# registry_inspect_blob_fetch <registry_url> <repository> <digest>  <username> <password>
# Prints the blob to stdout. Follows the storage redirect registries answer
# blob GETs with; curl drops the Authorization header on the cross-host hop,
# which the presigned redirect target requires.
registry_inspect_blob_fetch() {
  local registry_url="$1" repository="$2" digest="$3" username="$4" password="$5"
  __registry_inspect_authorization_args "$registry_url" "$repository" "$username" "$password"
  curl -sS -L "${REGISTRY_INSPECT_AUTHORIZATION_ARGS[@]}" \
    "https://${registry_url}/v2/${repository}/blobs/${digest}"
}

# registry_inspect_image_label <registry_url> <repository> <reference> <username> <password> <label_key>
# Prints the label value of the reference's linux/arm64 image, or nothing when
# the label, the image, or the registry itself cannot be read. Always
# returns 0: callers treat "no label" and "unreadable" alike.
registry_inspect_image_label() {
  local registry_url="$1" repository="$2" reference="$3" username="$4" password="$5" label_key="$6"
  local manifest_file config_digest
  manifest_file="$(mktemp)"
  if ! registry_inspect_arm64_manifest "$registry_url" "$repository" "$reference" "$username" "$password" "$manifest_file" >/dev/null 2>&1; then
    rm -f "$manifest_file"
    return 0
  fi
  config_digest="$(jq -r '.config.digest // empty' "$manifest_file")"
  rm -f "$manifest_file"
  if [[ -z "$config_digest" ]]; then
    return 0
  fi
  registry_inspect_blob_fetch "$registry_url" "$repository" "$config_digest" "$username" "$password" 2>/dev/null \
    | jq -r --arg label "$label_key" '.config.Labels[$label] // empty' 2>/dev/null || true
}
