import * as pulumi from "@pulumi/pulumi";
import * as oci from "@pulumi/oci";
import * as random from "@pulumi/random";

export interface OciHeadQuarterVcnArgs {
  region: pulumi.Input<string>;
}

export class OciHeadQuarterVcn extends pulumi.ComponentResource {
  ipv6cidrBlocks: pulumi.Output<string[]>;
  compartmentId: pulumi.Output<string>;
  vcnId: pulumi.Output<string>;

  constructor(
    name: string,
    args: OciHeadQuarterVcnArgs,
    opts: pulumi.ComponentResourceOptions
  ) {
    super("pkg:index:oci-head-quarter", name, args, opts);

    const compartmentSuffix = new random.RandomString(
      "compartment-suffix",
      {
        length: 8,
        special: false,
        upper: false,
      },
      { parent: this }
    ).result;

    const compartment = new oci.identity.Compartment(
      "compartment",
      {
        description: "Compartment for fn0 OCI Head Quarter",
        name: pulumi.interpolate`fn0-hq-${compartmentSuffix}`,
      },
      { parent: this }
    );
    this.compartmentId = compartment.id;

    const vcn = new oci.core.Vcn(
      "vcn",
      {
        compartmentId: compartment.id,
        isIpv6enabled: true,
        isOracleGuaAllocationEnabled: true,
        cidrBlocks: ["10.0.0.0/16"],
      },
      { parent: this }
    );
    this.vcnId = vcn.id;
    this.ipv6cidrBlocks = vcn.ipv6cidrBlocks;
  }
}
