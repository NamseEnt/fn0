import * as pulumi from "@pulumi/pulumi";
import * as cloudflare from "@pulumi/cloudflare";
import * as crypto from "crypto";

export interface StaticPageStorageArgs {
  accountId: pulumi.Input<string>;
  bucketName: pulumi.Input<string>;
}

export class StaticPageStorage extends pulumi.ComponentResource {
  public readonly accountId: pulumi.Output<string>;
  public readonly bucketName: pulumi.Output<string>;
  public readonly endpoint: pulumi.Output<string>;
  public readonly accessKeyId: pulumi.Output<string>;
  public readonly secretAccessKey: pulumi.Output<string>;

  constructor(
    name: string,
    args: StaticPageStorageArgs,
    opts: pulumi.ComponentResourceOptions,
  ) {
    super("pkg:index:static-page-storage", name, args, opts);

    const bucket = new cloudflare.R2Bucket(
      "bucket",
      {
        accountId: args.accountId,
        name: args.bucketName,
        location: "apac",
      },
      { parent: this },
    );

    const permissionGroups = cloudflare.getAccountApiTokenPermissionGroupsListOutput(
      {
        accountId: args.accountId,
        maxItems: 1000,
      },
      { parent: this },
    );

    const r2PermissionIds = permissionGroups.apply((list) => {
      const groups = list.results ?? [];
      const read = groups.find(
        (group) => group.name === "Workers R2 Storage Bucket Item Read",
      );
      const write = groups.find(
        (group) => group.name === "Workers R2 Storage Bucket Item Write",
      );
      if (!read || !write) {
        throw new Error(
          `Could not find R2 permission groups; found: ${groups
            .map((group) => group.name)
            .join(", ")}`,
        );
      }
      return [{ id: read.id }, { id: write.id }];
    });

    const token = new cloudflare.AccountToken(
      "r2-token",
      {
        accountId: args.accountId,
        name: pulumi.interpolate`fn0-static-page-storage-${bucket.name}`,
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
      { parent: this, dependsOn: [bucket] },
    );

    this.accountId = pulumi.output(args.accountId);
    this.bucketName = bucket.name;
    this.endpoint = pulumi.interpolate`https://${args.accountId}.r2.cloudflarestorage.com`;
    this.accessKeyId = token.id;
    this.secretAccessKey = pulumi.secret(
      token.value.apply((value) => crypto.createHash("sha256").update(value).digest("hex")),
    );
  }
}
