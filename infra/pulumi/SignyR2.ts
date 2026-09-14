import * as pulumi from "@pulumi/pulumi";
import * as cloudflare from "@pulumi/cloudflare";
import * as crypto from "crypto";

export interface SignyR2Args {
  tokenMintingApiToken: pulumi.Input<string>;
  accountId: pulumi.Input<string>;
  bucketName: pulumi.Input<string>;
  storagePrefix: pulumi.Input<string>;
  location?: pulumi.Input<string>;
}

export class SignyR2 extends pulumi.ComponentResource {
  public readonly accountId: pulumi.Output<string>;
  public readonly bucketName: pulumi.Output<string>;
  public readonly endpoint: pulumi.Output<string>;
  public readonly storagePrefix: pulumi.Output<string>;
  public readonly accessKeyId: pulumi.Output<string>;
  public readonly secretAccessKey: pulumi.Output<string>;

  constructor(
    name: string,
    args: SignyR2Args,
    opts: pulumi.ComponentResourceOptions,
  ) {
    super("pkg:index:signy-r2", name, args, opts);

    const bucket = new cloudflare.R2Bucket(
      "bucket",
      {
        accountId: args.accountId,
        name: args.bucketName,
        location: args.location ?? "apac",
      },
      { parent: this },
    );

    const lock = new cloudflare.R2BucketLock(
      "catalog-lock",
      {
        accountId: args.accountId,
        bucketName: bucket.name,
        rules: [
          {
            id: "signy-catalog-seven-days",
            enabled: true,
            prefix: pulumi.interpolate`${args.storagePrefix}/catalog/`,
            condition: {
              type: "Age",
              maxAgeSeconds: 604800,
            },
          },
        ],
      },
      { parent: this, dependsOn: [bucket] },
    );

    const permissionGroups =
      cloudflare.getAccountApiTokenPermissionGroupsListOutput({
        accountId: args.accountId,
        maxItems: 1000,
      });

    const r2PermissionIds = permissionGroups.apply((list) => {
      const groups = list.results ?? [];
      const read = groups.find(
        (group) => group.name === "Workers R2 Storage Bucket Item Read",
      );
      const write = groups.find(
        (group) => group.name === "Workers R2 Storage Bucket Item Write",
      );
      if (!read || !write) {
        throw new Error("Cloudflare R2 bucket permission groups are unavailable");
      }
      return [{ id: read.id }, { id: write.id }];
    });

    const tokenMintingProvider = new cloudflare.Provider(
      "token-minting",
      { apiToken: args.tokenMintingApiToken },
      { parent: this },
    );

    const token = new cloudflare.AccountToken(
      "r2-token",
      {
        accountId: args.accountId,
        name: pulumi.interpolate`fn0-signy-r2-${bucket.name}`,
        policies: [
          {
            effect: "allow",
            resources: pulumi
              .all([args.accountId, bucket.name])
              .apply(([accountId, bucketName]) =>
                JSON.stringify({
                  [`com.cloudflare.edge.r2.bucket.${accountId}_default_${bucketName}`]: "*",
                }),
              ),
            permissionGroups: r2PermissionIds,
          },
        ],
      },
      {
        parent: this,
        provider: tokenMintingProvider,
        dependsOn: [bucket, lock],
      },
    );

    this.accountId = pulumi.output(args.accountId);
    this.bucketName = bucket.name;
    this.endpoint = pulumi.interpolate`https://${args.accountId}.r2.cloudflarestorage.com`;
    this.storagePrefix = pulumi.output(args.storagePrefix);
    this.accessKeyId = token.id;
    this.secretAccessKey = pulumi.secret(
      token.value.apply((value) =>
        crypto.createHash("sha256").update(value).digest("hex"),
      ),
    );

    this.registerOutputs({
      accountId: this.accountId,
      bucketName: this.bucketName,
      endpoint: this.endpoint,
      storagePrefix: this.storagePrefix,
      accessKeyId: this.accessKeyId,
      secretAccessKey: this.secretAccessKey,
    });
  }
}
