import * as pulumi from "@pulumi/pulumi";
import * as crypto from "crypto";
import { R2AllBucketsItemToken } from "./R2AllBucketsItemToken";

export interface ObjectStorageStorageArgs {
  accountId: pulumi.Input<string>;
  cloudflareUserApiToken: pulumi.Input<string>;
}

// R2 S3-API credentials the worker uses to sign user object-storage requests.
// control creates per-project `fn0-object-storage-<project_id>` buckets on
// deploy; the worker reuses this one all-buckets Read/Write token to SigV4-sign
// requests against any of them. Kept separate from the static-asset presign
// token so the two scopes can be revoked independently.
export class ObjectStorageStorage extends pulumi.ComponentResource {
  public readonly accountId: pulumi.Output<string>;
  public readonly accessKeyId: pulumi.Output<string>;
  public readonly secretAccessKey: pulumi.Output<string>;

  constructor(
    name: string,
    args: ObjectStorageStorageArgs,
    opts: pulumi.ComponentResourceOptions,
  ) {
    super("pkg:index:object-storage-storage", name, args, opts);

    const r2ItemReadId = "6a018a9f2fc74eb6b293b0c548f38b39";
    const r2ItemWriteId = "2efd5506f9c8494dacb1fa10a3e7d5b6";

    const token = new R2AllBucketsItemToken(
      "token",
      {
        accountId: args.accountId,
        name: pulumi.interpolate`fn0-object-storage-${args.accountId}`,
        permissionGroupIds: [r2ItemReadId, r2ItemWriteId],
        userApiToken: args.cloudflareUserApiToken,
      },
      { parent: this },
    );

    this.accountId = pulumi.output(args.accountId);
    this.accessKeyId = token.id;
    this.secretAccessKey = pulumi.secret(
      token.value.apply((v) =>
        crypto.createHash("sha256").update(v).digest("hex"),
      ),
    );
  }
}
