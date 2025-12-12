import * as pulumi from "@pulumi/pulumi";

interface TursoGroupArgs {
  organizationSlug: pulumi.Input<string>;
  name: pulumi.Input<string>;
  location: pulumi.Input<string>;
}

export class TursoGroup extends pulumi.dynamic.Resource {
  public name!: pulumi.Output<string>;
  constructor(
    name: string,
    args: TursoGroupArgs,
    opts?: pulumi.CustomResourceOptions
  ) {
    super(new TursoGroupProvider(), name, args, opts);
  }
}

interface TursoGroupInputs {
  organizationSlug: string;
  name: string;
  location: string;
}

class TursoGroupProvider
  implements
    pulumi.dynamic.ResourceProvider<TursoGroupInputs, TursoGroupInputs>
{
  async create(
    inputs: TursoGroupInputs
  ): Promise<pulumi.dynamic.CreateResult<TursoGroupInputs>> {
    const config = new pulumi.Config();
    const apiKey = config.require("tursoApiToken");

    const response = await fetch(
      `https://api.turso.tech/v1/organizations/${inputs.organizationSlug}/groups`,
      {
        method: "POST",
        headers: {
          Authorization: `Bearer ${apiKey}`,
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          name: inputs.name,
          location: inputs.location,
        }),
      }
    );

    if (!response.ok) {
      const error = await response.text();
      throw new Error(`Failed to create group: ${error}`);
    }

    const data = await response.json();
    const group = data.group;

    return {
      id: group.uuid,
      outs: inputs,
    };
  }

  async delete(id: string, outputs: TursoGroupInputs) {
    const config = new pulumi.Config();
    const apiKey = config.require("tursoApiToken");

    const groupName = id.split("/").pop();
    const response = await fetch(
      `https://api.turso.tech/v1/organizations/${outputs.organizationSlug}/groups/${groupName}`,
      {
        method: "DELETE",
        headers: {
          Authorization: `Bearer ${apiKey}`,
        },
      }
    );

    if (!response.ok && response.status !== 404) {
      const error = await response.text();
      throw new Error(`Failed to delete group: ${error}`);
    }
  }
}
