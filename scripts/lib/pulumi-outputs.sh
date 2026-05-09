# shellcheck shell=bash
# Cached pulumi stack outputs and helpers. Source-only.

if [[ -n "${__FN0_PULUMI_OUTPUTS_LOADED:-}" ]]; then
  return 0
fi
__FN0_PULUMI_OUTPUTS_LOADED=1

need() {
  command -v "$1" >/dev/null 2>&1 || { echo "required command not found: $1" >&2; exit 1; }
}

PULUMI_OUTPUTS_JSON=""

load_pulumi_outputs() {
  if [[ -n "$PULUMI_OUTPUTS_JSON" ]]; then
    return 0
  fi
  local pulumi_dir="${PULUMI_DIR:-${REPO_ROOT}/infra/cloud}"
  echo ">> Reading Pulumi stack outputs from ${pulumi_dir}"
  PULUMI_OUTPUTS_JSON="$(cd "$pulumi_dir" && pulumi stack output --show-secrets --json)"
}

pulumi_pick() {
  jq -r ".${1} // empty" <<<"$PULUMI_OUTPUTS_JSON"
}

pulumi_pick_json() {
  jq -c ".${1} // empty" <<<"$PULUMI_OUTPUTS_JSON"
}

require_pulumi_output() {
  local name value
  for name in "$@"; do
    value="$(pulumi_pick "$name")"
    if [[ -z "$value" ]]; then
      echo "missing Pulumi output: ${name}" >&2
      exit 1
    fi
  done
}
