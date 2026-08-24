import * as pulumi from "@pulumi/pulumi";
import * as tls from "@pulumi/tls";
import * as cloudflare from "@pulumi/cloudflare";

export interface CloudflareDnsArgs {
  tokenMintingApiToken: pulumi.Input<string>;
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
    opts: pulumi.ComponentResourceOptions,
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
      { parent: this },
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
      { parent: this },
    );

    // Changing hostnames re-issues (replaces) this cert. Under Full (strict),
    // revoking the previous cert while a worker still presents it makes
    // Cloudflare reject the origin (526). retainOnDelete stops Pulumi from
    // revoking on replacement, so subdomains keep serving until workers roll
    // onto the new cert.
    const originCaCert = new cloudflare.OriginCaCertificate(
      "origin-ca-cert",
      {
        csr: csr.certRequestPem,
        requestType: "origin-ecc",
        hostnames: [pulumi.interpolate`*.${domain}`, domain],
        requestedValidity: 5475,
      },
      { parent: this, retainOnDelete: true },
    );

    this.certificate = originCaCert.certificate;

    // Cloudflare refuses to mint a token that can mint tokens ("sub-token is not
    // allowed to have permissions to manage other tokens"), so the operator
    // token the rest of this stack runs on cannot create the one below. The
    // bootstrap credential is the only thing that can, which is why it stays in
    // play at runtime rather than being a first-run-only input.
    const tokenMintingProvider = new cloudflare.Provider(
      "token-minting",
      { apiToken: args.tokenMintingApiToken },
      { parent: this },
    );

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
      { parent: this, provider: tokenMintingProvider },
    );

    this.saasApiToken = cloudflareSaasApiToken.value;

    // SaaS fallback origin points at a dedicated hostname with its own
    // explicit A record (see worker-lb-a). Using a human-touched name (like
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
      { parent: this },
    );
  }
}
