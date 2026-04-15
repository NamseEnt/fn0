import * as pulumi from "@pulumi/pulumi";
import * as oci from "@pulumi/oci";
import { hqGrafana } from "./grafana";
import { createNetworking } from "./networking";
import { createOkeCluster } from "./oke-cluster";
import { createDockerRegistry } from "./docker-registry";
import { deployK8sDashboard } from "./k8s-dashboard";
import { deployHqApplication } from "./hq-deployment";
import { SiteArgs, DnsProviderArg } from "../hqArgs.schema";

export interface OciHeadQuarterArgs {
  suffix: pulumi.Input<string>;
  ociRegion: pulumi.Input<string>;
  compartmentId: pulumi.Input<string>;
  vcnId: pulumi.Input<string>;
  ipv6cidrBlocks: pulumi.Input<string[]>;
  grafanaRegion: pulumi.Input<string>;
  grafanaSlug: pulumi.Input<string>;
  docDbUrl: pulumi.Input<string>;
  docDbToken: pulumi.Input<string>;
  sites: pulumi.Input<SiteArgs[]>;
  awsRegion: pulumi.Input<string>;
  wasmBucket: pulumi.Input<string>;
  cwasmBucket: pulumi.Input<string>;
  awsAccessKeyId: pulumi.Input<string>;
  awsSecretAccessKey: pulumi.Input<string>;
  selfDnsHostname: pulumi.Input<string>;
  selfDnsCloudflareZoneId: pulumi.Input<string>;
  selfDnsCloudflareApiToken: pulumi.Input<string>;
  dnsProvider: pulumi.Input<DnsProviderArg>;
}

export class OciHeadQuarter extends pulumi.ComponentResource {
  kubeconfig: pulumi.Output<string>;
  constructor(
    name: string,
    args: OciHeadQuarterArgs,
    opts: pulumi.ComponentResourceOptions
  ) {
    super("pkg:index:oci-head-quarter", name, args, opts);

    const {
      suffix,
      ociRegion,
      compartmentId,
      vcnId,
      docDbUrl,
      docDbToken,
      sites,
    } = args;

    const { regionalSubnet } = createNetworking(this, {
      compartmentId,
      vcnId,
      ipv6cidrBlocks: args.ipv6cidrBlocks,
    });

    const config = new pulumi.Config("oci");
    const tenancyOcid = config.require("tenancyOcid");
    const userOcid = config.require("userOcid");
    const fingerprint = config.require("fingerprint");
    const privateKey = config.require("privateKey");

    const { k8sProvider, kubeconfig, nodePool } = createOkeCluster(this, {
      compartmentId,
      vcnId,
      regionalSubnetId: regionalSubnet.id,
      suffix,
      region: ociRegion,
      tenancyOcid,
      userOcid,
      fingerprint,
      privateKey,
    });
    this.kubeconfig = kubeconfig;

    const { otlpEndpoint, workerOtlpEndpoint, workerOtlpBasicAuth } =
      hqGrafana(this, {
        regionSlug: args.grafanaRegion,
        slug: args.grafanaSlug,
        k8sProvider: k8sProvider,
        suffix,
      });

    const { hqImage } = createDockerRegistry(this, {
      compartmentId,
      suffix,
      region: ociRegion,
    });

    deployHqApplication(this, {
      k8sProvider,
      hqImage,
      otlpEndpoint,
      workerOtlpEndpoint,
      workerOtlpBasicAuth,
      hqArgs: {
        sites,
        dnsProvider: args.dnsProvider,
        docDb: {
          url: docDbUrl,
          token: docDbToken,
        },
        aws: {
          region: args.awsRegion,
          wasmBucket: args.wasmBucket,
          cwasmBucket: args.cwasmBucket,
          accessKeyId: args.awsAccessKeyId,
          secretAccessKey: args.awsSecretAccessKey,
        },
        selfDns: {
          hostname: args.selfDnsHostname,
          cloudflareZoneId: args.selfDnsCloudflareZoneId,
          cloudflareApiToken: args.selfDnsCloudflareApiToken,
        },
      },
    });
  }
}
