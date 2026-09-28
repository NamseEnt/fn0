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

export interface TursoGroupInputs {
  organizationSlug: string;
  name: string;
  location: string;
}

export class TursoGroupProvider
  implements
    pulumi.dynamic.ResourceProvider<TursoGroupInputs, TursoGroupInputs>
{
  async diff(
    id: string,
    olds: TursoGroupInputs,
    news: TursoGroupInputs
  ): Promise<pulumi.dynamic.DiffResult> {
    return {
      changes:
        olds.organizationSlug !== news.organizationSlug ||
        olds.name !== news.name ||
        olds.location !== news.location,
    };
  }

  async create(
    inputs: TursoGroupInputs
  ): Promise<pulumi.dynamic.CreateResult<TursoGroupInputs>> {
    const config = new pulumi.Config("turso");
    const apiKey = config.require("apiToken");

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

  async update(
    id: string,
    olds: TursoGroupInputs,
    news: TursoGroupInputs
  ): Promise<pulumi.dynamic.UpdateResult<TursoGroupInputs>> {
    const oldInputs = olds as TursoGroupInputs & { __provider?: string };
    const newInputs = news as TursoGroupInputs & { __provider?: string };
    const oldBusinessInputsAvailable = [
      oldInputs.organizationSlug,
      oldInputs.name,
      oldInputs.location,
    ].every((value) => typeof value === "string");

    if (
      !oldBusinessInputsAvailable &&
      oldInputs.organizationSlug === undefined &&
      oldInputs.name === undefined &&
      oldInputs.location === undefined &&
      typeof oldInputs.__provider === "string" &&
      typeof newInputs.__provider === "string" &&
      oldInputs.__provider !== newInputs.__provider
    ) {
      return {
        outs: {
          organizationSlug: news.organizationSlug,
          name: news.name,
          location: news.location,
        },
      };
    }

    const changedProperties = [
      oldInputs.organizationSlug !== news.organizationSlug && "organizationSlug",
      oldInputs.name !== news.name && "name",
      oldInputs.location !== news.location && "location",
    ].filter((property): property is string => property !== false);
    if (changedProperties.length > 0) {
      throw new Error(
        `Turso group update is unsupported until migration semantics are defined; changed inputs: ${changedProperties.join(", ")}. No Turso API request was made.`
      );
    }

    return {
      outs: {
        organizationSlug: news.organizationSlug,
        name: news.name,
        location: news.location,
      },
    };
  }

  async delete(id: string, outputs: TursoGroupInputs) {
    const config = new pulumi.Config("turso");
    const apiKey = config.require("apiToken");

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
