import * as pulumi from "@pulumi/pulumi";

interface TursoDatabaseTokenArgs {
  organizationSlug: pulumi.Input<string>;
  databaseName: pulumi.Input<string>;
}

export class TursoDatabaseToken extends pulumi.dynamic.Resource {
  public readonly jwt!: pulumi.Output<string>;

  constructor(
    name: string,
    args: TursoDatabaseTokenArgs,
    opts?: pulumi.CustomResourceOptions
  ) {
    super(new TursoDatabaseTokenProvider(), name, { ...args, jwt: undefined }, opts);
  }
}

interface TursoDatabaseTokenInputs {
  organizationSlug: string;
  databaseName: string;
}

type TursoDatabaseTokenOutputs = TursoDatabaseTokenInputs & {
  jwt: string;
};

async function issueDatabaseToken(
  inputs: TursoDatabaseTokenInputs
): Promise<string> {
  const config = new pulumi.Config("turso");
  const apiKey = config.require("apiToken");
  const response = await fetch(
    `https://api.turso.tech/v1/organizations/${inputs.organizationSlug}/databases/${inputs.databaseName}/auth/tokens`,
    {
      method: "POST",
      headers: {
        Authorization: `Bearer ${apiKey}`,
      },
    }
  );
  if (!response.ok) {
    const error = await response.text();
    throw new Error(`Failed to create database token: ${error}`);
  }
  const data = await response.json();
  return data.jwt;
}

class TursoDatabaseTokenProvider
  implements
    pulumi.dynamic.ResourceProvider<
      TursoDatabaseTokenInputs,
      TursoDatabaseTokenOutputs
    >
{
  async create(
    inputs: TursoDatabaseTokenInputs
  ): Promise<pulumi.dynamic.CreateResult> {
    const jwt = await issueDatabaseToken(inputs);
    return {
      id: `${inputs.databaseName}-token`,
      outs: { ...inputs, jwt },
    };
  }

  async diff(
    _id: string,
    olds: TursoDatabaseTokenOutputs,
    news: TursoDatabaseTokenInputs
  ): Promise<pulumi.dynamic.DiffResult> {
    const replaces: string[] = [];
    if (olds.organizationSlug !== news.organizationSlug)
      replaces.push("organizationSlug");
    if (olds.databaseName !== news.databaseName)
      replaces.push("databaseName");
    return {
      changes: replaces.length > 0,
      replaces,
      deleteBeforeReplace: true,
    };
  }

  async update(
    _id: string,
    olds: TursoDatabaseTokenOutputs,
    news: TursoDatabaseTokenInputs
  ): Promise<pulumi.dynamic.UpdateResult> {
    const jwt = olds.jwt ?? (await issueDatabaseToken(news));
    return { outs: { ...news, jwt } };
  }
}
