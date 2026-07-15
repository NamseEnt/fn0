import * as pulumi from "@pulumi/pulumi";
import * as cloudflare from "@pulumi/cloudflare";
import * as crypto from "crypto";
import { R2AllBucketsItemToken } from "./R2AllBucketsItemToken";

export interface StaticAssetStorageArgs {
  accountId: pulumi.Input<string>;
  zoneId: pulumi.Input<string>;
  publicBaseDomain: pulumi.Input<string>;
  bucketName: pulumi.Input<string>;
  cloudflareUserApiToken: pulumi.Input<string>;
}

export class StaticAssetStorage extends pulumi.ComponentResource {
  public readonly accountId: pulumi.Output<string>;
  public readonly zoneId: pulumi.Output<string>;
  public readonly publicBaseDomain: pulumi.Output<string>;
  public readonly bucketName: pulumi.Output<string>;
  public readonly presignAccessKeyId: pulumi.Output<string>;
  public readonly presignSecretAccessKey: pulumi.Output<string>;
  public readonly cloudflareApiToken: pulumi.Output<string>;

  constructor(
    name: string,
    args: StaticAssetStorageArgs,
    opts: pulumi.ComponentResourceOptions,
  ) {
    super("pkg:index:static-asset-storage", name, args, opts);

    const { accountId, zoneId, publicBaseDomain, bucketName } = args;

    // Every project's static assets share this one bucket under a
    // `{project_id}/` key prefix, served by a single custom domain. This keeps
    // the fn0.dev zone at exactly one DNS record for static assets regardless
    // of project count (the per-project custom-domain approach hit the zone's
    // DNS-record limit).
    const bucket = new cloudflare.R2Bucket(
      "bucket",
      {
        accountId,
        name: bucketName,
        location: "apac",
      },
      { parent: this },
    );

    new cloudflare.R2BucketCors(
      "cors",
      {
        accountId,
        bucketName: bucket.name,
        rules: [
          {
            allowed: {
              methods: ["GET", "HEAD"],
              origins: ["*"],
              headers: ["*"],
            },
            maxAgeSeconds: 86400,
          },
        ],
      },
      { parent: this, dependsOn: [bucket] },
    );

    new cloudflare.R2CustomDomain(
      "custom-domain",
      {
        accountId,
        bucketName: bucket.name,
        domain: publicBaseDomain,
        zoneId,
        enabled: true,
        minTls: "1.2",
      },
      { parent: this, dependsOn: [bucket] },
    );

    const r2ItemReadId = "6a018a9f2fc74eb6b293b0c548f38b39";
    const r2ItemWriteId = "2efd5506f9c8494dacb1fa10a3e7d5b6";
    const r2EditId = "bf7481a1826f439697cb59a20b22293e";
    const dnsWriteId = "4755a26eedb94da69e1066d98aa820be";

    // The presign token must grant Workers R2 Storage Bucket Item Read/Write
    // against ALL buckets in the account, because control creates per-project
    // R2 buckets on demand inside its new_project action and reuses this one
    // token to issue SigV4 PUT presigned URLs for any of them.
    const presignToken = new R2AllBucketsItemToken(
      "presign-token",
      {
        accountId,
        name: pulumi.interpolate`fn0-static-asset-presign-${accountId}`,
        permissionGroupIds: [r2ItemReadId, r2ItemWriteId],
        userApiToken: args.cloudflareUserApiToken,
      },
      { parent: this },
    );

    const adminToken = new cloudflare.AccountToken(
      "cloudflare-api-token",
      {
        accountId,
        name: pulumi.interpolate`fn0-static-asset-admin-${accountId}`,
        policies: [
          {
            effect: "allow",
            resources: pulumi.output(accountId).apply((acct) =>
              JSON.stringify({
                [`com.cloudflare.api.account.${acct}`]: "*",
              }),
            ),
            permissionGroups: [{ id: r2EditId }],
          },
          {
            effect: "allow",
            resources: pulumi.output(zoneId).apply((zid) =>
              JSON.stringify({
                [`com.cloudflare.api.account.zone.${zid}`]: "*",
              }),
            ),
            permissionGroups: [{ id: dnsWriteId }],
          },
        ],
      },
      { parent: this },
    );

    this.accountId = pulumi.output(accountId);
    this.zoneId = pulumi.output(zoneId);
    this.publicBaseDomain = pulumi.output(publicBaseDomain);
    this.bucketName = bucket.name;
    this.presignAccessKeyId = presignToken.id;
    this.presignSecretAccessKey = pulumi.secret(
      presignToken.value.apply((v) =>
        crypto.createHash("sha256").update(v).digest("hex"),
      ),
    );
    this.cloudflareApiToken = pulumi.secret(adminToken.value);
  }
}
