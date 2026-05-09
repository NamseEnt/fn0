import * as pulumi from "@pulumi/pulumi";
import * as oci from "@pulumi/oci";

export interface Fn0WorkerInstanceArgs {
  suffix: pulumi.Input<string>;
  displayName: pulumi.Input<string>;
  compartmentId: pulumi.Input<string>;
  availabilityDomain: pulumi.Input<string>;
  subnetId: pulumi.Input<string>;
  imageId: pulumi.Input<string>;
  shape: pulumi.Input<string>;
  ocpus: pulumi.Input<number>;
  memoryInGbs: pulumi.Input<number>;
  sshAuthorizedKeys?: pulumi.Input<string>;
  agentImageRef: pulumi.Input<string>;
  agentEnv: pulumi.Input<{ [k: string]: pulumi.Input<string> }>;
  workerEnv: pulumi.Input<{ [k: string]: pulumi.Input<string> }>;
}

const MANAGED_BY_TAG_VALUE = "fn0-control";

export class Fn0WorkerInstance extends pulumi.ComponentResource {
  public readonly instanceId: pulumi.Output<string>;
  public readonly publicIp: pulumi.Output<string | undefined>;

  constructor(
    name: string,
    args: Fn0WorkerInstanceArgs,
    opts?: pulumi.ComponentResourceOptions
  ) {
    super("pkg:index:fn0-worker-instance", name, args, opts);

    const cloudInit = pulumi
      .all([args.agentImageRef, args.agentEnv, args.workerEnv])
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

    const instance = new oci.core.Instance(
      "instance",
      {
        compartmentId: args.compartmentId,
        availabilityDomain: args.availabilityDomain,
        displayName: args.displayName,
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
    );

    this.instanceId = instance.id;
    this.publicIp = instance.publicIp;

    this.registerOutputs({
      instanceId: this.instanceId,
      publicIp: this.publicIp,
    });
  }
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

mkdir -p /etc/fn0-worker-agent
cat > /etc/fn0-worker-agent/env <<'EOF_AGENT_ENV'
${agentEnvFile}EOF_AGENT_ENV
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
