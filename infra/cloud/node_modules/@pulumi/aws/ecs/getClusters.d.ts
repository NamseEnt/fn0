import * as pulumi from "@pulumi/pulumi";
/**
 * Data source for managing an AWS ECS (Elastic Container) Clusters.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.ecs.getClusters({});
 * ```
 */
export declare function getClusters(args?: GetClustersArgs, opts?: pulumi.InvokeOptions): Promise<GetClustersResult>;
/**
 * A collection of arguments for invoking getClusters.
 */
export interface GetClustersArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getClusters.
 */
export interface GetClustersResult {
    /**
     * List of ECS cluster ARNs associated with the account.
     */
    readonly clusterArns: string[];
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly region: string;
}
/**
 * Data source for managing an AWS ECS (Elastic Container) Clusters.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.ecs.getClusters({});
 * ```
 */
export declare function getClustersOutput(args?: GetClustersOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetClustersResult>;
/**
 * A collection of arguments for invoking getClusters.
 */
export interface GetClustersOutputArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
