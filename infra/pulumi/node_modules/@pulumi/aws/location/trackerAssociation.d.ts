import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing an AWS Location Tracker Association.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.location.GeofenceCollection("example", {collectionName: "example"});
 * const exampleTracker = new aws.location.Tracker("example", {trackerName: "example"});
 * const exampleTrackerAssociation = new aws.location.TrackerAssociation("example", {
 *     consumerArn: example.collectionArn,
 *     trackerName: exampleTracker.trackerName,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Location Tracker Association using the `tracker_name|consumer_arn`. For example:
 *
 * ```sh
 * $ pulumi import aws:location/trackerAssociation:TrackerAssociation example "tracker_name|consumer_arn"
 * ```
 */
export declare class TrackerAssociation extends pulumi.CustomResource {
    /**
     * Get an existing TrackerAssociation resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: TrackerAssociationState, opts?: pulumi.CustomResourceOptions): TrackerAssociation;
    /**
     * Returns true if the given object is an instance of TrackerAssociation.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is TrackerAssociation;
    /**
     * The Amazon Resource Name (ARN) for the geofence collection to be associated to tracker resource. Used when you need to specify a resource across all AWS.
     */
    readonly consumerArn: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The name of the tracker resource to be associated with a geofence collection.
     */
    readonly trackerName: pulumi.Output<string>;
    /**
     * Create a TrackerAssociation resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: TrackerAssociationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering TrackerAssociation resources.
 */
export interface TrackerAssociationState {
    /**
     * The Amazon Resource Name (ARN) for the geofence collection to be associated to tracker resource. Used when you need to specify a resource across all AWS.
     */
    consumerArn?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The name of the tracker resource to be associated with a geofence collection.
     */
    trackerName?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a TrackerAssociation resource.
 */
export interface TrackerAssociationArgs {
    /**
     * The Amazon Resource Name (ARN) for the geofence collection to be associated to tracker resource. Used when you need to specify a resource across all AWS.
     */
    consumerArn: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The name of the tracker resource to be associated with a geofence collection.
     */
    trackerName: pulumi.Input<string>;
}
