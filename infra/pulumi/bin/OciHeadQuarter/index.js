"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.OciHeadQuarter = void 0;
const pulumi = require("@pulumi/pulumi");
const networking_1 = require("./networking");
const oke_cluster_1 = require("./oke-cluster");
const docker_registry_1 = require("./docker-registry");
const hq_deployment_1 = require("./hq-deployment");
class OciHeadQuarter extends pulumi.ComponentResource {
    constructor(name, args, opts) {
        super("pkg:index:oci-head-quarter", name, args, opts);
        const { suffix, ociRegion, compartmentId, vcnId, docDbUrl, docDbToken, sites, certificate, } = args;
        const { regionalSubnet } = (0, networking_1.createNetworking)(this, {
            compartmentId,
            vcnId,
            ipv6cidrBlocks: args.ipv6cidrBlocks,
        });
        const config = new pulumi.Config("oci");
        const tenancyOcid = config.require("tenancyOcid");
        const userOcid = config.require("userOcid");
        const fingerprint = config.require("fingerprint");
        const privateKey = config.require("privateKey");
        const { k8sProvider, kubeconfig } = (0, oke_cluster_1.createOkeCluster)(this, {
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
        // TODO: Fix grafana helm chart schema for new k8s-monitoring version
        // const { release: grafanaRelease, otlpEndpoint } = hqGrafana(this, {
        //   regionSlug: args.grafanaRegion,
        //   slug: args.grafanaSlug,
        //   k8sProvider: k8sProvider,
        //   suffix,
        // });
        const otlpEndpoint = pulumi.output("");
        const { hqImage } = (0, docker_registry_1.createDockerRegistry)(this, {
            compartmentId,
            suffix,
            region: ociRegion,
        });
        const { service } = (0, hq_deployment_1.deployHqApplication)(this, {
            k8sProvider,
            hqImage,
            otlpEndpoint,
            regionalSubnetId: regionalSubnet.id,
            hqArgs: {
                sites,
                docDb: {
                    url: docDbUrl,
                    token: docDbToken,
                },
                caCertPem: certificate,
                aws: {
                    region: args.awsRegion,
                    wasmBucket: args.wasmBucket,
                    accessKeyId: args.awsAccessKeyId,
                    secretAccessKey: args.awsSecretAccessKey,
                },
                githubClientId: args.githubClientId,
            },
        });
        this.hqServiceIp = service.status.apply((status) => status?.loadBalancer?.ingress?.[0]?.ip ?? "");
    }
}
exports.OciHeadQuarter = OciHeadQuarter;
//# sourceMappingURL=index.js.map