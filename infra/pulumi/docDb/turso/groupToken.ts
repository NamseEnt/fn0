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

    return {
      id: `${inputs.groupName}-token`,
      outs: {
        ...inputs,
        jwt: data.jwt,
      },
    };
  }
}
