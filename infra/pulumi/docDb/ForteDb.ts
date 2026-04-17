import * as pulumi from "@pulumi/pulumi";
import { TursoGroup } from "./turso/group";
import { TursoGroupToken } from "./turso/groupToken";

export interface ForteDbArgs {
  location: pulumi.Input<string>;
  organizationSlug: pulumi.Input<string>;
}

/**
 * ForteDb owns the Turso group ("forte-db") that holds one database per
 * deployed forte app. Databases are created on demand by HQ at `forte deploy`
 * time. A single group-scoped auth token is issued here and handed to workers
 * so they can speak hrana to any app database without per-db credentials.
 */
export class ForteDb extends pulumi.ComponentResource {
  public readonly groupName: pulumi.Output<string>;
  public readonly groupToken: pulumi.Output<string>;
  public readonly hostSuffix: pulumi.Output<string>;

  constructor(
    name: string,
    args: ForteDbArgs,
    opts: pulumi.ComponentResourceOptions
  ) {
    super("pkg:index:forte-db", name, args, opts);
    const { location, organizationSlug } = args;

    const group = new TursoGroup(
      "group",
      {
        organizationSlug,
        name: "forte-db",
        location,
      },
      { parent: this }
    );

    const token = new TursoGroupToken(
      "token",
      {
        organizationSlug,
        groupName: group.name,
      },
      { parent: this }
    );

    this.groupName = group.name;
    this.groupToken = token.jwt;
    this.hostSuffix = pulumi.interpolate`-${organizationSlug}.${location}.turso.io`;
  }
}
