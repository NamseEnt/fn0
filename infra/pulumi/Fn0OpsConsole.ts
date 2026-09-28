import * as pulumi from "@pulumi/pulumi";
import * as cloudflare from "@pulumi/cloudflare";
import { createRequire } from "module";
import * as path from "path";

// The operations console (ops/console): a Cloudflare Worker at its own
// hostname, so it keeps answering when fn0 does not, behind an Access
// application that admits one operator email.
//
// The Worker re-checks the Access JWT itself, so these must hold together:
// the application's `aud`, the account's Access team domain as issuer, and
// the operator email all reach the Worker as bindings from the same
// resources that define them.
//
// workers.dev and preview URLs are switched off: either would reach the
// Worker without passing the Access application on the custom domain.
export interface Fn0OpsConsoleArgs {
  // Only mints this component's Access-administration token; see below.
  tokenMintingApiToken: pulumi.Input<string>;
  accountId: pulumi.Input<string>;
  zoneId: pulumi.Input<string>;
  suffix: pulumi.Input<string>;
  hostname: pulumi.Input<string>;
  operatorEmail: pulumi.Input<string>;
  signyUrl: pulumi.Input<string>;
  signyAccessClientId: pulumi.Input<string>;
  signyAccessClientSecret: pulumi.Input<string>;
  platformTelemetryTenant: pulumi.Input<string>;
  canaryUrl: pulumi.Input<string>;
  canaryAccessClientId: pulumi.Input<string>;
  canaryAccessClientSecret: pulumi.Input<string>;
}

const COMPATIBILITY_DATE = "2026-09-01";
const OPS_CONSOLE_DIR = path.resolve(__dirname, "../../ops/console");
const ACCESS_ADMINISTRATION_PERMISSION =
  "Access: Organizations, Identity Providers, and Groups Write";

interface EsbuildOutputFile {
  text: string;
}

interface Esbuild {
  buildSync(options: Record<string, unknown>): { outputFiles: EsbuildOutputFile[] };
}

// Bundled here rather than read from a build directory, so the deployed
// Worker is always the checked-out source. Needs `npm ci` in ops/console.
function bundleWorker(): string {
  const requireFromConsole = createRequire(
    path.join(OPS_CONSOLE_DIR, "package.json"),
  );
  let esbuild: Esbuild;
  try {
    esbuild = requireFromConsole("esbuild") as Esbuild;
  } catch {
    throw new Error(
      `esbuild is not installed in ${OPS_CONSOLE_DIR}; run \`npm ci\` there first`,
    );
  }
  const result = esbuild.buildSync({
    entryPoints: [path.join(OPS_CONSOLE_DIR, "src/index.ts")],
    bundle: true,
    format: "esm",
    target: "es2022",
    write: false,
  });
  const bundle = result.outputFiles[0];
  if (bundle === undefined) {
    throw new Error("esbuild produced no output for the operations console");
  }
  return bundle.text;
}

export class Fn0OpsConsole extends pulumi.ComponentResource {
  public readonly hostname: pulumi.Output<string>;
  public readonly scriptName: pulumi.Output<string>;
  public readonly accessAudience: pulumi.Output<string>;
  public readonly accessIssuer: pulumi.Output<string>;

  constructor(
    name: string,
    args: Fn0OpsConsoleArgs,
    opts: pulumi.ComponentResourceOptions,
  ) {
    super("pkg:index:fn0-ops-console", name, args, opts);

    // Identity providers and the Access organization need a permission the
    // stack's operator token does not hold. It gets a token of its own with
    // only that permission, rather than widening the credential every other
    // Cloudflare resource here is created with.
    const permissionGroups =
      cloudflare.getAccountApiTokenPermissionGroupsListOutput({
        accountId: args.accountId,
        maxItems: 1000,
      });
    const accessAdministrationPermission = permissionGroups.apply((list) => {
      const group = (list.results ?? []).find(
        (candidate) =>
          candidate.name === ACCESS_ADMINISTRATION_PERMISSION &&
          (candidate.scopes ?? []).includes("com.cloudflare.api.account"),
      );
      if (!group) {
        throw new Error(
          `Cloudflare has no account-scoped permission group named "${ACCESS_ADMINISTRATION_PERMISSION}"`,
        );
      }
      return [{ id: group.id }];
    });
    const tokenMintingProvider = new cloudflare.Provider(
      "token-minting",
      { apiToken: args.tokenMintingApiToken },
      { parent: this },
    );
    const accessAdministrationToken = new cloudflare.AccountToken(
      "access-administration-token",
      {
        accountId: args.accountId,
        name: pulumi.interpolate`fn0-ops-console-access-${args.suffix}`,
        policies: [
          {
            effect: "allow",
            resources: pulumi.output(args.accountId).apply((accountId) =>
              JSON.stringify({ [`com.cloudflare.api.account.${accountId}`]: "*" }),
            ),
            permissionGroups: accessAdministrationPermission,
          },
        ],
      },
      { parent: this, provider: tokenMintingProvider },
    );
    const accessAdministrationProvider = new cloudflare.Provider(
      "access-administration",
      { apiToken: accessAdministrationToken.value },
      { parent: this },
    );

    // The account's one-time PIN login is shared by every Access application
    // on it, so it is looked up rather than owned: destroying this component
    // must not take the login method away from the rest of the account.
    const oneTimePinId = cloudflare
      .getZeroTrustAccessIdentityProvidersOutput(
        { accountId: args.accountId, maxItems: 100 },
        { parent: this, provider: accessAdministrationProvider },
      )
      .apply((list) => {
        const oneTimePin = list.results.find(
          (identityProvider) => identityProvider.type === "onetimepin",
        );
        if (!oneTimePin) {
          throw new Error(
            "the Cloudflare account has no one-time PIN identity provider; add one under Zero Trust > Settings > Authentication",
          );
        }
        return oneTimePin.id;
      });
    const organization = cloudflare.getZeroTrustOrganizationOutput(
      { accountId: args.accountId },
      { parent: this, provider: accessAdministrationProvider },
    );
    this.accessIssuer = pulumi.interpolate`https://${organization.authDomain}`;

    const application = new cloudflare.AccessApplication(
      "application",
      {
        zoneId: args.zoneId,
        name: pulumi.interpolate`fn0 ops console ${args.suffix}`,
        domain: args.hostname,
        type: "self_hosted",
        sessionDuration: "24h",
        allowedIdps: [oneTimePinId],
        autoRedirectToIdentity: true,
        policies: [
          {
            name: "operator",
            decision: "allow",
            includes: [{ email: { email: args.operatorEmail } }],
          },
        ],
      },
      { parent: this },
    );
    this.accessAudience = application.aud;

    const script = new cloudflare.WorkersScript(
      "script",
      {
        accountId: args.accountId,
        scriptName: pulumi.interpolate`fn0-ops-console-${args.suffix}`,
        compatibilityDate: COMPATIBILITY_DATE,
        mainModule: "worker.mjs",
        content: bundleWorker(),
        bindings: [
          { name: "SIGNY_URL", type: "plain_text", text: args.signyUrl },
          {
            name: "SIGNY_ACCESS_CLIENT_ID",
            type: "secret_text",
            text: args.signyAccessClientId,
          },
          {
            name: "SIGNY_ACCESS_CLIENT_SECRET",
            type: "secret_text",
            text: args.signyAccessClientSecret,
          },
          {
            name: "PLATFORM_TELEMETRY_TENANT",
            type: "plain_text",
            text: args.platformTelemetryTenant,
          },
          { name: "CANARY_URL", type: "plain_text", text: args.canaryUrl },
          {
            name: "CANARY_ACCESS_CLIENT_ID",
            type: "secret_text",
            text: args.canaryAccessClientId,
          },
          {
            name: "CANARY_ACCESS_CLIENT_SECRET",
            type: "secret_text",
            text: args.canaryAccessClientSecret,
          },
          { name: "ACCESS_ISSUER", type: "plain_text", text: this.accessIssuer },
          { name: "ACCESS_AUD", type: "plain_text", text: application.aud },
          {
            name: "OPS_ADMIN_EMAIL",
            type: "secret_text",
            text: args.operatorEmail,
          },
        ],
      },
      { parent: this },
    );

    new cloudflare.WorkersScriptSubdomain(
      "subdomain",
      {
        accountId: args.accountId,
        scriptName: script.scriptName,
        enabled: false,
        previewsEnabled: false,
      },
      { parent: this },
    );

    new cloudflare.WorkersCustomDomain(
      "custom-domain",
      {
        accountId: args.accountId,
        zoneId: args.zoneId,
        hostname: args.hostname,
        service: script.scriptName,
      },
      { parent: this, dependsOn: [application] },
    );

    this.hostname = pulumi.output(args.hostname);
    this.scriptName = script.scriptName;
    this.registerOutputs({
      hostname: this.hostname,
      scriptName: this.scriptName,
      accessAudience: this.accessAudience,
      accessIssuer: this.accessIssuer,
    });
  }
}
