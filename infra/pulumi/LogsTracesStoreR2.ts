import * as pulumi from "@pulumi/pulumi";
import * as cloudflare from "@pulumi/cloudflare";
import * as crypto from "crypto";

// Bucket naming follows the sibling stores: `fn0-<what-it-holds>-<suffix>`,
// minted here and never renamed — the name is baked into loggytracy's
// `LOGGYTRACY_OBJECT_STORE_URL` and into every object path already written.
//
// This bucket is the log/trace engine's source of truth, not a backup of one:
// the node's local disk is a cache and a WAL, and everything older than the
// unflushed window exists only here. Nothing may expire objects in it — no
// lifecycle rule, no GC sweep — which is what separates it from
// `MetricsBackupR2`, whose contents are rebuilt by the next backup run.
export interface LogsTracesStoreR2Args {
  tokenMintingApiToken: pulumi.Input<string>;
  accountId: pulumi.Input<string>;
  bucketName: pulumi.Input<string>;
  location?: pulumi.Input<string>;
}

export class LogsTracesStoreR2 extends pulumi.ComponentResource {
  public readonly accountId: pulumi.Output<string>;
  public readonly bucketName: pulumi.Output<string>;
  public readonly endpoint: pulumi.Output<string>;
  public readonly accessKeyId: pulumi.Output<string>;
  public readonly secretAccessKey: pulumi.Output<string>;

  constructor(
    name: string,
    args: LogsTracesStoreR2Args,
    opts: pulumi.ComponentResourceOptions,
  ) {
    super("pkg:index:logs-traces-store-r2", name, args, opts);

    const { accountId, bucketName, location } = args;

    const bucket = new cloudflare.R2Bucket(
      "bucket",
      {
        accountId,
        name: bucketName,
        location: location ?? "apac",
      },
      { parent: this },
    );

    const permissionGroups =
      // No `parent` on this invoke, deliberately: the permission-group catalog is
      // Cloudflare's own static list, and reading it through the component's
      // provider would make it depend on the operator token this stack is still
      // in the middle of creating. The default (bootstrap) provider can always
      // read it, so the lookup stays available on a first run.
      cloudflare.getAccountApiTokenPermissionGroupsListOutput({
        accountId,
        maxItems: 1000,
      });

    const r2PermissionIds = permissionGroups.apply((list) => {
      const groups = list.results ?? [];
      const read = groups.find(
        (g) => g.name === "Workers R2 Storage Bucket Item Read",
      );
      const write = groups.find(
        (g) => g.name === "Workers R2 Storage Bucket Item Write",
      );
      if (!read || !write) {
        throw new Error(
          `Could not find R2 permission groups; found: ${groups
            .map((g) => g.name)
            .join(", ")}`,
        );
      }
      return [{ id: read.id }, { id: write.id }];
    });

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

    const token = new cloudflare.AccountToken(
      "r2-token",
      {
        accountId,
        name: pulumi.interpolate`fn0-logs-traces-store-r2-${bucket.name}`,
        policies: [
          {
            effect: "allow",
            resources: pulumi
              .all([accountId, bucket.name])
              .apply(([acct, bn]) =>
                JSON.stringify({
                  [`com.cloudflare.edge.r2.bucket.${acct}_default_${bn}`]: "*",
                }),
              ),
            permissionGroups: r2PermissionIds,
          },
        ],
      },
      { parent: this, dependsOn: [bucket], provider: tokenMintingProvider },
    );

    this.accountId = pulumi.output(accountId);
    this.bucketName = bucket.name;
    this.endpoint = pulumi.interpolate`https://${accountId}.r2.cloudflarestorage.com`;
    this.accessKeyId = token.id;
    this.secretAccessKey = pulumi.secret(
      token.value.apply((v) =>
        crypto.createHash("sha256").update(v).digest("hex"),
      ),
    );
  }
}
