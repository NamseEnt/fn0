import * as pulumi from "@pulumi/pulumi";
import * as cloudflare from "@pulumi/cloudflare";

// The token every Cloudflare resource in this stack is created with.
//
// The bootstrap credential a person supplies holds one permission — minting
// account tokens — and Cloudflare lets a token grant permissions it does not
// itself hold, so the working credential can be a resource rather than another
// thing somebody made by hand. That is the whole point: a hand-made token has
// no record of what it was for, drifts silently when its permissions are
// edited, and is discovered to be wrong only when a deploy fails against it.
// Declared here, the permission set is reviewable, a change to it is a normal
// `pulumi up`, and a dashboard edit shows up as drift on the next preview.
//
// Every entry below is here because some resource in this stack needs it.
// Adding a Cloudflare resource type means adding its permission here in the
// same change, or the deploy fails with a 403 that names nothing useful.
//
// Token minting is the one thing deliberately absent, and it cannot be added:
// Cloudflare rejects a minted token that carries it ("sub-token is not allowed
// to have permissions to manage other tokens"). The per-bucket R2 tokens this
// stack creates therefore stay on the bootstrap credential.
export interface CloudflareOperatorTokenArgs {
  accountId: pulumi.Input<string>;
  zoneId: pulumi.Input<string>;
  name: pulumi.Input<string>;
}

const ACCOUNT_PERMISSIONS: { name: string; because: string }[] = [
  { name: "Workers R2 Storage Write", because: "R2 buckets and their CORS" },
  { name: "Workers Scripts Write", because: "the bundle-store worker" },
  { name: "Queues Write", because: "the R2 event notification queue" },
  { name: "Access: Apps and Policies Write", because: "the telemetry gate" },
  {
    name: "Access: Service Tokens Write",
    because: "Alloy's ingest credential",
  },
  {
    name: "Cloudflare One Connectors Write",
    because: "the telemetry node's tunnel",
  },
];

const ZONE_PERMISSIONS: { name: string; because: string }[] = [
  { name: "DNS Write", because: "records and the tunnel CNAMEs" },
  {
    name: "Zone Transform Rules Write",
    because: "the X-Scope-OrgID overwrite",
  },
  {
    name: "SSL and Certificates Write",
    because: "origin CA certs and the SaaS fallback origin",
  },
  { name: "Zone Settings Write", because: "zone-level settings" },
  { name: "Cache Settings Write", because: "tiered cache" },
  { name: "Cache Purge", because: "purging on deploy" },
];

// Only the token value is exposed; the caller builds the provider from it. A
// provider cannot be a component's output property — the generated SDK cannot
// name a type from another package's provider — and pushing it out here would
// buy nothing anyway, since the caller is the one attaching it to resources.
export class CloudflareOperatorToken extends pulumi.ComponentResource {
  public readonly value: pulumi.Output<string>;

  constructor(
    name: string,
    args: CloudflareOperatorTokenArgs,
    opts: pulumi.ComponentResourceOptions,
  ) {
    super("pkg:index:cloudflare-operator-token", name, args, opts);

    const { accountId, zoneId } = args;

    const permissionGroups =
      // No `parent` on this invoke, deliberately: the permission-group catalog is
      // Cloudflare's own static list, and reading it through the component's
      // provider would make it depend on the operator token this stack is still
      // in the middle of creating. The default (bootstrap) provider can always
      // read it, so the lookup stays available on a first run.
      cloudflare.getAccountApiTokenPermissionGroupsListOutput({
        accountId,
        maxItems: 1000,
      });

    // Permission group names are not unique across scopes — "Access: Apps and
    // Policies Write" exists at both account and zone level — so a lookup by
    // name alone can silently pick the wrong one and produce a token that
    // fails only on the resource that needed the other scope.
    const resolve = (
      wanted: { name: string; because: string }[],
      scope: "account" | "zone",
    ) =>
      permissionGroups.apply((list) => {
        const groups = list.results ?? [];
        return wanted.map(({ name: wantedName }) => {
          const match = groups.find(
            (group) =>
              group.name === wantedName &&
              (group.scopes ?? []).some((s) =>
                scope === "zone"
                  ? s.includes("zone")
                  : s === "com.cloudflare.api.account",
              ),
          );
          if (!match) {
            throw new Error(
              `Cloudflare has no ${scope}-scoped permission group named "${wantedName}"`,
            );
          }
          return { id: match.id };
        });
      });

    const token = new cloudflare.AccountToken(
      "operator-token",
      {
        accountId,
        name: args.name,
        policies: [
          {
            effect: "allow",
            resources: pulumi.output(accountId).apply((account) =>
              JSON.stringify({
                [`com.cloudflare.api.account.${account}`]: "*",
              }),
            ),
            permissionGroups: resolve(ACCOUNT_PERMISSIONS, "account"),
          },
          {
            effect: "allow",
            resources: pulumi.output(zoneId).apply((zone) =>
              JSON.stringify({
                [`com.cloudflare.api.account.zone.${zone}`]: "*",
              }),
            ),
            permissionGroups: resolve(ZONE_PERMISSIONS, "zone"),
          },
        ],
      },
      { parent: this },
    );

    this.value = pulumi.secret(token.value);
    this.registerOutputs({ value: this.value });
  }
}
