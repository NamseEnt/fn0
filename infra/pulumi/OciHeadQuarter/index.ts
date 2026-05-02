import * as pulumi from "@pulumi/pulumi";
import * as oci from "@pulumi/oci";
import { hqGrafana } from "./grafana";
import { createNetworking } from "./networking";
import { createOkeCluster } from "./oke-cluster";
import { createDockerRegistry } from "./docker-registry";
import { deployK8sDashboard } from "./k8s-dashboard";
import { deployHqApplication } from "./hq-deployment";
import {
  SiteArgs,
  DnsProviderArg,
  CwasmBucketArgs,
  ForteDbArgs,
  ForteR2Args,
  CloudflareSaasArgs,
} from "../hqArgs.schema";

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
  forteDb: pulumi.Input<ForteDbArgs>;
  forteR2: pulumi.Input<ForteR2Args>;
  sites: pulumi.Input<SiteArgs[]>;
  awsRegion: pulumi.Input<string>;
  wasmBucket: pulumi.Input<string>;
  cwasmBucket: pulumi.Input<CwasmBucketArgs>;
  awsAccessKeyId: pulumi.Input<string>;
  awsSecretAccessKey: pulumi.Input<string>;
  envEncryptionKeyBase64: pulumi.Input<string>;
  adminSigningKeyBase64: pulumi.Input<string>;
  sccacheBucket: pulumi.Input<string>;
  sccacheRegion: pulumi.Input<string>;
  sccacheEndpoint: pulumi.Input<string>;
  sccacheAccessKeyId: pulumi.Input<string>;
  sccacheSecretAccessKey: pulumi.Input<string>;
  selfDnsHostname: pulumi.Input<string>;
  selfDnsCloudflareZoneId: pulumi.Input<string>;
  selfDnsCloudflareApiToken: pulumi.Input<string>;
  dnsProvider: pulumi.Input<DnsProviderArg>;
  cloudflareSaas: pulumi.Input<CloudflareSaasArgs>;
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

    const { regionalSubnet, podSubnet } = createNetworking(this, {
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
      podSubnetId: podSubnet.id,
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
      sccacheBucket: args.sccacheBucket,
      sccacheRegion: args.sccacheRegion,
      sccacheEndpoint: args.sccacheEndpoint,
      sccacheAccessKeyId: args.sccacheAccessKeyId,
      sccacheSecretAccessKey: args.sccacheSecretAccessKey,
    });

    const augmentedSites = pulumi
      .all([sites, workerOtlpEndpoint, workerOtlpBasicAuth])
      .apply(([siteList, endpoint, basicAuth]) => {
        const u = new URL(endpoint);
        const target_host = u.host;
        const target_path_prefix = u.pathname.replace(/\/$/, "");
        const auth = basicAuth;
        return siteList.map((site) => {
          const hp = site.hostProvider as any;
          if (!hp.ociComputeVm) return site;
          return {
            ...site,
            hostProvider: {
              ociComputeVm: {
                ...hp.ociComputeVm,
                envs: {
                  ...hp.ociComputeVm.envs,
                  FN0_OTLP_TARGET_HOST: target_host,
                  FN0_OTLP_TARGET_PATH_PREFIX: target_path_prefix,
                  FN0_OTLP_AUTH: auth,
                },
              },
            },
          };
        });
      });

    deployHqApplication(this, {
      k8sProvider,
      hqImage,
      otlpEndpoint,
      workerOtlpEndpoint,
      workerOtlpBasicAuth,
      hqArgs: {
        sites: augmentedSites,
        dnsProvider: args.dnsProvider,
        docDb: {
          url: docDbUrl,
          token: docDbToken,
        },
        forteDb: args.forteDb,
        forteR2: args.forteR2,
        aws: {
          region: args.awsRegion,
          wasmBucket: args.wasmBucket,
          accessKeyId: args.awsAccessKeyId,
          secretAccessKey: args.awsSecretAccessKey,
        },
        envEncryptionKeyBase64: args.envEncryptionKeyBase64,
        adminSigningKeyBase64: args.adminSigningKeyBase64,
        cwasmBucket: args.cwasmBucket,
        selfDns: {
          hostname: args.selfDnsHostname,
          cloudflareZoneId: args.selfDnsCloudflareZoneId,
          cloudflareApiToken: args.selfDnsCloudflareApiToken,
        },
        cloudflareSaas: args.cloudflareSaas,
      },
    });
  }
}
