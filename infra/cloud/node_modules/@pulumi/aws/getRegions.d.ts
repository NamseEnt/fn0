import * as pulumi from "@pulumi/pulumi";
import * as inputs from "./types/input";
import * as outputs from "./types/output";
/**
 * Provides information about AWS Regions. Can be used to filter regions i.e., by Opt-In status or only regions enabled for current account. To get details like endpoint and description of each region the data source can be combined with the `aws.getRegion` data source.
 *
 * ## Example Usage
 *
 * Enabled AWS Regions:
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const current = aws.getRegions({});
 * ```
 *
 * All the regions regardless of the availability
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const current = aws.getRegions({
 *     allRegions: true,
 * });
 * ```
 *
 * To see regions that are filtered by `"not-opted-in"`, the `allRegions` argument needs to be set to `true` or no results will be returned.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const current = aws.getRegions({
 *     allRegions: true,
 *     filters: [{
 *         name: "opt-in-status",
 *         values: ["not-opted-in"],
 *     }],
 * });
 * ```
 */
export declare function getRegions(args?: GetRegionsArgs, opts?: pulumi.InvokeOptions): Promise<GetRegionsResult>;
/**
 * A collection of arguments for invoking getRegions.
 */
export interface GetRegionsArgs {
    /**
     * If true the source will query all regions regardless of availability.
     */
    allRegions?: boolean;
    /**
     * Configuration block(s) to use as filters. Detailed below.
     */
    filters?: inputs.GetRegionsFilter[];
    /**
     * Identifier of the current partition (e.g., `aws` in AWS Commercial, `aws-cn` in AWS China).
     */
    id?: string;
}
/**
 * A collection of values returned by getRegions.
 */
export interface GetRegionsResult {
    readonly allRegions?: boolean;
    readonly filters?: outputs.GetRegionsFilter[];
    /**
     * Identifier of the current partition (e.g., `aws` in AWS Commercial, `aws-cn` in AWS China).
     */
    readonly id: string;
    /**
     * Names of regions that meets the criteria.
     */
    readonly names: string[];
}
/**
 * Provides information about AWS Regions. Can be used to filter regions i.e., by Opt-In status or only regions enabled for current account. To get details like endpoint and description of each region the data source can be combined with the `aws.getRegion` data source.
 *
 * ## Example Usage
 *
 * Enabled AWS Regions:
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const current = aws.getRegions({});
 * ```
 *
 * All the regions regardless of the availability
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const current = aws.getRegions({
 *     allRegions: true,
 * });
 * ```
 *
 * To see regions that are filtered by `"not-opted-in"`, the `allRegions` argument needs to be set to `true` or no results will be returned.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const current = aws.getRegions({
 *     allRegions: true,
 *     filters: [{
 *         name: "opt-in-status",
 *         values: ["not-opted-in"],
 *     }],
 * });
 * ```
 */
export declare function getRegionsOutput(args?: GetRegionsOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetRegionsResult>;
/**
 * A collection of arguments for invoking getRegions.
 */
export interface GetRegionsOutputArgs {
    /**
     * If true the source will query all regions regardless of availability.
     */
    allRegions?: pulumi.Input<boolean>;
    /**
     * Configuration block(s) to use as filters. Detailed below.
     */
    filters?: pulumi.Input<pulumi.Input<inputs.GetRegionsFilterArgs>[]>;
    /**
     * Identifier of the current partition (e.g., `aws` in AWS Commercial, `aws-cn` in AWS China).
     */
    id?: pulumi.Input<string>;
}
