import * as pulumi from "@pulumi/pulumi";
import * as oci from "@pulumi/oci";
import * as tls from "@pulumi/tls";

export interface OciHeadQuarterArgs {
  region: pulumi.Input<string>;
}

export class OciHeadQuarter extends pulumi.ComponentResource {
  constructor(
    name: string,
    args: OciHeadQuarterArgs,
    opts: pulumi.ComponentResourceOptions
  ) {
    super("pkg:index:oci-head-quarter", name, args, opts);

    const compartment = new oci.identity.Compartment("compartment", {
      description: "Compartment for fn0 OCI Head Quarter",
      name: `fn0-head-quater-${name}`,
    });

    const queue = new oci.queue.Queue("build-replication-queue", {
      compartmentId: compartment.id,
      displayName: "build-replication-queue",
      visibilityInSeconds: 5,
      deadLetterQueueDeliveryCount: 5,
    });
  }
}
