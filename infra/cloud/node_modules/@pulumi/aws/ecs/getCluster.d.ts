import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * The ECS Cluster data source allows access to details of a specific
 * cluster within an AWS ECS service.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const ecs_mongo = aws.ecs.getCluster({
 *     clusterName: "ecs-mongo-production",
 * });
 * ```
 */
export declare function getCluster(args: GetClusterArgs, opts?: pulumi.InvokeOptions): Promise<GetClusterResult>;
/**
 * A collection of arguments for invoking getCluster.
 */
export interface GetClusterArgs {
    /**
     * Name of the ECS Cluster
     */
    clusterName: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * Key-value map of resource tags
     */
    tags?: {
        [key: string]: string;
    };
}
/**
 * A collection of values returned by getCluster.
 */
export interface GetClusterResult {
    /**
     * ARN of the ECS Cluster
     */
    readonly arn: string;
    readonly clusterName: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * Number of pending tasks for the ECS Cluster
     */
    readonly pendingTasksCount: number;
    readonly region: string;
    /**
     * The number of registered container instances for the ECS Cluster
     */
    readonly registeredContainerInstancesCount: number;
    /**
     * Number of running tasks for the ECS Cluster
     */
    readonly runningTasksCount: number;
    /**
     * The default Service Connect namespace
     */
    readonly serviceConnectDefaults: outputs.ecs.GetClusterServiceConnectDefault[];
    /**
     * Settings associated with the ECS Cluster
     */
    readonly settings: outputs.ecs.GetClusterSetting[];
    /**
     * Status of the ECS Cluster
     */
    readonly status: string;
    /**
     * Key-value map of resource tags
     */
    readonly tags: {
        [key: string]: string;
    };
}
/**
 * The ECS Cluster data source allows access to details of a specific
 * cluster within an AWS ECS service.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const ecs_mongo = aws.ecs.getCluster({
 *     clusterName: "ecs-mongo-production",
 * });
 * ```
 */
export declare function getClusterOutput(args: GetClusterOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetClusterResult>;
/**
 * A collection of arguments for invoking getCluster.
 */
export interface GetClusterOutputArgs {
    /**
     * Name of the ECS Cluster
     */
    clusterName: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
