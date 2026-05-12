import * as pulumi from "@pulumi/pulumi";
import { TursoGroup } from "./turso/group";
import { TursoDatabase } from "./turso/database";
import { TursoDatabaseToken } from "./turso/dbToken";
import { TursoTable } from "./turso/table";
import * as random from "@pulumi/random";

export interface TursoDocDbArgs {
  location: pulumi.Input<string>;
  organizationSlug: pulumi.Input<string>;
}

export class TursoDocDb extends pulumi.ComponentResource {
  public readonly url: pulumi.Output<string>;
  public readonly token: pulumi.Output<string>;

  constructor(
    name: string,
    args: TursoDocDbArgs,
    opts: pulumi.ComponentResourceOptions
  ) {
    super("pkg:index:turso-doc-db", name, args, opts);
    const { location, organizationSlug } = args;

    const nameSuffix8 = new random.RandomString(
      "name-suffix-8",
      {
        length: 8,
        special: false,
        upper: false,
      },
      { parent: this }
    ).result;

    const groupName = "fn0-doc-db";
    const databaseName = pulumi.interpolate`fn0-doc-db-${nameSuffix8}`;

    const group = new TursoGroup(
      "group",
      {
        organizationSlug,
        name: groupName,
        location,
      },
      { parent: this }
    );

    const database = new TursoDatabase(
      "database",
      {
        organizationSlug,
        name: databaseName,
        group: groupName,
      },
      { parent: this, dependsOn: [group] }
    );

    const token = new TursoDatabaseToken(
      "token",
      {
        organizationSlug,
        databaseName,
      },
      { parent: this, dependsOn: [database] }
    );

    new TursoTable(
      "docs-table",
      {
        organizationSlug,
        location,
        jwt: token.jwt,
        databaseName,
        createTableSql: `
CREATE TABLE IF NOT EXISTS docs (
  pk BLOB NOT NULL,
  sk BLOB NOT NULL,
  value BLOB NOT NULL,
  PRIMARY KEY (pk, sk)
) WITHOUT ROWID;`.trim(),
      },
      { parent: this, dependsOn: [database] }
    );

    this.url = pulumi.interpolate`libsql://${databaseName}-${organizationSlug}.${location}.turso.io`;
    this.token = token.jwt;
  }
}
