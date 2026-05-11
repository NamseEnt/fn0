import * as pulumi from "@pulumi/pulumi";
import * as oci from "@pulumi/oci";

export interface Fn0WorkerPoolArgs {
  count: number;

  compartmentId: pulumi.Input<string>;
  availabilityDomain: pulumi.Input<string>;
  subnetId: pulumi.Input<string>;
  imageId: pulumi.Input<string>;
  shape: pulumi.Input<string>;
  ocpus: pulumi.Input<number>;
  memoryInGbs: pulumi.Input<number>;
  sshAuthorizedKeys?: pulumi.Input<string>;

  agent: AgentArgs;
  worker: WorkerArgs;
}

export interface AgentArgs {
  imageRef: pulumi.Input<string>;
  docDb: AgentDocDbArgs;
  dnsRegister: AgentDnsRegisterArgs;
}

export interface AgentDocDbArgs {
  url: pulumi.Input<string>;
  authToken: pulumi.Input<string>;
}

export interface AgentDnsRegisterArgs {
  apiToken: pulumi.Input<string>;
  zoneId: pulumi.Input<string>;
  hostname: pulumi.Input<string>;
}

export interface WorkerArgs {
  cwasmBucket: WorkerCwasmBucketArgs;
  tlsOrigin: WorkerTlsOriginArgs;
  envEncryptionKeyBase64: pulumi.Input<string>;
  forteDb: WorkerForteDbArgs;
  queue: WorkerQueueArgs;
  controlInvokeAllowedSubdomain: pulumi.Input<string>;
  vault: WorkerVaultArgs;
  otlp: WorkerOtlpArgs;
}

export interface WorkerCwasmBucketArgs {
  name: pulumi.Input<string>;
  endpoint: pulumi.Input<string>;
  region: pulumi.Input<string>;
  accessKeyId: pulumi.Input<string>;
  secretAccessKey: pulumi.Input<string>;
}

export interface WorkerTlsOriginArgs {
  certPem: pulumi.Input<string>;
  keyPem: pulumi.Input<string>;
}

export interface WorkerForteDbArgs {
  groupToken: pulumi.Input<string>;
  hostSuffix: pulumi.Input<string>;
}

export interface WorkerQueueArgs {
  ocid: pulumi.Input<string>;
  messagesEndpoint: pulumi.Input<string>;
  region: pulumi.Input<string>;
  ociUserId: pulumi.Input<string>;
  ociTenancyId: pulumi.Input<string>;
  ociFingerprint: pulumi.Input<string>;
  ociPrivateKeyBase64: pulumi.Input<string>;
}

export interface WorkerVaultArgs {
  cryptoEndpoint: pulumi.Input<string>;
  keyOcid: pulumi.Input<string>;
  region: pulumi.Input<string>;
  allowedSubdomain: pulumi.Input<string>;
  workerCredentials: pulumi.Input<WorkerVaultCredentials>;
}

export interface WorkerVaultCredentials {
  ociUserId: string;
  ociTenancyId: string;
  ociFingerprint: string;
  ociPrivateKeyBase64: string;
}

export interface WorkerOtlpArgs {
  endpoint: pulumi.Input<string>;
  basicAuth: pulumi.Input<string>;
}

const MANAGED_BY_TAG_VALUE = "fn0-control";

export class Fn0WorkerPool extends pulumi.ComponentResource {
  public readonly instanceIds: pulumi.Output<string[]>;
  public readonly publicIps: pulumi.Output<(string | undefined)[]>;

  constructor(
    name: string,
    args: Fn0WorkerPoolArgs,
    opts?: pulumi.ComponentResourceOptions
  ) {
    super("pkg:index:fn0-worker-pool", name, args, opts);

    const agentEnv = buildAgentEnv(args.agent);
    const workerEnv = buildWorkerEnv(args.worker);

    const cloudInit = pulumi
      .all([args.agent.imageRef, agentEnv, workerEnv])
      .apply(([agentImageRef, agentEnv, workerEnv]) =>
        renderCloudInit(agentImageRef, agentEnv, workerEnv)
      );
    const userData = cloudInit.apply((s) =>
      Buffer.from(s, "utf8").toString("base64")
    );

    const metadata = pulumi
      .all([userData, args.sshAuthorizedKeys ?? pulumi.output("")])
      .apply(([ud, ssh]) => {
        const m: { [k: string]: string } = { user_data: ud };
        if (ssh) m["ssh_authorized_keys"] = ssh;
        return m;
      });

    const instances: oci.core.Instance[] = [];
    for (let i = 0; i < args.count; i++) {
      instances.push(
        new oci.core.Instance(
          `instance-${i}`,
          {
            compartmentId: args.compartmentId,
            availabilityDomain: args.availabilityDomain,
            displayName: `${name}-${i}`,
            shape: args.shape,
            shapeConfig: {
              ocpus: args.ocpus,
              memoryInGbs: args.memoryInGbs,
            },
            sourceDetails: {
              sourceType: "image",
              sourceId: args.imageId,
            },
            createVnicDetails: {
              subnetId: args.subnetId,
              assignPublicIp: "true",
              assignIpv6ip: true,
            },
            metadata,
            freeformTags: {
              managed_by: MANAGED_BY_TAG_VALUE,
              fn0_role: "worker",
            },
          },
          { parent: this }
        )
      );
    }

    this.instanceIds = pulumi.all(instances.map((i) => i.id));
    this.publicIps = pulumi.all(instances.map((i) => i.publicIp));

    this.registerOutputs({
      instanceIds: this.instanceIds,
      publicIps: this.publicIps,
    });
  }
}

function buildAgentEnv(
  agent: AgentArgs
): pulumi.Output<{ [k: string]: string }> {
  const base: { [k: string]: pulumi.Input<string> } = {
    TURSO_URL: agent.docDb.url,
    TURSO_AUTH_TOKEN: agent.docDb.authToken,
    FN0_AGENT_DNS_API_TOKEN: agent.dnsRegister.apiToken,
    FN0_AGENT_DNS_ZONE_ID: agent.dnsRegister.zoneId,
    FN0_AGENT_DNS_HOSTNAME: agent.dnsRegister.hostname,
  };
  return resolveEnvMap(base);
}

function buildWorkerEnv(
  worker: WorkerArgs
): pulumi.Output<{ [k: string]: string }> {
  const otlpParsed = pulumi.output(worker.otlp.endpoint).apply((u) => {
    const url = new URL(u);
    return {
      targetHost: url.host,
      targetPathPrefix: url.pathname.replace(/\/$/, ""),
    };
  });

  const vaultCredsEnv = pulumi
    .output(worker.vault.workerCredentials)
    .apply((c): { [k: string]: string } => ({
      FN0_VAULT_OCI_USER_ID: c.ociUserId,
      FN0_VAULT_OCI_TENANCY_ID: c.ociTenancyId,
      FN0_VAULT_OCI_FINGERPRINT: c.ociFingerprint,
      FN0_VAULT_OCI_PRIVATE_KEY_BASE64: c.ociPrivateKeyBase64,
    }));

  const base: { [k: string]: pulumi.Input<string> } = {
    CWASM_BUCKET: worker.cwasmBucket.name,
    S3_ENDPOINT: worker.cwasmBucket.endpoint,
    S3_REGION: worker.cwasmBucket.region,
    AWS_ACCESS_KEY_ID: worker.cwasmBucket.accessKeyId,
    AWS_SECRET_ACCESS_KEY: worker.cwasmBucket.secretAccessKey,

    ORIGIN_CERT_PEM: worker.tlsOrigin.certPem,
    ORIGIN_KEY_PEM: worker.tlsOrigin.keyPem,

    FN0_ENV_KEY_BASE64: worker.envEncryptionKeyBase64,

    TURSO_GROUP_TOKEN: worker.forteDb.groupToken,
    TURSO_DB_HOST_SUFFIX: worker.forteDb.hostSuffix,

    FN0_QUEUE_OCID: worker.queue.ocid,
    FN0_QUEUE_MESSAGES_ENDPOINT: worker.queue.messagesEndpoint,
    FN0_QUEUE_REGION: worker.queue.region,
    FN0_QUEUE_OCI_USER_ID: worker.queue.ociUserId,
    FN0_QUEUE_OCI_TENANCY_ID: worker.queue.ociTenancyId,
    FN0_QUEUE_OCI_FINGERPRINT: worker.queue.ociFingerprint,
    FN0_QUEUE_OCI_PRIVATE_KEY_BASE64: worker.queue.ociPrivateKeyBase64,

    FN0_CONTROL_INVOKE_QUEUE_OCID: worker.queue.ocid,
    FN0_CONTROL_INVOKE_QUEUE_MESSAGES_ENDPOINT: worker.queue.messagesEndpoint,
    FN0_CONTROL_INVOKE_QUEUE_OCI_USER_ID: worker.queue.ociUserId,
    FN0_CONTROL_INVOKE_QUEUE_OCI_TENANCY_ID: worker.queue.ociTenancyId,
    FN0_CONTROL_INVOKE_QUEUE_OCI_FINGERPRINT: worker.queue.ociFingerprint,
    FN0_CONTROL_INVOKE_QUEUE_OCI_PRIVATE_KEY_BASE64:
      worker.queue.ociPrivateKeyBase64,
    FN0_CONTROL_INVOKE_QUEUE_ALLOWED_SUBDOMAIN:
      worker.controlInvokeAllowedSubdomain,

    FN0_VAULT_CRYPTO_ENDPOINT: worker.vault.cryptoEndpoint,
    FN0_VAULT_KEY_OCID: worker.vault.keyOcid,
    FN0_VAULT_REGION: worker.vault.region,
    FN0_VAULT_ALLOWED_SUBDOMAIN: worker.vault.allowedSubdomain,

    OTLP_ENDPOINT: worker.otlp.endpoint,
    OTLP_BASIC_AUTH: worker.otlp.basicAuth,
    FN0_OTLP_TARGET_HOST: otlpParsed.targetHost,
    FN0_OTLP_TARGET_PATH_PREFIX: otlpParsed.targetPathPrefix,
    FN0_OTLP_AUTH: worker.otlp.basicAuth,
  };

  return pulumi
    .all([resolveEnvMap(base), vaultCredsEnv])
    .apply(([b, v]) => ({ ...b, ...v }));
}

function resolveEnvMap(
  m: { [k: string]: pulumi.Input<string> }
): pulumi.Output<{ [k: string]: string }> {
  const keys = Object.keys(m);
  const values = keys.map((k) => pulumi.output(m[k]));
  return pulumi.all(values).apply((resolved) => {
    const out: { [k: string]: string } = {};
    keys.forEach((k, i) => {
      out[k] = resolved[i];
    });
    return out;
  });
}

function renderCloudInit(
  agentImageRef: string,
  agentEnv: { [k: string]: string },
  workerEnv: { [k: string]: string }
): string {
  const agentEnvFile = renderEnvFile({
    ...agentEnv,
    FN0_AGENT_WORKER_ENV_FILE: "/etc/fn0-worker-agent/worker-env",
  });
  const workerEnvFile = renderEnvFile(workerEnv);
  const systemdUnit = `[Unit]
Description=fn0 worker agent
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
EnvironmentFile=/etc/fn0-worker-agent/env
ExecStartPre=-/usr/bin/podman pull ${agentImageRef}
ExecStart=/usr/local/bin/fn0-worker-agent
Restart=on-failure
RestartSec=5
User=opc

[Install]
WantedBy=multi-user.target
`;
  return `#!/bin/bash
set -euxo pipefail

if ! command -v podman >/dev/null 2>&1; then
  dnf install -y podman
fi

HOST_ID=$(curl -fsSH "Authorization: Bearer Oracle" http://169.254.169.254/opc/v2/instance/id)
if [ -z "$HOST_ID" ]; then
  echo "failed to fetch FN0_AGENT_HOST_ID from OCI metadata" >&2
  exit 1
fi

mkdir -p /etc/fn0-worker-agent
cat > /etc/fn0-worker-agent/env <<'EOF_AGENT_ENV'
${agentEnvFile}EOF_AGENT_ENV
echo "FN0_AGENT_HOST_ID=$HOST_ID" >> /etc/fn0-worker-agent/env
chmod 600 /etc/fn0-worker-agent/env

cat > /etc/fn0-worker-agent/worker-env <<'EOF_WORKER_ENV'
${workerEnvFile}EOF_WORKER_ENV
chmod 600 /etc/fn0-worker-agent/worker-env

podman pull ${agentImageRef}
agent_cid=$(podman create ${agentImageRef})
podman cp "$agent_cid:/usr/local/bin/fn0-worker-agent" /usr/local/bin/fn0-worker-agent.new
podman rm "$agent_cid"
chmod +x /usr/local/bin/fn0-worker-agent.new
mv /usr/local/bin/fn0-worker-agent.new /usr/local/bin/fn0-worker-agent

cat > /etc/systemd/system/fn0-worker-agent.service <<'EOF_UNIT'
${systemdUnit}EOF_UNIT

systemctl daemon-reload
systemctl enable --now fn0-worker-agent.service
`;
}

function renderEnvFile(env: { [k: string]: string }): string {
  const lines: string[] = [];
  for (const [k, v] of Object.entries(env)) {
    if (v === undefined || v === null) continue;
    lines.push(`${k}=${escapeEnvValue(v)}`);
  }
  return lines.length === 0 ? "" : lines.join("\n") + "\n";
}

function escapeEnvValue(v: string): string {
  return v.replace(/\n/g, "\\n");
}
