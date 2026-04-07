import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Retrieve information about an EMR Containers (EMR on EKS) Virtual Cluster.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.emrcontainers.getVirtualCluster({
 *     virtualClusterId: "example id",
 * });
 * export const name = example.then(example => example.name);
 * export const arn = example.then(example => example.arn);
 * ```
 */
export declare function getVirtualCluster(args: GetVirtualClusterArgs, opts?: pulumi.InvokeOptions): Promise<GetVirtualClusterResult>;
/**
 * A collection of arguments for invoking getVirtualCluster.
 */
export interface GetVirtualClusterArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * Key-value mapping of resource tags.
     */
    tags?: {
        [key: string]: string;
    };
    /**
     * ID of the cluster.
     */
    virtualClusterId: string;
}
/**
 * A collection of values returned by getVirtualCluster.
 */
export interface GetVirtualClusterResult {
    /**
     * ARN of the cluster.
     */
    readonly arn: string;
    /**
     * Nested attribute containing information about the underlying container provider (EKS cluster) for your EMR Containers cluster.
     */
    readonly containerProviders: outputs.emrcontainers.GetVirtualClusterContainerProvider[];
    /**
     * Unix epoch time stamp in seconds for when the cluster was created.
     */
    readonly createdAt: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * Name of the cluster.
     */
    readonly name: string;
    readonly region: string;
    /**
     * Status of the EKS cluster. One of `RUNNING`, `TERMINATING`, `TERMINATED`, `ARRESTED`.
     */
    readonly state: string;
    /**
     * Key-value mapping of resource tags.
     */
    readonly tags: {
        [key: string]: string;
    };
    readonly virtualClusterId: string;
}
/**
 * Retrieve information about an EMR Containers (EMR on EKS) Virtual Cluster.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.emrcontainers.getVirtualCluster({
 *     virtualClusterId: "example id",
 * });
 * export const name = example.then(example => example.name);
 * export const arn = example.then(example => example.arn);
 * ```
 */
export declare function getVirtualClusterOutput(args: GetVirtualClusterOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetVirtualClusterResult>;
/**
 * A collection of arguments for invoking getVirtualCluster.
 */
export interface GetVirtualClusterOutputArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value mapping of resource tags.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * ID of the cluster.
     */
    virtualClusterId: pulumi.Input<string>;
}
