import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides information for multiple EC2 Customer-Owned IP Pools, such as their identifiers.
 *
 * ## Example Usage
 *
 * The following shows outputting all COIP Pool Ids.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * export = async () => {
 *     const foo = await aws.ec2.getCoipPools({});
 *     return {
 *         foo: foo.ids,
 *     };
 * }
 * ```
 */
export declare function getCoipPools(args?: GetCoipPoolsArgs, opts?: pulumi.InvokeOptions): Promise<GetCoipPoolsResult>;
/**
 * A collection of arguments for invoking getCoipPools.
 */
export interface GetCoipPoolsArgs {
    /**
     * Custom filter block as described below.
     *
     * More complex filters can be expressed using one or more `filter` sub-blocks,
     * which take the following arguments:
     */
    filters?: inputs.ec2.GetCoipPoolsFilter[];
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * Mapping of tags, each pair of which must exactly match
     * a pair on the desired aws_ec2_coip_pools.
     */
    tags?: {
        [key: string]: string;
    };
}
/**
 * A collection of values returned by getCoipPools.
 */
export interface GetCoipPoolsResult {
    readonly filters?: outputs.ec2.GetCoipPoolsFilter[];
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * Set of COIP Pool Identifiers
     */
    readonly poolIds: string[];
    readonly region: string;
    readonly tags?: {
        [key: string]: string;
    };
}
/**
 * Provides information for multiple EC2 Customer-Owned IP Pools, such as their identifiers.
 *
 * ## Example Usage
 *
 * The following shows outputting all COIP Pool Ids.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * export = async () => {
 *     const foo = await aws.ec2.getCoipPools({});
 *     return {
 *         foo: foo.ids,
 *     };
 * }
 * ```
 */
export declare function getCoipPoolsOutput(args?: GetCoipPoolsOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetCoipPoolsResult>;
/**
 * A collection of arguments for invoking getCoipPools.
 */
export interface GetCoipPoolsOutputArgs {
    /**
     * Custom filter block as described below.
     *
     * More complex filters can be expressed using one or more `filter` sub-blocks,
     * which take the following arguments:
     */
    filters?: pulumi.Input<pulumi.Input<inputs.ec2.GetCoipPoolsFilterArgs>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Mapping of tags, each pair of which must exactly match
     * a pair on the desired aws_ec2_coip_pools.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
