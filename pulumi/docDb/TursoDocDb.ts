import * as pulumi from "@pulumi/pulumi";
import { TursoGroup } from "./turso/group";
import { TursoDatabase } from "./turso/database";
import { TursoDatabaseToken } from "./turso/dbToken";
import { TursoTable } from "./turso/table";

export interface TursoDocDbArgs {
  location: pulumi.Input<string>;
  organizationSlug: pulumi.Input<string>;
}

export class TursoDocDb extends pulumi.ComponentResource {
  constructor(
    name: string,
    args: TursoDocDbArgs,
    opts: pulumi.ComponentResourceOptions
  ) {
    super("pkg:index:turso-doc-db", name, args, opts);
    const { location, organizationSlug } = args;

    const group = new TursoGroup(
      "group",
      {
        organizationSlug,
        name: "fn0-doc-db",
        location,
      },
      { parent: this }
    );

    const database = new TursoDatabase(
      "database",
      {
        organizationSlug,
        name: "fn0-doc-db",
        group: group.name,
      },
      { parent: this }
    );

    const token = new TursoDatabaseToken(
      "token",
      {
        organizationSlug,
        databaseName: database.name,
      },
      { parent: this }
    );

    new TursoTable(
      "docs-table",
      {
        organizationSlug,
        jwt: token.jwt,
        databaseName: database.name,
        createTableSql:
          "CREATE TABLE IF NOT EXISTS docs (key TEXT PRIMARY KEY, value TEXT)",
      },
      { parent: this }
    );
  }
}
