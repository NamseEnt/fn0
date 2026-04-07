import * as pulumi from "@pulumi/pulumi";
/**
 * Use this data source to get a list of Load Balancer ARNs matching the specified criteria. Useful for passing to other
 * resources.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.lb.getLbs({
 *     tags: {
 *         "elbv2.k8s.aws/cluster": "my-cluster",
 *     },
 * });
 * ```
 */
export declare function getLbs(args?: GetLbsArgs, opts?: pulumi.InvokeOptions): Promise<GetLbsResult>;
/**
 * A collection of arguments for invoking getLbs.
 */
export interface GetLbsArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * Map of tags, each pair of which must exactly match
     * a pair on the desired Load Balancers.
     */
    tags?: {
        [key: string]: string;
    };
}
/**
 * A collection of values returned by getLbs.
 */
export interface GetLbsResult {
    /**
     * Set of Load Balancer ARNs.
     */
    readonly arns: string[];
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly region: string;
    readonly tags?: {
        [key: string]: string;
    };
}
/**
 * Use this data source to get a list of Load Balancer ARNs matching the specified criteria. Useful for passing to other
 * resources.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.lb.getLbs({
 *     tags: {
 *         "elbv2.k8s.aws/cluster": "my-cluster",
 *     },
 * });
 * ```
 */
export declare function getLbsOutput(args?: GetLbsOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetLbsResult>;
/**
 * A collection of arguments for invoking getLbs.
 */
export interface GetLbsOutputArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Map of tags, each pair of which must exactly match
     * a pair on the desired Load Balancers.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
