import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a list of Elastic IPs in a region.
 *
 * ## Example Usage
 *
 * The following shows outputting all Elastic IPs with the a specific tag value.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.ec2.getEips({
 *     tags: {
 *         Env: "dev",
 *     },
 * });
 * export const allocationIds = example.then(example => example.allocationIds);
 * export const publicIps = example.then(example => example.publicIps);
 * ```
 */
export declare function getEips(args?: GetEipsArgs, opts?: pulumi.InvokeOptions): Promise<GetEipsResult>;
/**
 * A collection of arguments for invoking getEips.
 */
export interface GetEipsArgs {
    /**
     * Custom filter block as described below.
     */
    filters?: inputs.ec2.GetEipsFilter[];
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * Map of tags, each pair of which must exactly match a pair on the desired Elastic IPs.
     */
    tags?: {
        [key: string]: string;
    };
}
/**
 * A collection of values returned by getEips.
 */
export interface GetEipsResult {
    /**
     * List of all the allocation IDs for address for use with EC2-VPC.
     */
    readonly allocationIds: string[];
    readonly filters?: outputs.ec2.GetEipsFilter[];
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * List of all the Elastic IP addresses.
     */
    readonly publicIps: string[];
    readonly region: string;
    readonly tags?: {
        [key: string]: string;
    };
}
/**
 * Provides a list of Elastic IPs in a region.
 *
 * ## Example Usage
 *
 * The following shows outputting all Elastic IPs with the a specific tag value.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.ec2.getEips({
 *     tags: {
 *         Env: "dev",
 *     },
 * });
 * export const allocationIds = example.then(example => example.allocationIds);
 * export const publicIps = example.then(example => example.publicIps);
 * ```
 */
export declare function getEipsOutput(args?: GetEipsOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetEipsResult>;
/**
 * A collection of arguments for invoking getEips.
 */
export interface GetEipsOutputArgs {
    /**
     * Custom filter block as described below.
     */
    filters?: pulumi.Input<pulumi.Input<inputs.ec2.GetEipsFilterArgs>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Map of tags, each pair of which must exactly match a pair on the desired Elastic IPs.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
