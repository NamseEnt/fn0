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

    const permissionGroups = cloudflare.getAccountApiTokenPermissionGroupsListOutput(
      {
        accountId,
        maxItems: 1000,
      },
      { parent: this }
    );

    const groupId = (preferred: string, fallbacks: string[] = []) =>
      permissionGroups.apply((list) => {
        const groups = list.results ?? [];
        for (const candidate of [preferred, ...fallbacks]) {
          const found = groups.find((g) => g.name === candidate);
          if (found) return found.id!;
        }
        const names = groups.map((g) => g.name).join(", ");
        throw new Error(
          `Could not find permission group; tried ${[preferred, ...fallbacks].join(", ")}; available: ${names}`
        );
      });

    const r2ItemReadId = groupId("Workers R2 Storage Bucket Item Read");
    const r2ItemWriteId = groupId("Workers R2 Storage Bucket Item Write");
    const r2EditId = groupId("Workers R2 Storage Edit");
    const dnsWriteId = groupId("DNS Write");

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
