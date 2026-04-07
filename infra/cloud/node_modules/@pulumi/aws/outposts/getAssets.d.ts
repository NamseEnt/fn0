import * as pulumi from "@pulumi/pulumi";
/**
 * Information about hardware assets in an Outpost.
 *
 * ## Example Usage
 *
 * ### Basic
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.outposts.getAssets({
 *     arn: exampleAwsOutpostsOutpost.arn,
 * });
 * ```
 *
 * ### With Host ID Filter
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.outposts.getAssets({
 *     arn: exampleAwsOutpostsOutpost.arn,
 *     hostIdFilters: ["h-x38g5n0yd2a0ueb61"],
 * });
 * ```
 *
 * ### With Status ID Filter
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.outposts.getAssets({
 *     arn: exampleAwsOutpostsOutpost.arn,
 *     statusIdFilters: ["ACTIVE"],
 * });
 * ```
 */
export declare function getAssets(args: GetAssetsArgs, opts?: pulumi.InvokeOptions): Promise<GetAssetsResult>;
/**
 * A collection of arguments for invoking getAssets.
 */
export interface GetAssetsArgs {
    /**
     * Outpost ARN.
     */
    arn: string;
    /**
     * Filters by list of Host IDs of a Dedicated Host.
     */
    hostIdFilters?: string[];
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * Filters by list of state status. Valid values: "ACTIVE", "RETIRING".
     */
    statusIdFilters?: string[];
}
/**
 * A collection of values returned by getAssets.
 */
export interface GetAssetsResult {
    readonly arn: string;
    /**
     * List of all the asset ids found. This data source will fail if none are found.
     */
    readonly assetIds: string[];
    readonly hostIdFilters?: string[];
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly region: string;
    readonly statusIdFilters?: string[];
}
/**
 * Information about hardware assets in an Outpost.
 *
 * ## Example Usage
 *
 * ### Basic
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.outposts.getAssets({
 *     arn: exampleAwsOutpostsOutpost.arn,
 * });
 * ```
 *
 * ### With Host ID Filter
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.outposts.getAssets({
 *     arn: exampleAwsOutpostsOutpost.arn,
 *     hostIdFilters: ["h-x38g5n0yd2a0ueb61"],
 * });
 * ```
 *
 * ### With Status ID Filter
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.outposts.getAssets({
 *     arn: exampleAwsOutpostsOutpost.arn,
 *     statusIdFilters: ["ACTIVE"],
 * });
 * ```
 */
export declare function getAssetsOutput(args: GetAssetsOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetAssetsResult>;
/**
 * A collection of arguments for invoking getAssets.
 */
export interface GetAssetsOutputArgs {
    /**
     * Outpost ARN.
     */
    arn: pulumi.Input<string>;
    /**
     * Filters by list of Host IDs of a Dedicated Host.
     */
    hostIdFilters?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Filters by list of state status. Valid values: "ACTIVE", "RETIRING".
     */
    statusIdFilters?: pulumi.Input<pulumi.Input<string>[]>;
}
