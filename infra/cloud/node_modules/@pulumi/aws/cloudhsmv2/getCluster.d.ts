import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Use this data source to get information about a CloudHSM v2 cluster
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const cluster = aws.cloudhsmv2.getCluster({
 *     clusterId: "cluster-testclusterid",
 * });
 * ```
 */
export declare function getCluster(args: GetClusterArgs, opts?: pulumi.InvokeOptions): Promise<GetClusterResult>;
/**
 * A collection of arguments for invoking getCluster.
 */
export interface GetClusterArgs {
    /**
     * ID of Cloud HSM v2 cluster.
     */
    clusterId: string;
    /**
     * State of the cluster to be found.
     */
    clusterState?: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getCluster.
 */
export interface GetClusterResult {
    /**
     * The list of cluster certificates.
     */
    readonly clusterCertificates: outputs.cloudhsmv2.GetClusterClusterCertificate[];
    readonly clusterId: string;
    readonly clusterState: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly region: string;
    /**
     * ID of the security group associated with the CloudHSM cluster.
     */
    readonly securityGroupId: string;
    /**
     * IDs of subnets in which cluster operates.
     */
    readonly subnetIds: string[];
    /**
     * ID of the VPC that the CloudHSM cluster resides in.
     */
    readonly vpcId: string;
}
/**
 * Use this data source to get information about a CloudHSM v2 cluster
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const cluster = aws.cloudhsmv2.getCluster({
 *     clusterId: "cluster-testclusterid",
 * });
 * ```
 */
export declare function getClusterOutput(args: GetClusterOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetClusterResult>;
/**
 * A collection of arguments for invoking getCluster.
 */
export interface GetClusterOutputArgs {
    /**
     * ID of Cloud HSM v2 cluster.
     */
    clusterId: pulumi.Input<string>;
    /**
     * State of the cluster to be found.
     */
    clusterState?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
