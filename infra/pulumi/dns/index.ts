import * as pulumi from "@pulumi/pulumi";
import * as tls from "@pulumi/tls";
import * as cloudflare from "@pulumi/cloudflare";

export interface CloudflareDnsArgs {
  suffix: pulumi.Input<string>;
  accountId: pulumi.Input<string>;
  zoneId: pulumi.Input<string>;
  domain: pulumi.Input<string>;
}

export class CloudflareDns extends pulumi.ComponentResource {
  privateKeyPem: pulumi.Output<string>;
  certificate: pulumi.Output<string>;

  constructor(
    name: string,
    args: CloudflareDnsArgs,
    opts: pulumi.ComponentResourceOptions
  ) {
    super("pkg:index:cloudflare-dns", name, args, opts);

    const { accountId, domain, zoneId, suffix } = args;

    const apiTokenPermissionGroups = pulumi
      .all([accountId])
      .apply(([accountId]) =>
        cloudflare
          .getAccountApiTokenPermissionGroupsList({
            accountId,
            name: "DNS Write",
          })
          .then((x) => x.results.map((x) => ({ id: x.id })))
      );

    const cloudflareApiToken = new cloudflare.AccountToken(
      "cloudflare-api-token",
      {
        accountId,
        name: pulumi.interpolate`fn0-${suffix}`,
        policies: [
          {
            effect: "allow",
            resources: {
              [`com.cloudflare.api.account.zone.${zoneId}`]: "*",
            },
            permissionGroups: apiTokenPermissionGroups,
          },
        ],
      },
      { parent: this }
    );

    const privateKey = new tls.PrivateKey(
      "private-key",
      {
        algorithm: "ECDSA",
        ecdsaCurve: "P384",
      },
      { parent: this }
    );

    this.privateKeyPem = privateKey.privateKeyPem;

    const csr = new tls.CertRequest(
      "csr",
      {
        privateKeyPem: privateKey.privateKeyPem,
        subject: {
          commonName: domain,
          organization: "fn0",
        },
      },
      { parent: this }
    );

    const originCaCert = new cloudflare.OriginCaCertificate(
      "origin-ca-cert",
      {
        csr: csr.certRequestPem,
        requestType: "origin-ecc",
        hostnames: [pulumi.interpolate`*.${domain}`],
        requestedValidity: 5475,
      },
      { parent: this }
    );

    this.certificate = originCaCert.certificate;
  }
}
