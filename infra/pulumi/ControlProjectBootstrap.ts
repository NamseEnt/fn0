import * as pulumi from "@pulumi/pulumi";
import { TursoDatabase } from "./docDb/turso/database";

/**
 * Provisions the turso database that hosts the fn0-control application. Doc
 * seeding (UserDoc / ProjectDoc / Fn0WasmtimeVersionDoc / CompiledBundleDoc /
 * WorkerManifestDoc) lives in scripts/bootstrap-fn0-control.sh — pulumi's
 * job ends at "the database exists".
 */
export interface ControlProjectBootstrapArgs {
  organizationSlug: pulumi.Input<string>;
  groupName: pulumi.Input<string>;
  projectId: pulumi.Input<string>;
}

export class ControlProjectBootstrap extends pulumi.ComponentResource {
  public readonly databaseName: pulumi.Output<string>;

  constructor(
    name: string,
    args: ControlProjectBootstrapArgs,
    opts?: pulumi.ComponentResourceOptions
  ) {
    super("pkg:index:control-project-bootstrap", name, args, opts);

    new TursoDatabase(
      "database",
      {
        organizationSlug: args.organizationSlug,
        name: args.projectId,
        group: args.groupName,
      },
      { parent: this }
    );

    this.databaseName = pulumi.output(args.projectId);

    this.registerOutputs({
      databaseName: this.databaseName,
    });
  }
}
