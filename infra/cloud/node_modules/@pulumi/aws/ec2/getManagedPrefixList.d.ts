import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * `aws.ec2.ManagedPrefixList` provides details about a specific AWS prefix list or
 * customer-managed prefix list in the current region.
 *
 * ## Example Usage
 *
 * ### Find the regional DynamoDB prefix list
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const current = aws.getRegion({});
 * const example = current.then(current => aws.ec2.getManagedPrefixList({
 *     name: `com.amazonaws.${current.region}.dynamodb`,
 * }));
 * ```
 *
 * ### Find a managed prefix list using filters
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.ec2.getManagedPrefixList({
 *     filters: [{
 *         name: "prefix-list-name",
 *         values: ["my-prefix-list"],
 *     }],
 * });
 * ```
 */
export declare function getManagedPrefixList(args?: GetManagedPrefixListArgs, opts?: pulumi.InvokeOptions): Promise<GetManagedPrefixListResult>;
/**
 * A collection of arguments for invoking getManagedPrefixList.
 */
export interface GetManagedPrefixListArgs {
    /**
     * Configuration block(s) for filtering. Detailed below.
     *
     * The arguments of this data source act as filters for querying the available
     * prefix lists. The given filters must match exactly one prefix list
     * whose data will be exported as attributes.
     */
    filters?: inputs.ec2.GetManagedPrefixListFilter[];
    /**
     * ID of the prefix list to select.
     */
    id?: string;
    /**
     * Name of the prefix list to select.
     */
    name?: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * Map of tags assigned to the resource.
     */
    tags?: {
        [key: string]: string;
    };
}
/**
 * A collection of values returned by getManagedPrefixList.
 */
export interface GetManagedPrefixListResult {
    /**
     * Address family of the prefix list. Valid values are `IPv4` and `IPv6`.
     */
    readonly addressFamily: string;
    /**
     * ARN of the selected prefix list.
     */
    readonly arn: string;
    /**
     * Set of entries in this prefix list. Each entry is an object with `cidr` and `description`.
     */
    readonly entries: outputs.ec2.GetManagedPrefixListEntry[];
    readonly filters?: outputs.ec2.GetManagedPrefixListFilter[];
    /**
     * ID of the selected prefix list.
     */
    readonly id: string;
    /**
     * When then prefix list is managed, the maximum number of entries it supports, or null otherwise.
     */
    readonly maxEntries: number;
    /**
     * Name of the selected prefix list.
     */
    readonly name: string;
    /**
     * Account ID of the owner of a customer-managed prefix list, or `AWS` otherwise.
     */
    readonly ownerId: string;
    readonly region: string;
    /**
     * Map of tags assigned to the resource.
     */
    readonly tags: {
        [key: string]: string;
    };
    readonly version: number;
}
/**
 * `aws.ec2.ManagedPrefixList` provides details about a specific AWS prefix list or
 * customer-managed prefix list in the current region.
 *
 * ## Example Usage
 *
 * ### Find the regional DynamoDB prefix list
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const current = aws.getRegion({});
 * const example = current.then(current => aws.ec2.getManagedPrefixList({
 *     name: `com.amazonaws.${current.region}.dynamodb`,
 * }));
 * ```
 *
 * ### Find a managed prefix list using filters
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.ec2.getManagedPrefixList({
 *     filters: [{
 *         name: "prefix-list-name",
 *         values: ["my-prefix-list"],
 *     }],
 * });
 * ```
 */
export declare function getManagedPrefixListOutput(args?: GetManagedPrefixListOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetManagedPrefixListResult>;
/**
 * A collection of arguments for invoking getManagedPrefixList.
 */
export interface GetManagedPrefixListOutputArgs {
    /**
     * Configuration block(s) for filtering. Detailed below.
     *
     * The arguments of this data source act as filters for querying the available
     * prefix lists. The given filters must match exactly one prefix list
     * whose data will be exported as attributes.
     */
    filters?: pulumi.Input<pulumi.Input<inputs.ec2.GetManagedPrefixListFilterArgs>[]>;
    /**
     * ID of the prefix list to select.
     */
    id?: pulumi.Input<string>;
    /**
     * Name of the prefix list to select.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Map of tags assigned to the resource.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
