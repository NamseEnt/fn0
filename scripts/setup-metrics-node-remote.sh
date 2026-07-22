#!/usr/bin/env bash
# Run scripts/setup-metrics-node.sh on a remote machine, feeding it every
# secret it needs from the pulumi stack (#59). Run on a workstation that is
# logged into pulumi; the remote ssh user does not need to be root — sudo
# prompts for its password on the allocated tty.
#
# Flow:
#   1. Read the Cloudflare API token, account/zone id, domain, and the
#      metrics-backup R2 credentials from the fn0Cloud prod stack.
#   2. Generate a single self-contained file (secret exports + the full
#      setup-metrics-node.sh body) and scp it to the remote home directory
#      with mode 0600, so no secret ever appears in ssh argv or process
#      lists.
#   3. ssh -t: sudo runs the file, then the file is deleted whether setup
#      succeeded or not (re-running this wrapper regenerates it).
#
# Usage:
#   ./setup-metrics-node-remote.sh --ssh <user@host> \
#     [--hostname metrics.fn0.dev] [--retention 30d]
#
# --hostname defaults to metrics.<fn0Cloud:domain>.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PULUMI_DIR="${REPO_ROOT}/infra/cloud"
STACK="prod"

ssh_target=""
public_hostname=""
retention="30d"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --ssh) ssh_target="$2"; shift 2 ;;
    --hostname) public_hostname="$2"; shift 2 ;;
    --retention) retention="$2"; shift 2 ;;
    *) echo "unknown argument: $1" >&2; exit 1 ;;
  esac
done

if [[ -z "$ssh_target" ]]; then
  echo "usage: $0 --ssh <user@host> [--hostname <fqdn>] [--retention 30d]" >&2
  exit 1
fi
for tool in pulumi ssh scp; do
  command -v "$tool" >/dev/null || { echo "missing required tool: $tool" >&2; exit 1; }
done

pulumi_config() {
  pulumi -C "$PULUMI_DIR" -s "$STACK" config get "$1"
}
pulumi_output() {
  pulumi -C "$PULUMI_DIR" -s "$STACK" stack output --show-secrets "$1"
}

echo "reading pulumi config and stack outputs..."
cloudflare_api_token="$(pulumi_config cloudflare:apiToken)"
account_id="$(pulumi_config fn0Cloud:cloudflareAccountId)"
zone_id="$(pulumi_config fn0Cloud:cloudflareZoneId)"
domain="$(pulumi_config fn0Cloud:domain)"
r2_bucket="$(pulumi_output metricsBackupR2BucketName)"
r2_access_key_id="$(pulumi_output metricsBackupR2AccessKeyId)"
r2_secret_access_key="$(pulumi_output metricsBackupR2SecretAccessKey)"

if [[ -z "$public_hostname" ]]; then
  public_hostname="metrics.${domain}"
fi

payload="$(mktemp)"
trap 'rm -f "$payload"' EXIT
chmod 0600 "$payload"
{
  printf '#!/usr/bin/env bash\n'
  printf 'export CLOUDFLARE_API_TOKEN=%q\n' "$cloudflare_api_token"
  printf 'export FN0_METRICS_R2_ACCESS_KEY_ID=%q\n' "$r2_access_key_id"
  printf 'export FN0_METRICS_R2_SECRET_ACCESS_KEY=%q\n' "$r2_secret_access_key"
  cat "${REPO_ROOT}/scripts/setup-metrics-node.sh"
} > "$payload"

remote_file="fn0-metrics-setup.sh"
scp -q "$payload" "${ssh_target}:${remote_file}"

echo "running setup on ${ssh_target} (sudo may prompt for a password)..."
setup_status=0
ssh -t "$ssh_target" \
  "chmod 0600 ${remote_file} \
   && sudo bash ${remote_file} \
     --hostname $(printf '%q' "$public_hostname") \
     --account-id $(printf '%q' "$account_id") \
     --zone-id $(printf '%q' "$zone_id") \
     --r2-bucket $(printf '%q' "$r2_bucket") \
     --retention $(printf '%q' "$retention"); \
   status=\$?; rm -f ${remote_file}; exit \$status" \
  || setup_status=$?

if [[ "$setup_status" -ne 0 ]]; then
  echo "remote setup failed (exit ${setup_status}); re-run this wrapper after fixing the cause" >&2
  exit "$setup_status"
fi
