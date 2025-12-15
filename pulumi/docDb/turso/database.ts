import * as pulumi from "@pulumi/pulumi";

interface TursoDatabaseArgs {
  organizationSlug: pulumi.Input<string>;
  name: pulumi.Input<string>;
  group: pulumi.Input<string>;
}

export class TursoDatabase extends pulumi.dynamic.Resource {
  public readonly name!: pulumi.Output<string>;

  constructor(
    name: string,
    args: TursoDatabaseArgs,
    opts?: pulumi.CustomResourceOptions
  ) {
    super(new TursoDatabaseProvider(), name, args, opts);
  }
}

interface TursoDatabaseInputs {
  organizationSlug: string;
  name: string;
  group: string;
}

type TursoDatabaseOutputs = TursoDatabaseInputs & {
  name: string;
};

class TursoDatabaseProvider
  implements
    pulumi.dynamic.ResourceProvider<TursoDatabaseInputs, TursoDatabaseOutputs>
{
  async create(
    inputs: TursoDatabaseInputs
  ): Promise<pulumi.dynamic.CreateResult<TursoDatabaseOutputs>> {
    const config = new pulumi.Config();
    const apiKey = config.require("tursoApiToken");

    const response = await fetch(
      `https://api.turso.tech/v1/organizations/${inputs.organizationSlug}/databases`,
      {
        method: "POST",
        headers: {
          Authorization: `Bearer ${apiKey}`,
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          name: inputs.name,
          group: inputs.group,
        }),
      }
    );

    if (!response.ok) {
      const error = await response.text();
      throw new Error(`Failed to create database: ${error}`);
    }

    const data = await response.json();

    return {
      id: data.database.Name,
      outs: {
        ...inputs,
        name: data.database.Name,
      },
    };
  }

  async delete(id: string, outputs: TursoDatabaseInputs) {
    const config = new pulumi.Config();
    const apiKey = config.require("tursoApiToken");

    const response = await fetch(
      `https://api.turso.tech/v1/organizations/${outputs.organizationSlug}/databases/${id}`,
      {
        method: "DELETE",
        headers: {
          Authorization: `Bearer ${apiKey}`,
        },
      }
    );

    if (!response.ok && response.status !== 404) {
      const error = await response.text();
      throw new Error(`Failed to delete database: ${error}`);
    }
  }
}
