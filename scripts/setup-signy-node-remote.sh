#!/usr/bin/env bash

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
pulumi_dir="${repo_root}/infra/cloud"
stack="prod"
ssh_target=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --ssh)
      ssh_target="$2"
      shift 2
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

if [[ -z "$ssh_target" ]]; then
  echo "usage: $0 --ssh user@host" >&2
  exit 1
fi
for tool in pulumi ssh scp; do
  command -v "$tool" >/dev/null || { echo "missing required tool: $tool" >&2; exit 1; }
done

pulumi_output() {
  pulumi -C "$pulumi_dir" -s "$stack" stack output --show-secrets "$1"
}
pulumi_config() {
  pulumi -C "$pulumi_dir" -s "$stack" config get "$1"
}

signy_image_ref="$(pulumi_config fn0Cloud:signyImageRef)"
signy_r2_bucket="$(pulumi_output signyR2BucketName)"
signy_r2_prefix="$(pulumi_output signyR2StoragePrefix)"
signy_r2_endpoint="$(pulumi_output signyR2Endpoint)"
signy_r2_access_key_id="$(pulumi_output signyR2AccessKeyId)"
signy_r2_secret_access_key="$(pulumi_output signyR2SecretAccessKey)"
cloudflare_api_token="$(pulumi_output cloudflareOperatorApiToken)"
cloudflare_account_id="$(pulumi_config fn0Cloud:cloudflareAccountId)"
signy_tunnel_id="$(pulumi_output signyTunnelId)"
signy_hostname="$(pulumi_output signyHostnameOutput)"

payload="$(mktemp)"
trap '/bin/unlink "$payload"' EXIT
chmod 0600 "$payload"
{
  printf '#!/usr/bin/env bash\n'
  printf 'export SIGNY_IMAGE_REF=%q\n' "$signy_image_ref"
  printf 'export SIGNY_R2_BUCKET=%q\n' "$signy_r2_bucket"
  printf 'export SIGNY_R2_PREFIX=%q\n' "$signy_r2_prefix"
  printf 'export SIGNY_R2_ENDPOINT=%q\n' "$signy_r2_endpoint"
  printf 'export SIGNY_R2_ACCESS_KEY_ID=%q\n' "$signy_r2_access_key_id"
  printf 'export SIGNY_R2_SECRET_ACCESS_KEY=%q\n' "$signy_r2_secret_access_key"
  printf 'export CLOUDFLARE_API_TOKEN=%q\n' "$cloudflare_api_token"
  printf 'export CLOUDFLARE_ACCOUNT_ID=%q\n' "$cloudflare_account_id"
  printf 'export SIGNY_TUNNEL_ID=%q\n' "$signy_tunnel_id"
  printf 'export SIGNY_HOSTNAME=%q\n' "$signy_hostname"
  cat "${repo_root}/scripts/setup-signy-node.sh"
} > "$payload"

remote_file="fn0-signy-setup.sh"
if [[ -n "${SSHPASS:-}" ]] && command -v sshpass >/dev/null; then
  SSHPASS="$SSHPASS" sshpass -e scp -q "$payload" "${ssh_target}:${remote_file}"
  printf '%s\n' "$SSHPASS" | SSHPASS="$SSHPASS" sshpass -e ssh -T "$ssh_target" "chmod 0600 ${remote_file} && sudo -S -p '' bash ${remote_file}; setup_status=\$?; rm -f ${remote_file}; exit \$setup_status"
else
  scp -q "$payload" "${ssh_target}:${remote_file}"
  ssh -t "$ssh_target" "chmod 0600 ${remote_file} && sudo bash ${remote_file}; setup_status=\$?; rm -f ${remote_file}; exit \$setup_status"
fi
