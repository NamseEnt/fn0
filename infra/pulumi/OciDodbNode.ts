import * as pulumi from "@pulumi/pulumi";
import * as oci from "@pulumi/oci";
import * as tls from "@pulumi/tls";
import { gzipSync } from "node:zlib";

export interface OciDodbNodeArgs {
  compartmentId: pulumi.Input<string>;
  subnetId: pulumi.Input<string>;
  osImageId: pulumi.Input<string>;
  sshPublicKey: pulumi.Input<string>;
}

export class OciDodbNode extends pulumi.ComponentResource {
  public readonly instanceId: pulumi.Output<string>;
  public readonly privateIp: pulumi.Output<string>;
  public readonly serverName: pulumi.Output<string>;
  public readonly rootCertPem: pulumi.Output<string>;

  constructor(
    name: string,
    args: OciDodbNodeArgs,
    opts?: pulumi.ComponentResourceOptions,
  ) {
    super("pkg:index:oci-dodb-node", name, args, opts);

    const privateKey = new tls.PrivateKey(
      "dodb-quic-key",
      { algorithm: "ECDSA", ecdsaCurve: "P256" },
      { parent: this },
    );
    const certificate = new tls.SelfSignedCert(
      "dodb-quic-certificate",
      {
        privateKeyPem: privateKey.privateKeyPem,
        subject: { commonName: "dodb.internal" },
        validityPeriodHours: 24 * 365 * 10,
        allowedUses: ["server_auth", "digital_signature", "key_encipherment"],
        dnsNames: ["dodb.internal"],
      },
      { parent: this },
    );

    const availabilityDomain = pulumi
      .output(args.compartmentId)
      .apply((compartmentId) =>
        oci.identity
          .getAvailabilityDomains({ compartmentId })
          .then((result) => {
            const availabilityDomainName = result.availabilityDomains[0]?.name;
            if (!availabilityDomainName) {
              throw new Error("can not find availability domain");
            }
            return availabilityDomainName;
          }),
      );

    const cloudInit = pulumi
      .all([certificate.certPem, privateKey.privateKeyPem])
      .apply(([certificatePem, privateKeyPem]) =>
        renderDodbCloudInit(certificatePem, privateKeyPem),
      )
      .apply((value) => gzipSync(Buffer.from(value, "utf8")).toString("base64"));

    const metadata = pulumi
      .all([cloudInit, args.sshPublicKey])
      .apply(([userData, sshPublicKey]) => ({
        user_data: userData,
        ssh_authorized_keys: sshPublicKey,
      }));

    const instance = new oci.core.Instance(
      "dodb-instance",
      {
        availabilityDomain,
        compartmentId: args.compartmentId,
        displayName: "fn0-dodb",
        shape: "VM.Standard.A1.Flex",
        shapeConfig: { ocpus: 1, memoryInGbs: 6 },
        sourceDetails: {
          sourceType: "image",
          sourceId: args.osImageId,
          bootVolumeSizeInGbs: "150",
        },
        createVnicDetails: {
          subnetId: args.subnetId,
          assignPublicIp: "false",
        },
        agentConfig: {
          pluginsConfigs: [{ name: "Bastion", desiredState: "ENABLED" }],
        },
        metadata,
        preserveBootVolume: true,
        freeformTags: {
          managed_by: "fn0-control",
          fn0_role: "dodb",
        },
      },
      { parent: this, protect: true },
    );

    this.instanceId = instance.id;
    this.privateIp = instance.privateIp;
    this.serverName = pulumi.output("dodb.internal");
    this.rootCertPem = certificate.certPem;

    this.registerOutputs({
      instanceId: this.instanceId,
      privateIp: this.privateIp,
      serverName: this.serverName,
      rootCertPem: this.rootCertPem,
    });
  }
}

function renderDodbCloudInit(certificatePem: string, privateKeyPem: string): string {
  return `#!/bin/bash
set -euo pipefail

if [ ! -x /usr/libexec/oci-growfs ]; then
  echo "required Oracle Linux root filesystem tool is missing: /usr/libexec/oci-growfs" >&2
  exit 1
fi
/usr/libexec/oci-growfs -y

groupadd --system dodb
useradd --system --gid dodb --home-dir /var/lib/dodb --shell /sbin/nologin dodb
mkdir -p /var/lib/dodb /etc/dodb
chown dodb:dodb /var/lib/dodb
chown root:dodb /etc/dodb
chmod 0750 /etc/dodb

cat > /etc/dodb/server.crt <<'EOF_DODB_CERT'
${certificatePem}EOF_DODB_CERT
chown root:dodb /etc/dodb/server.crt
chmod 0644 /etc/dodb/server.crt

cat > /etc/dodb/server.key <<'EOF_DODB_KEY'
${privateKeyPem}EOF_DODB_KEY
chown root:dodb /etc/dodb/server.key
chmod 0640 /etc/dodb/server.key

cat > /etc/systemd/system/dodb.service <<'EOF_DODB_UNIT'
[Unit]
Description=dodb database server
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=dodb
Group=dodb
ExecStart=/usr/local/bin/dodb-server --listen 0.0.0.0:18445 --data-dir /var/lib/dodb --tls-cert /etc/dodb/server.crt --tls-key /etc/dodb/server.key
Restart=on-failure
RestartSec=5
TimeoutStopSec=60

[Install]
WantedBy=multi-user.target
EOF_DODB_UNIT

systemctl daemon-reload
`;
}
