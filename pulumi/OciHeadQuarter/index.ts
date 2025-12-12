import * as pulumi from "@pulumi/pulumi";
import * as random from "@pulumi/random";
import { OciWorkerInfraEnvs } from "../OciComputeWorker";
import { hqGrafana } from "./grafana";
import { createNetworking } from "./networking";
import { createOkeCluster } from "./oke-cluster";
import { createDockerRegistry } from "./docker-registry";
import { deployHqApplication } from "./hq-deployment";
import { deployK8sDashboard } from "./k8s-dashboard";

export interface OciHeadQuarterArgs {
  region: pulumi.Input<string>;
  compartmentId: pulumi.Input<string>;
  vcnId: pulumi.Input<string>;
  ipv6cidrBlocks: pulumi.Input<string[]>;
  ociWorkerInfraEnvs: pulumi.Input<OciWorkerInfraEnvs>;
  grafanaRegion: pulumi.Input<string>;
  grafanaSlug: pulumi.Input<string>;
}

export class OciHeadQuarter extends pulumi.ComponentResource {
  kubeconfig: pulumi.Output<string>;
  constructor(
    name: string,
    args: OciHeadQuarterArgs,
    opts: pulumi.ComponentResourceOptions
  ) {
    const resourceInputs = { ...args };
    delete (resourceInputs as any).ociWorkerInfraEnvs;
    super("pkg:index:oci-head-quarter", name, resourceInputs, opts);

    const { region, compartmentId, vcnId, ociWorkerInfraEnvs } = args;

    const nameSuffix8 = new random.RandomString(
      "name-suffix-8",
      {
        length: 8,
        special: false,
        upper: false,
      },
      { parent: this }
    ).result;

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

    const { k8sProvider, kubeconfig } = createOkeCluster(this, {
      compartmentId,
      vcnId,
      regionalSubnetId: regionalSubnet.id,
      nameSuffix: nameSuffix8,
      region,
      tenancyOcid,
      userOcid,
      fingerprint,
      privateKey,
    });
    this.kubeconfig = kubeconfig;

    const { release: grafanaRelease, otlpEndpoint } = hqGrafana(this, {
      regionSlug: args.grafanaRegion,
      slug: args.grafanaSlug,
      k8sProvider: k8sProvider,
      suffix: nameSuffix8,
    });

    deployK8sDashboard(this, {
      k8sProvider,
      dependsOn: [grafanaRelease],
    });

    const { hqImage } = createDockerRegistry(this, {
      compartmentId,
      nameSuffix: nameSuffix8,
      region,
    });

    deployHqApplication(this, {
      k8sProvider,
      hqImage,
      ociWorkerInfraEnvs,
      otlpEndpoint,
    });
  }
}
