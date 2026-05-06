#!/usr/bin/env bash
# OCIR cleanup library: keep only the latest N SemVer-tagged manifest lists
# and the single-platform manifests they reference; delete everything else.
#
# Required commands: oci, jq, docker (for manifest inspect)
#
# Usage:
#   ocir_cleanup_keep_top_semver \
#     --registry-url   ocir.ap-osaka-1.oci.oraclecloud.com \
#     --repository     <namespace>/<repo-name> \
#     --compartment-id ocid1.compartment.oc1..xxxx \
#     --keep           2

ocir_cleanup_keep_top_semver() {
  local REG_URL="" REPO="" COMP_ID="" KEEP=2 DRY_RUN=0
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --registry-url)   REG_URL="$2"; shift 2 ;;
      --repository)     REPO="$2"; shift 2 ;;
      --compartment-id) COMP_ID="$2"; shift 2 ;;
      --keep)           KEEP="$2"; shift 2 ;;
      --dry-run)        DRY_RUN=1; shift ;;
      *) echo "ocir_cleanup_keep_top_semver: unknown arg: $1" >&2; return 2 ;;
    esac
  done

  if [[ -z "$REG_URL" || -z "$REPO" || -z "$COMP_ID" || -z "$KEEP" ]]; then
    echo "ocir_cleanup_keep_top_semver: missing required args" >&2
    return 2
  fi

  local REPO_NAME="${REPO#*/}"

  echo ">> [cleanup] listing images in ${REPO_NAME}"
  local IMG_LIST
  IMG_LIST=$(oci artifacts container image list \
    --compartment-id "$COMP_ID" \
    --repository-name "$REPO_NAME" \
    --all 2>/dev/null) || {
    echo "ocir_cleanup: failed to list images" >&2
    return 1
  }

  local TOTAL
  TOTAL=$(jq -r '(.data.items // []) | length' <<<"$IMG_LIST")
  if [[ -z "$TOTAL" || "$TOTAL" == "0" ]]; then
    echo ">> [cleanup] no images, nothing to do"
    return 0
  fi
  echo ">> [cleanup] total images: ${TOTAL}"

  local SEMVER_FILE
  SEMVER_FILE=$(mktemp)
  jq -r '
    (.data.items // [])[]
    | (.["display-name"] // "") as $n
    | $n
    | capture("^[^:]+:(?<v>[0-9]+\\.[0-9]+\\.[0-9]+)$")
    | .v
  ' <<<"$IMG_LIST" | sort -V -u > "$SEMVER_FILE"

  local SEMVER_VERSIONS_ARR=()
  while IFS= read -r line; do
    [[ -n "$line" ]] && SEMVER_VERSIONS_ARR+=("$line")
  done < "$SEMVER_FILE"
  rm -f "$SEMVER_FILE"

  local SEMVER_COUNT=${#SEMVER_VERSIONS_ARR[@]}
  if [[ "$SEMVER_COUNT" -eq 0 ]]; then
    echo ">> [cleanup] no SemVer-tagged images found; skipping"
    return 0
  fi
  echo ">> [cleanup] distinct SemVer count: ${SEMVER_COUNT}"

  local KEEP_VERSIONS_ARR=()
  local START=$(( SEMVER_COUNT > KEEP ? SEMVER_COUNT - KEEP : 0 ))
  local idx
  for (( idx=START; idx<SEMVER_COUNT; idx++ )); do
    KEEP_VERSIONS_ARR+=("${SEMVER_VERSIONS_ARR[$idx]}")
  done

  echo ">> [cleanup] keeping latest ${KEEP} SemVer:"
  for v in "${KEEP_VERSIONS_ARR[@]}"; do
    echo "     ${v}"
  done

  local KEEP_DIGESTS=()

  local v FULL_REF MANIFEST CHILDREN child SEMVER_DIGEST
  for v in "${KEEP_VERSIONS_ARR[@]}"; do
    SEMVER_DIGEST=$(jq -r --arg v ":$v" '
      (.data.items // [])[]
      | select((.["display-name"] // "") | endswith($v))
      | .digest
    ' <<<"$IMG_LIST" | head -1)
    [[ -n "$SEMVER_DIGEST" ]] && KEEP_DIGESTS+=("$SEMVER_DIGEST")

    FULL_REF="${REG_URL}/${REPO}:${v}"
    if MANIFEST=$(docker manifest inspect "$FULL_REF" 2>/dev/null); then
      CHILDREN=$(jq -r '(.manifests // [])[].digest // empty' <<<"$MANIFEST")
      for child in $CHILDREN; do
        KEEP_DIGESTS+=("$child")
      done
    else
      echo ">> [cleanup] warn: failed to inspect ${FULL_REF}; keeping only its top-level digest" >&2
    fi
  done

  echo ">> [cleanup] keep set has ${#KEEP_DIGESTS[@]} digests"

  local KEEP_RE
  KEEP_RE=$(printf '%s|' "${KEEP_DIGESTS[@]}")
  KEEP_RE="${KEEP_RE%|}"

  local TO_DELETE
  TO_DELETE=$(jq -r --arg keep "$KEEP_RE" '
    (.data.items // [])[]
    | select((.digest // "") | test("^(" + $keep + ")$") | not)
    | "\(.id)\t\(.digest)\t\(.["display-name"] // "<untagged>")"
  ' <<<"$IMG_LIST")

  local DEL_COUNT
  DEL_COUNT=$(printf '%s\n' "$TO_DELETE" | grep -c . || true)
  if [[ "$DEL_COUNT" -eq 0 ]]; then
    echo ">> [cleanup] nothing to delete"
    return 0
  fi

  if [[ "$DRY_RUN" -eq 1 ]]; then
    echo ">> [cleanup] DRY-RUN: would delete ${DEL_COUNT} images:"
    while IFS=$'\t' read -r id digest name; do
      [[ -z "$id" ]] && continue
      echo "     would delete  ${name}  (${digest:0:19}...)"
    done <<<"$TO_DELETE"
    return 0
  fi

  echo ">> [cleanup] deleting ${DEL_COUNT} images"

  local OK=0 FAIL=0
  while IFS=$'\t' read -r id digest name; do
    [[ -z "$id" ]] && continue
    if oci artifacts container image delete --image-id "$id" --force >/dev/null 2>&1; then
      OK=$((OK + 1))
      echo "     deleted ${name} (${digest:0:19}...)"
    else
      FAIL=$((FAIL + 1))
      echo "     FAILED  ${name} (${digest:0:19}...)" >&2
    fi
  done <<<"$TO_DELETE"

  echo ">> [cleanup] done: ${OK} deleted, ${FAIL} failed"
  return 0
}
