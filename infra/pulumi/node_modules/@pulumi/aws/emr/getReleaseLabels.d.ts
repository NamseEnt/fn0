import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Retrieve information about EMR Release Labels.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.emr.getReleaseLabels({
 *     filters: {
 *         application: "spark@2.1.0",
 *         prefix: "emr-5",
 *     },
 * });
 * ```
 */
export declare function getReleaseLabels(args?: GetReleaseLabelsArgs, opts?: pulumi.InvokeOptions): Promise<GetReleaseLabelsResult>;
/**
 * A collection of arguments for invoking getReleaseLabels.
 */
export interface GetReleaseLabelsArgs {
    /**
     * Filters the results of the request. Prefix specifies the prefix of release labels to return. Application specifies the application (with/without version) of release labels to return. See Filters.
     */
    filters?: inputs.emr.GetReleaseLabelsFilters;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getReleaseLabels.
 */
export interface GetReleaseLabelsResult {
    readonly filters?: outputs.emr.GetReleaseLabelsFilters;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly region: string;
    /**
     * Returned release labels.
     */
    readonly releaseLabels: string[];
}
/**
 * Retrieve information about EMR Release Labels.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.emr.getReleaseLabels({
 *     filters: {
 *         application: "spark@2.1.0",
 *         prefix: "emr-5",
 *     },
 * });
 * ```
 */
export declare function getReleaseLabelsOutput(args?: GetReleaseLabelsOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetReleaseLabelsResult>;
/**
 * A collection of arguments for invoking getReleaseLabels.
 */
export interface GetReleaseLabelsOutputArgs {
    /**
     * Filters the results of the request. Prefix specifies the prefix of release labels to return. Application specifies the application (with/without version) of release labels to return. See Filters.
     */
    filters?: pulumi.Input<inputs.emr.GetReleaseLabelsFiltersArgs>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
