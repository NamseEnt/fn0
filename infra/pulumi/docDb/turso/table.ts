import * as pulumi from "@pulumi/pulumi";

interface TursoTableArgs {
  organizationSlug: pulumi.Input<string>;
  location: pulumi.Input<string>;
  jwt: pulumi.Input<string>;
  databaseName: pulumi.Input<string>;
  createTableSql: pulumi.Input<string>;
}

export class TursoTable extends pulumi.dynamic.Resource {
  public readonly tableName!: pulumi.Output<string>;

  constructor(
    name: string,
    args: TursoTableArgs,
    opts?: pulumi.CustomResourceOptions
  ) {
    super(new TursoTableProvider(), name, args, opts);
  }
}

interface TursoTableInputs {
  organizationSlug: string;
  location: string;
  jwt: string;
  databaseName: string;
  createTableSql: string;
}

class TursoTableProvider implements pulumi.dynamic.ResourceProvider {
  async create(inputs: TursoTableInputs): Promise<pulumi.dynamic.CreateResult> {
    const config = new pulumi.Config("turso");
    const apiKey = config.require("apiToken");

    const tokenResp = await fetch(
      `https://api.turso.tech/v1/organizations/${inputs.organizationSlug}/databases/${inputs.databaseName}/auth/tokens`,
      {
        method: "POST",
        headers: { Authorization: `Bearer ${apiKey}` },
      }
    );
    if (!tokenResp.ok) {
      throw new Error(`Failed to get DB token: ${await tokenResp.text()}`);
    }
    const { jwt } = await tokenResp.json();

    const url = `https://${inputs.databaseName}-${inputs.organizationSlug}.${inputs.location}.turso.io/v2/pipeline`;
    const response = await fetch(
      url,
      {
        method: "POST",
        headers: {
          Authorization: `Bearer ${jwt}`,
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          requests: [
            {
              type: "execute",
              stmt: {
                sql: inputs.createTableSql,
              },
            },
            {
              type: "close",
            },
          ],
        }),
      }
    );

    if (!response.ok) {
      const error = await response.text();
      throw new Error(`Failed to create table at ${url}: ${error}`);
    }

    const crypto = await import("crypto");
    const hash = crypto.createHash("sha256");
    hash.update(inputs.createTableSql);
    const digest = hash.digest("hex");

    return {
      id: digest,
      outs: inputs,
    };
  }

  async delete(id: string, outputs: TursoTableInputs) {
    const response = await fetch(
      `https://${outputs.databaseName}-${outputs.organizationSlug}.turso.io`,
      {
        method: "POST",
        headers: {
          Authorization: `Bearer ${outputs.jwt}`,
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          requests: [
            {
              type: "execute",
              stmt: {
                sql: `DROP TABLE IF EXISTS ${id}`,
              },
            },
            {
              type: "close",
            },
          ],
        }),
      }
    );

    if (!response.ok) {
      const error = await response.text();
      throw new Error(`Failed to delete table: ${error}`);
    }
  }
}
