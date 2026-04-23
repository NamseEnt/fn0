"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.OciHeadQuarter = void 0;
const pulumi = require("@pulumi/pulumi");
const grafana_1 = require("./grafana");
const networking_1 = require("./networking");
const oke_cluster_1 = require("./oke-cluster");
const docker_registry_1 = require("./docker-registry");
const hq_deployment_1 = require("./hq-deployment");
class OciHeadQuarter extends pulumi.ComponentResource {
    constructor(name, args, opts) {
        super("pkg:index:oci-head-quarter", name, args, opts);
        const { suffix, ociRegion, compartmentId, vcnId, docDbUrl, docDbToken, sites, } = args;
        const { regionalSubnet, podSubnet } = (0, networking_1.createNetworking)(this, {
            compartmentId,
            vcnId,
            ipv6cidrBlocks: args.ipv6cidrBlocks,
        });
        const config = new pulumi.Config("oci");
        const tenancyOcid = config.require("tenancyOcid");
        const userOcid = config.require("userOcid");
        const fingerprint = config.require("fingerprint");
        const privateKey = config.require("privateKey");
        const { k8sProvider, kubeconfig, nodePool } = (0, oke_cluster_1.createOkeCluster)(this, {
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
        const { otlpEndpoint, workerOtlpEndpoint, workerOtlpBasicAuth } = (0, grafana_1.hqGrafana)(this, {
            regionSlug: args.grafanaRegion,
            slug: args.grafanaSlug,
            k8sProvider: k8sProvider,
            suffix,
        });
        const { hqImage } = (0, docker_registry_1.createDockerRegistry)(this, {
            compartmentId,
            suffix,
            region: ociRegion,
            sccacheBucket: args.sccacheBucket,
            sccacheRegion: args.sccacheRegion,
            sccacheEndpoint: args.sccacheEndpoint,
            sccacheAccessKeyId: args.sccacheAccessKeyId,
            sccacheSecretAccessKey: args.sccacheSecretAccessKey,
        });
        (0, hq_deployment_1.deployHqApplication)(this, {
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
                forteDb: args.forteDb,
                forteR2: args.forteR2,
                aws: {
                    region: args.awsRegion,
                    wasmBucket: args.wasmBucket,
                    accessKeyId: args.awsAccessKeyId,
                    secretAccessKey: args.awsSecretAccessKey,
                },
                envEncryptionKeyBase64: args.envEncryptionKeyBase64,
                cwasmBucket: args.cwasmBucket,
                selfDns: {
                    hostname: args.selfDnsHostname,
                    cloudflareZoneId: args.selfDnsCloudflareZoneId,
                    cloudflareApiToken: args.selfDnsCloudflareApiToken,
                },
            },
        });
    }
}
exports.OciHeadQuarter = OciHeadQuarter;
//# sourceMappingURL=index.js.map