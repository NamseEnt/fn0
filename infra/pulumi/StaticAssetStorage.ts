import * as pulumi from "@pulumi/pulumi";
import * as cloudflare from "@pulumi/cloudflare";
import * as crypto from "crypto";

export interface StaticAssetStorageArgs {
  accountId: pulumi.Input<string>;
  zoneId: pulumi.Input<string>;
  publicBaseDomain: pulumi.Input<string>;
  cloudflareUserApiToken: pulumi.Input<string>;
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

    // The presign token must grant Workers R2 Storage Bucket Item Read/Write
    // against ALL buckets in the account, because control creates per-project
    // R2 buckets on demand inside its new_project action and reuses this one
    // token to issue SigV4 PUT presigned URLs for any of them.
    //
    // Cloudflare expresses that scope as a NESTED resources map
    //   "com.cloudflare.api.account.<ACCOUNT_ID>": {
    //     "com.cloudflare.edge.r2.bucket.*": "*"
    //   }
    // (https://developers.cloudflare.com/r2/api/tokens/) which (a) is only
    // accepted on user-owned tokens (POST /user/tokens), and (b) cannot be
    // expressed through cloudflare.AccountToken/ApiToken because the
    // @pulumi/cloudflare v6 schema types policy resources as a flat
    // { [key: string]: string }. So this token is minted via a
    // pulumi.dynamic.Resource that calls Cloudflare's REST API directly,
    // authenticated with the user's bootstrap token (see args).
    const presignToken = new R2AllBucketsItemToken(
      "presign-token",
      {
        accountId,
        name: pulumi.interpolate`fn0-static-asset-presign-${accountId}`,
        permissionGroupIds: [r2ItemReadId, r2ItemWriteId],
        userApiToken: args.cloudflareUserApiToken,
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
            resources: pulumi.output(accountId).apply((acct) =>
              JSON.stringify({
                [`com.cloudflare.api.account.${acct}`]: "*",
              })
            ),
            permissionGroups: [{ id: r2EditId }],
          },
          {
            effect: "allow",
            resources: pulumi.output(zoneId).apply((zid) =>
              JSON.stringify({
                [`com.cloudflare.api.account.zone.${zid}`]: "*",
              })
            ),
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

interface R2AllBucketsItemTokenArgs {
  accountId: pulumi.Input<string>;
  name: pulumi.Input<string>;
  permissionGroupIds: pulumi.Input<pulumi.Input<string>[]>;
  userApiToken: pulumi.Input<string>;
}

class R2AllBucketsItemToken extends pulumi.dynamic.Resource {
  public readonly value!: pulumi.Output<string>;

  constructor(
    name: string,
    args: R2AllBucketsItemTokenArgs,
    opts?: pulumi.CustomResourceOptions
  ) {
    super(
      new R2AllBucketsItemTokenProvider(),
      name,
      { ...args, value: undefined },
      opts
    );
  }
}

interface R2AllBucketsItemTokenInputs {
  accountId: string;
  name: string;
  permissionGroupIds: string[];
  userApiToken: string;
}

type R2AllBucketsItemTokenOutputs = R2AllBucketsItemTokenInputs & {
  value: string;
};

function buildTokenPolicies(inputs: R2AllBucketsItemTokenInputs) {
  return [
    {
      effect: "allow",
      // Nested form: account-scoped wildcard over all R2 buckets. This is the
      // only Cloudflare-documented way to grant item-level R2 permissions
      // across every bucket in an account.
      resources: {
        [`com.cloudflare.api.account.${inputs.accountId}`]: {
          "com.cloudflare.edge.r2.bucket.*": "*",
        },
      },
      permission_groups: inputs.permissionGroupIds.map((id) => ({ id })),
    },
  ];
}

async function cloudflareUserTokenRequest(
  userApiToken: string,
  path: string,
  init: RequestInit
): Promise<{ id: string; value?: string }> {
  const response = await fetch(
    `https://api.cloudflare.com/client/v4/user/tokens${path}`,
    {
      ...init,
      headers: {
        Authorization: `Bearer ${userApiToken}`,
        "Content-Type": "application/json",
        ...(init.headers ?? {}),
      },
    }
  );
  const text = await response.text();
  if (!response.ok) {
    throw new Error(
      `Cloudflare token API ${init.method} ${path} failed: ${response.status} ${text}`
    );
  }
  const body = JSON.parse(text);
  if (body.success === false) {
    throw new Error(
      `Cloudflare token API ${init.method} ${path} returned errors: ${JSON.stringify(body.errors)}`
    );
  }
  return body.result;
}

class R2AllBucketsItemTokenProvider
  implements
    pulumi.dynamic.ResourceProvider<
      R2AllBucketsItemTokenInputs,
      R2AllBucketsItemTokenOutputs
    >
{
  async create(
    inputs: R2AllBucketsItemTokenInputs
  ): Promise<pulumi.dynamic.CreateResult<R2AllBucketsItemTokenOutputs>> {
    const result = await cloudflareUserTokenRequest(inputs.userApiToken, "", {
      method: "POST",
      body: JSON.stringify({
        name: inputs.name,
        policies: buildTokenPolicies(inputs),
      }),
    });
    if (!result.value) {
      throw new Error("Cloudflare token create response missing value");
    }
    return {
      id: result.id,
      outs: { ...inputs, value: result.value },
    };
  }

  async diff(
    _id: string,
    oldOutputs: R2AllBucketsItemTokenOutputs,
    newInputs: R2AllBucketsItemTokenInputs
  ): Promise<pulumi.dynamic.DiffResult> {
    // accountId change means a different Cloudflare account entirely — token
    // cannot be transferred, must replace.
    const replaces: string[] = [];
    if (oldOutputs.accountId !== newInputs.accountId) replaces.push("accountId");
    const changed =
      oldOutputs.accountId !== newInputs.accountId ||
      oldOutputs.name !== newInputs.name ||
      JSON.stringify(oldOutputs.permissionGroupIds) !==
        JSON.stringify(newInputs.permissionGroupIds);
    return { changes: changed, replaces };
  }

  async update(
    id: string,
    oldOutputs: R2AllBucketsItemTokenOutputs,
    newInputs: R2AllBucketsItemTokenInputs
  ): Promise<pulumi.dynamic.UpdateResult<R2AllBucketsItemTokenOutputs>> {
    await cloudflareUserTokenRequest(newInputs.userApiToken, `/${id}`, {
      method: "PUT",
      body: JSON.stringify({
        name: newInputs.name,
        policies: buildTokenPolicies(newInputs),
      }),
    });
    // Token value is unchanged by PUT — preserve from previous outputs.
    return { outs: { ...newInputs, value: oldOutputs.value } };
  }

  async delete(id: string, outputs: R2AllBucketsItemTokenOutputs) {
    const response = await fetch(
      `https://api.cloudflare.com/client/v4/user/tokens/${id}`,
      {
        method: "DELETE",
        headers: { Authorization: `Bearer ${outputs.userApiToken}` },
      }
    );
    if (!response.ok && response.status !== 404) {
      throw new Error(
        `Cloudflare token DELETE failed: ${response.status} ${await response.text()}`
      );
    }
  }
}
