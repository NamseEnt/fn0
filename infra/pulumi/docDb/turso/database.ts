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

export interface TursoDatabaseInputs {
  organizationSlug: string;
  name: string;
  group: string;
}

type TursoDatabaseOutputs = TursoDatabaseInputs & {
  name: string;
};

export class TursoDatabaseProvider
  implements
    pulumi.dynamic.ResourceProvider<TursoDatabaseInputs, TursoDatabaseOutputs>
{
  async diff(
    id: string,
    olds: TursoDatabaseOutputs,
    news: TursoDatabaseInputs
  ): Promise<pulumi.dynamic.DiffResult> {
    return {
      changes:
        olds.organizationSlug !== news.organizationSlug ||
        olds.name !== news.name ||
        olds.group !== news.group,
    };
  }

  async create(
    inputs: TursoDatabaseInputs
  ): Promise<pulumi.dynamic.CreateResult<TursoDatabaseOutputs>> {
    const config = new pulumi.Config("turso");
    const apiKey = config.require("apiToken");

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

  async update(
    id: string,
    olds: TursoDatabaseOutputs,
    news: TursoDatabaseInputs
  ): Promise<pulumi.dynamic.UpdateResult<TursoDatabaseOutputs>> {
    const oldInputs = olds as TursoDatabaseOutputs & { __provider?: string };
    const newInputs = news as TursoDatabaseInputs & { __provider?: string };
    const oldBusinessInputsAvailable = [
      oldInputs.organizationSlug,
      oldInputs.name,
      oldInputs.group,
    ].every((value) => typeof value === "string");

    if (
      !oldBusinessInputsAvailable &&
      oldInputs.organizationSlug === undefined &&
      oldInputs.name === undefined &&
      oldInputs.group === undefined &&
      typeof oldInputs.__provider === "string" &&
      typeof newInputs.__provider === "string" &&
      oldInputs.__provider !== newInputs.__provider
    ) {
      return {
        outs: {
          organizationSlug: news.organizationSlug,
          name: news.name,
          group: news.group,
        },
      };
    }

    const changedProperties = [
      oldInputs.organizationSlug !== news.organizationSlug && "organizationSlug",
      oldInputs.name !== news.name && "name",
      oldInputs.group !== news.group && "group",
    ].filter((property): property is string => property !== false);
    if (changedProperties.length > 0) {
      throw new Error(
        `Turso database update is unsupported until migration semantics are defined; changed inputs: ${changedProperties.join(", ")}. No Turso API request was made.`
      );
    }

    return {
      outs: {
        organizationSlug: news.organizationSlug,
        name: news.name,
        group: news.group,
      },
    };
  }

  async delete(id: string, outputs: TursoDatabaseInputs) {
    const config = new pulumi.Config("turso");
    const apiKey = config.require("apiToken");

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
