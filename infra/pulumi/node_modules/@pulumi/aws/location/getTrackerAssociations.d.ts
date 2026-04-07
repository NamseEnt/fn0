import * as pulumi from "@pulumi/pulumi";
/**
 * Retrieve information about Location Service Tracker Associations.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.location.getTrackerAssociations({
 *     trackerName: "example",
 * });
 * ```
 */
export declare function getTrackerAssociations(args: GetTrackerAssociationsArgs, opts?: pulumi.InvokeOptions): Promise<GetTrackerAssociationsResult>;
/**
 * A collection of arguments for invoking getTrackerAssociations.
 */
export interface GetTrackerAssociationsArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * Name of the tracker resource associated with a geofence collection.
     */
    trackerName: string;
}
/**
 * A collection of values returned by getTrackerAssociations.
 */
export interface GetTrackerAssociationsResult {
    /**
     * List of geofence collection ARNs associated to the tracker resource.
     */
    readonly consumerArns: string[];
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly region: string;
    readonly trackerName: string;
}
/**
 * Retrieve information about Location Service Tracker Associations.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.location.getTrackerAssociations({
 *     trackerName: "example",
 * });
 * ```
 */
export declare function getTrackerAssociationsOutput(args: GetTrackerAssociationsOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetTrackerAssociationsResult>;
/**
 * A collection of arguments for invoking getTrackerAssociations.
 */
export interface GetTrackerAssociationsOutputArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Name of the tracker resource associated with a geofence collection.
     */
    trackerName: pulumi.Input<string>;
}
