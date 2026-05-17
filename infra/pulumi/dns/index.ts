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
  saasApiToken: pulumi.Output<string>;
  saasFallbackLbHostname: pulumi.Output<string>;

  constructor(
    name: string,
    args: CloudflareDnsArgs,
    opts: pulumi.ComponentResourceOptions
  ) {
    super("pkg:index:cloudflare-dns", name, args, opts);

    const { accountId, domain, zoneId, suffix } = args;

    const PERMISSION_IDS = {
      SSL_AND_CERTIFICATES_WRITE: "c03055bc037c4ea9afb9a9f104b7b721",
      ZONE_READ: "c8fed203ed3043cba015a93ad1616f1f",
    };

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

    const cloudflareSaasApiToken = new cloudflare.AccountToken(
      "cloudflare-saas-api-token",
      {
        accountId,
        name: pulumi.interpolate`fn0-saas-${suffix}`,
        policies: [
          {
            effect: "allow",
            resources: JSON.stringify({
              [`com.cloudflare.api.account.zone.${zoneId}`]: "*",
            }),
            permissionGroups: [
              { id: PERMISSION_IDS.SSL_AND_CERTIFICATES_WRITE },
              { id: PERMISSION_IDS.ZONE_READ },
            ],
          },
        ],
      },
      { parent: this }
    );

    this.saasApiToken = cloudflareSaasApiToken.value;

    // SaaS fallback origin points at a hostname that the wildcard A record
    // (*.fn0.dev → NLB IP) already covers. Using a human-touched name (like
    // fallback.fn0.dev) would lock that name's A records under CF's
    // SaaS-fallback record protection (delete-blocked even via API). A
    // dedicated name keeps the lock scoped to a record we never touch by hand.
    const workerLbHostname = pulumi.interpolate`worker-lb.${domain}`;
    this.saasFallbackLbHostname = workerLbHostname;

    new cloudflare.CustomHostnameFallbackOrigin(
      "saas-fallback-origin",
      {
        zoneId,
        origin: workerLbHostname,
      },
      { parent: this }
    );
  }
}
