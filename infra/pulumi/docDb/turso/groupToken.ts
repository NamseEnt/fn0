import * as pulumi from "@pulumi/pulumi";

interface TursoGroupTokenArgs {
  organizationSlug: pulumi.Input<string>;
  groupName: pulumi.Input<string>;
}

export class TursoGroupToken extends pulumi.dynamic.Resource {
  public readonly jwt!: pulumi.Output<string>;

  constructor(
    name: string,
    args: TursoGroupTokenArgs,
    opts?: pulumi.CustomResourceOptions
  ) {
    super(
      new TursoGroupTokenProvider(),
      name,
      { ...args, jwt: undefined },
      opts
    );
  }
}

interface TursoGroupTokenInputs {
  organizationSlug: string;
  groupName: string;
}

type TursoGroupTokenOutputs = TursoGroupTokenInputs & {
  jwt: string;
};

async function issueToken(
  inputs: TursoGroupTokenInputs
): Promise<string> {
  const config = new pulumi.Config("turso");
  const apiKey = config.require("apiToken");

  const response = await fetch(
    `https://api.turso.tech/v1/organizations/${inputs.organizationSlug}/groups/${inputs.groupName}/auth/tokens`,
    {
      method: "POST",
      headers: {
        Authorization: `Bearer ${apiKey}`,
      },
    }
  );

  if (!response.ok) {
    const error = await response.text();
    throw new Error(`Failed to create group token: ${error}`);
  }

  const data = await response.json();
  return data.jwt;
}

class TursoGroupTokenProvider
  implements
    pulumi.dynamic.ResourceProvider<
      TursoGroupTokenInputs,
      TursoGroupTokenOutputs
    >
{
  async create(
    inputs: TursoGroupTokenInputs
  ): Promise<pulumi.dynamic.CreateResult> {
    const jwt = await issueToken(inputs);
    return {
      id: `${inputs.groupName}-token`,
      outs: { ...inputs, jwt },
    };
  }

  async diff(
    _id: string,
    olds: TursoGroupTokenOutputs,
    news: TursoGroupTokenInputs
  ): Promise<pulumi.dynamic.DiffResult> {
    const replaces: string[] = [];
    if (olds.groupName !== news.groupName) replaces.push("groupName");
    if (olds.organizationSlug !== news.organizationSlug)
      replaces.push("organizationSlug");
    return {
      changes: replaces.length > 0,
      replaces,
      deleteBeforeReplace: true,
    };
  }

  async update(
    _id: string,
    olds: TursoGroupTokenOutputs,
    news: TursoGroupTokenInputs
  ): Promise<pulumi.dynamic.UpdateResult> {
    const jwt = olds.jwt ?? (await issueToken(news));
    return { outs: { ...news, jwt } };
  }
}
