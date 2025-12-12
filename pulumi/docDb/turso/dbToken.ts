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
    super(new TursoDatabaseTokenProvider(), name, args, opts);
  }
}

interface TursoDatabaseTokenInputs {
  organizationSlug: string;
  databaseName: string;
}

type TursoDatabaseTokenOutputs = TursoDatabaseTokenInputs & {
  jwt: string;
};

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
    const config = new pulumi.Config();
    const apiKey = config.require("tursoApiToken");

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

    return {
      id: `${inputs.databaseName}-token`,
      outs: {
        ...inputs,
        ...data,
      },
    };
  }
}
