import * as pulumi from "@pulumi/pulumi";
import * as cloudflare from "@pulumi/cloudflare";
import { envCloudflareAccountId, envDomain } from "./env";

const accountId = envCloudflareAccountId();
const domain = envDomain();

const zone = new cloudflare.Zone("zone", {
  account: { id: accountId },
  name: domain,
  paused: false,
  type: "full",
});
