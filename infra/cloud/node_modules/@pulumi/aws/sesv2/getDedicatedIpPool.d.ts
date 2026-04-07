import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Data source for managing an AWS SESv2 (Simple Email V2) Dedicated IP Pool.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.sesv2.getDedicatedIpPool({
 *     poolName: "my-pool",
 * });
 * ```
 */
export declare function getDedicatedIpPool(args: GetDedicatedIpPoolArgs, opts?: pulumi.InvokeOptions): Promise<GetDedicatedIpPoolResult>;
/**
 * A collection of arguments for invoking getDedicatedIpPool.
 */
export interface GetDedicatedIpPoolArgs {
    /**
     * Name of the dedicated IP pool.
     */
    poolName: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * A map of tags attached to the pool.
     */
    tags?: {
        [key: string]: string;
    };
}
/**
 * A collection of values returned by getDedicatedIpPool.
 */
export interface GetDedicatedIpPoolResult {
    /**
     * ARN of the Dedicated IP Pool.
     */
    readonly arn: string;
    /**
     * A list of objects describing the pool's dedicated IP's. See `dedicatedIps`.
     */
    readonly dedicatedIps: outputs.sesv2.GetDedicatedIpPoolDedicatedIp[];
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly poolName: string;
    readonly region: string;
    /**
     * (Optional) IP pool scaling mode. Valid values: `STANDARD`, `MANAGED`.
     */
    readonly scalingMode: string;
    /**
     * A map of tags attached to the pool.
     */
    readonly tags: {
        [key: string]: string;
    };
}
/**
 * Data source for managing an AWS SESv2 (Simple Email V2) Dedicated IP Pool.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.sesv2.getDedicatedIpPool({
 *     poolName: "my-pool",
 * });
 * ```
 */
export declare function getDedicatedIpPoolOutput(args: GetDedicatedIpPoolOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetDedicatedIpPoolResult>;
/**
 * A collection of arguments for invoking getDedicatedIpPool.
 */
export interface GetDedicatedIpPoolOutputArgs {
    /**
     * Name of the dedicated IP pool.
     */
    poolName: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags attached to the pool.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
