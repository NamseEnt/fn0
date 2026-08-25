import * as pulumi from "@pulumi/pulumi";
import * as cloudflare from "@pulumi/cloudflare";

// The log/trace engine behind the telemetry hostname has no TLS and no
// authentication of its own: it reads `X-Scope-OrgID` and believes it, and it
// binds loopback so that only the tunnel can reach it. Everything that makes
// that safe is here, at the Cloudflare edge, which is why this is a component
// rather than a few resources inline — the pieces are only correct together.
//
// Two jobs:
//   1. Authenticate the writer. Only Alloy holds the service token, so only
//      Alloy can push logs and traces. Requests without it never leave the
//      edge.
//   2. Overwrite the tenant header. A caller that sets its own `X-Scope-OrgID`
//      must not have it survive, so the rule runs in the late-transform phase
//      — after the firewall, immediately before the request goes to the
//      origin — and uses `set` rather than `add`, because appending would send
//      the engine two values and let it pick.
export interface TelemetryEdgeGateArgs {
  accountId: pulumi.Input<string>;
  zoneId: pulumi.Input<string>;
  ingestHostname: pulumi.Input<string>;
  tenant: pulumi.Input<string>;
}

export class TelemetryEdgeGate extends pulumi.ComponentResource {
  public readonly serviceTokenClientId: pulumi.Output<string>;
  public readonly serviceTokenClientSecret: pulumi.Output<string>;

  constructor(
    name: string,
    args: TelemetryEdgeGateArgs,
    opts: pulumi.ComponentResourceOptions,
  ) {
    super("pkg:index:telemetry-edge-gate", name, args, opts);

    const { accountId, zoneId, ingestHostname, tenant } = args;

    const serviceToken = new cloudflare.ZeroTrustAccessServiceToken(
      "ingest-service-token",
      {
        accountId,
        name: pulumi.interpolate`fn0-telemetry-ingest-${ingestHostname}`,
      },
      { parent: this },
    );

    // serviceAuth401Redirect: an unauthenticated request has to fail as a 401.
    // The default is a redirect to the login page, which an agent would follow
    // and store as a successful push of an HTML document.
    new cloudflare.ZeroTrustAccessApplication(
      "ingest-application",
      {
        accountId,
        name: pulumi.interpolate`fn0 telemetry ingest (${ingestHostname})`,
        type: "self_hosted",
        domain: ingestHostname,
        serviceAuth401Redirect: true,
        appLauncherVisible: false,
        policies: [
          {
            name: "alloy service token",
            decision: "non_identity",
            includes: [{ serviceToken: { tokenId: serviceToken.id } }],
          },
        ],
      },
      { parent: this },
    );

    new cloudflare.Ruleset(
      "tenant-header",
      {
        zoneId,
        name: "fn0 telemetry tenant header",
        description:
          "Overwrite X-Scope-OrgID on the telemetry ingest hostname so a caller cannot choose its own tenant",
        kind: "zone",
        phase: "http_request_late_transform",
        rules: [
          {
            action: "rewrite",
            description: "stamp the fn0 tenant",
            enabled: true,
            expression: pulumi.interpolate`http.host eq "${ingestHostname}"`,
            actionParameters: {
              headers: {
                "X-Scope-OrgID": {
                  operation: "set",
                  value: tenant,
                },
              },
            },
          },
        ],
      },
      { parent: this },
    );

    this.serviceTokenClientId = serviceToken.clientId;
    this.serviceTokenClientSecret = pulumi.secret(serviceToken.clientSecret);
  }
}
