import * as pulumi from "@pulumi/pulumi";
import * as cloudflare from "@pulumi/cloudflare";
import * as crypto from "crypto";

export interface StaticAssetStorageArgs {
  accountId: pulumi.Input<string>;
  zoneId: pulumi.Input<string>;
  publicBaseDomain: pulumi.Input<string>;
}

export class StaticAssetStorage extends pulumi.ComponentResource {
  public readonly accountId: pulumi.Output<string>;
  public readonly zoneId: pulumi.Output<string>;
  public readonly publicBaseDomain: pulumi.Output<string>;
  public readonly presignAccessKeyId: pulumi.Output<string>;
  public readonly presignSecretAccessKey: pulumi.Output<string>;
  public readonly cloudflareApiToken: pulumi.Output<string>;

  constructor(
    name: string,
    args: StaticAssetStorageArgs,
    opts: pulumi.ComponentResourceOptions
  ) {
    super("pkg:index:static-asset-storage", name, args, opts);

    const { accountId, zoneId, publicBaseDomain } = args;

    const r2ItemReadId = "6a018a9f2fc74eb6b293b0c548f38b39";
    const r2ItemWriteId = "2efd5506f9c8494dacb1fa10a3e7d5b6";
    const r2EditId = "bf7481a1826f439697cb59a20b22293e";
    const dnsWriteId = "4755a26eedb94da69e1066d98aa820be";

    const presignToken = new cloudflare.AccountToken(
      "presign-token",
      {
        accountId,
        name: pulumi.interpolate`fn0-static-asset-presign-${accountId}`,
        policies: [
          {
            effect: "allow",
            resources: pulumi.output(accountId).apply((acct) => ({
              [`com.cloudflare.edge.r2.bucket.${acct}_default_*`]: "*",
            })),
            permissionGroups: [{ id: r2ItemReadId }, { id: r2ItemWriteId }],
          },
        ],
      },
      { parent: this }
    );

    const adminToken = new cloudflare.AccountToken(
      "cloudflare-api-token",
      {
        accountId,
        name: pulumi.interpolate`fn0-static-asset-admin-${accountId}`,
        policies: [
          {
            effect: "allow",
            resources: pulumi.output(accountId).apply((acct) => ({
              [`com.cloudflare.api.account.${acct}`]: "*",
            })),
            permissionGroups: [{ id: r2EditId }],
          },
          {
            effect: "allow",
            resources: pulumi.output(zoneId).apply((zid) => ({
              [`com.cloudflare.api.account.zone.${zid}`]: "*",
            })),
            permissionGroups: [{ id: dnsWriteId }],
          },
        ],
      },
      { parent: this }
    );

    this.accountId = pulumi.output(accountId);
    this.zoneId = pulumi.output(zoneId);
    this.publicBaseDomain = pulumi.output(publicBaseDomain);
    this.presignAccessKeyId = presignToken.id;
    this.presignSecretAccessKey = pulumi.secret(
      presignToken.value.apply((v) =>
        crypto.createHash("sha256").update(v).digest("hex")
      )
    );
    this.cloudflareApiToken = pulumi.secret(adminToken.value);
  }
}
