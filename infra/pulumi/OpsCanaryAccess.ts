import * as pulumi from "@pulumi/pulumi";
import * as cloudflare from "@pulumi/cloudflare";

// Puts the operations canary behind Cloudflare Access, so only a caller with
// the canary service token reaches fn0's ingress for it.
//
// The canary's probes are what the operations console reads to decide whether
// fn0 is up. Left public, anyone could exhaust the canary project's admission
// limit and make the console report fn0 down while it is not. Refusing at the
// edge keeps that traffic off the worker entirely; a secret checked inside the
// app would still spend the admission it is meant to protect.
//
// The hostname itself comes from `forte cloud init` for the canary project,
// which writes its DNS record; this component only guards it.
export interface OpsCanaryAccessArgs {
  zoneId: pulumi.Input<string>;
  hostname: pulumi.Input<string>;
  suffix: pulumi.Input<string>;
}

export class OpsCanaryAccess extends pulumi.ComponentResource {
  public readonly hostname: pulumi.Output<string>;
  public readonly accessClientId: pulumi.Output<string>;
  public readonly accessClientSecret: pulumi.Output<string>;

  constructor(
    name: string,
    args: OpsCanaryAccessArgs,
    opts: pulumi.ComponentResourceOptions,
  ) {
    super("pkg:index:ops-canary-access", name, args, opts);

    const serviceToken = new cloudflare.AccessServiceToken(
      "service-token",
      {
        zoneId: args.zoneId,
        name: pulumi.interpolate`fn0-ops-canary-${args.suffix}`,
        duration: "8760h",
      },
      { parent: this },
    );

    new cloudflare.AccessApplication(
      "application",
      {
        zoneId: args.zoneId,
        name: pulumi.interpolate`fn0 ops canary ${args.suffix}`,
        domain: args.hostname,
        type: "self_hosted",
        sessionDuration: "24h",
        serviceAuth401Redirect: true,
        policies: [
          {
            name: "ops canary service token",
            decision: "non_identity",
            includes: [{ serviceToken: { tokenId: serviceToken.id } }],
          },
        ],
      },
      { parent: this },
    );

    this.hostname = pulumi.output(args.hostname);
    this.accessClientId = pulumi.secret(serviceToken.clientId);
    this.accessClientSecret = pulumi.secret(serviceToken.clientSecret);
    this.registerOutputs({
      hostname: this.hostname,
      accessClientId: this.accessClientId,
      accessClientSecret: this.accessClientSecret,
    });
  }
}
