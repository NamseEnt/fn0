import * as fn0 from "@pulumi/fn0";
import * as pulumi from "@pulumi/pulumi";
import * as cloudflare from "@pulumi/cloudflare";

const config = new pulumi.Config();

const accountId = config.require("cloudflareAccountId");
const zoneId = config.require("cloudflareZoneId");
const domain = config.require("domain");
const awsRegion = "ap-northeast-1";

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

const wasmS3 = new fn0.AwsWasmS3("wasm-s3", {
  region: awsRegion,
});

const ociHeadQuarterVcn = new fn0.OciHeadQuarterVcn("oci-head-quarter-vcn", {
  suffix,
  region: config.require("ociHeadQuarterRegion"),
});

const ociComputeWorker = new fn0.OciComputeWorker("oci-compute-worker", {
  region: config.require("ociComputeWorkerRegion"),
  hqIpv6CidrBlocks: ociHeadQuarterVcn.ipv6cidrBlocks,
});

const cwasmS3Pusher = new fn0.CwasmS3Pusher("cwasm-s3-pusher", {
  awsS3Region: awsRegion,
  zoneId,
  cloudflareApiToken: dns.dnsApiToken,
  targetBuckets: [
    {
      endpoint: ociComputeWorker.cwasmBucket.endpoint,
      region: ociComputeWorker.cwasmBucket.region,
      bucketName: ociComputeWorker.cwasmBucket.bucketName,
      accessKeyId: ociComputeWorker.cwasmBucket.accessKeyId,
      secretAccessKey: ociComputeWorker.cwasmBucket.secretAccessKey,
    },
  ],
});

new fn0.AwsLambdaCwasmCompiler("cwasm-compiler", {
  region: awsRegion,
  wasmBucket: wasmS3.bucket,
  cWasmBucket: cwasmS3Pusher.cwasmZstBucket,
  queueArn: wasmS3.queueArn,
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
  awsRegion: awsRegion,
  wasmBucket: wasmS3.bucket,
  githubClientId: config.require("githubClientId"),
  sites: [
    {
      hostProvider: {
        ociContainerInstance: {
          privateKeyBase64: ociComputeWorker.infraEnvs.OCI_PRIVATE_KEY_BASE64,
          userId: ociComputeWorker.infraEnvs.OCI_USER_ID,
          fingerprint: ociComputeWorker.infraEnvs.OCI_FINGERPRINT,
          tenancyId: ociComputeWorker.infraEnvs.OCI_TENANCY_ID,
          region: config.require("ociComputeWorkerRegion"),
          compartmentId: ociComputeWorker.compartmentId,
          availabilityDomain: ociComputeWorker.infraEnvs.OCI_AVAILABILITY_DOMAIN,
          shape: "CI.Standard.A1.Flex",
          ocpus: 1,
          physicsCpuCores: 1,
          memoryInGbs: 6,
          subnetId: ociComputeWorker.subnetId,
          image: ociComputeWorker.workerImageUrl,
          envs: {
            CWASM_BUCKET: ociComputeWorker.cwasmBucket.bucketName,
            S3_ENDPOINT: ociComputeWorker.cwasmBucket.endpoint,
            S3_REGION: ociComputeWorker.cwasmBucket.region,
            AWS_ACCESS_KEY_ID: ociComputeWorker.cwasmBucket.accessKeyId,
            AWS_SECRET_ACCESS_KEY: ociComputeWorker.cwasmBucket.secretAccessKey,
          },
        },
      },
      dnsProvider: {
        cloudflare: {
          zoneId,
          asteriskDomain: `*.${domain}`,
          apiToken: dns.dnsApiToken,
        },
      },
    },
  ],
});

export const kubeconfig = pulumi.secret(ociHeadQuarter.kubeconfig);

new cloudflare.DnsRecord("hq-dns-record", {
  zoneId,
  name: `fn0-hq.${domain}`,
  type: "A",
  content: ociHeadQuarter.hqServiceIp,
  ttl: 60,
  proxied: false,
});
