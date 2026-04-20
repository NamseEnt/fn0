import * as pulumi from "@pulumi/pulumi";
import * as cloudflare from "@pulumi/cloudflare";
import * as crypto from "crypto";

export interface ForteR2Args {
  accountId: pulumi.Input<string>;
  zoneId: pulumi.Input<string>;
  domain: pulumi.Input<string>;
  staticHostname: pulumi.Input<string>;
  bucketName: pulumi.Input<string>;
  location?: pulumi.Input<string>;
}

export class ForteR2 extends pulumi.ComponentResource {
  public readonly bucketName: pulumi.Output<string>;
  public readonly endpoint: pulumi.Output<string>;
  public readonly publicBaseUrl: pulumi.Output<string>;
  public readonly accessKeyId: pulumi.Output<string>;
  public readonly secretAccessKey: pulumi.Output<string>;
  public readonly accountId: pulumi.Output<string>;

  constructor(
    name: string,
    args: ForteR2Args,
    opts: pulumi.ComponentResourceOptions
  ) {
    super("pkg:index:forte-r2", name, args, opts);

    const { accountId, zoneId, domain, staticHostname, bucketName, location } = args;

    const bucket = new cloudflare.R2Bucket(
      "bucket",
      {
        accountId,
        name: bucketName,
        location: location ?? "apac",
      },
      { parent: this }
    );

    new cloudflare.R2CustomDomain(
      "custom-domain",
      {
        accountId,
        bucketName: bucket.name,
        domain: staticHostname,
        zoneId,
        enabled: true,
        minTls: "1.2",
      },
      { parent: this, dependsOn: [bucket] }
    );

    new cloudflare.R2BucketCors(
      "bucket-cors",
      {
        accountId,
        bucketName: bucket.name,
        rules: [
          {
            id: "forte-cross-origin-scripts",
            allowed: {
              methods: ["GET", "HEAD"],
              origins: [pulumi.interpolate`https://*.${domain}`],
              headers: ["*"],
            },
            maxAgeSeconds: 86400,
          },
        ],
      },
      { parent: this, dependsOn: [bucket] }
    );

    const permissionGroups = cloudflare.getAccountApiTokenPermissionGroupsListOutput(
      {
        accountId,
        maxItems: 1000,
      },
      { parent: this }
    );

    const r2PermissionIds = permissionGroups.apply((list) => {
      const groups = list.results ?? [];
      const read = groups.find(
        (g) => g.name === "Workers R2 Storage Bucket Item Read"
      );
      const write = groups.find(
        (g) => g.name === "Workers R2 Storage Bucket Item Write"
      );
      if (!read || !write) {
        throw new Error(
          `Could not find R2 permission groups; found names: ${groups
            .map((g) => g.name)
            .join(", ")}`
        );
      }
      return [{ id: read.id }, { id: write.id }];
    });

    const token = new cloudflare.AccountToken(
      "r2-admin-token",
      {
        accountId,
        name: pulumi.interpolate`fn0-forte-r2-${bucket.name}`,
        policies: [
          {
            effect: "allow",
            resources: pulumi.all([accountId, bucket.name]).apply(
              ([acct, name]) => ({
                [`com.cloudflare.edge.r2.bucket.${acct}_default_${name}`]: "*",
              })
            ),
            permissionGroups: r2PermissionIds,
          },
        ],
      },
      { parent: this, dependsOn: [bucket] }
    );

    this.bucketName = bucket.name;
    this.accountId = pulumi.output(accountId);
    this.endpoint = pulumi.interpolate`https://${accountId}.r2.cloudflarestorage.com`;
    this.publicBaseUrl = pulumi.interpolate`https://${staticHostname}`;
    this.accessKeyId = token.id;
    this.secretAccessKey = pulumi.secret(
      token.value.apply((v) =>
        crypto.createHash("sha256").update(v).digest("hex")
      )
    );
  }
}
