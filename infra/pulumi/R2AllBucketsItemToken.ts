import * as pulumi from "@pulumi/pulumi";

// Cloudflare scopes R2 S3-API credentials to ALL buckets in an account via a
// NESTED resources map
//   "com.cloudflare.api.account.<ACCOUNT_ID>": {
//     "com.cloudflare.edge.r2.bucket.*": "*"
//   }
// (https://developers.cloudflare.com/r2/api/tokens/) which (a) is only
// accepted on user-owned tokens (POST /user/tokens), and (b) cannot be
// expressed through cloudflare.AccountToken/ApiToken because the
// @pulumi/cloudflare v6 schema types policy resources as a flat
// { [key: string]: string }. So this token is minted via a
// pulumi.dynamic.Resource that calls Cloudflare's REST API directly,
// authenticated with the user's bootstrap token.
//
// Used wherever fn0 creates per-project R2 buckets on demand and needs one
// long-lived credential that works against any of them (static-asset presign,
// object-storage worker access).

export interface R2AllBucketsItemTokenArgs {
  accountId: pulumi.Input<string>;
  name: pulumi.Input<string>;
  permissionGroupIds: pulumi.Input<pulumi.Input<string>[]>;
  userApiToken: pulumi.Input<string>;
}

export class R2AllBucketsItemToken extends pulumi.dynamic.Resource {
  public readonly value!: pulumi.Output<string>;

  constructor(
    name: string,
    args: R2AllBucketsItemTokenArgs,
    opts?: pulumi.CustomResourceOptions,
  ) {
    super(
      new R2AllBucketsItemTokenProvider(),
      name,
      { ...args, value: undefined },
      opts,
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
  init: RequestInit,
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
    },
  );
  const text = await response.text();
  if (!response.ok) {
    throw new Error(
      `Cloudflare token API ${init.method} ${path} failed: ${response.status} ${text}`,
    );
  }
  const body = JSON.parse(text);
  if (body.success === false) {
    throw new Error(
      `Cloudflare token API ${init.method} ${path} returned errors: ${JSON.stringify(body.errors)}`,
    );
  }
  return body.result;
}

class R2AllBucketsItemTokenProvider implements pulumi.dynamic.ResourceProvider<
  R2AllBucketsItemTokenInputs,
  R2AllBucketsItemTokenOutputs
> {
  async create(
    inputs: R2AllBucketsItemTokenInputs,
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
    newInputs: R2AllBucketsItemTokenInputs,
  ): Promise<pulumi.dynamic.DiffResult> {
    // accountId change means a different Cloudflare account entirely — token
    // cannot be transferred, must replace.
    const replaces: string[] = [];
    if (oldOutputs.accountId !== newInputs.accountId)
      replaces.push("accountId");
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
    newInputs: R2AllBucketsItemTokenInputs,
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
      },
    );
    if (!response.ok && response.status !== 404) {
      throw new Error(
        `Cloudflare token DELETE failed: ${response.status} ${await response.text()}`,
      );
    }
  }
}
