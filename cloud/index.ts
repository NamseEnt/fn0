import * as fn0 from "@pulumi/fn0";
import * as pulumi from "@pulumi/pulumi";

const config = new pulumi.Config();

const accountId = config.require("cloudflareAccountId");
const zoneId = config.require("cloudflareZoneId");
const domain = config.require("domain");

const suffix = new fn0.Suffix("suffix").result;

const dns = new fn0.CloudflareDns("cloudflare-dns", {
  suffix,
  accountId,
  zoneId,
  domain,
});

const docDb = new fn0.TursoDocDb("doc-db", {
  organizationSlug: config.require("tursoOrganizationSlug"),
  location: config.require("tursoLocation"),
});

const ociHeadQuarterVcn = new fn0.OciHeadQuarterVcn("oci-head-quarter-vcn", {
  suffix,
  region: config.require("ociHeadQuarterRegion"),
});

const ociComputeWorker = new fn0.OciComputeWorker("oci-compute-worker", {
  region: config.require("ociComputeWorkerRegion"),
  hqIpv6CidrBlocks: ociHeadQuarterVcn.ipv6cidrBlocks,
});

const ociHeadQuarter = new fn0.OciHeadQuarter("oci-head-quarter", {
  suffix,
  ociRegion: config.require("ociHeadQuarterRegion"),
  compartmentId: ociHeadQuarterVcn.compartmentId,
  vcnId: ociHeadQuarterVcn.vcnId,
  ipv6cidrBlocks: ociHeadQuarterVcn.ipv6cidrBlocks,
  grafanaSlug: config.require("grafanaSlug"),
  grafanaRegion: config.require("grafanaRegion"),
  docDbUrl: docDb.url,
  docDbToken: docDb.token,
  certificate: dns.certificate,
  sites: [],
});

export const kubeconfig = pulumi.secret(ociHeadQuarter.kubeconfig);

// new fn0.B2CloudflareStaticCdn("b2CloudflareStaticCdn", {
//   zoneId,
// });
