import * as cloudflare from "@pulumi/cloudflare";
import {
  envCloudflareAccountId,
  envDomain,
  envAwsWatchdogRegion,
  envOciComputeWorkerRegion,
} from "./env";
import * as fn0 from "@pulumi/fn0";

const accountId = envCloudflareAccountId();
const domain = envDomain();

const zone = new cloudflare.Zone("zone", {
  account: { id: accountId },
  name: domain,
  paused: false,
  type: "full",
});

const awsWatchdogVpc = new fn0.AwsWatchdogVpc("awsWatchdogVpc", {
  region: envAwsWatchdogRegion(),
});

const apiTokenPermissionGroups = cloudflare.getApiTokenPermissionGroupsList();

const cloudflareApiToken = new cloudflare.ApiToken("cloudflareApiToken", {
  name: "fn0Cloud",
  policies: [
    {
      effect: "allow",
      resources: {
        "com.cloudflare.api.account.zone": zone.id,
      },
      permissionGroups: apiTokenPermissionGroups.then((x) => {
        const groups = x.results.filter((x) => {
          ["DNS Read", "DNS Write"].includes(x.name);
        });
        if (!groups.length) {
          console.error("No permission groups found", x);
          throw new Error("No permission groups found");
        }
        return groups;
      }),
    },
  ],
  condition: {
    requestIp: {
      ins: [awsWatchdogVpc.ipv6CiderBlock],
    },
  },
});

const ociComputeWorker = new fn0.OciComputeWorker("ociComputeWorker", {
  region: envOciComputeWorkerRegion(),
  watchdogIpv6CiderBlock: awsWatchdogVpc.ipv6CiderBlock,
});

const awsWatchdog = new fn0.AwsWatchdog("awsWatchdog", {
  domain,
  region: envAwsWatchdogRegion(),
  vpcId: awsWatchdogVpc.vpc.id,
  subnetId: awsWatchdogVpc.subnet.id,
  securityGroupId: awsWatchdogVpc.securityGroup.id,
  maxGracefulShutdownWaitSecs: 300,
  maxHealthyCheckRetries: 5,
  maxStartTimeoutSecs: 180,
  maxStartingCount: 1,
  ociWorkerInfraEnvs: ociComputeWorker.infraEnvs,
  cloudflareEnvs: {
    CLOUDFLARE_API_TOKEN: cloudflareApiToken.value,
    CLOUDFLARE_ASTERISK_DOMAIN: envDomain(),
    CLOUDFLARE_ZONE_ID: zone.id,
  },
});
