#!/usr/bin/env bash
# Brings the operations canary (ops/canary) to its known state. Idempotent:
# re-run it from anywhere it failed, or whenever the canary source changes.
#
# Flow:
#   1. `forte cloud init` for the canary project at the Pulumi-owned hostname
#      (`opsCanaryHostnameOutput`). The first run registers the project and
#      writes ops/canary/Forte.toml; later runs only confirm it.
#   2. `forte deploy` of ops/canary.
#   3. `forte admin run seed_known_values`, which writes the known document and
#      object and reads both back. It goes through control, not the hostname,
#      so Cloudflare Access does not stand in its way.
#   4. Verify through the public ingress, with the canary's Access service
#      token, that every probe answers `{"ok":true}`, and that a request
#      without the token is refused at the edge.
#
# Steps 1-3 failing is a bootstrap failure (exit 1). Step 4 failing after a
# successful bootstrap is a probe failure (exit 2): the canary is set up and
# the path it measures is what is broken.
#
# Prerequisites: `pulumi up` has created the OpsCanaryAccess component, and the
# operator has run `forte login` and `forte cloud login --zone <zone>`.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export REPO_ROOT

# shellcheck source=scripts/lib/pulumi-outputs.sh
source "${REPO_ROOT}/scripts/lib/pulumi-outputs.sh"

need cargo
need curl
need jq
need npm
need pulumi

CANARY_DIR="${REPO_ROOT}/ops/canary"
PROBES=(runtime dodb storage)
PROBE_ATTEMPTS=30
PROBE_RETRY_SECONDS=5

bootstrap_failed() {
  echo "BOOTSTRAP FAILED: $1" >&2
  exit 1
}

probe_failed() {
  echo "PROBE FAILED: $1" >&2
  exit 2
}

# The local build rather than an installed binary: its deploy schema matches
# the control this repository deploys.
forte_cli() {
  (cd "$REPO_ROOT" && cargo run --release -p forte-cli --quiet -- "$@")
}

load_pulumi_outputs
require_pulumi_output opsCanaryHostnameOutput opsCanaryAccessClientId opsCanaryAccessClientSecret
canary_hostname="$(pulumi_pick opsCanaryHostnameOutput)"
canary_project_name="${canary_hostname%%.*}"
canary_zone="${canary_hostname#*.}"

access_headers_file="$(mktemp)"
trap 'rm -f "$access_headers_file"' EXIT
chmod 600 "$access_headers_file"
printf 'CF-Access-Client-Id: %s\nCF-Access-Client-Secret: %s\n' \
  "$(pulumi_pick opsCanaryAccessClientId)" \
  "$(pulumi_pick opsCanaryAccessClientSecret)" >"$access_headers_file"

echo ">> [1/4] forte cloud init (${canary_hostname})"
forte_cli cloud init \
  --project "$CANARY_DIR" \
  --project-name "$canary_project_name" \
  --zone "$canary_zone" ||
  bootstrap_failed "forte cloud init"

echo ">> [2/4] forte deploy (ops/canary)"
(cd "${CANARY_DIR}/fe" && npm ci --silent) || bootstrap_failed "npm ci"
forte_cli deploy --project "$CANARY_DIR" || bootstrap_failed "forte deploy"

echo ">> [3/4] seed known values"
seed_output="$(forte_cli admin run seed_known_values --project "$CANARY_DIR" --input '{}')" ||
  bootstrap_failed "seed_known_values"
echo "$seed_output"
jq -e '.doc_db == "matches" and .private_object == "matches"' <<<"$seed_output" >/dev/null ||
  bootstrap_failed "seed_known_values answered ${seed_output}"

echo ">> [4/4] verify probes through https://${canary_hostname}"
unguarded_status="$(curl -sS -o /dev/null -w '%{http_code}' "https://${canary_hostname}/api/runtime")"
if [[ "$unguarded_status" != "401" && "$unguarded_status" != "403" ]]; then
  probe_failed "a request without the Access service token answered ${unguarded_status}, not 401/403"
fi
echo "   without the service token: ${unguarded_status}"

for probe in "${PROBES[@]}"; do
  answer=""
  for attempt in $(seq 1 "$PROBE_ATTEMPTS"); do
    answer="$(curl -sS -H @"$access_headers_file" -w ' %{http_code}' \
      "https://${canary_hostname}/api/${probe}" || true)"
    if [[ "$answer" == '{"ok":true} 200' ]]; then
      break
    fi
    # A fresh deploy reaches the worker on its next manifest poll.
    sleep "$PROBE_RETRY_SECONDS"
  done
  if [[ "$answer" != '{"ok":true} 200' ]]; then
    probe_failed "/api/${probe} answered: ${answer} after ${attempt} attempts"
  fi
  echo "   /api/${probe}: ok"
done

echo ">> canary ready at https://${canary_hostname}"
