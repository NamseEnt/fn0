import * as pulumi from "@pulumi/pulumi";
import { TursoDatabase } from "./docDb/turso/database";
import { ControlProjectSeed } from "./ControlProjectSeed";

export interface ControlProjectBootstrapArgs {
  organizationSlug: pulumi.Input<string>;
  location: pulumi.Input<string>;
  groupName: pulumi.Input<string>;
  jwt: pulumi.Input<string>;
  projectId: pulumi.Input<string>;
  ownerGithubId: pulumi.Input<number>;
  ownerGithubLogin: pulumi.Input<string>;
  displayName: pulumi.Input<string>;
}

export class ControlProjectBootstrap extends pulumi.ComponentResource {
  public readonly databaseName: pulumi.Output<string>;

  constructor(
    name: string,
    args: ControlProjectBootstrapArgs,
    opts?: pulumi.ComponentResourceOptions
  ) {
    super("pkg:index:control-project-bootstrap", name, args, opts);

    const database = new TursoDatabase(
      "database",
      {
        organizationSlug: args.organizationSlug,
        name: args.projectId,
        group: args.groupName,
      },
      { parent: this }
    );

    new ControlProjectSeed(
      "seed",
      {
        organizationSlug: args.organizationSlug,
        location: args.location,
        databaseName: args.projectId,
        jwt: args.jwt,
        projectId: args.projectId,
        ownerGithubId: args.ownerGithubId,
        ownerGithubLogin: args.ownerGithubLogin,
        displayName: args.displayName,
      },
      { parent: this, dependsOn: [database] }
    );

    this.databaseName = pulumi.output(args.projectId);

    this.registerOutputs({
      databaseName: this.databaseName,
    });
  }
}
