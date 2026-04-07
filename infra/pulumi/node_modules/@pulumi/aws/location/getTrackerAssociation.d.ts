import * as pulumi from "@pulumi/pulumi";
/**
 * Retrieve information about a Location Service Tracker Association.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.location.getTrackerAssociation({
 *     consumerArn: "arn:aws:geo:region:account-id:geofence-collection/ExampleGeofenceCollectionConsumer",
 *     trackerName: "example",
 * });
 * ```
 */
export declare function getTrackerAssociation(args: GetTrackerAssociationArgs, opts?: pulumi.InvokeOptions): Promise<GetTrackerAssociationResult>;
/**
 * A collection of arguments for invoking getTrackerAssociation.
 */
export interface GetTrackerAssociationArgs {
    /**
     * ARN of the geofence collection associated to tracker resource.
     */
    consumerArn: string;
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
 * A collection of values returned by getTrackerAssociation.
 */
export interface GetTrackerAssociationResult {
    readonly consumerArn: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly region: string;
    readonly trackerName: string;
}
/**
 * Retrieve information about a Location Service Tracker Association.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.location.getTrackerAssociation({
 *     consumerArn: "arn:aws:geo:region:account-id:geofence-collection/ExampleGeofenceCollectionConsumer",
 *     trackerName: "example",
 * });
 * ```
 */
export declare function getTrackerAssociationOutput(args: GetTrackerAssociationOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetTrackerAssociationResult>;
/**
 * A collection of arguments for invoking getTrackerAssociation.
 */
export interface GetTrackerAssociationOutputArgs {
    /**
     * ARN of the geofence collection associated to tracker resource.
     */
    consumerArn: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Name of the tracker resource associated with a geofence collection.
     */
    trackerName: pulumi.Input<string>;
}
