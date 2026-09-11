#!/usr/bin/env bash
# Run scripts/setup-telemetry-node.sh on a remote machine, feeding it every
# secret it needs from the pulumi stack (#59, #62). Run on a workstation that is
# logged into pulumi; the remote ssh user does not need to be root — sudo
# prompts for its password on the allocated tty.
#
# Flow:
#   1. Read every input from the fn0Cloud prod stack: both hostnames, the
#      metrics basic-auth credential, the Cloudflare operator token (the
#      bootstrap credential in config only mints tokens), account/zone id,
#      both R2 credentials and the Access service token. The stack owns all of
#      them, so nothing is copied by hand in either direction.
#   2. Generate a single self-contained file (secret exports + the full
#      setup-telemetry-node.sh body) and scp it to the remote home directory
#      with mode 0600, so no secret ever appears in ssh argv or process lists.
#   3. ssh -t: sudo runs the file, then the file is deleted whether setup
#      succeeded or not (re-running this wrapper regenerates it).
#
# Usage:
#   ./setup-telemetry-node-remote.sh --ssh <user@host> [--retention 30d]

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PULUMI_DIR="${REPO_ROOT}/infra/cloud"
STACK="prod"

ssh_target=""
retention="30d"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --ssh) ssh_target="$2"; shift 2 ;;
    --retention) retention="$2"; shift 2 ;;
    *) echo "unknown argument: $1" >&2; exit 1 ;;
  esac
done

if [[ -z "$ssh_target" ]]; then
  echo "usage: $0 --ssh <user@host> [--retention 30d]" >&2
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
cloudflare_api_token="$(pulumi_output cloudflareOperatorApiToken)"
account_id="$(pulumi_config fn0Cloud:cloudflareAccountId)"
zone_id="$(pulumi_config fn0Cloud:cloudflareZoneId)"
metrics_hostname="$(pulumi_config fn0Cloud:metricsHostname)"
basic_auth_username="$(pulumi_output metricsBasicAuthUsername)"
basic_auth_password="$(pulumi_output metricsBasicAuthPassword)"
metrics_backup_bucket="$(pulumi_output metricsBackupR2BucketName)"
metrics_r2_access_key_id="$(pulumi_output metricsBackupR2AccessKeyId)"
metrics_r2_secret_access_key="$(pulumi_output metricsBackupR2SecretAccessKey)"

payload="$(mktemp)"
trap 'rm -f "$payload"' EXIT
chmod 0600 "$payload"
{
  printf '#!/usr/bin/env bash\n'
  printf 'export CLOUDFLARE_API_TOKEN=%q\n' "$cloudflare_api_token"
  printf 'export FN0_METRICS_PASSWORD=%q\n' "$basic_auth_password"
  printf 'export FN0_METRICS_R2_ACCESS_KEY_ID=%q\n' "$metrics_r2_access_key_id"
  printf 'export FN0_METRICS_R2_SECRET_ACCESS_KEY=%q\n' "$metrics_r2_secret_access_key"
  cat "${REPO_ROOT}/scripts/setup-telemetry-node.sh"
} > "$payload"

remote_file="fn0-telemetry-setup.sh"
scp -q "$payload" "${ssh_target}:${remote_file}"

echo "running setup on ${ssh_target} (sudo may prompt for a password)..."
setup_status=0
ssh -t "$ssh_target" \
  "chmod 0600 ${remote_file} \
   && sudo bash ${remote_file} \
     --metrics-hostname $(printf '%q' "$metrics_hostname") \
     --username $(printf '%q' "$basic_auth_username") \
     --account-id $(printf '%q' "$account_id") \
     --zone-id $(printf '%q' "$zone_id") \
     --metrics-backup-bucket $(printf '%q' "$metrics_backup_bucket") \
     --retention $(printf '%q' "$retention"); \
   status=\$?; rm -f ${remote_file}; exit \$status" \
  || setup_status=$?

if [[ "$setup_status" -ne 0 ]]; then
  echo "remote setup failed (exit ${setup_status}); re-run this wrapper after fixing the cause" >&2
  exit "$setup_status"
fi
