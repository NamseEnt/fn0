import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing an AWS Location Geofence Collection.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.location.GeofenceCollection("example", {collectionName: "example"});
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Location Geofence Collection using the `collection_name`. For example:
 *
 * ```sh
 * $ pulumi import aws:location/geofenceCollection:GeofenceCollection example example
 * ```
 */
export declare class GeofenceCollection extends pulumi.CustomResource {
    /**
     * Get an existing GeofenceCollection resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: GeofenceCollectionState, opts?: pulumi.CustomResourceOptions): GeofenceCollection;
    /**
     * Returns true if the given object is an instance of GeofenceCollection.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is GeofenceCollection;
    /**
     * The Amazon Resource Name (ARN) for the geofence collection resource. Used when you need to specify a resource across all AWS.
     */
    readonly collectionArn: pulumi.Output<string>;
    /**
     * The name of the geofence collection.
     *
     * The following arguments are optional:
     */
    readonly collectionName: pulumi.Output<string>;
    /**
     * The timestamp for when the geofence collection resource was created in ISO 8601 format.
     */
    readonly createTime: pulumi.Output<string>;
    /**
     * The optional description for the geofence collection.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * A key identifier for an AWS KMS customer managed key assigned to the Amazon Location resource.
     */
    readonly kmsKeyId: pulumi.Output<string | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Key-value tags for the geofence collection. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * The timestamp for when the geofence collection resource was last updated in ISO 8601 format.
     */
    readonly updateTime: pulumi.Output<string>;
    /**
     * Create a GeofenceCollection resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: GeofenceCollectionArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering GeofenceCollection resources.
 */
export interface GeofenceCollectionState {
    /**
     * The Amazon Resource Name (ARN) for the geofence collection resource. Used when you need to specify a resource across all AWS.
     */
    collectionArn?: pulumi.Input<string>;
    /**
     * The name of the geofence collection.
     *
     * The following arguments are optional:
     */
    collectionName?: pulumi.Input<string>;
    /**
     * The timestamp for when the geofence collection resource was created in ISO 8601 format.
     */
    createTime?: pulumi.Input<string>;
    /**
     * The optional description for the geofence collection.
     */
    description?: pulumi.Input<string>;
    /**
     * A key identifier for an AWS KMS customer managed key assigned to the Amazon Location resource.
     */
    kmsKeyId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value tags for the geofence collection. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The timestamp for when the geofence collection resource was last updated in ISO 8601 format.
     */
    updateTime?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a GeofenceCollection resource.
 */
export interface GeofenceCollectionArgs {
    /**
     * The name of the geofence collection.
     *
     * The following arguments are optional:
     */
    collectionName: pulumi.Input<string>;
    /**
     * The optional description for the geofence collection.
     */
    description?: pulumi.Input<string>;
    /**
     * A key identifier for an AWS KMS customer managed key assigned to the Amazon Location resource.
     */
    kmsKeyId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value tags for the geofence collection. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
