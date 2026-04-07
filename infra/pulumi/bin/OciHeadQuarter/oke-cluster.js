"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.createOkeCluster = createOkeCluster;
const pulumi = require("@pulumi/pulumi");
const oci = require("@pulumi/oci");
const k8s = require("@pulumi/kubernetes");
const yaml = require("js-yaml");
function createOkeCluster(parent, { compartmentId, vcnId, regionalSubnetId, suffix, region, tenancyOcid, userOcid, fingerprint, privateKey, }) {
    const ad1 = pulumi.all([compartmentId]).apply(([compartmentId]) => oci.identity
        .getAvailabilityDomain({
        adNumber: 1,
        compartmentId,
    })
        .then((x) => x.name));
    const clusterOptions = pulumi
        .all([compartmentId])
        .apply(([compartmentId]) => {
        return oci.containerengine.getClusterOption({
            clusterOptionId: "all",
            compartmentId,
        });
    });
    const kubernetesVersion = clusterOptions.apply((options) => {
        return options.kubernetesVersions.sort().pop();
    });
    const cluster = new oci.containerengine.Cluster("cluster", {
        compartmentId,
        kubernetesVersion,
        clusterPodNetworkOptions: [
            {
                cniType: "OCI_VCN_IP_NATIVE",
            },
        ],
        endpointConfig: {
            isPublicIpEnabled: true,
            subnetId: regionalSubnetId,
        },
        options: {
            ipFamilies: ["IPv4", "IPv6"],
        },
        vcnId,
        name: pulumi.interpolate `fn0-${suffix}`,
    }, { parent, deleteBeforeReplace: true });
    const poolOptions = pulumi
        .all([compartmentId, kubernetesVersion])
        .apply(([compartmentId, kubernetesVersion]) => {
        return oci.containerengine.getNodePoolOption({
            compartmentId,
            nodePoolOptionId: "all",
            nodePoolK8sVersion: kubernetesVersion,
        });
    });
    const imageId = poolOptions.apply((options) => options.sources
        .filter((x) => x.sourceName.includes("-aarch64-"))
        .sort((a, b) => a.sourceName.localeCompare(b.sourceName))
        .pop().imageId);
    const nodePool = new oci.containerengine.NodePool("node-pool", {
        compartmentId,
        clusterId: cluster.id,
        kubernetesVersion,
        name: pulumi.interpolate `fn0-nodepool-${suffix}`,
        nodeShape: "VM.Standard.A1.Flex",
        nodeShapeConfig: {
            ocpus: 1,
            memoryInGbs: 6,
        },
        nodeConfigDetails: {
            size: 1,
            placementConfigs: [
                {
                    availabilityDomain: ad1,
                    subnetId: regionalSubnetId,
                },
            ],
            nodePoolPodNetworkOptionDetails: {
                cniType: "OCI_VCN_IP_NATIVE",
                podSubnetIds: [regionalSubnetId],
            },
        },
        nodeSourceDetails: {
            imageId,
            sourceType: "IMAGE",
        },
    }, { parent, deleteBeforeReplace: true });
    const kubeconfig = pulumi
        .all([cluster.id, region])
        .apply(([clusterId, region]) => oci.containerengine
        .getClusterKubeConfig({
        clusterId,
    })
        .then((kc) => {
        const content = yaml.load(kc.content);
        const { env } = content.users[0].user.exec;
        env.push({ name: "OCI_CLI_AUTH", value: "api_key" }, { name: "OCI_CLI_REGION", value: region }, { name: "OCI_CLI_USER", value: userOcid }, { name: "OCI_CLI_TENANCY", value: tenancyOcid }, { name: "OCI_CLI_FINGERPRINT", value: fingerprint }, { name: "OCI_CLI_KEY_CONTENT", value: privateKey });
        const result = yaml.dump(content);
        return result;
    }));
    const k8sProvider = new k8s.Provider("oke-k8s-provider", {
        kubeconfig,
    }, { parent, dependsOn: [nodePool] });
    return {
        cluster,
        nodePool,
        kubernetesVersion,
        kubeconfig,
        k8sProvider,
    };
}
//# sourceMappingURL=oke-cluster.js.map